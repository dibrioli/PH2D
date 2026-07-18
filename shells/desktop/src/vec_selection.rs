//! O casamento de seleção entre o canvas vetorial e a Hierarquia (ADR-0110).
//!
//! A seleção do gizmo é **compartilhada**: sprites e paths vetoriais convivem no
//! mesmo conjunto de `Entity`. Este módulo é dono apenas do **subconjunto
//! vetorial** dela — nunca reescreve o que é de outro tipo. Foi exatamente essa
//! confusão que apagava a seleção dos sprites sozinha (Enio, 2026-07-09).
//!
//! A ponte de identidade (path ⟺ entidade) é o módulo irmão [`crate::vec_entities`].

use crate::vec_entities::{VecEntityMap, subtree_paths};
use ph2d_ecs::{ChildOf, Entity, SimWorld, VecPathRef};
use ph2d_editor::screens::hero::GizmoStateGroup;
use ph2d_vec_scene::{VecPathId, VecScene};

/// Teto de nós visitados numa varredura de sub-árvore (defesa contra save
/// corrompido, não limite de produto).
const MAX_NODES: usize = 4096;

/// Teto de profundidade das caminhadas de ancestral.
const MAX_DEPTH: usize = 64;

/// O espelho da última sincronia de seleção, dos DOIS lados. Ter os dois é o que
/// permite saber **quem** mudou neste frame — e portanto quem manda.
#[derive(Default)]
pub(crate) struct VecSelSync {
    /// Bits **vetoriais** que o gizmo tinha (ordenados). Só os nossos: um sprite
    /// selecionado nunca entra aqui, e por isso nunca é confundido com uma
    /// mudança da árvore.
    bits: Vec<u64>,
    /// Seleção de objeto do pen no mesmo instante.
    paths: Vec<VecPathId>,
}

/// A entidade é "nossa": ela ou algum descendente é um path vetorial. É o que
/// separa, dentro da seleção **compartilhada** do gizmo, o que a linha do vetor
/// pode reescrever do que é de outro tipo (um sprite) e deve ficar intocado.
fn owns_vector(w: &ph2d_ecs::World, root: Entity) -> bool {
    let mut stack = vec![root];
    for _ in 0..MAX_NODES {
        let Some(e) = stack.pop() else { return false };
        if w.get::<VecPathRef>(e).is_some() {
            return true;
        }
        if let Some(kids) = w.get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    false
}

/// Casa a seleção do canvas com a da Hierarquia — nos **dois sentidos**, sem laço
/// e **sem nunca pisar na seleção alheia**.
///
/// A seleção do gizmo é compartilhada: sprites e paths vetoriais convivem nela.
/// Então a regra não pode ser "mudou ⇒ a árvore mandou". É esta, por prioridade:
///
/// 1. **O canvas mandou** — a seleção do pen mudou desde a última sincronia. Só
///    acontece com a ferramenta vetorial ativa (é ela que recebe o ponteiro), e um
///    clique de canvas tem semântica de *substituir*: reescrevemos o gizmo.
/// 2. **A árvore mandou** — o subconjunto **vetorial** do gizmo mudou (clique numa
///    linha, Delete, agrupar). Adotamos no pen e **não tocamos no gizmo**, então um
///    sprite recém-selecionado continua selecionado.
/// 3. Ninguém mandou: nada acontece.
///
/// Ao publicar, um grupo cujas folhas estão TODAS selecionadas também entra — é o
/// que ilumina a linha do grupo na Hierarquia, e não só as dos filhos.
pub(crate) fn sync_selection(
    gizmo: &mut GizmoStateGroup,
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    pen: &mut ph2d_vec_edit::PenTool,
    state: &mut VecSelSync,
    vector_active: bool,
) {
    let w = sim.world();

    // 0. NINGUÉM mandou — as entidades foram **RESPAWNADAS** debaixo de nós.
    //
    // O `ProjectState::restore` (undo, e o load de projeto) despawna o mundo editável e re-spawna do
    // snapshot com **ids NOVOS** — está escrito lá. Os bits que espelhamos morrem nesse instante, e
    // o ramo 2 lê "os meus bits sumiram do gizmo" como *"a árvore deselecionou"*: ele limpa o pen, e
    // o gizmo fica para sempre com bits de uma entidade morta.
    //
    // O sintoma disso é traiçoeiro e foi como o Enio o descreveu: **a ferramenta continua a
    // funcionar e fica invisível.** O recook do envelope varre por QUERY, então a arte segue
    // deformada; mas o overlay (gaiola e pinos) é desenhado a partir dos bits da seleção, e esses
    // já não nomeiam nada.
    //
    // "Sumiram" e "morreram" são fatos DIFERENTES e pedem respostas opostas — e distingui-los é
    // barato, porque uma entidade deselecionada continua EXISTINDO. A seleção do pen é estável
    // (`VecPathId` viaja no snapshot), então a resposta certa é **re-derivar dela**, que é
    // exatamente o que o ramo 1 faz.
    let respawned = !state.bits.is_empty()
        && state
            .bits
            .iter()
            .all(|&b| w.get_entity(Entity::from_bits(b)).is_err());

    // 1. O canvas mandou (ou o mundo foi re-spawnado — mesma resposta: publicar a partir do pen).
    let pen_now = pen.selected_paths().to_vec();
    if vector_active && (pen_now != state.paths || respawned) {
        let mut bits: Vec<u64> = pen_now
            .iter()
            .filter_map(|id| map.get(id).copied())
            .collect();
        // ADR-0129 Fatia 3: se toda a seleção vive sob UM container de Envelope, o gizmo é o
        // CONTAINER SOZINHO — mover/girar/escalar o container move o envelope inteiro, e pôr os
        // filhos junto os CISALHARIA (eles têm geometria de mundo + `Transform` identidade, então
        // o gizmo os moveria uma vez pelo próprio `Transform` e outra pelo do pai). O pen fica com
        // os filhos (a gaiola do Node os alcança pelo container no gizmo). A pergunta "de quem é
        // este envelope?" é do `envelope_live` — porta única, partilhada com o `dissolve`.
        if let Some(container) = crate::envelope_live::sole_container(sim, &bits) {
            gizmo.replace_selection(Some(container));
            state.bits = vec![container];
            state.paths = pen_now;
            return;
        }
        for ancestor in fully_selected_ancestors(sim, scene, map, &bits) {
            if !bits.contains(&ancestor) {
                bits.push(ancestor);
            }
        }
        let primary = pen_now.last().and_then(|id| map.get(id).copied());
        gizmo.replace_selection(primary);
        for b in &bits {
            if Some(*b) != primary {
                gizmo.add_to_selection(*b);
            }
        }
        bits.sort_unstable();
        state.bits = bits;
        state.paths = pen_now;
        return;
    }

    // 2. A árvore mandou — mas só olhamos o que é NOSSO no gizmo.
    let mut mine: Vec<u64> = gizmo
        .iter_selected()
        .filter(|&b| {
            let e = Entity::from_bits(b);
            w.get_entity(e).is_ok() && owns_vector(w, e)
        })
        .collect();
    mine.sort_unstable();
    if mine == state.bits {
        return;
    }
    // Cada entidade nossa contribui com os paths da sub-árvore: selecionar um
    // grupo seleciona o que há de vetorial dentro.
    let mut paths: Vec<VecPathId> = Vec::new();
    for &bits in &mine {
        for id in subtree_paths(sim, scene, Entity::from_bits(bits)) {
            if !paths.contains(&id) {
                paths.push(id);
            }
        }
    }
    // Ordem de z, para a seleção ficar estável entre frames.
    let ordered: Vec<VecPathId> = scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| paths.contains(id))
        .collect();
    pen.select_many(&ordered);
    state.bits = mine;
    state.paths = pen.selected_paths().to_vec();
}

