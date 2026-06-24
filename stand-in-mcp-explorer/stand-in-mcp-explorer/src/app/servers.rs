//! Server persistence — `ServerEntry` type + load/save via `ProjectDirs`,
//! with deduplication by `ConnConfig` (program+args).
//!
//! ## Dedup key
//!
//! `add_dedup` considers two entries the same connection when their
//! `config` fields are equal (same program and args). A duplicate updates
//! the name; it never creates a second entry (BUG-9 class).

use std::path::Path;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::app::events::ConnConfig;
use crate::bars::sidebar::auth_state::AuthConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub config: ConnConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
}

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "mcp-explorer")
}

pub fn config_dir() -> Option<std::path::PathBuf> {
    project_dirs().map(|p| p.config_dir().to_path_buf())
}

fn config_path() -> Option<std::path::PathBuf> {
    config_dir().map(|d| d.join("servers.json"))
}

pub fn load() -> Vec<ServerEntry> {
    match config_path() {
        Some(p) => load_from(&p),
        None => Vec::new(),
    }
}

pub fn save(entries: &[ServerEntry]) -> std::io::Result<()> {
    let path = config_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory"))?;
    save_to(entries, &path)
}

pub fn load_from(path: &Path) -> Vec<ServerEntry> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    // Lenient per-entry parse: a single bad/legacy entry (e.g. a removed
    // transport like "Sse") must not discard the whole file.
    let raw: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();
    raw.into_iter()
        .filter_map(|v| serde_json::from_value::<ServerEntry>(v).ok())
        .collect()
}

pub fn save_to(entries: &[ServerEntry], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(entries)?;
    std::fs::write(path, json)
}

