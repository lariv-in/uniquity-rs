use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonSubmit, FieldText, FieldTitle,
        FormOpts, LayoutSidebar, ObjectList, PaginationPage, ShellChrome, ShellScaffold,
        SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter, TableColumnHeader,
        TablePagination, TableRow, button_clear, button_delete, button_link,
        button_submit, container_column, container_row, data_table_list,
        detail, field_text, field_title, form, form_hx_get_route, form_hx_post_main,
        label_inline, layout_sidebar, pagination_pages,
        row_attr_navigate_route, row_attr_select, shell_scaffold, table_button_filter,
        table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
};

use super::forms::{
    EmployeeFilterForm, EmployeeFilterFormField, EmployeeForm, EmployeeFormField, PointsForm,
    PointsFormField,
};
use super::keys::{
    EmployeeSelectTableKey, EmployeeTableKey, PointsTableKey,
};
use super::routes::{
    EmployeesCreateGetRouteTag, EmployeesCreatePostRouteTag, EmployeesDefaultRouteTag,
    EmployeesDeletePostRouteTag, EmployeesDetailRouteTag,
    EmployeesEditGetRouteTag, EmployeesEditPostRouteTag,
    PointsCreateGetRouteTag, PointsCreatePostRouteTag, PointsDetailRouteTag,
    PointsListRouteTag,
};
use super::scope::{EmployeeRow, PointsRow};

lariv_rs::define_register_items! {
    plugin: UniquityEmployeesTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        EmployeeListIdx: EmployeeListPageTag => EmployeeListPage,
        EmployeeDetailIdx: EmployeeDetailPageTag => EmployeeDetailPage,
        EmployeeFormIdx: EmployeeFormPageTag => EmployeeFormPage,
        EmployeeSelectIdx: EmployeeSelectPageTag => EmployeeSelectPage,
        PointsListIdx: PointsListPageTag => PointsListPage,
        PointsDetailIdx: PointsDetailPageTag => PointsDetailPage,
        PointsFormIdx: PointsFormPageTag => PointsFormPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityEmployeesTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn app_scaffold(title: &str, chrome: &ShellChrome, body: Markup) -> Markup {
    shell_scaffold(ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        body,
        ..Default::default()
    })
}

fn employee_filter_form(name: &str, email: &str) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<EmployeeTableKey, EmployeesDefaultRouteTag>(
            EmployeesDefaultRouteTag,
        ),
        inputs: EmployeeFilterForm::render_inputs(
            &FormCtx::form::<EmployeeFilterForm>()
                .value(EmployeeFilterFormField::Name, name)
                .value(EmployeeFilterFormField::Email, email),
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

#[derive(Generic)]
pub struct EmployeeListPage {
    pub employees: ObjectList<EmployeeRow>,
    pub filter_name: String,
    pub filter_email: String,
    pub path_and_query: String,
}

impl EmployeeListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Name", sort_url: None, push_url: true },
            TableColumnHeader { label: "Email", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .employees
            .items
            .iter()
            .map(|e| TableRow {
                attrs: row_attr_navigate_route(EmployeesDetailRouteTag::new(e.id)),
                cells: vec![
                    field_text(FieldText { value: &e.user_name, classes: "" }),
                    field_text(FieldText { value: &e.user_email, classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: employee_filter_form(&self.filter_name, &self.filter_email),
                ..Default::default()
            }))
            (button_link(ButtonLink {
                href: &EmployeesCreateGetRouteTag.url(),
                label: "New Employee",
                classes: "btn btn-primary btn-sm",
                ..Default::default()
            }))
            (button_link(ButtonLink {
                href: &PointsListRouteTag.url(),
                label: "Points",
                classes: "btn btn-outline btn-sm",
                ..Default::default()
            }))
        };
        let pagination = render_pagination::<EmployeeTableKey>(
            &self.path_and_query,
            self.employees.number,
            self.employees.num_pages,
        );
        data_table_list::<EmployeeTableKey>(
            "Employees",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }

    fn body(&self) -> Markup {
        html! {
            (container_column("", html! {
                (field_title(FieldTitle { value: "Employees", classes: "" }))
                (self.render_table())
            }))
        }
    }
}

impl RenderAppPane for EmployeeListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: html! {},
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for EmployeeListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Employees — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct EmployeeDetailPage {
    pub id: i64,
    pub user_name: String,
    pub user_email: String,
    pub total_points: String,
}

impl EmployeeDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.user_name, classes: "" }))
                    (label_inline("Email", field_text(FieldText { value: &self.user_email, classes: "" })))
                    (label_inline("Total points", field_text(FieldText { value: &self.total_points, classes: "" })))
                    (container_row("flex gap-2 mt-4", html! {
                        (button_link(ButtonLink {
                            href: &EmployeesEditGetRouteTag::new(self.id).url(),
                            label: "Edit",
                            classes: "btn btn-primary",
                            ..Default::default()
                        }))
                    }))
                }))
            }))
        }
    }
}

