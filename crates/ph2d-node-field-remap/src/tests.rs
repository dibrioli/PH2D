//! Tests for `field.remap` — the remap pipeline (invert → inner-offset → contour →
//! range/multiplier → clamp → strength), the contour transfers, and the D12 neutrals.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

// A source that emits a KNOWN `falloff` column (the mask to remap) + a dummy `P`.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.remap.test.src"),
    name: "field.remap.test.src",
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
struct Src(Vec<f32>);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let n = self.0.len();
        ctx.emit(
            Stream::new(n)
                .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
                .with("falloff", Column::Scalar(self.0.clone())),
        );
    }
}

// A source with NO `falloff` column at all (the absent-mask case).
struct BareSrc(usize);
impl NodeOp for BareSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0).with("P", Column::Vec2(vec![[0.0, 0.0]; self.0])));
    }
}

enum AnySrc {
    Falloff(Src),
    Bare(BareSrc),
}
struct Ops {
    src: AnySrc,
}
impl Ops {
    fn falloff(vals: Vec<f32>) -> Self {
        Ops {
            src: AnySrc::Falloff(Src(vals)),
        }
    }
    fn bare(n: usize) -> Self {
        Ops {
            src: AnySrc::Bare(BareSrc(n)),
        }
    }
}
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == MANIFEST.id {
            return Some(&FieldRemap);
        }
        if ty == SRC_MAN.id {
            return Some(match &self.src {
                AnySrc::Falloff(s) => s as &dyn NodeOp,
                AnySrc::Bare(s) => s as &dyn NodeOp,
            });
        }
        None
    }
}

fn falloff_of(g: &Graph, ops: &Ops, target: NodeId) -> Vec<f32> {
    let mut cook = Cook::new();
    let out = cook.cook(g, ops, target, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("falloff must be a Scalar column"),
    }
}

fn chain() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("field.remap.test.src");
    let rm = g.add_node("field.remap");
    g.connect(Edge {
        from: (src, 0),
        to: (rm, 0),
        delayed: false,
    })
    .unwrap();
    (g, rm)
}

/// Set the node to the LINEAR-identity config (contour None, full range, no clamp
/// surprises) so a single knob under test is isolated from the visible-Quadratic default.
fn linear(g: &mut Graph, rm: NodeId) {
    g.set_param(rm, "contour", 0.0); // None
}

#[test]
fn strength_zero_is_an_exact_passthrough() {
    // The D12 neutral: whatever the other knobs say, strength 0 returns the input mask
    // BYTE-for-byte. Set every knob to a NON-default so a leak would show.
    let (mut g, rm) = chain();
    g.set_param(rm, "contour", 3.0); // Quantize
    g.set_param(rm, "inner_offset", 0.4);
    g.set_param(rm, "min", 0.2);
    g.set_param(rm, "max", 0.7);
    g.set_param(rm, "multiplier", 1.5);
    g.set_param(rm, "invert", 1.0);
    g.set_param(rm, "strength", 0.0);
    let input = vec![0.0, 0.137, 0.5, 0.813, 1.0];
    let ops = Ops::falloff(input.clone());
    assert_eq!(
        falloff_of(&g, &ops, rm),
        input,
        "strength 0 must be a passthrough"
    );
}

#[test]
fn none_contour_full_range_is_the_identity() {
    // contour None + [0,1] range + multiplier 1 + strength 1 + no invert = identity.
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    let input = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    let ops = Ops::falloff(input.clone());
    let got = falloff_of(&g, &ops, rm);
    for (a, b) in got.iter().zip(&input) {
        assert!(approx(*a, *b), "identity: {a} vs {b}");
    }
}

