use super::{
    handlers,
    keys::{
        GandolaDeleteModalKey, GandolaSelectModalKey, GandolaSelectTableKey, GandolaSitesTableKey,
        GandolaTableKey, PurchaseOrderDeleteModalKey, PurchaseOrderSelectModalKey,
        PurchaseOrderSelectTableKey, PurchaseOrderTableKey, SiteDeleteModalKey,
        SiteFkSelectModalKey, SiteFkSelectTableKey, SiteGandolasTableKey, SiteSelectModalKey,
        SiteSelectTableKey, SiteTableKey,
    },
};

/// Gandola Manager routes. Allowlist is [`Hr`]; superuser always passes.
pub struct GandolaAccess;

/// Stored role `hr`. Superuser always passes allowlists that name this role.
#[derive(Clone, Copy, Debug, Default)]
pub struct Hr;

impl lariv_rs::plugins::users::role_registry::Role for Hr {
    const NAME: &'static str = "hr";
    const TITLE: &'static str = "HR";
    const DESCRIPTION: &'static str =
        "Accounting, Gandola Manager, the filesystem, and the assistant.";
}

lariv_rs::define_plugin_routes! {
    plugin: GandolaManagerTag;
    routes: [
        get GandolaDefaultRouteTag, "/gandola", handlers::gandolas::list, fragment(GandolaTableKey), authorize(GandolaAccess, [Hr]);
        get GandolaCreateGetRouteTag, "/gandola/create", handlers::gandolas::create_get, modal, authorize(GandolaAccess, [Hr]);
        post GandolaCreatePostRouteTag, "/gandola/create", handlers::gandolas::create_post, authorize(GandolaAccess, [Hr]);
        get GandolaDetailRouteTag, "/gandola/g/{id}", handlers::gandolas::detail, fragment(GandolaSitesTableKey), authorize(GandolaAccess, [Hr]);
        get GandolaEditGetRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_get, modal, authorize(GandolaAccess, [Hr]);
        post GandolaEditPostRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_post, authorize(GandolaAccess, [Hr]);
        get GandolaDeleteGetRouteTag, "/gandola/g/{id}/delete", handlers::gandolas::delete_get, modal, authorize(GandolaAccess, [Hr]);
        post GandolaDeletePostRouteTag, "/gandola/g/{id}/delete", bare handlers::gandolas::delete_post, fragment(GandolaDeleteModalKey), authorize(GandolaAccess, [Hr]);
        get GandolaSelectRouteTag, "/gandola/pick", handlers::gandolas::select, multi_select(GandolaSelectTableKey, GandolaSelectModalKey), authorize(GandolaAccess, [Hr]);

        get SiteDefaultRouteTag, "/gandola/sites", handlers::sites::list, fragment(SiteTableKey), authorize(GandolaAccess, [Hr]);
        get SiteCreateGetRouteTag, "/gandola/sites/create", handlers::sites::create_get, modal, authorize(GandolaAccess, [Hr]);
        post SiteCreatePostRouteTag, "/gandola/sites/create", handlers::sites::create_post, authorize(GandolaAccess, [Hr]);
        get SiteDetailRouteTag, "/gandola/sites/s/{id}", handlers::sites::detail, fragment(SiteGandolasTableKey), authorize(GandolaAccess, [Hr]);
        get SiteEditGetRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_get, modal, authorize(GandolaAccess, [Hr]);
        post SiteEditPostRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_post, authorize(GandolaAccess, [Hr]);
        get SiteDeleteGetRouteTag, "/gandola/sites/s/{id}/delete", handlers::sites::delete_get, modal, authorize(GandolaAccess, [Hr]);
        post SiteDeletePostRouteTag, "/gandola/sites/s/{id}/delete", bare handlers::sites::delete_post, fragment(SiteDeleteModalKey), authorize(GandolaAccess, [Hr]);
        get SiteSelectRouteTag, "/gandola/sites/pick", handlers::sites::select, multi_select(SiteSelectTableKey, SiteSelectModalKey), authorize(GandolaAccess, [Hr]);
        get SiteFkSelectRouteTag, "/gandola/sites/pick-site", handlers::sites::fk_select, fk_select(SiteFkSelectTableKey, SiteFkSelectModalKey), authorize(GandolaAccess, [Hr]);

        get GandolaPreferencesRouteTag, "/gandola/preferences", handlers::preferences::get, authorize(GandolaAccess, [Hr]);
        post GandolaPreferencesPostRouteTag, "/gandola/preferences", handlers::preferences::post, authorize(GandolaAccess, [Hr]);

        get PurchaseOrderDefaultRouteTag, "/gandola/purchase-orders", handlers::purchase_orders::list, fragment(PurchaseOrderTableKey), authorize(GandolaAccess, [Hr]);
        get PurchaseOrderCreateGetRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_get, modal, authorize(GandolaAccess, [Hr]);
        post PurchaseOrderCreatePostRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_post, authorize(GandolaAccess, [Hr]);
        get PurchaseOrderDetailRouteTag, "/gandola/purchase-orders/po/{id}", handlers::purchase_orders::detail, authorize(GandolaAccess, [Hr]);
        get PurchaseOrderEditGetRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_get, modal, authorize(GandolaAccess, [Hr]);
        post PurchaseOrderEditPostRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_post, authorize(GandolaAccess, [Hr]);
        get PurchaseOrderDeleteGetRouteTag, "/gandola/purchase-orders/po/{id}/delete", handlers::purchase_orders::delete_get, modal, authorize(GandolaAccess, [Hr]);
        post PurchaseOrderDeletePostRouteTag, "/gandola/purchase-orders/po/{id}/delete", bare handlers::purchase_orders::delete_post, fragment(PurchaseOrderDeleteModalKey), authorize(GandolaAccess, [Hr]);
        get PurchaseOrderSelectRouteTag, "/gandola/purchase-orders/pick", handlers::purchase_orders::select, multi_select(PurchaseOrderSelectTableKey, PurchaseOrderSelectModalKey), authorize(GandolaAccess, [Hr]);
    ]
}
