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

mod event;
mod paint;
mod paint_fx;
mod populate;
pub mod snapshot;
pub mod state;

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

// Effects rack (W3 block 3a) — ONE parametric effect at a time: a selector cycles
// the kinds, up to `MAX_FX_PARAMS` sliders tune it, Apply commits it to the target
// range (selection, or whole clip). The panel stays UI-only: it holds normalized
// 0..1 slider positions and an index; the shell owns the real DSP ranges,
// formatting and the `Effect`/`TailEffect` construction (`audio/fx_params.rs`).
/// Number of parameter sliders the rack paints. The shell publishes a label +
/// formatted value per slot; unused slots are hidden.
pub const MAX_FX_PARAMS: usize = 4;

/// Previous effect in the selector.
pub const AEDIT_FX_PREV: NodeId = hash_node_id("audio_editor_fx_prev");
/// Next effect in the selector.
pub const AEDIT_FX_NEXT: NodeId = hash_node_id("audio_editor_fx_next");
/// Apply the selected effect at the current parameters.
pub const AEDIT_FX_APPLY: NodeId = hash_node_id("audio_editor_fx_apply");
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
    /// Apply the effects rack's selected effect at its current parameters. The
    /// shell reads [`fx_kind`] + [`fx_norms`] to build it.
    ApplyFx,
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

/// Panel → shell: the selected effect index into the shell's `FX_KINDS`.
pub use snapshot::fx_kind;
/// Panel → shell: the normalized 0..1 position of every parameter slider.
pub use snapshot::fx_norms;
/// Panel → shell: whether looping is enabled (persistent).
pub use snapshot::looping;
/// Shell → panel: publish the loaded clip's display name.
pub use snapshot::set_clip_name;
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
/// Shell → panel: publish the effects rack's view — how many kinds exist, the
/// selected kind's name, its per-parameter `(label, formatted value)` pairs, and
/// the normalized defaults to load into the sliders when the kind changes.
pub use snapshot::{set_fx_defaults, set_fx_kind_count, set_fx_kind_name, set_fx_param_views};
