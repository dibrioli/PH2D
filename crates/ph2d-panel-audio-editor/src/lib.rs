//! `ph2d-panel-audio-editor` — the Audio Editor panel (docs/Audio/, W1).
//!
//! Docked in the shared Inspector slot (mirror of the Audio Mixer / Sprite
//! Inspector dock pattern): a compact **transport** (play/pause · stop · loop) +
//! **Load** + **Export** + a clip readout (name · position / duration). The
//! large **waveform + timeline** are painted separately as a resizable floating
//! overlay over the canvas (`ph2d_editor_core::ids::AUDIO_OVERLAY_PANEL`), not
//! here — the panel is the controls, the overlay is the spacious view.
//!
//! UI-only (no `ph2d-audio` dependency). The shell's editor bridge reads the
//! transport **intents** published here ([`snapshot`]) each frame and drives the
//! preview engine (`AudioEngine::play_preview` / `seek/pause/stop`), then
//! publishes the live position/duration back for the readout + the overlay
//! playhead.

#![forbid(unsafe_code)]

mod clipped_hits;
mod event;
pub mod loop_state;
mod paint;
mod paint_edit;
mod paint_fx;
mod paint_loop;
mod paint_variation;
mod populate;
pub mod presets;
pub mod snapshot;
pub mod state;
pub mod variation_state;

pub use state::AudioEditorState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tool_registry::hash_node_id;

/// Outer panel rect id. Single source in editor-core so the hero z-order paint
/// walk (`ids::AUDIO_EDITOR_PANEL`) resolves this panel.
pub const AEDIT_PANEL: NodeId = ph2d_editor_core::ids::AUDIO_EDITOR_PANEL;
/// Header close (X) — hides the dock (also toggled by the TopBar pill).
pub const AEDIT_CLOSE: NodeId = hash_node_id("audio_editor_close");
/// Clip name field — an editable `TextInput` (mirror of the Inspector's entity
/// name box). The shell publishes the loaded clip's name; the paint step syncs
/// it into the box on a new load (unless the user is editing it).
pub const AEDIT_NAME: NodeId = hash_node_id("audio_editor_name");
/// Play / Pause toggle — one control; the shell flips preview play/pause.
pub const AEDIT_PLAY: NodeId = hash_node_id("audio_editor_play");
/// Stop button — stops the preview and rewinds to the clip start.
pub const AEDIT_STOP: NodeId = hash_node_id("audio_editor_stop");
/// Loop toggle — whether the preview loops at the clip end.
pub const AEDIT_LOOP: NodeId = hash_node_id("audio_editor_loop");
/// Load button — the shell opens a file picker and decodes into an `EditClip`.
pub const AEDIT_LOAD: NodeId = hash_node_id("audio_editor_load");
/// Export button — the shell writes the current clip out (WAV).
pub const AEDIT_EXPORT: NodeId = hash_node_id("audio_editor_export");

// Edit ops (W2 §5) — one-shot: click arms an `AudioEditCmd` the shell applies to
// the `EditClip` (undo timeline). Whole-clip ops (no selection needed).
/// Undo the last edit.
pub const AEDIT_UNDO: NodeId = hash_node_id("audio_editor_undo");
/// Redo the last undone edit.
pub const AEDIT_REDO: NodeId = hash_node_id("audio_editor_redo");
/// Peak-normalize to 0 dBFS.
pub const AEDIT_NORMALIZE: NodeId = hash_node_id("audio_editor_normalize");
/// Loudness-normalize to −16 LUFS.
pub const AEDIT_NORM_LUFS: NodeId = hash_node_id("audio_editor_norm_lufs");
/// Reverse the clip.
pub const AEDIT_REVERSE: NodeId = hash_node_id("audio_editor_reverse");
/// Remove DC offset.
pub const AEDIT_DC: NodeId = hash_node_id("audio_editor_dc");
/// Invert polarity.
pub const AEDIT_INVERT: NodeId = hash_node_id("audio_editor_invert");
/// Force-to-mono — a **non-destructive** output toggle (downmix for 3D positional
/// audio); click again to revert. The clip stays stereo.
pub const AEDIT_MONO: NodeId = hash_node_id("audio_editor_mono");
/// Batch LUFS — normalize a whole folder of audio to a target loudness (file op).
pub const AEDIT_BATCH_LUFS: NodeId = hash_node_id("audio_editor_batch_lufs");
/// Gain −3 dB.
pub const AEDIT_GAIN_DOWN: NodeId = hash_node_id("audio_editor_gain_down");
/// Gain +3 dB.
pub const AEDIT_GAIN_UP: NodeId = hash_node_id("audio_editor_gain_up");

