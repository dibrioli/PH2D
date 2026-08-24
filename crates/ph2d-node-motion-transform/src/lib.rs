#![forbid(unsafe_code)]
//! `motion.transform` — a Motion **modifier**: scales then offsets the `P`
//! (Vec2) attribute of its input instance stream, masked per-instance by the
//! multiplicative `falloff` column (§1.2; absent → `1.0`) so a focus field makes
//! only its region move — consistent with every other spatial node
//! (orbit/stagger/oscillator/wiggle all read `falloff`). Passes every other
//! column through unchanged (count is preserved). Pure.
//!
//! Params (read via `ctx.param` — per-instance override else the manifest
//! default shown): `scale` (1.0), `offset_x` (0.0), `offset_y` (0.0). Per
//! instance: `full = P * scale + (offset_x, offset_y)`, then the falloff blends
//! `P' = lerp(P, full, falloff)` (`falloff = 0` keeps `P`, `1` takes the full
//! transform).
//!
//! **Scope vs its siblings (audit 2026-07-10):** `scale` here scales POSITIONS
//! about the world origin (the layout spreads/contracts; dot size is
//! untouched) — a different thing from `motion.scale`, which scales the `size`
//! column (each sprite grows; layout untouched). And with `scale = 1` this
//! node degenerates to exactly `motion.move` (offset·falloff). It stays: it is
//! the only node that scales the layout, and the combined `P·s + o` affine is
//! one node instead of two. Reach for `move` for a plain offset, `scale` for
//! sprite size, and this for spreading a layout about the origin.
//!
//! ⚠️ **And the missing third of the affine is DELIBERATE: the layout ROTATION is
//! [`motion.orbit`] with `speed = 0`.** It rotates `P` about a pivot — which is what rotating a
//! layout means — and it already pays the `cos/sin` this node would otherwise have to pay a
//! second time. Not writing that here is what made it read as a hole: the doc-89 sheet for the
//! DISTRIBUTION family independently listed *"Coordinates: centre + rotation"* as a gap on all
//! seven distributions, citing *"`motion.transform` dá o centro mas não tem rotação"* — a whole
//! cluster of work proposed because the factorisation lived in neither node's docs. The centre
//! is `offset_x`/`offset_y` here; the rotation is the orbit; there is nothing to add.
//!
//! ## The PIVOT (doc 89 folha 05 — the P0)
//!
//! The scale used to be about the **world origin** and nothing else, so a layout
//! centred at `(5, 0)` scaled by 2 also **translated** to `(10, 0)` — the artist
//! sees it in the first scene. `pivot_mode` chooses what it scales about: the
//! origin (the default, and byte-identical to before), a typed `Point`, or the
//! layout's own `Centroid` — Blender's `Center` socket on *Scale Instances*, its
//! *Transform Geometry* acting on the geometry's own origin, C4D's Transform
//! Space Node/Object.
//!
//! ⚠️ **The pivot FOLDS INTO THE OFFSET and the per-element law does not change.**
//! `(p − c)·s + c + o` is `p·s + (o + c(1−s))`, so there is one affine, computed
//! the way it always was; a second per-element expression would be a second place
//! for the falloff mask and the GPU port to disagree. The fold is hoisted out of
//! the loop on the CPU and inline in the kernel — same operations, same order.
//!
//! ⚠️ **And the centroid only became POSSIBLE to ask for on 2026-08-12.** The
//! sheet's cell measured it as *inexpressible* — *"`P` não chega ao domínio de
//! valor por rota nenhuma"* — and that was true until `motion.expression` grew
//! its `x`/`y` lanes; today `expression("x") → value.reduce(Mean)` measures it.
//! It is a MODE here rather than a wire because that chain is four nodes to say
//! *"the centre of this thing"*, twice over for two axes.
//!
//! ⚠️ **What IS open, and it is not this:** using the orbit statically costs the cook memo. Its
//! `Effect::Temporal` makes the fingerprint key on the playhead, so a rotation that cannot change
//! re-cooks every frame — measured, `0,0003 ms` here against `0,6477 ms` there over 102.400
//! elements, **2294×** (`ph2d-gpu-cook::measure_static_orbit`). Curing it means letting the cook
//! ask *"is this node temporal at THIS instant?"*, and both `NodeManifest` and `OpResolver` are
//! frozen (§6) — so it is an ADR, not an edit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.transform"),
    name: "motion.transform",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        ParamSpec {
            name: "offset_x",
            default: 0.0,
        },
        ParamSpec {
            name: "offset_y",
            default: 0.0,
        },
        // **O LINK DE CORRENTE e o eixo Y** — ver [`UNIFORM`]. `uniform = 1` ⇒ o `scale`
        // vale para os dois eixos, byte a byte (o precedente exato do `motion.scale`).
        ParamSpec {
            name: "uniform",
            default: 1.0,
        },
        ParamSpec {
            name: "scale_y",
            default: 1.0,
        },
        // WHAT the scale happens about: 0 the world origin (what this node always
        // did), 1 the typed point, 2 the layout's own centroid. Default 0 ⇒ every
        // document written before this reads exactly what it read before.
        ParamSpec {
            name: "pivot_mode",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_y",
            default: 0.0,
        },
    ],
    // `lowerings` stays `Cpu`: the `LoweringKind::Wgsl` path is the scalar
    // `eval_column` route (`ph2d-expr`), and `P` is a `Vec2` column, so that
    // route still doesn't apply. The GPU lowering this node DOES have is the
    // ADR-0126 side-channel kernel (`GPU_KERNEL` below, registered via
    // `register_gpu_kernel`) — a separate mechanism that never touches the
    // frozen `NodeManifest`, exactly like `motion.move`/`motion.oscillator`.
    lowerings: &[LoweringKind::Cpu],
};

