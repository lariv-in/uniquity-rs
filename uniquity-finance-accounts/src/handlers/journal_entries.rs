use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde::Deserialize;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, QueryPage, html_built_page_or_app_layout},
};

use uniquity_common::require_superuser;

use crate::{
    entities::journal_entry::{self},
    forms::JournalEntryForm,
    keys::{JournalEntrySelectModalKey, JournalEntrySelectTableKey},
    routes::{JournalDetailRouteTag, JournalEntryCreateGetRouteTag},
    scope::{
        find_journal_entry_scoped, find_journal_scoped, load_journal_entry_items,
        load_source_doc_by_id, query_journal_entries_for_select,
    },
    source_doc_label::source_doc_type_label,
    state::AccountsState,
    templates::{
        JournalEntryDetailPage, JournalEntryFormPage, JournalEntryItemRow, JournalEntryRow,
        JournalEntrySelectPage,
    },
};

use super::util::{parse_i64, path_and_query};

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;

#[derive(Debug, Deserialize, Default)]
pub struct JournalEntrySelectQuery {
    #[serde(default)]
    pub page: QueryPage,
    #[serde(default)]
    pub target_input: Option<String>,
}

pub async fn create_get(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(journal_id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&JournalDetailRouteTag::new(journal_id).url()).into_response();
    }
    let Some(journal) = find_journal_scoped(&state.db, journal_id, &ctx).await else {
        return Redirect::to("/finance/journals").into_response();
    };
    let page = JournalEntryFormPage::new(
        journal.id,
        journal.name,
        ctx.format_datetime_local_input(Utc::now()),
    );
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    Path(journal_id): Path<i64>,
    Form(form): Form<JournalEntryForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&JournalDetailRouteTag::new(journal_id).url()).into_response();
    }
    let Some(_journal) = find_journal_scoped(&state.db, journal_id, &ctx).await else {
        return Redirect::to("/finance/journals").into_response();
    };
    let datetime = ctx
        .parse_datetime_local_input(&form.datetime)
        .unwrap_or_else(|| Utc::now());
    let source_doc_id = parse_i64(&form.source_doc_id).unwrap_or(0);
    let now = Utc::now();
    let model = journal_entry::ActiveModel {
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        datetime: Set(datetime),
        source_doc_id: Set(source_doc_id),
        journal_id: Set(journal_id),
        ..Default::default()
    };
    match model.insert(&state.db).await {
        Ok(_) => Redirect::to(&JournalDetailRouteTag::new(journal_id).url()).into_response(),
        Err(_) => Redirect::to(&JournalEntryCreateGetRouteTag::new(journal_id).url()).into_response(),
    }
}

pub async fn detail(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(entry) = find_journal_entry_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/finance/journals").into_response();
    };
    let journal = find_journal_scoped(&state.db, entry.journal_id, &ctx)
        .await
        .map(|j| j.name)
        .unwrap_or_else(|| "—".into());
    let source_doc_label = load_source_doc_by_id(&state.db, entry.source_doc_id)
        .await
        .map(|d| source_doc_type_label(&d.source_doc_type))
        .unwrap_or_else(|| "—".into());
    let items_raw = load_journal_entry_items(&state.db, entry.id).await;
    let items: Vec<JournalEntryItemRow> = items_raw
        .into_iter()
        .map(|(item, acct)| JournalEntryItemRow {
            datetime: ctx.format_datetime_seconds(item.datetime),
            account_label: format!("{} — {}", acct.code, acct.name),
            amount: item.amount.to_string(),
        })
        .collect();
    let page = JournalEntryDetailPage {
        id: entry.id,
        datetime: ctx.format_datetime_seconds(entry.datetime),
        journal_id: entry.journal_id,
        journal_label: format!("{journal} (#{})", entry.journal_id),
        source_doc_label,
        items,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn select(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<JournalEntrySelectQuery>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance/journals").into_response();
    }
    let page = q.page.get();
    let (rows, total) =
        query_journal_entries_for_select(&state.db, &ctx, page, PAGE_SIZE).await;
    let entries: Vec<JournalEntryRow> = rows
        .into_iter()
        .map(|(e, journal_name)| {
            let jn = journal_name.clone();
            JournalEntryRow {
                id: e.id,
                datetime: ctx.format_datetime_seconds(e.datetime),
                source_doc_label: format!("entry #{}", e.id),
                journal_name: jn.clone(),
                label: format!(
                    "{} · {}",
                    jn,
                    ctx.format_datetime_short(e.datetime)
                ),
            }
        })
        .collect();
    let list = ObjectList::from_page(entries, page, PAGE_SIZE, total);
    let page = JournalEntrySelectPage {
        entries: list,
        target_input: q.target_input.unwrap_or_else(|| "JournalEntryID".to_string()),
        path_and_query: path_and_query(&uri),
    };
    respond_picker_select::<JournalEntrySelectTableKey, JournalEntrySelectModalKey, _>(&htmx, &page)
        .into_response()
}
