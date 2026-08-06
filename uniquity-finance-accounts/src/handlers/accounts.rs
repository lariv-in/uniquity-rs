use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use serde::Deserialize;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    html_form::{FormFieldKey, HtmlFormBody},
    picker::respond_picker_select,
    plugins::users::{middleware::RequireAuth, state::AuthContext},
    template::RenderAppPane,
    web::{
        Htmx, QueryI64, QueryPage, QueryStr, query_bool, html_built_page_or_app_layout,
        html_built_page_with_slots, respond_create_modal_done,
    },
};

use uniquity_common::require_superuser;

use crate::{
    account_validation::{
        account_descendant_ids, sync_account_children, validate_balance_type_change,
        validate_parent_balance_type_on_save, validate_parent_not_cycle, ACCOUNT_PARENT_UP_ROW_ID,
    },
    balance_type::BalanceType,
    entities::account::{self, Entity as AccountEntity},
    forms::{AccountForm, AccountFormField},
    handlers::ModalNameQuery,
    keys::AccountCreateModalKey,
    keys::AccountJournalEntriesTableKey,
    keys::AccountSelectModalKey,
    keys::AccountSelectTableKey,
    keys::AccountTableKey,
    routes::{
        AccountDetailRouteTag, AccountEditPostRouteTag, FinanceDefaultRouteTag,
    },
    scope::{
        apply_account_filters, find_account_scoped, load_account_parent_label,
        query_journal_entries_for_account_subtree, sum_account_subtree_balance,
    },
    source_doc_label::source_doc_type_label,
    state::AccountsState,
    templates::{
        AccountCreateModalPage, AccountDetailPage, AccountFormPage, AccountListPage,
        AccountRow, AccountSelectPage, JournalEntryRow,
    },
};

use super::util::{checkbox_on, parse_i32, parse_i64, path_and_query, query_param};

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;

#[derive(Debug, Deserialize, Default)]
pub struct AccountDetailQuery {
    #[serde(default)]
    pub page: QueryPage,
}

#[derive(Debug, Deserialize, Default)]
pub struct AccountCreateQuery {
    #[serde(default, rename = "ParentID", alias = "parent_id")]
    pub parent_id: QueryI64,
}

#[derive(Debug, Deserialize, Default)]
pub struct AccountListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: QueryStr,
    #[serde(default, rename = "Code", alias = "code")]
    pub code: QueryStr,
    #[serde(default, rename = "IsGroup", alias = "is_group", deserialize_with = "query_bool")]
    pub is_group: Option<bool>,
    #[serde(default, rename = "BalanceType", alias = "balance_type")]
    pub balance_type: QueryStr,
    #[serde(default)]
    pub page: QueryPage,
}

#[derive(Debug, Deserialize, Default)]
pub struct AccountSelectQuery {
    #[serde(flatten)]
    pub filter: AccountListQuery,
    #[serde(default, rename = "ParentID", alias = "parent_id")]
    pub parent_id: QueryI64,
    #[serde(default, rename = "balance_type_scope")]
    pub balance_type_scope: Option<String>,
    #[serde(default)]
    pub target_input: Option<String>,
    #[serde(default, rename = "exclude_account_id")]
    pub exclude_account_id: QueryI64,
}

fn model_to_row(a: account::Model, parent_label: String) -> AccountRow {
    AccountRow {
        id: a.id,
        name: a.name,
        code: a.code,
        is_group: a.is_group,
        balance_type: a.balance_type.to_string(),
        parent_label,
    }
}

