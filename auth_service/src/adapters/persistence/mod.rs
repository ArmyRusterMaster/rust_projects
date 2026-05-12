#[cfg(feature = "memory")]
pub mod memory;

pub mod unsupported;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "surrealdb")]
pub mod surrealdb;

#[cfg(feature = "memory")]
pub use memory::InMemoryAuthRepository;

#[cfg(feature = "sqlite")]
pub use sqlite::SqliteAuthRepository;
