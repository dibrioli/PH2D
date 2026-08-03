//! **A seção COMPONENT** do painel (plano UI/UX W5) — irmã de [`super::paint_anchors`] pelo teto
//! de 600 LOC, e o corte é o mesmo: aqui mora a seção de UM assunto.
//!
//! # Cada verbo aparece onde faz sentido, e a seção some quando nenhum faz
//!
//! Uma seção que existisse sempre, com três botões inertes e um vivo, ensinaria o artista a não
//! confiar nos botões desta janela. A shell responde *"que verbos fazem sentido?"* uma vez
//! ([`crate::state::ComponentState`]) e esta função pinta o que ela disser.
//!
//! # O readout de ÓRFÃ é uma FRASE, não um botão
//!
//! Uma instância cujo mestre sumiu ainda desenha (o suporte dela), e nada na forma diz porquê. A
//! linha *"main missing"* é o que torna a causa visível — o mesmo desenho da binding órfã da
//! timeline, e a razão de o produtor NOMEAR as órfãs em vez de as calar.

use ph2d_i18n::tr;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;

impl BodyCtx<'_> {
    /// **A seção COMPONENT** — o mestre, a instância, e o que se pode fazer com eles.
    pub(crate) fn component_section(&mut self, y: f32) -> f32 {
        let Some(c) = state::component_state() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_COMPONENT,
            tr("panel.vector.section.component"),
            y,
        );
        if collapsed {
            return y;
        }
        // Uma forma comum: o único verbo é PROMOVER.
        if !c.is_main && !c.is_instance {
            return self.action_button(
                ids::VECTOR_COMPONENT_CREATE,
                tr("panel.vector.component.create"),
                y,
            );
        }
        if c.is_main {
            y = self.action_button(
                ids::VECTOR_COMPONENT_PLACE,
                tr("panel.vector.component.place"),
                y,
            );
        }
        if c.is_instance {
            if c.main_missing {
                y = self.label_line(tr("panel.vector.component.missing"), y);
            }
            y = self.action_button(
                ids::VECTOR_COMPONENT_DETACH,
                tr("panel.vector.component.detach"),
                y,
            );
            // ⚠️ Só com o que resetar. Um *Reset* sobre uma instância limpa é um clique que não
            // faz nada, e o artista não tem como saber disso antes de o dar.
            if c.has_overrides {
                y = self.action_button(
                    ids::VECTOR_COMPONENT_RESET,
                    tr("panel.vector.component.reset"),
                    y,
                );
            }
        }
        y
    }
}
