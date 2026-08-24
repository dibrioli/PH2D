#![forbid(unsafe_code)]
//! `motion.grid` — a Motion **generator**: emits a `rows × cols` lattice of
//! instances **centered on the origin**, on the `P` (Vec2) attribute, with
//! independent `gap_x` / `gap_y` spacing. Also emits per-instance `Index` (`0..n`)
//! and `Count` (`n`) scalar columns — the stable identity downstream palette /
//! ramp / normalized effects address (Cavalry/Houdini `@ptnum`/`@numpt`). No
//! inputs. Pure (combinational). The stream convention is `ph2d-eval-motion`'s.
//!
//! Centering (vs. a corner origin) makes every downstream scale / rotate / circle
//! falloff act symmetrically about the middle of the grid, not a corner.
//!
//! Params (read via `ctx.param` — per-instance override else the manifest default
//! shown): `rows` (3), `cols` (3), `gap_x` (1.0), `gap_y` (1.0). `rows`/`cols` are
//! read as element counts via [`param_as_count`] (non-finite/negative → 0) and
//! the `rows × cols` product is capped at [`RECOMMENDED_MAX_ELEMENTS`], so no
//! param value can overflow the allocation.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
// ⚠️ **`Region`, e não `Domain`** — este arquivo já importa o `Domain` do `port`, que
// diz em que PLANO de dados a porta vive.
use ph2d_motion_region::carve;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.grid"),
    name: "motion.grid",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "rows",
            default: 3.0,
        },
        ParamSpec {
            name: "cols",
            default: 3.0,
        },
        ParamSpec {
            name: "gap_x",
            default: 1.0,
        },
        ParamSpec {
            name: "gap_y",
            default: 1.0,
        },
        // **A FORMA** (doc 89, folha 01) — `Rect` é a grade de sempre, ao bit.
        ParamSpec {
            name: ph2d_motion_region::SHAPE,
            default: 0.0,
        },
        // ⚠️ **O `fill = Shell` do C4D é este knob**, e não um param próprio: uma casca
        // É um anel de buraco grande, e `inner` intermédios dão espessuras que o par
        // *Solid/Shell* não sabe exprimir. Ver `ph2d_motion_region`.
        ParamSpec {
            name: ph2d_motion_region::INNER,
            default: 0.5,
        },
    ],
    // `lowerings` describes the `ph2d-expr` path only, and the grid is a
    // structural *generator* (it produces an element count), not a per-element
    // expression map — so `Cpu` stays. The GPU compute lowering exists, but it
    // rides the ADR-0126 SIDE CHANNEL (`register_gpu_kernel` below, `GPU_KERNEL`),
    // never this frozen manifest field.
    lowerings: &[LoweringKind::Cpu],
};

/// **A REGIÃO desta grade** — a forma inscrita na extensão que os pontos de facto
/// ocupam, `(cols−1)·gap_x` por `(rows−1)·gap_y`.
///
/// ⚠️ **A extensão é a dos PONTOS, não a das células.** Usar `cols·gap_x` poria a
/// fronteira meia célula para fora e o círculo nunca tocaria a coluna de fora — o
/// artista veria um disco que não encosta na grade que o gerou.
fn grid_region(
    rows: usize,
    cols: usize,
    gap_x: f32,
    gap_y: f32,
    shape: f32,
    inner: f32,
) -> ph2d_motion_region::Region {
    let w = (cols.max(1) as f32 - 1.0) * gap_x;
    let h = (rows.max(1) as f32 - 1.0) * gap_y;
    ph2d_motion_region::Region::of(shape, w, h, inner)
}

/// Build the `rows × cols` position grid (row-major), **centered on the origin**
/// with independent `gap_x` / `gap_y`, capping the element count at `max` so a
/// pathological `rows × cols` can never overflow the allocation. Pure and
/// `max`-parameterized so the cap is testable without allocating the full budget.
/// The emitted count is `positions.len()`. Centering uses the full `rows`/`cols`
/// (the cap is a pathological guard; normal grids are never capped).
fn build_grid(rows: usize, cols: usize, gap_x: f32, gap_y: f32, max: usize) -> Vec<[f32; 2]> {
    let count = rows.saturating_mul(cols).min(max);
    // Lattice midpoint at (0,0): shift each index by half the span.
    let cx = (cols as f32 - 1.0) * 0.5;
    let cy = (rows as f32 - 1.0) * 0.5;
    // Row-major: element `i` is cell `(r = i/cols, c = i%cols)`. The old nested
    // push produced exactly this order (the cap keeps the first `count` cells),
    // so the parallel build is bit-identical. `count == 0` when `cols == 0`, so
    // the div/mod never runs on a zero divisor. GPU/M5 Fase 0.
    par_build(count, |i| {
        let (r, c) = (i / cols, i % cols);
        [(c as f32 - cx) * gap_x, (r as f32 - cy) * gap_y]
    })
}

