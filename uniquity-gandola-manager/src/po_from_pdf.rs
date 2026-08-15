//! Extract purchase-order fields from an uploaded PDF via Gemini.

use base64::Engine;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

use lariv_rs::genai::{Blob, Content, GenaiClient, Part, Role, content_text};
use lariv_rs::plugins::customer::entities::customer::{self, Entity as CustomerEntity};
use lariv_rs::plugins::filesystem::{
    node::{self, NodeError, NodeFile},
    state::FilesystemState,
};
use lariv_rs::plugins::finance_invoices::logic::default_payment_term_lines_json;
use lariv_rs::plugins::finance_invoices::{PaymentTermAmountKind, PaymentTermDateKind};

use crate::forms::PurchaseOrderForm;
use crate::po_lines::default_po_lines_json;

pub const DEFAULT_GEMINI_PO_MODEL: &str = "gemini-2.5-flash";
const MAX_OUTPUT_TOKENS: i32 = 8192;

const SYSTEM_PROMPT: &str = r#"You extract purchase-order data from a PDF.

Return ONLY a JSON object. No markdown, no commentary, no code fences.

Use this exact shape:
{
  "number": "",
  "date": "DD/MM/YYYY",
  "customer_name": "",
  "cin": "",
  "gstin": "",
  "billing_address": "",
  "shipping_address": "",
  "lines": [
    {
      "item_code": "",
      "description": "",
      "unit": "",
      "delivery_date": "DD/MM/YYYY",
      "quantity": "",
      "rate": ""
    }
  ],
  "payment_terms": [
    {
      "date_kind": "relative",
      "due_date": "",
      "due_duration": "15 days",
      "amount_kind": "relative",
      "amount": "",
      "amount_percentage": "100"
    }
  ]
}

Rules:
- date_kind must be one of: relative, relative_delivery, absolute
- amount_kind must be one of: relative, absolute
- Dates must be DD/MM/YYYY when present
- due_duration must include a unit, e.g. "15 days" or "30 days"
- Use empty strings for unknown values
- Include every line item from the PDF
- customer_name is the buyer / bill-to company name
- gstin is the buyer's GSTIN when present
- cin is the buyer's CIN when present (used to match an existing customer); otherwise leave empty
- quantity is the line qty / number of units only. rate is the unit price only. Sometimes the printed columns are switched (rate in the qty column, qty in the rate column) or values are misaligned. Assign by meaning, not by left-to-right order or column position. quantity is never a money amount; rate is never a count of units.
- description is a short item name for that row only (what is being ordered). Keep it brief — a few words, never more than about 10. Write a short name; do not paste the full line text, a long spec, a header blurb, scope of work, preamble, terms, footer, tax summary, or any other document text. Do not invent an item that is not on the line. If the row has no item description, use an empty string.
"#;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct ExtractedPurchaseOrder {
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub customer_name: String,
    #[serde(default)]
    pub cin: String,
    #[serde(default)]
    pub gstin: String,
    #[serde(default)]
    pub billing_address: String,
    #[serde(default)]
    pub shipping_address: String,
    #[serde(default)]
    pub lines: Vec<ExtractedPoLine>,
    #[serde(default)]
    pub payment_terms: Vec<ExtractedPaymentTerm>,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct ExtractedPoLine {
    #[serde(default)]
    pub item_code: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub delivery_date: String,
    #[serde(default)]
    pub quantity: String,
    #[serde(default)]
    pub rate: String,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct ExtractedPaymentTerm {
    #[serde(default)]
    pub date_kind: String,
    #[serde(default)]
    pub due_date: String,
    #[serde(default)]
    pub due_duration: String,
    #[serde(default)]
    pub amount_kind: String,
    #[serde(default)]
    pub amount: String,
    #[serde(default)]
    pub amount_percentage: String,
}

pub fn extract_json_object(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    let raw = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .unwrap_or(raw);
    let raw = raw.strip_suffix("```").unwrap_or(raw).trim();
    let start = raw
        .find('{')
        .ok_or_else(|| "AI did not return JSON".to_string())?;
    let end = raw
        .rfind('}')
        .ok_or_else(|| "AI did not return JSON".to_string())?;
    if end < start {
        return Err("AI did not return JSON".into());
    }
    Ok(raw[start..=end].to_string())
}

pub fn parse_extracted_purchase_order(raw: &str) -> Result<ExtractedPurchaseOrder, String> {
    let json = extract_json_object(raw)?;
    serde_json::from_str(&json).map_err(|e| format!("AI returned invalid purchase order JSON: {e}"))
}

fn normalize_date(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    lariv_rs::datetime::parse_date(raw)
        .map(lariv_rs::datetime::format_date)
        .unwrap_or_else(|| raw.to_string())
}

fn normalize_duration(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return format!("{raw} days");
    }
    raw.to_string()
}