/// **O LINK DE CORRENTE** (doc 89 folha 04… não: folha 05 — Blender *Transform Geometry* ▸
/// Scale é um **Vector**; C4D Cloner/effector **S.XYZ**; Cavalry Duplicator *Shape Scale*).
///
/// ⚠️ **A cura é a CÓPIA da do irmão, e a célula pré-condenou a alternativa.** O
/// `motion.scale` teve exactamente este defeito — *"uma coluna Vec2 a partir de um número"* — e
/// foi curado em 08/08 com `uniform` + `amount_y` + `ParamGate`. A cerca dele (§4-C3 da folha)
/// diz por que o `amount` **NÃO** vira "o fator X": *"mudaria o comportamento de todo grafo já
/// autorado… e a demo de boot sozinha põe treze `motion.scale`"*. Partir o `scale` deste nó em
/// dois números soltos teria o mesmo defeito, no arquivo ao lado.
///
/// ⚠️ **`>= 0.5` e não `!= 0.0`**: o param chega como `f32` de um toggle, e é a mesma leitura
/// que o WGSL faz — o CPU e o device não podem discordar sobre o que *"uniform"* quer dizer.
///
/// ⚠️ **Ligado, os dois eixos são O MESMO NÚMERO** (não dois números iguais): é isso que faz o
/// default reduzir à expressão que sempre shipou, e não a uma que dá no mesmo.
const UNIFORM: &str = "uniform";
/// O fator do eixo Y quando o link está desligado — ver [`UNIFORM`].
const SCALE_Y: &str = "scale_y";

/// Os dois fatores que o artista autorou. Espelho verbatim do `authored_factors` do
/// `motion.scale`: uma segunda lei para *"o que o link significa"* é como as duas divergem.
fn authored_factors(scale: f32, uniform: f32, scale_y: f32) -> (f32, f32) {
    if uniform >= 0.5 {
        (scale, scale)
    } else {
        (scale, scale_y)
    }
}

