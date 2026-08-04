//! Gates dos VERBOS de componente (plano UI/UX W5) — a costura painel↔shell↔ECS.
//!
//! Os gates do produtor provam o que se DESENHA; estes provam que o gesto leva a algum lugar — a
//! quarta condição da política de UI (*a SEQUÊNCIA leva a algum lugar*), que não é implicada por
//! nenhuma das outras três.

use super::*;
use ph2d_ecs::{OverrideSlot, VecComponentMain, VecInstance};
use ph2d_vec_scene::rectangle;

/// Uma cena com uma forma, já sincronizada.
fn one_shape() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    (sim, scene, map, id)
}

/// **A sequência COMPLETA leva a algum lugar**: criar → colocar → a cópia desenha o mestre.
///
/// ⚠️ É o gate da wave. Cada metade sozinha (o componente existe · o botão está registado · o
/// clique chega ao barramento) pode estar verde com o gesto a não produzir nada.
#[test]
fn create_then_place_gives_a_copy_that_draws_the_main() {
    let (mut sim, mut scene, mut map, main_id) = one_shape();
    assert!(create_main(&mut sim, &map, &[main_id]), "Create recusou");
    let new_id = place_instance(&sim, &mut scene, &map, &[main_id]).expect("Place recusou");
    // O `sync` é o que dá entidade ao caminho novo — o mesmo do frame do produto.
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(arm_instance(
        &mut sim,
        &map,
        new_id,
        main_id,
        cascade_offset([3.0, -3.0], 0)
    ));
    let xf = crate::vec_transform::build(&sim, &map);
    let mut live = crate::instance_live::InstanceLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let items = live.live().get(&new_id).expect("a cópia desenha o mestre");
    assert_eq!(items.len(), 1, "um mestre de uma peça desenha uma");
}

/// Coloca `n` cópias do mestre pela MESMA sequência do frame do produto, com o degrau `step`.
fn place_n(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    main: VecPathId,
    step: [f32; 2],
    n: usize,
) -> Vec<VecPathId> {
    let mut out = Vec::new();
    for _ in 0..n {
        let Some(id) = place_instance(sim, scene, map, &[main]) else {
            break;
        };
        crate::vec_entities::sync(sim, scene, map);
        let already = instance_count(sim, scene, map, main);
        arm_instance(sim, map, id, main, cascade_offset(step, already));
        out.push(id);
    }
    out
}

/// A pose de um caminho, lida do ECS.
fn pose_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> [f32; 2] {
    let e = Entity::from_bits(map[&id]);
    sim.world()
        .get::<ph2d_ecs::Transform>(e)
        .map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y])
}

/// **Três cliques dão três sítios** — e nenhum deles longe do mestre.
///
/// ⚠️ O gate nasceu VERMELHO contra a v1, e nas DUAS metades: ela tinha um degrau `const`, então
/// as três cópias pousavam **no mesmo ponto** (o relato *"uma em cima da outra"*), e esse ponto
/// era 24 unidades de MUNDO — a ~700 px na cena `=53`, o relato *"nem saberia que existiam"*.
#[test]
fn three_places_cascade_and_none_of_them_wanders_off() {
    let (mut sim, mut scene, mut map, main_id) = one_shape();
    assert!(create_main(&mut sim, &map, &[main_id]));
    let step = [3.0_f32, -3.0];
    let ids = place_n(&mut sim, &mut scene, &mut map, main_id, step, 3);
    assert_eq!(ids.len(), 3, "o Place recusou uma das cópias");

    let base = pose_of(&sim, &map, main_id);
    for (i, id) in ids.iter().enumerate() {
        let p = pose_of(&sim, &map, *id);
        let want = [
            base[0] + step[0] * (i + 1) as f32,
            base[1] + step[1] * (i + 1) as f32,
        ];
        assert!(
            (p[0] - want[0]).abs() < 1e-4 && (p[1] - want[1]).abs() < 1e-4,
            "a cópia {i} nasceu em {p:?}, e o degrau {} manda {want:?}",
            i + 1
        );
    }
    // A metade da PILHA, dita sem a aritmética acima: duas cópias nunca coincidem.
    for a in 0..ids.len() {
        for b in (a + 1)..ids.len() {
            let (pa, pb) = (pose_of(&sim, &map, ids[a]), pose_of(&sim, &map, ids[b]));
            assert!(
                (pa[0] - pb[0]).abs() > 1e-4 || (pa[1] - pb[1]).abs() > 1e-4,
                "as cópias {a} e {b} pousaram no mesmo sítio ({pa:?})"
            );
        }
    }
}

