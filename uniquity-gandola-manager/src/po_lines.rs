use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};

use lariv_rs::plugins::finance_common::decimal::{self, parse_decimal};

use crate::entities::purchase_order_line::{self, Entity as PurchaseOrderLineEntity};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PoLinePending {
    pub item_code: String,
    pub description: String,
    pub unit: String,
    pub delivery_date: String,
    pub quantity: String,
    pub rate: String,
}

#[derive(Serialize)]
struct PoLineFormRow {
    item_code: String,
    description: String,
    unit: String,
    delivery_date: String,
    quantity: String,
    rate: String,
}

pub fn default_po_lines_json() -> String {
    r#"[{"item_code":"","description":"","unit":"","delivery_date":"","quantity":"1","rate":""}]"#
        .to_string()
}

pub fn parse_po_lines_json(raw: &str) -> Result<Vec<PoLinePending>, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("add at least one purchase order line".to_string());
    }
    serde_json::from_str(raw).map_err(|e| format!("invalid lines data: {e}"))
}

fn parse_required_date(s: &str) -> Result<NaiveDate, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("delivery date is required".to_string());
    }
    lariv_rs::datetime::parse_date(s).ok_or_else(|| "invalid delivery date".to_string())
}

pub async fn po_lines_form_json(db: &DatabaseConnection, purchase_order_id: i64) -> String {
    let lines = PurchaseOrderLineEntity::find()
        .filter(purchase_order_line::Column::PurchaseOrderId.eq(purchase_order_id))
        .all(db)
        .await
        .unwrap_or_default();

    if lines.is_empty() {
        return default_po_lines_json();
    }

    let rows: Vec<PoLineFormRow> = lines
        .into_iter()
        .map(|ln| PoLineFormRow {
            item_code: ln.item_code,
            description: ln.description,
            unit: ln.unit,
            delivery_date: lariv_rs::datetime::format_date(ln.delivery_date),
            quantity: decimal::decimal_display(ln.quantity),
            rate: decimal::decimal_display(ln.rate),
        })
        .collect();

    serde_json::to_string(&rows).unwrap_or_else(|_| default_po_lines_json())
}

pub struct PoLineDisplay {
    pub item_code: String,
    pub description: String,
    pub unit: String,
    pub delivery_date: String,
    pub quantity: String,
    pub rate: String,
}

pub async fn load_po_line_displays(
    db: &DatabaseConnection,
    purchase_order_id: i64,
) -> Vec<PoLineDisplay> {
    let lines = PurchaseOrderLineEntity::find()
        .filter(purchase_order_line::Column::PurchaseOrderId.eq(purchase_order_id))
        .all(db)
        .await
        .unwrap_or_default();
    lines
        .into_iter()
        .map(|ln| PoLineDisplay {
            item_code: ln.item_code,
            description: ln.description,
            unit: ln.unit,
            delivery_date: lariv_rs::datetime::format_date(ln.delivery_date),
            quantity: decimal::decimal_display(ln.quantity),
            rate: decimal::decimal_display(ln.rate),
        })
        .collect()
}

pub async fn replace_po_lines(
    db: &DatabaseConnection,
    purchase_order_id: i64,
    lines: &[PoLinePending],
) -> Result<(), String> {
    if lines.is_empty() {
        return Err("add at least one purchase order line".to_string());
    }
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    PurchaseOrderLineEntity::delete_many()
        .filter(purchase_order_line::Column::PurchaseOrderId.eq(purchase_order_id))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now();
    for row in lines {
        let item_code = row.item_code.trim();
        if item_code.is_empty() {
            return Err("item code is required".to_string());
        }
        let description = row.description.trim();
        if description.is_empty() {
            return Err("description is required".to_string());
        }
        let unit = row.unit.trim();
        if unit.is_empty() {
            return Err("unit is required".to_string());
        }
        let delivery_date = parse_required_date(&row.delivery_date)?;
        let qty = parse_decimal(&row.quantity)
            .filter(|d| *d > Decimal::ZERO)
            .ok_or_else(|| "quantity must be positive".to_string())?;
        let rate = parse_decimal(&row.rate).ok_or_else(|| "invalid rate".to_string())?;
        if rate < Decimal::ZERO {
            return Err("rate must be non-negative".to_string());
        }
        purchase_order_line::ActiveModel {
            purchase_order_id: Set(purchase_order_id),
            item_code: Set(item_code.to_string()),
            description: Set(description.to_string()),
            unit: Set(unit.to_string()),
            delivery_date: Set(delivery_date),
            rate: Set(decimal::normalize(rate)),
            quantity: Set(decimal::normalize(qty)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(|e| e.to_string())?;
    }
    txn.commit().await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_empty() {
        assert!(parse_po_lines_json("").is_err());
        assert!(parse_po_lines_json("   ").is_err());
    }

    #[test]
    fn parse_reads_fields() {
        let raw = r#"[{"item_code":"A1","description":"Bolt","unit":"pcs","delivery_date":"15/08/2026","quantity":"2","rate":"10.5"}]"#;
        let lines = parse_po_lines_json(raw).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].item_code, "A1");
        assert_eq!(lines[0].unit, "pcs");
    }
}
