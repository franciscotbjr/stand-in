//! Gallery shell — sidebar index + display area for the Storybook.
//!
//! In capture mode the sidebar and toolbar are hidden and only the requested
//! section's content is rendered deterministically (the visual-gate target).
//! In interactive mode the sidebar is visible with a clickable section index
//! and a toolbar (dark/light toggle + density selector) sits above the display.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::args::Args;
use crate::sections::{self, Section};
use gpui::prelude::FluentBuilder;
use gpui::{
    AppContext, ClickEvent, Entity, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, div,
};
use gpui_component::ThemeMode;
use gpui_component::input::InputState;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use stand_in_mcp_explorer_ds::prelude;
use stand_in_mcp_explorer_ds::theme::density::{Density, GlobalDensity};
use stand_in_mcp_explorer_ds::theme::{self, typography};

/// Global demo-clicks counter (readable during render without re-entrant lock).
#[derive(Debug, Default)]
pub struct GlobalDemoClicks(AtomicUsize);

impl gpui::Global for GlobalDemoClicks {}

impl GlobalDemoClicks {
    pub fn load(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }

    pub fn inc(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct GalleryShell {
    args: Args,
    sections: Vec<Section>,
    selected_index: usize,
    focus: FocusHandle,
    current_mode: ThemeMode,
    current_density: Density,
    /// Lazy-initialised input states for the Forms section (M7).
    pub cmd_input: Option<Entity<InputState>>,
    pub args_input: Option<Entity<InputState>>,
    pub err_input: Option<Entity<InputState>>,
    pub prose_input: Option<Entity<InputState>>,
    /// M8: SegmentedControl demo — selected transport (0=STDIO, 1=SSE, 2=HTTP).
    pub selected_transport: usize,
    /// M8: KeyValueRow env vars — parallel vecs (key, value, stable row id).
    pub env_var_keys: Vec<Entity<InputState>>,
    pub env_var_values: Vec<Entity<InputState>>,
    pub env_var_row_ids: Vec<u64>,
    pub next_env_row_id: u64,
    /// Whether the M8 env var states have been seeded.
    env_vars_seeded: bool,
    /// Badges-section ToggleLink demo — how many demo rows were added (the
    /// click must produce a VISIBLE result, not only the counter).
    pub demo_vars_added: usize,
    /// M9: Select demo — selected language index (0=pt, 1=en, 2=es).
    pub selected_lang: usize,
    /// M11: Tabbar demo — active tab index (0=tools, 1=resources, 2=history).
    pub active_tab: usize,
    /// M13: ListItem demo — selected item index (usize::MAX = none).
    pub selected_data_item: usize,
    /// M14: ListSearch demo — search input state (filters ListItems live).
    pub search_input: Option<Entity<InputState>>,
    /// M14: PresetCard demo — selected preset index (usize::MAX = none).
    pub selected_preset: usize,
}

impl GalleryShell {
    pub fn new(args: Args, cx: &mut gpui::Context<Self>) -> Self {
        let mode = match args.mode.as_str() {
            "light" => ThemeMode::Light,
            _ => ThemeMode::Dark,
        };
        cx.set_global(GlobalDemoClicks::default());
        Self {
            args,
            sections: sections::registry(),
            selected_index: 0,
            focus: cx.focus_handle(),
            current_mode: mode,
            current_density: Density::Regular,
            cmd_input: None,
            args_input: None,
            err_input: None,
            prose_input: None,
            selected_transport: 0,
            env_var_keys: Vec::new(),
            env_var_values: Vec::new(),
            env_var_row_ids: Vec::new(),
            next_env_row_id: 0,
            env_vars_seeded: false,
            demo_vars_added: 0,
            selected_lang: 0,
            active_tab: 0,
            selected_data_item: usize::MAX,
            search_input: None,
            selected_preset: usize::MAX,
        }
    }

    fn section_labels(&self) -> Vec<SharedString> {
        let mut labels: Vec<SharedString> = vec!["shell".into()];
        for s in &self.sections {
            labels.push(s.label.clone());
        }
        labels
    }

    fn apply_theme_now(&self, cx: &mut gpui::Context<Self>) {
        theme::apply_theme_and_density(self.current_mode, self.current_density, cx);
    }

    /// Whether the gallery is running in deterministic capture mode.
    pub fn is_capture(&self) -> bool {
        self.args.capture
    }

    /// Create the Form input states lazily on the first render.
    fn ensure_inputs(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        if self.cmd_input.is_some() {
            self.seed_env_vars(window, cx);
            return;
        }
        let is_cap = self.is_capture();
        self.cmd_input = Some(cx.new(|cx| {
            let mut s = InputState::new(window, cx).placeholder("npx");
            if is_cap {
                s = s.default_value("npx");
            }
            s
        }));
        self.args_input = Some(cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("args")
                .multi_line(true)
                .rows(3)
                .soft_wrap(true)
        }));
        self.err_input = Some(cx.new(|cx| InputState::new(window, cx).default_value("99999")));
        self.prose_input = Some(cx.new(|cx| {
            InputState::new(window, cx).placeholder("Free-form human-readable description")
        }));
        self.search_input = Some(cx.new(|cx| {
            let mut s = InputState::new(window, cx).placeholder("Filtrar tools\u{2026}");
            if is_cap {
                s = s.default_value("re");
            }
            s
        }));
        self.seed_env_vars(window, cx);
    }

