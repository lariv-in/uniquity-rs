pub mod employees;
pub mod points;

/// Modal opener query (`?name=…&refresh=table-id`). Case-sensitive vs filter `Name`.
pub use lariv_rs::web::ModalFormQuery as ModalNameQuery;
