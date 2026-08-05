use frunk::Generic;
use maud::{Markup, html};

use uniquity_finance_accounts::routes::JournalEntryDetailRouteTag;

use lariv_rs::{
    components::{
        ButtonSubmit, FieldText, FieldTitle, FormOpts, ManyToManyItem,
        ObjectList, PaginationPage, ShellChrome, SlotCapability, SlotRegistrar,
        SwapKey, TableButtonCreate, TableColumnHeader, TablePagination, TableRow,
        button_delete, button_download_route, button_link, button_submit, container_column, container_row, data_table_list, detail,
        detail_header, field_link, field_text, field_title, form, form_hx_post_main, label_inline, pagination_pages,
        row_attr_navigate, row_attr_select, table_button_create, table_pagination,
        ButtonDeletePost, button_delete_post_route, ButtonLink, DetailHeader, FieldLink,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    picker::RenderPickerSelect,
};

use uniquity_finance_accounts::accounting_detail_menu::{
    DetailMenuNavItem, detail_sidebar_menu,
};
use uniquity_finance_accounts::templates::{
    app_scaffold, app_scaffold_with_sidebar, layout_main_content, layout_with_entity_sidebar,
    layout_with_sidebar,
};

use crate::components::{self, field_invoice_lines, fiscal_year_environment_selector};
use crate::logic::invoice_line_editor::InvoiceLineDisplayRow;
use crate::logic::payment_term_type_label;

use super::forms::{
    CancelInvoiceForm, CancelInvoiceFormField, DraftInvoiceForm, DraftInvoiceFormField,
    InvoicePreferencesForm, InvoicePreferencesFormField, PaymentBatchForm, PaymentBatchFormField,
    PaymentForm, PaymentFormField,
    PaymentPreferencesForm, PaymentPreferencesFormField, PaymentTermForm, PaymentTermFormField,
};
use super::keys::{
    InvoiceHubTableKey, PaymentBatchTableKey, PaymentTableKey, PaymentTermSelectModalKey,
    PaymentTermSelectTableKey, PaymentTermTableKey, PostedInvoiceSelectModalKey,
    PostedInvoiceSelectTableKey,
};
use super::routes::{
    CancelledInvoiceDetailRouteTag, CancelledInvoiceNewDraftRouteTag, CancelledInvoicePdfRouteTag, DraftInvoiceCreateGetRouteTag,
    DraftInvoiceDeletePostRouteTag, DraftInvoiceDetailRouteTag, DraftInvoiceEditGetRouteTag,
    DraftInvoicePdfRouteTag, DraftInvoicePostRouteTag,
    InvoiceDefaultRouteTag, PaymentBatchDetailRouteTag,
    PaymentBatchListRouteTag,
    PaymentCreateGetRouteTag, PaymentDetailRouteTag, PaymentListRouteTag,
    PaidInvoiceDetailRouteTag, PaidInvoicePdfRouteTag, PartiallyPaidInvoiceDetailRouteTag,
    PartiallyPaidInvoicePdfRouteTag,
    PaymentTermCreateGetRouteTag, PaymentTermCreatePostRouteTag, PaymentTermDeletePostRouteTag,
    PaymentTermDetailRouteTag, PaymentTermEditGetRouteTag, PaymentTermEditPostRouteTag,
    PaymentTermListRouteTag, PostedInvoiceCancelGetRouteTag, PostedInvoiceDetailRouteTag,
    PostedInvoicePdfRouteTag,
};

lariv_rs::define_register_items! {
    plugin: UniquityFinanceInvoicesTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        InvoiceHubIdx: InvoiceHubPageTag => InvoiceHubPage,
        DraftInvoiceFormIdx: DraftInvoiceFormPageTag => DraftInvoiceFormPage,
        DraftInvoiceDetailIdx: DraftInvoiceDetailPageTag => DraftInvoiceDetailPage,
        PostedInvoiceDetailIdx: PostedInvoiceDetailPageTag => PostedInvoiceDetailPage,
        PaidInvoiceDetailIdx: PaidInvoiceDetailPageTag => PaidInvoiceDetailPage,
        PartiallyPaidInvoiceDetailIdx: PartiallyPaidInvoiceDetailPageTag => PartiallyPaidInvoiceDetailPage,
        CancelledInvoiceDetailIdx: CancelledInvoiceDetailPageTag => CancelledInvoiceDetailPage,
        PaymentListIdx: PaymentListPageTag => PaymentListPage,
        PaymentFormIdx: PaymentFormPageTag => PaymentFormPage,
        PaymentDetailIdx: PaymentDetailPageTag => PaymentDetailPage,
        PaymentBatchFormIdx: PaymentBatchFormPageTag => PaymentBatchFormPage,
        PaymentBatchListIdx: PaymentBatchListPageTag => PaymentBatchListPage,
        PaymentBatchDetailIdx: PaymentBatchDetailPageTag => PaymentBatchDetailPage,
        PaymentTermListIdx: PaymentTermListPageTag => PaymentTermListPage,
        PaymentTermFormIdx: PaymentTermFormPageTag => PaymentTermFormPage,
        PaymentTermDetailIdx: PaymentTermDetailPageTag => PaymentTermDetailPage,
        CancelInvoiceIdx: CancelInvoicePageTag => CancelInvoicePage,
        InvoicePreferencesIdx: InvoicePreferencesPageTag => InvoicePreferencesPage,
        PaymentPreferencesIdx: PaymentPreferencesPageTag => PaymentPreferencesPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceInvoicesTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn render_pagination<K: SwapKey>(path_and_query: &str, number: u32, num_pages: u32) -> Markup {
    let owned = pagination_pages(path_and_query, number, num_pages, true);
    let pages: Vec<PaginationPage<'_>> = owned
        .iter()
        .map(|(ellipsis, url, push_url, active, label)| PaginationPage {
            ellipsis: *ellipsis,
            url: url.as_str(),
            push_url: *push_url,
            active: *active,
            label: label.as_str(),
        })
        .collect();
    table_pagination(TablePagination {
        pages: &pages,
        hx_target: K::SELECTOR,
    })
}

fn tab_href(tab: &str) -> String {
    lariv_rs::http::RouteQueryBuilder::new(InvoiceDefaultRouteTag)
        .query("tab", tab)
        .build()
}

fn draft_invoice_detail_menu(id: i64, number: &str, active: &str, can_edit: bool) -> Markup {
    let label = if number.is_empty() {
        format!("Draft #{id}")
    } else {
        format!("Draft {number}")
    };
    let detail_url = DraftInvoiceDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Draft Invoice Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Draft",
            url: DraftInvoiceEditGetRouteTag::new(id).url(),
            active: active == "edit",
        });
    }
    detail_sidebar_menu(
        format!("Invoice: {label}"),
        "Back to Drafts",
        tab_href("drafts"),
        &nav,
        None,
        html! {},
    )
}

