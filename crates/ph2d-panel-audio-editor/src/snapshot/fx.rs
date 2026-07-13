//! The **effects rack**'s half of the snapshot (W3 blocks 3a/3b) — the chain, the selection, the
//! kind table, the audition flags.
//!
//! A child module of `snapshot`, split out when that file crossed the panel's 600-LOC cap
//! (HR-18). It is a coherent unit on its own: every `FX_*` thread-local and the only functions
//! that touch them, and nothing else. `snapshot` re-exports it, so no caller changes.

use std::cell::{Cell, RefCell};

use crate::{FxStage, MAX_FX_PARAMS, MAX_FX_STAGES};

thread_local! {
    /// Effects rack (W3 block 3b) — panel → shell: the effect **chain**, in render
    /// order. The panel owns it; the shell renders `clip → stage₀ → … → stageₙ`.
    /// Empty until the shell has published the kind table (see [`ensure_chain`]).
    static FX_CHAIN: RefCell<Vec<FxStage>> = const { RefCell::new(Vec::new()) };
    /// Which stage the selector + sliders edit. Clamped to the chain.
    static FX_SEL: Cell<usize> = const { Cell::new(0) };
    /// Global A/B: hear/see the dry clip while keeping the chain intact.
    static FX_BYPASS: Cell<bool> = const { Cell::new(false) };
    // Effects rack — shell → panel: what to paint.
    static FX_KIND_NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// Every kind's **neutral** normalized defaults, indexed by kind. A fresh or
    /// reset stage is seeded from here, so it is a byte-identical no-op.
    static FX_KIND_DEFAULTS: RefCell<Vec<[f32; MAX_FX_PARAMS]>> = const { RefCell::new(Vec::new()) };
    static FX_PARAM_VIEWS: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
    /// Set ONLY by real user input on the rack — never by the paint step's slider
    /// re-seed, or merely opening the panel would start auditioning a chain nobody
    /// asked for. While true the shell renders + hot-swaps the audition;
    /// Apply/Cancel clear it.
    static FX_DIRTY: Cell<bool> = const { Cell::new(false) };
    /// Shell → panel: an audition is sounding (enables Cancel + Bypass).
    static FX_AUDITIONING: Cell<bool> = const { Cell::new(false) };
    /// The panel moved the selected stage's parameters programmatically (kind
    /// switch, reset, add, select…) — the paint step must push them into the slider
    /// widgets once. Never set by a user drag, which would fight the drag.
    static FX_SLIDER_SYNC: Cell<bool> = const { Cell::new(false) };
}

// ---- Effects rack (W3 blocks 3a/3b) ----

/// How many effect kinds the selector cycles (derived from the published names).
pub(crate) fn fx_kind_count() -> usize {
    FX_KIND_NAMES.with(|c| c.borrow().len())
}

/// The neutral normalized defaults of `kind` (all-zero if the shell has not
/// published its table yet — see [`ensure_chain`], which waits for it).
fn defaults_for(kind: usize) -> [f32; MAX_FX_PARAMS] {
    FX_KIND_DEFAULTS.with(|c| c.borrow().get(kind).copied().unwrap_or_default())
}

/// Make sure the chain holds at least one stage, so the rack always has something
/// to edit. Deliberately a **no-op until the shell has published the kind table**:
/// a stage seeded with all-zero normals would sit on the bottom of every range
/// (Low-Pass at 20 Hz) instead of on its neutral point.
fn ensure_chain() {
    if fx_kind_count() == 0 {
        return;
    }
    FX_CHAIN.with(|c| {
        let mut chain = c.borrow_mut();
        if chain.is_empty() {
            chain.push(neutral_stage(0));
        }
    });
}

/// A fresh stage of `kind`, sitting exactly on its neutral (no-op) defaults.
fn neutral_stage(kind: usize) -> FxStage {
    FxStage {
        kind,
        norms: defaults_for(kind),
        enabled: true,
    }
}

