//! `SidebarState` — form state model for the connection sidebar.
//!
//! Holds transport selection, input state entities for fields, and dynamic
//! environment-variable rows. The caller (StudioApp) creates entity handles
//! via its own context and stores them here.

use crate::app::events::{ConnConfig, Transport};
use gpui::{App, Entity};
use gpui_component::input::InputState;

// ---------------------------------------------------------------------------
// EnvRow
// ---------------------------------------------------------------------------

pub struct EnvRow {
    pub key: Entity<InputState>,
    pub value: Entity<InputState>,
}

// ---------------------------------------------------------------------------
// SidebarState
// ---------------------------------------------------------------------------

pub struct SidebarState {
    pub transport: Transport,
    pub command_input: Entity<InputState>,
    pub args_input: Entity<InputState>,
    pub url_input: Entity<InputState>,
    pub env_rows: Vec<EnvRow>,
}

impl SidebarState {
    pub fn remove_env_row(&mut self, idx: usize) {
        if idx < self.env_rows.len() {
            self.env_rows.remove(idx);
        }
    }

    pub fn current_config(&self, cx: &App) -> ConnConfig {
        let read_input =
            |entity: &Entity<InputState>| entity.read(cx).text().chars().collect::<String>();

        let command = read_input(&self.command_input);
        let args: Vec<String> = read_input(&self.args_input)
            .split_whitespace()
            .map(String::from)
            .collect();
        let url = read_input(&self.url_input);
        let env: Vec<(String, String)> = self
            .env_rows
            .iter()
            .map(|row| {
                let key: String = row.key.read(cx).text().chars().collect();
                let value: String = row.value.read(cx).text().chars().collect();
                (key, value)
            })
            .filter(|(k, _)| !k.is_empty())
            .collect();

        ConnConfig {
            transport: self.transport,
            command,
            args,
            url,
            env,
        }
    }
}