fn posted_invoice_detail_menu(id: i64, number: &str) -> Markup {
    let label = if number.is_empty() {
        format!("#{id}")
    } else {
        number.to_string()
    };
    detail_sidebar_menu(
        format!("Posted invoice: {label}"),
        "Back to Posted",
        tab_href("posted"),
        &[DetailMenuNavItem {
            title: "Posted Invoice Detail",
            url: PostedInvoiceDetailRouteTag::new(id).url(),
            active: true,
        }],
        None,
        html! {},
    )
}

fn cancelled_invoice_detail_menu(id: i64, number: &str) -> Markup {
    let label = if number.is_empty() {
        format!("#{id}")
    } else {
        number.to_string()
    };
    detail_sidebar_menu(
        format!("Cancelled invoice: {label}"),
        "Back to Cancelled",
        tab_href("cancelled"),
        &[DetailMenuNavItem {
            title: "Cancelled Invoice Detail",
            url: CancelledInvoiceDetailRouteTag::new(id).url(),
            active: true,
        }],
        None,
        html! {},
    )
}

fn paid_invoice_detail_menu(id: i64, number: &str) -> Markup {
    let label = if number.is_empty() {
        format!("#{id}")
    } else {
        number.to_string()
    };
    detail_sidebar_menu(
        format!("Paid invoice: {label}"),
        "Back to Paid",
        tab_href("paid"),
        &[DetailMenuNavItem {
            title: "Paid Invoice Detail",
            url: PaidInvoiceDetailRouteTag::new(id).url(),
            active: true,
        }],
        None,
        html! {},
    )
}

fn partially_paid_invoice_detail_menu(id: i64, number: &str) -> Markup {
    let label = if number.is_empty() {
        format!("#{id}")
    } else {
        number.to_string()
    };
    detail_sidebar_menu(
        format!("Partially paid invoice: {label}"),
        "Back to Partially paid",
        tab_href("partial"),
        &[DetailMenuNavItem {
            title: "Partially Paid Invoice Detail",
            url: PartiallyPaidInvoiceDetailRouteTag::new(id).url(),
            active: true,
        }],
        None,
        html! {},
    )
}

fn payment_detail_menu(id: i64) -> Markup {
    detail_sidebar_menu(
        format!("Payment #{id}"),
        "Back to Payments",
        PaymentListRouteTag.url(),
        &[DetailMenuNavItem {
            title: "Payment Detail",
            url: PaymentDetailRouteTag::new(id).url(),
            active: true,
        }],
        None,
        html! {},
    )
}

fn payment_batch_detail_menu(id: i64) -> Markup {
    detail_sidebar_menu(
        format!("Batch #{id}"),
        "Back to Batches",
        PaymentBatchListRouteTag.url(),
        &[DetailMenuNavItem {
            title: "Batch Detail",
            url: PaymentBatchDetailRouteTag::new(id).url(),
            active: true,
        }],
        None,
        html! {},
    )
}

fn payment_batch_form_menu() -> Markup {
    detail_sidebar_menu(
        "Batch payment".to_string(),
        "Back to Batches",
        PaymentBatchListRouteTag.url(),
        &[],
        None,
        html! {},
    )
}

fn payment_term_display_label(summary: &str, id: i64) -> String {
    if summary.is_empty() {
        format!("#{id}")
    } else {
        summary.to_string()
    }
}

fn payment_term_detail_menu(id: i64, summary: &str, active: &str, can_edit: bool) -> Markup {
    let label = payment_term_display_label(summary, id);
    let detail_url = PaymentTermDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Payment Term Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Payment Term",
            url: PaymentTermEditGetRouteTag::new(id).url(),
            active: active == "edit",
        });
    }
    detail_sidebar_menu(
        format!("Payment term: {label}"),
        "Back to Payment Terms",
        PaymentTermListRouteTag.url(),
        &nav,
        None,
        html! {},
    )
}

#[derive(Clone)]
pub struct InvoiceRow {
    pub id: i64,
    pub number: String,
    pub datetime: String,
    pub status: String,
    pub detail_href: String,
    pub customer_name: String,
    pub open_balance: String,
    pub selectable: bool,
}

#[derive(Generic)]
pub struct InvoiceHubPage {
    pub invoices: ObjectList<InvoiceRow>,
    pub tab: String,
    pub path_and_query: String,
    pub fiscal_years: Vec<components::FiscalYearOption>,
    pub selected_fiscal_year_id: Option<i64>,
    pub can_edit: bool,
}

impl InvoiceHubPage {
    fn tab_link(&self, tab: &str, label: &str) -> Markup {
        use lariv_rs::components::attrs::escape_attr;
        use maud::PreEscaped;

        let active = self.tab == tab;
        let cls = if active { "tab tab-active" } else { "tab" };
        let href = tab_href(tab);
        let nav = lariv_rs::components::nav_content_attrs(&href);
        html! {
            (PreEscaped(format!(
                r#"<a class="{cls}" href="{href}"{attrs}>"#,
                cls = escape_attr(cls),
                href = escape_attr(&href),
                attrs = nav.as_string(),
            )))
            (label)
            (PreEscaped("</a>"))
        }
    }

