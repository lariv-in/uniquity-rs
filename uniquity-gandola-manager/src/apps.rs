lariv_core::define_register_apps! {
    plugin: GandolaManagerTag;
    key: "gandola_manager";
    name: "Gandola Manager";
    href: crate::routes::SiteDefaultRouteTag.url();
    icon: "building-office-2";
    roles: [lariv_plugin_users::roles::Superuser, crate::routes::Hr];
}
