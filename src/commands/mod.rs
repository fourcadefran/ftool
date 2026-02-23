pub mod file;
pub mod duckdb_inspector;

pub use file::File;
pub use duckdb_inspector::DuckDbInspector;
pub mod json_inspector;
pub use json_inspector::JsonInspector;
