//! **O canal de gesto da linha do tempo do Pixel** — a quarta superfície arrastável, irmã da tira
//! do Flip ([`super::flip_strip`]): o dispatch captura no Down o que estava sob o ponteiro, empurra
//! [`PixelTimelineGesture`]s por Move/Up e não interpreta nada; o painel resolveu o
//! [`PixelTimelineHitKind`] no PAINT, onde a geometria está.
//!
//! Canal próprio, e não o da timeline geral nem o do Flip: cada painel DRENA o seu canal inteiro a
//! cada quadro, e um canal partilhado entregaria os gestos de um ao outro.

use super::state::WidgetStore;
use super::types::{GestureMods, GesturePhase};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_host::{PointerButton, PointerEvent};

/// O que estava sob o ponteiro (ids ESTÁVEIS do sprite; opacos ao editor-core).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelTimelineHitKind {
    /// O cabeçalho de um quadro: clique escolhe, arrasto escolhe um intervalo contíguo.
    Header { frame: u32 },
    /// Um cel: clique escolhe, arrasto escolhe o rectângulo camadas × quadros.
    Cel { layer: u32, frame: u32 },
    /// A ponta esquerda (`right = false`) ou direita da tag `index`: arrastar muda o intervalo.
    TagEdge { index: u16, right: bool },
    /// A moldura do intervalo escolhido: arrastar move os quadros dele.
    RangeOutline,
    /// O colchete `[` (`right = false`) ou `]` da faixa do onion (P12 R4): arrastar muda quantos
    /// quadros antes / depois do actual.
    OnionEdge { right: bool },
}

/// Um gesto de ponteiro na linha do tempo do Pixel (posições globais).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PixelTimelineGesture {
    pub surface: NodeId,
    pub kind: PixelTimelineHitKind,
    pub phase: GesturePhase,
    pub x: f32,
    pub y: f32,
    pub button: PointerButton,
    pub mods: GestureMods,
}

/// O estado do canal dentro do [`WidgetStore`] — um campo.
#[derive(Clone, Debug, Default)]
pub struct PixelTimelineChannel {
    gestures: Vec<PixelTimelineGesture>,
    slop: crate::interaction::PressSlop,
}

