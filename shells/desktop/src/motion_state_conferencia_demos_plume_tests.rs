//! Gates da cena `=89` — **o emissor que deixa rasto** (doc 89, folha 01).
//!
//! ⚠️ **Estes gates são também a prova da COSTURA**, e é por isso que eles montam
//! o leque como o shell o monta: o nó sabe ler uma história, e quem a POUSA é o
//! `render_loop`. Um gate que só chamasse `emit` provaria a lei e deixaria a
//! costura por testar — a causa nº 1 da semana perdida no Painter.

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
    let sinks = build_plume_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

const DT: f64 = 1.0 / 60.0;

/// Coze `ticks` quadros **com os leques montados como o shell os monta**, e
/// devolve as posições do último.
fn run(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, ticks: u32) -> Vec<[f32; 2]> {
    let fans = ph2d_node_motion_emitter::time_fans(&doc.graph, reg, DT);
    let mut cook = Cook::new();
    let mut last = Vec::new();
    for k in 0..ticks {
        let t = f64::from(k) * DT;
        let out = cook
            .cook_scoped_fanned(&doc.graph, reg, sink, t, &Default::default(), &fans)
            .expect("cozinha");
        if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
            last = p.clone();
        }
        cook.advance_tick_fanned(&doc.graph, reg, t, &Default::default(), &fans)
            .expect("avanca o quadro");
    }
    last
}

fn row_of(m: Motion) -> usize {
    ROWS_TABLE
        .iter()
        .position(|r| r.motion == m)
        .expect("existe")
}

/// A largura horizontal que o penacho ocupa.
fn spread_x(p: &[[f32; 2]]) -> f32 {
    let (lo, hi) = p
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), q| (l.min(q[0]), h.max(q[0])));
    if p.is_empty() { 0.0 } else { hi - lo }
}

/// **A CENA CONSTRÓI AS TRÊS LINHAS**, e as três cospem.
#[test]
fn the_plume_scene_builds_every_row() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), ROWS_TABLE.len(), "uma sink por linha");
    for (k, &s) in sinks.iter().enumerate() {
        let p = run(&doc, &reg, s, 150);
        assert!(p.len() > 50, "linha {k}: so' {} particulas", p.len());
    }
}

/// ⭐ **O DEFEITO E A CURA, lado a lado.** O penacho que CARREGA é estreito (ele
/// anda inteiro com a fonte); o que DEIXA cobre o curso do varrimento.
///
/// ⚠️ **A régua é a LARGURA, e não a posição** — num varrimento periódico o
/// penacho está em qualquer sítio conforme o quadro, e uma régua de posição
/// mediria a fase.
#[test]
fn the_carried_plume_is_narrow_and_the_left_one_covers_the_sweep() {
    let (doc, reg, sinks) = scene();
    let carry = spread_x(&run(&doc, &reg, sinks[row_of(Motion::Carry)], 150));
    let leave = spread_x(&run(&doc, &reg, sinks[row_of(Motion::Leave)], 150));
    assert!(
        leave > carry * 2.0,
        "o rasto tinha de ser MUITO mais largo que o penacho carregado: {leave:.2} contra {carry:.2}"
    );
    // E ele cobre de facto o curso do varrimento (que vai de −SWEEP a +SWEEP).
    assert!(
        leave > SWEEP,
        "o rasto ({leave:.2}) tinha de cobrir o curso ({:.2})",
        2.0 * SWEEP
    );
}

