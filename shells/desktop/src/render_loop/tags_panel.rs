//! ⭐⭐⭐ **O painel TAGS** (TOP-20 #9, W4) — o instantâneo que ele lê e os gestos que ele escreve.
//! Irmão do [`super::inspector_tags`], e a metade que falta a ele: ali o sujeito é um OBJECTO, aqui
//! é a **árvore do projecto**.
//!
//! # ⚠️ A contagem só é derivada com o painel ABERTO
//!
//! A coluna *«quantos objectos»* é [`ph2d_ecs::tags::counts`], e o doc dela tem a tabela: no extremo
//! (10 000 objectos todos marcados, 584 tags) ela custa `2,97 ms` contra um quadro de `16,7`. Com o
//! painel fechado a coluna não existe — *uma medição que ninguém vê não se faz* —, e é por isso que
//! o chamador pergunta a visibilidade ANTES de construir o instantâneo.
//!
//! # ⚠️ A recusa é ESTADO DA SHELL, e não um valor derivado
//!
//! `TagTree::rename` devolve um `Result` no instante do gesto, e o painel repinta dezenas de vezes
//! depois disso. A frase tem de sobreviver até ao gesto SEGUINTE, então ela vive ao lado da árvore
//! ([`crate::app_state_gfx::AppGfx::tags_problem`]) e é publicada em todo quadro. ⛔ Derivá-la
//! outra vez a cada quadro obrigaria a repetir o gesto para saber porque ele falhou.

use ph2d_ecs::tags::{counts, scrub, tagged};
use ph2d_ecs::{Entity, World};
use ph2d_editor_core::TagTreeEdit;
use ph2d_editor_core::{TagsPanelInfo, TagsPanelRow};
use ph2d_tags::{TagError, TagId, TagTree};

/// O nome de omissão de uma tag nova. ⚠️ **Em inglês e literal**, como os rótulos dos painéis
/// vizinhos — ele é o texto que o artista **escreve por cima** no mesmo gesto, não uma etiqueta que
/// fica.
const NOME_BASE: &str = "Tag";

/// **O que a shell faz depois de um gesto do painel.**
///
/// ⚠️ Os quatro braços existem porque as consequências são de naturezas diferentes — o documento
/// mudou · nada mudou · a lei recusou · a SELECÇÃO da cena tem de mudar (que não é documento). Um
/// `bool` obrigaria o chamador a redescobrir qual deles aconteceu.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum TagEditOutcome {
    /// Nada mudou (um alvo que já não existe, ou um gesto sobre nada).
    Nothing,
    /// A árvore mudou. `born` é a tag que o gesto CRIOU — o painel abre o campo de renomear nela.
    Changed { born: Option<u64> },
    /// A lei recusou. `tag` é a linha onde a frase é pintada (`0` = nenhuma).
    Refused { tag: u64, why: TagError },
    /// Escolher estes objectos na cena — um gesto de EDITOR, não do documento.
    Select(Vec<Entity>),
}

/// **O instantâneo do painel**: a árvore inteira com a contagem de cada tag.
///
/// ⚠️ **Uma passagem só pelo mundo** ([`counts`]), nunca [`tagged`] em laço: a segunda forma custa
/// `tags × objectos` **por quadro**, e a tabela do `counts` tem os números.
///
/// ⚠️ **O `subtree` nunca é `0`** — ele conta a própria tag —, e é ele que faz o botão *Delete*
/// dizer que apagar `Enemy` leva `Flying` e `Boss` junto.
pub(super) fn build_tags_panel_info(
    world: &World,
    tree: &TagTree,
    problem: Option<&(u64, String)>,
) -> TagsPanelInfo {
    let membros = counts(world, tree);
    TagsPanelInfo {
        rows: tree
            .tags()
            .map(|t| TagsPanelRow {
                id: t.id.0,
                label: t.label().to_string(),
                depth: t.depth(),
                members: membros.get(&t.id).copied().unwrap_or(0),
                subtree: tree.subtree(t.id).len(),
            })
            .collect(),
        problem: problem.cloned(),
    }
}

