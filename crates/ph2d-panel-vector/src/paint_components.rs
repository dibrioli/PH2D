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
        // ⭐⭐⭐ **Promover uma CÓPIA faz uma versão nova do prefab** — o mesmo verbo e o mesmo id
        // do botão acima, com o rótulo a dizer o que ele faz **neste** sujeito. O menu da
        // Hierarquia já o oferecia (a tabela dele é plana); aqui ele não era pintado, e o artista
        // que vive no painel não tinha como criar uma variante.
        //
        // ⚠️ **A condição vem do PRODUTOR** (`can_make_variant`), e não de `is_instance`: só o
        // modelo geral tem esta lei — ver o doc daquele campo.
        if c.can_make_variant {
            y = self.action_button(
                ids::VECTOR_COMPONENT_CREATE,
                tr("panel.vector.component.make_variant"),
                y,
            );
        }
        if c.is_main {
            y = self.action_button(
                ids::VECTOR_COMPONENT_PLACE,
                tr("panel.vector.component.place"),
                y,
            );
            // ⭐⭐ **O IRMÃO, colado a ele** — a cópia que divide a ARTE da receita. ⚠️ Eles ficam
            // juntos pela mesma razão do par `Group`/`Ungroup` da Hierarquia: *a escolha entre os
            // dois só existe neste instante*, e um verbo cujo irmão está noutro painel não se usa.
            y = self.action_button(
                ids::VECTOR_COMPONENT_PLACE_LINKED,
                tr("panel.vector.component.place_linked"),
                y,
            );
        }
        // ⭐⭐⭐ **ABRIR a receita desta cópia.** Ele fica no topo dos verbos de cópia porque é o
        // único que leva o artista para **outro sujeito** — os de baixo agem sobre esta cópia, e
        // este muda para onde se está a olhar.
        //
        // ⚠️ A condição vem do PRODUTOR (`can_edit_prefab`): no motor vetorial o mestre é uma forma
        // visível, que se alcança clicando nela — lá este botão não teria sujeito.
        if c.can_edit_prefab {
            y = self.action_button(
                ids::VECTOR_COMPONENT_EDIT,
                tr("panel.vector.component.edit"),
                y,
            );
        }
        if c.is_instance {
            if c.main_missing {
                y = self.label_line(tr("panel.vector.component.missing"), y);
            }
            // ⛔⛔ **AQUI VIVIAM A FILEIRA DE VARIANTS E A LISTA DE PEÇAS** — as duas eram
            // desenhadas do `VecInstance`, e morreram com ele (F4.6c, 2026-09-07).
            //
            // ⚠️ **As capacidades ficaram, noutro gesto:** que versão esta cópia é vive no cartão
            // do Inspector (F5), e as diferenças por peça — esconder uma, pintá-la — são hoje o
            // `Visibility`/`Sprite` da própria peça, com régua em `instance_piece_override_tests`.
            // *A lista aqui era a porta de um motor; o motor tem uma porta melhor.*
            // **Swap** — o conta-gotas. O rótulo DIZ o que o próximo clique faz enquanto está
            // armado: um pick modal que não se anuncia é indistinguível de um clique perdido.
            y = self.action_button(
                ids::VECTOR_COMPONENT_SWAP,
                if c.swap_armed {
                    tr("panel.vector.component.swap_armed")
                } else {
                    tr("panel.vector.component.swap")
                },
                y,
            );
            y = self.action_button(
                ids::VECTOR_COMPONENT_DETACH,
                tr("panel.vector.component.detach"),
                y,
            );
            // ⚠️ Os dois só com o que absorver / resetar. Sobre uma instância limpa são cliques
            // que não fazem nada, e o artista não tem como saber disso antes de os dar.
            if c.has_overrides {
                y = self.action_button(
                    ids::VECTOR_COMPONENT_UPDATE_MAIN,
                    tr("panel.vector.component.update_main"),
                    y,
                );
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
