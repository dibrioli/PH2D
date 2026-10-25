//! Gates do **OBJECTO PINTADO** (F29): a tinta do balde é filha do caminho que a cerca, desenha
//! atrás dele, re-coze só contra as paredes do objecto dela, e sobrevive à solda.

use super::*;
use ph2d_ecs::{ChildOf, FillAnchor, ShowBehindParent};
use ph2d_vec_entities::entities::{self, VecEntityMap};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: ph2d_vec_scene::VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn setup() -> (SimWorld, VecScene, VecEntityMap) {
    (SimWorld::default(), VecScene::new(), VecEntityMap::new())
}

fn caixa(x0: f64, y0: f64, x1: f64, y1: f64) -> VecPath {
    VecPath {
        verts: vec![v(x0, y0), v(x1, y0), v(x1, y1), v(x0, y1)],
        closed: true,
        ..VecPath::default()
    }
}

/// A tinta a nascer como o `apply_bucket` a deixa: o caminho no documento, a receita pendente.
fn tinta(scene: &mut VecScene, parede: u64) -> (u64, Vec<(u64, [f32; 2], Vec<FillAnchor>)>) {
    let id = scene.push_path(VecPath {
        fill: Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
            200, 30, 30, 255,
        ))),
        ..caixa(0.0, 0.0, 10.0, 10.0)
    });
    let ancora = FillAnchor {
        path: parede,
        contorno: 0,
        frac: 0.1,
        frente: true,
    };
    (id, vec![(id, [4.0, 1.0], vec![ancora])])
}

fn pai(sim: &SimWorld, map: &VecEntityMap, id: u64) -> Option<Entity> {
    sim.world()
        .get::<ChildOf>(Entity::from_bits(map[&id]))
        .map(ChildOf::parent)
}

fn pilha(sim: &mut SimWorld) -> Vec<u64> {
    use ph2d_ecs::scene::{HierarchySnapshot, HierarchyWalkState, build_hierarchy_snapshot};
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let (mut scratch, mut snap) = (Vec::new(), HierarchySnapshot::default());
    build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snap);
    entities::z_order(sim.world(), &snap)
}

