//! Gates da cena `=82` — **o painel que encolhe** (doc 90, a cura dos knobs mortos).
//!
//! ⚠️ **O oráculo desta cena é a CONTAGEM DE FIGURAS, e o gate mede-a como o olho a mede:**
//! as duas cópias que uma célula desenha ou coincidem (⇒ uma linha) ou não (⇒ duas). Um gate
//! que medisse excursão passaria numa cena em que as duas metades fossem iguais — que é
//! exactamente o modo de falha desta cena.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// As posições de uma célula, já sem o deslocamento que a coloca na grelha.
fn points(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, cell: usize) -> Vec<[f32; 2]> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    let Some(Column::Vec2(v)) = Stream::get(s, "P") else {
        panic!("P")
    };
    let row = cell / 2;
    let half = cell % 2;
    let cx = if half == 0 { -COL_X } else { COL_X };
    let cy = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - row as f32 * ROW_GAP;
    v.iter().map(|p| [p[0] - cx, p[1] - cy]).collect()
}

/// A cor de cada peça, quando a célula a escreve.
fn tints(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<[f32; 4]> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match Stream::get(s, "tint") {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **A distância máxima entre as DUAS cópias sobrepostas.**
///
/// ⚠️ A célula é a concatenação das duas: as primeiras `COUNT` peças são a cópia com o controle
/// no mínimo, as seguintes com ele no máximo. *Coincidir* é essa distância ser zero.
fn split(p: &[[f32; 2]]) -> f32 {
    let n = COUNT as usize;
    assert_eq!(p.len(), 2 * n, "a celula tem de trazer as DUAS copias");
    (0..n)
        .map(|i| (p[i][1] - p[i + n][1]).abs())
        .fold(0.0f32, f32::max)
}

/// **A cena constrói as catorze células.**
#[test]
fn the_gate_scene_builds_every_cell() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gates_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len() * 2, "duas celulas por linha");
    let (n, count) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    assert!(count >= 8.0, "uma figura precisa de pecas que cheguem");
    for (k, sink) in sinks.iter().enumerate() {
        assert_eq!(
            points(&doc, &reg, *sink, k).len(),
            2 * count as usize,
            "celula {k}: as duas copias"
        );
    }
}

/// **ESQUERDA = UMA linha · DIREITA = DUAS.** É a cena inteira, numa asserção.
///
/// ⚠️ **As duas metades são obrigatórias.** Só a esquerda passaria numa cena em que o controle
/// não fizesse nada em lado nenhum (um nó partido); só a direita passaria numa cena que nunca
/// mostrasse o defeito. *O par é que é a prova.*
///
/// ⚠️ A linha da TINTA não entra aqui: ela responde em COR, e a distância vertical dela é
/// autorada (as duas cópias vivem em Y distintos de propósito — cor não se compara sobreposta).
/// O gate dela é o irmão abaixo.
#[test]
fn the_mute_half_shows_one_line_and_the_live_half_shows_two() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gates_demo_document(&mut doc, &reg).expect("constroi");
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        if matches!(row.knob, Knob::TintEnd) {
            continue;
        }
        let mute = split(&points(&doc, &reg, sinks[k * 2], k * 2));
        let live = split(&points(&doc, &reg, sinks[k * 2 + 1], k * 2 + 1));
        assert!(
            mute == 0.0,
            "linha {} ({}): a metade MUDA separou {mute} -- o controle AGE ai', \
             e o gate esta' a esconde-lo",
            k + 1,
            row.label
        );
        assert!(
            live > H * 0.05,
            "linha {} ({}): a metade VIVA nao separou (max {live}) -- \
             o controle continua mudo, ou a fixture nao contem o fenomeno",
            k + 1,
            row.label
        );
    }
}

/// **A TINTA responde em COR** — e o oráculo dela tem de ser a cor, não a posição.
#[test]
fn the_tint_row_is_one_colour_on_the_left_and_two_on_the_right() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gates_demo_document(&mut doc, &reg).expect("constroi");
    let k = ROWS_TABLE
        .iter()
        .position(|r| matches!(r.knob, Knob::TintEnd))
        .expect("a linha da tinta existe");
    let n = COUNT as usize;
    let spread = |cell: usize| -> f32 {
        let t = tints(&doc, &reg, sinks[cell]);
        assert_eq!(t.len(), 2 * n, "a celula da tinta tem de escrever `tint`");
        (0..n)
            .map(|i| {
                (0..4)
                    .map(|c| (t[i][c] - t[i + n][c]).abs())
                    .fold(0.0f32, f32::max)
            })
            .fold(0.0f32, f32::max)
    };
    assert_eq!(
        spread(k * 2),
        0.0,
        "em Solid as duas copias tem de ter a MESMA cor -- a `End` nao e' lida ai'"
    );
    assert!(
        spread(k * 2 + 1) > 0.1,
        "em Gradient as duas copias tem de diferir -- e' a `End` a agir"
    );
}

/// **NENHUMA CÉLULA INVADE A VIZINHA** — a lei de layout das cenas irmãs.
#[test]
fn no_cell_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gates_demo_document(&mut doc, &reg).expect("constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let p = points(&doc, &reg, *sink, k);
        let (mut mx, mut my) = (0.0f32, 0.0f32);
        for q in &p {
            mx = mx.max(q[0].abs());
            my = my.max(q[1].abs());
        }
        assert!(
            my < ROW_GAP * 0.5,
            "celula {k} sobe {my}, meia linha e' {}",
            ROW_GAP * 0.5
        );
        assert!(
            mx < COL_X,
            "celula {k} alarga {mx}, a coluna vive a {COL_X}"
        );
    }
}

/// **CADA LINHA DA CENA ENCENA UM PAR QUE A AUDITORIA CUROU** — e o seletor que ela vira é o
/// mesmo `when` do gate registado.
///
/// ⚠️ Sem isto a cena podia encenar um controle qualquer e continuar bonita: sete pares que
/// separam e coincidem provam uma LEI, não provam que ela é a lei dos dezanove.
#[test]
fn every_row_stages_a_param_the_audit_actually_cured() {
    let reg = registry();
    // (o nó da linha, o param curado, o seletor)
    let staged: &[(&str, &str, &str)] = &[
        ("motion.stagger", "ease_dir", "ease_curve"),
        ("value.step", "width", "mode"),
        ("value.map_range", "clamp", "interpolation"),
        ("value.instance_field", "seed", "mode"),
        ("value.noise", "roughness", "octaves"),
        ("motion.wiggle", "amp_mult", "octaves"),
        ("motion.tint", "r2", "mode"),
    ];
    assert_eq!(staged.len(), ROWS_TABLE.len(), "uma entrada por linha");
    for (node, param, when) in staged {
        let m = reg
            .manifests()
            .find(|m| m.name == *node)
            .unwrap_or_else(|| panic!("`{node}` existe"));
        let by_enum = reg
            .param_gates(m.id)
            .into_iter()
            .flatten()
            .find(|g| g.param == *param)
            .map(|g| g.when);
        let by_above = reg
            .param_gates_above(m.id)
            .into_iter()
            .flatten()
            .find(|g| g.param == *param)
            .map(|g| g.when);
        assert_eq!(
            by_enum.or(by_above),
            Some(*when),
            "`{node}::{param}` tem de estar gateado por `{when}` -- \
             a cena encena um par que a cura nao trata"
        );
    }
}
