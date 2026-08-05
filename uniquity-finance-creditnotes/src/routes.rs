use super::{handlers, keys::CreditNoteTableKey};

lariv_rs::define_plugin_routes! {
    plugin: UniquityFinanceCreditnotesTag;
    routes: [
        get CreditNoteDefaultRouteTag, "/finance-credit-notes", handlers::credit_notes::list, fragment(CreditNoteTableKey);
        get CreditNoteDetailRouteTag, "/finance-credit-notes/c/{id}", handlers::credit_notes::detail;
    ]
}
