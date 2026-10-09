use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

// A grid source: `n` instances at the origin, so `value.time` can read a count.
static GRID_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.time.test.grid"),
    name: "value.time.test.grid",
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
struct Grid;
impl NodeOp for Grid {
    fn manifest(&self) -> &'static NodeManifest {
        &GRID_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == GRID_MAN.id => Some(&Grid),
            t if t == MANIFEST.id => Some(&ValueTime),
            _ => None,
        }
    }
}

fn vals(s: &Stream) -> Vec<f32> {
    match s.get(VALUE_COL).unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("v"),
    }
}

/// Cook `value.time` at `playhead`; `connect_grid` decides whether it reads a
/// count from a source (length-N) or stands alone (length-1 global).
fn time_at(playhead: f64, connect_grid: bool, setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<f32> {
    let mut g = Graph::new();
    let time = g.add_node("value.time");
    if connect_grid {
        let grid = g.add_node("value.time.test.grid");
        g.connect(Edge {
            from: (grid, 0),
            to: (time, 0),
            delayed: false,
        })
        .unwrap();
    }
    setup(&mut g, time);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, time, playhead).unwrap();
    vals(out[0].as_stream())
}

/// UNCONNECTED input → one global clock (length-1). This is the field that
/// `motion.drive` broadcasts across every instance — *the* clock (doc 12).
#[test]
fn an_unconnected_time_emits_a_single_global_clock() {
    // Default rate 1 → the value IS the playhead.
    assert_eq!(time_at(0.0, false, |_, _| {}), vec![0.0], "t=0 -> 0");
    assert_eq!(time_at(2.5, false, |_, _| {}), vec![2.5], "t=2.5 -> 2.5");
    // Rate scales the clock; offset shifts its start.
    let v = time_at(2.0, false, |g, t| {
        g.set_param(t, "rate", 3.0);
        g.set_param(t, "offset", 1.0);
    });
    assert_eq!(v, vec![7.0], "2*3 + 1 = 7");
}

/// **Monotonic, not periodic** — the clock keeps CLIMBING past any period,
/// where `value.lfo(Saw)` would fold back. Falsifiable: a wrapped clock would
/// read the same at `t` and `t + 1/rate`.
#[test]
fn the_clock_climbs_and_never_folds() {
    let at = |t: f64| time_at(t, false, |g, node| g.set_param(node, "rate", 1.0))[0];
    let a = at(0.5);
    let b = at(10.5); // ten "periods" later — a Saw would read the same
    assert!(
        b > a + 9.5,
        "the clock climbed by ~10, did not fold: {a} -> {b}"
    );
}

/// CONNECTED input → the field's length follows the geometry (N=3), and
/// `stagger` sends a travelling ramp across it: at t=0 with stagger 0.5,
/// instance i reads `i·0.5`. As t advances the whole ramp shifts up.
#[test]
fn a_connected_time_emits_a_travelling_ramp() {
    let v = time_at(0.0, true, |g, t| g.set_param(t, "stagger", 0.5));
    assert_eq!(v, vec![0.0, 0.5, 1.0], "spatial ramp at t=0");
    // t=2 shifts every element up by 2 (rate 1) — the ramp travels.
    let v = time_at(2.0, true, |g, t| g.set_param(t, "stagger", 0.5));
    assert_eq!(v, vec![2.0, 2.5, 3.0], "the ramp travelled up by t");
}

/// A negative rate runs the clock BACKWARDS — the value decreases as the
/// playhead advances (reverse scroll / counter-rotation).
#[test]
fn a_negative_rate_runs_backwards() {
    let v = time_at(3.0, false, |g, t| g.set_param(t, "rate", -2.0));
    assert_eq!(v, vec![-6.0], "3*(-2) = -6");
}

/// ⭐ **O ATRASO NO TEMPO** (ciclo 17): `delay` distribui-se pela ordem e pela curva — a
/// primeira linha não espera, a última espera o `delay` inteiro; com `Quad In` o meio espera
/// só um quarto.
#[test]
fn the_delay_spreads_over_the_order_and_the_curve() {
    let v = time_at(2.0, true, |g, t| g.set_param(t, "delay", 1.0));
    assert_eq!(v, vec![2.0, 1.5, 1.0], "linear: a cauda espera 1 s");
    let v = time_at(2.0, true, |g, t| {
        g.set_param(t, "delay", 1.0);
        g.set_param(t, "ease_curve", ph2d_ease::QUAD as f32);
    });
    assert_eq!(v, vec![2.0, 1.75, 1.0], "Quad In: o meio espera 0,25");
    // Do centro para fora: o do meio arranca primeiro.
    let v = time_at(2.0, true, |g, t| {
        g.set_param(t, "delay", 1.0);
        g.set_param(t, "order", ph2d_ease::order::ORDER_FROM_CENTER as f32);
    });
    assert_eq!(v, vec![1.0, 2.0, 1.0], "o centro não espera");
}

/// Com o `delay` a zero o relógio é o de sempre AO BIT, mesmo com a onda toda configurada.
#[test]
fn a_zero_delay_is_the_old_clock_bit_for_bit() {
    let antes = time_at(1.37, true, |g, t| g.set_param(t, "stagger", 0.13));
    let depois = time_at(1.37, true, |g, t| {
        g.set_param(t, "stagger", 0.13);
        g.set_param(t, "order", 3.0);
        g.set_param(t, "ease_curve", 11.0);
    });
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&antes), bits(&depois));
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
