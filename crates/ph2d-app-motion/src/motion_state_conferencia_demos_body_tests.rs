//! Gates da cena `=87` — **o corpo que não é um retângulo** (doc 89, folha 03).
//!
//! ⚠️ **O oráculo é a FORMA, e não «a nuvem mudou de tamanho».** Uma régua de
//! contagem, de caixa envolvente ou de excursão daria verde sobre três nuvens
//! rectangulares de tamanhos diferentes e não diria nada. O que estas colunas
//! prometem é topologia: o anel tem um BURACO e a cruz tem CANTOS VAZIOS — duas
//! coisas que nenhum `rows × cols` tem, e as duas mediveis num tique.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_body_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// Coze `ticks` quadros e devolve `(o primeiro, o último)` — o `advance_tick` é o
/// que faz o `pre` andar; sem ele o laço lê o mesmo tique N vezes.
fn run(
    doc: &MotionDoc,
    reg: &NodeRegistry,
    sink: NodeId,
    ticks: u32,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut cook = Cook::new();
    let (mut first, mut last) = (Vec::new(), Vec::new());
    for k in 0..ticks {
        let t = f64::from(k) / 60.0;
        let out = cook.cook(&doc.graph, reg, sink, t).expect("cozinha");
        if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
            if k == 0 {
                first = p.clone();
            }
            last = p.clone();
        }
        cook.advance_tick(&doc.graph, reg, t)
            .expect("avanca o quadro");
    }
    (first, last)
}

fn seed(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<[f32; 2]> {
    run(doc, reg, sink, 1).0
}

/// O centro da nuvem — o CENTROIDE dela, nunca o ponto de mundo onde a cena a
/// pousou.
///
/// ⚠️ **A primeira versão usava o ponto autorado, e mediu a coisa errada:** um
/// corpo pendurado BALANÇA e CAI, então a distância dele ao lugar onde nasceu
/// cresce com o gesto e não com a deformação. A envergadura da malha de controle
/// lia-se **2,94×** com o corpo perfeitamente inteiro. Uma régua ancorada no
/// mundo mede o percurso; a forma mede-se do próprio centro.
fn centroid(p: &[[f32; 2]]) -> [f32; 2] {
    let s = p
        .iter()
        .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    [s[0] / p.len() as f32, s[1] / p.len() as f32]
}

/// A menor distância de `c` a qualquer partícula.
fn nearest(p: &[[f32; 2]], c: [f32; 2]) -> f32 {
    p.iter()
        .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
        .fold(f32::MAX, f32::min)
}

/// A meia-largura da nuvem (a maior das duas semi-extensões).
fn reach(p: &[[f32; 2]], c: [f32; 2]) -> f32 {
    p.iter()
        .map(|q| (q[0] - c[0]).abs().max((q[1] - c[1]).abs()))
        .fold(0.0, f32::max)
}

/// **A CENA CONSTRÓI AS TRÊS COLUNAS**, e cada uma emite instâncias.
#[test]
fn the_body_scene_builds_every_column() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), COLS_TABLE.len(), "uma sink por coluna");
    for (k, &s) in sinks.iter().enumerate() {
        assert!(
            !seed(&doc, &reg, s).is_empty(),
            "coluna {k} nao emitiu nada"
        );
    }
}

/// **A PORTA GANHA.** As três colunas têm contagens diferentes, e só a primeira
/// tem a da malha autorada — se a porta fosse ignorada, as três dariam `7 × 7`.
#[test]
fn the_port_decides_the_body_not_rows_and_cols() {
    let (doc, reg, sinks) = scene();
    let n: Vec<usize> = sinks.iter().map(|&s| seed(&doc, &reg, s).len()).collect();
    let mesh = (MESH_SIDE * MESH_SIDE) as usize;
    assert_eq!(n[0], mesh, "a coluna de controle e' a malha {mesh}");
    for (k, &c) in n.iter().enumerate().skip(1) {
        assert_ne!(
            c, mesh,
            "coluna {k}: a porta foi ignorada e o corpo voltou a ser a malha"
        );
    }
}

/// ⭐ **O ANEL TEM UM BURACO** — a propriedade que nenhuma malha tem, e o
/// CONTROLE ao lado: a coluna do retângulo tem partícula no próprio centro.
#[test]
fn the_ring_has_a_hole_and_the_mesh_does_not() {
    let (doc, reg, sinks) = scene();
    let ring = COLS_TABLE
        .iter()
        .position(|c| c.form == Form::Ring)
        .unwrap();
    let mesh = COLS_TABLE
        .iter()
        .position(|c| c.form == Form::Mesh)
        .unwrap();

    let pr = seed(&doc, &reg, sinks[ring]);
    let pm = seed(&doc, &reg, sinks[mesh]);
    let (cr, cm) = (centroid(&pr), centroid(&pm));
    let (hole, solid) = (nearest(&pr, cr), nearest(&pm, cm));
    // O buraco mede-se contra o ALCANCE da própria peça, nunca em unidades de
    // mundo: uma barra absoluta mediria o `spacing` da cena em vez da topologia.
    let frac = hole / reach(&pr, cr);
    assert!(
        frac > 0.35,
        "o anel devia ter um buraco: a particula mais perto do centro esta a {frac:.3} do alcance"
    );
    assert!(
        solid < 1e-3,
        "CONTROLE: a malha tem particula NO centro, e a mais perto esta a {solid:.4}"
    );
}

