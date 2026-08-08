//! Query scoping and helpers for employees and points.

use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Select,
    sea_query::Expr,
};

use lariv_rs::plugins::users::{
    entities::user::{self, Entity as UserEntity},
    state::AuthContext,
};

use uniquity_common::require_superuser;

use super::entities::{
    employee::{self, Entity as EmployeeEntity},
    points_transaction::{self, Entity as PointsTransactionEntity},
};

#[derive(Clone, Debug)]
pub struct EmployeeRow {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub user_email: String,
}

#[derive(Clone, Debug)]
pub struct PointsRow {
    pub id: i64,
    pub points: Decimal,
    pub from_user_name: String,
    pub to_employee_name: String,
    pub created_at: String,
}

pub fn scope_employees(query: Select<EmployeeEntity>, auth: &AuthContext) -> Select<EmployeeEntity> {
    if require_superuser(auth) {
        query
    } else {
        query.filter(Expr::cust("1 = 0"))
    }
}

pub fn scope_points(
    query: Select<PointsTransactionEntity>,
    auth: &AuthContext,
) -> Select<PointsTransactionEntity> {
    if require_superuser(auth) {
        query
    } else {
        query.filter(Expr::cust("1 = 0"))
    }
}

pub async fn load_user_map(
    db: &DatabaseConnection,
    user_ids: &[i64],
) -> std::collections::HashMap<i64, user::Model> {
    if user_ids.is_empty() {
        return std::collections::HashMap::new();
    }
    UserEntity::find()
        .filter(user::Column::Id.is_in(user_ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|u| (u.id, u))
        .collect()
}

pub async fn find_employee_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<employee::Model> {
    let query = EmployeeEntity::find_by_id(id);
    scope_employees(query, auth).one(db).await.ok().flatten()
}

pub async fn find_points_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<points_transaction::Model> {
    let query =
        PointsTransactionEntity::find_by_id(id);
    scope_points(query, auth).one(db).await.ok().flatten()
}

pub async fn employee_points_total(db: &DatabaseConnection, employee_id: i64) -> Decimal {
    use sea_orm::{FromQueryResult, Statement};
    #[derive(FromQueryResult)]
    struct SumRow {
        sum: Option<Decimal>,
    }
    let sql = format!(
        "SELECT COALESCE(SUM(points), 0) AS sum FROM points_transactions \
         WHERE to_employee_id = {employee_id}"
    );
    SumRow::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        sql,
    ))
    .one(db)
    .await
    .ok()
    .flatten()
    .and_then(|r| r.sum)
    .unwrap_or_else(|| Decimal::ZERO)
}

pub async fn query_employees(
    db: &DatabaseConnection,
    auth: &AuthContext,
    name: Option<&str>,
    email: Option<&str>,
    page: u32,
    page_size: u64,
) -> (Vec<EmployeeRow>, u32, u64) {
    let mut query = EmployeeEntity::find();
    query = scope_employees(query, auth);
    query = query.order_by_desc(employee::Column::UpdatedAt);

    let page = page.max(1);
    let paginator = query.paginate(db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();

    let user_ids: Vec<i64> = models.iter().map(|e| e.user_id).collect();
    let users = load_user_map(db, &user_ids).await;

    let mut rows: Vec<EmployeeRow> = models
        .into_iter()
        .filter_map(|e| {
            users.get(&e.user_id).map(|u| EmployeeRow {
                id: e.id,
                user_id: e.user_id,
                user_name: u.name.clone(),
                user_email: u.email.clone(),
            })
        })
        .collect();

    if name.filter(|s| !s.is_empty()).is_some() || email.filter(|s| !s.is_empty()).is_some() {
        rows.retain(|r| {
            let name_ok = name
                .filter(|s| !s.is_empty())
                .map(|n| r.user_name.to_lowercase().contains(&n.to_lowercase()))
                .unwrap_or(true);
            let email_ok = email
                .filter(|s| !s.is_empty())
                .map(|e| r.user_email.to_lowercase().contains(&e.to_lowercase()))
                .unwrap_or(true);
            name_ok && email_ok
        });
    }

    (rows, page, total)
}

pub async fn query_points(
    db: &DatabaseConnection,
    auth: &AuthContext,
    page: u32,
    page_size: u64,
) -> (Vec<PointsRow>, u32, u64) {
    let mut query = PointsTransactionEntity::find();
    query = scope_points(query, auth);
    query = query.order_by_desc(points_transaction::Column::CreatedAt);

    let page = page.max(1);
    let paginator = query.paginate(db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();

    let from_user_ids: Vec<i64> = models.iter().map(|p| p.from_user_id).collect();
    let to_employee_ids: Vec<i64> = models.iter().map(|p| p.to_employee_id).collect();

    let from_users = load_user_map(db, &from_user_ids).await;

    let employees = EmployeeEntity::find()
        .filter(employee::Column::Id.is_in(to_employee_ids.clone()))
        .all(db)
        .await
        .unwrap_or_default();
    let emp_user_ids: Vec<i64> = employees.iter().map(|e| e.user_id).collect();
    let emp_users = load_user_map(db, &emp_user_ids).await;
    let emp_map: std::collections::HashMap<i64, i64> =
        employees.iter().map(|e| (e.id, e.user_id)).collect();

    let rows: Vec<PointsRow> = models
        .into_iter()
        .map(|p| {
            let from_user_name = from_users
                .get(&p.from_user_id)
                .map(|u| u.name.clone())
                .unwrap_or_else(|| "—".into());
            let to_employee_name = emp_map
                .get(&p.to_employee_id)
                .and_then(|uid| emp_users.get(uid))
                .map(|u| u.name.clone())
                .unwrap_or_else(|| "—".into());
            PointsRow {
                id: p.id,
                points: p.points,
                from_user_name,
                to_employee_name,
                created_at: p
                    .created_at
                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default(),
            }
        })
        .collect();

    (rows, page, total)
}

pub async fn employee_display_name(db: &DatabaseConnection, employee_id: i64) -> String {
    let Some(emp) = EmployeeEntity::find_by_id(employee_id)
        .one(db)
        .await
        .ok()
        .flatten()
    else {
        return String::new();
    };
    load_user_map(db, &[emp.user_id])
        .await
        .get(&emp.user_id)
        .map(|u| u.name.clone())
        .unwrap_or_default()
}

pub async fn user_display_name(db: &DatabaseConnection, user_id: i64) -> String {
    load_user_map(db, &[user_id])
        .await
        .get(&user_id)
        .map(|u| u.name.clone())
        .unwrap_or_default()
}
