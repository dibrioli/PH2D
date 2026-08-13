//! **O que um estado de widget É** — irmão do `mod.rs` pelo teto de 700 LOC, e o corte é o que a
//! pasta já pratica em nove módulos: aqui fica o vocabulário (o `InteractiveState` e a paleta
//! nomeada), lá o que o store FAZ com ele.

use super::super::flip_strip::FlipStripHitKind;
use super::super::types::{BlenderHitKind, GraphHitKind, TimelineHitKind};
use crate::widget::{
    ButtonState, ChannelMode, CheckboxState, CheckboxValue, ColorPickerMode, ComboboxState,
    DropdownState, InterpolationMode, ListItemState, SliderOrientation, SliderState, TagState,
    TextInputState, ToggleState,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::ColorValue;

/// One per-widget state slot. Variants mirror the widget kinds in
/// `crate::widget::*`; mappings to the original widget's state enum
/// are 1:1 so `paint_X` keeps reading the same field names.
#[derive(Clone, Debug, PartialEq)]
pub enum InteractiveState {
    Button {
        state: ButtonState,
    },
    Toggle {
        state: ToggleState,
        on: bool,
    },
    Slider {
        state: SliderState,
        value: f32,
        orientation: SliderOrientation,
    },
    Checkbox {
        state: CheckboxState,
        value: CheckboxValue,
    },
    Radio {
        state: ButtonState,
        selected_index: usize,
    },
    Tag {
        state: TagState,
    },
    Tabs {
        selected: usize,
    },
    Dropdown {
        state: DropdownState,
        open: bool,
        selected_index: Option<usize>,
    },
    Combobox {
        state: ComboboxState,
        open: bool,
        query: String,
        caret: usize,
        /// Same semantics as `TextInput::selection_anchor`.
        selection_anchor: Option<usize>,
    },
    TextInput {
        state: TextInputState,
        text: String,
        caret: usize,
        /// `None` = collapsed (no selection); `Some(anchor)` = the
        /// selection covers `[min(anchor, caret), max(anchor, caret)]`.
        /// Set by double-click ("select all") and by Shift+Arrow; any
        /// non-shift cursor motion or text mutation collapses it.
        selection_anchor: Option<usize>,
    },
    NumberInput {
        state: TextInputState,
        value: f64,
        /// Mirror of `value` formatted as a string while the input is
        /// not focused; the user's in-progress edit while it is. Pre-
        /// allocated by the caller via [`InteractiveState::number_input`]
        /// so dispatch never grows the String at construction time.
        buffer: String,
        caret: usize,
        /// Snapshot of `value` taken when focus arrives — restored on
        /// Escape or on Blur with an unparsable buffer.
        last_committed: f64,
        /// Same semantics as `TextInput::selection_anchor`.
        selection_anchor: Option<usize>,
    },
    ListItem {
        state: ListItemState,
        selected: bool,
    },
    TreeView {
        // Tree expand/select state stays on the TreeView struct — the
        // store only carries hot/active flags. Value-bearing reads go
        // through the widget directly.
        last_focused_index: Option<usize>,
    },
    ColorPicker {
        mode: ColorPickerMode,
        rgba: [u8; 4],
    },
    /// `BlenderColorPicker` retained state. Painted by
    /// `paint_blender_color_picker_with_store`; mutated by clicks
    /// on registered wheel/value/swatch/segmented sub-rect ids.
    BlenderPicker {
        value: ColorValue,
        channel_mode: ChannelMode,
        interpolation: InterpolationMode,
        active_palette: usize,
        /// Retained HSV hue (0..1). Used by the SV-rect/hue-strip
        /// painters when `value.rgba` collapses to gray (S=0) or
        /// black (V=0) and would otherwise lose the user's chosen
        /// hue. Updated whenever a pick path knows the canonical H.
        ///
        /// **Don't read hue from `rgba_to_hsv(value.rgba)` directly**
        /// in painters / dispatchers — for dark or gray colors it
        /// returns 0 (red) and the SV cursor / hue thumb teleport.
        /// See `docs/UI_Bugs/README.md` §4.1.
        hsv_h: f32,
        /// Retained HSV saturation (0..1). Same role as `hsv_h` —
        /// preserved across V→0 transitions where round-tripping
        /// through RGBA loses the value.
        hsv_s: f32,
        /// The selected Color-Harmonies scheme (view-state; partners are DERIVED). Default `None`.
        harmony: crate::widget::Harmony,
    },
    /// Sub-control hit shim: pointing at a sub-rect of a parent
    /// BlenderPicker. The dispatcher uses `kind` to route the click
    /// into the correct widget-side mutation.
    BlenderHit {
        parent: NodeId,
        kind: BlenderHitKind,
    },
    /// 2-D draggable control point of a Curves/Levels-style editor (W4 §3). The
    /// dispatcher normalizes the pointer within `canvas` (the plotting area, NOT
    /// the small grab rect) to `(x, y)` in `[0, 1]` (y inverted: canvas top =
    /// 1.0 = high output) and stashes `(parent, channel, index, x, y)` via
    /// [`WidgetStore::set_curve_point_drag`] for the panel to read each frame and
    /// forward to its tool (`PainterTool::set_curve_point`). Mirrors
    /// [`InteractiveState::BlenderHit`]'s pointer-in-rect routing.
    CurvePoint {
        parent: NodeId,
        /// `0` = master, `1` = R, `2` = G, `3` = B (the editor's active channel).
        channel: u8,
        /// Index of the dragged point within the channel's control set.
        index: u8,
        /// The curve plotting area, in the same coords as the hit rect — the
        /// drag normalizes against THIS, not the handle's small grab rect.
        canvas: Rect,
    },
    /// Motion Nodes M0.T2 — a hit target inside a graph editor surface (node,
    /// socket, wire, backdrop, or the background itself). The motion-graph panel
    /// sets one per target as it paints, mirrored by a [`super::HitIndex`]
    /// registration. Dispatch captures it on Down and streams [`GraphGesture`]s
    /// (Begin/Update/End/Click) to the panel via
    /// [`WidgetStore::push_graph_gesture`] — it never interprets `kind`
    /// (editor-core knows no graph semantics). `canvas` is the surface's rect
    /// (for the panel's own coordinate mapping; the pointer position is carried
    /// raw in the gesture). Mirrors [`InteractiveState::CurvePoint`]'s
    /// pointer-in-rect routing shim.
    GraphSurface {
        parent: NodeId,
        kind: GraphHitKind,
        canvas: Rect,
    },
    /// A hit target inside the general timeline's dope-sheet surface (a key
    /// diamond or an empty lane). Mirror of [`InteractiveState::GraphSurface`]:
    /// dispatch captures it on Down and streams [`TimelineGesture`]s to the
    /// timeline panel via [`WidgetStore::push_timeline_gesture`]; editor-core
    /// never interprets `kind`. `canvas` is the dope-sheet rect (for the panel's
    /// own coordinate mapping; the pointer position is carried raw in the
    /// gesture).
    TimelineSurface {
        parent: NodeId,
        kind: TimelineHitKind,
        canvas: Rect,
    },
    /// Um alvo dentro da **tira de frames do Flip** (o corpo de uma célula ou a borda que
    /// define o hold). Terceira da família `*Surface`: o dispatch captura no Down e
    /// despeja [`FlipStripGesture`](super::FlipStripGesture)s para o painel da tira; o
    /// editor-core nunca interpreta `kind`.
    ///
    /// **Sem `canvas`, de propósito** — os dois irmãos o carregam para o zoom/pan ancorado
    /// da roda, e a tira *sempre cabe* (a escala é derivada do vão, sem scroll nem estado
    /// de pan escondido — `docs/Flip/05 §6`). Um retângulo que ninguém lê é um retângulo
    /// que envelhece calado.
    FlipStripSurface {
        parent: NodeId,
        kind: FlipStripHitKind,
    },
    Modal {
        // Open/closed lives on the host; store only tracks ESC->dismiss intent.
        dismissing: bool,
    },
    /// Generic chrome with a focusable hit rect but no interactive
    /// state to carry between frames (e.g., section headers,
    /// hierarchy header add-button).
    Plain,
}

/// One named colour palette in a [`InteractiveState::BlenderPicker`]'s set: a display name + its
/// swatches. A picker holds a `Vec<NamedPalette>` (≥1); the active index lives in the picker state
/// (`active_palette`). The "+ swatch", import/export and CRUD dispatch paths mutate the active one.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedPalette {
    /// Display name shown on the palette tab.
    pub name: String,
    /// Straight-RGBA swatches in author order.
    pub swatches: Vec<ColorValue>,
}
