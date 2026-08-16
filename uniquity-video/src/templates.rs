use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonLink, ButtonModalForm, ButtonSubmit, Crumb, FieldText, FieldTitle, FormOpts,
        LayoutMain, LayoutSidebar, ManyToManyItem, ObjectList, PaginationPage, ShellChrome,
        ShellScaffold, SidebarMenu, SidebarMenuItem, SlotCapability, SlotRegistrar, SwapKey,
        TableColumnHeader, TablePagination, TableRow, breadcrumbs, button_delete, button_link,
        button_modal_form, button_submit, column_sort_url, container_column, container_row,
        data_table_list, data_table_list_refresh, detail, field_text, field_title, form,
        form_hx_get_route, form_hx_post_main, form_hx_post_url, label, layout_main,
        layout_sidebar, modal_keyed, pagination_pages, row_attr_navigate_route, row_attr_select,
        shell_scaffold, sidebar_menu, sidebar_menu_item, sort_indicator, table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::modal_create_post_url,
};
use uniquity_employees::{
    routes::EmployeesDetailRouteTag,
    scope::EmployeeRow,
};

use super::forms::{
    EditedVideoForm, EditedVideoFormField, EditorPointsForm, EditorPointsFormField,
    PublishedVideoForm, PublishedVideoFormField, RawFootageFilterForm, RawFootageFilterFormField,
    RawFootageForm, RawFootageFormField,
};
use super::keys::{
    EditedCreateModalKey, EditedVideoSelectTableKey, EditedVideoTableKey,
    PublishedCreateModalKey, PublishedVideoSelectTableKey, PublishedVideoTableKey,
    RawCreateModalKey, RawFootageSelectTableKey, RawFootageTableKey,
    VideoEmployeeSelectTableKey,
};
use super::routes::{
    EditedCreateGetRouteTag, EditedCreatePostRouteTag,
    EditedDeletePostRouteTag, EditedDetailRouteTag, EditedEditGetRouteTag, EditedEditPostRouteTag,
    EditedListRouteTag, PublishedCreateGetRouteTag, PublishedCreatePostRouteTag,
    PublishedDeletePostRouteTag, PublishedDetailRouteTag,
    PublishedEditGetRouteTag, PublishedEditPostRouteTag, PublishedEditorPointsGetRouteTag,
    PublishedEditorPointsPostRouteTag, PublishedListRouteTag,
    RawCreateGetRouteTag, RawCreatePostRouteTag, RawDeletePostRouteTag,
    RawDetailRouteTag, RawEditGetRouteTag, RawEditPostRouteTag,
    RawListRouteTag, VideoHubRouteTag,
};
use super::scope::{EditedVideoRow, PublishedVideoRow, RawFootageRow};

