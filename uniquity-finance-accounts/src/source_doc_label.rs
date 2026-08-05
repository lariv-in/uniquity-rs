//! Human-readable labels for stored source document type keys.

const CREDIT_NOTE: &str = "p_uniquity_finance_creditnotes.CreditNote";
const POSTED_INVOICE: &str = "p_uniquity_finance_invoices.PostedInvoice";
const PAYMENT: &str = "p_uniquity_finance_invoices.Payment";
const PAYMENT_BATCH: &str = "p_uniquity_finance_invoices.PaymentBatch";

/// Map a stored source document type key to a display label.
pub fn source_doc_type_label(typ: &str) -> String {
    match typ {
        CREDIT_NOTE => "Credit Note".into(),
        POSTED_INVOICE => "Posted Invoice".into(),
        PAYMENT => "Payment".into(),
        PAYMENT_BATCH => "Payment Batch".into(),
        _ => humanize_type_name(typ),
    }
}

/// Build a short summary for pickers and journal entry headers.
pub fn source_doc_summary(typ: &str, source_doc_id: i64, row_id: i64) -> String {
    format!(
        "{} · ref {} · #{}",
        source_doc_type_label(typ),
        source_doc_id,
        row_id
    )
}

/// Build a compact summary without the source_docs row id.
pub fn source_doc_ref_summary(typ: &str, source_doc_id: i64) -> String {
    format!("{} · ref {}", source_doc_type_label(typ), source_doc_id)
}

fn humanize_type_name(typ: &str) -> String {
    let name = typ.rsplit('.').next().unwrap_or(typ);
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push(' ');
        }
        out.extend(ch.to_lowercase());
    }
    if out.is_empty() {
        typ.to_string()
    } else {
        let mut chars = out.chars();
        match chars.next() {
            None => typ.to_string(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_note_label() {
        assert_eq!(
            source_doc_type_label(CREDIT_NOTE),
            "Credit Note"
        );
    }

    #[test]
    fn unknown_type_humanizes() {
        assert_eq!(
            source_doc_type_label("p_example.SomeDocument"),
            "Some document"
        );
    }
}
