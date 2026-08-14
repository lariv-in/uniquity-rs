use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::site_status::SiteStatus;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub name: String,
    pub address: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub customer_id: i64,
    pub status: SiteStatus,
    pub po_rent: Option<String>,
    pub po_dti: Option<String>,
    pub po_tpi: Option<String>,
    pub po_extn1: Option<String>,
    pub po_extn2: Option<String>,
    pub po_extn3: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::gandola_site_link::Entity")]
    GandolaLinks,
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

impl ActiveModelBehavior for ActiveModel {}
