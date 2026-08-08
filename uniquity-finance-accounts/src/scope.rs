use std::collections::HashMap;

use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Select,
    QuerySelect, sea_query::Expr,
};

use lariv_rs::plugins::users::state::AuthContext;

use uniquity_common::{decimal::decimal_display, is_superuser};

use crate::{
    account_validation::{account_descendant_ids, BALANCE_TYPE_SCOPE_QUERY_PARAM},
    balance_type::BalanceType,
    entities::{
        account::{self, Entity as AccountEntity},
        currency::{self, Entity as CurrencyEntity},
        journal::{self, Entity as JournalEntity},
        journal_entry::{self, Entity as JournalEntryEntity},
        journal_entry_item::{self, Entity as JournalEntryItemEntity},
        source_doc::{self, Entity as SourceDocEntity},
    },
};

pub fn scope_superuser<E>(query: Select<E>, auth: &AuthContext) -> Select<E>
where
    E: EntityTrait,
{
    if is_superuser(auth) {
        return query;
    }
    query.filter(Expr::cust("1 = 0"))
}

pub fn apply_account_filters(
    mut query: Select<AccountEntity>,
    name: Option<&str>,
    code: Option<&str>,
    is_group: Option<bool>,
    balance_type: Option<&str>,
    parent_id: Option<i64>,
    balance_type_scope: Option<&str>,
    root_only: bool,
) -> Select<AccountEntity> {
    if root_only {
        query = query.filter(account::Column::ParentId.is_null());
    }
    if let Some(pid) = parent_id.filter(|&id| id > 0) {
        query = query.filter(account::Column::ParentId.eq(pid));
    }
    if let Some(bt) = balance_type_scope
        .filter(|s| !s.is_empty())
        .and_then(BalanceType::parse)
    {
        query = query.filter(account::Column::BalanceType.eq(bt));
    }
    if let Some(n) = name.filter(|s| !s.is_empty()) {
        query = query.filter(account::Column::Name.contains(n));
    }
    if let Some(c) = code.filter(|s| !s.is_empty()) {
        if let Ok(n) = c.parse::<i32>() {
            query = query.filter(account::Column::Code.eq(n));
        }
    }
    if let Some(g) = is_group {
        query = query.filter(account::Column::IsGroup.eq(g));
    }
    if let Some(bt) = balance_type
        .filter(|s| !s.is_empty())
        .and_then(BalanceType::parse)
    {
        query = query.filter(account::Column::BalanceType.eq(bt));
    }
    query.order_by_asc(account::Column::Code)
}

pub fn apply_currency_filters(
    mut query: Select<CurrencyEntity>,
    code: Option<&str>,
    name: Option<&str>,
    symbol: Option<&str>,
    minor_unit: Option<&str>,
) -> Select<CurrencyEntity> {
    if let Some(c) = code.filter(|s| !s.is_empty()) {
        if let Ok(n) = c.parse::<i32>() {
            query = query.filter(currency::Column::Code.eq(n));
        }
    }
    if let Some(n) = name.filter(|s| !s.is_empty()) {
        query = query.filter(currency::Column::Name.contains(n));
    }
    if let Some(s) = symbol.filter(|s| !s.is_empty()) {
        query = query.filter(currency::Column::Symbol.contains(s));
    }
    if let Some(m) = minor_unit.filter(|s| !s.is_empty()) {
        if let Ok(n) = m.parse::<i32>() {
            query = query.filter(currency::Column::MinorUnit.eq(n));
        }
    }
    query.order_by_asc(currency::Column::Code)
}

pub fn apply_journal_filters(
    mut query: Select<JournalEntity>,
    name: Option<&str>,
    is_active: Option<bool>,
    currency_id: Option<&str>,
    journal_type: Option<&str>,
) -> Select<JournalEntity> {
    if let Some(n) = name.filter(|s| !s.is_empty()) {
        query = query.filter(journal::Column::Name.contains(n));
    }
    if let Some(a) = is_active {
        query = query.filter(journal::Column::IsActive.eq(a));
    }
    if let Some(cid) = currency_id.filter(|s| !s.is_empty()) {
        if let Ok(n) = cid.parse::<i64>() {
            query = query.filter(journal::Column::CurrencyId.eq(n));
        }
    }
    if let Some(t) = journal_type.filter(|s| !s.is_empty()) {
        if let Some(jt) = crate::journal_type::JournalType::parse(t) {
            query = query.filter(journal::Column::JournalType.eq(jt));
        }
    }
    query.order_by_desc(journal::Column::CreatedAt)
}

pub async fn find_account_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<account::Model> {
    scope_superuser(
        AccountEntity::find_by_id(id),
        auth,
    )
    .one(db)
    .await
    .ok()
    .flatten()
}

pub async fn find_currency_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<currency::Model> {
    scope_superuser(
        CurrencyEntity::find_by_id(id),
        auth,
    )
    .one(db)
    .await
    .ok()
    .flatten()
}

