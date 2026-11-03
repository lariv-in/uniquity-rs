use lariv_plugin_customer::keys::CUSTOMER_TABLE;

use super::{
    handlers,
    keys::{
        GANDOLA_DELETE_MODAL, GANDOLA_SELECT_MODAL, GANDOLA_SELECT_TABLE, GANDOLA_SITES_TABLE,
        GANDOLA_TABLE, PURCHASE_ORDER_DELETE_MODAL, PURCHASE_ORDER_SELECT_MODAL,
        PURCHASE_ORDER_SELECT_TABLE, PURCHASE_ORDER_TABLE, SITE_DELETE_MODAL,
        SITE_FK_SELECT_MODAL, SITE_FK_SELECT_TABLE, SITE_GANDOLAS_TABLE,
        SITE_INVOICE_SELECT_MODAL, SITE_INVOICE_SELECT_TABLE, SITE_SELECT_MODAL,
        SITE_SELECT_TABLE, SITE_TABLE,
    },
};

/// Gandola Manager routes. Allowlist is [`Accountant`]; superuser always passes.
pub struct GandolaAccess;

/// Stored role `accountant`. Superuser always passes allowlists that name this role.
#[derive(Clone, Copy, Debug, Default)]
pub struct Accountant;

impl lariv_plugin_users::role_registry::Role for Accountant {
    const NAME: &'static str = "accountant";
    const TITLE: &'static str = "Accountant";
    const DESCRIPTION: &'static str =
        "Accounting, Gandola Manager, contacts, CRM, the filesystem, and the assistant.";
}

lariv_core::define_plugin_routes! {
    plugin: GandolaManagerTag;
    routes: [
        get GandolaDefaultRouteTag, "/gandola", handlers::gandolas::list, fragment(GANDOLA_TABLE), authorize(GandolaAccess, [Accountant]);
        get GandolaCreateGetRouteTag, "/gandola/create", handlers::gandolas::create_get, modal, authorize(GandolaAccess, [Accountant]);
        post GandolaCreatePostRouteTag, "/gandola/create", handlers::gandolas::create_post, authorize(GandolaAccess, [Accountant]);
        get GandolaDetailRouteTag, "/gandola/g/{id}", handlers::gandolas::detail, fragment(GANDOLA_SITES_TABLE), authorize(GandolaAccess, [Accountant]);
        get GandolaEditGetRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_get, modal, authorize(GandolaAccess, [Accountant]);
        post GandolaEditPostRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_post, authorize(GandolaAccess, [Accountant]);
        get GandolaDeleteGetRouteTag, "/gandola/g/{id}/delete", handlers::gandolas::delete_get, modal, authorize(GandolaAccess, [Accountant]);
        post GandolaDeletePostRouteTag, "/gandola/g/{id}/delete", bare handlers::gandolas::delete_post, fragment(GANDOLA_DELETE_MODAL), authorize(GandolaAccess, [Accountant]);
        get GandolaSelectRouteTag, "/gandola/pick", handlers::gandolas::select, multi_select(GANDOLA_SELECT_TABLE, GANDOLA_SELECT_MODAL), authorize(GandolaAccess, [Accountant]);

        get SiteDefaultRouteTag, "/gandola/sites", handlers::sites::list, fragment(SITE_TABLE), authorize(GandolaAccess, [Accountant]);
        get SiteCreateGetRouteTag, "/gandola/sites/create", handlers::sites::create_get, modal, authorize(GandolaAccess, [Accountant]);
        post SiteCreatePostRouteTag, "/gandola/sites/create", handlers::sites::create_post, authorize(GandolaAccess, [Accountant]);
        get SiteDetailRouteTag, "/gandola/sites/s/{id}", handlers::sites::detail, fragment(SITE_GANDOLAS_TABLE), authorize(GandolaAccess, [Accountant]);
        get SiteEditGetRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_get, modal, authorize(GandolaAccess, [Accountant]);
        post SiteEditPostRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_post, authorize(GandolaAccess, [Accountant]);
        get SiteDeleteGetRouteTag, "/gandola/sites/s/{id}/delete", handlers::sites::delete_get, modal, authorize(GandolaAccess, [Accountant]);
        post SiteDeletePostRouteTag, "/gandola/sites/s/{id}/delete", bare handlers::sites::delete_post, fragment(SITE_DELETE_MODAL), authorize(GandolaAccess, [Accountant]);
        get SiteSelectRouteTag, "/gandola/sites/pick", handlers::sites::select, multi_select(SITE_SELECT_TABLE, SITE_SELECT_MODAL), authorize(GandolaAccess, [Accountant]);
        get SiteFkSelectRouteTag, "/gandola/sites/pick-site", handlers::sites::fk_select, fk_select(SITE_FK_SELECT_TABLE, SITE_FK_SELECT_MODAL), authorize(GandolaAccess, [Accountant]);
        get SiteInvoiceSelectRouteTag, "/gandola/sites/invoices/pick", handlers::sites::select_invoices, multi_select(SITE_INVOICE_SELECT_TABLE, SITE_INVOICE_SELECT_MODAL), authorize(GandolaAccess, [Accountant]);

        get GandolaPreferencesRouteTag, "/gandola/preferences", handlers::preferences::get, authorize(GandolaAccess, [Accountant]);
        post GandolaPreferencesPostRouteTag, "/gandola/preferences", handlers::preferences::post, authorize(GandolaAccess, [Accountant]);

        get PurchaseOrderDefaultRouteTag, "/gandola/purchase-orders", handlers::purchase_orders::list, fragment(PURCHASE_ORDER_TABLE), authorize(GandolaAccess, [Accountant]);
        get PurchaseOrderCreateGetRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_get, modal, authorize(GandolaAccess, [Accountant]);
        post PurchaseOrderCreatePostRouteTag, "/gandola/purchase-orders/create", handlers::purchase_orders::create_post, authorize(GandolaAccess, [Accountant]);
        get PurchaseOrderDetailRouteTag, "/gandola/purchase-orders/po/{id}", handlers::purchase_orders::detail, authorize(GandolaAccess, [Accountant]);
        get PurchaseOrderEditGetRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_get, modal, authorize(GandolaAccess, [Accountant]);
        post PurchaseOrderEditPostRouteTag, "/gandola/purchase-orders/po/{id}/edit", handlers::purchase_orders::edit_post, authorize(GandolaAccess, [Accountant]);
        get PurchaseOrderDeleteGetRouteTag, "/gandola/purchase-orders/po/{id}/delete", handlers::purchase_orders::delete_get, modal, authorize(GandolaAccess, [Accountant]);
        post PurchaseOrderDeletePostRouteTag, "/gandola/purchase-orders/po/{id}/delete", bare handlers::purchase_orders::delete_post, fragment(PURCHASE_ORDER_DELETE_MODAL), authorize(GandolaAccess, [Accountant]);
        get PurchaseOrderSelectRouteTag, "/gandola/purchase-orders/pick", handlers::purchase_orders::select, multi_select(PURCHASE_ORDER_SELECT_TABLE, PURCHASE_ORDER_SELECT_MODAL), authorize(GandolaAccess, [Accountant]);

        get CustomerListRouteTag, "/gandola/customers", handlers::customers::list, fragment(CUSTOMER_TABLE), authorize(GandolaAccess, [Accountant]);
    ]
}
