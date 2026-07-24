//! **O canal de gesto da tira do Flip** — a terceira superfície arrastável do app.
//!
//! O motion graph tem a sua ([`GraphGesture`](super::GraphGesture)), o dope-sheet da
//! timeline tem a sua ([`TimelineGesture`](super::TimelineGesture)), e a tira de frames do
//! Flip passa a ter a dela: `dispatch` captura no Down o que estava sob o ponteiro, empurra
//! [`FlipStripGesture`]s por Move/Up, e **não interpreta nada** — quem sabe o que é uma
//! chave é o painel, que resolveu o [`FlipStripHitKind`] no PAINT, onde a geometria está.
//!
//! ## Por que um canal, e não um `PanelEvent`
//!
//! `PanelEvent` está **congelado** (CLAUDE.md §6, ADR-0040): `Click`/`SetValue`/`Toggle`/
//! `SelectOption`. Um arrasto 2D não cabe em nenhum deles — ele tem *começo, percurso e
//! fim*, e o percurso é o que o artista está olhando. Forçá-lo num `SetValue` custaria um
//! variant novo num contrato congelado para expressar mal o que a família de gesto já
//! expressa bem, duas vezes, neste mesmo arquivo-irmão.
//!
//! ## Por que um arquivo só
//!
//! ADR-0107/§1.5.2.1: foundational novo nasce **projetado para isolamento**. Os tipos, o
//! estado e os métodos do store vivem todos aqui; o que a linha acrescenta fora deste
//! arquivo é *um* campo no `WidgetStore`, *um* variant em `InteractiveState` e os três
//! hooks de dispatch — cada um uma adição, nunca uma edição.

use super::state::WidgetStore;
use super::types::{GestureMods, GesturePhase};
use ph2d_a11y::NodeId;
use ph2d_host::PointerButton;

/// Percurso do ponteiro (px lógicos, Chebyshev) a partir do Down antes de um gesto da tira
/// contar como ARRASTO em vez de toque.
///
/// Sem a folga, um pixel de tremor entre apertar e soltar faz o Up escolher `End` sobre
/// `Click`, e **selecionar uma chave vira cara-ou-coroa** — o mesmo motivo (e o mesmo
/// número) do irmão da timeline.
pub const FLIP_STRIP_DRAG_SLOP_PX: f32 = 4.0;

/// O que estava sob o ponteiro na tira de frames.
///
/// Os índices são **opacos** ao editor-core: são posições na lista de células que o painel
/// pintou neste frame, e só ele as mapeia de volta para uma chave do `ph2d-flip`. É a mesma
/// disciplina dos handles crus do [`TimelineHitKind`](super::TimelineHitKind).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FlipStripHitKind {
    /// O corpo da célula — tocar seleciona a chave, arrastar a **move no tempo**.
    Cell {
        /// Índice da célula na lista pintada neste frame.
        index: u16,
    },
    /// A **borda direita** da célula: a fronteira entre este desenho e o próximo. Arrastar
    /// estica ou encolhe a EXPOSIÇÃO (o hold) — o gesto que toda tira de animação tem,
    /// porque a largura da célula *é* a duração.
    HoldEdge {
        /// Índice da célula cuja borda foi pega.
        index: u16,
    },
}

/// Um gesto de ponteiro na tira, guardado pelo dispatch e drenado pelo painel a cada frame
/// ([`WidgetStore::drain_flip_strip_gestures`]).
///
/// As posições são px globais; o painel as mapeia para QUADRO com a régua que ele mesmo
/// usou para pintar (`ppf` + o primeiro quadro visível) — uma segunda régua aqui divergiria
/// da que o artista está vendo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FlipStripGesture {
    /// A superfície (o painel da tira) a que o gesto pertence — o `parent` do
    /// [`InteractiveState::FlipStripSurface`](super::InteractiveState::FlipStripSurface)
    /// atingido.
    pub surface: NodeId,
    /// O que estava sob o ponteiro no `Begin`, carregado inalterado por Update/End/Click.
    pub kind: FlipStripHitKind,
    pub phase: GesturePhase,
    pub x: f32,
    pub y: f32,
    pub button: PointerButton,
    pub mods: GestureMods,
}

/// O estado do canal dentro do [`WidgetStore`] — **um campo**, não cinco.
///
/// O irmão da timeline espalhou `timeline_gestures`/`_press`/`_moved`/`_double`/… pelo
/// struct compartilhado. Agrupar aqui deixa a superfície que outra linha vê (e com que
/// outra linha colide) em uma linha só.
#[derive(Clone, Debug, Default)]
pub struct FlipStripChannel {
    /// Gestos deste frame (dispatch escreve, painel drena).
    gestures: Vec<FlipStripGesture>,
    /// Ponto do Down — a origem de onde a folga é medida.
    press: (f32, f32),
    /// Já passou da folga? (uma vez arrasto, sempre arrasto até o Up).
    moved: bool,
}

impl WidgetStore {
    /// Guarda um gesto da tira (dispatch → painel).
    pub fn push_flip_strip_gesture(&mut self, gesture: FlipStripGesture) {
        self.flip_strip.gestures.push(gesture);
    }

