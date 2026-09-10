//! **O PIVÔ DO CALEIDOSCÓPIO** — os gates do vocabulário partilhado (ciclo 3, W1 — doc 106 §2.3).
//!
//! Cortado do `lib.rs` no teto de LOC do HR-18 (700 para `crates/`), pela mesma costura que o
//! `reindex_tests` já usa: o corte é por RESPONSABILIDADE — este ficheiro responde *«em torno de
//! quê este nó gira?»* e nada aqui mede a simetria em si.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::pivot::PivotMode;

/// Três elementos **longe da origem** — o único regime em que o modo se vê.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.kaleidoscope.pivot.src"),
    name: "motion.kaleidoscope.pivot.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(3).with(
            "P",
            Column::Vec2(vec![[9.0, 4.0], [10.0, 5.0], [11.0, 6.0]]),
        ));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionKaleidoscope),
            _ => None,
        }
    }
}

fn kal(mode: f32, px: f32, py: f32) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.kaleidoscope.pivot.src");
    let k = g.add_node("motion.kaleidoscope");
    g.set_param(k, "segments", 4.0);
    g.set_param(k, "reflect", 0.0);
    g.set_param(k, ph2d_nodegraph::pivot::PARAM, mode);
    g.set_param(k, "pivot_x", px);
    g.set_param(k, "pivot_y", py);
    g.connect(Edge {
        from: (src, 0),
        to: (k, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, k, 0.0).expect("cook");
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    }
}

/// **O default é o nó que shipou, AO BIT.**
///
/// ⚠️ E o default é `Point`, **não** o `World Origin` do `motion.transform`: este nó sempre
/// honrou o ponto digitado. Com o pivô em `(0,0)` os dois valores dão o mesmo número, mas
/// não a mesma UI — com `World Origin` o `ParamGate` esconderia dois sliders que o artista
/// já usa. *O default é a lei da identidade de cada nó, não uma propriedade do enum.*
#[test]
fn the_default_mode_is_the_node_that_shipped_bit_for_bit() {
    for (px, py) in [(0.0f32, 0.0f32), (2.5, -1.5), (-3.0, 2.0)] {
        let com_modo = kal(1.0, px, py);
        let sem_modo = kaleidoscope(
            &[[9.0, 4.0], [10.0, 5.0], [11.0, 6.0]],
            4,
            false,
            [px, py],
            0.0,
        );
        let a: Vec<[u32; 2]> = com_modo
            .iter()
            .map(|p| [p[0].to_bits(), p[1].to_bits()])
            .collect();
        let b: Vec<[u32; 2]> = sem_modo
            .iter()
            .map(|p| [p[0].to_bits(), p[1].to_bits()])
            .collect();
        assert_eq!(a, b, "pivo ({px}, {py})");
    }
}

/// ⭐ **O modo `Centroid` encontra o centro sozinho, e SEGUE o layout.**
///
/// O centroide de `[9, 10, 11] × [4, 5, 6]` é `(10, 5)`, e a estrela tem de nascer à volta
/// dele — não da origem, que é onde um `pivot_mode` ignorado a poria.
#[test]
fn the_centroid_mode_finds_the_centre_without_being_told() {
    let digitado = kal(1.0, 10.0, 5.0);
    let achado = kal(2.0, -99.0, 77.0);
    assert_eq!(
        achado, digitado,
        "o centroide e' (10, 5) e o modo VENCE o ponto digitado"
    );
    // E o ponto fixo e' o centro: uma fonte EM cima dele fica onde esta'.
    let meio = achado[1];
    assert!(
        (meio[0] - 10.0).abs() < 1e-4 && (meio[1] - 5.0).abs() < 1e-4,
        "o elemento no centroide e' o ponto fixo da rotacao: {meio:?}"
    );
}

/// **O vocabulário é o da porta** — a mesma régua do `motion.transform`, porque o achado que
/// abriu este ciclo foi *seis respostas diferentes à mesma pergunta*.
#[test]
fn the_pivot_vocabulary_is_the_ports() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == ph2d_nodegraph::pivot::PARAM)
        .expect("o no' declara o param do pivo");
    match hint.widget {
        ParamWidget::Enum { labels } => assert_eq!(labels, ph2d_nodegraph::pivot::LABELS),
        outro => panic!("o pivot_mode tem de ser um Enum, e' {outro:?}"),
    }
    // E a escada do hospedeiro e' a da porta, nao uma copia.
    assert_eq!(PivotMode::of(2.0), PivotMode::Centroid);
}
