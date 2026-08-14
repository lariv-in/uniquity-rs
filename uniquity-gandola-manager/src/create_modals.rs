use super::keys::{
    GandolaCreateModalKey, GandolaSelectModalKey, GandolaSelectTableKey, SiteCreateModalKey,
    SiteSelectModalKey, SiteSelectTableKey,
};
use super::routes::{
    GandolaCreateGetRouteTag, GandolaCreatePostRouteTag, SiteCreateGetRouteTag,
    SiteCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    GandolaCreateModalKey,
    GandolaCreateGetRouteTag,
    GandolaCreatePostRouteTag,
    "p_gandola_manager.GandolaCreateForm"
);
lariv_rs::impl_picker_modal!(GandolaSelectModalKey, GandolaSelectTableKey);

lariv_rs::impl_create_modal!(
    SiteCreateModalKey,
    SiteCreateGetRouteTag,
    SiteCreatePostRouteTag,
    "p_gandola_manager.SiteCreateForm"
);
lariv_rs::impl_picker_modal!(SiteSelectModalKey, SiteSelectTableKey);
