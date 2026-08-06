use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonModalForm, ButtonSubmit, FieldCheckbox, FieldDate,
        FieldText, FieldTitle, FormOpts, ObjectList, PaginationPage, ShellChrome,
        SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter, TableColumnHeader,
        TablePagination, TableRow, button_clear, button_delete, button_modal_form,
        button_submit, container_column, container_row, data_table_list,
        data_table_list_refresh, detail, field_checkbox, field_date, field_text, field_title,
        form, form_hx_get_route, form_hx_post_main, form_hx_post_url, label_inline, modal_keyed,
        pagination_pages, row_attr_navigate_route, row_attr_select,
        table_button_filter, table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    picker::RenderPickerSelect,
    web::modal_create_post_url,
};

use uniquity_finance_accounts::accounting_detail_menu::{
    DetailMenuNavItem, detail_sidebar_menu,
};
use uniquity_finance_accounts::templates::{
    app_scaffold, app_scaffold_with_sidebar, layout_main_content, layout_with_entity_sidebar,
    layout_with_sidebar,
};

use super::forms::{
    FiscalYearFilterForm, FiscalYearFilterFormField, FiscalYearForm, FiscalYearFormField,
};
use super::keys::{
    FiscalYearCreateModalKey, FiscalYearSelectModalKey, FiscalYearSelectTableKey, FiscalYearTableKey,
};
use super::routes::{
    FiscalYearCreateGetRouteTag, FiscalYearCreatePostRouteTag, FiscalYearDefaultRouteTag,
    FiscalYearDeletePostRouteTag, FiscalYearDetailRouteTag,
    FiscalYearEditGetRouteTag, FiscalYearEditPostRouteTag, FiscalYearSelectRouteTag,
};

