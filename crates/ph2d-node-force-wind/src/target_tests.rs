//! Os gates do modo **`Target Velocity`** (doc 89, folha 02).

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

const N: usize = 6;

/// Um stream com velocidades escolhidas: parado, no alvo, e acima dele.
fn source(vel: &[[f32; 2]]) -> Stream {
    let n = vel.len();
    Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
        .with("vel", Column::Vec2(vel.to_vec()))
}

/// Coza o vento sobre `vel` e devolva o `accel` que ele acumulou.
fn accel_for(vel: &[[f32; 2]], params: &[(&str, f32)]) -> Vec<[f32; 2]> {
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.wind.target.test.src"),
        name: "force.wind.target.test.src",
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
    struct Src(Stream);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(self.0.clone());
        }
    }
    struct Reg(Src, ForceWind);
    impl OpResolver for Reg {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == SRC.id {
                Some(&self.0 as &dyn NodeOp)
            } else if ty == MANIFEST.id {
                Some(&self.1 as &dyn NodeOp)
            } else {
                None
            }
        }
    }
    let reg = Reg(Src(source(vel)), ForceWind);
    let mut g = Graph::new();
    let s = g.add_node(SRC.name);
    let w = g.add_node(MANIFEST.name);
    // Sem rajada: o que se mede é a lei do modo, não o ruído.
    g.set_param(w, "gust", 0.0);
    for (k, v) in params {
        g.set_param(w, *k, *v);
    }
    g.connect(Edge {
        from: (s, 0),
        to: (w, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, w, 0.0).expect("coze");
    match out[0].as_stream().get("accel") {
        Some(Column::Vec2(a)) => a.clone(),
        _ => Vec::new(),
    }
}

/// ⭐ **O MODO `Force` É O NÓ DE SEMPRE, AO BIT** — a velocidade nem é lida.
#[test]
fn the_force_mode_is_the_old_node_bit_for_bit() {
    let still = vec![[0.0_f32, 0.0]; N];
    let fast = vec![[9.0_f32, -4.0]; N];
    let base = &[("strength", 3.0_f32), ("angle", 0.0)];
    let a = accel_for(&still, base);
    let b = accel_for(&fast, base);
    assert_eq!(a.len(), N);
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert_eq!(
            x.map(f32::to_bits),
            y.map(f32::to_bits),
            "no modo Force a velocidade NAO entra na conta ({i})"
        );
        assert!((x[0] - 3.0).abs() < 1e-5, "sopra +X na forca pedida: {x:?}");
    }
}

/// ⭐⭐⭐ **O `Target Velocity` SATURA** — quem já anda à velocidade do vento não recebe
/// nada, e quem anda mais depressa é TRAVADO.
///
/// ⚠️ É a metade que uma aceleração constante não sabe fazer: ela acelera para sempre, e
/// uma folha ao vento não faz isso. A referência descreve-o com a mesma palavra —
/// *«satura: só acelera até a partícula igualar o vento»*.
#[test]
fn the_target_velocity_saturates_and_even_brakes() {
    let vel = vec![
        [0.0_f32, 0.0], // parado: empurrado com tudo
        [1.5, 0.0],     // a meio caminho
        [3.0, 0.0],     // exactamente no alvo
        [6.0, 0.0],     // acima do alvo: travado
    ];
    let a = accel_for(
        &vel,
        &[
            ("strength", 3.0),
            ("angle", 0.0),
            (MODE, 1.0),
            (AIR_RESIST, 2.0),
        ],
    );
    assert!((a[0][0] - 6.0).abs() < 1e-4, "parado: {a:?}");
    assert!((a[1][0] - 3.0).abs() < 1e-4, "a meio: {a:?}");
    assert!(a[2][0].abs() < 1e-5, "NO alvo a aceleracao e' ZERO: {a:?}");
    assert!(a[3][0] < -1e-4, "acima do alvo ele TRAVA: {a:?}");
}

/// ⚠️ **A resistência escala a APROXIMAÇÃO, não o alvo** — dobrá-la faz chegar mais
/// depressa ao mesmo sítio, e não ir para um sítio diferente.
#[test]
fn the_resistance_scales_how_fast_not_where_to() {
    let vel = vec![[0.0_f32, 0.0]];
    let one = accel_for(
        &vel,
        &[
            ("strength", 3.0),
            ("angle", 0.0),
            (MODE, 1.0),
            (AIR_RESIST, 1.0),
        ],
    );
    let two = accel_for(
        &vel,
        &[
            ("strength", 3.0),
            ("angle", 0.0),
            (MODE, 1.0),
            (AIR_RESIST, 2.0),
        ],
    );
    assert!(
        (two[0][0] - 2.0 * one[0][0]).abs() < 1e-4,
        "{one:?} {two:?}"
    );
    // E o ponto fixo NÃO se mexe: quem está no alvo continua a receber zero.
    let at = accel_for(
        &[[3.0, 0.0]],
        &[
            ("strength", 3.0),
            ("angle", 0.0),
            (MODE, 1.0),
            (AIR_RESIST, 9.0),
        ],
    );
    assert!(
        at[0][0].abs() < 1e-5,
        "o alvo e' o mesmo em toda resistencia"
    );
}

/// ⚠️ **Uma resistência de `0` não é «sem vento», é «nunca chega»** — e um valor negativo
/// não inverte a lei, ele é aparado.
///
/// ⚠️ **O `NaN` NÃO entra por aqui, e isso é um facto sobre o substrato que vale registar:**
/// o `Graph::set_param` tem um `debug_assert!(value.is_finite())`, então um override
/// autorado **não pode** ser não-finito. A única porta por onde um `NaN` chega a um param é
/// o FIO (um `value.*` a conduzi-lo), que este arnês não tem — e é contra essa porta que os
/// guardas `is_finite` deste nó e do `Profile` do irmão existem. *Um guarda testado pela
/// porta errada mede o guarda da porta errada.*
#[test]
fn a_zero_or_negative_resistance_never_poisons_the_accel() {
    for r in [0.0_f32, -5.0] {
        let a = accel_for(
            &[[0.0, 0.0], [3.0, 0.0]],
            &[("strength", 3.0), (MODE, 1.0), (AIR_RESIST, r)],
        );
        for (i, v) in a.iter().enumerate() {
            assert!(v[0].is_finite() && v[1].is_finite(), "r={r} [{i}] = {v:?}");
        }
    }
}

/// O param novo é declarado, com hint, e a resistência só aparece no modo que a lê.
#[test]
fn the_mode_is_declared_and_the_resistance_is_gated_to_it() {
    assert_eq!(MANIFEST.param_default(MODE), Some(0.0));
    assert!(PARAM_HINTS.iter().any(|h| h.param == MODE));
    let g = MODE_GATES
        .iter()
        .find(|g| g.param == AIR_RESIST)
        .expect("a resistencia e' gateada");
    assert_eq!(g.when, MODE);
    assert_eq!(g.values, &[1]);
}