fn parse_date_kind(raw: &str) -> PaymentTermDateKind {
    let s = raw.trim().to_ascii_lowercase().replace(' ', "_");
    match s.as_str() {
        "absolute" => PaymentTermDateKind::Absolute,
        "relative_delivery" | "delivery" | "relative_delivery_date" => {
            PaymentTermDateKind::RelativeDelivery
        }
        _ => PaymentTermDateKind::Relative,
    }
}

fn parse_amount_kind(raw: &str) -> PaymentTermAmountKind {
    match raw.trim().to_ascii_lowercase().as_str() {
        "absolute" => PaymentTermAmountKind::Absolute,
        _ => PaymentTermAmountKind::Relative,
    }
}

fn lines_json(lines: &[ExtractedPoLine]) -> String {
    if lines.is_empty() {
        return default_po_lines_json();
    }
    let rows: Vec<serde_json::Value> = lines
        .iter()
        .map(|line| {
            serde_json::json!({
                "item_code": line.item_code.trim(),
                "description": line.description.trim(),
                "unit": line.unit.trim(),
                "delivery_date": normalize_date(&line.delivery_date),
                "quantity": line.quantity.trim(),
                "rate": line.rate.trim(),
            })
        })
        .collect();
    serde_json::to_string(&rows).unwrap_or_else(|_| default_po_lines_json())
}

fn payment_terms_json(terms: &[ExtractedPaymentTerm]) -> String {
    if terms.is_empty() {
        return default_payment_term_lines_json();
    }
    let rows: Vec<serde_json::Value> = terms
        .iter()
        .map(|term| {
            serde_json::json!({
                "date_kind": parse_date_kind(&term.date_kind).as_str(),
                "due_date": normalize_date(&term.due_date),
                "due_duration": normalize_duration(&term.due_duration),
                "amount_kind": parse_amount_kind(&term.amount_kind).as_str(),
                "amount": term.amount.trim(),
                "amount_percentage": term.amount_percentage.trim(),
            })
        })
        .collect();
    serde_json::to_string(&rows).unwrap_or_else(|_| default_payment_term_lines_json())
}

pub fn form_from_extracted(
    extracted: &ExtractedPurchaseOrder,
    customer_id: i64,
    site_id: i64,
    file_id: i64,
) -> PurchaseOrderForm {
    let date = normalize_date(&extracted.date);
    PurchaseOrderForm {
        number: extracted.number.trim().to_string(),
        date: if date.is_empty() {
            lariv_rs::datetime::format_date(chrono::Utc::now().date_naive())
        } else {
            date
        },
        customer_id,
        site_id,
        file_id: if file_id > 0 {
            file_id.to_string()
        } else {
            String::new()
        },
        payment_term_lines_json: payment_terms_json(&extracted.payment_terms),
        po_lines_json: lines_json(&extracted.lines),
        billing_address: extracted.billing_address.trim().to_string(),
        shipping_address: extracted.shipping_address.trim().to_string(),
    }
}

