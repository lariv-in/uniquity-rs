use lariv_core::web::CreateModal;

use super::keys::{GANDOLA_CREATE_MODAL, PURCHASE_ORDER_CREATE_MODAL, SITE_CREATE_MODAL};
use super::routes::{
    GandolaCreateGetRouteTag, GandolaCreatePostRouteTag, PurchaseOrderCreateGetRouteTag,
    PurchaseOrderCreatePostRouteTag, SiteCreateGetRouteTag, SiteCreatePostRouteTag,
};

pub const GANDOLA_CREATE: CreateModal<GandolaCreateGetRouteTag, GandolaCreatePostRouteTag> =
    CreateModal {
        key: GANDOLA_CREATE_MODAL,
        form_name: "gandola_manager.GandolaCreateForm",
        get: GandolaCreateGetRouteTag,
        post: GandolaCreatePostRouteTag,
    };

pub const SITE_CREATE: CreateModal<SiteCreateGetRouteTag, SiteCreatePostRouteTag> = CreateModal {
    key: SITE_CREATE_MODAL,
    form_name: "gandola_manager.SiteCreateForm",
    get: SiteCreateGetRouteTag,
    post: SiteCreatePostRouteTag,
};

pub const PURCHASE_ORDER_CREATE: CreateModal<
    PurchaseOrderCreateGetRouteTag,
    PurchaseOrderCreatePostRouteTag,
> = CreateModal {
    key: PURCHASE_ORDER_CREATE_MODAL,
    form_name: "gandola_manager.PurchaseOrderCreateForm",
    get: PurchaseOrderCreateGetRouteTag,
    post: PurchaseOrderCreatePostRouteTag,
};