#[test]
fn quadratic_contour_eases_in_and_out() {
    let (mut g, rm) = chain();
    // Full ease-in (t²): 0.5 → 0.25.
    g.set_param(rm, "curvature", 1.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.5]), rm);
    assert!(approx(got[0], 0.25), "ease-in: {}", got[0]);
    // Full ease-out (1-(1-t)²): 0.5 → 0.75.
    g.set_param(rm, "curvature", -1.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.5]), rm);
    assert!(approx(got[0], 0.75), "ease-out: {}", got[0]);
    // Curvature 0 is linear even in Quadratic mode.
    g.set_param(rm, "curvature", 0.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.5]), rm);
    assert!(approx(got[0], 0.5), "curvature 0 = linear: {}", got[0]);
}

#[test]
fn step_is_a_floor_staircase() {
    // Step (2), 4 levels: floor(t·4) mapped to {0, 1/3, 2/3, 1}. Hits both 0 and 1.
    let (mut g, rm) = chain();
    g.set_param(rm, "contour", 2.0);
    g.set_param(rm, "steps", 4.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.1, 0.2, 0.3, 0.6, 0.9, 1.0]), rm);
    let third = 1.0 / 3.0;
    assert!(approx(got[0], 0.0), "0.1 → 0: {}", got[0]);
    // 🔴 0.2: floor(0.8)=0 → 0. A ROUND (Quantize) would give round(0.6)=1 → 1/3, so this
    // is the value that tells Step from Quantize — the phenomenon the fixture must contain.
    assert!(
        approx(got[1], 0.0),
        "0.2 → 0 (floor, not round): {}",
        got[1]
    );
    assert!(approx(got[2], third), "0.3 → 1/3: {}", got[2]);
    assert!(approx(got[3], 2.0 * third), "0.6 → 2/3: {}", got[3]);
    assert!(approx(got[4], 1.0), "0.9 → 1: {}", got[4]);
    assert!(approx(got[5], 1.0), "1.0 → 1: {}", got[5]);
}

#[test]
fn quantize_rounds_to_the_nearest_level() {
    // Quantize (3), 4 levels {0, 1/3, 2/3, 1}: round(t·3)/3.
    let (mut g, rm) = chain();
    g.set_param(rm, "contour", 3.0);
    g.set_param(rm, "steps", 4.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.1, 0.2, 0.5, 0.9]), rm);
    let third = 1.0 / 3.0;
    // round(0.3)=0 → 0; round(1.5)=2 → 2/3; round(2.7)=3 → 1.
    assert!(approx(got[0], 0.0), "0.1 → 0: {}", got[0]);
    // 🔴 0.2: round(0.6)=1 → 1/3. A FLOOR (Step) would give floor(0.8)=0 → 0, so this is
    // the value that tells Quantize from Step.
    assert!(
        approx(got[1], third),
        "0.2 → 1/3 (round, not floor): {}",
        got[1]
    );
    assert!(approx(got[2], 2.0 * third), "0.5 → 2/3: {}", got[2]);
    assert!(approx(got[3], 1.0), "0.9 → 1: {}", got[3]);
}

#[test]
fn inner_offset_expands_the_solid_core() {
    // inner_offset 0.5 ⇒ input ≥ 0.5 saturates to 1; 0.25 maps to 0.5.
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "inner_offset", 0.5);
    let got = falloff_of(&g, &Ops::falloff(vec![0.25, 0.5, 0.75]), rm);
    assert!(approx(got[0], 0.5), "0.25 → 0.5: {}", got[0]);
    assert!(approx(got[1], 1.0), "0.5 → 1: {}", got[1]);
    assert!(approx(got[2], 1.0), "0.75 → 1: {}", got[2]);
}

#[test]
fn min_max_remaps_the_output_range() {
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "min", 0.2);
    g.set_param(rm, "max", 0.8);
    let got = falloff_of(&g, &Ops::falloff(vec![0.0, 0.5, 1.0]), rm);
    assert!(approx(got[0], 0.2), "0 → min: {}", got[0]);
    assert!(approx(got[1], 0.5), "0.5 → mid: {}", got[1]);
    assert!(approx(got[2], 0.8), "1 → max: {}", got[2]);
}

