//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector.rs` em 2026-09-13** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Audio Editor **floating overlay** — root id for the resizable waveform +
/// timeline window that floats over the canvas in the gap between the Hierarchy
/// and Inspector docks. Drag/resize reuse the panel-agnostic
/// `blender_picker_offset` + `panel_resize_delta` store, keyed by this id
/// (mirror of the Inspector dock).
pub const AUDIO_OVERLAY_PANEL: NodeId = hash_node_id("audio_overlay_panel");

/// Audio Editor overlay — title-bar drag handle. Registered as
/// `BlenderHit { parent: AUDIO_OVERLAY_PANEL, kind: DragHandle }` by the editor
/// panel's populate; the panel-agnostic dispatch moves the overlay via
/// `blender_picker_offset`.
pub const AUDIO_OVERLAY_DRAG_HANDLE: NodeId = hash_node_id("audio_overlay_drag_handle");

/// Audio Editor overlay — bottom-right resize gripper (`ResizeHandle`).
pub const AUDIO_OVERLAY_RESIZE_HANDLE: NodeId = hash_node_id("audio_overlay_resize_handle");

/// Audio Editor overlay — bottom-left resize gripper (`ResizeHandleBl`).
pub const AUDIO_OVERLAY_RESIZE_HANDLE_BL: NodeId = hash_node_id("audio_overlay_resize_handle_bl");
