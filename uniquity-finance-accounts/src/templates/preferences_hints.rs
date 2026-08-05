//! Tooltip copy for accounting preference fields (see invoice_number.rs and invoice_pdf.rs).

pub const INVOICE_NUMBER_FORMAT_HINT: &str = "\
Applied when posting a draft that has no invoice number (blank or unset). \
If the draft already has a number, that value is kept unchanged.

This is not a template engine — only these literal placeholders are replaced at post time:
• {{FISCAL_CODE}} — fiscal year code for the invoice date (Finance → Fiscal years; empty if none matches)
• {{YY}} — two-digit year of the invoice date (e.g. 26)
• {{YYYY}} — four-digit year (e.g. 2026)
• {{POSTED_SEQ}} — next posted_invoices row id (MAX(id)+1 among live rows), not a per-year sequence counter

Leave blank to default to INV-{{YYYY}}-{{POSTED_SEQ}}.
Example: INV/{{FISCAL_CODE}}/{{POSTED_SEQ}}";

pub const INVOICE_PDF_TEMPLATE_HINT: &str = "\
Minijinja (Jinja2-style) template. Minijinja expands {% … %} and {{ … }}; the result must be valid Typst source, which is then compiled to PDF. Leave blank to use the built-in example template.

Root context (PascalCase field names):
• ID, Number, Reference, PaymentReference, BankAccount
• DatetimeDisplay (DD/MM/YYYY), DatetimeYear, DatetimeMonth, DatetimeDay, DeliveryDate
• CustomerId, Customer.Name, Customer.Address, Customer.GSTIN, Customer.PAN, Customer.Phone, Customer.Email, Customer.Website
• PaymentTermType, PaymentTermId, PaymentTerm.Summary
• Taxes[] — invoice-level taxes: Name, Percentage, TaxType (levied or withholding)
• Lines[] — Product.Name, Product.HSNCode, Product.Reference, Quantity, Rate, line Taxes[]
• Payments[] — Amount, DatetimeDisplay

Template functions:
• num2words(n), num2wordsAnd(n) — English cardinal words
• num2wordsRupees(n) — amount in words with \"Rupees\"
• invoiceGrandTotalWords() — receivable grand total in words (computed from lines + header taxes)
• urlImage(url) — download an image into the Typst work directory for #image(\"…\")

Use Preview sample PDF below the template field to render the built-in example invoice data before saving. Use default template to restore the shipped example (you will be asked to confirm before the field is overwritten).

Write Typst markup (#set, #let, #table, …) literally; only {% %} and {{ }} regions are evaluated by Minijinja. Reset this field to empty and save to restore the shipped example layout.";