impl WidgetStore {
    /// Drena os gestos deste quadro (o painel da linha do tempo do Pixel).
    pub fn drain_pixel_timeline_gestures(&mut self) -> std::vec::Drain<'_, PixelTimelineGesture> {
        self.pixel_timeline.gestures.drain(..)
    }

    /// Guarda um gesto (dispatch → painel; os testes do painel também o usam).
    pub fn push_pixel_timeline_gesture(&mut self, g: PixelTimelineGesture) {
        self.pixel_timeline.gestures.push(g);
    }

    /// `(superfície, o que foi atingido)` se `id` é uma superfície da linha do tempo do Pixel.
    #[must_use]
    pub fn pixel_timeline_surface_at_id(
        &self,
        id: NodeId,
    ) -> Option<(NodeId, PixelTimelineHitKind)> {
        match self.get(id) {
            Some(super::InteractiveState::PixelTimelineSurface { parent, kind }) => {
                Some((*parent, *kind))
            }
            _ => None,
        }
    }

    fn pixel_timeline_push(&mut self, id: NodeId, phase: GesturePhase, e: &PointerEvent) -> bool {
        let Some((surface, kind)) = self.pixel_timeline_surface_at_id(id) else {
            return false;
        };
        let mods = self.gesture_mods();
        self.push_pixel_timeline_gesture(PixelTimelineGesture {
            surface,
            kind,
            phase,
            x: e.x,
            y: e.y,
            button: e.button,
            mods,
        });
        true
    }

    /// O ponteiro que a linha do tempo do Pixel pede (D2/D3, smoke 31/10): a captura em curso
    /// manda, senão o que está sob o rato (`hover`) — a ponta da tag estica de lado, a moldura do
    /// intervalo agarra-se.
    #[must_use]
    pub fn pixel_timeline_look(
        &self,
        hover: Option<NodeId>,
    ) -> Option<crate::doc_surface::PointerLook> {
        use crate::doc_surface::PointerLook as L;
        let agarrada = self
            .active_id()
            .and_then(|a| self.pixel_timeline_surface_at_id(a));
        let (_, kind) = agarrada.or_else(|| self.pixel_timeline_surface_at_id(hover?))?;
        match kind {
            PixelTimelineHitKind::TagEdge { .. } | PixelTimelineHitKind::OnionEdge { .. } => {
                Some(L::ResizeEw)
            }
            PixelTimelineHitKind::RangeOutline if agarrada.is_some() => Some(L::Grabbing),
            PixelTimelineHitKind::RangeOutline => Some(L::Grab),
            _ => None,
        }
    }

    /// O Down (botão esquerdo) sobre uma superfície: captura e `Begin`. `false` = não é dela.
    pub(crate) fn pixel_timeline_down(
        &mut self,
        hit: Option<(NodeId, Rect)>,
        e: &PointerEvent,
    ) -> bool {
        let Some((id, rect)) = hit.filter(|_| e.button == PointerButton::Primary) else {
            return false;
        };
        if !self.pixel_timeline_push(id, GesturePhase::Begin, e) {
            return false;
        }
        self.set_active(Some(id));
        self.set_active_rect(Some(rect));
        self.pixel_timeline.slop.begin(e.x, e.y);
        true
    }

    /// O Move com a captura dela: `Update`, mesmo fora do rect (o arrasto atravessa colunas).
    pub(crate) fn pixel_timeline_move(&mut self, active: NodeId, e: &PointerEvent) -> bool {
        if self.pixel_timeline_surface_at_id(active).is_none() {
            return false;
        }
        self.pixel_timeline.slop.note(e.x, e.y);
        self.pixel_timeline_push(active, GesturePhase::Update, e)
    }

    /// `id` é uma das superfícies que capturam o ponteiro (grafo, timeline, tira do Flip, linha
    /// do tempo do Pixel) — o que um modal ou um menu aberto tem de saltar ou dispensar.
    pub(crate) fn is_pointer_surface(&self, id: NodeId) -> bool {
        self.graph_surface_at_id(id).is_some()
            || self.timeline_surface_at_id(id).is_some()
            || self.flip_strip_surface_at_id(id).is_some()
            || self.pixel_timeline_surface_at_id(id).is_some()
    }

    /// O Up com a captura dela: `End` se andou, senão `Click`; larga a captura.
    pub(crate) fn pixel_timeline_up(&mut self, active: NodeId, e: &PointerEvent) -> bool {
        let phase = if self.pixel_timeline.slop.take_moved() {
            GesturePhase::End
        } else {
            GesturePhase::Click
        };
        if !self.pixel_timeline_push(active, phase, e) {
            return false;
        }
        self.set_active(None);
        self.set_active_rect(None);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_host::PointerKind;

    fn ev(kind: PointerKind, x: f32) -> PointerEvent {
        PointerEvent {
            kind,
            x,
            y: 10.0,
            pressure: 1.0,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        }
    }

    fn store_with_header() -> (WidgetStore, NodeId) {
        let mut s = WidgetStore::with_capacity(0);
        let id = NodeId(11);
        let kind = PixelTimelineHitKind::Header { frame: 3 };
        s.register(
            id,
            super::super::InteractiveState::PixelTimelineSurface {
                parent: NodeId(1),
                kind,
            },
        );
        (s, id)
    }

    #[test]
    fn a_press_and_release_in_place_is_a_click_and_a_drag_streams_updates_then_ends() {
        let (mut s, id) = store_with_header();
        let r = Rect::new(0.0, 0.0, 20.0, 20.0);
        assert!(s.pixel_timeline_down(Some((id, r)), &ev(PointerKind::Down, 5.0)));
        assert!(s.pixel_timeline_up(id, &ev(PointerKind::Up, 5.0)));
        let fases: Vec<_> = s.drain_pixel_timeline_gestures().map(|g| g.phase).collect();
        assert_eq!(fases, [GesturePhase::Begin, GesturePhase::Click]);
        assert!(s.pixel_timeline_down(Some((id, r)), &ev(PointerKind::Down, 5.0)));
        assert!(s.pixel_timeline_move(id, &ev(PointerKind::Move, 90.0)));
        assert!(s.pixel_timeline_up(id, &ev(PointerKind::Up, 90.0)));
        let g: Vec<_> = s.drain_pixel_timeline_gestures().collect();
        let fases: Vec<_> = g.iter().map(|g| g.phase).collect();
        assert_eq!(
            fases,
            [GesturePhase::Begin, GesturePhase::Update, GesturePhase::End]
        );
        assert!(
            g.iter()
                .all(|g| g.kind == PixelTimelineHitKind::Header { frame: 3 })
        );
        assert_eq!(s.active_id(), None, "o Up larga a captura");
    }

    #[test]
    fn another_widget_or_the_right_button_is_not_this_channel() {
        let (mut s, id) = store_with_header();
        let r = Rect::new(0.0, 0.0, 20.0, 20.0);
        let mut direito = ev(PointerKind::Down, 5.0);
        direito.button = PointerButton::Secondary;
        assert!(!s.pixel_timeline_down(Some((id, r)), &direito));
        assert!(!s.pixel_timeline_down(Some((NodeId(99), r)), &ev(PointerKind::Down, 5.0)));
        assert!(s.drain_pixel_timeline_gestures().next().is_none());
    }

    /// D2/D3 (smoke 31/10): a ponta da tag estica de lado; a moldura é mão aberta sob o rato e
    /// fechada enquanto se agarra — mesmo com o rato fora dela (a captura manda).
    #[test]
    fn the_tag_end_and_the_range_outline_ask_for_their_pointer() {
        use crate::doc_surface::PointerLook as L;
        let mut s = WidgetStore::with_capacity(0);
        let (ponta, moldura) = (NodeId(21), NodeId(22));
        for (id, kind) in [
            (
                ponta,
                PixelTimelineHitKind::TagEdge {
                    index: 0,
                    right: true,
                },
            ),
            (moldura, PixelTimelineHitKind::RangeOutline),
        ] {
            let parent = NodeId(1);
            s.register(
                id,
                super::super::InteractiveState::PixelTimelineSurface { parent, kind },
            );
        }
        assert_eq!(s.pixel_timeline_look(Some(ponta)), Some(L::ResizeEw));
        assert_eq!(s.pixel_timeline_look(Some(moldura)), Some(L::Grab));
        assert_eq!(s.pixel_timeline_look(None), None);
        let r = Rect::new(0.0, 0.0, 20.0, 20.0);
        assert!(s.pixel_timeline_down(Some((moldura, r)), &ev(PointerKind::Down, 5.0)));
        assert_eq!(s.pixel_timeline_look(None), Some(L::Grabbing));
        assert!(s.pixel_timeline_up(moldura, &ev(PointerKind::Up, 90.0)));
        assert_eq!(s.pixel_timeline_look(Some(moldura)), Some(L::Grab));
    }
}
