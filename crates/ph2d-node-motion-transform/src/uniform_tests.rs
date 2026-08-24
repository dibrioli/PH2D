//! Os gates do [`super::UNIFORM`] — **o layout espalha-se mais num eixo que no outro** (doc 89
//! folha 05), e o **flip** que uma escala negativa faz.
//!
//! ⚠️ **A fixtura tem de ter os dois eixos DIFERENTES.** Um layout quadrado é o caso em que
//! `sx` e `sy` dão a mesma figura por simetria — ele serviria de controle e nunca de prova.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte com quatro pontos num RETÂNGULO deitado — largura 4, altura 2.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.transform.uniform.src"),
    name: "motion.transform.uniform.src",
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
        ctx.emit(Stream::new(4).with(
            "P",
            Column::Vec2(vec![[-2.0, -1.0], [2.0, -1.0], [2.0, 1.0], [-2.0, 1.0]]),
        ));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionTransform),
            _ => None,
        }
    }
}

fn run(setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId)) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.transform.uniform.src");
    let xf = g.add_node("motion.transform");
    g.connect(Edge {
        from: (src, 0),
        to: (xf, 0),
        delayed: false,
    })
    .expect("in");
    setup(&mut g, xf);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, xf, 0.0).expect("cozinha")[0]
        .as_stream()
        .get("P")
        .expect("P")
    {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P e' Vec2"),
    }
}

/// A largura e a altura da figura.
fn extents(p: &[[f32; 2]]) -> (f32, f32) {
    let f = |a: usize| {
        p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
            (lo.min(q[a]), hi.max(q[a]))
        })
    };
    let (x, y) = (f(0), f(1));
    (x.1 - x.0, y.1 - y.0)
}

/// ⭐ **O DEFAULT É O NÓ QUE SHIPOU, AO BIT** — e o link nasce LIGADO, então um `scale_y`
/// autorado por engano não muda nada até alguém desligar a corrente.
#[test]
fn the_linked_default_is_the_law_that_shipped_bit_for_bit() {
    for k in [2.0_f32, 0.35, 1.0, -1.0] {
        let now = run(|g, n| g.set_param(n, "scale", k));
        // A lei de sempre, escrita à mão — ⚠️ **incluindo a MISTURA DA MÁSCARA**, que a
        // primeira versão deste oráculo esqueceu e que custou 1 ULP: `p + (full − p)·1` NÃO é
        // `full` em IEEE-754 (com `p = −2` e `full = −0,7` a subtracção cai num empate e o
        // arredondamento-para-par escolhe o outro lado). *Um oráculo que para no meio da lei
        // acusa o produto de um erro que é dele.*
        for (i, q) in now.iter().enumerate() {
            let src = [[-2.0_f32, -1.0], [2.0, -1.0], [2.0, 1.0], [-2.0, 1.0]][i];
            let full = [src[0] * k + 0.0, src[1] * k + 0.0];
            let want = [
                src[0] + (full[0] - src[0]) * 1.0,
                src[1] + (full[1] - src[1]) * 1.0,
            ];
            assert_eq!(
                (q[0].to_bits(), q[1].to_bits()),
                (want[0].to_bits(), want[1].to_bits()),
                "escala {k}, elemento {i}: {q:?} contra {want:?}"
            );
        }
    }
    // E um `scale_y` autorado é INERTE enquanto o link estiver ligado.
    let linked = run(|g, n| {
        g.set_param(n, "scale", 2.0);
        g.set_param(n, SCALE_Y, 5.0);
    });
    let plain = run(|g, n| g.set_param(n, "scale", 2.0));
    for (a, b) in linked.iter().zip(&plain) {
        assert_eq!(
            (a[0].to_bits(), a[1].to_bits()),
            (b[0].to_bits(), b[1].to_bits()),
            "o link LIGADO tem de ignorar o scale_y"
        );
    }
}

/// ⭐ **Desligado, o layout espalha-se mais num eixo que no outro** — a célula, literalmente.
#[test]
fn unlinked_the_layout_spreads_more_on_one_axis_than_the_other() {
    let (w0, h0) = extents(&run(|_, _| {}));
    let (w, h) = extents(&run(|g, n| {
        g.set_param(n, "scale", 3.0);
        g.set_param(n, UNIFORM, 0.0);
        g.set_param(n, SCALE_Y, 1.0);
    }));
    assert!(
        (w - w0 * 3.0).abs() < 1e-4,
        "o X tinha de triplicar: {w0} -> {w}"
    );
    assert!(
        (h - h0).abs() < 1e-4,
        "e o Y tinha de ficar quieto: {h0} -> {h}"
    );
}

