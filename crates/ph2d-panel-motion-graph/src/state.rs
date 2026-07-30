//! Ephemeral editor state (Motion Nodes M1.E4/E5) — pan / zoom / selection /
//! in-progress drag. Non-undoable (only doc mutations, via `GraphIntent`, are).
//! Owned by the typed panel registry; passed `&mut` into `paint`.

use crate::snapshot::PortChoice;
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
        /// **The input this wire was PULLED OFF** (F2, doc 45) — set when the gesture began
        /// on an occupied input socket rather than on an output.
        ///
        /// A grabbed wire-end MOVES; it does not copy. So the drop emits `MoveWireEnd` (one
        /// undo step: unplug the old input, plug the new) instead of `Connect`, and the
        /// painter hides the edge being dragged — a wire that is still drawn into the socket
        /// you are visibly pulling it out of is a wire that has not moved.
        detached: Option<(u32, u16)>,
    },
    /// Dragging a wire BACKWARDS out of an empty input socket, hunting for an output to
    /// feed it (F2, doc 45). `target` is the OUTPUT socket under the pointer (with local
    /// type-compatibility), the mirror of `DrawWire`'s input target.
    ///
    /// Both directions exist because a graph is read in both: sometimes you know what a node
    /// needs and go looking for it, and an editor that only lets you wire forwards makes you
    /// cross the canvas to say so.
    DrawWireBack {
        to_node: u32,
        to_port: u16,
        cur: (f32, f32),
        target: Option<(u32, u16, bool)>,
    },
    /// Dragging the add-menu's scrollbar thumb. `grab` is where INSIDE the thumb the cursor
    /// seized it — without that, the thumb jumps to put its TOP edge under the cursor the moment
    /// you touch it, which is the scrollbar bug everyone has met.
    MenuScroll { grab: f32 },
}

/// The inline rename box: F2 opened it over something, and it is waiting for a name (doc 61).
///
/// The TEXT is not here — it lives in the `WidgetStore`, which owns the buffer, the caret and the
/// selection. This is only what the box is *pointing at* and whether it has claimed the keyboard
/// yet (once, on the frame it opens; re-claiming every frame would fight anything else the artist
/// touches, and re-seeding would stomp what they are typing).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Rename {
    pub target: crate::snapshot::RenameTarget,
    /// What the thing was called when the box opened — the string the field is seeded with.
    pub seed: String,
    pub opened: bool,
}

/// An open popup (E7). Ephemeral — never undoable. The frame (panel, scrollbar,
/// rows) is one widget; WHAT it lists and what a pick MEANS is [`MenuBody`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Menu {
    /// How far the list is scrolled, in px. The library has 86 node types and a popup is not
    /// allowed to run off the screen (Enio), so the panel is capped and the list scrolls inside it.
    pub scroll: f32,
    /// Top-left of the popup panel, screen space (the R-click point, clamped at
    /// paint so the list stays on-canvas).
    pub screen: (f32, f32),
    /// Graph-space point the chosen node lands at (the R-click point mapped
    /// through the view — stable under a later pan/zoom while the menu is open).
    pub spawn: (f32, f32),
    /// **What the artist has typed** — mirrored from the search field's `TextInput` at the
    /// top of every `process`, so the store owns the buffer (caret, selection, keyboard) and
    /// this is only ever a READ of it. Two copies of the same string, edited on both sides,
    /// is the bug that field is famous for.
    pub query: String,
    /// The search field claims focus exactly ONCE, on the frame the menu opens — re-claiming
    /// it every frame would fight anything else the artist focuses, and re-seeding the buffer
    /// would stomp what they are typing. (Same flag, same reason, as the timeline's inline
    /// rename.)
    pub opened: bool,
    pub body: MenuBody,
}

impl Menu {
    /// **Only the library has a search field.** Eighty-eight node types is a list you have to
    /// search; eight tints and a handful of ports are lists you READ. A field on those two is a
    /// box that filters nothing and *takes the keyboard while it does it* — which is what it had
    /// been doing on the card-ports menu since the search landed (doc 59), quietly.
    pub(crate) fn has_search(&self) -> bool {
        matches!(self.body, MenuBody::Library { .. })
    }
}