fn pdf_vnode_name(filename: &str) -> String {
    let name = node::sanitize_node_name(filename);
    if name.is_empty() {
        return "purchase-order.pdf".into();
    }
    if std::path::Path::new(&name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    {
        name
    } else {
        format!("{name}.pdf")
    }
}

fn uniquify_filename(name: &str, attempt: u32) -> String {
    if attempt == 0 {
        return name.to_string();
    }
    let path = std::path::Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("purchase-order");
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_else(|| ".pdf".into());
    format!("{stem}-{attempt}{ext}")
}

/// Store the uploaded PDF as a filesystem vnode under Purchase Orders.
pub async fn store_purchase_order_pdf(
    fs: &FilesystemState,
    filename: &str,
    bytes: Vec<u8>,
) -> Result<i64, String> {
    let parent_id = node::ensure_directory_path(
        &fs.db,
        fs.store.as_ref(),
        None,
        &["Purchase Orders".to_string()],
    )
    .await
    .map_err(|e| e.to_string())?;
    let parent = match parent_id {
        Some(id) => node::get_by_id(&fs.db, id)
            .await
            .map_err(|e| e.to_string())?,
        None => None,
    };
    let base_name = pdf_vnode_name(filename);
    let mut name = String::new();
    for attempt in 0..50 {
        let candidate = uniquify_filename(&base_name, attempt);
        let taken = node::find_child(&fs.db, parent_id, &candidate, false)
            .await
            .map_err(|e| e.to_string())?
            .is_some();
        if !taken {
            name = candidate;
            break;
        }
    }
    if name.is_empty() {
        return Err("could not store the PDF with a unique name".into());
    }
    match node::create(
        &fs.db,
        fs.store.as_ref(),
        name.clone(),
        false,
        Some(NodeFile::Bytes {
            filename: name,
            data: bytes,
        }),
        parent.as_ref(),
    )
    .await
    {
        Ok(vnode) => Ok(vnode.id),
        Err(NodeError::Conflict) => Err("could not store the PDF with a unique name".into()),
        Err(e) => Err(e.to_string()),
    }
}

async fn find_customer_by_identifier(
    db: &DatabaseConnection,
    column: customer::Column,
    value: &str,
) -> Option<customer::Model> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(Some(c)) = CustomerEntity::find()
        .filter(column.eq(value))
        .one(db)
        .await
    {
        return Some(c);
    }
    let upper = value.to_ascii_uppercase();
    if upper != value {
        if let Ok(Some(c)) = CustomerEntity::find()
            .filter(column.eq(upper))
            .one(db)
            .await
        {
            return Some(c);
        }
    }
    None
}

pub async fn find_customer_for_extracted(
    db: &DatabaseConnection,
    extracted: &ExtractedPurchaseOrder,
) -> Option<customer::Model> {
    if let Some(c) =
        find_customer_by_identifier(db, customer::Column::Gstin, &extracted.gstin).await
    {
        return Some(c);
    }
    if let Some(c) = find_customer_by_identifier(db, customer::Column::Cin, &extracted.cin).await {
        return Some(c);
    }

    let name = extracted.customer_name.trim();
    if name.is_empty() {
        return None;
    }
    let Ok(matches) = CustomerEntity::find()
        .filter(customer::Column::Name.contains(name))
        .all(db)
        .await
    else {
        return None;
    };
    let exact: Vec<_> = matches
        .iter()
        .filter(|c| c.name.trim().eq_ignore_ascii_case(name))
        .cloned()
        .collect();
    if exact.len() == 1 {
        return exact.into_iter().next();
    }
    if matches.len() == 1 {
        return matches.into_iter().next();
    }
    None
}

