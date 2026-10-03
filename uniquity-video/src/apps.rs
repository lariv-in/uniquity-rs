lariv_rs::define_register_apps! {
    plugin: UniquityVideoTag;
    key: "p_uniquity_video";
    name: "Video editors";
    href: crate::routes::VideoHubRouteTag.url();
    icon: "film";
    roles: ["superuser", "student", "admin"];
}
