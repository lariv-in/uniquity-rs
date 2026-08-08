use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, Crumb, FieldText, FieldTitle, FormOpts, ObjectList, PaginationPage, ShellChrome, SlotCapability,
        SlotRegistrar, SwapKey, TableButtonFilter, TableColumnHeader, TablePagination, TableRow,
        ButtonModalForm, breadcrumbs, button_clear, button_delete, button_modal_form, button_submit, container_column,
        container_row, data_table_list_refresh, detail, field_text, field_title,
        form, form_hx_get_route, form_hx_post_main, form_hx_post_url, label_inline, modal_keyed,
        pagination_pages,
        row_attr_navigate_route, row_attr_select,
        table_button_filter, table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::modal_create_post_url,
};

use uniquity_finance_accounts::accounting_detail_menu::{
    DetailMenuNavItem, detail_sidebar_menu,
};
use uniquity_finance_accounts::templates::{
    app_scaffold, app_scaffold_with_sidebar, layout_main_with_crumbs,
    layout_with_entity_sidebar_crumbs, layout_with_sidebar_crumbs,
};

use super::forms::{
    CustomerFilterForm, CustomerFilterFormField, CustomerForm, CustomerFormField,
};
use super::keys::{CustomerCreateModalKey, CustomerSelectModalKey, CustomerSelectTableKey, CustomerTableKey};
use super::routes::{
    CustomerCreateGetRouteTag, CustomerCreatePostRouteTag, CustomerDefaultRouteTag,
    CustomerDeletePostRouteTag, CustomerDetailRouteTag,
    CustomerEditGetRouteTag, CustomerEditPostRouteTag, CustomerFkSelectRouteTag,
};

fn customers_list_crumbs() -> Markup {
    breadcrumbs(&[Crumb {
        label: "Customers",
        href: None,
    }])
}

fn customer_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    let list_url = CustomerDefaultRouteTag.url();
    let detail_url = CustomerDetailRouteTag::new(id).url();
    match action {
        None => breadcrumbs(&[
            Crumb {
                label: "Customers",
                href: Some(&list_url),
            },
            Crumb {
                label: name,
                href: None,
            },
        ]),
        Some(act) => breadcrumbs(&[
            Crumb {
                label: "Customers",
                href: Some(&list_url),
            },
            Crumb {
                label: name,
                href: Some(&detail_url),
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
    }
}

fn customer_detail_menu(id: i64, name: &str, active: &str, can_edit: bool) -> Markup {
    let menu_title = format!("Customer: {name}");
    let detail_url = CustomerDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Customer Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Customer",
            url: CustomerEditGetRouteTag::new(id).url(),
            active: active == "edit",
        });
    }
    detail_sidebar_menu(
        menu_title,
        &nav,
        None,
        html! {},
    )
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceCustomerTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        CustomerListIdx: CustomerListPageTag => CustomerListPage,
        CustomerDetailIdx: CustomerDetailPageTag => CustomerDetailPage,
        CustomerFormIdx: CustomerFormPageTag => CustomerFormPage,
        CustomerCreateModalIdx: CustomerCreateModalPageTag => CustomerCreateModalPage,
        CustomerSelectIdx: CustomerSelectPageTag => CustomerSelectPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceCustomerTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn customer_filter_form(name: &str, email: &str) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<CustomerTableKey, CustomerDefaultRouteTag>(
            CustomerDefaultRouteTag,
        ),
        inputs: CustomerFilterForm::render_inputs(
            &FormCtx::form::<CustomerFilterForm>()
                .value(CustomerFilterFormField::Name, name)
                .value(CustomerFilterFormField::Email, email),
        ),
        actions: html! {
            (container_row("flex gap-2", html! {
                (button_submit(ButtonSubmit { label: "Apply Filters", ..Default::default() }))
                (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
            }))
        },
        ..Default::default()
    })
}

fn customer_select_filter_form(name: &str, email: &str, target_input: &str) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<CustomerSelectTableKey, CustomerFkSelectRouteTag>(
            CustomerFkSelectRouteTag,
        )
        .set("hx-push-url", "false"),
        inputs: html! {
            (CustomerFilterForm::render_inputs(
                &FormCtx::form::<CustomerFilterForm>()
                    .value(CustomerFilterFormField::Name, name)
                    .value(CustomerFilterFormField::Email, email),
            ))
            input type="hidden" name="target_input" value=(target_input) {}
        },
        actions: html! {
            (container_row("flex gap-2", html! {
                (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
            }))
        },
        ..Default::default()
    })
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

#[derive(Clone)]
pub struct CustomerRow {
    pub id: i64,
    pub customer_type: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub gstin: String,
}

