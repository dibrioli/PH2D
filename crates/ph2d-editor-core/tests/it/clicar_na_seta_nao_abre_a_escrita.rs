//! ⭐ **Clicar numa seta dá o passo e não abre a escrita** (F13). Quem abre a escrita num clique é o corpo
//! do campo e o ícone (`a_pega_abre_a_escrita`). O arrasto no corpo fecha ao largar; a seta, igual.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{
    HitIndex, InteractiveState, WidgetEvent, WidgetStore, dispatch_pointer,
};
use ph2d_editor_core::widget::NumberInput;
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::widget::card_field::CardFieldSpec;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

const ID: NodeId = NodeId(0x7713);
const CAMPO: Rect = Rect {
    x: 100.0,
    y: 100.0,
    w: 80.0,
    h: 28.0,
};

struct Mesa {
    store: WidgetStore,
    hits: HitIndex,
    relogio: u128,
}

impl Mesa {
    fn nova() -> Self {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            ID,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 5.0,
                buffer: "5".to_owned(),
                caret: 0,
                last_committed: 5.0,
                selection_anchor: None,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(ID, CAMPO);
        Self {
            store,
            hits,
            relogio: 1_000_000_000,
        }
    }

    /// O chip da fila ②: partilha o id com um campo de cartão e tem setas.
    fn da_fila() -> Self {
        let mut m = Self::nova();
        m.store.mark_card_field(
            ID,
            CardFieldSpec {
                step: 1.0,
                scrub: 1.0,
                default: None,
            },
        );
        m.store.set_tool_row_slots(vec![(ID, CAMPO)]);
        m
    }

    fn mao(&mut self, kind: PointerKind, (x, y): (f32, f32)) -> Vec<WidgetEvent> {
        let arena = Bump::new();
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: self.relogio,
        };
        dispatch_pointer(&mut self.store, &self.hits, ev, &arena).to_vec()
    }

    fn clique(&mut self, p: (f32, f32)) -> Vec<WidgetEvent> {
        self.relogio += 2_000_000_000;
        let mut e = self.mao(PointerKind::Down, p);
        e.extend(self.mao(PointerKind::Up, p));
        e
    }

    fn valor(&self) -> f64 {
        self.store.number_value(ID).expect("o campo")
    }
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn seta_de_cima() -> (f32, f32) {
    centro(NumberInput::new(ID, "", 5.0).up_rect(CAMPO))
}

fn seta_de_baixo() -> (f32, f32) {
    centro(NumberInput::new(ID, "", 5.0).down_rect(CAMPO))
}

fn corpo() -> (f32, f32) {
    (CAMPO.x + 4.0, CAMPO.y + CAMPO.h * 0.5)
}

/// *Contraprova: tirar a largada do foco do `pointer_up` ⇒ o clique na seta fica a escrever.*
#[test]
fn clicking_an_arrow_steps_and_does_not_open_typing() {
    for (mesa, nome) in [
        (Mesa::nova as fn() -> Mesa, "campo"),
        (Mesa::da_fila, "fila"),
    ] {
        for (seta, esperado) in [(seta_de_cima(), 6.0), (seta_de_baixo(), 4.0)] {
            let mut m = mesa();
            let ev = m.clique(seta);
            assert_eq!(m.valor(), esperado, "{nome}: a seta não deu o passo");
            assert_eq!(m.store.focus_id(), None, "{nome}: a seta abriu a escrita");
            assert!(
                ev.contains(&WidgetEvent::ValueChanged(ID)),
                "{nome}: o painel não soube do passo: {ev:?}"
            );
            // Quem abriu um gesto no `ValueChanged` fecha-o no `Blur`, como no arrasto.
            assert_eq!(
                ev.iter().filter(|e| **e == WidgetEvent::Blur(ID)).count(),
                1,
                "{nome}: {ev:?}"
            );
        }
    }
}

/// O clique no corpo continua a abrir a escrita: a seta é que não.
#[test]
fn clicking_the_body_still_opens_typing() {
    for mesa in [Mesa::nova as fn() -> Mesa, Mesa::da_fila] {
        let mut m = mesa();
        m.clique(corpo());
        assert_eq!(m.store.focus_id(), Some(ID));
        assert_eq!(m.valor(), 5.0);
    }
}

/// A escrever já, a seta dá o passo e a escrita continua aberta.
#[test]
fn an_arrow_on_a_field_already_typing_keeps_it_open() {
    let mut m = Mesa::nova();
    m.clique(corpo());
    let ev = m.clique(seta_de_cima());
    assert_eq!(m.valor(), 6.0);
    assert_eq!(m.store.focus_id(), Some(ID), "a seta fechou a escrita");
    assert!(!ev.contains(&WidgetEvent::Blur(ID)), "{ev:?}");
}

/// O candidato a arrasto nem nasce numa seta: a largada dela não passa pelo clique sem scrub.
#[test]
fn an_arrow_seeds_no_drag() {
    let mut m = Mesa::da_fila();
    m.mao(PointerKind::Down, seta_de_cima());
    assert!(m.store.number_input_drag().is_none());
    assert!(m.store.number_stepper_hold().is_some());
    m.mao(PointerKind::Up, seta_de_cima());
    assert!(m.store.number_stepper_hold().is_none());
}