async fn query_accounts(
    db: &sea_orm::DatabaseConnection,
    q: &AccountListQuery,
    auth: &AuthContext,
    parent_id: Option<i64>,
    balance_type_scope: Option<&str>,
    root_only: bool,
    page_size: u32,
) -> (Vec<AccountRow>, u32, u64) {
    let mut query = AccountEntity::find();
    query = apply_account_filters(
        query,
        q.name.as_deref(),
        q.code.as_deref(),
        q.is_group,
        q.balance_type.as_deref(),
        parent_id,
        balance_type_scope,
        root_only,
    );
    query = crate::scope::scope_superuser(query, auth);
    let page = q.page.get();
    let paginator = query.paginate(db, page_size as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for m in models {
        let parent_label = load_account_parent_label(db, m.parent_id).await;
        rows.push(model_to_row(m, parent_label));
    }
    (rows, page, total)
}

async fn load_account_rows(
    db: &sea_orm::DatabaseConnection,
    q: &AccountListQuery,
    auth: &AuthContext,
    parent_id: Option<i64>,
    balance_type_scope: Option<&str>,
    root_only: bool,
    page_size: u32,
) -> ObjectList<AccountRow> {
    let (rows, page, total) = query_accounts(
        db,
        q,
        auth,
        parent_id,
        balance_type_scope,
        root_only,
        page_size,
    )
    .await;
    ObjectList::from_page(rows, page, page_size, total)
}

async fn filter_excluded_account_rows(
    db: &sea_orm::DatabaseConnection,
    exclude_account_id: Option<i64>,
    mut accounts: ObjectList<AccountRow>,
) -> ObjectList<AccountRow> {
    let Some(exclude_id) = exclude_account_id.filter(|&id| id > 0) else {
        return accounts;
    };
    let forbidden: std::collections::HashSet<i64> = account_descendant_ids(db, exclude_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect();
    if forbidden.is_empty() {
        return accounts;
    }
    let removed = accounts
        .items
        .iter()
        .filter(|r| r.id != ACCOUNT_PARENT_UP_ROW_ID && forbidden.contains(&r.id))
        .count();
    accounts.items.retain(|r| {
        r.id == ACCOUNT_PARENT_UP_ROW_ID || !forbidden.contains(&r.id)
    });
    accounts.total = accounts.total.saturating_sub(removed as u64);
    accounts
}

fn prepend_parent_up_row(mut accounts: ObjectList<AccountRow>) -> ObjectList<AccountRow> {
    if accounts.number != 1 {
        return accounts;
    }
    let mut items = vec![AccountRow {
        id: ACCOUNT_PARENT_UP_ROW_ID,
        name: "..".into(),
        code: 0,
        is_group: true,
        balance_type: String::new(),
        parent_label: String::new(),
    }];
    items.append(&mut accounts.items);
    accounts.items = items;
    accounts.total += 1;
    accounts
}

async fn load_child_items_for_account(
    db: &sea_orm::DatabaseConnection,
    parent_id: i64,
) -> Vec<ManyToManyItem> {
    let children = AccountEntity::find()
        .filter(account::Column::ParentId.eq(parent_id))
        .filter(account::Column::DeletedAt.is_null())
        .all(db)
        .await
        .unwrap_or_default();
    children
        .into_iter()
        .map(|a| ManyToManyItem {
            key: a.id.to_string(),
            value: format!("{} — {}", a.code, a.name),
        })
        .collect()
}

pub async fn list(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<AccountListQuery>,
) -> maud::Markup {
    let accounts =
        load_account_rows(&state.db, &q, &ctx, None, None, true, PAGE_SIZE).await;
    let page = AccountListPage {
        accounts,
        filter_name: q.name.or_empty(),
        filter_code: q.code.or_empty(),
        filter_is_group: q.is_group.unwrap_or(false),
        filter_balance_type: q.balance_type.or_empty(),
        path_and_query: path_and_query(&uri),
        can_edit: require_superuser(&ctx),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<AccountTableKey>() {
        return page.render_table();
    }
    if htmx.wants_main_content() {
        return page.render_main().into();
    }
    if htmx.wants_app_layout() {
        return page.render_pane().into();
    }
    html_built_page_with_slots(&page, &chrome, &slot_ctx)
}

pub async fn detail(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Path(id): Path<i64>,
    Query(q): Query<AccountDetailQuery>,
) -> Response {
    let Some(a) = find_account_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    };
    let parent_label = load_account_parent_label(&state.db, a.parent_id).await;
    let balance_total = sum_account_subtree_balance(&state.db, a.id).await;
    let mut children = ObjectList::from_page(vec![], 1, PAGE_SIZE, 0);
    if a.is_group {
        let child_q = AccountListQuery::default();
        children = load_account_rows(&state.db, &child_q, &ctx, Some(a.id), None, false, 100).await;
        children = prepend_parent_up_row(children);
    }
    let page_num = q.page.get();
    let (entry_models, entry_total) =
        query_journal_entries_for_account_subtree(&state.db, &ctx, a.id, page_num, PAGE_SIZE).await;
    let mut entry_rows = Vec::with_capacity(entry_models.len());
    for (e, journal_name) in entry_models {
        let source_doc_label = crate::scope::load_source_doc_by_id(&state.db, e.source_doc_id)
            .await
            .map(|d| source_doc_type_label(&d.source_doc_type))
            .unwrap_or_else(|| "—".into());
        entry_rows.push(JournalEntryRow {
            id: e.id,
            datetime: ctx.format_datetime_seconds(e.datetime),
            source_doc_label,
            journal_name,
            label: String::new(),
        });
    }
    let entries = ObjectList::from_page(entry_rows, page_num, PAGE_SIZE, entry_total);
    let page = AccountDetailPage {
        id: a.id,
        name: a.name,
        code: a.code,
        is_group: a.is_group,
        balance_type: a.balance_type.to_string(),
        parent_label,
        parent_id: a.parent_id.unwrap_or(0),
        balance_total,
        children,
        entries,
        path_and_query: path_and_query(&uri),
        can_edit: require_superuser(&ctx),
    };
    if htmx.targets::<AccountJournalEntriesTableKey>() {
        return page.render_entries_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
    Query(create_q): Query<AccountCreateQuery>,
) -> maud::Markup {
    if !require_superuser(&ctx) {
        return maud::html! { div class="alert alert-error" { "Forbidden" } };
    }
    let mut page = AccountCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        name: String::new(),
        code: String::new(),
        is_group: false,
        balance_type: String::new(),
        parent_id: String::new(),
        parent_display: String::new(),
        error: String::new(),
    };
    if let Some(pid) = create_q.parent_id.positive() {
        if let Some(parent) = find_account_scoped(&state.db, pid, &ctx).await {
            page.parent_id = pid.to_string();
            page.parent_display = format!("{} — {}", parent.code, parent.name);
            page.balance_type = parent.balance_type.to_string();
        }
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

fn account_create_modal_page_from_form(
    form: &AccountForm,
    form_name: String,
    refresh_table: String,
    parent_display: String,
    error: String,
) -> AccountCreateModalPage {
    AccountCreateModalPage {
        form_name,
        refresh_table,
        name: form.name.clone(),
        code: form.code.clone(),
        is_group: checkbox_on(&form.is_group),
        balance_type: form.balance_type.clone(),
        parent_id: form.parent_id.clone(),
        parent_display,
        error,
    }
}

async fn save_account_from_form(
    db: &sea_orm::DatabaseConnection,
    form: &AccountForm,
    existing: Option<account::Model>,
) -> Result<account::Model, String> {
    let balance_type = BalanceType::parse(&form.balance_type)
        .ok_or_else(|| "invalid balance type".to_string())?;
    let parent_id = parse_i64(&form.parent_id);
    validate_parent_balance_type_on_save(db, parent_id, balance_type).await?;
    validate_parent_not_cycle(db, existing.as_ref().map(|a| a.id), parent_id).await?;
    if let Some(ref old) = existing {
        validate_balance_type_change(db, old.id, old.balance_type, balance_type).await?;
    }
    let code = parse_i32(&form.code).ok_or_else(|| "code is required".to_string())?;
    let now = Utc::now();
    if let Some(old) = existing {
        let model = account::ActiveModel {
            id: Set(old.id),
            updated_at: Set(Some(now)),
            name: Set(form.name.clone()),
            code: Set(code),
            is_group: Set(checkbox_on(&form.is_group)),
            balance_type: Set(balance_type),
            parent_id: Set(parent_id),
            ..Default::default()
        };
        model.update(db).await.map_err(|e| e.to_string())
    } else {
        let model = account::ActiveModel {
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            name: Set(form.name.clone()),
            code: Set(code),
            is_group: Set(checkbox_on(&form.is_group)),
            balance_type: Set(balance_type),
            parent_id: Set(parent_id),
            ..Default::default()
        };
        model.insert(db).await.map_err(|e| e.to_string())
    }
}

pub async fn create_post(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<AccountForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    }
    match save_account_from_form(&state.db, &form, None).await {
        Ok(saved) => respond_create_modal_done::<AccountCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &AccountDetailRouteTag::new(saved.id).url(),
        ),
        Err(e) => {
            let parent_display = if !form.parent_id.is_empty() {
                load_account_parent_label(
                    &state.db,
                    parse_i64(&form.parent_id),
                )
                .await
            } else {
                String::new()
            };
            let page = account_create_modal_page_from_form(
                &form,
                q.form_name(),
                q.refresh_table(),
                parent_display,
                e,
            );
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    }
    let Some(a) = find_account_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    };
    let parent_display = load_account_parent_label(&state.db, a.parent_id).await;
    let child_items = load_child_items_for_account(&state.db, a.id).await;
    let page = AccountFormPage::from_model(&a, parent_display, child_items);
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<AccountForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    }
    let Some(existing) = find_account_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    };
    match save_account_from_form(&state.db, &form, Some(existing.clone())).await {
        Ok(saved) => {
            if checkbox_on(&form.is_group) {
                if let Err(_e) = sync_account_children(
                    &state.db,
                    saved.id,
                    saved.balance_type,
                    &form.child_ids,
                )
                .await
                {
                    return Redirect::to(&AccountEditPostRouteTag::new(id).url()).into_response();
                }
            }
            Redirect::to(&AccountDetailRouteTag::new(id).url()).into_response()
        }
        Err(_) => Redirect::to(&AccountEditPostRouteTag::new(id).url()).into_response(),
    }
}

