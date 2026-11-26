//! User preferences that the Rust side needs to act on (window, tray, exit).

use serde::{Deserialize, Serialize};

pub const KEY: &str = "settings";

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Closing the window hides it to the tray instead of quitting.
    pub hide_on_close: bool,
    /// Launch Fontin when the user logs in.
    pub autostart: bool,
    /// Start with the window hidden (tray only).
    pub start_hidden: bool,
    /// Remove every activation from the OS when Fontin quits; the saved
    /// state is kept and re-applied on the next launch.
    pub deactivate_on_exit: bool,
}

impl Settings {
    pub fn parse(raw: Option<String>) -> Self {
        raw.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
}
