//! Rune sandbox bindings for Gandola site purchase-order workflows.

use std::sync::Arc;

use lariv_rs::{
    plugins::finance_invoices::logic::{
        default_payment_term_lines_json, parse_payment_term_lines_json,
    },
    plugins::finance_invoices::{PaymentTermAmountKind, PaymentTermDateKind},
    rune_env::{NativeBinding, RuneEnvCapability, RuneEnvCtx, RuneEnvRegistrar},
};
use serde::Deserialize;
use serde_json::json;

use crate::forms::PurchaseOrderForm;
use crate::po_lines::PoLinePending;

/// Registers site invoicing and purchase-order helpers onto the assistant Rune environment.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl RuneEnvRegistrar for Hook {
    fn register_rune_env(self, rune_env: &mut RuneEnvCapability) {
        rune_env.register_contextual(
            "find_site",
            "find_site(#{ site_id?: int|string, name?: string, site_name?: string }) -> #{ id, site_id?, name, customer_id, address?, remarks?, status } — int site_id is the primary key; string site_id / name / site_name match name or string site_id",
            |_ctx| NativeBinding::Function(Arc::new(find_site)),
        );
        rune_env.register_contextual(
            "list_site_purchase_orders",
            "list_site_purchase_orders(#{ site_id?: int|string, name?: string, site_name?: string }) -> #{ site: SiteSummary, purchase_orders: [#{ id, number, date, customer_id, already_invoiced, existing_invoice_id?, lines: [...] }] }",
            |_ctx| NativeBinding::Function(Arc::new(list_site_purchase_orders)),
        );
        rune_env.register_contextual(
            "create_invoices_for_site",
            "create_invoices_for_site(#{ site_id?: int|string, site_name?: string, name?: string, timezone?: string, dry_run?: bool }) -> #{ site_id, site_name, dry_run, created: [InvoiceAction], skipped: [InvoiceAction], errors: [InvoiceAction] }",
            |_ctx| NativeBinding::Function(Arc::new(create_invoices_for_site)),
        );
        rune_env.register_contextual(
            "link_site_invoice",
            "link_site_invoice(#{ site_id: int, invoice_id?: int, draft_invoice_id?: int }) -> #{ site_id, invoice_id, linked: true }",
            |_ctx| NativeBinding::Function(Arc::new(link_site_invoice)),
        );
        rune_env.register_contextual(
            "unlink_site_invoice",
            "unlink_site_invoice(#{ site_id: int, invoice_id?: int, draft_invoice_id?: int }) -> #{ site_id, invoice_id, unlinked: true }",
            |_ctx| NativeBinding::Function(Arc::new(unlink_site_invoice)),
        );
        rune_env.register_contextual(
            "create_purchase_order",
            "create_purchase_order(#{ site_id: int|string, name?: string, site_name?: string, customer_id?: int, number: string, date: string, file_id: int, billing_address?: string, shipping_address?: string, timezone?: string, payment_term_lines?: [#{ date_kind: \"absolute\"|\"relative\"|\"relative_delivery\", amount_kind: \"absolute\"|\"relative\", due_date?: string, due_duration?: string, amount?: number|string, amount_percentage?: number|string }], lines: [#{ item_code?: string, description?: string, unit?: string, delivery_date: string, quantity: number|string, rate: number|string }] }) -> #{ id, number, site_id, customer_id, file_id?}",
            |_ctx| NativeBinding::Function(Arc::new(create_purchase_order)),
        );
        rune_env.register_contextual(
            "update_purchase_order",
            "update_purchase_order(#{ id: int, site_id?: int|string, name?: string, site_name?: string, customer_id?: int, number?: string, date?: string, file_id?: int, billing_address?: string, shipping_address?: string, timezone?: string, payment_term_lines?: [#{ date_kind: \"absolute\"|\"relative\"|\"relative_delivery\", amount_kind: \"absolute\"|\"relative\", due_date?: string, due_duration?: string, amount?: number|string, amount_percentage?: number|string }], lines?: [#{ item_code?: string, description?: string, unit?: string, delivery_date: string, quantity: number|string, rate: number|string }] }) -> #{ id, number, site_id, customer_id, file_id? }  // omitted fields keep their current values",
            |_ctx| NativeBinding::Function(Arc::new(update_purchase_order)),
        );
        rune_env.register_contextual(
            "create_site",
            "create_site(#{ name: string, customer_id: int, site_id?: string, status?: string, start_date?: string, end_date?: string, address?: string, remarks?: string, gandolas?: [int], invoices?: [int], purchase_orders?: [int] }) -> #{ id, site_id?, name, customer_id, address?, remarks?, status }",
            |_ctx| NativeBinding::Function(Arc::new(create_site)),
        );
        rune_env.register_contextual(
            "update_site",
            "update_site(#{ id: int, name?: string, customer_id?: int, site_id?: string, status?: string, start_date?: string, end_date?: string, address?: string, remarks?: string, gandolas?: [int], invoices?: [int], purchase_orders?: [int] }) -> #{ id, site_id?, name, customer_id, address?, remarks?, status }  // omitted fields keep their current values",
            |_ctx| NativeBinding::Function(Arc::new(update_site)),
        );
    }
}

