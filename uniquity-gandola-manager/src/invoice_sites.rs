//! Sites many-to-many on draft and posted invoices (deployment-specific).

use std::collections::HashMap;

use async_trait::async_trait;
use maud::{Markup, html};
use sea_orm::DatabaseConnection;
use sea_orm::sea_query::{Expr, SimpleExpr};

use lariv_core::components::label;
use lariv_core::html_form::{CsrfToken, FormCtx, HtmlForm, UrlencodedFields};
use lariv_plugin_finance_invoices::draft_form_addon::DraftInvoiceFormAddon;
use lariv_plugin_finance_invoices::hub_filter_addon::{HubQueryParams, InvoiceHubFilterAddon};
use lariv_plugin_finance_invoices::hub_table_addon::InvoiceHubTableAddon;
use lariv_plugin_finance_invoices::invoice_addon_key::InvoiceAddonKey;
use lariv_plugin_finance_invoices::invoice_pdf_addon::InvoicePdfContextAddon;
use serde_json::{Value, json};

use crate::forms::{
    DraftInvoiceSitesForm, DraftInvoiceSitesFormField, InvoiceHubSitesFilterForm,
    InvoiceHubSitesFilterFormField,
};
use crate::routes::SiteDetailRouteTag;
use crate::scope::{
    copy_sites_draft_to_posted, copy_sites_posted_to_draft, load_sites_for_draft_invoice,
    load_sites_for_posted_invoice, site_items_for_invoice, site_items_from_ids, site_name,
    site_names_for_draft_invoices, site_names_for_posted_invoices, sync_invoice_sites,
};

pub static INVOICE_SITES_ADDON: InvoiceSitesAddon = InvoiceSitesAddon;

pub struct InvoiceSitesAddon;

pub fn register() {
    lariv_plugin_finance_invoices::draft_form_addon::register_draft_invoice_form_addon(
        &INVOICE_SITES_ADDON,
    );
    lariv_plugin_finance_invoices::hub_table_addon::register_invoice_hub_table_addon(
        &INVOICE_SITES_ADDON,
    );
    lariv_plugin_finance_invoices::hub_filter_addon::register_invoice_hub_filter_addon(
        &INVOICE_SITES_ADDON,
    );
    lariv_plugin_finance_invoices::invoice_pdf_addon::register_invoice_pdf_context_addon(
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
            &FormCtx::form::<DraftInvoiceSitesForm>(CsrfToken::current())
                .m2m(DraftInvoiceSitesFormField::Sites, &items),
        )
    }

    async fn render_detail(&self, db: &DatabaseConnection, draft_id: i64) -> Markup {
        let sites = load_sites_for_draft_invoice(db, draft_id).await;
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

    fn bulk_has_values(&self, fields: &UrlencodedFields) -> bool {
        match fields.deserialize::<DraftInvoiceSitesForm>() {
            Ok(form) => !form.sites.is_empty(),
            Err(_) => false,
        }
    }

    async fn on_posted(
        &self,
        db: &sea_orm::DatabaseTransaction,
        draft_id: i64,
        posted_id: i64,
    ) -> Result<(), String> {
        copy_sites_draft_to_posted(db, draft_id, posted_id).await
    }

    async fn on_new_draft_from_posted(
        &self,
        db: &sea_orm::DatabaseTransaction,
        posted_id: i64,
        new_draft_id: i64,
    ) -> Result<(), String> {
        copy_sites_posted_to_draft(db, posted_id, new_draft_id).await
    }
}

#[async_trait]
impl InvoiceHubTableAddon for InvoiceSitesAddon {
    fn id(&self) -> &'static str {
        "uniquity-site-invoices"
    }

    fn column_key(&self) -> &'static str {
        "Sites"
    }

    fn column_label(&self) -> &'static str {
        "Sites"
    }

    async fn cell_values(
        &self,
        db: &DatabaseConnection,
        keys: &[InvoiceAddonKey],
    ) -> HashMap<InvoiceAddonKey, String> {
        let mut out = HashMap::new();
        let draft_ids: Vec<i64> = keys
            .iter()
            .filter_map(|k| match k {
                InvoiceAddonKey::Draft(id) => Some(*id),
                InvoiceAddonKey::Posted(_) => None,
            })
            .collect();
        let posted_ids: Vec<i64> = keys
            .iter()
            .filter_map(|k| match k {
                InvoiceAddonKey::Posted(id) => Some(*id),
                InvoiceAddonKey::Draft(_) => None,
            })
            .collect();
        for (id, name) in site_names_for_draft_invoices(db, &draft_ids).await {
            out.insert(InvoiceAddonKey::Draft(id), name);
        }
        for (id, name) in site_names_for_posted_invoices(db, &posted_ids).await {
            out.insert(InvoiceAddonKey::Posted(id), name);
        }
        out
    }
}

#[async_trait]
impl InvoiceHubFilterAddon for InvoiceSitesAddon {
    fn id(&self) -> &'static str {
        "uniquity-site-invoices"
    }

    async fn render_inputs(&self, db: &DatabaseConnection, params: &HubQueryParams<'_>) -> Markup {
        let raw = params.get("Sites").unwrap_or_default();
        let site_id = raw.parse::<i64>().ok().filter(|id| *id > 0);
        let (value, display) = match site_id {
            Some(id) => (id.to_string(), site_name(db, id).await),
            None => (String::new(), String::new()),
        };
        render_site_filter(&value, &display)
    }

    fn sql_predicate(
        &self,
        params: &HubQueryParams<'_>,
        invoice_id_sql: &str,
    ) -> Option<SimpleExpr> {
        let raw = params.get("Sites")?;
        match raw.parse::<i64>() {
            Ok(site_id) if site_id > 0 => Some(sites_linked_to(invoice_id_sql, site_id)),
            Ok(_) => None,
            Err(_) => Some(Expr::cust("FALSE")),
        }
    }
}