    /// Seed 2 env-var pairs (capture mode) or leave empty (live mode).
    fn seed_env_vars(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        if self.env_vars_seeded {
            return;
        }
        self.env_vars_seeded = true;
        if self.is_capture() {
            let pairs = [
                ("NODE_ENV", "production"),
                ("DATABASE_URL", "postgres://localhost:5432"),
            ];
            for (key, val) in &pairs {
                let id = self.next_env_row_id;
                self.next_env_row_id += 1;
                self.env_var_row_ids.push(id);
                self.env_var_keys.push(cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder("CHAVE")
                        .default_value(*key)
                }));
                self.env_var_values.push(cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder("valor")
                        .default_value(*val)
                }));
            }
        }
    }

    fn render_sidebar(&self, cx: &mut gpui::Context<Self>) -> impl IntoElement + use<> {
        let t = cx.theme();
        v_flex()
            .w(prelude::px(264.))
            .flex_none()
            .h_full()
            .bg(t.sidebar)
            .border_r_1()
            .border_color(t.border)
            .child(
                v_flex().px_3().py_2().child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(t.muted_foreground)
                        .child("Gallery"),
                ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h(prelude::px(0.))
                    .overflow_y_scrollbar()
                    .children(
                        self.section_labels()
                            .into_iter()
                            .enumerate()
                            .map(|(i, label)| {
                                let selected = i == self.selected_index;
                                h_flex()
                                    .id(("section", i))
                                    .px_3()
                                    .py_2()
                                    .gap_2()
                                    .cursor_pointer()
                                    .when(selected, |s| s.bg(t.list_active))
                                    .hover(|h| if !selected { h.bg(t.list_hover) } else { h })
                                    .on_click(cx.listener(
                                        move |this, _: &ClickEvent, _window, cx| {
                                            this.selected_index = i;
                                            cx.notify();
                                        },
                                    ))
                                    .child(
                                        // flex_1 + min_w(0) give the label a definite
                                        // width so long compound names wrap instead of
                                        // clipping at the sidebar border.
                                        div()
                                            .flex_1()
                                            .min_w(prelude::px(0.))
                                            .text_sm()
                                            .text_color(if selected {
                                                t.foreground
                                            } else {
                                                t.muted_foreground
                                            })
                                            .font_weight(if selected {
                                                FontWeight::MEDIUM
                                            } else {
                                                FontWeight::NORMAL
                                            })
                                            .child(label.clone()),
                                    )
                            }),
                    ),
            )
    }

    fn render_toolbar(&self, cx: &mut gpui::Context<Self>) -> impl IntoElement + use<> {
        let t = cx.theme();
        let density_labels = ["Compact", "Regular", "Comfy"];
        let mode_label = if self.current_mode == ThemeMode::Dark {
            "Light"
        } else {
            "Dark"
        };
        h_flex()
            .w_full()
            .px_3()
            .py_2()
            .gap_3()
            .bg(t.sidebar)
            .border_b_1()
            .border_color(t.border)
            .children(density_labels.iter().enumerate().map(|(i, &label)| {
                let active = matches!(
                    (i, self.current_density),
                    (0, Density::Compact) | (1, Density::Regular) | (2, Density::Comfy)
                );
                div()
                    .id(("density", i))
                    .px_2()
                    .py_1()
                    .text_sm()
                    .font_family(t.mono_font_family.clone())
                    .text_color(if active {
                        t.foreground
                    } else {
                        t.muted_foreground
                    })
                    .font_weight(if active {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    })
                    .cursor_pointer()
                    .when(active, |s| {
                        s.bg(t.secondary).rounded(prelude::px(typography::FS_XS))
                    })
                    .hover(|h| {
                        if !active {
                            h.bg(t.list_hover).rounded(prelude::px(typography::FS_XS))
                        } else {
                            h
                        }
                    })
                    .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                        let d = match i {
                            0 => Density::Compact,
                            1 => Density::Regular,
                            _ => Density::Comfy,
                        };
                        this.current_density = d;
                        cx.set_global(GlobalDensity::new(d));
                        cx.notify();
                    }))
                    .child(label)
            }))
            .child(div().flex_1())
            .child(
                div()
                    .id("theme-toggle")
                    .px_2()
                    .py_1()
                    .text_sm()
                    .font_family(t.mono_font_family.clone())
                    .text_color(t.muted_foreground)
                    .cursor_pointer()
                    .hover(|h| h.bg(t.list_hover).rounded(prelude::px(typography::FS_XS)))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                        this.current_mode = if this.current_mode == ThemeMode::Dark {
                            ThemeMode::Light
                        } else {
                            ThemeMode::Dark
                        };
                        this.apply_theme_now(cx);
                        cx.notify();
                    }))
                    .child(format!("Mode: {}", mode_label)),
            )
    }

    fn render_placeholder(&self, cx: &mut gpui::Context<Self>) -> impl IntoElement + use<> {
        let t = cx.theme();
        v_flex()
            .flex_1()
            .min_w(prelude::px(0.))
            .h_full()
            .items_center()
            .justify_center()
            .bg(t.background)
            .child(
                v_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(t.muted_foreground)
                            .font_family(t.mono_font_family.clone())
                            .child("Design System"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(t.muted_foreground)
                            .child("sections appear here as milestones land"),
                    ),
            )
    }

    fn render_display(&self, cx: &mut gpui::Context<Self>) -> impl IntoElement + use<> {
        let section_id: Option<&str> = if self.args.capture {
            Some(&self.args.section)
        } else if self.selected_index == 0 {
            Some("shell")
        } else {
            self.sections
                .get(self.selected_index - 1)
                .map(|s| s.id.as_ref())
        };

        if let Some(id) = section_id {
            if id == "shell" {
                return self.render_placeholder(cx).into_any_element();
            }
            for s in &self.sections {
                if s.id.as_ref() == id {
                    return (s.render)(&self.args.state, &self.args.mode, self, cx);
                }
            }
        }
        self.render_placeholder(cx).into_any_element()
    }
}

impl Render for GalleryShell {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        self.ensure_inputs(window, cx);
        let t = cx.theme();
        if self.args.capture {
            h_flex()
                .track_focus(&self.focus)
                .size_full()
                .bg(t.background)
                .text_color(t.foreground)
                .font_family(t.font_family.clone())
                .child(self.render_display(cx))
                .into_any_element()
        } else {
            v_flex()
                .track_focus(&self.focus)
                .size_full()
                .bg(t.background)
                .text_color(t.foreground)
                .font_family(t.font_family.clone())
                .child(self.render_toolbar(cx))
                .child(
                    h_flex()
                        .flex_1()
                        .min_h(prelude::px(0.))
                        .child(self.render_sidebar(cx))
                        .child(self.render_display(cx)),
                )
                .into_any_element()
        }
    }
}
