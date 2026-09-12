//! A ponte entre o documento vetorial e a árvore do editor (ADR-0110).
//!
//! Cada `VecPath` tem uma entidade ECS que o referencia (`VecPathRef`). O
//! documento é dono da **geometria**; a entidade é dona da **identidade e do lugar
//! na árvore** — nome, visibilidade, trava, pai, ordem. É por isso que um path
//! vetorial pode ser filho de um sprite, e que um grupo é só uma entidade comum
//! com filhos: não há tipo de nó especial, e nada teve de ser inventado.
//!
//! Este módulo mantém o único invariante que a ponte exige: **um path ⟺ uma
//! entidade**. Path novo ⇒ entidade spawnada. Path apagado ⇒ entidade despawnada.
//! Entidade apagada pela Hierarquia ⇒ path removido do documento. Tudo em
//! [`sync`], uma vez por frame, antes de qualquer leitura.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecPathRef};
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState};

/// `VecPathId` → `Entity::to_bits()`. Autoritativo: só ele decide se um path
/// perdeu a entidade (ou vice-versa), então nem um respawn fantasma nem um path
/// órfão podem acontecer.
// ⭐ **O alias mudou-se para a crate da família** (W2/L4 Fase B): 18 ficheiros precisavam só
// dele e ficavam presos a este módulo, que está preso a três predicados de outras famílias.
pub use crate::entity_map::VecEntityMap;

/// Nome inicial de um path novo. O usuário renomeia pela Hierarquia como qualquer
/// entidade; o id só garante unicidade no nascimento.
// ⚠️ `pub` sob `test-support` (HOWTO §2.5): um gate da SHELL afirma que um path novo nasce com
// este nome, e do outro lado de uma crate ele não via a função. ⛔ Escrever `format!("Path {id}")`
// no gate seria a SEGUNDA resposta à mesma pergunta, e a que envelhece é sempre a do teste.
#[cfg(not(any(test, feature = "test-support")))]
fn initial_name(id: VecPathId) -> String {
    initial_name_impl(id)
}
#[cfg(any(test, feature = "test-support"))]
#[must_use]
pub fn initial_name(id: VecPathId) -> String {
    initial_name_impl(id)
}

fn initial_name_impl(id: VecPathId) -> String {
    format!("Path {id}")
}

