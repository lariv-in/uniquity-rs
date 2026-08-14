use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonDeletePost, ButtonModalForm, ButtonSubmit, Crumb, FieldText, FieldTitle,
        FormOpts, LayoutMain, LayoutSidebar, ManyToManyItem, ObjectList, PaginationPage,
        ShellChrome, ShellScaffold, SidebarMenu, SidebarMenuItem, SlotCapability, SlotRegistrar,
        SwapKey, TableButtonFilter, TableColumnHeader, TablePagination, TableRow, breadcrumbs,
        button_clear, button_delete_post_route, button_modal_form, button_submit, column_sort_url,
        container_column, container_row, data_table_list_refresh, detail, field_text, field_title,
        form, form_hx_get_picker_route, form_hx_get_route, form_hx_post_main_url, form_hx_post_url,
        label_inline,
        layout_main, layout_sidebar, modal_keyed, pagination_pages, row_attr_navigate_route,
        row_attr_select_multi, shell_scaffold, sidebar_menu, sidebar_menu_item_pane, sort_indicator,
        table_button_filter, table_create_button, table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::{RenderPickerSelect, picker_create_button},
    plugins::customer::routes::CustomerDetailRouteTag,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use super::forms::{
    GandolaFilterForm, GandolaFilterFormField, GandolaForm, GandolaFormField,
    GandolaPreferencesForm, GandolaPreferencesFormField, SiteFilterForm, SiteFilterFormField,
    SiteForm, SiteFormField,
};
use super::keys::{
    GandolaCreateModalKey, GandolaEditModalKey, GandolaSelectModalKey, GandolaSelectTableKey,
    GandolaTableKey, SiteCreateModalKey, SiteEditModalKey, SiteSelectModalKey, SiteSelectTableKey,
    SiteTableKey,
};
use super::routes::{
    GandolaCreatePostRouteTag, GandolaDefaultRouteTag, GandolaDeletePostRouteTag,
    GandolaDetailRouteTag, GandolaEditGetRouteTag, GandolaEditPostRouteTag,
    GandolaPreferencesPostRouteTag, GandolaPreferencesRouteTag, GandolaSelectRouteTag,
    SiteCreatePostRouteTag, SiteDefaultRouteTag, SiteDeletePostRouteTag, SiteDetailRouteTag,
    SiteEditGetRouteTag, SiteEditPostRouteTag, SiteSelectRouteTag,
};
use super::site_status::SiteStatus;

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

fn scaffold_pane(
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> lariv_rs::components::AppLayoutHtml {
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

fn fk_value(id: i64) -> String {
    if id <= 0 {
        String::new()
    } else {
        id.to_string()
    }
}

fn gandola_menu(active: &str) -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Gandola Manager",
        children: html! {
            (sidebar_menu_item_pane(SidebarMenuItem {
                title: "Gandolas",
                url: &GandolaDefaultRouteTag.url(),
                active: active == "gandolas",
                ..Default::default()
            }))
            (sidebar_menu_item_pane(SidebarMenuItem {
                title: "Sites",
                url: &SiteDefaultRouteTag.url(),
                active: active == "sites",
                ..Default::default()
            }))
            (sidebar_menu_item_pane(SidebarMenuItem {
                title: "Settings",
                url: &GandolaPreferencesRouteTag.url(),
                active: active == "settings",
                ..Default::default()
            }))
        },
    })
}

fn list_crumbs(label: &'static str) -> Markup {
    breadcrumbs(&[Crumb {
        label,
        href: None,
    }])
}

