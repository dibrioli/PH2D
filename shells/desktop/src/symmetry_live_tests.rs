//! Gates da **SIMETRIA VIVA** — arquivo irmão de `symmetry_live.rs`.
//!
//! Os dois factos do USUÁRIO que esta família existe para pinar (Enio, 2026-08-01):
//!
//! > *"Uma vez que o desenho é feito, a referência para a linha passa a ser o próprio desenho, ou
//! > seja, se o usuário mover o objeto no canvas a linha de simetria acompanha mantendo a mesma
//! > distância relativa ao objeto."*
//!
//! > *"Se a simetria for desmarcada antes do apply, as cópias somem mas não são destruídas."*
//!
//! ⚠️ **O oráculo NÃO é "há duas formas desenhadas"** — isso é verdade em qualquer implementação
//! que reflicta alguma coisa em algum lugar. Os oráculos daqui são: (a) o desenho inteiro TRANSLADA
//! com a pose, ponto a ponto; (b) a curva autorada fica byte-idêntica através do ciclo
//! armar→desarmar; (c) uma forma que já é simétrica no próprio eixo coincide com a própria cópia
//! sob QUALQUER pose — que é a propriedade que só a ordem *reflectir → assar* tem.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_vec_scene::VecVertex;
use ph2d_vec_scene::symmetry::SymmetryKind;

/// Um **meio-perfil aberto** com as pontas no eixo `x = 0`, posado. A fixture tem de conter o
/// fenômeno: é a forma para a qual a simetria existe (metade desenhada, metade derivada), e é a
/// única em que a fusão tem o que fazer.
fn half_profile_scene() -> (
    VecScene,
    ph2d_ecs::SimWorld,
    VecEntityMap,
    VecXforms,
    VecPathId,
    ph2d_ecs::Entity,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut p = VecPath::default();
    p.verts = [[0.0, -1.0], [0.8, -0.3], [0.5, 0.4], [0.0, 1.0]]
        .map(VecVertex::corner)
        .to_vec();
    p.closed = false;
    let id = scene.push_path(p);
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(3.0, 1.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("Vase"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let xf = crate::vec_transform::build(&sim, &map);
    (scene, sim, map, xf, id, e)
}

/// Um **quadrado centrado em `x = 0`** — ele já é simétrico no eixo que a spec vai usar, e é isso
/// que o torna oráculo da ORDEM (ver o gate da pose).
fn symmetric_square_scene() -> (
    VecScene,
    ph2d_ecs::SimWorld,
    VecEntityMap,
    VecPathId,
    ph2d_ecs::Entity,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Square"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id, e)
}

/// A spec de espelho vertical pelo eixo local `x = 0` — o default de todos os gates daqui.
fn mirror_x() -> SymmetrySpec {
    SymmetrySpec {
        kind: SymmetryKind::MirrorX,
        center: [0.0, 0.0],
        ..SymmetrySpec::default()
    }
}

/// As âncoras da geometria DESENHADA de `id` neste frame, na ordem de emissão.
fn drawn(live: &SymmetryLive, id: VecPathId) -> Vec<[f64; 2]> {
    live.live()
        .get(&id)
        .map(|paths| {
            paths
                .iter()
                .flat_map(|p| p.verts_all().map(|v| v.anchor))
                .collect()
        })
        .unwrap_or_default()
}

/// Todos os vértices AUTORADOS de um caminho, na ordem.
fn authored(scene: &VecScene, id: VecPathId) -> Vec<VecVertex> {
    scene
        .path(id)
        .expect("o caminho existe")
        .verts_all()
        .copied()
        .collect()
}

/// Move a entidade por `delta` e devolve as poses re-construídas.
fn nudge(
    sim: &mut ph2d_ecs::SimWorld,
    map: &VecEntityMap,
    e: ph2d_ecs::Entity,
    delta: [f32; 2],
) -> VecXforms {
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.translation.x += delta[0];
        t.translation.y += delta[1];
    }
    crate::vec_transform::build(sim, map)
}

/// **A promessa central.** Mover o objeto no canvas leva a linha de simetria junto, *mantendo a
/// mesma distância relativa ao objeto* — e a forma executável disso é que o desenho INTEIRO (fonte
/// e cópia) translada pelo mesmo delta, ponto a ponto.
///
/// ⚠️ Este gate é também o do MEMO: o `recook` corre duas vezes sem ninguém limpar nada, então uma
/// chave que ignorasse a pose devolveria o desenho velho e a asserção falha.
#[test]
fn moving_the_shape_carries_the_axis_with_it() {
    let (scene, mut sim, map, xf, id, e) = half_profile_scene();
    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);

    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let before = drawn(&live, id);
    assert!(!before.is_empty(), "a simetria armada desenha alguma coisa");

    let delta = [5.0_f32, -2.0_f32];
    let xf2 = nudge(&mut sim, &map, e, delta);
    live.recook(&scene, &sim, &map, &xf2);
    let after = drawn(&live, id);

    assert_eq!(before.len(), after.len(), "o desenho não muda de tamanho");
    for (i, (a, b)) in before.iter().zip(after.iter()).enumerate() {
        let want = [a[0] + f64::from(delta[0]), a[1] + f64::from(delta[1])];
        assert!(
            (b[0] - want[0]).abs() < 1e-9 && (b[1] - want[1]).abs() < 1e-9,
            "ponto {i}: esperado {want:?}, veio {b:?} — o eixo ficou para trás"
        );
    }
}

