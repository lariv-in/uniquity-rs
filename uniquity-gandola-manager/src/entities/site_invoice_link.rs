use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use lariv_plugin_finance_invoices::entities::{draft_invoice, posted_invoice};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "site_invoices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub site_id: i64,
    pub draft_invoice_id: Option<i64>,
    pub posted_invoice_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::site::Entity",
        from = "Column::SiteId",
        to = "super::site::Column::Id",
        on_delete = "Cascade"
    )]
    Site,
    #[sea_orm(
        belongs_to = "draft_invoice::Entity",
        from = "Column::DraftInvoiceId",
        to = "draft_invoice::Column::Id",
        on_delete = "Cascade"
    )]
    DraftInvoice,
    #[sea_orm(
        belongs_to = "posted_invoice::Entity",
        from = "Column::PostedInvoiceId",
        to = "posted_invoice::Column::Id",
        on_delete = "Cascade"
    )]
    PostedInvoice,
}

impl Related<super::site::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}

impl Related<draft_invoice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DraftInvoice.def()
    }
}

impl Related<posted_invoice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PostedInvoice.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
