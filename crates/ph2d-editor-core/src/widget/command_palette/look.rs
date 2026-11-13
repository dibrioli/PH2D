//! **O ACABAMENTO da paleta** — o único sítio que decide as medidas e os pesos do cartão.
//!
//! Dois acabamentos: o [`classic`] dos três consumidores de sempre (Add Component, `Ctrl+K`, a
//! biblioteca de nós do Motion), byte a byte o de antes, e o [`compact`] do Add Object (dono 07/11:
//! «mais estreito, mais próximo da referência» — a maquete B do Add Component da Crítica,
//! `critica_ph2d.html` §addcomponent: `.ph{max-width:820px}`, `.palg` 3 colunas com `gap:8px`,
//! `.cxg{padding:6px 8px}`, `h6` com a contagem `em` à direita e o filete, `.ent{padding:3px 7px}`
//! a 12 px peso normal com o ponto de 6 px).
//!
//! ⚠️ **A decisão mora AQUI e só aqui** ([`look`]): metrics, arranjo, banda e pintor leem o
//! [`Look`]; nenhum deles pergunta pelo modelo.

use super::PaletteModel;
use ph2d_text::FontWeight;
use ph2d_tokens::{Spacing, TypeToken};

/// As medidas e os pesos de um acabamento.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Look {
    /// A largura máxima da janela.
    pub(super) card_max_w: f32,
    /// A altura da banda do cabeçalho e o corpo do título dela.
    pub(super) header_h: f32,
    pub(super) title_font: f32,
    /// Quantas unidades de coluna tem a grade (uma categoria GRANDE ocupa duas, se houver ≥ 4).
    pub(super) units: usize,
    /// O vão entre colunas e entre cartões empilhados.
    pub(super) col_gap: f32,
    pub(super) sect_gap: f32,
    /// O recuo interno do cartão de grupo: lados, e em cima/em baixo.
    pub(super) card_pad_x: f32,
    pub(super) card_pad_y: f32,
    /// A pílula: altura, vão, recuo, raio do ponto, o corpo em que se MEDE, o corpo em que se
    /// PINTA e o peso do rótulo. ⚠️ O de sempre mede a 12 e pinta a 11 (a folga de 1 px por
    /// letra é o ar das pílulas dele); o da referência mede e pinta no mesmo corpo.
    pub(super) pill_h: f32,
    pub(super) pill_gap: f32,
    pub(super) pill_pad_x: f32,
    pub(super) dot_r: f32,
    pub(super) pill_font: f32,
    pub(super) pill_ink: f32,
    pub(super) pill_weight: FontWeight,
    /// O filete colorido sob o título do grupo, e o vão entre o título e ele.
    pub(super) rule_h: f32,
    pub(super) rule_gap: f32,
    /// A contagem do grupo encostada à DIREITA, em cinza (`false` = «Title · N» colado).
    pub(super) count_right: bool,
    /// A lupa no campo de busca.
    pub(super) search_icon: bool,
    /// A busca pinta-se COM FOCO (anel + caret) — só depois de um clique nela ou de uma tecla: o pintor
    /// cruza isto com o store (dono 08/11).
    pub(super) search_focus: bool,
}

/// O acabamento de sempre — as consts do pai e os tokens vivos, sem mudar um bit.
pub(super) fn classic() -> Look {
    Look {
        card_max_w: super::CARD_MAX_W,
        header_h: super::HEADER_H,
        title_font: TypeToken::Md.px(),
        units: 4, // LITERAL-PX-OK: CONTAGEM de unidades da grade, não medida
        col_gap: super::COL_GAP,
        sect_gap: super::SECT_GAP,
        card_pad_x: super::CARD_PAD,
        card_pad_y: super::CARD_PAD,
        pill_h: super::PILL_H,
        pill_gap: super::PILL_GAP,
        pill_pad_x: super::PILL_PAD_X,
        dot_r: super::DOT_R,
        pill_font: TypeToken::Sm.px(),
        pill_ink: TypeToken::Xs.px(),
        pill_weight: FontWeight::MEDIUM,
        rule_h: super::RULE_H,
        rule_gap: Spacing::Xs.px(),
        count_right: false,
        search_icon: false,
        search_focus: false,
    }
}

/// O acabamento da referência (Add Object) — medidas do CSS da maquete B.
pub(super) fn compact() -> Look {
    Look {
        card_max_w: 820.0, // LITERAL-PX-OK: `.m-col.wide>.ph{max-width:820px}` da maquete B
        header_h: 34.0,    // LITERAL-PX-OK: banda da maquete B (campo de 26 px + margem)
        title_font: TypeToken::Base.px(),
        units: 3, // LITERAL-PX-OK: CONTAGEM de colunas da maquete B, não medida
        col_gap: Spacing::Md.px(),
        sect_gap: Spacing::Md.px(),
        card_pad_x: Spacing::Md.px(),
        card_pad_y: Spacing::Sm.px(),
        pill_h: 22.0, // LITERAL-PX-OK: `.ent` 12 px × 1,3 + 2 × 3 px da maquete B
        pill_gap: Spacing::Xs.px(),
        pill_pad_x: 7.0, // LITERAL-PX-OK: `.ent{padding:3px 7px}` da maquete B
        dot_r: 3.0,      // LITERAL-PX-OK: `.ent u{width:6px}` da maquete B
        pill_font: TypeToken::Sm.px(),
        pill_ink: TypeToken::Sm.px(),
        pill_weight: FontWeight::NORMAL,
        rule_h: 1.0, // LITERAL-PX-OK: filete fino (dono 07/11; a maquete tem 1,5)
        rule_gap: Spacing::Sm.px(), // `h6` 11,5 px × 1,3 + 3 px de recuo, sobre o corpo de 12

        count_right: true,
        search_icon: true,
        search_focus: true,
    }
}

/// ⭐ **O acabamento de um modelo** — a decisão única: o modo «destaca, depois escolhe» (o Add
/// Object) leva o da referência; os outros, o de sempre.
pub(super) fn look(model: &PaletteModel) -> Look {
    if model.is_list() {
        // A lista: o acabamento da referência numa janela mais estreita, a busca com o foco.
        Look {
            card_max_w: super::lista::LIST_MAX_W,
            // O campo tem a altura de uma linha da lista (o campo é `header_h − Sm`).
            header_h: super::lista::item_h() + Spacing::Sm.px(),
            ..compact()
        }
    } else if model.select.is_some() {
        compact()
    } else {
        classic()
    }
}

impl Look {
    /// A altura do bloco do título de um cartão de grupo (nome + vão + filete + vão).
    pub(super) fn head_h(&self) -> f32 {
        TypeToken::Sm.px() + self.rule_gap + self.rule_h + Spacing::Sm.px()
    }
}

#[cfg(test)]
#[path = "look_tests.rs"]
mod tests;
