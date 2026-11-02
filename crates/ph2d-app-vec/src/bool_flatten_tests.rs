//! Os gates do **Flatten**: o que estava na tela continua na tela, numa forma só.

use super::*;
use crate::bool_gesture::{arm, group_of_selection};
use ph2d_vec_scene::{VecPathId, rectangle};

struct Mundo {
    sim: SimWorld,
    scene: VecScene,
    map: VecEntityMap,
    pen: ph2d_vec_edit::PenTool,
}

impl Mundo {
    fn com(rects: &[([f64; 2], [f64; 2])]) -> (Self, Vec<VecPathId>) {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();
        let ids = rects
            .iter()
            .map(|&(a, b)| scene.push_path(rectangle(a, b)))
            .collect();
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
        let pen = ph2d_vec_edit::PenTool::default();
        (
            Self {
                sim,
                scene,
                map,
                pen,
            },
            ids,
        )
    }

    fn achata(&mut self, sel: &[VecPathId]) -> usize {
        self.pen.select_many(sel);
        let mut live = ph2d_vec_render::LiveGeometry::new();
        let mut bl = BoolLive::default();
        let xf = VecXforms::default();
        bl.recook(&self.scene, &self.sim, &self.map, &xf, &[], &mut live);
        flatten(
            &mut self.sim,
            &mut self.scene,
            &self.map,
            &mut self.pen,
            &bl,
            &xf,
        )
    }

    fn cheio(&self, p: [f64; 2]) -> bool {
        self.scene
            .paths()
            .iter()
            .any(|c| self.scene.path_contains_point(c.id, p))
    }
}

/// Duas formas que se SOBREPÕEM viram uma — e o miolo comum continua cheio (uma fusão por
/// `EvenOdd` furava-o: a tela mudaria no clique).
#[test]
fn two_overlapping_shapes_become_one_and_the_overlap_stays_filled() {
    let (mut m, ids) = Mundo::com(&[([0.0, 0.0], [2.0, 2.0]), ([1.0, 1.0], [3.0, 3.0])]);
    assert_eq!(m.achata(&ids), 1);
    assert_eq!(m.scene.paths().len(), 1, "uma forma só");
    for p in [[0.5, 0.5], [1.5, 1.5], [2.5, 2.5]] {
        assert!(m.cheio(p), "{p:?} deixou de estar pintado");
    }
    assert_eq!(
        m.pen.selected_paths().len(),
        1,
        "o resultado fica escolhido"
    );
}

/// Uma booleana VIVA sozinha: consolida o que desenha (o furo do Subtract continua furo) e o
/// grupo morre.
#[test]
fn a_live_boolean_alone_keeps_its_hole_and_its_group_dies() {
    let (mut m, ids) = Mundo::com(&[([0.0, 0.0], [4.0, 4.0]), ([1.0, 1.0], [3.0, 3.0])]);
    assert!(arm(&mut m.sim, &m.scene, &m.map, &ids, 1));
    let g = group_of_selection(&m.sim, &m.map, &ids).unwrap();
    assert_eq!(m.achata(&ids), 1);
    assert!(m.sim.world().get_entity(g).is_err(), "o grupo ficou");
    assert!(m.cheio([0.5, 0.5]), "o anel");
    assert!(!m.cheio([2.0, 2.0]), "o furo encheu");
}

/// Uma booleana viva MAIS uma forma solta: as duas fundem-se numa só.
#[test]
fn a_live_boolean_and_a_loose_shape_merge_into_one() {
    let (mut m, ids) = Mundo::com(&[
        ([0.0, 0.0], [4.0, 4.0]),
        ([1.0, 1.0], [3.0, 3.0]),
        ([3.5, 3.5], [6.0, 6.0]),
    ]);
    assert!(arm(&mut m.sim, &m.scene, &m.map, &ids[..2], 1));
    assert_eq!(m.achata(&ids), 1);
    assert_eq!(m.scene.paths().len(), 1, "uma forma só");
    assert!(m.cheio([5.0, 5.0]), "a solta");
    assert!(!m.cheio([2.0, 2.0]), "o furo da booleana");
}

/// Uma forma solta sozinha não tem o que achatar: o documento fica igual.
#[test]
fn a_single_loose_shape_is_left_alone() {
    let (mut m, ids) = Mundo::com(&[([0.0, 0.0], [2.0, 2.0])]);
    let antes = m.scene.paths().to_vec();
    m.achata(&ids);
    assert_eq!(m.scene.paths(), antes.as_slice());
}

// ─── a booleana destrutiva (vinda da shell, F50) ──────────────────────────

/// Um quadrado externo e outro DENTRO dele, ambos selecionados (externo atrás).
fn nested_selection() -> (VecScene, ph2d_vec_edit::PenTool, [u64; 2]) {
    let mut scene = VecScene::new();
    let outer = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    let inner = scene.push_path(rectangle([3.0, 3.0], [7.0, 7.0]));
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[outer, inner]);
    (scene, pen, [outer, inner])
}

