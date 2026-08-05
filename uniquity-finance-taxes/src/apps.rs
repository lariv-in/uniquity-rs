lariv_rs::define_register_apps! {
    plugin: UniquityFinanceTaxesTag;
    key: "p_uniquity_finance_taxes";
    name: "Finance taxes";
    href: "/finance-taxes/";
    icon: "receipt-percent";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["superuser"];
}