/// **A cascata conta só as cópias DESTE mestre.**
///
/// ⚠️ Sem isto, colocar uma cópia de um botão empurraria a próxima cópia de um ícone — a folga
/// passaria a depender do que mais existe no documento, e o artista não teria como a prever.
#[test]
fn the_cascade_counts_only_copies_of_this_main() {
    let (mut sim, mut scene, mut map, main_id) = one_shape();
    assert!(create_main(&mut sim, &map, &[main_id]));
    // Um segundo mestre, com duas cópias já colocadas.
    let other = scene.push_path(rectangle([40.0, 0.0], [50.0, 6.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(create_main(&mut sim, &map, &[other]));
    place_n(&mut sim, &mut scene, &mut map, other, [1.0, -1.0], 2);

    assert_eq!(
        instance_count(&sim, &scene, &map, main_id),
        0,
        "as cópias do OUTRO mestre entraram na conta deste"
    );
    let step = [3.0_f32, -3.0];
    let base = pose_of(&sim, &map, main_id);
    let ids = place_n(&mut sim, &mut scene, &mut map, main_id, step, 1);
    let p = pose_of(&sim, &map, ids[0]);
    assert!(
        (p[0] - (base[0] + step[0])).abs() < 1e-4,
        "a 1ª cópia deste mestre nasceu no degrau {:.1}, e não no 1º",
        (p[0] - base[0]) / step[0]
    );
}

/// **O degrau é um MÚLTIPLO, e o primeiro nunca é zero** — uma cópia sob o mestre é uma cópia que
/// o artista não vê, e é o outro modo de falha do mesmo botão.
#[test]
fn the_first_copy_is_one_step_away_and_the_nth_is_n_steps() {
    let step = [2.0_f32, -5.0];
    assert_eq!(cascade_offset(step, 0), step);
    assert_eq!(cascade_offset(step, 1), [4.0, -10.0]);
    assert_eq!(cascade_offset(step, 4), [10.0, -25.0]);
}

/// **Uma instância não vira mestre** — o que ela mostra é derivado.
#[test]
fn an_instance_cannot_be_promoted_to_a_main() {
    let (mut sim, _scene, map, id) = one_shape();
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecInstance::new(999));
    assert!(!create_main(&mut sim, &map, &[id]));
}

/// **Place recusa quem não é mestre** — senão o botão faria uma cópia de uma coisa qualquer.
#[test]
fn place_refuses_a_plain_shape() {
    let (sim, mut scene, map, id) = one_shape();
    assert!(place_instance(&sim, &mut scene, &map, &[id]).is_none());
}

/// Um mestre de DUAS peças (a caixa e o rótulo dentro dela) e uma instância deslocada.
///
/// `child_first` põe o rótulo ANTES da caixa na ordem da CENA. ⚠️ Isso não é uma cena artificial:
/// a ordem da cena é a ordem de z, e z é autorável — o artista pode trazer o rótulo para a frente.
/// Foi exactamente aí que o Detach trocou pai e filho.
fn master_child_instance(
    child_first: bool,
) -> (
    SimWorld,
    VecScene,
    VecEntityMap,
    VecPathId,
    VecPathId,
    VecPathId,
) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let (main_id, child_id) = if child_first {
        let c = scene.push_path(rectangle([2.0, 2.0], [8.0, 8.0]));
        let m = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
        (m, c)
    } else {
        let m = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
        let c = scene.push_path(rectangle([2.0, 2.0], [8.0, 8.0]));
        (m, c)
    };
    let inst_id = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let main_e = Entity::from_bits(map[&main_id]);
    sim.world_mut().entity_mut(main_e).insert(VecComponentMain);
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&child_id]))
        .insert(ph2d_ecs::ChildOf(main_e));
    // ⚠️ A instância NÃO está na origem — é a pose dela que a v1 aplicava duas vezes.
    let mut t = ph2d_ecs::Transform::default();
    t.translation.x = 100.0;
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&inst_id]))
        .insert((t, VecInstance::new(main_id)));
    (sim, scene, map, main_id, child_id, inst_id)
}

/// O que o PRODUTOR desenha para `at`, e as peças por trás — o par que o Detach consome.
fn cooked(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    at: VecPathId,
) -> (Vec<ph2d_vec_scene::VecPath>, Vec<VecPathId>) {
    let xf = crate::vec_transform::build(sim, map);
    let mut live = crate::instance_live::InstanceLive::default();
    live.recook(scene, sim, map, &xf);
    (
        live.live().get(&at).expect("a instância desenha").clone(),
        live.pieces_of(at).expect("as peças").to_vec(),
    )
}

/// A caixa de mundo de um caminho do documento (geometria ∘ pose da entidade).
fn world_box(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap, id: VecPathId) -> [f64; 2] {
    let xf = crate::vec_transform::build(sim, map);
    let (lo, _) = scene.path_curve_bbox(id).expect("bbox");
    ph2d_vec_scene::xform_of(&xf, id).apply(lo)
}

