//! ⭐ **A LIGAÇÃO CURVA mantém o número e o thumb JUNTOS** — pelas três portas que mexem num
//! deles: digitar e confirmar, arrastar a pista, e um valor fora da faixa.
//!
//! Irmão do `number_input_mapped_link` (o mapa afim); aqui o mesmo mapa com o expoente do
//! `slider_curve` por cima. A faixa é a do raio da escultura (`1..5000` px, curva `3`), que é
//! quem a pediu (report de 2026-09-16: *«O radius máximo permitido é pouco»*).
//!
//! ⚠️ **Cada metade reprova sozinha** se a curva faltar num dos quatro sítios do store: o commit
//! (chip → thumb), o arrasto (thumb → chip), o `set_slider_value` programático (thumb → chip), e a
//! re-sincronização de um valor fora da faixa, que tem de medir a saturação na fracção LINEAR.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::dispatch::keymap::{KEY_BACKSPACE, KEY_ENTER};
use ph2d_editor_core::interaction::{
    HitIndex, InteractiveState, WidgetStore, dispatch_key, dispatch_pointer, dispatch_text_input,
    pista_para_fracao,
};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, TextInputState};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{KeyEvent, KeyKind, Modifiers, PointerEvent, PointerKind, PointerSource};

const SLIDER: NodeId = NodeId(1);
const CHIP: NodeId = NodeId(2);
/// `valor = 1 + 4999 · pista³` — a faixa do raio.
const SCALE: f32 = 4999.0;
const OFFSET: f32 = 1.0;
const CURVA: f32 = 3.0;

fn par() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(8);
    store.register(
        SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Focused,
            value: 1.0,
            buffer: "1.000".to_string(),
            caret: 5,
            last_committed: 1.0,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(CHIP));
    store.link_slider_number_curved(SLIDER, CHIP, SCALE, OFFSET, CURVA);
    store
}

fn digita(store: &mut WidgetStore, texto: &str) {
    let arena = Bump::new();
    let tecla = |kc| KeyEvent {
        keycode: kc,
        modifiers: Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    };
    for _ in 0..8 {
        let _ = dispatch_key(store, tecla(KEY_BACKSPACE), &arena);
    }
    for ch in texto.chars() {
        let _ = dispatch_text_input(store, ch, &arena);
    }
    let _ = dispatch_key(store, tecla(KEY_ENTER), &arena);
}

fn thumb(store: &WidgetStore) -> f32 {
    store.slider(SLIDER).expect("slider").1
}

fn chip(store: &WidgetStore) -> f64 {
    store.number_input(CHIP).expect("chip").1
}

#[test]
fn a_curved_link_is_registered_and_the_linear_one_is_untouched() {
    let store = par();
    assert_eq!(store.linked_slider_curve(CHIP), CURVA);
    assert_eq!(
        store.linked_slider_mapping(CHIP),
        (SCALE, OFFSET),
        "o mapa afim mudou de forma"
    );
    // ⚠️ **A curva sozinha também é uma ligação**: um mapa afim identidade com expoente tem de
    // ser guardado — a mutação que só guardava mapas afins não-identidade sobreviveu à 1.ª
    // redacção deste gate, que só usava a faixa do raio.
    let mut so_curva = par();
    so_curva.link_slider_number_curved(SLIDER, CHIP, 1.0, 0.0, CURVA);
    assert_eq!(
        so_curva.linked_slider_curve(CHIP),
        CURVA,
        "a curva sobre a identidade foi descartada"
    );
    let mut linear = par();
    linear.link_slider_number_curved(SLIDER, CHIP, 1.0, 0.0, 1.0);
    assert_eq!(linear.linked_slider_curve(CHIP), 1.0);
    assert_eq!(
        linear.linked_slider_mapping(CHIP),
        (1.0, 0.0),
        "a identidade deixou de ser a de sempre"
    );
}

/// Digitar `125` põe o thumb onde a CURVA diz, não onde o mapa linear diria (`0,0248`).
#[test]
fn typing_a_value_puts_the_thumb_on_the_curve() {
    let mut store = par();
    digita(&mut store, "125");
    let esperado = ((125.0 - OFFSET) / SCALE).powf(CURVA.recip());
    assert!(
        (thumb(&store) - esperado).abs() < 1e-5,
        "thumb {} contra {esperado}",
        thumb(&store)
    );
    assert!(
        (chip(&store) - 125.0).abs() < 1e-9,
        "o numero digitado mudou: {}",
        chip(&store)
    );
}

/// Arrastar a pista a meio mostra `1 + 4999/8` no número — pelas duas portas que escrevem o thumb.
#[test]
fn moving_the_thumb_shows_the_value_on_the_curve() {
    let esperado = f64::from(OFFSET + SCALE * pista_para_fracao(0.5, CURVA));
    assert!(
        (esperado - 625.875).abs() < 1e-3,
        "a curva nao e' o cubo: {esperado}"
    );

    let mut store = par();
    let mut hits = HitIndex::new();
    hits.register(SLIDER, Rect::new(0.0, 0.0, 100.0, 30.0));
    let pointer = PointerEvent {
        x: 50.0,
        y: 15.0,
        pressure: 1.0,
        kind: PointerKind::Down,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    };
    let _ = dispatch_pointer(&mut store, &hits, pointer, &Bump::new());
    assert!(
        (chip(&store) - esperado).abs() < 1e-3,
        "arrasto: {} contra {esperado}",
        chip(&store)
    );

    let mut store = par();
    store.set_slider_value(SLIDER, 0.5);
    assert!(
        (chip(&store) - esperado).abs() < 1e-3,
        "programatico: {} contra {esperado}",
        chip(&store)
    );
}

/// Um valor acima da faixa satura o thumb no fim e devolve o número ao TOPO da faixa — a saturação
/// mede-se na fracção linear, e a curva leva `1` em `1`.
#[test]
fn a_value_past_the_range_saturates_at_the_top() {
    let mut store = par();
    digita(&mut store, "99999");
    assert!(
        (thumb(&store) - 1.0).abs() < 1e-6,
        "thumb {}",
        thumb(&store)
    );
    assert!(
        (chip(&store) - f64::from(OFFSET + SCALE)).abs() < 1e-3,
        "chip {}",
        chip(&store)
    );
}
