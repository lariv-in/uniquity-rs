lariv_rs::define_register_apps! {
    plugin: UniquityFinanceFiscalYearTag;
    key: "p_uniquity_finance_fiscal_year";
    name: "Finance fiscal years";
    href: "/finance-fiscal-years/";
    icon: "calendar";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["superuser"];
}
