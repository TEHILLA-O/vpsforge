//! Shared types used by every VPSForge crate.

pub mod error;
pub mod format;
pub mod host;
pub mod paths;
pub mod plan;
pub mod trust;

pub use error::{ForgeError, ForgeResult};
pub use host::*;
pub use paths::ForgePaths;
pub use plan::*;
pub use trust::*;

pub const APP_NAME: &str = "vpsforge";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