/// O caminho de uma tag nova debaixo de `parent`, com um rótulo que ainda não está ocupado ali.
///
/// ⚠️ **A busca é pela PORTA `find`**, que compara DOBRADO: um `Tag 2` ao lado de um `tag 2` seria
/// recusado pelo `create`, e nascer com um nome que vai ser recusado é o gesto que o painel existe
/// para não oferecer.
fn caminho_novo(tree: &TagTree, parent: Option<TagId>) -> Option<String> {
    let prefixo = match parent {
        Some(p) => format!("{}/", tree.get(p)?.path),
        None => String::new(),
    };
    // ⚠️ O tecto é a própria árvore: com `n` tags há no máximo `n` nomes ocupados, então `n + 1`
    // tentativas bastam sempre. ⛔ Um `loop` sem fim aqui seria o app pendurado num clique.
    for i in 1..=tree.len() + 1 {
        let nome = if i == 1 {
            NOME_BASE.to_string()
        } else {
            format!("{NOME_BASE} {i}")
        };
        let caminho = format!("{prefixo}{nome}");
        if tree.find(&caminho).is_none() {
            return Some(caminho);
        }
    }
    None
}

/// **Aplica um gesto do painel.** ⚠️ `&mut World` porque apagar leva a PERTENÇA junto — é o par
/// obrigatório do `TagTree::delete`, no mesmo gesto e no mesmo passo de undo.
pub(super) fn apply_tag_tree_edit(
    world: &mut World,
    tree: &mut TagTree,
    edit: &TagTreeEdit,
) -> TagEditOutcome {
    match edit {
        TagTreeEdit::Create { parent } => {
            let pai = parent.map(TagId);
            if let Some(p) = pai
                && tree.get(p).is_none()
            {
                return TagEditOutcome::Refused {
                    tag: parent.unwrap_or(0),
                    why: TagError::Missing,
                };
            }
            let Some(caminho) = caminho_novo(tree, pai) else {
                return TagEditOutcome::Nothing;
            };
            match tree.create(&caminho) {
                Ok(id) => TagEditOutcome::Changed { born: Some(id.0) },
                Err(why) => TagEditOutcome::Refused {
                    tag: parent.unwrap_or(0),
                    why,
                },
            }
        }
        TagTreeEdit::Rename { id, label } => match tree.rename(TagId(*id), label) {
            Ok(()) => TagEditOutcome::Changed { born: None },
            Err(why) => TagEditOutcome::Refused { tag: *id, why },
        },
        TagTreeEdit::Move { id, parent } => match tree.move_under(TagId(*id), parent.map(TagId)) {
            Ok(()) => TagEditOutcome::Changed { born: None },
            Err(why) => TagEditOutcome::Refused { tag: *id, why },
        },
        TagTreeEdit::Delete { id } => {
            // ⚠️ **As duas metades no MESMO gesto.** O `delete` devolve a subárvore que saiu e o
            // `scrub` tira esses ids dos objectos — saltar a segunda deixaria ids órfãos que o
            // `counts` não conta e que um id futuro nunca reusa, mas que ocupam lugar no tecto do
            // objecto.
            let saem = tree.delete(TagId(*id));
            if saem.is_empty() {
                return TagEditOutcome::Nothing;
            }
            scrub(world, &saem);
            TagEditOutcome::Changed { born: None }
        }
        TagTreeEdit::SelectTagged { id } => {
            // ⚠️ **Aqui a porta LENTA é a certa:** é uma tag só, uma vez, por clique — e o que se
            // quer não é a contagem, são as ENTIDADES, que o `counts` não devolve.
            let alvos = tagged(world, tree, TagId(*id));
            if alvos.is_empty() {
                TagEditOutcome::Nothing
            } else {
                TagEditOutcome::Select(alvos)
            }
        }
    }
}

/// Os objectos que perderiam a tag — o número que o botão *Delete* mostra.
///
/// ⚠️ Ela existe para o **gate**, não para o produto: o painel lê o `members` do instantâneo, que já
/// é este número. *Duas respostas à mesma pergunta divergem* — o gate prova que não divergem.
#[cfg(test)]
pub(super) fn membros_de(world: &World, tree: &TagTree, id: TagId) -> usize {
    world
        .iter_entities()
        .filter(|e| {
            e.get::<ph2d_ecs::tags::Tags>()
                .is_some_and(|t| ph2d_ecs::tags::belongs(t, tree, id))
        })
        .count()
}

#[cfg(test)]
#[path = "tags_panel_tests.rs"]
mod tests;
