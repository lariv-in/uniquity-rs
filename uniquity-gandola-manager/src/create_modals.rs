use super::keys::{
    GandolaCreateModalKey, GandolaSelectModalKey, GandolaSelectTableKey,
    PurchaseOrderCreateModalKey, PurchaseOrderSelectModalKey, PurchaseOrderSelectTableKey,
    SiteCreateModalKey, SiteFkSelectModalKey, SiteFkSelectTableKey, SiteSelectModalKey,
    SiteSelectTableKey,
};
use super::routes::{
    GandolaCreateGetRouteTag, GandolaCreatePostRouteTag, PurchaseOrderCreateGetRouteTag,
    PurchaseOrderCreatePostRouteTag, SiteCreateGetRouteTag, SiteCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    GandolaCreateModalKey,
    GandolaCreateGetRouteTag,
    GandolaCreatePostRouteTag,
    "gandola_manager.GandolaCreateForm"
);
lariv_rs::impl_picker_modal!(GandolaSelectModalKey, GandolaSelectTableKey);

lariv_rs::impl_create_modal!(
    SiteCreateModalKey,
    SiteCreateGetRouteTag,
    SiteCreatePostRouteTag,
    "gandola_manager.SiteCreateForm"
);
lariv_rs::impl_picker_modal!(SiteSelectModalKey, SiteSelectTableKey);
lariv_rs::impl_picker_modal!(SiteFkSelectModalKey, SiteFkSelectTableKey);

lariv_rs::impl_create_modal!(
    PurchaseOrderCreateModalKey,
    PurchaseOrderCreateGetRouteTag,
    PurchaseOrderCreatePostRouteTag,
    "gandola_manager.PurchaseOrderCreateForm"
);
lariv_rs::impl_picker_modal!(PurchaseOrderSelectModalKey, PurchaseOrderSelectTableKey);