    /// Drena os gestos da tira deste frame. O `Drain` preserva a capacidade do `Vec`, então
    /// um arrasto contínuo reusa a alocação (HR-3).
    pub fn drain_flip_strip_gestures(&mut self) -> std::vec::Drain<'_, FlipStripGesture> {
        self.flip_strip.gestures.drain(..)
    }

    /// Se o estado de `id` for uma
    /// [`InteractiveState::FlipStripSurface`](super::InteractiveState::FlipStripSurface),
    /// devolve `(superfície, o que foi atingido)`. O editor-core copia os dois e **nunca**
    /// olha dentro do índice.
    pub fn flip_strip_surface_at_id(&self, id: NodeId) -> Option<(NodeId, FlipStripHitKind)> {
        match self.get(id) {
            Some(super::InteractiveState::FlipStripSurface { parent, kind }) => {
                Some((*parent, *kind))
            }
            _ => None,
        }
    }

    /// Arma uma captura nova no ponto do Down: ainda não é arrasto, e é daqui que a folga é
    /// medida.
    pub fn begin_flip_strip_press(&mut self, x: f32, y: f32) {
        self.flip_strip.press = (x, y);
        self.flip_strip.moved = false;
    }

    /// Promove a captura a ARRASTO quando o ponteiro passa de [`FLIP_STRIP_DRAG_SLOP_PX`].
    /// Chamado em todo Move.
    pub fn note_flip_strip_pointer(&mut self, x: f32, y: f32) {
        if !self.flip_strip.moved {
            let (px, py) = self.flip_strip.press;
            let travel = (x - px).abs().max((y - py).abs());
            self.flip_strip.moved = travel > FLIP_STRIP_DRAG_SLOP_PX;
        }
    }

    /// Lê e zera o "a captura andou" (o Up escolhe `End` contra `Click`).
    pub fn take_flip_strip_moved(&mut self) -> bool {
        std::mem::take(&mut self.flip_strip.moved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> WidgetStore {
        WidgetStore::with_capacity(0)
    }

    #[test]
    fn a_tap_that_trembles_within_the_slop_is_still_a_tap() {
        // Sem isto, escolher a chave a inspecionar (o gesto mais comum da tira) vira
        // cara-ou-coroa: um pixel de tremor entre apertar e soltar viraria um arrasto,
        // e o arrasto MOVE a chave no tempo — o toque perdido não é inerte, é destrutivo.
        let mut s = store();
        s.begin_flip_strip_press(100.0, 50.0);
        s.note_flip_strip_pointer(101.0, 52.0);
        s.note_flip_strip_pointer(100.0, 47.0);
        assert!(!s.take_flip_strip_moved(), "ainda é um toque");
    }

    #[test]
    fn travelling_past_the_slop_makes_it_a_drag_for_good() {
        let mut s = store();
        s.begin_flip_strip_press(100.0, 50.0);
        s.note_flip_strip_pointer(100.0 + FLIP_STRIP_DRAG_SLOP_PX + 0.1, 50.0);
        // Voltar para dentro da folga NÃO rebaixa: arrasto é arrasto até o Up.
        s.note_flip_strip_pointer(100.0, 50.0);
        assert!(s.take_flip_strip_moved());
        assert!(!s.take_flip_strip_moved(), "a marca zera na leitura");
    }

    #[test]
    fn each_press_rearms_the_slop_from_its_own_origin() {
        let mut s = store();
        s.begin_flip_strip_press(0.0, 0.0);
        s.note_flip_strip_pointer(500.0, 0.0);
        assert!(s.take_flip_strip_moved());
        s.begin_flip_strip_press(500.0, 0.0);
        s.note_flip_strip_pointer(501.0, 0.0);
        assert!(!s.take_flip_strip_moved(), "medida a partir do Down NOVO");
    }

    #[test]
    fn the_channel_carries_gestures_in_order_and_keeps_its_capacity() {
        let mut s = store();
        let g = |x: f32| FlipStripGesture {
            surface: NodeId(1),
            kind: FlipStripHitKind::Cell { index: 0 },
            phase: GesturePhase::Update,
            x,
            y: 0.0,
            button: PointerButton::Primary,
            mods: GestureMods::default(),
        };
        s.push_flip_strip_gesture(g(1.0));
        s.push_flip_strip_gesture(g(2.0));
        let xs: Vec<f32> = s.drain_flip_strip_gestures().map(|g| g.x).collect();
        assert_eq!(xs, vec![1.0, 2.0], "a ordem do percurso é o percurso");
        assert!(
            s.drain_flip_strip_gestures().next().is_none(),
            "drenar esvazia"
        );
    }

    #[test]
    fn an_id_that_is_not_a_strip_surface_answers_nothing() {
        // O `_` do match não pode virar um `Some` otimista: o dispatch pergunta a TODO id
        // sob o ponteiro, e a tira divide a tela com o resto do app.
        let mut s = store();
        let id = NodeId(7);
        s.register(
            id,
            super::super::InteractiveState::Button {
                state: crate::widget::ButtonState::Normal,
            },
        );
        assert!(s.flip_strip_surface_at_id(id).is_none());
        assert!(s.flip_strip_surface_at_id(NodeId(999)).is_none());
    }
}
