//! SidebarShell — casca genérica de 3 zonas (brand fixo / corpo rolável /
//! rodapé fixo) para a barra lateral.
//!
//! 1:1 com `navigation/SidebarShell.prompt.md` + `.side`/`.brand`/
//! `.brand-mark`/`.side-scroll`/`.side-foot` em `navigation/navigation.css`.
//! **Regra central:** é uma CASCA genérica — o interior é do projeto
//! consumidor. O DS não define "Sidebar do MCP Explorer"; fornece a estrutura.
//!
//! Anatomy:
//! - Outer: coluna, bg sidebar, borda direita 1px, h_full, min_h(0).
//!   **SEM width própria** — o caller ou grid a define (rustdoc).
//! - Brand (fixo): h_flex, gap 11, padding 18/pad/16, border-bottom 1px.
//!   Brand-mark 34×34: raio RADIUS_BTN (9), o gradiente legítimo
//!   (150deg oby→genipina 60%→yandi — proibição 5 permite exatamente este),
//!   inset ring 1px branco 8% (realizado como border sutil), ícone centralizado
//!   em on-primary. Texto: nome (fs-xl 15, peso 700, nowrap) + sub (fs-xs,
//!   text-3, mt 3). min_w(0) na coluna de texto.
//! - Corpo (rolável): flex_1, min_h(0), overflow_y_scrollbar, padding pad
//!   (densidade!), coluna gap 18. Children = seções do caller (cada uma abrindo
//!   com SectionLabel — convenção, rustdoc).
//! - Rodapé (fixo): flex_none; slot opcional (callouts persistentes, nunca
//!   navegação; rustdoc).
//!
//! API:
//! ```ignore
//! use stand_in_mcp_explorer_ds::navigation::SidebarShell;
//!
//! SidebarShell::new()
//!     .brand_mark(Icon::new(IconName::Leaf).with_px(px(18.)))
//!     .brand_name("MCP Explorer")
//!     .brand_sub("MCP \u{b7} local-first")
//!     .children([...])
//!     .footer(footer_element);
//! ```

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, px,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme as _, v_flex};

use super::brand_header::BrandHeader;
use crate::theme::density::GlobalDensity;

// ---------------------------------------------------------------------------
// SidebarShell
// ---------------------------------------------------------------------------

/// Casca genérica da barra lateral com 3 zonas: brand fixo, corpo rolável
/// e rodapé fixo. O DS fornece a estrutura; cada projeto decide o conteúdo.
///
/// A largura é definida pelo caller ou pelo grid do app (ex.: 304px regular,
/// 280px compact via `Density::sidebar_w()` na 026) — a casca NÃO se
/// auto-dimensiona.
#[derive(IntoElement)]
pub struct SidebarShell {
    brand_mark: Option<AnyElement>,
    brand_name: SharedString,
    brand_sub: Option<SharedString>,
    show_brand: bool,
    footer: Option<AnyElement>,
    body_children: Vec<AnyElement>,
    id: ElementId,
}

impl SidebarShell {
    /// Cria uma casca de sidebar vazia. Chame os setters antes de renderizar.
    pub fn new() -> Self {
        Self {
            brand_mark: None,
            brand_name: "".into(),
            brand_sub: None,
            show_brand: true,
            footer: None,
            body_children: Vec::new(),
            id: ElementId::from("sidebar-shell"),
        }
    }

    /// Elemento a ser exibido dentro do brand-mark 34×34 (tipicamente um
    /// Icon 18px). O gradiente e o inset ring são aplicados pela casca;
    /// o slot herda `on-primary` como cor.
    pub fn brand_mark(mut self, el: impl IntoElement) -> Self {
        self.brand_mark = Some(el.into_any_element());
        self
    }

    /// Nome da marca (fs-xl 15, peso 700, nowrap). Obrigatório para que
    /// a zona do brand seja renderizada.
    pub fn brand_name(mut self, name: impl Into<SharedString>) -> Self {
        self.brand_name = name.into();
        self
    }

    /// Subtítulo abaixo do nome (fs-xs, text-3). Opcional.
    pub fn brand_sub(mut self, sub: impl Into<SharedString>) -> Self {
        self.brand_sub = Some(sub.into());
        self
    }

    /// Renderiza (ou não) a zona do brand no topo da sidebar. Default `true`.
    /// `false` quando o brand vive no header row do app (grid 2×2 — 028 Item
    /// #13 releitura): a sidebar fica só com `side-scroll` + footer.
    pub fn show_brand(mut self, show: bool) -> Self {
        self.show_brand = show;
        self
    }

    /// Elemento fixo no rodapé da sidebar (callouts persistentes, nunca
    /// navegação). Opcional — se não definido, o rodapé não é renderizado.
    pub fn footer(mut self, el: impl IntoElement) -> Self {
        self.footer = Some(el.into_any_element());
        self
    }