/// The per-element affine map `p' = p * scale + (ox, oy)`. Pure and isolated so
/// the arithmetic is unit-tested directly with non-identity values, alongside
/// the end-to-end cook test that drives it via per-instance param overrides.
///
/// ⚠️ Com o link ligado `sx` e `sy` são **o mesmo `f32`**, então isto é literalmente
/// `p * scale + (ox, oy)` — a expressão de sempre, não uma que a iguala.
fn apply_xform(p: [f32; 2], sx: f32, sy: f32, ox: f32, oy: f32) -> [f32; 2] {
    [p[0] * sx + ox, p[1] * sy + oy]
}

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`) —
/// shared masking read, identical to the other spatial Motion nodes.
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// **What the scale happens about** (doc 89 folha 05 — the P0).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Pivot {
    /// The world origin — what this node always did, and the default.
    WorldOrigin,
    /// The `pivot_x`/`pivot_y` the artist typed.
    Point,
    /// The mean of the input's own `P` — Blender's *Transform Geometry* acting
    /// on the geometry's origin, C4D's Object transform space.
    Centroid,
}

impl Pivot {
    /// From the `pivot_mode` param. Out of range falls back to `WorldOrigin` —
    /// the behaviour that was always there beats a mode nobody asked for.
    #[must_use]
    pub fn of(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Point,
            2 => Self::Centroid,
            _ => Self::WorldOrigin,
        }
    }
}

/// The mean of a `P` column — *"the centre of this layout"*, and `None` for a
/// stream with no positions to average (an empty mean is not zero, it is absent,
/// and the caller then has nothing to pivot about).
fn centroid(s: &Stream) -> Option<[f32; 2]> {
    let Some(Column::Vec2(p)) = s.get("P") else {
        return None;
    };
    if p.is_empty() {
        return None;
    }
    #[expect(clippy::cast_precision_loss, reason = "an element count")]
    let n = p.len() as f32;
    let sum = p
        .iter()
        .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
    Some([sum[0] / n, sum[1] / n])
}

/// **The offset the affine actually uses, with the pivot folded in.**
///
/// `(p − c)·s + c + o` is `p·s + (o + c(1−s))`, so a pivot never becomes a second
/// per-element expression — there is one affine, and the falloff mask and the GPU
/// port keep having exactly one thing to agree about.
///
/// ⚠️ The zero pivot returns the offset **verbatim** rather than computing
/// `o + 0·(1−s)`: the two differ only in the sign of a zero, which nothing can
/// see — and that is the point. Byte-identity for every document written before
/// the pivot existed is then a fact of STRUCTURE, not an argument about IEEE.
fn folded_offset(sx: f32, sy: f32, ox: f32, oy: f32, c: [f32; 2]) -> (f32, f32) {
    if c[0] == 0.0 && c[1] == 0.0 {
        return (ox, oy);
    }
    (ox + c[0] * (1.0 - sx), oy + c[1] * (1.0 - sy))
}

/// Apply the affine map to `p`, then blend from the original toward the
/// transformed position by `f` (the falloff): `f = 0` keeps `p`, `f = 1` takes
/// the full transform. Mirrors `motion.orbit`'s focus blend.
fn xform_masked(p: [f32; 2], sx: f32, sy: f32, ox: f32, oy: f32, f: f32) -> [f32; 2] {
    let full = apply_xform(p, sx, sy, ox, oy);
    [p[0] + (full[0] - p[0]) * f, p[1] + (full[1] - p[1]) * f]
}

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): the exact per-element map of
/// the CPU `eval` — `full = p·scale + (ox, oy)` then `p' = p + (full − p)·falloff`
/// — in the SAME multiply/add order, so parity holds within GPU-FMA ε (the ε
/// gate). No `applicable`: a plain affine covers the whole param space (no enum,
/// no partial coverage). `ReadWriteExisting` on `P` mirrors the CPU's
/// pattern-match — a stream WITHOUT a `P` column passes through untouched, so
/// absence means the same thing on both paths (the falloff read materializes
/// its `1.0` identity when absent = full effect).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let xf_f = read_falloff(i);\n\
        let xf_p = read_P(i);\n\
        // The pivot folded into the offset, the same expression and the same\n\
        // order as the CPU's `folded_offset` -- including its zero shortcut, so\n\
        // the neutral is structural on both paths and not an IEEE argument.\n\
        // O link de corrente, lido como no CPU (`authored_factors`): `>= 0.5`, e ligado os\n\
        // dois eixos sao o MESMO numero.\n\
        let xf_sx = params.scale;\n\
        let xf_sy = select(params.scale_y, params.scale, params.uniform >= 0.5);\n\
        var xf_ox = params.offset_x;\n\
        var xf_oy = params.offset_y;\n\
        if (params.pivot_x != 0.0 || params.pivot_y != 0.0) {\n\
            xf_ox = params.offset_x + params.pivot_x * (1.0 - xf_sx);\n\
            xf_oy = params.offset_y + params.pivot_y * (1.0 - xf_sy);\n\
        }\n\
        let xf_full = vec2<f32>(\n\
            xf_p.x * xf_sx + xf_ox,\n\
            xf_p.y * xf_sy + xf_oy);\n\
        write_P(i, vec2<f32>(\n\
            xf_p.x + (xf_full.x - xf_p.x) * xf_f,\n\
            xf_p.y + (xf_full.y - xf_p.y) * xf_f));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWriteExisting,
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
    ],
    params: &[
        "scale", "uniform", "scale_y", "offset_x", "offset_y", "pivot_x", "pivot_y",
    ],
    count_law: None,
    variant_by_param: None,
    // ⚠️ The device handles the origin and the typed point -- both are just
    // numbers -- and RECUSES the centroid, which is a REDUCTION over the whole
    // stream and not a per-element map. Refusing is the honest answer (the
    // ADR-0155 precedent, and `motion.look_at` does the same for its Object and
    // Cursor modes): the named cost is that a layout pivoting on its own centre
    // loses GPU residency for this node. The `reduce -> broadcast -> map` channel
    // the deformers use is what would lift it, and that is a wave, not a line.
    applicable: Some(|p| Pivot::of(p("pivot_mode")) != Pivot::Centroid),
};