/// A regressão que motivou o bloco: Subtract agia nas duas últimas regiões
/// fechadas do DOCUMENTO, ignorando a seleção — e devolvia dois discos
/// sólidos em vez de uma rosquinha.
#[test]
fn boolean_subtract_uses_the_selection_and_makes_a_real_hole() {
    let (mut scene, mut pen, _) = nested_selection();
    // Um terceiro path, NÃO selecionado, bem longe: a booleana antiga o teria
    // agarrado (é uma das duas últimas fechadas); a nova tem de ignorá-lo.
    let bystander = scene.push_path(ph2d_vec_scene::rectangle([90.0, 90.0], [95.0, 95.0]));

    pathfinder_selection(
        &mut scene,
        &mut pen,
        &ph2d_vec_scene::VecXforms::new(),
        PathfinderOp::Subtract,
    );

    assert_eq!(scene.paths().len(), 2, "resultado + o bystander intacto");
    assert!(scene.paths().iter().any(|p| p.id == bystander));
    let donut = scene.paths().iter().find(|p| p.id != bystander).unwrap();
    assert!(donut.is_compound(), "o furo vive num subpath");
    let id = donut.id;
    assert!(scene.path_contains_point(id, [1.0, 5.0]), "o anel é sólido");
    assert!(
        !scene.path_contains_point(id, [5.0, 5.0]),
        "o centro é vazado"
    );
    // O resultado entra na fatia de z da BASE (não salta pro topo).
    assert_eq!(scene.paths()[0].id, id);
    assert_eq!(pen.selected(), Some(id), "a booleana seleciona o resultado");
}

#[test]
fn boolean_needs_two_selected_closed_regions() {
    let (mut scene, mut pen, ids) = nested_selection();
    pen.select(Some(ids[0])); // só um selecionado
    pathfinder_selection(
        &mut scene,
        &mut pen,
        &ph2d_vec_scene::VecXforms::new(),
        PathfinderOp::Union,
    );
    assert_eq!(scene.paths().len(), 2, "no-op");
}

// ─── curas da auditoria (fecho F46–F50) ───────────────────────────────────

/// Quantas peças o plano que `recook` cozinha para `g` neste estado.
fn plano(m: &Mundo, g: Entity) -> Option<usize> {
    let mut live = ph2d_vec_render::LiveGeometry::new();
    let mut bl = BoolLive::default();
    bl.recook(
        &m.scene,
        &m.sim,
        &m.map,
        &VecXforms::default(),
        &[],
        &mut live,
    );
    bl.plan(g).map(|p| p.out.len())
}

/// Um traço ABERTO filho de uma booleana viva não é operando: o Flatten guarda-o (vai para o pai do
/// grupo) e só o grupo morre.
#[test]
fn flatten_keeps_an_open_stroke_inside_a_live_boolean() {
    let (mut m, ids) = Mundo::com(&[([0.0, 0.0], [4.0, 4.0]), ([1.0, 1.0], [3.0, 3.0])]);
    let mut aberto = rectangle([5.0, 5.0], [6.0, 6.0]);
    aberto.closed = false;
    let fio = m.scene.push_path(aberto);
    ph2d_vec_entities::entities::sync(&mut m.sim, &mut m.scene, &mut m.map);
    assert!(arm(&mut m.sim, &m.scene, &m.map, &ids, 1));
    let g = group_of_selection(&m.sim, &m.map, &ids).unwrap();
    let e = Entity::from_bits(m.map[&fio]);
    m.sim.world_mut().entity_mut(e).insert(ph2d_ecs::ChildOf(g));
    m.achata(&ids);
    ph2d_vec_entities::entities::sync(&mut m.sim, &mut m.scene, &mut m.map);
    assert!(m.sim.world().get_entity(g).is_err(), "o grupo ficou");
    assert!(
        m.scene.paths().iter().any(|p| p.id == fio),
        "o traço aberto sumiu com o grupo"
    );
}

/// Uma receita antiga (Trim, código 5) consolida-se no que desenha — cada peça com a sua cor —, sem
/// a Union que as fundiria numa só.
#[test]
fn flatten_keeps_the_pieces_of_a_recipe_group() {
    let (mut m, ids) = Mundo::com(&[([0.0, 0.0], [2.0, 2.0]), ([1.0, 1.0], [3.0, 3.0])]);
    assert!(arm(&mut m.sim, &m.scene, &m.map, &ids, 5));
    let g = group_of_selection(&m.sim, &m.map, &ids).unwrap();
    let n = plano(&m, g).expect("o Trim cozinha");
    assert!(n >= 2, "a fixtura precisa de peças ({n})");
    m.achata(&ids);
    assert_eq!(m.scene.paths().len(), n, "as peças do Trim fundiram-se");
}

/// Uma booleana que embrulha só outra (sem plano) não esconde a de dentro: o furo do Subtract
/// sobrevive ao Flatten.
#[test]
fn flatten_bakes_the_outermost_group_that_has_a_plan() {
    let (mut m, ids) = Mundo::com(&[([0.0, 0.0], [4.0, 4.0]), ([1.0, 1.0], [3.0, 3.0])]);
    assert!(arm(&mut m.sim, &m.scene, &m.map, &ids, 1));
    let g1 = group_of_selection(&m.sim, &m.map, &ids).unwrap();
    let g2 = m.sim.world_mut().spawn(VecBoolGroup { op: 0 }).id();
    m.sim
        .world_mut()
        .entity_mut(g1)
        .insert(ph2d_ecs::ChildOf(g2));
    assert!(plano(&m, g2).is_none(), "G2 não pode ter plano");
    m.achata(&ids);
    assert!(m.cheio([0.5, 0.5]), "o anel");
    assert!(!m.cheio([2.0, 2.0]), "o furo encheu");
}