// Range ops (W2 §5, block 2b) — act on the waveform SELECTION (enabled only when
// one exists).
/// Crop to the selection.
pub const AEDIT_TRIM: NodeId = hash_node_id("audio_editor_trim");
/// Delete the selection (ripple).
pub const AEDIT_CUT: NodeId = hash_node_id("audio_editor_cut");
/// Silence the selection.
pub const AEDIT_SILENCE: NodeId = hash_node_id("audio_editor_silence");
/// Fade in across the selection.
pub const AEDIT_FADE_IN: NodeId = hash_node_id("audio_editor_fade_in");
/// Fade out across the selection.
pub const AEDIT_FADE_OUT: NodeId = hash_node_id("audio_editor_fade_out");

// Effects rack (W3 blocks 3a/3b) — a **chain** of parametric effects. One stage is
// SELECTED: the selector cycles its kind, up to `MAX_FX_PARAMS` sliders tune it, and
// the stage list adds/removes/reorders/bypasses stages. The audition is
// `clip → stage₀ → … → stageₙ`; Apply commits it as one undo step. Every effect
// starts at its NEUTRAL point, so a fresh stage changes nothing until a slider moves.
//
// The panel stays UI-only: it owns the chain as indices + normalized 0..1 slider
// positions; the shell owns the real DSP ranges, formatting and the
// `Effect`/`TailEffect` construction (`audio/fx_params.rs`).
/// Number of parameter sliders the rack paints. The shell publishes a label +
/// formatted value per slot; unused slots are hidden.
pub const MAX_FX_PARAMS: usize = 4;
/// How many effects the chain can hold. Bounded so the stage list stays inside the
/// docked panel (and so a runaway Add can't make the audition unaffordable).
pub const MAX_FX_STAGES: usize = 6;

/// One effect in the rack's chain. The panel owns these; the shell reads them each
/// frame and renders `clip → stage₀ → … → stageₙ` into the live audition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxStage {
    /// Index into the shell's `FX_KINDS`.
    pub kind: usize,
    /// Normalized 0..1 slider positions; the shell maps them onto real DSP units.
    pub norms: [f32; MAX_FX_PARAMS],
    /// A bypassed stage stays in the chain but is skipped by the render — the
    /// per-stage half of the rack's A/B.
    pub enabled: bool,
}

/// Previous effect kind for the selected stage.
pub const AEDIT_FX_PREV: NodeId = hash_node_id("audio_editor_fx_prev");
/// Next effect kind for the selected stage.
pub const AEDIT_FX_NEXT: NodeId = hash_node_id("audio_editor_fx_next");
/// Return the SELECTED effect's parameters to their neutral defaults.
pub const AEDIT_FX_RESET: NodeId = hash_node_id("audio_editor_fx_reset");
/// Commit the audition (one undo step) — what you hear is what lands.
pub const AEDIT_FX_APPLY: NodeId = hash_node_id("audio_editor_fx_apply");
/// Discard the audition and restore the clip's sound.
pub const AEDIT_FX_CANCEL: NodeId = hash_node_id("audio_editor_fx_cancel");
/// Append a fresh neutral stage after the selected one, and select it.
pub const AEDIT_FX_ADD: NodeId = hash_node_id("audio_editor_fx_add");
/// Remove the selected stage from the chain.
pub const AEDIT_FX_REMOVE: NodeId = hash_node_id("audio_editor_fx_remove");
/// Move the selected stage one slot earlier in the chain.
pub const AEDIT_FX_UP: NodeId = hash_node_id("audio_editor_fx_up");
/// Move the selected stage one slot later in the chain.
pub const AEDIT_FX_DOWN: NodeId = hash_node_id("audio_editor_fx_down");
/// Global A/B: mute the whole chain and hear/see the dry clip, without losing it.
pub const AEDIT_FX_BYPASS: NodeId = hash_node_id("audio_editor_fx_bypass");

