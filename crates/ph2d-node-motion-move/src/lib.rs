#![forbid(unsafe_code)]
//! `motion.move` — a Motion **modifier**: adds a constant offset `(dx, dy)` to
//! the `P` (Vec2) attribute of its input stream, scaled per-instance by the
//! multiplicative `falloff` column (§1.2; absent → `1.0`, full effect). Every
//! other column passes through unchanged (count preserved). Pure.
//!
//! Params (read via `ctx.param`): `dx` (0), `dy` (0), [`SPACE`] (0 World).
//! `P'_i = P_i + (dx, dy) * falloff_i`.
//!
//! ## O ESPAÇO do deslocamento (doc 89 folha 05 — a varredura PRO da família TRANSFORM)
//!
//! `(dx, dy)` era sempre um vetor de MUNDO: mandar cada elemento *"um passo para
//! a frente do seu próprio nariz"* — o que uma sim orientada pede o tempo todo —
//! não era difícil, era **duas caixas de texto**. A cadeia medida na célula:
//! `motion.expression("a*cos(rot*0.0174533) − b*sin(rot*0.0174533)") →
//! motion.drive(X, Add)` **mais o gêmeo** para Y, ou seja o mesmo `(dx, dy)` vivo
//! em duas fórmulas de texto que têm de concordar — a falha de duas-portas, e
//! quatro nós.
//!
//! ⚠️ **`Local` roda o OFFSET, nunca o elemento.** O `rot` continua a ser lido e
//! nunca escrito: este nó move, quem gira é o `motion.rotate`. É o que separa
//! *"andar para a frente"* de *"virar-se"*, e é o que mantém os dois compostáveis
//! na ordem que o artista quiser.

mod trig;

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **O ESPAÇO em que `(dx, dy)` é lido** — `0` Mundo (o que sempre shipou), `1`
/// Local (o offset roda pelo `rot` de cada elemento).
///
/// ⚠️ **O `0` é LITERAL e não aritmético**, e a distinção é o que faz o default
/// não mover um bit: o caminho de Mundo não passa por rotação nenhuma. Rodar por
/// `0°` daria o mesmo número aqui (os quatro quartos de volta da senoide
/// parabólica são exatos, ver [`trig`]) — mas a igualdade seria uma propriedade
/// da aproximação, e uma propriedade pode mudar. A estrutura não.
///
/// ⚠️ **Uma lista SEM coluna `rot` em modo Local devolve `(dx, dy)`** — o mesmo
/// que Mundo. Não é um caso especial: é a identidade `rot = 0` a valer, na CPU
/// pelo braço literal e no device pelo `identity: 0.0` da binding de leitura. As
/// duas portas dão o mesmo número porque respondem a mesma pergunta.
const SPACE: &str = "space";

