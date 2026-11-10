//! ⭐ **O ponteiro sobre cada alvo da Timeline** — a tabela vive ao lado das variantes, para que uma
//! alça nova nasça a dizer o cursor dela. A shell só traduz [`PointerLook`] para o do sistema
//! (`timeline_resize_cursor`); o mesmo id do clique decide, então o cursor não discorda do gesto.

use super::types::TimelineHitKind;
use crate::doc_surface::PointerLook;

/// O meio de uma faixa (strip/loop): `0`/`1` são pontas, `2` é o corpo que se move.
const CORPO: u8 = 2;

impl TimelineHitKind {
    /// O ponteiro deste alvo: `↔` nas pontas, fades, chaves do loop e a `⟷` da duração; a mão aberta
    /// (fechada quando `apertado`) no que se agarra e move; `None` no fundo (a seta). As bordas do
    /// painel ([`Self::ResizeEdge`]) têm diagonais que [`PointerLook`] não diz: a shell resolve-as.
    #[must_use]
    pub fn pointer_look(self, apertado: bool) -> Option<PointerLook> {
        let mao = if apertado {
            PointerLook::Grabbing
        } else {
            PointerLook::Grab
        };
        Some(match self {
            Self::LabelSplitter | Self::DurationHandle | Self::SelectionTimeHandle { .. } => {
                PointerLook::ResizeEw
            }
            Self::GraphResize => PointerLook::ResizeNs,
            Self::Strip { edge, .. } | Self::LoopBrace { edge } if edge != CORPO => {
                PointerLook::ResizeEw
            }
            Self::Strip { .. }
            | Self::LoopBrace { .. }
            | Self::Key { .. }
            | Self::SummaryKey { .. }
            | Self::GroupKey { .. }
            | Self::MacroStep { .. }
            | Self::Marker { .. }
            | Self::CurveAnchor { .. }
            | Self::CurveHandle { .. } => mao,
            Self::Lane
            | Self::Twirl { .. }
            | Self::Row { .. }
            | Self::SummaryLock
            | Self::ResizeEdge { .. }
            | Self::LaneHeader { .. }
            | Self::ContainerRow { .. }
            | Self::GraphBufferButton { .. }
            | Self::GroupTwirl { .. } => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_pontas_esticam_o_corpo_agarra_o_fundo_e_a_seta() {
        let strip = |edge| TimelineHitKind::Strip {
            lane: 0,
            strip: 1,
            edge,
        };
        for edge in [0, 1, 3, 4, 5, 6] {
            assert_eq!(strip(edge).pointer_look(false), Some(PointerLook::ResizeEw));
        }
        assert_eq!(strip(CORPO).pointer_look(false), Some(PointerLook::Grab));
        assert_eq!(strip(CORPO).pointer_look(true), Some(PointerLook::Grabbing));
        let brace = |edge| TimelineHitKind::LoopBrace { edge };
        assert_eq!(brace(0).pointer_look(false), Some(PointerLook::ResizeEw));
        assert_eq!(brace(CORPO).pointer_look(true), Some(PointerLook::Grabbing));
        let key = TimelineHitKind::Key { target: 1, key: 2 };
        assert_eq!(key.pointer_look(false), Some(PointerLook::Grab));
        assert_eq!(TimelineHitKind::Lane.pointer_look(false), None);
        assert_eq!(
            TimelineHitKind::DurationHandle.pointer_look(false),
            Some(PointerLook::ResizeEw)
        );
    }
}