/// GPU compute kernel (GPU/M5 Fase 1, ADR-0126) — the same row-major centered
/// lattice as [`build_grid`], one element per invocation. `source_count`
/// mirrors the CPU's `param_as_count` + product cap EXACTLY (the dispatch size
/// must equal the CPU stream count, or parity is dead on arrival); the body
/// re-derives `cols`/`cx`/`cy` with the same floor/clamp so the geometry
/// matches within float ULPs. Registered on the side — the frozen `MANIFEST`
/// (and its `lowerings: Cpu`, which describes the `ph2d-expr` path, not this
/// side channel) is untouched.
const GPU_KERNEL: GpuKernel = GpuKernel {
    // Matches `build_grid`: element `i` = cell `(r = i/cols, c = i%cols)`,
    // centered via `((cols|rows) - 1) / 2`. The floor/clamp mirrors
    // `param_as_count` (16777216 = RECOMMENDED_MAX_ELEMENTS) so a pathological
    // param yields the same lattice the CPU builds. `cols == 0` never reaches
    // the div/mod: `source_count` is 0, so nothing dispatches.
    wgsl: "\
        let colsf = min(max(floor(params.cols), 0.0), 16777216.0);\n\
        let rowsf = min(max(floor(params.rows), 0.0), 16777216.0);\n\
        let cols = u32(colsf);\n\
        let cx = (colsf - 1.0) * 0.5;\n\
        let cy = (rowsf - 1.0) * 0.5;\n\
        let r = i / cols;\n\
        let c = i % cols;\n\
        write_P(i, vec2<f32>((f32(c) - cx) * params.gap_x, (f32(r) - cy) * params.gap_y));\n\
        write_Index(i, f32(i));\n\
        write_Count(i, f32(params.count));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Index",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Count",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["rows", "cols", "gap_x", "gap_y"],
    // A grid is static: its count moves with neither the playhead nor an input
    // (it has none). Those fields of `CountLawCtx` exist for the stateless
    // emitter (ADR-0130) and for `value.lfo`, not for this.
    count_law: Some(|c| {
        let rows = param_as_count((c.param)("rows"), RECOMMENDED_MAX_ELEMENTS);
        let cols = param_as_count((c.param)("cols"), RECOMMENDED_MAX_ELEMENTS);
        SourceWindow::of_count(rows.saturating_mul(cols).min(RECOMMENDED_MAX_ELEMENTS))
    }),
    variant_by_param: None,
    // ⛔ **FRONTEIRA NOMEADA: uma grade RECORTADA não tem `count_law`.**
    //
    // A contagem de pontos de uma rede dentro de um círculo é o **problema do círculo
    // de Gauss** — não existe forma fechada, e uma `count_law` é obrigada a devolver a
    // largura ANTES de o kernel correr (ela só recebe params, nunca dados: pedir o
    // número ao device seria um readback, medido-negativo). Recusar é a mesma cerca que
    // o `probability` do `motion.emitter` já paga pelo mesmo mecanismo — *um portão que
    // torna a contagem dependente de DADOS sai do caminho do device.*
    //
    // ⚠️ **O default fica no device.** Só `shape != Rect` cai para a CPU, e é por isso
    // que a fronteira custa zero a todo documento que não usa a forma nova. A saída
    // certa, quando alguém a quiser, é o prefix-sum que o `motion.cull` já tem — ligá-lo
    // a um GERADOR é uma wave própria.
    applicable: Some(|param| param(ph2d_motion_region::SHAPE).round() as i32 == 0),
};

struct MotionGrid;

impl NodeOp for MotionGrid {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // `rows`/`cols` come from `f32` params; convert *totally* (a non-finite
        // or negative override yields 0, huge values clamp) and cap the product
        // so a corrupt scene value can never overflow the allocation.
        let rows = param_as_count(ctx.param("rows"), RECOMMENDED_MAX_ELEMENTS);
        let cols = param_as_count(ctx.param("cols"), RECOMMENDED_MAX_ELEMENTS);
        let (gap_x, gap_y) = (ctx.param("gap_x"), ctx.param("gap_y"));
        let positions = build_grid(rows, cols, gap_x, gap_y, RECOMMENDED_MAX_ELEMENTS);
        // **A FORMA RECORTA** (doc 89, folha 01) — ver `carve`.
        let positions = carve(
            positions,
            &grid_region(
                rows,
                cols,
                gap_x,
                gap_y,
                ctx.param(ph2d_motion_region::SHAPE),
                ctx.param(ph2d_motion_region::INNER),
            ),
        );
        let n = positions.len();
        // Per-instance identity: `Index` (0..n) + `Count` (n) — the stable handle
        // downstream palette / ramp / normalized effects read. `clone` replicates
        // them per copy, so each copy is a self-contained indexed set.
        let index: Vec<f32> = par_build(n, |i| i as f32);
        let count = vec![n as f32; n];
        ctx.emit(
            Stream::new(n)
                .with("P", Column::Vec2(positions))
                .with("Index", Column::Scalar(index))
                .with("Count", Column::Scalar(count)),
        );
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionGrid))?;
    // M1.R1 — UI metadata for the card (a generator → green source, widening
    // trapezoid silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Grid",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    // M1.P1 — param rows: whole-number row/column counts, continuous per-axis gap.
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5 Fase 1 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `rows`/`cols` — MEDIDO** (doc 88 A1 · §0), enquanto o slider fica nos 20 que
/// cobrem a autoria confortável. A grade é um laço linear e o cook mediu, pela porta do produto
/// (`measure_the_count_ceiling`, `cols = 1` para o eixo ser o que a linha nomeia):
///
/// | instâncias | cook |
/// |---|---|
/// | 100.000 | 0,542 ms |
/// | 400.000 | 1,466 ms |
/// | **1.000.000** | **3,661 ms** |
///
/// Um milhão de pontos custa **22% de um quadro de 60 fps** — 50.000× o que o slider alcança. O
/// teto é o número que a medição deu, e não uma potência bonita acima dela.
///
/// ⚠️ **Este é um freio ERGONÔMICO por eixo, não uma garantia de recurso** — o precedente exato do
/// `rate` do emitter: as instâncias são `rows × cols`, e **nenhum cap estático sobre um FATOR
/// exprime um limite sobre o PRODUTO**. Quem quiser a garantia tem de a pôr onde o produto existe.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "rows",
        max: 1_000_000.0,
    },
    ParamHardMax {
        param: "cols",
        max: 1_000_000.0,
    },
];

