mod paths;
pub mod toggle;

pub use paths::{get_log_dir, get_state_dir};
pub use toggle::{is_recording, start_recording, stop_recording, RecordingState};
