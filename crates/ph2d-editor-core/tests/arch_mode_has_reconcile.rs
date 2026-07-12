//! Wave 10 / Etapa 5.3 — every `pub fn set_<X>_mode` setter must
//! either reconcile dependent state or be on the opt-out list.
//!
//! Recurring bug class (Image Tools Bug §2): a setter wrote a single
//! field (`self.mode = m`) but ignored downstream state that the
//! mode-switch invalidates (e.g. cached previews, derived params,
//! widget enable masks). Result: the tool reverted to stale state on
//! the next paint.
//!
//! Canonical pattern: a setter that switches MODE-style state should
//! either (a) reconcile downstream state inline, or (b) call a
//! companion `reconcile_<X>_state` (or `reset_<X>_caches`,
//! `invalidate_<X>`) helper.
//!
//! Gate scans all crates' src/ for `pub fn set_*_mode` signatures,
//! reads the setter body, and verifies one of:
//!   - body contains the substring `reconcile_` OR `invalidate_`
//!     OR `reset_` (companion call), OR
//!   - body is a simple field write AND the symbol is on `BENIGN_SET_MODE`
//!     opt-out below (benign single-field setters).

use std::fs;
use std::path::{Path, PathBuf};

/// Setters that legitimately need only a single field write. Each
/// entry: (function name, justification). Adding here requires
/// confirming the setter has NO downstream state to reconcile (most
/// new tools/panels DO; resist the urge).
const BENIGN_SET_MODE: &[(&str, &str)] = &[
    (
        "set_mode",
        "VectorTool draw-mode: single-field write (`self.mode = mode`), same as the \
         per-mode arms of `handle_panel_event`; the shell mirrors it, nothing derived",
    ),
    (
        "set_tool_view_mode",
        "TopBar icon view mode — purely visual, no dependent caches",
    ),
    (
        "set_mode",
        "FlipTool canvas mode (Select/Draw/Erase): a single `mode` field write on a \
         deliberately-thin tool (ADR-0114). No derived cache — the shell reads `mode()` \
         fresh each frame from the published snapshot to route input + the gizmo.",
    ),
    (
        "set_erase_mode",
        "FlipTool erase sub-mode (Soft/Hard/Stroke): a single `erase` field write, read \
         fresh at erase time by `flip_erase::erase_at`. No derived cache keyed on it.",
    ),
    (
        "set_blender_channel_mode",
        "BlenderPicker RGB↔HSV display only; channel values are derived per-paint",
    ),
    (
        "set_blend_mode",
        "LayerStack data-model field write; the compositor reads `blend_mode` \
         fresh on each recomposite (no derived cache keyed on it inside the \
         stack) — same pattern as the unflagged `set_opacity`. Composite-cache \
         invalidation is the caller's job (the painter tool's dirty model), not \
         the setter's.",
    ),
    (
        "set_coord_mode",
        "FillParamsUbo single `ucontrol`-lane write (W6 procedural fill). The \
         recompile-free design (gate `procedural_fill_no_recompile_on_animate`) \
         makes an enum-mode change JUST a UBO field write the GPU reads fresh \
         next frame — no pipeline recompile and no derived CPU state keyed on it. \
         Same pattern as the unflagged `set_math_op`/`set_blend_mode` UBO setters.",
    ),
    (
        "set_color_model",
        "WashCompositor Linear↔K–M toggle (ADR-0089/0091): a single `s.color_model` \
         field write. The re-render is driven by the bridge (sets `seeded=false` on a \
         model flip), NOT this setter; the compositor holds no derived cache keyed on \
         the model (the field is encoding-agnostic — both modes read the same buffers).",
    ),
    (
        "set_texture_ramp_alpha_mode",
        "Painter brush Color Ramp alpha action (Off/Strength/SpriteAlpha): a single \
         `texture_ramp_alpha_mode` field write. It is NOT baked into the ramp LUT (which \
         holds RGBA colours) — the stamp reads it fresh each dab in `stamp_dabs_ramped`, \
         exactly like the unflagged `set_blend_mode`. No derived cache keyed on it.",
    ),
    (
        "set_shape_ramp_alpha_mode",
        "Painter brush SHAPE Color Ramp alpha action: the Shape ramp twin of \
         `set_texture_ramp_alpha_mode` — a single `shape_color_ramp_alpha_mode` field write, \
         read fresh each dab at stamp time, not baked into any LUT. No derived cache keyed on it.",
    ),
    (
        "set_paint_tool_mode",
        "Painter rail tool-mode setter (smear/blur/clone/mask/eyedropper/eraser/brush): writes only \
         simple mode fields (`paint_mode`/`eraser`/`eyedropper_armed` + a one-shot Spacing default). \
         It no longer discards the transient Mask scratch — the scratch is INTENTIONALLY kept alive \
         across tool switches (self-gated by `mask_scratch_active()` on target == active), so a \
         tool-mode change has no derived cache to settle. Apply (a separate action) promotes the \
         scratch to a layer mask.",
    ),
    (
        "set_selection_mode",
        "Painter Selection sub-mode setter (Automatic/Freehand/Rectangle/Ellipse — ADR-0103): a single \
         `selection_mode` field write. The on-canvas router reads it FRESH at each gesture Down (which \
         engine to arm); no derived cache or mask is keyed on it — the mask only changes when a gesture \
         actually runs. Same single-field pattern as the unflagged `set_selection_bool_op`.",
    ),
    (
        "set_deform_mode",
        "Painter Deform sub-mode setter (Push/Twist/Pinch/Wrinkle/Fold/Reconstruct — Deform Wave 1): a \
         single `deform.mode` field write. The warp kernel reads it FRESH per dab (`warp_dab_at` builds \
         the DabField each stamp); no derived cache is keyed on it. Same single-field pattern as \
         `set_selection_mode`.",
    ),
    (
        "set_deform_transform_mode",
        "Painter Deform Transform sub-mode setter (Uniform/Free — Deform Wave 2): a single \
         `deform.transform_mode` field write. The gizmo reads it FRESH on each handle drag \
         (`deform_gizmo_move` branches on it to lock/free the aspect); no derived cache is keyed on it. \
         Same single-field pattern as `set_deform_mode`.",
    ),
];