    pub fn render_table(&self) -> Markup {
        let posted_hub = self.tab == "posted";
        let show_select = posted_hub && self.can_edit;

        let mut headers = Vec::new();
        if show_select {
            headers.push(TableColumnHeader {
                label: "",
                sort_url: None,
                push_url: true,
            });
        }
        headers.push(TableColumnHeader {
            label: "Number",
            sort_url: None,
            push_url: true,
        });
        if posted_hub {
            headers.push(TableColumnHeader {
                label: "Customer",
                sort_url: None,
                push_url: true,
            });
            headers.push(TableColumnHeader {
                label: "Open balance",
                sort_url: None,
                push_url: true,
            });
        }
        headers.push(TableColumnHeader {
            label: "Date",
            sort_url: None,
            push_url: true,
        });
        headers.push(TableColumnHeader {
            label: "Status",
            sort_url: None,
            push_url: true,
        });

        let rows: Vec<TableRow> = self
            .invoices
            .items
            .iter()
            .map(|inv| {
                let mut cells = Vec::new();
                if show_select && inv.selectable {
                    cells.push(maud::PreEscaped(format!(
                        r#"<label class="flex justify-center" @click.stop=""><input type="checkbox" class="checkbox checkbox-sm" @change="toggle({id})" :checked="!!selected['{id}']" /></label>"#,
                        id = inv.id,
                    ))
                    .into());
                } else if show_select {
                    cells.push(html! {}.into());
                }
                cells.push(field_text(FieldText {
                    value: &inv.number,
                    classes: "",
                }));
                if posted_hub {
                    cells.push(field_text(FieldText {
                        value: &inv.customer_name,
                        classes: "",
                    }));
                    cells.push(field_text(FieldText {
                        value: &inv.open_balance,
                        classes: "text-end tabular-nums",
                    }));
                }
                cells.push(field_text(FieldText {
                    value: &inv.datetime,
                    classes: "",
                }));
                cells.push(field_text(FieldText {
                    value: &inv.status,
                    classes: "",
                }));
                TableRow {
                    attrs: row_attr_navigate(&inv.detail_href),
                    cells,
                }
            })
            .collect();

        let pagination = render_pagination::<InvoiceHubTableKey>(
            &self.path_and_query,
            self.invoices.number,
            self.invoices.num_pages,
        );

        let draft_create = if self.can_edit && self.tab == "drafts" {
            table_button_create(TableButtonCreate {
                href: &DraftInvoiceCreateGetRouteTag.url(),
                ..Default::default()
            })
        } else {
            html! {}
        };

        let table = data_table_list::<InvoiceHubTableKey>(
            "Invoices",
            html! {},
            &headers,
            &rows,
            pagination.clone(),
        );

        if show_select {
            html! {
                div x-data=r#"{
                    selected: {},
                    toggle(id) {
                        const k = String(id);
                        if (this.selected[k]) delete this.selected[k];
                        else this.selected[k] = true;
                    },
                    selectedIds() {
                        return Object.keys(this.selected).filter(k => this.selected[k]);
                    },
                    paySelectedHref() {
                        const ids = this.selectedIds();
                        if (ids.length < 2) return '#';
                        return '/finance-invoices/payments/batch/create/?PostedInvoiceIDs=' + ids.join(',');
                    }
                }"#
                {
                    div class="flex flex-wrap items-center gap-2 mb-2" {
                        (draft_create)
                        a
                            class="btn btn-primary btn-sm"
                            x-bind:href="paySelectedHref()"
                            x-bind:class="selectedIds().length >= 2 ? '' : 'btn-disabled pointer-events-none opacity-50'"
                        {
                            "Pay selected"
                        }
                    }
                    (table)
                }
            }
        } else {
            let actions = if self.can_edit && self.tab == "drafts" {
                draft_create
            } else {
                html! {}
            };
            data_table_list::<InvoiceHubTableKey>(
                "Invoices",
                actions,
                &headers,
                &rows,
                pagination,
            )
        }
    }

    fn body(&self) -> Markup {
        html! {
            (container_column("", html! {
                (fiscal_year_environment_selector(&self.fiscal_years, self.selected_fiscal_year_id))
                div class="tabs tabs-boxed mb-4" {
                    (self.tab_link("drafts", "Drafts"))
                    (self.tab_link("posted", "Posted"))
                    (self.tab_link("cancelled", "Cancelled"))
                    (self.tab_link("paid", "Paid"))
                    (self.tab_link("partial", "Partially paid"))
                }
                (self.render_table())
            }))
        }
    }
}

impl RenderAppPane for InvoiceHubPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for InvoiceHubPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Finance Invoices — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct DraftInvoiceFormPage {
    pub id: i64,
    pub title: String,
    pub form: DraftInvoiceForm,
    pub action_href: String,
    pub error: Option<String>,
    pub can_edit: bool,
    pub customer_display: String,
    pub payment_term_display: String,
    pub tax_items: Vec<ManyToManyItem>,
    pub invoice_lines_preview: String,
}

impl DraftInvoiceFormPage {
    fn body(&self) -> Markup {
        html! {
            (field_title(FieldTitle { value: &self.title, classes: "" }))
            @if let Some(e) = &self.error {
                p class="text-error" { (e) }
            }
            form method="post" action=(self.action_href) {
                (DraftInvoiceForm::render_inputs(&FormCtx::form::<DraftInvoiceForm>()
                    .value(DraftInvoiceFormField::Number, &self.form.number)
                    .value(DraftInvoiceFormField::Reference, &self.form.reference)
                    .value(DraftInvoiceFormField::PaymentReference, &self.form.payment_reference)
                    .value(DraftInvoiceFormField::BankAccount, &self.form.bank_account)
                    .value(DraftInvoiceFormField::Datetime, &self.form.datetime)
                    .value(DraftInvoiceFormField::CustomerId, &self.form.customer_id.to_string())
                    .value(DraftInvoiceFormField::PaymentTermId, &self.form.payment_term_id.to_string())
                    .value(DraftInvoiceFormField::InvoiceLinesJson, &self.form.invoice_lines_json)
                    .display(DraftInvoiceFormField::CustomerId, &self.customer_display)
                    .display(DraftInvoiceFormField::PaymentTermId, &self.payment_term_display)
                    .display(DraftInvoiceFormField::InvoiceLinesJson, &self.invoice_lines_preview)
                    .m2m(DraftInvoiceFormField::Taxes, &self.tax_items)))
                (container_row("flex gap-2 mt-2", html! {
                    (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                    @if self.id > 0 && self.can_edit {
                        (button_delete(
                            DraftInvoiceDeletePostRouteTag::new(self.id),
                            "Delete Draft",
                            "Permanently delete this draft invoice?",
                        ))
                    }
                }))
            }
        }
    }

    fn sidebar(&self) -> Markup {
        if self.id > 0 {
            draft_invoice_detail_menu(self.id, &self.form.number, "edit", self.can_edit)
        } else {
            uniquity_finance_accounts::accounting_sidebar::accounting_sidebar()
        }
    }
}

impl RenderAppPane for DraftInvoiceFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.sidebar(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for DraftInvoiceFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar(&self.title, chrome, self.sidebar(), self.body())
    }
}

