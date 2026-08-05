//! Journal entry and source document operations.

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, TransactionTrait,
};

use uniquity_common::decimal;

use crate::{
    account_validation::validate_leaf_account_balance_type,
    balance_type::BalanceType,
    entities::{
        accounting_preferences,
        journal_entry::{self, Entity as JournalEntryEntity},
        journal_entry_item::{self, Entity as JournalEntryItemEntity},
        source_doc::{self},
    },
};

#[derive(Clone, Debug)]
pub struct JournalLineSpec {
    pub account_id: i64,
    pub amount: Decimal,
}

pub async fn load_accounting_preferences(db: &DatabaseConnection) -> accounting_preferences::Model {
    use accounting_preferences::Entity as PrefsEntity;
    if let Ok(Some(p)) = PrefsEntity::find_by_id(1i64).one(db).await {
        return p;
    }
    let now = Utc::now();
    let am = accounting_preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        invoice_number_format: Set(Some("INV-{{YYYY}}-{{POSTED_SEQ}}".to_string())),
        ..Default::default()
    };
    am.insert(db).await.unwrap_or(accounting_preferences::Model {
        id: 1,
        created_at: Some(now),
        updated_at: Some(now),
        deleted_at: None,
        invoice_number_format: Some("INV-{{YYYY}}-{{POSTED_SEQ}}".to_string()),
        invoice_pdf_template: None,
    })
}

pub async fn create_source_doc<C: ConnectionTrait>(db: &C, source_doc_type: &str) -> Result<i64> {
    let now = Utc::now();
    let am = source_doc::ActiveModel {
        source_doc_type: Set(source_doc_type.to_string()),
        source_doc_id: Set(0),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let m = am.insert(db).await?;
    Ok(m.id)
}

pub async fn update_source_doc_id<C: ConnectionTrait>(
    db: &C,
    doc_id: i64,
    source_doc_id: i64,
) -> Result<()> {
    let mut am: source_doc::ActiveModel = source_doc::Entity::find_by_id(doc_id)
        .one(db)
        .await?
        .context("source doc not found")?
        .into();
    am.source_doc_id = Set(source_doc_id);
    am.updated_at = Set(Some(Utc::now()));
    am.update(db).await?;
    Ok(())
}

pub async fn insert_journal_entry<C: ConnectionTrait>(
    db: &C,
    datetime: DateTime<Utc>,
    journal_id: i64,
    source_doc_id: i64,
    lines: &[JournalLineSpec],
) -> Result<(i64, Vec<journal_entry_item::Model>)> {
    if lines.is_empty() {
        bail!("journal entry requires at least one line");
    }
    let now = Utc::now();
    let je_am = journal_entry::ActiveModel {
        datetime: Set(datetime),
        source_doc_id: Set(source_doc_id),
        journal_id: Set(journal_id),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let je = je_am.insert(db).await?;
    let mut items = Vec::with_capacity(lines.len());
    for line in lines {
        let item_am = journal_entry_item::ActiveModel {
            datetime: Set(datetime),
            account_id: Set(line.account_id),
            amount: Set(decimal::normalize(line.amount)),
            journal_entry_id: Set(je.id),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        items.push(item_am.insert(db).await?);
    }
    let balance: Decimal = items.iter().map(|i| i.amount).sum();
    if !decimal::dec_is_zero(balance) {
        bail!("internal error: journal entry does not balance");
    }
    Ok((je.id, items))
}

pub async fn create_journal_entry_with_lines(
    db: &DatabaseConnection,
    datetime: DateTime<Utc>,
    journal_id: i64,
    source_doc_type: &str,
    lines: &[JournalLineSpec],
) -> Result<(i64, Vec<journal_entry_item::Model>)> {
    let txn = db.begin().await?;
    let doc_id = create_source_doc(&txn, source_doc_type).await?;
    let (je_id, items) = insert_journal_entry(&txn, datetime, journal_id, doc_id, lines).await?;
    txn.commit().await?;
    Ok((je_id, items))
}

pub async fn load_journal_entry_lines(
    db: &DatabaseConnection,
    journal_entry_id: i64,
) -> Result<Vec<journal_entry_item::Model>> {
    Ok(JournalEntryItemEntity::find()
        .filter(journal_entry_item::Column::JournalEntryId.eq(journal_entry_id))
        .filter(journal_entry_item::Column::DeletedAt.is_null())
        .order_by_asc(journal_entry_item::Column::Id)
        .all(db)
        .await?)
}

pub async fn create_reversing_journal_entry_in_txn<C: ConnectionTrait>(
    db: &C,
    original_entry_id: i64,
    datetime: DateTime<Utc>,
    source_doc_type: &str,
) -> Result<(i64, i64)> {
    let orig = JournalEntryEntity::find_by_id(original_entry_id)
        .one(db)
        .await?
        .context("load journal entry")?;
    let items = JournalEntryItemEntity::find()
        .filter(journal_entry_item::Column::JournalEntryId.eq(original_entry_id))
        .filter(journal_entry_item::Column::DeletedAt.is_null())
        .order_by_asc(journal_entry_item::Column::Id)
        .all(db)
        .await?;
    if items.is_empty() {
        bail!("journal entry has no lines to reverse");
    }
    let doc_id = create_source_doc(db, source_doc_type).await?;
    let rev_lines: Vec<JournalLineSpec> = items
        .iter()
        .map(|it| JournalLineSpec {
            account_id: it.account_id,
            amount: decimal::dec_neg(it.amount),
        })
        .collect();
    let (rev_id, _) =
        insert_journal_entry(db, datetime, orig.journal_id, doc_id, &rev_lines).await?;
    Ok((doc_id, rev_id))
}

pub async fn validate_leaf_account_for_posting(
    db: &DatabaseConnection,
    account_id: i64,
    want: BalanceType,
    label: &str,
) -> Result<()> {
    validate_leaf_account_balance_type(db, account_id, want, label)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub fn debit_balance_type() -> BalanceType {
    BalanceType::Debit
}

pub fn credit_balance_type() -> BalanceType {
    BalanceType::Credit
}