    /// Children do corpo rolável. O caller deve organizar em seções que
    /// abrem com SectionLabel (convenção documentada no rustdoc).
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.body_children = children.into_iter().map(|c| c.into_any_element()).collect();
        self
    }

    /// Override do element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl Default for SidebarShell {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for SidebarShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme().clone();
        let density = cx.global::<GlobalDensity>().0;
        let pad = density.pad();

        // --- Scroll body (Dialog pattern, §5b) ---
        // The leaf (size_full + overflow_y_scrollbar) MUST sit inside its own
        // flex_1 + overflow_hidden wrapper — sibling of the fixed brand/footer.
        // `overflow_y_scrollbar()` re-wraps the leaf in a `size_full` div that
        // inherits only `size` (not flex_1/min_h), so the leaf cannot itself be
        // the flex child sharing the column with brand/footer — it would take
        // 100% height, ignore them, and push content off-screen instead of
        // scrolling (028 QA Item #20; the long-latent sidebar scroll bug).
        let scroll_body = div()
            .id("side-scroll-wrap")
            .flex_1()
            .min_h(px(0.))
            .overflow_hidden()
            .child(
                div()
                    .id("side-scroll")
                    .flex()
                    .flex_col()
                    .size_full()
                    .overflow_y_scrollbar()
                    .p(px(pad))
                    .gap(px(18.))
                    .children(self.body_children),
            );

        // --- Outer shell ---
        let mut shell = v_flex()
            .id(self.id)
            .h_full()
            .min_h(px(0.))
            .bg(t.colors.sidebar)
            .border_r_1()
            .border_color(t.sidebar_border);

        // --- Brand zone (fixed, optional) ---
        // The brand-mark gradient is owned by `BrandHeader` (DS) — never
        // duplicated. When `show_brand` is false the brand lives in the app
        // header row (grid 2×2 — 028 Item #13 releitura) and the sidebar is
        // just scroll + footer.
        if self.show_brand {
            let mut brand = BrandHeader::new().name(self.brand_name.clone());
            if let Some(mark) = self.brand_mark {
                brand = brand.mark(mark);
            }
            if let Some(sub) = self.brand_sub.clone() {
                brand = brand.subtitle(sub);
            }
            shell = shell.child(
                div()
                    .id("brand-zone")
                    .flex_none()
                    .pt(px(18.))
                    .pb(px(16.))
                    .px(px(pad))
                    .border_b_1()
                    .border_color(t.border)
                    .child(brand),
            );
        }

        shell = shell.child(scroll_body);

        // --- Footer (fixed, optional) ---
        if let Some(footer) = self.footer {
            shell = shell.child(div().id("side-foot").flex_none().child(footer));
        }

        shell
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::icon::{Icon, IconName};

    #[test]
    fn test_new_defaults() {
        let shell = SidebarShell::new();
        assert!(shell.brand_mark.is_none());
        assert!(shell.brand_name.is_empty());
        assert!(shell.brand_sub.is_none());
        assert!(shell.footer.is_none());
        assert!(shell.body_children.is_empty());
        assert_eq!(shell.id, ElementId::from("sidebar-shell"));
    }

    #[test]
    fn test_default_trait() {
        let shell = SidebarShell::default();
        assert!(shell.brand_name.is_empty());
        assert!(shell.body_children.is_empty());
    }

    #[test]
    fn test_brand_name_setter() {
        let shell = SidebarShell::new().brand_name("MCP Explorer");
        assert_eq!(shell.brand_name.as_ref(), "MCP Explorer");
    }

    #[test]
    fn test_brand_sub_setter() {
        let shell = SidebarShell::new().brand_sub("MCP · local-first");
        assert_eq!(shell.brand_sub.as_deref(), Some("MCP · local-first"));
    }

    #[test]
    fn test_brand_mark_presence() {
        let icon = Icon::new(IconName::Leaf).with_px(px(18.));
        let shell = SidebarShell::new().brand_mark(icon);
        assert!(shell.brand_mark.is_some());
    }

    #[test]
    fn test_footer_presence() {
        let el = div().child("privacy");
        let shell = SidebarShell::new().footer(el);
        assert!(shell.footer.is_some());
    }

    #[test]
    fn test_children_populated() {
        let shell = SidebarShell::new().children([div().child("a"), div().child("b")]);
        assert_eq!(shell.body_children.len(), 2);
    }

    #[test]
    fn test_children_empty_by_default() {
        let shell = SidebarShell::new();
        assert!(shell.body_children.is_empty());
    }

    #[test]
    fn test_id_override() {
        let shell = SidebarShell::new().id("my-sidebar");
        assert_eq!(shell.id, ElementId::from("my-sidebar"));
    }

    #[test]
    fn test_brand_sub_none_when_not_set() {
        let shell = SidebarShell::new().brand_name("X");
        assert!(shell.brand_sub.is_none());
    }

    #[test]
    fn test_no_footer_by_default() {
        let shell = SidebarShell::new();
        assert!(shell.footer.is_none());
    }

    #[test]
    fn test_full_builder_chain() {
        let shell = SidebarShell::new()
            .brand_mark(Icon::new(IconName::Leaf).with_px(px(18.)))
            .brand_name("MCP Explorer")
            .brand_sub("MCP · local-first")
            .children([div()])
            .footer(div().child("callout"))
            .id("test-shell");
        assert!(shell.brand_mark.is_some());
        assert_eq!(shell.brand_name.as_ref(), "MCP Explorer");
        assert_eq!(shell.brand_sub.as_deref(), Some("MCP · local-first"));
        assert_eq!(shell.body_children.len(), 1);
        assert!(shell.footer.is_some());
        assert_eq!(shell.id, ElementId::from("test-shell"));
    }

    #[test]
    fn test_show_brand_default_true() {
        assert!(SidebarShell::new().show_brand);
    }

    #[test]
    fn test_show_brand_false() {
        assert!(!SidebarShell::new().show_brand(false).show_brand);
    }
}
