use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonLink, ButtonSubmit, FieldText, FieldTitle, FormOpts,
        LayoutSidebar, ManyToManyItem, ObjectList, PaginationPage, ShellChrome, ShellScaffold,
        SidebarMenu, SidebarMenuBack, SidebarMenuItem, SlotCapability, SlotRegistrar, SwapKey,
        TableColumnHeader, TablePagination, TableRow, button_delete, button_link,
        button_submit, container_column, container_row, data_table_list,
        detail, field_text, field_title, form, form_hx_get_route, form_hx_post_main,
        label_inline, layout_sidebar, pagination_pages,
        row_attr_navigate_route, row_attr_select, shell_scaffold, sidebar_menu,
        sidebar_menu_item, table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    plugins::dashboard::routes::DashboardAppsRouteTag,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
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
    EditedVideoSelectTableKey, EditedVideoTableKey,
    PublishedVideoSelectTableKey, PublishedVideoTableKey,
    RawFootageSelectTableKey, RawFootageTableKey,
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
        RawSelectIdx: RawSelectPageTag => RawSelectPage,
        VideoEmployeeSelectIdx: VideoEmployeeSelectPageTag => VideoEmployeeSelectPage,
        EditedListIdx: EditedListPageTag => EditedListPage,
        EditedDetailIdx: EditedDetailPageTag => EditedDetailPage,
        EditedFormIdx: EditedFormPageTag => EditedFormPage,
        EditedSelectIdx: EditedSelectPageTag => EditedSelectPage,
        PublishedListIdx: PublishedListPageTag => PublishedListPage,
        PublishedDetailIdx: PublishedDetailPageTag => PublishedDetailPage,
        PublishedFormIdx: PublishedFormPageTag => PublishedFormPage,
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

fn app_scaffold(title: &str, chrome: &ShellChrome, sidebar: Markup, body: Markup) -> Markup {
    shell_scaffold(ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        sidebar,
        body,
        ..Default::default()
    })
}

fn main_menu(active: &str) -> Markup {
    let back_url = DashboardAppsRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: "Video editors",
        back: Some(SidebarMenuBack {
            title: "Back to Home",
            url: &back_url,
        }),
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("hub"),
            content: self.hub_body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.hub_body())
    }
}

impl RenderTemplate for HubPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Video — Uniquity",
            chrome,
            main_menu("hub"),
            self.hub_body(),
        )
    }
}

#[derive(Generic)]
pub struct RawListPage {
    pub items: ObjectList<RawFootageRow>,
    pub filter_title: String,
    pub path_and_query: String,
}

impl RawListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Title", sort_url: None, push_url: true },
            TableColumnHeader { label: "Assigned to", sort_url: None, push_url: true },
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
            (button_link(ButtonLink {
                href: &RawCreateGetRouteTag.url(),
                label: "+",
                classes: "btn-square btn-outline btn-sm",
                ..Default::default()
            }))
        };
        let pagination = render_pagination::<RawFootageTableKey>(
            &self.path_and_query,
            self.items.number,
            self.items.num_pages,
        );
        data_table_list::<RawFootageTableKey>("Raw footage", actions, &headers, &rows, pagination)
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("raw"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for RawListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Raw footage — Uniquity", chrome, main_menu("raw"), self.body())
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
                    (label_inline("Assigned to", field_text(FieldText {
                        value: &self.assigned_to_name,
                        classes: "",
                    })))
                    (label_inline("Files", html! { @for f in &files { (f) } }))
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("raw"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for RawDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Raw footage — Uniquity", chrome, main_menu("raw"), self.body())
    }
}

#[derive(Generic)]
pub struct RawFormPage {
    pub id: i64,
    pub title: String,
    pub assigned_to_id: i64,
    pub assigned_display: String,
    pub file_items: Vec<ManyToManyItem>,
    pub is_edit: bool,
}

