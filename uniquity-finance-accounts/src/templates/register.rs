use lariv_rs::{
    components::{SlotCapability, SlotRegistrar},
    http::ProvideRequestCaps,
    template::{TemplateCapability, TemplateOf, TemplateRegistrar},
};


use super::{
    accounts::{
        AccountCreateModalPage, AccountDetailPage, AccountFormPage, AccountJournalEntriesPage,
        AccountJournalEntryItemsPage, AccountListPage, AccountSelectPage,
    },
    currencies::{
        CurrencyCreateModalPage, CurrencyDetailPage, CurrencyFormPage, CurrencyListPage,
        CurrencySelectPage,
    },
    journals::{
        JournalCreateModalPage, JournalDetailPage, JournalEntryCreateModalPage,
        JournalEntryDeletePage, JournalEntryDetailPage, JournalEntrySelectPage, JournalFormPage,
        JournalListPage, JournalSelectPage,
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
        AccountJournalEntriesIdx: AccountJournalEntriesPageTag => AccountJournalEntriesPage,
        AccountJournalEntryItemsIdx: AccountJournalEntryItemsPageTag => AccountJournalEntryItemsPage,
        AccountFormIdx: AccountFormPageTag => AccountFormPage,
        AccountCreateModalIdx: AccountCreateModalPageTag => AccountCreateModalPage,
        AccountSelectIdx: AccountSelectPageTag => AccountSelectPage,
        CurrencyListIdx: CurrencyListPageTag => CurrencyListPage,
        CurrencyDetailIdx: CurrencyDetailPageTag => CurrencyDetailPage,
        CurrencyFormIdx: CurrencyFormPageTag => CurrencyFormPage,
        CurrencyCreateModalIdx: CurrencyCreateModalPageTag => CurrencyCreateModalPage,
        CurrencySelectIdx: CurrencySelectPageTag => CurrencySelectPage,
        JournalListIdx: JournalListPageTag => JournalListPage,
        JournalDetailIdx: JournalDetailPageTag => JournalDetailPage,
        JournalFormIdx: JournalFormPageTag => JournalFormPage,
        JournalCreateModalIdx: JournalCreateModalPageTag => JournalCreateModalPage,
        JournalSelectIdx: JournalSelectPageTag => JournalSelectPage,
        JournalEntryCreateModalIdx: JournalEntryCreateModalPageTag => JournalEntryCreateModalPage,
        JournalEntryDetailIdx: JournalEntryDetailPageTag => JournalEntryDetailPage,
        JournalEntryDeleteIdx: JournalEntryDeletePageTag => JournalEntryDeletePage,
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