/// **Desmarcar antes do Apply não destrói nada.** As cópias somem porque nunca estiveram no
/// documento, e a curva autorada volta byte-idêntica.
#[test]
fn disarming_removes_the_copies_and_leaves_the_authored_curve_untouched() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    let pristine = authored(&scene, id);

    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);
    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        !drawn(&live, id).is_empty(),
        "armada, a simetria põe geometria derivada na tela"
    );

    assert_eq!(arm(&mut sim, &map, &[id], None), 1);
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        drawn(&live, id).is_empty(),
        "desarmada, não sobra derivada nenhuma — o dispatch volta a desenhar a fonte"
    );
    assert_eq!(
        authored(&scene, id),
        pristine,
        "a curva AUTORADA atravessou o ciclo sem ser tocada"
    );
}

/// **A ordem é `reflectir → assar a pose`, e este é o oráculo que a distingue.**
///
/// Um quadrado centrado em `x = 0` **já é** simétrico no eixo da spec, então a reflexão local é a
/// identidade sobre ele: a cópia tem de coincidir com a fonte sob QUALQUER pose. Reflectir depois
/// de assar usaria uma linha que a pose não-conforme torna outra, e as duas deixam de coincidir.
#[test]
fn the_pose_is_baked_after_the_reflection_not_before() {
    let (scene, mut sim, map, id, e) = symmetric_square_scene();
    // Rotação + escala NÃO-uniforme: é a pose em que as duas ordens divergem. Sob escala uniforme
    // (ou sem rotação) elas coincidem, e o gate não podia falhar.
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.rotation = 0.5;
        t.scale = ph2d_core::Vec2::new(2.0, 1.0);
    }
    let xf = crate::vec_transform::build(&sim, &map);
    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);

    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let paths = live.live().get(&id).expect("há derivada");
    assert_eq!(paths.len(), 2, "espelho simples: a fonte e a cópia");

    // A cópia é percorrida ao contrário (a reposição do winding), então o conjunto de âncoras é
    // que tem de coincidir — não a sequência.
    let mut src: Vec<[f64; 2]> = paths[0].verts_all().map(|v| v.anchor).collect();
    let mut cpy: Vec<[f64; 2]> = paths[1].verts_all().map(|v| v.anchor).collect();
    let key = |p: &[f64; 2]| (p[0].to_bits(), p[1].to_bits());
    src.sort_by_key(key);
    cpy.sort_by_key(key);
    assert_eq!(src.len(), cpy.len());
    for (a, b) in src.iter().zip(cpy.iter()) {
        assert!(
            (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
            "a cópia de uma forma já simétrica saiu noutro lugar: {a:?} vs {b:?}"
        );
    }
}

/// **Radial põe `segments` formas na tela**, e a fonte é uma delas — o painel promete a contagem,
/// e é ela que tem de aparecer.
#[test]
fn radial_draws_one_shape_per_segment() {
    let (scene, mut sim, map, id, _e) = symmetric_square_scene();
    let xf = crate::vec_transform::build(&sim, &map);
    let spec = SymmetrySpec {
        kind: SymmetryKind::Radial,
        center: [3.0, 0.0],
        segments: 5,
        ..SymmetrySpec::default()
    };
    assert_eq!(arm(&mut sim, &map, &[id], Some(spec)), 1);

    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf);
    assert_eq!(
        live.live().get(&id).map(Vec::len),
        Some(5),
        "cinco pétalas, contando o original"
    );
}

/// **Re-armar com a MESMA spec não mexe em nada** — é o guard que torna seguro um painel que
/// re-emite o valor a cada frame de arrasto.
#[test]
fn arming_with_the_same_spec_touches_nothing() {
    let (_scene, mut sim, map, _xf, id, _e) = half_profile_scene();
    assert_eq!(arm(&mut sim, &map, &[id], Some(mirror_x())), 1);
    assert_eq!(
        arm(&mut sim, &map, &[id], Some(mirror_x())),
        0,
        "a segunda chamada é um no-op"
    );
}

/// **`forget` limpa memo E derivada** — o load de projeto troca a cena debaixo do cozimento, e os
/// `VecPathId` são reciclados entre documentos.
#[test]
fn forget_drops_the_memo_and_the_drawing() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    arm(&mut sim, &map, &[id], Some(mirror_x()));
    let mut live = SymmetryLive::default();
    live.recook(&scene, &sim, &map, &xf);
    assert!(!drawn(&live, id).is_empty());
    live.forget();
    assert!(drawn(&live, id).is_empty(), "nada sobrevive ao forget");
}
