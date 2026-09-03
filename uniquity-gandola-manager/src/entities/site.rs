use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use lariv_rs::plugins::finance_invoices::entities::draft_invoice;

use crate::site_status::SiteStatus;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub name: String,
    /// External / business site identifier (distinct from primary key `id`).
    pub site_id: Option<String>,
    pub address: Option<String>,
    pub remarks: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub customer_id: i64,
    pub status: SiteStatus,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::gandola_site_link::Entity")]
    GandolaLinks,
    #[sea_orm(has_many = "super::site_invoice_link::Entity")]
    InvoiceLinks,
    #[sea_orm(has_many = "super::purchase_order::Entity")]
    PurchaseOrders,
}

impl Related<super::gandola_site_link::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GandolaLinks.def()
    }
}

impl Related<super::gandola::Entity> for Entity {
    fn to() -> RelationDef {
        super::gandola_site_link::Relation::Gandola.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::gandola_site_link::Relation::Site.def().rev())
    }
}

impl Related<super::site_invoice_link::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InvoiceLinks.def()
    }
}

impl Related<super::purchase_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PurchaseOrders.def()
    }
}

impl Related<draft_invoice::Entity> for Entity {
    fn to() -> RelationDef {
        super::site_invoice_link::Relation::DraftInvoice.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::site_invoice_link::Relation::Site.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
