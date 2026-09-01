//! Query helpers and M2M sync for the video pipeline.

use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Select,
};

use lariv_rs::plugins::{
    filesystem::entities::filesystem_node::{self, Entity as VNodeEntity},
    users::state::AuthContext,
};
use uniquity_employees::{
    entities::employee::{self, Entity as EmployeeEntity},
    scope::employee_display_name,
};

use super::entities::{
    edited_video::{self, Entity as EditedVideoEntity},
    published_video::{self, Entity as PublishedVideoEntity},
    raw_footage::{self, Entity as RawFootageEntity},
    raw_footage_file::{self, Entity as RawFootageFileEntity},
};

#[derive(Clone, Debug)]
pub struct RawFootageRow {
    pub id: i64,
    pub title: String,
    pub assigned_to_name: String,
}

#[derive(Clone, Debug)]
pub struct EditedVideoRow {
    pub id: i64,
    pub raw_title: String,
    pub output_name: String,
}

#[derive(Clone, Debug)]
pub struct PublishedVideoRow {
    pub id: i64,
    pub youtube_id: String,
    pub raw_title: String,
}

#[derive(Clone, Debug)]
pub struct RawFootageDetail {
    pub id: i64,
    pub title: String,
    pub assigned_to_id: i64,
    pub assigned_to_name: String,
    pub file_ids: Vec<i64>,
    pub file_names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct EditedVideoDetail {
    pub id: i64,
    pub raw_footage_id: i64,
    pub raw_title: String,
    pub assigned_to_id: i64,
    pub assigned_to_name: String,
    pub raw_file_names: Vec<String>,
    pub edited_v_node_id: i64,
    pub output_name: String,
}

#[derive(Clone, Debug)]
pub struct PublishedVideoDetail {
    pub id: i64,
    pub edited_video_id: i64,
    pub youtube_id: String,
    pub raw_title: String,
    pub assigned_to_id: i64,
    pub assigned_to_name: String,
}

pub fn scope_raw_list(
    query: Select<RawFootageEntity>,
    _auth: &AuthContext,
) -> Select<RawFootageEntity> {
    query
}

pub async fn scope_raw_select(
    query: Select<RawFootageEntity>,
    db: &DatabaseConnection,
    auth: &AuthContext,
) -> Select<RawFootageEntity> {
    if auth.user.is_superuser {
        return query;
    }
    let Ok(Some(emp)) = EmployeeEntity::find()
        .filter(employee::Column::UserId.eq(auth.user.id))
        .one(db)
        .await
    else {
        return query.filter(Expr::cust("1 = 0"));
    };
    query.filter(raw_footage::Column::AssignedToId.eq(emp.id))
}

pub async fn load_vnode_names(
    db: &DatabaseConnection,
    ids: &[i64],
) -> std::collections::HashMap<i64, String> {
    if ids.is_empty() {
        return std::collections::HashMap::new();
    }
    VNodeEntity::find()
        .filter(filesystem_node::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|n| (n.id, n.name))
        .collect()
}

pub async fn load_raw_file_ids(db: &DatabaseConnection, raw_id: i64) -> Vec<i64> {
    RawFootageFileEntity::find()
        .filter(raw_footage_file::Column::RawFootageId.eq(raw_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.v_node_id)
        .collect()
}

pub async fn sync_raw_footage_files(
    db: &DatabaseConnection,
    raw_id: i64,
    file_ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    RawFootageFileEntity::delete_many()
        .filter(raw_footage_file::Column::RawFootageId.eq(raw_id))
        .exec(db)
        .await?;
    for vid in file_ids {
        raw_footage_file::ActiveModel {
            raw_footage_id: Set(raw_id),
            v_node_id: Set(*vid),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub async fn query_raw_footages(
    db: &DatabaseConnection,
    auth: &AuthContext,
    title: Option<&str>,
    page: u32,
    page_size: u64,
    sort: Option<&str>,
) -> (Vec<RawFootageRow>, u32, u64) {
    let mut query = RawFootageEntity::find();
    query = scope_raw_list(query, auth);
    if let Some(t) = title.filter(|s| !s.is_empty()) {
        query = query.filter(raw_footage::Column::Title.contains(t));
    }
    let sort = sort.unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Title DESC") => {
            query.order_by_desc(raw_footage::Column::Title)
        }
        s if s.eq_ignore_ascii_case("Title ASC") || s.eq_ignore_ascii_case("Title") => {
            query.order_by_asc(raw_footage::Column::Title)
        }
        _ => query.order_by_desc(raw_footage::Column::Id),
    };
    let page = page.max(1);
    let paginator = query.paginate(db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::new();
    for m in models {
        let assigned_to_name = employee_display_name(db, m.assigned_to_id).await;
        rows.push(RawFootageRow {
            id: m.id,
            title: m.title,
            assigned_to_name,
        });
    }
    (rows, page, total)
}

pub async fn find_raw_footage(db: &DatabaseConnection, id: i64) -> Option<RawFootageDetail> {
    let m =
        lariv_rs::web::opt_or_log(RawFootageEntity::find_by_id(id).one(db).await, "find by id")?;
    let file_ids = load_raw_file_ids(db, m.id).await;
    let names = load_vnode_names(db, &file_ids).await;
    let file_names: Vec<String> = file_ids
        .iter()
        .filter_map(|id| names.get(id).cloned())
        .collect();
    Some(RawFootageDetail {
        id: m.id,
        title: m.title,
        assigned_to_id: m.assigned_to_id,
        assigned_to_name: employee_display_name(db, m.assigned_to_id).await,
        file_ids,
        file_names,
    })
}

pub async fn query_edited_videos(
    db: &DatabaseConnection,
    page: u32,
    page_size: u64,
) -> (Vec<EditedVideoRow>, u32, u64) {
    let query = EditedVideoEntity::find().order_by_desc(edited_video::Column::Id);
    let page = page.max(1);
    let paginator = query.paginate(db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::new();
    for m in models {
        let raw_title = lariv_rs::web::opt_or_log(
            RawFootageEntity::find_by_id(m.raw_footage_id).one(db).await,
            "find by id",
        )
        .map(|r| r.title)
        .unwrap_or_else(|| "—".into());
        let output_name = load_vnode_names(db, &[m.edited_v_node_id])
            .await
            .get(&m.edited_v_node_id)
            .cloned()
            .unwrap_or_else(|| "—".into());
        rows.push(EditedVideoRow {
            id: m.id,
            raw_title,
            output_name,
        });
    }
    (rows, page, total)
}

pub async fn find_edited_video(db: &DatabaseConnection, id: i64) -> Option<EditedVideoDetail> {
    let m = lariv_rs::web::opt_or_log(
        EditedVideoEntity::find_by_id(id).one(db).await,
        "find by id",
    )?;
    let raw = find_raw_footage(db, m.raw_footage_id).await?;
    let output_name = load_vnode_names(db, &[m.edited_v_node_id])
        .await
        .get(&m.edited_v_node_id)
        .cloned()
        .unwrap_or_default();
    Some(EditedVideoDetail {
        id: m.id,
        raw_footage_id: m.raw_footage_id,
        raw_title: raw.title,
        assigned_to_id: raw.assigned_to_id,
        assigned_to_name: raw.assigned_to_name,
        raw_file_names: raw.file_names,
        edited_v_node_id: m.edited_v_node_id,
        output_name,
    })
}

pub async fn query_published_videos(
    db: &DatabaseConnection,
    page: u32,
    page_size: u64,
    sort: Option<&str>,
) -> (Vec<PublishedVideoRow>, u32, u64) {
    let mut query = PublishedVideoEntity::find();
    let sort = sort.unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("YouTubeID DESC") => {
            query.order_by_desc(published_video::Column::YouTubeVideoId)
        }
        s if s.eq_ignore_ascii_case("YouTubeID ASC") || s.eq_ignore_ascii_case("YouTubeID") => {
            query.order_by_asc(published_video::Column::YouTubeVideoId)
        }
        _ => query.order_by_desc(published_video::Column::Id),
    };
    let page = page.max(1);
    let paginator = query.paginate(db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::new();
    for m in models {
        let raw_title = if let Some(ev) = lariv_rs::web::opt_or_log(
            EditedVideoEntity::find_by_id(m.edited_video_id)
                .one(db)
                .await,
            "find by id",
        ) {
            lariv_rs::web::opt_or_log(
                RawFootageEntity::find_by_id(ev.raw_footage_id)
                    .one(db)
                    .await,
                "find by id",
            )
            .map(|r| r.title)
            .unwrap_or_else(|| "—".into())
        } else {
            "—".into()
        };
        rows.push(PublishedVideoRow {
            id: m.id,
            youtube_id: m.you_tube_video_id,
            raw_title,
        });
    }
    (rows, page, total)
}

pub async fn find_published_video(
    db: &DatabaseConnection,
    id: i64,
) -> Option<PublishedVideoDetail> {
    let m = lariv_rs::web::opt_or_log(
        PublishedVideoEntity::find_by_id(id).one(db).await,
        "find by id",
    )?;
    let edited = find_edited_video(db, m.edited_video_id).await?;
    Some(PublishedVideoDetail {
        id: m.id,
        edited_video_id: m.edited_video_id,
        youtube_id: m.you_tube_video_id,
        raw_title: edited.raw_title,
        assigned_to_id: edited.assigned_to_id,
        assigned_to_name: edited.assigned_to_name,
    })
}

pub async fn raw_footage_title(db: &DatabaseConnection, id: i64) -> String {
    lariv_rs::web::opt_or_log(RawFootageEntity::find_by_id(id).one(db).await, "find by id")
        .map(|r| r.title)
        .unwrap_or_default()
}

pub async fn edited_video_display(db: &DatabaseConnection, id: i64) -> String {
    let Some(ev) = lariv_rs::web::opt_or_log(
        EditedVideoEntity::find_by_id(id).one(db).await,
        "find by id",
    ) else {
        return String::new();
    };
    raw_footage_title(db, ev.raw_footage_id).await
}

pub async fn vnode_display_name(db: &DatabaseConnection, id: i64) -> String {
    load_vnode_names(db, &[id])
        .await
        .get(&id)
        .cloned()
        .unwrap_or_default()
}