impl RenderAppPane for EmployeeDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: html! {},
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for EmployeeDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Employee — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct EmployeeFormPage {
    pub id: i64,
    pub user_id: i64,
    pub user_display: String,
    pub is_edit: bool,
}

impl EmployeeFormPage {
    fn body(&self) -> Markup {
        let title = if self.is_edit { "Edit Employee" } else { "Create Employee" };
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(EmployeesEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(EmployeesCreatePostRouteTag)
                    },
                    inputs: EmployeeForm::render_inputs(
                        &FormCtx::form::<EmployeeForm>()
                            .value(EmployeeFormField::UserId, self.user_id.to_string())
                            .display(EmployeeFormField::UserId, &self.user_display),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    EmployeesDeletePostRouteTag::new(self.id),
                                    "Delete",
                                    "Permanently delete this employee?",
                                ))
                            }
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }
}

impl RenderAppPane for EmployeeFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: html! {},
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for EmployeeFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Employee Form — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct EmployeeSelectPage {
    pub employees: ObjectList<EmployeeRow>,
    pub filter_name: String,
    pub filter_email: String,
    pub target_input: String,
}

impl EmployeeSelectPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "User", sort_url: None, push_url: false },
            TableColumnHeader { label: "Email", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .employees
            .items
            .iter()
            .map(|e| TableRow {
                attrs: row_attr_select(&self.target_input, &e.id.to_string(), &e.user_name),
                cells: vec![
                    field_text(FieldText { value: &e.user_name, classes: "" }),
                    field_text(FieldText { value: &e.user_email, classes: "" }),
                ],
            })
            .collect();
        data_table_list::<EmployeeSelectTableKey>(
            "Select employee",
            html! {},
            &headers,
            &rows,
            html! {},
        )
    }
}

impl RenderTemplate for EmployeeSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_table()
    }
}

#[derive(Generic)]
pub struct PointsListPage {
    pub points: ObjectList<PointsRow>,
    pub path_and_query: String,
}

impl PointsListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Points", sort_url: None, push_url: true },
            TableColumnHeader { label: "From", sort_url: None, push_url: true },
            TableColumnHeader { label: "To", sort_url: None, push_url: true },
            TableColumnHeader { label: "When", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .points
            .items
            .iter()
            .map(|p| TableRow {
                attrs: row_attr_navigate_route(PointsDetailRouteTag::new(p.id)),
                cells: vec![
                    field_text(FieldText { value: &p.points.to_string(), classes: "" }),
                    field_text(FieldText { value: &p.from_user_name, classes: "" }),
                    field_text(FieldText { value: &p.to_employee_name, classes: "" }),
                    field_text(FieldText { value: &p.created_at, classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (button_link(ButtonLink {
                href: &PointsCreateGetRouteTag.url(),
                label: "Award Points",
                classes: "btn btn-primary btn-sm",
                ..Default::default()
            }))
            (button_link(ButtonLink {
                href: &EmployeesDefaultRouteTag.url(),
                label: "Employees",
                classes: "btn btn-outline btn-sm",
                ..Default::default()
            }))
        };
        let pagination = render_pagination::<PointsTableKey>(
            &self.path_and_query,
            self.points.number,
            self.points.num_pages,
        );
        data_table_list::<PointsTableKey>("Points", actions, &headers, &rows, pagination)
    }

    fn body(&self) -> Markup {
        html! {
            (container_column("", html! {
                (field_title(FieldTitle { value: "Points Transactions", classes: "" }))
                (self.render_table())
            }))
        }
    }
}

impl RenderAppPane for PointsListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: html! {},
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for PointsListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Points — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct PointsDetailPage {
    pub id: i64,
    pub points: String,
    pub from_user_name: String,
    pub to_employee_name: String,
    pub created_at: String,
}

impl PointsDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: "Points Transaction", classes: "" }))
                    (label_inline("Points", field_text(FieldText { value: &self.points, classes: "" })))
                    (label_inline("From", field_text(FieldText { value: &self.from_user_name, classes: "" })))
                    (label_inline("To employee", field_text(FieldText { value: &self.to_employee_name, classes: "" })))
                    (label_inline("Created", field_text(FieldText { value: &self.created_at, classes: "" })))
                }))
            }))
        }
    }
}

impl RenderAppPane for PointsDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: html! {},
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for PointsDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Points — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct PointsFormPage {
    pub to_employee_id: i64,
    pub employee_display: String,
    pub points: String,
}

impl PointsFormPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Award Points", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(PointsCreatePostRouteTag),
                    inputs: PointsForm::render_inputs(
                        &FormCtx::form::<PointsForm>()
                            .value(PointsFormField::ToEmployeeId, self.to_employee_id.to_string())
                            .display(PointsFormField::ToEmployeeId, &self.employee_display)
                            .value(PointsFormField::Points, &self.points),
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit {
                            label: "Create",
                            classes: "btn-primary",
                            ..Default::default()
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }
}

impl RenderAppPane for PointsFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: html! {},
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for PointsFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Award Points — Uniquity", chrome, self.body())
    }
}
