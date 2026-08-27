use super::{
    handlers,
    keys::{
        GandolaDeleteModalKey, GandolaSelectModalKey, GandolaSelectTableKey, GandolaTableKey,
        PurchaseOrderDeleteModalKey, PurchaseOrderSelectModalKey, PurchaseOrderSelectTableKey,
        PurchaseOrderTableKey, SiteDeleteModalKey, SiteFkSelectModalKey, SiteFkSelectTableKey,
        SiteSelectModalKey, SiteSelectTableKey, SiteTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: GandolaManagerTag;
    routes: [
        get GandolaDefaultRouteTag, "/gandola", handlers::gandolas::list, fragment(GandolaTableKey);
        get GandolaCreateGetRouteTag, "/gandola/create", handlers::gandolas::create_get, modal;
        post GandolaCreatePostRouteTag, "/gandola/create", handlers::gandolas::create_post;
        get GandolaDetailRouteTag, "/gandola/g/{id}", handlers::gandolas::detail;
        get GandolaEditGetRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_get, modal;
        post GandolaEditPostRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_post;
        get GandolaDeleteGetRouteTag, "/gandola/g/{id}/delete", handlers::gandolas::delete_get, modal;
        post GandolaDeletePostRouteTag, "/gandola/g/{id}/delete", bare handlers::gandolas::delete_post, fragment(GandolaDeleteModalKey);
        get GandolaSelectRouteTag, "/gandola/pick", handlers::gandolas::select, multi_select(GandolaSelectTableKey, GandolaSelectModalKey);

        get SiteDefaultRouteTag, "/gandola/sites", handlers::sites::list, fragment(SiteTableKey);
        get SiteCreateGetRouteTag, "/gandola/sites/create", handlers::sites::create_get, modal;
        post SiteCreatePostRouteTag, "/gandola/sites/create", handlers::sites::create_post;
        get SiteDetailRouteTag, "/gandola/sites/s/{id}", handlers::sites::detail;
        get SiteEditGetRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_get, modal;
        post SiteEditPostRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_post;
        get SiteDeleteGetRouteTag, "/gandola/sites/s/{id}/delete", handlers::sites::delete_get, modal;
        post SiteDeletePostRouteTag, "/gandola/sites/s/{id}/delete", bare handlers::sites::delete_post, fragment(SiteDeleteModalKey);
        get SiteSelectRouteTag, "/gandola/sites/pick", handlers::sites::select, multi_select(SiteSelectTableKey, SiteSelectModalKey);
        get SiteFkSelectRouteTag, "/gandola/sites/pick-site", handlers::sites::fk_select, fk_select(SiteFkSelectTableKey, SiteFkSelectModalKey);

        get GandolaPreferencesRouteTag, "/gandola/preferences", handlers::preferences::get;
        post GandolaPreferencesPostRouteTag, "/gandola/preferences", handlers::preferences::post;

        get PurchaseOrderDefaultRouteTag, "/gandola/purchase-orders", handlers::purchase_orders::list, fragment(PurchaseOrderTableKey);
        get PurchaseOrderCreateGetRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_get, modal;
        post PurchaseOrderCreatePostRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_post;
        get PurchaseOrderFromPdfGetRouteTag, "/gandola/sites/s/{id}/purchase-orders/create-from-pdf", handlers::purchase_orders::from_pdf_get, modal;
        post PurchaseOrderFromPdfPostRouteTag, "/gandola/sites/s/{id}/purchase-orders/create-from-pdf", handlers::purchase_orders::from_pdf_post;
        get PurchaseOrderImportJobsRouteTag, "/gandola/sites/s/{id}/purchase-orders/import-jobs", bare handlers::purchase_orders::import_jobs_get, raw;
        post PurchaseOrderImportJobsDismissRouteTag, "/gandola/sites/s/{id}/purchase-orders/import-jobs/dismiss", bare handlers::purchase_orders::import_jobs_dismiss, raw;
        get PurchaseOrderDetailRouteTag, "/gandola/purchase-orders/po/{id}", handlers::purchase_orders::detail;
        get PurchaseOrderEditGetRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_get, modal;
        post PurchaseOrderEditPostRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_post;
        get PurchaseOrderDeleteGetRouteTag, "/gandola/purchase-orders/po/{id}/delete", handlers::purchase_orders::delete_get, modal;
        post PurchaseOrderDeletePostRouteTag, "/gandola/purchase-orders/po/{id}/delete", bare handlers::purchase_orders::delete_post, fragment(PurchaseOrderDeleteModalKey);
        get PurchaseOrderSelectRouteTag, "/gandola/purchase-orders/pick", handlers::purchase_orders::select, multi_select(PurchaseOrderSelectTableKey, PurchaseOrderSelectModalKey);
    ]
}
