//! **THE `blend` IS A FIELD, NOT A NUMBER** (doc 89 folha 08 — the P0).
//!
//! The defect lived in `eval`, not in `mix`: the reduction was always able to take
//! a per-element weight, and what threw it away was `v.first()` one line above.
//! So these gates drive the node through the **real cook** with a value producer
//! wired to the `blend` port — a gate that called `mix` directly would exercise
//! the half that was never broken and stay green over the bug.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

const fn stream_src(id: &'static str) -> NodeManifest {
    NodeManifest {
        id: NodeTypeId::of(id),
        name: id,
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[] as &[ParamSpec],
        lowerings: &[LoweringKind::Cpu],
    }
}

static SA: NodeManifest = stream_src("motion.mixer.field.a");
static SB: NodeManifest = stream_src("motion.mixer.field.b");

/// Four elements on a line at `y = 0`.
struct A;
impl NodeOp for A {
    fn manifest(&self) -> &'static NodeManifest {
        &SA
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(4).with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        ));
    }
}

/// The same four, ten units up — so the blended `y` reads off as the weight × 10.
struct B;
impl NodeOp for B {
    fn manifest(&self) -> &'static NodeManifest {
        &SB
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(4).with(
            "P",
            Column::Vec2(vec![[0.0, 10.0], [1.0, 10.0], [2.0, 10.0], [3.0, 10.0]]),
        ));
    }
}

/// A VALUE producer whose column length is whatever the fixture asks for — the
/// three shapes the broadcast rule distinguishes (absent / 1 / N) all come from
/// here, so no gate below can be green because it built the wrong shape.
static SV: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mixer.field.v"),
    name: "motion.mixer.field.v",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "n",
            default: 1.0,
        },
        ParamSpec {
            name: "step",
            default: 0.0,
        },
        ParamSpec {
            name: "base",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};
struct V;
impl NodeOp for V {
    fn manifest(&self) -> &'static NodeManifest {
        &SV
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let n = ctx.param("n").max(0.0) as usize;
        let (base, step) = (ctx.param("base"), ctx.param("step"));
        #[expect(clippy::cast_precision_loss, reason = "a tiny fixture index")]
        let v: Vec<f32> = (0..n).map(|i| base + step * i as f32).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SA.id => Some(&A),
            t if t == SB.id => Some(&B),
            t if t == SV.id => Some(&V),
            t if t == MANIFEST.id => Some(&MotionMixer),
            _ => None,
        }
    }
}

/// The blended `y` of the four elements. `field` is `None` for an unconnected
/// `blend` port, or `Some((len, base, step))` for a column of that length.
fn blended_y(field: Option<(usize, f32, f32)>) -> Vec<f32> {
    let mut g = Graph::new();
    let a = g.add_node("motion.mixer.field.a");
    let b = g.add_node("motion.mixer.field.b");
    let m = g.add_node("motion.mixer");
    g.set_param(m, "mode", MODE_BLEND as f32);
    for (src, port) in [(a, 0u16), (b, 1)] {
        g.connect(Edge {
            from: (src, 0),
            to: (m, port),
            delayed: false,
        })
        .expect("stream port");
    }
    if let Some((n, base, step)) = field {
        let v = g.add_node("motion.mixer.field.v");
        #[expect(clippy::cast_precision_loss, reason = "a tiny fixture length")]
        g.set_param(v, "n", n as f32);
        g.set_param(v, "base", base);
        g.set_param(v, "step", step);
        g.connect(Edge {
            from: (v, 0),
            to: (m, 4),
            delayed: false,
        })
        .expect("blend port");
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, m, 0.0).expect("cook");
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(p)) => p.iter().map(|q| q[1]).collect(),
        _ => panic!("the mixer always emits P"),
    }
}

/// **Each element blends on its own schedule** — the P0.
///
/// A length-4 field of `0, 0.25, 0.5, 0.75` has to produce four DIFFERENT heights.
/// Before this, `v.first()` handed element zero's `0.0` to the whole stream and the
/// four came out flat at `y = 0`: a per-element blend was inexpressible, and the
/// only way to see it was to look at the picture.
#[test]
fn a_length_n_field_blends_each_element_on_its_own_schedule() {
    let y = blended_y(Some((4, 0.0, 0.25)));
    let want = [0.0f32, 2.5, 5.0, 7.5];
    for (i, (got, w)) in y.iter().zip(want).enumerate() {
        assert!(
            (got - w).abs() < 1e-5,
            "element {i}: blend {} of the 10-unit gap is {w}, got {got} (all of {y:?})",
            0.25 * i as f32
        );
    }
}

