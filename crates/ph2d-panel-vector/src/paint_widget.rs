//! **A seção WIDGET SKIN** do painel (plano UI/UX W6.2) — irmã de [`super::paint_components`]
//! pelo teto de 600 LOC, e o corte é o mesmo: aqui mora a seção de UM assunto.
//!
//! # Dois verbos que nunca aparecem juntos
//!
//! *Wear a Widget* existe para uma forma que ainda não veste; *Back to Drawing* para a que veste.
//! Pintar os dois sempre daria, em toda seleção, um botão que não faz nada — e é assim que o
//! artista aprende a não confiar nos botões desta janela.
//!
//! # O rótulo NÃO tem campo, e a linha que o diz é parte da feature
//!
//! O texto do widget é o `Name` da entidade. Sem a linha *"Label follows the object name"* o
//! artista procuraria um campo que não existe e concluiria que o controle é mudo — a mesma razão
//! pela qual a instância órfã tem um readout em vez de um silêncio.

use ph2d_i18n::tr;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;

impl BodyCtx<'_> {
    /// **A seção WIDGET SKIN** — que controle do catálogo esta forma veste.
    pub(crate) fn widget_skin_section(&mut self, y: f32) -> f32 {
        let Some(s) = state::widget_skin_state() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_WIDGET,
            tr("panel.vector.section.widget"),
            y,
        );
        if collapsed {
            return y;
        }

        // Uma forma que ainda não veste: o único verbo é VESTIR.
        let Some(selected) = s.selected else {
            if s.unknown {
                // ⚠️ Este ramo é o do documento do FUTURO: a forma carrega um tipo que este build
                // não conhece, desenha como vetor, e a linha é o que impede o artista de concluir
                // que a pele dele se perdeu.
                y = self.label_line(tr("panel.vector.widget.unknown"), y);
                return self.action_button(
                    ids::VECTOR_WIDGET_REMOVE,
                    tr("panel.vector.widget.remove"),
                    y,
                );
            }
            return self.action_button(ids::VECTOR_WIDGET_WEAR, tr("panel.vector.widget.wear"), y);
        };

        let opts: Vec<(ph2d_a11y::NodeId, &str, bool)> = s
            .kinds
            .iter()
            .enumerate()
            .map(|(i, label)| (ids::vector_widget_kind_id(i), label.as_str(), i == selected))
            .collect();
        y = self.segmented("", &opts, y);

        // ⚠️ O excedente é ESCRITO — a mesma lei da lista de variants. Um teto silencioso aqui
        // seria pior que lá: um tipo além do teto é INALCANÇÁVEL (não há conta-gotas por trás).
        let beyond = state::widget_kinds_beyond();
        if beyond > 0 {
            y = self.label_line(
                &format!("{beyond} {}", tr("panel.vector.component.variants_beyond")),
                y,
            );
        }
        y = self.label_line(tr("panel.vector.widget.label_is_name"), y);
        self.action_button(
            ids::VECTOR_WIDGET_REMOVE,
            tr("panel.vector.widget.remove"),
            y,
        )
    }
}
