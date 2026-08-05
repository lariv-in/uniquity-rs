use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonSubmit, FieldText, FieldTitle, FormOpts,
        ObjectList, ShellChrome, TableButtonFilter, TableColumnHeader, TableRow, button_clear,
        button_delete, button_link, button_submit, container_column, container_row,
        data_table_list, detail, field_text, field_title, form,
        form_hx_get_picker_route, form_hx_get_route, form_hx_post_main, label_inline, row_attr_navigate_route, row_attr_select, table_button_filter,
    },
    html_form::{FormCtx, HtmlForm},
    picker::RenderPickerSelect,
    template::{RenderAppPane, RenderTemplate},
};

use crate::{
    entities::currency,
    forms::{
        CurrencyFilterForm, CurrencyFilterFormField, CurrencyForm, CurrencyFormField,
        CurrencySelectionFilterForm, CurrencySelectionFilterFormField,
    },
    keys::{CurrencySelectModalKey, CurrencySelectTableKey, CurrencyTableKey},
    routes::{
        CurrencyCreateGetRouteTag, CurrencyCreatePostRouteTag, CurrencyDeletePostRouteTag,
        CurrencyDetailRouteTag, CurrencyEditGetRouteTag,
        CurrencyEditPostRouteTag, CurrencyListRouteTag, CurrencySelectRouteTag,
    },
};

use super::common::{
    app_scaffold, app_scaffold_with_sidebar, layout_main_content, layout_with_entity_sidebar,
    layout_with_sidebar, render_pagination, render_picker_pagination,
};
use crate::accounting_detail_menu::{DetailMenuNavItem, detail_sidebar_menu};

fn currency_detail_menu(id: i64, name: &str, active: &str, can_edit: bool) -> Markup {
    let menu_title = format!("Currency: {name}");
    let detail_url = CurrencyDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Currency Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Currency",
            url: CurrencyEditGetRouteTag::new(id).url(),
            active: active == "edit",
        });
    }
    detail_sidebar_menu(
        menu_title,
        "Back to Currencies",
        CurrencyListRouteTag.url(),
        &nav,
        None,
        html! {},
    )
}

#[derive(Clone)]
pub struct CurrencyRow {
    pub id: i64,
    pub code: i32,
    pub name: String,
    pub symbol: String,
    pub minor_unit: i32,
}