/// O modo Local do [`SPACE`] — o valor que o enum do painel autora.
const SPACE_LOCAL: f32 = 1.0;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.move"),
    name: "motion.move",
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
            name: "dx",
            default: 0.0,
        },
        ParamSpec {
            name: "dy",
            default: 0.0,
        },
        // Apendado (doc 89 folha 05). `0` = World, o nó que sempre shipou.
        ParamSpec {
            name: "space",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent column or short
/// column → `1.0`, i.e. full effect). Shared shape across all modifiers.
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// **O offset que este elemento leva**, antes do falloff. `rot = None` é o modo
/// Mundo **e também** o modo Local sobre uma lista sem orientação — o mesmo
/// braço literal para os dois, porque é a mesma resposta (ver [`SPACE`]).
///
/// A porta é ÚNICA de propósito: a `eval` e o gate de paridade perguntam a ela, e
/// não a duas expressões que teriam de concordar sobre o sinal do seno.
fn offset_for(dx: f32, dy: f32, rot: Option<f32>) -> (f32, f32) {
    match rot {
        Some(deg) => {
            let (c, s) = cos_sin_cycles(deg / 360.0);
            (dx * c - dy * s, dx * s + dy * c)
        }
        None => (dx, dy),
    }
}

/// GPU compute kernel (GPU/M5 Fase 1, ADR-0126): `P' = P + (dx, dy) · falloff`,
/// the exact per-element map of the CPU `eval` (same multiply/add order → same
/// float result up to GPU FMA contraction, covered by the ε parity gate).
/// `ReadWriteExisting` mirrors the CPU's pattern-match: a stream WITHOUT a `P`
/// column passes through untouched (the CPU only rewrites `P` when it exists),
/// so absence means the same thing on both paths.
const MOVE_WORLD: GpuKernel = GpuKernel {
    wgsl: "\
        let mv_f = read_falloff(i);\n\
        let mv_p = read_P(i);\n\
        write_P(i, vec2<f32>(mv_p.x + params.dx * mv_f, mv_p.y + params.dy * mv_f));\n",
    wgsl_lib: "",
    bindings: MOVE_WORLD_BINDINGS,
    params: &["dx", "dy"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// As bindings do modo Mundo — nomeadas para o dispatcher as reusar sem uma
/// segunda cópia que pudesse divergir.
const MOVE_WORLD_BINDINGS: &[ColumnBinding] = &[
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
];

/// **O modo LOCAL** (doc 89 folha 05): o offset roda pelo `rot` do elemento antes
/// de ser pesado pelo falloff — a transcrição exacta de [`offset_for`].
///
/// ⚠️ **Uma VARIANTE e não um `select` no corpo**, porque as bindings diferem: o
/// modo Mundo não lê `rot`, e declará-la sempre poria uma coluna a mais no
/// bind group de todo `motion.move` do grafo — o custo é do plano, não do laço.
/// ⚠️ E o `rot` é `Read` com `identity: 0`, que é a MESMA resposta que a CPU dá
/// quando a coluna falta (ver [`SPACE`]): as duas portas concordam por
/// construção, não por sorte.
const MOVE_LOCAL: GpuKernel = GpuKernel {
    wgsl: "\
        let mv_f = read_falloff(i);\n\
        let mv_b = mv_cos_sin(read_rot(i) / 360.0);\n\
        let mv_ox = params.dx * mv_b.x - params.dy * mv_b.y;\n\
        let mv_oy = params.dx * mv_b.y + params.dy * mv_b.x;\n\
        let mv_p = read_P(i);\n\
        write_P(i, vec2<f32>(mv_p.x + mv_ox * mv_f, mv_p.y + mv_oy * mv_f));\n",
    // A MESMA senoide parabólica corrigida do `trig.rs` da CPU, verbatim — o
    // `sin` do WGSL não tem garantia entre fabricantes e poria o device noutro
    // círculo (HR-5, o precedente do `motion.orbit`).
    wgsl_lib: "\
        fn mv_sin_cycles(phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn mv_cos_sin(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(mv_sin_cycles(phase + 0.25), mv_sin_cycles(phase));\n\
        }\n",
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
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["dx", "dy"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// O kernel registado: o dispatcher. A forma de topo **é** a de Mundo, para quem
/// nunca resolver ver um kernel real (o molde do `motion.drive`).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: MOVE_WORLD.wgsl,
    wgsl_lib: MOVE_WORLD.wgsl_lib,
    bindings: MOVE_WORLD_BINDINGS,
    params: MOVE_WORLD.params,
    count_law: None,
    variant_by_param: Some(|param| {
        if param(SPACE) >= 0.5 {
            &MOVE_LOCAL
        } else {
            &MOVE_WORLD
        }
    }),
    applicable: None,
};

struct MotionMove;

impl NodeOp for MotionMove {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (dx, dy) = (ctx.param("dx"), ctx.param("dy"));
        let local = ctx.param(SPACE) >= 0.5;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // ⚠️ A coluna só é procurada no modo que a lê: em Mundo o `rot` de um
            // stream orientado não pode nem chegar ao caminho quente.
            let rot: Option<&Vec<f32>> = match (local, input.get("rot")) {
                (true, Some(Column::Scalar(r))) => Some(r),
                _ => None,
            };
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                match (name.as_str(), col) {
                    ("P", Column::Vec2(v)) => {
                        // Pure per-instance map → parallel above the threshold
                        // (bit-identical, no reduction). GPU/M5 Fase 0.
                        let moved: Vec<[f32; 2]> = par_build(v.len(), |i| {
                            let p = v[i];
                            let f = falloff_at(input, i);
                            let (ox, oy) = offset_for(dx, dy, rot.and_then(|r| r.get(i).copied()));
                            [p[0] + ox * f, p[1] + oy * f]
                        });
                        out.set("P", Column::Vec2(moved));
                    }
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
    reg.register(Box::new(MotionMove))?;
    // M1.R1 — UI metadata (a spatial modifier → blue transform, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Move",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5 Fase 1 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamHardMin, ParamUiHint, ParamWidget};

/// **O teto DIGITÁVEL do deslocamento, MEDIDO** — bloco Z, doc 91.
///
/// ⚠️ **A cena `=15` (o caleidoscópio) autora `dx = 260` e o campo digitava até `10`** — vinte e
/// seis vezes o alcance do artista, no nó mais simples do catálogo. Sem entrada aqui o digitado
/// para no fim do ARRASTO (`ui.rs:206`). Acusação da sonda
/// `what_the_corpus_authors_and_no_one_can_type`.
///
/// **O recurso é a PRECISÃO** (`CLAUDE.md` §0.0): mover não satura — mais longe é mais longe —,
/// então o que acaba é o `f32`: acima daqui somar o `step` do slider (0,1) **não move o
/// número**. Derivado a cada corrida pelo gate
/// `every_precision_bound_param_types_to_the_measured_ceiling`.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "dx",
        max: 2_097_151.875,
    },
    ParamHardMax {
        param: "dy",
        max: 2_097_151.875,
    },
];

/// O piso, e ele é o SIMÉTRICO do teto porque um deslocamento tem sinal.
///
/// ⚠️ **As duas pontas ou nenhuma.** Um teto generoso com o piso de ontem faria o nó andar
/// duzentos metros para a direita e dez para a esquerda — e um gesto que só funciona para um
/// lado lê-se como bug do nó, não como faixa de slider.
static PARAM_HARD_MIN: &[ParamHardMin] = &[
    ParamHardMin {
        param: "dx",
        min: -2_097_151.875,
    },
    ParamHardMin {
        param: "dy",
        min: -2_097_151.875,
    },
];

/// Param UI hints (M1.P1): signed X/Y offsets in metres.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "dx",
        label: "Move X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "dy",
        label: "Move Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: SPACE,
        label: "Space",
        min: 0.0,
        max: SPACE_LOCAL,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["World", "Local"],
        },
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
        param: "dx",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "dy",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances at (0,0),(1,1) with a falloff column [1, 0.5].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.move.test.src"),
        name: "motion.move.test.src",
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
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionMove),
                _ => None,
            }
        }
    }

    #[test]
    fn offset_is_scaled_by_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("motion.move.test.src");
        let mv = g.add_node("motion.move");
        g.connect(Edge {
            from: (src, 0),
            to: (mv, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(mv, "dx", 10.0);
        g.set_param(mv, "dy", 4.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, mv, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // i0 f=1: (0,0)+(10,4)=(10,4) ; i1 f=0.5: (1,1)+(5,2)=(6,3)
            Column::Vec2(v) => assert_eq!(v, &vec![[10.0, 4.0], [6.0, 3.0]]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn missing_falloff_means_full_effect() {
        // A source without a falloff column → weight 1 everywhere.
        let s = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        assert_eq!(falloff_at(&s, 0), 1.0);
    }
}

#[cfg(test)]
#[path = "space_tests.rs"]
mod space_tests;