struct MotionTransform;

impl NodeOp for MotionTransform {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Os dois fatores — ver [`UNIFORM`]. Ligado (o default), `sx` e `sy` são o MESMO `f32`.
        let (sx, sy) = authored_factors(ctx.param("scale"), ctx.param(UNIFORM), ctx.param(SCALE_Y));
        let (ox, oy) = (ctx.param("offset_x"), ctx.param("offset_y"));
        let mode = Pivot::of(ctx.param("pivot_mode"));
        let typed = [ctx.param("pivot_x"), ctx.param("pivot_y")];
        let out = {
            let input = ctx.input(0);
            // Resolved ONCE, outside the loop: a pivot is a property of the
            // layout, not of an element. A centroid a stream cannot supply
            // (no `P`, or empty) falls back to the origin — the transform an
            // artist can still see, rather than a NaN that removes the art.
            let pivot = match mode {
                Pivot::WorldOrigin => [0.0, 0.0],
                Pivot::Point => typed,
                Pivot::Centroid => centroid(input).unwrap_or([0.0, 0.0]),
            };
            let (ox, oy) = folded_offset(sx, sy, ox, oy, pivot);
            // The port type guarantees `P` is `Vec2`; a `P` of any other dim is
            // an upstream node-author bug. Assert it loudly in debug/test rather
            // than silently passing it through untransformed (which would emit
            // geometry the params claim to have moved).
            debug_assert!(
                !matches!(input.get("P"), Some(c) if !matches!(c, Column::Vec2(_))),
                "motion.transform expects `P` to be a Vec2 column (port type guarantees it)"
            );
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                match (name.as_str(), col) {
                    ("P", Column::Vec2(v)) => {
                        // Pure per-instance map → parallel above the threshold
                        // (bit-identical, no reduction). GPU/M5 Fase 0.
                        let t: Vec<[f32; 2]> = par_build(v.len(), |i| {
                            xform_masked(v[i], sx, sy, ox, oy, falloff_at(input, i))
                        });
                        out.set("P", Column::Vec2(t));
                    }
                    // Every other column is per-element data this node does not
                    // touch — passed through unchanged (count is preserved).
                    _ => out.set(name.clone(), col.clone()),
                }
            }
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTransform))?;
    // M1.R1 — UI metadata for the card (a spatial modifier → blue transform,
    // rounded-rect silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Transform",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // M1.P1 — param rows: uniform scale + signed offsets.
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

