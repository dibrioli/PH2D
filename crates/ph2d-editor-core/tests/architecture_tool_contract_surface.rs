//! Arch-gate: the substrate's contracts that *external tool crates implement*
//! must stay tiny. Every `ph2d-tool-*` crate implements `Tool` (and, for raster-
//! edit tools, the sub-trait `RasterEditTool`). If these grow, the change ripples
//! to every tool in the workspace and re-serializes the multi-agent fan-out
//! (the failure mode ADR-0040 was built to avoid).
//!
//! Mirrors `ph2d-nodegraph`'s `architecture_contract_surface` — the node-system
//! freeze under ADR-0039 — and applies the same discipline to tools.
//!
//! **FROZEN at ADR-0040 TG-E (2026-05-22), amended at ADR-0041 (2026-05-23):**
//! the caps below are pinned to the *current* surface (no headroom), so ANY
//! addition to the tool-implemented contract now trips this gate and forces a
//! conscious cap bump + ADR amendment. This is what makes the freeze bite —
//! the fan-out builds against a fixed contract.
//!
//! How to raise a cap: bump the number here *and justify it in review*. A
//! contract change is a rare, Coordenador-only event (the freeze discipline).

/// Count `fn ` declarations inside the body of `trait_decl` (up to its first
/// closing `\n}`). Doc-comment lines (`///`) are skipped so that prose
/// containing `fn` (e.g. an `// fn foo() {...}` example in a default impl's
/// docstring) does not inflate the count. Method signatures end in `;` or
/// `{ ... }`; the first `\n}` after the declaration is the trait's own close.
fn trait_method_count(src: &str, trait_decl: &str) -> usize {
    let start = src
        .find(trait_decl)
        .unwrap_or_else(|| panic!("trait declaration {trait_decl:?} present"));
    let rest = &src[start..];
    let end = rest.find("\n}").expect("trait body closes");
    rest[..end]
        .lines()
        .filter(|l| !l.trim_start().starts_with("///"))
        .filter(|l| l.contains("fn "))
        .count()
}

#[test]
fn tool_contract_is_capped() {
    let src = include_str!("../src/tool.rs");
    let n = trait_method_count(src, "pub trait Tool");
    assert!(
        n <= 12,
        "Tool has {n} methods; cap is 12. FROZEN at ADR-0040 TG-E to the \
         current surface (id / label / icon_slug / build_panel / on_activate / \
         on_deactivate / handle_panel_event / on_tick / as_any_mut / \
         as_raster_edit_mut / as_canvas_paint_mut / is_default) — every \
         `ph2d-tool-*` crate implements this, so any growth ripples the whole \
         fan-out. Adding a method is a Coordenador-only contract change: bump \
         the cap here + write the ADR amendment. Cap 10→11 at \
         ADR-0040-amendment-2 (2026-06-07): `on_tick` per-frame heartbeat for \
         the ADR-0049 watercolor live diffusion. Cap 11→12 at \
         ADR-0040-amendment-3 (2026-06-20): `as_canvas_paint_mut` capability \
         upcast for canvas pointer delivery to the new Painter brush (a default \
         `None`, so existing tool impls are unaffected)."
    );
}

#[test]
fn raster_edit_tool_contract_is_capped() {
    let src = include_str!("../src/tool.rs");
    let n = trait_method_count(src, "pub trait RasterEditTool");
    assert!(
        n <= 5,
        "RasterEditTool has {n} methods; cap is 5. FROZEN at ADR-0040 TG-E \
         + ADR-0041 to the current surface (set_source / current_preview / \
         take_pending_commit / run_full / deactivate) — every raster-edit \
         tool crate implements this on top of `Tool`. Adding a method \
         ripples to every raster tool. A Coordenador-only contract change: \
         bump the cap here + write the ADR-0040/0041 amendment."
    );
}

#[test]
fn canvas_paint_tool_contract_is_capped() {
    let src = include_str!("../src/tool.rs");
    let n = trait_method_count(src, "pub trait CanvasPaintTool");
    assert!(
        n <= 1,
        "CanvasPaintTool has {n} methods; cap is 1. FROZEN at ADR-0040 \
         Amendment 3 (2026-06-20) to the current surface (on_canvas_pointer) — \
         every canvas-painting tool implements this on top of `Tool`, so any \
         growth ripples the painting fan-out. A Coordenador-only contract \
         change: bump the cap here + write the ADR-0040 amendment."
    );
}

#[test]
fn panel_event_variant_count_is_capped() {
    // `PanelEvent` is the generic event carrier on the `ToolPanelEvent` action
    // bus channel (ADR-0040 TG-B). Every tool's `handle_panel_event` matches
    // on these variants, so adding one ripples to every tool that handles
    // panel input. Frozen at the current surface (Click / SetValue / Toggle /
    // SelectOption).
    let src = include_str!("../src/tool.rs");
    let start = src
        .find("pub enum PanelEvent {")
        .expect("PanelEvent enum present");
    let body_start = src[start..].find('{').expect("opening brace") + start;
    let body_end = src[body_start..].find("\n}").expect("enum closes") + body_start;
    // Variants are lines whose stripped form starts with an uppercase ASCII
    // letter — skips doc-comments (`///`) and blank lines.
    let n = src[body_start..body_end]
        .lines()
        .filter(|line| {
            line.trim_start()
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
        })
        .count();
    assert!(
        n <= 4,
        "PanelEvent has {n} variants; cap is 4. FROZEN at ADR-0040 TG-E to \
         the current surface (Click / SetValue / Toggle / SelectOption) — \
         adding a variant ripples to every tool's `handle_panel_event` arm \
         set. A Coordenador-only contract change: bump the cap here + write \
         the ADR-0040 amendment."
    );
}