/// Run `f` on the selected stage (after clamping the selection to the chain).
/// Marks the rack dirty — every caller is a real user edit of the audible chain.
fn edit_selected(f: impl FnOnce(&mut FxStage)) {
    ensure_chain();
    FX_CHAIN.with(|c| {
        let mut chain = c.borrow_mut();
        let sel = FX_SEL.with(Cell::get).min(chain.len().saturating_sub(1));
        FX_SEL.with(|s| s.set(sel));
        if let Some(stage) = chain.get_mut(sel) {
            f(stage);
        }
    });
    FX_DIRTY.with(|c| c.set(true));
}

/// Panel: step the SELECTED stage's effect kind by `delta`, wrapping, and re-seed
/// its parameters with that kind's neutral defaults. No-op until the shell has
/// published the kind table.
pub(crate) fn cycle_fx_kind(delta: isize) {
    let count = fx_kind_count();
    if count == 0 {
        return;
    }
    edit_selected(|stage| {
        let next = (stage.kind as isize + delta).rem_euclid(count as isize) as usize;
        stage.kind = next;
        stage.norms = FX_KIND_DEFAULTS.with(|c| c.borrow().get(next).copied().unwrap_or_default());
    });
    FX_SLIDER_SYNC.with(|c| c.set(true));
}

/// Panel → shell: is there a live audition to render / keep hot-swapped?
pub fn fx_dirty() -> bool {
    FX_DIRTY.with(Cell::get)
}

/// Shell: the audition was committed (Apply) or thrown away (Cancel) — the rack
/// goes back to idle.
pub fn clear_fx_dirty() {
    FX_DIRTY.with(|c| c.set(false));
}

/// Shell: put the rack back to a single neutral stage — after Apply (the chain is
/// baked into the clip; re-rendering it would double every effect), after Cancel,
/// and on Load.
pub fn reset_fx_chain() {
    FX_CHAIN.with(|c| c.borrow_mut().clear());
    FX_SEL.with(|c| c.set(0));
    FX_BYPASS.with(|c| c.set(false));
    FX_DIRTY.with(|c| c.set(false));
    ensure_chain();
    FX_SLIDER_SYNC.with(|c| c.set(true));
}

/// Shell: **replace** the whole chain — the preset load path (a factory preset or a
/// user file). Marks it dirty so it auditions immediately, exactly like tuning a
/// slider; Apply then commits it. Clamped to [`MAX_FX_STAGES`]; an empty chain (a
/// preset that resolved to nothing) falls back to one neutral stage so the rack is
/// never left with nothing to edit.
pub fn set_fx_chain(mut chain: Vec<FxStage>) {
    chain.truncate(MAX_FX_STAGES);
    FX_CHAIN.with(|c| {
        let mut c = c.borrow_mut();
        *c = chain;
        if c.is_empty() {
            c.push(neutral_stage(0));
        }
    });
    FX_SEL.with(|c| c.set(0));
    FX_BYPASS.with(|c| c.set(false));
    FX_DIRTY.with(|c| c.set(true));
    FX_SLIDER_SYNC.with(|c| c.set(true));
}

/// Panel: return the SELECTED stage's parameters to its kind's neutral defaults.
/// Stays dirty if an audition is running — the shell simply re-renders the chain
/// with this stage now transparent.
pub(crate) fn reset_fx_params() {
    edit_selected(|stage| stage.norms = defaults_for(stage.kind));
    FX_SLIDER_SYNC.with(|c| c.set(true));
}

/// Whether the selected stage already sits on its neutral defaults — then there is
/// nothing to reset and the button is dimmed.
pub(crate) fn fx_at_defaults() -> bool {
    const EPS: f32 = 1e-4;
    let Some(stage) = selected_stage() else {
        return true;
    };
    let defaults = defaults_for(stage.kind);
    stage
        .norms
        .iter()
        .zip(&defaults)
        .all(|(n, d)| (n - d).abs() <= EPS)
}