mod ui;
use ui::{PARAM_GATES, PARAM_HARD_MIN, PARAM_HINTS, PARAM_UNITS};

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances at (1,1),(2,2) with a size column to verify passthrough.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.transform.test.src"),
        name: "motion.transform.test.src",
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
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]))
                    .with("size", Column::Vec2(vec![[5.0, 5.0], [5.0, 5.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTransform),
                _ => None,
            }
        }
    }

    #[test]
    fn scales_and_offsets_p_passes_size_through() {
        // default scale=1, offset=0 → P unchanged; assert passthrough + identity.
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.src");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, xf, 0.0).unwrap();
        assert_eq!(out[0].as_stream().count(), 2);
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 1.0], [2.0, 2.0]]),
            _ => panic!("P"),
        }
        // size carried through unchanged
        match out[0].as_stream().get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[5.0, 5.0], [5.0, 5.0]]),
            _ => panic!("size"),
        }
    }

    #[test]
    fn per_instance_overrides_drive_the_affine_through_the_cook() {
        // The real authoring path: override scale + offset on the instance and
        // see P transformed end-to-end (the cook only fed identity defaults
        // before per-instance params landed). src emits (1,1),(2,2).
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.src");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(xf, "scale", 2.0);
        g.set_param(xf, "offset_x", 10.0);
        g.set_param(xf, "offset_y", 1.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, xf, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // (1,1)*2+(10,1) = (12,3) ; (2,2)*2+(10,1) = (14,5)
            Column::Vec2(v) => assert_eq!(v, &vec![[12.0, 3.0], [14.0, 5.0]]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn apply_xform_scales_then_offsets() {
        // Unit-proves the `p*scale + offset` arithmetic directly with
        // non-identity values (the end-to-end override path is covered by
        // `per_instance_overrides_drive_the_affine_through_the_cook`).
        assert_eq!(apply_xform([2.0, 3.0], 2.0, 2.0, 1.0, -1.0), [5.0, 5.0]);
        assert_eq!(apply_xform([0.0, 0.0], 10.0, 10.0, 4.0, 7.0), [4.0, 7.0]);
        assert_eq!(apply_xform([1.0, 1.0], 0.0, 0.0, 0.0, 0.0), [0.0, 0.0]); // collapse
        assert_eq!(apply_xform([-2.0, 5.0], 1.0, 1.0, 0.0, 0.0), [-2.0, 5.0]); // identity
    }

    #[test]
    fn xform_masked_blends_by_falloff() {
        // f=1 → full transform; f=0 → unmoved; f=0.5 → halfway between.
        assert_eq!(
            xform_masked([1.0, 1.0], 2.0, 2.0, 10.0, 0.0, 1.0),
            [12.0, 2.0]
        );
        assert_eq!(
            xform_masked([1.0, 1.0], 2.0, 2.0, 10.0, 0.0, 0.0),
            [1.0, 1.0]
        );
        // full = (12, 2); midpoint with (1,1) = (6.5, 1.5).
        assert_eq!(
            xform_masked([1.0, 1.0], 2.0, 2.0, 10.0, 0.0, 0.5),
            [6.5, 1.5]
        );
    }

    // Source with a falloff column so the mask is exercised end to end.
    static MASK_SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.transform.test.masksrc"),
        name: "motion.transform.test.masksrc",
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
    struct MaskSrc;
    impl NodeOp for MaskSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &MASK_SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.0])),
            );
        }
    }
    struct MaskOps;
    impl OpResolver for MaskOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == MASK_SRC_MAN.id => Some(&MaskSrc),
                t if t == MANIFEST.id => Some(&MotionTransform),
                _ => None,
            }
        }
    }

    #[test]
    fn falloff_column_masks_the_transform_per_instance() {
        // scale 2 + offset (10,1): instance 0 (falloff 1) fully transforms;
        // instance 1 (falloff 0) is untouched — the focus field gates the move,
        // consistent with orbit/stagger/oscillator/wiggle.
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.masksrc");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(xf, "scale", 2.0);
        g.set_param(xf, "offset_x", 10.0);
        g.set_param(xf, "offset_y", 1.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &MaskOps, xf, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // (1,1)*2+(10,1) = (12,3) at full falloff ; (2,2) unmoved at falloff 0.
            Column::Vec2(v) => assert_eq!(v, &vec![[12.0, 3.0], [2.0, 2.0]]),
            _ => panic!("P"),
        }
    }
}

#[cfg(test)]
#[path = "pivot_tests.rs"]
mod pivot_tests;

#[cfg(test)]
#[path = "uniform_tests.rs"]
mod uniform_tests;
