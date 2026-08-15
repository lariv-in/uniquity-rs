use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QueryOrder,
};

use lariv_rs::html_form::{FieldRender, FormCtx, FormWidget};
use lariv_rs::plugins::finance_common::decimal::{self, parse_decimal};
use lariv_rs::plugins::finance_invoices::components::{
    InputPaymentTermLinesDraft, PaymentTermDateKindOption, input_payment_term_lines_draft,
};
use lariv_rs::plugins::finance_invoices::logic::{
    DraftPaymentTermLineInput, default_payment_term_lines_json, parse_due_date_for_term,
};
use lariv_rs::plugins::finance_invoices::{PaymentTermAmountKind, PaymentTermDateKind};
use maud::Markup;

use crate::entities::purchase_order_payment_term::{
    self, Entity as PurchaseOrderPaymentTermEntity,
};
use crate::entities::purchase_order_payment_term_line::{
    self, Entity as PurchaseOrderPaymentTermLineEntity,
};

const PERCENTAGE_TOLERANCE: Decimal = Decimal::ONE;

pub const PO_PAYMENT_TERM_DATE_KINDS: &[PaymentTermDateKindOption] = &[
    PaymentTermDateKindOption {
        value: PaymentTermDateKind::Relative.as_str(),
        label: "Relative (order date)",
    },
    PaymentTermDateKindOption {
        value: PaymentTermDateKind::RelativeDelivery.as_str(),
        label: "Relative (delivery date)",
    },
    PaymentTermDateKindOption {
        value: PaymentTermDateKind::Absolute.as_str(),
        label: "Absolute (date)",
    },
];

pub struct PurchaseOrderPaymentTermLinesDraft;

impl FormWidget for PurchaseOrderPaymentTermLinesDraft {
    fn render(_ctx: &FormCtx<'_>, field: &FieldRender<'_>) -> Markup {
        input_payment_term_lines_draft(InputPaymentTermLinesDraft {
            name: field.name,
            defaults: field.value,
            date_kinds: PO_PAYMENT_TERM_DATE_KINDS,
            default_date_kind: PaymentTermDateKind::Relative.as_str(),
            ..Default::default()
        })
    }
}

fn relative_duration_fields(
    line: &DraftPaymentTermLineInput,
) -> Result<(Option<DateTime<Utc>>, Option<i64>), String> {
    let nanos = lariv_rs::duration::parse_duration(line.due_duration.as_deref().unwrap_or(""))
        .map_err(|e| e.to_string())?;
    Ok((None, Some(nanos)))
}

fn validate_duration(raw: &str) -> Result<(), String> {
    let dur = raw.trim();
    if dur.is_empty() {
        return Err("duration is required for relative date".to_string());
    }
    let nanos =
        lariv_rs::duration::parse_duration(dur).map_err(|e| format!("invalid duration: {e}"))?;
    if nanos <= 0 {
        return Err("duration must be positive".to_string());
    }
    Ok(())
}

fn validate_po_line_input(line: &DraftPaymentTermLineInput, tz: &str) -> Result<(), String> {
    match line.date_kind {
        PaymentTermDateKind::Absolute => {
            parse_due_date_for_term(line.due_date.as_deref().unwrap_or(""), tz)?;
        }
        PaymentTermDateKind::Relative => {
            validate_duration(line.due_duration.as_deref().unwrap_or(""))?;
        }
        PaymentTermDateKind::RelativeDelivery => {
            validate_duration(line.due_duration.as_deref().unwrap_or(""))?;
        }
    }

    match line.amount_kind {
        PaymentTermAmountKind::Absolute => {
            let amt = parse_decimal(line.amount.as_deref().unwrap_or(""))
                .ok_or_else(|| "amount is required for absolute amount".to_string())?;
            if amt <= Decimal::ZERO {
                return Err("amount must be positive".to_string());
            }
        }
        PaymentTermAmountKind::Relative => {
            let pct = parse_decimal(line.amount_percentage.as_deref().unwrap_or(""))
                .ok_or_else(|| "percentage is required for relative amount".to_string())?;
            if pct <= Decimal::ZERO {
                return Err("percentage must be positive".to_string());
            }
        }
    }
    Ok(())
}

fn validate_purchase_order_payment_term_lines(
    lines: &[DraftPaymentTermLineInput],
    tz: &str,
) -> Result<(), String> {
    if lines.is_empty() {
        return Err("add at least one payment term line".to_string());
    }
    for line in lines {
        validate_po_line_input(line, tz)?;
    }

    let all_relative = lines
        .iter()
        .all(|l| l.amount_kind == PaymentTermAmountKind::Relative);
    if all_relative {
        let sum: Decimal = lines
            .iter()
            .map(|l| {
                parse_decimal(l.amount_percentage.as_deref().unwrap_or("")).unwrap_or(Decimal::ZERO)
            })
            .sum();
        let diff = (sum - Decimal::from(100)).abs();
        if diff > PERCENTAGE_TOLERANCE {
            return Err(format!("relative percentages must sum to 100 (got {sum})"));
        }
    }
    Ok(())
}

