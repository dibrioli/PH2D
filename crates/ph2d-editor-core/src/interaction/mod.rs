//! Editor input pipeline + retained widget state (ADR-0024).
//!
//! Three pieces work together:
//!
//! - [`WidgetStore`] holds the per-widget interactive state
//!   (hovered, pressed, focused, drag value, text content) keyed by
//!   the same `NodeId` AccessKit uses. Pre-populated when a screen
//!   is constructed; never grows during the hot path.
//! - [`HitIndex`] maps screen coords back to the `NodeId` of the
//!   widget under the cursor. Rebuilt by the paint pass each frame
//!   (paint registers each rect as it emits geometry).
//! - [`dispatch_pointer`] / [`dispatch_key`] read the index, mutate
//!   the store, and emit [`WidgetEvent`]s into a per-frame `bumpalo`
//!   arena that the caller drains and resets.
//!
//! ## HR-3 (zero alloc on the hot path)
//!
//! The four mitigations from the ADR are wired here:
//!
//! 1. The store uses `HashMap::with_capacity(N)` and exposes only
//!    `register` (called at construction) — never `insert`. Mutations
//!    are `get_mut`, which never reallocates.
//! 2. [`HitIndex`] backs the rect list with [`smallvec::SmallVec`]
//!    inline up to 128 entries.
//! 3. Emitted events live in a caller-supplied `bumpalo::Bump` and
//!    are returned as `&'frame [WidgetEvent]`.
//! 4. [`WidgetEvent`] has no heap payload — value-bearing variants
//!    carry only the `NodeId`; the caller re-reads from the store.
//!
//! The bench `tests/interaction_no_alloc.rs` enforces this with
//! `dhat-rs`.

pub mod dispatch;
pub mod drag;
pub mod event;
pub mod hit;
pub mod state;
pub mod types;
pub mod util;

pub use dispatch::{
    KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_DELETE,
    KEY_ENTER, KEY_ESCAPE, KEY_KEY_A, KEY_KEY_C, KEY_KEY_D, KEY_KEY_F, KEY_KEY_K, KEY_KEY_P,
    KEY_KEY_V, KEY_KEY_X, KEY_SPACE, KEY_TAB, apply_clipboard_paste, dispatch_key,
    dispatch_pointer, dispatch_pointer_with_text, dispatch_text_input, dispatch_tick,
    dispatch_wheel,
};
pub use drag::{
    DRAG_RATE_X, DRAG_RATE_Y, DRAG_SHIFT_MUL, HierarchyDragState, LONG_PRESS_THRESHOLD_NS,
    NUMBER_INPUT_DRAG_THRESHOLD_PX, NumberInputDragState, NumberStepperHoldState,
    STEPPER_HOLD_INITIAL_DELAY_NS, STEPPER_REPEAT_INTERVAL_NS, ScrollbarDragAnchor,
};
pub use event::{PainterLayerDrop, WidgetEvent};
pub use hit::HitIndex;
pub use state::{InteractiveState, NamedPalette, WidgetStore};
pub use types::{
    BlenderHitKind, ContextMenuKind, ContextMenuRequest, GestureMods, GesturePhase, GraphGesture,
    GraphHitKind, GraphKey, GraphZoom, NoteData, PaletteIoKind, TIMELINE_EDGE_B, TIMELINE_EDGE_L,
    TIMELINE_EDGE_R, TIMELINE_EDGE_T, TL_NO_EASE_MODE, TimelineGesture, TimelineHitKind,
    TimelineInterpPick, TimelineWheel,
};
pub use util::{format_number, hsv_to_color_value};
