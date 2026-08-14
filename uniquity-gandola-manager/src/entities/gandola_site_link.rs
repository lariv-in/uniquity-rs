use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "p_gandola_sites")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub gandola_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub site_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::gandola::Entity",
        from = "Column::GandolaId",
        to = "super::gandola::Column::Id",
        on_delete = "Cascade"
    )]
    Gandola,
    #[sea_orm(
        belongs_to = "super::site::Entity",
        from = "Column::SiteId",
        to = "super::site::Column::Id",
        on_delete = "Cascade"
    )]
    Site,
}

impl Related<super::gandola::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Gandola.def()
    }
}

impl Related<super::site::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
