use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

const DT: f32 = 1.0 / 60.0;

fn marcha(mode: i32, duration: f32, de: f32, para: f32, tiques: usize) -> Vec<f32> {
    let (mut v, mut w) = (de, 0.0);
    (0..tiques)
        .map(|_| {
            (v, w) = passo(mode, duration, 0.0, v, w, para, DT);
            v
        })
        .collect()
}

/// Linear: um salto de 0 a 1 leva `duration` — a meio vai a meio, e chega sem passar.
#[test]
fn linear_crosses_a_unit_jump_in_its_duration() {
    let v = marcha(MODE_LINEAR, 0.5, 0.0, 1.0, 40);
    assert!((v[14] - 0.5).abs() < 1e-5, "a meio do caminho: {}", v[14]);
    assert!(
        v[28] < 1.0 && v[29] == 1.0,
        "chega ao tique 30: {} {}",
        v[28],
        v[29]
    );
    assert!(v.iter().all(|&x| x <= 1.0));
    let volta = marcha(MODE_LINEAR, 0.5, 1.0, 0.0, 40);
    assert_eq!(volta[29], 0.0, "e volta pelo mesmo tempo");
}

/// Smooth: a `duration` é a MEIA-VIDA — depois dela falta metade.
#[test]
fn smooth_halves_the_distance_every_duration() {
    let v = marcha(MODE_SMOOTH, 0.25, 0.0, 1.0, 30);
    assert!(
        (v[14] - 0.5).abs() < 1e-3,
        "uma meia-vida (15 tiques): {}",
        v[14]
    );
    assert!((v[29] - 0.75).abs() < 1e-3, "duas: {}", v[29]);
}

/// Spring: assenta no alvo; com `bounce` 0 não passa dele, com `bounce` positivo passa.
#[test]
fn spring_settles_and_overshoots_only_when_asked() {
    let v = marcha(MODE_SPRING, 0.5, 0.0, 1.0, 120);
    assert!((v[119] - 1.0).abs() < 1e-3, "assenta: {}", v[119]);
    assert!(v.iter().all(|&x| x <= 1.0 + 1e-3), "sem salto");
    let (mut x, mut w) = (0.0, 0.0);
    let mut pico = 0.0f32;
    for _ in 0..120 {
        (x, w) = passo(MODE_SPRING, 0.5, 0.5, x, w, 1.0, DT);
        pico = pico.max(x);
    }
    assert!(pico > 1.05, "com bounce passa do alvo: pico {pico}");
}

#[test]
fn a_zero_duration_is_the_target_at_once() {
    for mode in [MODE_LINEAR, MODE_SMOOTH, MODE_SPRING] {
        assert_eq!(passo(mode, 0.0, 0.0, 0.0, 0.0, 1.0, DT), (1.0, 0.0));
    }
}

// ── no grafo: o laço `pre`, a semente no alvo ──────────────────────────────────

static ALVO: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.alvo"),
    name: "test.alvo",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "x",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};
struct Alvo;
impl NodeOp for Alvo {
    fn manifest(&self) -> &'static NodeManifest {
        &ALVO
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let x = ctx.param("x");
        ctx.emit(Stream::new(1).with("v", Column::Scalar(vec![x])));
    }
}
struct Reg;
impl OpResolver for Reg {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == MANIFEST.id {
            Some(&ValueFollow)
        } else if ty == ALVO.id {
            Some(&Alvo)
        } else {
            None
        }
    }
}

/// ⭐ No grafo: nasce NO alvo (sem salto do zero), e quando o alvo muda persegue-o no tempo.
#[test]
fn in_a_graph_it_is_born_at_the_target_and_then_chases_it() {
    let mut g = Graph::new();
    let a = g.add_node("test.alvo");
    let f = g.add_node("value.follow");
    g.connect(Edge {
        from: (a, 0),
        to: (f, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (f, 0),
        to: (f, 1),
        delayed: true,
    })
    .unwrap();
    g.set_param(a, "x", 0.4);
    g.set_param(f, DURATION, 0.5);
    let mut cook = Cook::new();
    let ler = |cook: &mut Cook, g: &Graph, t: f64| -> f32 {
        let v = match cook.cook(g, &Reg, f, t).unwrap()[0].as_stream().get("v") {
            Some(Column::Scalar(v)) => v[0],
            _ => f32::NAN,
        };
        cook.advance_tick(g, &Reg, t).unwrap();
        v
    };
    assert_eq!(ler(&mut cook, &g, 0.0), 0.4, "nasce no alvo");
    g.set_param(a, "x", 1.4);
    let v: Vec<f32> = (1..=40)
        .map(|k| ler(&mut cook, &g, f64::from(k) / 60.0))
        .collect();
    assert_eq!(
        v[0], 0.4,
        "no tique da mudança ainda vale o de antes (o Godot)"
    );
    assert!((v[15] - 0.9).abs() < 1e-4, "a meio: {}", v[15]);
    assert_eq!(v[39], 1.4);
}