/// Reconcilia documento e árvore. Chamado **antes** de ler ordem, visibilidade ou
/// seleção, para que o frame veja um estado consistente.
///
/// Três direções, nesta ordem:
/// 1. entidade sumiu (Delete na Hierarquia) ⇒ apaga o path;
/// 2. path sumiu (Delete no canvas / booleana / cut) ⇒ despawna a entidade;
/// 3. path novo ⇒ spawna a entidade, no topo da ordem de raiz.
pub fn sync(sim: &mut SimWorld, scene: &mut VecScene, map: &mut VecEntityMap) {
    // 1. Entidades que a Hierarquia apagou levam o path junto.
    let vanished: Vec<VecPathId> = map
        .iter()
        .filter(|(_, bits)| sim.world().get_entity(Entity::from_bits(**bits)).is_err())
        .map(|(&id, _)| id)
        .collect();
    for id in vanished {
        scene.remove_path(id);
        map.remove(&id);
    }

    // 2. Paths que sumiram do documento levam a entidade junto.
    //
    // ⚠️⚠️ **O conjunto não é higiene — ele é o TETO deste passe.** Escrito com um
    // `scene.paths().iter().any(…)` dentro do filtro, este passo é **O(formas²)**, e a medição
    // (`the_second_pass_costs_this_much_of_a_frame`) apanhou-o quando a rede do fim do quadro o
    // pôs a correr duas vezes: `100 → 0,015 ms`, `1 000 → 0,451`, **`5 000 → 10,459`** — 5× as
    // formas por 23× o relógio, que é lógica e não carga. A 5 000 formas o `sync` sozinho comia
    // **63 %** de um quadro de 16,7 ms, e comia-o **desde antes desta wave**: a segunda passagem
    // não criou o custo, revelou-o.
    //
    // ⚠️ **`BTreeSet`, nunca `HashSet`** — a espinha do determinismo desta casa (`CLAUDE.md` §5.1,
    // física), e aqui ele não custa nada: o passe percorre o `map`, que já é um `BTreeMap`.
    let vivos: std::collections::BTreeSet<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    let dead: Vec<(VecPathId, u64)> = map
        .iter()
        .filter(|(id, _)| !vivos.contains(id))
        .map(|(&id, &bits)| (id, bits))
        .collect();
    for (id, bits) in dead {
        if let Ok(e) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
            e.despawn();
        }
        map.remove(&id);
    }

    // 3. Paths novos ganham entidade — **no FIM da lista**, que desde a lei de Godot
    //    (2026-08-04) é a FRENTE, e é onde o artista acabou de desenhar.
    //
    //    Enio: *"quando se cria um objeto novo ele vai para o último abaixo na hierarquia, mas
    //    aqui no nosso ele aparece no topo"*. Não é uma segunda decisão: é a MESMA — enquanto a
    //    primeira linha era a da frente, nascer na frente obrigava a abrir espaço no começo e a
    //    empurrar toda raiz um lugar para baixo; com a lista a correr fundo → topo, nascer na
    //    frente é simplesmente **apender**.
    //
    //        RootOrder BAIXO  → primeira linha  → FUNDO
    //        RootOrder ALTO   → última linha    → FRENTE
    let missing: Vec<VecPathId> = scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| !map.contains_key(id))
        .collect();
    if missing.is_empty() {
        return;
    }
    // ⚠️ O `next_root_order` é a porta única de *"qual é o próximo lugar livre?"* — a mesma que o
    // Flip e o envelope usam. Sem o shift, o custo deixou de ser O(raízes) por forma nova.
    let base = next_root_order(sim);
    for (k, id) in missing.iter().enumerate() {
        // **O nome passa pela porta do nome ÚNICO** (`ph2d_unique_name::unique_name`), a mesma que o
        // import e o rename usam. `initial_name` é único entre PATHS (o id é), mas não no MUNDO:
        // basta o artista ter renomeado um sprite para "Path 3" e a próxima forma com id 3 nasce
        // homônima dele.
        //
        // Nome duplicado não é cosmético desde o W4.T6: a animação reencontra o objeto **pelo
        // nome** (`wire_id` = hash do `Name`), então dois homônimos fazem duas tracks colarem no
        // MESMO objeto — e a outra fica sem dono, em silêncio. O nome é identidade agora.
        let name = ph2d_unique_name::unique_name(sim, &initial_name(*id));
        let order = base.saturating_add(u32::try_from(k).unwrap_or(0));
        let e = sim.world_mut().spawn((
            Transform::default(),
            Name::new(name),
            VecPathRef(*id),
            RootOrder(order),
        ));
        map.insert(*id, e.id().to_bits());
    }
}

/// A ordem de **z** (a projeção da árvore) e quem a reescreve — módulo irmão, pelo teto de 600
/// LOC da shell.
#[path = "zorder.rs"]
pub mod zorder;
pub use zorder::{restack, z_order};

/// Reconstrói o mapa path↔entidade **a partir do mundo** — varre cada `VecPathRef`
/// e devolve `VecPathId → Entity::to_bits()`.
///
/// É o que um restore (undo ou load de projeto) precisa ANTES do primeiro [`sync`].
/// O mapa é runtime-only e não é serializado; sem este rebuild, o `sync` veria o
/// mapa vazio e trataria cada path restaurado como novo — spawnando um SEGUNDO
/// conjunto de entidades e deixando as restauradas órfãs. Com o rebuild, as três
/// direções do `sync` viram no-op e a ponte fica consistente de graça.
#[must_use]
pub fn rebuild_map(sim: &mut SimWorld) -> VecEntityMap {
    let mut map = VecEntityMap::new();
    let mut q = sim.world_mut().query::<(Entity, &VecPathRef)>();
    for (e, vp) in q.iter(sim.world()) {
        map.insert(vp.0, e.to_bits());
    }
    map
}