fn currency_filter_form(
    code: &str,
    name: &str,
    symbol: &str,
    minor_unit: &str,
) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<CurrencyTableKey, CurrencyListRouteTag>(CurrencyListRouteTag),
        inputs: CurrencyFilterForm::render_inputs(
            &FormCtx::form::<CurrencyFilterForm>()
                .value(CurrencyFilterFormField::Code, code)
                .value(CurrencyFilterFormField::Name, name)
                .value(CurrencyFilterFormField::Symbol, symbol)
                .value(CurrencyFilterFormField::MinorUnit, minor_unit),
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

#[derive(Generic)]
pub struct CurrencyListPage {
    pub currencies: ObjectList<CurrencyRow>,
    pub filter_code: String,
    pub filter_name: String,
    pub filter_symbol: String,
    pub filter_minor_unit: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl CurrencyListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Code", sort_url: None, push_url: true },
            TableColumnHeader { label: "Name", sort_url: None, push_url: true },
            TableColumnHeader { label: "Symbol", sort_url: None, push_url: true },
            TableColumnHeader { label: "Minor unit", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .currencies
            .items
            .iter()
            .map(|c| TableRow {
                attrs: row_attr_navigate_route(CurrencyDetailRouteTag::new(c.id)),
                cells: vec![
                    field_text(FieldText {
                        value: &c.code.to_string(),
                        classes: "",
                    }),
                    field_text(FieldText { value: &c.name, classes: "" }),
                    field_text(FieldText { value: &c.symbol, classes: "" }),
                    field_text(FieldText {
                        value: &c.minor_unit.to_string(),
                        classes: "",
                    }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: currency_filter_form(
                    &self.filter_code,
                    &self.filter_name,
                    &self.filter_symbol,
                    &self.filter_minor_unit,
                ),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (button_link(ButtonLink {
                    href: &CurrencyCreateGetRouteTag.url(),
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        let pagination = render_pagination::<CurrencyTableKey>(
            &self.path_and_query,
            self.currencies.number,
            self.currencies.num_pages,
        );
        data_table_list::<CurrencyTableKey>(
            "Currencies",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }

    fn body(&self) -> Markup {
        self.render_table()
    }
}

impl RenderAppPane for CurrencyListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for CurrencyListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Currencies — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct CurrencyDetailPage {
    pub id: i64,
    pub code: i32,
    pub name: String,
    pub symbol: String,
    pub minor_unit: i32,
    pub can_edit: bool,
}

impl CurrencyDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (field_text(FieldText {
                        value: &format!("ISO 4217 code {}", self.code),
                        classes: "text-base-content/70",
                    }))
                    (label_inline("Symbol", field_text(FieldText { value: &self.symbol, classes: "" })))
                    (label_inline("Minor unit", field_text(FieldText { value: &self.minor_unit.to_string(), classes: "" })))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        currency_detail_menu(self.id, &self.name, "detail", self.can_edit)
    }
}

impl RenderAppPane for CurrencyDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for CurrencyDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Currency — Uniquity", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct CurrencyFormPage {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub minor_unit: String,
    pub is_edit: bool,
}

impl CurrencyFormPage {
    pub fn new(is_edit: bool) -> Self {
        Self {
            id: 0,
            code: String::new(),
            name: String::new(),
            symbol: String::new(),
            minor_unit: String::new(),
            is_edit,
        }
    }

    pub fn from_model(c: &currency::Model, is_edit: bool) -> Self {
        Self {
            id: c.id,
            code: c.code.to_string(),
            name: c.name.clone(),
            symbol: c.symbol.clone(),
            minor_unit: c.minor_unit.to_string(),
            is_edit,
        }
    }

    fn body(&self) -> Markup {
        let title = if self.is_edit {
            "Edit Currency"
        } else {
            "Create Currency"
        };
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(CurrencyEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(CurrencyCreatePostRouteTag)
                    },
                    inputs: CurrencyForm::render_inputs(
                        &FormCtx::form::<CurrencyForm>()
                            .value(CurrencyFormField::Code, &self.code)
                            .value(CurrencyFormField::Name, &self.name)
                            .value(CurrencyFormField::Symbol, &self.symbol)
                            .value(CurrencyFormField::MinorUnit, &self.minor_unit),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save Currency",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    CurrencyDeletePostRouteTag::new(self.id),
                                    "Delete Currency",
                                    "Permanently delete this currency?",
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
            currency_detail_menu(self.id, &self.name, "edit", true)
        } else {
            crate::accounting_sidebar::accounting_sidebar()
        }
    }
}

impl RenderAppPane for CurrencyFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.sidebar(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for CurrencyFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Currency Form — Uniquity", chrome, self.sidebar(), self.body())
    }
}

#[derive(Generic)]
pub struct CurrencySelectPage {
    pub currencies: ObjectList<CurrencyRow>,
    pub filter_code: String,
    pub filter_name: String,
    pub filter_symbol: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<CurrencySelectTableKey, CurrencySelectModalKey> for CurrencySelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Code", sort_url: None, push_url: false },
            TableColumnHeader { label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { label: "Symbol", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .currencies
            .items
            .iter()
            .map(|c| {
                let label = format!("{} — {}", c.code, c.name);
                TableRow {
                    attrs: row_attr_select(&self.target_input, &c.id.to_string(), &label),
                    cells: vec![
                        field_text(FieldText {
                            value: &c.code.to_string(),
                            classes: "",
                        }),
                        field_text(FieldText { value: &c.name, classes: "" }),
                        field_text(FieldText { value: &c.symbol, classes: "" }),
                    ],
                }
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        CurrencySelectTableKey,
                        CurrencySelectModalKey,
                        CurrencySelectRouteTag,
                    >(CurrencySelectRouteTag),
                    inputs: html! {
                        (CurrencySelectionFilterForm::render_inputs(
                            &FormCtx::form::<CurrencySelectionFilterForm>()
                                .value(CurrencySelectionFilterFormField::Code, &self.filter_code)
                                .value(CurrencySelectionFilterFormField::Name, &self.filter_name)
                                .value(CurrencySelectionFilterFormField::Symbol, &self.filter_symbol),
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
        let pagination = render_picker_pagination::<CurrencySelectModalKey>(
            &self.path_and_query,
            self.currencies.number,
            self.currencies.num_pages,
        );
        data_table_list::<CurrencySelectTableKey>(
            "Select Currency",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for CurrencySelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}
