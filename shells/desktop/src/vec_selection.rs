//! O casamento de seleção entre o canvas vetorial e a Hierarquia (ADR-0110).
//!
//! A seleção do gizmo é **compartilhada**: sprites e paths vetoriais convivem no
//! mesmo conjunto de `Entity`. Este módulo é dono apenas do **subconjunto
//! vetorial** dela — nunca reescreve o que é de outro tipo. Foi exatamente essa
//! confusão que apagava a seleção dos sprites sozinha (Enio, 2026-07-09).
//!
//! A ponte de identidade (path ⟺ entidade) é o módulo irmão [`crate::vec_entities`].

use crate::vec_entities::{VecEntityMap, subtree_paths};
use ph2d_ecs::{ChildOf, Entity, SimWorld, VecBlend, VecPathRef};
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

/// Os bits que o GIZMO deve ter para uma seleção de pen: cada path vira sua entidade, MAS um **blend
/// spine** vira as ENTIDADES DAS FONTES dele — o gizmo move as formas, não a linha (ADR-0122).
/// Devolve `(bits, expandiu)`; `expandiu` = havia ao menos um blend (o gizmo difere da seleção
/// literal, e por isso a seleção precisa ser redirecionada nos dois sentidos da sincronia).
fn gizmo_bits_for(
    w: &ph2d_ecs::World,
    map: &VecEntityMap,
    paths: &[VecPathId],
) -> (Vec<u64>, bool) {
    let mut bits = Vec::new();
    let mut expanded = false;
    for id in paths {
        let Some(&b) = map.get(id) else { continue };
        match w.get::<VecBlend>(Entity::from_bits(b)) {
            Some(blend) => {
                expanded = true;
                bits.extend(blend.sources.iter().filter_map(|s| map.get(s).copied()));
            }
            None => bits.push(b),
        }
    }
    (bits, expanded)
}

