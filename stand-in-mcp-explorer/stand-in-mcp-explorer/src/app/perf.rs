//! Performance instrumentation (037 / O-024 / O-027) — gated, dev-only telemetry
//! for the list rendering paths. Emits JSONL (one JSON object per line) for
//! offline analysis of "rolagem dura" (hard scroll) with large lists.
//!
//! Behind the dev-only `perf` Cargo feature (off by default). Enabled at runtime
//! when the perf fixture is active (`--capture <region> perf`) or `MCPX_PERF` is
//! set. Two event kinds:
//!   - `frame`: one per rendered frame of the active list. Carries the inter-frame
//!     delta (`dt_ms`, fluidity), the **attribution** of the per-frame cost
//!     (`render_us` = the `StudioApp::render` body span; `clone_us` = the O(N)
//!     snapshot clone span; `build_us` = the visible-item build in the windowing
//!     closure), the visible/not-rendered/total counts, and the scroll position
//!     (`scroll_ix`).
//!   - `sample`: periodic own-process CPU%/RSS, taken by a background `sysinfo`
//!     thread.
//!
//! ## Cost attribution (O-027)
//!
//! `dt_ms` is the whole frame interval (render + layout + paint + idle).
//! `render_us` is just the `render` body (clones + eager tree build; the windowed
//! item build is deferred to layout, so it is NOT in `render_us`). `clone_us` is
//! the O(N) snapshot clone alone. Comparing them attributes the cost: if
//! `render_us` ≈ `dt_ms` the bottleneck is our render body (and `clone_us` says
//! how much is the clone); if `render_us` ≪ `dt_ms` it is gpui layout/paint.
//!
//! ## Phase (O-027 fix)
//!
//! `scroll_ix` is read from the scroll handle (`logical_scroll_top_index` /
//! `logical_scroll_top().item_ix`) — the true top-visible index, immune to the
//! `uniform_list` measurement pass (which calls the closure with range `0..1`).
//! `phase` is `scroll` when `scroll_ix > 0`, else `init`. The authoritative
//! segmentation is still done offline from `scroll_ix` / `dt_ms` / `t_ms`.
//!
//! ## Overhead
//!
//! Off by default (the whole module is `#[cfg(feature = "perf")]`); when built
//! in, each hook is a mutex lock plus arithmetic, and the per-frame emit is a
//! hand-formatted line + a buffered, flushed write (no `serde_json::Value`
//! allocation on the frame path) — a measurement caveat.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Sampler cadence. ≥ `sysinfo`'s minimum CPU update interval so `cpu_usage()`
/// is meaningful.
const SAMPLE_INTERVAL_MS: u64 = 200;

/// Default synthetic list size when `MCPX_PERF_N` is unset/invalid.
const DEFAULT_N: usize = 5000;

static PERF: OnceLock<Option<PerfLog>> = OnceLock::new();

/// Configured synthetic list size (`MCPX_PERF_N`, default 5000). Used by the
/// perf fixtures (`conn_state::perf_snapshot` / `capture_seed_history`).
pub fn perf_n() -> usize {
    std::env::var("MCPX_PERF_N")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_N)
}

/// Whether perf instrumentation should be active for these CLI args.
pub fn is_perf_run(capture: bool, state: &str) -> bool {
    (capture && state == "perf") || std::env::var_os("MCPX_PERF").is_some()
}

/// Initialize the global perf log + start the sampler thread (idempotent — a
/// second call is a no-op). Prints the absolute log path to stdout on success.
pub fn init() {
    PERF.get_or_init(|| match PerfLog::open() {
        Ok(log) => {
            println!("PERF LOG: {}", log.path);
            Some(log)
        }
        Err(e) => {
            eprintln!("perf: failed to open log file: {e}");
            None
        }
    });
    if get().is_some() {
        spawn_sampler();
    }
}

/// The active log, or `None` when perf is disabled / failed to open.
#[inline]
pub fn get() -> Option<&'static PerfLog> {
    PERF.get().and_then(|o| o.as_ref())
}

/// RAII span for the `StudioApp::render` body: created at the top of `render`,
/// records its lifetime (≈ the render-body duration) on drop. Place it AFTER
/// `on_frame` so it lands in the current frame's accumulator.
pub struct RenderTimer {
    t0: Instant,
}

impl Drop for RenderTimer {
    fn drop(&mut self) {
        if let Some(p) = get() {
            p.record_render(self.t0.elapsed().as_micros());
        }
    }
}

/// Start a render-body span, or `None` when perf is disabled.
pub fn render_timer() -> Option<RenderTimer> {
    get().map(|_| RenderTimer { t0: Instant::now() })
}

/// A live perf-measurement log: a buffered JSONL writer + the in-flight frame
/// accumulator.
pub struct PerfLog {
    path: String,
    origin: Instant,
    writer: Mutex<BufWriter<File>>,
    frame: Mutex<FrameState>,
}

#[derive(Default)]
struct FrameState {
    last_render: Option<Instant>,
    frame_index: u64,
    // --- accumulator for the in-flight frame (flushed at the next on_frame) ---
    acc_dirty: bool,
    acc_list: &'static str,
    acc_visible: u64,
    acc_total: u64,
    acc_build_us: u128,
    acc_clone_us: u128,
    acc_render_us: u128,
    /// Top-visible index for the in-flight frame (set at `on_frame`, flushed next).
    acc_scroll_ix: u64,
    /// Most recent scroll index — for the sampler thread's phase tag.
    last_scroll_ix: u64,
}

