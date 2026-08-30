use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonModalForm, ButtonSubmit, Crumb, DeleteConfirmation, FieldText,
        FieldTitle, FormOpts, LayoutMain, LayoutSidebar, ObjectList, PaginationPage, ShellChrome,
        ShellScaffold, SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter,
        TableColumnHeader, TablePagination, TableRow, breadcrumbs, button_clear, button_link,
        button_modal_form, button_submit, container_column, container_row, column_sort_url,
        data_table_list_refresh, delete_confirmation, detail, field_text,
        field_title, form, form_hx_get_route, form_hx_post_main, form_hx_post_selector,
        form_hx_post_url, label, layout_main, layout_sidebar, modal, modal_keyed, pagination_pages,
        page_size_only_filter_form, row_attr_navigate_route, row_attr_select, shell_scaffold,
        sort_indicator, table_button_filter, table_pagination, with_list_filter_common,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::modal_create_post_url,
};

use super::forms::{
    EmployeeFilterForm, EmployeeFilterFormField, EmployeeForm, EmployeeFormField, PointsForm,
    PointsFormField,
};
use super::keys::{
    EmployeeCreateModalKey, EmployeeDeleteModalKey, EmployeeSelectTableKey, EmployeeTableKey,
    PointsCreateModalKey, PointsTableKey,
};
use super::routes::{
    EmployeesCreateGetRouteTag, EmployeesCreatePostRouteTag, EmployeesDefaultRouteTag,
    EmployeesDeleteGetRouteTag, EmployeesDeletePostRouteTag, EmployeesDetailRouteTag,
    EmployeesEditGetRouteTag, EmployeesEditPostRouteTag, EmployeesSelectRouteTag,
    PointsCreateGetRouteTag, PointsCreatePostRouteTag, PointsDetailRouteTag, PointsListRouteTag,
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
        EmployeeCreateModalIdx: EmployeeCreateModalPageTag => EmployeeCreateModalPage,
        EmployeeSelectIdx: EmployeeSelectPageTag => EmployeeSelectPage,
        PointsListIdx: PointsListPageTag => PointsListPage,
        PointsDetailIdx: PointsDetailPageTag => PointsDetailPage,
        PointsCreateModalIdx: PointsCreateModalPageTag => PointsCreateModalPage,
        ConfirmDeleteIdx: EmployeeConfirmDeletePageTag => ConfirmDeletePage,
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

fn app_scaffold(title: &str, chrome: &ShellChrome, crumbs: Markup, body: Markup) -> Markup {
    shell_scaffold(ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        breadcrumbs: crumbs,
        body,
        ..Default::default()
    })
}

fn scaffold_pane(crumbs: Markup, body: Markup) -> lariv_rs::components::AppLayoutHtml {
    layout_sidebar(LayoutSidebar {
        sidebar: html! {},
        breadcrumbs: crumbs,
        content: body,
    })
}

fn scaffold_main(crumbs: Markup, body: Markup) -> lariv_rs::components::MainContentHtml {
    layout_main(LayoutMain {
        breadcrumbs: crumbs,
        content: body,
    })
}

fn employees_list_crumbs() -> Markup {
    breadcrumbs(&[Crumb {
        label: "Employees",
        href: None,
    }])
}

fn points_list_crumbs() -> Markup {
    let employees_url = EmployeesDefaultRouteTag.url();
    breadcrumbs(&[
        Crumb {
            label: "Employees",
            href: Some(&employees_url),
        },
        Crumb {
            label: "Points",
            href: None,
        },
    ])
}

