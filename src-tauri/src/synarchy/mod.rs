pub mod registry;
pub mod sync;

pub use registry::{ProjectEntry, ProjectRegistry, ProjectScanner, ProjectStatus, ProjectType};
pub use sync::{ChangeType, ProjectChangeEvent, Synchronizer};