fn find_site(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed: SiteLookupArgs = parse_object_args(args, "find_site")?;
    let db = ctx.db.clone();
    let site = block_on_async(async move {
        crate::invoice_site_pos::find_site(&db, parsed.lookup_pk(), parsed.lookup_text()).await
    })?;
    lariv_rs::rune_env::json_to_rune(json!(crate::invoice_site_pos::site_summary(&site)))
}

fn list_site_purchase_orders(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed: SiteLookupArgs = parse_object_args(args, "list_site_purchase_orders")?;
    let db = ctx.db.clone();
    let (site, purchase_orders) = block_on_async(async move {
        let site =
            crate::invoice_site_pos::find_site(&db, parsed.lookup_pk(), parsed.lookup_text())
                .await?;
        let (_, pos) = crate::invoice_site_pos::list_site_purchase_orders(&db, site.id).await?;
        Ok::<_, String>((site, pos))
    })?;
    lariv_rs::rune_env::json_to_rune(json!({
        "site": crate::invoice_site_pos::site_summary(&site),
        "purchase_orders": purchase_orders,
    }))
}

fn create_invoices_for_site(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed: CreateInvoicesArgs = parse_object_args(args, "create_invoices_for_site")?;
    let timezone = parsed.timezone.clone().unwrap_or_else(|| "UTC".to_string());
    let site_text = parsed.lookup_text().map(str::to_string);
    let site_pk = parsed.lookup_pk();
    let dry_run = parsed.dry_run.unwrap_or(false);
    let db = ctx.db.clone();
    let result = block_on_async(async move {
        crate::invoice_site_pos::create_invoices_for_site(
            &db,
            site_pk,
            site_text.as_deref(),
            &timezone,
            dry_run,
        )
        .await
    })?;
    lariv_rs::rune_env::json_to_rune(json!(result))
}

fn link_site_invoice(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed: LinkSiteInvoiceArgs = parse_object_args(args, "link_site_invoice")?;
    let site_id = parsed.site_id;
    let invoice_id = parsed.invoice_id_for("link_site_invoice")?;
    let db = ctx.db.clone();
    block_on_async(async move { crate::scope::link_site_invoice(&db, site_id, invoice_id).await })?;
    lariv_rs::rune_env::json_to_rune(json!({
        "site_id": site_id,
        "invoice_id": invoice_id,
        "linked": true,
    }))
}

fn create_purchase_order(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed: CreatePurchaseOrderArgs = parse_object_args(args, "create_purchase_order")?;
    let site_pk = parsed.lookup_pk();
    let site_text = parsed.lookup_text().map(str::to_string);
    if site_pk.is_none()
        && site_text
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_none()
    {
        return Err("create_purchase_order requires a non-empty site_id".into());
    }
    if parsed.file_id <= 0 {
        return Err("create_purchase_order requires file_id".into());
    }
    let (mut form, timezone) = parsed.into_form()?;
    if form.number.trim().is_empty() {
        return Err("create_purchase_order requires number".into());
    }
    if form.date.trim().is_empty() {
        return Err("create_purchase_order requires date".into());
    }
    let lines: Vec<PoLinePending> = serde_json::from_str(&form.po_lines_json)
        .map_err(|e| format!("invalid create_purchase_order lines: {e}"))?;
    if lines.is_empty() {
        return Err("create_purchase_order requires at least one line".into());
    }

    let db = ctx.db.clone();
    let result = block_on_async(async move {
        let site = crate::invoice_site_pos::find_site(&db, site_pk, site_text.as_deref()).await?;
        if form.customer_id <= 0 {
            form.customer_id = site.customer_id;
        }
        form.site_id = site.id;
        let saved = crate::po_persist::persist_new_purchase_order(&db, &form, &timezone).await?;
        Ok::<_, String>(json!({
            "id": saved.id,
            "number": saved.number,
            "site_id": saved.site_id,
            "customer_id": saved.customer_id,
            "file_id": saved.file_id,
        }))
    })?;

    lariv_rs::rune_env::json_to_rune(result)
}

