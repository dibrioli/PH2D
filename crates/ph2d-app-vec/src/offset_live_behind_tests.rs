//! ⭐ F48 (dono 02/11, «faça tudo como no inspector»): o `Result: New shape behind` e o `Miter limit`
//! do Offset Path vivo.

use super::*;
use ph2d_vec_entities::entities::VecEntityMap;

/// Uma estrela posada em `(4, 0)` — quinas agudas, onde o limite do Miter tem o que cortar.
fn star_scene() -> (VecScene, SimWorld, VecEntityMap, VecXforms, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::star([0.0, 0.0], 1.0, 1.0, 5, 0.4));
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(4.0, 0.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let xf = ph2d_vec_entities::transform::build(&sim, &map);
    (scene, sim, map, xf, id)
}

fn spec(d: f64, result: u8) -> VecOffset {
    VecOffset {
        result,
        ..VecOffset::new(d, 0, 2)
    }
}

fn world_of(scene: &VecScene, xf: &VecXforms, id: VecPathId) -> VecPath {
    let mut w = scene.path(id).expect("a fonte").cooked().into_owned();
    bake_xform(&mut w, &xform_of(xf, id));
    w
}

/// ⭐ **Atrás: o offset e a FONTE por cima, no z dela.** *Mutação: tirar o `out.push(world)` do
/// `recook` ⇒ vermelho.*
#[test]
fn behind_draws_the_source_on_top_of_the_offset() {
    let (scene, mut sim, map, xf, id) = star_scene();
    let mut live = OffsetLive::default();
    arm(&mut sim, &map, &[id], spec(0.2, ph2d_ecs::RESULT_REPLACE));
    live.recook(&scene, &sim, &map, &xf);
    let replace = live.live()[&id].clone();
    arm(&mut sim, &map, &[id], spec(0.2, ph2d_ecs::RESULT_BEHIND));
    live.recook(&scene, &sim, &map, &xf);
    let behind = &live.live()[&id];
    assert_eq!(behind.len(), replace.len() + 1);
    assert_eq!(&behind[..replace.len()], &replace[..], "o offset é o mesmo");
    assert_eq!(
        behind.last(),
        Some(&world_of(&scene, &xf, id)),
        "a fonte por cima"
    );
}

/// ⭐ **Aplicar atrás: a fonte FICA (o mesmo id), o offset entra logo atrás dela, e ela perde o
/// offset vivo.** *Mutação: inserir em `z + 1`, ou não desarmar a fonte ⇒ vermelho.*
#[test]
fn applying_behind_keeps_the_source_and_puts_the_offset_under_it() {
    let (mut scene, mut sim, map, xf, id) = star_scene();
    arm(&mut sim, &map, &[id], spec(0.2, ph2d_ecs::RESULT_BEHIND));
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    let mut restack = Vec::new();
    assert!(materialise(
        &mut scene,
        &mut sim,
        &mut pen,
        &map,
        &xf,
        &[id],
        &mut restack
    ));
    let ids: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    assert_eq!(
        restack,
        vec![vec![ids[0], id]],
        "a Hierarchy põe a nova atrás da fonte"
    );
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[1], id, "a fonte por cima, com o id dela");
    assert!(
        spec_of(&sim, &map, id).is_none(),
        "a fonte ainda tem o offset vivo"
    );
    assert_eq!(
        pen.selected_paths(),
        &[ids[0]],
        "a forma nova fica selecionada"
    );
}

/// ⭐ **À frente (dono 02/11): a fonte por BAIXO do offset, e o Apply põe a forma nova logo à frente
/// dela.** *Mutação: o `insert(0, …)` virar `push`, ou inserir em `z` ⇒ vermelho.*
#[test]
fn in_front_draws_the_offset_over_the_source_and_applies_above_it() {
    let (mut scene, mut sim, map, xf, id) = star_scene();
    let mut live = OffsetLive::default();
    arm(&mut sim, &map, &[id], spec(0.2, ph2d_ecs::RESULT_FRONT));
    live.recook(&scene, &sim, &map, &xf);
    assert_eq!(
        live.live()[&id].first(),
        Some(&world_of(&scene, &xf, id)),
        "a fonte por baixo"
    );
    let mut pen = ph2d_vec_edit::PenTool::default();
    let mut restack = Vec::new();
    assert!(materialise(
        &mut scene,
        &mut sim,
        &mut pen,
        &map,
        &xf,
        &[id],
        &mut restack
    ));
    let ids: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    assert_eq!(ids[0], id, "a fonte por baixo, com o id dela");
    assert_eq!(
        restack,
        vec![vec![id, ids[1]]],
        "a Hierarchy põe a nova à frente da fonte"
    );
}

/// ⭐ **O `Miter limit` chega ao motor**: com o limite 1 toda quina Miter vira Bevel, e o desenho é
/// o do Bevel. *Mutação: o `recook` chamar o `offset_path` sem o limite ⇒ vermelho.*
#[test]
fn the_miter_limit_reaches_the_drawing() {
    let (scene, mut sim, map, xf, id) = star_scene();
    let mut live = OffsetLive::default();
    let mut desenho = |o: VecOffset| {
        arm(&mut sim, &map, &[id], o);
        live.recook(&scene, &sim, &map, &xf);
        live.live()[&id].clone()
    };
    let miter = desenho(VecOffset::new(0.2, 0, 2));
    let cortado = desenho(VecOffset {
        miter: 1.0,
        ..VecOffset::new(0.2, 0, 2)
    });
    let bevel = desenho(VecOffset::new(0.2, 2, 2));
    assert_ne!(cortado, miter, "o limite não mudou nada");
    assert_eq!(cortado, bevel, "limite 1 = Bevel");
}

/// O limite de nascença do componente é o do traço desenhado — e o do painel.
#[test]
fn the_birth_miter_is_the_drawn_strokes() {
    assert_eq!(ph2d_ecs::MITER_AT_BIRTH, ph2d_vec_scene::MITER_LIMIT);
    assert_eq!(
        ph2d_panel_vector::EXPAND_KNOBS_AT_BIRTH.miter,
        ph2d_ecs::MITER_AT_BIRTH
    );
}
