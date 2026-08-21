//! Os gates da cena `=73` — identidade, posto, curva e forma.
//!
//! ⚠️ O que estes gates defendem é a ESTRUTURA: que cada par difere no número que a
//! banda anuncia, e **só** nele. O caminho é do olho do Enio.

use super::*;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn scene() -> (MotionDoc, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_rank_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipado");
    (doc, sinks)
}

fn nodes_of(doc: &MotionDoc, ty: &str) -> Vec<NodeId> {
    doc.graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ty)
        .map(|n| n.id)
        .collect()
}

fn param(doc: &MotionDoc, id: NodeId, name: &str) -> f32 {
    doc.graph
        .node_param_overrides(id)
        .and_then(|m| m.get(name).copied())
        .unwrap_or(f32::NAN)
}

/// **QUATRO PARES, OITO BANDAS, OITO SAÍDAS.**
#[test]
fn the_scene_is_four_pairs() {
    let (doc, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
    assert_eq!(nodes_of(&doc, "motion.output").len(), 8);
    assert_eq!(
        nodes_of(&doc, "motion.grid").len(),
        8,
        "uma semente por banda"
    );
}

/// **O PAR 1 DIFERE SÓ NO `reindex` — a fracção cortada é a MESMA.**
///
/// ⚠️ É a alma do par: se as duas metades cortassem números diferentes de peças, a
/// diferença de cor teria uma segunda explicação, e o smoke leria a errada.
#[test]
fn the_cull_pair_cuts_the_same_half_and_differs_only_in_the_renumbering() {
    let (doc, _) = scene();
    let culls = nodes_of(&doc, "motion.cull");
    assert_eq!(culls.len(), 2);
    for c in &culls {
        assert_eq!(param(&doc, *c, "amount"), KEEP, "a mesma fracção");
        assert_eq!(param(&doc, *c, "mode"), 0.0, "e o mesmo modo (Fraction)");
    }
    assert_eq!(param(&doc, culls[0], "reindex"), 0.0, "esquerda: como era");
    assert_eq!(param(&doc, culls[1], "reindex"), 1.0, "direita: renumera");
}

/// **O DEGRADÊ DO PAR 1 É O MESMO DOS DOIS LADOS, e ele é o INSTRUMENTO.**
///
/// A cor é o que se lê; se as duas pontas diferissem, o par mediria o degradê em vez
/// da contagem. E o `mode` tem de ser Gradient — em Solid o nó não olha o `Count` de
/// todo, e a cena inteira ficaria muda sobre a célula.
#[test]
fn the_cull_pair_reads_through_one_and_the_same_gradient() {
    let (doc, _) = scene();
    let tints: Vec<NodeId> = nodes_of(&doc, "motion.tint")
        .into_iter()
        .filter(|t| param(&doc, *t, "mode") == 1.0)
        .collect();
    assert_eq!(tints.len(), 2, "um degradê por banda do par 1");
    for k in ["r", "g", "b", "r2", "g2", "b2"] {
        assert_eq!(
            param(&doc, tints[0], k),
            param(&doc, tints[1], k),
            "`{k}` tem de ser o mesmo nos dois lados"
        );
    }
    assert_eq!(param(&doc, tints[0], "r"), RAMP_START[0]);
    assert_eq!(param(&doc, tints[0], "r2"), RAMP_END[0]);
}

/// **O PAR 2 DIFERE SÓ NO `key`, E SÓ A DIREITA TEM CAMPO — na porta 1.**
///
/// ⚠️ A porta importa: ligado à porta 0 o ruído seria geometria, e a banda sairia
/// espalhada pelo motivo errado.
#[test]
fn the_rank_pair_changes_only_what_orders_the_band() {
    let (doc, _) = scene();
    let irs = nodes_of(&doc, "field.index_range");
    assert_eq!(irs.len(), 2);
    for k in ["start", "end", "soft", "curve"] {
        assert_eq!(
            param(&doc, irs[0], k),
            param(&doc, irs[1], k),
            "a BANDA tem de ser a mesma; só o que a ordena muda (`{k}`)"
        );
    }
    assert_eq!(param(&doc, irs[0], "key"), 0.0, "esquerda: Index");
    assert_eq!(
        param(&doc, irs[1], "key"),
        ORDER_BY_ATTRIBUTE,
        "direita: Attribute"
    );
    let noises = nodes_of(&doc, "value.noise");
    assert_eq!(noises.len(), 1, "só a direita tem atributo");
    assert!(
        doc.graph
            .edges()
            .iter()
            .any(|e| e.from.0 == noises[0] && e.to == (irs[1], 1) && !e.delayed),
        "o campo tem de alimentar a porta `attr` (índice 1)"
    );
}

/// **O CAMPO DO PAR 2 DESCORRELACIONA VIZINHOS** — senão a banda "espalhada" sairia
/// em manchas, que é indistinguível de um bloco maior.
#[test]
fn the_rank_attribute_is_decorrelated_across_the_grid() {
    let (doc, _) = scene();
    let g8 = nodes_of(&doc, "motion.grid")
        .into_iter()
        .find(|g| param(&doc, *g, "cols") == 8.0)
        .expect("a grelha do par 2");
    let gap = param(&doc, g8, "gap_x");
    assert!(
        gap * ATTR_FREQ >= 0.5,
        "vizinhas a {} no espaço do ruído: perto demais",
        gap * ATTR_FREQ
    );
}

/// **O PAR 3 DIFERE SÓ NO `curve_offset`, e o contorno é `Curve` nos DOIS.**
///
/// ⚠️ Fora do contorno `Curve` o deslocamento é inerte — um par cuja esquerda
/// estivesse noutro contorno mediria o contorno, não o knob.
#[test]
fn the_shift_pair_changes_only_the_curve_offset() {
    let (doc, _) = scene();
    let rms = nodes_of(&doc, "field.remap");
    assert_eq!(rms.len(), 2);
    for r in &rms {
        assert_eq!(
            param(&doc, *r, "contour"),
            CONTOUR_CURVE,
            "o deslocamento só age no contorno Curve"
        );
    }
    assert_eq!(param(&doc, rms[0], "curve_offset"), 0.0);
    assert_eq!(param(&doc, rms[1], "curve_offset"), CURVE_SHIFT);
}

/// **A RAMPA DO PAR 3 CHEGA MESMO AO `falloff`** — pelo canal que existe para isso.
///
/// ⚠️ Sem este gate, um `channel` errado no `motion.drive` daria duas fileiras
/// idênticas (o remap sobre uma máscara ausente = 1 constante), e o smoke leria isso
/// como *"o `Curve Offset` não faz nada"* — que é a conclusão errada sobre o código
/// certo.
#[test]
fn the_shift_pair_actually_writes_the_mask_it_then_remaps() {
    let (doc, _) = scene();
    let drives = nodes_of(&doc, "motion.drive");
    assert_eq!(drives.len(), 2, "um por banda do par 3");
    for d in &drives {
        assert_eq!(param(&doc, *d, "channel"), DRIVE_FALLOFF, "o canal Falloff");
        assert_eq!(param(&doc, *d, "mode"), DRIVE_SET, "Set, não Add");
        assert!(
            doc.graph
                .edges()
                .iter()
                .any(|e| e.to == (*d, 1) && !e.delayed),
            "o valor tem de estar ligado à porta 1 do drive"
        );
    }
    let fields = nodes_of(&doc, "value.instance_field");
    assert_eq!(fields.len(), 2);
    for f in &fields {
        assert_eq!(
            param(&doc, *f, "mode"),
            FIELD_RAMP,
            "modo Ramp: o índice NORMALIZADO 0..1, não o índice cru"
        );
    }
}

/// **O PAR 4 DIFERE SÓ NO `Path Mode`, e o pentágono é o MESMO.**
#[test]
fn the_shape_pair_changes_only_the_path_mode() {
    let (doc, _) = scene();
    let shapes = nodes_of(&doc, "field.shape");
    assert_eq!(shapes.len(), 2);
    assert_eq!(param(&doc, shapes[0], "mode"), 0.0, "esquerda: Filled Path");
    assert_eq!(param(&doc, shapes[1], "mode"), 1.0, "direita: Path Edges");
    for k in ["distance", "curve"] {
        assert_eq!(param(&doc, shapes[0], k), param(&doc, shapes[1], k));
    }
    let pents = nodes_of(&doc, "motion.distribute_radial");
    assert_eq!(pents.len(), 2, "uma forma por banda");
    for p in &pents {
        assert_eq!(param(&doc, *p, "count"), SHAPE_SIDES);
        assert_eq!(param(&doc, *p, "radius"), SHAPE_RADIUS);
    }
    // E cada forma alimenta a PORTA 1 do campo dela — na porta 0 ela seria a arte.
    for (fs, pent) in shapes.iter().zip(&pents) {
        assert!(
            doc.graph
                .edges()
                .iter()
                .any(|e| e.from.0 == *pent && e.to == (*fs, 1) && !e.delayed),
            "a forma tem de entrar pela porta `shape`"
        );
    }
}

/// **A FORMA CABE DENTRO DA GRELHA QUE ELA MASCARA.**
///
/// ⚠️ Um pentágono de raio maior que a meia-largura da grelha mascararia TUDO, e as
/// duas bandas do par sairiam iguais — o par estaria verde e mudo. A conta é derivada
/// da grelha, não escrita à mão.
#[test]
fn the_pentagon_fits_inside_the_grid_it_masks() {
    let (doc, _) = scene();
    let g11 = nodes_of(&doc, "motion.grid")
        .into_iter()
        .find(|g| param(&doc, *g, "cols") == 11.0)
        .expect("a grelha do par 4");
    let half = (param(&doc, g11, "cols") - 1.0) * param(&doc, g11, "gap_x") * 0.5;
    assert!(
        SHAPE_RADIUS + SHAPE_DISTANCE < half,
        "o pentágono (+ penumbra) mede {} contra a meia-largura {half}",
        SHAPE_RADIUS + SHAPE_DISTANCE
    );
}

/// **O DIAGNOSER DA CASA NÃO ACHA BURACO NESTA CENA.**
#[test]
fn the_house_diagnoser_finds_no_hole_in_this_scene() {
    let (doc, _) = scene();
    let reg = registry();
    let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
    assert!(d.is_empty(), "a cena não encena defeito nenhum: {d:?}");
}