/// Setters that reconcile via implementation-specific verbs (rather
/// than `reconcile_*` / `invalidate_*` / `reset_*`). Each entry:
/// (function name, verb that signals reconciliation, justification).
/// The gate accepts the setter as reconciling iff its body contains
/// the named verb. Adding here requires (a) confirming the verb is
/// the canonical reconciliation for that subsystem and (b) the
/// reconciliation actually flushes the downstream state the mode
/// switch invalidates.
///
/// Etapa 6 M-1 refinement: the previous gate accepted ANY method
/// call as evidence; a buggy setter like
/// `self.mode = on; self.toast.show("…")` would pass. The new gate
/// requires either a canonical helper name OR an explicit entry
/// here naming the specific verb.
const RECONCILES_VIA: &[(&str, &str, &str)] = &[
    (
        "set_present_mode",
        "configure",
        "wgpu: surface.configure() rebuilds the swapchain — the actual reconciliation",
    ),
    (
        "set_filter_mode",
        "rebuild_material_bind_group",
        "ph2d-render renderer: rebuilds the material bind group bound to the old sampler",
    ),
    (
        "set_filter_mode",
        "create_sprite_sampler",
        "ph2d-render atlas/individual: recreates the sampler — the dependent state itself",
    ),
    (
        "set_texture_ramp_mode",
        "texture_ramp_dirty",
        "Painter brush ramp colour-interpolation space: sets `texture_ramp_dirty = true` so \
         `ensure_ramp_lut` re-bakes the LUT (the colours the mode changes) before the next stamp",
    ),
    (
        "set_shape_ramp_mode",
        "shape_ramp_changed",
        "Painter brush SHAPE ramp colour-interpolation space: calls `shape_ramp_changed()` which \
         flags both the colour + tone LUTs dirty (and bumps the version) so they re-bake before the \
         next stamp — the Shape twin of `set_texture_ramp_mode`",
    ),
    (
        "set_texture_layer_ramp_mode",
        "rerender_texture_layer",
        "Painter Texture LAYER ramp mode: a Texture layer is pre-rendered, so the setter \
         re-renders the whole layer (the dependent state the mode change invalidates)",
    ),
    (
        "set_texture_layer_ramp_alpha_mode",
        "rerender_texture_layer",
        "Painter Texture LAYER ramp alpha action: re-renders the pre-rendered layer, like \
         `set_texture_layer_ramp_mode`",
    ),
    (
        "set_stroke_op_mode",
        "refill_open_shape",
        "Painter multi-shape Operation mode (Overlay/Add/Remove): retags the active shape's op and \
         calls `refill_open_shape` to recompose the derived pixels (a boolean/offset result may change)",
    ),
];

