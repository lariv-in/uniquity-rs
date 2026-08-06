use std::collections::HashMap;

use lariv_rs::components::{attrs::HtmlAttrs, htmx::{row_attr_select_extra, row_attr_select_multi}};
use lariv_rs::html_form::FormFieldKey;

use crate::{
    account_validation::{ACCOUNT_PARENT_UP_ROW_ID, BALANCE_TYPE_SCOPE_QUERY_PARAM},
    forms::AccountFormField,
    routes::AccountSelectRouteTag,
};

pub const ACCOUNT_SELECTION_MODAL_ID: &str = "finance-account-selection-modal";

pub fn account_select_drill_url(path_and_query: &str, parent_id: i64) -> String {
    merge_select_query(AccountSelectRouteTag.url().as_str(), path_and_query, |q| {
        q.insert("ParentID".into(), parent_id.to_string());
        q.insert("page".into(), "1".into());
    })
}

pub fn account_select_parent_up_url(
    path_and_query: &str,
    grandparent_id: Option<i64>,
) -> Option<String> {
    Some(merge_select_query(
        AccountSelectRouteTag.url().as_str(),
        path_and_query,
        |q| {
            q.remove("ParentID");
            if let Some(gpid) = grandparent_id.filter(|&id| id > 0) {
                q.insert("ParentID".into(), gpid.to_string());
            }
            q.insert("page".into(), "1".into());
        },
    ))
}

fn merge_select_query(
    base: &str,
    path_and_query: &str,
    patch: impl FnOnce(&mut HashMap<String, String>),
) -> String {
    let mut params = parse_query(path_and_query);
    params.remove("page");
    patch(&mut params);
    if params.is_empty() {
        return base.to_string();
    }
    let qs: Vec<String> = params
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    format!("{base}?{}", qs.join("&"))
}

fn parse_query(path_and_query: &str) -> HashMap<String, String> {
    let q = path_and_query.split('?').nth(1).unwrap_or("");
    let mut out = HashMap::new();
    for pair in q.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            out.insert(k.to_string(), v.to_string());
        }
    }
    out
}

pub fn account_selection_row_attrs(
    row_id: i64,
    is_group: bool,
    balance_type: &str,
    target_input: &str,
    display: &str,
    path_and_query: &str,
    parent_up_url: Option<&str>,
    drill_parent_id: i64,
) -> HtmlAttrs {
    if row_id == ACCOUNT_PARENT_UP_ROW_ID {
        if let Some(url) = parent_up_url {
            return HtmlAttrs::new()
                .set("class", "cursor-pointer hover:bg-base-200 transition-colors")
                .set("hx-get", url)
                .set("hx-target", &format!("#{ACCOUNT_SELECTION_MODAL_ID}"))
                .set("hx-swap", "outerHTML")
                .set("hx-push-url", "false");
        }
    }
    let parent_picker = target_input == AccountFormField::ParentId.target_input();
    let child_picker = target_input == AccountFormField::ChildIds.target_input();
    if is_group && !parent_picker && !child_picker {
        let url = account_select_drill_url(path_and_query, drill_parent_id);
        return HtmlAttrs::new()
            .set("class", "cursor-pointer hover:bg-base-200 transition-colors")
            .set("hx-get", &url)
            .set("hx-target", &format!("#{ACCOUNT_SELECTION_MODAL_ID}"))
            .set("hx-swap", "outerHTML")
            .set("hx-push-url", "false");
    }
    if child_picker {
        return row_attr_select_multi(target_input, &row_id.to_string(), display);
    }
    row_attr_select_extra(
        target_input,
        &row_id.to_string(),
        display,
        &[("balance_type", balance_type)],
    )
}

/// HTMX attrs to drill into a group account row in the parent picker.
pub fn account_selection_drill_attrs(path_and_query: &str, drill_parent_id: i64) -> HtmlAttrs {
    let url = account_select_drill_url(path_and_query, drill_parent_id);
    HtmlAttrs::new()
        .set("type", "button")
        .set("class", "btn btn-ghost btn-xs")
        .set("hx-get", url)
        .set("hx-target", &format!("#{ACCOUNT_SELECTION_MODAL_ID}"))
        .set("hx-swap", "outerHTML")
        .set("hx-push-url", "false")
}

pub fn account_select_url_with_balance_type(balance_type: &str) -> String {
    format!(
        "{}?{}={balance_type}",
        AccountSelectRouteTag.url(),
        BALANCE_TYPE_SCOPE_QUERY_PARAM,
    )
}
