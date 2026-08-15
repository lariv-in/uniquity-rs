use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "purchase_orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub number: String,
    pub date: NaiveDate,
    pub customer_id: i64,
    pub site_id: i64,
    pub file_id: Option<i64>,
    pub payment_term_id: Option<i64>,
    pub billing_address: Option<String>,
    pub shipping_address: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::purchase_order_line::Entity")]
    Lines,
    #[sea_orm(
        belongs_to = "super::purchase_order_payment_term::Entity",
        from = "Column::PaymentTermId",
        to = "super::purchase_order_payment_term::Column::Id",
        on_delete = "SetNull"
    )]
    PaymentTerm,
    #[sea_orm(
        belongs_to = "super::site::Entity",
        from = "Column::SiteId",
        to = "super::site::Column::Id",
        on_delete = "Restrict"
    )]
    Site,
}

impl Related<super::purchase_order_line::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lines.def()
    }
}

impl Related<super::purchase_order_payment_term::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaymentTerm.def()
    }
}

impl Related<super::site::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
