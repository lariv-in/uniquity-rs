use super::{
    handlers,
    keys::{EmployeeSelectTableKey, EmployeeTableKey, PointsTableKey},
};

lariv_rs::define_plugin_routes! {
    plugin: UniquityEmployeesTag;
    routes: [
        get EmployeesDefaultRouteTag, "/employees/", handlers::employees::list, fragment(EmployeeTableKey);
        get EmployeesCreateGetRouteTag, "/employees/create/", handlers::employees::create_get;
        post EmployeesCreatePostRouteTag, "/employees/create/", handlers::employees::create_post;
        get EmployeesSelectRouteTag, "/employees/select/", handlers::employees::select, fragment(EmployeeSelectTableKey);
        get EmployeesDetailRouteTag, "/employees/emp/{id}/", handlers::employees::detail;
        get EmployeesEditGetRouteTag, "/employees/emp/{id}/edit/", handlers::employees::edit_get;
        post EmployeesEditPostRouteTag, "/employees/emp/{id}/edit/", handlers::employees::edit_post;
        post EmployeesDeletePostRouteTag, "/employees/emp/{id}/delete/", bare handlers::employees::delete_post, redirect;
        get PointsListRouteTag, "/employees/points/", handlers::points::list, fragment(PointsTableKey);
        get PointsCreateGetRouteTag, "/employees/points/create/", handlers::points::create_get;
        post PointsCreatePostRouteTag, "/employees/points/create/", handlers::points::create_post;
        get PointsDetailRouteTag, "/employees/points/{id}/", handlers::points::detail;
    ]
}
