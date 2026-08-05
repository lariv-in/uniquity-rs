//! Sidebar links and accounting preferences patched onto the shared Accounting app.

use uniquity_finance_accounts::accounting_sidebar::{self, AccountingSidebarRegistrar};

use crate::routes::{
    InvoiceDefaultRouteTag, PaymentBatchListRouteTag, PaymentListRouteTag, PaymentTermListRouteTag,
};

#[derive(Clone, Copy, Default)]
pub struct Hook;

impl AccountingSidebarRegistrar for Hook {
    fn register_accounting_sidebar(
        self,
        cap: accounting_sidebar::AccountingSidebarRegistry,
    ) -> accounting_sidebar::AccountingSidebarRegistry {
        cap.push(accounting_sidebar::link::<InvoiceDefaultRouteTag>(
            "invoices",
            "Invoices",
            100,
            Some("document-text"),
        ))
        .push(accounting_sidebar::link::<PaymentTermListRouteTag>(
            "payment-terms",
            "Payment terms",
            110,
            Some("clock"),
        ))
        .push(accounting_sidebar::link::<PaymentListRouteTag>(
            "payments",
            "Payments",
            120,
            Some("banknotes"),
        ))
        .push(accounting_sidebar::link::<PaymentBatchListRouteTag>(
            "payment-batches",
            "Batches",
            125,
            Some("rectangle-stack"),
        ))
    }

    fn register_accounting_preferences(
        self,
        cap: uniquity_finance_accounts::accounting_preferences_patch::AccountingPreferencesRegistry,
    ) -> uniquity_finance_accounts::accounting_preferences_patch::AccountingPreferencesRegistry {
        cap.register_addon(&crate::accounting_preferences_patch::INVOICES_ADDON)
    }
}
