use super::{
    handlers,
    keys::{CustomerSelectModalKey, CustomerSelectTableKey, CustomerTableKey},
};

lariv_rs::define_plugin_routes! {
    plugin: UniquityFinanceCustomerTag;
    routes: [
        get CustomerDefaultRouteTag, "/finance-customers", handlers::customers::list, fragment(CustomerTableKey);
        get CustomerCreateGetRouteTag, "/finance-customers/create", handlers::customers::create_get;
        post CustomerCreatePostRouteTag, "/finance-customers/create", handlers::customers::create_post;
        get CustomerDetailRouteTag, "/finance-customers/c/{id}", handlers::customers::detail;
        get CustomerEditGetRouteTag, "/finance-customers/c/{id}/edit", handlers::customers::edit_get;
        post CustomerEditPostRouteTag, "/finance-customers/c/{id}/edit", handlers::customers::edit_post;
        post CustomerDeletePostRouteTag, "/finance-customers/c/{id}/delete", bare handlers::customers::delete_post, redirect;
        get CustomerFkSelectRouteTag, "/finance-customers/pick-customer", handlers::customers::select, fk_select(CustomerSelectTableKey, CustomerSelectModalKey);
    ]
}