pub async fn delete_post(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    }
    let Some(existing) = find_account_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&FinanceDefaultRouteTag.url()).into_response();
    };
    let now = Utc::now();
    let model = account::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let _ = model.update(&state.db).await;
    Redirect::to(&FinanceDefaultRouteTag.url()).into_response()
}

pub async fn select(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<AccountSelectQuery>,
) -> maud::Markup {
    let parent_id = q.parent_id.positive();
    let mut accounts = load_account_rows(
        &state.db,
        &q.filter,
        &ctx,
        parent_id,
        q.balance_type_scope.as_deref(),
        false,
        PAGE_SIZE,
    )
    .await;
    accounts = filter_excluded_account_rows(&state.db, q.exclude_account_id.get(), accounts).await;
    let grandparent_id = if let Some(pid) = parent_id {
        AccountEntity::find_by_id(pid)
            .filter(account::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .ok()
            .flatten()
            .and_then(|a| a.parent_id)
    } else {
        None
    };
    if parent_id.is_some() && accounts.number == 1 {
        let mut items = vec![AccountRow {
            id: ACCOUNT_PARENT_UP_ROW_ID,
            name: "..".into(),
            code: 0,
            is_group: true,
            balance_type: String::new(),
            parent_label: String::new(),
        }];
        items.extend(accounts.items.drain(..));
        accounts.items = items;
        accounts.total += 1;
    }
    let target_input = q
        .target_input
        .clone()
        .or_else(|| query_param(&path_and_query(&uri), "target_input"))
        .unwrap_or_else(|| AccountFormField::ParentId.target_input().into());
    let page = AccountSelectPage {
        accounts,
        filter_name: q.filter.name.or_empty(),
        filter_code: q.filter.code.or_empty(),
        filter_balance_type: q.filter.balance_type.or_empty(),
        balance_type_scope: q.balance_type_scope.clone().unwrap_or_default(),
        parent_id: parent_id.unwrap_or(0),
        grandparent_id,
        path_and_query: path_and_query(&uri),
        target_input,
        exclude_account_id: q.exclude_account_id.or_zero(),
    };
    if htmx.wants_main_content() {
        return page.render_main().into();
    }
    if htmx.wants_app_layout() {
        return page.render_pane().into();
    }
    respond_picker_select::<AccountSelectTableKey, AccountSelectModalKey, _>(&htmx, &page)
}
