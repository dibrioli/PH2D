//! **A CURVA QUE O ARTISTA DESENHOU** — os gates da rota nova.
//!
//! O smoke de 2026-08-12 reprovou o modelo do nó, não um número dele: *"esse é o
//! tipo de nó que simplesmente não faz sentido num app de última geração. Pontos e
//! alças em sliders num painel. Absurdo!"* A resposta já era a do app — o irmão
//! `motion.path` percorre *"uma forma desenhada de verdade em vez de quatro params
//! de ponto de controle"* desde o doc 65 —, e o que faltava era o DEFORMADOR
//! usá-la.
//!
//! ⚠️ **A metade que carrega o peso é a AUSÊNCIA.** Uma rota nova que muda o que o
//! nó faz sem forma escolhida moveria todo documento já autorado, em silêncio; os
//! gates de presença provam a feature, e o de ausência prova que ela não custou
//! nada a quem não a pediu.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::Graph;

/// Uma fonte de layout: uma fila em `x ∈ [0, 3]`, toda em `y = 0`.
static ROW: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spline_wrap.drawn.row"),
    name: "motion.spline_wrap.drawn.row",
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
struct Row;
impl NodeOp for Row {
    fn manifest(&self) -> &'static NodeManifest {
        &ROW
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p: Vec<P2> = (0..4u8).map(|i| [f32::from(i), 0.0]).collect();
        ctx.emit(Stream::new(4).with("P", Column::Vec2(p)));
    }
}
struct RowOps;
impl OpResolver for RowOps {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == ROW.id => Some(&Row),
            t if t == MANIFEST.id => Some(&MotionSplineWrap),
            _ => None,
        }
    }
}

