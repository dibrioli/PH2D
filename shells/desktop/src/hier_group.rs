//! ⭐⭐⭐ **AGRUPAR / DESAGRUPAR pela Hierarquia** (Enio, 2026-08-30).
//!
//! # O que faltava, e o que não faltava
//!
//! O modelo de grupo **já existia inteiro**: uma entidade sem geometria própria com filhos
//! ([`crate::vec_entities`]), com selecção que entra e sai do grupo inteira, gizmo de canvas,
//! árvore recolhível na Hierarquia e persistência. Até o gesto existia — `Ctrl+G` / `Ctrl+Shift+G`.
//!
//! ⛔ **O que não existia era ALCANCE.** Nenhum menu, botão, rótulo ou entrada de paleta em todo o
//! app dizia a palavra *Group*. É a lei deste repo aplicada à UI: *uma ferramenta que nenhum passo
//! escrito chama pelo nome morre* — e esta estava morta para quem não adivinhasse o atalho.
//!
//! ⚠️ **E o atalho tinha três cercas que ninguém veria:** ele só responde com a ferramenta **Vector**
//! em mãos, lê **apenas** a selecção de caminhos da caneta (uma sprite escolhida na Hierarquia era
//! invisível para ele, embora o `group_entities` a aceite de bom grado), e falhava para o `stderr`.
//! O verbo da Hierarquia entra por outra porta e não herda nenhuma das três.
//!
//! # A lei do SUJEITO — e ela **já estava decidida nesta casa**
//!
//! O menu é por LINHA, mas agrupar é sobre um CONJUNTO. O *Merge Sprites* enfrentou exactamente
//! esta pergunta e a resposta dele fica: **se a linha clicada está na selecção, o sujeito é a
//! selecção**; se está fora e há uma selecção múltipla, o verbo **não age — ORIENTA**
//! (*"right-click on one of the selected objects"*), porque agir sobre a união traria para dentro
//! do grupo um objecto que o artista não escolheu, e agir só sobre a linha faria *Group* falhar
//! por ter um sujeito só.
//!
//! ⛔ Inventar aqui uma terceira lei para a mesma pergunta seria a divergência que este repo paga
//! sempre: *duas respostas à mesma pergunta, e a que o artista encontra é a que envelhece.*

use ph2d_editor::Toast;

/// O que o verbo fez — ou porque não fez nada.
///
/// ⚠️ **Os dois casos de recusa são variantes próprias, e não um `None`.** Cada um tem uma frase
/// diferente a dizer ao artista, e *um verbo que come o clique em silêncio é pior que um ausente* —
/// é a lei que a própria tabela deste menu declara.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
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
    pub(crate) fn toast(self) -> Toast {
        match self {
            Self::Grouped { members, .. } => Toast::success(format!("Grouped {members} objects")),
            Self::Ungrouped { groups: 1 } => Toast::success("Ungrouped"),
            Self::Ungrouped { groups } => Toast::success(format!("Ungrouped {groups} groups")),
            // ⚠️ `warning` e não `error`: o app está correcto e o documento está intacto — o que
            // falhou foi a pré-condição do gesto. E a frase diz **o que fazer**, não só o que
            // faltou: uma recusa que não ensina o próximo gesto obriga a adivinhar.
            Self::NeedsTwo => Toast::warning("Select at least 2 objects to group"),
            Self::NotGrouped => Toast::warning("Nothing in the selection is inside a group"),
            // ⚠️ A MESMA frase que o *Merge Sprites* usa para a mesma ambiguidade, com o sujeito
            // trocado. Duas redacções para a mesma situação ensinariam que são situações
            // diferentes.
            Self::ClickedOutsideSelection => {
                Toast::warning("Right-click on one of the selected objects")
            }
        }
    }
}

/// Sobre quem o verbo age — ou porque ele não age.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Subject {
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
pub(crate) fn subject(row: u64, selected: &[u64]) -> Subject {
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
pub(crate) fn apply(sim: &mut ph2d_ecs::SimWorld, subject: &Subject, group: bool) -> Outcome {
    let Subject::These(subjects) = subject else {
        return Outcome::ClickedOutsideSelection;
    };
    if group {
        // ⚠️ O nome conta os MEMBROS de topo, não os sujeitos: dois caminhos do mesmo grupo são um
        // membro só, e um "Group 2" sobre uma coisa só seria mentira no primeiro sítio que o
        // artista lê.
        let membros = crate::vec_entities::top_members(sim, subjects);
        let nome = format!("Group {}", membros.len());
        crate::vec_entities::group_entities(sim, subjects, nome).map_or(
            Outcome::NeedsTwo,
            |group| Outcome::Grouped {
                group,
                members: membros.len(),
            },
        )
    } else {
        match crate::vec_entities::ungroup_entities(sim, subjects) {
            0 => Outcome::NotGrouped,
            groups => Outcome::Ungrouped { groups },
        }
    }
}

#[cfg(test)]
#[path = "hier_group_tests.rs"]
mod tests;
