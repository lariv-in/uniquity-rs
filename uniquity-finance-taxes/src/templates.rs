use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonSubmit, FieldText, FieldTitle, FormOpts,
        ObjectList, PaginationPage, ShellChrome, SlotCapability,
        SlotRegistrar, SwapKey, TableButtonFilter, TableColumnHeader, TablePagination, TableRow,
        button_clear, button_delete, button_link, button_submit, container_column,
        container_row, data_table_list, detail, field_text, field_title,
        form, form_hx_get_picker_route, form_hx_get_route, form_hx_post_main,
        pagination_pages, row_attr_navigate_route, row_attr_select_multi,
        table_button_filter, table_pagination,
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
    layout_with_sidebar, render_picker_pagination,
};

use super::forms::{TaxFilterForm, TaxFilterFormField, TaxForm, TaxFormField, tax_type_choices};
use super::keys::{TaxMultiSelectModalKey, TaxMultiSelectTableKey, TaxTableKey};
use super::routes::{
    TaxCreateGetRouteTag, TaxCreatePostRouteTag, TaxDefaultRouteTag,
    TaxDeletePostRouteTag, TaxDetailRouteTag, TaxEditGetRouteTag, TaxEditPostRouteTag,
    TaxMultiSelectRouteTag,
};

fn tax_detail_menu(id: i64, name: &str, active: &str, can_edit: bool) -> Markup {
    let menu_title = format!("Tax: {name}");
    let detail_url = TaxDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Tax Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Tax",
            url: TaxEditGetRouteTag::new(id).url(),
            active: active == "edit",
        });
    }
    detail_sidebar_menu(
        menu_title,
        "Back to Taxes",
        TaxDefaultRouteTag.url(),
        &nav,
        None,
        html! {},
    )
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceTaxesTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        TaxListIdx: TaxListPageTag => TaxListPage,
        TaxDetailIdx: TaxDetailPageTag => TaxDetailPage,
        TaxFormIdx: TaxFormPageTag => TaxFormPage,
        TaxMultiSelectIdx: TaxMultiSelectPageTag => TaxMultiSelectPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceTaxesTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn tax_filter_form(name: &str) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<TaxTableKey, TaxDefaultRouteTag>(TaxDefaultRouteTag),
        inputs: TaxFilterForm::render_inputs(
            &FormCtx::form::<TaxFilterForm>().value(TaxFilterFormField::Name, name),
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

fn render_pagination(path_and_query: &str, number: u32, num_pages: u32) -> Markup {
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
        hx_target: TaxTableKey::SELECTOR,
    })
}

#[derive(Clone)]
pub struct TaxRow {
    pub id: i64,
    pub name: String,
    pub tax_type: String,
    pub percentage: String,
    pub account_label: String,
}

