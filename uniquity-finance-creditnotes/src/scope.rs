use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select};

use lariv_rs::plugins::users::state::AuthContext;

use uniquity_common::is_superuser;

use crate::entities::credit_note::{self, Entity as CreditNoteEntity};

pub fn scope_credit_notes(
    query: Select<CreditNoteEntity>,
    auth: &AuthContext,
) -> Select<CreditNoteEntity> {
    if is_superuser(auth) {
        return query;
    }
    query.filter(credit_note::Column::Id.eq(-1))
}

pub async fn find_credit_note_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<credit_note::Model> {
    let query = CreditNoteEntity::find_by_id(id).filter(credit_note::Column::DeletedAt.is_null());
    scope_credit_notes(query, auth).one(db).await.ok().flatten()
}

pub fn order_credit_notes(query: Select<CreditNoteEntity>) -> Select<CreditNoteEntity> {
    query
        .order_by_desc(credit_note::Column::Datetime)
        .order_by_desc(credit_note::Column::Id)
}
