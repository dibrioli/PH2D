//! Os gates do EMBRULHO — a lei da cúbica, o falloff e a máscara.
//!
//! Saíram do `lib.rs` no teto de LOC, por assunto: o pai fica com **o que o nó É**
//! (o manifesto, a curva, o embrulho, o registro) e este irmão com **o que ele
//! promete**. Segue FILHO por `#[path]`, então `use super::*` alcança os privados.
use super::*;

/// A curva INTEIRA sem deslize -- o mapeamento que shipava.
const WHOLE: ArcMap = ArcMap {
    from: 0.0,
    to: 1.0,
    offset: 0.0,
};
const S_CURVE: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
const LINE: [P2; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
// A symmetric arch (hump) — its arc-midpoint lifts clearly off the endpoint chord,
// unlike the antisymmetric S-curve whose midpoint sits *on* the chord.
const ARCH: [P2; 4] = [[-3.0, 0.0], [-1.0, 3.0], [1.0, 3.0], [3.0, 0.0]];

/// `amount` 0 is the identity — the layout is untouched.
#[test]
fn amount_zero_is_the_identity() {
    let p = vec![[-2.0, 0.5], [0.0, -0.3], [2.0, 0.1]];
    let out = wrap(&p, &Curve::cubic(&S_CURVE), 1.0, WHOLE, false, &[0.0], &[]);
    for (o, q) in out.iter().zip(&p) {
        assert!(
            (o[0] - q[0]).abs() < 1e-5 && (o[1] - q[1]).abs() < 1e-5,
            "{o:?} vs {q:?}"
        );
    }
}

/// Wrapping onto a straight horizontal line keeps a straight input row straight (the
/// remap is affine there): three points at constant y stay collinear.
#[test]
fn a_row_on_a_straight_curve_stays_straight() {
    let p = vec![[-2.0, 0.4], [0.0, 0.4], [2.0, 0.4]];
    let out = wrap(&p, &Curve::cubic(&LINE), 1.0, WHOLE, false, &[1.0], &[]);
    // Constant normal (+y) ⇒ all share the same y; collinear.
    assert!((out[0][1] - out[1][1]).abs() < 1e-3 && (out[1][1] - out[2][1]).abs() < 1e-3);
}

/// Wrapping onto a curved spline BENDS a straight input row: the midpoint leaves the
/// chord between the endpoints. FALSIFIED by a flat deformer (midpoint on the chord).
#[test]
fn a_row_on_a_curved_spline_bends() {
    let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]]; // a straight row along x
    let out = wrap(&p, &Curve::cubic(&ARCH), 1.0, WHOLE, false, &[1.0], &[]);
    // Cross product of (mid−a) and (b−a): non-zero ⇒ the midpoint bent off the line.
    let (a, mid, b) = (out[0], out[1], out[2]);
    let cross = (mid[0] - a[0]) * (b[1] - a[1]) - (mid[1] - a[1]) * (b[0] - a[0]);
    assert!(cross.abs() > 0.5, "the row bent (cross {cross})");
}

/// Falloff masks the wrap per element: falloff 0 leaves an element where it was.
#[test]
fn falloff_masks_the_wrap() {
    let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]];
    let falloff = vec![1.0, 0.0, 1.0]; // middle element pinned
    let out = wrap(
        &p,
        &Curve::cubic(&S_CURVE),
        1.0,
        WHOLE,
        false,
        &[1.0],
        &falloff,
    );
    assert_eq!(out[1], p[1], "falloff 0 -> unchanged");
}

