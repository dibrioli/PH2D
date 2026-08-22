//! Gates do `value.mix` — a lei do crossfade, os nove modos de blend e os DOIS
//! clamps (o do factor e o do resultado, que são grandezas diferentes).
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a casa já usa noutros nós (`value.pattern`): o `lib.rs` fica com a
//! LEI e o irmão com as PROVAS. Os caminhos não mudam — `#[path]` mantém o módulo
//! a chamar-se `tests` e `use super::*` resolve como sempre resolveu.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// **The bare node crossfades by the `factor` param** (t unconnected). At
/// factor 0 it is all `a`; at 1 all `b`; at 0.5 the midpoint. A regression that
/// ignored the factor (always 0, or always the port identity) would fail.
#[test]
fn the_factor_param_crossfades_when_t_is_unconnected() {
    let a = [2.0];
    let b = [10.0];
    let mix = |f: f32| {
        blend(
            &a,
            &b,
            &[],
            false,
            Blend {
                factor: f,
                clamp: true,
                mode: BlendMode::Mix,
                clamp_result: false,
            },
        )[0]
    };
    assert_eq!(mix(0.0), 2.0, "factor 0 = all a");
    assert_eq!(mix(1.0), 10.0, "factor 1 = all b");
    assert_eq!(mix(0.5), 6.0, "factor 0.5 = midpoint");
    assert_eq!(mix(0.25), 4.0, "quarter of the way from a to b");
}

/// **A connected `t` port OVERRIDES the factor** — the driver takes over. The
/// factor here is a decoy 0.9; the port's per-element `t` is what lands, so a
/// regression that read the param instead of the port would produce 9.2, not
/// the port's answers.
#[test]
fn a_connected_t_port_overrides_the_factor() {
    let a = [0.0, 0.0, 0.0];
    let b = [100.0, 100.0, 100.0];
    let t = [0.0, 0.5, 1.0];
    let out = blend(
        &a,
        &b,
        &t,
        true,
        Blend {
            factor: 0.9,
            clamp: true,
            mode: BlendMode::Mix,
            clamp_result: false,
        },
    );
    assert_eq!(out, vec![0.0, 50.0, 100.0], "the port drives the blend");
}

/// **`clamp` holds `t` in `[0,1]`; Off lets it overshoot.** With clamp on,
/// `t = 1.5` is pinned to `b`; with clamp off it extrapolates PAST `b`.
#[test]
fn clamp_pins_the_ends_and_off_overshoots() {
    let a = [0.0];
    let b = [10.0];
    // t = 1.5 (past b) and t = -0.5 (before a).
    assert_eq!(
        blend(
            &a,
            &b,
            &[1.5],
            true,
            Blend {
                factor: 0.0,
                clamp: true,
                mode: BlendMode::Mix,
                clamp_result: false
            }
        )[0],
        10.0,
        "clamped to b"
    );
    assert_eq!(
        blend(
            &a,
            &b,
            &[-0.5],
            true,
            Blend {
                factor: 0.0,
                clamp: true,
                mode: BlendMode::Mix,
                clamp_result: false
            }
        )[0],
        0.0,
        "clamped to a"
    );
    assert_eq!(
        blend(
            &a,
            &b,
            &[1.5],
            true,
            Blend {
                factor: 0.0,
                clamp: false,
                mode: BlendMode::Mix,
                clamp_result: false
            }
        )[0],
        15.0,
        "unclamped overshoots past b"
    );
    assert_eq!(
        blend(
            &a,
            &b,
            &[-0.5],
            true,
            Blend {
                factor: 0.0,
                clamp: false,
                mode: BlendMode::Mix,
                clamp_result: false
            }
        )[0],
        -5.0,
        "unclamped undershoots before a"
    );
}

/// **The `1→N` broadcast rule** (doc 12): a length-1 `a`/`b` is HELD across a
/// length-N `t`, so a per-element factor blends between two constants. Output
/// length is the `max` of the inputs.
#[test]
fn a_length_one_field_is_held_across_a_length_n_factor() {
    let a = [0.0]; // one constant, broadcast
    let b = [8.0]; // one constant, broadcast
    let t = [0.0, 0.25, 0.5, 0.75, 1.0];
    let out = blend(
        &a,
        &b,
        &t,
        true,
        Blend {
            factor: 0.5,
            clamp: true,
            mode: BlendMode::Mix,
            clamp_result: false,
        },
    );
    assert_eq!(out.len(), 5, "output is as wide as the widest input");
    assert_eq!(out, vec![0.0, 2.0, 4.0, 6.0, 8.0], "the ramp blends a→b");
}