fn line_input_to_active(
    purchase_order_payment_term_id: i64,
    line_order: i32,
    line: &DraftPaymentTermLineInput,
    tz: &str,
    now: DateTime<Utc>,
) -> Result<purchase_order_payment_term_line::ActiveModel, String> {
    let (due_datetime, due_duration) = match line.date_kind {
        PaymentTermDateKind::Absolute => (
            Some(parse_due_date_for_term(
                line.due_date.as_deref().unwrap_or(""),
                tz,
            )?),
            None,
        ),
        PaymentTermDateKind::Relative => relative_duration_fields(line)?,
        PaymentTermDateKind::RelativeDelivery => relative_duration_fields(line)?,
    };

    let (amount, amount_percentage) = match line.amount_kind {
        PaymentTermAmountKind::Absolute => (
            Some(decimal::normalize(
                parse_decimal(line.amount.as_deref().unwrap_or("")).unwrap(),
            )),
            None,
        ),
        PaymentTermAmountKind::Relative => (
            None,
            Some(decimal::normalize(
                parse_decimal(line.amount_percentage.as_deref().unwrap_or("")).unwrap(),
            )),
        ),
    };

    Ok(purchase_order_payment_term_line::ActiveModel {
        purchase_order_payment_term_id: Set(purchase_order_payment_term_id),
        line_order: Set(line_order),
        date_kind: Set(line.date_kind),
        due_datetime: Set(due_datetime),
        due_duration: Set(due_duration),
        amount_kind: Set(line.amount_kind),
        amount: Set(amount),
        amount_percentage: Set(amount_percentage),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    })
}

pub async fn upsert_purchase_order_payment_term_lines<C: ConnectionTrait>(
    conn: &C,
    existing_term_id: Option<i64>,
    lines: &[DraftPaymentTermLineInput],
    tz: &str,
) -> Result<purchase_order_payment_term::Model, String> {
    validate_purchase_order_payment_term_lines(lines, tz)?;
    let now = Utc::now();

    let term = if let Some(id) = existing_term_id {
        PurchaseOrderPaymentTermEntity::find_by_id(id)
            .one(conn)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "purchase order payment term not found".to_string())?
    } else {
        purchase_order_payment_term::ActiveModel {
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(conn)
        .await
        .map_err(|e| e.to_string())?
    };

    PurchaseOrderPaymentTermLineEntity::delete_many()
        .filter(purchase_order_payment_term_line::Column::PurchaseOrderPaymentTermId.eq(term.id))
        .exec(conn)
        .await
        .map_err(|e| e.to_string())?;

    for (i, line) in lines.iter().enumerate() {
        let am = line_input_to_active(term.id, i as i32, line, tz, now)?;
        am.insert(conn).await.map_err(|e| e.to_string())?;
    }

    Ok(term)
}

async fn load_lines_for_term<C: ConnectionTrait>(
    conn: &C,
    term_id: i64,
) -> Result<Vec<purchase_order_payment_term_line::Model>, String> {
    PurchaseOrderPaymentTermLineEntity::find()
        .filter(purchase_order_payment_term_line::Column::PurchaseOrderPaymentTermId.eq(term_id))
        .order_by_asc(purchase_order_payment_term_line::Column::LineOrder)
        .order_by_asc(purchase_order_payment_term_line::Column::Id)
        .all(conn)
        .await
        .map_err(|e| e.to_string())
}

pub async fn payment_term_lines_form_json_for_po_term<C: ConnectionTrait>(
    conn: &C,
    term_id: Option<i64>,
    tz: &str,
) -> String {
    let lines = match term_id {
        Some(id) => load_lines_for_term(conn, id).await.unwrap_or_default(),
        None => Vec::new(),
    };
    if lines.is_empty() {
        return default_payment_term_lines_json();
    }
    let out: Vec<serde_json::Value> = lines
        .iter()
        .map(|l| {
            let due_date = l
                .due_datetime
                .map(|dt| lariv_rs::datetime::format_date_in_tz(dt, tz))
                .unwrap_or_default();
            let due_duration = l
                .due_duration
                .map(lariv_rs::duration::format_duration)
                .unwrap_or_default();
            serde_json::json!({
                "date_kind": l.date_kind,
                "due_date": due_date,
                "due_duration": due_duration,
                "amount_kind": l.amount_kind,
                "amount": l.amount.map(decimal::decimal_display).unwrap_or_default(),
                "amount_percentage": l.amount_percentage.map(decimal::decimal_display).unwrap_or_default(),
            })
        })
        .collect();
    serde_json::to_string(&out).unwrap_or_else(|_| default_payment_term_lines_json())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relative_delivery_line() -> DraftPaymentTermLineInput {
        DraftPaymentTermLineInput {
            date_kind: PaymentTermDateKind::RelativeDelivery,
            due_date: None,
            due_duration: Some("15 days".into()),
            amount_kind: PaymentTermAmountKind::Relative,
            amount: None,
            amount_percentage: Some("100".into()),
        }
    }

    #[test]
    fn accepts_relative_to_delivery_date() {
        validate_purchase_order_payment_term_lines(&[relative_delivery_line()], "UTC")
            .expect("relative_delivery should be valid");
    }

    #[test]
    fn relative_delivery_requires_duration() {
        let mut line = relative_delivery_line();
        line.due_duration = Some(String::new());
        let err = validate_purchase_order_payment_term_lines(&[line], "UTC").unwrap_err();
        assert!(err.contains("duration"));
    }
}
