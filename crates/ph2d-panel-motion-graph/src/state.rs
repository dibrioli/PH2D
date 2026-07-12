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
    /// Driven by the **middle** button, from anywhere on the surface (over a card,
    /// a wire, a backdrop — the graph moves under the cursor), which is the node-
    /// editor convention (Blender / Nuke / Houdini). The LEFT button is for
    /// selecting, never for panning (Enio, smoke 2026-07-12).
    Pan { last: (f32, f32) },
    /// Rubber-band selection — a left-drag on empty canvas. `anchor` is the press
    /// point and `cur` the live cursor (both SCREEN space, like the canvas's own
    /// rubber band: the band must stay put under the cursor, and the graph cannot
    /// pan mid-drag anyway since panning is on another button). `additive` (Shift
    /// at press) unions with the current selection instead of replacing it.
    BoxSelect {
        anchor: (f32, f32),
        cur: (f32, f32),
        additive: bool,
    },
    /// Slicing a knife stroke across the canvas (armed by `K`): every wire the
    /// segment crosses is cut on release, as ONE undo step. Screen space.
    Knife { anchor: (f32, f32), cur: (f32, f32) },
    /// Dragging the selected nodes. `last` is the previous pointer (screen);
    /// each Update pushes an incremental `MoveNodes` delta the shell applies
    /// live (so the node tracks the cursor with no end-jump). `started` gates the
    /// one-undo-step bracket (BeginDrag on the first move, EndDrag on release).
    DragNodes {
        nodes: Vec<u32>,
        last: (f32, f32),
        started: bool,
    },
    /// Dragging a backdrop by its header — it carries the nodes it FRAMES, whose
    /// set is captured once at grab time (`nodes`) rather than re-tested each
    /// frame: a node that drifts to the edge mid-drag must not silently join or
    /// leave the group it is being carried with. Each Update pushes a
    /// `MoveBackdrop` + a companion `MoveNodes` (one undo step for the pair).
    DragBackdrop {
        id: u32,
        nodes: Vec<u32>,
        last: (f32, f32),
        started: bool,
    },
    /// Dragging a wire's routing waypoint (F2, doc 44). `last` is the previous cursor
    /// position (screen); `started` brackets the undo step on the first real movement, so a
    /// click that merely grazes a dot does not mint an undo entry.
    DragWaypoint {
        to_node: u32,
        to_port: u16,
        index: usize,
        last: (f32, f32),
        started: bool,
    },
    /// Dragging one of a backdrop's bottom grippers — grows/shrinks it in place.
    /// `left` is the corner grabbed (the opposite edge stays anchored).
    ResizeBackdrop {
        id: u32,
        left: bool,
        last: (f32, f32),
        started: bool,
    },
    /// Dragging a new wire out of an output socket (E6). `cur` is the live
    /// pointer (screen) the ghost wire tracks; `target` is the input socket
    /// currently under the pointer (if any) plus whether it is locally
    /// type-compatible (domain + dim + clock) — drives the ghost color + target
    /// highlight. The drop emits `Connect` for the bridge to validate for real.
    DrawWire {
        from_node: u32,
        from_port: u16,
        cur: (f32, f32),
        target: Option<(u32, u16, bool)>,
    },
}

/// An open add-node popup (E7). Ephemeral — never undoable.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct AddMenu {
    /// Top-left of the popup panel, screen space (the R-click point, clamped at
    /// paint so the list stays on-canvas).
    pub screen: (f32, f32),
    /// Graph-space point the chosen node lands at (the R-click point mapped
    /// through the view — stable under a later pan/zoom while the menu is open).
    pub spawn: (f32, f32),
    /// **Smart-connect** (F2): the output socket a wire was dragged FROM and
    /// dropped on empty canvas. When set, the popup lists only the node types with
    /// an input this wire can feed, and picking one both creates it AND wires it —
    /// the gesture already said what it wanted, so making the artist draw the wire
    /// a second time would be asking twice.
    pub connect_from: Option<(u32, u16)>,
}

/// Retained panel state.
#[derive(Default)]
pub struct MotionGraphPanelState {
    pub(crate) view: ViewState,
    /// `false` until the first paint auto-fits the graph (then user-controlled).
    pub(crate) fitted: bool,
    /// Selected node ids (`NodeId.0`).
    pub(crate) selected: BTreeSet<u32>,
    /// The selected backdrop, if any. Mutually exclusive with `selected`: the
    /// params panel shows the properties of ONE subject, and a Delete must never
    /// be ambiguous about what it removes.
    pub(crate) selected_backdrop: Option<u32>,
    pub(crate) interaction: Interaction,
    /// Open add-node popup, or `None`. Opened by R-click on empty canvas / `A`;
    /// closed by picking a row, clicking away, or Esc.
    pub(crate) add_menu: Option<AddMenu>,
    /// `P` armed the probe: the NEXT click on a node picks it as the probe target.
    /// Disarmed by the pick, by Esc, or by a second `P` — same three exits as the
    /// knife (a mode you cannot leave is a trap).
    pub(crate) probe_armed: bool,
    /// The node whose output the probe is reading (its readout + sparkline draw
    /// beside the card). `None` = no probe.
    pub(crate) probe: Option<u32>,
    /// `K` armed the knife: the NEXT left-drag on the canvas slices wires instead
    /// of rubber-band selecting. Disarmed by the stroke itself, by Esc, or by a
    /// second `K` — a mode that cannot be left is a trap, so it has three exits.
    pub(crate) knife_armed: bool,
}
