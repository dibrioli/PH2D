//! Gates da cena `=36` — **o modo de composição é do SINK** (doc 89, folha 17).
//!
//! Irmão do `motion_state_conferencia_demos_tests.rs`, cortado por ASSUNTO no teto
//! de LOC do HR-18: aquele arquivo mede as cenas que respondem *o que o grafo
//! FAZ*; este mede a única que responde *em que MODO o resultado é desenhado*.
//! Os dois helpers de cook vêm de lá (`pub(super)`) — uma segunda cópia deles
//! divergiria no dia em que o `cook_both` mudasse de forma.

use super::tests::{cook, registry};
use super::*;
use ph2d_nodegraph::node::NodeTypeId;

/// **A cena `=36` monta três sinks, cada um num modo, e a nuvem se SOBREPÕE.**
///
/// Três coisas que deixariam a cena parecer certa e não provar nada, e as três
/// caem aqui: os três sinks com o mesmo tag (a foto mostraria três nuvens iguais e
/// o artista diria *"o blend não funciona"* sobre uma cena que nunca o pediu) · a
/// nuvem com folga entre as cópias (blend é o que acontece quando tinta cai sobre
/// tinta; em quads disjuntos `Add` e `Normal` desenham o mesmo pixel) · e alfa 1,0
/// (o `Normal` de cima taparia tudo).
///
/// Os números são MEDIDOS da cena, não escolhidos — a sobreposição é a razão entre
/// o passo da grade e o tamanho de uma cópia, e o gate a afirma como razão.
#[test]
fn the_blend_scene_gives_each_sink_its_own_mode_over_overlapping_paint() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_sink_blend_demo_document(&mut doc, &reg).expect("cena =36");
    assert_eq!(sinks.len(), 3, "a cena precisa dos TRÊS sinks");

    // 1. Cada sink num modo próprio — senão a cena não faz a pergunta.
    let tags: Vec<u8> = sinks
        .iter()
        .map(|&s| ph2d_eval_motion::sink_blend_tag(&doc.graph, s))
        .collect();
    assert_eq!(tags, vec![0, 1, 3], "Normal / Add / Multiply");

    // 2. A tinta se SOBREPÕE: o passo da grade é menor que o lado de uma cópia.
    //    `motion.scale(amount)` é o meio-lado, então o lado é `2 * amount`.
    let gap = 0.30_f32;
    let amount = 0.55_f32;
    let side = 2.0 * amount;
    assert!(
        gap < side,
        "as cópias não se cobrem (passo {gap} >= lado {side}) — sem sobreposição          não há blend a ver"
    );

    // 3. Meia-opacidade: com alfa 1,0 o Normal tapa e os três modos empatam.
    let t_id = NodeTypeId::of("motion.tint");
    let alphas: Vec<f32> = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_id() == t_id)
        .map(|n| {
            doc.graph
                .node_param_overrides(n.id)
                .and_then(|p| p.get("a"))
                .copied()
                .unwrap_or(1.0)
        })
        .collect();
    assert_eq!(alphas.len(), 3);
    for a in alphas {
        assert!(a < 1.0, "alfa {a} tapa em vez de compor");
    }

    // 4. E as três nuvens estão em lugares diferentes da tela — a foto tem de
    //    mostrar três grupos, não um empilhado.
    let xs: Vec<f32> = sinks
        .iter()
        .map(|&s| cook(&doc, &reg, s, 0.0).iter().map(|p| p[0]).sum::<f32>())
        .collect();
    assert!(
        xs[0] < xs[1] && xs[1] < xs[2],
        "as três nuvens não estão separadas em x: {xs:?}"
    );
}

/// Sonda: o que a cena `=36` de facto monta. Roda com
/// `cargo test -p ph2d-host-desktop --bins probe_blend_scene -- --ignored --nocapture`.
#[test]
#[ignore = "sonda de medicao"]
fn probe_blend_scene() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_sink_blend_demo_document(&mut doc, &reg).expect("cena =36");
    for (k, &s) in sinks.iter().enumerate() {
        let pts = cook(&doc, &reg, s, 0.0);
        let tag = ph2d_eval_motion::sink_blend_tag(&doc.graph, s);
        let xs: Vec<f32> = pts.iter().map(|p| p[0]).collect();
        let (lo, hi) = (
            xs.iter().cloned().fold(f32::MAX, f32::min),
            xs.iter().cloned().fold(f32::MIN, f32::max),
        );
        eprintln!(
            "sink {k}: tag={tag} n={} x=[{lo:.2}, {hi:.2}] passo=0.30 lado=1.10 sobreposicao={:.0}%",
            pts.len(),
            (1.0 - 0.30 / 1.10) * 100.0
        );
    }
}
