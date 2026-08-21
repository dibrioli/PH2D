//! Os gates do [`super::CURVE`] — o perfil da borda (doc 89, folha 02).
//!
//! ⚠️ **A 1ª versão destes gates só testava a função `curve`, e a mutação que
//! NUNCA A APLICA sobreviveu.** Um perfil correcto que o `eval` ignora desenha o
//! mesmo vórtice de sempre — e era isso que a célula pedia para curar. Metade
//! destes gates coze o NÓ.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId, PortSpec};

/// UMA peça a meio raio do centro — onde os quatro perfis discordam.
///
/// ⚠️ **Nem no centro nem na borda:** ali `curve` é endpoint-exacto e os quatro
/// dariam o mesmo número (a lei paga na wave dos deformadores).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.vortex.test.one"),
    name: "force.vortex.test.one",
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
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[1.0, 0.0]])));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&ForceVortex),
            _ => None,
        }
    }
}

/// Coze `src → force.vortex(radius = 2, curve)` e devolve o `accel` da peça (que
/// está a `d = 1`, ou seja `t = 0,5` — o meio, onde dois perfis se fixam e dois não).
fn swirl(curve_kind: f32) -> [f32; 2] {
    let mut g = Graph::new();
    let s = g.add_node("force.vortex.test.one");
    let v = g.add_node("force.vortex");
    g.connect(Edge {
        from: (s, 0),
        to: (v, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(v, "radius", 2.0);
    g.set_param(v, "strength", 4.0);
    g.set_param(v, CURVE, curve_kind);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, v, 0.0).unwrap()[0]
        .as_stream()
        .get("accel")
    {
        Some(Column::Vec2(x)) => x[0],
        other => panic!("accel: {other:?}"),
    }
}

/// **O PERFIL CHEGA AO `eval`** — e é este o gate que a 1ª leva não tinha.
///
/// ⚠️ A peça está a `t = 0,5`. O Linear dá `0,5`; o Quad dá `0,25`. Se o `eval`
/// ignorasse o knob, os dois dariam o mesmo vetor.
#[test]
fn the_profile_reaches_the_force_and_not_only_the_helper() {
    let linear = swirl(0.0);
    let quad = swirl(1.0);
    assert_ne!(
        linear, quad,
        "o knob tem de morder o vórtice, não só a função"
    );
    // E a razão é a das duas rampas em `t = 0,5`: `0,25 / 0,5` = ½.
    assert!(
        (quad[1] / linear[1] - 0.5).abs() < 1e-5,
        "{quad:?} contra {linear:?}"
    );
}

/// **O LINEAR É O VÓRTICE QUE SEMPRE SHIPOU, AO BIT** — pelo nó, não pela função.
#[test]
fn linear_is_the_vortex_that_shipped() {
    // `strength · (1 − d/R) · falloff / d` com `d = 1`, `R = 2`, `strength = 4`
    // ⇒ `mag = 2`. ⚠️ O default de `clockwise` é LIGADO, então o vetor é
    // `(dy, −dx)·mag` = `(0, −2)` — o gate lê o nó, não a minha expectativa (a 1ª
    // versão dizia `+2` e o nó estava certo).
    assert_eq!(swirl(0.0), [0.0, -2.0]);
}

/// **O LINEAR É A RAMPA CRAVADA DE SEMPRE, AO BIT** — `curve(0, t)` é o próprio `t`.
#[test]
fn linear_is_the_ramp_that_shipped() {
    for t in [0.0f32, 0.17, 0.5, 0.83, 1.0] {
        assert_eq!(curve(0, t), t, "o Linear não pode tocar no número");
    }
}

/// **OS QUATRO SÃO ENDPOINT-EXACTOS** — a mesma lei do `force.attractor`.
///
/// ⚠️ Sem isto um perfil poderia mudar a FORÇA no centro (onde `t = 1`) em vez de só
/// a forma da queda, e o artista veria o vórtice mudar de intensidade ao trocar de
/// dropdown — que é o defeito que separa um perfil de um segundo `strength`.
#[test]
fn every_profile_is_endpoint_exact() {
    for k in 0..4 {
        assert_eq!(curve(k, 0.0), 0.0, "perfil {k} na borda");
        assert!((curve(k, 1.0) - 1.0).abs() < 1e-6, "perfil {k} no centro");
    }
}

/// **OS QUATRO DISCORDAM ONDE INTERESSA, e a amostra evita os pontos FIXOS.**
///
/// ⚠️ **Lei paga na wave dos deformadores:** o smoothstep e o smootherstep FIXAM o
/// meio (`0,5 → 0,5`), então uma amostra em `t = 0,5` faria três dos quatro perfis
/// darem o mesmo número e o gate acusaria de decorativo um enum que funciona. A
/// amostra é em `0,25`, onde eles discordam.
#[test]
fn the_four_profiles_disagree_away_from_their_fixed_point() {
    let at = 0.25f32;
    let v: Vec<f32> = (0..4).map(|k| curve(k, at)).collect();
    for i in 0..4 {
        for j in (i + 1)..4 {
            assert!(
                (v[i] - v[j]).abs() > 1e-3,
                "os perfis {i} e {j} coincidem em {at}: {:?}",
                v
            );
        }
    }
    // E o meio é MESMO um ponto fixo de dois deles — o controle da lei acima.
    assert!((curve(2, 0.5) - 0.5).abs() < 1e-6);
    assert!((curve(3, 0.5) - 0.5).abs() < 1e-6);
}

/// **É A MESMA LEI DA FAMÍLIA** — os polinómios do `force.attractor`, byte a byte.
///
/// ⚠️ A célula não veio de uma referência: veio de a nossa própria família discordar
/// de si. Curá-la com uma SEGUNDA lei (um `smoothstep` escrito de outra maneira)
/// faria o mesmo rótulo significar duas coisas em dois nós — o defeito ao contrário.
#[test]
fn the_profiles_are_the_attractors_own_polynomials() {
    for (k, expected) in [
        (1, 0.25f32 * 0.25),
        (2, 0.25 * 0.25 * (3.0 - 2.0 * 0.25)),
        (3, {
            let s = 0.25f32;
            s * s * s * (s * (s * 6.0 - 15.0) + 10.0)
        }),
    ] {
        assert_eq!(curve(k, 0.25), expected, "perfil {k}");
    }
}

/// **O KNOB ESTÁ PINTADO, com os MESMOS rótulos do irmão, e sobe ao device.**
#[test]
fn the_knob_is_painted_with_the_family_labels_and_uploaded() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == CURVE)
        .expect("o Curve tem de estar pintado");
    match h.widget {
        ParamWidget::Enum { labels } => {
            assert_eq!(labels, &["Linear", "Quad", "Smooth", "Smoother"]);
        }
        _ => panic!("o Curve é um Enum"),
    }
    assert!(GPU_KERNEL.params.contains(&CURVE));
    assert!(
        MANIFEST
            .params
            .iter()
            .any(|s| s.name == CURVE && s.default == 0.0),
        "o Linear é o default"
    );
}