// Chain presets (W3) — a `◀ name ▶` selector over the factory presets, Apply to load
// the selected one into the chain, and Save/Load for user preset files.
/// Previous factory preset in the selector.
pub const AEDIT_PRESET_PREV: NodeId = hash_node_id("audio_editor_preset_prev");
/// Next factory preset in the selector.
pub const AEDIT_PRESET_NEXT: NodeId = hash_node_id("audio_editor_preset_next");
/// Load the selected factory preset into the chain (auditions immediately).
pub const AEDIT_PRESET_APPLY: NodeId = hash_node_id("audio_editor_preset_apply");
/// Save the current chain to a user preset file (native dialog).
pub const AEDIT_PRESET_SAVE: NodeId = hash_node_id("audio_editor_preset_save");
/// Load a user preset file into the chain (native dialog).
pub const AEDIT_PRESET_LOAD: NodeId = hash_node_id("audio_editor_preset_load");

// Loop points (W6 — asset-prep). Set the loop region from the selection (auto-snapped
// to zero crossings) or Clear it; the Crossfade slider tunes the click-free seam. The
// loop is metadata (not an undo edit), plays via **Loop + Play** (no separate Audition
// control), and is written to the exported WAV's `smpl` chunk.
/// Set the loop region from the current waveform selection (auto-snaps to zero cross).
pub const AEDIT_LOOP_SET: NodeId = hash_node_id("audio_editor_loop_set");
/// Clear the loop region.
pub const AEDIT_LOOP_CLEAR: NodeId = hash_node_id("audio_editor_loop_clear");
/// Loop crossfade length slider (normalized `0..1`; the shell maps it to ms).
pub const AEDIT_LOOP_XFADE: NodeId = hash_node_id("audio_editor_loop_xfade");

// Markers / cue points (W6) — named timeline points exported to the WAV `cue`+`adtl`.
/// Add a cue marker at the playhead.
pub const AEDIT_MARK_ADD: NodeId = hash_node_id("audio_editor_mark_add");
/// Delete the cue marker nearest the playhead.
pub const AEDIT_MARK_DEL: NodeId = hash_node_id("audio_editor_mark_del");

// Variation containers (W6 asset-prep) — a set of clips the runtime plays one of per
// trigger (Random / Sequence / Shuffle) with per-play pitch/gain jitter + per-entry
// weights. Authored + auditioned + saved here; the shell owns the `VariationSet`, the
// decoded clips and the picker. See `paint_variation.rs` + `variation_state.rs`.
/// How many variation rows the list can hold (bounds the row-id array + Add).
pub const MAX_VARIATIONS: usize = 12;
/// Add a variation (the shell opens a file picker + decodes).
pub const AEDIT_VAR_ADD: NodeId = hash_node_id("audio_editor_var_add");
/// Remove the selected variation.
pub const AEDIT_VAR_REMOVE: NodeId = hash_node_id("audio_editor_var_remove");
/// Audition the next variation (pick per strategy + jitter → preview).
pub const AEDIT_VAR_PLAY: NodeId = hash_node_id("audio_editor_var_play");
/// Previous pick strategy in the `◀ name ▶` selector.
pub const AEDIT_VAR_STRAT_PREV: NodeId = hash_node_id("audio_editor_var_strat_prev");
/// Next pick strategy.
pub const AEDIT_VAR_STRAT_NEXT: NodeId = hash_node_id("audio_editor_var_strat_next");
/// Halve the selected variation's weight.
pub const AEDIT_VAR_WEIGHT_DOWN: NodeId = hash_node_id("audio_editor_var_weight_down");
/// Double the selected variation's weight.
pub const AEDIT_VAR_WEIGHT_UP: NodeId = hash_node_id("audio_editor_var_weight_up");
/// Save the variation set to a manifest file (native dialog).
pub const AEDIT_VAR_SAVE: NodeId = hash_node_id("audio_editor_var_save");
/// Load a variation set from a manifest file (native dialog).
pub const AEDIT_VAR_LOAD: NodeId = hash_node_id("audio_editor_var_load");
/// Pitch-jitter slider (normalized `0..1`; the shell maps it to `± semitones`).
pub const AEDIT_VAR_PITCH: NodeId = hash_node_id("audio_editor_var_pitch");
/// Gain-jitter slider (normalized `0..1`; the shell maps it to `± dB`).
pub const AEDIT_VAR_GAIN: NodeId = hash_node_id("audio_editor_var_gain");

