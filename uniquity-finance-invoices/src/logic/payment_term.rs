//! Payment term create (polymorphic due date / relative backing).

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DatabaseConnection, EntityTrait,
    TransactionTrait,
};

use crate::entities::{
    payment_term::{self, PAYMENT_TERM_TYPE_DUE_DATE, PAYMENT_TERM_TYPE_RELATIVE},
    payment_term_due_date::{self, Entity as PaymentTermDueDateEntity},
    payment_term_relative::{self, Entity as PaymentTermRelativeEntity},
};

pub struct CreatePaymentTermDueDate {
    pub datetime: DateTime<Utc>,
}

pub struct CreatePaymentTermRelative {
    pub duration_nanos: i64,
}

pub enum CreatePaymentTermInput {
    DueDate(CreatePaymentTermDueDate),
    Relative(CreatePaymentTermRelative),
}

pub fn parse_due_datetime(s: &str, tz: &str) -> Result<DateTime<Utc>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("datetime is required for due date payment term".to_string());
    }
    lariv_rs::datetime::DatetimeLocalInput::from_raw(s)
        .to_stored(tz)
        .ok_or_else(|| "invalid datetime".to_string())
}

/// Parse an HTML `type="date"` value (`YYYY-MM-DD`) as end-of-day in `tz`.
pub fn parse_due_date(s: &str, tz: &str) -> Result<DateTime<Utc>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("due date is required".to_string());
    }
    let date = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| "invalid due date".to_string())?;
    let naive = date
        .and_hms_opt(23, 59, 59)
        .ok_or_else(|| "invalid due date".to_string())?;
    lariv_rs::datetime::parse_timezone(tz)
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .ok_or_else(|| "invalid due date".to_string())
}

pub fn format_due_date_local_input(dt: DateTime<Utc>, tz: &str) -> String {
    dt.with_timezone(&lariv_rs::datetime::parse_timezone(tz))
        .format("%Y-%m-%d")
        .to_string()
}

pub struct PaymentTermFormValues {
    pub term_type: String,
    pub due_datetime: String,
    pub duration: String,
}

pub async fn payment_term_form_values(
    db: &DatabaseConnection,
    pt: &payment_term::Model,
    tz: &str,
) -> PaymentTermFormValues {
    match pt.term_type.as_str() {
        PAYMENT_TERM_TYPE_DUE_DATE => {
            let due_datetime = PaymentTermDueDateEntity::find_by_id(pt.backing_id)
                .one(db)
                .await
                .ok()
                .flatten()
                .map(|row| {
                    lariv_rs::datetime::DatetimeLocalInput::from_stored(row.datetime, tz)
                        .into_string()
                })
                .unwrap_or_default();
            PaymentTermFormValues {
                term_type: pt.term_type.clone(),
                due_datetime,
                duration: String::new(),
            }
        }
        PAYMENT_TERM_TYPE_RELATIVE => {
            let duration = PaymentTermRelativeEntity::find_by_id(pt.backing_id)
                .one(db)
                .await
                .ok()
                .flatten()
                .map(|row| lariv_rs::duration::format_duration(row.duration))
                .unwrap_or_default();
            PaymentTermFormValues {
                term_type: pt.term_type.clone(),
                due_datetime: String::new(),
                duration,
            }
        }
        _ => PaymentTermFormValues {
            term_type: pt.term_type.clone(),
            due_datetime: String::new(),
            duration: String::new(),
        },
    }
}

