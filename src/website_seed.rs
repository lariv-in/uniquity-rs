//! Idempotent seed for the Uniquity Ventures public homepage, static media, and Custom theme.

use chrono::Utc;
use lariv_rs::plugins::filesystem::node::{self, NodeFile};
use lariv_rs::plugins::filesystem::storage::DynFilestore;
use lariv_rs::plugins::website::{
    builder_assets::public_asset_url,
    entities::{
        WebsitePreferences,
        db_route::{self, Column as DbRouteColumn, Entity as DbRouteEntity},
    },
    preferences::{self, CUSTOM_THEME_ID},
    render,
    state::WebsiteState,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tokio::io::AsyncReadExt;

const HOMEPAGE_HTML: &str = include_str!("../assets/homepage.html");
const THEME_CSS: &[u8] = include_bytes!("../assets/theme/uniquity.css");
const THEME_JS: &[u8] = include_bytes!("../assets/theme/uniquity.js");
const ROUTE_PATH: &str = "/";
const PAGE_NAME: &str = "index.html";
const THEME_CSS_NAME: &str = "uniquity.css";
const THEME_JS_NAME: &str = "uniquity.js";
const THEME: &str = CUSTOM_THEME_ID;

struct StaticAsset {
    name: &'static str,
    bytes: &'static [u8],
}

const STATIC_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        name: "logo.svg",
        bytes: include_bytes!("../assets/static/logo.svg"),
    },
    StaticAsset {
        name: "hero.jpg",
        bytes: include_bytes!("../assets/static/hero.jpg"),
    },
    StaticAsset {
        name: "equipment.png",
        bytes: include_bytes!("../assets/static/equipment.png"),
    },
];

pub async fn ensure_homepage(state: &WebsiteState) -> anyhow::Result<()> {
    ensure_homepage_state(&state.db, state.store.as_ref()).await
}

async fn ensure_homepage_state(
    db: &DatabaseConnection,
    store: &DynFilestore,
) -> anyhow::Result<()> {
    ensure_static_assets(db, store).await?;
    remove_legacy_static_routes(db).await?;
    reject_nonportable_asset_urls("homepage.html", HOMEPAGE_HTML)?;
    ensure_custom_theme(db, store).await?;
    let (page, page_rewritten) = ensure_page_vnode(db, store, HOMEPAGE_HTML.as_bytes()).await?;
    ensure_db_route(db, ROUTE_PATH, page.id, THEME, page_rewritten).await?;
    tracing::info!(page_id = page.id, "uniquity website: homepage route ready");
    Ok(())
}

/// Seeds theme CSS/JS under `website/themes/` and points Custom theme preferences at them.
async fn ensure_custom_theme(db: &DatabaseConnection, store: &DynFilestore) -> anyhow::Result<()> {
    let segments = ["website".into(), "themes".into()];
    let parent_id = node::ensure_directory_path(db, store, None, &segments)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let parent = match parent_id {
        Some(id) => match node::get_by_id(db, id).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "get node by id for website themes parent");
                None
            }
        },
        None => None,
    };

    let css = ensure_file_vnode(
        db,
        store,
        parent_id,
        parent.as_ref(),
        THEME_CSS_NAME,
        THEME_CSS,
    )
    .await?
    .0;
    let js = ensure_file_vnode(
        db,
        store,
        parent_id,
        parent.as_ref(),
        THEME_JS_NAME,
        THEME_JS,
    )
    .await?
    .0;

    preferences::save_preferences(
        db,
        WebsitePreferences {
            id: 1,
            created_at: None,
            updated_at: None,
            custom_theme_css_vnode_id: Some(css.id),
            custom_theme_js_vnode_id: Some(js.id),
        },
    )
    .await?;

    tracing::info!(
        css_vnode_id = css.id,
        js_vnode_id = js.id,
        "uniquity website: custom theme preferences ready"
    );
    Ok(())
}

fn first_hardcoded_media_url(source: &str) -> Option<&str> {
    const PREFIX: &str = "/media/";
    let mut from = 0;
    while let Some(rel) = source[from..].find(PREFIX) {
        let start = from + rel;
        let rest = &source[start + PREFIX.len()..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 {
            let mut end = start + PREFIX.len() + digits;
            if source.as_bytes().get(end) == Some(&b'/') {
                end += 1;
            }
            return Some(&source[start..end]);
        }
        from = start + PREFIX.len();
    }
    None
}

fn has_legacy_static_url(source: &str) -> bool {
    source.contains("\"/static/") || source.contains("'/static/")
}

fn reject_nonportable_asset_urls(label: &str, source: &str) -> anyhow::Result<()> {
    if let Some(url) = first_hardcoded_media_url(source) {
        anyhow::bail!(
            "{label} must use media_url(\"/website/static/{{filename}}\"), not hardcoded vnode URL {url}"
        );
    }
    if has_legacy_static_url(source) {
        anyhow::bail!("{label} must use media_url(\"/website/static/{{filename}}\")");
    }
    Ok(())
}

async fn ensure_page_vnode(
    db: &DatabaseConnection,
    store: &DynFilestore,
    html: &[u8],
) -> anyhow::Result<(lariv_rs::plugins::filesystem::entities::VNode, bool)> {
    let segments = ["website".into(), "pages".into()];
    let parent_id = node::ensure_directory_path(db, store, None, &segments)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let parent = match parent_id {
        Some(id) => match node::get_by_id(db, id).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "get node by id for website page parent");
                None
            }
        },
        None => None,
    };

    ensure_file_vnode(db, store, parent_id, parent.as_ref(), PAGE_NAME, html).await
}