/// Variation list row ids (click selects the row). Sized to [`MAX_VARIATIONS`].
pub const AEDIT_VAR_ROWS: [NodeId; MAX_VARIATIONS] = [
    hash_node_id("audio_editor_var_row0"),
    hash_node_id("audio_editor_var_row1"),
    hash_node_id("audio_editor_var_row2"),
    hash_node_id("audio_editor_var_row3"),
    hash_node_id("audio_editor_var_row4"),
    hash_node_id("audio_editor_var_row5"),
    hash_node_id("audio_editor_var_row6"),
    hash_node_id("audio_editor_var_row7"),
    hash_node_id("audio_editor_var_row8"),
    hash_node_id("audio_editor_var_row9"),
    hash_node_id("audio_editor_var_row10"),
    hash_node_id("audio_editor_var_row11"),
];

/// Parameter slider 0.
pub const AEDIT_FX_P0: NodeId = hash_node_id("audio_editor_fx_p0");
/// Parameter slider 1.
pub const AEDIT_FX_P1: NodeId = hash_node_id("audio_editor_fx_p1");
/// Parameter slider 2.
pub const AEDIT_FX_P2: NodeId = hash_node_id("audio_editor_fx_p2");
/// Parameter slider 3.
pub const AEDIT_FX_P3: NodeId = hash_node_id("audio_editor_fx_p3");
/// The parameter sliders, indexed by slot.
pub const AEDIT_FX_PARAMS: [NodeId; MAX_FX_PARAMS] =
    [AEDIT_FX_P0, AEDIT_FX_P1, AEDIT_FX_P2, AEDIT_FX_P3];

/// Chain row 0 — click selects the stage.
pub const AEDIT_FX_S0: NodeId = hash_node_id("audio_editor_fx_s0");
/// Chain row 1.
pub const AEDIT_FX_S1: NodeId = hash_node_id("audio_editor_fx_s1");
/// Chain row 2.
pub const AEDIT_FX_S2: NodeId = hash_node_id("audio_editor_fx_s2");
/// Chain row 3.
pub const AEDIT_FX_S3: NodeId = hash_node_id("audio_editor_fx_s3");
/// Chain row 4.
pub const AEDIT_FX_S4: NodeId = hash_node_id("audio_editor_fx_s4");
/// Chain row 5.
pub const AEDIT_FX_S5: NodeId = hash_node_id("audio_editor_fx_s5");
/// The chain rows, indexed by stage.
pub const AEDIT_FX_STAGES: [NodeId; MAX_FX_STAGES] = [
    AEDIT_FX_S0,
    AEDIT_FX_S1,
    AEDIT_FX_S2,
    AEDIT_FX_S3,
    AEDIT_FX_S4,
    AEDIT_FX_S5,
];