/// ⭐ **A CRUZ TEM CANTOS VAZIOS** — os quatro cantos da caixa envolvente não têm
/// ninguém perto, e as quatro pontas têm. Um retângulo reprova nas duas metades.
#[test]
fn the_cross_is_a_cross_and_the_mesh_is_not() {
    let (doc, reg, sinks) = scene();
    let cross = COLS_TABLE
        .iter()
        .position(|c| c.form == Form::Cross)
        .unwrap();
    let mesh = COLS_TABLE
        .iter()
        .position(|c| c.form == Form::Mesh)
        .unwrap();

    for (k, form) in [(cross, Form::Cross), (mesh, Form::Mesh)] {
        let p = seed(&doc, &reg, sinks[k]);
        let c = centroid(&p);
        let r = reach(&p, c);
        let corner = [[1.0f32, 1.0], [1.0, -1.0], [-1.0, 1.0], [-1.0, -1.0]]
            .iter()
            .map(|d| nearest(&p, [c[0] + d[0] * r, c[1] + d[1] * r]) / r)
            .fold(f32::MAX, f32::min);
        let tip = [[1.0f32, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]]
            .iter()
            .map(|d| nearest(&p, [c[0] + d[0] * r, c[1] + d[1] * r]) / r)
            .fold(0.0, f32::max);
        if form == Form::Cross {
            assert!(
                corner > 0.4,
                "a cruz devia ter cantos VAZIOS; o canto mais povoado esta a {corner:.3}"
            );
            assert!(
                tip < 0.25,
                "a cruz devia ter as quatro PONTAS povoadas; a pior esta a {tip:.3}"
            );
        } else {
            assert!(
                corner < 0.1,
                "CONTROLE: a malha TEM canto, e o mais vazio esta a {corner:.3}"
            );
        }
    }
}

/// ⭐ **AS TRÊS BALANÇAM E NENHUMA SE DESFAZ.** Depois de dois segundos de mastro
/// a varrer, cada corpo andou de verdade **e** continua com a envergadura que
/// tinha — que é a promessa inteira de um corpo mole contra uma nuvem de pontos
/// soltos.
#[test]
fn every_body_swings_and_keeps_its_span() {
    let (doc, reg, sinks) = scene();
    for (k, &s) in sinks.iter().enumerate() {
        let (a, b) = run(&doc, &reg, s, 120);
        let travelled = a
            .iter()
            .zip(&b)
            .map(|(p, q)| (q[0] - p[0]).abs().max((q[1] - p[1]).abs()))
            .fold(0.0f32, f32::max);
        assert!(
            travelled > 0.2,
            "coluna {k}: o mastro varre e o corpo nao andou ({travelled:.4})"
        );
        let (r0, r1) = (reach(&a, centroid(&a)), reach(&b, centroid(&b)));
        let ratio = r1 / r0;
        assert!(
            (0.7..1.6).contains(&ratio),
            "coluna {k}: a envergadura foi de {r0:.3} para {r1:.3} ({ratio:.2}x) — o corpo desfez-se"
        );
    }
}

/// **O TOPO DE CADA FORMA FICA PRESO**, e o topo é o `y` máximo do REPOUSO — a lei
/// nova. Numa malha é a primeira fileira; no anel é o arco de cima; na cruz é o
/// braço de cima. E o CONTROLE está no mesmo teste: o fundo de cada corpo cai.
#[test]
fn every_body_hangs_by_the_top_of_its_own_shape() {
    let (doc, reg, sinks) = scene();
    for (k, &s) in sinks.iter().enumerate() {
        let (a, b) = run(&doc, &reg, s, 120);
        let top_y = a.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        let bot_y = a.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        let span = (top_y - bot_y).max(1e-3);
        // Quem nasceu no topo continua na altura em que nasceu (o mastro só varre
        // em x, então o pino não desce).
        let top_drop = a
            .iter()
            .zip(&b)
            .filter(|(p, _)| p[1] >= top_y - span * 1e-3)
            .map(|(p, q)| (p[1] - q[1]).abs())
            .fold(0.0f32, f32::max);
        assert!(
            top_drop < span * 0.05,
            "coluna {k}: o topo escorregou {top_drop:.4} numa peca de {span:.3}"
        );
        let bottom_moved = a
            .iter()
            .zip(&b)
            .filter(|(p, _)| p[1] <= bot_y + span * 1e-3)
            .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
            .fold(0.0f32, f32::max);
        assert!(
            bottom_moved > span * 0.05,
            "CONTROLE coluna {k}: o fundo tinha de se mexer, e andou {bottom_moved:.4}"
        );
    }
}

/// As fichas do canvas: uma por coluna, no lugar da coluna.
#[test]
fn every_column_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), COLS_TABLE.len(), "uma ficha por coluna");
    for (k, c) in caps.iter().enumerate() {
        assert!(
            (c.world[0] - col_x(k)).abs() < 1e-4,
            "a ficha {k} nao esta sobre a sua coluna"
        );
    }
}
