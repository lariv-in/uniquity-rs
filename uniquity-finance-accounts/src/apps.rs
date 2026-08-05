pub const ACCOUNTING_APP_KEY: &str = "p_uniquity_finance_accounts";

lariv_rs::define_register_apps! {
    plugin: UniquityFinanceAccountsTag;
    key: ACCOUNTING_APP_KEY;
    name: "Accounting";
    href: "/finance/";
    icon: "building-library";
    roles: ["superuser"];
}