fn fiscal_year_detail_menu(id: i64, name: &str, active: &str, can_edit: bool) -> Markup {
    let menu_title = format!("Fiscal year: {name}");
    let detail_url = FiscalYearDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Fiscal Year Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Fiscal Year",
            url: FiscalYearEditGetRouteTag::new(id).url(),
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
    plugin: UniquityFinanceFiscalYearTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        FiscalYearListIdx: FiscalYearListPageTag => FiscalYearListPage,
        FiscalYearDetailIdx: FiscalYearDetailPageTag => FiscalYearDetailPage,
        FiscalYearFormIdx: FiscalYearFormPageTag => FiscalYearFormPage,
        FiscalYearCreateModalIdx: FiscalYearCreateModalPageTag => FiscalYearCreateModalPage,
        FiscalYearSelectIdx: FiscalYearSelectPageTag => FiscalYearSelectPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceFiscalYearTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn fiscal_year_filter_form(code: &str, name: &str) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<FiscalYearTableKey, FiscalYearDefaultRouteTag>(
            FiscalYearDefaultRouteTag,
        ),
        inputs: FiscalYearFilterForm::render_inputs(
            &FormCtx::form::<FiscalYearFilterForm>()
                .value(FiscalYearFilterFormField::Code, code)
                .value(FiscalYearFilterFormField::Name, name),
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
pub struct FiscalYearRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub start: String,
    pub end: String,
    pub is_active: bool,
}

#[derive(Generic)]
pub struct FiscalYearListPage {
    pub fiscal_years: ObjectList<FiscalYearRow>,
    pub filter_code: String,
    pub filter_name: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl FiscalYearListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Code", sort_url: None, push_url: true },
            TableColumnHeader { label: "Name", sort_url: None, push_url: true },
            TableColumnHeader { label: "Start", sort_url: None, push_url: true },
            TableColumnHeader { label: "End", sort_url: None, push_url: true },
            TableColumnHeader { label: "Active", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .fiscal_years
            .items
            .iter()
            .map(|fy| TableRow {
                attrs: row_attr_navigate_route(FiscalYearDetailRouteTag::new(fy.id)),
                cells: vec![
                    field_text(FieldText { value: &fy.code, classes: "" }),
                    field_text(FieldText { value: &fy.name, classes: "" }),
                    field_date(FieldDate { value: &fy.start, classes: "" }),
                    field_date(FieldDate { value: &fy.end, classes: "" }),
                    field_checkbox(FieldCheckbox {
                        checked: fy.is_active,
                        classes: "",
                    }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: fiscal_year_filter_form(&self.filter_code, &self.filter_name),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (button_modal_form(ButtonModalForm {
                    name: "p_uniquity_finance_fiscal_year.FiscalYearCreateForm",
                    href: &FiscalYearCreateGetRouteTag.url(),
                    form_post_url: &FiscalYearCreateGetRouteTag.path(),
                    modal_uid: FiscalYearCreateModalKey::ID,
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        let pagination = render_pagination::<FiscalYearTableKey>(
            &self.path_and_query,
            self.fiscal_years.number,
            self.fiscal_years.num_pages,
        );
        data_table_list_refresh::<FiscalYearTableKey>(
            "Fiscal Years",
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

impl RenderAppPane for FiscalYearListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(&self.path_and_query, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for FiscalYearListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Fiscal Years — Uniquity",
            chrome,
            self.body(),
            &self.path_and_query,
        )
    }
}

#[derive(Generic)]
pub struct FiscalYearDetailPage {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub start: String,
    pub end: String,
    pub is_active: bool,
    pub can_edit: bool,
}

impl FiscalYearDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (label_inline("Code", field_text(FieldText { value: &self.code, classes: "" })))
                    (label_inline("Start", field_date(FieldDate { value: &self.start, classes: "" })))
                    (label_inline("End", field_date(FieldDate { value: &self.end, classes: "" })))
                    (label_inline("Active", field_checkbox(FieldCheckbox {
                        checked: self.is_active,
                        classes: "",
                    })))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        fiscal_year_detail_menu(self.id, &self.name, "detail", self.can_edit)
    }
}

impl RenderAppPane for FiscalYearDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for FiscalYearDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Fiscal Year — Uniquity", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct FiscalYearFormPage {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub start: String,
    pub end: String,
    pub is_active: bool,
}

impl FiscalYearFormPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Edit Fiscal Year", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(FiscalYearEditPostRouteTag::new(self.id)),
                    inputs: FiscalYearForm::render_inputs(
                        &FormCtx::form::<FiscalYearForm>()
                            .value(FiscalYearFormField::Code, &self.code)
                            .value(FiscalYearFormField::Name, &self.name)
                            .value(FiscalYearFormField::Start, &self.start)
                            .value(FiscalYearFormField::End, &self.end)
                            .value(
                                FiscalYearFormField::IsActive,
                                if self.is_active { "on" } else { "" },
                            ),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Update",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            (button_delete(
                                FiscalYearDeletePostRouteTag::new(self.id),
                                "Delete Fiscal Year",
                                "Permanently delete this fiscal year?",
                            ))
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }

    fn sidebar(&self) -> Markup {
        fiscal_year_detail_menu(self.id, &self.name, "edit", true)
    }
}

impl RenderAppPane for FiscalYearFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.sidebar(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for FiscalYearFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Edit Fiscal Year — Uniquity", chrome, self.sidebar(), self.body())
    }
}

#[derive(Generic)]
pub struct FiscalYearCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub code: String,
    pub name: String,
    pub start: String,
    pub end: String,
    pub is_active: bool,
    pub error: String,
}

impl RenderTemplate for FiscalYearCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_uniquity_finance_fiscal_year.FiscalYearCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<FiscalYearCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Fiscal Year",
                subtitle: "Create a new fiscal year",
                attrs: form_hx_post_url::<FiscalYearCreateModalKey>(
                    &modal_create_post_url(
                        FiscalYearCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                    ),
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: FiscalYearForm::render_inputs(
                    &FormCtx::form::<FiscalYearForm>()
                        .value(FiscalYearFormField::Code, &self.code)
                        .value(FiscalYearFormField::Name, &self.name)
                        .value(FiscalYearFormField::Start, &self.start)
                        .value(FiscalYearFormField::End, &self.end)
                        .value(
                            FiscalYearFormField::IsActive,
                            if self.is_active { "on" } else { "" },
                        ),
                ),
                actions: html! {
                    (container_row("flex justify-end gap-2 mt-2", html! {
                        (button_submit(ButtonSubmit {
                            label: "Save Fiscal Year",
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
pub struct FiscalYearSelectPage {
    pub fiscal_years: ObjectList<FiscalYearRow>,
    pub filter_code: String,
    pub filter_name: String,
    pub target_input: String,
}

impl RenderPickerSelect<FiscalYearSelectTableKey, FiscalYearSelectModalKey> for FiscalYearSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Code", sort_url: None, push_url: false },
            TableColumnHeader { label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { label: "Start", sort_url: None, push_url: false },
            TableColumnHeader { label: "End", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .fiscal_years
            .items
            .iter()
            .map(|fy| TableRow {
                attrs: row_attr_select(&self.target_input, &fy.id.to_string(), &fy.name),
                cells: vec![
                    field_text(FieldText { value: &fy.code, classes: "" }),
                    field_text(FieldText { value: &fy.name, classes: "" }),
                    field_date(FieldDate { value: &fy.start, classes: "" }),
                    field_date(FieldDate { value: &fy.end, classes: "" }),
                ],
            })
            .collect();
        let filter = form(FormOpts {
            attrs: form_hx_get_route::<FiscalYearSelectTableKey, FiscalYearSelectRouteTag>(
                FiscalYearSelectRouteTag,
            ),
            inputs: FiscalYearFilterForm::render_inputs(
                &FormCtx::form::<FiscalYearFilterForm>()
                    .value(FiscalYearFilterFormField::Code, &self.filter_code)
                    .value(FiscalYearFilterFormField::Name, &self.filter_name),
            ),
            actions: html! {
                (container_row("flex gap-2", html! {
                    (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                    (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                }))
            },
            ..Default::default()
        });
        data_table_list::<FiscalYearSelectTableKey>(
            "Select Fiscal Year",
            table_button_filter(TableButtonFilter {
                panel: filter,
                ..Default::default()
            }),
            &headers,
            &rows,
            html! {},
        )
    }
}

impl RenderTemplate for FiscalYearSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}
