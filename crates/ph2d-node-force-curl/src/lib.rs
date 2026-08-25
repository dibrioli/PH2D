#![forbid(unsafe_code)]
//! `force.curl` — **curl noise**: a divergence-free turbulent flow field
//! (Bridson, Houriham & Nordenstam, *Curl-Noise for Procedural Fluid Flow*,
//! SIGGRAPH 2007). A Motion **force**: a node that adds its per-instance
//! contribution into the transient `accel` column (× the multiplicative
//! `falloff` field, plan §1.6); `motion.integrate` turns it into motion.
//!
//! ## Why the curl, and not just "noise pushed at the particle"
//!
//! Sampling a noise vector directly gives a field with sources and sinks:
//! particles pile up in the sinks and the cloud clumps and dies. The curl of a
//! scalar potential is **divergence-free by construction** — in 2D,
//! `v = (∂ψ/∂y, −∂ψ/∂x)` satisfies `∇·v = ψ_yx − ψ_xy = 0` — so the flow
//! swirls forever and never compresses. That is the whole idea of the paper.
//!
//! `ψ` is fBm value noise ([`noise::fbm`]), evaluated with central differences.
//! Everything is integer-hash + polynomial: **transcendental-free and
//! deterministic** (HR-5), so the same particle at the same playhead always
//! feels the same eddy — CPU today, GPU lane tomorrow.
//!
//! Params: `strength` (accel scale), `scale` (spatial frequency of the eddies),
//! `speed` (how fast the field itself drifts, in playhead-seconds), `octaves`
//! (1..=4 scales of swirl), `seed`.
//!
//! ## ⛔ **RECUSA MEDIDA: o campo NÃO contorna colisores** (doc 89, folha 02)
//!
//! O POP Curl Noise da referência tem um *Add Collision Objects*: *«o campo CONTORNA SDFs
//! de colisores»*. Aqui não, e o mecanismo é de ORDEM e não de custo.
//!
//! ⚠️ **Uma força é `Pure` e roda ANTES do solver.** O `sim.collide`/`motion.collide` age
//! **depois**, sobre `vel`/`P`, empurrando de volta quem penetrou. Para o campo contornar
//! um colisor ele teria de *consultar a geometria dele* na hora de calcular a aceleração —
//! ou seja, uma força a ler o estado de um nó que ainda não correu.
//!
//! ⚠️ **E as duas saídas são as duas caras** que esta casa já recusa noutro sítio: uma
//! porta nova trazendo a geometria do colisor para dentro da força (o campo passa a ter uma
//! dependência que a topologia `pre` não sabe ordenar), ou uma segunda representação do
//! colisor dentro do próprio curl (dois sítios onde a mesma parede vive, e o dia em que
//! discordarem é invisível). O doc 63 já a marcou P2 pela mesma leitura.
//!
//! ⇒ **É trabalho de outra wave, não um param**, e o que existe hoje já entrega o caso
//! comum: o colisor empurra de volta *depois* do campo, então uma partícula não atravessa
//! a parede — ela apenas não a **antecipa**.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::par_build;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
mod noise;
use accum::{add_accel, falloff_at, vec2_at};
use noise::octave;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Central-difference step, in noise space. Small enough to approximate the
/// derivative, large enough that `ψ(x+ε) − ψ(x−ε)` keeps its significant digits
/// in `f32` (the noise lattice has unit period, so `ψ` changes by ~`ε` here — a
/// far smaller step would subtract two nearly equal numbers and keep only noise).
const EPS: f32 = 0.01;
/// Ceiling on `octaves` — the field is sampled `4 × octaves` times per instance
/// per tick, and past four the extra octaves are below a pixel of motion.
const MAX_OCTAVES: u32 = 4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.curl"),
    name: "force.curl",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // The field drifts with the playhead (`speed`).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "strength",
            default: 6.0,
        },
        ParamSpec {
            name: "scale",
            default: 0.35,
        },
        ParamSpec {
            name: "speed",
            default: 0.2,
        },
        ParamSpec {
            name: "octaves",
            default: 2.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⚠️ **O cluster de NOISE, apendado** (doc 89 folha 02). A família de
        // animadores já o tinha; a de forças herdou `octaves` e mais nada — o
        // `lacunarity` e o `roughness` eram LITERAIS cravados no laço
        // (`freq *= 2.0; amp *= 0.5`), e os defaults abaixo são esses literais,
        // então o campo de antes sai AO BIT.
        ParamSpec {
            name: "type",
            default: 0.0,
        },
        ParamSpec {
            name: "lacunarity",
            default: 2.0,
        },
        ParamSpec {
            name: "roughness",
            default: 0.5,
        },
        // O *Pan Noise Field* do Niagara: desliza o campo sem mover as
        // instâncias — como as vórtices caem noutro lugar sem a cena mudar.
        ParamSpec {
            name: "offset_x",
            default: 0.0,
        },
        ParamSpec {
            name: "offset_y",
            default: 0.0,
        },
        // O *Looping + Loop Length* do Cavalry: `0` = nunca fecha, o mundo de
        // sempre (e o segundo instante nem é avaliado).
        ParamSpec {
            name: "loop_period",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The scalar potential `ψ` at a world point, for a given field configuration.
/// Time drifts the field along its own x-axis (the noise is 2D; a third lattice
/// axis is the follow-up when the noise grows one).
fn psi(x: f32, y: f32, drift: f32, seed: f32, spec: ph2d_fbm::Spec, off: [f32; 2]) -> f32 {
    ph2d_fbm::eval(spec, x + drift + seed + off[0], y + off[1], octave)
}

/// `curl ψ` at a world point: `(∂ψ/∂y, −∂ψ/∂x)` by central differences.
/// Divergence-free (see the module docs).
fn curl(x: f32, y: f32, drift: f32, seed: f32, spec: ph2d_fbm::Spec, off: [f32; 2]) -> [f32; 2] {
    let dpsi_dx = psi(x + EPS, y, drift, seed, spec, off) - psi(x - EPS, y, drift, seed, spec, off);
    let dpsi_dy = psi(x, y + EPS, drift, seed, spec, off) - psi(x, y - EPS, drift, seed, spec, off);
    let inv = 1.0 / (2.0 * EPS);
    [dpsi_dy * inv, -dpsi_dx * inv]
}

/// O curl **fechado no tempo**: a mistura dos dois instantes que a costura do laço
/// nomeia.
///
/// ⚠️ **Misturar as duas CURLS é o mesmo que a curl da mistura, e isso não é
/// acaso — é linearidade.** A curl é uma diferença central, um operador linear,
/// então `lerp` e `∂` comutam: o campo misturado continua **divergence-free**, que
/// é a razão de este nó existir. Se a mistura acontecesse depois de uma operação
/// não-linear, o laço quebraria a propriedade em silêncio.
fn curl_looped(
    x: f32,
    y: f32,
    t: (f32, f32, f32),
    speed: f32,
    seed: f32,
    spec: ph2d_fbm::Spec,
    off: [f32; 2],
) -> [f32; 2] {
    let (t_a, t_b, w) = t;
    let a = curl(x, y, t_a * speed, seed, spec, off);
    // `w == 0` é o caminho de sempre: a segunda amostra nem é avaliada, e o nó
    // sem laço custa exactamente o que custava.
    if w == 0.0 {
        return a;
    }
    let b = curl(x, y, t_b * speed, seed, spec, off);
    [a[0] + (b[0] - a[0]) * w, a[1] + (b[1] - a[1]) * w]
}

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel): the exact
/// per-element map of the CPU `eval` — the divergence-free curl of an fBm
/// potential (Bridson 2007), drifting with the playhead.
///
/// **HR-5:** the potential is this node's OWN integer-hash value noise (a port
/// of `noise.rs`, the same mix `motion.wiggle` proved bit-exact on the RTX) —
/// `bitcast<u32>` is Rust's `as u32`, `u32(x)` would be a value cast and diverge
/// on negatives. The divergence-free property is a *cancellation* of mixed
/// differences at exactly step `EPS`, so the stencil must be the CPU's to the
/// letter: a different `h` would measure the noise's curvature instead.
///
/// The octave count is a rounded param (half-AWAY, like Rust) and bounds a real
/// loop — 4 octaves × 4 psi samples × 4 lattice hashes is the heaviest kernel of
/// the five, and still a pure per-element map.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let cl_p = read_P(i);\n\
        var cl_s: ClSpec;\n\
        cl_s.oct = i32(min(max(cl_round(params.octaves), 1.0), CL_MAX_OCTAVES));\n\
        cl_s.lac = params.lacunarity;\n\
        cl_s.rough = params.roughness;\n\
        cl_s.ty = i32(cl_round(params.type_));\n\
        cl_s.off = vec2<f32>(params.offset_x, params.offset_y);\n\
        // A costura do laco (a transcricao de `ph2d_fbm::loop_times`).\n\
        var cl_ta = params.playhead;\n\
        var cl_tb = params.playhead;\n\
        var cl_bw = 0.0;\n\
        if (params.loop_period > 0.0) {\n\
        \x20   let u0 = params.playhead / params.loop_period;\n\
        \x20   let u = u0 - floor(u0);\n\
        \x20   cl_ta = u * params.loop_period;\n\
        \x20   cl_tb = cl_ta - params.loop_period;\n\
        \x20   cl_bw = u * u * (3.0 - 2.0 * u);\n\
        }\n\
        let cl_x = cl_p.x * params.scale;\n\
        let cl_y = cl_p.y * params.scale;\n\
        var cl_v = cl_curl(cl_x, cl_y, cl_ta * params.speed, params.seed, cl_s);\n\
        if (cl_bw != 0.0) {\n\
        \x20   let cl_b = cl_curl(cl_x, cl_y, cl_tb * params.speed, params.seed, cl_s);\n\
        \x20   cl_v = cl_v + (cl_b - cl_v) * cl_bw;\n\
        }\n\
        let cl_w = params.strength * read_falloff(i);\n\
        write_accel(i, read_accel(i) + vec2<f32>(cl_v.x * cl_w, cl_v.y * cl_w));\n",
    wgsl_lib: "\
        const CL_EPS: f32 = 0.01;\n\
        const CL_MAX_OCTAVES: f32 = 4.0;\n\
        fn cl_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn cl_hash2(ix: i32, iy: i32) -> f32 {\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du + bitcast<u32>(iy) * 0x165667b1u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return (f32(h) / f32(0xffffffffu)) * 2.0 - 1.0;\n\
        }\n\
        fn cl_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn cl_noise(x: f32, y: f32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let u = cl_fade(x - x0);\n\
            let v = cl_fade(y - y0);\n\
            let n00 = cl_hash2(ix, iy);\n\
            let n10 = cl_hash2(ix + 1, iy);\n\
            let n01 = cl_hash2(ix, iy + 1);\n\
            let n11 = cl_hash2(ix + 1, iy + 1);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return nx0 + v * (nx1 - nx0);\n\
        }\n\
        struct ClSpec {\n\
            oct: i32,\n\
            lac: f32,\n\
            rough: f32,\n\
            ty: i32,\n\
            off: vec2<f32>,\n\
        };\n\
        // A transcricao de `ph2d_fbm::eval` — a coordenada e escalada por\n\
        // multiplicacao repetida, como na folha. Com lacunarity 2 isso e\n\
        // identico ao `x * freq` que estava aqui; fora das potencias de dois\n\
        // nao e, e a folha e a lei.\n\
        fn cl_fbm(x0: f32, y0: f32, s: ClSpec) -> f32 {\n\
            let gain = clamp(s.rough, 0.0, 1.0);\n\
            var x = x0;\n\
            var y = y0;\n\
            var amp = 1.0;\n\
            var sum = 0.0;\n\
            var total = 0.0;\n\
            let n = max(s.oct, 1);\n\
            for (var k = 0; k < n; k = k + 1) {\n\
                let nz = cl_noise(x, y);\n\
                var shaped = nz;\n\
                if (s.ty == 1) {\n\
                    shaped = abs(nz);\n\
                } else if (s.ty == 2) {\n\
                    let r = 1.0 - abs(nz);\n\
                    shaped = r * r;\n\
                }\n\
                sum = sum + amp * shaped;\n\
                total = total + amp;\n\
                amp = amp * gain;\n\
                x = x * s.lac;\n\
                y = y * s.lac;\n\
            }\n\
            return sum / total;\n\
        }\n\
        fn cl_psi(x: f32, y: f32, drift: f32, seed: f32, s: ClSpec) -> f32 {\n\
            return cl_fbm(x + drift + seed + s.off.x, y + s.off.y, s);\n\
        }\n\
        fn cl_curl(x: f32, y: f32, drift: f32, seed: f32, s: ClSpec) -> vec2<f32> {\n\
            let dpsi_dx =\n\
                cl_psi(x + CL_EPS, y, drift, seed, s)\n\
                - cl_psi(x - CL_EPS, y, drift, seed, s);\n\
            let dpsi_dy =\n\
                cl_psi(x, y + CL_EPS, drift, seed, s)\n\
                - cl_psi(x, y - CL_EPS, drift, seed, s);\n\
            let inv = 1.0 / (2.0 * CL_EPS);\n\
            return vec2<f32>(dpsi_dy * inv, -dpsi_dx * inv);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "strength",
        "scale",
        "speed",
        "octaves",
        "seed",
        "type",
        "lacunarity",
        "roughness",
        "offset_x",
        "offset_y",
        "loop_period",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ForceCurl;

impl NodeOp for ForceCurl {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let strength = ctx.param("strength");
        let scale = ctx.param("scale");
        let speed = ctx.param("speed");
        let octaves = (ctx.param("octaves").round().max(1.0) as u32).min(MAX_OCTAVES);
        let seed = ctx.param("seed");
        let spec = ph2d_fbm::Spec {
            octaves,
            lacunarity: ctx.param("lacunarity"),
            roughness: ctx.param("roughness"),
            ty: ph2d_fbm::NoiseType::from_index(ctx.param("type")),
        };
        let off = [ctx.param("offset_x"), ctx.param("offset_y")];
        let t = ph2d_fbm::loop_times(ctx.playhead() as f32, ctx.param("loop_period"));
        let out = {
            let input = ctx.input(0);
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let contrib: Vec<[f32; 2]> = par_build(input.count(), |i| {
                let p = vec2_at(input, "P", i, [0.0, 0.0]);
                let v = curl_looped(p[0] * scale, p[1] * scale, t, speed, seed, spec, off);
                let w = strength * falloff_at(input, i);
                [v[0] * w, v[1] * w]
            });
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceCurl))?;
    // ADR-0155: a force accumulates `accel`; inert without an integrator downstream.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Produces("accel")],
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Curl Noise",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element force: accumulates accel, identity preserved.
    reg.register_dense_window(MANIFEST.id);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "speed",
    max: 5.0,
}];

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    // **O cluster de NOISE** (doc 89 folha 02) — a família de animadores já o
    // tinha; esta é a de forças a herdá-lo. Os defaults são os literais que
    // estavam cravados no laço, então o campo de antes sai ao bit.
    ParamUiHint {
        param: "type",
        label: "Noise Type",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["fBm", "Turbulence", "Ridged"],
        },
    },
    // ⚠️ A faixa começa em 1: abaixo disso as oitavas ficam MAIORES que a base e
    // o campo perde a leitura fractal. `2` é o universal.
    ParamUiHint {
        param: "lacunarity",
        label: "Lacunarity",
        min: 1.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "roughness",
        label: "Roughness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset_x",
        label: "Offset X",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset_y",
        label: "Offset Y",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // `0` = nunca fecha (o mundo de sempre, e a segunda amostra nem é avaliada).
    ParamUiHint {
        param: "loop_period",
        label: "Loop Period",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: 0.01,
        max: 3.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 2.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 100.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

#[cfg(test)]
#[path = "cluster_tests.rs"]
mod cluster_tests;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
