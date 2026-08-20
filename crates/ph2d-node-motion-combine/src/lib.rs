#![forbid(unsafe_code)]
//! `motion.combine` — **concatenate several instance streams into one**: the Houdini
//! "Merge" SOP / the Blender "Join Geometry" (Motion Nodes M1, streams — doc 01 §1.7 /
//! doc 30). The first node that **merges** — until now every graph was a single linear
//! chain; this lets two distributions (a grid *and* a ring) become one stream that draws
//! together. The companion of `motion.mixer` (which blends element-wise); combine just
//! stacks them end to end.
//!
//! **Algorithm — column union with zero-fill.** The output count is the sum of the
//! non-empty inputs' counts. For every column name that appears in **any** input the
//! rows are concatenated in input order; an input that lacks the column contributes
//! zeros (the Merge convention — a missing attribute reads as its default), so the
//! streams line up. Up to four inputs; unconnected ones are empty and skipped.
//! Transcendental-free (HR-5): pure copying. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{GpuKernel, StreamOp};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.combine"),
    name: "motion.combine",
    inputs: &[
        PortSpec {
            name: "in0",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in1",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in2",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in3",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // **A RENUMERAÇÃO** (doc 89 folha 08). `0` = as colunas de identidade viajam
    // verbatim, que é o que sempre aconteceu; `1` = a lista junta ganha um `Index`
    // e um `Count` honestos. Ver [`REINDEX`].
    params: &[ParamSpec {
        name: "reindex",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// As duas colunas de IDENTIDADE de um stream — quem é este elemento na lista, e
/// quantos elementos a lista tem.
const INDEX: &str = "Index";
const COUNT: &str = "Count";

/// **A RENUMERAÇÃO** (doc 89 folha 08 — a célula do `reindex`).
///
/// Medido antes desta wave (`measure_stream_join_defects`): `grid(9) + grid(4)` devolve
/// **13 linhas** com `Index = [0..8, 0..3]` e `Count = [9×9, 4×4]` — as duas colunas
/// **mentem**, e todo efeito dirigido por índice a jusante (uma rampa de cor, um
/// stagger) lê a lista como se fossem duas. Os irmãos `motion.clone` e
/// `motion.duplicator` renumeram de propósito (*"so a downstream ramp reads one
/// 0..total run"*); este não.
///
/// ⚠️ **O DEFAULT É `0`, e isso NÃO é a escolha certa — é a escolha conservadora.** A
/// referência de origem (MiniCavalry `combineStreams`) tem `reindex` com default
/// **`true`**, e documenta o gotcha do contrário. Ligá-lo por omissão mudaria arte já
/// autorada, o que nesta casa é decisão do Enio e não do agente — o param entra
/// desligado, e a pergunta vai com o smoke.
///
/// ⚠️ **E ele RECUSA o device quando ligado**, pela mesma porta que os dois modos do
/// `value.reduce` usam (`applicable`): a concatenação no device é um `StreamOp::Concat`
/// que o sequenciador executa com `copy_buffer_to_buffer`, sem shader — renumerar ali
/// seria uma segunda implementação da mesma lei, e duas implementações de uma lei é
/// como elas divergem. Desligado (o default) nada recua.
const REINDEX: &str = "reindex";

/// O kernel deste nó: o passthrough de sempre, **mais a recusa** quando a
/// renumeração está ligada.
const GPU_KERNEL: GpuKernel = GpuKernel {
    applicable: Some(|p| p(REINDEX) < 0.5),
    ..GpuKernel::PASSTHROUGH
};

/// A cloned snapshot of one input (so several inputs can be held at once).
struct Snap {
    count: usize,
    cols: Vec<(String, Column)>,
}

impl Snap {
    fn column(&self, name: &str) -> Option<&Column> {
        self.cols.iter().find(|(n, _)| n == name).map(|(_, c)| c)
    }
}

fn snapshot(s: &Stream) -> Snap {
    Snap {
        count: s.count(),
        cols: s.columns().map(|(n, c)| (n.clone(), c.clone())).collect(),
    }
}

/// Concatenate the column `name` across `snaps`, filling zeros where an input lacks it.
/// `proto` fixes the variant (the first input that has the column).
fn concat(proto: &Column, snaps: &[Snap], name: &str) -> Column {
    macro_rules! build {
        ($variant:path, $zero:expr) => {{
            let mut v = Vec::new();
            for s in snaps {
                match s.column(name) {
                    Some($variant(d)) => v.extend_from_slice(d),
                    _ => v.extend(std::iter::repeat($zero).take(s.count)),
                }
            }
            $variant(v)
        }};
    }
    match proto {
        Column::Scalar(_) => build!(Column::Scalar, 0.0),
        Column::Vec2(_) => build!(Column::Vec2, [0.0; 2]),
        Column::Vec3(_) => build!(Column::Vec3, [0.0; 3]),
        Column::Vec4(_) => build!(Column::Vec4, [0.0; 4]),
    }
}

/// Concatenate the non-empty snapshots into one stream (column union, zero-filled).
fn combine(snaps: &[Snap]) -> Stream {
    let total: usize = snaps.iter().map(|s| s.count).sum();
    let mut out = Stream::new(total);
    // Ordered unique column names, each with the first-seen variant as prototype.
    let mut seen: Vec<String> = Vec::new();
    for s in snaps {
        for (name, _) in &s.cols {
            if !seen.iter().any(|n| n == name) {
                seen.push(name.clone());
            }
        }
    }
    for name in &seen {
        let proto = snaps.iter().find_map(|s| s.column(name)).unwrap();
        out.set(name.clone(), concat(proto, snaps, name));
    }
    out
}

/// **Reescreve as duas colunas de identidade** para a lista JUNTA: `Index = 0..n−1` e
/// `Count = n` em todas as linhas.
///
/// ⚠️ **Escreve as duas mesmo que nenhuma entrada as trouxesse**, e é isso que o torna
/// uma renumeração e não um remendo: com `reindex` ligado o artista pediu identidade
/// para a lista junta, e uma lista sem `Index` é uma em que o nó a jusante inventa a
/// dele a partir da posição — a mesma resposta por acidente, até alguém pôr um `sort`
/// no meio.
fn reindex(out: &mut Stream) {
    let n = out.count();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let total = n as f32;
    out.set(INDEX, Column::Scalar(idx));
    out.set(COUNT, Column::Scalar(vec![total; n]));
}

struct MotionCombine;

impl NodeOp for MotionCombine {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Snapshot each input, one at a time (each borrow ends at the clone), keeping the
        // non-empty ones in port order.
        let mut snaps: Vec<Snap> = Vec::new();
        for k in 0..4u16 {
            let s = snapshot(ctx.input(k as usize));
            if s.count > 0 {
                snaps.push(s);
            }
        }
        let mut out = combine(&snaps);
        if ctx.param(REINDEX) >= 0.5 {
            reindex(&mut out);
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionCombine))?;
    // ADR-0136: concatenation is `copy_buffer_to_buffer` + zero-fill in the
    // sequencer — no shader, so the kernel is the passthrough (it makes the
    // plan claim the node; the `StreamOp::Concat` is what runs).
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(
        MANIFEST.id,
        StreamOp::Concat {
            ports: &[0, 1, 2, 3],
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Combine",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

/// ⚠️ **Um `Toggle`, e não um slider**: a renumeração acontece ou não acontece — meia
/// renumeração não quer dizer nada, e um slider convidaria a procurar o meio.
static PARAM_HINTS: &[ph2d_node_registry::ParamUiHint] = &[ph2d_node_registry::ParamUiHint {
    param: REINDEX,
    label: "Reindex",
    min: 0.0,
    max: 1.0,
    step: 1.0,
    widget: ph2d_node_registry::ParamWidget::Toggle,
}];

#[cfg(test)]
mod tests {
    use super::*;

    fn snap_of(cols: Vec<(&str, Column)>) -> Snap {
        let count = cols
            .first()
            .map(|(_, c)| match c {
                Column::Scalar(v) => v.len(),
                Column::Vec2(v) => v.len(),
                Column::Vec3(v) => v.len(),
                Column::Vec4(v) => v.len(),
            })
            .unwrap_or(0);
        Snap {
            count,
            cols: cols.into_iter().map(|(n, c)| (n.to_string(), c)).collect(),
        }
    }

    /// Concatenation sums the counts and lays the rows end to end. FALSIFIED if it
    /// overwrote instead of appended.
    #[test]
    fn combine_sums_counts_and_concatenates() {
        let a = snap_of(vec![("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))]);
        let b = snap_of(vec![("P", Column::Vec2(vec![[9.0, 9.0]]))]);
        let out = combine(&[a, b]);
        assert_eq!(out.count(), 3, "2 + 1");
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [1.0, 0.0], [9.0, 9.0]]),
            _ => panic!("P"),
        }
    }

    /// A column present in only one input is zero-filled for the other (the Merge default
    /// convention), so every column ends at the full combined length.
    #[test]
    fn a_missing_column_is_zero_filled() {
        let a = snap_of(vec![
            ("P", Column::Vec2(vec![[0.0, 0.0]])),
            ("size", Column::Scalar(vec![0.7])),
        ]);
        let b = snap_of(vec![("P", Column::Vec2(vec![[1.0, 1.0]]))]); // no size
        let out = combine(&[a, b]);
        match out.get("size").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.7, 0.0], "b's size zero-filled"),
            _ => panic!("size"),
        }
    }

    /// Deterministic + cooks through the registry: two source nodes merge into one stream
    /// at the summed count.
    #[test]
    fn registers_and_merges_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};
        use ph2d_nodegraph::node::ParamSpec;

        const fn src(id: &'static str) -> NodeManifest {
            NodeManifest {
                id: NodeTypeId::of(id),
                name: id,
                inputs: &[],
                outputs: &[PortSpec {
                    name: "out",
                    ty: INST_VEC2,
                }],
                effect: Effect::Pure,
                clock: Clock::Frame,
                params: &[] as &[ParamSpec],
                lowerings: &[LoweringKind::Cpu],
            }
        }
        static SA: NodeManifest = src("motion.combine.test.a");
        static SB: NodeManifest = src("motion.combine.test.b");
        struct A;
        impl NodeOp for A {
            fn manifest(&self) -> &'static NodeManifest {
                &SA
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3])));
            }
        }
        struct B;
        impl NodeOp for B {
            fn manifest(&self) -> &'static NodeManifest {
                &SB
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[5.0, 5.0]; 2])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SA.id => Some(&A),
                    t if t == SB.id => Some(&B),
                    t if t == MANIFEST.id => Some(&MotionCombine),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let a = g.add_node("motion.combine.test.a");
        let b = g.add_node("motion.combine.test.b");
        let c = g.add_node("motion.combine");
        g.connect(Edge {
            from: (a, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (b, 0),
            to: (c, 1),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
        assert_eq!(out[0].as_stream().count(), 5, "3 + 2 merged");
    }
}

#[cfg(test)]
#[path = "reindex_tests.rs"]
mod reindex_tests;