/// Panel: append a fresh neutral stage after the selected one and select it.
/// A neutral stage is a no-op, so Add alone never changes the sound.
pub(crate) fn add_fx_stage() {
    ensure_chain();
    let added = FX_CHAIN.with(|c| {
        let mut chain = c.borrow_mut();
        if chain.len() >= MAX_FX_STAGES {
            return false;
        }
        let at = (FX_SEL.with(Cell::get) + 1).min(chain.len());
        chain.insert(at, neutral_stage(0));
        FX_SEL.with(|s| s.set(at));
        true
    });
    if added {
        FX_DIRTY.with(|c| c.set(true));
        FX_SLIDER_SYNC.with(|c| c.set(true));
    }
}

/// Panel: drop the selected stage. The chain never empties — removing the last
/// one leaves a fresh neutral stage behind, so the rack still has something to edit.
pub(crate) fn remove_fx_stage() {
    ensure_chain();
    FX_CHAIN.with(|c| {
        let mut chain = c.borrow_mut();
        let sel = FX_SEL.with(Cell::get);
        if sel < chain.len() {
            chain.remove(sel);
        }
        if chain.is_empty() {
            chain.push(neutral_stage(0));
        }
        FX_SEL.with(|s| s.set(sel.min(chain.len() - 1)));
    });
    FX_DIRTY.with(|c| c.set(true));
    FX_SLIDER_SYNC.with(|c| c.set(true));
}

/// Panel: move the selected stage `delta` slots (order matters — a filter before
/// a reverb is not the same as after). Clamped at the ends; the selection follows.
pub(crate) fn move_fx_stage(delta: isize) {
    ensure_chain();
    let moved = FX_CHAIN.with(|c| {
        let mut chain = c.borrow_mut();
        let sel = FX_SEL.with(Cell::get);
        let dst = sel as isize + delta;
        if sel >= chain.len() || dst < 0 || dst >= chain.len() as isize {
            return false;
        }
        chain.swap(sel, dst as usize);
        FX_SEL.with(|s| s.set(dst as usize));
        true
    });
    if moved {
        FX_DIRTY.with(|c| c.set(true));
    }
}

/// Panel: make stage `i` the one the selector + sliders edit.
pub(crate) fn select_fx_stage(i: usize) {
    if i < fx_stage_count() {
        FX_SEL.with(|c| c.set(i));
        FX_SLIDER_SYNC.with(|c| c.set(true));
    }
}

/// Panel: flip stage `i` in/out of the render without removing it (per-stage A/B).
pub(crate) fn toggle_fx_stage_enabled(i: usize) {
    let toggled = FX_CHAIN.with(|c| match c.borrow_mut().get_mut(i) {
        Some(stage) => {
            stage.enabled = !stage.enabled;
            true
        }
        None => false,
    });
    if toggled {
        FX_DIRTY.with(|c| c.set(true));
    }
}

/// Panel: flip the global A/B. Does **not** dirty the rack — the chain is intact,
/// the shell just puts the dry clip back on the wire.
pub(crate) fn toggle_fx_bypass() {
    FX_BYPASS.with(|c| c.set(!c.get()));
}

/// Panel → shell: is the whole chain bypassed (hear/see the dry clip)?
pub fn fx_bypass() -> bool {
    FX_BYPASS.with(Cell::get)
}

/// Shell → panel: an audition is sounding (enables the Cancel + Bypass buttons).
pub fn set_fx_auditioning(v: bool) {
    FX_AUDITIONING.with(|c| c.set(v));
}

pub(crate) fn fx_auditioning() -> bool {
    FX_AUDITIONING.with(Cell::get)
}

/// Panel → shell: the whole chain, in render order.
pub fn fx_chain() -> Vec<FxStage> {
    ensure_chain();
    FX_CHAIN.with(|c| c.borrow().clone())
}

pub(crate) fn fx_stage_count() -> usize {
    FX_CHAIN.with(|c| c.borrow().len())
}

/// Panel → shell: the chain index the selector + sliders edit, clamped to the
/// chain. The shell caches everything upstream of it, so dragging a slider
/// re-renders only this stage and the ones below it.
pub fn fx_sel() -> usize {
    FX_SEL
        .with(Cell::get)
        .min(fx_stage_count().saturating_sub(1))
}

fn selected_stage() -> Option<FxStage> {
    FX_CHAIN.with(|c| c.borrow().get(fx_sel()).copied())
}