/// Chain row 0's enable (eye) toggle.
pub const AEDIT_FX_S0_ON: NodeId = hash_node_id("audio_editor_fx_s0_on");
/// Chain row 1's enable toggle.
pub const AEDIT_FX_S1_ON: NodeId = hash_node_id("audio_editor_fx_s1_on");
/// Chain row 2's enable toggle.
pub const AEDIT_FX_S2_ON: NodeId = hash_node_id("audio_editor_fx_s2_on");
/// Chain row 3's enable toggle.
pub const AEDIT_FX_S3_ON: NodeId = hash_node_id("audio_editor_fx_s3_on");
/// Chain row 4's enable toggle.
pub const AEDIT_FX_S4_ON: NodeId = hash_node_id("audio_editor_fx_s4_on");
/// Chain row 5's enable toggle.
pub const AEDIT_FX_S5_ON: NodeId = hash_node_id("audio_editor_fx_s5_on");
/// The per-stage enable toggles, indexed by stage.
pub const AEDIT_FX_STAGE_ONS: [NodeId; MAX_FX_STAGES] = [
    AEDIT_FX_S0_ON,
    AEDIT_FX_S1_ON,
    AEDIT_FX_S2_ON,
    AEDIT_FX_S3_ON,
    AEDIT_FX_S4_ON,
    AEDIT_FX_S5_ON,
];

/// A one-shot edit command the panel arms (via a click) and the shell drains +
/// applies to the loaded `EditClip`. UI-only enum (no `ph2d-audio-edit` dep here);
/// the shell maps each variant to the matching `EditClip::apply_*` / undo/redo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEditCmd {
    /// Undo the last edit.
    Undo,
    /// Redo the last undone edit.
    Redo,
    /// Peak-normalize to 0 dBFS.
    NormalizePeak,
    /// Loudness-normalize to −16 LUFS.
    NormalizeLufs,
    /// Reverse the clip.
    Reverse,
    /// Remove DC offset.
    RemoveDc,
    /// Invert polarity.
    Invert,
    /// Gain −3 dB.
    GainDown,
    /// Gain +3 dB.
    GainUp,
    /// Crop to the selection.
    Trim,
    /// Delete the selection (ripple).
    Cut,
    /// Silence the selection.
    Silence,
    /// Fade in across the selection.
    FadeIn,
    /// Fade out across the selection.
    FadeOut,
    /// Commit the effects rack's live audition as one undo step.
    ApplyFx,
    /// Discard the live audition; the clip goes back to how it sounded.
    CancelFx,
}

/// Zero-size marker implementing the typed Audio Editor panel contract.
pub struct AudioEditorPanel;

impl Panel for AudioEditorPanel {
    type State = AudioEditorState;

    const ID: &'static str = "audio_editor";
    const NODE_ID: NodeId = AEDIT_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut AudioEditorState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut AudioEditorState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}

/// Shell → panel: the audition ended (Apply/Cancel) — stop asking for renders.
pub use snapshot::clear_fx_dirty;
/// Panel → shell: whether the global Bypass (A/B) is engaged — the dry clip is
/// what sounds and shows, but the chain is kept.
pub use snapshot::fx_bypass;
/// Panel → shell: the whole effect chain, in render order.
pub use snapshot::fx_chain;
/// Panel → shell: whether the user has touched the rack since the last
/// Apply/Cancel — i.e. whether the shell should be auditioning the chain live.
pub use snapshot::fx_dirty;
/// Panel → shell: which chain stage the sliders edit (the shell caches upstream).
pub use snapshot::fx_sel;
/// Panel → shell: the SELECTED stage's `(kind, norms)` — what the sliders show.
pub use snapshot::fx_sel_stage;
/// Panel → shell: whether looping is enabled (persistent).
pub use snapshot::looping;
/// Shell: put the rack back to a single neutral stage (after Apply/Cancel/Load).
pub use snapshot::reset_fx_chain;
/// Shell → panel: publish the loaded clip's display name.
pub use snapshot::set_clip_name;
/// Shell → panel: whether an audition is currently sounding (enables Cancel).
pub use snapshot::set_fx_auditioning;
/// Shell → panel: replace the whole chain (the preset-load path); it auditions at
/// once and Apply commits it.
pub use snapshot::set_fx_chain;
/// Shell → panel: publish whether a waveform selection exists (range-op buttons).
pub use snapshot::set_has_selection;
/// Panel → shell: the pending edit command (one-shot; the shell applies it to the
/// `EditClip`).
pub use snapshot::take_edit_cmd;
/// Panel → shell: whether the user requested Export (one-shot; the shell writes
/// the clip to WAV).
pub use snapshot::take_export;
/// Panel → shell: whether the user requested Load (one-shot; the shell opens a
/// file picker + decodes).
pub use snapshot::take_load;
/// Panel → shell: whether the user requested a play/pause toggle this frame
/// (one-shot; the bridge drains it and flips the preview transport).
pub use snapshot::take_play_pause;
/// Panel → shell: whether the user requested Stop (one-shot).
pub use snapshot::take_stop;
/// Shell → panel: publish whether undo/redo are currently available (button dim).
pub use snapshot::{set_can_redo, set_can_undo};
/// Shell → panel: publish the live transport state for the readout + buttons.
pub use snapshot::{set_duration_secs, set_loaded, set_playing, set_position_secs};
/// Shell → panel: publish the effects rack's view — the display name of every
/// effect kind, each kind's **neutral** normalized defaults (so the panel can seed
/// a fresh or reset stage), and the selected stage's per-parameter
/// `(label, formatted value)` pairs.
pub use snapshot::{set_fx_kind_defaults, set_fx_kind_names, set_fx_param_views};