pub async fn find_journal_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<journal::Model> {
    scope_superuser(
        JournalEntity::find_by_id(id),
        auth,
    )
    .one(db)
    .await
    .ok()
    .flatten()
}

pub async fn find_journal_entry_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<journal_entry::Model> {
    scope_superuser(
        JournalEntryEntity::find_by_id(id),
        auth,
    )
    .one(db)
    .await
    .ok()
    .flatten()
}

pub async fn load_journal_display_label(db: &DatabaseConnection, journal_id: Option<i64>) -> String {
    use crate::entities::journal::Entity as JournalEntity;
    let Some(jid) = journal_id.filter(|&id| id > 0) else {
        return "—".into();
    };
    JournalEntity::find_by_id(jid)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|j| j.name)
        .unwrap_or_else(|| "—".into())
}

pub async fn load_account_parent_label(db: &DatabaseConnection, parent_id: Option<i64>) -> String {
    let Some(pid) = parent_id.filter(|&id| id > 0) else {
        return "—".into();
    };
    AccountEntity::find_by_id(pid)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|a| format!("{} — {}", a.code, a.name))
        .unwrap_or_else(|| "—".into())
}

/// Walk `parent_id` toward the root; returns `(id, name)` from root → immediate parent.
///
/// Does not include the current account. Cycle-guarded.
pub async fn load_account_ancestors(
    db: &DatabaseConnection,
    mut parent_id: Option<i64>,
) -> Vec<(i64, String)> {
    let mut chain = Vec::new();
    for _ in 0..64 {
        let Some(pid) = parent_id.filter(|&id| id > 0) else {
            break;
        };
        let Some(a) = AccountEntity::find_by_id(pid)
            .one(db)
            .await
            .ok()
            .flatten()
        else {
            break;
        };
        parent_id = a.parent_id;
        chain.push((a.id, a.name));
    }
    chain.reverse();
    chain
}

pub async fn load_currency_by_id(db: &DatabaseConnection, id: i64) -> Option<currency::Model> {
    CurrencyEntity::find_by_id(id)
        .one(db)
        .await
        .ok()
        .flatten()
}

pub fn currency_summary(c: &currency::Model) -> String {
    format!("{} — {} ({})", c.symbol, c.name, c.code)
}

/// Currency symbol for a journal entry via its journal (`""` if unresolved).
pub async fn load_journal_entry_currency_symbol(
    db: &DatabaseConnection,
    journal_entry_id: i64,
) -> String {
    if journal_entry_id <= 0 {
        return String::new();
    }
    let Some(entry) = JournalEntryEntity::find_by_id(journal_entry_id)
        .one(db)
        .await
        .ok()
        .flatten()
    else {
        return String::new();
    };
    let Some(journal) = JournalEntity::find_by_id(entry.journal_id)
        .one(db)
        .await
        .ok()
        .flatten()
    else {
        return String::new();
    };
    load_currency_by_id(db, journal.currency_id)
        .await
        .map(|c| c.symbol)
        .unwrap_or_default()
}

pub async fn load_journal_entries_for_journal(
    db: &DatabaseConnection,
    journal_id: i64,
) -> Vec<journal_entry::Model> {
    JournalEntryEntity::find()
        .filter(journal_entry::Column::JournalId.eq(journal_id))
        .order_by_desc(journal_entry::Column::Datetime)
        .order_by_desc(journal_entry::Column::Id)
        .all(db)
        .await
        .unwrap_or_default()
}

pub async fn load_source_doc_by_id(db: &DatabaseConnection, id: i64) -> Option<source_doc::Model> {
    SourceDocEntity::find_by_id(id)
        .one(db)
        .await
        .ok()
        .flatten()
}

pub async fn load_journal_entry_items(
    db: &DatabaseConnection,
    entry_id: i64,
) -> Vec<(journal_entry_item::Model, account::Model)> {
    let items = JournalEntryItemEntity::find()
        .filter(journal_entry_item::Column::JournalEntryId.eq(entry_id))
        .order_by_asc(journal_entry_item::Column::Id)
        .all(db)
        .await
        .unwrap_or_default();
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if let Some(acct) = AccountEntity::find_by_id(item.account_id)
            .one(db)
            .await
            .ok()
            .flatten()
        {
            out.push((item, acct));
        }
    }
    out
}