#[test]
fn multiplier_scales_then_clamp_bounds() {
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "multiplier", 2.0);
    // clamp default ON: 0.3·2 = 0.6; 0.5·2 = 1.0 (clamped, not 1.0 overshoot).
    let got = falloff_of(&g, &Ops::falloff(vec![0.3, 0.5, 0.8]), rm);
    assert!(approx(got[0], 0.6), "0.3·2: {}", got[0]);
    assert!(approx(got[1], 1.0), "0.5·2 clamped: {}", got[1]);
    assert!(approx(got[2], 1.0), "0.8·2 clamped: {}", got[2]);
    // clamp OFF: the overshoot survives.
    g.set_param(rm, "clamp", 0.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.8]), rm);
    assert!(approx(got[0], 1.6), "0.8·2 unclamped: {}", got[0]);
}

#[test]
fn invert_flips_the_input() {
    // invert + None contour = 1 − t (a transfer, so 0.8 → 0.2, not 0.8·something).
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "invert", 1.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.3, 0.8]), rm);
    assert!(approx(got[0], 0.7), "0.3 → 0.7: {}", got[0]);
    assert!(approx(got[1], 0.2), "0.8 → 0.2: {}", got[1]);
}

#[test]
fn strength_blends_input_to_remapped() {
    // strength 0.5, invert (remapped = 1−t): out = t + (1−t − t)·0.5 = 0.5 at any t.
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "invert", 1.0);
    g.set_param(rm, "strength", 0.5);
    let got = falloff_of(&g, &Ops::falloff(vec![0.2, 0.4, 0.9]), rm);
    for v in got {
        assert!(approx(v, 0.5), "half-blend to the crossover: {v}");
    }
}

#[test]
fn absent_falloff_reads_as_full_effect() {
    // No `falloff` column ⇒ the input mask is 1.0 (full effect, the identity). Under the
    // None contour + invert that is 1 − 1 = 0.
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "invert", 1.0);
    let ops = Ops::bare(3);
    let got = falloff_of(&g, &ops, rm);
    assert_eq!(got, vec![0.0, 0.0, 0.0], "absent mask = 1, inverted = 0");
}

#[test]
fn remap_replaces_the_mask_it_does_not_multiply() {
    // A field MULTIPLIES its mask into `falloff`; a remap REPLACES it. invert of 0.8 is
    // 0.2 — if this node multiplied like a field, it would be 0.8·0.2 = 0.16.
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "invert", 1.0);
    let got = falloff_of(&g, &Ops::falloff(vec![0.8]), rm);
    assert!(approx(got[0], 0.2), "replace, not multiply: {}", got[0]);
}

#[test]
fn probability_one_keeps_everyone() {
    // The neutral: probability 1 (default) ⇒ the gate is 1 for every instance (the hash
    // is always < 1), so with a passthrough remap the mask is unchanged.
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    let input = vec![0.2, 0.5, 0.8, 1.0];
    let ops = Ops::falloff(input.clone());
    assert_eq!(falloff_of(&g, &ops, rm), input);
}

#[test]
fn probability_gates_by_a_stable_hash() {
    // probability 0.5 keeps ~half the instances (gate 1) and ZEROES the rest — a binary
    // mask, and deterministic (the SAME instances every cook, from the index hash).
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "probability", 0.5);
    let ops = Ops::falloff(vec![1.0; 1000]);
    let a = falloff_of(&g, &ops, rm);
    let b = falloff_of(&g, &ops, rm);
    assert_eq!(a, b, "the selection is deterministic");
    assert!(a.iter().all(|&v| v == 0.0 || v == 1.0), "binary mask");
    let kept = a.iter().filter(|&&v| v == 1.0).count();
    assert!((400..=600).contains(&kept), "keeps ~half: {kept}");
}

#[test]
fn seed_changes_which_instances_survive() {
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "probability", 0.5);
    let ops = Ops::falloff(vec![1.0; 500]);
    let a = falloff_of(&g, &ops, rm);
    g.set_param(rm, "seed", 12345.0);
    let b = falloff_of(&g, &ops, rm);
    assert_ne!(a, b, "a different seed selects different instances");
}

