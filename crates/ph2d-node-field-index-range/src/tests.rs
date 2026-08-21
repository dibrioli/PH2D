//! Os gates do `field.index_range`.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

// Source: 11 instances (ordinals s = 0.0, 0.1, …, 1.0). Positions are inert
// here — this field never reads them — but the stream must carry a column so
// the count is real.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.index_range.test.src"),
    name: "field.index_range.test.src",
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
struct Src(usize);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let n = self.0;
        let p = par_build(n, |i| [i as f32, 0.0]);
        ctx.emit(Stream::new(n).with("P", Column::Vec2(p)));
    }
}
// Holds the source so the resolver hands out a `&self`-lifetime op keyed by
// the count of THIS `Ops` — no shared static (which would pin the first n).
struct Ops {
    src: Src,
}
impl Ops {
    fn new(n: usize) -> Self {
        Ops { src: Src(n) }
    }
}
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&self.src),
            t if t == MANIFEST.id => Some(&FieldIndexRange),
            _ => None,
        }
    }
}

/// ⚠️ Genérico no resolver **de propósito**: os gates do posto trazem um resolver
/// próprio (a fonte de valor), e um parâmetro concreto obrigaria a uma segunda
/// cópia desta função — que é como duas leituras da mesma saída divergem.
fn falloff_of<R: OpResolver>(g: &Graph, ops: &R, target: NodeId) -> Vec<f32> {
    let mut cook = Cook::new();
    let out = cook.cook(g, ops, target, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("falloff must be a Scalar column"),
    }
}

/// A ramp value is inherently f32-inexact (`(0.3 − 0.25)/0.1` computes to
/// `0.5000001`, not `0.5`), so the mask SHAPE is asserted within a tolerance.
/// The neutral/passthrough tests below stay `assert_eq!` on purpose — an
/// identity that is off must be off AT THE BIT (D12), which is a stronger claim.
fn assert_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len(), "length");
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert!((a - e).abs() < 1e-5, "at {i}: {a} vs {e}");
    }
}

fn chain() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("field.index_range.test.src");
    let foc = g.add_node("field.index_range");
    g.connect(Edge {
        from: (src, 0),
        to: (foc, 0),
        delayed: false,
    })
    .unwrap();
    (g, foc)
}

#[test]
fn default_middle_band_is_a_clean_trapezoid() {
    let (mut g, foc) = chain();
    // Linear curve so the ramp math is transparent; defaults start .25/end .75/soft .1.
    g.set_param(foc, "curve", 0.0);
    // s: .0 .1 .2 .3 .4 .5 .6 .7 .8 .9 1.0 — band [.25,.75], ramp width .1:
    // rise 0 until .25, 1 by .35; fall 1 until .65, 0 by .75.
    assert_close(
        &falloff_of(&g, &Ops::new(11), foc),
        &[0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 0.5, 0.0, 0.0, 0.0],
    );
}

#[test]
fn full_range_no_softness_is_the_identity() {
    // The neutral: start 0, end 1, soft 0 ⇒ mask 1 everywhere ⇒ the falloff
    // column is multiplied by the identity (D12 — off is exactly off).
    let (mut g, foc) = chain();
    g.set_param(foc, "start", 0.0);
    g.set_param(foc, "end", 1.0);
    g.set_param(foc, "soft", 0.0);
    assert_eq!(falloff_of(&g, &Ops::new(7), foc), vec![1.0; 7]);
}

#[test]
fn invert_flips_the_mask() {
    let (mut g, foc) = chain();
    g.set_param(foc, "curve", 0.0);
    g.set_param(foc, "invert", 1.0);
    // 1 − trapezoid.
    assert_close(
        &falloff_of(&g, &Ops::new(11), foc),
        &[1.0, 1.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0],
    );
}

#[test]
fn start_after_end_auto_swaps() {
    // Dragging Start past End yields the SAME band as End..Start — the mask is
    // a function of the interval, not of which handle names which bound.
    let a = band_mask(0.5, 0.25, 0.75, 0.1, 0);
    let b = band_mask(0.5, 0.75, 0.25, 0.1, 0);
    assert_eq!(a, b);
    assert_eq!(a, 1.0);
}