/// **A length-1 field is HELD across the stream** — and this is the regression
/// guard, because it is the shape every document written before the fix has.
///
/// A `value.lfo` left unconnected produces exactly one value (its count law), so
/// the overwhelming majority of graphs out there hand this node a length-1 field.
/// They must be untouched.
#[test]
fn a_length_one_field_is_broadcast_and_the_old_answer_survives() {
    for w in [0.0f32, 0.3, 0.5, 1.0] {
        let y = blended_y(Some((1, w, 0.0)));
        assert!(
            y.iter().all(|g| (g - w * 10.0).abs() < 1e-5),
            "a single value holds across all four elements at blend {w}: {y:?}"
        );
    }
}

/// **Unconnected is the midpoint**, which is what the node has always said.
#[test]
fn an_unconnected_blend_is_the_midpoint() {
    let y = blended_y(None);
    assert!(
        y.iter().all(|g| (g - 5.0).abs() < 1e-5),
        "no field at all is the 0.5 the node documents: {y:?}"
    );
}

/// **`blend = 1` is `in1` to the BIT, per element** — the node's own doc promises
/// it, and the two-term form is what makes it true rather than nearly true.
///
/// The field here is length-N and constant, so this also proves the per-element
/// path did not trade exactness for the feature.
#[test]
fn the_endpoints_are_exact_element_by_element() {
    let all_b = blended_y(Some((4, 1.0, 0.0)));
    assert_eq!(
        all_b.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        vec![10.0f32.to_bits(); 4],
        "blend 1 is in1 bit for bit"
    );
    let all_a = blended_y(Some((4, 0.0, 0.0)));
    assert_eq!(
        all_a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        vec![0.0f32.to_bits(); 4],
        "blend 0 is in0 bit for bit"
    );
    // And a MIXED field keeps both ends exact while the middle interpolates —
    // the case a constant field cannot show.
    let mixed = blended_y(Some((4, 0.0, 1.0 / 3.0)));
    assert_eq!(mixed[0].to_bits(), 0.0f32.to_bits(), "row 0 is in0 exactly");
    assert_eq!(
        mixed[3].to_bits(),
        10.0f32.to_bits(),
        "row 3 is in1 exactly, even with the rows between it interpolating"
    );
}

/// **And the exactness is a fact about the FORM, not about the fixture.**
///
/// ⚠️ The gate above ran on `0` and `10`, where `a + (b − a)·t` and `a·(1−t) + b·t`
/// agree to the bit — so the mutation that swaps one for the other **survived it**.
/// The two forms only part company where `b − a` has to round, and then the
/// one-term form misses the endpoint by an ulp: `−3.3 → 7.1` at `t = 1` gives
/// `7.0999994`, not `7.1`. A gate whose fixture cannot distinguish two laws is not
/// testing either of them.
///
/// This one calls `lerp_col` directly, and that is the right split rather than a
/// shortcut: the wiring lives in `eval` and is covered through the cook above,
/// while THIS is arithmetic, and the arithmetic's own door is the function.
#[test]
fn the_two_term_form_is_what_makes_the_endpoint_exact() {
    let a = Column::Scalar(vec![-3.3, -3.3]);
    let b = Column::Scalar(vec![7.1, 7.1]);
    // A length-N field pinned at both ends: row 0 all `a`, row 1 all `b`.
    let out = lerp_col(&a, &b, &[0.0, 1.0], 2);
    let Column::Scalar(v) = out else {
        panic!("scalar in, scalar out")
    };
    assert_eq!(
        v[0].to_bits(),
        (-3.3f32).to_bits(),
        "t = 0 is `a` to the bit"
    );
    assert_eq!(
        v[1].to_bits(),
        7.1f32.to_bits(),
        "t = 1 is `b` to the bit -- the one-term form lands on 7.0999994 here"
    );
}

/// **The field reaches past `1`, and that is deliberate** — unlike a heading, a
/// layout thrown past the target one has a picture, so the mixer does not clamp
/// where `motion.morph` does. Pinned so nobody "harmonises" the two later.
#[test]
fn the_blend_is_not_clamped_and_the_sibling_that_clamps_is_a_different_question() {
    let over = blended_y(Some((1, 1.5, 0.0)));
    assert!(
        over.iter().all(|g| (g - 15.0).abs() < 1e-4),
        "blend 1.5 overshoots past in1, it is not pinned at it: {over:?}"
    );
}

/// Uma coluna escalar de teste.
fn scal(v: &[f32]) -> Column {
    Column::Scalar(v.to_vec())
}

