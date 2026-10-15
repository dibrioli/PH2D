//! A pré-edição do IME ([`crate::ime`]): a composição mostra-se no caret, sublinhada, e nunca entra
//! no valor antes do `Commit`. As sequências são as MEDIDAS no winit 0.30.13 (tabela no
//! [`crate::ime`]): com IME = `Preedit` → `Preedit("")` → `Commit`; sem IME = `Key::Dead` e depois o
//! `KeyEvent.text` já composto.

use super::*;
use crate::ime::ComposeKey;
use crate::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;

const FIELD: NodeId = NodeId(1);
const OTHER: NodeId = NodeId(2);

fn focused_field(value: &str) -> WidgetStore {
    let mut store = WidgetStore::with_capacity(4);
    store.register(FIELD, text_input(value));
    store.register(OTHER, text_input("x"));
    store.set_focus(Some(FIELD));
    store
}

/// O que a pintura do campo focado produz: `(glifos, segmentos de caminho, caret)`.
fn paint(store: &WidgetStore) -> (usize, u32, Rect) {
    crate::ime::begin_frame(store);
    let Some(InteractiveState::TextInput { text, caret, .. }) = store.get(FIELD) else {
        panic!("o campo é um TextInput");
    };
    let input = TextInput::new(FIELD, "").state(TextInputState::Focused);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let rect = Rect::new(0.0, 0.0, 400.0, 32.0); // LITERAL-PX-OK: um campo largo de teste
    paint_text_input_with_buffer(
        &input,
        Some(text),
        Some(*caret),
        None,
        rect,
        &mut scene,
        &mut ts,
        Theme::default(),
    );
    let enc = scene.inner().encoding();
    let caret = crate::ime::caret_area().expect("o campo focado publica o caret");
    (enc.resources.glyphs.len(), enc.n_path_segments, caret)
}

#[test]
fn ime_preedit_shows_at_the_caret_and_inserts_nothing() {
    let mut store = focused_field("ab");
    let (glyphs, segments, caret) = paint(&store);

    dispatch_preedit(&mut store, "´", None);

    assert_eq!(
        store.text(FIELD),
        Some("ab"),
        "a pré-edição entrou no valor"
    );
    assert_eq!(store.composition().map(|c| c.text.as_str()), Some("´"));
    let (g, s, c) = paint(&store);
    assert_eq!(g, glyphs + 1, "o acento pendente não foi pintado");
    assert!(s > segments, "a pré-edição não foi sublinhada");
    assert!(c.x > caret.x, "o caret não passou para depois do acento");
}

#[test]
fn ime_empty_preedit_drops_the_composition() {
    let mut store = focused_field("ab");
    let before = paint(&store);
    dispatch_preedit(&mut store, "´", None);
    dispatch_preedit(&mut store, "", None);
    assert_eq!(store.composition(), None);
    assert_eq!(paint(&store), before, "a composição ainda se pinta");
}

#[test]
fn ime_commit_inserts_exactly_once_and_ends_the_composition() {
    let arena = Bump::new();
    // Wayland text-input-v3 / XIM: `Preedit("´")`, `Preedit("")`, `Commit("á")`.
    let mut store = focused_field("ab");
    dispatch_preedit(&mut store, "´", None);
    dispatch_preedit(&mut store, "", None);
    let evts = dispatch_text_input(&mut store, 'á', &arena);
    assert_eq!(evts, &[WidgetEvent::TextChanged(FIELD)]);
    assert_eq!(store.text(FIELD), Some("abá"));
    // E um `Commit` sem o `Preedit("")` antes também acaba a composição.
    dispatch_preedit(&mut store, "~", None);
    let _ = dispatch_text_input(&mut store, 'ã', &arena);
    assert_eq!(store.text(FIELD), Some("abáã"));
    assert_eq!(store.composition(), None);
}

#[test]
fn ime_focus_change_mid_composition_drops_it_and_keeps_the_value() {
    let mut store = focused_field("ab");
    dispatch_preedit(&mut store, "´", None);
    store.set_focus(Some(OTHER));
    assert_eq!(store.composition(), None);
    store.set_focus(Some(FIELD));
    assert_eq!(
        store.composition(),
        None,
        "a composição só estava escondida"
    );
    assert_eq!(store.text(FIELD), Some("ab"));
    assert_eq!(store.text(OTHER), Some("x"));
}

/// Sem IME (a sessão do dono): `´`, Shift, `A` ⇒ `Key::Dead`, um modificador, e o `Á` composto.
#[test]
fn a_dead_key_composes_and_the_composed_char_lands_once() {
    let arena = Bump::new();
    let mut store = focused_field("ab");
    assert!(!dispatch_compose_key(&mut store, ComposeKey::Dead('´')));
    assert_eq!(store.composition().map(|c| c.text.as_str()), Some("´"));
    assert!(!dispatch_compose_key(&mut store, ComposeKey::Modifier));
    assert!(
        store.composition().is_some(),
        "o Shift desfez a tecla morta"
    );
    assert!(!dispatch_compose_key(&mut store, ComposeKey::Other));
    let _ = dispatch_text_input(&mut store, 'Á', &arena);
    assert_eq!(store.text(FIELD), Some("abÁ"));
    assert_eq!(store.composition(), None);
}

/// `´` + `q` não compõe: o xkb cancela e o `q` chega sem texto — o acento pendente tem de sumir.
#[test]
fn a_cancelled_dead_key_leaves_nothing_behind() {
    let mut store = focused_field("ab");
    dispatch_compose_key(&mut store, ComposeKey::Dead('´'));
    dispatch_compose_key(&mut store, ComposeKey::Other);
    assert_eq!(store.composition(), None);
    assert_eq!(store.text(FIELD), Some("ab"));
}

#[test]
fn escape_cancels_a_pending_dead_key_and_keeps_the_field() {
    let mut store = focused_field("ab");
    dispatch_compose_key(&mut store, ComposeKey::Dead('~'));
    assert!(dispatch_compose_key(&mut store, ComposeKey::Escape));
    assert_eq!(store.composition(), None);
    assert_eq!(store.focus_id(), Some(FIELD));
    assert!(
        !dispatch_compose_key(&mut store, ComposeKey::Escape),
        "sem tecla morta pendente o Escape é do campo"
    );
}

/// Uma composição do IME é do IME: uma tecla que ele deixou passar não a desfaz.
#[test]
fn an_ime_composition_survives_keys_the_ime_let_through() {
    let mut store = focused_field("ab");
    dispatch_preedit(&mut store, "にほ", Some((3, 3)));
    assert!(!dispatch_compose_key(&mut store, ComposeKey::Other));
    assert!(!dispatch_compose_key(&mut store, ComposeKey::Escape));
    assert_eq!(store.composition().map(|c| c.text.as_str()), Some("にほ"));
}

#[test]
fn a_number_input_never_composes() {
    let mut store = make_number_store(5.0);
    store.set_focus(Some(NodeId(1)));
    dispatch_preedit(&mut store, "´", None);
    dispatch_compose_key(&mut store, ComposeKey::Dead('´'));
    assert_eq!(store.composition(), None);
}