fn update_purchase_order(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed: UpdatePurchaseOrderArgs = parse_object_args(args, "update_purchase_order")?;
    if parsed.id <= 0 {
        return Err("update_purchase_order requires a positive id".into());
    }
    if parsed.lines.as_ref().is_some_and(|lines| lines.is_empty()) {
        return Err("update_purchase_order requires at least one line".into());
    }
    let po_id = parsed.id;
    let site_pk = parsed.lookup_pk();
    let site_text = parsed.lookup_text().map(str::to_string);
    let db = ctx.db.clone();
    let result = block_on_async(async move {
        use crate::entities::purchase_order::Entity as PurchaseOrderEntity;
        use sea_orm::EntityTrait;

        let existing = PurchaseOrderEntity::find_by_id(po_id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("purchase order {po_id} not found"))?;
        let timezone = parsed
            .timezone
            .clone()
            .unwrap_or_else(|| "Asia/Kolkata".to_string());
        let customer_id_omitted = parsed.customer_id.is_none();
        let mut form =
            crate::po_persist::purchase_order_form_from_model(&db, &existing, &timezone).await;
        parsed.apply_to_form(&mut form)?;
        if site_pk.is_some()
            || site_text
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_some()
        {
            let site =
                crate::invoice_site_pos::find_site(&db, site_pk, site_text.as_deref()).await?;
            form.site_id = site.id;
            if customer_id_omitted {
                form.customer_id = site.customer_id;
            }
        }
        let saved =
            crate::po_persist::persist_updated_purchase_order(&db, &existing, &form, &timezone)
                .await?;
        Ok::<_, String>(json!({
            "id": saved.id,
            "number": saved.number,
            "site_id": saved.site_id,
            "customer_id": saved.customer_id,
            "file_id": saved.file_id,
        }))
    })?;
    lariv_rs::rune_env::json_to_rune(result)
}

fn unlink_site_invoice(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed: LinkSiteInvoiceArgs = parse_object_args(args, "unlink_site_invoice")?;
    let site_id = parsed.site_id;
    let invoice_id = parsed.invoice_id_for("unlink_site_invoice")?;
    let db = ctx.db.clone();
    block_on_async(
        async move { crate::scope::unlink_site_invoice(&db, site_id, invoice_id).await },
    )?;
    lariv_rs::rune_env::json_to_rune(json!({
        "site_id": site_id,
        "invoice_id": invoice_id,
        "unlinked": true,
    }))
}

fn create_site(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed: CreateSiteArgs = parse_object_args(args, "create_site")?;
    let gandolas = parsed.gandolas.clone().unwrap_or_default();
    let invoices = parsed.invoices.clone().unwrap_or_default();
    let purchase_orders = parsed.purchase_orders.clone().unwrap_or_default();
    let fields = parsed.into_fields()?;
    let db = ctx.db.clone();
    let saved = block_on_async(async move {
        crate::site_persist::persist_new_site(&db, &fields, &gandolas, &invoices, &purchase_orders)
            .await
    })?;
    lariv_rs::rune_env::json_to_rune(json!(crate::invoice_site_pos::site_summary(&saved)))
}

fn update_site(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed: UpdateSiteArgs = parse_object_args(args, "update_site")?;
    if parsed.id <= 0 {
        return Err("update_site requires a positive id".into());
    }
    let db = ctx.db.clone();
    let saved = block_on_async(async move {
        use crate::entities::site::Entity as SiteEntity;
        use crate::scope::opt_string;
        use sea_orm::EntityTrait;

        let existing = SiteEntity::find_by_id(parsed.id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("site {} not found", parsed.id))?;
        let name = parsed
            .name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(existing.name.as_str());
        let customer_id = parsed.customer_id.unwrap_or(existing.customer_id);
        let status = match parsed.status.as_deref() {
            Some(raw) => crate::site_persist::parse_site_status(raw)?.unwrap_or(existing.status),
            None => existing.status,
        };
        let start_date = match parsed.start_date.as_deref() {
            Some(raw) => crate::site_persist::parse_optional_date(raw)?,
            None => existing.start_date,
        };
        let end_date = match parsed.end_date.as_deref() {
            Some(raw) => crate::site_persist::parse_optional_date(raw)?,
            None => existing.end_date,
        };
        let site_id = match parsed.site_id.as_ref() {
            Some(raw) => opt_string(raw.clone()),
            None => existing.site_id.clone(),
        };
        let address = match parsed.address.as_ref() {
            Some(raw) => opt_string(raw.clone()),
            None => existing.address.clone(),
        };
        let remarks = match parsed.remarks.as_ref() {
            Some(raw) => opt_string(raw.clone()),
            None => existing.remarks.clone(),
        };
        let fields = crate::site_persist::validate_site_fields(
            name,
            customer_id,
            status,
            start_date,
            end_date,
            site_id,
            address,
            remarks,
        )?;
        crate::site_persist::persist_updated_site(
            &db,
            &existing,
            &fields,
            parsed.gandolas.as_deref(),
            parsed.invoices.as_deref(),
            parsed.purchase_orders.as_deref(),
        )
        .await
    })?;
    lariv_rs::rune_env::json_to_rune(json!(crate::invoice_site_pos::site_summary(&saved)))
}