/// O próximo `RootOrder` livre (o maior em uso + 1). `RootOrder(u32::MAX)` é o
/// "sem ordem" das raízes que nunca receberam uma, então não conta.
pub fn next_root_order(sim: &mut SimWorld) -> u32 {
    let mut q = sim.world_mut().query::<&RootOrder>();
    let max = q
        .iter(sim.world())
        .map(|r| r.0)
        .filter(|&o| o != u32::MAX)
        .max();
    max.map_or(0, |m| m.saturating_add(1))
}

/// **O estado de vista que um GESTO deve consultar** — o que a árvore diz AGORA, mais os fatos
/// que o último DESENHO derivou.
///
/// ⚠️ Existe porque as duas metades têm relógios diferentes, e as duas estão certas:
///
/// - *escondido* e *travado* saem do mundo **deste instante** (o artista pode ter acabado de
///   apagar o olhinho, e o clique tem de honrá-lo já);
/// - os **intervalos das molduras**, as **poses do auto layout** e **quem foi ABSORVIDO por uma
///   booleana viva** são resultado do passe de desenho — e o artista clica no que está na TELA,
///   que é o último frame desenhado.
///
/// Sem esta fusão o gesto vê as três listas VAZIAS e decide como se nenhuma moldura existisse:
/// a moldura ganha o clique dos próprios filhos, o hit-test procura cada forma colocada no
/// lugar de onde ela saiu, e um operando absorvido fica inalcançável pelo canvas.
#[must_use]
pub fn view_state_for_pick(
    sim: &SimWorld,
    map: &VecEntityMap,
    derived: &VecViewState,
) -> VecViewState {
    let mut v = view_state(sim, map);
    v.clips.clone_from(&derived.clips);
    v.poses.clone_from(&derived.poses);
    v.absorbed.clone_from(&derived.absorbed);
    v
}

/// O que a árvore esconde ou trava, com a herança já resolvida.
///
/// Visibilidade é `AND` da cadeia de pais (esconder um grupo esconde os filhos sem
/// tocar no flag deles). Trava usa `is_locked_for_edit`, o mesmo predicado do gizmo
/// de sprite: `Locked` no próprio, ou `GroupedChildren` em algum ancestral.
#[must_use]
pub fn view_state(sim: &SimWorld, map: &VecEntityMap) -> VecViewState {
    let w = sim.world();
    let mut view = VecViewState::default();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if w.get_entity(e).is_err() {
            continue;
        }
        if !visible_chain(w, e) {
            view.hidden.push(id);
        }
        if ph2d_ecs::is_locked_for_edit(w, e) {
            view.locked.push(id);
        }
        // ⭐ **O preenchimento do balde é DERIVADO** (plano 40): a área é re-cozida a partir das
        // linhas, então ela não tem nós próprios para agarrar — as alças do nó são as do traço.
        if w.get::<ph2d_ecs::VecBucketFill>(e).is_some() {
            view.derived.push(id);
        }
        // ⭐⭐⭐ **O ISOLAMENTO do *Edit Prefab*** (Enio, 2026-09-07: *«o canvas deve ser borrado
        // levemente… e o Prefab aparece acima de tudo, livre do blur»*).
        //
        // ⚠️ **A pergunta é o `MasterEditing`, e ele é DERIVADO da selecção** — a mesma marca que
        // faz a receita voltar a ser visível ([`crate::render_loop::master_editing`]), carimbada na
        // sub-árvore inteira. ⇒ o conjunto exempto é exactamente o que já está na tela por causa
        // do gesto, e não uma segunda resposta a *«o que é a receita aberta?»*.
        if w.get::<ph2d_ecs::MasterEditing>(e).is_some() {
            view.isolated.push(id);
        }
    }
    view
}

