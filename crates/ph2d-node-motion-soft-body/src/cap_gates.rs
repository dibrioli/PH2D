//! Os gates do TECTO da malha (o tecto por nó da casa, doc 124 §0.1) e da linearidade do custo —
//! separados do `lib.rs` pelo tecto de LOC (HR-18), por responsabilidade.

use super::shape::{rest_shape, shape_goals};
use super::{MAX_SIDE, PARAM_HARD_MAX, PARAM_HINTS};

/// **A CAPACIDADE tem de ser alcançável, e o SLIDER tem de ser arrastável** — duas
/// perguntas, e este gate afirmava as duas com um número só.
///
/// A preocupação original está inteira e continua gateada: *um controle que para em 40
/// sobre um clamp de 512 tem o terço de cima morto, e nada mais notaria*. O que mudou é
/// **de que lado do par soft/hard essa igualdade mora**: com o slider preso ao clamp, um
/// track de ~154 px movia **3,3 linhas por pixel** sobre um default de 4 — o `MAX_SIDE`
/// era alcançável e o DEFAULT não (doc 88 §11). Agora o teto DIGITÁVEL é o clamp (nada
/// se perdeu) e o curso do dedo é uma faixa de autoria estritamente abaixo dele.
#[test]
fn the_mesh_sliders_reach_exactly_the_clamp() {
    for param in ["rows", "cols"] {
        let hint = PARAM_HINTS
            .iter()
            .find(|h| h.param == param)
            .unwrap_or_else(|| panic!("{param} has a hint"));
        let hard = PARAM_HARD_MAX
            .iter()
            .find(|h| h.param == param)
            .unwrap_or_else(|| panic!("{param} has a hard max"));
        assert_eq!(
            hard.max, MAX_SIDE as f32,
            "o teto digitavel de {param} tem de alcancar o clamp, e parar nele"
        );
        assert!(
            hint.max < hard.max,
            "o slider de {param} e a FAIXA DE AUTORIA, nao o teto: \
             soft {} deveria ficar abaixo do hard {}",
            hint.max,
            hard.max
        );
    }
}

/// Coze o nó UMA vez (a semente) com `rows`/`cols` dados e devolve a saída.
fn semeia(rows: f32, cols: f32) -> ph2d_nodegraph::attr::Stream {
    use super::{MANIFEST, MotionSoftBody};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;
    use ph2d_nodegraph::node::{NodeOp, NodeTypeId};
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionSoftBody as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let sb = g.add_node("motion.soft_body");
    g.set_param(sb, "rows", rows);
    g.set_param(sb, "cols", cols);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, sb, 0.0).expect("coze");
    out[0].as_stream().clone()
}

/// ⭐⭐ **A ORDEM DO DONO (2026-10-08): o corpo mole respeita o tecto de instâncias por nó.**
/// Pedir um lado absurdo dá a malha cheia da CASA — `LADO_MAX_DE_GRELHA²`, nunca mais. O
/// controlo: um pedido modesto é obedecido ao número (o corte não come o que cabe).
#[test]
fn the_mesh_never_passes_the_per_node_ceiling() {
    use ph2d_nodegraph::node::{CELULAS_DE_UMA_GRELHA_CHEIA, MAX_INSTANCIAS_POR_NO};
    let cheia = semeia(1e9, 1e9).count();
    assert_eq!(
        cheia, CELULAS_DE_UMA_GRELHA_CHEIA,
        "o extremo e' a grelha cheia"
    );
    assert!(cheia <= MAX_INSTANCIAS_POR_NO);
    assert_eq!(
        semeia(40.0, 7.0).count(),
        280,
        "CONTROLO: o que cabe e' obedecido"
    );
}

/// ⭐ **O corpo declara o repouso e quem ele é** (doc 124 §0.2): `rest` é a malha de repouso e
/// `body` é o MESMO número não nulo em toda partícula.
#[test]
fn the_body_declares_its_rest_and_its_identity() {
    use ph2d_nodegraph::attr::Column;
    let s = semeia(3.0, 4.0);
    let Some(Column::Vec2(rest)) = s.get(super::REST_COL) else {
        panic!("o corpo publica o repouso")
    };
    assert_eq!(
        *rest,
        rest_shape(3, 4, 0.7),
        "o repouso e' a malha autorada"
    );
    let Some(Column::Scalar(body)) = s.get(super::BODY_COL) else {
        panic!("o corpo publica a identidade")
    };
    assert!(body.iter().all(|b| *b == body[0] && *b != 0.0), "{body:?}");
}

/// **The cap's whole justification is that the cost is LINEAR** — shape
/// matching reduces the cloud to 8 scalars, so there is no `N²` anywhere.
/// Asserted as a RATIO, never wall-clock: `ci-test` builds at `opt-level=1`
/// and a millisecond bar would measure the PROFILE (the ADR-0124 lesson).
/// Someone who made this quadratic would take the cap from 71% of the HR-4
/// budget to 45× it, and every unit test would stay green.
#[test]
fn the_shape_match_is_linear_in_the_mesh() {
    let cost = |side: usize| {
        let rest = rest_shape(side, side, 1.0);
        let pred: Vec<[f32; 2]> = rest.iter().map(|p| [p[0] * 1.1, p[1] * 0.9]).collect();
        let t0 = std::time::Instant::now();
        for _ in 0..5 {
            std::hint::black_box(shape_goals(&pred, &rest, 0.3, 1.0));
        }
        t0.elapsed().as_secs_f64()
    };
    let small = cost(128);
    let big = cost(256); // 4x the particles
    let ratio = big / small.max(1e-9);
    assert!(
        ratio <= 8.0,
        "4x the mesh must cost ~4x, not {ratio:.1}x — the cap assumes linear"
    );
}
