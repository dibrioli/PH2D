//! Gates da cena `=92` — **o que a simulação e a contagem não sabiam dizer** (folhas 03 e 07).
//!
//! ⚠️ **Estas bandas SIMULAM**, então o harness tem de chamar `advance_tick`: uma cadeia que
//! só coza lê o mesmo tique para sempre e todo gate de feedback fica verde por vácuo — a
//! armadilha que reprovou duas fixtures do gate da corda antes de ele medir alguma coisa.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const TICKS: usize = 150;
const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_ladder_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// Corre a cena `TICKS` tiques e devolve a última pose de cada sink pedido.
fn settle(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Vec<[f32; 2]>> {
    let mut cook = Cook::new();
    let mut last = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let out = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                last[i] = p.clone();
            }
        }
    }
    last
}

/// Os comprimentos entre elementos vizinhos, já sem o deslocamento do quadrante.
fn links(p: &[[f32; 2]]) -> Vec<f32> {
    p.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .collect()
}

/// A maior distância entre duas peças.
fn extent(p: &[[f32; 2]]) -> f32 {
    let mut d = 0.0_f32;
    for a in p {
        for b in p {
            d = d.max((a[0] - b[0]).hypot(a[1] - b[1]));
        }
    }
    d
}

/// **A CENA MONTA AS OITO BANDAS**, e as oito cospem.
#[test]
fn the_ladder_scene_builds_all_eight_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
    assert_eq!(band_labels().count(), 8, "um rotulo por banda");
    let poses = settle(&doc, &reg, &sinks);
    for (k, p) in poses.iter().enumerate() {
        assert!(!p.is_empty(), "banda {k} vazia");
        for q in p {
            assert!(q[0].is_finite() && q[1].is_finite(), "banda {k} explodiu");
        }
    }
}

/// ⭐ **O par 1: a corda afunila e NÃO encolhe** — as duas metades da célula, num gate.
#[test]
fn the_tapered_rope_thins_along_itself_without_shrinking() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks[..2]);
    let (flat, tapered) = (links(&poses[0]), links(&poses[1]));
    assert!(flat.len() > 8, "a corda tem elos que medir");

    // A uniforme é uniforme.
    let spread = flat.iter().fold(0.0_f32, |a, s| a.max((s - flat[0]).abs()));
    assert!(
        spread < flat[0] * 0.2,
        "CONTROLE: a corda de sempre tinha de ter elos iguais (desvio {spread:.4})"
    );
    // A afunilada não é: o primeiro elo é bem maior que o último.
    assert!(
        tapered[0] > tapered[tapered.len() - 1] * 2.0,
        "o elo de cima tinha de ser bem maior que o de baixo: {:.4} contra {:.4}",
        tapered[0],
        tapered[tapered.len() - 1]
    );
    // E o COMPRIMENTO TOTAL sobrevive — o perfil redistribui.
    let (t0, t1): (f32, f32) = (flat.iter().sum(), tapered.iter().sum());
    assert!(
        (t1 - t0).abs() < t0 * 0.15,
        "a corda mudou de comprimento: {t0:.4} -> {t1:.4}"
    );
}

/// ⭐ **O par 2: uma mola num eixo cola-se no outro; a de par atrasa nos DOIS.**
#[test]
fn the_pair_channel_lags_on_both_axes_where_the_scalar_one_lags_on_one() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks[2..4]);
    // Cada banda é UMA peça; o que se mede é onde ela está contra o alvo em órbita.
    assert_eq!(poses[0].len(), 1, "uma peca por banda");
    assert_eq!(poses[1].len(), 1, "idem");
    // As duas estão em sítios diferentes — se o canal não fosse lido, coincidiriam.
    let (a, b) = (poses[0][0], poses[1][0]);
    let d = (a[0] - b[0]).abs().max((a[1] - b[1]).abs());
    assert!(
        d > 0.05,
        "os dois canais coincidiram ({a:?} contra {b:?}) -- o `Position XY` nao esta' a ser lido"
    );
}

/// ⭐ **O par 3: a cauda com teto nasce mais apagada** — e a CABEÇA continua igual.
#[test]
fn the_capped_tail_starts_dimmer_and_the_head_does_not_move() {
    let (doc, reg, sinks) = scene();
    let mut cook = Cook::new();
    let mut alphas: Vec<Vec<(f32, f32)>> = vec![Vec::new(); 2];
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        for (i, &s) in sinks[4..6].iter().enumerate() {
            let out = cook.cook(&doc.graph, &reg, s, t).expect("coze");
            let st = out[0].as_stream();
            let ages = match st.get("trail_age") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            };
            if let Some(Column::Vec4(c)) = st.get("tint") {
                alphas[i] = ages.iter().copied().zip(c.iter().map(|q| q[3])).collect();
            }
        }
    }
    let newest = |v: &[(f32, f32)]| -> f32 {
        v.iter()
            .filter(|(a, _)| *a > 0.0)
            .min_by(|x, y| x.0.total_cmp(&y.0))
            .map_or(0.0, |(_, al)| *al)
    };
    let head =
        |v: &[(f32, f32)]| -> f32 { v.iter().find(|(a, _)| *a == 0.0).map_or(0.0, |(_, al)| *al) };
    assert!(
        !alphas[0].is_empty() && !alphas[1].is_empty(),
        "as duas cospem"
    );
    assert!(
        newest(&alphas[1]) < newest(&alphas[0]) * 0.6,
        "o eco mais novo com teto tinha de nascer bem mais apagado: {:.4} contra {:.4}",
        newest(&alphas[1]),
        newest(&alphas[0])
    );
    assert!(
        (head(&alphas[0]) - head(&alphas[1])).abs() < 1e-5,
        "e a CABECA nao se mexe: {:.5} contra {:.5}",
        head(&alphas[0]),
        head(&alphas[1])
    );
}

/// ⭐ **O par 4: a escada invertida percorre os mesmos degraus** — a mesma extensão, ao contrário.
#[test]
fn the_reversed_ladder_walks_the_same_rungs() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks[6..8]);
    let (up, down) = (&poses[0], &poses[1]);
    assert_eq!(up.len(), down.len(), "a mesma contagem de pecas");
    // A extensão vertical que a escada cobre é a mesma nos dois sentidos.
    let span = |p: &[[f32; 2]]| {
        p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
            (lo.min(q[1]), hi.max(q[1]))
        })
    };
    let (a, b) = (span(up), span(down));
    assert!(
        ((a.1 - a.0) - (b.1 - b.0)).abs() < 0.2,
        "os dois sentidos cobrem a mesma extensao: {:.3} contra {:.3}",
        a.1 - a.0,
        b.1 - b.0
    );
    // E as duas figuras não são a mesma — senão o `Direction` estaria morto.
    let d = up
        .iter()
        .zip(down)
        .map(|(x, y)| (x[1] - y[1]).abs())
        .fold(0.0_f32, f32::max);
    assert!(d > 0.1, "as duas escadas coincidiram ({d:.4})");
    assert!(extent(up) > 0.5, "CONTROLE: a escada de facto degrau-a");
}

/// As fichas do canvas: uma por banda, curta.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 8, "uma ficha por banda");
    for c in &caps {
        assert!(!c.text.contains("--"), "a ficha e' curta: {:?}", c.text);
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}