/// Um snapshot de uma coluna só, com o nome dado.
fn snap_of(name: &str, c: Column, n: usize) -> Snap {
    Snap {
        count: n,
        cols: vec![(name.to_string(), c)],
    }
}

/// **OS QUATRO MODOS APENDADOS DOBRAM AS ENTRADAS — e `0..2` não se mexeram.**
///
/// ⚠️ A segunda metade é o gate: um documento autorado guarda o ÍNDICE, não o nome.
#[test]
fn the_appended_fold_modes_do_what_they_say_and_the_old_indices_did_not_move() {
    let a = snap_of("v", scal(&[6.0, 2.0]), 2);
    let b = snap_of("v", scal(&[2.0, 8.0]), 2);
    let got = |mode: i64| -> Vec<f32> {
        let out = mix(mode, &[&a, &b], &[], &[1.0, 1.0], None);
        match out.get("v").unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("v"),
        }
    };
    assert_eq!(got(MODE_AVG), vec![4.0, 5.0], "Avg (0) nao se mexeu");
    assert_eq!(got(MODE_ADD), vec![8.0, 10.0], "Add (1) nao se mexeu");
    assert_eq!(got(MODE_SUB), vec![4.0, -6.0], "Subtract (3)");
    assert_eq!(got(MODE_MUL), vec![12.0, 16.0], "Multiply (4)");
    assert_eq!(got(MODE_MIN), vec![2.0, 2.0], "Min (5)");
    assert_eq!(got(MODE_MAX), vec![6.0, 8.0], "Max (6)");
    // E o menu do painel oferece exactamente sete, na mesma ordem.
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    let hints = reg.param_ui(MANIFEST.id).expect("hints");
    let row = hints
        .iter()
        .find(|h| h.param == "mode")
        .expect("row do mode");
    let ParamWidget::Enum { labels } = row.widget else {
        panic!("enum")
    };
    assert_eq!(labels.len(), 7, "um rotulo por indice");
    assert_eq!(row.max, 6.0, "o teto do slider segue");
}

/// **COM UMA ENTRADA SÓ, UMA DOBRA É A IDENTIDADE** — a resposta certa, e a mesma
/// que os modos antigos dão.
#[test]
fn a_fold_over_one_input_is_the_identity() {
    let a = snap_of("v", scal(&[3.0, -7.0]), 2);
    for mode in [MODE_SUB, MODE_MUL, MODE_MIN, MODE_MAX] {
        let out = mix(mode, &[&a], &[], &[1.0], None);
        match out.get("v").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![3.0, -7.0], "modo {mode}"),
            _ => panic!("v"),
        }
    }
}

/// **A GEOMETRIA VEM DA LANE ESCOLHIDA — e o resto continua misturado.**
///
/// ⚠️ **É o gate que separa «escolher a geometria» de «escolher tudo».** Uma
/// implementação que tomasse a lane inteira passaria num teste que só olhasse `P`, e
/// o nó deixaria de ser um mixer.
#[test]
fn the_chosen_lane_gives_the_geometry_and_the_rest_stays_mixed() {
    let a = Snap {
        count: 1,
        cols: vec![
            ("P".to_string(), Column::Vec2(vec![[0.0, 0.0]])),
            ("geometry_id".to_string(), scal(&[1.0])),
            ("size".to_string(), scal(&[2.0])),
        ],
    };
    let b = Snap {
        count: 1,
        cols: vec![
            ("P".to_string(), Column::Vec2(vec![[10.0, 0.0]])),
            ("geometry_id".to_string(), scal(&[3.0])),
            ("size".to_string(), scal(&[6.0])),
        ],
    };
    // Sem escolha: a média de tudo — e o `geometry_id` sai `2`, uma TERCEIRA forma
    // que nenhuma das duas entradas tinha. É o perigo que o knob cura.
    let mixed = mix(MODE_AVG, &[&a, &b], &[], &[1.0, 1.0], None);
    match mixed.get("geometry_id").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![2.0], "a media inventa uma forma"),
        _ => panic!(),
    }
    // Com a lane `a` escolhida: a forma e a posição são as DELA…
    let picked = mix(MODE_AVG, &[&a, &b], &[], &[1.0, 1.0], Some(&a));
    match picked.get("geometry_id").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![1.0], "a forma vem da lane"),
        _ => panic!(),
    }
    match picked.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0]], "a posicao tambem"),
        _ => panic!(),
    }
    // …e o `size` continua MISTURADO — o nó não deixou de ser um mixer.
    match picked.get("size").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![4.0], "o size continua a media"),
        _ => panic!(),
    }
}