#[test]
fn every_set_mode_setter_reconciles_or_is_benign() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let crates_dir = workspace_root.join("crates");
    let shells_dir = workspace_root.join("shells");

    let mut files: Vec<PathBuf> = Vec::new();
    collect_rs(&crates_dir, &mut files);
    if shells_dir.is_dir() {
        collect_rs(&shells_dir, &mut files);
    }
    // Don't scan this gate's own source.
    files.retain(|p| !p.ends_with("arch_mode_has_reconcile.rs"));

    let benign_names: Vec<&str> = BENIGN_SET_MODE.iter().map(|(n, _)| *n).collect();
    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        let Ok(src) = fs::read_to_string(path) else {
            continue;
        };
        for (fn_name, body) in extract_set_mode_setters(&src) {
            if benign_names.contains(&fn_name.as_str()) {
                continue;
            }
            // Reconciliation signal: body either uses one of the
            // canonical helper names (reconcile_/invalidate_/reset_/
            // on_mode_changed) OR matches the verb on the symbol's
            // RECONCILES_VIA entry. ANY method call is NOT enough —
            // that was the old over-loose gate that let a buggy
            // `self.toast.show(…)` slip through.
            let canonical_helper = body.contains("reconcile_")
                || body.contains("invalidate_")
                || body.contains("reset_")
                || body.contains("on_mode_changed");
            let via_specific_verb = RECONCILES_VIA
                .iter()
                .filter(|(name, _, _)| *name == fn_name)
                .any(|(_, verb, _)| body.contains(verb));
            let reconciles = canonical_helper || via_specific_verb;
            if !reconciles {
                violations.push(format!(
                    "{}::{} — body has no canonical reconcile call \
                     (reconcile_*/invalidate_*/reset_*/on_mode_changed) \
                     and no RECONCILES_VIA entry naming the specific verb",
                    path.display(),
                    fn_name
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`set_*_mode` setter(s) without a reconcile/invalidate call:\n  \
         {}\n\n\
         Either: (a) reconcile downstream state inline (clear caches, \
         reset derived params, etc.), (b) call a companion \
         `reconcile_<X>_state` helper, or (c) — only if the setter is \
         truly a single-field write with no dependents — add the name \
         to BENIGN_SET_MODE in this test with a 1-line justification.",
        violations.join("\n  ")
    );
}

/// Find every `pub fn set_<name>_mode(...) { ... }` and return
/// `(fn_name, body_source)` pairs. Body is the brace-delimited block.
fn extract_set_mode_setters(src: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let bytes = src.as_bytes();
    let needle = "pub fn set_";
    let mut search_from = 0;
    while let Some(rel) = src[search_from..].find(needle) {
        let pos = search_from + rel;
        // Extract the function name: `pub fn set_<name>(` — name is
        // everything between `set_` and the first `(` (must contain `_mode`).
        let after = &src[pos + "pub fn ".len()..];
        let Some(paren) = after.find('(') else {
            break;
        };
        let name = after[..paren].trim().to_string();
        if !name.contains("_mode") || !name.starts_with("set_") {
            search_from = pos + 1;
            continue;
        }
        // Walk to opening `{`.
        let mut i = pos + "pub fn ".len() + paren;
        let mut paren_depth = 1i32;
        while i < bytes.len() && paren_depth > 0 {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                b'(' => paren_depth += 1,
                b')' => paren_depth -= 1,
                _ => {}
            }
        }
        // Skip the return type until `{`.
        while i < bytes.len() && bytes[i] != b'{' && bytes[i] != b';' {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b';' {
            search_from = pos + 1;
            continue;
        }
        let body_start = i;
        let mut depth = 0i32;
        while i < bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let body = src[body_start..=i].to_string();
        out.push((name, body));
        search_from = i + 1;
    }
    out
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            // Skip target/ and node_modules/-style trees.
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name == "target" || name == "node_modules" || name.starts_with('.') {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn detector_extracts_set_mode_signatures() {
    let src = r#"
pub fn set_view_mode(&mut self, m: u8) {
    self.view = m;
}
pub fn set_brush_mode(&mut self, m: BrushMode) {
    self.brush = m;
    self.reconcile_brush_state();
}
"#;
    let setters = extract_set_mode_setters(src);
    assert_eq!(setters.len(), 2);
    assert_eq!(setters[0].0, "set_view_mode");
    assert_eq!(setters[1].0, "set_brush_mode");
    assert!(setters[1].1.contains("reconcile_"));
}
