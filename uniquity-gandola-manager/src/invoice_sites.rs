//! Sites many-to-many on draft invoices (deployment-specific).

use std::collections::HashMap;

use async_trait::async_trait;
use maud::{Markup, html};
use sea_orm::DatabaseConnection;
use sea_orm::sea_query::{Expr, SimpleExpr};

use lariv_rs::components::label;
use lariv_rs::html_form::{CsrfToken, FormCtx, HtmlForm, UrlencodedFields};
use lariv_rs::plugins::finance_invoices::draft_form_addon::DraftInvoiceFormAddon;
use lariv_rs::plugins::finance_invoices::hub_filter_addon::{
    HubQueryParams, InvoiceHubFilterAddon,
};
use lariv_rs::plugins::finance_invoices::hub_table_addon::InvoiceHubTableAddon;
use lariv_rs::plugins::finance_invoices::invoice_pdf_addon::InvoicePdfContextAddon;
use serde_json::{Value, json};

use crate::forms::{
    DraftInvoiceSitesForm, DraftInvoiceSitesFormField, InvoiceHubSitesFilterForm,
    InvoiceHubSitesFilterFormField,
};
use crate::routes::SiteDetailRouteTag;
use crate::scope::{
    load_sites_for_invoice, site_items_for_invoice, site_items_from_ids, site_name,
    site_names_for_invoices, sync_invoice_sites,
};

pub static INVOICE_SITES_ADDON: InvoiceSitesAddon = InvoiceSitesAddon;

pub struct InvoiceSitesAddon;

pub fn register() {
    lariv_rs::plugins::finance_invoices::draft_form_addon::register_draft_invoice_form_addon(
        &INVOICE_SITES_ADDON,
    );
    lariv_rs::plugins::finance_invoices::hub_table_addon::register_invoice_hub_table_addon(
        &INVOICE_SITES_ADDON,
    );
    lariv_rs::plugins::finance_invoices::hub_filter_addon::register_invoice_hub_filter_addon(
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
            &FormCtx::form::<DraftInvoiceSitesForm>(CsrfToken::current())
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

    fn bulk_has_values(&self, fields: &UrlencodedFields) -> bool {
        match fields.deserialize::<DraftInvoiceSitesForm>() {
            Ok(form) => !form.sites.is_empty(),
            Err(_) => false,
        }
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
        draft_invoice_ids: &[i64],
    ) -> HashMap<i64, String> {
        site_names_for_invoices(db, draft_invoice_ids).await
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
        draft_invoice_id_sql: &str,
    ) -> Option<SimpleExpr> {
        let raw = params.get("Sites")?;
        match raw.parse::<i64>() {
            Ok(site_id) if site_id > 0 => Some(sites_linked_to(draft_invoice_id_sql, site_id)),
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

/// Core hub passes one of a few trusted draft-id expressions. Reject anything else
/// before it is interpolated into SQL.
fn draft_id_sql_is_safe(sql: &str) -> bool {
    if sql == "draft_invoices.id" || sql == "posted_invoices.draft_invoice_id" {
        return true;
    }
    let Some(rest) =
        sql.strip_prefix("(SELECT pi.draft_invoice_id FROM posted_invoices pi WHERE pi.id = ")
    else {
        return false;
    };
    let Some(table) = rest.strip_suffix(".posted_invoice_id)") else {
        return false;
    };
    !table.is_empty() && table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn sites_linked_to(draft_invoice_id_sql: &str, site_id: i64) -> SimpleExpr {
    if !draft_id_sql_is_safe(draft_invoice_id_sql) {
        return Expr::cust("FALSE");
    }
    Expr::cust_with_values(
        format!(
            "EXISTS (SELECT 1 FROM site_invoices si WHERE si.draft_invoice_id = ({draft_invoice_id_sql}) AND si.site_id = $1)"
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
        draft_invoice_id: i64,
    ) -> Result<Value, String> {
        let sites = load_sites_for_invoice(db, draft_invoice_id).await;
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

    fn predicate_sql(query: &str, draft_sql: &str) -> Option<(String, Vec<Value>)> {
        let expr = InvoiceSitesAddon.sql_predicate(&HubQueryParams::new(query), draft_sql)?;
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
        let via =
            lariv_rs::plugins::finance_invoices::hub_filter_addon::draft_invoice_id_sql_via_posted(
                "paid_invoices",
            );
        let (sql, _) = predicate_sql("Sites=7", &via).expect("predicate");
        assert!(sql.contains("paid_invoices.posted_invoice_id"));
        let (posted, _) =
            predicate_sql("Sites=7", "posted_invoices.draft_invoice_id").expect("posted");
        assert!(posted.contains("posted_invoices.draft_invoice_id"));
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