#[test]
fn hash01_is_in_range_and_varies() {
    // The spine of the gate: a stable per-index hash in [0,1), varying by id AND seed.
    for id in 0..100u32 {
        let h = hash01(id, 0);
        assert!((0.0..1.0).contains(&h), "hash01({id}) = {h} out of [0,1)");
    }
    assert_ne!(hash01(0, 0), hash01(1, 0), "adjacent ids differ");
    assert_ne!(hash01(5, 0), hash01(5, 1), "seed changes the hash");
}

#[test]
fn round_haz_matches_rust_round() {
    for x in [-2.5, -0.5, 0.0, 0.5, 1.5, 2.5, 2.4, 2.6] {
        assert_eq!(round_haz(x), x.round(), "round_haz({x})");
    }
}

#[test]
fn contour_none_and_empty_curve_pass_through() {
    // None (0) is the identity, and Curve (4) with NO authored curve is too — an
    // unset text param is a passthrough (the documented neutral).
    for mode in [0, 4] {
        for t in [0.0, 0.3, 0.7, 1.0] {
            assert!(
                approx(contour(mode, t, 0.7, 5.0, None, 0.0), t),
                "mode {mode} at {t}"
            );
        }
    }
}

#[test]
fn curve_contour_applies_the_authored_shape() {
    // An inverting curve (0->1, 1->0): the Curve contour must evaluate it, not pass
    // through. This is the unit-level red-first — mode 4 was the identity until A1.
    let inv = ph2d_curve::Curve {
        points: vec![
            ph2d_curve::Point {
                x: 0.0,
                y: 1.0,
                interp: ph2d_curve::Interp::Linear,
            },
            ph2d_curve::Point {
                x: 1.0,
                y: 0.0,
                interp: ph2d_curve::Interp::Linear,
            },
        ],
    };
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(
            approx(contour(4, t, 0.0, 4.0, Some(&inv), 0.0), 1.0 - t),
            "curve contour at {t} must invert"
        );
    }
}

#[test]
fn a_curve_contour_changes_the_cooked_falloff() {
    // Product-level proof: contour = Curve + an inverting curve in the text param
    // rewrites the mask through the WHOLE cook. Was the identity before A1 (red-first:
    // without the wiring the output would equal the input).
    let (mut g, rm) = chain();
    g.set_param(rm, "contour", 4.0); // Curve
    g.set_text_param(rm, CURVE_KEY, "c1 0:1:L 1:0:L".to_string()); // invert
    let input = vec![0.0, 0.25, 1.0];
    let ops = Ops::falloff(input.clone());
    let out = falloff_of(&g, &ops, rm);
    let want = [1.0, 0.75, 0.0]; // 1 - t
    for (got, exp) in out.iter().zip(want) {
        assert!(
            approx(*got, exp),
            "curve-remapped falloff {out:?} != {want:?}"
        );
    }
    assert_ne!(
        out, input,
        "the curve must change the mask (not a passthrough)"
    );
}

#[test]
fn every_contour_mode_including_the_curve_cooks_on_the_gpu() {
    // A1-gpu: no `applicable` gate — the Curve contour (mode 4) samples the LUT on the
    // device, so every mode is device-resident and the sequencer never falls back to
    // the CPU for this node. Before A1-gpu it declined mode 4 (the `oscillator`
    // precedent); the LUT channel dissolved that boundary.
    assert!(
        GPU_KERNEL.applicable.is_none(),
        "no CPU fallback: the LUT makes every contour mode device-resident"
    );
    // The LUT it registers carries the SAME text key the CPU `eval` reads, and its name
    // is what makes the WGSL accessor `rm_curve_sample`.
    assert_eq!(LUTS.len(), 1, "one LUT: the Curve contour's shape");
    assert_eq!(LUTS[0].text_key, CURVE_KEY, "same key the CPU eval parses");
    assert_eq!(
        LUTS[0].name, "rm_curve",
        "the accessor is `rm_curve_sample`"
    );
    assert!(
        LUTS[0].resolution >= 2,
        "at least two samples to lerp between"
    );
}

