use maud::Markup;

use lariv_rs::components::{AppLayoutHtml, LayoutSidebar, MainContentHtml, layout_main, layout_sidebar};

use crate::accounting_sidebar;

pub fn app_scaffold(
    title: &str,
    chrome: &lariv_rs::components::ShellChrome,
    body: Markup,
    current_path: &str,
) -> Markup {
    app_scaffold_with_sidebar(
        title,
        chrome,
        accounting_sidebar::accounting_sidebar(current_path),
        body,
    )
}

pub fn app_scaffold_with_sidebar(
    title: &str,
    chrome: &lariv_rs::components::ShellChrome,
    sidebar: Markup,
    body: Markup,
) -> Markup {
    lariv_rs::components::shell_scaffold(lariv_rs::components::ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        sidebar,
        body,
        ..Default::default()
    })
}

pub fn render_pagination<K: lariv_rs::components::SwapKey>(
    path_and_query: &str,
    number: u32,
    num_pages: u32,
) -> Markup {
    use lariv_rs::components::{PaginationPage, TablePagination, pagination_pages, table_pagination};

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

/// Pagination for FK picker modals — swaps the dialog, not the inner table fragment.
pub fn render_picker_pagination<M: lariv_rs::components::SwapKey>(
    path_and_query: &str,
    number: u32,
    num_pages: u32,
) -> Markup {
    use lariv_rs::components::{
        PaginationPage, TablePagination, pagination_pages, table_pagination_picker,
    };

    let owned = pagination_pages(path_and_query, number, num_pages, false);
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
    table_pagination_picker(TablePagination {
        pages: &pages,
        hx_target: M::SELECTOR,
    })
}

pub fn layout_main_content(content: Markup) -> MainContentHtml {
    layout_main(content)
}

pub fn layout_with_sidebar(current_path: &str, content: Markup) -> AppLayoutHtml {
    layout_with_entity_sidebar(accounting_sidebar::accounting_sidebar(current_path), content)
}

pub fn layout_with_entity_sidebar(sidebar: Markup, content: Markup) -> AppLayoutHtml {
    layout_sidebar(LayoutSidebar { sidebar, content })
}