lariv_rs::define_register_items! {
    plugin: UniquityVideoTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        HubIdx: HubPageTag => HubPage,
        RawListIdx: RawListPageTag => RawListPage,
        RawDetailIdx: RawDetailPageTag => RawDetailPage,
        RawFormIdx: RawFormPageTag => RawFormPage,
        RawCreateModalIdx: RawCreateModalPageTag => RawCreateModalPage,
        RawSelectIdx: RawSelectPageTag => RawSelectPage,
        VideoEmployeeSelectIdx: VideoEmployeeSelectPageTag => VideoEmployeeSelectPage,
        EditedListIdx: EditedListPageTag => EditedListPage,
        EditedDetailIdx: EditedDetailPageTag => EditedDetailPage,
        EditedFormIdx: EditedFormPageTag => EditedFormPage,
        EditedCreateModalIdx: EditedCreateModalPageTag => EditedCreateModalPage,
        EditedSelectIdx: EditedSelectPageTag => EditedSelectPage,
        PublishedListIdx: PublishedListPageTag => PublishedListPage,
        PublishedDetailIdx: PublishedDetailPageTag => PublishedDetailPage,
        PublishedFormIdx: PublishedFormPageTag => PublishedFormPage,
        PublishedCreateModalIdx: PublishedCreateModalPageTag => PublishedCreateModalPage,
        PublishedSelectIdx: PublishedSelectPageTag => PublishedSelectPage,
        EditorPointsIdx: EditorPointsPageTag => EditorPointsPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityVideoTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn app_scaffold(
    title: &str,
    chrome: &ShellChrome,
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> Markup {
    shell_scaffold(ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        sidebar,
        breadcrumbs: crumbs,
        body,
        ..Default::default()
    })
}

fn scaffold_pane(sidebar: Markup, crumbs: Markup, body: Markup) -> lariv_rs::components::AppLayoutHtml {
    layout_sidebar(LayoutSidebar {
        sidebar,
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

fn video_hub_crumbs() -> Markup {
    breadcrumbs(&[Crumb {
        label: "Video",
        href: None,
    }])
}

fn raw_list_crumbs() -> Markup {
    let hub_url = VideoHubRouteTag.url();
    breadcrumbs(&[
        Crumb {
            label: "Video",
            href: Some(&hub_url),
        },
        Crumb {
            label: "Raw footage",
            href: None,
        },
    ])
}

fn raw_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let hub_url = VideoHubRouteTag.url();
    let list_url = RawListRouteTag.url();
    let detail_url = RawDetailRouteTag::new(id).url();
    match action {
        None => breadcrumbs(&[
            Crumb {
                label: "Video",
                href: Some(&hub_url),
            },
            Crumb {
                label: "Raw footage",
                href: Some(&list_url),
            },
            Crumb {
                label: title,
                href: None,
            },
        ]),
        Some(act) => breadcrumbs(&[
            Crumb {
                label: "Video",
                href: Some(&hub_url),
            },
            Crumb {
                label: "Raw footage",
                href: Some(&list_url),
            },
            Crumb {
                label: title,
                href: Some(&detail_url),
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
    }
}

fn edited_list_crumbs() -> Markup {
    let hub_url = VideoHubRouteTag.url();
    breadcrumbs(&[
        Crumb {
            label: "Video",
            href: Some(&hub_url),
        },
        Crumb {
            label: "Edited videos",
            href: None,
        },
    ])
}

fn edited_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let hub_url = VideoHubRouteTag.url();
    let list_url = EditedListRouteTag.url();
    let detail_url = EditedDetailRouteTag::new(id).url();
    match action {
        None => breadcrumbs(&[
            Crumb {
                label: "Video",
                href: Some(&hub_url),
            },
            Crumb {
                label: "Edited videos",
                href: Some(&list_url),
            },
            Crumb {
                label: title,
                href: None,
            },
        ]),
        Some(act) => breadcrumbs(&[
            Crumb {
                label: "Video",
                href: Some(&hub_url),
            },
            Crumb {
                label: "Edited videos",
                href: Some(&list_url),
            },
            Crumb {
                label: title,
                href: Some(&detail_url),
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
    }
}

fn published_list_crumbs() -> Markup {
    let hub_url = VideoHubRouteTag.url();
    breadcrumbs(&[
        Crumb {
            label: "Video",
            href: Some(&hub_url),
        },
        Crumb {
            label: "Published",
            href: None,
        },
    ])
}

fn published_crumbs(id: i64, label: &str, action: Option<&str>) -> Markup {
    let hub_url = VideoHubRouteTag.url();
    let list_url = PublishedListRouteTag.url();
    let detail_url = PublishedDetailRouteTag::new(id).url();
    match action {
        None => breadcrumbs(&[
            Crumb {
                label: "Video",
                href: Some(&hub_url),
            },
            Crumb {
                label: "Published",
                href: Some(&list_url),
            },
            Crumb {
                label: label,
                href: None,
            },
        ]),
        Some(act) => breadcrumbs(&[
            Crumb {
                label: "Video",
                href: Some(&hub_url),
            },
            Crumb {
                label: "Published",
                href: Some(&list_url),
            },
            Crumb {
                label: label,
                href: Some(&detail_url),
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
    }
}

fn main_menu(active: &str) -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Video editors",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Overview",
                url: &VideoHubRouteTag.url(),
                icon_name: Some("home"),
                active: active == "hub",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Raw footage",
                url: &RawListRouteTag.url(),
                icon_name: Some("folder"),
                active: active == "raw",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Edited videos",
                url: &EditedListRouteTag.url(),
                icon_name: Some("scissors"),
                active: active == "edited",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Published",
                url: &PublishedListRouteTag.url(),
                icon_name: Some("play"),
                active: active == "published",
                ..Default::default()
            }))
        },
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

#[derive(Clone, Copy, Default)]
pub struct HubPage;

impl HubPage {
    fn hub_body(&self) -> Markup {
        html! {
            (container_column("p-6 max-w-2xl", html! {
                (field_title(FieldTitle { value: "Video pipeline", classes: "" }))
                (field_text(FieldText {
                    value: "Manage raw footage, edited outputs, and YouTube publications from the sidebar.",
                    classes: "",
                }))
            }))
        }
    }
}

impl RenderAppPane for HubPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(main_menu("hub"), video_hub_crumbs(), self.hub_body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(video_hub_crumbs(), self.hub_body())
    }
}

impl RenderTemplate for HubPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Video — Uniquity",
            chrome,
            main_menu("hub"),
            video_hub_crumbs(),
            self.hub_body(),
        )
    }
}

#[derive(Generic)]
pub struct RawListPage {
    pub items: ObjectList<RawFootageRow>,
    pub filter_title: String,
    pub sort: String,
    pub path_and_query: String,
}

impl RawListPage {
    pub fn render_table(&self) -> Markup {
        let title_sort = column_sort_url(&self.path_and_query, "Title", &self.sort);
        let title_label = format!("Title{}", sort_indicator(&self.sort, "Title"));
        let headers = [
            TableColumnHeader {
                key: "Title",
                label: &title_label,
                sort_url: Some(&title_sort),
                push_url: true,
            },
            TableColumnHeader {  key: "AssignedTo",label: "Assigned to", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_navigate_route(RawDetailRouteTag::new(r.id)),
                cells: vec![
                    field_text(FieldText { value: &r.title, classes: "" }),
                    field_text(FieldText { value: &r.assigned_to_name, classes: "" }),
                ],
            })
            .collect();
        let filter = form(FormOpts {
            attrs: form_hx_get_route::<RawFootageTableKey, RawListRouteTag>(RawListRouteTag),
            inputs: RawFootageFilterForm::render_inputs(
                &FormCtx::form::<RawFootageFilterForm>()
                    .value(RawFootageFilterFormField::Title, &self.filter_title),
            ),
            actions: button_submit(ButtonSubmit { label: "Filter", ..Default::default() }),
            ..Default::default()
        });
        let actions = html! {
            (filter)
            (button_modal_form(ButtonModalForm {
                name: "p_uniquity_video.RawCreateForm",
                href: &RawCreateGetRouteTag.url(),
                form_post_url: &RawCreateGetRouteTag.path(),
                modal_uid: RawCreateModalKey::ID,
                icon_name: Some("plus"),
                classes: "btn-square btn-outline btn-sm",
                ..Default::default()
            }))
        };
        let pagination = render_pagination::<RawFootageTableKey>(
            &self.path_and_query,
            self.items.number,
            self.items.num_pages,
        );
        data_table_list_refresh::<RawFootageTableKey>(
            "Raw footage",
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
                (field_title(FieldTitle { value: "Raw footage", classes: "" }))
                (self.render_table())
            }))
        }
    }
}