/// **A arte destacada fica ONDE ESTAVA.**
///
/// ⚠️ Gate red-first, e o número é do produto: a v1 gravava a geometria de MUNDO num caminho cuja
/// entidade ainda carrega a pose da instância, então a pose entrava **duas vezes** — uma peça
/// desenhada em `x = 100` aterrava em `x = 200`. Um Detach que move a arte é um Detach que o
/// artista tem de desfazer.
#[test]
fn detach_leaves_the_art_where_it_was_drawn() {
    let (mut sim, mut scene, mut map, _main, _child, inst) = master_child_instance(false);
    let (drawn, pieces) = cooked(&sim, &scene, &map, inst);
    let before: Vec<[f64; 2]> = drawn
        .iter()
        .map(|p| {
            let mut lo = [f64::MAX; 2];
            for v in p.verts_all() {
                for (slot, c) in lo.iter_mut().zip(v.anchor) {
                    *slot = slot.min(c);
                }
            }
            lo
        })
        .collect();
    let plan = detach(
        &mut sim,
        &mut scene,
        &map,
        &[inst],
        Some(&drawn),
        Some(&pieces),
    )
    .expect("Detach recusou");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(arm_detached(&mut sim, &map, &plan));
    let after: Vec<[f64; 2]> = std::iter::once(inst)
        .chain(plan.extra.iter().copied())
        .map(|id| world_box(&sim, &scene, &map, id))
        .collect();
    for (b, a) in before.iter().zip(after.iter()) {
        assert!(
            (b[0] - a[0]).abs() < 1e-6 && (b[1] - a[1]).abs() < 1e-6,
            "a arte saltou: desenhada em {b:?}, destacada em {a:?}"
        );
    }
}

/// **A raiz do destacado é a raiz do MESTRE, não a primeira peça desenhada.**
///
/// ⚠️ Red-first com o rótulo À FRENTE da caixa: a v1 tomava `items.first()`, que é ordem de **z**,
/// e o resultado era o relato do Enio — *"quem era pai virou filho, quem era filho virou pai"*.
#[test]
fn detach_roots_the_masters_root_even_when_the_child_is_in_front() {
    for child_first in [false, true] {
        let (mut sim, mut scene, mut map, main, child, inst) = master_child_instance(child_first);
        let (drawn, pieces) = cooked(&sim, &scene, &map, inst);
        assert_eq!(pieces.len(), 2, "o mestre tem duas peças");
        let plan = detach(
            &mut sim,
            &mut scene,
            &map,
            &[inst],
            Some(&drawn),
            Some(&pieces),
        )
        .expect("Detach recusou");
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        assert!(arm_detached(&mut sim, &map, &plan));
        // A caixa do mestre mede 20 de largura, o rótulo 6: é isso que os distingue sem depender
        // de qualquer id.
        let w = |id: VecPathId| {
            let (lo, hi) = scene.path_curve_bbox(id).expect("bbox");
            hi[0] - lo[0]
        };
        assert!(
            (w(inst) - w(main)).abs() < 1e-6,
            "child_first={child_first}: a raiz ficou com a peça errada (largura {}, esperada {})",
            w(inst),
            w(main)
        );
        assert_eq!(plan.extra.len(), 1);
        assert!((w(plan.extra[0]) - w(child)).abs() < 1e-6);
        assert_eq!(
            plan.parents,
            vec![(plan.extra[0], inst)],
            "child_first={child_first}: o parentesco não espelhou a árvore do mestre"
        );
        let ce = Entity::from_bits(map[&plan.extra[0]]);
        assert_eq!(
            sim.world().get::<ph2d_ecs::ChildOf>(ce).map(|c| c.parent()),
            Some(Entity::from_bits(map[&inst])),
            "a peça ficou solta na raiz: o Detach desfez o agrupamento"
        );
    }
}

/// **Reset num instância LIMPA não mexe no mundo** — o `post_frame_undo` regista por diff, e um
/// passo vazio é ruído que o artista paga com um Ctrl+Z que não faz nada.
#[test]
fn reset_on_a_clean_instance_is_a_no_op() {
    let (mut sim, _scene, map, id) = one_shape();
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecInstance::new(7));
    assert!(!reset_overrides(&mut sim, &map, &[id]));
    let mut with = VecInstance::new(7);
    with.set(1, OverrideSlot::Hidden);
    sim.world_mut().entity_mut(e).insert(with);
    assert!(reset_overrides(&mut sim, &map, &[id]));
}

