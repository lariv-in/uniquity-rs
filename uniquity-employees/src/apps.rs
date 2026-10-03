lariv_rs::define_register_apps! {
    plugin: UniquityEmployeesTag;
    key: "p_uniquity_employees";
    name: "Employees & points";
    href: crate::routes::EmployeesDefaultRouteTag.url();
    icon: "users";
    roles: ["superuser"];
}
