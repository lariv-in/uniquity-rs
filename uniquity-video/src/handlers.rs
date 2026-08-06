pub mod edited;
pub mod hub;
pub mod published;
pub mod raw;

/// Modal opener query (`?name=…&refresh=table-id`). Case-sensitive vs filter `Name`.
pub use lariv_rs::web::ModalFormQuery as ModalNameQuery;