#[test]
fn fill_curve_lut_samples_the_curve_and_falls_back_to_identity() {
    // The node-side half of the LUT channel — the CI-runnable proof, since the GPU
    // parity gate is `#[ignore]`. An unset or malformed string is the identity ramp
    // (the SAME passthrough the CPU takes on `None`); a valid curve is `eval` at the
    // sample points.
    let mut buf = [0.0f32; 5];
    for bad in ["", "not a curve", "c1 garbage"] {
        fill_curve_lut(bad, &mut buf);
        for (k, v) in buf.iter().enumerate() {
            let t = k as f32 / 4.0;
            assert!(
                (v - t).abs() < 1e-6,
                "an unauthored/malformed curve is the identity ramp; {bad:?} at {k}: {v} != {t}"
            );
        }
    }
    // A tent (0 -> 1 -> 0): each sample equals `Curve::eval`, and the mid-sample peaks.
    let tent = ph2d_curve::parse("c1 0:0:L 0.5:1:L 1:0:L").expect("valid tent");
    fill_curve_lut("c1 0:0:L 0.5:1:L 1:0:L", &mut buf);
    for (k, v) in buf.iter().enumerate() {
        let t = k as f32 / 4.0;
        assert!((v - tent.eval(t)).abs() < 1e-6, "tent sample {k}: {v}");
    }
    assert!((buf[2] - 1.0).abs() < 1e-6, "the tent peaks at t = 0.5");
}

// ─────────────────────────────────────────────────────────────────────────────
// Doc 89 folha 10 — o `curve_offset` (C4D *Spline Offset* / *Spline Animation
// Speed*). Ver [`shifted`] para o mecanismo e o porquê da guarda do zero.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma curva ASSIMÉTRICA — a rampa `t`, cujos extremos NÃO concordam (`0 ≠ 1`).
///
/// ⚠️ Uma curva simétrica (ou a identidade) tornaria os gates abaixo vácuos: deslocar
/// uma forma que se repete não muda nada, e um gate que não vê diferença nenhuma
/// passaria com o `curve_offset` inteiramente desligado.
fn ramp() -> Curve {
    Curve::identity()
}

/// **O ZERO É A IDENTIDADE, E O TOPO DO INTERVALO É ONDE ISSO SE MEDE.**
///
/// ⚠️ **`t = 1.0` não é um caso de canto: é o que TODA peça a máscara cheia entrega.**
/// O wrap natural (`x − floor(x)`) leva `1.0` a `0.0`, então sem a guarda ligar o nó
/// sem tocar no knob trocaria `curve(1)` por `curve(0)` em metade da cena. Este gate é
/// a guarda, e ele testa exactamente o ponto onde ela morde.
#[test]
fn a_zero_offset_is_the_identity_including_at_the_very_top() {
    let c = ramp();
    for t in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        assert_eq!(shifted(t, 0.0), t, "sem deslocamento, `t` sai intacto");
        assert_eq!(
            contour(4, t, 0.0, 4.0, Some(&c), 0.0),
            c.eval(t),
            "e a contour lê a curva no mesmo sítio de sempre"
        );
    }
    // O CONTROLE: com um deslocamento, o topo JÁ NÃO é o topo — é o que prova que a
    // igualdade acima é da guarda e não de a função ser constante.
    assert_ne!(
        shifted(1.0, 0.25),
        1.0,
        "com deslocamento o wrap age, e é por isso que o zero precisa da guarda"
    );
}

/// **O DESLOCAMENTO ANDA COM A FORMA, E DÁ A VOLTA.**
#[test]
fn the_offset_slides_the_curve_and_wraps_around() {
    // 0.25 adiante: o que estava em 0.5 passa a ser lido em 0.25.
    assert!((shifted(0.25, 0.25) - 0.5).abs() < 1e-6);
    // E o que passa do fim reentra pelo começo.
    assert!((shifted(0.9, 0.25) - 0.15).abs() < 1e-6);
}

