//! Stdio client transport — launches an MCP server as a subprocess and
//! communicates via line-delimited JSON over stdin/stdout.

use std::ffi::OsString;
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, info};

use crate::error::{Error, Result};

use super::ClientTransport;

/// Client-side stdio transport that communicates with an MCP server subprocess.
///
/// Launches a server as a child process and talks JSON-RPC over its stdin/stdout,
/// mirroring the framing used by the `stand-in` server stdio transport:
/// line-delimited JSON with `\n` as the delimiter, `trim` on receive, and
/// `flush` after every write.
///
/// Stderr is drained to `tracing` (debug level) so the child's diagnostics
/// don't block on a full pipe buffer. Per MCP stdio convention, stderr is the
/// server's log channel — healthy operation should not produce `warn` spam.
///
/// # Example
///
/// ```rust,no_run
/// # use stand_in_client::transport::StdioTransport;
/// # use stand_in_client::transport::ClientTransport;
/// # #[tokio::main]
/// # async fn main() -> stand_in_client::error::Result<()> {
/// let mut t = StdioTransport::command("my-mcp-server", &[] as &[&str]);
/// t.connect().await?;
/// t.send(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}"#).await?;
/// let response = t.receive().await?;
/// t.close().await?;
/// # Ok(())
/// # }
/// ```
pub struct StdioTransport {
    // Configuration (set before connect)
    program: OsString,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
    current_dir: Option<PathBuf>,

    // Post-connect state — interior mutability so the transport is fully shareable
    // behind Arc. The stdin mutex and reader mutex are independent (no contention
    // between send and receive).
    stdin: Mutex<Option<ChildStdin>>,
    reader: Mutex<Option<BufReader<ChildStdout>>>,
    child: Mutex<Option<Child>>,
    stderr_task: Mutex<Option<JoinHandle<()>>>,
}

impl StdioTransport {
    /// Create a transport that will launch `program` with the given `args`.
    ///
    /// The subprocess is not started until [`connect`](Self::connect) is called.
    pub fn command(program: impl Into<OsString>, args: &[impl AsRef<std::ffi::OsStr>]) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(|a| a.as_ref().to_os_string()).collect(),
            env: Vec::new(),
            current_dir: None,
            stdin: Mutex::new(None),
            reader: Mutex::new(None),
            child: Mutex::new(None),
            stderr_task: Mutex::new(None),
        }
    }

    /// Add an environment variable for the subprocess.
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Add multiple environment variables for the subprocess.
    pub fn envs(
        mut self,
        iter: impl IntoIterator<Item = (impl Into<OsString>, impl Into<OsString>)>,
    ) -> Self {
        for (k, v) in iter {
            self.env.push((k.into(), v.into()));
        }
        self
    }

    /// Set the working directory for the subprocess.
    pub fn current_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ClientTransport for StdioTransport {
    async fn connect(&mut self) -> Result<()> {
        debug_assert!(
            self.stdin.get_mut().is_none(),
            "connect called twice on StdioTransport"
        );

        let mut cmd = tokio::process::Command::new(&self.program);
        cmd.args(&self.args);
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        if let Some(ref dir) = self.current_dir {
            cmd.current_dir(dir);
        }

        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            Error::ConnectionError(format!(
                "failed to spawn '{}': {e}",
                self.program.to_string_lossy()
            ))
        })?;

        let child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::ConnectionError("child stdin not available".into()))?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::ConnectionError("child stdout not available".into()))?;
        let child_stderr = child
            .stderr
            .take()
            .ok_or_else(|| Error::ConnectionError("child stderr not available".into()))?;

        // Spawn a background task to drain stderr so the child doesn't deadlock
        // on a full pipe buffer.
        let program_label = self.program.to_string_lossy().into_owned();
        let stderr_task = tokio::spawn(drain_stderr(child_stderr, program_label));

        *self.stdin.get_mut() = Some(child_stdin);
        *self.reader.get_mut() = Some(BufReader::new(child_stdout));
        *self.child.get_mut() = Some(child);
        *self.stderr_task.get_mut() = Some(stderr_task);

        info!("spawned '{}'", self.program.to_string_lossy());
        Ok(())
    }

    async fn send(&self, message: &str) -> Result<()> {
        let mut stdin_guard = self.stdin.lock().await;
        let stdin = stdin_guard
            .as_mut()
            .ok_or_else(|| Error::TransportClosed("not connected".into()))?;

        stdin.write_all(message.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await.map_err(Error::from)
    }

    async fn receive(&self) -> Result<Option<String>> {
        let mut reader_guard = self.reader.lock().await;
        let reader = match reader_guard.as_mut() {
            Some(r) => r,
            None => return Ok(None),
        };

        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        loop {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                // Skip empty lines, same as the server-side stdio transport.
                line.clear();
                let bytes_read = reader.read_line(&mut line).await?;
                if bytes_read == 0 {
                    return Ok(None);
                }
            } else {
                return Ok(Some(trimmed.to_string()));
            }
        }
    }

    async fn close(&self) -> Result<()> {
        // Flush and release stdin to signal EOF to the child.
        {
            let mut stdin_guard = self.stdin.lock().await;
            if let Some(ref mut stdin) = *stdin_guard {
                let _ = stdin.shutdown().await;
            }
            *stdin_guard = None;
        }

        // Kill the child and wait for it to avoid zombies.
        {
            let mut child_guard = self.child.lock().await;
            if let Some(ref mut child) = *child_guard {
                let _ = child.start_kill();
                let _ = child.wait().await;
            }
            *child_guard = None;
        }

        // Abort the stderr drain task and clear the reader.
        {
            let mut task_guard = self.stderr_task.lock().await;
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }
        {
            let mut reader_guard = self.reader.lock().await;
            *reader_guard = None;
        }

        Ok(())
    }
}

