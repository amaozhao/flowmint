pub mod asset;
pub mod error;
pub mod exporters;
pub mod fs_safety;
pub mod import;
pub mod project;
pub mod store;
pub mod sync;
pub mod validation;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
