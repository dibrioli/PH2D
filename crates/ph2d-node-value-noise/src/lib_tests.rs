//! Gates do NÓ — o manifesto, a amostra e o cook de ponta a ponta.
//!
//! Saíram do `lib.rs` no teto de LOC (814 > 700), por assunto e seguindo o
//! precedente dos dois irmãos que já existiam: `noise_tests.rs` mede **o que o
//! campo desenha**, `space_tests.rs` **onde ele amostra**, `time_tests.rs`
//! **quando**, e este **o que o nó É**. Segue FILHO por `#[path]`, então
//! `use super::*` alcança os privados.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// A default-ish sampler for the row tests (frequency low = a smooth swell).
fn smooth() -> Sample {
    Sample {
        frequency: 0.1,
        speed: 0.5,
        octaves: 1,
        roughness: 0.5,
        amplitude: 1.0,
        offset: 0.0,
        seed: 0.0,
        // A premissa desta fixture, DECLARADA: ela mede o kernel de VALOR,
        // o que o no sempre shipou. Herda-la de um default e o que faz um
        // teste inverter de sentido quando o default se move.
        kernel: Kernel::Value,
        feature: CellFeature::Cells,
        jitter: 1.0,
        // ⚠️ A premissa desta fixture, DECLARADA (a lição do grupo A): ela mede
        // o mundo ANTERIOR ao grupo B — lacunarity 2, sem laço, sem pan.
        // Herdá-los de um default é o que faz um teste inverter de sentido
        // quando o default se move.
        lacunarity: 2.0,
        loop_period: 0.0,
        pan_x: 0.0,
        pan_y: 0.0,
    }
}

/// THE falsification: the field is COHERENT, not white. At a low frequency
/// adjacent instances read nearby lattice points, so the mean step between
/// neighbours is SMALL; raise the frequency past one lattice unit per instance
/// and neighbours DECORRELATE (a large step). A regression to white noise (a
/// per-instance hash, like `instance_field` Random) would fail the low-freq
/// half — its neighbour step is ~2/3 of the full range, always.
#[test]
fn the_field_is_coherent_not_white() {
    let n = 24u32;
    let mean_step = |freq: f32| {
        let s = Sample {
            frequency: freq,
            ..smooth()
        };
        let row: Vec<f32> = (0..n).map(|i| s.at(i, 0.0)).collect();
        let total: f32 = row.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
        total / (n - 1) as f32
    };
    let coherent = mean_step(0.1); // 10 instances per feature → smooth
    let decorrelated = mean_step(3.0); // 3 units apart → white-ish
    assert!(
        coherent < 0.15,
        "low frequency must be smooth, got mean step {coherent}"
    );
    assert!(
        coherent > 0.0,
        "but not constant — it is still a varying field"
    );
    assert!(
        decorrelated > 2.0 * coherent,
        "high frequency decorrelates: {decorrelated} vs {coherent}"
    );
}

/// The field EVOLVES over time (the `wiggle`/CHOP-translate behaviour): the
/// same instance reads a different value at a different playhead when speed > 0.
#[test]
fn time_evolves_the_field() {
    let s = smooth();
    assert_ne!(s.at(5, 0.0), s.at(5, 2.0), "speed > 0 drifts the field");
}

/// Speed 0 FREEZES the field — a static per-instance coherent random,
/// independent of the playhead (the degenerate case, and a useful one).
#[test]
fn speed_zero_freezes_the_field() {
    let s = Sample {
        speed: 0.0,
        ..smooth()
    };
    for i in 0..24 {
        assert_eq!(s.at(i, 0.0), s.at(i, 7.5), "speed 0 is time-invariant");
    }
}

/// The output is bounded by `|amplitude| + |offset|` (fBm ∈ [-1,1]): a value
/// stream downstream never sees a runaway magnitude, whatever the octaves.
#[test]
fn the_output_is_bounded_by_amplitude_and_offset() {
    let s = Sample {
        octaves: 8,
        amplitude: 4.0,
        offset: 10.0,
        ..smooth()
    };
    for i in 0..200 {
        let v = s.at(i, i as f32 * 0.3);
        assert!(v.is_finite(), "finite at {i}");
        assert!(
            (6.0..=14.0).contains(&v),
            "within offset±amplitude: {v} at {i}"
        );
    }
}

/// A value source emitting an N-wide instance stream, so `value.noise` can be
/// driven for its COUNT through a real cook.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.noise.test.src"),
    name: "value.noise.test.src",
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
struct Src(usize);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // A vec2 `P` column of length N — the noise reads it for count only.
        ctx.emit(Stream::new(self.0).with("P", Column::Vec2(vec![[0.0, 0.0]; self.0])));
    }
}

struct Ops(usize);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(Box::leak(Box::new(Src(self.0))) as &dyn NodeOp),
            t if t == MANIFEST.id => Some(&ValueNoise),
            _ => None,
        }
    }
}

/// End-to-end through the cook: connected to a length-8 stream it emits a
/// length-8 field (cardinality follows the geometry) of finite values, and the
/// values match `Sample::at` (the eval reaches the same math the tests probe).
#[test]
fn emits_a_length_n_field_through_the_cook() {
    let ops = Ops(8);
    let mut g = Graph::new();
    let src = g.add_node("value.noise.test.src");
    let vn = g.add_node("value.noise");
    g.set_param(vn, "frequency", 0.1);
    g.connect(Edge {
        from: (src, 0),
        to: (vn, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vn, 3.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => {
            assert_eq!(v.len(), 8, "cardinality follows the length-8 stream");
            let s = Sample {
                frequency: 0.1,
                ..Sample::from_ctx_defaults()
            };
            for (i, &got) in v.iter().enumerate() {
                assert!(got.is_finite(), "finite at {i}");
                assert_eq!(got, s.at(i as u32, 3.0), "eval == Sample::at at {i}");
            }
        }
        _ => panic!("v"),
    }
}

/// Unconnected, the field is ONE global value (the count law's `max(_, 1)`) —
/// not the zero-count stage the engine's default would skip.
#[test]
fn an_unconnected_noise_is_one_global_value() {
    let ops = Ops(0);
    let mut g = Graph::new();
    let vn = g.add_node("value.noise");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vn, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v.len(), 1, "one global oscillation"),
        _ => panic!("v"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

impl Sample {
    /// The MANIFEST defaults, for tests that assert the eval path matches the
    /// direct sampler (only `frequency` is overridden in the cook test).
    fn from_ctx_defaults() -> Self {
        Self {
            frequency: 0.2,
            speed: 0.5,
            octaves: 1,
            roughness: 0.5,
            amplitude: 1.0,
            offset: 0.0,
            seed: 0.0,
            kernel: Kernel::Value,
            feature: CellFeature::Cells,
            jitter: 1.0,
            // ⚠️ Premissa DECLARADA: os defaults do MANIFEST para o grupo B.
            lacunarity: 2.0,
            loop_period: 0.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}
