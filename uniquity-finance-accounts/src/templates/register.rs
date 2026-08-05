use lariv_rs::{
    components::{SlotCapability, SlotRegistrar},
    http::ProvideRequestCaps,
    template::{TemplateCapability, TemplateOf, TemplateRegistrar},
};


use super::{
    accounts::{
        AccountDetailPage, AccountFormPage, AccountListPage,
        AccountSelectPage,
    },
    currencies::{
        CurrencyDetailPage, CurrencyFormPage, CurrencyListPage,
        CurrencySelectPage,
    },
    journals::{
        JournalDetailPage, JournalEntryDetailPage, JournalEntryFormPage,
        JournalEntrySelectPage, JournalFormPage, JournalListPage, JournalSelectPage,
    },
    preferences::AccountingPreferencesPage,
    source_docs::SourceDocSelectPage,
};

lariv_rs::define_register_items! {
    plugin: UniquityFinanceAccountsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        AccountListIdx: AccountListPageTag => AccountListPage,
        AccountDetailIdx: AccountDetailPageTag => AccountDetailPage,
        AccountFormIdx: AccountFormPageTag => AccountFormPage,
        AccountSelectIdx: AccountSelectPageTag => AccountSelectPage,
        CurrencyListIdx: CurrencyListPageTag => CurrencyListPage,
        CurrencyDetailIdx: CurrencyDetailPageTag => CurrencyDetailPage,
        CurrencyFormIdx: CurrencyFormPageTag => CurrencyFormPage,
        CurrencySelectIdx: CurrencySelectPageTag => CurrencySelectPage,
        JournalListIdx: JournalListPageTag => JournalListPage,
        JournalDetailIdx: JournalDetailPageTag => JournalDetailPage,
        JournalFormIdx: JournalFormPageTag => JournalFormPage,
        JournalSelectIdx: JournalSelectPageTag => JournalSelectPage,
        JournalEntryFormIdx: JournalEntryFormPageTag => JournalEntryFormPage,
        JournalEntryDetailIdx: JournalEntryDetailPageTag => JournalEntryDetailPage,
        JournalEntrySelectIdx: JournalEntrySelectPageTag => JournalEntrySelectPage,
        SourceDocSelectIdx: SourceDocSelectPageTag => SourceDocSelectPage,
        AccountingPreferencesIdx: AccountingPreferencesPageTag => AccountingPreferencesPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceAccountsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}
