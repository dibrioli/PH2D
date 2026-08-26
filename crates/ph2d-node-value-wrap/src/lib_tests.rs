//! Gates de [`super`] — cortados para o irmao no teto de LOC (HR-18).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// **Clamp holds at the edges** — inside the range is identity, below `min`
/// reads `min`, above `max` reads `max`. The plateau either side.
#[test]
fn clamp_holds_at_the_edges() {
    let (lo, hi) = (0.0, 1.0);
    assert_eq!(
        wrap_one(0.3, lo, hi, Mode::Clamp),
        0.3,
        "inside is identity"
    );
    assert_eq!(
        wrap_one(-2.0, lo, hi, Mode::Clamp),
        0.0,
        "below pins to min"
    );
    assert_eq!(wrap_one(5.0, lo, hi, Mode::Clamp), 1.0, "above pins to max");
    // A non-`[0,1]` range clamps just the same.
    assert_eq!(
        wrap_one(9.0, -2.0, 3.0, Mode::Clamp),
        3.0,
        "clamps to max=3"
    );
}

/// **Repeat tiles the range into a sawtooth** — a value `w` past `min` reads
/// the same as `min`, and `max` itself wraps back to `min` (the half-open
/// `[min, max)` tile). Falsifiable: a Clamp implementation would pin the tail
/// to `max` instead of wrapping it.
#[test]
fn repeat_tiles_into_a_sawtooth() {
    let (lo, hi) = (0.0, 1.0); // width 1
    assert_eq!(
        wrap_one(0.3, lo, hi, Mode::Repeat),
        0.3,
        "inside is identity"
    );
    assert!(
        (wrap_one(1.3, lo, hi, Mode::Repeat) - 0.3).abs() < 1e-6,
        "1.3 wraps to 0.3"
    );
    assert!(
        (wrap_one(2.3, lo, hi, Mode::Repeat) - 0.3).abs() < 1e-6,
        "2.3 wraps to 0.3"
    );
    assert!(
        (wrap_one(-0.3, lo, hi, Mode::Repeat) - 0.7).abs() < 1e-6,
        "-0.3 wraps to 0.7"
    );
    assert_eq!(wrap_one(1.0, lo, hi, Mode::Repeat), 0.0, "max wraps to min");
    // A shifted range: [2, 5], width 3. 8 → 2, 6.5 → 3.5.
    assert!(
        (wrap_one(8.0, 2.0, 5.0, Mode::Repeat) - 2.0).abs() < 1e-6,
        "8 wraps into [2,5] as 2"
    );
    assert!(
        (wrap_one(6.5, 2.0, 5.0, Mode::Repeat) - 3.5).abs() < 1e-6,
        "6.5 -> 3.5"
    );
}

/// **Mirror folds back and forth into a triangle** — it rises to `max`, then
/// FALLS back to `min` over the next `w`, period `2w`. The distinguishing case
/// is `1.5·w` past `min`: Repeat reads `0.5·w` (rising), Mirror reads the
/// mirror `0.5·w` down from the top. Falsifiable against Repeat.
#[test]
fn mirror_folds_into_a_triangle() {
    let (lo, hi) = (0.0, 1.0); // width 1, period 2
    assert_eq!(wrap_one(0.0, lo, hi, Mode::Mirror), 0.0, "min");
    assert_eq!(wrap_one(1.0, lo, hi, Mode::Mirror), 1.0, "peak at max");
    // 1.3 is 0.3 into the falling half → 1 − 0.3 = 0.7 (Repeat would give 0.3).
    assert!(
        (wrap_one(1.3, lo, hi, Mode::Mirror) - 0.7).abs() < 1e-6,
        "1.3 folds to 0.7"
    );
    assert!(
        (wrap_one(1.7, lo, hi, Mode::Mirror) - 0.3).abs() < 1e-6,
        "1.7 folds to 0.3"
    );
    assert!(
        (wrap_one(2.0, lo, hi, Mode::Mirror)).abs() < 1e-6,
        "2.0 back to min (period 2)"
    );
    assert!(
        (wrap_one(2.3, lo, hi, Mode::Mirror) - 0.3).abs() < 1e-6,
        "2.3 rises again to 0.3"
    );
    // Negative side mirrors symmetrically: -0.3 folds up to 0.3.
    assert!(
        (wrap_one(-0.3, lo, hi, Mode::Mirror) - 0.3).abs() < 1e-6,
        "-0.3 folds to 0.3"
    );
}