pub async fn extract_purchase_order_from_pdf(
    api_key: &str,
    model: &str,
    pdf_bytes: &[u8],
) -> Result<ExtractedPurchaseOrder, String> {
    if api_key.trim().is_empty() {
        return Err("Set the Gemini API key in Gandola preferences.".into());
    }
    let model = model.trim();
    let model = if model.is_empty() {
        DEFAULT_GEMINI_PO_MODEL
    } else {
        model
    };
    let client = GenaiClient::new(api_key.to_string(), model.to_string());
    let encoded = base64::engine::general_purpose::STANDARD.encode(pdf_bytes);
    let contents = vec![Content {
        role: Role::User,
        parts: vec![
            Part {
                inline_data: Some(Blob {
                    mime_type: "application/pdf".into(),
                    data: encoded,
                }),
                ..Default::default()
            },
            Part {
                text: Some("Extract every field from this purchase order PDF as JSON.".into()),
                ..Default::default()
            },
        ],
    }];
    let content = client
        .generate_content_with_system(
            contents,
            Some(Content::text(Role::User, SYSTEM_PROMPT)),
            MAX_OUTPUT_TOKENS,
            &[],
        )
        .await
        .map_err(|e| e.to_string())?;
    let text = content_text(&content);
    if text.trim().is_empty() {
        return Err("Gemini returned an empty response".into());
    }
    parse_extracted_purchase_order(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fences_and_parses() {
        let raw = r#"```json
{"number":"PO-1","date":"2026-08-15","customer_name":"Acme","cin":"U123","gstin":"27ABC","billing_address":"A","shipping_address":"B","lines":[{"item_code":"X","description":"Bolt","unit":"pcs","delivery_date":"16/08/2026","quantity":"2","rate":"10"}],"payment_terms":[]}
```"#;
        let extracted = parse_extracted_purchase_order(raw).unwrap();
        assert_eq!(extracted.number, "PO-1");
        assert_eq!(extracted.customer_name, "Acme");
        assert_eq!(extracted.lines.len(), 1);
        assert_eq!(extracted.lines[0].item_code, "X");
    }

    #[test]
    fn form_uses_defaults_when_terms_missing() {
        let extracted = ExtractedPurchaseOrder {
            number: "PO-9".into(),
            date: "15/08/2026".into(),
            ..Default::default()
        };
        let form = form_from_extracted(&extracted, 0, 0, 0);
        assert_eq!(form.number, "PO-9");
        assert_eq!(form.date, "15/08/2026");
        assert_eq!(
            form.payment_term_lines_json,
            default_payment_term_lines_json()
        );
        assert_eq!(form.po_lines_json, default_po_lines_json());
    }

    #[test]
    fn normalizes_iso_date_and_bare_duration() {
        let extracted = ExtractedPurchaseOrder {
            date: "2026-08-15".into(),
            payment_terms: vec![ExtractedPaymentTerm {
                date_kind: "relative_delivery".into(),
                due_duration: "30".into(),
                amount_kind: "relative".into(),
                amount_percentage: "100".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let form = form_from_extracted(&extracted, 4, 8, 12);
        assert_eq!(form.date, "15/08/2026");
        assert_eq!(form.customer_id, 4);
        assert_eq!(form.site_id, 8);
        assert_eq!(form.file_id, "12");
        assert!(form.payment_term_lines_json.contains("relative_delivery"));
        assert!(form.payment_term_lines_json.contains("30 days"));
    }

    #[test]
    fn pdf_vnode_name_keeps_pdf_and_fills_blank() {
        assert_eq!(pdf_vnode_name("Acme PO.pdf"), "Acme PO.pdf");
        assert_eq!(pdf_vnode_name("notes"), "notes.pdf");
        assert_eq!(pdf_vnode_name("../"), "purchase-order.pdf");
        assert_eq!(uniquify_filename("Acme PO.pdf", 0), "Acme PO.pdf");
        assert_eq!(uniquify_filename("Acme PO.pdf", 2), "Acme PO-2.pdf");
    }
}