pub fn add_dedup(entries: &mut Vec<ServerEntry>, new: ServerEntry) {
    for entry in entries.iter_mut() {
        if entry.config == new.config {
            entry.name = new.name;
            return;
        }
    }
    entries.push(new);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::events::ConnConfig;

    fn test_config(program: &str) -> ConnConfig {
        ConnConfig {
            transport: crate::app::events::Transport::Stdio,
            command: program.to_string(),
            args: vec![],
            url: String::new(),
            env: Vec::new(),
        }
    }

    fn test_entry(name: &str, program: &str) -> ServerEntry {
        ServerEntry {
            name: name.to_string(),
            config: test_config(program),
            auth: None,
        }
    }

    #[test]
    fn load_from_missing_path_returns_empty() {
        let entries = load_from(Path::new("/nonexistent/path/definitely/not/there.json"));
        assert!(entries.is_empty());
    }

    #[test]
    fn load_from_corrupt_file_returns_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("servers.json");
        std::fs::write(&path, b"not valid json {{{").expect("write");
        let entries = load_from(&path);
        assert!(entries.is_empty());
    }

    #[test]
    fn round_trip_save_and_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("servers.json");
        let original = vec![
            test_entry("My Server", "my-server"),
            test_entry("Another", "another-server"),
        ];
        save_to(&original, &path).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded, original);
    }

    #[test]
    fn round_trip_preserves_args() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("servers.json");
        let entry = ServerEntry {
            name: "with args".to_string(),
            config: ConnConfig {
                transport: crate::app::events::Transport::Stdio,
                command: "node".to_string(),
                args: vec![
                    "server.js".to_string(),
                    "--port".to_string(),
                    "3000".to_string(),
                ],
                url: String::new(),
                env: Vec::new(),
            },
            auth: None,
        };
        save_to(std::slice::from_ref(&entry), &path).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded, vec![entry]);
    }

    #[test]
    fn save_to_creates_parent_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("sub").join("deep").join("servers.json");
        let entries = vec![test_entry("deep", "deep-server")];
        save_to(&entries, &nested).expect("save");
        assert!(nested.exists());
        let loaded = load_from(&nested);
        assert_eq!(loaded, entries);
    }

    #[test]
    fn add_dedup_same_config_updates_name_not_duplicates() {
        let mut entries = vec![test_entry("Original", "my-server")];
        add_dedup(
            &mut entries,
            ServerEntry {
                name: "Renamed".to_string(),
                config: test_config("my-server"),
                auth: None,
            },
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Renamed");
    }

    #[test]
    fn add_dedup_different_config_appends() {
        let mut entries = vec![test_entry("First", "server-a")];
        add_dedup(&mut entries, test_entry("Second", "server-b"));
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "First");
        assert_eq!(entries[1].name, "Second");
    }

    #[test]
    fn add_dedup_matches_on_program_and_args() {
        let mut entries = vec![ServerEntry {
            name: "Old".to_string(),
            config: ConnConfig {
                transport: crate::app::events::Transport::Stdio,
                command: "node".to_string(),
                args: vec!["a.js".to_string()],
                url: String::new(),
                env: Vec::new(),
            },
            auth: None,
        }];
        add_dedup(
            &mut entries,
            ServerEntry {
                name: "New".to_string(),
                config: ConnConfig {
                    transport: crate::app::events::Transport::Stdio,
                    command: "node".to_string(),
                    args: vec!["a.js".to_string()],
                    url: String::new(),
                    env: Vec::new(),
                },
                auth: None,
            },
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "New");
    }

    #[test]
    fn load_from_drops_legacy_sse_entry_keeps_rest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("servers.json");
        // Build a legacy file with 3 entries: Stdio (valid), Sse (legacy — no
        // longer deserializes), Http (valid). Use a raw JSON string because
        // `Transport::Sse` no longer exists in the enum.
        let json = serde_json::json!([
            {
                "name": "stdio-server",
                "config": {
                    "transport": "Stdio",
                    "command": "node",
                    "args": ["server.js"],
                    "url": "",
                    "env": []
                }
            },
            {
                "name": "old-sse",
                "config": {
                    "transport": "Sse",
                    "command": "",
                    "args": [],
                    "url": "http://x/sse",
                    "env": []
                }
            },
            {
                "name": "http-server",
                "config": {
                    "transport": "Http",
                    "command": "",
                    "args": [],
                    "url": "https://example.com/mcp",
                    "env": []
                }
            }
        ]);
        std::fs::write(&path, json.to_string()).expect("write");
        let loaded = load_from(&path);
        assert_eq!(loaded.len(), 2, "legacy Sse entry should be dropped");
        assert_eq!(loaded[0].name, "stdio-server");
        assert_eq!(loaded[1].name, "http-server");
    }

    #[test]
    fn load_from_drops_single_corrupt_entry_keeps_valid() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("servers.json");
        let json = serde_json::json!([
            {
                "name": "valid-server",
                "config": {
                    "transport": "Stdio",
                    "command": "cargo",
                    "args": ["run"],
                    "url": "",
                    "env": []
                }
            },
            {"garbage": "not a valid server entry"}
        ]);
        std::fs::write(&path, json.to_string()).expect("write");
        let loaded = load_from(&path);
        assert_eq!(loaded.len(), 1, "corrupt entry should be dropped");
        assert_eq!(loaded[0].name, "valid-server");
    }

    #[test]
    fn add_dedup_different_args_are_different_configs() {
        let mut entries = vec![ServerEntry {
            name: "Old".to_string(),
            config: ConnConfig {
                transport: crate::app::events::Transport::Stdio,
                command: "node".to_string(),
                args: vec!["a.js".to_string()],
                url: String::new(),
                env: Vec::new(),
            },
            auth: None,
        }];
        add_dedup(
            &mut entries,
            ServerEntry {
                name: "New".to_string(),
                config: ConnConfig {
                    transport: crate::app::events::Transport::Stdio,
                    command: "node".to_string(),
                    args: vec!["b.js".to_string()],
                    url: String::new(),
                    env: Vec::new(),
                },
                auth: None,
            },
        );
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn load_legacy_entry_without_auth_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("servers.json");
        // Legacy format: no "auth" field
        let json = serde_json::json!([{
            "name": "legacy-server",
            "config": {
                "transport": "Stdio",
                "command": "node",
                "args": ["server.js"],
                "url": "",
                "env": []
            }
        }]);
        std::fs::write(&path, json.to_string()).expect("write");
        let loaded = load_from(&path);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "legacy-server");
        assert!(
            loaded[0].auth.is_none(),
            "legacy entry without auth should be None"
        );
    }

    #[test]
    fn round_trip_preserves_auth_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("servers.json");
        let entry = ServerEntry {
            name: "with-auth".to_string(),
            config: ConnConfig {
                transport: crate::app::events::Transport::Http,
                command: String::new(),
                args: vec![],
                url: "https://example.com/mcp".to_string(),
                env: vec![],
            },
            auth: Some(AuthConfig {
                method: crate::bars::sidebar::auth_state::AuthMethod::Bearer,
                basic_username: String::new(),
                oauth_client_id: String::new(),
                oauth_auth_url: String::new(),
                oauth_token_url: String::new(),
                oauth_scopes: String::new(),
            }),
        };
        save_to(std::slice::from_ref(&entry), &path).expect("save");
        let loaded = load_from(&path);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "with-auth");
        assert!(loaded[0].auth.is_some());
    }
}