impl RenderAppPane for RawListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(main_menu("raw"), raw_list_crumbs(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(raw_list_crumbs(), self.body())
    }
}

impl RenderTemplate for RawListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Raw footage — Uniquity",
            chrome,
            main_menu("raw"),
            raw_list_crumbs(),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct RawDetailPage {
    pub id: i64,
    pub title: String,
    pub assigned_to_name: String,
    pub file_names: Vec<String>,
}

impl RawDetailPage {
    fn body(&self) -> Markup {
        let files = self
            .file_names
            .iter()
            .map(|n| field_text(FieldText { value: n, classes: "" }))
            .collect::<Vec<_>>();
        html! {
            (detail(html! {
                (container_column("p-4 gap-2", html! {
                    (field_title(FieldTitle { value: &self.title, classes: "" }))
                    (label("Assigned to", field_text(FieldText {
                        value: &self.assigned_to_name,
                        classes: "",
                    })))
                    (label("Files", html! { @for f in &files { (f) } }))
                    (button_link(ButtonLink {
                        href: &RawEditGetRouteTag::new(self.id).url(),
                        label: "Edit",
                        classes: "btn btn-primary mt-4",
                        ..Default::default()
                    }))
                }))
            }))
        }
    }
}

impl RenderAppPane for RawDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = raw_crumbs(self.id, &self.title, None);
        scaffold_pane(main_menu("raw"), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(raw_crumbs(self.id, &self.title, None), self.body())
    }
}

