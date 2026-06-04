use super::*;
use crate::interaction::{HitIndex, InteractiveState, PainterLayerDrop};
use crate::widget::{
    ButtonState, CheckboxState, CheckboxValue, SliderOrientation, SliderState, ToggleState,
};
use crate::zones::Rect;
use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_host::{KeyEvent, KeyKind, Modifiers, PointerEvent, PointerKind, PointerSource};

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn key(kc: u32, shift: bool) -> KeyEvent {
    KeyEvent {
        keycode: kc,
        modifiers: Modifiers {
            shift,
            ctrl: false,
            alt: false,
            meta: false,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    }
}

fn one_button_setup() -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(7),
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    let mut hits = HitIndex::new();
    hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
    (store, hits)
}

// -----------------------------------------------------------------
// Phase C — TextInput / NumberInput / Combobox / Dropdown
// -----------------------------------------------------------------

use crate::widget::TextInputState;

fn text_input(text: &str) -> InteractiveState {
    InteractiveState::TextInput {
        state: TextInputState::Normal,
        text: text.into(),
        caret: text.len(),
        selection_anchor: None,
    }
}

fn make_number_store(value: f64) -> WidgetStore {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(1),
        InteractiveState::NumberInput {
            state: TextInputState::Focused,
            value,
            buffer: super::super::format_number(value),
            caret: super::super::format_number(value).len(),
            last_committed: value,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(NodeId(1)));
    store
}

// -----------------------------------------------------------------
// BlenderColorPicker sub-control dispatch (B4 fix)
// -----------------------------------------------------------------

fn blender_picker_setup() -> (WidgetStore, HitIndex) {
    use crate::interaction::BlenderHitKind;
    use crate::widget::{ChannelMode, InterpolationMode};
    use ph2d_tokens::ColorValue;

    let mut store = WidgetStore::with_capacity(32);
    store.register(
        NodeId(100),
        InteractiveState::BlenderPicker {
            value: ColorValue::from_rgba8(128, 64, 32, 255),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Perceptual,
            active_palette: 0,
            hsv_h: 0.07,
            hsv_s: 0.75,
        },
    );
    // Seed the picker's palette so swatch clicks have something
    // to read (the default 12 colors from `default_palette`).
    store.init_blender_palette(
        NodeId(100),
        crate::widget::default_palette().swatches.clone(),
    );
    // Channel slider shims (0..3 = R, G, B, A).
    for idx in 0u8..4 {
        store.register(
            NodeId(200 + idx as u64),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::ChannelSlider(idx),
            },
        );
    }
    // Interpolation toggle shims.
    store.register(
        NodeId(210),
        InteractiveState::BlenderHit {
            parent: NodeId(100),
            kind: BlenderHitKind::InterpolationLinear,
        },
    );
    store.register(
        NodeId(211),
        InteractiveState::BlenderHit {
            parent: NodeId(100),
            kind: BlenderHitKind::InterpolationPerceptual,
        },
    );
    // Channel mode toggle shims.
    store.register(
        NodeId(212),
        InteractiveState::BlenderHit {
            parent: NodeId(100),
            kind: BlenderHitKind::ChannelRgb,
        },
    );
    store.register(
        NodeId(213),
        InteractiveState::BlenderHit {
            parent: NodeId(100),
            kind: BlenderHitKind::ChannelHsv,
        },
    );
    // Palette swatch shims.
    for swatch in 0u8..4 {
        store.register(
            NodeId(220 + swatch as u64),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::PaletteSwatch(swatch),
            },
        );
    }
    let mut hits = HitIndex::new();
    // Channel slider track rects — painter now registers only the
    // inner track (no label/value chip), so x=0..110 covers the
    // interactive region directly.
    for idx in 0u8..4 {
        hits.register(
            NodeId(200 + idx as u64),
            Rect::new(0.0, idx as f32 * 30.0, 110.0, 22.0),
        );
    }
    // Toggle half-rects.
    hits.register(NodeId(210), Rect::new(0.0, 200.0, 100.0, 28.0));
    hits.register(NodeId(211), Rect::new(100.0, 200.0, 100.0, 28.0));
    hits.register(NodeId(212), Rect::new(0.0, 240.0, 100.0, 28.0));
    hits.register(NodeId(213), Rect::new(100.0, 240.0, 100.0, 28.0));
    // Swatch rects.
    for swatch in 0u8..4 {
        hits.register(
            NodeId(220 + swatch as u64),
            Rect::new(swatch as f32 * 30.0, 300.0, 24.0, 24.0),
        );
    }
    (store, hits)
}

// ── Multi-line click mapping (TextArea) ────────────────────────────────

fn textarea_setup(initial: &str) -> (WidgetStore, HitIndex, Rect) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(42),
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Normal,
            text: initial.to_string(),
            caret: 0,
            selection_anchor: None,
        },
    );
    let rect = Rect::new(100.0, 200.0, 240.0, 60.0);
    let mut hits = HitIndex::new();
    hits.register(NodeId(42), rect);
    (store, hits, rect)
}

// ── Combobox clear-✕ button ────────────────────────────────────────────

fn combobox_setup(initial_query: &str) -> (WidgetStore, HitIndex, Rect) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(55),
        InteractiveState::Combobox {
            state: crate::widget::ComboboxState::Normal,
            open: false,
            query: initial_query.to_string(),
            caret: initial_query.len(),
            selection_anchor: None,
        },
    );
    let rect = Rect::new(50.0, 100.0, 240.0, 32.0);
    let mut hits = HitIndex::new();
    hits.register(NodeId(55), rect);
    (store, hits, rect)
}

// ── NumberInput stepper buttons ────────────────────────────────────────

fn number_input_setup(initial: f64) -> (WidgetStore, HitIndex, Rect) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(77),
        InteractiveState::NumberInput {
            state: crate::widget::TextInputState::Normal,
            value: initial,
            buffer: super::super::format_number(initial),
            caret: 0,
            last_committed: initial,
            selection_anchor: None,
        },
    );
    let rect = Rect::new(0.0, 0.0, 80.0, 28.0);
    let mut hits = HitIndex::new();
    hits.register(NodeId(77), rect);
    (store, hits, rect)
}

/// Read NumberInput value via the store accessor — avoids
/// boilerplate in the M14.A drag tests below.
fn read_value(store: &WidgetStore, id: NodeId) -> f64 {
    store.number_value(id).expect("NumberInput value")
}

fn meta_key(kc: u32) -> KeyEvent {
    KeyEvent {
        keycode: kc,
        modifiers: Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: true,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    }
}

fn focused_text_input(text: &str, caret: usize, anchor: Option<usize>) -> WidgetStore {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(50),
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text: text.to_string(),
            caret,
            selection_anchor: anchor,
        },
    );
    store.set_focus(Some(NodeId(50)));
    store
}

mod clipboard;
mod curve;
mod inputs;
mod number_drag;
mod widgets;
