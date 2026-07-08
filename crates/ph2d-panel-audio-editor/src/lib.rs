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

/// Panel → shell: whether the user requested a play/pause toggle this frame
/// (one-shot; the bridge drains it and flips the preview transport).
pub use snapshot::take_play_pause;
/// Panel → shell: whether the user requested Stop (one-shot).
pub use snapshot::take_stop;
/// Panel → shell: whether the user requested Load (one-shot; the shell opens a
/// file picker + decodes).
pub use snapshot::take_load;
/// Panel → shell: whether the user requested Export (one-shot; the shell writes
/// the clip to WAV).
pub use snapshot::take_export;
/// Panel → shell: whether looping is enabled (persistent).
pub use snapshot::looping;
/// Shell → panel: publish the live transport state for the readout + buttons.
pub use snapshot::{set_duration_secs, set_loaded, set_playing, set_position_secs};
/// Shell → panel: publish the loaded clip's display name.
pub use snapshot::set_clip_name;
