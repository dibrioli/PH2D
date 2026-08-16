//! Os gates da cena `=55` — a forma da onda e o deslize da rampa.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const DT: f64 = 1.0 / 60.0;
const TICKS: usize = 180;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Coze a cena e devolve, por banda, os Y de TODOS os elementos no tique `k`.
fn at(k: usize) -> Vec<Vec<f32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_shape_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    let t = k as f64 * DT;
    sinks
        .iter()
        .map(|s| {
            let v = cook.cook(&doc.graph, &reg, *s, t).expect("a banda coze");
            match v[0].as_stream().get("P") {
                Some(Column::Vec2(p)) => p.iter().map(|q| q[1]).collect(),
                _ => panic!("sem P"),
            }
        })
        .collect()
}

/// **A CENA MONTA AS QUATRO BANDAS E CADA UMA TERMINA NUM OUTPUT.**
#[test]
fn the_scene_builds_four_bands_that_all_end_in_an_output() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_shape_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas, dois pares");
    for s in &sinks {
        assert_eq!(
            doc.graph.node(*s).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "toda banda termina num Output"
        );
    }
    assert_eq!(band_labels().count(), 4, "um rótulo por banda");
}

/// **O CICLO DE TRABALHO É O QUE A CENA PROMETE** — a fração do tempo em cima.
///
/// ⚠️ O oráculo CONTA os tiques, não amostra um instante: num instante as duas
/// bandas podem estar as duas em cima, e a cena não diria nada.
#[test]
fn the_duty_cycle_is_what_the_scene_claims() {
    let (pw, _) = knobs();
    let mut up = [0usize; 2];
    for k in 0..TICKS {
        let p = at(k);
        for b in 0..2 {
            // ⚠️ Comparado com a MÉDIA da própria banda: cada uma carrega o seu
            // `BAND_DY`, e comparar coordenada crua mediria o layout.
            let mean = p[b].iter().sum::<f32>() / p[b].len() as f32;
            if p[b][0] > mean {
                up[b] += 1;
            }
        }
    }
    let frac = |n: usize| n as f32 / TICKS as f32;
    assert!(
        (frac(up[0]) - 0.5).abs() < 0.08,
        "o CONTROLE fica metade do ciclo em cima: {:.3}",
        frac(up[0])
    );
    assert!(
        (frac(up[1]) - pw).abs() < 0.08,
        "e a de baixo fica {pw:.2} dele: {:.3}",
        frac(up[1])
    );
}

/// **A RAMPA ROLA, E O TOPO MUDA DE LUGAR.**
///
/// ⚠️ As duas metades: o índice do MÁXIMO tem de andar (é a feature) e o
/// conjunto tem de continuar a ser uma rampa que sobe e cai UMA vez (senão o
/// knob estaria a embaralhar, não a rolar).
#[test]
fn the_ramp_rolls_and_the_top_moves() {
    let p = at(0);
    let top = |v: &Vec<f32>| {
        v.iter()
            .enumerate()
            .fold((0usize, f32::NEG_INFINITY), |a, (i, x)| {
                if *x > a.1 { (i, *x) } else { a }
            })
            .0
    };
    let (a, b) = (top(&p[2]), top(&p[3]));
    assert_eq!(a, p[2].len() - 1, "o CONTROLE tem o topo na ponta: {a}");
    assert!(
        b < a && b > 0,
        "e o rolado põe o topo NO MEIO da fileira: {b} de {}",
        p[3].len()
    );
    // Uma rampa rolada sobe uma vez e cai uma vez — nunca serrilha.
    let drops = p[3].windows(2).filter(|w| w[1] < w[0]).count();
    assert_eq!(
        drops, 1,
        "a rampa rolada cai UMA vez (a emenda): {:?}",
        p[3]
    );
}

/// **A SONDA** — imprime os números que a mensagem do smoke cita.
#[test]
#[ignore = "sonda"]
fn measure_what_the_scene_shows() {
    let (pw, off) = knobs();
    println!("cena 55 (pulse width {pw}, offset {off}):");
    let mut up = [0usize; 2];
    for k in 0..TICKS {
        let p = at(k);
        for b in 0..2 {
            let mean = p[b].iter().sum::<f32>() / p[b].len() as f32;
            if p[b][0] > mean {
                up[b] += 1;
            }
        }
    }
    for b in 0..2 {
        println!(
            "  banda {}: em cima {:.3} do ciclo",
            b + 1,
            up[b] as f32 / TICKS as f32
        );
    }
    let p = at(0);
    for b in 2..4 {
        let top = p[b]
            .iter()
            .enumerate()
            .fold((0usize, f32::NEG_INFINITY), |a, (i, x)| {
                if *x > a.1 { (i, *x) } else { a }
            });
        println!(
            "  banda {}: topo no índice {} de {}",
            b + 1,
            top.0,
            p[b].len()
        );
    }
}