/// ⭐ **O FLIP é uma escala negativa, e ele espelha só o eixo que a leva.**
#[test]
fn a_negative_factor_flips_the_axis_that_carries_it() {
    let flipped = run(|g, n| {
        g.set_param(n, UNIFORM, 0.0);
        g.set_param(n, SCALE_Y, -1.0);
    });
    let plain = run(|_, _| {});
    for (i, (a, b)) in flipped.iter().zip(&plain).enumerate() {
        assert!((a[0] - b[0]).abs() < 1e-5, "o X nao se mexe: elemento {i}");
        assert!(
            (a[1] + b[1]).abs() < 1e-5,
            "e o Y espelha: elemento {i}, {a:?} contra {b:?}"
        );
    }
    // CONTROLE: a figura não é a mesma (um retângulo simétrico em y espelharia para si próprio
    // como CONJUNTO, mas elemento a elemento os y's trocam de sinal — que é o que se mede).
    assert!(
        flipped.iter().any(|q| q[1] > 0.0) && flipped.iter().any(|q| q[1] < 0.0),
        "CONTROLE: a figura continua a ter os dois lados"
    );
}

/// **O PIVÔ dobra por EIXO** — sem isto, um pivô com escala não-uniforme puxaria a figura para
/// o lado, porque a dobra usaria um fator só para os dois eixos.
#[test]
fn the_pivot_folds_per_axis() {
    // Pivô no ponto `(2, 1)` — um canto. Escalando só o X por 3 em torno dele, aquele canto
    // tem de ficar EXACTAMENTE onde estava.
    let out = run(|g, n| {
        g.set_param(n, "scale", 3.0);
        g.set_param(n, UNIFORM, 0.0);
        g.set_param(n, SCALE_Y, 1.0);
        g.set_param(n, "pivot_mode", 1.0);
        g.set_param(n, "pivot_x", 2.0);
        g.set_param(n, "pivot_y", 1.0);
    });
    let corner = out[2]; // o `[2.0, 1.0]` da fonte
    assert!(
        (corner[0] - 2.0).abs() < 1e-4 && (corner[1] - 1.0).abs() < 1e-4,
        "o canto no pivo tinha de ficar parado, e foi para {corner:?}"
    );
}

/// **O device declara os dois params novos** — sem isto o kernel lê `params.uniform` e o WGSL
/// nem compila (ou pior: lê um slot alheio).
#[test]
fn the_kernel_declares_the_link_it_reads() {
    for p in [UNIFORM, SCALE_Y] {
        assert!(
            GPU_KERNEL.params.contains(&p),
            "o kernel le' `params.{p}` e nao o declara"
        );
    }
    assert!(
        GPU_KERNEL.wgsl.contains("xf_sy"),
        "e corre a MESMA lei da CPU"
    );
}

/// **O `scale_y` só aparece com o link desligado, e o piso digitável desce até ao flip.**
#[test]
fn the_panel_hides_the_second_axis_while_the_chain_is_linked() {
    assert!(
        PARAM_GATES
            .iter()
            .any(|g| g.param == SCALE_Y && g.when == UNIFORM && g.values == [0]),
        "o `scale_y` tem de estar gateado ao link"
    );
    for p in [UNIFORM, SCALE_Y] {
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == p),
            "`{p}` sem hint de painel"
        );
    }
    // E o flip é DIGITÁVEL: o piso desce abaixo do curso do slider.
    for p in ["scale", SCALE_Y] {
        let floor = PARAM_HARD_MIN
            .iter()
            .find(|m| m.param == p)
            .unwrap_or_else(|| panic!("`{p}` sem piso digitavel"));
        let hint = PARAM_HINTS.iter().find(|h| h.param == p).expect("hint");
        assert!(
            floor.min < hint.min,
            "um piso que nao desce abaixo do slider nao alarga nada ({} vs {})",
            floor.min,
            hint.min
        );
    }
}