pub(crate) fn block_on_async<T, F>(fut: F) -> T
where
    T: Send,
    F: std::future::Future<Output = T> + Send,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(fut))
        }
        Ok(_) | Err(_) => std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("rune native runtime")
                        .block_on(fut)
                })
                .join()
                .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
        }),
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FlexibleSiteId {
    Id(i64),
    Code(String),
}

#[derive(Debug, Deserialize)]
struct SiteLookupArgs {
    #[serde(default)]
    site_id: Option<FlexibleSiteId>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    site_name: Option<String>,
}

impl SiteLookupArgs {
    fn lookup_pk(&self) -> Option<i64> {
        match &self.site_id {
            Some(FlexibleSiteId::Id(id)) => Some(*id),
            _ => None,
        }
    }

    fn lookup_text(&self) -> Option<&str> {
        match &self.site_id {
            Some(FlexibleSiteId::Code(code)) => {
                let trimmed = code.trim();
                if trimmed.is_empty() {
                    self.name.as_deref().or(self.site_name.as_deref())
                } else {
                    Some(trimmed)
                }
            }
            _ => self.name.as_deref().or(self.site_name.as_deref()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateInvoicesArgs {
    #[serde(default)]
    site_id: Option<FlexibleSiteId>,
    #[serde(default)]
    site_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    dry_run: Option<bool>,
}

impl CreateInvoicesArgs {
    fn lookup_pk(&self) -> Option<i64> {
        match &self.site_id {
            Some(FlexibleSiteId::Id(id)) => Some(*id),
            _ => None,
        }
    }

    fn lookup_text(&self) -> Option<&str> {
        match &self.site_id {
            Some(FlexibleSiteId::Code(code)) => {
                let trimmed = code.trim();
                if trimmed.is_empty() {
                    self.site_name.as_deref().or(self.name.as_deref())
                } else {
                    Some(trimmed)
                }
            }
            _ => self.site_name.as_deref().or(self.name.as_deref()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NumberOrString {
    Number(serde_json::Number),
    String(String),
}

impl NumberOrString {
    fn into_string(self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => s,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PoLineArg {
    #[serde(default)]
    item_code: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    unit: Option<String>,
    delivery_date: NumberOrString,
    quantity: NumberOrString,
    rate: NumberOrString,
}

#[derive(Debug, Deserialize)]
struct PaymentTermArg {
    date_kind: PaymentTermDateKind,
    #[serde(default)]
    due_date: Option<NumberOrString>,
    #[serde(default)]
    due_duration: Option<NumberOrString>,
    amount_kind: PaymentTermAmountKind,
    #[serde(default)]
    amount: Option<NumberOrString>,
    #[serde(default)]
    amount_percentage: Option<NumberOrString>,
}

#[derive(Debug, Deserialize)]
struct CreatePurchaseOrderArgs {
    site_id: FlexibleSiteId,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    site_name: Option<String>,
    #[serde(default)]
    customer_id: Option<i64>,
    number: String,
    date: String,
    file_id: i64,
    #[serde(default)]
    billing_address: Option<String>,
    #[serde(default)]
    shipping_address: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    payment_term_lines: Option<Vec<PaymentTermArg>>,
    #[serde(default)]
    lines: Vec<PoLineArg>,
}

impl CreatePurchaseOrderArgs {
    fn lookup_pk(&self) -> Option<i64> {
        match &self.site_id {
            FlexibleSiteId::Id(id) => Some(*id),
            FlexibleSiteId::Code(_) => None,
        }
    }

    fn lookup_text(&self) -> Option<&str> {
        match &self.site_id {
            FlexibleSiteId::Code(code) => {
                let trimmed = code.trim();
                if trimmed.is_empty() {
                    self.name.as_deref().or(self.site_name.as_deref())
                } else {
                    Some(trimmed)
                }
            }
            FlexibleSiteId::Id(_) => self.name.as_deref().or(self.site_name.as_deref()),
        }
    }

    fn payment_term_lines_json(&self) -> Result<String, String> {
        match &self.payment_term_lines {
            Some(lines) => payment_term_args_json(lines),
            None => Ok(default_payment_term_lines_json()),
        }
    }

    fn into_form(self) -> Result<(PurchaseOrderForm, String), String> {
        let timezone = self
            .timezone
            .clone()
            .unwrap_or_else(|| "Asia/Kolkata".to_string());
        let site_pk = self.lookup_pk().unwrap_or(0);
        let customer_id = self.customer_id.unwrap_or(0);
        let payment_term_lines_json = self.payment_term_lines_json()?;
        let file_id = self.file_id.to_string();
        let billing_address = self.billing_address.unwrap_or_default();
        let shipping_address = self.shipping_address.unwrap_or_default();
        let lines: Vec<PoLinePending> = self
            .lines
            .into_iter()
            .map(PoLineArg::into_pending)
            .collect();
        let po_lines_json = serde_json::to_string(&lines).map_err(|e| e.to_string())?;

        let form = PurchaseOrderForm {
            number: self.number,
            date: self.date,
            customer_id,
            // Replaced with the resolved site primary key before persist.
            site_id: site_pk,
            file_id,
            payment_term_lines_json,
            po_lines_json,
            billing_address,
            shipping_address,
        };
        Ok((form, timezone))
    }
}

#[derive(Debug, Deserialize)]
struct UpdatePurchaseOrderArgs {
    id: i64,
    #[serde(default)]
    site_id: Option<FlexibleSiteId>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    site_name: Option<String>,
    #[serde(default)]
    customer_id: Option<i64>,
    #[serde(default)]
    number: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    file_id: Option<i64>,
    #[serde(default)]
    billing_address: Option<String>,
    #[serde(default)]
    shipping_address: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    payment_term_lines: Option<Vec<PaymentTermArg>>,
    #[serde(default)]
    lines: Option<Vec<PoLineArg>>,
}

impl UpdatePurchaseOrderArgs {
    fn lookup_pk(&self) -> Option<i64> {
        match &self.site_id {
            Some(FlexibleSiteId::Id(id)) => Some(*id),
            _ => None,
        }
    }

    fn lookup_text(&self) -> Option<&str> {
        match &self.site_id {
            Some(FlexibleSiteId::Code(code)) => {
                let trimmed = code.trim();
                if trimmed.is_empty() {
                    self.name.as_deref().or(self.site_name.as_deref())
                } else {
                    Some(trimmed)
                }
            }
            _ => self.name.as_deref().or(self.site_name.as_deref()),
        }
    }

    fn apply_to_form(self, form: &mut PurchaseOrderForm) -> Result<(), String> {
        if let Some(number) = self.number {
            form.number = number;
        }
        if let Some(date) = self.date {
            form.date = date;
        }
        if let Some(customer_id) = self.customer_id {
            form.customer_id = customer_id;
        }
        if let Some(file_id) = self.file_id {
            form.file_id = if file_id > 0 {
                file_id.to_string()
            } else {
                String::new()
            };
        }
        if let Some(billing_address) = self.billing_address {
            form.billing_address = billing_address;
        }
        if let Some(shipping_address) = self.shipping_address {
            form.shipping_address = shipping_address;
        }
        if let Some(lines) = self.payment_term_lines {
            form.payment_term_lines_json = payment_term_args_json(&lines)?;
        }
        if let Some(lines) = self.lines {
            if lines.is_empty() {
                return Err("update_purchase_order requires at least one line".into());
            }
            let pending: Vec<PoLinePending> =
                lines.into_iter().map(PoLineArg::into_pending).collect();
            form.po_lines_json = serde_json::to_string(&pending).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

impl PoLineArg {
    fn into_pending(self) -> PoLinePending {
        PoLinePending {
            item_code: self.item_code.unwrap_or_default(),
            description: self.description.unwrap_or_default(),
            unit: self.unit.unwrap_or_default(),
            delivery_date: self.delivery_date.into_string(),
            quantity: self.quantity.into_string(),
            rate: self.rate.into_string(),
        }
    }
}

fn payment_term_args_json(lines: &[PaymentTermArg]) -> Result<String, String> {
    let rows: Vec<serde_json::Value> = lines
        .iter()
        .map(|line| {
            json!({
                "date_kind": line.date_kind.as_str(),
                "due_date": line.due_date.as_ref().map(|v| match v {
                    NumberOrString::Number(n) => n.to_string(),
                    NumberOrString::String(s) => s.clone(),
                }).unwrap_or_default(),
                "due_duration": line.due_duration.as_ref().map(|v| match v {
                    NumberOrString::Number(n) => n.to_string(),
                    NumberOrString::String(s) => s.clone(),
                }).unwrap_or_default(),
                "amount_kind": line.amount_kind.as_str(),
                "amount": line.amount.as_ref().map(|v| match v {
                    NumberOrString::Number(n) => n.to_string(),
                    NumberOrString::String(s) => s.clone(),
                }).unwrap_or_default(),
                "amount_percentage": line.amount_percentage.as_ref().map(|v| match v {
                    NumberOrString::Number(n) => n.to_string(),
                    NumberOrString::String(s) => s.clone(),
                }).unwrap_or_default(),
            })
        })
        .collect();
    let raw = serde_json::to_string(&rows).map_err(|e| e.to_string())?;
    parse_payment_term_lines_json(&raw)?;
    Ok(raw)
}

#[derive(Debug, Deserialize)]
struct CreateSiteArgs {
    name: String,
    customer_id: i64,
    #[serde(default)]
    site_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    end_date: Option<String>,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    remarks: Option<String>,
    #[serde(default)]
    gandolas: Option<Vec<i64>>,
    #[serde(default)]
    invoices: Option<Vec<i64>>,
    #[serde(default)]
    purchase_orders: Option<Vec<i64>>,
}

impl CreateSiteArgs {
    fn into_fields(self) -> Result<crate::site_persist::SiteFields, String> {
        let status = crate::site_persist::parse_site_status(self.status.as_deref().unwrap_or(""))?
            .unwrap_or_default();
        let start_date =
            crate::site_persist::parse_optional_date(self.start_date.as_deref().unwrap_or(""))?;
        let end_date =
            crate::site_persist::parse_optional_date(self.end_date.as_deref().unwrap_or(""))?;
        crate::site_persist::validate_site_fields(
            &self.name,
            self.customer_id,
            status,
            start_date,
            end_date,
            self.site_id,
            self.address,
            self.remarks,
        )
    }
}

#[derive(Debug, Deserialize)]
struct UpdateSiteArgs {
    id: i64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    customer_id: Option<i64>,
    #[serde(default)]
    site_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    end_date: Option<String>,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    remarks: Option<String>,
    #[serde(default)]
    gandolas: Option<Vec<i64>>,
    #[serde(default)]
    invoices: Option<Vec<i64>>,
    #[serde(default)]
    purchase_orders: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
struct LinkSiteInvoiceArgs {
    site_id: i64,
    #[serde(default)]
    invoice_id: Option<i64>,
    #[serde(default)]
    draft_invoice_id: Option<i64>,
}

impl LinkSiteInvoiceArgs {
    fn invoice_id_for(&self, fn_name: &str) -> Result<i64, String> {
        self.invoice_id
            .or(self.draft_invoice_id)
            .ok_or_else(|| format!("{fn_name} requires invoice_id"))
    }
}

pub(crate) fn parse_object_args<T: for<'de> Deserialize<'de>>(
    args: &[rune::Value],
    fn_name: &str,
) -> Result<T, String> {
    let value = args
        .first()
        .ok_or_else(|| format!("{fn_name} requires an object argument"))?;
    serde_json::from_value(lariv_rs::rune_env::rune_to_json(value)?)
        .map_err(|e| format!("invalid {fn_name} arguments: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use lariv_rs::plugins::filesystem::storage::{DynFilestore, UnimplementedFilestore};
    use sea_orm::DatabaseConnection;

    fn test_env_ctx<'a>(
        db: &'a DatabaseConnection,
        store: &'a Arc<DynFilestore>,
    ) -> RuneEnvCtx<'a> {
        RuneEnvCtx {
            db,
            store: Arc::clone(store),
            session_id: None,
        }
    }

    fn registered_env() -> RuneEnvCapability {
        let mut cap = RuneEnvCapability::new();
        Hook.register_rune_env(&mut cap);
        cap
    }

    #[test]
    fn registers_site_invoice_bindings() {
        let names = registered_env().all_names();
        for expected in [
            "find_site",
            "list_site_purchase_orders",
            "create_invoices_for_site",
            "link_site_invoice",
            "unlink_site_invoice",
            "create_purchase_order",
            "update_purchase_order",
            "create_site",
            "update_site",
        ] {
            assert!(
                names.iter().any(|name| name == expected),
                "expected {expected} in {names:?}"
            );
        }
        assert!(
            !names
                .iter()
                .any(|name| name == "create_purchase_order_from_pdf"),
            "pdf create binding should be removed: {names:?}"
        );
    }

    fn assert_create_invoices_rejects_missing_site() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "create_invoices_for_site")
            .map(|(_, f)| f)
            .expect("create_invoices_for_site");
        let arg = rune::to_value(HashMap::<String, rune::Value>::new()).expect("empty object");
        let err = f(&env_ctx, &[arg]).expect_err("missing site should fail");
        assert!(
            err.contains("site_id") || err.contains("site_name"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn create_invoices_for_site_rejects_missing_site_on_current_thread() {
        assert_create_invoices_rejects_missing_site();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_invoices_for_site_rejects_missing_site() {
        assert_create_invoices_rejects_missing_site();
    }

    #[test]
    fn find_site_rejects_missing_argument() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "find_site")
            .map(|(_, f)| f)
            .expect("find_site");
        let err = f(&env_ctx, &[]).expect_err("missing argument should fail");
        assert!(
            err.contains("find_site requires an object argument"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn link_site_invoice_rejects_missing_invoice_id() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "link_site_invoice")
            .map(|(_, f)| f)
            .expect("link_site_invoice");
        let mut args = HashMap::<String, rune::Value>::new();
        args.insert("site_id".into(), rune::to_value(12i64).expect("site_id"));
        let arg = rune::to_value(args).expect("object");
        let err = f(&env_ctx, &[arg]).expect_err("missing invoice_id should fail");
        assert!(err.contains("invoice_id"), "unexpected error: {err}");
    }

    fn assert_create_po_rejects_missing_lines() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "create_purchase_order")
            .map(|(_, f)| f)
            .expect("create_purchase_order");
        let mut args = HashMap::<String, rune::Value>::new();
        args.insert("site_id".into(), rune::to_value(12i64).expect("site_id"));
        args.insert("file_id".into(), rune::to_value(99i64).expect("file_id"));
        args.insert(
            "number".into(),
            rune::to_value("PO-1".to_string()).expect("number"),
        );
        args.insert(
            "date".into(),
            rune::to_value("01/01/2026".to_string()).expect("date"),
        );
        args.insert(
            "lines".into(),
            rune::to_value(Vec::<HashMap<String, rune::Value>>::new()).expect("lines"),
        );
        let arg = rune::to_value(args).expect("object");
        let err = f(&env_ctx, &[arg]).expect_err("missing lines should fail");
        assert!(err.contains("line"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn create_po_rejects_missing_lines_on_current_thread() {
        assert_create_po_rejects_missing_lines();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_po_rejects_missing_lines() {
        assert_create_po_rejects_missing_lines();
    }

    #[test]
    fn create_po_args_parse_structured_fields() {
        let raw = json!({
            "site_id": 12,
            "file_id": 99,
            "number": "PO-100",
            "date": "15/03/2026",
            "customer_id": 7,
            "billing_address": "Bill me",
            "lines": [{
                "item_code": "A1",
                "description": "Widget",
                "unit": "NOS",
                "delivery_date": "20/03/2026",
                "quantity": 2,
                "rate": "10.5"
            }],
            "payment_term_lines": [{
                "date_kind": "relative",
                "due_duration": "15d",
                "amount_kind": "relative",
                "amount_percentage": 100
            }]
        });
        let parsed: CreatePurchaseOrderArgs = serde_json::from_value(raw).expect("parse");
        let (form, tz) = parsed.into_form().expect("form");
        assert_eq!(form.number, "PO-100");
        assert_eq!(form.site_id, 12);
        assert_eq!(form.file_id, "99");
        assert_eq!(form.customer_id, 7);
        assert_eq!(form.billing_address, "Bill me");
        assert_eq!(tz, "Asia/Kolkata");
        let lines: Vec<PoLinePending> = serde_json::from_str(&form.po_lines_json).expect("lines");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].item_code, "A1");
        assert_eq!(lines[0].quantity, "2");
        assert_eq!(lines[0].rate, "10.5");
    }

    #[test]
    fn unlink_site_invoice_rejects_missing_invoice_id() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "unlink_site_invoice")
            .map(|(_, f)| f)
            .expect("unlink_site_invoice");
        let mut args = HashMap::<String, rune::Value>::new();
        args.insert("site_id".into(), rune::to_value(12i64).expect("site_id"));
        let arg = rune::to_value(args).expect("object");
        let err = f(&env_ctx, &[arg]).expect_err("missing invoice_id should fail");
        assert!(err.contains("invoice_id"), "unexpected error: {err}");
    }

    #[test]
    fn update_purchase_order_rejects_missing_id() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "update_purchase_order")
            .map(|(_, f)| f)
            .expect("update_purchase_order");
        let arg = rune::to_value(HashMap::<String, rune::Value>::new()).expect("empty object");
        let err = f(&env_ctx, &[arg]).expect_err("missing id should fail");
        assert!(err.contains("id"), "unexpected error: {err}");
    }

    #[test]
    fn update_purchase_order_rejects_non_positive_id() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "update_purchase_order")
            .map(|(_, f)| f)
            .expect("update_purchase_order");
        let mut args = HashMap::<String, rune::Value>::new();
        args.insert("id".into(), rune::to_value(0i64).expect("id"));
        let arg = rune::to_value(args).expect("object");
        let err = f(&env_ctx, &[arg]).expect_err("non-positive id should fail");
        assert!(err.contains("positive"), "unexpected error: {err}");
    }

    #[test]
    fn update_purchase_order_rejects_empty_lines() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "update_purchase_order")
            .map(|(_, f)| f)
            .expect("update_purchase_order");
        let mut args = HashMap::<String, rune::Value>::new();
        args.insert("id".into(), rune::to_value(1i64).expect("id"));
        args.insert(
            "lines".into(),
            rune::to_value(Vec::<HashMap<String, rune::Value>>::new()).expect("lines"),
        );
        let arg = rune::to_value(args).expect("object");
        let err = f(&env_ctx, &[arg]).expect_err("empty lines should fail");
        assert!(err.contains("line"), "unexpected error: {err}");
    }

    #[test]
    fn create_site_rejects_missing_name() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "create_site")
            .map(|(_, f)| f)
            .expect("create_site");
        let mut args = HashMap::<String, rune::Value>::new();
        args.insert(
            "customer_id".into(),
            rune::to_value(7i64).expect("customer_id"),
        );
        let arg = rune::to_value(args).expect("object");
        let err = f(&env_ctx, &[arg]).expect_err("missing name should fail");
        assert!(
            err.contains("name") || err.contains("invalid create_site"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn update_site_rejects_missing_id() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "update_site")
            .map(|(_, f)| f)
            .expect("update_site");
        let arg = rune::to_value(HashMap::<String, rune::Value>::new()).expect("empty object");
        let err = f(&env_ctx, &[arg]).expect_err("missing id should fail");
        assert!(err.contains("id"), "unexpected error: {err}");
    }

    #[test]
    fn update_po_args_overlay_number_and_lines() {
        let raw = json!({
            "id": 5,
            "number": "PO-200",
            "lines": [{
                "item_code": "B2",
                "description": "Bolt",
                "unit": "NOS",
                "delivery_date": "21/03/2026",
                "quantity": 3,
                "rate": "1.5"
            }]
        });
        let parsed: UpdatePurchaseOrderArgs = serde_json::from_value(raw).expect("parse");
        let mut form = PurchaseOrderForm {
            number: "PO-OLD".into(),
            date: "01/01/2026".into(),
            customer_id: 7,
            site_id: 12,
            file_id: "99".into(),
            payment_term_lines_json: default_payment_term_lines_json(),
            po_lines_json: "[]".into(),
            billing_address: String::new(),
            shipping_address: String::new(),
        };
        parsed.apply_to_form(&mut form).expect("overlay");
        assert_eq!(form.number, "PO-200");
        assert_eq!(form.date, "01/01/2026");
        assert_eq!(form.site_id, 12);
        let lines: Vec<PoLinePending> = serde_json::from_str(&form.po_lines_json).expect("lines");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].item_code, "B2");
        assert_eq!(lines[0].quantity, "3");
    }

    #[test]
    fn create_site_args_parse_fields() {
        let raw = json!({
            "name": "Yard A",
            "customer_id": 7,
            "site_id": "SITE-001",
            "status": "docs_done",
            "address": "1 Road",
            "remarks": "Gate code 12"
        });
        let parsed: CreateSiteArgs = serde_json::from_value(raw).expect("parse");
        let fields = parsed.into_fields().expect("fields");
        assert_eq!(fields.name, "Yard A");
        assert_eq!(fields.customer_id, 7);
        assert_eq!(fields.site_id.as_deref(), Some("SITE-001"));
        assert_eq!(fields.status.as_str(), "docs_done");
        assert_eq!(fields.address.as_deref(), Some("1 Road"));
        assert_eq!(fields.remarks.as_deref(), Some("Gate code 12"));
    }
}