impl RenderTemplate for RawDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = raw_crumbs(self.id, &self.title, None);
        app_scaffold(
            "Raw footage — Uniquity",
            chrome,
            main_menu("raw"),
            crumbs,
            self.body(),
        )
    }
}

/// Edit raw footage form (full page). Create uses [`RawCreateModalPage`].
#[derive(Generic)]
pub struct RawFormPage {
    pub id: i64,
    pub title: String,
    pub assigned_to_id: i64,
    pub assigned_display: String,
    pub file_items: Vec<ManyToManyItem>,
}

impl RawFormPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Edit raw footage", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(RawEditPostRouteTag::new(self.id)),
                    inputs: RawFootageForm::render_inputs(
                        &FormCtx::form::<RawFootageForm>()
                            .value(RawFootageFormField::Title, &self.title)
                            .value(RawFootageFormField::AssignedToId, self.assigned_to_id.to_string())
                            .display(RawFootageFormField::AssignedToId, &self.assigned_display)
                            .m2m(RawFootageFormField::Files, &self.file_items),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Update",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            (button_delete(
                                RawDeletePostRouteTag::new(self.id),
                                "Delete",
                                "Permanently delete this raw footage?",
                            ))
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }
}

impl RenderAppPane for RawFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = raw_crumbs(self.id, &self.title, Some("Edit"));
        scaffold_pane(main_menu("raw"), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(raw_crumbs(self.id, &self.title, Some("Edit")), self.body())
    }
}

impl RenderTemplate for RawFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = raw_crumbs(self.id, &self.title, Some("Edit"));
        app_scaffold(
            "Edit raw footage — Uniquity",
            chrome,
            main_menu("raw"),
            crumbs,
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct RawCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub title: String,
    pub assigned_to_id: i64,
    pub assigned_display: String,
    pub file_items: Vec<ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for RawCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_uniquity_video.RawCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<RawCreateModalKey>(
            "",
            form(FormOpts {
                title: "New raw footage",
                subtitle: "Create a raw footage record",
                classes: "@container",
                attrs: form_hx_post_url::<RawCreateModalKey>(
                    &modal_create_post_url(
                        RawCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                    ),
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: RawFootageForm::render_inputs(
                    &FormCtx::form::<RawFootageForm>()
                        .value(RawFootageFormField::Title, &self.title)
                        .value(
                            RawFootageFormField::AssignedToId,
                            self.assigned_to_id.to_string(),
                        )
                        .display(RawFootageFormField::AssignedToId, &self.assigned_display)
                        .m2m(RawFootageFormField::Files, &self.file_items),
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
pub struct RawSelectPage {
    pub items: ObjectList<RawFootageRow>,
    pub filter_title: String,
    pub sort: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RawSelectPage {
    pub fn render_table(&self) -> Markup {
        let title_sort = column_sort_url(&self.path_and_query, "Title", &self.sort);
        let title_label = format!("Title{}", sort_indicator(&self.sort, "Title"));
        let headers = [TableColumnHeader {
            key: "Title",
            label: &title_label,
            sort_url: Some(&title_sort),
            push_url: false,
        }];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_select(&self.target_input, &r.id.to_string(), &r.title),
                cells: vec![field_text(FieldText { value: &r.title, classes: "" })],
            })
            .collect();
        data_table_list_refresh::<RawFootageSelectTableKey>(
            "Select raw footage",
            html! {},
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for RawSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_table()
    }
}

#[derive(Generic)]
pub struct VideoEmployeeSelectPage {
    pub employees: ObjectList<EmployeeRow>,
    pub filter_name: String,
    pub filter_email: String,
    pub target_input: String,
}

impl VideoEmployeeSelectPage {
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
        data_table_list::<VideoEmployeeSelectTableKey>(
            "Select employee",
            html! {},
            &headers,
            &rows,
            html! {},
        )
    }
}

impl RenderTemplate for VideoEmployeeSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_table()
    }
}

#[derive(Generic)]
pub struct EditedListPage {
    pub items: ObjectList<EditedVideoRow>,
    pub path_and_query: String,
}

impl EditedListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {  key: "RawTitle",label: "Raw title", sort_url: None, push_url: true },
            TableColumnHeader {  key: "OutputFile",label: "Output file", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_navigate_route(EditedDetailRouteTag::new(r.id)),
                cells: vec![
                    field_text(FieldText { value: &r.raw_title, classes: "" }),
                    field_text(FieldText { value: &r.output_name, classes: "" }),
                ],
            })
            .collect();
        let actions = button_modal_form(ButtonModalForm {
            name: "p_uniquity_video.EditedCreateForm",
            href: &EditedCreateGetRouteTag.url(),
            form_post_url: &EditedCreateGetRouteTag.path(),
            modal_uid: EditedCreateModalKey::ID,
            icon_name: Some("plus"),
            classes: "btn-square btn-outline btn-sm",
            ..Default::default()
        });
        let pagination = render_pagination::<EditedVideoTableKey>(
            &self.path_and_query,
            self.items.number,
            self.items.num_pages,
        );
        data_table_list_refresh::<EditedVideoTableKey>(
            "Edited videos",
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
                (field_title(FieldTitle { value: "Edited videos", classes: "" }))
                (self.render_table())
            }))
        }
    }
}