/// Teto de profundidade das caminhadas de ancestral (defesa, não limite de produto).
/// Partilhado com o irmão [`selection`] — uma árvore corrompida tem UMA profundidade máxima.
pub use crate::entity_map::MAX_DEPTH;

/// `Visibility` do próprio E de cada ancestral.
fn visible_chain(w: &ph2d_ecs::World, entity: Entity) -> bool {
    let mut cur = Some(entity);
    // A cadeia é acíclica (`hierarchy_set_parent` recusa ciclos); o teto defende
    // contra um save corrompido.
    for _ in 0..MAX_DEPTH {
        let Some(e) = cur else { return true };
        // ⭐⭐ **A MESMA porta que o extract de sprites usa** — o olho do artista **e** *«uma receita
        // não está na cena»*. O porquê vive lá (F4.6).
        if ph2d_entity_visibility::off_canvas::is_off_canvas(w, e) {
            return false;
        }
        // ⭐⭐ **Ser MEMBRO de um conjunto de Morph States esconde, e isso é DERIVADO** (plano 32
        // W11f): a lista de estados são os filhos, então a ocultação tem de sair da MESMA pergunta
        // — senão arrastar na Hierarquia move uma metade e deixa a outra para trás, nos dois
        // sentidos. ⚠️ Ela ACRESCENTA uma razão para esconder; o `Visibility` do artista fica
        // intacto e volta a valer sozinho quando a forma sai.
        if crate::morph_set::is_set_member(w, e) {
            return false;
        }
        cur = w.get::<ChildOf>(e).map(|c| c.parent());
    }
    true
}

/// **Agrupar e desagrupar** — módulo irmão pelo mesmo tecto e pelo mesmo critério: aqui em cima
/// mora a ponte `path ⟺ entidade`; ali, o verbo que muda a ÁRVORE.
#[path = "entities_group.rs"]
mod group;
/// **A ancestralidade e o que uma SELEÇÃO significa** — módulo irmão, pelo teto de 600 LOC
/// da shell. O corte é por assunto: aqui em cima mora o que a ponte MANTÉM (a identidade
/// path ⟺ entidade, a ordem, o que a árvore esconde); ali, o que a árvore RESPONDE — *quem é
/// o objeto que este clique nomeia, e o que selecioná-lo significa*.
#[path = "entities_selection.rs"]
mod selection;
pub use group::{group_entities, top_members, ungroup_entities};
pub use selection::{object_selection_for, selection_paths, subtree_paths, top_ancestor};

// ⚠️ **Os GATES deste passe ficaram na shell** (HOWTO §1.2): eles atravessam o `input_dispatch`
// e o `render_loop::master_editing`, e medem a ponte a partir do GESTO — em qualquer outra crate
// mediriam zero. Vivem em `shells/desktop/src/vec_entities_tests.rs`.
//
// ⭐ **A FIXTURA, essa, veio** — e tinha de vir: o irmão da SELECÇÃO usa-a de DENTRO da crate e
// aquele ficheiro de fora, e *duas fixturas para a mesma ponte seriam duas respostas a «como
// nasce uma cena de teste?»* (a razão que o doc dela já dava, agora a atravessar uma fronteira).
// ⚠️ `test-support` porque um `#[cfg(test)]` é falso numa dependência (HOWTO §2.5).

/// A cena de teste vazia: mundo, documento e o mapa que os ata.
#[cfg(any(test, feature = "test-support"))]
#[must_use]
pub fn setup() -> (SimWorld, VecScene, VecEntityMap) {
    (SimWorld::default(), VecScene::new(), VecEntityMap::new())
}

/// A entidade de um path, pelo mapa.
#[cfg(any(test, feature = "test-support"))]
#[must_use]
pub fn bits(map: &VecEntityMap, id: VecPathId) -> Entity {
    Entity::from_bits(map[&id])
}