// Loop points + batch + force-mono (W6 asset-prep).
/// Shell → panel: publish the loop region as `(start_secs, end_secs)` for the readout.
pub use loop_state::set_loop_span;
/// Shell → panel: publish the marker count (readout + Del enablement).
pub use loop_state::set_marker_count;
/// Shell → panel: publish whether force-mono is engaged (lights the toggle).
pub use loop_state::set_mono_on;
/// Panel → shell: whether the user asked to add a marker at the playhead (one-shot).
pub use loop_state::take_add_marker;
/// Panel → shell: whether the user asked to batch-normalize a folder (one-shot).
pub use loop_state::take_batch_lufs;
/// Panel → shell: whether the user asked to clear the loop (one-shot).
pub use loop_state::take_clear_loop;
/// Panel → shell: whether the user asked to delete the nearest marker (one-shot).
pub use loop_state::take_del_marker;
/// Panel → shell: whether the user asked to set the loop from the selection (one-shot).
pub use loop_state::take_set_loop;
/// Panel → shell: whether the user asked to flip the force-mono toggle (one-shot).
pub use loop_state::take_toggle_mono;
/// Panel → shell: the loop crossfade slider position (`0..1`).
pub use loop_state::xfade_norm;

// Variation containers (W6 asset-prep).
/// Shell → panel: the current strategy's display name (selector readout).
pub use variation_state::set_strategy_name;
/// Shell → panel: the variation row labels (name + weight), in list order.
pub use variation_state::set_variation_names;
/// Panel → shell: whether the user asked to add a variation (one-shot; file picker).
pub use variation_state::take_add_variation;
/// Panel → shell: whether the user asked to load a variation set (one-shot; dialog).
pub use variation_state::take_load_variation_set;
/// Panel → shell: whether the user asked to audition the next variation (one-shot).
pub use variation_state::take_play_variation;
/// Panel → shell: whether the user asked to remove the selected variation (one-shot).
pub use variation_state::take_remove_variation;
/// Panel → shell: whether the user asked to save the variation set (one-shot; dialog).
pub use variation_state::take_save_variation_set;
/// Panel → shell: net strategy-cycle steps since the last drain (0 = none).
pub use variation_state::take_strategy_step;
/// Panel → shell: net weight-doubling steps for the selected entry (0 = none).
pub use variation_state::take_weight_step;
/// Panel → shell: the selected variation row (Remove / Weight target).
pub use variation_state::variation_sel;
/// Panel → shell: the pitch- / gain-jitter slider positions `0..1`.
pub use variation_state::{gain_jitter_norm, pitch_jitter_norm};

// Chain presets (W3).
/// Panel → shell: which factory preset the selector shows (the shell builds it).
pub use presets::preset_sel;
/// Shell → panel: publish the factory preset names the selector cycles.
pub use presets::set_preset_names;
/// Panel → shell: whether the user asked to apply the selected factory preset.
pub use presets::take_apply_preset;
/// Panel → shell: whether the user asked to load a user preset file (native dialog).
pub use presets::take_load_preset;
/// Panel → shell: whether the user asked to save the chain to a user preset file.
pub use presets::take_save_preset;