#[derive(Generic)]
pub struct DraftInvoiceDetailPage {
    pub id: i64,
    pub number: String,
    pub reference: String,
    pub payment_reference: String,
    pub bank_account: String,
    pub datetime: String,
    pub customer_name: String,
    pub payment_term_summary: String,
    pub tax_labels: String,
    pub line_rows: Vec<InvoiceLineDisplayRow>,
    pub can_edit: bool,
    pub error: Option<String>,
}

impl DraftInvoiceDetailPage {
    fn body(&self) -> Markup {
        let actions = html! {
            (button_download_route(DraftInvoicePdfRouteTag::new(self.id), "PDF", "btn-outline"))
            @if self.can_edit {
                (button_delete_post_route(
                    DraftInvoicePostRouteTag::new(self.id),
                    ButtonDeletePost {
                        label: "Post invoice",
                        confirm: "Post this draft invoice? This will create a posted invoice.",
                        classes: "btn-primary",
                    },
                ))
            }
        };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (detail_header(DetailHeader {
                        title: &format!("Draft invoice #{}", self.id),
                        actions,
                    }))
                    @if let Some(e) = &self.error {
                        p class="text-error mb-2" { (e) }
                    }
                    (label_inline("Number", field_text(FieldText { value: &self.number, classes: "" })))
                    (label_inline("Reference", field_text(FieldText { value: &self.reference, classes: "" })))
                    (label_inline("Payment reference", field_text(FieldText { value: &self.payment_reference, classes: "" })))
                    (label_inline("Bank account", field_text(FieldText { value: &self.bank_account, classes: "" })))
                    (label_inline("Datetime", field_text(FieldText { value: &self.datetime, classes: "" })))
                    (label_inline("Customer", field_text(FieldText { value: &self.customer_name, classes: "" })))
                    (label_inline("Payment term", field_text(FieldText { value: &self.payment_term_summary, classes: "" })))
                    (label_inline("Taxes", field_text(FieldText { value: &self.tax_labels, classes: "" })))
                    (field_invoice_lines(&self.line_rows))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        draft_invoice_detail_menu(self.id, &self.number, "detail", self.can_edit)
    }
}

impl RenderAppPane for DraftInvoiceDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for DraftInvoiceDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Draft Invoice", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct PostedInvoiceDetailPage {
    pub id: i64,
    pub number: String,
    pub reference: String,
    pub payment_reference: String,
    pub bank_account: String,
    pub datetime: String,
    pub customer_name: String,
    pub payment_term_summary: String,
    pub tax_labels: String,
    pub line_rows: Vec<InvoiceLineDisplayRow>,
    pub journal_entry_id: i64,
    pub can_edit: bool,
    pub can_pay: bool,
}

impl PostedInvoiceDetailPage {
    fn body(&self) -> Markup {
        let pay_href = PaymentCreateGetRouteTag
            .with_query()
            .query("PostedInvoiceID", self.id)
            .build();
        let actions = html! {
            (button_download_route(PostedInvoicePdfRouteTag::new(self.id), "PDF", "btn-outline"))
            @if self.can_pay {
                (button_link(ButtonLink {
                    label: "Pay",
                    href: &pay_href,
                    classes: "btn-primary",
                    ..Default::default()
                }))
            }
            @if self.can_edit {
                a class="btn btn-error" href=(PostedInvoiceCancelGetRouteTag::new(self.id).url()) { "Cancel" }
            }
        };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (detail_header(DetailHeader {
                        title: &format!("Posted invoice {}", self.number),
                        actions,
                    }))
                    (label_inline("Reference", field_text(FieldText { value: &self.reference, classes: "" })))
                    (label_inline("Payment reference", field_text(FieldText { value: &self.payment_reference, classes: "" })))
                    (label_inline("Bank account", field_text(FieldText { value: &self.bank_account, classes: "" })))
                    (label_inline("Datetime", field_text(FieldText { value: &self.datetime, classes: "" })))
                    (label_inline("Customer", field_text(FieldText { value: &self.customer_name, classes: "" })))
                    (label_inline("Payment term", field_text(FieldText { value: &self.payment_term_summary, classes: "" })))
                    (label_inline("Taxes", field_text(FieldText { value: &self.tax_labels, classes: "" })))
                    (label_inline("Journal entry", journal_entry_link(self.journal_entry_id)))
                    (field_invoice_lines(&self.line_rows))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        posted_invoice_detail_menu(self.id, &self.number)
    }
}

impl RenderAppPane for PostedInvoiceDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PostedInvoiceDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Posted Invoice", chrome, self.menu(), self.body())
    }
}

#[derive(Clone)]
pub struct SettlementDetailContext {
    pub settlement_id: i64,
    pub posted_invoice_id: i64,
    pub number: String,
    pub reference: String,
    pub payment_reference: String,
    pub bank_account: String,
    pub datetime: String,
    pub posted_at: Option<String>,
    pub customer_name: String,
    pub payment_term_summary: String,
    pub tax_labels: String,
    pub line_rows: Vec<InvoiceLineDisplayRow>,
    pub journal_entry_id: i64,
    pub payment_id: i64,
    pub payment_label: String,
    pub payment_href: String,
    pub payment_datetime: String,
    pub prior_partial_label: Option<String>,
    pub prior_partial_href: Option<String>,
}

impl SettlementDetailContext {
    fn empty(settlement_id: i64) -> Self {
        Self {
            settlement_id,
            posted_invoice_id: 0,
            number: "Not found".to_string(),
            reference: String::new(),
            payment_reference: String::new(),
            bank_account: String::new(),
            datetime: String::new(),
            posted_at: None,
            customer_name: String::new(),
            payment_term_summary: String::new(),
            tax_labels: String::new(),
            line_rows: vec![],
            journal_entry_id: 0,
            payment_id: 0,
            payment_label: String::new(),
            payment_href: String::new(),
            payment_datetime: String::new(),
            prior_partial_label: None,
            prior_partial_href: None,
        }
    }
}