impl RawFormPage {
    fn body(&self) -> Markup {
        let title = if self.is_edit { "Edit raw footage" } else { "New raw footage" };
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(RawEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(RawCreatePostRouteTag)
                    },
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
                                label: if self.is_edit { "Update" } else { "Save" },
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    RawDeletePostRouteTag::new(self.id),
                                    "Delete",
                                    "Permanently delete this raw footage?",
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

impl RenderAppPane for RawFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("raw"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for RawFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Raw form — Uniquity", chrome, main_menu("raw"), self.body())
    }
}

#[derive(Generic)]
pub struct RawSelectPage {
    pub items: ObjectList<RawFootageRow>,
    pub filter_title: String,
    pub target_input: String,
}

impl RawSelectPage {
    pub fn render_table(&self) -> Markup {
        let headers = [TableColumnHeader { label: "Title", sort_url: None, push_url: false }];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|r| TableRow {
                attrs: row_attr_select(&self.target_input, &r.id.to_string(), &r.title),
                cells: vec![field_text(FieldText { value: &r.title, classes: "" })],
            })
            .collect();
        data_table_list::<RawFootageSelectTableKey>(
            "Select raw footage",
            html! {},
            &headers,
            &rows,
            html! {},
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
            TableColumnHeader { label: "Raw title", sort_url: None, push_url: true },
            TableColumnHeader { label: "Output file", sort_url: None, push_url: true },
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
        let actions = button_link(ButtonLink {
            href: &EditedCreateGetRouteTag.url(),
            label: "+",
            classes: "btn-square btn-outline btn-sm",
            ..Default::default()
        });
        let pagination = render_pagination::<EditedVideoTableKey>(
            &self.path_and_query,
            self.items.number,
            self.items.num_pages,
        );
        data_table_list::<EditedVideoTableKey>(
            "Edited videos",
            actions,
            &headers,
            &rows,
            pagination,
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("edited"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for EditedListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Edited — Uniquity", chrome, main_menu("edited"), self.body())
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
                    (label_inline("Raw footage", button_link(ButtonLink {
                        href: &RawDetailRouteTag::new(self.raw_footage_id).url(),
                        label: &self.raw_title,
                        classes: "link link-hover",
                        ..Default::default()
                    })))
                    (label_inline("Assigned to", button_link(ButtonLink {
                        href: &EmployeesDetailRouteTag::new(self.assigned_to_id).url(),
                        label: &self.assigned_to_name,
                        classes: "link link-hover",
                        ..Default::default()
                    })))
                    (label_inline("Output file", field_text(FieldText {
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("edited"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for EditedDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Edited video — Uniquity", chrome, main_menu("edited"), self.body())
    }
}

#[derive(Generic)]
pub struct EditedFormPage {
    pub id: i64,
    pub raw_footage_id: i64,
    pub raw_display: String,
    pub edited_v_node_id: i64,
    pub vnode_display: String,
    pub is_edit: bool,
}

impl EditedFormPage {
    fn body(&self) -> Markup {
        let title = if self.is_edit {
            "Edit edited video"
        } else {
            "New edited video"
        };
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(EditedEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(EditedCreatePostRouteTag)
                    },
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
                                label: if self.is_edit { "Update" } else { "Save" },
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    EditedDeletePostRouteTag::new(self.id),
                                    "Delete",
                                    "Permanently delete this edited video?",
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

impl RenderAppPane for EditedFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("edited"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for EditedFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Edited form — Uniquity", chrome, main_menu("edited"), self.body())
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
            label: "Raw footage",
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
    pub path_and_query: String,
}

impl PublishedListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "YouTube ID", sort_url: None, push_url: true },
            TableColumnHeader { label: "Raw title", sort_url: None, push_url: true },
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
        let actions = button_link(ButtonLink {
            href: &PublishedCreateGetRouteTag.url(),
            label: "+",
            classes: "btn-square btn-outline btn-sm",
            ..Default::default()
        });
        let pagination = render_pagination::<PublishedVideoTableKey>(
            &self.path_and_query,
            self.items.number,
            self.items.num_pages,
        );
        data_table_list::<PublishedVideoTableKey>(
            "Published videos",
            actions,
            &headers,
            &rows,
            pagination,
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("published"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for PublishedListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Published — Uniquity", chrome, main_menu("published"), self.body())
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
                (label_inline("YouTube Studio", button_link(ButtonLink {
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
                    (label_inline("YouTube video", youtube_field))
                    (studio_field)
                    (label_inline("Title (YouTube)", field_text(FieldText { value: &self.yt_title, classes: "" })))
                    (label_inline("Published on YouTube", field_text(FieldText {
                        value: &self.yt_published_at,
                        classes: "",
                    })))
                    (label_inline("YouTube upload status", field_text(FieldText {
                        value: &self.yt_upload_status,
                        classes: "",
                    })))
                    (label_inline("Views", field_text(FieldText { value: &self.yt_view_count, classes: "" })))
                    (label_inline("Likes", field_text(FieldText { value: &self.yt_like_count, classes: "" })))
                    (label_inline("Comments", field_text(FieldText {
                        value: &self.yt_comment_count,
                        classes: "",
                    })))
                    (label_inline("Edited from (raw)", field_text(FieldText {
                        value: &self.raw_title,
                        classes: "",
                    })))
                    (label_inline("Assigned to", button_link(ButtonLink {
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("published"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for PublishedDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Published video — Uniquity",
            chrome,
            main_menu("published"),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct PublishedFormPage {
    pub id: i64,
    pub edited_video_id: i64,
    pub edited_display: String,
    pub you_tube_video_id: String,
    pub is_edit: bool,
}

impl PublishedFormPage {
    fn body(&self) -> Markup {
        let title = if self.is_edit {
            "Edit published video"
        } else {
            "New published video"
        };
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(PublishedEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(PublishedCreatePostRouteTag)
                    },
                    inputs: PublishedVideoForm::render_inputs(
                        &FormCtx::form::<PublishedVideoForm>()
                            .value(PublishedVideoFormField::EditedVideoId, self.edited_video_id.to_string())
                            .display(PublishedVideoFormField::EditedVideoId, &self.edited_display)
                            .value(PublishedVideoFormField::YouTubeVideoId, &self.you_tube_video_id),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: if self.is_edit { "Update" } else { "Save" },
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    PublishedDeletePostRouteTag::new(self.id),
                                    "Delete",
                                    "Permanently delete this published video?",
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

impl RenderAppPane for PublishedFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("published"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for PublishedFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Published form — Uniquity", chrome, main_menu("published"), self.body())
    }
}

#[derive(Generic)]
pub struct PublishedSelectPage {
    pub items: ObjectList<PublishedVideoRow>,
    pub target_input: String,
}

impl PublishedSelectPage {
    pub fn render_table(&self) -> Markup {
        let headers = [TableColumnHeader {
            label: "YouTube ID",
            sort_url: None,
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
        data_table_list::<PublishedVideoSelectTableKey>(
            "Select published video",
            html! {},
            &headers,
            &rows,
            html! {},
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
                (label_inline("Responsible editor", field_text(FieldText {
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
        layout_sidebar(LayoutSidebar {
            sidebar: main_menu("published"),
            content: self.body(),
        })
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        lariv_rs::components::layout_main(self.body())
    }
}

impl RenderTemplate for EditorPointsPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Editor points — Uniquity",
            chrome,
            main_menu("published"),
            self.body(),
        )
    }
}
