//! Os gates do [`super::SCALE_X`]/[`super::SCALE_Y`] — o arrasto anisotrópico
//! (doc 89, folha 02).
//!
//! ⚠️ **Estes gates COZINHAM o nó, e a 1ª versão deles não o fazia.** Eu escrevi um
//! `run()` que repetia a aritmética do `eval`, e **três mutações sobreviveram** —
//! apagar a anisotropia, trocar os dois eixos, e (no irmão empuxo) ignorar a coluna
//! inteira. Um gate que reimplementa a lei mede a cópia dele, não o produto.
//! [[feedback_gate_must_drive_the_product_not_a_transcription]]

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId, PortSpec};

/// Uma peça a mover-se na DIAGONAL, com velocidade igual nos dois eixos — a fixture
/// que torna a anisotropia legível: qualquer diferença na saída é do knob.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.drag.test.diag"),
    name: "force.drag.test.diag",
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
        ctx.emit(
            Stream::new(1)
                .with("P", Column::Vec2(vec![[0.0, 0.0]]))
                .with("vel", Column::Vec2(vec![[4.0, 4.0]])),
        );
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&ForceDrag),
            _ => None,
        }
    }
}

/// Coze `src → force.drag(coefficient = 2, scale_x, scale_y)` e devolve o `accel`.
fn drag(sx: f32, sy: f32) -> [f32; 2] {
    let mut g = Graph::new();
    let s = g.add_node("force.drag.test.diag");
    let d = g.add_node("force.drag");
    g.connect(Edge {
        from: (s, 0),
        to: (d, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(d, "coefficient", 2.0);
    g.set_param(d, SCALE_X, sx);
    g.set_param(d, SCALE_Y, sy);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, d, 0.0).unwrap()[0]
        .as_stream()
        .get("accel")
    {
        Some(Column::Vec2(v)) => v[0],
        other => panic!("accel: {other:?}"),
    }
}

/// **`1`/`1` É O ARRASTO DE SEMPRE, AO BIT.**
///
/// ⚠️ A identidade aqui é ARITMÉTICA e não estrutural — `x * 1.0` é exacto em
/// IEEE-754 —, então o gate afirma a igualdade EXACTA. Um `lerp` ou um `powf` no
/// lugar da multiplicação acusa aqui.
#[test]
fn the_neutral_pair_is_the_isotropic_drag_bit_for_bit() {
    assert_eq!(drag(1.0, 1.0), [-8.0, -8.0]);
}

/// **CADA EIXO FREIA POR SI** — a folha que cai balançando.
///
/// ⚠️ Os dois valores são DIFERENTES de propósito: com `sx == sy` a mutação que
/// troca os dois knobs sobreviveria, e ela sobreviveu à primeira leva.
#[test]
fn each_axis_is_braked_on_its_own_and_they_do_not_swap() {
    assert_eq!(drag(1.0, 0.25), [-8.0, -2.0]);
    assert_eq!(
        drag(0.25, 1.0),
        [-2.0, -8.0],
        "e a troca dos dois é visível"
    );
}

/// **UM EIXO DESLIGADO NÃO VAZA PARA O OUTRO** — o defeito que um vetor rodado teria.
#[test]
fn a_disabled_axis_leaks_nothing_into_the_other() {
    let a = drag(0.0, 1.0);
    assert_eq!(a, [0.0, -8.0], "sem resíduo em X: {a:?}");
}

/// **OS DOIS KNOBS ESTÃO PINTADOS E SOBEM AO DEVICE.**
#[test]
fn both_knobs_are_painted_and_uploaded() {
    for p in [SCALE_X, SCALE_Y] {
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == p),
            "`{p}` tem de estar pintado"
        );
        assert!(
            GPU_KERNEL.params.contains(&p),
            "`{p}` tem de chegar ao device: {:?}",
            GPU_KERNEL.params
        );
        assert!(
            MANIFEST
                .params
                .iter()
                .any(|s| s.name == p && s.default == 1.0),
            "`{p}` nasce neutro"
        );
    }
}
