use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonSubmit, FieldText, FieldTitle, FormOpts,
        ObjectList, ShellChrome, SwapKey, TableButtonFilter, TableColumnHeader, TableRow, button_clear,
        button_delete, button_link, button_submit, container_column, container_row,
        data_table_list, detail, field_text, field_title, form,
        form_hx_get_picker_route, form_hx_get_route, form_hx_post_main, label_inline, row_attr_navigate_route, row_attr_select, table_button_filter,
    },
    html_form::{FormCtx, HtmlForm},
    picker::RenderPickerSelect,
    template::{RenderAppPane, RenderTemplate},
};

use crate::{
    entities::journal,
    forms::{
        JournalEntryForm, JournalEntryFormField, JournalFilterForm, JournalFilterFormField,
        JournalForm, JournalFormField,
    },
    keys::{
        JournalEntrySelectModalKey, JournalEntrySelectTableKey, JournalSelectModalKey,
        JournalSelectTableKey, JournalTableKey,
    },
    routes::{
        JournalCreateGetRouteTag, JournalCreatePostRouteTag, JournalDeletePostRouteTag,
        JournalDetailRouteTag, JournalEditGetRouteTag,
        JournalEditPostRouteTag, JournalEntryCreateGetRouteTag, JournalEntryCreatePostRouteTag,
        JournalEntryDetailRouteTag, JournalListRouteTag,
        JournalSelectRouteTag,
    },
};

use super::common::{
    app_scaffold, app_scaffold_with_sidebar, layout_main_content, layout_with_entity_sidebar,
    layout_with_sidebar, render_pagination, render_picker_pagination,
};
use crate::accounting_detail_menu::{DetailMenuNavItem, detail_sidebar_menu};

fn journal_detail_menu(id: i64, name: &str, active: &str, can_edit: bool) -> Markup {
    let menu_title = format!("Journal: {name}");
    let detail_url = JournalDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Journal Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Journal",
            url: JournalEditGetRouteTag::new(id).url(),
            active: active == "edit",
        });
    }
    detail_sidebar_menu(
        menu_title,
        "Back to Journals",
        JournalListRouteTag.url(),
        &nav,
        None,
        html! {},
    )
}

fn journal_entry_detail_menu(entry_id: i64, journal_id: i64) -> Markup {
    detail_sidebar_menu(
        format!("Journal entry #{entry_id}"),
        "Back to Journal",
        JournalDetailRouteTag::new(journal_id).url(),
        &[DetailMenuNavItem {
            title: "Entry Detail",
            url: JournalEntryDetailRouteTag::new(entry_id).url(),
            active: true,
        }],
        None,
        html! {},
    )
}

#[derive(Clone)]
pub struct JournalRow {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
    pub currency_label: String,
    pub journal_type: String,
}

#[derive(Clone)]
pub struct JournalEntryRow {
    pub id: i64,
    pub datetime: String,
    pub source_doc_label: String,
    pub journal_name: String,
    pub label: String,
}

#[derive(Clone)]
pub struct JournalEntryItemRow {
    pub datetime: String,
    pub account_label: String,
    pub amount: String,
}