/// **A degenerate range pins to `min`** — `max ≤ min` has no interval to fold
/// into, so every value collapses to `min`, finite, never a division by a
/// zero width. Falsifiable: an unguarded `r / w` would be `inf`/`NaN`.
#[test]
fn a_degenerate_range_pins_to_min_and_stays_finite() {
    for &m in &[Mode::Clamp, Mode::Repeat, Mode::Mirror] {
        for &v in &[-3.0f32, 0.0, 2.5, 100.0] {
            // Zero-width and inverted ranges both degenerate.
            assert_eq!(
                wrap_one(v, 0.5, 0.5, m),
                0.5,
                "zero width pins to lo ({m:?})"
            );
            assert_eq!(wrap_one(v, 1.0, 0.0, m), 1.0, "inverted pins to lo ({m:?})");
        }
    }
}

/// **The fold always lands in `[min, max]` and stays finite** for any value,
/// range and mode — the whole point of an address mode (a sampler never reads
/// off the texture). Repeat's half-open top is the only exclusion.
#[test]
fn the_result_is_finite_and_inside_the_range() {
    for &m in &[Mode::Clamp, Mode::Repeat, Mode::Mirror] {
        for &(lo, hi) in &[(0.0f32, 1.0), (-2.0, 3.0), (1.5, 1.6)] {
            for k in -200..200 {
                let v = k as f32 * 0.13;
                let o = wrap_one(v, lo, hi, m);
                assert!(o.is_finite(), "finite at v={v} [{lo},{hi}] {m:?}");
                // A tiny ε for the floor's fp reconstruction at the seam.
                assert!(
                    o >= lo - 1e-4 && o <= hi + 1e-4,
                    "in range at v={v} [{lo},{hi}] {m:?}: {o}"
                );
            }
        }
    }
}

/// A value source emitting a fixed field, so `value.wrap` can be driven
/// through a real cook (the whole-chain proof, not just the math).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.wrap.test.src"),
    name: "value.wrap.test.src",
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
struct Src(Vec<f32>);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
    }
}

