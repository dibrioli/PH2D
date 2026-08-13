//! Os gates do REGISTRY — o round-trip de cada canal de side-metadata.
//!
//! Saíram do `lib.rs` no teto de LOC. Seguem FILHO por `#[path]`, então
//! `use super::*` alcança os privados.
use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{LoweringKind, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const T: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const fn port(name: &'static str) -> PortSpec {
    PortSpec { name, ty: T }
}

static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("reg.src"),
    name: "reg.src",
    inputs: &[],
    outputs: &[port("out")],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
static PASS_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("reg.pass"),
    name: "reg.pass",
    inputs: &[port("in")],
    outputs: &[port("out")],
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
        ctx.emit(Stream::new(2).with("v", Column::Scalar(vec![10.0, 20.0])));
    }
}

struct Pass;
impl NodeOp for Pass {
    fn manifest(&self) -> &'static NodeManifest {
        &PASS_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let passthrough = ctx.input(0).clone();
        ctx.emit(passthrough);
    }
}

/// Shares SRC_MAN's id — a collision.
struct Dup;
impl NodeOp for Dup {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
}

#[test]
fn register_and_resolve() {
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).unwrap();
    reg.register(Box::new(Pass)).unwrap();
    assert_eq!(reg.len(), 2);
    assert!(reg.resolve(SRC_MAN.id).is_some());
    assert!(reg.resolve(NodeTypeId::of("nope")).is_none());
}

#[test]
fn param_ui_hints_round_trip() {
    static HINTS: &[ParamUiHint] = &[ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 1.0,
        max: 20.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    }];
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).unwrap();
    assert!(reg.param_ui(SRC_MAN.id).is_none()); // none until registered
    reg.register_param_ui(SRC_MAN.id, HINTS);
    let got = reg.param_ui(SRC_MAN.id).expect("registered");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].param, "rows");
    assert!(got[0].widget.is_integer());
    assert!(reg.param_ui(NodeTypeId::of("nope")).is_none());
}

#[test]
fn param_gates_round_trip() {
    static GATES: &[ParamGate] = &[ParamGate {
        param: "hole",
        when: "kind",
        values: &[7],
    }];
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).unwrap();
    assert!(reg.param_gates(SRC_MAN.id).is_none()); // none until registered
    reg.register_param_gates(SRC_MAN.id, GATES);
    let got = reg.param_gates(SRC_MAN.id).expect("registered");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].param, "hole");
    assert_eq!(got[0].when, "kind");
    assert_eq!(got[0].values, &[7]);
    assert!(reg.param_gates(NodeTypeId::of("nope")).is_none());
}

/// **O gate de PRESENÇA DE TEXTO faz o round-trip, e nasce ausente.**
///
/// Side-metadata com default `&[]` — o molde do `param_gates`/`reduces`/`luts`:
/// os cento e tantos nós que não o usam não são tocados, e o contrato congelado
/// (`NodeManifest = 8`) não sente nada.
#[test]
fn param_gates_text_round_trip() {
    static GATES: &[ParamGateText] = &[ParamGateText {
        param: "p0x",
        when_text: "path",
        when_present: false,
    }];
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).expect("src");
    assert!(
        reg.param_gates_text(SRC_MAN.id).is_none(),
        "um no sem entrada aqui nao e gateado"
    );
    reg.register_param_gates_text(SRC_MAN.id, GATES);
    let got = reg.param_gates_text(SRC_MAN.id).expect("registrado");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].when_text, "path");
    assert!(!got[0].when_present, "as coordenadas somem COM a forma");
    assert!(reg.param_gates_text(NodeTypeId::of("nope")).is_none());
}

#[test]
fn couplings_round_trip() {
    static COUPLINGS: &[Coupling] = &[Coupling::Produces("accel"), Coupling::Requires("P")];
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).unwrap();
    assert!(reg.couplings(SRC_MAN.id).is_none()); // none until registered
    reg.register_couplings(SRC_MAN.id, COUPLINGS);
    let got = reg.couplings(SRC_MAN.id).expect("registered");
    assert_eq!(got.len(), 2);
    // ⚠️ Casa por PADRÃO, não por `==`: desde o `ProducesWhen` um acoplamento
    // pode carregar um ponteiro de função, e é assim que TODO leitor de
    // produção pergunta (o `matches!` do `ph2d-motion-diagnose`).
    assert!(matches!(got[0], Coupling::Produces("accel")));
    assert_eq!(got[0].column(), "accel");
    assert!(matches!(got[1], Coupling::Requires("P")));
    assert!(reg.couplings(NodeTypeId::of("nope")).is_none());
}

#[test]
fn duplicate_id_is_rejected() {
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).unwrap();
    assert_eq!(
        reg.register(Box::new(Dup)),
        Err(RegistryError::Collision {
            id: SRC_MAN.id,
            name: "reg.src"
        })
    );
}

#[test]
fn registry_is_a_resolver_the_cook_can_use() {
    // The registry IS the OpResolver — validate + cook a real graph through it.
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).unwrap();
    reg.register(Box::new(Pass)).unwrap();

    let mut g = Graph::new();
    let s = g.add_node("reg.src");
    let p = g.add_node("reg.pass");
    g.connect(Edge {
        from: (s, 0),
        to: (p, 0),
        delayed: false,
    })
    .unwrap();
    g.validate(&reg).expect("graph is well-typed");

    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, p, 0.0).unwrap();
    match out[0].as_stream().get("v") {
        Some(Column::Scalar(v)) => assert_eq!(v, &vec![10.0, 20.0]),
        other => panic!("expected scalar column, got {other:?}"),
    }
}
