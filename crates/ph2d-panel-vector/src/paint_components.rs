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
            // ⚠️ **Os variants vêm ANTES das peças**, e a ordem é a da pergunta: *que versão é
            // esta?* precede *e o que nela difere?*. Escolher o variant troca o mestre, e trocar o
            // mestre reescreve a lista de peças — pô-la em cima faria o artista autorar
            // diferenças numa lista que o clique seguinte substitui.
            y = self.variant_rows(y);
            y = self.instance_pieces(y);
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

    /// **Os eixos de VARIANT** — que versão do componente esta cópia é (plano UI/UX W5c).
    ///
    /// Uma fileira segmentada por propriedade, com todos os valores à vista. ⚠️ **Chips e não um
    /// dropdown**, porque um eixo de variant tem tipicamente dois a quatro valores: mostrá-los
    /// todos deixa o artista *ver* o catálogo em vez de o abrir, e a fileira quebra em linhas
    /// sozinha quando não cabem.
    ///
    /// ⚠️ **A seção não pinta nada quando o mestre não tem irmãos.** Um conjunto de variants É os
    /// mestres irmãos; sem irmãos não há escolha, e uma fileira com um chip só é uma escolha que
    /// não escolhe.
    fn variant_rows(&mut self, mut y: f32) -> f32 {
        let rows = state::variant_rows();
        if rows.is_empty() {
            return y;
        }
        for (axis, row) in rows.iter().enumerate() {
            let opts: Vec<(ph2d_a11y::NodeId, &str, bool)> = row
                .values
                .iter()
                .enumerate()
                .map(|(v, label)| {
                    (
                        ids::vector_variant_option_id(axis, v),
                        label.as_str(),
                        v == row.selected,
                    )
                })
                .collect();
            // Nome vazio = o modo de nomes crus, e aí o rótulo é a palavra do produto.
            let label = if row.name.is_empty() {
                tr("panel.vector.component.variant")
            } else {
                &row.name
            };
            y = self.segmented(label, &opts, y);
        }
        // ⚠️ O excedente é ESCRITO — a mesma lei da lista de peças. Um teto silencioso lê-se como
        // *"o conjunto só tem estas versões"*, e o artista procuraria a que falta onde ela não está.
        let beyond = state::variant_rows_beyond();
        if beyond > 0 {
            y = self.label_line(
                &format!("{beyond} {}", tr("panel.vector.component.variants_beyond")),
                y,
            );
        }
        y
    }

    /// **A lista de PEÇAS** — a porta do override (plano UI/UX W5b).
    ///
    /// Duas linhas por peça: o interruptor de visibilidade (cujo rótulo É o nome da peça) e a
    /// swatch de cor. ⚠️ A cor mora numa linha própria porque a swatch é alvo de PICKER e o
    /// interruptor é um botão — **um id só pode ter um tipo de widget no store**, a cicatriz que
    /// o `vector_fx_toggle_id` já pagou.
    fn instance_pieces(&mut self, mut y: f32) -> f32 {
        let pieces = state::instance_pieces();
        if pieces.is_empty() {
            return y;
        }
        y = self.label_line(tr("panel.vector.component.pieces"), y);
        for (row, piece) in pieces.iter().enumerate() {
            y = self.checkbox_row(
                ids::vector_instance_piece_show_id(row),
                &piece.name,
                piece.visible,
                y,
            );
            // O rótulo da cor diz se ela é HERDADA ou desta instância — sem isso, *"esta cópia
            // está diferente"* só se descobre carregando em Reset e vendo o que muda.
            let label = if piece.overridden {
                tr("panel.vector.component.piece_colour_own")
            } else {
                tr("panel.vector.component.piece_colour")
            };
            y = self.colour_swatch_row(
                ids::vector_instance_piece_colour_id(row),
                piece.colour,
                label,
                "Instance piece colour",
                y,
            );
        }
        // ⚠️ O excedente é ESCRITO. Um teto que trunca em silêncio lê-se como *"o mestre só tem
        // estas peças"*, e o artista procuraria a que falta onde ela não está.
        let beyond = state::instance_pieces_beyond();
        if beyond > 0 {
            y = self.label_line(
                &format!("{beyond} {}", tr("panel.vector.component.pieces_beyond")),
                y,
            );
        }
        y
    }
}