fn journal_filter_form(
    name: &str,
    is_active: bool,
    currency_id: &str,
    journal_type: &str,
) -> Markup {
    let jt_choices = crate::forms::journal_type_filter_choices();
    form(FormOpts {
        attrs: form_hx_get_route::<JournalTableKey, JournalListRouteTag>(JournalListRouteTag),
        inputs: JournalFilterForm::render_inputs(
            &FormCtx::form::<JournalFilterForm>()
                .value(JournalFilterFormField::Name, name)
                .value(JournalFilterFormField::IsActive, if is_active { "on" } else { "" })
                .value(JournalFilterFormField::CurrencyId, currency_id)
                .value(JournalFilterFormField::JournalType, journal_type)
                .choices(JournalFilterFormField::JournalType, &jt_choices),
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
pub struct JournalListPage {
    pub journals: ObjectList<JournalRow>,
    pub filter_name: String,
    pub filter_is_active: bool,
    pub filter_currency_id: String,
    pub filter_journal_type: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl JournalListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Name", sort_url: None, push_url: true },
            TableColumnHeader { label: "Active", sort_url: None, push_url: true },
            TableColumnHeader { label: "Currency", sort_url: None, push_url: true },
            TableColumnHeader { label: "Type", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .journals
            .items
            .iter()
            .map(|j| {
                let active = if j.is_active { "Yes" } else { "No" };
                TableRow {
                    attrs: row_attr_navigate_route(JournalDetailRouteTag::new(j.id)),
                    cells: vec![
                        field_text(FieldText { value: &j.name, classes: "" }),
                        field_text(FieldText { value: active, classes: "" }),
                        field_text(FieldText { value: &j.currency_label, classes: "" }),
                        field_text(FieldText { value: &j.journal_type, classes: "" }),
                    ],
                }
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: journal_filter_form(
                    &self.filter_name,
                    self.filter_is_active,
                    &self.filter_currency_id,
                    &self.filter_journal_type,
                ),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (button_link(ButtonLink {
                    href: &JournalCreateGetRouteTag.url(),
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        let pagination = render_pagination::<JournalTableKey>(
            &self.path_and_query,
            self.journals.number,
            self.journals.num_pages,
        );
        data_table_list::<JournalTableKey>(
            "Journals",
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

impl RenderAppPane for JournalListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for JournalListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Journals — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct JournalDetailPage {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
    pub currency_id: i64,
    pub currency_label: String,
    pub journal_type: String,
    pub entries: ObjectList<JournalEntryRow>,
    pub can_edit: bool,
}

impl JournalDetailPage {
    fn entries_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "ID", sort_url: None, push_url: false },
            TableColumnHeader { label: "Date & time", sort_url: None, push_url: false },
            TableColumnHeader { label: "Source document type", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .entries
            .items
            .iter()
            .map(|e| TableRow {
                attrs: row_attr_navigate_route(JournalEntryDetailRouteTag::new(e.id)),
                cells: vec![
                    field_text(FieldText { value: &e.id.to_string(), classes: "" }),
                    field_text(FieldText { value: &e.datetime, classes: "" }),
                    field_text(FieldText { value: &e.source_doc_label, classes: "" }),
                ],
            })
            .collect();
        let mut actions = html! {};
        if self.can_edit {
            actions = html! {
                (button_link(ButtonLink {
                    href: &JournalEntryCreateGetRouteTag::new(self.id).url(),
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        data_table_list::<JournalTableKey>(
            "Journal entries",
            actions,
            &headers,
            &rows,
            html! {},
        )
    }

    fn body(&self) -> Markup {
        let active = if self.is_active { "Active" } else { "Inactive" };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (field_text(FieldText {
                        value: &format!("{active} · {}", self.journal_type),
                        classes: "text-base-content/70",
                    }))
                    (label_inline("Currency", field_text(FieldText { value: &self.currency_label, classes: "" })))
                    div class="mt-6" {
                        (self.entries_table())
                    }
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        journal_detail_menu(self.id, &self.name, "detail", self.can_edit)
    }
}

impl RenderAppPane for JournalDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for JournalDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Journal — Uniquity", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct JournalFormPage {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
    pub currency_id: String,
    pub currency_display: String,
    pub journal_type: String,
    pub is_edit: bool,
}

impl JournalFormPage {
    pub fn new(is_edit: bool) -> Self {
        Self {
            id: 0,
            name: String::new(),
            is_active: true,
            currency_id: String::new(),
            currency_display: String::new(),
            journal_type: "Debit".to_string(),
            is_edit,
        }
    }

    pub fn from_model(j: &journal::Model, currency_display: String, is_edit: bool) -> Self {
        Self {
            id: j.id,
            name: j.name.clone(),
            is_active: j.is_active,
            currency_id: j.currency_id.to_string(),
            currency_display,
            journal_type: j.journal_type.to_string(),
            is_edit,
        }
    }

    fn body(&self) -> Markup {
        let title = if self.is_edit {
            "Edit Journal"
        } else {
            "Create Journal"
        };
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(JournalEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(JournalCreatePostRouteTag)
                    },
                    inputs: JournalForm::render_inputs(
                        &FormCtx::form::<JournalForm>()
                            .value(JournalFormField::Name, &self.name)
                            .value(JournalFormField::IsActive, if self.is_active { "on" } else { "" })
                            .value(JournalFormField::CurrencyId, &self.currency_id)
                            .display(JournalFormField::CurrencyId, &self.currency_display)
                            .value(JournalFormField::JournalType, &self.journal_type)
                            .choices(
                                JournalFormField::JournalType,
                                &crate::forms::journal_type_choices(),
                            ),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save Journal",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    JournalDeletePostRouteTag::new(self.id),
                                    "Delete Journal",
                                    "Permanently delete this journal?",
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
            journal_detail_menu(self.id, &self.name, "edit", true)
        } else {
            crate::accounting_sidebar::accounting_sidebar()
        }
    }
}

impl RenderAppPane for JournalFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.sidebar(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for JournalFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Journal Form — Uniquity", chrome, self.sidebar(), self.body())
    }
}

#[derive(Generic)]
pub struct JournalSelectPage {
    pub journals: ObjectList<JournalRow>,
    pub filter_name: String,
    pub filter_is_active: bool,
    pub filter_currency_id: String,
    pub filter_journal_type: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<JournalSelectTableKey, JournalSelectModalKey> for JournalSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { label: "Currency", sort_url: None, push_url: false },
            TableColumnHeader { label: "Type", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .journals
            .items
            .iter()
            .map(|j| TableRow {
                attrs: row_attr_select(&self.target_input, &j.id.to_string(), &j.name),
                cells: vec![
                    field_text(FieldText { value: &j.name, classes: "" }),
                    field_text(FieldText { value: &j.currency_label, classes: "" }),
                    field_text(FieldText { value: &j.journal_type, classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        JournalSelectTableKey,
                        JournalSelectModalKey,
                        JournalSelectRouteTag,
                    >(JournalSelectRouteTag),
                    inputs: html! {
                        (JournalFilterForm::render_inputs(
                            &FormCtx::form::<JournalFilterForm>()
                                .value(JournalFilterFormField::Name, &self.filter_name)
                                .value(
                                    JournalFilterFormField::IsActive,
                                    if self.filter_is_active { "on" } else { "" },
                                )
                                .value(JournalFilterFormField::CurrencyId, &self.filter_currency_id)
                                .value(JournalFilterFormField::JournalType, &self.filter_journal_type)
                                .choices(
                                    JournalFilterFormField::JournalType,
                                    &crate::forms::journal_type_filter_choices(),
                                ),
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
        let pagination = render_picker_pagination::<JournalSelectModalKey>(
            &self.path_and_query,
            self.journals.number,
            self.journals.num_pages,
        );
        data_table_list::<JournalSelectTableKey>(
            "Select Journal",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for JournalSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct JournalEntryFormPage {
    pub journal_id: i64,
    pub journal_name: String,
    pub datetime: String,
    pub source_doc_id: String,
    pub source_doc_display: String,
}

impl JournalEntryFormPage {
    pub fn new(journal_id: i64, journal_name: String, datetime: String) -> Self {
        Self {
            journal_id,
            journal_name,
            datetime,
            source_doc_id: String::new(),
            source_doc_display: String::new(),
        }
    }

    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle {
                    value: &format!("New entry — {}", self.journal_name),
                    classes: "",
                }))
                (form(FormOpts {
                    attrs: form_hx_post_main(JournalEntryCreatePostRouteTag::new(self.journal_id)),
                    inputs: JournalEntryForm::render_inputs(
                        &FormCtx::form::<JournalEntryForm>()
                            .value(JournalEntryFormField::Datetime, &self.datetime)
                            .value(JournalEntryFormField::SourceDocId, &self.source_doc_id)
                            .display(JournalEntryFormField::SourceDocId, &self.source_doc_display),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save Entry",
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

impl RenderAppPane for JournalEntryFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for JournalEntryFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Journal Entry — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct JournalEntryDetailPage {
    pub id: i64,
    pub datetime: String,
    pub journal_id: i64,
    pub journal_label: String,
    pub source_doc_label: String,
    pub items: Vec<JournalEntryItemRow>,
}

impl JournalEntryDetailPage {
    fn items_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Date & time", sort_url: None, push_url: false },
            TableColumnHeader { label: "Account", sort_url: None, push_url: false },
            TableColumnHeader { label: "Amount", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .items
            .iter()
            .map(|item| TableRow {
                attrs: lariv_rs::components::HtmlAttrs::new(),
                cells: vec![
                    field_text(FieldText { value: &item.datetime, classes: "" }),
                    field_text(FieldText { value: &item.account_label, classes: "" }),
                    field_text(FieldText { value: &item.amount, classes: "text-end tabular-nums" }),
                ],
            })
            .collect();
        data_table_list::<JournalTableKey>(
            "Line items",
            html! {},
            &headers,
            &rows,
            html! {},
        )
    }

    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &format!("Journal entry #{}", self.id), classes: "" }))
                    (field_text(FieldText {
                        value: &self.datetime,
                        classes: "text-base-content/70",
                    }))
                    (label_inline("Journal", field_text(FieldText { value: &self.journal_label, classes: "" })))
                    (label_inline("Source document type", field_text(FieldText { value: &self.source_doc_label, classes: "" })))
                    div class="mt-6" {
                        (self.items_table())
                    }
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        journal_entry_detail_menu(self.id, self.journal_id)
    }
}

impl RenderAppPane for JournalEntryDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for JournalEntryDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Journal Entry — Uniquity", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct JournalEntrySelectPage {
    pub entries: ObjectList<JournalEntryRow>,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<JournalEntrySelectTableKey, JournalEntrySelectModalKey>
    for JournalEntrySelectPage
{
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "ID", sort_url: None, push_url: false },
            TableColumnHeader { label: "Date & time", sort_url: None, push_url: false },
            TableColumnHeader { label: "Journal", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .entries
            .items
            .iter()
            .map(|e| {
                let select_label = if e.label.is_empty() {
                    format!("{} · {}", e.journal_name, e.datetime)
                } else {
                    e.label.clone()
                };
                TableRow {
                    attrs: row_attr_select(&self.target_input, &e.id.to_string(), &select_label),
                    cells: vec![
                        field_text(FieldText { value: &e.id.to_string(), classes: "" }),
                        field_text(FieldText { value: &e.datetime, classes: "" }),
                        field_text(FieldText { value: &e.journal_name, classes: "" }),
                    ],
                }
            })
            .collect();
        let pagination = render_picker_pagination::<JournalEntrySelectModalKey>(
            &self.path_and_query,
            self.entries.number,
            self.entries.num_pages,
        );
        data_table_list::<JournalEntrySelectTableKey>(
            "Select Journal Entry",
            html! {},
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for JournalEntrySelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}