/// Stage `i`'s `(display name, enabled)` for the chain list.
pub(crate) fn fx_stage_view(i: usize) -> Option<(String, bool)> {
    let stage = FX_CHAIN.with(|c| c.borrow().get(i).copied())?;
    Some((fx_kind_name(stage.kind), stage.enabled))
}

/// Panel → shell: the SELECTED stage's `(kind, normalized parameters)` — what the
/// selector names and the sliders show.
pub fn fx_sel_stage() -> (usize, [f32; MAX_FX_PARAMS]) {
    ensure_chain();
    selected_stage()
        .map(|s| (s.kind, s.norms))
        .unwrap_or((0, [0.0; MAX_FX_PARAMS]))
}

/// The selected stage's normalized slider positions (paint).
pub(crate) fn fx_norms() -> [f32; MAX_FX_PARAMS] {
    selected_stage().map(|s| s.norms).unwrap_or_default()
}

/// Panel: record slider `i`'s new normalized position from a **user drag**.
pub(crate) fn set_fx_norm(i: usize, v: f32) {
    if i < MAX_FX_PARAMS {
        edit_selected(|stage| stage.norms[i] = v.clamp(0.0, 1.0));
    }
}

/// Shell → panel: the display name of every effect kind, in `FX_KINDS` order.
/// Also fixes how far the selector cycles.
pub fn set_fx_kind_names(names: &[&str]) {
    FX_KIND_NAMES.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend(names.iter().map(|n| (*n).to_string()));
    });
}

pub(crate) fn fx_kind_name(kind: usize) -> String {
    FX_KIND_NAMES.with(|c| c.borrow().get(kind).cloned().unwrap_or_default())
}

/// Shell → panel: each kind's **neutral** normalized defaults, in `FX_KINDS` order.
pub fn set_fx_kind_defaults(defaults: &[[f32; MAX_FX_PARAMS]]) {
    FX_KIND_DEFAULTS.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend_from_slice(defaults);
    });
}

thread_local! {
    /// Shell → panel: the selected stage is a Convolution Reverb (so it needs a room), and
    /// which room is loaded. Derived by the shell from the built effect — no new column in the
    /// kind table, because "needs an IR" is a fact ABOUT the effect, not a knob on it.
    static FX_NEEDS_IR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FX_IR_READOUT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    /// Panel → shell: open the picker and load a room.
    static LOAD_IR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Shell: publish whether the selected stage wants an impulse response, and the loaded one.
pub fn set_fx_ir(needs: bool, readout: &str) {
    FX_NEEDS_IR.with(|c| c.set(needs));
    FX_IR_READOUT.with(|c| c.borrow_mut().replace_range(.., readout));
}

pub(crate) fn fx_needs_ir() -> bool {
    FX_NEEDS_IR.with(std::cell::Cell::get)
}

pub(crate) fn fx_ir_readout() -> String {
    FX_IR_READOUT.with(|c| c.borrow().clone())
}

/// Panel: the user asked for a room.
pub(crate) fn request_load_ir() {
    LOAD_IR.with(|c| c.set(true));
}

/// Shell: drain the request (one-shot).
pub fn take_load_ir() -> bool {
    LOAD_IR.with(|c| c.replace(false))
}

/// Shell → panel: `(label, formatted value)` per parameter of the SELECTED stage.
/// Length = that effect's parameter count; the panel hides the rest.
pub fn set_fx_param_views(views: &[(String, String)]) {
    FX_PARAM_VIEWS.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend_from_slice(views);
    });
}

pub(crate) fn fx_param_views() -> Vec<(String, String)> {
    FX_PARAM_VIEWS.with(|c| c.borrow().clone())
}

/// The selected stage's normals, iff the panel moved them **programmatically**
/// (kind switch, reset, add, remove, select) since the last paint. `None` while
/// only the user is dragging, so the drag isn't overwritten every frame.
pub(crate) fn fx_sliders_need_sync() -> Option<[f32; MAX_FX_PARAMS]> {
    FX_SLIDER_SYNC.with(|c| c.replace(false)).then(fx_norms)
}
