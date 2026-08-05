use axum::{
    extract::Query,
    http::Uri,
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList},
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, QueryPage},
};

use crate::{
    entities::source_doc::{self, Entity as SourceDocEntity},
    keys::{SourceDocSelectModalKey, SourceDocSelectTableKey},
    scope::scope_superuser,
    source_doc_label::{source_doc_summary, source_doc_type_label},
    state::AccountsState,
    templates::{SourceDocRow, SourceDocSelectPage},
};

use super::util::path_and_query;

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;

#[derive(Debug, Deserialize, Default)]
pub struct SourceDocSelectQuery {
    #[serde(default)]
    pub page: QueryPage,
    #[serde(default)]
    pub target_input: Option<String>,
}

pub async fn select(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SourceDocSelectQuery>,
) -> maud::Markup {
    let page_num = q.page.get();
    let query = scope_superuser(SourceDocEntity::find(), &ctx)
        .filter(source_doc::Column::DeletedAt.is_null())
        .order_by_desc(source_doc::Column::Id);
    let paginator = query.paginate(&state.db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows: Vec<SourceDocRow> = models
        .into_iter()
        .map(|d| {
            let typ = d.source_doc_type.clone();
            SourceDocRow {
                id: d.id,
                source_doc_type: source_doc_type_label(&typ),
                source_doc_id: d.source_doc_id,
                label: source_doc_summary(&typ, d.source_doc_id, d.id),
            }
        })
        .collect();
    let docs = ObjectList::from_page(rows, page_num, PAGE_SIZE, total);
    let page = SourceDocSelectPage {
        docs,
        target_input: q.target_input.unwrap_or_else(|| "SourceDocID".into()),
        path_and_query: path_and_query(&uri),
    };
    respond_picker_select::<SourceDocSelectTableKey, SourceDocSelectModalKey, _>(&htmx, &page)
}