/// End-to-end through the cook: a ramp that runs `[0, 2]` folded by Repeat
/// into `[0, 1]` becomes two copies of the tile, length preserved (the unary
/// contract).
#[test]
fn tiles_a_field_through_the_cook() {
    struct Ops(Vec<f32>);
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => {
                    Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                }
                t if t == MANIFEST.id => Some(&ValueWrap),
                _ => None,
            }
        }
    }
    // 0, 0.5, 1.0, 1.5 over [0,1] Repeat → 0, 0.5, 0, 0.5 (the tile repeats).
    let ops = Ops(vec![0.0, 0.5, 1.0, 1.5]);
    let mut g = Graph::new();
    let src = g.add_node("value.wrap.test.src");
    let w = g.add_node("value.wrap");
    g.set_param(w, "lo", 0.0);
    g.set_param(w, "hi", 1.0);
    g.set_param(w, "mode", 1.0); // Repeat
    g.connect(Edge {
        from: (src, 0),
        to: (w, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, w, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => {
            assert_eq!(v.len(), 4, "length preserved");
            assert!((v[0] - 0.0).abs() < 1e-6, "0 -> 0");
            assert!((v[1] - 0.5).abs() < 1e-6, "0.5 -> 0.5");
            assert!((v[2] - 0.0).abs() < 1e-6, "1.0 wraps to 0");
            assert!((v[3] - 0.5).abs() < 1e-6, "1.5 wraps to 0.5");
        }
        _ => panic!("v"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

// ── As portas de FAIXA (doc 89, folha 15 linha 69) ───────────────────────────────
macro_rules! range_src {
    ($man:ident, $ty:ident, $name:literal) => {
        static $man: NodeManifest = NodeManifest {
            id: NodeTypeId::of($name),
            name: $name,
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
        struct $ty(Vec<f32>);
        impl NodeOp for $ty {
            fn manifest(&self) -> &'static NodeManifest {
                &$man
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
            }
        }
    };
}
range_src!(LO_MAN, LoSrc, "value.wrap.test.lo");
range_src!(HI_MAN, HiSrc, "value.wrap.test.hi");

struct RangeOps(Vec<f32>, Vec<f32>, Vec<f32>);
impl OpResolver for RangeOps {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(Box::leak(Box::new(Src(self.0.clone())))),
            t if t == LO_MAN.id => Some(Box::leak(Box::new(LoSrc(self.1.clone())))),
            t if t == HI_MAN.id => Some(Box::leak(Box::new(HiSrc(self.2.clone())))),
            t if t == MANIFEST.id => Some(&ValueWrap),
            _ => None,
        }
    }
}

/// Coza `input` com as faixas dadas; um vetor VAZIO deixa a porta desligada.
fn fold_with(input: &[f32], lo: &[f32], hi: &[f32], param_lo: f32, param_hi: f32) -> Vec<f32> {
    let ops = RangeOps(input.to_vec(), lo.to_vec(), hi.to_vec());
    let mut g = Graph::new();
    let src = g.add_node(SRC_MAN.name);
    let w = g.add_node("value.wrap");
    g.set_param(w, "lo", param_lo);
    g.set_param(w, "hi", param_hi);
    g.set_param(w, "mode", 1.0); // Repeat
    let lo_node = (!lo.is_empty()).then(|| g.add_node(LO_MAN.name));
    let hi_node = (!hi.is_empty()).then(|| g.add_node(HI_MAN.name));
    for (from, to_port) in [
        Some((src, 0)),
        lo_node.map(|n| (n, 1)),
        hi_node.map(|n| (n, 2)),
    ]
    .into_iter()
    .flatten()
    {
        g.connect(Edge {
            from: (from, 0),
            to: (w, to_port),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, w, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("o wrap emite um escalar"),
    }
}

/// ⭐⭐ **A FAIXA POR-INSTÂNCIA** — e as três leituras do comprimento de uma porta.
///
/// ⚠️ **O CONTROLO é a porta desligada**: sem ele, um gate que só medisse a porta ligada
/// passaria mesmo que ela tivesse passado a ignorar o param — e toda cena já salva muda
/// de valor em silêncio.
#[test]
fn the_range_ports_make_the_fold_per_instance() {
    let ramp = [0.5_f32, 1.5, 2.5, 3.5];
    // CONTROLE: desligadas ⇒ os params de hoje, ao bit.
    let base = fold_with(&ramp, &[], &[], 0.0, 1.0);
    assert_eq!(
        base,
        vec![0.5, 0.5, 0.5, 0.5],
        "portas desligadas tem de dar exactamente o que os params davam"
    );
    // UM valor DIFUNDE: uma faixa `[0,2]` para o campo inteiro.
    let bcast = fold_with(&ramp, &[0.0], &[2.0], 0.0, 1.0);
    assert_eq!(
        bcast,
        vec![0.5, 1.5, 0.5, 1.5],
        "uma porta de UM valor vale para o campo todo"
    );
    // `n` valores: uma faixa DIFERENTE por elemento.
    //
    // ⚠️ **A fixtura é ENTRADAS IGUAIS com faixas diferentes**, e a escolha é o que faz o
    // controlo abaixo valer: com uma faixa uniforme, entradas iguais dão **forçosamente**
    // saídas iguais — logo qualquer resposta com valores distintos é inalcançável sem a
    // porta, por construção e não por sorte. ⛔ A 1.ª fixtura era uma RAMPA dentro de
    // `[0,4]`, onde dobrar é a identidade e a faixa uniforme dava o mesmo: *uma fixtura
    // só prova o que contém.*
    let same = [3.0_f32; 4];
    let per = fold_with(
        &same,
        &[0.0, 0.0, 0.0, 0.0],
        &[1.0, 2.0, 4.0, 8.0],
        0.0,
        1.0,
    );
    assert_eq!(
        per,
        vec![0.0, 1.0, 3.0, 3.0],
        "cada elemento dobra na SUA faixa: 3 mod [1,2,4,8]"
    );
    // ⛔ E a prova estrutural: entradas iguais + faixa uniforme = saídas iguais.
    for hi in [1.0_f32, 2.0, 4.0, 8.0] {
        let uniform = fold_with(&same, &[], &[], 0.0, hi);
        assert!(
            uniform.windows(2).all(|w| w[0] == w[1]),
            "faixa uniforme sobre entradas iguais tinha de dar saidas iguais: {uniform:?}"
        );
        assert_ne!(
            uniform, per,
            "se uma faixa uniforme `[0,{hi}]` desse o mesmo, a porta nao compraria nada"
        );
    }
}
