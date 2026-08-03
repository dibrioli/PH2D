//! **A seção CONSTRAINTS** do painel (plano UI/UX W3) — irmã de [`super::paint_layout`] pelo teto
//! de 600 LOC, e o corte é o mesmo: aqui mora a seção de UM assunto.
//!
//! # Ela e a seção Layout são MUTUAMENTE EXCLUSIVAS, e isso é a lei do plano
//!
//! *"Um filho de moldura ou está num fluxo ou está ancorado, nunca os dois."* Quem decide é a
//! shell (a porta do passe recusa um pai que flui), então esta seção simplesmente não é oferecida
//! ali — e é por isso que ela não precisa de uma regra própria a repetir a mesma pergunta. Se
//! aparecessem juntas, o artista teria dois controlos de posição, um deles inerte.
//!
//! # Duas fileiras, e nada mais
//!
//! É o que o Figma tem, e é o superset do que o modelo exprime pela UI. Não há linha de offset:
//! o offset é a POSIÇÃO em que o filho já está — arrastá-lo continua a ser como se autora isso.

use ph2d_i18n::tr;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;

impl BodyCtx<'_> {
    /// **A seção CONSTRAINTS** — o que este filho faz quando a moldura muda de tamanho.
    pub(crate) fn anchors_section(&mut self, y: f32) -> f32 {
        let Some(a) = state::anchor_state() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_ANCHORS,
            tr("panel.vector.section.anchors"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.segmented(
            tr("panel.vector.anchors.h"),
            &[
                (
                    ids::VECTOR_ANCHOR_H_START,
                    tr("panel.vector.anchors.left"),
                    a.h == Some(ids::VECTOR_ANCHOR_H_START),
                ),
                (
                    ids::VECTOR_ANCHOR_H_CENTER,
                    tr("panel.vector.anchors.center"),
                    a.h == Some(ids::VECTOR_ANCHOR_H_CENTER),
                ),
                (
                    ids::VECTOR_ANCHOR_H_END,
                    tr("panel.vector.anchors.right"),
                    a.h == Some(ids::VECTOR_ANCHOR_H_END),
                ),
                (
                    ids::VECTOR_ANCHOR_H_STRETCH,
                    tr("panel.vector.anchors.stretch"),
                    a.h == Some(ids::VECTOR_ANCHOR_H_STRETCH),
                ),
            ],
            y,
        );
        self.segmented(
            tr("panel.vector.anchors.v"),
            &[
                (
                    ids::VECTOR_ANCHOR_V_START,
                    tr("panel.vector.anchors.top"),
                    a.v == Some(ids::VECTOR_ANCHOR_V_START),
                ),
                (
                    ids::VECTOR_ANCHOR_V_CENTER,
                    tr("panel.vector.anchors.center"),
                    a.v == Some(ids::VECTOR_ANCHOR_V_CENTER),
                ),
                (
                    ids::VECTOR_ANCHOR_V_END,
                    tr("panel.vector.anchors.bottom"),
                    a.v == Some(ids::VECTOR_ANCHOR_V_END),
                ),
                (
                    ids::VECTOR_ANCHOR_V_STRETCH,
                    tr("panel.vector.anchors.stretch"),
                    a.v == Some(ids::VECTOR_ANCHOR_V_STRETCH),
                ),
            ],
            y,
        )
    }
}