fn settlement_detail_body(
    title: &str,
    ctx: &SettlementDetailContext,
    pdf_route: impl lariv_rs::http::RouteUrl,
    can_pay: bool,
    can_edit: bool,
) -> Markup {
    let pay_href = PaymentCreateGetRouteTag
        .with_query()
        .query("PostedInvoiceID", ctx.posted_invoice_id)
        .build();
    let actions = html! {
        (button_download_route(pdf_route, "PDF", "btn-outline"))
        @if can_pay {
            (button_link(ButtonLink {
                label: "Pay",
                href: &pay_href,
                classes: "btn-primary",
                ..Default::default()
            }))
        }
        @if can_edit && ctx.posted_invoice_id > 0 {
            a class="btn btn-error" href=(PostedInvoiceCancelGetRouteTag::new(ctx.posted_invoice_id).url()) { "Cancel" }
        }
    };
    let posted_at_display = ctx.posted_at.as_deref().unwrap_or("—");
    html! {
        (detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader { title, actions }))
                (label_inline("Number", field_text(FieldText { value: &ctx.number, classes: "" })))
                (label_inline("Reference", field_text(FieldText { value: &ctx.reference, classes: "" })))
                (label_inline("Payment reference", field_text(FieldText { value: &ctx.payment_reference, classes: "" })))
                (label_inline("Bank account", field_text(FieldText { value: &ctx.bank_account, classes: "" })))
                (label_inline("Posted date", field_text(FieldText { value: posted_at_display, classes: "" })))
                (label_inline("Invoice date", field_text(FieldText { value: &ctx.datetime, classes: "" })))
                (label_inline("Customer", field_text(FieldText { value: &ctx.customer_name, classes: "" })))
                (label_inline("Payment term", field_text(FieldText { value: &ctx.payment_term_summary, classes: "" })))
                (label_inline("Taxes", field_text(FieldText { value: &ctx.tax_labels, classes: "" })))
                (label_inline("Journal entry", journal_entry_link(ctx.journal_entry_id)))
                (label_inline("Payment", cancelled_detail_link(&Some(ctx.payment_href.clone()), &ctx.payment_label)))
                (label_inline("Payment date", field_text(FieldText { value: &ctx.payment_datetime, classes: "" })))
                (label_inline("Prior partial record", cancelled_detail_link(
                    &ctx.prior_partial_href,
                    ctx.prior_partial_label.as_deref().unwrap_or("—"),
                )))
                (field_invoice_lines(&ctx.line_rows))
            }))
        }))
    }
}

#[derive(Generic)]
pub struct PaidInvoiceDetailPage {
    pub ctx: SettlementDetailContext,
    pub can_edit: bool,
    pub can_pay: bool,
}

impl PaidInvoiceDetailPage {
    pub fn not_found(id: i64) -> Self {
        Self {
            ctx: SettlementDetailContext::empty(id),
            can_edit: false,
            can_pay: false,
        }
    }

    fn body(&self) -> Markup {
        let title = if self.ctx.number.is_empty() || self.ctx.number == "Not found" {
            format!("Paid invoice #{}", self.ctx.settlement_id)
        } else {
            format!("Paid invoice {}", self.ctx.number)
        };
        settlement_detail_body(
            &title,
            &self.ctx,
            PaidInvoicePdfRouteTag::new(self.ctx.settlement_id),
            self.can_pay,
            self.can_edit,
        )
    }

    fn menu(&self) -> Markup {
        paid_invoice_detail_menu(self.ctx.settlement_id, &self.ctx.number)
    }
}

impl RenderAppPane for PaidInvoiceDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaidInvoiceDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Paid Invoice", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct PartiallyPaidInvoiceDetailPage {
    pub ctx: SettlementDetailContext,
    pub can_edit: bool,
    pub can_pay: bool,
}

impl PartiallyPaidInvoiceDetailPage {
    pub fn not_found(id: i64) -> Self {
        Self {
            ctx: SettlementDetailContext::empty(id),
            can_edit: false,
            can_pay: false,
        }
    }

    fn body(&self) -> Markup {
        let title = if self.ctx.number.is_empty() || self.ctx.number == "Not found" {
            format!("Partially paid invoice #{}", self.ctx.settlement_id)
        } else {
            format!("Partially paid invoice {}", self.ctx.number)
        };
        settlement_detail_body(
            &title,
            &self.ctx,
            PartiallyPaidInvoicePdfRouteTag::new(self.ctx.settlement_id),
            self.can_pay,
            self.can_edit,
        )
    }

    fn menu(&self) -> Markup {
        partially_paid_invoice_detail_menu(self.ctx.settlement_id, &self.ctx.number)
    }
}

impl RenderAppPane for PartiallyPaidInvoiceDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PartiallyPaidInvoiceDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Partially Paid Invoice", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct CancelledInvoiceDetailPage {
    pub id: i64,
    pub number: String,
    pub reference: String,
    pub payment_reference: String,
    pub bank_account: String,
    pub datetime: String,
    pub customer_name: String,
    pub payment_term_summary: String,
    pub tax_labels: String,
    pub line_rows: Vec<InvoiceLineDisplayRow>,
    pub posted_invoice_label: String,
    pub posted_invoice_href: Option<String>,
    pub credit_note_label: String,
    pub credit_note_href: Option<String>,
    pub can_edit: bool,
}

impl CancelledInvoiceDetailPage {
    fn body(&self) -> Markup {
        let actions = html! {
            (button_download_route(CancelledInvoicePdfRouteTag::new(self.id), "PDF", "btn-outline"))
            @if self.can_edit {
                (button_delete_post_route(
                    CancelledInvoiceNewDraftRouteTag::new(self.id),
                    ButtonDeletePost {
                        label: "New draft from cancelled",
                        confirm: "Create a new draft invoice from this cancelled invoice? The cancelled record will be unchanged.",
                        classes: "btn-primary",
                    },
                ))
            }
        };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (detail_header(DetailHeader {
                        title: &format!("Cancelled invoice {}", self.number),
                        actions,
                    }))
                    (label_inline("Reference", field_text(FieldText { value: &self.reference, classes: "" })))
                    (label_inline("Payment reference", field_text(FieldText { value: &self.payment_reference, classes: "" })))
                    (label_inline("Bank account", field_text(FieldText { value: &self.bank_account, classes: "" })))
                    (label_inline("Datetime", field_text(FieldText { value: &self.datetime, classes: "" })))
                    (label_inline("Customer", field_text(FieldText { value: &self.customer_name, classes: "" })))
                    (label_inline("Payment term", field_text(FieldText { value: &self.payment_term_summary, classes: "" })))
                    (label_inline("Taxes", field_text(FieldText { value: &self.tax_labels, classes: "" })))
                    (label_inline("Posted invoice", cancelled_detail_link(&self.posted_invoice_href, &self.posted_invoice_label)))
                    (label_inline("Credit note", cancelled_detail_link(&self.credit_note_href, &self.credit_note_label)))
                    (field_invoice_lines(&self.line_rows))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        cancelled_invoice_detail_menu(self.id, &self.number)
    }
}

