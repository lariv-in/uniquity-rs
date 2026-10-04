use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonModalForm, ButtonSubmit, Crumb, DeleteConfirmation, DetailHeader,
        FieldText, FieldTextarea, FieldTitle, FormOpts, HtmlAttrs, LayoutMain, LayoutSidebar,
        MainContentKey, ManyToManyItem, ObjectList, PaginationPage, ShellChrome, ShellScaffold,
        SidebarMenu, SidebarMenuItem, SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter,
        TableColumnHeader, TablePagination, TableRow, breadcrumbs, button_clear, button_modal_form,
        button_submit, column_sort_url, container_column, container_row, data_table_list_refresh,
        delete_confirmation, detail, detail_header, field_text, field_textarea, field_title, form,
        form_hx_get_picker_route, form_hx_get_route, form_hx_get_url, form_hx_post_selector,
        form_hx_post_url, label, layout_main, layout_sidebar, modal, modal_keyed, pagination_pages,
        row_attr_navigate, row_attr_navigate_route, row_attr_select, row_attr_select_multi,
        shell_scaffold, sidebar_menu, sidebar_menu_item_pane, sort_indicator, table_button_filter,
        table_create_button, table_pagination, with_list_filter_common,
    },
    html_form::{CsrfToken, FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::{RenderPickerSelect, picker_create_button},
    plugins::customer::routes::CustomerDetailRouteTag,
    plugins::filesystem::routes::VNodeDetailRouteTag,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use super::forms::{
    GandolaFilterForm, GandolaFilterFormField, GandolaForm, GandolaFormField,
    GandolaPreferencesForm, GandolaPreferencesFormField, PurchaseOrderFilterForm,
    PurchaseOrderFilterFormField, PurchaseOrderForm, PurchaseOrderFormField, SiteFilterForm,
    SiteFilterFormField, SiteForm, SiteFormField, SiteInvoiceFilterForm,
    SiteInvoiceFilterFormField,
};
use super::keys::{
    GandolaCreateModalKey, GandolaDeleteModalKey, GandolaEditModalKey, GandolaSelectModalKey,
    GandolaSelectTableKey, GandolaSitesTableKey, GandolaTableKey, PurchaseOrderCreateModalKey,
    PurchaseOrderDeleteModalKey, PurchaseOrderEditModalKey, PurchaseOrderSelectModalKey,
    PurchaseOrderSelectTableKey, PurchaseOrderTableKey, SiteCreateModalKey, SiteDeleteModalKey,
    SiteEditModalKey, SiteFkSelectModalKey, SiteFkSelectTableKey, SiteGandolasTableKey,
    SiteInvoicesTableKey, SitePurchaseOrdersTableKey, SiteSelectModalKey, SiteSelectTableKey,
    SiteTableKey,
};
use super::routes::{
    GandolaCreatePostRouteTag, GandolaDefaultRouteTag, GandolaDeleteGetRouteTag,
    GandolaDeletePostRouteTag, GandolaDetailRouteTag, GandolaEditGetRouteTag,
    GandolaEditPostRouteTag, GandolaPreferencesPostRouteTag, GandolaPreferencesRouteTag,
    GandolaSelectRouteTag, PurchaseOrderCreatePostRouteTag, PurchaseOrderDefaultRouteTag,
    PurchaseOrderDeleteGetRouteTag, PurchaseOrderDeletePostRouteTag, PurchaseOrderDetailRouteTag,
    PurchaseOrderEditGetRouteTag, PurchaseOrderEditPostRouteTag, PurchaseOrderSelectRouteTag,
    SiteCreatePostRouteTag, SiteDefaultRouteTag, SiteDeleteGetRouteTag, SiteDeletePostRouteTag,
    SiteDetailRouteTag, SiteEditGetRouteTag, SiteEditPostRouteTag, SiteFkSelectRouteTag,
    SiteSelectRouteTag,
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
                title: "Sites",
                url: &SiteDefaultRouteTag.url(),
                active: active == "sites",
                ..Default::default()
            }))
            (sidebar_menu_item_pane(SidebarMenuItem {
                title: "Gandolas",
                url: &GandolaDefaultRouteTag.url(),
                active: active == "gandolas",
                ..Default::default()
            }))
            (sidebar_menu_item_pane(SidebarMenuItem {
                title: "Purchase Orders",
                url: &PurchaseOrderDefaultRouteTag.url(),
                active: active == "purchase_orders",
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
    breadcrumbs(&[Crumb { label, href: None }])
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

fn purchase_order_crumbs(id: i64, number: &str, action: Option<&str>) -> Markup {
    entity_crumbs(
        "Purchase Orders",
        &PurchaseOrderDefaultRouteTag.url(),
        number,
        &PurchaseOrderDetailRouteTag::new(id).url(),
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

fn col_sort(path_and_query: &str, key: &str, label: &str, sort: &str) -> (String, String) {
    (
        column_sort_url(path_and_query, key, sort),
        format!("{label}{}", sort_indicator(sort, key)),
    )
}

fn split_query(path_and_query: &str) -> (String, Vec<(String, String)>) {
    let (path, query) = path_and_query
        .split_once('?')
        .unwrap_or((path_and_query, ""));
    let pairs = if query.is_empty() {
        Vec::new()
    } else {
        query
            .split('&')
            .filter_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                Some((k.to_string(), v.to_string()))
            })
            .collect()
    };
    (path.to_string(), pairs)
}

fn join_query(path: &str, pairs: &[(String, String)]) -> String {
    if pairs.is_empty() {
        return path.to_string();
    }
    let qs = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{path}?{qs}")
}

fn rename_in_url(path_and_query: &str, from: &str, to: &str) -> String {
    let (path, mut pairs) = split_query(path_and_query);
    if from != to {
        for (k, _) in &mut pairs {
            if k == from {
                *k = to.to_string();
            }
        }
    }
    join_query(&path, &pairs)
}

/// Sort URL for one table on a page that hosts several tables.
///
/// `sort_key` and `page_key` keep each table's sort and page out of the others' query params.
fn keyed_sort_url(
    path_and_query: &str,
    sort_key: &str,
    page_key: &str,
    column: &str,
    current_sort: &str,
) -> String {
    let synthetic = rename_in_url(
        &rename_in_url(path_and_query, sort_key, "sort"),
        page_key,
        "page",
    );
    let sorted = column_sort_url(&synthetic, column, current_sort);
    rename_in_url(&rename_in_url(&sorted, "sort", sort_key), "page", page_key)
}

fn keyed_col_sort(
    path_and_query: &str,
    sort_key: &str,
    page_key: &str,
    column: &str,
    label: &str,
    sort: &str,
    default_desc: bool,
) -> (String, String) {
    let indicator_sort = if default_desc && sort.trim().is_empty() {
        format!("{column} DESC")
    } else {
        sort.to_string()
    };
    let url_current = if default_desc
        && (sort.trim().is_empty() || sort.trim().eq_ignore_ascii_case(&format!("{column} DESC")))
    {
        ""
    } else {
        sort
    };
    (
        keyed_sort_url(path_and_query, sort_key, page_key, column, url_current),
        format!("{label}{}", sort_indicator(&indicator_sort, column)),
    )
}

fn keyed_pagination<K: SwapKey>(
    path_and_query: &str,
    page_key: &str,
    number: u32,
    num_pages: u32,
) -> Markup {
    let synthetic = rename_in_url(path_and_query, page_key, "page");
    let owned = pagination_pages(&synthetic, number, num_pages, true);
    let rewritten: Vec<(bool, String, bool, bool, String)> = owned
        .into_iter()
        .map(|(ellipsis, url, push_url, active, label)| {
            let url = if url.is_empty() {
                url
            } else {
                rename_in_url(&url, "page", page_key)
            };
            (ellipsis, url, push_url, active, label)
        })
        .collect();
    let pages: Vec<PaginationPage<'_>> = rewritten
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

fn relation_filter<K: SwapKey>(
    path_and_query: &str,
    page_key: &str,
    inputs: Markup,
    page_size: u32,
) -> Markup {
    table_button_filter(TableButtonFilter {
        panel: form(
            &CsrfToken::current(),
            FormOpts {
                attrs: form_hx_get_url::<K>(path_and_query),
                inputs: html! {
                    (with_list_filter_common(inputs, page_size))
                    input type="hidden" name=(page_key) value="1" {}
                },
                actions: html! {
                    (container_row("flex gap-2", html! {
                        (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                        (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                    }))
                },
                ..Default::default()
            },
        ),
        ..Default::default()
    })
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
        SiteFkSelectIdx: SiteFkSelectPageTag => SiteFkSelectPage,
        PreferencesIdx: GandolaPreferencesPageTag => GandolaPreferencesPage,
        PurchaseOrderListIdx: PurchaseOrderListPageTag => PurchaseOrderListPage,
        PurchaseOrderDetailIdx: PurchaseOrderDetailPageTag => PurchaseOrderDetailPage,
        PurchaseOrderEditModalIdx: PurchaseOrderEditModalPageTag => PurchaseOrderEditModalPage,
        PurchaseOrderCreateModalIdx: PurchaseOrderCreateModalPageTag => PurchaseOrderCreateModalPage,
        PurchaseOrderSelectIdx: PurchaseOrderSelectPageTag => PurchaseOrderSelectPage,
        ConfirmDeleteIdx: GandolaConfirmDeletePageTag => ConfirmDeletePage,
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
pub struct RelatedInvoice {
    pub id: i64,
    pub name: String,
    pub href: String,
    pub date: String,
    pub status: String,
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
    pub page_size: u32,
}

fn gandola_column_labels(
    path_and_query: &str,
    sort: &str,
    sort_key: &str,
    page_key: &str,
) -> (String, String, String, String, String, String) {
    let (name_sort, name_label) = keyed_col_sort(
        path_and_query,
        sort_key,
        page_key,
        "Name",
        "Name",
        sort,
        false,
    );
    let (current_site_sort, current_site_label) = keyed_col_sort(
        path_and_query,
        sort_key,
        page_key,
        "CurrentSite",
        "Current Site",
        sort,
        false,
    );
    let (sites_sort, sites_label) = keyed_col_sort(
        path_and_query,
        sort_key,
        page_key,
        "Sites",
        "Sites",
        sort,
        false,
    );
    (
        name_sort,
        name_label,
        current_site_sort,
        current_site_label,
        sites_sort,
        sites_label,
    )
}

fn gandola_table_rows(items: &[GandolaRow]) -> Vec<TableRow> {
    items
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
        .collect()
}

impl GandolaListPage {
    pub fn render_table(&self) -> Markup {
        let (name_sort, name_label, current_site_sort, current_site_label, sites_sort, sites_label) =
            gandola_column_labels(&self.path_and_query, &self.sort, "sort", "page");
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "CurrentSite",
                label: &current_site_label,
                sort_url: Some(&current_site_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Sites",
                label: &sites_label,
                sort_url: Some(&sites_sort),
                push_url: true,
            },
        ];
        let rows = gandola_table_rows(&self.gandolas.items);
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_route::<GandolaTableKey, GandolaDefaultRouteTag>(
                        GandolaDefaultRouteTag,
                    ),
                    inputs: with_list_filter_common(
                        GandolaFilterForm::render_inputs(
                            &FormCtx::form::<GandolaFilterForm>(CsrfToken::current())
                                .value(GandolaFilterFormField::Name, &self.filter_name),
                        ),
                        self.page_size,
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
    pub sites: ObjectList<SiteRow>,
    pub filter_name: String,
    pub filter_site_id: String,
    pub sort: String,
    pub path_and_query: String,
    pub page_size: u32,
    pub can_edit: bool,
}

impl GandolaDetailPage {
    pub fn render_sites_table(&self) -> Markup {
        let labels = site_column_labels(&self.path_and_query, &self.sort, true);
        let headers = site_column_headers(&labels, true);
        let rows = site_table_rows(&self.sites.items, |site| {
            row_attr_navigate_route(SiteDetailRouteTag::new(site.id))
        });
        let actions = table_button_filter(TableButtonFilter {
            panel: form(
                &CsrfToken::current(),
                FormOpts {
                    attrs: form_hx_get_route::<GandolaSitesTableKey, GandolaDetailRouteTag>(
                        GandolaDetailRouteTag::new(self.id),
                    ),
                    inputs: with_list_filter_common(
                        SiteFilterForm::render_inputs(
                            &FormCtx::form::<SiteFilterForm>(CsrfToken::current())
                                .value(SiteFilterFormField::Name, &self.filter_name)
                                .value(SiteFilterFormField::SiteId, &self.filter_site_id),
                        ),
                        self.page_size,
                    ),
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                },
            ),
            ..Default::default()
        });
        let pagination = render_pagination::<GandolaSitesTableKey>(
            &self.path_and_query,
            self.sites.number,
            self.sites.num_pages,
        );
        data_table_list_refresh::<GandolaSitesTableKey>(
            "Sites",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }

    fn body(&self) -> Markup {
        let assigned_label = if self.is_assigned { "Yes" } else { "No" };
        let current_name = self
            .current_site
            .as_ref()
            .map(|s| s.name.as_str())
            .unwrap_or("Not assigned");
        let actions = if self.can_edit {
            html! {
                (button_modal_form(ButtonModalForm {
                    name: "gandola_manager.GandolaEditForm",
                    href: &GandolaEditGetRouteTag::new(self.id).url(),
                    form_post_url: &GandolaEditPostRouteTag::new(self.id).path(),
                    modal_uid: GandolaEditModalKey::ID,
                    label: "Edit",
                    classes: "btn-outline",
                    ..Default::default()
                }))
            }
        } else {
            html! {}
        };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (detail_header(DetailHeader {
                        title: &self.name,
                        actions,
                    }))
                    (label("Is Currently Assigned", field_text(FieldText { value: assigned_label, classes: "" })))
                    (label("Current Site", assigned_badge(self.is_assigned, current_name)))
                }))
            }))
            div class="mt-6" {
                (self.render_sites_table())
            }
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
        &FormCtx::form::<GandolaForm>(CsrfToken::current())
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
        let delete_url = GandolaDeleteGetRouteTag::new(self.id).url();
        modal_keyed::<GandolaEditModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "Edit gandola" }
                (form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_post_url::<GandolaEditModalKey>(&modal_edit_post_url(
                        GandolaEditPostRouteTag::new(self.id),
                        &self.form_name,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: gandola_form_inputs(&self.name, &self.sites),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                        (button_modal_form(ButtonModalForm {
                            label: "Delete",
                            icon_name: Some("trash"),
                            name: "gandola_manager.GandolaDeleteForm",
                            href: &delete_url,
                            form_post_url: &delete_url,
                            modal_uid: GandolaDeleteModalKey::ID,
                            classes: "btn-error",
                            ..Default::default()
                        }))
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
            "gandola_manager.GandolaCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<GandolaCreateModalKey>(
            "",
            form(
                &CsrfToken::current(),
                FormOpts {
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
                },
            ),
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
    pub page_size: u32,
}

impl RenderPickerSelect<GandolaSelectTableKey, GandolaSelectModalKey> for GandolaSelectPage {
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "Gandolas"
        } else {
            self.target_input.as_str()
        };
        let (name_sort, name_label) = col_sort(&self.path_and_query, "Name", "Name", &self.sort);
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
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_picker_route::<
                        GandolaSelectTableKey,
                        GandolaSelectModalKey,
                        GandolaSelectRouteTag,
                    >(GandolaSelectRouteTag),
                    inputs: html! {
                        (with_list_filter_common(
                            GandolaFilterForm::render_inputs(
                                &FormCtx::form::<GandolaFilterForm>(CsrfToken::current())
                                    .value(GandolaFilterFormField::Name, &self.filter_name),
                            ),
                            self.page_size,
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
    pub site_id: String,
    pub address: String,
    pub remarks: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub status_label: String,
    pub gandola_names: Vec<String>,
}

struct SiteColumnLabels {
    name_sort: String,
    name_label: String,
    site_id_sort: String,
    site_id_label: String,
    address_sort: String,
    address_label: String,
    remarks_sort: String,
    remarks_label: String,
    start_date_sort: String,
    start_date_label: String,
    end_date_sort: String,
    end_date_label: String,
    status_sort: String,
    status_label: String,
    gandolas_sort: String,
    gandolas_label: String,
}

fn site_col_sort(
    path_and_query: &str,
    key: &str,
    label: &str,
    sort: &str,
    default_start_desc: bool,
) -> (String, String) {
    let indicator_sort = if default_start_desc && sort.trim().is_empty() {
        "StartDate DESC"
    } else {
        sort
    };
    let url_current = if default_start_desc
        && key.eq_ignore_ascii_case("StartDate")
        && (sort.trim().is_empty() || sort.trim().eq_ignore_ascii_case("StartDate DESC"))
    {
        ""
    } else {
        sort
    };
    (
        column_sort_url(path_and_query, key, url_current),
        format!("{label}{}", sort_indicator(indicator_sort, key)),
    )
}

fn site_column_labels(
    path_and_query: &str,
    sort: &str,
    default_start_desc: bool,
) -> SiteColumnLabels {
    let (name_sort, name_label) =
        site_col_sort(path_and_query, "Name", "Name", sort, default_start_desc);
    let (site_id_sort, site_id_label) = site_col_sort(
        path_and_query,
        "SiteId",
        "Site ID",
        sort,
        default_start_desc,
    );
    let (address_sort, address_label) = site_col_sort(
        path_and_query,
        "Address",
        "Address",
        sort,
        default_start_desc,
    );
    let (remarks_sort, remarks_label) = site_col_sort(
        path_and_query,
        "Remarks",
        "Remarks",
        sort,
        default_start_desc,
    );
    let (start_date_sort, start_date_label) = site_col_sort(
        path_and_query,
        "StartDate",
        "Start Date",
        sort,
        default_start_desc,
    );
    let (end_date_sort, end_date_label) = site_col_sort(
        path_and_query,
        "EndDate",
        "End Date",
        sort,
        default_start_desc,
    );
    let (status_sort, status_label) =
        site_col_sort(path_and_query, "Status", "Status", sort, default_start_desc);
    let (gandolas_sort, gandolas_label) = site_col_sort(
        path_and_query,
        "Gandolas",
        "Gandolas",
        sort,
        default_start_desc,
    );
    SiteColumnLabels {
        name_sort,
        name_label,
        site_id_sort,
        site_id_label,
        address_sort,
        address_label,
        remarks_sort,
        remarks_label,
        start_date_sort,
        start_date_label,
        end_date_sort,
        end_date_label,
        status_sort,
        status_label,
        gandolas_sort,
        gandolas_label,
    }
}

fn site_column_headers(labels: &SiteColumnLabels, push_url: bool) -> Vec<TableColumnHeader<'_>> {
    vec![
        TableColumnHeader {
            key: "Name",
            label: &labels.name_label,
            sort_url: Some(&labels.name_sort),
            push_url,
        },
        TableColumnHeader {
            key: "SiteId",
            label: &labels.site_id_label,
            sort_url: Some(&labels.site_id_sort),
            push_url,
        },
        TableColumnHeader {
            key: "Address",
            label: &labels.address_label,
            sort_url: Some(&labels.address_sort),
            push_url,
        },
        TableColumnHeader {
            key: "Remarks",
            label: &labels.remarks_label,
            sort_url: Some(&labels.remarks_sort),
            push_url,
        },
        TableColumnHeader {
            key: "StartDate",
            label: &labels.start_date_label,
            sort_url: Some(&labels.start_date_sort),
            push_url,
        },
        TableColumnHeader {
            key: "EndDate",
            label: &labels.end_date_label,
            sort_url: Some(&labels.end_date_sort),
            push_url,
        },
        TableColumnHeader {
            key: "Status",
            label: &labels.status_label,
            sort_url: Some(&labels.status_sort),
            push_url,
        },
        TableColumnHeader {
            key: "Gandolas",
            label: &labels.gandolas_label,
            sort_url: Some(&labels.gandolas_sort),
            push_url,
        },
    ]
}

fn site_cells(site: &SiteRow) -> Vec<Markup> {
    let gandolas = site.gandola_names.join(", ");
    vec![
        field_text(FieldText {
            value: &site.name,
            classes: "",
        }),
        field_text(FieldText {
            value: &site.site_id,
            classes: "",
        }),
        field_text(FieldText {
            value: &site.address,
            classes: "",
        }),
        field_text(FieldText {
            value: &site.remarks,
            classes: "",
        }),
        field_text(FieldText {
            value: &site.start_date,
            classes: "",
        }),
        field_text(FieldText {
            value: &site.end_date,
            classes: "",
        }),
        status_badge(&site.status, &site.status_label),
        field_text(FieldText {
            value: &gandolas,
            classes: "",
        }),
    ]
}

fn site_table_rows(sites: &[SiteRow], attrs: impl Fn(&SiteRow) -> HtmlAttrs) -> Vec<TableRow> {
    sites
        .iter()
        .map(|site| TableRow {
            attrs: attrs(site),
            cells: site_cells(site),
        })
        .collect()
}

#[derive(Generic)]
pub struct SiteListPage {
    pub sites: ObjectList<SiteRow>,
    pub filter_name: String,
    pub filter_site_id: String,
    pub sort: String,
    pub path_and_query: String,
    pub can_edit: bool,
    pub page_size: u32,
}

impl SiteListPage {
    pub fn render_table(&self) -> Markup {
        let labels = site_column_labels(&self.path_and_query, &self.sort, false);
        let headers = site_column_headers(&labels, true);
        let rows = site_table_rows(&self.sites.items, |site| {
            row_attr_navigate_route(SiteDetailRouteTag::new(site.id))
        });
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_route::<SiteTableKey, SiteDefaultRouteTag>(SiteDefaultRouteTag),
                    inputs: with_list_filter_common(
                        SiteFilterForm::render_inputs(
                            &FormCtx::form::<SiteFilterForm>(CsrfToken::current())
                                .value(SiteFilterFormField::Name, &self.filter_name)
                                .value(SiteFilterFormField::SiteId, &self.filter_site_id),
                        ),
                        self.page_size,
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
    pub site_id: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub status_label: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub address: String,
    pub remarks: String,
    pub gandolas: ObjectList<GandolaRow>,
    pub gandola_filter_name: String,
    pub gandola_sort: String,
    pub purchase_orders: ObjectList<PurchaseOrderRow>,
    pub po_filter_number: String,
    pub po_sort: String,
    pub invoices: ObjectList<RelatedInvoice>,
    pub invoice_filter_number: String,
    pub invoice_filter_status: String,
    pub invoice_sort: String,
    pub path_and_query: String,
    pub page_size: u32,
    pub can_edit: bool,
}

impl SiteDetailPage {
    pub fn render_gandolas_table(&self) -> Markup {
        let (name_sort, name_label, current_site_sort, current_site_label, sites_sort, sites_label) =
            gandola_column_labels(&self.path_and_query, &self.gandola_sort, "g_sort", "g_page");
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "CurrentSite",
                label: &current_site_label,
                sort_url: Some(&current_site_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Sites",
                label: &sites_label,
                sort_url: Some(&sites_sort),
                push_url: true,
            },
        ];
        let rows = gandola_table_rows(&self.gandolas.items);
        let actions = relation_filter::<SiteGandolasTableKey>(
            &self.path_and_query,
            "g_page",
            GandolaFilterForm::render_inputs(
                &FormCtx::form::<GandolaFilterForm>(CsrfToken::current())
                    .value(GandolaFilterFormField::Name, &self.gandola_filter_name),
            ),
            self.page_size,
        );
        let pagination = keyed_pagination::<SiteGandolasTableKey>(
            &self.path_and_query,
            "g_page",
            self.gandolas.number,
            self.gandolas.num_pages,
        );
        data_table_list_refresh::<SiteGandolasTableKey>(
            "Gandolas",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }

    pub fn render_purchase_orders_table(&self) -> Markup {
        let (
            number_sort,
            number_label,
            date_sort,
            date_label,
            customer_sort,
            customer_label,
            site_sort,
            site_label,
        ) = purchase_order_column_labels(&self.path_and_query, &self.po_sort, "po_sort", "po_page");
        let headers = [
            TableColumnHeader {
                key: "Number",
                label: &number_label,
                sort_url: Some(&number_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Date",
                label: &date_label,
                sort_url: Some(&date_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Customer",
                label: &customer_label,
                sort_url: Some(&customer_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Site",
                label: &site_label,
                sort_url: Some(&site_sort),
                push_url: true,
            },
        ];
        let rows = purchase_order_table_rows(&self.purchase_orders.items);
        let actions = relation_filter::<SitePurchaseOrdersTableKey>(
            &self.path_and_query,
            "po_page",
            PurchaseOrderFilterForm::render_inputs(
                &FormCtx::form::<PurchaseOrderFilterForm>(CsrfToken::current())
                    .value(PurchaseOrderFilterFormField::Number, &self.po_filter_number),
            ),
            self.page_size,
        );
        let pagination = keyed_pagination::<SitePurchaseOrdersTableKey>(
            &self.path_and_query,
            "po_page",
            self.purchase_orders.number,
            self.purchase_orders.num_pages,
        );
        data_table_list_refresh::<SitePurchaseOrdersTableKey>(
            "Purchase Orders",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }

    pub fn render_invoices_table(&self) -> Markup {
        let (number_sort, number_label) = keyed_col_sort(
            &self.path_and_query,
            "inv_sort",
            "inv_page",
            "Number",
            "Number",
            &self.invoice_sort,
            false,
        );
        let (date_sort, date_label) = keyed_col_sort(
            &self.path_and_query,
            "inv_sort",
            "inv_page",
            "Date",
            "Date",
            &self.invoice_sort,
            true,
        );
        let (status_sort, status_label) = keyed_col_sort(
            &self.path_and_query,
            "inv_sort",
            "inv_page",
            "Status",
            "Status",
            &self.invoice_sort,
            false,
        );
        let headers = [
            TableColumnHeader {
                key: "Number",
                label: &number_label,
                sort_url: Some(&number_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Date",
                label: &date_label,
                sort_url: Some(&date_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Status",
                label: &status_label,
                sort_url: Some(&status_sort),
                push_url: true,
            },
        ];
        let rows: Vec<TableRow> = self
            .invoices
            .items
            .iter()
            .map(|inv| TableRow {
                attrs: row_attr_navigate(&inv.href),
                cells: vec![
                    field_text(FieldText {
                        value: &inv.name,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &inv.date,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &inv.status,
                        classes: "",
                    }),
                ],
            })
            .collect();
        let status_choices = choice_pairs(SiteInvoiceFilterForm::status_choices());
        let actions = relation_filter::<SiteInvoicesTableKey>(
            &self.path_and_query,
            "inv_page",
            SiteInvoiceFilterForm::render_inputs(
                &FormCtx::form::<SiteInvoiceFilterForm>(CsrfToken::current())
                    .value(
                        SiteInvoiceFilterFormField::Number,
                        &self.invoice_filter_number,
                    )
                    .value(
                        SiteInvoiceFilterFormField::Status,
                        &self.invoice_filter_status,
                    )
                    .choices(SiteInvoiceFilterFormField::Status, &status_choices),
            ),
            self.page_size,
        );
        let pagination = keyed_pagination::<SiteInvoicesTableKey>(
            &self.path_and_query,
            "inv_page",
            self.invoices.number,
            self.invoices.num_pages,
        );
        data_table_list_refresh::<SiteInvoicesTableKey>(
            "Invoices",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }

    fn body(&self) -> Markup {
        let customer_url = CustomerDetailRouteTag::new(self.customer_id).url();
        let actions = if self.can_edit {
            html! {
                (button_modal_form(ButtonModalForm {
                    name: "gandola_manager.SiteEditForm",
                    href: &SiteEditGetRouteTag::new(self.id).url(),
                    form_post_url: &SiteEditPostRouteTag::new(self.id).path(),
                    modal_uid: SiteEditModalKey::ID,
                    label: "Edit",
                    classes: "btn-outline",
                    ..Default::default()
                }))
            }
        } else {
            html! {}
        };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (detail_header(DetailHeader {
                        title: &self.name,
                        actions,
                    }))
                    (label("Site ID", field_text(FieldText { value: &self.site_id, classes: "" })))
                    (label("Customer", html! {
                        a class="link" href=(customer_url) { (self.customer_name) }
                    }))
                    (label("Status", status_badge(&self.status, &self.status_label)))
                    (label("Start Date", field_text(FieldText { value: &self.start_date, classes: "" })))
                    (label("End Date", field_text(FieldText { value: &self.end_date, classes: "" })))
                    (label("Address", field_text(FieldText { value: &self.address, classes: "" })))
                    (label("Remarks", field_textarea(FieldTextarea {
                        value: &self.remarks,
                        classes: "break-words min-w-0 max-w-full overflow-x-hidden",
                    })))
                }))
            }))
            div class="mt-6" { (self.render_gandolas_table()) }
            div class="mt-6" { (self.render_purchase_orders_table()) }
            div class="mt-6" { (self.render_invoices_table()) }
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
        scaffold_pane(
            self.menu(),
            site_crumbs(self.id, &self.name, None),
            self.body(),
        )
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
    site_id: &str,
    customer_id: i64,
    customer_display: &str,
    status: &str,
    start_date: &str,
    end_date: &str,
    address: &str,
    remarks: &str,
    gandolas: &[ManyToManyItem],
    invoices: &[ManyToManyItem],
    purchase_orders: &[ManyToManyItem],
) -> Markup {
    let customer_id_s = fk_value(customer_id);
    let choices = choice_pairs(SiteForm::status_choices());
    SiteForm::render_inputs(
        &FormCtx::form::<SiteForm>(CsrfToken::current())
            .value(SiteFormField::Name, name)
            .value(SiteFormField::SiteId, site_id)
            .value(SiteFormField::CustomerId, customer_id_s.as_str())
            .display(SiteFormField::CustomerId, customer_display)
            .value(SiteFormField::Status, status)
            .choices(SiteFormField::Status, &choices)
            .value(SiteFormField::StartDate, start_date)
            .value(SiteFormField::EndDate, end_date)
            .value(SiteFormField::Address, address)
            .value(SiteFormField::Remarks, remarks)
            .m2m(SiteFormField::Gandolas, gandolas)
            .m2m(SiteFormField::Invoices, invoices)
            .m2m(SiteFormField::PurchaseOrders, purchase_orders),
    )
}

#[derive(Generic)]
pub struct SiteEditModalPage {
    pub id: i64,
    pub form_name: String,
    pub name: String,
    pub site_id: String,
    pub customer_id: i64,
    pub customer_display: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub address: String,
    pub remarks: String,
    pub gandolas: Vec<ManyToManyItem>,
    pub invoices: Vec<ManyToManyItem>,
    pub purchase_orders: Vec<ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for SiteEditModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let delete_url = SiteDeleteGetRouteTag::new(self.id).url();
        modal_keyed::<SiteEditModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "Edit site" }
                (form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_post_url::<SiteEditModalKey>(&modal_edit_post_url(
                        SiteEditPostRouteTag::new(self.id),
                        &self.form_name,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: site_form_inputs(
                        &self.name,
                        &self.site_id,
                        self.customer_id,
                        &self.customer_display,
                        &self.status,
                        &self.start_date,
                        &self.end_date,
                        &self.address,
                        &self.remarks,
                        &self.gandolas,
                        &self.invoices,
                        &self.purchase_orders,
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                        (button_modal_form(ButtonModalForm {
                            label: "Delete",
                            icon_name: Some("trash"),
                            name: "gandola_manager.SiteDeleteForm",
                            href: &delete_url,
                            form_post_url: &delete_url,
                            modal_uid: SiteDeleteModalKey::ID,
                            classes: "btn-error",
                            ..Default::default()
                        }))
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
    pub site_id: String,
    pub customer_id: i64,
    pub customer_display: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub address: String,
    pub remarks: String,
    pub gandolas: Vec<ManyToManyItem>,
    pub invoices: Vec<ManyToManyItem>,
    pub purchase_orders: Vec<ManyToManyItem>,
    pub error: String,
}

impl RenderTemplate for SiteCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "gandola_manager.SiteCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<SiteCreateModalKey>(
            "",
            form(
                &CsrfToken::current(),
                FormOpts {
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
                        &self.site_id,
                        self.customer_id,
                        &self.customer_display,
                        &self.status,
                        &self.start_date,
                        &self.end_date,
                        &self.address,
                        &self.remarks,
                        &self.gandolas,
                        &self.invoices,
                        &self.purchase_orders,
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
                },
            ),
        )
    }
}

#[derive(Generic)]
pub struct SiteSelectPage {
    pub sites: ObjectList<SiteRow>,
    pub filter_name: String,
    pub filter_site_id: String,
    pub sort: String,
    pub path_and_query: String,
    pub target_input: String,
    pub can_edit: bool,
    pub page_size: u32,
}

impl RenderPickerSelect<SiteSelectTableKey, SiteSelectModalKey> for SiteSelectPage {
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "Sites"
        } else {
            self.target_input.as_str()
        };
        let labels = site_column_labels(&self.path_and_query, &self.sort, true);
        let headers = site_column_headers(&labels, false);
        let rows = site_table_rows(&self.sites.items, |site| {
            row_attr_select_multi(target, &site.id.to_string(), &site.name)
        });
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_picker_route::<
                        SiteSelectTableKey,
                        SiteSelectModalKey,
                        SiteSelectRouteTag,
                    >(SiteSelectRouteTag),
                    inputs: html! {
                        (with_list_filter_common(
                            SiteFilterForm::render_inputs(
                                &FormCtx::form::<SiteFilterForm>(CsrfToken::current())
                                    .value(SiteFilterFormField::Name, &self.filter_name)
                                    .value(SiteFilterFormField::SiteId, &self.filter_site_id),
                            ),
                            self.page_size,
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
pub struct SiteFkSelectPage {
    pub sites: ObjectList<SiteRow>,
    pub filter_name: String,
    pub filter_site_id: String,
    pub sort: String,
    pub path_and_query: String,
    pub target_input: String,
    pub can_edit: bool,
    pub page_size: u32,
}

impl RenderPickerSelect<SiteFkSelectTableKey, SiteFkSelectModalKey> for SiteFkSelectPage {
    fn render_table(&self) -> Markup {
        let (name_sort, name_label) = col_sort(&self.path_and_query, "Name", "Name", &self.sort);
        let (status_sort, status_label) =
            col_sort(&self.path_and_query, "Status", "Status", &self.sort);
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: false,
            },
            TableColumnHeader {
                key: "Status",
                label: &status_label,
                sort_url: Some(&status_sort),
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .sites
            .items
            .iter()
            .map(|s| TableRow {
                attrs: row_attr_select(&self.target_input, &s.id.to_string(), &s.name),
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
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_picker_route::<
                        SiteFkSelectTableKey,
                        SiteFkSelectModalKey,
                        SiteFkSelectRouteTag,
                    >(SiteFkSelectRouteTag),
                    inputs: html! {
                        (with_list_filter_common(
                            SiteFilterForm::render_inputs(
                                &FormCtx::form::<SiteFilterForm>(CsrfToken::current())
                                    .value(SiteFilterFormField::Name, &self.filter_name)
                                    .value(SiteFilterFormField::SiteId, &self.filter_site_id),
                            ),
                            self.page_size,
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
        let pagination = render_pagination::<SiteFkSelectTableKey>(
            &self.path_and_query,
            self.sites.number,
            self.sites.num_pages,
        );
        data_table_list_refresh::<SiteFkSelectTableKey>(
            "Select Site",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for SiteFkSelectPage {
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
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub gemini_model_choices: Vec<(String, String)>,
    pub purchase_order_files_directory_id: String,
    pub purchase_order_files_directory_display: String,
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
                        (form(&CsrfToken::current(), FormOpts {
                            // outerHTML (not outerMorph): re-init Alpine FK/payment-term state
                            // from the server-rendered values after save.
                            attrs: form_hx_post_url::<MainContentKey>(
                                &GandolaPreferencesPostRouteTag.path(),
                            )
                            .set("hx-swap", "outerHTML"),
                            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                            inputs: GandolaPreferencesForm::render_inputs(
                                &FormCtx::form::<GandolaPreferencesForm>(CsrfToken::current())
                                    .value(GandolaPreferencesFormField::GandolaProductId, &self.gandola_product_id)
                                    .display(GandolaPreferencesFormField::GandolaProductId, &self.gandola_product_display)
                                    .value(GandolaPreferencesFormField::TpiProductId, &self.tpi_product_id)
                                    .display(GandolaPreferencesFormField::TpiProductId, &self.tpi_product_display)
                                    .value(GandolaPreferencesFormField::DtiProductId, &self.dti_product_id)
                                    .display(GandolaPreferencesFormField::DtiProductId, &self.dti_product_display)
                                    .value(
                                        GandolaPreferencesFormField::PurchaseOrderFilesDirectoryId,
                                        &self.purchase_order_files_directory_id,
                                    )
                                    .display(
                                        GandolaPreferencesFormField::PurchaseOrderFilesDirectoryId,
                                        &self.purchase_order_files_directory_display,
                                    )
                                    .value(GandolaPreferencesFormField::GeminiApiKey, &self.gemini_api_key)
                                    .value(GandolaPreferencesFormField::GeminiModel, &self.gemini_model)
                                    .choices(GandolaPreferencesFormField::GeminiModel, &self.gemini_model_choices)
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
                        (label("Gandola Rent Product", field_text(FieldText { value: &self.gandola_product_display, classes: "" })))
                        (label("TPI Product", field_text(FieldText { value: &self.tpi_product_display, classes: "" })))
                        (label("DTI Product", field_text(FieldText { value: &self.dti_product_display, classes: "" })))
                        (label("Purchase order files directory", field_text(FieldText {
                            value: if self.purchase_order_files_directory_display.trim().is_empty() {
                                "Not set"
                            } else {
                                &self.purchase_order_files_directory_display
                            },
                            classes: "",
                        })))
                        (label("Gemini API key", field_text(FieldText {
                            value: if self.gemini_api_key.trim().is_empty() { "Not set" } else { "Configured" },
                            classes: "",
                        })))
                        (label("Gemini model", field_text(FieldText { value: &self.gemini_model, classes: "" })))
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

#[derive(Clone)]
pub struct PurchaseOrderRow {
    pub id: i64,
    pub number: String,
    pub date: String,
    pub customer_name: String,
    pub site_name: String,
}

#[derive(Clone)]
pub struct PoLineRow {
    pub item_code: String,
    pub description: String,
    pub unit: String,
    pub delivery_date: String,
    pub quantity: String,
    pub rate: String,
}

#[derive(Generic)]
pub struct PurchaseOrderListPage {
    pub purchase_orders: ObjectList<PurchaseOrderRow>,
    pub filter_number: String,
    pub sort: String,
    pub path_and_query: String,
    pub can_edit: bool,
    pub page_size: u32,
}

fn purchase_order_column_labels(
    path_and_query: &str,
    sort: &str,
    sort_key: &str,
    page_key: &str,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    let (number_sort, number_label) = keyed_col_sort(
        path_and_query,
        sort_key,
        page_key,
        "Number",
        "Number",
        sort,
        false,
    );
    let (date_sort, date_label) = keyed_col_sort(
        path_and_query,
        sort_key,
        page_key,
        "Date",
        "Date",
        sort,
        false,
    );
    let (customer_sort, customer_label) = keyed_col_sort(
        path_and_query,
        sort_key,
        page_key,
        "Customer",
        "Customer",
        sort,
        false,
    );
    let (site_sort, site_label) = keyed_col_sort(
        path_and_query,
        sort_key,
        page_key,
        "Site",
        "Site",
        sort,
        false,
    );
    (
        number_sort,
        number_label,
        date_sort,
        date_label,
        customer_sort,
        customer_label,
        site_sort,
        site_label,
    )
}

fn purchase_order_table_rows(items: &[PurchaseOrderRow]) -> Vec<TableRow> {
    items
        .iter()
        .map(|po| TableRow {
            attrs: row_attr_navigate_route(PurchaseOrderDetailRouteTag::new(po.id)),
            cells: vec![
                field_text(FieldText {
                    value: &po.number,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &po.date,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &po.customer_name,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &po.site_name,
                    classes: "",
                }),
            ],
        })
        .collect()
}

impl PurchaseOrderListPage {
    pub fn render_table(&self) -> Markup {
        let (
            number_sort,
            number_label,
            date_sort,
            date_label,
            customer_sort,
            customer_label,
            site_sort,
            site_label,
        ) = purchase_order_column_labels(&self.path_and_query, &self.sort, "sort", "page");
        let headers = [
            TableColumnHeader {
                key: "Number",
                label: &number_label,
                sort_url: Some(&number_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Date",
                label: &date_label,
                sort_url: Some(&date_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Customer",
                label: &customer_label,
                sort_url: Some(&customer_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Site",
                label: &site_label,
                sort_url: Some(&site_sort),
                push_url: true,
            },
        ];
        let rows = purchase_order_table_rows(&self.purchase_orders.items);
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_route::<PurchaseOrderTableKey, PurchaseOrderDefaultRouteTag>(
                        PurchaseOrderDefaultRouteTag,
                    ),
                    inputs: with_list_filter_common(
                        PurchaseOrderFilterForm::render_inputs(
                            &FormCtx::form::<PurchaseOrderFilterForm>(CsrfToken::current())
                                .value(PurchaseOrderFilterFormField::Number, &self.filter_number),
                        ),
                        self.page_size,
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
                (table_create_button::<PurchaseOrderTableKey, PurchaseOrderCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            };
        }
        let pagination = render_pagination::<PurchaseOrderTableKey>(
            &self.path_and_query,
            self.purchase_orders.number,
            self.purchase_orders.num_pages,
        );
        data_table_list_refresh::<PurchaseOrderTableKey>(
            "Purchase Orders",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderAppPane for PurchaseOrderListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            gandola_menu("purchase_orders"),
            list_crumbs("Purchase Orders"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(list_crumbs("Purchase Orders"), self.render_table())
    }
}

impl RenderTemplate for PurchaseOrderListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Purchase Orders",
            chrome,
            gandola_menu("purchase_orders"),
            list_crumbs("Purchase Orders"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct PurchaseOrderDetailPage {
    pub id: i64,
    pub number: String,
    pub date: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub site_id: i64,
    pub site_name: String,
    pub file_id: Option<i64>,
    pub file_name: String,
    pub billing_address: String,
    pub shipping_address: String,
    pub lines: Vec<PoLineRow>,
    pub can_edit: bool,
}

impl PurchaseOrderDetailPage {
    fn body(&self) -> Markup {
        let customer_url = CustomerDetailRouteTag::new(self.customer_id).url();
        let site_url = SiteDetailRouteTag::new(self.site_id).url();
        let actions = if self.can_edit {
            html! {
                (button_modal_form(ButtonModalForm {
                    name: "gandola_manager.PurchaseOrderEditForm",
                    href: &PurchaseOrderEditGetRouteTag::new(self.id).url(),
                    form_post_url: &PurchaseOrderEditPostRouteTag::new(self.id).path(),
                    modal_uid: PurchaseOrderEditModalKey::ID,
                    label: "Edit",
                    classes: "btn-outline",
                    ..Default::default()
                }))
            }
        } else {
            html! {}
        };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (detail_header(DetailHeader {
                        title: &self.number,
                        actions,
                    }))
                    (label("Date", field_text(FieldText { value: &self.date, classes: "" })))
                    (label("Customer", html! {
                        a class="link" href=(customer_url) { (self.customer_name) }
                    }))
                    (label("Site", html! {
                        a class="link" href=(site_url) { (self.site_name) }
                    }))
                    (label("File", html! {
                        @if let Some(fid) = self.file_id.filter(|&id| id > 0) {
                            a class="link" href=(VNodeDetailRouteTag::new(fid).url()) { (self.file_name) }
                        } @else {
                            (field_text(FieldText { value: "—", classes: "" }))
                        }
                    }))
                    (label("Billing address", field_textarea(FieldTextarea {
                        value: &self.billing_address,
                        classes: "break-words min-w-0 max-w-full overflow-x-hidden",
                    })))
                    (label("Shipping address", field_textarea(FieldTextarea {
                        value: &self.shipping_address,
                        classes: "break-words min-w-0 max-w-full overflow-x-hidden",
                    })))
                    (label("Lines", html! {
                        div class="w-full min-w-0" {
                            div class="overflow-x-auto min-w-0 rounded-box border border-base-300 bg-base-100" {
                                table class="table table-sm min-w-max w-full" {
                                    thead {
                                        tr {
                                            th class="whitespace-nowrap" { "Item code" }
                                            th class="min-w-[12rem] max-w-md" { "Description" }
                                            th class="whitespace-nowrap" { "Unit" }
                                            th class="whitespace-nowrap" { "Delivery date" }
                                            th class="whitespace-nowrap text-end" { "Quantity" }
                                            th class="whitespace-nowrap text-end" { "Rate" }
                                        }
                                    }
                                    tbody {
                                        @if self.lines.is_empty() {
                                            tr {
                                                td colspan="6" class="text-center opacity-50 py-4" { "No lines" }
                                            }
                                        } @else {
                                            @for line in &self.lines {
                                                tr {
                                                    td class="whitespace-nowrap" { (line.item_code) }
                                                    td class="align-top max-w-md min-w-[12rem]" {
                                                        div class="max-w-md whitespace-normal break-words" {
                                                            (line.description)
                                                        }
                                                    }
                                                    td class="whitespace-nowrap" { (line.unit) }
                                                    td class="whitespace-nowrap" { (line.delivery_date) }
                                                    td class="whitespace-nowrap text-end tabular-nums" { (line.quantity) }
                                                    td class="whitespace-nowrap text-end tabular-nums" { (line.rate) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        detail_menu(
            format!("PO: {}", self.number),
            PurchaseOrderDetailRouteTag::new(self.id).url(),
        )
    }
}

impl RenderAppPane for PurchaseOrderDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            self.menu(),
            purchase_order_crumbs(self.id, &self.number, None),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            purchase_order_crumbs(self.id, &self.number, None),
            self.body(),
        )
    }
}

impl RenderTemplate for PurchaseOrderDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Purchase Order",
            chrome,
            self.menu(),
            purchase_order_crumbs(self.id, &self.number, None),
            self.body(),
        )
    }
}

fn purchase_order_form_inputs(
    form: &PurchaseOrderForm,
    customer_display: &str,
    site_display: &str,
    file_display: &str,
    file_select_url: &str,
) -> Markup {
    let mut ctx = FormCtx::form::<PurchaseOrderForm>(CsrfToken::current())
        .value(PurchaseOrderFormField::Number, &form.number)
        .value(PurchaseOrderFormField::Date, &form.date)
        .value(
            PurchaseOrderFormField::CustomerId,
            fk_value(form.customer_id),
        )
        .display(PurchaseOrderFormField::CustomerId, customer_display)
        .value(PurchaseOrderFormField::SiteId, fk_value(form.site_id))
        .display(PurchaseOrderFormField::SiteId, site_display)
        .value(PurchaseOrderFormField::FileId, &form.file_id)
        .display(PurchaseOrderFormField::FileId, file_display)
        .value(
            PurchaseOrderFormField::PaymentTermLinesJson,
            &form.payment_term_lines_json,
        )
        .value(PurchaseOrderFormField::PoLinesJson, &form.po_lines_json)
        .value(
            PurchaseOrderFormField::BillingAddress,
            &form.billing_address,
        )
        .value(
            PurchaseOrderFormField::ShippingAddress,
            &form.shipping_address,
        );
    if !file_select_url.is_empty() {
        ctx = ctx.url(PurchaseOrderFormField::FileId, file_select_url);
    }
    PurchaseOrderForm::render_inputs(&ctx)
}

#[derive(Generic)]
pub struct PurchaseOrderEditModalPage {
    pub id: i64,
    pub form_name: String,
    pub form: PurchaseOrderForm,
    pub customer_display: String,
    pub site_display: String,
    pub file_display: String,
    pub file_select_url: String,
    pub error: String,
}

impl RenderTemplate for PurchaseOrderEditModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let delete_url = PurchaseOrderDeleteGetRouteTag::new(self.id).url();
        modal_keyed::<PurchaseOrderEditModalKey>(
            "!max-w-6xl w-full",
            html! {
                h3 class="font-bold text-lg mb-4" { "Edit purchase order" }
                (form(&CsrfToken::current(), FormOpts {
                    classes: "@container",
                    attrs: form_hx_post_url::<PurchaseOrderEditModalKey>(&modal_edit_post_url(
                        PurchaseOrderEditPostRouteTag::new(self.id),
                        &self.form_name,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: purchase_order_form_inputs(
                        &self.form,
                        &self.customer_display,
                        &self.site_display,
                        &self.file_display,
                        &self.file_select_url,
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                        (button_modal_form(ButtonModalForm {
                            label: "Delete",
                            icon_name: Some("trash"),
                            name: "gandola_manager.PurchaseOrderDeleteForm",
                            href: &delete_url,
                            form_post_url: &delete_url,
                            modal_uid: PurchaseOrderDeleteModalKey::ID,
                            classes: "btn-error",
                            ..Default::default()
                        }))
                    },
                    ..Default::default()
                }))
            },
        )
    }
}

#[derive(Generic)]
pub struct PurchaseOrderCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub form: PurchaseOrderForm,
    pub customer_display: String,
    pub site_display: String,
    pub file_display: String,
    pub file_select_url: String,
    pub error: String,
}

impl RenderTemplate for PurchaseOrderCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            "gandola_manager.PurchaseOrderCreateForm"
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<PurchaseOrderCreateModalKey>(
            "!max-w-6xl w-full",
            form(
                &CsrfToken::current(),
                FormOpts {
                    title: "Create purchase order",
                    subtitle: "Create a new purchase order",
                    classes: "@container",
                    attrs: form_hx_post_url::<PurchaseOrderCreateModalKey>(
                        &modal_create_post_query(
                            PurchaseOrderCreatePostRouteTag,
                            form_name,
                            &self.refresh_table,
                            &self.target_input,
                        ),
                    ),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: purchase_order_form_inputs(
                        &self.form,
                        &self.customer_display,
                        &self.site_display,
                        &self.file_display,
                        &self.file_select_url,
                    ),
                    actions: html! {
                        (container_row("flex justify-end gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save purchase order",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                        }))
                    },
                    ..Default::default()
                },
            ),
        )
    }
}

#[derive(Generic)]
pub struct PurchaseOrderSelectPage {
    pub purchase_orders: ObjectList<PurchaseOrderRow>,
    pub filter_number: String,
    pub sort: String,
    pub path_and_query: String,
    pub target_input: String,
    pub can_edit: bool,
    pub page_size: u32,
}

impl RenderPickerSelect<PurchaseOrderSelectTableKey, PurchaseOrderSelectModalKey>
    for PurchaseOrderSelectPage
{
    fn render_table(&self) -> Markup {
        let target = if self.target_input.is_empty() {
            "PurchaseOrders"
        } else {
            self.target_input.as_str()
        };
        let (number_sort, number_label) =
            col_sort(&self.path_and_query, "Number", "Number", &self.sort);
        let (date_sort, date_label) = col_sort(&self.path_and_query, "Date", "Date", &self.sort);
        let headers = [
            TableColumnHeader {
                key: "Number",
                label: &number_label,
                sort_url: Some(&number_sort),
                push_url: false,
            },
            TableColumnHeader {
                key: "Date",
                label: &date_label,
                sort_url: Some(&date_sort),
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .purchase_orders
            .items
            .iter()
            .map(|po| TableRow {
                attrs: row_attr_select_multi(target, &po.id.to_string(), &po.number),
                cells: vec![
                    field_text(FieldText {
                        value: &po.number,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &po.date,
                        classes: "",
                    }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_picker_route::<
                        PurchaseOrderSelectTableKey,
                        PurchaseOrderSelectModalKey,
                        PurchaseOrderSelectRouteTag,
                    >(PurchaseOrderSelectRouteTag),
                    inputs: html! {
                        (with_list_filter_common(
                            PurchaseOrderFilterForm::render_inputs(
                                &FormCtx::form::<PurchaseOrderFilterForm>(CsrfToken::current())
                                    .value(PurchaseOrderFilterFormField::Number, &self.filter_number),
                            ),
                            self.page_size,
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
                (picker_create_button::<PurchaseOrderCreateModalKey>(
                    &self.target_input,
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            };
        }
        let pagination = render_pagination::<PurchaseOrderSelectTableKey>(
            &self.path_and_query,
            self.purchase_orders.number,
            self.purchase_orders.num_pages,
        );
        data_table_list_refresh::<PurchaseOrderSelectTableKey>(
            "Select Purchase Orders",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for PurchaseOrderSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
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
            format!("#{}", GandolaDeleteModalKey::ID)
        } else {
            format!("#{}", self.modal_uid)
        };
        let uid = if self.modal_uid.is_empty() {
            GandolaDeleteModalKey::ID
        } else {
            self.modal_uid.as_str()
        };
        let post_url = if self.modal_uid == SiteDeleteModalKey::ID {
            SiteDeletePostRouteTag::new(self.id).url()
        } else if self.modal_uid == PurchaseOrderDeleteModalKey::ID {
            PurchaseOrderDeletePostRouteTag::new(self.id).url()
        } else {
            GandolaDeletePostRouteTag::new(self.id).url()
        };
        modal(lariv_rs::components::Modal {
            uid,
            children: delete_confirmation(DeleteConfirmation {
                title: "Confirm Deletion",
                message: &self.message,
                attrs: form_hx_post_selector(&post_url, &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