/// Os ancestrais cuja sub-árvore vetorial está INTEIRAMENTE selecionada. Sem eles a
/// linha do grupo nunca acenderia na Hierarquia — só as dos filhos.
fn fully_selected_ancestors(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[u64],
) -> Vec<u64> {
    let w = sim.world();
    let mut out: Vec<u64> = Vec::new();
    for &bits in selected {
        let mut cur = w
            .get::<ChildOf>(Entity::from_bits(bits))
            .map(|c| c.parent());
        for _ in 0..MAX_DEPTH {
            let Some(p) = cur else { break };
            let leaves = subtree_paths(sim, scene, p);
            let all_in = !leaves.is_empty()
                && leaves
                    .iter()
                    .all(|id| map.get(id).is_some_and(|b| selected.contains(b)));
            if all_in && !out.contains(&p.to_bits()) {
                out.push(p.to_bits());
            }
            cur = w.get::<ChildOf>(p).map(|c| c.parent());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec_entities::{VecEntityMap, group_entities, sync};
    use ph2d_ecs::{Name, Transform};
    use ph2d_vec_scene::rectangle;

    /// **APAGAR UMA de duas formas selecionadas PODA o pen — não re-deriva dele.**
    ///
    /// Este é o par do gate `the_pins_survive_an_undo`: os dois falam de bits que morreram, e a
    /// resposta certa é **oposta**. Se TODOS morreram, o mundo foi re-spawnado e a verdade está no
    /// pen (re-derivar). Se **algum** morreu, foi um delete — e a verdade está no gizmo, que ainda
    /// tem os sobreviventes; re-derivar do pen aqui deixaria o id da forma apagada pendurado na
    /// seleção para sempre.
    ///
    /// Foi a mutação `.all()` → `.any()` que sobreviveu e cobrou este teste.
    #[test]
    fn deleting_one_of_two_selected_shapes_prunes_the_pen() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);

        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();
        pen.select_many(&[a, b]);
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);
        assert_eq!(pen.selected_paths().len(), 2, "fixture morto");

        // Apaga a entidade de `b` (o que um Delete faz), deixando a de `a` viva.
        let dead = Entity::from_bits(*map.get(&b).expect("b tem entidade"));
        let _ = sim.world_mut().despawn(dead);
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);

        assert_eq!(
            pen.selected_paths(),
            &[a],
            "o pen ficou com o id da forma apagada — 'algum bit morreu' foi lido como respawn"
        );
    }

    fn setup() -> (SimWorld, VecScene, VecEntityMap) {
        (SimWorld::default(), VecScene::new(), VecEntityMap::new())
    }

    /// REGRESSÃO (Enio 2026-07-09: "ao selecionar sprites elas são desselecionadas
    /// automaticamente"). A seleção do gizmo é COMPARTILHADA. Um sprite selecionado
    /// pela Hierarquia não é uma mudança da árvore *vetorial*, e a sincronia não
    /// pode reescrever o gizmo por causa dele — nem uma vez, nem no frame seguinte.
    #[test]
    fn a_selected_sprite_survives_every_frame_of_the_vector_sync() {
        let (mut sim, mut scene, mut map) = setup();
        let p = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let sprite = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("Spr")))
            .id();

        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();

        // A Hierarquia seleciona o SPRITE. Com a ferramenta de mover ativa, e
        // depois com a vetorial: em nenhum caso o sprite pode cair.
        gizmo.replace_selection(Some(sprite.to_bits()));
        for active in [false, false, true, true] {
            sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, active);
            assert_eq!(
                gizmo.iter_selected().collect::<Vec<_>>(),
                vec![sprite.to_bits()],
                "o sprite continua selecionado (active={active})"
            );
        }
        assert!(pen.selected_paths().is_empty(), "e o pen não adotou nada");
        assert!(!scene.paths().is_empty() && map.contains_key(&p));
    }

    /// REGRESSÃO (Enio 2026-07-09: "os handles não aparecem mais"). Publicada a
    /// seleção do canvas, os frames seguintes NÃO podem readotá-la da árvore — isso
    /// chamava `select_many`, que zera o vértice primário e apaga os handles. Vale
    /// inclusive quando o gizmo perde os nossos bits (era o que o podador de
    /// `snapshots.rs` fazia todo frame, por não haver `Sprite`).
    #[test]
    fn the_pen_selection_survives_the_frames_after_it_is_published() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();

        // O canvas selecionou o path e as âncoras dele.
        pen.box_select(&scene, [-0.5, -0.5], [1.5, 1.5]);
        let verts = pen.selected_verts().to_vec();
        assert!(!verts.is_empty(), "o box-select pegou âncoras");
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);
        assert_eq!(gizmo.selection, Some(map[&a]), "publicou no gizmo");

        // Frames seguintes, sem input nenhum: nada muda.
        for _ in 0..3 {
            sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);
        }
        assert_eq!(pen.selected(), Some(a), "o primário sobreviveu");
        assert_eq!(
            pen.selected_verts(),
            verts,
            "e os vértices — logo, os handles"
        );

        // E o pior caso real: o gizmo PERDE os nossos bits (era o podador de
        // `snapshots.rs`). Isso é a árvore desselecionando — o pen acompanha —,
        // mas jamais deve acontecer sozinho, e é o que o teste de lá trava.
        gizmo.clear_all_selection();
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);
        assert_eq!(
            pen.selected(),
            None,
            "desselecionar na árvore chega no canvas"
        );
    }

    /// **Seleção-só-o-container (ADR-0129 Fatia 3):** clicar um FILHO de um envelope no canvas põe
    /// SÓ o container no gizmo — nunca o filho, nunca os dois. Somar o filho o cisalharia no drag (a
    /// geometria dele é MUNDO e o `Transform` é identidade, então o gizmo o moveria uma vez pelo
    /// próprio `Transform` e outra pela do container). Mutação: sem o ramo, cai no grupo comum e o
    /// gizmo fica com `{filho, container}` (2 bits) — o gate exige 1.
    #[test]
    fn selecting_an_envelope_child_selects_only_the_container() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
        sync(&mut sim, &mut scene, &mut map);
        let container = crate::envelope_live::create(&mut sim, &mut scene, &map, &[a]).unwrap();

        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();

        // O canvas seleciona o FILHO.
        pen.select_many(&[a]);
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);

        assert_eq!(
            gizmo.iter_selected().collect::<Vec<_>>(),
            vec![container],
            "o gizmo tem SÓ o container — nem o filho, nem os dois"
        );
        assert_eq!(
            gizmo.selection,
            Some(container),
            "e o container é o primário"
        );
    }

    /// A árvore manda quando é o subconjunto VETORIAL do gizmo que muda: clicar a
    /// linha de um grupo seleciona as folhas dele no canvas.
    #[test]
    fn a_click_on_a_group_row_selects_its_vector_leaves_on_the_canvas() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let g = group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap();

        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();

        gizmo.replace_selection(Some(g));
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, false);
        assert_eq!(
            pen.selected_paths(),
            [a, b],
            "adotou as folhas, em ordem de z"
        );
        assert_eq!(gizmo.selection, Some(g), "e não reescreveu o gizmo");
    }
}
