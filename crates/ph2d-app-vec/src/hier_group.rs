//! ⭐⭐⭐ **AGRUPAR / DESAGRUPAR pela Hierarquia** (Enio, 2026-08-30).
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `hier_group.rs`.

use ph2d_editor_core::Toast;
use ph2d_i18n::{tr, tr_with};

/// O que o verbo fez — ou porque não fez nada.
///
/// ⚠️ **Os dois casos de recusa são variantes próprias, e não um `None`.** Cada um tem uma frase
/// diferente a dizer ao artista, e *um verbo que come o clique em silêncio é pior que um ausente* —
/// é a lei que a própria tabela deste menu declara.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nasceu um grupo com `n` membros. Carrega os bits dele: quem chama selecciona-o e recolhe-o.
    Grouped { group: u64, members: usize },
    /// A selecção tinha menos de dois objectos DISTINTOS de topo.
    NeedsTwo,
    /// Dissolveram-se `n` grupos.
    Ungrouped { groups: usize },
    /// Nada na selecção estava dentro de um grupo.
    NotGrouped,
    /// O clique caiu fora de uma selecção múltipla — ver [`Subject`].
    ClickedOutsideSelection,
}

impl Outcome {
    /// A frase que o artista lê. ⚠️ Ela nomeia **o que aconteceu**, não o verbo que ele carregou:
    /// *"Grouped 3 objects"* diz-lhe que a conta bateu; *"Group"* não diria nada que ele já não
    /// soubesse.
    pub fn toast(self) -> Toast {
        match self {
            Self::Grouped { members, .. } => Toast::success(tr_with(
                "shell.hier_group.grouped_objects",
                &[("members", &members)],
            )),
            Self::Ungrouped { groups: 1 } => Toast::success(tr("shell.hier_group.ungrouped")),
            Self::Ungrouped { groups } => Toast::success(tr_with(
                "shell.hier_group.ungrouped_groups",
                &[("groups", &groups)],
            )),
            // ⚠️ `warning` e não `error`: o app está correcto e o documento está intacto — o que
            // falhou foi a pré-condição do gesto. E a frase diz **o que fazer**, não só o que
            // faltou: uma recusa que não ensina o próximo gesto obriga a adivinhar.
            Self::NeedsTwo => Toast::warning(tr("shell.hier_group.select_at_least_2")),
            Self::NotGrouped => Toast::warning(tr("shell.hier_group.nothing_in_the")),
            // ⚠️ A MESMA frase que o *Merge Sprites* usa para a mesma ambiguidade, com o sujeito
            // trocado. Duas redacções para a mesma situação ensinariam que são situações
            // diferentes.
            Self::ClickedOutsideSelection => {
                Toast::warning(tr("shell.hier_group.right_click_on_one_of"))
            }
        }
    }
}

/// Sobre quem o verbo age — ou porque ele não age.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// Estes objectos, **na ordem da selecção**.
    ///
    /// ⚠️ A ordem é load-bearing: o `group_entities` insere os filhos por ela e o `Children`
    /// preserva-a, então ela vira a ordem de z **dentro** do grupo.
    These(Vec<u64>),
    /// O clique caiu FORA de uma selecção múltipla. ⚠️ Não é um erro do artista — é uma ambiguidade
    /// que só ele resolve, e o verbo diz-lho em vez de escolher por ele.
    ClickedOutsideSelection,
}

/// A lei do sujeito, pura — ver o doc do módulo.
#[must_use]
pub fn subject(row: u64, selected: &[u64]) -> Subject {
    if selected.contains(&row) {
        return Subject::These(selected.to_vec());
    }
    if selected.len() >= 2 {
        return Subject::ClickedOutsideSelection;
    }
    // ⚠️ A linha clicada SOZINHA, e não a união com uma selecção de um só: se o artista tinha um
    // objecto escolhido e carregou com o direito noutro, o que ele apontou foi o segundo. Unir os
    // dois agruparia por acidente algo que ele nunca pôs junto.
    Subject::These(vec![row])
}

/// Aplica o verbo. A mutação vai pelas portas que já existem — esta função decide **o quê** e
/// **o que dizer**, e não reimplementa nenhuma delas.
pub fn apply(sim: &mut ph2d_ecs::SimWorld, subject: &Subject, group: bool) -> Outcome {
    let Subject::These(subjects) = subject else {
        return Outcome::ClickedOutsideSelection;
    };
    if group {
        // ⚠️ O nome conta os MEMBROS de topo, não os sujeitos: dois caminhos do mesmo grupo são um
        // membro só, e um "Group 2" sobre uma coisa só seria mentira no primeiro sítio que o
        // artista lê.
        let membros = ph2d_vec_entities::entities::top_members(sim, subjects);
        let nome = tr_with("shell.hier_group.group", &[("membros", &(membros.len()))]);
        ph2d_vec_entities::entities::group_entities(sim, subjects, nome).map_or(
            Outcome::NeedsTwo,
            |group| Outcome::Grouped {
                group,
                members: membros.len(),
            },
        )
    } else {
        match ph2d_vec_entities::entities::ungroup_entities(sim, subjects) {
            0 => Outcome::NotGrouped,
            groups => Outcome::Ungrouped { groups },
        }
    }
}

#[cfg(test)]
#[path = "hier_group_tests.rs"]
mod tests;
