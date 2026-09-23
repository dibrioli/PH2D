#![forbid(unsafe_code)]
//! `motion.kaleidoscope` — **N-fold radial symmetry**: replicate a layout into
//! `segments` slices rotated around a pivot, optionally mirroring alternate slices — a
//! mandala / kaleidoscope generator (Motion Nodes M3, distributions — doc 01 §3 /
//! doc 26). The N-fold generalisation of `motion.mirror`, which is the 2-fold (D₁)
//! case: mirror reflects once, this rotates `segments` copies (and, with `reflect`,
//! mirrors every other one).
//!
//! **Algorithm — the orbit of the source under the dihedral group Dₙ.** For a source of
//! `n` elements and `segments = k`, the output has `k · n` elements: slice `s` is the
//! source rotated about `(pivot_x, pivot_y)` by `s · (1/k)` turn (plus a global `spin`).
//! With **rotational** symmetry (`reflect` off) that is the cyclic group Cₖ — `k` plain
//! rotated copies. With **reflected** symmetry (`reflect` on) every odd slice is first
//! mirrored (its local `y` negated), so adjacent slices meet as mirror images — the
//! dihedral group Dₖ, the true kaleidoscope look. Only the **position** `P` is
//! transformed; every other column (`size`, `tint`, `id`, …) is duplicated onto each
//! copy. Transcendental-free (HR-5): the rotation uses `cos_sin_cycles` — the parabolic
//! sine copied from `motion.orbit` — so no `sin`/`cos`; no `sqrt`. `Effect::Pure` (no
//! clock — the spin animation arrives through the value input).
//!
//! ## O que este nó NÃO faz, e onde isso mora (doc 89 folha 04, o padrão §9.2)
//!
//! ⚠️ **A CUNHA de origem** (AE *CC Kaleida* ▸ *Size*: um caleidoscópio real dobra a fonte
//! numa fatia antes de a repetir; nós repetimos a fonte **INTEIRA**, então uma fonte larga faz
//! as cópias invadirem as vizinhas). Isto é **composição, e a cadeia foi verificada peça por
//! peça**:
//!
//! ```text
//! field.radial_sweep   (a cunha angular → escreve `falloff`)
//!   → motion.cull      (`mode = 1` Falloff — mantém `falloff ≥ amount`)
//!     → motion.kaleidoscope
//! ```
//!
//! Os dois nós existem e casam. ⛔ **Não construa um `wedge` aqui:** o recorte é uma pergunta
//! sobre *que parte da fonte entra*, e o grafo já a responde antes de a fonte chegar — pô-la
//! dentro deste nó seria um segundo sítio a decidir a mesma coisa.
//!
//! ⚠️ **O SETOR** (as cópias num leque em vez do giro cheio — C4D Cloner *Radial*, Cavalry
//! Duplicator). ⛔ **RECUSADO, e desde 2026-08-24 a recusa tem ENDEREÇO:** um leque é trabalho
//! de um *cloner*, e o `motion.clone` ganhou o **modo `Radial`** (`Mode` · `Arc` · `Pivot`) —
//! `arc = 360` é exactamente a lei deste nó, e um `arc` menor reparte o setor. Um `start`/`end`
//! aqui daria à casa dois nós a disputar a mesma capacidade, e ainda por cima **este** é o que
//! não a sabe fazer: `spin` gira o padrão INTEIRO e um `motion.cull` a jusante **APAGA** cópias
//! (deixa buracos) em vez de as comprimir.
//!
//! ⚠️ *Até essa wave a recusa apontava para um dono que **não entregava** — nenhum nó do
//! catálogo dispunha cópias num arco. Uma recusa que delega para quem não faz é um adiamento
//! com cara de decisão, e é por isso que a capacidade foi construída antes de a célula fechar.*
//!
//! ⚠️ **O `falloff` é IGNORADO aqui, e isso está CERTO** (refutado por medição em 2026-08-23):
//! este nó não é um deformador, é um **REPLICADOR** (`n → k·n`), e a família dele lê `falloff`
//! zero por unanimidade (`mirror` · `clone` · `duplicator` · `kaleidoscope`). Uma máscara
//! por-elemento multiplica uma DEFORMAÇÃO; ela não tem significado sobre uma operação que muda
//! a CONTAGEM — `f = 0` poria as `k·n` cópias coincidentes, que não é a identidade em contagem.
//! Atenuar a k-ésima CÓPIA é outra feature, e o `motion.clone` exprime-a (`scale_taper` /
//! `rot_taper`).

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::StreamOp;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `spin` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Max slices (a bound on the fan-out — `segments · n` elements).
const MAX_SEGMENTS: i64 = 256;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.kaleidoscope"),
    name: "motion.kaleidoscope",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Global rotation of the whole pattern, in degrees (animatable). Optional:
        // unconnected reads as 0.
        PortSpec {
            name: "spin",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Number of rotational slices (the N of N-fold).
        ParamSpec {
            name: "segments",
            default: 6.0,
        },
        // 0 = rotational (Cₙ); 1 = mirrored (Dₙ, the kaleidoscope look).
        ParamSpec {
            name: "reflect",
            default: 1.0,
        },
        // **EM TORNO DE QUÊ** — o vocabulário é o de [`ph2d_nodegraph::pivot`], partilhado
        // pela família (ciclo 3, W1 — doc 106 §2.3). ⚠️ O default é `Point` e **não** o `0`
        // do `motion.transform`: este nó SEMPRE honrou o ponto digitado, então `Point` é o
        // que o deixa byte-idêntico — *o default é a lei da identidade de cada nó, não uma
        // propriedade do enum*. Com o pivô em `(0,0)` os dois valores dão o mesmo número, e
        // é a row visível que os separa: com `World Origin` o gate esconderia os dois
        // sliders que o artista já usa.
        ParamSpec {
            name: ph2d_nodegraph::pivot::PARAM,
            default: 1.0,
        },
        // Centre of symmetry (world units) — lido só no modo `Point`.
        ParamSpec {
            name: "pivot_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_y",
            default: 0.0,
        },
        // Apendado (doc 89 folha 05). `0` = o nó que sempre shipou.
        ParamSpec {
            name: "reindex",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Replicate `p` into `segments` slices about `pivot`, rotated by `spin_cycles`, with
/// every odd slice mirrored when `reflect`. Returns the `segments · n` positions
/// (slice-major: all of slice 0, then slice 1, …). A pure function.
fn kaleidoscope(
    p: &[[f32; 2]],
    segments: usize,
    reflect: bool,
    pivot: [f32; 2],
    spin_cycles: f32,
) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(segments * p.len());
    for s in 0..segments {
        let (c, sn) = cos_sin_cycles(s as f32 / segments.max(1) as f32 + spin_cycles);
        let mirror = reflect && s % 2 == 1;
        for q in p {
            let lx = q[0] - pivot[0];
            // Odd slices are mirrored across the local x-axis so neighbours meet as
            // mirror images (the dihedral fold).
            let ly = if mirror {
                -(q[1] - pivot[1])
            } else {
                q[1] - pivot[1]
            };
            out.push([lx * c - ly * sn + pivot[0], lx * sn + ly * c + pivot[1]]);
        }
    }
    out
}

/// **A RENUMERAÇÃO** (doc 89 folha 05) — `0` (default) mantém as colunas de
/// identidade que sempre saíram daqui.
///
/// ⚠️ **MEDIDO**: uma grelha 3×3 (`Index = 0..8`, `Count = 9`) sai deste nó com
/// **n = 54** e o mesmo `Index` repetido **seis** vezes, `Count = 9` — as duas
/// colunas descrevem a lista de ANTES. É a irmã exacta da linha do `motion.mirror`,
/// e o default é `0` pela mesma razão de produto: o `Index` repetido é o que faz
/// cada FATIA ler a rampa inteira, que é plausivelmente o que se quer de um
/// caleidoscópio. Ligar é pedir *"uma lista só"*.
const REINDEX: &str = "reindex";

/// As duas colunas de identidade — os mesmos nomes do `motion.sort`/`combine`.
const INDEX: &str = "Index";
const COUNT: &str = "Count";

/// Reescreve as duas colunas para a lista replicada (`Index = 0..segments·n−1`,
/// `Count = segments·n`), mesmo que nenhuma entrada as trouxesse — a contagem
/// MUDOU, então um `Count` herdado mente. Ver [`REINDEX`].
fn reindex(out: &mut Stream) {
    let n = out.count();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let total = n as f32;
    out.set(INDEX, Column::Scalar(idx));
    out.set(COUNT, Column::Scalar(vec![total; n]));
}

/// Duplicate a column into `segments` copies (`[a, b] → [a, b, a, b, …]`).
fn dup_n(col: &Column, segments: usize) -> Column {
    fn rep<T: Clone>(v: &[T], n: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(v.len() * n);
        for _ in 0..n {
            out.extend_from_slice(v);
        }
        out
    }
    match col {
        Column::Scalar(v) => Column::Scalar(rep(v, segments)),
        Column::Vec2(v) => Column::Vec2(rep(v, segments)),
        Column::Vec3(v) => Column::Vec3(rep(v, segments)),
        Column::Vec4(v) => Column::Vec4(rep(v, segments)),
    }
}

struct MotionKaleidoscope;

impl NodeOp for MotionKaleidoscope {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let segments = (ctx.param("segments").round() as i64).clamp(1, MAX_SEGMENTS) as usize;
        // ⭐⭐⭐ **O TECTO É SOBRE O PRODUTO, e é por isso que ele não é o `MAX_SEGMENTS`** (ordem
        // do dono, 2026-09-21: *«nenhum deles pode gerar mais de 16384 objetos»*).
        //
        // ⚠️ Este nó MULTIPLICA: a saída é `segments × entrada`. Um tecto sobre o factor — que é o
        // que o `MAX_SEGMENTS` é — **não exprime um limite sobre o produto**, e a lei está escrita
        // por extenso no cabeçalho do `motion.grid` desde antes desta wave. ⇒ com uma entrada já no
        // tecto, `segments` cai para `1` e o nó passa a ser a identidade em contagem.
        let entrada = ctx.input(0).count().max(1);
        let segments = segments
            .min(ph2d_nodegraph::node::MAX_INSTANCIAS_POR_NO / entrada)
            .max(1);
        let reflect = ctx.param("reflect").round() as i64 != 0;
        let mode = ph2d_nodegraph::pivot::PivotMode::of(ctx.param(ph2d_nodegraph::pivot::PARAM));
        let typed = [ctx.param("pivot_x"), ctx.param("pivot_y")];
        let spin = match ctx.input(1).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        // ⚠️ O centroide é o do que ENTROU (`n` linhas), não o da saída de `segments · n`:
        // o segundo seria a média de uma figura que este nó ainda não desenhou, e o kernel
        // reduz sobre a porta 0 pela mesma razão.
        let pivot = mode.resolve(typed, &p);
        let positions = kaleidoscope(&p, segments, reflect, pivot, spin / 360.0);
        // Every column is duplicated onto each slice; only `P` is transformed.
        let mut out = Stream::new(positions.len());
        for (name, col) in input.columns() {
            if name == "P" {
                continue;
            }
            out.set(name.clone(), dup_n(col, segments));
        }
        out.set("P", Column::Vec2(positions));
        if ctx.param(REINDEX) >= 0.5 {
            reindex(&mut out);
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionKaleidoscope))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.motion.kaleidoscope.name",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5 (ADR-0136): a count-changing SourceRows kernel — the first that
    // READS its template (via `ColumnAccess::SourceRead`). Side metadata on the
    // registry; the frozen node contract is untouched.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(MANIFEST.id, StreamOp::SourceRows { port: 0 });
    // Ciclo 3 W1: o par `Sum(v.x)`/`Sum(v.y)` da porta do pivô — é o que faz o modo
    // `Centroid` correr no dispositivo em vez de derrubar a cadeia para a CPU.
    reg.register_reduces(MANIFEST.id, ph2d_nodegraph::pivot::CENTROID_REDUCES);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `segments` — é o CLAMP DO KERNEL** (doc 88 A1), enquanto o slider fica nos 64.
///
/// ⚠️ O `eval` **e** a lei de contagem clampam em [`MAX_SEGMENTS`], então um teto digitável acima
/// dele daria uma caixa que aceita 1.000 e um produto que entrega 256 — a forma de controle morto
/// que aceita e não avisa. Medido pela porta do produto (`measure_the_count_ceiling`, fonte de
/// 100 instâncias): no teto de 256 o cook custa **0,032 ms** para 25.600 instâncias, ou seja o
/// clamp é de FORMA (quantas dobras o padrão ainda lê), nunca de custo.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "segments",
    max: MAX_SEGMENTS as f32,
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "segments",
        label: "node.motion.kaleidoscope.param.segments",
        min: 1.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "reflect",
        label: "node.motion.kaleidoscope.param.reflect",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.kaleidoscope.param.reflect.0",
                "node.motion.kaleidoscope.param.reflect.1",
            ],
        },
    },
    ParamUiHint {
        param: ph2d_nodegraph::pivot::PARAM,
        label: "node.motion.kaleidoscope.param.pivot_mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: ph2d_nodegraph::pivot::LABELS,
        },
    },
    ParamUiHint {
        param: "pivot_x",
        label: "node.motion.kaleidoscope.param.pivot_x",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pivot_y",
        label: "node.motion.kaleidoscope.param.pivot_y",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: REINDEX,
        label: "node.motion.kaleidoscope.param.reindex",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

/// **As duas coordenadas só aparecem no modo que as LÊ** — a mesma família de gate que o
/// `motion.transform` já declara sobre os dele. ⚠️ Esconder não é apagar: o valor sobrevive à
/// troca de modo, e é por isso que o kernel tem de olhar o MODO e não «o ponto é diferente de
/// zero?» (o defeito que a W1a mediu a divergir `1,369` unidades de mundo entre a CPU e o
/// dispositivo).
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "pivot_x",
        when: ph2d_nodegraph::pivot::PARAM,
        values: &[1],
    },
    ph2d_node_registry::ParamGate {
        param: "pivot_y",
        when: ph2d_nodegraph::pivot::PARAM,
        values: &[1],
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
        param: "pivot_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "pivot_y",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    const O: [f32; 2] = [0.0, 0.0];

    /// `segments` multiplies the element count exactly.
    #[test]
    fn segments_multiply_the_count() {
        let p = vec![[1.0, 0.0], [1.5, 0.3], [0.8, -0.2]];
        assert_eq!(kaleidoscope(&p, 6, false, O, 0.0).len(), 18);
        assert_eq!(
            kaleidoscope(&p, 1, false, O, 0.0).len(),
            3,
            "1 slice = passthrough"
        );
    }

    /// Rotational symmetry (reflect off): a source point on +x is replicated at each
    /// `1/segments` turn. FALSIFIED if the copies landed anywhere but the ring.
    #[test]
    fn rotational_copies_sit_on_the_ring() {
        let p = vec![[2.0, 0.0]];
        let out = kaleidoscope(&p, 4, false, O, 0.0);
        // slice 0 → +x, 1 → +y, 2 → −x, 3 → −y (quarter turns).
        assert!(
            (out[0][0] - 2.0).abs() < 1e-3 && out[0][1].abs() < 1e-3,
            "0: {:?}",
            out[0]
        );
        assert!(
            out[1][0].abs() < 1e-2 && (out[1][1] - 2.0).abs() < 1e-2,
            "1: {:?}",
            out[1]
        );
        assert!(
            (out[2][0] + 2.0).abs() < 1e-2 && out[2][1].abs() < 1e-2,
            "2: {:?}",
            out[2]
        );
        assert!(
            out[3][0].abs() < 1e-2 && (out[3][1] + 2.0).abs() < 1e-2,
            "3: {:?}",
            out[3]
        );
    }

    /// Mirrored symmetry (reflect on): the odd slice is the source *mirrored* before it
    /// rotates, so it differs from the plain rotational copy. FALSIFIED if `reflect`
    /// were ignored (odd slice identical to the rotational one).
    #[test]
    fn reflect_mirrors_alternate_slices() {
        let p = vec![[1.0, 0.5]];
        let plain = kaleidoscope(&p, 4, false, O, 0.0);
        let mirrored = kaleidoscope(&p, 4, true, O, 0.0);
        assert_eq!(plain[0], mirrored[0], "even slice 0 identical");
        // Slice 1 rotated by 90°: plain (1,0.5)→(−0.5,1); mirrored reflects y first
        // (1,−0.5)→(0.5,1). The x-sign flips.
        assert!(
            (plain[1][0] + 0.5).abs() < 1e-2,
            "plain slice 1 x=−0.5: {:?}",
            plain[1]
        );
        assert!(
            (mirrored[1][0] - 0.5).abs() < 1e-2,
            "mirrored slice 1 x=+0.5: {:?}",
            mirrored[1]
        );
    }

    /// `spin` rotates the whole pattern: a quarter-turn spin sends a +x source to +y.
    #[test]
    fn spin_rotates_the_pattern() {
        let p = vec![[2.0, 0.0]];
        let out = kaleidoscope(&p, 3, false, O, 0.25); // +90°
        assert!(
            out[0][0].abs() < 1e-2 && (out[0][1] - 2.0).abs() < 1e-2,
            "spun to +y: {:?}",
            out[0]
        );
    }

    /// A pivot off the origin is the fixed point: a source *at* the pivot stays put
    /// under every slice.
    #[test]
    fn the_pivot_is_the_fixed_point() {
        let piv = [3.0, -1.0];
        let out = kaleidoscope(&[piv], 5, true, piv, 0.13);
        for q in &out {
            assert!(
                (q[0] - piv[0]).abs() < 1e-4 && (q[1] - piv[1]).abs() < 1e-4,
                "fixed: {q:?}"
            );
        }
    }

    /// Deterministic + cooks through the registry: `P` fans out to `segments · n` and
    /// every other column is duplicated to match.
    #[test]
    fn registers_and_folds_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.kaleidoscope.test.src"),
            name: "motion.kaleidoscope.test.src",
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
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[1.0, 0.0], [2.0, 0.5]]))
                        .with("size", Column::Vec2(vec![[0.3, 0.3], [0.3, 0.3]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionKaleidoscope),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.kaleidoscope.test.src");
        let k = g.add_node("motion.kaleidoscope");
        g.set_param(k, "segments", 6.0);
        g.connect(Edge {
            from: (src, 0),
            to: (k, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, k, 0.0).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 12, "2 elements × 6 slices");
        match s.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 12, "size duplicated per slice"),
            _ => panic!("size"),
        }
    }
}

#[cfg(test)]
mod hard_max_gates {
    use super::{MAX_SEGMENTS, PARAM_HARD_MAX};

    /// **O teto DIGITÁVEL não pode passar do que o KERNEL honra.**
    ///
    /// `eval` e a lei de contagem clampam `segments` em [`MAX_SEGMENTS`]. Uma caixa que
    /// aceitasse 1.000 mostraria 1.000 e o caleidoscópio continuaria com 256 dobras — um
    /// controle que **aceita e mente**. O irmão deste gate mora no `motion.lattice`.
    #[test]
    fn the_typed_ceiling_stops_where_the_kernel_clamps() {
        let limit = PARAM_HARD_MAX
            .iter()
            .find(|h| h.param == "segments")
            .expect("segments tem teto duro");
        assert_eq!(
            limit.max, MAX_SEGMENTS as f32,
            "o teto digitável tem de ser o clamp do kernel, nem mais nem menos"
        );
    }
}

mod kernel;
use kernel::GPU_KERNEL;

#[cfg(test)]
#[path = "pivot_tests.rs"]
mod pivot_tests;

#[cfg(test)]
#[path = "reindex_tests.rs"]
mod reindex_tests;