#[test]
fn a_prior_falloff_column_is_multiplied_not_overwritten() {
    // Fields COMPOSE multiplicatively (the MOPs contract): a carried `falloff`
    // is scaled by this band, never replaced.
    static FSRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.index_range.test.fsrc"),
        name: "field.index_range.test.fsrc",
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
    struct FSrc;
    impl NodeOp for FSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &FSRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // 3 instances, prior falloff [0.5, 0.9, 0.4].
            ctx.emit(
                Stream::new(3)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
                    .with("falloff", Column::Scalar(vec![0.5, 0.9, 0.4])),
            );
        }
    }
    struct FOps;
    impl OpResolver for FOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == FSRC_MAN.id => Some(&FSrc),
                t if t == MANIFEST.id => Some(&FieldIndexRange),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("field.index_range.test.fsrc");
    let foc = g.add_node("field.index_range");
    g.connect(Edge {
        from: (src, 0),
        to: (foc, 0),
        delayed: false,
    })
    .unwrap();
    // 3 instances ⇒ s = 0.0, 0.5, 1.0. Full range so mask = 1 everywhere ⇒
    // the carried column survives unchanged.
    g.set_param(foc, "start", 0.0);
    g.set_param(foc, "end", 1.0);
    g.set_param(foc, "soft", 0.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &FOps, foc, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.9, 0.4]),
        _ => panic!("falloff"),
    }
}

#[test]
fn curves_are_monotone_and_endpoint_exact() {
    for k in 0..=3 {
        assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
        assert_eq!(curve(k, 1.0), 1.0, "curve {k} at 1");
    }
    assert_eq!(curve(0, 0.5), 0.5); // Linear
    assert_eq!(curve(1, 0.5), 0.25); // Quad
    assert_eq!(curve(2, 0.5), 0.5); // Smoothstep symmetric
    assert!((curve(3, 0.5) - 0.5).abs() < 1e-6); // Smootherstep symmetric
}

#[test]
fn degenerate_empty_band_masks_almost_everything() {
    // start == end ⇒ the interval has no width ⇒ the mask is 0 everywhere
    // except the single ordinal exactly on the point.
    assert_eq!(band_mask(0.3, 0.5, 0.5, 0.1, 2), 0.0);
    assert_eq!(band_mask(0.5, 0.5, 0.5, 0.1, 2), 1.0); // exactly on it
    assert_eq!(band_mask(0.7, 0.5, 0.5, 0.1, 2), 0.0);
}

#[test]
fn single_element_reads_the_band_start_ordinal() {
    // n == 1 ⇒ s = 0 (no division). With the default band starting at .25 the
    // lone element sits below it ⇒ mask 0; a band containing 0 lights it.
    let (g, foc) = chain();
    assert_eq!(falloff_of(&g, &Ops::new(1), foc), vec![0.0]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Doc 89 folha 10 — o POSTO POR ATRIBUTO (`key = Attribute`). Ver [`KEY`] para o
// mecanismo, a medição que deixou o `Auto Range` de fora, e a conta que justifica
// a recusa do device.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma fonte de VALOR de `n` linhas com os valores escritos à mão.
static ATTR_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.index_range.test.attr"),
    name: "field.index_range.test.attr",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct AttrSrc(Vec<f32>);
impl NodeOp for AttrSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &ATTR_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
    }
}
struct RankOps {
    src: Src,
    attr: AttrSrc,
}
impl OpResolver for RankOps {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&self.src),
            t if t == ATTR_MAN.id => Some(&self.attr),
            t if t == MANIFEST.id => Some(&FieldIndexRange),
            _ => None,
        }
    }
}

/// Cozinha `src(n) → index_range` com uma banda DURA de meia lista (`soft = 0`,
/// `curve = Linear`), e devolve a máscara. Com `attr = Some(v)` a porta é ligada e o
/// modo passa a `Attribute`.
fn cook_rank(values: Option<Vec<f32>>) -> Vec<f32> {
    let n = 5;
    let ops = RankOps {
        src: Src(n),
        attr: AttrSrc(values.clone().unwrap_or_default()),
    };
    let mut g = Graph::new();
    let src = g.add_node("field.index_range.test.src");
    let ir = g.add_node("field.index_range");
    // A banda dura da METADE DE BAIXO: `s <= 0.5` passa, o resto não. Com 5 peças os
    // ordinais são [0, .25, .5, .75, 1] ⇒ a máscara é [1,1,1,0,0] na ordem que o
    // ordinal impuser.
    g.set_param(ir, "start", 0.0);
    g.set_param(ir, "end", 0.5);
    g.set_param(ir, "soft", 0.0);
    g.set_param(ir, "curve", 0.0);
    g.connect(Edge {
        from: (src, 0),
        to: (ir, 0),
        delayed: false,
    })
    .unwrap();
    if values.is_some() {
        g.set_param(ir, "key", KEY_ATTRIBUTE);
        let a = g.add_node("field.index_range.test.attr");
        g.connect(Edge {
            from: (a, 0),
            to: (ir, 1),
            delayed: false,
        })
        .unwrap();
    }
    falloff_of(&g, &ops, ir)
}