/// **O `amount` É UM CAMPO** — cada elemento embrulha pelo SEU valor, não pelo do elemento 0.
///
/// ⚠️ **Este gate nasceu de um defeito que não dava erro nenhum** (doc 90 §5): a porta é do
/// domínio `Instances`, e o gesto óbvio — ligar-lhe um `value.instance_field(Ramp)` para o
/// embrulho crescer ao longo do layout — entregava ao nó INTEIRO o elemento `0` da rampa, que é
/// exactamente `0.0`. O nó virava identidade e **os catorze knobs de geometria ficavam mudos ao
/// mesmo tempo**, sem log e sem erro.
///
/// O oráculo é **cada elemento contra a corrida UNIFORME que lhe corresponde**: o que tem
/// `amount = 0` tem de estar onde a corrida-identidade o põe, e o que tem `amount = 1` onde a
/// corrida-cheia o põe. Um `first()` daria `0` a todos e o segundo par morre.
///
/// ⚠️ **A primeira versão deste gate mediu o ÚLTIMO elemento contra a posição original e
/// reprovou sobre a cura correcta** — `moveu 0`. O último elemento senta em `u = 1`, e a ponta
/// da curva `ARCH` é `[3, 0]`, que é exactamente onde ele já estava: *a fixture não continha o
/// fenómeno no ponto em que eu o media*. O elemento do MEIO é o que sai da corda.
#[test]
fn the_amount_is_a_field_not_the_first_element() {
    let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]];
    let curve = Curve::cubic(&ARCH);
    let flat = wrap(&p, &curve, 1.0, WHOLE, false, &[0.0], &[]);
    let full = wrap(&p, &curve, 1.0, WHOLE, false, &[1.0], &[]);
    // CONTROLE: a fixture tem de conter o fenómeno no elemento que o gate mede.
    let span = (full[1][1] - flat[1][1]).abs();
    assert!(span > 0.5, "controle: o meio tem de sair da corda ({span})");

    // Primeiro elemento desligado, do meio para a frente ligado.
    let per_el = wrap(&p, &curve, 1.0, WHOLE, false, &[0.0, 1.0, 1.0], &[]);
    assert_eq!(
        per_el[0], flat[0],
        "o elemento 0 segue a corrida-identidade"
    );
    assert_eq!(per_el[1], full[1], "o elemento 1 segue a corrida-cheia");
}

/// **UM VALOR SÓ CONTINUA A VALER PARA TODOS** — a regra `1 → broadcast`, que é o que torna a
/// mudança acima byte-idêntica para toda cena que já existe.
///
/// ⚠️ Sem este par, a cura poderia ter partido o caso comum (uma porta ligada a um `value.lfo`,
/// que emite UM valor) e nenhum teste diria nada — *uma feature nova que quebra o caminho antigo
/// em silêncio é pior que a ausência dela*.
#[test]
fn a_single_value_still_speaks_for_every_element() {
    let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]];
    let one = wrap(&p, &Curve::cubic(&ARCH), 1.0, WHOLE, false, &[0.0], &[]);
    for (o, q) in one.iter().zip(&p) {
        assert_eq!(o, q, "um `0` desliga o embrulho de TODOS");
    }
    let full = wrap(&p, &Curve::cubic(&ARCH), 1.0, WHOLE, false, &[1.0], &[]);
    let flat = wrap(&p, &Curve::cubic(&ARCH), 1.0, WHOLE, false, &[], &[]);
    assert_eq!(
        full, flat,
        "desligada (vazia) e' o mesmo que `1` — o neutro"
    );
}

/// Deterministic + cooks through the registry, copying columns and wrapping P.
#[test]
fn registers_and_wraps_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.spline_wrap.test.src"),
        name: "motion.spline_wrap.test.src",
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
            ctx.emit(
                Stream::new(3)
                    .with("P", Column::Vec2(vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]]))
                    .with("size", Column::Vec2(vec![[0.3, 0.3]; 3])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionSplineWrap),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let src = g.add_node("motion.spline_wrap.test.src");
    let sw = g.add_node("motion.spline_wrap");
    g.connect(Edge {
        from: (src, 0),
        to: (sw, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, sw, 0.0).unwrap();
    let s = out[0].as_stream();
    assert!(s.get("size").is_some(), "columns pass through");
    match s.get("P").unwrap() {
        Column::Vec2(v) => {
            // The wrapped row is no longer flat on y = 0 (the S-curve lifted it).
            assert!(
                v.iter().any(|q| q[1].abs() > 0.3),
                "wrapped off the axis: {v:?}"
            );
        }
        _ => panic!("P"),
    }
}