/// Publica `bits` (+ os ancestrais cheios) no gizmo: o último vira o primário, o resto entra.
fn set_gizmo(gizmo: &mut GizmoStateGroup, bits: &[u64]) {
    let primary = bits.last().copied();
    gizmo.replace_selection(primary);
    for b in bits {
        if Some(*b) != primary {
            gizmo.add_to_selection(*b);
        }
    }
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

    // 1. O canvas mandou.
    let pen_now = pen.selected_paths().to_vec();
    if vector_active && pen_now != state.paths {
        // Um **blend spine** no gizmo vira as FONTES dele (ADR-0122): arrastar a linha move as formas
        // juntas, "como filhas". As fontes têm geometria FIXA (só o `Transform` delas se move); um
        // gizmo sobre o spine dobraria (a bbox dele segue as fontes, e o gizmo somaria a isso). O PEN
        // mantém o spine (o painel e o modo Node dependem dele) — só o gizmo é redirecionado.
        let (mut bits, _) = gizmo_bits_for(w, map, &pen_now);
        for ancestor in fully_selected_ancestors(sim, scene, map, &bits) {
            if !bits.contains(&ancestor) {
                bits.push(ancestor);
            }
        }
        set_gizmo(gizmo, &bits);
        bits.sort_unstable();
        bits.dedup();
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
    state.paths = pen.selected_paths().to_vec();
    // **Redireciona o gizmo para as FONTES** se o clique/árvore trouxe um blend spine (o caminho do
    // modo Select: o clique mexe no gizmo, e a adoção acima só mudou o PEN). Sem blend, o gizmo fica
    // como está — `state.bits = mine` preserva o comportamento de grupo (o teste das folhas).
    let (mut want, expanded) = gizmo_bits_for(w, map, &ordered);
    if expanded {
        for ancestor in fully_selected_ancestors(sim, scene, map, &want) {
            if !want.contains(&ancestor) {
                want.push(ancestor);
            }
        }
        set_gizmo(gizmo, &want);
        want.sort_unstable();
        want.dedup();
        state.bits = want;
    } else {
        state.bits = mine;
    }
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

    /// **O gizmo de um blend spine mira as FONTES** (ADR-0122): selecionar a linha do blend e
    /// arrastá-la move as formas juntas, "como filhas". Tem de ser as fontes — um gizmo sobre o
    /// spine faria DRIFT (a bbox dele segue as fontes que se movem, e o gizmo somaria a isso). O pen
    /// mantém o spine (o painel de Blend e o modo Node dependem dele).
    #[test]
    fn the_gizmo_of_a_blend_spine_targets_its_sources() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([4.0, 0.0], [5.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        // O spine (geometria irrelevante ao teste) + o componente VecBlend sobre as duas formas.
        let spine = scene.push_path(rectangle([0.0, 0.0], [5.0, 0.1]));
        sync(&mut sim, &mut scene, &mut map);
        let se = Entity::from_bits(map[&spine]);
        sim.world_mut()
            .get_entity_mut(se)
            .expect("spine entity")
            .insert(VecBlend::new(vec![a, b], 3));

        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();

        pen.select_many(&[spine]);
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);

        // O gizmo mira as FONTES (a, b), não o spine.
        let mut got: Vec<u64> = gizmo.iter_selected().collect();
        got.sort_unstable();
        let mut want = vec![map[&a], map[&b]];
        want.sort_unstable();
        assert_eq!(got, want, "o gizmo mira as fontes do blend");
        assert!(
            !got.contains(&map[&spine]),
            "e NÃO o spine (ele faria drift)"
        );
        // O pen mantém o spine — o painel e o modo Node dependem dele.
        assert_eq!(pen.selected_paths(), [spine], "o pen mantém o spine");

        // Frames seguintes sem input: a seleção fica estável (nem o pen nem o gizmo se reescrevem).
        for _ in 0..3 {
            sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);
        }
        assert_eq!(pen.selected_paths(), [spine], "o pen segue no spine");
        let mut got2: Vec<u64> = gizmo.iter_selected().collect();
        got2.sort_unstable();
        assert_eq!(got2, want, "o gizmo segue nas fontes");
    }

    /// **O caminho REAL do modo Select: o clique mexe no GIZMO primeiro** (caso 2, não o caso 1).
    /// O canvas seleciona o spine no gizmo; a sincronia adota o spine no PEN E redireciona o gizmo
    /// para as FONTES. Sem tratar o caso 2, o clique deixava o gizmo na linha (o bug do smoke: "não
    /// engloba, não move as shapes").
    #[test]
    fn clicking_a_blend_spine_retargets_the_gizmo_to_the_sources() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([4.0, 0.0], [5.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let spine = scene.push_path(rectangle([0.0, 0.0], [5.0, 0.1]));
        sync(&mut sim, &mut scene, &mut map);
        sim.world_mut()
            .get_entity_mut(Entity::from_bits(map[&spine]))
            .expect("spine entity")
            .insert(VecBlend::new(vec![a, b], 3));

        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();

        // O canvas (Select) publica o SPINE direto no gizmo — o pen ainda está vazio.
        gizmo.replace_selection(Some(map[&spine]));
        sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);

        // A adoção pôs o spine no pen E redirecionou o gizmo para as fontes.
        assert_eq!(pen.selected_paths(), [spine], "o pen adotou o spine");
        let mut got: Vec<u64> = gizmo.iter_selected().collect();
        got.sort_unstable();
        let mut want = vec![map[&a], map[&b]];
        want.sort_unstable();
        assert_eq!(got, want, "o gizmo foi redirecionado para as fontes");

        // Estável: os frames seguintes não readotam as fontes no pen (senão o spine se perderia).
        for _ in 0..3 {
            sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut state, true);
        }
        assert_eq!(pen.selected_paths(), [spine], "o pen segue no spine");
        let mut got2: Vec<u64> = gizmo.iter_selected().collect();
        got2.sort_unstable();
        assert_eq!(got2, want, "o gizmo segue nas fontes");
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