impl RenderAppPane for EditedListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(main_menu("edited"), edited_list_crumbs(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(edited_list_crumbs(), self.body())
    }
}

impl RenderTemplate for EditedListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Edited — Uniquity",
            chrome,
            main_menu("edited"),
            edited_list_crumbs(),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct EditedDetailPage {
    pub id: i64,
    pub raw_footage_id: i64,
    pub raw_title: String,
    pub assigned_to_id: i64,
    pub assigned_to_name: String,
    pub raw_file_names: Vec<String>,
    pub output_name: String,
}

impl EditedDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("p-4 gap-2", html! {
                    (label("Raw footage", button_link(ButtonLink {
                        href: &RawDetailRouteTag::new(self.raw_footage_id).url(),
                        label: &self.raw_title,
                        classes: "link link-hover",
                        ..Default::default()
                    })))
                    (label("Assigned to", button_link(ButtonLink {
                        href: &EmployeesDetailRouteTag::new(self.assigned_to_id).url(),
                        label: &self.assigned_to_name,
                        classes: "link link-hover",
                        ..Default::default()
                    })))
                    (label("Output file", field_text(FieldText {
                        value: &self.output_name,
                        classes: "",
                    })))
                    (button_link(ButtonLink {
                        href: &EditedEditGetRouteTag::new(self.id).url(),
                        label: "Edit",
                        classes: "btn btn-primary mt-4",
                        ..Default::default()
                    }))
                }))
            }))
        }
    }
}

impl RenderAppPane for EditedDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = edited_crumbs(self.id, &self.raw_title, None);
        scaffold_pane(main_menu("edited"), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(edited_crumbs(self.id, &self.raw_title, None), self.body())
    }
}

impl RenderTemplate for EditedDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = edited_crumbs(self.id, &self.raw_title, None);
        app_scaffold(
            "Edited video — Uniquity",
            chrome,
            main_menu("edited"),
            crumbs,
            self.body(),
        )
    }
}

/// Edit edited video form (full page). Create uses [`EditedCreateModalPage`].
#[derive(Generic)]
pub struct EditedFormPage {
    pub id: i64,
    pub raw_footage_id: i64,
    pub raw_display: String,
    pub edited_v_node_id: i64,
    pub vnode_display: String,
}

impl EditedFormPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Edit edited video", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(EditedEditPostRouteTag::new(self.id)),
                    inputs: EditedVideoForm::render_inputs(
                        &FormCtx::form::<EditedVideoForm>()
                            .value(EditedVideoFormField::RawFootageId, self.raw_footage_id.to_string())
                            .display(EditedVideoFormField::RawFootageId, &self.raw_display)
                            .value(EditedVideoFormField::EditedVNodeId, self.edited_v_node_id.to_string())
                            .display(EditedVideoFormField::EditedVNodeId, &self.vnode_display),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Update",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            (button_delete(
                                EditedDeletePostRouteTag::new(self.id),
                                "Delete",
                                "Permanently delete this edited video?",
                            ))
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }
}