/// Cozinha `row → spline_wrap`, com `shape` publicada sob `name` (ou nada) e o
/// nome escrito (ou não) no text param. Devolve as posições.
fn wrapped(shape: Option<(&str, &[P2])>, named: Option<&str>) -> Vec<P2> {
    let mut g = Graph::new();
    let row = g.add_node("motion.spline_wrap.drawn.row");
    let sw = g.add_node("motion.spline_wrap");
    if let Some(n) = named {
        g.set_text_param(sw, PATH_PARAM, n);
    }
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (row, 0),
        to: (sw, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    if let Some((n, pts)) = shape {
        cook.set_external(
            ph2d_nodegraph::external::curve_of(n),
            Stream::new(pts.len()).with("P", Column::Vec2(pts.to_vec())),
        );
    }
    match cook.cook(&g, &RowOps, sw, 0.0).expect("coze")[0]
        .as_stream()
        .get("P")
    {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Uma forma desenhada com geometria que a cúbica dos defaults **não tem**: uma
/// escada em `y` positivo, longe do S que os oito params descrevem.
const DRAWN: [P2; 3] = [[0.0, 10.0], [5.0, 10.0], [5.0, 20.0]];

/// **A fila pousa na curva DESENHADA** — e a prova é que ela pousa onde a cúbica
/// dos params não alcança.
///
/// A escada vive toda em `y ≥ 10`; a cúbica default varre `y ∈ [−2, 2]`. Um
/// oráculo que só pedisse *"as posições mudaram"* ficaria verde com a forma
/// ignorada e qualquer outro defeito no lugar.
#[test]
fn the_row_lands_on_the_shape_the_artist_drew() {
    let out = wrapped(Some(("Track", &DRAWN)), Some("Track"));
    assert_eq!(out.len(), 4);
    for p in &out {
        assert!(
            p[1] >= 9.99,
            "todo elemento pousa na escada desenhada, e {p:?} nao esta nela"
        );
    }
    // Os extremos da fila pousam nos extremos da curva (o `from`/`to` default é a
    // curva inteira, e o amostrador CLAMPA em vez de enrolar).
    assert!(
        (out[0][0]).abs() < 1e-3 && (out[0][1] - 10.0).abs() < 1e-3,
        "o primeiro no comeco: {:?}",
        out[0]
    );
    assert!(
        (out[3][0] - 5.0).abs() < 1e-3 && (out[3][1] - 20.0).abs() < 1e-3,
        "e o ultimo no FIM -- se o amostrador enrolasse, este saltaria para o comeco: {:?}",
        out[3]
    );
}

/// **Sem forma escolhida, o nó é o que sempre foi** — a cúbica dos oito params,
/// **AO BIT**. É esta metade que deixa todo documento já autorado intocado.
#[test]
fn with_no_shape_named_the_node_is_the_cubic_it_always_was() {
    let cp: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
    let p: Vec<P2> = (0..4u8).map(|i| [f32::from(i), 0.0]).collect();
    let expected = wrap(
        &p,
        &Curve::cubic(&cp),
        1.0,
        ArcMap {
            from: 0.0,
            to: 1.0,
            offset: 0.0,
        },
        false,
        1.0,
        &[],
    );
    assert_eq!(
        wrapped(None, None),
        expected,
        "o caminho sem forma tem de ser a cubica dos defaults, bit a bit"
    );
}

/// **Uma forma que não está lá cai na cúbica, e NÃO apaga o layout.**
///
/// ⚠️ Aqui o deformador diverge do irmão de propósito: o `motion.path` é uma FONTE
/// e emite vazio quando não acha a curva (*"ele não adivinha, e não falha"*).
/// Fazer o mesmo aqui apagaria a arte do artista porque ele renomeou uma forma —
/// um DEFORMADOR sem curva passa adiante.
#[test]
fn a_shape_that_is_not_there_falls_back_instead_of_deleting_the_layout() {
    for named in ["Apagada", ""] {
        let out = wrapped(None, Some(named));
        assert_eq!(out.len(), 4, "a contagem sobrevive (`{named}`)");
        assert_eq!(out, wrapped(None, None), "e e a cubica, bit a bit");
    }
    // Um ponto só não tem arco a percorrer — mesma rota.
    assert_eq!(
        wrapped(Some(("Ponto", &[[7.0, 7.0]])), Some("Ponto")),
        wrapped(None, None),
        "uma forma de um ponto so nao e uma curva"
    );
}

/// **Os controles do MAPA valem na curva desenhada** — `from`/`to`/`offset` são
/// aritmética sobre a fração de arco, não sobre a cúbica, então a forma desenhada
/// os herda sem uma linha por controle. É a razão de o que se abstrai ser a CURVA
/// e não o WRAP.
#[test]
fn the_map_controls_reach_the_drawn_curve_too() {
    let mut g = Graph::new();
    let row = g.add_node("motion.spline_wrap.drawn.row");
    let sw = g.add_node("motion.spline_wrap");
    g.set_text_param(sw, PATH_PARAM, "Track");
    g.set_param(sw, "to", 0.25);
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (row, 0),
        to: (sw, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::curve_of("Track"),
        Stream::new(DRAWN.len()).with("P", Column::Vec2(DRAWN.to_vec())),
    );
    let out = match cook.cook(&g, &RowOps, sw, 0.0).expect("coze")[0]
        .as_stream()
        .get("P")
    {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    // ⚠️ O arco da escada mede **15** (uma perna de 5 e uma de 10), não 20 — a
    // primeira versão deste gate somou errado e reprovou sobre um produto certo.
    // Um quarto de 15 são 3,75: a fila inteira cabe na primeira perna, que mede 5,
    // e nenhum elemento sobe a segunda.
    for p in &out {
        assert!(
            (p[1] - 10.0).abs() < 1e-3,
            "com `to = 0,25` a fila nao passa da quina: {p:?}"
        );
    }
    assert!(
        (out[3][0] - 3.75).abs() < 1e-3,
        "e o ultimo pousa exactamente no fim do trecho: {:?}",
        out[3]
    );
}

/// **A normal é a MESMA nos dois braços.** Trocar a convenção (esquerda × direita)
/// num deles espelharia o `height_scale` só na curva desenhada — um defeito que
/// nenhum gate de posição sobre `y = 0` pode ver, porque ali a normal não é lida.
#[test]
fn the_left_normal_convention_is_the_same_on_both_arms() {
    let line: [P2; 2] = [[0.0, 0.0], [4.0, 0.0]];
    let cubic: [P2; 4] = [[0.0, 0.0], [1.0, 0.0], [3.0, 0.0], [4.0, 0.0]];
    let map = ArcMap {
        from: 0.0,
        to: 1.0,
        offset: 0.0,
    };
    // Um layout ACIMA do eixo: o `y` é o que a normal desloca.
    let p: Vec<P2> = (0..4u8).map(|i| [f32::from(i), 1.0]).collect();
    let d = wrap(
        &p,
        &Curve::drawn(&line).expect("ha arco"),
        1.0,
        map,
        false,
        1.0,
        &[],
    );
    let c = wrap(&p, &Curve::cubic(&cubic), 1.0, map, false, 1.0, &[]);
    for (a, b) in d.iter().zip(&c) {
        assert!(
            (a[1] - b[1]).abs() < 1e-3 && a[1] > 0.9,
            "a mesma reta deslocada pela mesma normal: desenhada {a:?} contra cubica {b:?}"
        );
    }
}