/// What the popup is a list OF.
///
/// An enum rather than two optional fields: the two bodies are alternatives, and a
/// struct that can hold both can hold neither meaningfully — the popup would have to
/// guess which question it was asking.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MenuBody {
    /// **The node library.** Plain `A` / R-click lists all 86 types.
    ///
    /// `connect_from` is **smart-connect** (F2): the output socket a wire was dragged
    /// FROM and dropped on empty canvas. When set, the popup lists only the node types
    /// with an input this wire can feed, and picking one both creates it AND wires it —
    /// the gesture already said what it wanted, so making the artist draw the wire a
    /// second time would be asking twice.
    ///
    /// `splice` is the wire (its unique target `(to_node, to_port)`) the R-press landed ON:
    /// picking a node then **inserts it into that wire** — source → new → target, one undo
    /// step, the shell validating and refusing if the type does not fit (the generalisation
    /// of the double-click reroute to any chosen node). Mutually exclusive with `connect_from`
    /// (you are either dragging a loose end or aiming at an existing wire, never both).
    Library {
        connect_from: Option<(u32, u16)>,
        splice: Option<(u32, u16)>,
    },
    /// **The eight tints a backdrop can take** (doc 62) — R-click on a backdrop's header.
    ///
    /// A backdrop's colour is the only thing about it the artist could not change: the tint
    /// cycled by id at birth and stayed there for good. The intent to set it EXISTED and was
    /// handled by the shell; nothing emitted it — the same dead half-gesture the rename found
    /// (doc 61 §2), one shelf down.
    BackdropTints {
        /// The backdrop being re-tinted.
        backdrop: u32,
        /// Its tint right now, so the palette can show you which one you are on.
        current: u8,
    },
    /// **The ports hidden inside a collapsed card** (doc 57 §5) — a wire was dropped on
    /// a card's BODY, and the card exposes no socket for it.
    ///
    /// A card's sockets ARE the wires that already cross its boundary (derived, never
    /// declared), so a *new* wire into a group has nowhere to land: the socket it wants
    /// does not exist yet, and cannot, until the wire exists. Something has to name the
    /// port inside — and only the artist knows which of the group's free ports they
    /// meant. So the drop asks. On the pick, the wire connects to the REAL port inside,
    /// and the card grows the socket **by derivation**, from the edge that now crosses it.
    ///
    /// `rows` is captured at DROP time, not re-derived per frame: the list must not shift
    /// under a cursor that is already travelling toward a row.
    CardPorts {
        rows: Vec<PortChoice>,
        /// The end already in hand — the outside node's port the wire was drawn from
        /// (`forward`) or hunting for (`!forward`).
        other: (u32, u16),
        /// The wire was drawn FORWARD, out of an output: it needs an INPUT inside, and
        /// the connection reads `other -> row`. Backwards, it reads `row -> other`.
        forward: bool,
        /// The input this wire's end was pulled OFF, when the gesture was a wire being
        /// MOVED rather than drawn (doc 45). The pick then moves it — one undo step,
        /// unplug and re-plug — instead of making a second copy of the same wire.
        ///
        /// Until the artist picks, the wire is still drawn where it was: it has not moved,
        /// and a wire drawn nowhere while a menu is open would be a wire the editor
        /// dropped on the floor.
        detach: Option<(u32, u16)>,
    },
}

/// Retained panel state.
#[derive(Default)]
pub struct MotionGraphPanelState {
    pub(crate) view: ViewState,
    /// `false` until the first paint auto-fits the graph (then user-controlled).
    pub(crate) fitted: bool,
    /// A MANUAL fit (the chip or `F`) asked to frame the SELECTION, not the whole
    /// graph. Set only by [`Self::request_fit`] when there is a selection; the
    /// auto-fits (first sight, level change) leave it `false`, so they always frame
    /// everything. Consumed and reset by the paint's fit pass.
    pub(crate) fit_selection: bool,
    /// Selected node ids (`NodeId.0`).
    pub(crate) selected: BTreeSet<u32>,
    /// The selected backdrop, if any. Mutually exclusive with `selected`: the
    /// params panel shows the properties of ONE subject, and a Delete must never
    /// be ambiguous about what it removes.
    pub(crate) selected_backdrop: Option<u32>,
    pub(crate) interaction: Interaction,
    /// Open add-node popup, or `None`. Opened by R-click on empty canvas / `A`;
    /// closed by picking a row, clicking away, or Esc.
    pub(crate) menu: Option<Menu>,
    /// **The open rename box** (doc 61), or `None`. Ephemeral — never undoable; what IS undoable
    /// is the name it commits.
    pub(crate) rename: Option<Rename>,
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
    /// **The level this panel last painted** (doc 57) — mirrored from the snapshot,
    /// never authored here (the SHELL owns the level, so that an undo which dissolves
    /// the group you are standing in can put you back on solid ground). It exists for
    /// one reason: to notice a level CHANGE and re-fit, so entering a group never
    /// lands the artist on a canvas they cannot see.
    pub(crate) level: Option<u32>,
    /// The input a wire's end was just pulled off, held for the ONE frame between
    /// the drop and the shell's answer (doc 45.1).
    ///
    /// The shell applies the panel's intents at the top of the next frame and only
    /// then republishes the snapshot — so the frame in which the drop happens still
    /// paints from a snapshot where the wire is plugged in where it was. Without
    /// this, the released end **snapped back to its old socket for one frame** before
    /// vanishing: a wire the artist had just torn out, drawn back where it no longer
    /// is. Keeping the suppression alive across that boundary means the end is never
    /// drawn anywhere it is not.
    ///
    /// Cleared at the top of the next `interact::apply` — by then the snapshot is the
    /// shell's answer, whatever it was, and it is the truth to paint. (A REFUSED move
    /// therefore shows the original wire again, which is exactly right: it never moved.)
    pub(crate) pending_detach: Option<(u32, u16)>,
}