/// Param UI hints (M1.P1) for the grid (editable range + widget + label).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 1.0,
        max: 20.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "cols",
        label: "Columns",
        min: 1.0,
        max: 20.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "gap_x",
        label: "Gap X",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gap_y",
        label: "Gap Y",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: ph2d_motion_region::SHAPE,
        label: "Shape",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: ph2d_motion_region::SHAPE_LABELS,
        },
    },
    ParamUiHint {
        param: ph2d_motion_region::INNER,
        label: "Hole",
        min: 0.0,
        max: 0.98,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "gap_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "gap_y",
        unit: ParamUnit::Length,
    },
];

/// O buraco só existe no anel.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[ph2d_node_registry::ParamGate {
    param: ph2d_motion_region::INNER,
    when: ph2d_motion_region::SHAPE,
    values: &[ph2d_motion_region::SHAPE_RING],
}];

#[cfg(test)]
mod region_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionGrid as &dyn NodeOp)
        }
    }

    #[test]
    fn emits_default_3x3_grid_centered_with_index_and_count() {
        let mut g = Graph::new();
        let n = g.add_node("motion.grid");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 9); // 3×3
        match s.get("P").unwrap() {
            Column::Vec2(v) => {
                // Centered on the origin: spans [-1,1] × [-1,1] at gap 1.0.
                assert_eq!(v[0], [-1.0, -1.0]);
                assert_eq!(v[1], [0.0, -1.0]); // col 1
                assert_eq!(v[3], [-1.0, 0.0]); // row 1
                assert_eq!(v[8], [1.0, 1.0]); // last
            }
            _ => panic!("P must be Vec2"),
        }
        // Per-instance identity columns.
        match s.get("Index").unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v, &vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
            }
            _ => panic!("Index must be Scalar"),
        }
        match s.get("Count").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![9.0; 9]),
            _ => panic!("Count must be Scalar"),
        }
    }

    #[test]
    fn per_instance_override_changes_the_grid() {
        // The headline of per-instance params: override `rows` to 2 → 2×3 = 6
        // points (vs the 3×3 = 9 default), proven through the real cook path.
        let mut g = Graph::new();
        let n = g.add_node("motion.grid");
        g.set_param(n, "rows", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        assert_eq!(out[0].as_stream().count(), 6);
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => {
                assert_eq!(v.len(), 6);
                // 2×3 centered: cx=1, cy=0.5 → last = (2-1, 1-0.5) = [1, 0.5].
                assert_eq!(v[5], [1.0, 0.5]);
            }
            _ => panic!("P must be Vec2"),
        }
    }

    #[test]
    fn independent_gaps_make_a_non_square_lattice() {
        // gap_x 2, gap_y 1 → wider than tall. Centered 3×3: first row y = -1,
        // x steps by 2 from -2.
        let g = build_grid(3, 3, 2.0, 1.0, 64);
        assert_eq!(g[0], [-2.0, -1.0]);
        assert_eq!(g[1], [0.0, -1.0]);
        assert_eq!(g[2], [2.0, -1.0]);
    }

    #[test]
    fn build_grid_caps_pathological_product_at_max() {
        // 100 × 100 = 10_000 requested, but max is 4 → exactly 4 emitted (the
        // emit invariant); row-major (the four share a row, stepping by gap_x).
        let g = build_grid(100, 100, 1.0, 1.0, 4);
        assert_eq!(g.len(), 4);
        assert_eq!(g[0][1], g[3][1], "same row (equal y)");
        assert!((g[1][0] - g[0][0] - 1.0).abs() < 1e-6, "x steps by gap_x");
    }

    #[test]
    fn build_grid_zero_dim_is_empty() {
        assert!(build_grid(0, 5, 1.0, 1.0, 64).is_empty());
        assert!(build_grid(5, 0, 1.0, 1.0, 64).is_empty());
    }
}