#[derive(Generic)]
pub struct TaxListPage {
    pub taxes: ObjectList<TaxRow>,
    pub filter_name: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl TaxListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Name", sort_url: None, push_url: true },
            TableColumnHeader { label: "Type", sort_url: None, push_url: true },
            TableColumnHeader { label: "Percentage", sort_url: None, push_url: true },
            TableColumnHeader { label: "Account", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .taxes
            .items
            .iter()
            .map(|t| TableRow {
                attrs: row_attr_navigate_route(TaxDetailRouteTag::new(t.id)),
                cells: vec![
                    field_text(FieldText { value: &t.name, classes: "" }),
                    field_text(FieldText { value: &t.tax_type, classes: "" }),
                    field_text(FieldText { value: &t.percentage, classes: "" }),
                    field_text(FieldText { value: &t.account_label, classes: "" }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: tax_filter_form(&self.filter_name),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (button_link(ButtonLink {
                    href: &TaxCreateGetRouteTag.url(),
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        let pagination = render_pagination(
            &self.path_and_query,
            self.taxes.number,
            self.taxes.num_pages,
        );
        data_table_list::<TaxTableKey>("Taxes", actions, &headers, &rows, pagination)
    }

    fn body(&self) -> Markup {
        self.render_table()
    }
}

impl RenderAppPane for TaxListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for TaxListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Taxes — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct TaxDetailPage {
    pub id: i64,
    pub name: String,
    pub tax_type: String,
    pub percentage: String,
    pub account_label: String,
    pub can_edit: bool,
}

impl TaxDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (lariv_rs::components::label_inline("Type", field_text(FieldText { value: &self.tax_type, classes: "" })))
                    (lariv_rs::components::label_inline("Percentage", field_text(FieldText { value: &self.percentage, classes: "" })))
                    (lariv_rs::components::label_inline("Account", field_text(FieldText { value: &self.account_label, classes: "" })))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        tax_detail_menu(self.id, &self.name, "detail", self.can_edit)
    }
}

impl RenderAppPane for TaxDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for TaxDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Tax — Uniquity", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct TaxFormPage {
    pub id: i64,
    pub name: String,
    pub tax_type: String,
    pub percentage: String,
    pub account_id: String,
    pub account_display: String,
    pub is_edit: bool,
}

impl TaxFormPage {
    fn body(&self) -> Markup {
        let title = if self.is_edit { "Edit Tax" } else { "Create Tax" };
        let choices = tax_type_choices();
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(TaxEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(TaxCreatePostRouteTag)
                    },
                    inputs: TaxForm::render_inputs(
                        &FormCtx::form::<TaxForm>()
                            .value(TaxFormField::Name, &self.name)
                            .value(TaxFormField::TaxType, &self.tax_type)
                            .value(TaxFormField::Percentage, &self.percentage)
                            .value(TaxFormField::AccountId, &self.account_id)
                            .display(TaxFormField::AccountId, &self.account_display)
                            .choices(TaxFormField::TaxType, &choices),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save Tax",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    TaxDeletePostRouteTag::new(self.id),
                                    "Delete Tax",
                                    "Permanently delete this tax?",
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
            tax_detail_menu(self.id, &self.name, "edit", true)
        } else {
            uniquity_finance_accounts::accounting_sidebar::accounting_sidebar()
        }
    }
}

impl RenderAppPane for TaxFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.sidebar(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for TaxFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Tax Form — Uniquity", chrome, self.sidebar(), self.body())
    }
}

#[derive(Generic)]
pub struct TaxMultiSelectPage {
    pub taxes: ObjectList<TaxRow>,
    pub filter_name: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<TaxMultiSelectTableKey, TaxMultiSelectModalKey> for TaxMultiSelectPage {
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "Taxes"
        } else {
            self.target_input.as_str()
        };
        let headers = [
            TableColumnHeader { label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { label: "Type", sort_url: None, push_url: false },
            TableColumnHeader { label: "Percentage", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .taxes
            .items
            .iter()
            .map(|t| TableRow {
                attrs: row_attr_select_multi(target, &t.id.to_string(), &t.name),
                cells: vec![
                    field_text(FieldText { value: &t.name, classes: "" }),
                    field_text(FieldText { value: &t.tax_type, classes: "" }),
                    field_text(FieldText { value: &t.percentage, classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        TaxMultiSelectTableKey,
                        TaxMultiSelectModalKey,
                        TaxMultiSelectRouteTag,
                    >(TaxMultiSelectRouteTag),
                    inputs: html! {
                        (TaxFilterForm::render_inputs(
                            &FormCtx::form::<TaxFilterForm>()
                                .value(TaxFilterFormField::Name, &self.filter_name),
                        ))
                        input type="hidden" name="target_input" value=(self.target_input) {}
                    },
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
        };
        let pagination = render_picker_pagination::<TaxMultiSelectModalKey>(
            &self.path_and_query,
            self.taxes.number,
            self.taxes.num_pages,
        );
        data_table_list::<TaxMultiSelectTableKey>(
            "Select Taxes",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for TaxMultiSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}
