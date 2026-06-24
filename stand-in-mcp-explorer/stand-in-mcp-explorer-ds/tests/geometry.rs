//! Headless geometry assertions (TestAppContext) — the 5th-review spike.
//!
//! Catches the layout classes that crossed 17 green milestones in 025 and only
//! fell to the human eye (FE-G16 family: gpui `Div` defaults to display:block,
//! flex props inert, children stack; min-content squash). These tests render
//! REAL DS components headlessly and assert measured `Bounds` via
//! `debug_selector`/`debug_bounds` — no display, no capture.
//!
//! Harness notes (the spike's findings, mirrored in the gpui skill):
//! - Components carrying interactivity (hover/active/on_click) call
//!   `window.current_view()` during paint, so a raw `vcx.draw(element)` panics
//!   ("rendered_entity_stack empty") — the element must be drawn THROUGH a
//!   view. The `Probe` view below is that shim: it renders a content-hugging
//!   flex row tagged `debug_selector("probe")` around the element under test.
//! - The core idiom is the **differential row assertion**: adding an icon (or
//!   a count) to a row component must WIDEN it by at least the icon footprint.
//!   If the container regresses to display:block the children stack and the
//!   width stays put. Self-calibrating: both sides are real renders.
//! - Calibration was proven by intentionally removing `.flex()` from Badge —
//!   `badge_icon_widens_the_row` fails — then restoring it (a probe that never
//!   saw a failure proves nothing).

use gpui::{
    AnyElement, AnyView, App, AppContext as _, AvailableSpace, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, point, px, size,
};
use gpui_component::{ThemeMode, v_flex};
use stand_in_mcp_explorer_ds::core::{Badge, BadgeKind, Button, IconName};
use stand_in_mcp_explorer_ds::data::{LIST_ROW_HEIGHT, ListItem};
use stand_in_mcp_explorer_ds::navigation::{CapChip, TabItem, Tabbar};
use stand_in_mcp_explorer_ds::theme::density::Density;

/// Builder for the element under test, re-invoked on every render.
type ProbeBuilder = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// View shim: renders the element under test inside a content-hugging probe
/// row, so interactive components have a `current_view` during paint.
struct Probe(ProbeBuilder);

impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .debug_selector(|| "probe".into())
            .child((self.0)(window, cx))
    }
}

/// Bootstrap the DS inside a headless test window (theme + density + globals).
fn setup(cx: &mut TestAppContext) -> &mut VisualTestContext {
    let vcx = cx.add_empty_window();
    vcx.update(|_, cx| {
        stand_in_mcp_explorer_ds::init(cx);
        stand_in_mcp_explorer_ds::theme::apply_theme_and_density(
            ThemeMode::Dark,
            Density::Regular,
            cx,
        );
    });
    vcx
}

/// Draw `build()` through the Probe view and return the probe's hugged bounds
/// for `selector` ("probe" = the component's own footprint).
fn probe_bounds(
    vcx: &mut VisualTestContext,
    selector: &'static str,
    build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
) -> gpui::Bounds<gpui::Pixels> {
    let view = vcx.update(|_, cx| cx.new(|_| Probe(Box::new(build))));
    vcx.draw(
        point(px(0.), px(0.)),
        size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
        |_, _| AnyView::from(view),
    );
    vcx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("bounds not recorded for selector {selector}"))
}

fn probe_width(
    vcx: &mut VisualTestContext,
    build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
) -> f32 {
    probe_bounds(vcx, "probe", build).size.width.into()
}

/// FE-G16 (Badge): the icon must sit BESIDE the label — adding it widens the
/// chip by at least the icon footprint (12px icon + 5px gap). Under the
/// display:block regression the icon stacks above the label and the width
/// barely moves.
#[gpui::test]
fn badge_icon_widens_the_row(cx: &mut TestAppContext) {
    let vcx = setup(cx);
    let plain = probe_width(vcx, |_, _| {
        Badge::new("escrita", BadgeKind::Write)
            .id("b1")
            .into_any_element()
    });
    let with_icon = probe_width(vcx, |_, _| {
        Badge::new("escrita", BadgeKind::Write)
            .icon(IconName::Bolt)
            .id("b2")
            .into_any_element()
    });
    assert!(
        with_icon >= plain + 12.0,
        "icon must widen the badge row: plain={plain} with_icon={with_icon}"
    );
}

/// FE-G16 (CapChip): icon + count are row siblings of the label.
#[gpui::test]
fn cap_chip_icon_and_count_widen_the_row(cx: &mut TestAppContext) {
    let vcx = setup(cx);
    let plain = probe_width(vcx, |_, _| {
        CapChip::new("tools").id("c1").into_any_element()
    });
    let full = probe_width(vcx, |_, _| {
        CapChip::new("tools")
            .count(6)
            .icon(IconName::Tool)
            .id("c2")
            .into_any_element()
    });
    assert!(
        full >= plain + 12.0,
        "icon+count must widen the chip row: plain={plain} full={full}"
    );
}

