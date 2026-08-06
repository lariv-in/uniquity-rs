use frunk::Generic;
use maud::{Markup, PreEscaped, html};

use lariv_rs::{
    components::{
        ButtonSubmit, FieldTitle, FormOpts, ShellChrome, button_submit, container_column,
        container_row, field_title, form, form_hx_post_main, label_newline_hint,
        attrs::escape_attr,
        htmx::{HTMX_SWAP_BODY_MODAL, HTMX_TARGET_BODY_MODAL},
    },
    html_form::FormFieldKey,
    template::{RenderAppPane, RenderTemplate},
};

use crate::{
    forms::AccountingPreferencesFormField,
    invoice_pdf_template::DEFAULT_INVOICE_PDF_TEMPLATE,
    routes::AccountingPreferencesPostRouteTag,
};

use super::common::{app_scaffold, layout_main_content, layout_with_sidebar};
use super::preferences_hints::{INVOICE_NUMBER_FORMAT_HINT, INVOICE_PDF_TEMPLATE_HINT};

#[derive(Generic)]
pub struct AccountingPreferencesPage {
    pub invoice_number_format: String,
    pub invoice_pdf_template: String,
    pub addon_inputs: Markup,
}

impl AccountingPreferencesPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Accounting Preferences", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(AccountingPreferencesPostRouteTag),
                    inputs: html! {
                        (label_newline_hint(
                            "Invoice number format",
                            Some(INVOICE_NUMBER_FORMAT_HINT),
                            html! {
                                input type="text"
                                    name=(AccountingPreferencesFormField::InvoiceNumberFormat.html_name())
                                    class="input input-bordered w-full"
                                    value=(self.invoice_number_format) {}
                            },
                        ))
                        (label_newline_hint(
                            "Invoice PDF template (Typst + Minijinja)",
                            Some(INVOICE_PDF_TEMPLATE_HINT),
                            html! {
                                textarea
                                    id="invoice-pdf-template-field"
                                    name=(AccountingPreferencesFormField::InvoicePdfTemplate.html_name())
                                    rows="16"
                                    class="textarea textarea-bordered w-full font-mono text-sm min-h-48" {
                                    (self.invoice_pdf_template)
                                }
                                textarea id="default-invoice-pdf-template" hidden readonly {
                                    (DEFAULT_INVOICE_PDF_TEMPLATE)
                                }
                                div class="flex justify-end gap-2 mt-2" {
                                    button type="button" class="btn btn-ghost btn-sm"
                                        onclick="if (confirm('This will overwrite the template in the field with the default example template. Continue?')) { document.getElementById('invoice-pdf-template-field').value = document.getElementById('default-invoice-pdf-template').value; }" {
                                        "Use default template"
                                    }
                                    div class="fk-modal-host" {
                                        (PreEscaped(format!(
                                            r#"<button type="button" class="btn btn-outline btn-sm" hx-post="/finance-invoices/invoice-pdf-preview" hx-target="{}" hx-swap="{}" hx-include="closest form" hx-push-url="false">Preview sample PDF</button>"#,
                                            escape_attr(HTMX_TARGET_BODY_MODAL),
                                            escape_attr(HTMX_SWAP_BODY_MODAL),
                                        )))
                                    }
                                }
                            },
                        ))
                        (self.addon_inputs)
                    },
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save Preferences",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }
}

impl RenderAppPane for AccountingPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(&crate::routes::AccountingPreferencesRouteTag.url(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for AccountingPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Accounting Preferences — Uniquity",
            chrome,
            self.body(),
            &crate::routes::AccountingPreferencesRouteTag.url(),
        )
    }
}
