use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "gandolas")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::gandola_site_link::Entity")]
    SiteLinks,
}

impl Related<super::gandola_site_link::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SiteLinks.def()
    }
}

impl Related<super::site::Entity> for Entity {
    fn to() -> RelationDef {
        super::gandola_site_link::Relation::Site.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::gandola_site_link::Relation::Gandola.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