/// Three DISTINCT value-source types, so the cook can feed `a`, `b`, and `t`
/// different fields (the `OpResolver` keys on the node TYPE, so three nodes of
/// one type would all emit the same field).
macro_rules! src {
    ($man:ident, $ty:ident, $id:literal, $field:expr) => {
        static $man: NodeManifest = NodeManifest {
            id: NodeTypeId::of($id),
            name: $id,
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
        struct $ty;
        impl NodeOp for $ty {
            fn manifest(&self) -> &'static NodeManifest {
                &$man
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let f: Vec<f32> = $field;
                ctx.emit(Stream::new(f.len()).with(VALUE_COL, Column::Scalar(f)));
            }
        }
    };
}
src!(SRC_A_MAN, SrcA, "value.mix.test.a", vec![0.0, 0.0, 0.0]);
src!(SRC_B_MAN, SrcB, "value.mix.test.b", vec![10.0, 20.0, 30.0]);
src!(SRC_T_MAN, SrcT, "value.mix.test.t", vec![0.0, 0.5, 1.0]);

/// End-to-end through the cook: `a = [0,0,0]`, `b = [10,20,30]`, `t = [0,0.5,1]`
/// blends to `[0, 10, 30]` — the factor reaches the output element-wise, the
/// port overrides the (decoy) factor param, and the length is preserved.
#[test]
fn blends_two_fields_through_the_cook() {
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == MANIFEST.id => Some(&ValueMix),
                t if t == SRC_A_MAN.id => Some(&SrcA),
                t if t == SRC_B_MAN.id => Some(&SrcB),
                t if t == SRC_T_MAN.id => Some(&SrcT),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let sa = g.add_node("value.mix.test.a");
    let sb = g.add_node("value.mix.test.b");
    let st = g.add_node("value.mix.test.t");
    let mix = g.add_node("value.mix");
    g.set_param(mix, "factor", 0.9); // a decoy — the connected `t` must win
    for (from, port) in [(sa, 0u16), (sb, 1), (st, 2)] {
        g.connect(Edge {
            from: (from, 0),
            to: (mix, port),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, mix, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![0.0, 10.0, 30.0], "blended per element"),
        _ => panic!("v"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

/// **`Mix` reduz LITERALMENTE ao crossfade que este no' sempre foi** -- o
/// default, byte a byte contra a expressao que shipava.
#[test]
fn the_mix_mode_is_the_old_crossfade_to_the_bit() {
    for k in 0..80 {
        let a = k as f32 * 0.031 - 0.5;
        let b = 1.0 - a * 0.7;
        let t = (k as f32 * 0.017) % 1.0;
        let now = {
            let vb = BlendMode::Mix.apply(a, b);
            a + t * (vb - a)
        };
        let before = a + t * (b - a);
        assert_eq!(now.to_bits(), before.to_bits(), "k={k}");
    }
}

/// ⚠️ **Nao e' uma segunda porta do `value.math`, e o FACTOR e' a diferenca.**
/// Com `t = 1` os dois coincidem -- e essa coincidencia e' a prova de que a
/// lei e' `lerp(a, f(a,b), t)` e nao uma copia; a meio caminho este no' diz o
/// que a aritmetica nao sabe dizer sem um segundo no' atras dela.
#[test]
fn a_blend_is_the_arithmetic_faded_not_the_arithmetic() {
    let (a, b) = (0.8_f32, 0.25_f32);
    let full = BlendMode::Multiply.apply(a, b);
    assert!((full - a * b).abs() < 1e-6, "com t=1 e' o produto: {full}");
    let half = a + 0.5 * (full - a);
    assert!((half - 0.5 * (a + a * b)).abs() < 1e-6, "half={half}");
    assert!(
        (half - a * b).abs() > 0.1,
        "meio caminho NAO e' o produto: {half} vs {}",
        a * b
    );
}

/// **Os nove modos respondem DIFERENTE na mesma entrada.** O gate que impede
/// um braco copiado (ou um `else if` que cai no vizinho) de passar
/// despercebido; a rampa cruza `0,5`, entao os DOIS ramos do Overlay correm.
#[test]
fn every_blend_mode_answers_differently() {
    const MODES: [BlendMode; 9] = [
        BlendMode::Mix,
        BlendMode::Add,
        BlendMode::Subtract,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Difference,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::Overlay,
    ];
    let sig: Vec<Vec<f32>> = MODES
        .iter()
        .map(|m| (0..=20).map(|k| m.apply(k as f32 / 20.0, 0.6)).collect())
        .collect();
    for i in 0..MODES.len() {
        for j in (i + 1)..MODES.len() {
            let d = sig[i]
                .iter()
                .zip(&sig[j])
                .map(|(x, y)| (x - y).abs())
                .fold(0.0f32, f32::max);
            assert!(
                d > 1e-3,
                "{:?} e {:?} respondem igual (max |d| = {d:e})",
                MODES[i],
                MODES[j]
            );
        }
    }
}

/// **O Overlay RAMIFICA em `a < 0,5`** -- a metade escura multiplica, a clara
/// faz screen. Um kernel que so' escrevesse um dos ramos passaria no gate de
/// distincao acima (ele ainda diferiria dos outros oito) e falha aqui.
#[test]
fn the_overlay_branches_at_the_midpoint() {
    let b = 0.6_f32;
    let dark = BlendMode::Overlay.apply(0.25, b);
    assert!(
        (dark - 2.0 * 0.25 * b).abs() < 1e-6,
        "ramo Multiply: {dark}"
    );
    let light = BlendMode::Overlay.apply(0.75, b);
    let want = 1.0 - 2.0 * (1.0 - 0.75) * (1.0 - b);
    assert!((light - want).abs() < 1e-6, "ramo Screen: {light}");
    assert!(dark < light, "os dois ramos nao podem colapsar");
}

/// **OS DOIS CLAMPS SAO GRANDEZAS DIFERENTES, e este gate e' o que os separa.**
///
/// ⚠️ **E' o gate que a conferencia teve de MEDIR** (folha 15 l.53): o param
/// que este no' sempre teve chama-se `clamp` e e' o *Clamp Factor* -- ele
/// segura o **peso**. O `clamp_result` segura a **saida**. A fixture escolhe o
/// caso em que so' um deles pode agir: `t = 0.5` (ja' dentro de `[0,1]`, entao
/// o Clamp Factor nao tem nada a fazer) com `Add` sobre `a = 0.8, b = 0.9`,
/// cuja soma transborda. Uma implementacao que tivesse ligado o botao novo ao
/// clamp do FACTOR passaria em qualquer teste de "o botao faz alguma coisa" e
/// reprovaria aqui.
#[test]
fn the_two_clamps_are_different_quantities() {
    let (a, b, t) = (vec![0.8], vec![0.9], vec![0.5]);
    let free = blend(
        &a,
        &b,
        &t,
        true,
        Blend {
            factor: 0.0,
            clamp: true,
            mode: BlendMode::Add,
            clamp_result: false,
        },
    );
    // Mix(0.8, Add(0.8,0.9)=1.7, t=0.5) = 0.8 + 0.5*0.9 = 1.25 -- transborda.
    assert!((free[0] - 1.25).abs() < 1e-6, "sem clamp: {}", free[0]);
    let held = blend(
        &a,
        &b,
        &t,
        true,
        Blend {
            factor: 0.0,
            clamp: true,
            mode: BlendMode::Add,
            clamp_result: true,
        },
    );
    assert_eq!(held[0], 1.0, "o clamp do RESULTADO segura em 1.0");
    // E o Clamp Factor, LIGADO nos dois casos, nao mudou nada -- ele nao e'
    // este botao.
    let no_factor_clamp = blend(
        &a,
        &b,
        &t,
        true,
        Blend {
            factor: 0.0,
            clamp: false,
            mode: BlendMode::Add,
            clamp_result: false,
        },
    );
    assert_eq!(
        no_factor_clamp[0], free[0],
        "o t=0.5 ja' estava na faixa: o Clamp Factor e' inerte aqui"
    );
}

/// **DESLIGADO, o `clamp_result` e' o no' que sempre shipou -- BIT-A-BIT**, nos
/// nove modos de blend e numa varredura de `t` que passa dos dois lados.
#[test]
fn clamp_result_off_is_byte_identical_to_the_node_that_shipped() {
    let a = vec![-0.4, 0.25, 0.8, 3.0];
    let b = vec![0.9, -1.2, 0.5, 0.1];
    for m in 0..=8 {
        let mode = BlendMode::from_param(m as f32);
        for k in -10..=20 {
            let t = vec![k as f32 * 0.1];
            let off = blend(
                &a,
                &b,
                &t,
                true,
                Blend {
                    factor: 0.0,
                    clamp: false,
                    mode,
                    clamp_result: false,
                },
            );
            // O oraculo e' a formula antiga, escrita aqui a' mao: `va + t*(vb-va)`.
            for (i, o) in off.iter().enumerate() {
                let va = a[i.min(a.len() - 1)];
                let vb = mode.apply(va, b[i.min(b.len() - 1)]);
                assert_eq!(*o, va + t[0] * (vb - va), "modo {m} t={} i={i}", t[0]);
            }
        }
    }
}

/// **Ligado, a saida NUNCA sai de `[0,1]`** -- para qualquer entrada, peso e
/// modo. E' a promessa inteira do botao, afirmada como propriedade e nao num
/// ponto.
#[test]
fn clamp_result_on_never_leaves_the_unit_range() {
    let a = vec![-9.0, -0.4, 0.25, 0.8, 3.0, 40.0];
    let b = vec![0.9, -1.2, 0.5, 0.1, -7.0, 2.0];
    for m in 0..=8 {
        let mode = BlendMode::from_param(m as f32);
        for k in -30..=30 {
            let t = vec![k as f32 * 0.17];
            for o in blend(
                &a,
                &b,
                &t,
                true,
                Blend {
                    factor: 0.0,
                    clamp: false,
                    mode,
                    clamp_result: true,
                },
            ) {
                assert!(o.is_finite() && (0.0..=1.0).contains(&o), "modo {m}: {o}");
            }
        }
    }
}
