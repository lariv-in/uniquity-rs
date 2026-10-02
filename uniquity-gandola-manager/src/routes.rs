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

/// Gandola Manager routes. Allowlist is `hr`; superuser always passes.
pub struct GandolaAccess;

lariv_rs::define_plugin_routes! {
    plugin: GandolaManagerTag;
    routes: [
        get GandolaDefaultRouteTag, "/gandola", handlers::gandolas::list, fragment(GandolaTableKey), authorize(GandolaAccess, ["hr"]);
        get GandolaCreateGetRouteTag, "/gandola/create", handlers::gandolas::create_get, modal, authorize(GandolaAccess, ["hr"]);
        post GandolaCreatePostRouteTag, "/gandola/create", handlers::gandolas::create_post, authorize(GandolaAccess, ["hr"]);
        get GandolaDetailRouteTag, "/gandola/g/{id}", handlers::gandolas::detail, fragment(GandolaSitesTableKey), authorize(GandolaAccess, ["hr"]);
        get GandolaEditGetRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_get, modal, authorize(GandolaAccess, ["hr"]);
        post GandolaEditPostRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_post, authorize(GandolaAccess, ["hr"]);
        get GandolaDeleteGetRouteTag, "/gandola/g/{id}/delete", handlers::gandolas::delete_get, modal, authorize(GandolaAccess, ["hr"]);
        post GandolaDeletePostRouteTag, "/gandola/g/{id}/delete", bare handlers::gandolas::delete_post, fragment(GandolaDeleteModalKey), authorize(GandolaAccess, ["hr"]);
        get GandolaSelectRouteTag, "/gandola/pick", handlers::gandolas::select, multi_select(GandolaSelectTableKey, GandolaSelectModalKey), authorize(GandolaAccess, ["hr"]);

        get SiteDefaultRouteTag, "/gandola/sites", handlers::sites::list, fragment(SiteTableKey), authorize(GandolaAccess, ["hr"]);
        get SiteCreateGetRouteTag, "/gandola/sites/create", handlers::sites::create_get, modal, authorize(GandolaAccess, ["hr"]);
        post SiteCreatePostRouteTag, "/gandola/sites/create", handlers::sites::create_post, authorize(GandolaAccess, ["hr"]);
        get SiteDetailRouteTag, "/gandola/sites/s/{id}", handlers::sites::detail, fragment(SiteGandolasTableKey), authorize(GandolaAccess, ["hr"]);
        get SiteEditGetRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_get, modal, authorize(GandolaAccess, ["hr"]);
        post SiteEditPostRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_post, authorize(GandolaAccess, ["hr"]);
        get SiteDeleteGetRouteTag, "/gandola/sites/s/{id}/delete", handlers::sites::delete_get, modal, authorize(GandolaAccess, ["hr"]);
        post SiteDeletePostRouteTag, "/gandola/sites/s/{id}/delete", bare handlers::sites::delete_post, fragment(SiteDeleteModalKey), authorize(GandolaAccess, ["hr"]);
        get SiteSelectRouteTag, "/gandola/sites/pick", handlers::sites::select, multi_select(SiteSelectTableKey, SiteSelectModalKey), authorize(GandolaAccess, ["hr"]);
        get SiteFkSelectRouteTag, "/gandola/sites/pick-site", handlers::sites::fk_select, fk_select(SiteFkSelectTableKey, SiteFkSelectModalKey), authorize(GandolaAccess, ["hr"]);

        get GandolaPreferencesRouteTag, "/gandola/preferences", handlers::preferences::get, authorize(GandolaAccess, ["hr"]);
        post GandolaPreferencesPostRouteTag, "/gandola/preferences", handlers::preferences::post, authorize(GandolaAccess, ["hr"]);

        get PurchaseOrderDefaultRouteTag, "/gandola/purchase-orders", handlers::purchase_orders::list, fragment(PurchaseOrderTableKey), authorize(GandolaAccess, ["hr"]);
        get PurchaseOrderCreateGetRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_get, modal, authorize(GandolaAccess, ["hr"]);
        post PurchaseOrderCreatePostRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_post, authorize(GandolaAccess, ["hr"]);
        get PurchaseOrderDetailRouteTag, "/gandola/purchase-orders/po/{id}", handlers::purchase_orders::detail, authorize(GandolaAccess, ["hr"]);
        get PurchaseOrderEditGetRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_get, modal, authorize(GandolaAccess, ["hr"]);
        post PurchaseOrderEditPostRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_post, authorize(GandolaAccess, ["hr"]);
        get PurchaseOrderDeleteGetRouteTag, "/gandola/purchase-orders/po/{id}/delete", handlers::purchase_orders::delete_get, modal, authorize(GandolaAccess, ["hr"]);
        post PurchaseOrderDeletePostRouteTag, "/gandola/purchase-orders/po/{id}/delete", bare handlers::purchase_orders::delete_post, fragment(PurchaseOrderDeleteModalKey), authorize(GandolaAccess, ["hr"]);
        get PurchaseOrderSelectRouteTag, "/gandola/purchase-orders/pick", handlers::purchase_orders::select, multi_select(PurchaseOrderSelectTableKey, PurchaseOrderSelectModalKey), authorize(GandolaAccess, ["hr"]);
    ]
}
