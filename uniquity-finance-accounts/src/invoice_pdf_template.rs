//! Default invoice PDF Typst template (Minijinja syntax).

/// Example / default invoice PDF template shipped with the deployment.
pub const DEFAULT_INVOICE_PDF_TEMPLATE: &str =
    include_str!("../templates/example_invoice_pdf_template.typ.tmpl");