/// ⭐⭐ **A VELOCIDADE HERDADA MANTÉM O JACTO FRESCO JUNTO DA FONTE**, e a
/// primeira versão deste gate afirmava o CONTRÁRIO.
///
/// Eu escrevi *"o jacto que herda espalha-se mais lateralmente"* e medi
/// **0,1522 contra 0,7943** — a herança APERTA o grupo, não o abre. E a física
/// diz porquê em uma linha: uma partícula que leva a velocidade da fonte
/// **viaja COM ela**, então ao fim de `Δt` está onde a fonte está. Sem herança
/// ela fica onde nasceu e a fonte foge — e são as que ficam que se esticam ao
/// longo do curso.
///
/// É a mangueira varrida: com herança a água sai INCLINADA e acompanha o bico;
/// sem ela, cada jorro fica no sítio e o varrimento desenha um leque.
///
/// ⚠️ Só as recém-nascidas, e de propósito: o integrador acumula, então uma
/// partícula velha já andou por outras razões e afogaria o sinal.
#[test]
fn inheriting_the_speed_keeps_the_fresh_jet_with_the_source() {
    let (doc, reg, sinks) = scene();
    // Um quadro em que o varrimento está DEPRESSA (o meio do curso): a `value.lfo`
    // cruza o zero a um quarto do período.
    let ticks = (PERIOD * 0.25 * 60.0) as u32 + 120 * 2;
    let lateral = |m: Motion| -> f32 {
        let p = run(&doc, &reg, sinks[row_of(m)], ticks);
        // As 20 mais novas são as últimas da lista (o emitter sai do mais velho
        // para o mais novo).
        let tail = &p[p.len().saturating_sub(20)..];
        let cx = tail.iter().map(|q| q[0]).sum::<f32>() / tail.len() as f32;
        let head = tail.last().copied().unwrap_or([0.0, 0.0]);
        (cx - head[0]).abs()
    };
    let (leave, inherit) = (lateral(Motion::Leave), lateral(Motion::Inherit));
    // Medido: 0,1522 (herda) contra 0,7943 (só deixa) — 5,2×.
    assert!(
        inherit < leave * 0.5,
        "o jacto que HERDA tinha de ficar JUNTO da fonte, e o que so' deixa de se esticar: \
         {inherit:.4} contra {leave:.4}"
    );
    // CONTROLE: se a fonte estivesse parada os dois dariam zero e o gate seria vácuo.
    assert!(
        leave > 0.1,
        "CONTROLE: sem varrimento nao ha' o que medir ({leave:.4})"
    );
}

/// ⚠️ **A COSTURA: sem o leque montado, as três linhas voltam a ser a mesma.**
/// É o gate que apanha um shell que esqueceu de pousar os mapas — o nó continua
/// correcto e a cena mente.
#[test]
fn without_the_fan_every_row_draws_the_same_plume() {
    let (doc, reg, sinks) = scene();
    let bare = |sink: NodeId| -> f32 {
        let mut cook = Cook::new();
        let mut last = Vec::new();
        for k in 0..150 {
            let t = f64::from(k) * DT;
            let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
            if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                last = p.clone();
            }
            cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        }
        spread_x(&last)
    };
    let a = bare(sinks[row_of(Motion::Carry)]);
    let b = bare(sinks[row_of(Motion::Leave)]);
    assert!(
        (a - b).abs() < 0.05,
        "CONTROLE: sem leque os dois modos coincidem ({a:.3} vs {b:.3}) — e e' com o leque \\
         montado que eles divergem"
    );
}

/// **O shell POUSA o leque do emissor** — o gate de FONTE da costura.
#[test]
fn the_shell_lays_the_emitters_fan_beside_the_trails() {
    let src = include_str!("render_loop/motion_bridge.rs");
    assert!(
        src.contains("ph2d_node_motion_emitter::time_fans"),
        "o render_loop tem de montar o leque do emissor"
    );
    assert!(
        src.contains("ph2d_node_motion_trail::time_fans"),
        "e o do rastro, no mesmo sitio"
    );
    assert!(src.contains("set_time_fans"), "e pousa-los na marcha");
}

/// As fichas do canvas: uma por linha, acima dela.
#[test]
fn every_row_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), ROWS_TABLE.len(), "uma ficha por linha");
    for (k, c) in caps.iter().enumerate() {
        assert!(c.world[1] > row_y(k), "a ficha {k} fica ACIMA da sua linha");
    }
}