fn employee_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    let list_url = EmployeesDefaultRouteTag.url();
    let detail_url = EmployeesDetailRouteTag::new(id).url();
    match action {
        None => breadcrumbs(&[
            Crumb {
                label: "Employees",
                href: Some(&list_url),
            },
            Crumb {
                label: name,
                href: None,
            },
        ]),
        Some(act) => breadcrumbs(&[
            Crumb {
                label: "Employees",
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

fn points_crumbs(_id: i64, label: &str) -> Markup {
    let employees_url = EmployeesDefaultRouteTag.url();
    let list_url = PointsListRouteTag.url();
    breadcrumbs(&[
        Crumb {
            label: "Employees",
            href: Some(&employees_url),
        },
        Crumb {
            label: "Points",
            href: Some(&list_url),
        },
        Crumb {
            label: label,
            href: None,
        },
    ])
}

fn employee_filter_form(name: &str, email: &str, page_size: u32) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<EmployeeTableKey, EmployeesDefaultRouteTag>(
            EmployeesDefaultRouteTag,
        ),
        inputs: with_list_filter_common(
            EmployeeFilterForm::render_inputs(
                &FormCtx::form::<EmployeeFilterForm>()
                    .value(EmployeeFilterFormField::Name, name)
                    .value(EmployeeFilterFormField::Email, email),
            ),
            page_size,
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

fn employee_select_filter_form(
    name: &str,
    email: &str,
    target_input: &str,
    page_size: u32,
) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<EmployeeSelectTableKey, EmployeesSelectRouteTag>(
            EmployeesSelectRouteTag,
        )
        .set("hx-push-url", "false"),
        inputs: html! {
            (with_list_filter_common(
                EmployeeFilterForm::render_inputs(
                    &FormCtx::form::<EmployeeFilterForm>()
                        .value(EmployeeFilterFormField::Name, name)
                        .value(EmployeeFilterFormField::Email, email),
                ),
                page_size,
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

#[derive(Generic)]
pub struct EmployeeListPage {
    pub employees: ObjectList<EmployeeRow>,
    pub filter_name: String,
    pub filter_email: String,
    pub path_and_query: String,
    pub page_size: u32,
}

impl EmployeeListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {  key: "Name",label: "Name", sort_url: None, push_url: true },
            TableColumnHeader {  key: "Email",label: "Email", sort_url: None, push_url: true },
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
                panel: employee_filter_form(&self.filter_name, &self.filter_email, self.page_size),
                ..Default::default()
            }))
            (button_modal_form(ButtonModalForm {
                name: "p_uniquity_employees.EmployeeCreateForm",
                href: &EmployeesCreateGetRouteTag.url(),
                form_post_url: &EmployeesCreateGetRouteTag.path(),
                modal_uid: EmployeeCreateModalKey::ID,
                icon_name: Some("plus"),
                classes: "btn-square btn-outline btn-sm",
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
        data_table_list_refresh::<EmployeeTableKey>(
            "Employees",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
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
        scaffold_pane(employees_list_crumbs(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(employees_list_crumbs(), self.body())
    }
}

impl RenderTemplate for EmployeeListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Employees — Uniquity", chrome, employees_list_crumbs(), self.body())
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
                    (label("Email", field_text(FieldText { value: &self.user_email, classes: "" })))
                    (label("Total points", field_text(FieldText { value: &self.total_points, classes: "" })))
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
        let crumbs = employee_crumbs(self.id, &self.user_name, None);
        scaffold_pane(crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(employee_crumbs(self.id, &self.user_name, None), self.body())
    }
}

impl RenderTemplate for EmployeeDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = employee_crumbs(self.id, &self.user_name, None);
        app_scaffold("Employee — Uniquity", chrome, crumbs, self.body())
    }
}

/// Edit employee form (full page). Create uses [`EmployeeCreateModalPage`].
#[derive(Generic)]
pub struct EmployeeFormPage {
    pub id: i64,
    pub user_id: i64,
    pub user_display: String,
}

impl EmployeeFormPage {
    fn body(&self) -> Markup {
        let delete_url = EmployeesDeleteGetRouteTag::new(self.id).url();
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Edit Employee", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(EmployeesEditPostRouteTag::new(self.id)),
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
                            (button_modal_form(ButtonModalForm {
                                label: "Delete",
                                icon_name: Some("trash"),
                                name: "p_uniquity_employees.EmployeeDeleteForm",
                                href: &delete_url,
                                form_post_url: &delete_url,
                                modal_uid: EmployeeDeleteModalKey::ID,
                                classes: "btn-error",
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

impl RenderAppPane for EmployeeFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = employee_crumbs(self.id, &self.user_display, Some("Edit"));
        scaffold_pane(crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            employee_crumbs(self.id, &self.user_display, Some("Edit")),
            self.body(),
        )
    }
}

impl RenderTemplate for EmployeeFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = employee_crumbs(self.id, &self.user_display, Some("Edit"));
        app_scaffold("Edit Employee — Uniquity", chrome, crumbs, self.body())
    }
}

#[derive(Generic)]
pub struct EmployeeCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub user_id: i64,
    pub user_display: String,
    pub error: String,
}

impl RenderTemplate for EmployeeCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_uniquity_employees.EmployeeCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<EmployeeCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Employee",
                subtitle: "Link a user as an employee",
                classes: "@container",
                attrs: form_hx_post_url::<EmployeeCreateModalKey>(
                    &modal_create_post_url(
                        EmployeesCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                    ),
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: EmployeeForm::render_inputs(
                    &FormCtx::form::<EmployeeForm>()
                        .value(EmployeeFormField::UserId, self.user_id.to_string())
                        .display(EmployeeFormField::UserId, &self.user_display),
                ),
                actions: html! {
                    (container_row("flex justify-end gap-2 mt-2", html! {
                        (button_submit(ButtonSubmit {
                            label: "Save",
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
pub struct EmployeeSelectPage {
    pub employees: ObjectList<EmployeeRow>,
    pub filter_name: String,
    pub filter_email: String,
    pub target_input: String,
    pub path_and_query: String,
    pub page_size: u32,
}

impl EmployeeSelectPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {  key: "User",label: "User", sort_url: None, push_url: false },
            TableColumnHeader {  key: "Email",label: "Email", sort_url: None, push_url: false },
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
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: employee_select_filter_form(
                    &self.filter_name,
                    &self.filter_email,
                    &self.target_input,
                    self.page_size,
                ),
                ..Default::default()
            }))
        };
        let pagination = render_pagination::<EmployeeSelectTableKey>(
            &self.path_and_query,
            self.employees.number,
            self.employees.num_pages,
        );
        data_table_list_refresh::<EmployeeSelectTableKey>(
            "Select employee",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
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
    pub sort: String,
    pub path_and_query: String,
    pub page_size: u32,
}

impl PointsListPage {
    pub fn render_table(&self) -> Markup {
        let points_sort = column_sort_url(&self.path_and_query, "Points", &self.sort);
        let when_sort = column_sort_url(&self.path_and_query, "When", &self.sort);
        let points_label = format!("Points{}", sort_indicator(&self.sort, "Points"));
        let when_label = format!("When{}", sort_indicator(&self.sort, "When"));
        let headers = [
            TableColumnHeader {
                key: "Points",
                label: &points_label,
                sort_url: Some(&points_sort),
                push_url: true,
            },
            TableColumnHeader {  key: "From",label: "From", sort_url: None, push_url: true },
            TableColumnHeader {  key: "To",label: "To", sort_url: None, push_url: true },
            TableColumnHeader {
                key: "When",
                label: &when_label,
                sort_url: Some(&when_sort),
                push_url: true,
            },
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
            (table_button_filter(TableButtonFilter {
                panel: page_size_only_filter_form::<PointsTableKey, PointsListRouteTag>(
                    self.page_size,
                ),
                ..Default::default()
            }))
            (button_modal_form(ButtonModalForm {
                name: "p_uniquity_employees.PointsCreateForm",
                href: &PointsCreateGetRouteTag.url(),
                form_post_url: &PointsCreateGetRouteTag.path(),
                modal_uid: PointsCreateModalKey::ID,
                icon_name: Some("plus"),
                classes: "btn-square btn-outline btn-sm",
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
        data_table_list_refresh::<PointsTableKey>(
            "Points",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
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
        scaffold_pane(points_list_crumbs(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(points_list_crumbs(), self.body())
    }
}

impl RenderTemplate for PointsListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Points — Uniquity", chrome, points_list_crumbs(), self.body())
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
                    (label("Points", field_text(FieldText { value: &self.points, classes: "" })))
                    (label("From", field_text(FieldText { value: &self.from_user_name, classes: "" })))
                    (label("To employee", field_text(FieldText { value: &self.to_employee_name, classes: "" })))
                    (label("Created", field_text(FieldText { value: &self.created_at, classes: "" })))
                }))
            }))
        }
    }
}

impl RenderAppPane for PointsDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let label = format!("{} points", self.points);
        let crumbs = points_crumbs(self.id, &label);
        scaffold_pane(crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        let label = format!("{} points", self.points);
        scaffold_main(points_crumbs(self.id, &label), self.body())
    }
}

impl RenderTemplate for PointsDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let label = format!("{} points", self.points);
        let crumbs = points_crumbs(self.id, &label);
        app_scaffold("Points — Uniquity", chrome, crumbs, self.body())
    }
}

#[derive(Generic)]
pub struct PointsCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub to_employee_id: i64,
    pub employee_display: String,
    pub points: String,
    pub error: String,
}

impl RenderTemplate for PointsCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_uniquity_employees.PointsCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<PointsCreateModalKey>(
            "",
            form(FormOpts {
                title: "Award Points",
                subtitle: "Create a points transaction",
                classes: "@container",
                attrs: form_hx_post_url::<PointsCreateModalKey>(
                    &modal_create_post_url(
                        PointsCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                    ),
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: PointsForm::render_inputs(
                    &FormCtx::form::<PointsForm>()
                        .value(PointsFormField::ToEmployeeId, self.to_employee_id.to_string())
                        .display(PointsFormField::ToEmployeeId, &self.employee_display)
                        .value(PointsFormField::Points, &self.points),
                ),
                actions: html! {
                    (container_row("flex justify-end gap-2 mt-2", html! {
                        (button_submit(ButtonSubmit {
                            label: "Create",
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
pub struct ConfirmDeletePage {
    pub modal_uid: String,
    pub message: String,
    pub form_name: String,
    pub id: i64,
    pub error: String,
}

impl RenderTemplate for ConfirmDeletePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let target = if self.modal_uid.is_empty() {
            format!("#{}", EmployeeDeleteModalKey::ID)
        } else {
            format!("#{}", self.modal_uid)
        };
        let uid = if self.modal_uid.is_empty() {
            EmployeeDeleteModalKey::ID
        } else {
            self.modal_uid.as_str()
        };
        modal(lariv_rs::components::Modal {
            uid,
            children: delete_confirmation(DeleteConfirmation {
                title: "Confirm Deletion",
                message: &self.message,
                attrs: form_hx_post_selector(
                    &EmployeesDeletePostRouteTag::new(self.id).url(),
                    &target,
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
