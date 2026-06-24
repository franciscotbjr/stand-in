//! `StudioApp` — top-level app component, driven by connection state.
//!
//! Holds the command sender to the tokio engine and receives engine events
//! via a channel drained on the gpui executor. The render is a pure function
//! of `ConnState` — sidebar (M5) on the left, work area (M8+) on the right.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::active_tab::{Tab, TabCounts};
use crate::app::conn_state::{ConnState, capture_fixture_state, reduce};
use crate::app::engine_loop::spawn_engine;
use crate::app::events::{ConnConfig, EngineEvent, Transport, UiCommand};
use crate::app::history::{self, HistKind, HistoryEntry, PendingCall, push_history};
use crate::app::i18n::Lang;
use crate::app::log::{LogEntry, LogFilter, log_for_event, now_time, push_log};
use crate::app::secrets;
use crate::app::servers::{self, ServerEntry};
use crate::app::settings::{self, AppSettings, DensityChoice, PrimaryChoice, ThemeChoice};
use crate::args::Args;
use crate::bars::sidebar::auth_state::{
    AuthConfig, AuthDraft, AuthMethod, AuthStatus, credential_from,
};
use crate::bars::sidebar::brand_header::brand_mark_icon;
use crate::bars::sidebar::sidebar_state::SidebarState;
use crate::bars::topbar;
use crate::bars::{sidebar, tabbar};
use crate::tabs::prompts::PromptRun;
use crate::tabs::prompts::args::{build_prompt_args, missing_required as prompt_missing_required};
use crate::tabs::resources::{ResourceRead, reduce_resource_read};
use crate::tabs::tools::schema::{ParamField, ToolRun};
use crate::tabs::work_area;
use gpui::{
    App, AppContext as _, ClickEvent, Context, Entity, InteractiveElement, IntoElement,
    ListAlignment, ListState, ParentElement, Render, Styled, UniformListScrollHandle, WeakEntity,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_component::input::InputState;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_client::prelude::{
    Credential, OAuthConfig, OAuthTokens, PromptDefinition, Resource, ResourceTemplate,
    ToolDefinition,
};
use stand_in_mcp_explorer_ds::core::button::ClickHandler;
use stand_in_mcp_explorer_ds::navigation::BrandHeader;
use stand_in_mcp_explorer_ds::theme::density::GlobalDensity;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResultView {
    #[default]
    Friendly,
    Raw,
}

pub struct StudioApp {
    pub cmd_tx: UnboundedSender<UiCommand>,
    pub state: ConnState,
    evt_rx: Option<UnboundedReceiver<crate::app::events::EngineEvent>>,
    sidebar_form: Option<SidebarState>,
    saved_servers: Vec<ServerEntry>,
    last_dispatched: Option<ConnConfig>,
    lang: Lang,
    settings: AppSettings,
    settings_open: bool,
    auth_panel_open: bool,
    env_panel_open: bool,
    auth_draft: Option<AuthDraft>,
    guided: bool,
    active_tab: Tab,
    capture: bool,
    capture_longtext: bool,
    capture_tools_state: Option<String>,
    capture_resources_state: Option<String>,
    capture_env_state: Option<String>,
    // Tools tab state (M9)
    tools_scroll: UniformListScrollHandle,
    tool_filter_input: Option<Entity<InputState>>,
    selected_tool: Option<String>,
    tool_params: Vec<(ParamField, Entity<InputState>)>,
    pub tool_run: ToolRun,
    result_view: ResultView,
    tool_validation: Option<String>,
    // Resources tab state (M11)
    resource_filter_input: Option<Entity<InputState>>,
    selected_resource_uri: Option<String>,
    selected_template_uri: Option<String>,
    template_param_entities: Vec<(String, Entity<InputState>)>,
    resource_read: ResourceRead,
    subscribed_resources: HashSet<String>,
    // Resources scroll handle (M3)
    resources_scroll: UniformListScrollHandle,
    // Prompts scroll handle (M3)
    prompts_scroll: UniformListScrollHandle,
    // Prompts tab state (M12)
    prompt_filter_input: Option<Entity<InputState>>,
    selected_prompt: Option<String>,
    prompt_args: Vec<(stand_in::prompt::PromptArgument, Entity<InputState>)>,
    prompt_run: PromptRun,
    prompt_validation: Option<String>,
    capture_prompts_state: Option<String>,
    capture_auth_method: Option<AuthMethod>,
    capture_oauth_authorized: bool,
    pending_connect: bool,
    // OAuth config stashed for refresh-then-connect flow
    pending_oauth_config: Option<Box<OAuthConfig>>,
    // Notifications tab state (M13)
    logs: Vec<LogEntry>,
    log_filter: LogFilter,
    capture_notifications_state: Option<String>,
    // History tab state (M14)
    history: Vec<HistoryEntry>,
    pending_call: Option<PendingCall>,
    capture_history_state: Option<String>,
    // Virtual list state (M4)
    notifications_list: ListState,
    history_list: ListState,
    // O-013: cached Arc of the connected snapshot's tools, refreshed only on
    // state change so render() clones an O(1) handle instead of the N-item Vec
    // every frame (the ~17–24 ms/frame clone the 037 perf test isolated).
    tools_arc: Arc<[ToolDefinition]>,
    // O-025: timestamp of the last mouse-wheel scroll. While recent, the list
    // row hover highlight is suppressed — it otherwise strobes between items as
    // content scrolls under a stationary cursor. Trackpad scrolling is untouched.
    wheel_scroll_at: Option<Instant>,
    wheel_clear_armed: bool,
    // O-025: last frame's scroll position + tab, to detect a mouse-wheel "notch"
    // (a large one-frame jump) vs smooth trackpad motion.
    last_scroll_pos: f32,
    last_scroll_tab: Tab,
}

/// How long after the last mouse-wheel scroll to keep the row hover suppressed
/// (037 / O-025) — bridges the gaps between wheel notches, restores shortly after.
const HOVER_RESUME_MS: u64 = 160;

/// Build the cached tools Arc from the current connection state (O-013). Called
/// once per state transition (`new` + `handle_event`), never per frame.
fn tools_arc_from(state: &ConnState) -> Arc<[ToolDefinition]> {
    match state {
        ConnState::Connected(snap) => snap.tools.clone().into(),
        _ => Vec::new().into(),
    }
}

impl StudioApp {
    pub fn new(args: &Args, cx: &mut gpui::App) -> Self {
        let (cmd_tx, evt_rx) = spawn_engine();

        // Perf instrumentation (037 / O-024) — behind the dev-only `perf`
        // feature. Opens the JSONL log + starts the CPU/mem sampler when the
        // perf fixture/env is active.
        #[cfg(feature = "perf")]
        if crate::app::perf::is_perf_run(args.capture, &args.state) {
            crate::app::perf::init();
        }

        let state = if args.capture {
            capture_fixture_state(&args.state)
        } else {
            ConnState::Disconnected
        };
        let tools_arc = tools_arc_from(&state);

        let saved_servers = if args.capture {
            capture_presets()
        } else {
            servers::load()
        };

        // Load settings: capture uses defaults (theme forced by main.rs args),
        // live app loads persisted settings.
        let capture_settings_open = args.capture && args.region == "settings";
        let capture_auth_open = args.capture && args.region == "auth" && args.state != "stdio";
        let capture_env_open = args.capture && args.region == "env" && args.state != "trigger";
        // Auth capture: preset method from fixture state
        let capture_auth_method = if args.capture && args.region == "auth" {
            match args.state.as_str() {
                "basic" => Some(AuthMethod::Basic),
                "bearer" => Some(AuthMethod::Bearer),
                "oauth" | "oauth-authorized" => Some(AuthMethod::OAuth),
                _ => Some(AuthMethod::NoAuth),
            }
        } else {
            None
        };
        let capture_oauth_authorized =
            args.capture && args.region == "auth" && args.state == "oauth-authorized";
        let settings = if args.capture {
            // Capture overrides for settings region screenshots.
            let mut s = AppSettings::default();
            match args.state.as_str() {
                "light" => {
                    s.theme = ThemeChoice::Light;
                }
                "density-compact" => {
                    s.density = DensityChoice::Compact;
                }
                "comfy" => {
                    s.density = DensityChoice::Comfy;
                }
                "primary-genipina" => {
                    s.primary = PrimaryChoice::Genipina;
                }
                "oby" => {
                    s.primary = PrimaryChoice::Oby;
                }
                "guided-on" => {
                    s.guided = true;
                }
                _ => {}
            }
            s
        } else {
            let s = settings::load();
            // Apply the loaded settings immediately (cx is &mut App here).
            settings::apply_full(&s, cx);
            s
        };

        let capture_tools_state = if args.capture && args.region == "tools" {
            Some(args.state.clone())
        } else {
            None
        };

        let capture_resources_state = if args.capture && args.region == "resources" {
            Some(args.state.clone())
        } else {
            None
        };

        let capture_env_state = if args.capture && args.region == "env" {
            Some(args.state.clone())
        } else {
            None
        };

        let capture_prompts_state = if args.capture && args.region == "prompts" {
            Some(args.state.clone())
        } else {
            None
        };

        let capture_notifications_state = if args.capture && args.region == "notifications" {
            Some(args.state.clone())
        } else {
            None
        };

        let capture_history_state = if args.capture && args.region == "history" {
            Some(args.state.clone())
        } else {
            None
        };

        let (logs, log_filter) = if let Some(ref state) = capture_notifications_state {
            (capture_seed_logs(state.clone()), LogFilter::All)
        } else {
            (Vec::new(), LogFilter::All)
        };

        let history = if let Some(ref state) = capture_history_state {
            capture_seed_history(state.clone())
        } else {
            Vec::new()
        };

        let notifications_count = logs.len();
        let history_count = history.len();

        Self {
            cmd_tx,
            state,
            evt_rx: Some(evt_rx),
            sidebar_form: None,
            saved_servers,
            last_dispatched: None,
            lang: if args.capture && args.state == "lang-es" {
                Lang::Es
            } else {
                Lang::PtBr
            },
            settings,
            settings_open: capture_settings_open,
            auth_panel_open: capture_auth_open,
            env_panel_open: capture_env_open,
            auth_draft: None,
            guided: if args.capture {
                args.state == "guided" || args.state == "guided-on"
            } else {
                settings.guided
            },
            active_tab: if args.capture {
                if args.state == "perf" {
                    // Perf fixture (037): the active tab follows the region arg
                    // (the state is the perf marker, not a tab hint).
                    match args.region.as_str() {
                        "history" => Tab::History,
                        "resources" => Tab::Resources,
                        "prompts" => Tab::Prompts,
                        "notifications" => Tab::Notifications,
                        _ => Tab::Tools,
                    }
                } else {
                    match args.state.as_str() {
                        "tools" => Tab::Tools,
                        "resources" | "text" | "json" | "binary" | "subscribed" | "loading"
                        | "search" => Tab::Resources,
                        "prompts" => Tab::Prompts,
                        "notifications" | "populated" | "empty" | "filtered-error" | "large" => {
                            Tab::Notifications
                        }
                        "history" => Tab::History,
                        _ => Tab::Tools,
                    }
                }
            } else {
                Tab::Tools
            },
            capture: args.capture,
            capture_longtext: args.capture && args.state == "longtext",
            capture_tools_state,
            capture_resources_state,
            capture_env_state,
            tools_scroll: UniformListScrollHandle::new(),
            tool_filter_input: None,
            selected_tool: None,
            tool_params: Vec::new(),
            tool_run: ToolRun::default(),
            result_view: ResultView::default(),
            tool_validation: None,
            resource_filter_input: None,
            selected_resource_uri: None,
            selected_template_uri: None,
            template_param_entities: Vec::new(),
            resource_read: ResourceRead::default(),
            subscribed_resources: HashSet::new(),
            resources_scroll: UniformListScrollHandle::new(),
            prompts_scroll: UniformListScrollHandle::new(),
            prompt_filter_input: None,
            selected_prompt: None,
            prompt_args: Vec::new(),
            prompt_run: PromptRun::default(),
            prompt_validation: None,
            capture_prompts_state,
            capture_auth_method,
            capture_oauth_authorized,
            pending_connect: false,
            pending_oauth_config: None,
            logs,
            log_filter,
            capture_notifications_state,
            history,
            pending_call: None,
            capture_history_state,
            notifications_list: ListState::new(notifications_count, ListAlignment::Top, px(200.)),
            history_list: ListState::new(history_count, ListAlignment::Top, px(200.)),
            tools_arc,
            wheel_scroll_at: None,
            wheel_clear_armed: false,
            last_scroll_pos: 0.0,
            last_scroll_tab: Tab::Tools,
        }
    }
}

impl StudioApp {
    /// Start the event drain on the gpui executor. Called from `render()` once
    /// (via `evt_rx.take()`) — the spawned task runs until the channel closes.
    /// Arm a one-shot poller (gpui executor, NOT tokio) that restores list-row
    /// hover once mouse-wheel scrolling goes idle (037 / O-025).
    fn arm_hover_resume(&mut self, cx: &mut Context<Self>) {
        if self.wheel_clear_armed {
            return;
        }
        self.wheel_clear_armed = true;
        cx.spawn(|entity: WeakEntity<StudioApp>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(HOVER_RESUME_MS))
                        .await;
                    let keep = entity
                        .update(&mut cx, |this, cx| {
                            let idle = this.wheel_scroll_at.is_none_or(|t| {
                                (t.elapsed().as_millis() as u64) >= HOVER_RESUME_MS
                            });
                            if idle {
                                this.wheel_clear_armed = false;
                                cx.notify();
                            }
                            !idle
                        })
                        .unwrap_or(false);
                    if !keep {
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn start_event_drain(&mut self, cx: &mut Context<Self>) {
        let Some(mut rx) = self.evt_rx.take() else {
            return;
        };
        cx.spawn(|entity: WeakEntity<StudioApp>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(event) = rx.recv().await {
                    entity
                        .update(&mut cx, |app, cx| {
                            Self::handle_event(app, cx, event);
                        })
                        .ok();
                }
            }
        })
        .detach();
    }

    fn handle_event(app: &mut StudioApp, cx: &mut Context<StudioApp>, event: EngineEvent) {
        // Log every engine event for the Notifications tab (M13)
        if let Some(entry) = log_for_event(&event, now_time()) {
            push_log(&mut app.logs, entry);
            app.notifications_list
                .reset(crate::app::log::filter_logs(&app.logs, &app.log_filter).len());
        }

        match event {
            EngineEvent::ToolResult(r) => {
                let has_error = r.is_error == Some(true);
                let response = serde_json::to_value(&*r).unwrap_or_default();
                app.tool_run = ToolRun::Result(r);
                if let Some(pending) = app.pending_call.take()
                    && pending.kind == HistKind::Tool
                {
                    let entry =
                        history::make_history_entry(pending, response, now_time(), has_error);
                    push_history(&mut app.history, entry);
                    app.history_list.reset(app.history.len());
                }
            }
            EngineEvent::ToolError(e) => {
                app.tool_run = ToolRun::Error(e.clone());
                if let Some(pending) = app.pending_call.take()
                    && pending.kind == HistKind::Tool
                {
                    let response = serde_json::json!({"error": e});
                    let entry = history::make_history_entry(pending, response, now_time(), true);
                    push_history(&mut app.history, entry);
                    app.history_list.reset(app.history.len());
                }
            }
            EngineEvent::ResourceResult(r) => {
                let prev = std::mem::replace(&mut app.resource_read, ResourceRead::Idle);
                app.resource_read = reduce_resource_read(prev, &EngineEvent::ResourceResult(r));
            }
            EngineEvent::ResourceError(e) => {
                let prev = std::mem::replace(&mut app.resource_read, ResourceRead::Idle);
                app.resource_read = reduce_resource_read(prev, &EngineEvent::ResourceError(e));
            }
            EngineEvent::Subscribed(uri) => {
                app.subscribed_resources.insert(uri);
            }
            EngineEvent::Unsubscribed(uri) => {
                app.subscribed_resources.remove(&uri);
            }
            EngineEvent::PromptMessages(r) => {
                let response = serde_json::to_value(&*r).unwrap_or_default();
                app.prompt_run = PromptRun::Messages(r);
                if let Some(pending) = app.pending_call.take()
                    && pending.kind == HistKind::Prompt
                {
                    let entry = history::make_history_entry(pending, response, now_time(), false);
                    push_history(&mut app.history, entry);
                    app.history_list.reset(app.history.len());
                }
            }
            EngineEvent::PromptError(e) => {
                app.prompt_run = PromptRun::Error(e.clone());
                if let Some(pending) = app.pending_call.take()
                    && pending.kind == HistKind::Prompt
                {
                    let response = serde_json::json!({"error": e});
                    let entry = history::make_history_entry(pending, response, now_time(), true);
                    push_history(&mut app.history, entry);
                    app.history_list.reset(app.history.len());
                }
            }
            EngineEvent::Authorized(tokens) => {
                if let Some(ref mut draft) = app.auth_draft {
                    let access = tokens.access_token.clone();
                    let refresh = tokens.refresh_token.clone();
                    let expires = tokens.expires_at.map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    });
                    draft.oauth_status = AuthStatus::Authorized;
                    draft.oauth_tokens = Some(*tokens);
                    draft.oauth_error = None;
                    // Persist secrets to keychain
                    if let Some(ref config) = app.last_dispatched {
                        let mut s = secrets::load_secrets(&config.url);
                        s.oauth_access_token = Some(access);
                        s.oauth_refresh_token = refresh;
                        s.oauth_expires_at_unix = expires;
                        secrets::save_secrets(&config.url, &s);
                    }
                }
                // If there's a pending connect after refresh, fire it now
                if app.pending_connect {
                    app.pending_connect = false;
                    let oauth_config = app.pending_oauth_config.take();
                    // Build credential from fresh tokens and connect
                    if let Some(ref config) = app.last_dispatched
                        && let Some(ref draft) = app.auth_draft
                    {
                        let access = draft.oauth_tokens.as_ref().map(|t| t.access_token.as_str());
                        let cred = credential_from(draft.method, "", "", "", access);
                        let _ = app.cmd_tx.send(UiCommand::Connect {
                            config: config.clone(),
                            credential: Box::new(cred),
                        });
                    }
                    drop(oauth_config);
                }
            }
            EngineEvent::AuthorizationError(e) => {
                app.pending_connect = false;
                app.pending_oauth_config = None;
                if let Some(ref mut draft) = app.auth_draft {
                    draft.oauth_status = AuthStatus::Failed;
                    draft.oauth_error = Some(e.clone());
                }
            }
            EngineEvent::Notification(_) => {}
            evt => {
                let prev = std::mem::replace(&mut app.state, ConnState::Disconnected);
                app.state = reduce(prev, evt);
                app.tools_arc = tools_arc_from(&app.state);

                app.selected_tool = None;
                app.tool_params = Vec::new();
                app.tool_run = ToolRun::Idle;
                app.result_view = ResultView::Friendly;
                app.selected_resource_uri = None;
                app.selected_template_uri = None;
                app.resource_read = ResourceRead::Idle;
                app.selected_prompt = None;
                app.prompt_args = Vec::new();
                app.prompt_run = PromptRun::Idle;
                app.prompt_validation = None;
            }
        }

        // Auto-save on Connected (BUG-9)
        if let ConnState::Connected(ref snap) = app.state
            && let Some(ref config) = app.last_dispatched
        {
            let name = snap.server_info.name.clone();
            let auth = app.auth_draft.as_ref().map(|draft| AuthConfig {
                method: draft.method,
                basic_username: draft.basic_username.read(cx).text().to_string(),
                oauth_client_id: draft.oauth_client_id.read(cx).text().to_string(),
                oauth_auth_url: draft.oauth_auth_url.read(cx).text().to_string(),
                oauth_token_url: draft.oauth_token_url.read(cx).text().to_string(),
                oauth_scopes: draft.oauth_scopes.read(cx).text().to_string(),
            });
            let entry = ServerEntry {
                name,
                config: config.clone(),
                auth,
            };
            servers::add_dedup(&mut app.saved_servers, entry);
            let _ = servers::save(&app.saved_servers);
            // Persist secrets to keychain (HTTP only)
            if config.transport == Transport::Http
                && let Some(ref draft) = app.auth_draft
            {
                let access = draft.oauth_tokens.as_ref().map(|t| t.access_token.clone());
                let refresh = draft
                    .oauth_tokens
                    .as_ref()
                    .and_then(|t| t.refresh_token.clone());
                let expires = draft.oauth_tokens.as_ref().and_then(|t| {
                    t.expires_at.map(|st| {
                        st.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                });
                let s = crate::bars::sidebar::auth_state::AuthSecrets {
                    basic_password: draft.basic_password.read(cx).text().to_string(),
                    bearer_token: draft.bearer_token.read(cx).text().to_string(),
                    oauth_access_token: access,
                    oauth_refresh_token: refresh,
                    oauth_expires_at_unix: expires,
                };
                secrets::save_secrets(&config.url, &s);
            }
        }

        cx.notify();
    }
}

/// Pixel scroll offset (downward = positive) of a `uniform_list` handle, for the
/// O-025 mouse-wheel jump detection.
fn uniform_scroll_offset_px(handle: &UniformListScrollHandle) -> f32 {
    -f32::from(handle.0.borrow().base_handle.offset().y)
}

/// Top-visible item index of a `uniform_list` scroll handle — perf phase signal
/// (037/O-027). `uniform_list` scrolls via the base handle's pixel offset
/// (negative when scrolled down); its `logical_scroll_top()` returns 0 here.
/// Tools/Resources/Prompts use the fixed `LIST_ROW_HEIGHT`, so the top index is
/// `-offset.y / row_height`.
#[cfg(feature = "perf")]
fn uniform_scroll_top_ix(handle: &UniformListScrollHandle) -> usize {
    let offset_y = f32::from(handle.0.borrow().base_handle.offset().y);
    let row_h = stand_in_mcp_explorer_ds::data::LIST_ROW_HEIGHT;
    if row_h > 0.0 {
        ((-offset_y) / row_h).max(0.0) as usize
    } else {
        0
    }
}

impl Render for StudioApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.start_event_drain(cx);

        // O-025: suppress list-row hover during a mouse-wheel scroll so the
        // highlight doesn't strobe between items as content jumps under a
        // stationary cursor. A wheel "notch" moves the list a large amount in one
        // frame (measured ~2–3 rows); a trackpad scrolls smoothly (small per-frame
        // deltas), so classifying by the one-frame jump size keeps the trackpad
        // untouched. Detection = scroll-offset diff in render (reliable).
        let (scroll_pos, mouse_threshold) = match self.active_tab {
            Tab::Tools => (uniform_scroll_offset_px(&self.tools_scroll), 120.0),
            Tab::Resources => (uniform_scroll_offset_px(&self.resources_scroll), 120.0),
            Tab::Prompts => (uniform_scroll_offset_px(&self.prompts_scroll), 120.0),
            Tab::Notifications => (
                self.notifications_list.logical_scroll_top().item_ix as f32 * 100.0,
                150.0,
            ),
            Tab::History => (
                self.history_list.logical_scroll_top().item_ix as f32 * 100.0,
                150.0,
            ),
        };
        let jump = if self.last_scroll_tab == self.active_tab {
            (scroll_pos - self.last_scroll_pos).abs()
        } else {
            0.0
        };
        self.last_scroll_pos = scroll_pos;
        self.last_scroll_tab = self.active_tab;
        if jump > mouse_threshold {
            self.wheel_scroll_at = Some(Instant::now());
            self.arm_hover_resume(cx);
        }
        let hover_suppressed = self
            .wheel_scroll_at
            .is_some_and(|t| (t.elapsed().as_millis() as u64) < HOVER_RESUME_MS);
        cx.set_global(stand_in_mcp_explorer_ds::data::ListScrollHoverSuppressed(
            hover_suppressed,
        ));

        // Perf (037 / O-027): flush the previous frame + start the render-body
        // span. `scroll_ix` from the scroll handle = the true top-visible index
        // (immune to the uniform_list measure pass). Compiled only with `perf`.
        #[cfg(feature = "perf")]
        let _render_timer = {
            if let Some(p) = crate::app::perf::get() {
                let scroll_ix = match self.active_tab {
                    Tab::Tools => uniform_scroll_top_ix(&self.tools_scroll),
                    Tab::Resources => uniform_scroll_top_ix(&self.resources_scroll),
                    Tab::Prompts => uniform_scroll_top_ix(&self.prompts_scroll),
                    Tab::Notifications => self.notifications_list.logical_scroll_top().item_ix,
                    Tab::History => self.history_list.logical_scroll_top().item_ix,
                };
                p.on_frame(self.active_tab.perf_name(), scroll_ix as u64);
            }
            crate::app::perf::render_timer()
        };

        // --- Lazy-init sidebar form state (needs Window + cx for InputState entities) ---
        if self.sidebar_form.is_none() {
            let mut form = SidebarState {
                transport: Transport::Http, // HTTP default (036 addendum)
                command_input: cx.new(|cx| gpui_component::input::InputState::new(window, cx)),
                args_input: cx.new(|cx| gpui_component::input::InputState::new(window, cx)),
                url_input: cx.new(|cx| gpui_component::input::InputState::new(window, cx)),
                env_rows: Vec::new(),
            };
            if self.capture && self.capture_env_state.is_some() {
                // Env panel is STDIO-only — force STDIO so the env capture renders it
                // regardless of the new HTTP default.
                form.transport = Transport::Stdio;
                let n = match self.capture_env_state.as_deref() {
                    Some("empty") => 0,
                    Some("1") => 1,
                    Some("3") => 3,
                    Some("6") => 6,
                    Some("9") => 9,
                    _ => 2,
                };
                for _ in 0..n {
                    form.env_rows
                        .push(crate::bars::sidebar::sidebar_state::EnvRow {
                            key: cx.new(|cx| gpui_component::input::InputState::new(window, cx)),
                            value: cx.new(|cx| gpui_component::input::InputState::new(window, cx)),
                        });
                }
            }
            self.sidebar_form = Some(form);
        }

        // --- Lazy-init auth draft (needs Window + cx for InputState entities) ---
        if self.auth_draft.is_none() {
            let method = self.capture_auth_method.unwrap_or(AuthMethod::NoAuth);

            // Capture: pre-fill secrets for visual gate
            let is_capture_auth = self.capture_auth_method.is_some();
            let basic_pw_val: Option<&str> = if is_capture_auth {
                Some("s3cret!")
            } else {
                None
            };
            let bearer_token_val: Option<&str> = if is_capture_auth {
                Some("sk-demo-token-1234567890")
            } else {
                None
            };

            // Build each InputState entity individually to avoid closure borrow conflicts.
            let make_input =
                |cx: &mut Context<StudioApp>, window: &mut Window| -> Entity<InputState> {
                    cx.new(|cx| InputState::new(window, cx))
                };
            let make_input_masked = |cx: &mut Context<StudioApp>,
                                     window: &mut Window,
                                     default_val: Option<&str>|
             -> Entity<InputState> {
                cx.new(|cx| {
                    let mut state = InputState::new(window, cx);
                    state = state.masked(true);
                    if let Some(val) = default_val {
                        state = state.default_value(val.to_string());
                    }
                    state
                })
            };

            let basic_username = make_input(cx, window);
            let basic_password = make_input_masked(cx, window, basic_pw_val);
            let bearer_token = make_input_masked(cx, window, bearer_token_val);
            let oauth_client_id = if self.capture_oauth_authorized {
                cx.new(|cx| InputState::new(window, cx).default_value("demo-client-id"))
            } else {
                make_input(cx, window)
            };
            let oauth_auth_url = if self.capture_oauth_authorized {
                cx.new(|cx| {
                    InputState::new(window, cx).default_value("https://auth.example.com/authorize")
                })
            } else {
                make_input(cx, window)
            };
            let oauth_token_url = if self.capture_oauth_authorized {
                cx.new(|cx| {
                    InputState::new(window, cx).default_value("https://auth.example.com/token")
                })
            } else {
                make_input(cx, window)
            };
            let oauth_scopes = make_input(cx, window);

            let (oauth_status, oauth_tokens) = if self.capture_oauth_authorized {
                (
                    AuthStatus::Authorized,
                    Some(OAuthTokens {
                        access_token: "fixture-access-token".into(),
                        refresh_token: Some("fixture-refresh-token".into()),
                        expires_at: Some(
                            std::time::SystemTime::now() + std::time::Duration::from_secs(3600),
                        ),
                        token_type: "Bearer".into(),
                    }),
                )
            } else {
                (AuthStatus::default(), None)
            };

            self.auth_draft = Some(AuthDraft {
                method,
                basic_username,
                basic_password,
                bearer_token,
                oauth_client_id,
                oauth_auth_url,
                oauth_token_url,
                oauth_scopes,
                oauth_status,
                oauth_tokens,
                oauth_error: None,
            });
        }

        // --- Lazy-init tool filter input (M9) ---
        if self.tool_filter_input.is_none() {
            let filter = if let Some(ref state) = self.capture_tools_state
                && state == "search"
            {
                cx.new(|cx| {
                    gpui_component::input::InputState::new(window, cx).default_value("greet")
                })
            } else {
                cx.new(|cx| gpui_component::input::InputState::new(window, cx))
            };
            self.tool_filter_input = Some(filter);
        }

        // --- Lazy-init resource filter input (M11) ---
        if self.resource_filter_input.is_none() {
            let filter = if let Some(ref state) = self.capture_resources_state
                && state == "search"
            {
                cx.new(|cx| {
                    gpui_component::input::InputState::new(window, cx).default_value("readme")
                })
            } else {
                cx.new(|cx| gpui_component::input::InputState::new(window, cx))
            };
            self.resource_filter_input = Some(filter);
        }

        // --- Lazy-init prompt filter input (M12) ---
        if self.prompt_filter_input.is_none() {
            let filter = if let Some(ref state) = self.capture_prompts_state
                && state == "search"
            {
                cx.new(|cx| {
                    gpui_component::input::InputState::new(window, cx).default_value("greeting")
                })
            } else {
                cx.new(|cx| gpui_component::input::InputState::new(window, cx))
            };
            self.prompt_filter_input = Some(filter);
        }

        // --- Capture state: pre-select resource / template ---
        if self.capture
            && let ConnState::Connected(ref snap) = self.state
            && let Some(ref state) = self.capture_resources_state
            && self.template_param_entities.is_empty()
        {
            match state.as_str() {
                "text" | "json" | "binary" | "subscribed" | "loading" => {
                    // Select the matching resource by mime type heuristic
                    let target = match state.as_str() {
                        "json" => "file:///config",
                        "binary" => "file:///data",
                        _ => "file:///readme",
                    };
                    self.selected_resource_uri = Some(target.to_string());
                    self.selected_template_uri = None;
                }
                "search" => {
                    // Select the first resource
                    if let Some(r) = snap.resources.first() {
                        self.selected_resource_uri = Some(r.uri.clone());
                    }
                }
                _ => {}
            }
        }
        // --- Capture state: pre-select prompt ---
        if self.capture
            && let ConnState::Connected(ref snap) = self.state
            && let Some(ref state) = self.capture_prompts_state
            && self.prompt_args.is_empty()
        {
            match state.as_str() {
                "list" | "selected" | "messages-1" | "messages-2" | "search" => {
                    // Find the greeting prompt
                    if let Some(prompt) = snap.prompts.first() {
                        self.selected_prompt = Some(prompt.name.clone());
                        self.prompt_args = crate::tabs::prompts::rebuild_prompt_args(
                            self.selected_prompt.as_deref(),
                            &snap.prompts,
                            window,
                            cx,
                        );
                    }
                }
                _ => {}
            }
        }
        if self.capture
            && let ConnState::Connected(ref snap) = self.state
        {
            let tools = &snap.tools;
            if let Some(ref state) = self.capture_tools_state {
                // Only set up on first render when params are empty
                if self.tool_params.is_empty() {
                    match state.as_str() {
                        "selected" => {
                            if let Some(tool) = tools.first() {
                                self.selected_tool = Some(tool.name.clone());
                                self.tool_params = crate::tabs::tools::rebuild_params(
                                    self.selected_tool.as_deref(),
                                    tools,
                                    window,
                                    cx,
                                );
                            }
                        }
                        "empty" => {
                            // Select the tool with no params (echo)
                            if let Some(tool) = tools.iter().find(|t| t.name == "echo") {
                                self.selected_tool = Some(tool.name.clone());
                                self.tool_params = crate::tabs::tools::rebuild_params(
                                    self.selected_tool.as_deref(),
                                    tools,
                                    window,
                                    cx,
                                );
                            }
                        }
                        "search" => {
                            // Filter is pre-filled; select a tool too
                            if let Some(tool) = tools.first() {
                                self.selected_tool = Some(tool.name.clone());
                                self.tool_params = crate::tabs::tools::rebuild_params(
                                    self.selected_tool.as_deref(),
                                    tools,
                                    window,
                                    cx,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // --- Capture: seed tool run result fixtures (M16 N3) ---
        if self.capture
            && let ConnState::Connected(ref snap) = self.state
            && let Some(ref state) = self.capture_tools_state
            && matches!(self.tool_run, ToolRun::Idle)
        {
            match state.as_str() {
                "result-friendly" | "result-raw" => {
                    if let Some(tool) = snap.tools.first() {
                        self.selected_tool = Some(tool.name.clone());
                        self.tool_params = crate::tabs::tools::rebuild_params(
                            self.selected_tool.as_deref(),
                            &snap.tools,
                            window,
                            cx,
                        );
                        let result = stand_in_client::prelude::CallToolResult {
                            content: vec![stand_in_client::prelude::Content::text(
                                "Hello, Explorer! Greetings from the reference server.",
                            )],
                            is_error: None,
                        };
                        self.tool_run = ToolRun::Result(Box::new(result));
                        if state == "result-raw" {
                            self.result_view = ResultView::Raw;
                        }
                    }
                }
                "result-error" => {
                    if let Some(tool) = snap.tools.first() {
                        self.selected_tool = Some(tool.name.clone());
                        self.tool_params = crate::tabs::tools::rebuild_params(
                            self.selected_tool.as_deref(),
                            &snap.tools,
                            window,
                            cx,
                        );
                        let err_result = stand_in_client::prelude::CallToolResult {
                            content: vec![stand_in_client::prelude::Content::text(
                                "Tool execution failed: resource not found",
                            )],
                            is_error: Some(true),
                        };
                        self.tool_run = ToolRun::Result(Box::new(err_result));
                    }
                }
                "running" => {
                    if let Some(tool) = snap.tools.first() {
                        self.selected_tool = Some(tool.name.clone());
                        self.tool_params = crate::tabs::tools::rebuild_params(
                            self.selected_tool.as_deref(),
                            &snap.tools,
                            window,
                            cx,
                        );
                        self.tool_run = ToolRun::Running;
                    }
                }
                _ => {}
            }
        }

        let theme = cx.theme().clone();
        let density = cx.global::<GlobalDensity>().0;
        let sidebar_w = density.sidebar_w();
        let pad = density.pad();
        let lang = self.lang;
        let guided = self.guided;

        // --- Build env-remove handlers eagerly (one per index, up to current count) ---
        let env_count = self.sidebar_form.as_ref().map_or(0, |f| f.env_rows.len());
        let max_remove = if env_count == 0 { 1 } else { env_count };
        let mut remove_handlers: Vec<ClickHandler> = Vec::with_capacity(max_remove);
        for i in 0..max_remove {
            let h = cx.listener(move |this, _: &ClickEvent, _window, cx| {
                if let Some(ref mut f) = this.sidebar_form {
                    f.remove_env_row(i);
                }
                cx.notify();
            });
            remove_handlers.push(Box::new(h) as ClickHandler);
        }

        // --- Transport change handlers (segmented control) ---
        let t_stdio = cx.listener(move |this, _: &ClickEvent, _window, cx| {
            if let Some(ref mut f) = this.sidebar_form {
                f.transport = Transport::Stdio;
            }
            cx.notify();
        });
        let t_http = cx.listener(move |this, _: &ClickEvent, _window, cx| {
            if let Some(ref mut f) = this.sidebar_form {
                f.transport = Transport::Http;
            }
            cx.notify();
        });
        // HTTP first in the selector (036 addendum) → handlers ordered [http, stdio].
        let transport_handlers: Vec<ClickHandler> = vec![Box::new(t_http), Box::new(t_stdio)];

        // --- Auth handlers ---

        // Open auth panel
        let on_open_auth: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.auth_panel_open = true;
                cx.notify();
            }));

        // Close auth panel (for X button)
        let on_close_auth_x: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.auth_panel_open = false;
                cx.notify();
            }));

        // Close auth panel (for Save button — same behavior in M3)
        let on_close_auth_save: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.auth_panel_open = false;
                cx.notify();
            }));

        // Close auth panel on click-outside (mouse-down catcher)
        let auth_close_outside_entity = cx.entity().downgrade();
        let on_close_auth_outside: crate::screens::auth_panel::ClickOutsideHandler =
            Box::new(move |_window, cx| {
                if let Some(studio) = auth_close_outside_entity.upgrade() {
                    studio.update(cx, |app, cx| {
                        app.auth_panel_open = false;
                        cx.notify();
                    });
                }
            });

        // --- Env handlers ---

        let on_open_env: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.env_panel_open = true;
                cx.notify();
            }));

        let on_close_env_x: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.env_panel_open = false;
                cx.notify();
            }));

        let on_close_env_save: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.env_panel_open = false;
                cx.notify();
            }));

        let env_close_outside_entity = cx.entity().downgrade();
        let on_close_env_outside: crate::screens::auth_panel::ClickOutsideHandler =
            Box::new(move |_window, cx| {
                if let Some(studio) = env_close_outside_entity.upgrade() {
                    studio.update(cx, |app, cx| {
                        app.env_panel_open = false;
                        cx.notify();
                    });
                }
            });

        // Auth method change (Arc/weak — uses entity.update, NOT cx.listener, to avoid double-lease)
        let auth_entity = cx.entity().downgrade();
        let on_auth_method_change: stand_in_mcp_explorer_ds::forms::select::SelectHandler =
            std::sync::Arc::new(move |ix, _value, _window, cx| {
                if let Some(studio) = auth_entity.upgrade() {
                    studio.update(cx, |app, cx| {
                        if let Some(ref mut draft) = app.auth_draft {
                            draft.method = AuthMethod::from_ix(ix);
                        }
                        cx.notify();
                    });
                }
            });

        // OAuth authorize button (Arc/weak — pattern matching on_auth_method_change to avoid double-lease)
        let cmd_tx_auth = self.cmd_tx.clone();
        let auth_entity_authorize = cx.entity().downgrade();
        let on_authorize: ClickHandler = Box::new({
            move |_: &ClickEvent, _window: &mut Window, cx: &mut App| {
                if let Some(studio) = auth_entity_authorize.upgrade() {
                    studio.update(cx, |app, cx| {
                        let Some(ref draft) = app.auth_draft else {
                            return;
                        };
                        let client_id = draft.oauth_client_id.read(cx).text().to_string();
                        let auth_url = draft.oauth_auth_url.read(cx).text().to_string();
                        let token_url = draft.oauth_token_url.read(cx).text().to_string();
                        let scopes = draft.oauth_scopes.read(cx).text().to_string();
                        if client_id.is_empty() || auth_url.is_empty() || token_url.is_empty() {
                            if let Some(ref mut d) = app.auth_draft {
                                d.oauth_status = AuthStatus::Failed;
                                d.oauth_error = Some(
                                    crate::app::i18n::tr("auth.authorizeFirst", app.lang)
                                        .to_string(),
                                );
                            }
                            cx.notify();
                            return;
                        }
                        if let Some(ref mut d) = app.auth_draft {
                            d.oauth_status = AuthStatus::Authorizing;
                            d.oauth_error = None;
                        }
                        let oauth_config = OAuthConfig::new(
                            client_id,
                            auth_url,
                            token_url,
                            scopes.split_whitespace().map(|s| s.to_string()).collect(),
                        );
                        let _ = cmd_tx_auth.send(UiCommand::Authorize {
                            config: Box::new(oauth_config),
                        });
                        cx.notify();
                    });
                }
            }
        });

        // --- Connect handler: reads input states, sends UiCommand, stashes config ---
        // Auth credential is derived from the current auth_draft (M4).
        let cmd_tx_connect = self.cmd_tx.clone();
        let on_connect: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                if let Some(ref f) = this.sidebar_form {
                    let config = f.current_config(cx);
                    this.last_dispatched = Some(config.clone());
                    // OAuth refresh-before-connect: if the access token is expired and a
                    // refresh_token is available, trigger a refresh first instead of
                    // connecting with a stale token. The Authorized handler fires the real
                    // Connect when the refresh completes (AC M4 §7, B1 fix).
                    if let Some(ref draft) = this.auth_draft
                        && config.transport == Transport::Http
                        && draft.method == AuthMethod::OAuth
                        && let Some(ref tokens) = draft.oauth_tokens
                        && tokens.is_expired()
                        && let Some(ref refresh_token) = tokens.refresh_token
                    {
                        let client_id = draft.oauth_client_id.read(cx).text().to_string();
                        let auth_url = draft.oauth_auth_url.read(cx).text().to_string();
                        let token_url = draft.oauth_token_url.read(cx).text().to_string();
                        let scopes = draft.oauth_scopes.read(cx).text().to_string();
                        let oauth_config = OAuthConfig::new(
                            client_id,
                            auth_url,
                            token_url,
                            scopes.split_whitespace().map(|s| s.to_string()).collect(),
                        );
                        this.pending_connect = true;
                        this.pending_oauth_config = Some(Box::new(oauth_config.clone()));
                        let _ = cmd_tx_connect.send(UiCommand::RefreshAuth {
                            config: Box::new(oauth_config),
                            refresh_token: refresh_token.clone(),
                        });
                        cx.notify();
                        return;
                    }
                    let credential = match this.auth_draft.as_ref() {
                        Some(draft) if config.transport == Transport::Http => {
                            let user = draft.basic_username.read(cx).text().to_string();
                            let pw = draft.basic_password.read(cx).text().to_string();
                            let bearer = draft.bearer_token.read(cx).text().to_string();
                            let access =
                                draft.oauth_tokens.as_ref().map(|t| t.access_token.as_str());
                            credential_from(draft.method, &user, &pw, &bearer, access)
                        }
                        _ => Credential::default(),
                    };
                    let _ = cmd_tx_connect.send(UiCommand::Connect {
                        config,
                        credential: Box::new(credential),
                    });
                }
                cx.notify();
            }));

        // --- Disconnect handler ---
        let cmd_tx_disconnect = self.cmd_tx.clone();
        let on_disconnect: ClickHandler =
            Box::new(cx.listener(move |_this, _: &ClickEvent, _window, cx| {
                let _ = cmd_tx_disconnect.send(UiCommand::Disconnect);
                cx.notify();
            }));

        // --- Add env row handler ---
        let on_add_env: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, window, cx| {
                use gpui_component::input::InputState;
                if let Some(ref mut f) = this.sidebar_form {
                    f.env_rows
                        .push(crate::bars::sidebar::sidebar_state::EnvRow {
                            key: cx.new(|cx| InputState::new(window, cx)),
                            value: cx.new(|cx| InputState::new(window, cx)),
                        });
                }
                cx.notify();
            }));

        // --- Pick-preset handlers (one per saved server entry) ---
        let mut on_pick_preset: Vec<ClickHandler> = Vec::with_capacity(self.saved_servers.len());
        for (i, _entry) in self.saved_servers.iter().enumerate() {
            let h = cx.listener(move |this, _: &ClickEvent, window, cx| {
                use crate::bars::sidebar::sidebar_state::EnvRow;
                if this.saved_servers.get(i).is_none() {
                    return;
                }
                let config = this.saved_servers[i].config.clone();
                if let Some(form) = &mut this.sidebar_form {
                    form.transport = config.transport;
                    form.command_input = cx.new(|cx| {
                        gpui_component::input::InputState::new(window, cx)
                            .default_value(config.command.clone())
                    });
                    form.args_input = cx.new(|cx| {
                        gpui_component::input::InputState::new(window, cx)
                            .default_value(config.args.join(" "))
                    });
                    form.url_input = cx.new(|cx| {
                        gpui_component::input::InputState::new(window, cx)
                            .default_value(config.url.clone())
                    });
                    form.env_rows = config
                        .env
                        .iter()
                        .map(|(k, v)| EnvRow {
                            key: cx.new(|cx| {
                                gpui_component::input::InputState::new(window, cx)
                                    .default_value(k.clone())
                            }),
                            value: cx.new(|cx| {
                                gpui_component::input::InputState::new(window, cx)
                                    .default_value(v.clone())
                            }),
                        })
                        .collect();
                }
                // Hydrate auth draft from stored config + secrets (M4)
                // Replaces the "reset-on-preset" behavior of M3.
                let secrets_for_hydrate = if config.transport == Transport::Http {
                    Some(secrets::load_secrets(&config.url))
                } else {
                    None
                };
                this.auth_draft = match this.saved_servers[i].auth.as_ref() {
                    Some(auth_cfg) => {
                        let method = auth_cfg.method;
                        let s = secrets_for_hydrate.unwrap_or_default();
                        let oauth_tokens =
                            s.oauth_access_token.as_ref().map(|access| OAuthTokens {
                                access_token: access.clone(),
                                refresh_token: s.oauth_refresh_token.clone(),
                                expires_at: s.oauth_expires_at_unix.map(|unixtime| {
                                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(unixtime)
                                }),
                                token_type: "Bearer".into(),
                            });
                        let oauth_status = if oauth_tokens.is_some() {
                            AuthStatus::Authorized
                        } else {
                            AuthStatus::Idle
                        };
                        Some(AuthDraft {
                            method,
                            basic_username: cx.new(|cx| {
                                gpui_component::input::InputState::new(window, cx)
                                    .default_value(auth_cfg.basic_username.clone())
                            }),
                            basic_password: cx.new(|cx| {
                                let mut st = gpui_component::input::InputState::new(window, cx);
                                st = st.masked(true);
                                if !s.basic_password.is_empty() {
                                    st = st.default_value(s.basic_password.clone());
                                }
                                st
                            }),
                            bearer_token: cx.new(|cx| {
                                let mut st = gpui_component::input::InputState::new(window, cx);
                                st = st.masked(true);
                                if !s.bearer_token.is_empty() {
                                    st = st.default_value(s.bearer_token.clone());
                                }
                                st
                            }),
                            oauth_client_id: cx.new(|cx| {
                                gpui_component::input::InputState::new(window, cx)
                                    .default_value(auth_cfg.oauth_client_id.clone())
                            }),
                            oauth_auth_url: cx.new(|cx| {
                                gpui_component::input::InputState::new(window, cx)
                                    .default_value(auth_cfg.oauth_auth_url.clone())
                            }),
                            oauth_token_url: cx.new(|cx| {
                                gpui_component::input::InputState::new(window, cx)
                                    .default_value(auth_cfg.oauth_token_url.clone())
                            }),
                            oauth_scopes: cx.new(|cx| {
                                gpui_component::input::InputState::new(window, cx)
                                    .default_value(auth_cfg.oauth_scopes.clone())
                            }),
                            oauth_status,
                            oauth_tokens,
                            oauth_error: None,
                        })
                    }
                    None => None,
                };
                this.auth_panel_open = false;
                cx.notify();
            });
            on_pick_preset.push(Box::new(h) as ClickHandler);
        }

        // --- Topbar handlers ---

        // Language change (UI-state, not engine command)
        let lang_entity = cx.entity().downgrade();
        let on_lang_change: stand_in_mcp_explorer_ds::forms::select::SelectHandler =
            std::sync::Arc::new(move |ix, _value, _window, cx| {
                if let Some(studio_app) = lang_entity.upgrade() {
                    studio_app.update(cx, |app, cx| {
                        app.lang = match ix {
                            1 => Lang::En,
                            2 => Lang::Es,
                            _ => Lang::PtBr,
                        };
                        cx.notify();
                    });
                }
            });

        // Guided toggle — unified with settings (single source of truth).
        let on_guided_toggle: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.guided = !this.guided;
                this.settings.guided = this.guided;
                let _ = settings::save(&this.settings);
                cx.notify();
            }));

        // Settings: open overlay handler
        let on_open_settings: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings_open = !this.settings_open;
                cx.notify();
            }));

        // Settings: close overlay handler (scrim click / ESC)
        let on_close_settings: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings_open = false;
                cx.notify();
            }));

        // Settings: theme segmented control handlers
        let on_theme_dark: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.theme = ThemeChoice::Dark;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let on_theme_light: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.theme = ThemeChoice::Light;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let theme_handlers: Vec<ClickHandler> = vec![on_theme_dark, on_theme_light];

        // Settings: density segmented control handlers
        let on_density_compact: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.density = DensityChoice::Compact;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let on_density_regular: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.density = DensityChoice::Regular;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let on_density_comfy: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.density = DensityChoice::Comfy;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let density_handlers: Vec<ClickHandler> =
            vec![on_density_compact, on_density_regular, on_density_comfy];

        // Settings: primary swatch handlers
        let on_primary_jandi: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.primary = PrimaryChoice::Jandi;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let on_primary_genipina: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.primary = PrimaryChoice::Genipina;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let on_primary_oby: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.settings.primary = PrimaryChoice::Oby;
                settings::apply_full(&this.settings, cx);
                let _ = settings::save(&this.settings);
                cx.notify();
            }));
        let primary_handlers: Vec<ClickHandler> =
            vec![on_primary_jandi, on_primary_genipina, on_primary_oby];

        // Settings: guided toggle (in the settings panel — syncs both flags)
        let on_settings_guided_toggle: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.guided = !this.guided;
                this.settings.guided = this.guided;
                let _ = settings::save(&this.settings);
                cx.notify();
            }));

        // Reconnect
        let cmd_tx_reconnect = self.cmd_tx.clone();
        let on_reconnect: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                if let Some(ref config) = this.last_dispatched {
                    // OAuth refresh-before-connect (same guard as on_connect, B1 fix)
                    if let Some(ref draft) = this.auth_draft
                        && config.transport == Transport::Http
                        && draft.method == AuthMethod::OAuth
                        && let Some(ref tokens) = draft.oauth_tokens
                        && tokens.is_expired()
                        && let Some(ref refresh_token) = tokens.refresh_token
                    {
                        let client_id = draft.oauth_client_id.read(cx).text().to_string();
                        let auth_url = draft.oauth_auth_url.read(cx).text().to_string();
                        let token_url = draft.oauth_token_url.read(cx).text().to_string();
                        let scopes = draft.oauth_scopes.read(cx).text().to_string();
                        let oauth_config = OAuthConfig::new(
                            client_id,
                            auth_url,
                            token_url,
                            scopes.split_whitespace().map(|s| s.to_string()).collect(),
                        );
                        this.pending_connect = true;
                        this.pending_oauth_config = Some(Box::new(oauth_config.clone()));
                        let _ = cmd_tx_reconnect.send(UiCommand::RefreshAuth {
                            config: Box::new(oauth_config),
                            refresh_token: refresh_token.clone(),
                        });
                        cx.notify();
                        return;
                    }
                    let credential = match this.auth_draft.as_ref() {
                        Some(draft) if config.transport == Transport::Http => {
                            let user = draft.basic_username.read(cx).text().to_string();
                            let pw = draft.basic_password.read(cx).text().to_string();
                            let bearer = draft.bearer_token.read(cx).text().to_string();
                            let access =
                                draft.oauth_tokens.as_ref().map(|t| t.access_token.as_str());
                            credential_from(draft.method, &user, &pw, &bearer, access)
                        }
                        _ => Credential::default(),
                    };
                    let _ = cmd_tx_reconnect.send(UiCommand::Connect {
                        config: config.clone(),
                        credential: Box::new(credential),
                    });
                }
                cx.notify();
            }));

        // --- Tab counts (derived from state) ---
        let tab_counts = match &self.state {
            ConnState::Connected(snap) => TabCounts::from_snapshot(snap),
            _ => TabCounts::ZERO,
        };

        let connected = matches!(&self.state, ConnState::Connected(_));

        // O-013: clone the cached Arc handle (O(1)) instead of the snapshot's
        // N-item Vec every frame. `tools_arc` is refreshed only on state change.
        let tools: Arc<[ToolDefinition]> = {
            #[cfg(feature = "perf")]
            let _c0 = std::time::Instant::now();
            #[allow(clippy::let_and_return)]
            let cloned = self.tools_arc.clone();
            #[cfg(feature = "perf")]
            if let Some(p) = crate::app::perf::get() {
                p.record_clone(_c0.elapsed().as_micros());
            }
            cloned
        };

        let resources: Vec<Resource> = match &self.state {
            ConnState::Connected(snap) => snap.resources.clone(),
            _ => vec![],
        };
        let templates: Vec<ResourceTemplate> = match &self.state {
            ConnState::Connected(snap) => snap.templates.clone(),
            _ => vec![],
        };
        let prompts: Vec<PromptDefinition> = match &self.state {
            ConnState::Connected(snap) => snap.prompts.clone(),
            _ => vec![],
        };

        // --- Tool selection handler (M9) ---
        let studio_entity = cx.entity().downgrade();
        let on_select_tool: crate::tabs::tools::ToolSelectFn = std::sync::Arc::new(
            move |tool: &ToolDefinition, window: &mut Window, app_cx: &mut App| {
                let tool_name = tool.name.clone();
                if let Some(studio) = studio_entity.upgrade() {
                    studio.update(app_cx, |app, cx| {
                        app.selected_tool = Some(tool_name.clone());
                        let tools = match &app.state {
                            ConnState::Connected(snap) => snap.tools.clone(),
                            _ => vec![],
                        };
                        app.tool_params = crate::tabs::tools::rebuild_params(
                            Some(&tool_name),
                            &tools,
                            window,
                            cx,
                        );
                        app.tool_run = ToolRun::Idle;
                        app.tool_validation = None;
                        cx.notify();
                    });
                }
            },
        );

        // --- Run handler (M10): collects params, validates, dispatches CallTool ---
        let cmd_tx_run = self.cmd_tx.clone();
        let on_run: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                let Some(ref tool_name) = this.selected_tool else {
                    return;
                };
                let tool_name = tool_name.clone();

                let texts: Vec<(crate::tabs::tools::schema::ParamField, String)> = this
                    .tool_params
                    .iter()
                    .map(|(field, state)| (field.clone(), state.read(cx).text().to_string()))
                    .collect();

                let missing = crate::tabs::tools::schema::missing_required(&texts);
                if !missing.is_empty() {
                    this.tool_validation = Some(format!(
                        "{}{}",
                        crate::app::i18n::tr("tools.fill", this.lang),
                        missing.join(", ")
                    ));
                    cx.notify();
                    return;
                }

                match crate::tabs::tools::schema::build_arguments(&texts) {
                    Some(args) => {
                        this.tool_run = ToolRun::Running;
                        this.tool_validation = None;
                        this.result_view = ResultView::Friendly;
                        this.pending_call = Some(PendingCall {
                            kind: HistKind::Tool,
                            name: tool_name.clone(),
                            request: args.clone(),
                            started: std::time::Instant::now(),
                        });
                        let _ = cmd_tx_run.send(UiCommand::CallTool {
                            name: tool_name,
                            arguments: args,
                        });
                    }
                    None => {
                        this.tool_validation = Some(String::from(crate::app::i18n::tr(
                            "tools.invalidNumber",
                            this.lang,
                        )));
                    }
                }
                cx.notify();
            }));

        // --- Result-view toggle handler (M10) ---
        let on_result_view_toggle: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.result_view = match this.result_view {
                    ResultView::Friendly => ResultView::Raw,
                    ResultView::Raw => ResultView::Friendly,
                };
                cx.notify();
            }));

        // --- Resource selection handler (M11) ---
        let studio_entity_res = cx.entity().downgrade();
        let on_select_concrete: crate::tabs::resources::ResourceSelectFn = std::sync::Arc::new(
            move |resource: &Resource, _window: &mut Window, app_cx: &mut App| {
                let uri = resource.uri.clone();
                if let Some(studio) = studio_entity_res.upgrade() {
                    studio.update(app_cx, |app, cx| {
                        app.selected_resource_uri = Some(uri.clone());
                        app.selected_template_uri = None;
                        app.template_param_entities = Vec::new();
                        app.resource_read = ResourceRead::Loading;
                        cx.notify();
                        // Auto-read on select
                        let _ = app.cmd_tx.send(UiCommand::ReadResource { uri });
                    });
                }
            },
        );

        let studio_entity_tpl = cx.entity().downgrade();
        let on_select_template: crate::tabs::resources::TemplateSelectFn = std::sync::Arc::new(
            move |tpl: &ResourceTemplate, window: &mut Window, app_cx: &mut App| {
                let uri_template = tpl.uri_template.clone();
                if let Some(studio) = studio_entity_tpl.upgrade() {
                    studio.update(app_cx, |app, cx| {
                        app.selected_resource_uri = None;
                        app.selected_template_uri = Some(uri_template.clone());
                        app.resource_read = ResourceRead::Idle;
                        // Build param entities for each template param
                        use crate::tabs::resources::content::template_params;
                        let params = template_params(&uri_template);
                        app.template_param_entities = params
                            .iter()
                            .map(|p| {
                                (
                                    p.clone(),
                                    cx.new(|cx| gpui_component::input::InputState::new(window, cx)),
                                )
                            })
                            .collect();
                        cx.notify();
                    });
                }
            },
        );

        // --- Resource read handler (M11) — dispatches ReadResource to engine ---
        let cmd_tx_read = self.cmd_tx.clone();
        let on_resource_read: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                // Determine URI: concrete or substituted template
                let uri = if let Some(ref uri) = this.selected_resource_uri {
                    Some(uri.clone())
                } else if let Some(ref tpl_uri) = this.selected_template_uri {
                    let params: Vec<(String, String)> = this
                        .template_param_entities
                        .iter()
                        .map(|(k, e)| (k.clone(), e.read(cx).text().to_string()))
                        .collect();
                    let params_ref: Vec<(&str, &str)> = params
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    Some(crate::tabs::resources::content::substitute_template(
                        tpl_uri,
                        &params_ref,
                    ))
                } else {
                    None
                };
                if let Some(uri) = uri {
                    this.resource_read = ResourceRead::Loading;
                    let _ = cmd_tx_read.send(UiCommand::ReadResource { uri });
                }
                cx.notify();
            }));

        // --- Subscribe handler (M11) ---
        let cmd_tx_sub = self.cmd_tx.clone();
        let on_subscribe: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                if let Some(ref uri) = this.selected_resource_uri {
                    let _ = cmd_tx_sub.send(UiCommand::Subscribe { uri: uri.clone() });
                }
                cx.notify();
            }));

        // --- Unsubscribe handler (M11) ---
        let cmd_tx_unsub = self.cmd_tx.clone();
        let on_unsubscribe: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                if let Some(ref uri) = this.selected_resource_uri {
                    let _ = cmd_tx_unsub.send(UiCommand::Unsubscribe { uri: uri.clone() });
                }
                cx.notify();
            }));

        // --- Prompt selection handler (M12) ---
        let studio_entity_prompt = cx.entity().downgrade();
        let on_select_prompt: crate::tabs::prompts::PromptSelectFn = std::sync::Arc::new(
            move |prompt: &PromptDefinition, window: &mut Window, app_cx: &mut App| {
                let prompt_name = prompt.name.clone();
                if let Some(studio) = studio_entity_prompt.upgrade() {
                    studio.update(app_cx, |app, cx| {
                        app.selected_prompt = Some(prompt_name.clone());
                        let prompts = match &app.state {
                            ConnState::Connected(snap) => snap.prompts.clone(),
                            _ => vec![],
                        };
                        app.prompt_args = crate::tabs::prompts::rebuild_prompt_args(
                            Some(&prompt_name),
                            &prompts,
                            window,
                            cx,
                        );
                        app.prompt_run = PromptRun::Idle;
                        app.prompt_validation = None;
                        cx.notify();
                    });
                }
            },
        );

        // --- Generate handler (M12): collects args, validates, dispatches GetPrompt ---
        let cmd_tx_generate = self.cmd_tx.clone();
        let on_generate: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                let Some(ref prompt_name) = this.selected_prompt else {
                    return;
                };
                let prompt_name = prompt_name.clone();

                let texts: Vec<(stand_in::prompt::PromptArgument, String)> = this
                    .prompt_args
                    .iter()
                    .map(|(arg, state)| (arg.clone(), state.read(cx).text().to_string()))
                    .collect();

                let missing = prompt_missing_required(&texts);
                if !missing.is_empty() {
                    this.prompt_validation = Some(format!(
                        "{}{}",
                        crate::app::i18n::tr("prompts.fill", this.lang),
                        missing.join(", ")
                    ));
                    cx.notify();
                    return;
                }

                let args = build_prompt_args(&texts);
                this.prompt_run = PromptRun::Building;
                this.prompt_validation = None;
                this.pending_call = Some(PendingCall {
                    kind: HistKind::Prompt,
                    name: prompt_name.clone(),
                    request: args.clone(),
                    started: std::time::Instant::now(),
                });
                let _ = cmd_tx_generate.send(UiCommand::GetPrompt {
                    name: prompt_name,
                    arguments: args,
                });
                cx.notify();
            }));

        // --- Tab change handlers (one per tab) ---
        let mut tab_handlers: Vec<Option<ClickHandler>> = Vec::with_capacity(5);
        {
            let ordered = [
                Tab::Tools,
                Tab::Resources,
                Tab::Prompts,
                Tab::Notifications,
                Tab::History,
            ];
            for tab in ordered {
                let h: ClickHandler =
                    Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                        this.active_tab = tab;
                        cx.notify();
                    }));
                tab_handlers.push(Some(h));
            }
        }

        // --- Notification handlers (M13) ---

        // Log filter change handlers (one per SegmentedControl segment)
        let mut on_filter_change: Vec<ClickHandler> = Vec::with_capacity(5);
        for ix in 0..5 {
            let fill = LogFilter::from_ix(ix);
            let h = cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.log_filter = fill;
                this.notifications_list
                    .reset(crate::app::log::filter_logs(&this.logs, &this.log_filter).len());
                cx.notify();
            });
            on_filter_change.push(Box::new(h) as ClickHandler);
        }

        // Clear logs handler
        let on_clear_logs: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.logs.clear();
                this.notifications_list.reset(0);
                cx.notify();
            }));

        // History clear handler (M14)
        let on_clear_history: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.history.clear();
                this.history_list.reset(0);
                cx.notify();
            }));

        // History expand toggle handlers (M14) — one per entry, up to current length
        let history_len = self.history.len();
        let max_toggle = if history_len == 0 { 1 } else { history_len };
        let mut history_toggle_handlers: Vec<ClickHandler> = Vec::with_capacity(max_toggle);
        for i in 0..max_toggle {
            let h: ClickHandler =
                Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                    if let Some(entry) = this.history.get_mut(i) {
                        entry.expanded = !entry.expanded;
                        this.history_list.splice(i..i + 1, 1);
                    }
                    cx.notify();
                }));
            history_toggle_handlers.push(h);
        }

        // --- Work-area CTA handler (cloned connect logic for onboarding screen) ---
        let cmd_tx_cta = self.cmd_tx.clone();
        let on_connect_work_area: ClickHandler =
            Box::new(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                if let Some(ref f) = this.sidebar_form {
                    let config = f.current_config(cx);
                    this.last_dispatched = Some(config.clone());
                    // OAuth refresh-before-connect (same guard as on_connect, B1 fix)
                    if let Some(ref draft) = this.auth_draft
                        && config.transport == Transport::Http
                        && draft.method == AuthMethod::OAuth
                        && let Some(ref tokens) = draft.oauth_tokens
                        && tokens.is_expired()
                        && let Some(ref refresh_token) = tokens.refresh_token
                    {
                        let client_id = draft.oauth_client_id.read(cx).text().to_string();
                        let auth_url = draft.oauth_auth_url.read(cx).text().to_string();
                        let token_url = draft.oauth_token_url.read(cx).text().to_string();
                        let scopes = draft.oauth_scopes.read(cx).text().to_string();
                        let oauth_config = OAuthConfig::new(
                            client_id,
                            auth_url,
                            token_url,
                            scopes.split_whitespace().map(|s| s.to_string()).collect(),
                        );
                        this.pending_connect = true;
                        this.pending_oauth_config = Some(Box::new(oauth_config.clone()));
                        let _ = cmd_tx_cta.send(UiCommand::RefreshAuth {
                            config: Box::new(oauth_config),
                            refresh_token: refresh_token.clone(),
                        });
                        cx.notify();
                        return;
                    }
                    let credential = match this.auth_draft.as_ref() {
                        Some(draft) if config.transport == Transport::Http => {
                            let user = draft.basic_username.read(cx).text().to_string();
                            let pw = draft.basic_password.read(cx).text().to_string();
                            let bearer = draft.bearer_token.read(cx).text().to_string();
                            let access =
                                draft.oauth_tokens.as_ref().map(|t| t.access_token.as_str());
                            credential_from(draft.method, &user, &pw, &bearer, access)
                        }
                        _ => Credential::default(),
                    };
                    let _ = cmd_tx_cta.send(UiCommand::Connect {
                        config,
                        credential: Box::new(credential),
                    });
                }
                cx.notify();
            }));

        let sidebar_w_val = px(sidebar_w);

        // Layout: prototype `.app` grid (028 Item #13 releitura). A single
        // header row [brand cell (sidebar width) | topbar] sits above a body
        // row [sidebar | main]. The brand and the topbar share one 60px height,
        // and the sidebar's vertical border runs unbroken from the brand cell
        // to the footer (the brand and topbar bottom borders form one line).
        v_flex()
            .id("app-root")
            .size_full()
            .bg(theme.background)
            // Light theme: a 1px top line separates the app from the OS title
            // bar (both are light surfaces and otherwise blend). Dark needs no
            // line — its dark bg already reads as separate (028 QA Item #15).
            .when(!theme.is_dark(), |root| {
                root.border_t_1().border_color(theme.border)
            })
            // --- Header row: brand cell + topbar (both 60px tall) ---
            .child(
                h_flex()
                    .id("app-header")
                    .h(px(60.))
                    .flex_none()
                    .child(
                        // Brand cell — sidebar width; continues the sidebar
                        // surface and its right/bottom borders (grid corner).
                        div()
                            .id("brand-cell")
                            .w(sidebar_w_val)
                            .min_w(sidebar_w_val)
                            .h_full()
                            .px(px(pad))
                            .flex()
                            .items_center()
                            .bg(theme.colors.sidebar)
                            .border_b_1()
                            .border_r_1()
                            .border_color(theme.border)
                            .child(
                                BrandHeader::new()
                                    .mark(brand_mark_icon())
                                    .name("MCP Explorer")
                                    .subtitle("MCP · local-first"),
                            ),
                    )
                    .child(topbar::render_topbar(
                        &self.state,
                        self.last_dispatched.as_ref(),
                        lang,
                        guided,
                        Some(on_lang_change),
                        Some(on_guided_toggle),
                        Some(on_reconnect),
                        Some(on_open_settings),
                        window,
                        cx,
                    )),
            )
            // --- Body row: sidebar + main column ---
            .child(
                h_flex()
                    .id("app-body")
                    .flex_1()
                    .min_h(px(0.))
                    .min_w(px(0.))
                    .child(
                        // Sidebar column (fixed width, no brand — it lives in
                        // the header row above).
                        v_flex()
                            .id("sidebar-col")
                            .w(sidebar_w_val)
                            .min_w(sidebar_w_val)
                            .h_full()
                            .min_h(px(0.))
                            .child({
                                if let Some(ref form) = self.sidebar_form {
                                    let current_method = self
                                        .auth_draft
                                        .as_ref()
                                        .map_or(AuthMethod::NoAuth, |d| d.method);
                                    let open_handler = if matches!(
                                        form.transport,
                                        crate::app::events::Transport::Http
                                    ) {
                                        Some(on_open_auth)
                                    } else {
                                        None
                                    };
                                    let env_open_handler = if matches!(
                                        form.transport,
                                        crate::app::events::Transport::Stdio
                                    ) {
                                        Some(on_open_env)
                                    } else {
                                        None
                                    };
                                    sidebar::render_sidebar(
                                        form,
                                        &self.state,
                                        &self.saved_servers,
                                        lang,
                                        guided,
                                        current_method,
                                        open_handler,
                                        transport_handlers,
                                        Some(on_connect),
                                        Some(on_disconnect),
                                        env_open_handler,
                                        on_pick_preset,
                                        window,
                                        cx,
                                    )
                                    .into_any_element()
                                } else {
                                    div().into_any_element()
                                }
                            }),
                    )
                    .child(
                        // Main column — tabbar (when connected) + bounded work
                        // area (§5b scroll chain). 1:1 with the prototype
                        // (`.main > .tabs`); the M5 §1.1 move into #sidebar-col
                        // was a misread (PRD §1.1 = the Autorização trigger, not
                        // the tabbar) and broke the layout — reverted (035 fix).
                        v_flex()
                            .id("main-column")
                            .flex_1()
                            .h_full()
                            .min_w(px(0.))
                            .min_h(px(0.))
                            .when(connected, |col| {
                                col.child(tabbar::render_tabbar(
                                    self.active_tab,
                                    &tab_counts,
                                    lang,
                                    tab_handlers,
                                ))
                            })
                            .child(div().flex().flex_col().flex_1().min_h(px(0.)).child(
                                work_area::render_work_area(
                                    &self.state,
                                    self.active_tab,
                                    self.last_dispatched.as_ref(),
                                    lang,
                                    Some(on_connect_work_area),
                                    self.capture_longtext,
                                    self.capture_tools_state.as_deref(),
                                    Some(tools),
                                    self.selected_tool.as_deref(),
                                    self.tool_filter_input.as_ref(),
                                    Some(&self.tool_params),
                                    guided,
                                    Some(on_select_tool.clone()),
                                    &self.tools_scroll,
                                    &self.tool_run,
                                    self.tool_validation.as_deref(),
                                    self.result_view,
                                    Some(on_run),
                                    Some(on_result_view_toggle),
                                    // Resources tab (M11)
                                    Some(&resources),
                                    Some(&templates),
                                    self.selected_resource_uri.as_deref(),
                                    self.selected_template_uri.as_deref(),
                                    &self.subscribed_resources,
                                    self.resource_filter_input.as_ref(),
                                    Some(&self.template_param_entities),
                                    &self.resource_read,
                                    self.capture_resources_state.as_deref(),
                                    Some(on_select_concrete.clone()),
                                    Some(on_select_template.clone()),
                                    Some(on_resource_read),
                                    Some(on_subscribe),
                                    Some(on_unsubscribe),
                                    &self.resources_scroll,
                                    // Prompts tab (M12)
                                    Some(&prompts),
                                    self.selected_prompt.as_deref(),
                                    self.prompt_filter_input.as_ref(),
                                    Some(&self.prompt_args),
                                    &self.prompt_run,
                                    &self.prompts_scroll,
                                    self.prompt_validation.as_deref(),
                                    self.capture_prompts_state.as_deref(),
                                    Some(on_select_prompt.clone()),
                                    Some(on_generate),
                                    // Notifications tab (M13)
                                    &self.logs,
                                    self.log_filter,
                                    &self.notifications_list,
                                    self.capture_notifications_state.as_deref(),
                                    on_filter_change,
                                    Some(on_clear_logs),
                                    // History tab (M14)
                                    &self.history,
                                    &self.history_list,
                                    self.capture_history_state.as_deref(),
                                    Some(on_clear_history),
                                    history_toggle_handlers,
                                    window,
                                    cx,
                                ),
                            )),
                    ),
            )
            // --- Settings overlay (M15) — scrim + elevated card (BUG-3) ---
            .when(self.settings_open, |root| {
                root.child(crate::screens::settings::render_settings(
                    &self.settings,
                    lang,
                    theme_handlers,
                    density_handlers,
                    primary_handlers,
                    on_settings_guided_toggle,
                    on_close_settings,
                    window,
                    cx,
                ))
            })
            // --- Auth panel overlay (M3) — floating panel to the right ---
            .when(self.auth_panel_open, |root| {
                if let Some(ref draft) = self.auth_draft {
                    root.child(crate::screens::auth_panel::render_auth_panel(
                        draft,
                        lang,
                        Some(on_auth_method_change.clone()),
                        on_authorize,
                        on_close_auth_x,
                        on_close_auth_save,
                        on_close_auth_outside,
                        window,
                        cx,
                    ))
                } else {
                    root.child(div())
                }
            })
            // --- Env panel overlay (036/M1) — floating panel to the right ---
            .when(self.env_panel_open, |root| {
                if let Some(ref f) = self.sidebar_form {
                    root.child(crate::screens::env_panel::render_env_panel(
                        f,
                        lang,
                        Some(on_add_env),
                        remove_handlers,
                        on_close_env_x,
                        on_close_env_save,
                        on_close_env_outside,
                        window,
                        cx,
                    ))
                } else {
                    root.child(div())
                }
            })
    }
}