fn cancelled_detail_link(href: &Option<String>, label: &str) -> Markup {
    if let Some(url) = href {
        field_link(FieldLink {
            href: url,
            label,
            classes: "link link-hover",
        })
    } else if label.is_empty() {
        field_text(FieldText { value: "—", classes: "" })
    } else {
        field_text(FieldText { value: label, classes: "" })
    }
}

fn journal_entry_link(id: i64) -> Markup {
    if id > 0 {
        field_link(FieldLink {
            href: &JournalEntryDetailRouteTag::new(id).url(),
            label: &format!("Entry #{id}"),
            classes: "link link-hover",
        })
    } else {
        field_text(FieldText { value: "—", classes: "" })
    }
}

impl RenderAppPane for CancelledInvoiceDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for CancelledInvoiceDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Cancelled Invoice", chrome, self.menu(), self.body())
    }
}

#[derive(Clone)]
pub struct PaymentRow {
    pub id: i64,
    pub invoice_label: String,
    pub amount: String,
    pub datetime: String,
}

#[derive(Generic)]
pub struct PaymentListPage {
    pub payments: ObjectList<PaymentRow>,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl PaymentListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Invoice", sort_url: None, push_url: true },
            TableColumnHeader { label: "Amount", sort_url: None, push_url: true },
            TableColumnHeader { label: "Date", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .payments
            .items
            .iter()
            .map(|p| TableRow {
                attrs: row_attr_navigate(&format!("/finance-invoices/payments/{}/", p.id)),
                cells: vec![
                    field_text(FieldText { value: &p.invoice_label, classes: "" }),
                    field_text(FieldText { value: &p.amount, classes: "" }),
                    field_text(FieldText { value: &p.datetime, classes: "" }),
                ],
            })
            .collect();
        let pagination = render_pagination::<PaymentTableKey>(
            &self.path_and_query,
            self.payments.number,
            self.payments.num_pages,
        );
        let actions = if self.can_edit {
            table_button_create(TableButtonCreate {
                href: &PaymentCreateGetRouteTag.url(),
                ..Default::default()
            })
        } else {
            html! {}
        };
        data_table_list::<PaymentTableKey>("Payments", actions, &headers, &rows, pagination)
    }

    fn body(&self) -> Markup {
        container_column("", self.render_table())
    }
}

impl RenderAppPane for PaymentListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Payments", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct PaymentFormPage {
    pub form: PaymentForm,
    pub posted_invoice_display: String,
    pub account_display: String,
    pub tax_items: Vec<ManyToManyItem>,
    pub can_edit: bool,
}

impl PaymentFormPage {
    fn body(&self) -> Markup {
        html! {
            (field_title(FieldTitle { value: "Record payment", classes: "" }))
            form method="post" action="/finance-invoices/payments/create/" {
                (PaymentForm::render_inputs(&FormCtx::form::<PaymentForm>()
                    .value(PaymentFormField::PostedInvoiceId, &self.form.posted_invoice_id.to_string())
                    .value(PaymentFormField::Amount, &self.form.amount)
                    .value(PaymentFormField::AccountId, &self.form.account_id)
                    .value(PaymentFormField::Datetime, &self.form.datetime)
                    .display(PaymentFormField::PostedInvoiceId, &self.posted_invoice_display)
                    .display(PaymentFormField::AccountId, &self.account_display)
                    .m2m(PaymentFormField::Taxes, &self.tax_items)))
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
            }
        }
    }
}

impl RenderAppPane for PaymentFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Record Payment", chrome, self.body())
    }
}

#[derive(Clone)]
pub struct PaymentBatchRow {
    pub id: i64,
    pub datetime: String,
    pub total_amount: String,
    pub payment_count: u64,
}

#[derive(Generic)]
pub struct PaymentBatchListPage {
    pub batches: ObjectList<PaymentBatchRow>,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl PaymentBatchListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Date", sort_url: None, push_url: true },
            TableColumnHeader { label: "Total", sort_url: None, push_url: true },
            TableColumnHeader { label: "Payments", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .batches
            .items
            .iter()
            .map(|b| TableRow {
                attrs: row_attr_navigate(&PaymentBatchDetailRouteTag::new(b.id).url()),
                cells: vec![
                    field_text(FieldText { value: &b.datetime, classes: "" }),
                    field_text(FieldText { value: &b.total_amount, classes: "" }),
                    field_text(FieldText {
                        value: &b.payment_count.to_string(),
                        classes: "",
                    }),
                ],
            })
            .collect();
        let pagination = render_pagination::<PaymentBatchTableKey>(
            &self.path_and_query,
            self.batches.number,
            self.batches.num_pages,
        );
        let actions = html! {};
        data_table_list::<PaymentBatchTableKey>("Payment batches", actions, &headers, &rows, pagination)
    }

    fn body(&self) -> Markup {
        container_column("", self.render_table())
    }
}

impl RenderAppPane for PaymentBatchListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentBatchListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Payment Batches", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct PaymentDetailPage {
    pub id: i64,
    pub posted_invoice_label: String,
    pub posted_invoice_href: Option<String>,
    pub amount: String,
    pub tax_labels: String,
    pub datetime: String,
    pub journal_entry_id: i64,
    pub payment_batch_id: Option<i64>,
    pub payment_batch_href: Option<String>,
    pub can_edit: bool,
}

impl PaymentDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &format!("Payment #{}", self.id), classes: "" }))
                    (label_inline("Posted invoice", cancelled_detail_link(&self.posted_invoice_href, &self.posted_invoice_label)))
                    @if let Some(href) = &self.payment_batch_href {
                        @if let Some(batch_id) = self.payment_batch_id {
                            (label_inline("Batch", field_link(FieldLink { href: href.as_str(), label: &format!("Batch #{batch_id}"), classes: "" })))
                        }
                    }
                    (label_inline("Settlement amount", field_text(FieldText { value: &self.amount, classes: "" })))
                    (label_inline("Withholding taxes", field_text(FieldText { value: &self.tax_labels, classes: "" })))
                    (label_inline("Datetime", field_text(FieldText { value: &self.datetime, classes: "" })))
                    (label_inline("Journal entry", journal_entry_link(self.journal_entry_id)))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        payment_detail_menu(self.id)
    }
}