impl RenderAppPane for EditedFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = edited_crumbs(self.id, &self.raw_display, Some("Edit"));
        scaffold_pane(main_menu("edited"), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(edited_crumbs(self.id, &self.raw_display, Some("Edit")), self.body())
    }
}

impl RenderTemplate for EditedFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = edited_crumbs(self.id, &self.raw_display, Some("Edit"));
        app_scaffold(
            "Edit edited video — Uniquity",
            chrome,
            main_menu("edited"),
            crumbs,
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct EditedCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub raw_footage_id: i64,
    pub raw_display: String,
    pub edited_v_node_id: i64,
    pub vnode_display: String,
    pub error: String,
}

impl RenderTemplate for EditedCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_uniquity_video.EditedCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<EditedCreateModalKey>(
            "",
            form(FormOpts {
                title: "New edited video",
                subtitle: "Link raw footage to an edited output file",
                classes: "@container",
                attrs: form_hx_post_url::<EditedCreateModalKey>(
                    &modal_create_post_url(
                        EditedCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                    ),
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: EditedVideoForm::render_inputs(
                    &FormCtx::form::<EditedVideoForm>()
                        .value(
                            EditedVideoFormField::RawFootageId,
                            self.raw_footage_id.to_string(),
                        )
                        .display(EditedVideoFormField::RawFootageId, &self.raw_display)
                        .value(
                            EditedVideoFormField::EditedVNodeId,
                            self.edited_v_node_id.to_string(),
                        )
                        .display(EditedVideoFormField::EditedVNodeId, &self.vnode_display),
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
pub struct EditedSelectPage {
    pub items: ObjectList<EditedVideoRow>,
    pub target_input: String,
}

impl EditedSelectPage {
    pub fn render_table(&self) -> Markup {
        let headers = [TableColumnHeader {
             key: "RawFootage",label: "Raw footage",
            sort_url: None,
            push_url: false,
        }];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_select(&self.target_input, &r.id.to_string(), &r.raw_title),
                cells: vec![field_text(FieldText { value: &r.raw_title, classes: "" })],
            })
            .collect();
        data_table_list::<EditedVideoSelectTableKey>(
            "Select edited video",
            html! {},
            &headers,
            &rows,
            html! {},
        )
    }
}

impl RenderTemplate for EditedSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_table()
    }
}

#[derive(Generic)]
pub struct PublishedListPage {
    pub items: ObjectList<PublishedVideoRow>,
    pub sort: String,
    pub path_and_query: String,
}

impl PublishedListPage {
    pub fn render_table(&self) -> Markup {
        let youtube_sort = column_sort_url(&self.path_and_query, "YouTubeID", &self.sort);
        let youtube_label = format!("YouTube ID{}", sort_indicator(&self.sort, "YouTubeID"));
        let headers = [
            TableColumnHeader {
                key: "YouTubeID",
                label: &youtube_label,
                sort_url: Some(&youtube_sort),
                push_url: true,
            },
            TableColumnHeader {  key: "RawTitle",label: "Raw title", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_navigate_route(PublishedDetailRouteTag::new(r.id)),
                cells: vec![
                    field_text(FieldText { value: &r.youtube_id, classes: "" }),
                    field_text(FieldText { value: &r.raw_title, classes: "" }),
                ],
            })
            .collect();
        let actions = button_modal_form(ButtonModalForm {
            name: "p_uniquity_video.PublishedCreateForm",
            href: &PublishedCreateGetRouteTag.url(),
            form_post_url: &PublishedCreateGetRouteTag.path(),
            modal_uid: PublishedCreateModalKey::ID,
            icon_name: Some("plus"),
            classes: "btn-square btn-outline btn-sm",
            ..Default::default()
        });
        let pagination = render_pagination::<PublishedVideoTableKey>(
            &self.path_and_query,
            self.items.number,
            self.items.num_pages,
        );
        data_table_list_refresh::<PublishedVideoTableKey>(
            "Published videos",
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
                (field_title(FieldTitle { value: "Published videos", classes: "" }))
                (self.render_table())
            }))
        }
    }
}

impl RenderAppPane for PublishedListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(main_menu("published"), published_list_crumbs(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(published_list_crumbs(), self.body())
    }
}