fn capture_seed_logs(state: String) -> Vec<LogEntry> {
    use stand_in_mcp_explorer_ds::data::LogLevel;

    let time = "12:34:56.789";

    match state.as_str() {
        "empty" => vec![],
        "filtered-error" => vec![
            LogEntry {
                time: time.into(),
                level: LogLevel::Info,
                message: "conectando…".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Ok,
                message: "conectado a stand-in-reference (57ms)".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Error,
                message: "tool call error: timeout".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Ok,
                message: "tool call ok".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Error,
                message: "resource error: not found".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Info,
                message: "conexao encerrada".into(),
            },
        ],
        "large" => (0..80)
            .map(|i| {
                let (level, msg) = match i % 5 {
                    0 => (LogLevel::Info, format!("info message number {}", i)),
                    1 => (LogLevel::Ok, format!("ok operation {} completed", i)),
                    2 => (LogLevel::Warn, format!("warning condition {} detected", i)),
                    3 => (LogLevel::Error, format!("error {} occurred", i)),
                    _ => (LogLevel::Debug, format!("debug detail {}", i)),
                };
                LogEntry {
                    time: format!(
                        "{:02}:{:02}:{:02}.000",
                        12 + (i / 60),
                        (i / 60) % 60,
                        i % 60
                    ),
                    level,
                    message: msg,
                }
            })
            .collect(),
        _ => vec![
            LogEntry {
                time: time.into(),
                level: LogLevel::Info,
                message: "conectando…".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Ok,
                message: "conectado a stand-in-reference (57ms)".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Info,
                message: "inscrito file:///readme".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Warn,
                message: "transport closed".into(),
            },
            LogEntry {
                time: time.into(),
                level: LogLevel::Error,
                message: "tool call error: timeout".into(),
            },
        ],
    }
}