impl PerfLog {
    fn open() -> std::io::Result<PerfLog> {
        let path = std::env::var("MCPX_PERF_LOG").unwrap_or_else(|_| {
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            format!("mcpx-perf-{ms}.jsonl")
        });
        let file = File::create(&path)?;
        let shown = std::fs::canonicalize(&path)
            .ok()
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or_else(|| path.clone());
        Ok(PerfLog {
            path: shown,
            origin: Instant::now(),
            writer: Mutex::new(BufWriter::new(file)),
            frame: Mutex::new(FrameState::default()),
        })
    }

    fn t_ms(&self) -> u128 {
        self.origin.elapsed().as_millis()
    }

    fn emit(&self, line: &str) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = writeln!(w, "{line}");
            // Flush each line: the human closes the window to end the run, so a
            // buffered tail would be lost. Cheap enough for a dev instrument.
            let _ = w.flush();
        }
    }

    /// Record the build of `visible` items of `list` (total `total`), costing
    /// `build_us` microseconds. Called from the windowing closures — once per
    /// frame for `uniform_list` (whole range), once per item for `gpui::list`.
    pub fn record_items(&self, list: &'static str, visible: u64, total: u64, build_us: u128) {
        if let Ok(mut f) = self.frame.lock() {
            f.acc_dirty = true;
            f.acc_list = list;
            f.acc_total = total;
            f.acc_visible += visible;
            f.acc_build_us += build_us;
        }
    }

    /// Record an O(N) snapshot clone span (microseconds) for the in-flight frame.
    pub fn record_clone(&self, us: u128) {
        if let Ok(mut f) = self.frame.lock() {
            f.acc_clone_us += us;
        }
    }

    /// Record the `render` body span (microseconds) for the in-flight frame.
    pub fn record_render(&self, us: u128) {
        if let Ok(mut f) = self.frame.lock() {
            f.acc_render_us += us;
        }
    }

    /// Flush the previous frame's accumulated stats as a `frame` event and open
    /// a new frame. Called at the top of `StudioApp::render` with the active
    /// list's top-visible index (`scroll_ix`).
    pub fn on_frame(&self, tab: &'static str, scroll_ix: u64) {
        let now = Instant::now();
        let line = {
            let mut f = match self.frame.lock() {
                Ok(f) => f,
                Err(_) => return,
            };
            let dt = f.last_render.map(|l| (now - l).as_secs_f64() * 1000.0);
            f.last_render = Some(now);
            f.last_scroll_ix = scroll_ix;
            let out = if !f.acc_dirty {
                None
            } else {
                let phase = if f.acc_scroll_ix > 0 {
                    "scroll"
                } else {
                    "init"
                };
                let idx = f.frame_index;
                let visible = f.acc_visible;
                let total = f.acc_total;
                let not_rendered = total.saturating_sub(visible);
                let build_us = f.acc_build_us;
                let per_item = if visible > 0 {
                    build_us as f64 / visible as f64
                } else {
                    0.0
                };
                let clone_us = f.acc_clone_us;
                let render_us = f.acc_render_us;
                let s_ix = f.acc_scroll_ix;
                let list = f.acc_list;
                let t_ms = self.t_ms();
                let dt_str = match dt {
                    Some(v) => format!("{v:.3}"),
                    None => "null".to_string(),
                };
                f.frame_index += 1;
                f.acc_dirty = false;
                f.acc_visible = 0;
                f.acc_build_us = 0;
                f.acc_clone_us = 0;
                f.acc_render_us = 0;
                Some(format!(
                    r#"{{"ev":"frame","t_ms":{t_ms},"frame":{idx},"tab":"{tab}","list":"{list}","phase":"{phase}","scroll_ix":{s_ix},"dt_ms":{dt_str},"visible":{visible},"not_rendered":{not_rendered},"total":{total},"build_us":{build_us},"build_per_item_us":{per_item:.2},"clone_us":{clone_us},"render_us":{render_us}}}"#
                ))
            };
            // Store this frame's scroll index for its flush at the NEXT on_frame
            // (its items are recorded during the upcoming layout).
            f.acc_scroll_ix = scroll_ix;
            out
        };
        if let Some(line) = line {
            self.emit(&line);
        }
    }

    fn current_phase(&self) -> &'static str {
        match self.frame.lock() {
            Ok(f) if f.last_scroll_ix > 0 => "scroll",
            _ => "init",
        }
    }
}

/// Background thread: sample own-process CPU%/RSS every `SAMPLE_INTERVAL_MS`.
/// Detached — process exit ends it.
fn spawn_sampler() {
    let _ = std::thread::Builder::new()
        .name("mcpx-perf-sampler".into())
        .spawn(|| {
            let Some(perf) = get() else {
                return;
            };
            let Ok(pid) = sysinfo::get_current_pid() else {
                eprintln!("perf: cannot resolve current pid; CPU/mem sampling off");
                return;
            };
            let mut sys = sysinfo::System::new();
            loop {
                // This is a dedicated OS sampler thread, NOT the gpui main thread
                // or its executor — `std::thread::sleep` is the correct primitive
                // here (the disallowed-method lint targets sleeping the UI thread).
                #[allow(clippy::disallowed_methods)]
                std::thread::sleep(Duration::from_millis(SAMPLE_INTERVAL_MS));
                sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                let Some(proc) = sys.process(pid) else {
                    continue;
                };
                let cpu = proc.cpu_usage();
                let rss = proc.memory();
                let phase = perf.current_phase();
                let t_ms = perf.t_ms();
                perf.emit(&format!(
                    r#"{{"ev":"sample","t_ms":{t_ms},"phase":"{phase}","cpu_pct":{cpu:.2},"rss_bytes":{rss}}}"#
                ));
            }
        });
}