impl RenderTemplate for PublishedListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Published — Uniquity",
            chrome,
            main_menu("published"),
            published_list_crumbs(),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct PublishedDetailPage {
    pub id: i64,
    pub youtube_id: String,
    pub watch_url: String,
    pub studio_url: String,
    pub yt_title: String,
    pub yt_published_at: String,
    pub yt_upload_status: String,
    pub yt_view_count: String,
    pub yt_like_count: String,
    pub yt_comment_count: String,
    pub raw_title: String,
    pub assigned_to_id: i64,
    pub assigned_to_name: String,
    pub can_award_points: bool,
}

impl PublishedDetailPage {
    fn body(&self) -> Markup {
        let youtube_field = if self.watch_url.is_empty() {
            field_text(FieldText {
                value: &self.youtube_id,
                classes: "",
            })
        } else {
            button_link(ButtonLink {
                href: &self.watch_url,
                label: &self.youtube_id,
                classes: "link link-hover break-all",
                ..Default::default()
            })
        };
        let studio_field = if self.studio_url.is_empty() {
            html! {}
        } else {
            html! {
                (label("YouTube Studio", button_link(ButtonLink {
                    href: &self.studio_url,
                    label: "Open video in YouTube Studio",
                    classes: "link link-hover",
                    ..Default::default()
                })))
            }
        };
        html! {
            (detail(html! {
                (container_column("p-4 gap-2", html! {
                    (label("YouTube video", youtube_field))
                    (studio_field)
                    (label("Title (YouTube)", field_text(FieldText { value: &self.yt_title, classes: "" })))
                    (label("Published on YouTube", field_text(FieldText {
                        value: &self.yt_published_at,
                        classes: "",
                    })))
                    (label("YouTube upload status", field_text(FieldText {
                        value: &self.yt_upload_status,
                        classes: "",
                    })))
                    (label("Views", field_text(FieldText { value: &self.yt_view_count, classes: "" })))
                    (label("Likes", field_text(FieldText { value: &self.yt_like_count, classes: "" })))
                    (label("Comments", field_text(FieldText {
                        value: &self.yt_comment_count,
                        classes: "",
                    })))
                    (label("Edited from (raw)", field_text(FieldText {
                        value: &self.raw_title,
                        classes: "",
                    })))
                    (label("Assigned to", button_link(ButtonLink {
                        href: &EmployeesDetailRouteTag::new(self.assigned_to_id).url(),
                        label: &self.assigned_to_name,
                        classes: "link link-hover",
                        ..Default::default()
                    })))
                    (container_row("flex gap-2 mt-4", html! {
                        (button_link(ButtonLink {
                            href: &PublishedEditGetRouteTag::new(self.id).url(),
                            label: "Edit",
                            classes: "btn btn-primary",
                            ..Default::default()
                        }))
                        @if self.can_award_points {
                            (button_link(ButtonLink {
                                href: &PublishedEditorPointsGetRouteTag::new(self.id).url(),
                                label: "Give points to editor",
                                classes: "btn btn-outline",
                                ..Default::default()
                            }))
                        }
                    }))
                }))
            }))
        }
    }
}

impl RenderAppPane for PublishedDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = published_crumbs(self.id, &self.youtube_id, None);
        scaffold_pane(main_menu("published"), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(published_crumbs(self.id, &self.youtube_id, None), self.body())
    }
}

impl RenderTemplate for PublishedDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = published_crumbs(self.id, &self.youtube_id, None);
        app_scaffold(
            "Published video — Uniquity",
            chrome,
            main_menu("published"),
            crumbs,
            self.body(),
        )
    }
}

/// Edit published video form (full page). Create uses [`PublishedCreateModalPage`].
#[derive(Generic)]
pub struct PublishedFormPage {
    pub id: i64,
    pub edited_video_id: i64,
    pub edited_display: String,
    pub you_tube_video_id: String,
}

impl PublishedFormPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Edit published video", classes: "" }))
                (form(FormOpts {
                    attrs: form_hx_post_main(PublishedEditPostRouteTag::new(self.id)),
                    inputs: PublishedVideoForm::render_inputs(
                        &FormCtx::form::<PublishedVideoForm>()
                            .value(PublishedVideoFormField::EditedVideoId, self.edited_video_id.to_string())
                            .display(PublishedVideoFormField::EditedVideoId, &self.edited_display)
                            .value(PublishedVideoFormField::YouTubeVideoId, &self.you_tube_video_id),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Update",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            (button_delete(
                                PublishedDeletePostRouteTag::new(self.id),
                                "Delete",
                                "Permanently delete this published video?",
                            ))
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }
}