/// FE-G16 (Button): the icon rides before the label on the same line.
#[gpui::test]
fn button_icon_widens_the_row(cx: &mut TestAppContext) {
    let vcx = setup(cx);
    let plain = probe_width(vcx, |_, _| {
        Button::new("Conectar").id("bt1").into_any_element()
    });
    let with_icon = probe_width(vcx, |_, _| {
        Button::new("Conectar")
            .icon(IconName::Play)
            .id("bt2")
            .into_any_element()
    });
    assert!(
        with_icon >= plain + 15.0,
        "icon must widen the button row: plain={plain} with_icon={with_icon}"
    );
}

/// FE-G16 (Tabbar): icon + count are row siblings inside the tab.
#[gpui::test]
fn tabbar_tab_lays_children_in_a_row(cx: &mut TestAppContext) {
    let vcx = setup(cx);
    let plain = probe_width(vcx, |_, _| {
        Tabbar::new("t1", vec![TabItem::new("tools", "Tools")], 0).into_any_element()
    });
    let full = probe_width(vcx, |_, _| {
        Tabbar::new(
            "t2",
            vec![TabItem::new("tools", "Tools").icon(IconName::Tool).count(6)],
            0,
        )
        .into_any_element()
    });
    assert!(
        full >= plain + 15.0,
        "icon+count must widen the tab row: plain={plain} full={full}"
    );
}

/// BUG-10/§5b (min-content squash): a shrinkable child (`min_h(0)`, the
/// semantics a scroll container imposes) of a height-limited flex column is
/// squashed BEFORE overflow scrolls; `flex_none` keeps the natural height.
/// This is the harness recipe that guards the gallery's `section_body()` rule.
#[gpui::test]
fn flex_none_defeats_min_content_squash(cx: &mut TestAppContext) {
    let vcx = setup(cx);

    let measure = |vcx: &mut VisualTestContext, protected: bool| -> f32 {
        let sel: &'static str = if protected {
            "body-flex-none"
        } else {
            "body-shrinkable"
        };
        let b = probe_bounds(vcx, sel, move |_, _| {
            let mut body = v_flex().w(px(100.)).debug_selector(|| sel.into());
            body = if protected {
                body.flex_none()
            } else {
                body.min_h(px(0.))
            };
            for _ in 0..3 {
                body = body.child(div().h(px(60.)).w(px(100.)));
            }
            // Height-limited column (the scroll container in real code).
            v_flex()
                .h(px(120.))
                .w(px(100.))
                .child(body)
                .into_any_element()
        });
        b.size.height.into()
    };

    let squashed = measure(vcx, false);
    let natural = measure(vcx, true);
    assert!(
        squashed <= 120.0,
        "shrinkable body should be squashed to the container: {squashed}"
    );
    assert!(
        (natural - 180.0).abs() < 1.0,
        "flex_none body must keep its natural 180px height: {natural}"
    );
}

/// 031/M1: every ListItem variant renders at the same height == LIST_ROW_HEIGHT
/// — the prerequisite for `gpui::uniform_list` windowing. Under the old
/// variable-height regime lines without `desc` would be shorter and the test
/// fails (self-calibrating: real renders against the const).
#[gpui::test]
fn list_item_rows_are_uniform_height(cx: &mut TestAppContext) {
    let vcx = setup(cx);
    let h = |vcx: &mut VisualTestContext, build: fn(&mut Window, &mut App) -> AnyElement| {
        let height: f32 = probe_bounds(vcx, "probe", build).size.height.into();
        height
    };
    // (a) with 2-line desc + badge — the canonical tall case
    let two_line = h(vcx, |_, _| {
        ListItem::new("a", "a")
            .desc("Linha 1\nLinha 2")
            .badge(Badge::new("leitura", BadgeKind::Read).into_any_element())
            .into_any_element()
    });
    // (b) no desc + badge
    let no_desc = h(vcx, |_, _| {
        ListItem::new("b", "b")
            .badge(Badge::new("leitura", BadgeKind::Read).into_any_element())
            .into_any_element()
    });
    // (c) long desc (3+ lines, clamped) + badge
    let long = h(vcx, |_, _| {
        ListItem::new("c", "c")
            .desc("Linha 1\nLinha 2\nLinha 3\nLinha 4")
            .badge(Badge::new("leitura", BadgeKind::Read).into_any_element())
            .into_any_element()
    });
    // (d) short desc, no badge
    let no_badge = h(vcx, |_, _| {
        ListItem::new("d", "d")
            .desc("Uma linha só")
            .into_any_element()
    });
    for (name, got) in [("no_desc", no_desc), ("long", long), ("no_badge", no_badge)] {
        assert!(
            (got - two_line).abs() < 1.0,
            "{name} height {got} != two_line {two_line}"
        );
    }
    assert!(
        (two_line - LIST_ROW_HEIGHT).abs() < 1.0,
        "row height {two_line} != const {LIST_ROW_HEIGHT}"
    );
}