fn entity_crumbs(
    list_label: &'static str,
    list_url: &str,
    name: &str,
    detail_url: &str,
    action: Option<&str>,
) -> Markup {
    match action {
        None => breadcrumbs(&[
            Crumb {
                label: list_label,
                href: Some(list_url),
            },
            Crumb {
                label: name,
                href: None,
            },
        ]),
        Some(act) => breadcrumbs(&[
            Crumb {
                label: list_label,
                href: Some(list_url),
            },
            Crumb {
                label: name,
                href: Some(detail_url),
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
    }
}

fn gandola_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    entity_crumbs(
        "Gandolas",
        &GandolaDefaultRouteTag.url(),
        name,
        &GandolaDetailRouteTag::new(id).url(),
        action,
    )
}

fn site_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    entity_crumbs(
        "Sites",
        &SiteDefaultRouteTag.url(),
        name,
        &SiteDetailRouteTag::new(id).url(),
        action,
    )
}

fn detail_menu(title: String, detail_url: String) -> Markup {
    sidebar_menu(SidebarMenu {
        title: title.as_str(),
        children: html! {
            (sidebar_menu_item_pane(SidebarMenuItem {
                title: "Detail",
                url: &detail_url,
                active: true,
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

fn assigned_badge(is_assigned: bool, site_name: &str) -> Markup {
    if is_assigned {
        html! { span class="badge badge-success" { (site_name) } }
    } else {
        html! { span class="badge badge-error" { "Not assigned" } }
    }
}

fn status_badge(status: &str, label: &str) -> Markup {
    let class = SiteStatus::parse(status)
        .map(SiteStatus::badge_class)
        .unwrap_or("badge");
    html! { span class=(class) { (label) } }
}

fn choice_pairs(choices: &[(&str, &str)]) -> Vec<(String, String)> {
    choices
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

lariv_rs::define_register_items! {
    plugin: GandolaManagerTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        GandolaListIdx: GandolaListPageTag => GandolaListPage,
        GandolaDetailIdx: GandolaDetailPageTag => GandolaDetailPage,
        GandolaEditModalIdx: GandolaEditModalPageTag => GandolaEditModalPage,
        GandolaCreateModalIdx: GandolaCreateModalPageTag => GandolaCreateModalPage,
        GandolaSelectIdx: GandolaSelectPageTag => GandolaSelectPage,
        SiteListIdx: SiteListPageTag => SiteListPage,
        SiteDetailIdx: SiteDetailPageTag => SiteDetailPage,
        SiteEditModalIdx: SiteEditModalPageTag => SiteEditModalPage,
        SiteCreateModalIdx: SiteCreateModalPageTag => SiteCreateModalPage,
        SiteSelectIdx: SiteSelectPageTag => SiteSelectPage,
        PreferencesIdx: GandolaPreferencesPageTag => GandolaPreferencesPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: GandolaManagerTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct RelatedName {
    pub id: i64,
    pub name: String,
}

#[derive(Clone)]
pub struct GandolaRow {
    pub id: i64,
    pub name: String,
    pub is_assigned: bool,
    pub current_site_name: String,
    pub site_names: Vec<String>,
}

#[derive(Generic)]
pub struct GandolaListPage {
    pub gandolas: ObjectList<GandolaRow>,
    pub filter_name: String,
    pub sort: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl GandolaListPage {
    pub fn render_table(&self) -> Markup {
        let name_sort = column_sort_url(&self.path_and_query, "Name", &self.sort);
        let name_label = format!("Name{}", sort_indicator(&self.sort, "Name"));
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "CurrentSite",
                label: "Current Site",
                sort_url: None,
                push_url: true,
            },
            TableColumnHeader {
                key: "Sites",
                label: "Sites",
                sort_url: None,
                push_url: true,
            },
        ];
        let rows: Vec<TableRow> = self
            .gandolas
            .items
            .iter()
            .map(|g| {
                let sites = g.site_names.join(", ");
                TableRow {
                    attrs: row_attr_navigate_route(GandolaDetailRouteTag::new(g.id)),
                    cells: vec![
                        field_text(FieldText {
                            value: &g.name,
                            classes: "",
                        }),
                        assigned_badge(g.is_assigned, &g.current_site_name),
                        field_text(FieldText {
                            value: &sites,
                            classes: "",
                        }),
                    ],
                }
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<GandolaTableKey, GandolaDefaultRouteTag>(
                        GandolaDefaultRouteTag,
                    ),
                    inputs: GandolaFilterForm::render_inputs(
                        &FormCtx::form::<GandolaFilterForm>()
                            .value(GandolaFilterFormField::Name, &self.filter_name),
                    ),
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
        if self.can_edit {
            actions = html! {
                (actions)
                (table_create_button::<GandolaTableKey, GandolaCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            };
        }
        let pagination = render_pagination::<GandolaTableKey>(
            &self.path_and_query,
            self.gandolas.number,
            self.gandolas.num_pages,
        );
        data_table_list_refresh::<GandolaTableKey>(
            "Gandolas",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderAppPane for GandolaListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            gandola_menu("gandolas"),
            list_crumbs("Gandolas"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(list_crumbs("Gandolas"), self.render_table())
    }
}

impl RenderTemplate for GandolaListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Gandolas",
            chrome,
            gandola_menu("gandolas"),
            list_crumbs("Gandolas"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct GandolaDetailPage {
    pub id: i64,
    pub name: String,
    pub is_assigned: bool,
    pub current_site: Option<RelatedName>,
    pub sites: Vec<RelatedName>,
    pub can_edit: bool,
}

impl GandolaDetailPage {
    fn body(&self) -> Markup {
        let assigned_label = if self.is_assigned { "Yes" } else { "No" };
        let current_name = self
            .current_site
            .as_ref()
            .map(|s| s.name.as_str())
            .unwrap_or("Not assigned");
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (label_inline("Is Currently Assigned", field_text(FieldText { value: assigned_label, classes: "" })))
                    (label_inline("Current Site", assigned_badge(self.is_assigned, current_name)))
                    (label_inline("Sites", html! {
                        div class="flex flex-col gap-1" {
                            @for site in &self.sites {
                                a class="link" href=(SiteDetailRouteTag::new(site.id).url()) { (site.name) }
                            }
                        }
                    }))
                    @if self.can_edit {
                        (container_row("flex gap-2 mt-4", html! {
                            (button_modal_form(ButtonModalForm {
                                name: "p_gandola_manager.GandolaEditForm",
                                href: &GandolaEditGetRouteTag::new(self.id).url(),
                                form_post_url: &GandolaEditPostRouteTag::new(self.id).path(),
                                modal_uid: GandolaEditModalKey::ID,
                                label: "Edit",
                                classes: "btn-outline",
                                ..Default::default()
                            }))
                        }))
                    }
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        detail_menu(
            format!("Gandola: {}", self.name),
            GandolaDetailRouteTag::new(self.id).url(),
        )
    }
}

impl RenderAppPane for GandolaDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            self.menu(),
            gandola_crumbs(self.id, &self.name, None),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(gandola_crumbs(self.id, &self.name, None), self.body())
    }
}

impl RenderTemplate for GandolaDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Gandola",
            chrome,
            self.menu(),
            gandola_crumbs(self.id, &self.name, None),
            self.body(),
        )
    }
}

fn gandola_form_inputs(name: &str, sites: &[ManyToManyItem]) -> Markup {
    GandolaForm::render_inputs(
        &FormCtx::form::<GandolaForm>()
            .value(GandolaFormField::Name, name)
            .m2m(GandolaFormField::Sites, sites),
    )
}

#[derive(Generic)]
pub struct GandolaEditModalPage {
    pub id: i64,
    pub form_name: String,
    pub name: String,
    pub sites: Vec<ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for GandolaEditModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        modal_keyed::<GandolaEditModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "Edit gandola" }
                (form(FormOpts {
                    attrs: form_hx_post_url::<GandolaEditModalKey>(&modal_edit_post_url(
                        GandolaEditPostRouteTag::new(self.id),
                        &self.form_name,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: gandola_form_inputs(&self.name, &self.sites),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                        (button_delete_post_route(
                            GandolaDeletePostRouteTag::new(self.id),
                            ButtonDeletePost {
                                label: "Delete",
                                confirm: "Permanently delete this gandola?",
                                classes: "btn-error",
                            },
                        ))
                    },
                    ..Default::default()
                }))
            },
        )
    }
}

#[derive(Generic)]
pub struct GandolaCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub name: String,
    pub sites: Vec<ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for GandolaCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_gandola_manager.GandolaCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<GandolaCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Gandola",
                subtitle: "Create a new gandola",
                classes: "@container",
                attrs: form_hx_post_url::<GandolaCreateModalKey>(&modal_create_post_query(
                    GandolaCreatePostRouteTag,
                    form_name,
                    &self.refresh_table,
                    &self.target_input,
                )),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: gandola_form_inputs(&self.name, &self.sites),
                actions: html! {
                    (container_row("flex justify-end gap-2 mt-2", html! {
                        (button_submit(ButtonSubmit {
                            label: "Save Gandola",
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
pub struct GandolaSelectPage {
    pub gandolas: ObjectList<GandolaRow>,
    pub filter_name: String,
    pub sort: String,
    pub path_and_query: String,
    pub target_input: String,
    pub can_edit: bool,
}

impl RenderPickerSelect<GandolaSelectTableKey, GandolaSelectModalKey> for GandolaSelectPage {
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "Gandolas"
        } else {
            self.target_input.as_str()
        };
        let name_sort = column_sort_url(&self.path_and_query, "Name", &self.sort);
        let name_label = format!("Name{}", sort_indicator(&self.sort, "Name"));
        let headers = [TableColumnHeader {
            key: "Name",
            label: &name_label,
            sort_url: Some(&name_sort),
            push_url: false,
        }];
        let rows: Vec<TableRow> = self
            .gandolas
            .items
            .iter()
            .map(|g| TableRow {
                attrs: row_attr_select_multi(target, &g.id.to_string(), &g.name),
                cells: vec![field_text(FieldText {
                    value: &g.name,
                    classes: "",
                })],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        GandolaSelectTableKey,
                        GandolaSelectModalKey,
                        GandolaSelectRouteTag,
                    >(GandolaSelectRouteTag),
                    inputs: html! {
                        (GandolaFilterForm::render_inputs(
                            &FormCtx::form::<GandolaFilterForm>()
                                .value(GandolaFilterFormField::Name, &self.filter_name),
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
        if self.can_edit {
            actions = html! {
                (actions)
                (picker_create_button::<GandolaCreateModalKey>(
                    &self.target_input,
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            };
        }
        let pagination = render_pagination::<GandolaSelectTableKey>(
            &self.path_and_query,
            self.gandolas.number,
            self.gandolas.num_pages,
        );
        data_table_list_refresh::<GandolaSelectTableKey>(
            "Select Gandolas",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for GandolaSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Clone)]
pub struct SiteRow {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub status_label: String,
    pub gandola_names: Vec<String>,
}

#[derive(Generic)]
pub struct SiteListPage {
    pub sites: ObjectList<SiteRow>,
    pub filter_name: String,
    pub sort: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl SiteListPage {
    pub fn render_table(&self) -> Markup {
        let name_sort = column_sort_url(&self.path_and_query, "Name", &self.sort);
        let status_sort = column_sort_url(&self.path_and_query, "Status", &self.sort);
        let name_label = format!("Name{}", sort_indicator(&self.sort, "Name"));
        let status_label = format!("Status{}", sort_indicator(&self.sort, "Status"));
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Address",
                label: "Address",
                sort_url: None,
                push_url: true,
            },
            TableColumnHeader {
                key: "StartDate",
                label: "Start Date",
                sort_url: None,
                push_url: true,
            },
            TableColumnHeader {
                key: "EndDate",
                label: "End Date",
                sort_url: None,
                push_url: true,
            },
            TableColumnHeader {
                key: "Status",
                label: &status_label,
                sort_url: Some(&status_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Gandolas",
                label: "Gandolas",
                sort_url: None,
                push_url: true,
            },
        ];
        let rows: Vec<TableRow> = self
            .sites
            .items
            .iter()
            .map(|s| {
                let gandolas = s.gandola_names.join(", ");
                TableRow {
                    attrs: row_attr_navigate_route(SiteDetailRouteTag::new(s.id)),
                    cells: vec![
                        field_text(FieldText {
                            value: &s.name,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: &s.address,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: &s.start_date,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: &s.end_date,
                            classes: "",
                        }),
                        status_badge(&s.status, &s.status_label),
                        field_text(FieldText {
                            value: &gandolas,
                            classes: "",
                        }),
                    ],
                }
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<SiteTableKey, SiteDefaultRouteTag>(SiteDefaultRouteTag),
                    inputs: SiteFilterForm::render_inputs(
                        &FormCtx::form::<SiteFilterForm>()
                            .value(SiteFilterFormField::Name, &self.filter_name),
                    ),
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
        if self.can_edit {
            actions = html! {
                (actions)
                (table_create_button::<SiteTableKey, SiteCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            };
        }
        let pagination = render_pagination::<SiteTableKey>(
            &self.path_and_query,
            self.sites.number,
            self.sites.num_pages,
        );
        data_table_list_refresh::<SiteTableKey>(
            "Sites",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderAppPane for SiteListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            gandola_menu("sites"),
            list_crumbs("Sites"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(list_crumbs("Sites"), self.render_table())
    }
}

impl RenderTemplate for SiteListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Sites",
            chrome,
            gandola_menu("sites"),
            list_crumbs("Sites"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct SiteDetailPage {
    pub id: i64,
    pub name: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub status_label: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub address: String,
    pub po_rent: String,
    pub po_dti: String,
    pub po_tpi: String,
    pub po_extn1: String,
    pub po_extn2: String,
    pub po_extn3: String,
    pub gandolas: Vec<RelatedName>,
    pub can_edit: bool,
}

impl SiteDetailPage {
    fn body(&self) -> Markup {
        let customer_url = CustomerDetailRouteTag::new(self.customer_id).url();
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (label_inline("Customer", html! {
                        a class="link" href=(customer_url) { (self.customer_name) }
                    }))
                    (label_inline("Status", status_badge(&self.status, &self.status_label)))
                    (label_inline("Start Date", field_text(FieldText { value: &self.start_date, classes: "" })))
                    (label_inline("End Date", field_text(FieldText { value: &self.end_date, classes: "" })))
                    (label_inline("Address", field_text(FieldText { value: &self.address, classes: "" })))
                    (label_inline("PO Rent", field_text(FieldText { value: &self.po_rent, classes: "" })))
                    (label_inline("PO DTI", field_text(FieldText { value: &self.po_dti, classes: "" })))
                    (label_inline("PO TPI", field_text(FieldText { value: &self.po_tpi, classes: "" })))
                    (label_inline("PO Extension 1", field_text(FieldText { value: &self.po_extn1, classes: "" })))
                    (label_inline("PO Extension 2", field_text(FieldText { value: &self.po_extn2, classes: "" })))
                    (label_inline("PO Extension 3", field_text(FieldText { value: &self.po_extn3, classes: "" })))
                    (label_inline("Gandolas", html! {
                        div class="flex flex-col gap-1" {
                            @for g in &self.gandolas {
                                a class="link" href=(GandolaDetailRouteTag::new(g.id).url()) { (g.name) }
                            }
                        }
                    }))
                    @if self.can_edit {
                        (container_row("flex gap-2 mt-4", html! {
                            (button_modal_form(ButtonModalForm {
                                name: "p_gandola_manager.SiteEditForm",
                                href: &SiteEditGetRouteTag::new(self.id).url(),
                                form_post_url: &SiteEditPostRouteTag::new(self.id).path(),
                                modal_uid: SiteEditModalKey::ID,
                                label: "Edit",
                                classes: "btn-outline",
                                ..Default::default()
                            }))
                        }))
                    }
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        detail_menu(
            format!("Site: {}", self.name),
            SiteDetailRouteTag::new(self.id).url(),
        )
    }
}

impl RenderAppPane for SiteDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(self.menu(), site_crumbs(self.id, &self.name, None), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(site_crumbs(self.id, &self.name, None), self.body())
    }
}

impl RenderTemplate for SiteDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Site",
            chrome,
            self.menu(),
            site_crumbs(self.id, &self.name, None),
            self.body(),
        )
    }
}

fn site_form_inputs(
    name: &str,
    customer_id: i64,
    customer_display: &str,
    status: &str,
    start_date: &str,
    end_date: &str,
    address: &str,
    po_rent: &str,
    po_dti: &str,
    po_tpi: &str,
    po_extn1: &str,
    po_extn2: &str,
    po_extn3: &str,
    gandolas: &[ManyToManyItem],
) -> Markup {
    let customer_id_s = fk_value(customer_id);
    let choices = choice_pairs(SiteForm::status_choices());
    SiteForm::render_inputs(
        &FormCtx::form::<SiteForm>()
            .value(SiteFormField::Name, name)
            .value(SiteFormField::CustomerId, customer_id_s.as_str())
            .display(SiteFormField::CustomerId, customer_display)
            .value(SiteFormField::Status, status)
            .choices(SiteFormField::Status, &choices)
            .value(SiteFormField::StartDate, start_date)
            .value(SiteFormField::EndDate, end_date)
            .value(SiteFormField::Address, address)
            .value(SiteFormField::PoRent, po_rent)
            .value(SiteFormField::PoDti, po_dti)
            .value(SiteFormField::PoTpi, po_tpi)
            .value(SiteFormField::PoExtn1, po_extn1)
            .value(SiteFormField::PoExtn2, po_extn2)
            .value(SiteFormField::PoExtn3, po_extn3)
            .m2m(SiteFormField::Gandolas, gandolas),
    )
}

#[derive(Generic)]
pub struct SiteEditModalPage {
    pub id: i64,
    pub form_name: String,
    pub name: String,
    pub customer_id: i64,
    pub customer_display: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub address: String,
    pub po_rent: String,
    pub po_dti: String,
    pub po_tpi: String,
    pub po_extn1: String,
    pub po_extn2: String,
    pub po_extn3: String,
    pub gandolas: Vec<ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for SiteEditModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        modal_keyed::<SiteEditModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "Edit site" }
                (form(FormOpts {
                    attrs: form_hx_post_url::<SiteEditModalKey>(&modal_edit_post_url(
                        SiteEditPostRouteTag::new(self.id),
                        &self.form_name,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: site_form_inputs(
                        &self.name,
                        self.customer_id,
                        &self.customer_display,
                        &self.status,
                        &self.start_date,
                        &self.end_date,
                        &self.address,
                        &self.po_rent,
                        &self.po_dti,
                        &self.po_tpi,
                        &self.po_extn1,
                        &self.po_extn2,
                        &self.po_extn3,
                        &self.gandolas,
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                        (button_delete_post_route(
                            SiteDeletePostRouteTag::new(self.id),
                            ButtonDeletePost {
                                label: "Delete",
                                confirm: "Permanently delete this site?",
                                classes: "btn-error",
                            },
                        ))
                    },
                    ..Default::default()
                }))
            },
        )
    }
}

#[derive(Generic)]
pub struct SiteCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub name: String,
    pub customer_id: i64,
    pub customer_display: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub address: String,
    pub po_rent: String,
    pub po_dti: String,
    pub po_tpi: String,
    pub po_extn1: String,
    pub po_extn2: String,
    pub po_extn3: String,
    pub gandolas: Vec<ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for SiteCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "p_gandola_manager.SiteCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<SiteCreateModalKey>(
            "",
            form(FormOpts {
                title: "Create Site",
                subtitle: "Create a new site",
                classes: "@container",
                attrs: form_hx_post_url::<SiteCreateModalKey>(&modal_create_post_query(
                    SiteCreatePostRouteTag,
                    form_name,
                    &self.refresh_table,
                    &self.target_input,
                )),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: site_form_inputs(
                    &self.name,
                    self.customer_id,
                    &self.customer_display,
                    &self.status,
                    &self.start_date,
                    &self.end_date,
                    &self.address,
                    &self.po_rent,
                    &self.po_dti,
                    &self.po_tpi,
                    &self.po_extn1,
                    &self.po_extn2,
                    &self.po_extn3,
                    &self.gandolas,
                ),
                actions: html! {
                    (container_row("flex justify-end gap-2 mt-2", html! {
                        (button_submit(ButtonSubmit {
                            label: "Save Site",
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
pub struct SiteSelectPage {
    pub sites: ObjectList<SiteRow>,
    pub filter_name: String,
    pub sort: String,
    pub path_and_query: String,
    pub target_input: String,
    pub can_edit: bool,
}

impl RenderPickerSelect<SiteSelectTableKey, SiteSelectModalKey> for SiteSelectPage {
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "Sites"
        } else {
            self.target_input.as_str()
        };
        let name_sort = column_sort_url(&self.path_and_query, "Name", &self.sort);
        let name_label = format!("Name{}", sort_indicator(&self.sort, "Name"));
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: false,
            },
            TableColumnHeader {
                key: "Status",
                label: "Status",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .sites
            .items
            .iter()
            .map(|s| TableRow {
                attrs: row_attr_select_multi(target, &s.id.to_string(), &s.name),
                cells: vec![
                    field_text(FieldText {
                        value: &s.name,
                        classes: "",
                    }),
                    status_badge(&s.status, &s.status_label),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        SiteSelectTableKey,
                        SiteSelectModalKey,
                        SiteSelectRouteTag,
                    >(SiteSelectRouteTag),
                    inputs: html! {
                        (SiteFilterForm::render_inputs(
                            &FormCtx::form::<SiteFilterForm>()
                                .value(SiteFilterFormField::Name, &self.filter_name),
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
        if self.can_edit {
            actions = html! {
                (actions)
                (picker_create_button::<SiteCreateModalKey>(
                    &self.target_input,
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            };
        }
        let pagination = render_pagination::<SiteSelectTableKey>(
            &self.path_and_query,
            self.sites.number,
            self.sites.num_pages,
        );
        data_table_list_refresh::<SiteSelectTableKey>(
            "Select Sites",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for SiteSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct GandolaPreferencesPage {
    pub gandola_product_id: String,
    pub gandola_product_display: String,
    pub tpi_product_id: String,
    pub tpi_product_display: String,
    pub dti_product_id: String,
    pub dti_product_display: String,
    pub payment_term_lines_json: String,
    pub error: String,
    pub can_edit: bool,
}

impl GandolaPreferencesPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: "Gandola Configuration", classes: "" }))
                    @if self.can_edit {
                        (form(FormOpts {
                            attrs: form_hx_post_main_url(
                                &GandolaPreferencesPostRouteTag.url(),
                            ),
                            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                            inputs: GandolaPreferencesForm::render_inputs(
                                &FormCtx::form::<GandolaPreferencesForm>()
                                    .value(GandolaPreferencesFormField::GandolaProductId, &self.gandola_product_id)
                                    .display(GandolaPreferencesFormField::GandolaProductId, &self.gandola_product_display)
                                    .value(GandolaPreferencesFormField::TpiProductId, &self.tpi_product_id)
                                    .display(GandolaPreferencesFormField::TpiProductId, &self.tpi_product_display)
                                    .value(GandolaPreferencesFormField::DtiProductId, &self.dti_product_id)
                                    .display(GandolaPreferencesFormField::DtiProductId, &self.dti_product_display)
                                    .value(GandolaPreferencesFormField::PaymentTermLinesJson, &self.payment_term_lines_json),
                            ),
                            actions: html! {
                                (button_submit(ButtonSubmit {
                                    label: "Save settings",
                                    classes: "btn-primary",
                                    ..Default::default()
                                }))
                            },
                            ..Default::default()
                        }))
                    } @else {
                        (label_inline("Gandola Rent Product", field_text(FieldText { value: &self.gandola_product_display, classes: "" })))
                        (label_inline("TPI Product", field_text(FieldText { value: &self.tpi_product_display, classes: "" })))
                        (label_inline("DTI Product", field_text(FieldText { value: &self.dti_product_display, classes: "" })))
                    }
                }))
            }))
        }
    }
}

impl RenderAppPane for GandolaPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            gandola_menu("settings"),
            list_crumbs("Settings"),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(list_crumbs("Settings"), self.body())
    }
}

impl RenderTemplate for GandolaPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Gandola Settings",
            chrome,
            gandola_menu("settings"),
            list_crumbs("Settings"),
            self.body(),
        )
    }
}