/// Transfer amount per journal entry: sum of debit lines (positive amounts).
/// For a balanced entry this equals the sum of absolute credit amounts.
pub async fn load_journal_entry_transfer_amounts(
    db: &DatabaseConnection,
    entry_ids: &[i64],
) -> HashMap<i64, String> {
    if entry_ids.is_empty() {
        return HashMap::new();
    }
    let items = JournalEntryItemEntity::find()
        .filter(journal_entry_item::Column::JournalEntryId.is_in(entry_ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    let mut sums: HashMap<i64, Decimal> = HashMap::new();
    for item in items {
        if item.amount > Decimal::ZERO {
            *sums.entry(item.journal_entry_id).or_insert(Decimal::ZERO) += item.amount;
        }
    }
    sums
        .into_iter()
        .map(|(id, amount)| (id, decimal_display(amount)))
        .collect()
}

pub async fn sum_account_subtree_balance(db: &DatabaseConnection, account_id: i64) -> String {
    let ids = account_descendant_ids(db, account_id).await.unwrap_or_default();
    if ids.is_empty() {
        return "—".into();
    }
    let sum: Option<rust_decimal::Decimal> = JournalEntryItemEntity::find()
        .filter(journal_entry_item::Column::AccountId.is_in(ids))
        .select_only()
        .column_as(
            sea_orm::sea_query::Expr::cust("COALESCE(SUM(amount), 0)"),
            "total",
        )
        .into_tuple()
        .one(db)
        .await
        .ok()
        .flatten();
    sum.map(|d| d.to_string()).unwrap_or_else(|| "—".into())
}

pub fn balance_type_scope_param() -> &'static str {
    BALANCE_TYPE_SCOPE_QUERY_PARAM
}

pub fn scope_journal_entries(
    query: Select<JournalEntryEntity>,
    auth: &AuthContext,
) -> Select<JournalEntryEntity> {
    scope_superuser(query, auth)
}

/// Journal entry items posting to an account or its descendants, with parent `source_doc_id`.
pub async fn query_journal_entry_items_for_account_subtree(
    db: &DatabaseConnection,
    auth: &AuthContext,
    account_id: i64,
    page: u32,
    page_size: u32,
) -> (Vec<(journal_entry_item::Model, i64)>, u64) {
    let account_ids = match account_descendant_ids(db, account_id).await {
        Ok(ids) if !ids.is_empty() => ids,
        _ => return (vec![], 0),
    };

    let query = scope_superuser(JournalEntryItemEntity::find(), auth)
        .filter(journal_entry_item::Column::AccountId.is_in(account_ids))
        .order_by_desc(journal_entry_item::Column::Datetime)
        .order_by_desc(journal_entry_item::Column::Id);
    let paginator = query.paginate(db, page_size as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();

    let mut entry_ids: Vec<i64> = models.iter().map(|i| i.journal_entry_id).collect();
    entry_ids.sort_unstable();
    entry_ids.dedup();
    let mut source_doc_by_entry: HashMap<i64, i64> = HashMap::new();
    if !entry_ids.is_empty() {
        let entries = JournalEntryEntity::find()
            .filter(journal_entry::Column::Id.is_in(entry_ids))
            .all(db)
            .await
            .unwrap_or_default();
        for e in entries {
            source_doc_by_entry.insert(e.id, e.source_doc_id);
        }
    }

    let rows = models
        .into_iter()
        .map(|item| {
            let source_doc_id = source_doc_by_entry
                .get(&item.journal_entry_id)
                .copied()
                .unwrap_or(0);
            (item, source_doc_id)
        })
        .collect();
    (rows, total)
}

pub async fn query_journal_entries_for_account_subtree(
    db: &DatabaseConnection,
    auth: &AuthContext,
    account_id: i64,
    page: u32,
    page_size: u32,
) -> (Vec<(journal_entry::Model, String)>, u64) {
    let account_ids = match account_descendant_ids(db, account_id).await {
        Ok(ids) if !ids.is_empty() => ids,
        _ => return (vec![], 0),
    };

    let entry_ids: std::collections::HashSet<i64> = JournalEntryItemEntity::find()
        .filter(journal_entry_item::Column::AccountId.is_in(account_ids))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|i| i.journal_entry_id)
        .collect();

    if entry_ids.is_empty() {
        return (vec![], 0);
    }

    let query = scope_journal_entries(JournalEntryEntity::find(), auth)
        .filter(journal_entry::Column::Id.is_in(entry_ids.into_iter().collect::<Vec<_>>()))
        .order_by_desc(journal_entry::Column::Datetime)
        .order_by_desc(journal_entry::Column::Id);
    let paginator = query.paginate(db, page_size as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for m in models {
        let name = JournalEntity::find_by_id(m.journal_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|j| j.name)
            .unwrap_or_else(|| "—".to_string());
        rows.push((m, name));
    }
    (rows, total)
}

pub async fn query_journal_entries_for_select(
    db: &DatabaseConnection,
    auth: &AuthContext,
    page: u32,
    page_size: u32,
) -> (Vec<(journal_entry::Model, String)>, u64) {
    let query = scope_journal_entries(JournalEntryEntity::find(), auth)
        .order_by_desc(journal_entry::Column::Datetime)
        .order_by_desc(journal_entry::Column::Id);
    let paginator = query.paginate(db, page_size as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for m in models {
        let name = JournalEntity::find_by_id(m.journal_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|j| j.name)
            .unwrap_or_else(|| "—".to_string());
        rows.push((m, name));
    }
    (rows, total)
}