pub async fn update_payment_term(
    db: &DatabaseConnection,
    pt_id: i64,
    input: CreatePaymentTermInput,
) -> Result<payment_term::Model, String> {
    let Some(existing) = payment_term::Entity::find_by_id(pt_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Err("payment term not found".to_string());
    };

    let (new_term_type, due_datetime, duration_nanos) = match input {
        CreatePaymentTermInput::DueDate(d) => {
            (PAYMENT_TERM_TYPE_DUE_DATE.to_string(), Some(d.datetime), None)
        }
        CreatePaymentTermInput::Relative(r) => {
            if r.duration_nanos <= 0 {
                return Err("duration must be positive".to_string());
            }
            (
                PAYMENT_TERM_TYPE_RELATIVE.to_string(),
                None,
                Some(r.duration_nanos),
            )
        }
    };

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let now = Utc::now();

    let new_backing_id = if existing.term_type == new_term_type {
        match new_term_type.as_str() {
            PAYMENT_TERM_TYPE_DUE_DATE => {
                let datetime = due_datetime.ok_or_else(|| "invalid datetime".to_string())?;
                let am = payment_term_due_date::ActiveModel {
                    id: Set(existing.backing_id),
                    datetime: Set(datetime),
                    updated_at: Set(Some(now)),
                    ..Default::default()
                };
                am.update(&txn).await.map_err(|e| e.to_string())?;
                existing.backing_id
            }
            PAYMENT_TERM_TYPE_RELATIVE => {
                let duration = duration_nanos.ok_or_else(|| "invalid duration".to_string())?;
                let am = payment_term_relative::ActiveModel {
                    id: Set(existing.backing_id),
                    duration: Set(duration),
                    updated_at: Set(Some(now)),
                    ..Default::default()
                };
                am.update(&txn).await.map_err(|e| e.to_string())?;
                existing.backing_id
            }
            _ => return Err("invalid payment term type".to_string()),
        }
    } else {
        let backing_id = match new_term_type.as_str() {
            PAYMENT_TERM_TYPE_DUE_DATE => {
                let datetime = due_datetime.ok_or_else(|| "invalid datetime".to_string())?;
                let row = payment_term_due_date::ActiveModel {
                    datetime: Set(datetime),
                    created_at: Set(Some(now)),
                    updated_at: Set(Some(now)),
                    ..Default::default()
                }
                .insert(&txn)
                .await
                .map_err(|e| e.to_string())?;
                row.id
            }
            PAYMENT_TERM_TYPE_RELATIVE => {
                let duration = duration_nanos.ok_or_else(|| "invalid duration".to_string())?;
                let row = payment_term_relative::ActiveModel {
                    duration: Set(duration),
                    created_at: Set(Some(now)),
                    updated_at: Set(Some(now)),
                    ..Default::default()
                }
                .insert(&txn)
                .await
                .map_err(|e| e.to_string())?;
                row.id
            }
            _ => return Err("invalid payment term type".to_string()),
        };
        delete_payment_term_backing(&txn, &existing).await?;
        backing_id
    };

    let pt = payment_term::ActiveModel {
        id: Set(existing.id),
        term_type: Set(new_term_type),
        backing_id: Set(new_backing_id),
        updated_at: Set(Some(now)),
        ..Default::default()
    }
    .update(&txn)
    .await
    .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(pt)
}

async fn delete_payment_term_backing(
    txn: &sea_orm::DatabaseTransaction,
    pt: &payment_term::Model,
) -> Result<(), String> {
    match pt.term_type.as_str() {
        PAYMENT_TERM_TYPE_DUE_DATE => {
            payment_term_due_date::Entity::delete_by_id(pt.backing_id)
                .exec(txn)
                .await
                .map_err(|e| e.to_string())?;
        }
        PAYMENT_TERM_TYPE_RELATIVE => {
            payment_term_relative::Entity::delete_by_id(pt.backing_id)
                .exec(txn)
                .await
                .map_err(|e| e.to_string())?;
        }
        _ => {}
    }
    Ok(())
}

pub async fn insert_payment_term<C: ConnectionTrait>(
    conn: &C,
    input: CreatePaymentTermInput,
) -> Result<payment_term::Model, String> {
    let now = Utc::now();
    let (term_type, backing_id) = match input {
        CreatePaymentTermInput::DueDate(d) => {
            let row = payment_term_due_date::ActiveModel {
                datetime: Set(d.datetime),
                created_at: Set(Some(now)),
                updated_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(conn)
            .await
            .map_err(|e| e.to_string())?;
            (PAYMENT_TERM_TYPE_DUE_DATE.to_string(), row.id)
        }
        CreatePaymentTermInput::Relative(r) => {
            if r.duration_nanos <= 0 {
                return Err("duration must be positive".to_string());
            }
            let row = payment_term_relative::ActiveModel {
                duration: Set(r.duration_nanos),
                created_at: Set(Some(now)),
                updated_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(conn)
            .await
            .map_err(|e| e.to_string())?;
            (PAYMENT_TERM_TYPE_RELATIVE.to_string(), row.id)
        }
    };
    payment_term::ActiveModel {
        term_type: Set(term_type),
        backing_id: Set(backing_id),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(|e| e.to_string())
}

pub async fn create_payment_term(
    db: &DatabaseConnection,
    input: CreatePaymentTermInput,
) -> Result<payment_term::Model, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let pt = insert_payment_term(&txn, input).await?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(pt)
}

/// Human-readable label for a polymorphic payment term type key.
pub fn payment_term_type_label(term_type: &str) -> &str {
    match term_type {
        PAYMENT_TERM_TYPE_DUE_DATE => "Due Date",
        PAYMENT_TERM_TYPE_RELATIVE => "Relative",
        other => other,
    }
}

pub async fn payment_term_summary(
    db: &DatabaseConnection,
    pt: &payment_term::Model,
    tz: &str,
) -> String {
    match pt.term_type.as_str() {
        PAYMENT_TERM_TYPE_DUE_DATE => {
            if let Ok(Some(row)) = PaymentTermDueDateEntity::find_by_id(pt.backing_id)
                .one(db)
                .await
            {
                return lariv_rs::datetime::DatetimeLabel::short(row.datetime, tz).into_string();
            }
            format!("Due date #{}", pt.backing_id)
        }
        PAYMENT_TERM_TYPE_RELATIVE => {
            if let Ok(Some(row)) = PaymentTermRelativeEntity::find_by_id(pt.backing_id)
                .one(db)
                .await
            {
                return lariv_rs::duration::format_duration(row.duration);
            }
            format!("Relative #{}", pt.backing_id)
        }
        _ => format!("{} #{}", pt.term_type, pt.backing_id),
    }
}