impl RenderAppPane for PublishedFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = published_crumbs(self.id, &self.you_tube_video_id, Some("Edit"));
        scaffold_pane(main_menu("published"), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            published_crumbs(self.id, &self.you_tube_video_id, Some("Edit")),
            self.body(),
        )
    }
}

impl RenderTemplate for PublishedFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = published_crumbs(self.id, &self.you_tube_video_id, Some("Edit"));
        app_scaffold(
            "Edit published video — Uniquity",
            chrome,
            main_menu("published"),
            crumbs,
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct PublishedCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub edited_video_id: i64,
    pub edited_display: String,
    pub you_tube_video_id: String,
    pub error: String,
}

impl RenderTemplate for PublishedCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_uniquity_video.PublishedCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<PublishedCreateModalKey>(
            "",
            form(FormOpts {
                title: "New published video",
                subtitle: "Link an edited video to a YouTube publication",
                classes: "@container",
                attrs: form_hx_post_url::<PublishedCreateModalKey>(
                    &modal_create_post_url(
                        PublishedCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                    ),
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: PublishedVideoForm::render_inputs(
                    &FormCtx::form::<PublishedVideoForm>()
                        .value(
                            PublishedVideoFormField::EditedVideoId,
                            self.edited_video_id.to_string(),
                        )
                        .display(PublishedVideoFormField::EditedVideoId, &self.edited_display)
                        .value(
                            PublishedVideoFormField::YouTubeVideoId,
                            &self.you_tube_video_id,
                        ),
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
pub struct PublishedSelectPage {
    pub items: ObjectList<PublishedVideoRow>,
    pub sort: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl PublishedSelectPage {
    pub fn render_table(&self) -> Markup {
        let youtube_sort = column_sort_url(&self.path_and_query, "YouTubeID", &self.sort);
        let youtube_label = format!("YouTube ID{}", sort_indicator(&self.sort, "YouTubeID"));
        let headers = [TableColumnHeader {
            key: "YouTubeID",
            label: &youtube_label,
            sort_url: Some(&youtube_sort),
            push_url: false,
        }];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_select(&self.target_input, &r.id.to_string(), &r.youtube_id),
                cells: vec![field_text(FieldText { value: &r.youtube_id, classes: "" })],
            })
            .collect();
        data_table_list_refresh::<PublishedVideoSelectTableKey>(
            "Select published video",
            html! {},
            &headers,
            &rows,
            html! {},
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for PublishedSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_table()
    }
}

#[derive(Generic)]
pub struct EditorPointsPage {
    pub published_id: i64,
    pub editor_name: String,
    pub points: String,
}

impl EditorPointsPage {
    fn body(&self) -> Markup {
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: "Give points to editor", classes: "" }))
                (field_text(FieldText {
                    value: "Awards points to the employee assigned to the source raw footage.",
                    classes: "text-base-content/70 mb-4",
                }))
                (label("Responsible editor", field_text(FieldText {
                    value: &self.editor_name,
                    classes: "",
                })))
                (form(FormOpts {
                    attrs: form_hx_post_main(PublishedEditorPointsPostRouteTag::new(self.published_id)),
                    inputs: EditorPointsForm::render_inputs(
                        &FormCtx::form::<EditorPointsForm>()
                            .value(EditorPointsFormField::Points, &self.points),
                    ),
                    actions: button_submit(ButtonSubmit {
                        label: "Award points",
                        classes: "btn-primary",
                        ..Default::default()
                    }),
                    ..Default::default()
                }))
            }))
        }
    }
}

impl RenderAppPane for EditorPointsPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        let crumbs = published_crumbs(self.published_id, "Published video", Some("Give points"));
        scaffold_pane(main_menu("published"), crumbs, self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            published_crumbs(self.published_id, "Published video", Some("Give points")),
            self.body(),
        )
    }
}

impl RenderTemplate for EditorPointsPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let crumbs = published_crumbs(self.published_id, "Published video", Some("Give points"));
        app_scaffold(
            "Editor points — Uniquity",
            chrome,
            main_menu("published"),
            crumbs,
            self.body(),
        )
    }
}
