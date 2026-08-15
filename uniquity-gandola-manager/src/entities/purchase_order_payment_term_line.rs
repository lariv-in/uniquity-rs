use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use lariv_rs::plugins::finance_invoices::{PaymentTermAmountKind, PaymentTermDateKind};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "purchase_order_payment_term_lines")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub purchase_order_payment_term_id: i64,
    pub line_order: i32,
    pub date_kind: PaymentTermDateKind,
    pub due_datetime: Option<DateTime<Utc>>,
    pub due_duration: Option<i64>,
    pub amount_kind: PaymentTermAmountKind,
    #[sea_orm(column_type = "Decimal(Some((19, 6)))", nullable)]
    pub amount: Option<Decimal>,
    #[sea_orm(column_type = "Decimal(Some((19, 6)))", nullable)]
    pub amount_percentage: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::purchase_order_payment_term::Entity",
        from = "Column::PurchaseOrderPaymentTermId",
        to = "super::purchase_order_payment_term::Column::Id",
        on_delete = "Cascade"
    )]
    PaymentTerm,
}

impl Related<super::purchase_order_payment_term::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaymentTerm.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
