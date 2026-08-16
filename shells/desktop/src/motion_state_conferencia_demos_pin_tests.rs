//! Gates da cena `=50` — **O PINO ALCANÇA AS SIMULAÇÕES**.
//!
//! Eles medem a geometria que a cena COZINHA, não a intenção com que ela foi
//! escrita.
//!
//! ⚠️ **Toda banda desta cena é `Effect::Temporal`, então o harness TEM de chamar
//! `advance_tick`** — sem ele a aresta `pre` nunca carrega estado, os três
//! geradores ficam na pose de semeadura e **as duas metades de cada par saem bit a
//! bit idênticas**, com todo gate a passar por vácuo. É a lição que o
//! `rope_thickness` pagou duas vezes, e ela é a mesma aqui.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const TICKS: usize = 180;
const DT: f64 = 1.0 / 60.0;

/// O tique em que a leitura CEDO é tirada — o primeiro depois de a aresta `pre`
/// carregar estado, ou seja o primeiro em que o pino já está no laço.
const EARLY: usize = 2;

/// As poses de TODA banda em dois instantes: `EARLY` e o fim.
///
/// ⚠️ **Dois instantes, e não um, porque *segurar* é uma propriedade do TEMPO.**
/// Uma foto do fim não distingue *ficou onde estava* de *foi parar ali*, e a
/// dispersão de um grupo pinado é a que a SEMEADURA lhe deu — não algo que o pino
/// escolha (medido: os três pinados nascem a 1,4580 de dispersão, e um gate que
/// pedisse *"eles estão juntos"* estaria a medir o gerador de números aleatórios).
struct Frames {
    early: Vec<Vec<[f32; 2]>>,
    late: Vec<Vec<[f32; 2]>>,
}

fn cook_all() -> Vec<Vec<[f32; 2]>> {
    cook_frames().late
}

/// ⚠️ **Uma cena, um `Cook`** — as seis bandas partilham o documento, então cozer
/// cada uma num `Cook` próprio faria seis simulações independentes. O harness
/// avança o tique UMA vez e lê as seis saídas do MESMO estado.
fn cook_frames() -> Frames {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_pin_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), PAIRS * 2, "duas bandas por par");

    let mut cook = Cook::new();
    let mut early = vec![Vec::new(); sinks.len()];
    let mut late = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, playhead)
            .expect("o tique avanca o `pre`");
        for (lane, sink) in sinks.iter().enumerate() {
            let s = cook
                .cook(&doc.graph, &reg, *sink, playhead)
                .expect("a banda coze")[0]
                .as_stream()
                .clone();
            if k == EARLY || k + 1 == TICKS {
                let p = match s.get("P") {
                    Some(Column::Vec2(v)) => v.clone(),
                    _ => panic!("a banda {lane} nao emite P"),
                };
                if k == EARLY {
                    early[lane] = p;
                } else {
                    late[lane] = p;
                }
            }
        }
    }
    Frames { early, late }
}

fn worst(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    assert_eq!(a.len(), b.len(), "o par tem o mesmo tamanho");
    a.iter()
        .zip(b)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max)
}

/// **A CENA MONTA AS SEIS BANDAS QUE O LOG NOMEIA.**
///
/// ⚠️ A contagem é DERIVADA, nunca um literal: um número escrito à mão só sabe
/// dizer *"mudou"*, e a pergunta é se a lista que o artista lê corresponde ao que a
/// cena de facto empilha.
#[test]
fn the_scene_builds_every_band_the_log_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_pin_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(
        sinks.len(),
        band_labels().count(),
        "cada banda montada tem um rótulo no log, e cada rótulo tem uma banda",
    );
}

/// **TODA BANDA TERMINA NUM `motion.output`.**
///
/// O laço de render RE-RESOLVE os sinks a cada quadro a partir dos nós de saída do
/// grafo, então uma cadeia sem Output cozinha certo, passa nos outros gates e
/// **desenha NADA na tela** — indistinguível da feature quebrada.
#[test]
fn every_band_ends_in_an_output_node() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_pin_demo_document(&mut doc, &reg).expect("a cena monta");
    for (i, sink) in sinks.iter().enumerate() {
        assert_eq!(
            doc.graph.node(*sink).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "a banda {i} tem de terminar num Output",
        );
    }
}

