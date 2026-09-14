//! ⭐⭐⭐ **A secção TAGS** (TOP-20 #9, W3) — o snapshot que a secção lê e o commit que ela escreve.
//! Irmão do [`super::inspector_camera`], pela mesma razão dele.
//!
//! # ⚠️ Ela é a ÚNICA secção do Inspector cujo commit toca em DOIS documentos
//!
//! As irmãs escrevem um componente e acabaram. Aqui, *Create “…”* escreve a **árvore do projecto**
//! (que não é o mundo) **e** o componente do objecto, no mesmo gesto — é por isso que o commit pede
//! `&mut TagTree` e corre na fase que o tem, e não no `inspector_commits`.
//!
//! ⛔ **E é por isso que ele recusa criar quando o objecto está cheio:** meio gesto deixaria na
//! árvore uma tag que ninguém pediu, e o artista veria a lista crescer sem o chip aparecer.
//!
//! # ⚠️ A cache do documento NÃO se invalida aqui, e isso é uma propriedade, não um esquecimento
//!
//! O [`ph2d_app_components::tags_doc::TagsCache`] compara a **revisão** da árvore, e toda mutação a
//! incrementa ⇒ o `doc()` do save seguinte já recodifica sozinho. O `invalidate()` é para quem
//! **SUBSTITUI** a árvore (um restore nasce na revisão `0` e colidiria com a anterior) — ver o gate
//! `the_cache_forgets_the_previous_tree_even_when_the_revision_collides`.
//!
//! # ⚠️ O snapshot lê o conjunto DIRECTO, e é o único sítio do repo autorizado a fazê-lo
//!
//! Ele MOSTRA os chips — não responde *«pertence?»*, que alcança a subárvore e passa por
//! `ph2d_ecs::tags::belongs`. O censo `only_the_door_reads_tags` nomeia este ficheiro com a razão.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::tags::{TAGS_MAX, Tags};
use ph2d_ecs::{Entity, World};
use ph2d_editor_core::{InspectorTagRow, InspectorTagsInfo, TagsFieldEdit};
use ph2d_inspector_ordering::queue_set;
use ph2d_tags::{Tag, TagId, TagTree};

const TAGS: &str = "ph2d::ecs::Tags";

/// Uma linha do painel a partir de uma tag da árvore.
///
/// ⚠️ **O rótulo e a profundidade saem da [`Tag`]**, que os deriva pela álgebra de caminhos — o
/// painel nunca corta um `"Enemy/Flying"` à mão, senão a regra do separador viveria em dois sítios.
fn row(t: &Tag) -> InspectorTagRow {
    InspectorTagRow {
        id: t.id.0,
        path: t.path.clone(),
        label: t.label().to_string(),
        depth: t.depth(),
    }
}

/// O snapshot da secção, ou `None` quando o objecto não tem o componente (ADR-0166).
///
/// ⚠️ **Os chips saem da travessia da ÁRVORE**, e por isso vêm na ordem dela.
///
/// ⭐ **A razão é UMA ordem para tags em todo o app**: a caixa de escolha oferece-as na ordem da
/// árvore e o painel *Tags* (W4) vai listá-las na mesma. Derivar `on_object` do conjunto do
/// componente daria a ordem dos **ids** (a de criação), e o artista veria o mesmo conjunto de tags
/// em duas ordens diferentes no MESMO ecrã.
///
/// ⛔ **E NÃO é «assim os chips não saltam ao renomear»** — com a ordem da árvore eles saltam, de
/// propósito: o nome mudou, logo o lugar alfabético mudou. É a ordem dos **ids** que os deixaria
/// parados. ⚠️ *Esta linha afirmava o contrário, e foi uma mutação SOBREVIVENTE que mandou reler a
/// fixtura e, com ela, o motivo escrito ao lado — um gate fraco esconde um argumento invertido tão
/// bem como esconde um defeito.*
pub(super) fn build_tags_info(
    world: &World,
    tree: &TagTree,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorTagsInfo> {
    let tags = world.get::<Tags>(Entity::from_bits(entity_bits))?;
    let on_object: Vec<InspectorTagRow> = tree
        .tags()
        .filter(|t| tags.direct_ids().any(|i| i == t.id))
        .map(row)
        .collect();
    Some(InspectorTagsInfo {
        entity_bits,
        // ⚠️ O tecto mede o conjunto do COMPONENTE, não os chips: um id órfão (de uma tag apagada
        // sem o `scrub`) ocupa lugar e não se desenha, e dizer *«cabe»* ali seria oferecer um gesto
        // que o `Tags::insert` recusa.
        full: tags.len() >= TAGS_MAX,
        on_object,
        selected_count,
    })
}

/// ⭐⭐⭐ **A ÁRVORE DO PROJECTO, na ordem dela** — as opções de toda caixa de escolha de tag.
///
/// ⛔ **Não depende da selecção**, e é essa a diferença que a torna uma porta própria: ela é
/// publicada em TODO quadro, com ou sem objecto escolhido, porque o segundo consumidor (o alvo de
/// uma *Signal Action*) vive num objecto que pode não ter `Tags` nenhum.
pub(super) fn tag_tree_rows(tree: &TagTree) -> Vec<InspectorTagRow> {
    tree.tags().map(row).collect()
}

/// Aplica uma [`TagsFieldEdit`]. Devolve **`true` se a ÁRVORE mudou** — o que o chamador precisa de
/// saber para o caso de a querer regravar, e o que nenhuma das irmãs tem.
///
/// ⚠️ **A resposta é a REVISÃO comparada, nunca «chamei o `create`»**: criar um caminho que já
/// existe devolve o id de lá e não mexe em nada, e um `true` nesse caso seria uma mentira barata
/// que só se nota num gate de contagem de escritas.
pub(super) fn apply_tags_edit(
    world: &World,
    tree: &mut TagTree,
    entity_bits: u64,
    edit: &TagsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    let entity = Entity::from_bits(entity_bits);
    let Some(mut t) = world.get::<Tags>(entity).cloned() else {
        return false;
    };
    let antes = tree.revision();
    let mudou = match edit {
        TagsFieldEdit::Add(id) => t.insert(TagId(*id)),
        TagsFieldEdit::Remove(id) => t.remove(TagId(*id)),
        TagsFieldEdit::Create(path) => {
            // ⛔ Ver o cabeçalho: cheio ⇒ nem a árvore cresce.
            if t.len() >= TAGS_MAX {
                false
            } else {
                match tree.create(path) {
                    Ok(id) => t.insert(id),
                    // ⚠️ O único erro que o `create` devolve é o caminho VAZIO, e o painel já não
                    // oferece a linha *Create* sem texto — isto é a cerca, não o caminho normal.
                    Err(_) => false,
                }
            }
        }
    };
    if mudou {
        queue_set(queue, registry, entity_bits, TAGS, &t);
    }
    tree.revision() != antes
}

#[cfg(test)]
#[path = "inspector_tags_tests.rs"]
mod tests;