/// **A BANDA SEGUE O POSTO NO ATRIBUTO, e o stream NÃO se mexe.**
///
/// O atributo é `[9, 1, 7, 3, 5]` — postos `[4, 0, 3, 1, 2]`, ordinais
/// `[1, 0, .75, .25, .5]`. A banda `[0, 0.5]` passa quem tem ordinal ≤ 0,5, ou seja
/// as peças **1, 3 e 4** — as três de menor valor, **nas posições originais delas**.
///
/// ⚠️ **É este `assert` que separa a cura da composição que já existia:** com um
/// `motion.sort` a montante a máscara sairia `[1,1,1,0,0]` (contígua) porque as peças
/// teriam trocado de lugar. Aqui ela sai ESPALHADA, que é a definição de
/// não-destrutivo.
#[test]
fn the_band_follows_the_attribute_rank_without_moving_the_stream() {
    let got = cook_rank(Some(vec![9.0, 1.0, 7.0, 3.0, 5.0]));
    assert_eq!(
        got,
        vec![0.0, 1.0, 0.0, 1.0, 1.0],
        "as três de MENOR valor passam, cada uma no lugar dela"
    );
    // O CONTROLE: sem atributo, a mesma banda é o prefixo contíguo de sempre.
    assert_eq!(cook_rank(None), vec![1.0, 1.0, 1.0, 0.0, 0.0]);
}

/// **A PORTA DESLIGADA CAI NO MODO `Index`, e não num campo de zeros.**
///
/// Um `key = Attribute` com nada ligado é o pedido incompleto do artista. Um campo de
/// zeros daria o MESMO número (todos empatados, desempate pelo índice) — mas por
/// acidente, e deixaria de dar no dia em que o desempate mudasse.
#[test]
fn attribute_mode_with_nothing_wired_is_the_index_mode() {
    assert_eq!(ordinals(4, &[]), vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]);
}

/// **EMPATES DESEMPATAM PELO ÍNDICE — sempre, em qualquer plataforma.**
///
/// ⚠️ Sem o desempate explícito, dois valores iguais receberiam postos numa ordem que
/// depende do algoritmo de ordenação, e o hash de replay deixaria de bater entre
/// máquinas. *Não se escolhe um desempate melhor: não se tem empate.*
#[test]
fn ties_break_by_index_so_the_answer_is_platform_stable() {
    assert_eq!(
        ordinals(4, &[5.0, 5.0, 5.0, 5.0]),
        vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0],
        "tudo empatado ⇒ a ordem do índice"
    );
    // Empate parcial: os dois `2.0` ficam nos postos 0 e 1, na ordem em que estão.
    assert_eq!(ordinals(3, &[2.0, 9.0, 2.0]), vec![0.0, 1.0, 0.5]);
}

/// **`NaN` E `-0.0` NÃO ROMPEM A ORDEM TOTAL** — é para isso que serve o `total_cmp`.
///
/// FALSIFICADO por um `partial_cmp().unwrap()`: um `NaN` vindo de uma divisão a
/// montante entraria em pânico, e `-0.0 < 0.0` daria uma ordenação inconsistente que
/// alguns algoritmos detectam e outros não.
#[test]
fn a_nan_or_a_negative_zero_still_yields_a_total_order() {
    let s = ordinals(4, &[f32::NAN, 0.0, -0.0, 1.0]);
    assert_eq!(s.len(), 4);
    let mut sorted = s.clone();
    sorted.sort_by(f32::total_cmp);
    assert_eq!(
        sorted,
        vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0],
        "os quatro postos existem, um por peça: {s:?}"
    );
}

/// **UM ELEMENTO SÓ NÃO DIVIDE POR ZERO** — nos dois modos.
#[test]
fn a_single_element_reads_zero_in_both_modes() {
    assert_eq!(ordinals(1, &[]), vec![0.0]);
    assert_eq!(ordinals(1, &[42.0]), vec![0.0]);
    assert_eq!(ordinals(0, &[]), Vec::<f32>::new());
}

/// **O MODO `Attribute` RECUSA O DEVICE, E O `Index` NÃO.**
///
/// ⚠️ As duas direcções, senão um `applicable` que devolvesse `false` sempre passaria
/// por vácuo — e teria posto na CPU um nó que hoje cozinha inteiro na placa.
#[test]
fn the_attribute_mode_refuses_the_device_and_the_default_does_not() {
    let f = GPU_KERNEL.applicable.expect("o kernel declara a recusa");
    assert!(f(&|_: &str| 0.0), "Index: o device continua a valer");
    assert!(
        !f(&|name: &str| if name == KEY { KEY_ATTRIBUTE } else { 0.0 }),
        "Attribute: um posto é uma ordenação global, o nó recua"
    );
}
