use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub const PAYMENT_TERM_TYPE_DUE_DATE: &str = "p_uniquity_finance_invoices.PaymentTermDueDate";
pub const PAYMENT_TERM_TYPE_RELATIVE: &str = "p_uniquity_finance_invoices.PaymentTermRelative";

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payment_terms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[sea_orm(column_name = "type")]
    pub term_type: String,
    pub backing_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