/// Synthetic History fixture for perf measurement (037 / O-024): `MCPX_PERF_N`
/// entries with varied request/response payloads so layout cost is realistic.
/// Bypasses `push_history`'s 200-entry cap (built directly).
#[cfg(feature = "perf")]
fn perf_history() -> Vec<HistoryEntry> {
    let n = crate::app::perf::perf_n();
    (0..n)
        .map(|i| {
            let kind = if i % 3 == 0 {
                HistKind::Prompt
            } else {
                HistKind::Tool
            };
            let has_error = i % 7 == 0;
            HistoryEntry {
                kind,
                name: format!("call_{i:06}"),
                request: serde_json::json!({ "index": i, "query": format!("synthetic request {i}") }),
                response: serde_json::json!({
                    "ok": !has_error,
                    "index": i,
                    "items": (0..(i % 5)).map(|j| format!("row_{j}")).collect::<Vec<_>>(),
                }),
                time: "12:34:56.789".into(),
                timing_ms: Some((i % 250) as u64),
                has_error,
                expanded: false,
            }
        })
        .collect()
}

fn capture_seed_history(state: String) -> Vec<HistoryEntry> {
    let time = "12:34:56.789";
    match state.as_str() {
        "empty" => vec![],
        #[cfg(feature = "perf")]
        "perf" => perf_history(),
        "expanded" => {
            let tool_req = serde_json::json!({"name": "World"});
            let tool_res =
                serde_json::json!({"greeting": "Hello, World!", "source": "stand-in-reference"});
            let prompt_req = serde_json::json!({"style": "formal"});
            let prompt_res = serde_json::json!({"messages": [
                {"role": "system", "content": {"type": "text", "text": "You are a helpful assistant."}},
                {"role": "user", "content": {"type": "text", "text": "Greet the user formally."}},
                {"role": "assistant", "content": {"type": "text", "text": "Good evening, esteemed user."}}
            ]});
            vec![
                HistoryEntry {
                    kind: HistKind::Tool,
                    name: "greet".into(),
                    request: tool_req,
                    response: tool_res,
                    time: time.into(),
                    timing_ms: Some(12),
                    has_error: false,
                    expanded: true,
                },
                HistoryEntry {
                    kind: HistKind::Prompt,
                    name: "greeting".into(),
                    request: prompt_req,
                    response: prompt_res,
                    time: time.into(),
                    timing_ms: Some(34),
                    has_error: false,
                    expanded: false,
                },
            ]
        }
        "longtext" => {
            let big = serde_json::json!({
                "rows": (0..30).map(|i| format!("row_{}", i)).collect::<Vec<_>>(),
                "nested": {"deep": {"deeper": {"deepest": (0..20).map(|i| format!("val_{}", i)).collect::<Vec<_>>()}}},
                "list": (0..10).map(|i| serde_json::json!({"id": i, "name": format!("item_{}", i), "active": i % 2 == 0})).collect::<Vec<_>>()
            });
            vec![HistoryEntry {
                kind: HistKind::Tool,
                name: "search".into(),
                request: serde_json::json!({"query": "large result set"}),
                response: big,
                time: time.into(),
                timing_ms: Some(150),
                has_error: false,
                expanded: true,
            }]
        }
        _ => {
            // "populated" — default fixture with 3 entries
            let e1 = HistoryEntry {
                kind: HistKind::Tool,
                name: "greet".into(),
                request: serde_json::json!({"name": "World"}),
                response: serde_json::json!({"greeting": "Hello, World!"}),
                time: "12:34:56.789".into(),
                timing_ms: Some(12),
                has_error: false,
                expanded: false,
            };
            let e2 = HistoryEntry {
                kind: HistKind::Prompt,
                name: "greeting".into(),
                request: serde_json::json!({"style": "formal"}),
                response: serde_json::json!({"messages": [
                    {"role": "user", "content": {"type": "text", "text": "Hello"}}
                ]}),
                time: "12:34:57.123".into(),
                timing_ms: Some(34),
                has_error: false,
                expanded: false,
            };
            let e3 = HistoryEntry {
                kind: HistKind::Tool,
                name: "add".into(),
                request: serde_json::json!({"a": 3, "b": 4}),
                response: serde_json::json!({"result": 7}),
                time: "12:35:01.456".into(),
                timing_ms: Some(5),
                has_error: false,
                expanded: false,
            };
            vec![e3, e2, e1]
        }
    }
}

