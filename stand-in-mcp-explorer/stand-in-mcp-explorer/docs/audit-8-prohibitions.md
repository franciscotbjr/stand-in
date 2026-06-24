# Audit: 8 Design-System Prohibitions

> M17 deliverable — the app (`stand-in-mcp-explorer`) audited against the 8 absolute prohibitions
> of the [canonical Design System](../../../../stand-in-client-prototipo/readme.md).
> Verdicts backed by code scans against `stand-in-mcp-explorer/src/` (2026-06-13).

## 1. Tokens-only — never a literal colour

**Verdict: PASS**

- Zero `hsla(` or `rgb(` literal in app render code.
- `settings.rs` calls `PrimaryChoice::to_hsla()` — but the values (`JANDI`, `GENIPINA`, `OBY`) are re-exported DS constants, not inline literals. This is the legitimate Settings primary-choice feature.
- All rendering goes through `cx.theme().colors.*` or `JandiExt` helpers.
- The one legitimate transparent literal (`hsla(0., 0., 0., 0.)`) is confined to the DS crate.

## 2. Two fonts only — Hanken Grotesk + JetBrains Mono

**Verdict: PASS**

- Fonts are loaded via `DsAssets::init(cx)` (the DS crate embeds both).
- The app never calls a raw font API. Mono/sans routing is handled by DS components (e.g. `CodeView` uses mono, `ListItem` labels use sans).

## 3. On-catalog icons only (22 glyphs)

**Verdict: PASS**

- All `IconName::*` references in app source trace to the DS icon catalog: `Play`, `Plug`, `Tool`, `Doc`, `Chat`, `Bell`, `History`, `X`, `Chevron`, `Bolt`, `Lock`, `Leaf`, `File`, `Info`, `Refresh`, `Check`.
- Zero external icon libraries. Zero emoji. Zero decorative Unicode.

## 4. No shadow on in-flow cards (DS-flat)

**Verdict: PASS** (1 legitimate exception)

- **The only `shadow_*` call in the entire app** is `screens/settings.rs:88` → `.shadow_lg()` on the Settings overlay. This is explicitly legitimate: overlays are the single surface class that carries a shadow per the DS elevation rule (BUG-3). The overlay also has an elevated surface + scrim.
- Zero other `shadow_*` usage across all 51 source files.

## 5. No decorative gradients

**Verdict: PASS**

- Zero gradient calls in app source.
- The comment in `brand_header.rs:2` notes *"the shell already provides the gradient + ring"* — this references the DS `SidebarShell`'s embedded brand-mark gradient, a legitimate DS gradient (one of the system's two allowed gradients).

## 6. Mono/sans routing

**Verdict: PASS**

- Identifiers, paths, JSON, timestamps, counters, and badges render in mono. Prose renders in sans.
- This routing is enforced by DS components (e.g. `ListItem`'s metadata slot is mono; `CodeView`/`JsonView` are mono). The app does not override font families.

## 7. Density-driven values

**Verdict: PASS**

- Spacing, row heights, gaps, font sizes, and radii are driven by the DS `GlobalDensity` + `apply_theme(cx)`.
- The app never hardcodes a sizing value that should be density-controlled.

## 8. Reuse before create — the app composes the DS

**Verdict: PASS**

- The app **composes** the DS (`stand-in-mcp-explorer-ds`, path dep) — it never re-implements a control.
- `git log` confirms the DS crate was touched **only** by the M2 commit (`5fd34c1`, O-006 contrast fix), a deliberate design-canon update per iteration decision D-4. All 16 other 028 milestones left the DS crate untouched.
- No DS-extension proposals were needed during the 028 build; the 24-component DS catalogue covered every UI need.

---

## Summary

| Prohibition | Verdict |
|------------|---------|
| 1. Tokens-only | PASS |
| 2. Two fonts | PASS |
| 3. Icon catalog | PASS |
| 4. No in-flow shadow | PASS (1 legitimate overlay exception) |
| 5. No decorative gradients | PASS |
| 6. Mono/sans routing | PASS |
| 7. Density-driven | PASS |
| 8. Reuse before create | PASS (DS touched only by M2 canon update) |

**Overall: 8/8 PASS. Zero violations in the app.**