/// **UMA VOLTA INTEIRA É A IDENTIDADE (a menos do épsilon do float), e é isso que faz
/// a faixa `−1..1` do slider ser o percurso COMPLETO** — nada além dela é alcançável.
///
/// ⚠️ O `1e-6` aqui não é preguiça: `t + 1.0` perde bits baixos de `t`, então a volta
/// não fecha ao bit. O que o gate afirma é a periodicidade, não a exactidão.
#[test]
fn one_whole_turn_lands_back_where_it_started() {
    for t in [0.1_f32, 0.37, 0.62, 0.99] {
        assert!(
            (shifted(t, 1.0) - t).abs() < 1e-6,
            "uma volta em {t} devia voltar a {t}, deu {}",
            shifted(t, 1.0)
        );
        // …e meia volta duas vezes é uma volta.
        let half = shifted(t, 0.5);
        assert!((shifted(half, 0.5) - t).abs() < 1e-6);
    }
}

/// **UM DESLOCAMENTO NEGATIVO ANDA PARA TRÁS, e não sai do intervalo.**
///
/// ⚠️ `x − floor(x)` é o `rem_euclid`, não o `%`: em Rust `(-0.25) % 1.0` é `-0.25`,
/// que como argumento de uma curva é fora do domínio.
#[test]
fn a_negative_offset_walks_backwards_and_stays_inside() {
    for t in [0.0_f32, 0.1, 0.5, 1.0] {
        let u = shifted(t, -0.25);
        assert!((0.0..=1.0).contains(&u), "t={t} saiu do intervalo: {u}");
    }
    assert!(
        (shifted(0.1, -0.25) - 0.85).abs() < 1e-6,
        "reentra pelo fim"
    );
}

/// **SÓ O MODO `Curve` OUVE O DESLOCAMENTO** — nos outros quatro ele é inerte, porque
/// ali não há tabela a deslizar: são fórmulas.
///
/// FALSIFICADO se o offset entrasse antes do `match` — as quatro formas mudariam de
/// sítio, e o knob que a folha pediu para a curva estaria a torcer o Step.
#[test]
fn only_the_curve_contour_listens_to_the_offset() {
    for mode in [0, 1, 2, 3] {
        for t in [0.0_f32, 0.3, 0.7, 1.0] {
            assert_eq!(
                contour(mode, t, 0.7, 5.0, None, 0.0),
                contour(mode, t, 0.7, 5.0, None, 0.37),
                "o modo {mode} não pode mexer-se com o offset"
            );
        }
    }
    // O CONTROLE: no modo Curve ele mexe-se mesmo.
    let c = ramp();
    assert_ne!(
        contour(4, 0.3, 0.0, 4.0, Some(&c), 0.0),
        contour(4, 0.3, 0.0, 4.0, Some(&c), 0.37),
        "e no modo Curve o offset TEM de mudar a resposta"
    );
}

/// **O KERNEL CARREGA O PARAM E A LEI, E A GUARDA DO ZERO ESTÁ NO DEVICE TAMBÉM.**
///
/// ⚠️ Sem a guarda no WGSL, o device e a CPU discordariam **em toda peça a máscara
/// cheia** com o knob em zero — a divergência mais cara possível, porque acontece no
/// estado default e ninguém a procuraria ali.
#[test]
fn the_kernel_carries_the_offset_and_the_same_zero_guard() {
    assert!(
        GPU_KERNEL.params.contains(&"curve_offset"),
        "o uniforme tem de carregar o deslocamento: {:?}",
        GPU_KERNEL.params
    );
    assert!(
        GPU_KERNEL.wgsl.contains("params.curve_offset"),
        "e o corpo tem de o passar à contour"
    );
    assert!(
        GPU_KERNEL
            .wgsl_lib
            .contains("if (offset == 0.0) { return t; }"),
        "a guarda do zero tem de existir no device como existe na CPU"
    );
    assert!(
        GPU_KERNEL
            .wgsl_lib
            .contains("rm_curve_sample(rm_shifted(t, offset))"),
        "e o deslocamento tem de entrar SÓ no braço da curva"
    );
}
