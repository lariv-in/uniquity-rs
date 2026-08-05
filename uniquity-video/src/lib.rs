#![feature(impl_trait_in_assoc_type)]

//! Uniquity video pipeline plugin.

pub mod apps;
pub mod config;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod migrations;
pub mod routes;
pub mod scope;
pub mod state;
pub mod templates;
pub mod youtube;

use frunk::{HCons, HNil, hlist::HList};

use lariv_rs::{
    app::App,
    capability::CapStore,
    config::{ConfigCap, ConfigTag},
    db::{DbCap, DbTag},
    hooks::AttachState,
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::{GetByCapTag, GetByTag},
    },
};

use config::{VideoConfig, VideoConfigTag};
use state::VideoState;

pub struct UniquityVideoTag;

lariv_rs::define_passthrough_cap!(UniquityVideoStateCap, UniquityVideoTag, VideoState);

lariv_rs::define_plugin_install! {
    plugin: UniquityVideoTag;
    steps: [
        apps(apps::Hook),
        migrations(migrations::Hook),
        config(VideoConfigTag, VideoConfig),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        http(routes::Hook),
        state(StateHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, CfgIdx, Configs, VidCfgIdx, DbIdx, TagProof>
    AttachState<L, (CfgIdx, Configs, VidCfgIdx, DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<HNil, Configs>>,
    Configs: GetByTag<VideoConfigTag, VidCfgIdx, Value = VideoConfig>,
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<UniquityVideoTag, TagProof>,
{
    type Output = HCons<UniquityVideoStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let config = <Configs as GetByTag<VideoConfigTag, VidCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(VideoState::new(conn, config)))
    }
}
