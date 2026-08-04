//! Os gates do SEGUIMENTO — *"se o canto do mestre está imóvel ao redimensionar, o canto da
//! instância se comporta igual"* (report do Enio).
//!
//! ⚠️ **O oráculo é sempre a geometria DESENHADA**, nunca o documento da cópia: o suporte guardado
//! não é tocado por esta wave, então um gate que o lesse ficaria verde com o seguimento inteiro
//! desligado. É a mesma disciplina do `instance_live_tests`, e a razão pela qual ela existe lá.
//!
//! ⚠️ **E a fixture tem de conter o GESTO.** Uma alça de escala não multiplica a escala: ela
//! multiplica a escala **e compensa a translação** para segurar a âncora. Uma fixture que só
//! multiplicasse a escala não reproduz nada — o `ΔTm` seria zero e todo gate desta lista passaria
//! com o seguimento removido.

use super::*;
use ph2d_ecs::{Transform, VecComponentMain};
use ph2d_vec_scene::{VecPathId, rectangle, xform_of};

/// Um mestre de 20×10 (com um filho, para a sub-árvore entrar no desenho) e `n` cópias.
///
/// ⚠️ **A geometria local do mestre é CENTRADA no pivô, e isto é a fixture a conter o fenómeno.**
/// Com o retângulo a começar em `[0, 0]` o pivô É a quina mínima, logo escalar por ela não
/// compensa translação nenhuma — e o gate da quina mínima ficaria VERDE com a wave inteira
/// desligada. Medido: foi exactamente assim que a M1 sobreviveu à primeira versão desta fixture.
/// Centrada é também o que o produto produz, porque o `settle_origins` põe o pivô no centro.
fn rig(n: usize) -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let main_id = scene.push_path(rectangle([-10.0, -5.0], [10.0, 5.0]));
    let piece_id = scene.push_path(rectangle([-8.0, -3.0], [-2.0, 3.0]));
    let insts: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([-10.0, -5.0], [10.0, 5.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let main_e = Entity::from_bits(map[&main_id]);
    // O mestre fora da origem: uma translação nula esconderia a compensação que o gesto produz.
    let mut mt = Transform::default();
    mt.translation.x = 40.0;
    mt.translation.y = 7.0;
    sim.world_mut()
        .entity_mut(main_e)
        .insert((mt, VecComponentMain));
    let piece_e = Entity::from_bits(map[&piece_id]);
    sim.world_mut()
        .entity_mut(piece_e)
        .insert(ph2d_ecs::ChildOf(main_e));
    for (k, id) in insts.iter().enumerate() {
        let e = Entity::from_bits(map[id]);
        let mut t = Transform::default();
        t.translation.x = 100.0 * (k as f32 + 1.0);
        sim.world_mut()
            .entity_mut(e)
            .insert((t, VecInstance::new(main_id)));
    }
    (sim, scene, map, main_id, insts)
}

/// A caixa de MUNDO que uma cópia desenha.
fn drawn(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    id: VecPathId,
) -> ([f64; 2], [f64; 2]) {
    let xf = crate::vec_transform::build(sim, map);
    let mut il = crate::instance_live::InstanceLive::default();
    il.recook(scene, sim, map, &xf);
    let items = il.live().get(&id).expect("a cópia desenha").clone();
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for it in &items {
        for v in it.verts_all() {
            for a in 0..2 {
                lo[a] = lo[a].min(v.anchor[a]);
                hi[a] = hi[a].max(v.anchor[a]);
            }
        }
    }
    (lo, hi)
}

/// As quinas de MUNDO da caixa do mestre. Sem rotação e com escala positiva `apply` preserva a
/// ordem das quinas — a fixture é escolhida assim de propósito.
fn master_box(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    main_id: VecPathId,
) -> ([f64; 2], [f64; 2]) {
    let xf = crate::vec_transform::build(sim, map);
    let (lo, hi) = scene.path_curve_bbox(main_id).expect("bbox do mestre");
    let x = xform_of(&xf, main_id);
    (x.apply(lo), x.apply(hi))
}

/// **O GESTO**: escala a pose do mestre por `f` segurando o ponto de MUNDO `anchor`.
///
/// É o que `compute_gizmo_transform` produz para uma alça — `T' = A − (A − T)·f` — e é a metade
/// que uma fixture ingênua deixa de fora.
fn scale_master_about(sim: &mut SimWorld, main_e: Entity, f: [f32; 2], anchor: [f64; 2]) {
    let mut t = sim
        .world()
        .get::<Transform>(main_e)
        .copied()
        .unwrap_or_default();
    let a = [anchor[0] as f32, anchor[1] as f32];
    t.translation.x = a[0] - (a[0] - t.translation.x) * f[0];
    t.translation.y = a[1] - (a[1] - t.translation.y) * f[1];
    t.scale.x *= f[0];
    t.scale.y *= f[1];
    sim.world_mut().entity_mut(main_e).insert(t);
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
}

/// **A quina que o mestre segura, a cópia segura.**
///
/// Red-first: sem o [`apply`] a quina da cópia anda — é o relato do Enio, medido.
#[test]
fn the_corner_of_the_copy_holds_still_when_the_masters_does() {
    let (mut sim, scene, map, main_id, insts) = rig(2);
    let main_e = Entity::from_bits(map[&main_id]);
    let (a, _) = master_box(&sim, &scene, &map, main_id);
    let before: Vec<_> = insts
        .iter()
        .map(|i| drawn(&sim, &scene, &map, *i))
        .collect();
    let follow =
        begin(&sim, &scene, &map, &[main_e.to_bits()], main_e.to_bits()).expect("há cópias");
    scale_master_about(&mut sim, main_e, [2.0, 2.0], a);
    apply(&mut sim, &follow);
    for (k, id) in insts.iter().enumerate() {
        let after = drawn(&sim, &scene, &map, *id);
        assert!(
            approx(after.0[0], before[k].0[0]) && approx(after.0[1], before[k].0[1]),
            "a quina mínima da cópia {k} andou: {:?} -> {:?}",
            before[k].0,
            after.0
        );
        assert!(
            after.1[0] - after.0[0] > (before[k].1[0] - before[k].0[0]) * 1.5,
            "a cópia {k} nem cresceu — a fixture não contém o fenómeno"
        );
    }
}

/// **A OUTRA quina também** — e é este gate que prova que passámos o limite herdado do Figma.
///
/// ⚠️ Uma quina FIXA do documento (a mínima, que era a tentativa anterior) só pode casar com uma
/// das quatro alças; arrastando a de baixo-esquerda o crescimento saía espelhado. Aqui a âncora é
/// a quina MÁXIMA, e é ela que tem de ficar parada.
#[test]
fn the_other_corner_holds_when_the_master_holds_the_other_corner() {
    let (mut sim, scene, map, main_id, insts) = rig(1);
    let main_e = Entity::from_bits(map[&main_id]);
    let (_, a) = master_box(&sim, &scene, &map, main_id);
    let before = drawn(&sim, &scene, &map, insts[0]);
    let follow =
        begin(&sim, &scene, &map, &[main_e.to_bits()], main_e.to_bits()).expect("há cópias");
    scale_master_about(&mut sim, main_e, [2.0, 2.0], a);
    apply(&mut sim, &follow);
    let after = drawn(&sim, &scene, &map, insts[0]);
    assert!(
        approx(after.1[0], before.1[0]) && approx(after.1[1], before.1[1]),
        "a quina máxima da cópia andou: {:?} -> {:?}",
        before.1,
        after.1
    );
}

/// **Âncora no CENTRO (Ctrl): nada anda.**
///
/// O gizmo não compensa translação nenhuma nesse modo ⇒ `ΔTm = 0` ⇒ o seguimento é um no-op e a
/// cópia cresce simétrica, como o mestre. ⚠️ É o CONTROLE da wave: sem ele um seguimento que
/// deslocasse SEMPRE passaria nos dois gates acima.
#[test]
fn a_centre_anchored_resize_leaves_every_copy_where_it_is() {
    let (mut sim, scene, map, main_id, insts) = rig(1);
    let main_e = Entity::from_bits(map[&main_id]);
    let inst_e = Entity::from_bits(map[&insts[0]]);
    let pivot = {
        let t = sim.world().get::<Transform>(main_e).copied().unwrap();
        [f64::from(t.translation.x), f64::from(t.translation.y)]
    };
    let before_t = sim.world().get::<Transform>(inst_e).copied().unwrap();
    let before = drawn(&sim, &scene, &map, insts[0]);
    let follow =
        begin(&sim, &scene, &map, &[main_e.to_bits()], main_e.to_bits()).expect("há cópias");
    scale_master_about(&mut sim, main_e, [2.0, 2.0], pivot);
    apply(&mut sim, &follow);
    let after_t = sim.world().get::<Transform>(inst_e).copied().unwrap();
    assert!(
        approx(
            f64::from(after_t.translation.x),
            f64::from(before_t.translation.x)
        ) && approx(
            f64::from(after_t.translation.y),
            f64::from(before_t.translation.y)
        ),
        "a cópia andou numa escala ancorada no centro: {:?} -> {:?}",
        before_t.translation,
        after_t.translation
    );
    let after = drawn(&sim, &scene, &map, insts[0]);
    assert!(
        after.1[0] - after.0[0] > (before.1[0] - before.0[0]) * 1.5,
        "a cópia nem cresceu — a fixture não contém o fenómeno"
    );
}

/// **A cópia anda pela PRÓPRIA parte linear** — uma cópia escalada 2× tem o conteúdo 2× maior,
/// então a quina dela está 2× mais longe e o passo tem de ser 2× também.
///
/// Mutação: usar o delta cru (sem `apply_vec`) deixa a quina desta cópia a andar.
#[test]
fn the_copy_follows_through_its_own_linear_part() {
    let (mut sim, scene, map, main_id, insts) = rig(1);
    let main_e = Entity::from_bits(map[&main_id]);
    let inst_e = Entity::from_bits(map[&insts[0]]);
    let mut it = sim.world().get::<Transform>(inst_e).copied().unwrap();
    it.scale.x = 2.0;
    it.scale.y = 2.0;
    sim.world_mut().entity_mut(inst_e).insert(it);
    let (a, _) = master_box(&sim, &scene, &map, main_id);
    let before = drawn(&sim, &scene, &map, insts[0]);
    let follow =
        begin(&sim, &scene, &map, &[main_e.to_bits()], main_e.to_bits()).expect("há cópias");
    scale_master_about(&mut sim, main_e, [2.0, 2.0], a);
    apply(&mut sim, &follow);
    let after = drawn(&sim, &scene, &map, insts[0]);
    assert!(
        approx(after.0[0], before.0[0]) && approx(after.0[1], before.0[1]),
        "a quina da cópia escalada andou: {:?} -> {:?}",
        before.0,
        after.0
    );
}

/// **Uma cópia que está ELA PRÓPRIA no arrasto fica de fora** — o artista está a movê-la com a
/// mão, e somar-lhe o seguimento daria dois autores para a mesma translação.
#[test]
fn a_copy_that_is_itself_dragged_is_left_alone() {
    let (sim, scene, map, main_id, insts) = rig(2);
    let main_bits = map[&main_id];
    let inst0 = map[&insts[0]];
    let follow = begin(&sim, &scene, &map, &[main_bits, inst0], main_bits).expect("sobra uma");
    assert_eq!(follow.len(), 1, "a cópia arrastada entrou no instantâneo");
}

/// **Cada cópia segue o SEU mestre** — arrastar um não pode mover as cópias do outro.
#[test]
fn each_copy_follows_its_own_master() {
    let (mut sim, mut scene, mut map, main_id, insts) = rig(1);
    let other_id = scene.push_path(rectangle([-2.5, -2.5], [2.5, 2.5]));
    let stray_id = scene.push_path(rectangle([-2.5, -2.5], [2.5, 2.5]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let other_e = Entity::from_bits(map[&other_id]);
    sim.world_mut().entity_mut(other_e).insert(VecComponentMain);
    let stray_e = Entity::from_bits(map[&stray_id]);
    sim.world_mut()
        .entity_mut(stray_e)
        .insert(VecInstance::new(other_id));
    let main_bits = map[&main_id];
    let follow = begin(&sim, &scene, &map, &[main_bits], main_bits).expect("há cópias");
    assert_eq!(
        follow.len(),
        insts.len(),
        "a cópia do outro mestre entrou no instantâneo"
    );
}

/// **Sem cópias não há instantâneo** — e o caminho comum não paga nem uma escrita.
#[test]
fn nothing_to_follow_costs_nothing() {
    let (sim, scene, map, main_id, _) = rig(0);
    let main_bits = map[&main_id];
    assert!(begin(&sim, &scene, &map, &[main_bits], main_bits).is_none());
}

/// **Aplicar duas vezes aterra no mesmo sítio.**
///
/// O alvo é ABSOLUTO contra o instantâneo do pen-down; um seguimento incremental multiplicaria o
/// deslocamento pela taxa de amostragem do rato — a lei que o `vec_frame_resize` já enuncia.
#[test]
fn applying_twice_lands_in_the_same_place() {
    let (mut sim, scene, map, main_id, insts) = rig(1);
    let main_e = Entity::from_bits(map[&main_id]);
    let inst_e = Entity::from_bits(map[&insts[0]]);
    let (a, _) = master_box(&sim, &scene, &map, main_id);
    let follow =
        begin(&sim, &scene, &map, &[main_e.to_bits()], main_e.to_bits()).expect("há cópias");
    scale_master_about(&mut sim, main_e, [2.0, 2.0], a);
    apply(&mut sim, &follow);
    let once = sim.world().get::<Transform>(inst_e).copied().unwrap();
    apply(&mut sim, &follow);
    apply(&mut sim, &follow);
    let thrice = sim.world().get::<Transform>(inst_e).copied().unwrap();
    assert!(
        approx(
            f64::from(once.translation.x),
            f64::from(thrice.translation.x)
        ) && approx(
            f64::from(once.translation.y),
            f64::from(thrice.translation.y)
        ),
        "o seguimento acumula: {:?} -> {:?}",
        once.translation,
        thrice.translation
    );
}
