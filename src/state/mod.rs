pub mod paths;
pub mod toggle;

pub use paths::{get_log_dir, get_state_dir};
pub use toggle::{cleanup_processing, is_recording};