impl RenderAppPane for PaymentDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Payment", chrome, self.menu(), self.body())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PaymentBatchAllocationRow {
    pub posted_invoice_id: i64,
    pub amount: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tax_ids: Vec<i64>,
    pub invoice_number: String,
    pub customer_name: String,
    pub open_balance: String,
}

#[derive(Generic)]
pub struct PaymentBatchFormPage {
    pub form: PaymentBatchForm,
    pub account_display: String,
    pub batch_allocations_preview: String,
    pub error: Option<String>,
    pub can_edit: bool,
}

impl PaymentBatchFormPage {
    fn body(&self) -> Markup {
        html! {
            (field_title(FieldTitle { value: "Batch payment", classes: "" }))
            @if let Some(e) = &self.error {
                p class="text-error" { (e) }
            }
            form method="post" action="/finance-invoices/payments/batch/create/" {
                (PaymentBatchForm::render_inputs(&FormCtx::form::<PaymentBatchForm>()
                    .value(PaymentBatchFormField::Datetime, &self.form.datetime)
                    .value(PaymentBatchFormField::AccountId, &self.form.account_id)
                    .value(PaymentBatchFormField::AllocationsJson, &self.form.allocations_json)
                    .display(PaymentBatchFormField::AccountId, &self.account_display)
                    .display(PaymentBatchFormField::AllocationsJson, &self.batch_allocations_preview)))
                (button_submit(ButtonSubmit { label: "Record batch payment", ..Default::default() }))
            }
        }
    }

    fn menu(&self) -> Markup {
        payment_batch_form_menu()
    }
}

impl RenderAppPane for PaymentBatchFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentBatchFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Batch Payment", chrome, self.menu(), self.body())
    }
}

#[derive(Clone)]
pub struct PaymentBatchPaymentRow {
    pub id: i64,
    pub href: String,
    pub invoice_label: String,
    pub invoice_href: String,
    pub amount: String,
    pub tax_labels: String,
}

#[derive(Generic)]
pub struct PaymentBatchDetailPage {
    pub id: i64,
    pub datetime: String,
    pub account_label: String,
    pub total_amount: String,
    pub journal_entry_id: i64,
    pub payments: Vec<PaymentBatchPaymentRow>,
    pub can_edit: bool,
}