#[derive(Generic)]
pub struct CustomerListPage {
    pub customers: ObjectList<CustomerRow>,
    pub filter_name: String,
    pub filter_email: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl CustomerListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Name", sort_url: None, push_url: true },
            TableColumnHeader { label: "Type", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .customers
            .items
            .iter()
            .map(|c| TableRow {
                attrs: row_attr_navigate_route(CustomerDetailRouteTag::new(c.id)),
                cells: vec![
                    field_text(FieldText { value: &c.name, classes: "" }),
                    field_text(FieldText { value: &c.customer_type, classes: "" }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: customer_filter_form(&self.filter_name, &self.filter_email),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (button_modal_form(ButtonModalForm {
                    name: "p_uniquity_finance_customer.CustomerCreateForm",
                    href: &CustomerCreateGetRouteTag.url(),
                    form_post_url: &CustomerCreateGetRouteTag.path(),
                    modal_uid: CustomerCreateModalKey::ID,
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        let pagination = render_pagination::<CustomerTableKey>(
            &self.path_and_query,
            self.customers.number,
            self.customers.num_pages,
        );
        data_table_list_refresh::<CustomerTableKey>(
            "Customers",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }

    fn body(&self) -> Markup {
        self.render_table()
    }
}

impl RenderAppPane for CustomerListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar_crumbs(&self.path_and_query, customers_list_crumbs(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_with_crumbs(customers_list_crumbs(), self.body())
    }
}

impl RenderTemplate for CustomerListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Finance Customers — Uniquity",
            chrome,
            customers_list_crumbs(),
            self.body(),
            &self.path_and_query,
        )
    }
}

#[derive(Generic)]
pub struct CustomerDetailPage {
    pub id: i64,
    pub customer_type: String,
    pub name: String,
    pub address_line_1: String,
    pub address_line_2: String,
    pub city: String,
    pub pincode: String,
    pub state: String,
    pub gstin: String,
    pub pan: String,
    pub phone: String,
    pub email: String,
    pub website: String,
    pub can_edit: bool,
}

impl CustomerDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (label_inline("Type", field_text(FieldText { value: &self.customer_type, classes: "" })))
                    (label_inline("Address line 1", field_text(FieldText { value: &self.address_line_1, classes: "" })))
                    (label_inline("Address line 2", field_text(FieldText { value: &self.address_line_2, classes: "" })))
                    (label_inline("City", field_text(FieldText { value: &self.city, classes: "" })))
                    (label_inline("Pincode", field_text(FieldText { value: &self.pincode, classes: "" })))
                    (label_inline("State", field_text(FieldText { value: &self.state, classes: "" })))
                    (label_inline("GSTIN", field_text(FieldText { value: &self.gstin, classes: "" })))
                    (label_inline("PAN", field_text(FieldText { value: &self.pan, classes: "" })))
                    (label_inline("Phone", field_text(FieldText { value: &self.phone, classes: "" })))
                    (label_inline("Email", field_text(FieldText { value: &self.email, classes: "" })))
                    (label_inline("Website", field_text(FieldText { value: &self.website, classes: "" })))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        customer_detail_menu(self.id, &self.name, "detail", self.can_edit)
    }
}

impl RenderAppPane for CustomerDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = customer_crumbs(self.id, &self.name, None);
        layout_with_entity_sidebar_crumbs(self.menu(), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_with_crumbs(customer_crumbs(self.id, &self.name, None), self.body())
    }
}

impl RenderTemplate for CustomerDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = customer_crumbs(self.id, &self.name, None);
        app_scaffold_with_sidebar("Customer — Uniquity", chrome, self.menu(), crumbs, self.body())
    }
}

#[derive(Generic)]
pub struct CustomerFormPage {
    pub id: i64,
    pub customer_type: String,
    pub name: String,
    pub address_line_1: String,
    pub address_line_2: String,
    pub city: String,
    pub pincode: String,
    pub state: String,
    pub gstin: String,
    pub pan: String,
    pub phone: String,
    pub email: String,
    pub website: String,
}

impl CustomerFormPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Edit Customer", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(CustomerEditPostRouteTag::new(self.id)),
                    inputs: {
                        let choices = CustomerForm::customer_type_choices();
                        CustomerForm::render_inputs(
                            &FormCtx::form::<CustomerForm>()
                                .value(CustomerFormField::CustomerType, &self.customer_type)
                                .value(CustomerFormField::Name, &self.name)
                                .value(CustomerFormField::AddressLine1, &self.address_line_1)
                                .value(CustomerFormField::AddressLine2, &self.address_line_2)
                                .value(CustomerFormField::City, &self.city)
                                .value(CustomerFormField::Pincode, &self.pincode)
                                .value(CustomerFormField::State, &self.state)
                                .value(CustomerFormField::Gstin, &self.gstin)
                                .value(CustomerFormField::Pan, &self.pan)
                                .value(CustomerFormField::Phone, &self.phone)
                                .value(CustomerFormField::Email, &self.email)
                                .value(CustomerFormField::Website, &self.website)
                                .choices(
                                    CustomerFormField::CustomerType,
                                    &choices
                                        .iter()
                                        .map(|(k, v)| (k.to_string(), v.to_string()))
                                        .collect::<Vec<_>>(),
                                ),
                        )
                    },
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save Customer",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            (button_delete(
                                CustomerDeletePostRouteTag::new(self.id),
                                "Delete Customer",
                                "Permanently delete this customer?",
                            ))
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }

    fn sidebar(&self) -> Markup {
        customer_detail_menu(self.id, &self.name, "edit", true)
    }
}