/// Drains stderr line by line, logging each line at debug level.
/// Prevents the child process from blocking on a full stderr pipe buffer.
async fn drain_stderr(stderr: tokio::process::ChildStderr, program: String) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("stderr drain for '{program}' closed (EOF)");
                return;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    debug!("[{program} stderr] {trimmed}");
                }
            }
            Err(e) => {
                debug!("stderr drain for '{program}' error: {e}");
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_transport_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StdioTransport>();
    }

    #[test]
    fn test_command_configures_program_and_args() {
        #[cfg(windows)]
        let t = StdioTransport::command("cmd.exe", &["/c"] as &[&str]);
        #[cfg(not(windows))]
        let t = StdioTransport::command("echo", &[] as &[&str]);

        assert_eq!(t.args.len(), if cfg!(windows) { 1 } else { 0 });
        assert!(t.env.is_empty());
        assert!(t.current_dir.is_none());
    }

    #[test]
    fn test_env_configures_variable() {
        let t = StdioTransport::command("dummy", &[] as &[&str]).env("KEY", "value");
        assert_eq!(t.env.len(), 1);
        assert_eq!(t.env[0].0, "KEY");
        assert_eq!(t.env[0].1, "value");
    }

    #[test]
    fn test_envs_configures_multiple_variables() {
        let t = StdioTransport::command("dummy", &[] as &[&str]).envs([("K1", "v1"), ("K2", "v2")]);
        assert_eq!(t.env.len(), 2);
    }

    #[test]
    fn test_current_dir_configures_path() {
        let t = StdioTransport::command("dummy", &[] as &[&str]).current_dir("/tmp");
        assert_eq!(t.current_dir.as_deref(), Some(std::path::Path::new("/tmp")));
    }

    #[test]
    fn test_builder_chaining() {
        #[cfg(windows)]
        let t = StdioTransport::command("cmd.exe", &["/c"])
            .env("A", "1")
            .current_dir("C:\\");
        #[cfg(not(windows))]
        let t = StdioTransport::command("sh", &["-c"])
            .env("A", "1")
            .current_dir("/tmp");

        assert_eq!(t.env.len(), 1);
        assert!(t.current_dir.is_some());
    }
}
