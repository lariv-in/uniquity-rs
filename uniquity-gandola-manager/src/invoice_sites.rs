//! Sites many-to-many on draft invoices (deployment-specific).

use async_trait::async_trait;
use maud::{Markup, html};
use sea_orm::DatabaseConnection;

use lariv_rs::components::label;
use lariv_rs::html_form::{FormCtx, HtmlForm, UrlencodedFields};
use lariv_rs::plugins::finance_invoices::draft_form_addon::DraftInvoiceFormAddon;
use lariv_rs::plugins::finance_invoices::invoice_pdf_addon::InvoicePdfContextAddon;
use serde_json::{Value, json};

use crate::forms::{DraftInvoiceSitesForm, DraftInvoiceSitesFormField};
use crate::routes::SiteDetailRouteTag;
use crate::scope::{
    load_sites_for_invoice, site_items_for_invoice, site_items_from_ids, sync_invoice_sites,
};

pub static INVOICE_SITES_ADDON: InvoiceSitesAddon = InvoiceSitesAddon;

pub struct InvoiceSitesAddon;

pub fn register() {
    lariv_rs::plugins::finance_invoices::draft_form_addon::register_draft_invoice_form_addon(
        &INVOICE_SITES_ADDON,
    );
    lariv_rs::plugins::finance_invoices::invoice_pdf_addon::register_invoice_pdf_context_addon(
        &INVOICE_SITES_ADDON,
    );
}

#[async_trait]
impl DraftInvoiceFormAddon for InvoiceSitesAddon {
    fn id(&self) -> &'static str {
        "uniquity-site-invoices"
    }

    async fn render_inputs(
        &self,
        db: &DatabaseConnection,
        draft_id: Option<i64>,
        posted: Option<&UrlencodedFields>,
    ) -> Markup {
        let items = if let Some(posted) = posted {
            match posted.deserialize::<DraftInvoiceSitesForm>() {
                Ok(form) => site_items_from_ids(db, &form.sites).await,
                Err(_) => match draft_id {
                    Some(id) => site_items_for_invoice(db, id).await,
                    None => Vec::new(),
                },
            }
        } else if let Some(id) = draft_id {
            site_items_for_invoice(db, id).await
        } else {
            Vec::new()
        };
        DraftInvoiceSitesForm::render_inputs(
            &FormCtx::form::<DraftInvoiceSitesForm>()
                .m2m(DraftInvoiceSitesFormField::Sites, &items),
        )
    }

    async fn render_detail(&self, db: &DatabaseConnection, draft_id: i64) -> Markup {
        let sites = load_sites_for_invoice(db, draft_id).await;
        if sites.is_empty() {
            return Markup::default();
        }
        html! {
            (label("Sites", html! {
                div class="flex flex-col gap-1" {
                    @for s in &sites {
                        a class="link" href=(SiteDetailRouteTag::new(s.id).url()) { (s.name) }
                    }
                }
            }))
        }
    }

    async fn save(
        &self,
        db: &DatabaseConnection,
        draft_id: i64,
        fields: &UrlencodedFields,
    ) -> Result<(), String> {
        let form: DraftInvoiceSitesForm = fields.deserialize().map_err(|e| e.to_string())?;
        sync_invoice_sites(db, draft_id, &form.sites).await
    }
}

#[async_trait]
impl InvoicePdfContextAddon for InvoiceSitesAddon {
    fn id(&self) -> &'static str {
        "uniquity-site-invoices"
    }

    async fn extra_context(
        &self,
        db: &DatabaseConnection,
        draft_invoice_id: i64,
    ) -> Result<Value, String> {
        let sites = load_sites_for_invoice(db, draft_invoice_id).await;
        Ok(json!({
            "Sites": sites
                .into_iter()
                .map(|s| {
                    json!({
                        "ID": s.id,
                        "Name": s.name,
                        "Address": s.address.unwrap_or_default(),
                    })
                })
                .collect::<Vec<_>>(),
        }))
    }

    fn sample_extra_context(&self) -> Value {
        json!({
            "Sites": [{
                "ID": 1,
                "Name": "Sample Site",
                "Address": "Plot 12, Industrial Area",
            }]
        })
    }
}