/// Área (shoelace) de todos os contornos de um caminho de arestas rectas.
fn area(p: &VecPath) -> f64 {
    (0..p.contour_count())
        .filter_map(|c| p.contour(c))
        .map(|(vs, _)| {
            let n = vs.len();
            (0..n)
                .map(|i| {
                    let (a, b) = (vs[i].anchor, vs[(i + 1) % n].anchor);
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f64>()
                .abs()
                * 0.5
        })
        .sum()
}

/// ⭐⭐⭐ **A tinta nasce FILHA do caminho-base e desenha logo ATRÁS dele** — um objecto que estava
/// atrás do caminho continua atrás da tinta (report do dono 25/10: *«o objeto composto não se
/// comporta como um único objeto com um z index fixo»*).
///
/// Mutação que deve sangrar: o `arm_new_fills` voltar a mandar a tinta para o fundo da cena.
#[test]
fn a_tinta_do_balde_nasce_filha_do_caminho_e_atras_dele() {
    let (mut sim, mut scene, mut map) = setup();
    let outro = scene.push_path(caixa(5.0, -5.0, 15.0, 15.0));
    let parede = scene.push_path(caixa(0.0, 0.0, 10.0, 10.0));
    let (t, mut pendentes) = tinta(&mut scene, parede);
    entities::sync(&mut sim, &mut scene, &mut map);
    arm_new_fills(&mut sim, &scene, &map, &mut pendentes);

    assert_eq!(pai(&sim, &map, t), Some(Entity::from_bits(map[&parede])));
    assert!(
        sim.world()
            .get::<ShowBehindParent>(Entity::from_bits(map[&t]))
            .is_some(),
        "a tinta tem de desenhar atrás do traço do pai"
    );
    assert_eq!(
        pilha(&mut sim),
        vec![outro, t, parede],
        "o outro objecto não se pode meter entre a tinta e o caminho dela"
    );
}

/// ⭐⭐⭐ **Outro objecto POR CIMA não corta a tinta** (foto 3 da F28: *«o quadrado corta a tinta
/// amarela»*). A rede da tinta é a do objecto dela; o quadrado alheio não é parede.
///
/// Mutação que deve sangrar: as paredes de um objecto voltarem a ser a cena inteira.
#[test]
fn outro_objecto_sobre_o_composto_nao_corta_a_tinta() {
    let (mut sim, mut scene, mut map) = setup();
    let parede = scene.push_path(caixa(0.0, 0.0, 10.0, 10.0));
    let (t, mut pendentes) = tinta(&mut scene, parede);
    scene.push_path(caixa(5.0, -5.0, 15.0, 15.0));
    entities::sync(&mut sim, &mut scene, &mut map);
    arm_new_fills(&mut sim, &scene, &map, &mut pendentes);

    let (mut cache, mut face) = (None, None);
    upkeep(&sim, &mut scene, &map, false, &mut cache, &mut face);
    let a = area(scene.path(t).expect("a tinta continua lá"));
    assert!(
        (a - 100.0).abs() < 1e-6,
        "a tinta mediu {a:.3} de 100 — o quadrado alheio partiu-a"
    );
}

/// ⭐⭐ **Apagar o caminho leva a tinta filha** pela cascata da árvore (a §7 da F28, agora pela
/// porta da Hierarquia: o `sync` despawna o pai com os filhos).
///
/// Mutação que deve sangrar: a tinta não ser pendurada no caminho.
#[test]
fn apagar_o_pai_leva_a_tinta_filha() {
    let (mut sim, mut scene, mut map) = setup();
    let parede = scene.push_path(caixa(0.0, 0.0, 10.0, 10.0));
    let (t, mut pendentes) = tinta(&mut scene, parede);
    entities::sync(&mut sim, &mut scene, &mut map);
    arm_new_fills(&mut sim, &scene, &map, &mut pendentes);

    scene.remove_path(parede);
    entities::sync(&mut sim, &mut scene, &mut map);
    entities::sync(&mut sim, &mut scene, &mut map);
    assert!(scene.path(t).is_none(), "o caminho saiu e a tinta dele ficou");
}

/// ⭐⭐ **A solda leva as filhas do traço CONSUMIDO para a rede** — sem isto a tinta pendurada
/// num traço que a solda apaga morreria com ele.
///
/// Mutação que deve sangrar: o `soldar` não chamar o `adopt_children`.
#[test]
fn a_solda_leva_as_filhas_do_traco_consumido() {
    let (mut sim, mut scene, mut map) = setup();
    let reta = |scene: &mut VecScene, a: [f64; 2], b: [f64; 2]| {
        scene.push_path(VecPath {
            verts: vec![v(a[0], a[1]), v(b[0], b[1])],
            ..VecPath::default()
        })
    };
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    let filha = scene.push_path(caixa(0.0, 0.0, 1.0, 1.0));
    entities::sync(&mut sim, &mut scene, &mut map);
    let ev = Entity::from_bits(map[&vt]);
    assert!(ph2d_vec_entities::transform::reparent_keeping_world(
        &mut sim,
        Entity::from_bits(map[&filha]),
        ev
    ));

    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[h, vt]);
    crate::weld::soldar(&mut sim, &mut scene, &map, &mut pen, 0.0);
    assert!(scene.path(vt).is_none(), "a solda consome o traço de cima");
    entities::sync(&mut sim, &mut scene, &mut map);
    entities::sync(&mut sim, &mut scene, &mut map);

    assert!(scene.path(filha).is_some(), "a filha morreu com o traço consumido");
    assert_eq!(pai(&sim, &map, filha), Some(Entity::from_bits(map[&h])));
}