fn capture_presets() -> Vec<ServerEntry> {
    vec![
        ServerEntry {
            name: "filesystem".into(),
            config: ConnConfig {
                transport: Transport::Stdio,
                command: "cargo".into(),
                args: vec![
                    "run".into(),
                    "--example".into(),
                    "filesystem".into(),
                    "--manifest-path".into(),
                    "stand-in-reference/Cargo.toml".into(),
                ],
                url: String::new(),
                env: Vec::new(),
            },
            auth: None,
        },
        ServerEntry {
            name: "git".into(),
            config: ConnConfig {
                transport: Transport::Stdio,
                command: "cargo".into(),
                args: vec![
                    "run".into(),
                    "--example".into(),
                    "git".into(),
                    "--manifest-path".into(),
                    "stand-in-reference/Cargo.toml".into(),
                ],
                url: String::new(),
                env: Vec::new(),
            },
            auth: None,
        },
        ServerEntry {
            name: "weather".into(),
            config: ConnConfig {
                transport: Transport::Stdio,
                command: "cargo".into(),
                args: vec![
                    "run".into(),
                    "--example".into(),
                    "weather".into(),
                    "--manifest-path".into(),
                    "stand-in-reference/Cargo.toml".into(),
                ],
                url: String::new(),
                env: Vec::new(),
            },
            auth: None,
        },
        ServerEntry {
            name: "everything".into(),
            config: ConnConfig {
                transport: Transport::Stdio,
                command: "cargo".into(),
                args: vec![
                    "run".into(),
                    "--example".into(),
                    "everything".into(),
                    "--manifest-path".into(),
                    "stand-in-reference/Cargo.toml".into(),
                ],
                url: String::new(),
                env: Vec::new(),
            },
            auth: None,
        },
    ]
}