/// **A CORDA DA DIREITA DOBRA SOBRE UM PONTO QUE NÃO É PONTA.**
#[test]
fn the_pinned_rope_folds_over_a_point_that_is_not_an_end() {
    let p = cook_all();
    let (free, held) = (&p[0], &p[1]);
    let i = 12usize;

    // O CONTROLE: sem o pino aquele ponto está a chicotear com o resto.
    assert!(
        worst(free, held) > 1.0,
        "o pino tem de mudar a corda; o maior desvio foi {:.4}",
        worst(free, held)
    );
    // E o ponto pinado ficou LONGE de onde a corda livre o deixou.
    let d = ((free[i][0] - held[i][0]).powi(2) + (free[i][1] - held[i][1]).powi(2)).sqrt();
    assert!(
        d > 0.5,
        "o ponto {i} tem de ficar noutro lugar; ele mediu {d:.4}",
    );
}

/// **O CORPO DA DIREITA PENDE DA PRIMEIRA LINHA EM VEZ DE CAIR.**
#[test]
fn the_pinned_body_hangs_instead_of_falling() {
    let p = cook_all();
    let (free, held) = (&p[2], &p[3]);
    let top = |v: &[[f32; 2]]| v.iter().map(|q| q[1]).fold(f32::NEG_INFINITY, f32::max);
    assert!(
        top(held) > top(free) + 2.0,
        "o corpo pinado tem de ficar bem mais alto: livre {:.4} · pinado {:.4}",
        top(free),
        top(held)
    );
}

/// **TRÊS AGENTES DA DIREITA NÃO SE MOVEM, E O RESTO DO BANDO SIM.**
///
/// ⚠️ **O oráculo é o DESLOCAMENTO entre dois instantes**, e a primeira versão
/// deste gate media a **dispersão** dos três — que é a que a SEMEADURA lhes deu
/// (1,4580 contra 2,7781 do resto, uma razão de 0,52 sobre uma barra de 0,5): ele
/// reprovava sobre produto correto porque estava a medir o gerador de números
/// aleatórios do bando e a chamar-lhe pino.
///
/// ⚠️ E as DUAS metades são precisas: *os pinados não andam* passaria sobre um bando
/// inteiro congelado, e *o resto anda* passaria sobre um pino inerte.
#[test]
fn three_pinned_agents_hold_while_the_flock_flies() {
    let f = cook_frames();
    let (a, b) = (&f.early[5], &f.late[5]);
    let moved = |i: usize| ((b[i][0] - a[i][0]).powi(2) + (b[i][1] - a[i][1]).powi(2)).sqrt();
    for i in 0..3 {
        assert!(
            moved(i) < 1e-5,
            "o agente pinado {i} tem de segurar; ele andou {:.6}",
            moved(i)
        );
    }
    let rest = (3..b.len()).map(moved).fold(0.0f32, f32::max);
    assert!(
        rest > 1.0,
        "o resto do bando tem de voar; o maior deslocamento foi {rest:.4}",
    );
}

/// A SONDA que imprime os números da mensagem do smoke.
#[test]
#[ignore = "sonda"]
fn probe_the_scene_numbers() {
    let p = cook_all();
    let labels: Vec<&str> = band_labels().map(|(_, l)| l).collect();
    for (i, l) in labels.iter().enumerate() {
        println!("  {}. {l}", i + 1);
    }
    println!(
        "corda: maior desvio {:.4} | ponto 12 livre {:?} pinado {:?}",
        worst(&p[0], &p[1]),
        p[0][12],
        p[1][12]
    );
    let top = |v: &[[f32; 2]]| v.iter().map(|q| q[1]).fold(f32::NEG_INFINITY, f32::max);
    println!(
        "corpo: topo livre {:.4} | topo pinado {:.4}",
        top(&p[2]),
        top(&p[3])
    );
    let f = cook_frames();
    let (a, b) = (&f.early[5], &f.late[5]);
    let moved = |i: usize| ((b[i][0] - a[i][0]).powi(2) + (b[i][1] - a[i][1]).powi(2)).sqrt();
    println!(
        "bando: {} agentes | pinados andaram {:.6}/{:.6}/{:.6} | o resto ate' {:.4}",
        b.len(),
        moved(0),
        moved(1),
        moved(2),
        (3..b.len()).map(moved).fold(0.0f32, f32::max)
    );
}