/// **O painel recebe os verbos que fazem sentido, e só eles.**
#[test]
fn the_panel_is_told_which_verbs_make_sense() {
    let (mut sim, _scene, map, id) = one_shape();
    let plain = selected_component(&sim, &map, &[id], &[]).expect("uma forma tem seção");
    assert!(!plain.is_main && !plain.is_instance && !plain.has_overrides);

    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecComponentMain);
    assert!(
        selected_component(&sim, &map, &[id], &[])
            .expect("mestre")
            .is_main
    );

    sim.world_mut().entity_mut(e).remove::<VecComponentMain>();
    let mut inst = VecInstance::new(7);
    inst.set(1, OverrideSlot::Hidden);
    sim.world_mut().entity_mut(e).insert(inst);
    let s = selected_component(&sim, &map, &[id], &[id]).expect("instância");
    assert!(s.is_instance && s.has_overrides && s.main_missing);
}

/// **Sem seleção — ou com duas — a seção não é oferecida.** Um prefab é sobre UMA coisa.
#[test]
fn no_section_without_exactly_one_selected_shape() {
    let (sim, mut scene, mut map, id) = one_shape();
    assert!(selected_component(&sim, &map, &[], &[]).is_none());
    let other = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let mut sim2 = sim;
    crate::vec_entities::sync(&mut sim2, &mut scene, &mut map);
    assert!(selected_component(&sim2, &map, &[id, other], &[]).is_none());
}

/// **A órfã vem do PRODUTOR** — o painel lê a resposta dele, não uma segunda.
///
/// ⚠️ Sem este gate, alguém "simplificaria" o `selected_component` para perguntar *"o mestre
/// existe?"* por conta própria — e o painel diria que está tudo bem no frame em que o produtor
/// recusasse por pose degenerada ou por laço.
#[test]
fn the_missing_readout_comes_from_the_producers_answer() {
    let (mut sim, _scene, map, id) = one_shape();
    let e = Entity::from_bits(map[&id]);
    // Um mestre que EXISTE — mas o produtor, por outra razão, recusou.
    sim.world_mut().entity_mut(e).insert(VecInstance::new(id));
    let s = selected_component(&sim, &map, &[id], &[id]).expect("instância");
    assert!(s.main_missing, "o painel ignorou a recusa do produtor");
}

/// **O suporte cobre o que a cópia DESENHA** — é ele a caixa do gizmo e o alvo do clique.
///
/// ⚠️ Red-first, e falha por dois motivos independentes: a v1 media só o caminho-RAIZ (um rótulo
/// que transborda ficava de fora) e media-o em LOCAL, cega à pose do mestre — com a forma do
/// mestre a propagar, um mestre escalado 2× dá uma cópia do dobro do tamanho dentro de uma caixa
/// do tamanho antigo. Uma caixa de gizmo que não é a arte é uma alça que agarra o vazio.
#[test]
fn the_support_covers_what_the_copy_draws() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let main_id = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
    // ⚠️ O filho TRANSBORDA a caixa do pai: é isso que separa "medi a raiz" de "medi o conteúdo".
    let child_id = scene.push_path(rectangle([-6.0, -3.0], [24.0, 14.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let main_e = Entity::from_bits(map[&main_id]);
    let mut mt = ph2d_ecs::Transform::default();
    mt.translation.x = 40.0;
    mt.scale.x = 2.0;
    mt.scale.y = 2.0;
    sim.world_mut()
        .entity_mut(main_e)
        .insert((mt, VecComponentMain));
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&child_id]))
        .insert(ph2d_ecs::ChildOf(main_e));

    let inst = place_instance(&sim, &mut scene, &map, &[main_id]).expect("Place");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(arm_instance(&mut sim, &map, inst, main_id, [7.0, -5.0]));

    let (drawn, _) = cooked(&sim, &scene, &map, inst);
    let (mut dlo, mut dhi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in &drawn {
        for v in p.verts_all() {
            for ((lo, hi), c) in dlo.iter_mut().zip(dhi.iter_mut()).zip(v.anchor) {
                *lo = lo.min(c);
                *hi = hi.max(c);
            }
        }
    }
    let xf = crate::vec_transform::build(&sim, &map);
    let (slo, shi) = scene.path_curve_bbox(inst).expect("suporte");
    let x = ph2d_vec_scene::xform_of(&xf, inst);
    let (wlo, whi) = (x.apply(slo), x.apply(shi));
    for a in 0..2 {
        assert!(
            (wlo[a] - dlo[a]).abs() < 1e-6 && (whi[a] - dhi[a]).abs() < 1e-6,
            "eixo {a}: o suporte cobre {:?}..{:?} e a arte ocupa {:?}..{:?}",
            wlo[a],
            whi[a],
            dlo[a],
            dhi[a]
        );
    }
}