/// Seeds blobs under `website/static/` for `media_url("/website/static/{name}")`.
async fn ensure_static_assets(db: &DatabaseConnection, store: &DynFilestore) -> anyhow::Result<()> {
    let segments = ["website".into(), "static".into()];
    let parent_id = node::ensure_directory_path(db, store, None, &segments)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let parent = match parent_id {
        Some(id) => match node::get_by_id(db, id).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "get node by id for website static parent");
                None
            }
        },
        None => None,
    };

    for asset in STATIC_ASSETS {
        let vnode = ensure_file_vnode(
            db,
            store,
            parent_id,
            parent.as_ref(),
            asset.name,
            asset.bytes,
        )
        .await?
        .0;
        tracing::info!(
            name = asset.name,
            vnode_id = vnode.id,
            media_url = %public_asset_url(vnode.id),
            bytes = asset.bytes.len(),
            "uniquity website: static asset ready"
        );
    }
    Ok(())
}

/// Drop leftover `/static/{name}` aliases from earlier seeds.
async fn remove_legacy_static_routes(db: &DatabaseConnection) -> anyhow::Result<()> {
    for asset in STATIC_ASSETS {
        let path = format!("/static/{}", asset.name);
        let res = DbRouteEntity::delete_many()
            .filter(DbRouteColumn::Path.eq(path.clone()))
            .exec(db)
            .await?;
        if res.rows_affected > 0 {
            tracing::info!(path, "uniquity website: removed legacy static route");
        }
    }
    Ok(())
}

async fn ensure_file_vnode(
    db: &DatabaseConnection,
    store: &DynFilestore,
    parent_id: Option<i64>,
    parent: Option<&lariv_rs::plugins::filesystem::entities::VNode>,
    name: &str,
    bytes: &[u8],
) -> anyhow::Result<(lariv_rs::plugins::filesystem::entities::VNode, bool)> {
    if let Some(existing) = node::find_child(db, parent_id, name, false).await? {
        if vnode_bytes_match(store, &existing, bytes).await? {
            return Ok((existing, false));
        }
        tracing::warn!(
            name,
            vnode_id = existing.id,
            stored_path = existing.file_path.as_deref().unwrap_or(""),
            "uniquity website: rewriting vnode blob"
        );
        let updated = render::replace_vnode_content(db, store, existing, bytes)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        return Ok((updated, true));
    }

    tracing::info!(name, "uniquity website: creating vnode");
    let created = node::create(
        db,
        store,
        name.into(),
        false,
        Some(NodeFile::Bytes {
            filename: name.into(),
            data: bytes.to_vec(),
        }),
        parent,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok((created, true))
}

async fn vnode_bytes_match(
    store: &DynFilestore,
    existing: &lariv_rs::plugins::filesystem::entities::VNode,
    bytes: &[u8],
) -> anyhow::Result<bool> {
    let path = existing.file_path.as_deref().unwrap_or("");
    let mut download = match store.open(path, &existing.name).await {
        Ok(d) => d,
        Err(e) if e.is_missing() => {
            tracing::warn!(
                name = %existing.name,
                vnode_id = existing.id,
                stored_path = path,
                "uniquity website: blob missing from store"
            );
            return Ok(false);
        }
        Err(e) => return Err(anyhow::anyhow!("{e}")),
    };
    let mut current = Vec::new();
    download.reader.read_to_end(&mut current).await?;
    Ok(current == bytes)
}

async fn ensure_db_route(
    db: &DatabaseConnection,
    path: &str,
    page_id: i64,
    theme: &str,
    reset_grapes_project: bool,
) -> anyhow::Result<()> {
    if let Some(existing) = DbRouteEntity::find()
        .filter(DbRouteColumn::Path.eq(path))
        .one(db)
        .await?
    {
        let mut am: db_route::ActiveModel = existing.into();
        am.page_id = Set(page_id);
        am.is_active = Set(true);
        am.theme = Set(theme.into());
        if reset_grapes_project {
            // Drop stale GrapesJS project JSON so the builder reloads from seeded HTML.
            am.grapes_project = Set(None);
        }
        am.updated_at = Set(Some(Utc::now()));
        am.update(db).await?;
        tracing::info!(path, page_id, "uniquity website: updated db route");
        return Ok(());
    }

    let now = Utc::now();
    db_route::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        path: Set(path.into()),
        page_id: Set(page_id),
        is_active: Set(true),
        theme: Set(theme.into()),
        grapes_project: Set(None),
    }
    .insert(db)
    .await?;
    tracing::info!(path, page_id, "uniquity website: created db route");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        HOMEPAGE_HTML, THEME_CSS, first_hardcoded_media_url, has_legacy_static_url,
        reject_nonportable_asset_urls,
    };

    #[test]
    fn seeded_assets_use_media_url_not_static_routes() {
        let css = std::str::from_utf8(THEME_CSS).expect("theme css is utf-8");
        for (label, source) in [("homepage.html", HOMEPAGE_HTML), ("uniquity.css", css)] {
            assert_eq!(
                first_hardcoded_media_url(source),
                None,
                "{label} hardcodes a /media/{{id}}/ vnode URL"
            );
            assert!(
                !has_legacy_static_url(source),
                "{label} still references /static/ instead of media_url"
            );
        }
        assert!(
            HOMEPAGE_HTML.contains("media_url('/website/static/"),
            "homepage.html should call media_url(\"/website/static/...\")"
        );
    }

    #[test]
    fn reject_nonportable_asset_urls_catches_legacy_refs() {
        let err = reject_nonportable_asset_urls("page", r#"<img src="/media/23/">"#).unwrap_err();
        assert!(err.to_string().contains("/media/23/"), "{err}");
        let err =
            reject_nonportable_asset_urls("page", r#"<img src="/static/logo.svg">"#).unwrap_err();
        assert!(err.to_string().contains("media_url"), "{err}");
    }
}
