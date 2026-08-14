use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use lariv_rs::plugins::finance_invoices::entities::draft_invoice;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "site_invoices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub site_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub draft_invoice_id: i64,
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

impl ActiveModelBehavior for ActiveModel {}