fn render_site_filter(site_id: &str, display: &str) -> Markup {
    InvoiceHubSitesFilterForm::render_inputs(
        &FormCtx::form::<InvoiceHubSitesFilterForm>(CsrfToken::current())
            .value(InvoiceHubSitesFilterFormField::Sites, site_id)
            .display(InvoiceHubSitesFilterFormField::Sites, display),
    )
}

/// Core hub passes one of a few trusted invoice-id expressions. Reject anything else
/// before it is interpolated into SQL.
fn invoice_id_sql_is_safe(sql: &str) -> bool {
    matches!(
        sql,
        "draft_invoices.id"
            | "posted_invoices.id"
            | "cancelled_invoices.posted_invoice_id"
            | "paid_invoices.posted_invoice_id"
            | "partially_paid_invoices.posted_invoice_id"
    )
}

fn sites_linked_to(invoice_id_sql: &str, site_id: i64) -> SimpleExpr {
    if !invoice_id_sql_is_safe(invoice_id_sql) {
        return Expr::cust("FALSE");
    }
    let link_col = if invoice_id_sql == "draft_invoices.id" {
        "draft_invoice_id"
    } else {
        "posted_invoice_id"
    };
    Expr::cust_with_values(
        format!(
            "EXISTS (SELECT 1 FROM site_invoices si WHERE si.{link_col} = ({invoice_id_sql}) AND si.site_id = $1)"
        ),
        [site_id],
    )
}

#[async_trait]
impl InvoicePdfContextAddon for InvoiceSitesAddon {
    fn id(&self) -> &'static str {
        "uniquity-site-invoices"
    }

    async fn extra_context(
        &self,
        db: &DatabaseConnection,
        key: InvoiceAddonKey,
    ) -> Result<Value, String> {
        let sites = match key {
            InvoiceAddonKey::Draft(id) => load_sites_for_draft_invoice(db, id).await,
            InvoiceAddonKey::Posted(id) => load_sites_for_posted_invoice(db, id).await,
        };
        Ok(json!({
            "Sites": sites
                .into_iter()
                .map(|s| {
                    json!({
                        "ID": s.id,
                        "SiteId": s.site_id.unwrap_or_default(),
                        "Name": s.name,
                        "Address": s.address.unwrap_or_default(),
                        "Remarks": s.remarks.unwrap_or_default(),
                    })
                })
                .collect::<Vec<_>>(),
        }))
    }

    fn sample_extra_context(&self) -> Value {
        json!({
            "Sites": [{
                "ID": 1,
                "SiteId": "SITE-001",
                "Name": "Sample Site",
                "Address": "Plot 12, Industrial Area",
                "Remarks": "Access from north gate",
            }]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::sea_query::{PostgresQueryBuilder, Query, Value};

    fn predicate_sql(query: &str, invoice_sql: &str) -> Option<(String, Vec<Value>)> {
        let expr = InvoiceSitesAddon.sql_predicate(&HubQueryParams::new(query), invoice_sql)?;
        let (sql, values) = Query::select()
            .expr(expr)
            .to_owned()
            .build(PostgresQueryBuilder);
        Some((sql, values.into_iter().collect()))
    }

    #[test]
    fn sites_filter_is_inactive_without_a_value() {
        assert!(predicate_sql("", "draft_invoices.id").is_none());
        assert!(predicate_sql("Sites=", "draft_invoices.id").is_none());
        assert!(predicate_sql("Sites=%20", "draft_invoices.id").is_none());
        assert!(predicate_sql("Number=1", "draft_invoices.id").is_none());
    }

    #[test]
    fn sites_filter_binds_the_selected_site_id() {
        let (sql, values) = predicate_sql("Sites=42", "draft_invoices.id").expect("predicate");
        assert!(sql.contains("site_invoices"));
        assert!(sql.contains("draft_invoices.id"));
        assert!(sql.contains("si.draft_invoice_id"));
        assert!(sql.contains("si.site_id = $1"));
        assert!(!sql.contains("42"));
        assert_eq!(values, vec![Value::BigInt(Some(42))]);
    }

    #[test]
    fn sites_filter_rejects_a_non_numeric_value() {
        let (sql, values) = predicate_sql("Sites=North", "draft_invoices.id").expect("predicate");
        assert!(sql.contains("FALSE"));
        assert!(values.is_empty());
    }

    #[test]
    fn sites_filter_covers_posted_and_settlement_rows() {
        let (sql, _) = predicate_sql("Sites=7", "paid_invoices.posted_invoice_id").expect("paid");
        assert!(sql.contains("paid_invoices.posted_invoice_id"));
        assert!(sql.contains("si.posted_invoice_id"));
        let (posted, _) = predicate_sql("Sites=7", "posted_invoices.id").expect("posted");
        assert!(posted.contains("posted_invoices.id"));
        assert!(posted.contains("si.posted_invoice_id"));
    }

    #[test]
    fn sites_filter_rejects_untrusted_draft_id_sql() {
        let (sql, values) =
            predicate_sql("Sites=7", "draft_invoices.id) OR (TRUE").expect("predicate");
        assert!(sql.contains("FALSE"));
        assert!(values.is_empty());
        assert!(!sql.contains("site_invoices"));
    }

    #[test]
    fn sites_filter_input_is_a_site_picker() {
        let html = render_site_filter("42", "North Gate").into_string();
        assert!(html.contains(r#"name="Sites""#));
        assert!(html.contains("North Gate"));
        assert!(html.contains("/gandola/sites/pick-site"));
        assert!(html.contains("Select site"));
    }
}
