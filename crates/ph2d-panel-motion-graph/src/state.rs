//! Ephemeral editor state (Motion Nodes M1.E4/E5) — pan / zoom / selection /
//! in-progress drag. Non-undoable (only doc mutations, via `GraphIntent`, are).
//! Owned by the typed panel registry; passed `&mut` into `paint`.

use std::collections::BTreeSet;

/// graph-space → screen affine: `screen = panel_origin + pan + graph * zoom`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct ViewState {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

/// The active pointer interaction on the canvas.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Interaction {
    #[default]
    Idle,
    /// Panning the canvas — `last` is the previous pointer position (screen).
    Pan { last: (f32, f32) },
    /// Dragging the selected nodes. `last` is the previous pointer (screen);
    /// each Update pushes an incremental `MoveNodes` delta the shell applies
    /// live (so the node tracks the cursor with no end-jump). `started` gates the
    /// one-undo-step bracket (BeginDrag on the first move, EndDrag on release).
    DragNodes {
        nodes: Vec<u32>,
        last: (f32, f32),
        started: bool,
    },
}

/// Retained panel state.
#[derive(Default)]
pub struct MotionGraphPanelState {
    pub(crate) view: ViewState,
    /// `false` until the first paint auto-fits the graph (then user-controlled).
    pub(crate) fitted: bool,
    /// Selected node ids (`NodeId.0`).
    pub(crate) selected: BTreeSet<u32>,
    pub(crate) interaction: Interaction,
}