impl PaymentBatchDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &format!("Payment batch #{}", self.id), classes: "" }))
                    (label_inline("Datetime", field_text(FieldText { value: &self.datetime, classes: "" })))
                    (label_inline("Bank account", field_text(FieldText { value: &self.account_label, classes: "" })))
                    (label_inline("Total settlement", field_text(FieldText { value: &self.total_amount, classes: "" })))
                    (label_inline("Journal entry", journal_entry_link(self.journal_entry_id)))
                    h3 class="text-lg font-semibold mt-4" { "Payments in batch" }
                    div class="overflow-x-auto" {
                        table class="table table-zebra w-full" {
                            thead {
                                tr {
                                    th { "Payment" }
                                    th { "Invoice" }
                                    th class="text-end" { "Amount" }
                                    th { "Withholding" }
                                }
                            }
                            tbody {
                                @for p in &self.payments {
                                    tr {
                                        td {
                                            (field_link(FieldLink { href: &p.href, label: &format!("#{}", p.id), classes: "" }))
                                        }
                                        td {
                                            (field_link(FieldLink { href: &p.invoice_href, label: &p.invoice_label, classes: "" }))
                                        }
                                        td class="text-end tabular-nums" {
                                            (field_text(FieldText { value: &p.amount, classes: "" }))
                                        }
                                        td {
                                            (field_text(FieldText { value: &p.tax_labels, classes: "" }))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        payment_batch_detail_menu(self.id)
    }
}

impl RenderAppPane for PaymentBatchDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentBatchDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Payment Batch", chrome, self.menu(), self.body())
    }
}

#[derive(Clone)]
pub struct PaymentTermRow {
    pub id: i64,
    pub term_type: String,
    pub summary: String,
}

#[derive(Generic)]
pub struct PaymentTermListPage {
    pub terms: ObjectList<PaymentTermRow>,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl PaymentTermListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Type", sort_url: None, push_url: true },
            TableColumnHeader { label: "Summary", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .terms
            .items
            .iter()
            .map(|t| {
                let type_label = payment_term_type_label(&t.term_type);
                TableRow {
                attrs: row_attr_navigate(&format!("/finance-invoices/pt/{}/", t.id)),
                cells: vec![
                    field_text(FieldText { value: type_label, classes: "" }),
                    field_text(FieldText { value: &t.summary, classes: "" }),
                ],
            }
            })
            .collect();
        let pagination = render_pagination::<PaymentTermTableKey>(
            &self.path_and_query,
            self.terms.number,
            self.terms.num_pages,
        );
        let actions = if self.can_edit {
            table_button_create(TableButtonCreate {
                href: &PaymentTermCreateGetRouteTag.url(),
                ..Default::default()
            })
        } else {
            html! {}
        };
        data_table_list::<PaymentTermTableKey>(
            "Payment Terms",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }

    fn body(&self) -> Markup {
        container_column("", self.render_table())
    }
}

impl RenderAppPane for PaymentTermListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentTermListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Payment Terms", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct PaymentTermSelectPage {
    pub terms: ObjectList<PaymentTermRow>,
    pub target_input: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl RenderPickerSelect<PaymentTermSelectTableKey, PaymentTermSelectModalKey> for PaymentTermSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Type", sort_url: None, push_url: false },
            TableColumnHeader { label: "Summary", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .terms
            .items
            .iter()
            .map(|t| {
                let type_label = payment_term_type_label(&t.term_type);
                TableRow {
                attrs: row_attr_select(&self.target_input, &t.id.to_string(), &t.summary),
                cells: vec![
                    field_text(FieldText { value: type_label, classes: "" }),
                    field_text(FieldText { value: &t.summary, classes: "" }),
                ],
            }
            })
            .collect();
        let actions = if self.can_edit {
            html! {
                (button_link(ButtonLink {
                    href: &PaymentTermCreateGetRouteTag.url(),
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            }
        } else {
            html! {}
        };
        let pagination = render_pagination::<PaymentTermSelectTableKey>(
            &self.path_and_query,
            self.terms.number,
            self.terms.num_pages,
        );
        data_table_list::<PaymentTermSelectTableKey>(
            "Select Payment Term",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for PaymentTermSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct PostedInvoiceSelectRow {
    pub id: i64,
    pub number: String,
    pub datetime: String,
}

#[derive(Generic)]
pub struct PostedInvoiceSelectPage {
    pub invoices: ObjectList<PostedInvoiceSelectRow>,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<PostedInvoiceSelectTableKey, PostedInvoiceSelectModalKey>
    for PostedInvoiceSelectPage
{
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Number", sort_url: None, push_url: false },
            TableColumnHeader { label: "Date", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .invoices
            .items
            .iter()
            .map(|inv| {
                let label = if inv.number.is_empty() {
                    format!("#{}", inv.id)
                } else {
                    inv.number.clone()
                };
                TableRow {
                    attrs: row_attr_select(&self.target_input, &inv.id.to_string(), &label),
                    cells: vec![
                        field_text(FieldText { value: &inv.number, classes: "" }),
                        field_text(FieldText { value: &inv.datetime, classes: "" }),
                    ],
                }
            })
            .collect();
        let pagination = render_pagination::<PostedInvoiceSelectTableKey>(
            &self.path_and_query,
            self.invoices.number,
            self.invoices.num_pages,
        );
        data_table_list::<PostedInvoiceSelectTableKey>(
            "Select Posted Invoice",
            html! {},
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for PostedInvoiceSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct PaymentTermFormPage {
    pub id: i64,
    pub form: PaymentTermForm,
    pub summary: String,
    pub is_edit: bool,
}

impl PaymentTermFormPage {
    fn term_choices() -> Vec<(String, String)> {
        PaymentTermForm::term_type_choices()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn body(&self) -> Markup {
        let title = if self.is_edit {
            "Edit payment term"
        } else {
            "Create payment term"
        };
        let choices = Self::term_choices();
        let x_data = PaymentTermForm::alpine_x_data(&self.form.term_type);
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(PaymentTermEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(PaymentTermCreatePostRouteTag)
                    },
                    inputs: PaymentTermForm::render_inputs(&FormCtx::form::<PaymentTermForm>()
                        .x_data(&x_data)
                        .value(PaymentTermFormField::TermType, &self.form.term_type)
                        .choices(PaymentTermFormField::TermType, &choices)
                        .value(PaymentTermFormField::DueDatetime, &self.form.due_datetime)
                        .value(PaymentTermFormField::Duration, &self.form.duration)),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: if self.is_edit { "Save" } else { "Save" },
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    PaymentTermDeletePostRouteTag::new(self.id),
                                    "Delete Payment Term",
                                    "Permanently delete this payment term?",
                                ))
                            }
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }

    fn sidebar(&self) -> Markup {
        if self.is_edit {
            payment_term_detail_menu(self.id, &self.summary, "edit", true)
        } else {
            uniquity_finance_accounts::accounting_sidebar::accounting_sidebar()
        }
    }
}

impl RenderAppPane for PaymentTermFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.sidebar(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentTermFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let title = if self.is_edit {
            "Edit Payment Term"
        } else {
            "Create Payment Term"
        };
        app_scaffold_with_sidebar(title, chrome, self.sidebar(), self.body())
    }
}


#[derive(Generic)]
pub struct PaymentTermDetailPage {
    pub id: i64,
    pub term_type: String,
    pub summary: String,
    pub can_edit: bool,
}

impl PaymentTermDetailPage {
    fn title(&self) -> String {
        payment_term_display_label(&self.summary, self.id)
    }

    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.title(), classes: "" }))
                    (label_inline("Type", field_text(FieldText { value: payment_term_type_label(&self.term_type), classes: "" })))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        payment_term_detail_menu(self.id, &self.summary, "detail", self.can_edit)
    }
}

impl RenderAppPane for PaymentTermDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentTermDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar(&self.title(), chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct CancelInvoicePage {
    pub id: i64,
    pub form: CancelInvoiceForm,
    pub can_edit: bool,
}

impl CancelInvoicePage {
    fn body(&self) -> Markup {
        html! {
            (field_title(FieldTitle { value: &format!("Cancel posted invoice #{}", self.id), classes: "" }))
            form method="post" action=(format!("/finance-invoices/posted/{}/cancel/", self.id)) {
                (CancelInvoiceForm::render_inputs(
                    &FormCtx::form::<CancelInvoiceForm>()
                        .value(CancelInvoiceFormField::Reason, &self.form.reason),
                ))
                (button_submit(ButtonSubmit { label: "Cancel invoice", ..Default::default() }))
            }
        }
    }

    fn menu(&self) -> Markup {
        posted_invoice_detail_menu(self.id, "")
    }
}

impl RenderAppPane for CancelInvoicePage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for CancelInvoicePage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Cancel Invoice", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct InvoicePreferencesPage {
    pub form: InvoicePreferencesForm,
    pub can_edit: bool,
}

impl InvoicePreferencesPage {
    fn body(&self) -> Markup {
        html! {
            (field_title(FieldTitle { value: "Invoice preferences", classes: "" }))
            form method="post" action="/finance-invoices/preferences/" {
                (InvoicePreferencesForm::render_inputs(&FormCtx::form::<InvoicePreferencesForm>()
                    .value(
                        InvoicePreferencesFormField::AccountReceivableId,
                        &self.form.account_receivable_id,
                    )
                    .value(
                        InvoicePreferencesFormField::AccountRevenueId,
                        &self.form.account_revenue_id,
                    )
                    .value(
                        InvoicePreferencesFormField::AccountTaxPayableId,
                        &self.form.account_tax_payable_id,
                    )
                    .value(InvoicePreferencesFormField::JournalId, &self.form.journal_id)))
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
            }
        }
    }
}

impl RenderAppPane for InvoicePreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for InvoicePreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Invoice Preferences", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct PaymentPreferencesPage {
    pub form: PaymentPreferencesForm,
    pub can_edit: bool,
}

impl PaymentPreferencesPage {
    fn body(&self) -> Markup {
        html! {
            (field_title(FieldTitle { value: "Payment preferences", classes: "" }))
            form method="post" action="/finance-invoices/payment-preferences/" {
                (PaymentPreferencesForm::render_inputs(&FormCtx::form::<PaymentPreferencesForm>()
                    .value(
                        PaymentPreferencesFormField::PaymentAccountId,
                        &self.form.payment_account_id,
                    )))
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
            }
        }
    }
}

impl RenderAppPane for PaymentPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for PaymentPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Payment Preferences", chrome, self.body())
    }
}