impl MotionGraphPanelState {
    /// The view (pan/zoom), for the seam gates — see `menu_for_test`.
    pub(crate) fn view_for_test(&self) -> ViewState {
        self.view
    }

    /// The open menu, for the seam gates (the state is private and a paint-driven test has no
    /// other way to see what the artist is looking at).
    pub(crate) fn menu_for_test(&self) -> Option<&Menu> {
        self.menu.as_ref()
    }

    /// **A new level is a new canvas** (doc 57). Adopt the level the shell published;
    /// if it CHANGED, drop the fit so the next paint re-frames.
    ///
    /// The fold moves nothing — a group's members sit exactly where they always sat —
    /// so a view zoomed in on the collapsed card would open the group showing a corner
    /// of it, or empty canvas. Blender and Houdini both re-frame on the way in.
    ///
    /// Returns whether the level changed (the paint does not care; the gate does).
    pub(crate) fn sync_level(&mut self, level: Option<u32>) -> bool {
        if self.level == level {
            return false;
        }
        self.level = level;
        self.fitted = false;
        true
    }

    /// **Request a re-frame** (the Fit chip / `F`) — the draw pass owns the fit math,
    /// this only asks for it. It frames the SELECTION when there is one and the whole
    /// graph otherwise: the universal node-editor `F` (Blender/Nuke/Houdini frame the
    /// selection), and the same "one gesture, two behaviours" the Backdrop chip uses.
    /// ONE door, so the chip and the key can never diverge on the rule.
    pub(crate) fn request_fit(&mut self) {
        self.fitted = false;
        self.fit_selection = !self.selected.is_empty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_or_leaving_a_level_re_fits_and_staying_put_does_not() {
        let mut st = MotionGraphPanelState {
            fitted: true,
            ..Default::default()
        };
        assert!(
            !st.sync_level(None),
            "already at the root: nothing happened"
        );
        assert!(st.fitted, "and the artist's pan/zoom is left alone");

        assert!(st.sync_level(Some(3)), "entered a group");
        assert!(!st.fitted, "so the next paint re-frames on its contents");

        st.fitted = true;
        assert!(!st.sync_level(Some(3)), "still in the same room");
        assert!(
            st.fitted,
            "a re-fit on every frame would fight the artist's zoom"
        );

        assert!(st.sync_level(None), "walked the breadcrumb back out");
        assert!(!st.fitted);
    }

    /// **Fit is selection-aware.** With a selection, a manual fit asks to frame it
    /// (the universal `F`); with nothing selected it frames the whole graph. Both
    /// drop `fitted` so the next paint re-frames. FALSIFIED by a `request_fit` that
    /// ignores the selection (always frames all, or always frames "selection").
    #[test]
    fn request_fit_frames_the_selection_only_when_there_is_one() {
        let mut st = MotionGraphPanelState {
            fitted: true,
            ..Default::default()
        };

        st.request_fit();
        assert!(!st.fitted, "an empty-selection fit still re-frames");
        assert!(!st.fit_selection, "nothing selected: frame the whole graph");

        st.selected.insert(7);
        st.fitted = true;
        st.request_fit();
        assert!(!st.fitted);
        assert!(
            st.fit_selection,
            "a selection is present: frame it, not everything"
        );
    }
}