impl RenderAppPane for CustomerFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = customer_crumbs(self.id, &self.name, Some("Edit"));
        layout_with_entity_sidebar_crumbs(self.sidebar(), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_with_crumbs(customer_crumbs(self.id, &self.name, Some("Edit")), self.body())
    }
}

impl RenderTemplate for CustomerFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = customer_crumbs(self.id, &self.name, Some("Edit"));
        app_scaffold_with_sidebar(
            "Edit Customer — Uniquity",
            chrome,
            self.sidebar(),
            crumbs,
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct CustomerCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub customer_type: String,
    pub name: String,
    pub address_line_1: String,
    pub address_line_2: String,
    pub city: String,
    pub pincode: String,
    pub state: String,
    pub gstin: String,
    pub pan: String,
    pub phone: String,
    pub email: String,
    pub website: String,
    pub error: String,
}

impl RenderTemplate for CustomerCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_uniquity_finance_customer.CustomerCreateForm"
        } else {
            self.form_name.as_str()
        };
        let choices = CustomerForm::customer_type_choices();
        modal_keyed::<CustomerCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Customer",
                subtitle: "Create a new customer",
                classes: "@container",
                attrs: form_hx_post_url::<CustomerCreateModalKey>(
                    &modal_create_post_url(
                        CustomerCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                    ),
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: CustomerForm::render_inputs(
                    &FormCtx::form::<CustomerForm>()
                        .value(CustomerFormField::CustomerType, &self.customer_type)
                        .value(CustomerFormField::Name, &self.name)
                        .value(CustomerFormField::AddressLine1, &self.address_line_1)
                        .value(CustomerFormField::AddressLine2, &self.address_line_2)
                        .value(CustomerFormField::City, &self.city)
                        .value(CustomerFormField::Pincode, &self.pincode)
                        .value(CustomerFormField::State, &self.state)
                        .value(CustomerFormField::Gstin, &self.gstin)
                        .value(CustomerFormField::Pan, &self.pan)
                        .value(CustomerFormField::Phone, &self.phone)
                        .value(CustomerFormField::Email, &self.email)
                        .value(CustomerFormField::Website, &self.website)
                        .choices(
                            CustomerFormField::CustomerType,
                            &choices
                                .iter()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect::<Vec<_>>(),
                        ),
                ),
                actions: html! {
                    (container_row("flex justify-end gap-2 mt-2", html! {
                        (button_submit(ButtonSubmit {
                            label: "Save Customer",
                            classes: "btn-primary",
                            ..Default::default()
                        }))
                    }))
                },
                ..Default::default()
            }),
        )
    }
}

#[derive(Generic)]
pub struct CustomerSelectPage {
    pub customers: ObjectList<CustomerRow>,
    pub filter_name: String,
    pub filter_email: String,
    pub target_input: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl RenderPickerSelect<CustomerSelectTableKey, CustomerSelectModalKey> for CustomerSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { label: "Email", sort_url: None, push_url: false },
            TableColumnHeader { label: "Phone", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .customers
            .items
            .iter()
            .map(|c| TableRow {
                attrs: row_attr_select(&self.target_input, &c.id.to_string(), &c.name),
                cells: vec![
                    field_text(FieldText { value: &c.name, classes: "" }),
                    field_text(FieldText { value: &c.email, classes: "" }),
                    field_text(FieldText { value: &c.phone, classes: "" }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: customer_select_filter_form(
                    &self.filter_name,
                    &self.filter_email,
                    &self.target_input,
                ),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (button_modal_form(ButtonModalForm {
                    name: "p_uniquity_finance_customer.CustomerCreateForm",
                    href: &CustomerCreateGetRouteTag.url(),
                    form_post_url: &CustomerCreateGetRouteTag.path(),
                    modal_uid: CustomerCreateModalKey::ID,
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        let pagination = render_pagination::<CustomerSelectTableKey>(
            &self.path_and_query,
            self.customers.number,
            self.customers.num_pages,
        );
        data_table_list_refresh::<CustomerSelectTableKey>(
            "Select Customer",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for CustomerSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}
