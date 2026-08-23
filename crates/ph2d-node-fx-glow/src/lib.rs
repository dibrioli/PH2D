#![forbid(unsafe_code)]
//! `fx.glow` — the Motion module's HDR bloom, authored as a **node** (doc 67).
//!
//! ## Why a node, and not a document field or a panel
//!
//! The glow is a *pass* effect — it runs on the whole rendered Motion image, not
//! on one instance stream. The plan called for the document to "declare
//! `layer_fx`", and the graph **is** the document: a node IS a declaration in it.
//! Making the glow a node buys everything for free and keeps the module honest to
//! its own idiom:
//!
//! - **Authoring:** the existing params panel edits it the moment it is selected —
//!   no new gotcha-prone UI (a doc-level panel section is where this codebase
//!   bleeds time: the 1-px click drift, painted-≠-populated, a slider no test
//!   clicks). Its four knobs come with [`ParamUiHint`]s so the panel renders the
//!   right sliders and ranges.
//! - **Persistence / undo / driven params:** all of it comes from the graph
//!   infrastructure it already lives in. A `set_param` is a normal undo step; the
//!   textual format already serialises it.
//!
//! ## It configures the pass — it does not scope it
//!
//! `fx.glow` is a **passthrough** (`out == in`, `Effect::Pure`): dropping it in a
//! chain changes nothing about the stream, and leaving it unwired is fine too. The
//! shell finds it with [`from_graph`] and reads its params to drive
//! `ph2d_render::MotionFx` — the glow always applies to the whole Motion image
//! regardless of where the node sits or what (if anything) feeds it. Placement is
//! for readability; it never limits the effect. Zero glow nodes → the pass never
//! runs and the frame is byte-identical (the neutral point).
//!
//! Transcendental-free (HR-5): the node itself does no math — it forwards its
//! input and carries four numbers the render pass reads.

use ph2d_color::parse_gradient;
use ph2d_node_registry::{
    NodeRegistry, NodeSilhouette, NodeUiCategory, NodeUiManifest, ParamHardMax, ParamUiHint,
    ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::collections::BTreeMap;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Canonical node type — the shell matches on this when scanning the graph.
pub const TYPE_NAME: &str = "fx.glow";

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.glow"),
    name: "fx.glow",
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
        // Brightness above which a pixel glows (premult max(r,g,b)). 1.0 = only
        // genuinely HDR (emissive) pixels — an LDR scene is left untouched.
        ParamSpec {
            name: "threshold",
            default: 1.0,
        },
        ParamSpec {
            name: "knee",
            default: 0.6,
        },
        ParamSpec {
            name: "intensity",
            default: 0.8,
        },
        ParamSpec {
            name: "radius",
            default: 1.0,
        },
        // Glow colour (doc 67, 2nd pass). `saturation` pulls the halo toward
        // grey (0 = a white bloom regardless of source colour); `tint_*`
        // multiplies it (default white = the source's own colour). What Unity /
        // Unreal / After Effects all expose on a bloom.
        ParamSpec {
            name: "saturation",
            default: 1.0,
        },
        ParamSpec {
            name: "tint_r",
            default: 1.0,
        },
        ParamSpec {
            name: "tint_g",
            default: 1.0,
        },
        ParamSpec {
            name: "tint_b",
            default: 1.0,
        },
        ParamSpec {
            name: "tint_a",
            default: 1.0,
        },
        // Apendados (doc 89 folha 11). Os três em neutro = o halo de sempre.
        ParamSpec {
            name: "stretch",
            default: 1.0,
        },
        ParamSpec {
            name: "angle",
            default: 0.0,
        },
        ParamSpec {
            name: "clamp",
            default: 0.0,
        },
        // Apendados (doc 89 folha 11). Os dois em `0` = o passe de sempre, ao bit.
        ParamSpec {
            name: OPERATION,
            default: 0.0,
        },
        ParamSpec {
            name: SOURCE,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **A OPERAÇÃO do halo** — o *Glow Operation* do AE (doc 89 folha 11).
///
/// ⚠️ **A célula ofereceu TRÊS modos e só um passou pela navalha do §0.** O `Multiply` do AE
/// escurece, e um passe aditivo que escurecesse **quebraria o z**: o halo compõe-se sobre a cena
/// já desenhada, sem profundidade, então subtrair luz ali pintaria por cima do que estivesse à
/// frente. O `Screen` não tem esse problema — `a + b − ab` é monótono e **nunca escurece** —, e
/// é por isso que ele entra e o outro não. *Uma célula que pede três modos pode ser uma que pede
/// um; o que decide é o mecanismo, não a contagem.*
///
/// ⚠️ **Ele mora no BLEND STATE do pipeline, não no shader**, e é essa a razão de a célula dizer
/// *"nenhum nó o alcança"*: `Screen` é exactamente `src·(1−dst) + dst·1`, ou seja um par de
/// factores que a máquina de mistura já sabe fazer. Fazê-lo no shader exigiria LER o alvo, que
/// um passe de fullscreen não pode.
pub const OPERATION: &str = "operation";

/// Os modos que este halo oferece, na ordem das tags.
///
/// ⚠️ **A lista TEM de ter o mesmo tamanho que o array de pipelines do renderer** — um nó é uma
/// folha e não alcança o `ph2d-render`, então quem liga as duas pontas é um gate na shell
/// (`the_glow_operations_are_the_pipelines_the_renderer_built`). Sem ele, um modo a mais aqui
/// seria escolhível no dropdown e silenciosamente rebaixado para `Add`.
pub const OPERATION_LABELS: [&str; 2] = ["Add", "Screen"];

/// **DE QUE O BRIGHT-PASS SE ALIMENTA** — o *Glow Based On* do AE (doc 89 folha 11).
///
/// `Luminance` (o de sempre) lê `max(r, g, b)` do pixel premultiplicado; `Alpha` lê o alfa, e a
/// diferença é o que a referência oferece: **uma silhueta escura passa a brilhar**. Com luma,
/// uma peça preta opaca não tem nada acima do limiar e nunca acende; com alfa, ela acende pela
/// COBERTURA, que é o que se quer quando o halo é uma aura e não uma emissão.
pub const SOURCE: &str = "source";

/// As fontes do bright-pass, na ordem das tags.
pub const SOURCE_LABELS: [&str; 2] = ["Luminance", "Alpha"];

/// **A RAMPA DO HALO** — a chave do param de TEXTO em que ela viaja (doc 32/85).
///
/// ⚠️ **Nós temos a peça que a referência não tem, e é isso que torna esta célula barata.** O AE
/// oferece *Glow Colors A & B* — DUAS cores e um *Color Looping* —, e o Unreal cinco tintas por
/// tamanho de bloom. Aqui já existe o editor de gradiente ([`ParamWidget::Gradient`], doc 85)
/// com paradas arrastáveis e selector OKLCH por parada: a rampa inteira é **um param de texto**
/// e o painel desenha-a sem uma linha de UI nova.
///
/// **A rampa é indexada pela LUMINÂNCIA do halo**, que é o que a referência faz: o valor do
/// brilho escolhe a cor. Sem texto autorado o passe usa o `tint` constante de sempre, **ao
/// bit** — a LUT nem chega a ser assada.
pub const RAMP_KEY: &str = "ramp";

/// Quantos texels a LUT do halo carrega — **MEDIDO**, não escolhido.
///
/// A rampa é assada aqui, na CPU, em amostras **uniformes**: a biblioteca de cor fica com a
/// semântica (interpolação, espaço, caminho do matiz) e o shader faz apenas um `mix` entre
/// vizinhas. Uma grelha uniforme também dispensa a busca — o índice é `t · (n−1)`.
///
/// ⚠️ **O recurso tem nome: o PASSO DO ECRÃ.** A saída do halo é tonemapeada para 8 bits, então
/// um erro de reconstrução abaixo de `1/255` não tem como aparecer. A sonda
/// `measure_ramp_lut_resolution` varre os quatro presets da casa com
/// 2 049 pontos por rampa (o erro de uma reconstrução linear é máximo **no meio** de um
/// intervalo, então amostrar nos nós daria zero em toda parte) e imprime a tabela; o gate irmão
/// exige as **duas** metades — que esta contagem seja invisível **e** que METADE dela seja
/// visível, senão estaríamos a pagar memória de uniforme por nada.
pub const HALO_LUT_TEXELS: usize = 512;

/// The bloom settings the shell reads to drive the Motion glow pass. Mirrors
/// `ph2d_render::BloomParams` (this crate has no render dep — the shell converts
/// at the boundary).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Glow {
    pub threshold: f32,
    pub knee: f32,
    pub intensity: f32,
    pub radius: f32,
    /// `0` = a white bloom, `1` = the source's own colour.
    pub saturation: f32,
    /// Multiplies the (desaturated) glow — default white is a no-op.
    pub tint: [f32; 4],
    /// **A ANAMORFOSE** — a razão entre o alcance ao longo de [`Self::angle`] e o
    /// perpendicular. `1` é o halo redondo de sempre; `>1` é o *streak* de cinema.
    pub stretch: f32,
    /// A direção do *streak*, em graus. Inerte em `stretch = 1` — e o `ParamGate`
    /// esconde-a ali, porque *um controle que não faz nada não é pintado*.
    pub angle: f32,
    /// **O TETO do bright-pass** — `0` desliga. O antídoto dos *fireflies*.
    pub clamp: f32,
    /// **A OPERAÇÃO do halo** — `0` = `Add` (o de sempre), `1` = `Screen`. Ver [`OPERATION`].
    pub operation: f32,
    /// **De que o bright-pass se alimenta** — `0` = `Luminance` (o de sempre), `1` = `Alpha`.
    /// Ver [`SOURCE`].
    pub source: f32,
}

/// **A LUT DO HALO, ASSADA** — `None` quando não há rampa autorada (e aí o passe usa o `tint`
/// constante de sempre, **ao bit**).
///
/// ⚠️ **A semântica fica com a biblioteca de cor**: `eval` honra as CINCO interpolações e os
/// TRÊS espaços que o editor oferece. Reimplementá-los em WGSL seria a segunda porta que
/// diverge da primeira, e divergiria exactamente nas rampas em HSV — as que o painel oferece por
/// botão. O shader recebe uma tabela e interpola entre texels vizinhos; mais nada.
///
/// ⚠️ **Um texto que não parse conta como SEM RAMPA**, e não como uma rampa vazia: um documento
/// de uma versão futura, ou uma edição por MCP, não pode espalhar `NaN` por seis níveis de mip.
#[must_use]
pub fn bake_halo_lut(graph: &Graph) -> Option<Vec<[f32; 4]>> {
    let node = graph.nodes().iter().find(|n| n.type_name == TYPE_NAME)?.id;
    let ramp = graph
        .node_text_param_overrides(node)
        .and_then(|m| m.get(RAMP_KEY))
        .and_then(|s| parse_gradient(s))?;
    #[expect(clippy::cast_precision_loss, reason = "HALO_LUT_TEXELS <= 4096")]
    let at = |k: usize| ramp.eval(k as f32 / (HALO_LUT_TEXELS - 1) as f32);
    Some((0..HALO_LUT_TEXELS).map(at).collect())
}

/// The manifest default for a param name (the single source of a knob's neutral
/// value — the reader never hard-codes a second copy).
fn default_of(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.default)
        .unwrap_or(0.0)
}

/// A param's effective value: the graph override if the artist set it, else the
/// manifest default.
fn read(overrides: Option<&BTreeMap<String, f32>>, name: &str) -> f32 {
    overrides
        .and_then(|m| m.get(name))
        .copied()
        .unwrap_or_else(|| default_of(name))
}

/// Read the glow settings from the graph: the **first** `fx.glow` node's
/// (override-or-default) params. `None` when there is no glow node — the pass
/// does not run and the frame is byte-identical to no FX.
pub fn from_graph(graph: &Graph) -> Option<Glow> {
    let node = graph.nodes().iter().find(|n| n.type_name == TYPE_NAME)?.id;
    let ov = graph.node_param_overrides(node);

    Some(Glow {
        threshold: read(ov, "threshold"),
        knee: read(ov, "knee"),
        intensity: read(ov, "intensity"),
        radius: read(ov, "radius"),
        saturation: read(ov, "saturation"),
        tint: [
            read(ov, "tint_r"),
            read(ov, "tint_g"),
            read(ov, "tint_b"),
            read(ov, "tint_a"),
        ],
        stretch: read(ov, "stretch"),
        angle: read(ov, "angle"),
        clamp: read(ov, "clamp"),
        operation: read(ov, OPERATION),
        source: read(ov, SOURCE),
    })
}

struct FxGlow;

impl NodeOp for FxGlow {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Passthrough: the node carries settings, it does not transform the
        // stream. Forwarding the input verbatim keeps `out == in` byte-identical,
        // so wiring it into a chain is always safe and never changes the cook.
        let out = ctx.input(0).clone();
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxGlow))?;
    reg.register_ui(
        MANIFEST.id,
        NodeUiManifest {
            display_name: "Glow",
            category: NodeUiCategory::Fx,
            silhouette: NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ **Os dois modos PRIMEIRO** (doc 89 folha 11): eles decidem o que os números abaixo
    // significam — um limiar sobre luma e um sobre alfa são perguntas diferentes —, e a ordem do
    // painel é a das perguntas.
    ParamUiHint {
        param: OPERATION,
        label: "Operation",
        min: 0.0,
        max: (OPERATION_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &OPERATION_LABELS,
        },
    },
    // **A RAMPA**, logo depois dos modos: ela substitui o `tint`, então tem de aparecer antes
    // dele — senão o artista arrasta a cor constante e não entende porque nada muda.
    //
    // ⚠️ **Um param de TEXTO precisa de hint para EXISTIR na UI.** Sem esta linha o editor de
    // gradiente nunca é pintado, e a rampa seria um controle que o kernel lê e que gesto nenhum
    // alcança — exactamente o defeito que o doc 90 curou dezanove vezes. `min/max/step` são
    // inertes num widget de gradiente.
    ParamUiHint {
        param: RAMP_KEY,
        label: "Halo Ramp",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Gradient,
    },
    ParamUiHint {
        param: SOURCE,
        label: "Glow Based On",
        min: 0.0,
        max: (SOURCE_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &SOURCE_LABELS,
        },
    },
    ParamUiHint {
        param: "threshold",
        label: "Threshold",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "knee",
        label: "Soft Knee",
        min: 0.0,
        max: 2.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "intensity",
        label: "Intensity",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.25,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "saturation",
        label: "Saturation",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // A real OKLCH swatch (the panel wires the picker for a Color widget); the 4
    // channels are the tint params. White is the neutral no-op.
    ParamUiHint {
        param: "tint_r",
        label: "Tint",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["tint_r", "tint_g", "tint_b", "tint_a"],
        },
    },
    ParamUiHint {
        param: "stretch",
        label: "Anamorphic",
        min: 0.2,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Streak Angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "clamp",
        label: "Clamp",
        min: 0.0,
        max: 16.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// **A direção do *streak* não existe num halo redondo** — um círculo rodado é o
/// mesmo círculo, e um controle que não faz nada não é pintado. O `1` é o único
/// valor de `stretch` que esconde o ângulo, e é o default.
///
/// ⚠️ **Um `ParamGate` compara INTEIROS** (`values: &[i32]`), então ele esconde o
/// ângulo exactamente em `stretch = 1` — que é onde a anamorfose é a identidade.
/// Entre `1` e `2` o gate deixa o ângulo visível, e tem de deixar: ali ele morde.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[ph2d_node_registry::ParamGate {
    param: "angle",
    when: "stretch",
    values: &[1],
}];

/// **O que cada número É** (doc 88, Wave A · doc 89 folha 11) — o terceiro membro da
/// família a declarar a unidade, e o que fecha a lacuna que a folha mediu.
///
/// ⚠️ **O `radius` do glow NÃO é `Length`, e é a distinção que a lei do doc 88 pede.**
/// Ele multiplica um raio de tenda em **UV** (`BASE_FILTER_RADIUS = 0,006` da
/// `ph2d-render`), não uma distância de mundo — mostrá-lo em pixels ou metros seria
/// ensinar ao artista uma coisa falsa. É um `Ratio`, e é isso que ele é.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "angle",
        unit: ParamUnit::Angle,
    },
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "stretch",
        unit: ParamUnit::Ratio,
    },
];

/// O curso da MÃO fica no slider; o da MÁQUINA alcança-se por digitação (doc 88 §11).
///
/// ⚠️ **O teto do `clamp` é o do FORMATO e está MEDIDO**: o RT do glow é
/// `Rgba16Float`, cujo maior finito é `65 504` — acima disso o valor vira `inf` e
/// envenena a soma de toda a cadeia de mips. Um clamp maior que esse número não
/// pode morder nada, então ele é o teto honesto. A `intensity` parava em `4`, que é
/// pouco para um flare autoral.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "clamp",
        max: 65_504.0,
    },
    ParamHardMax {
        param: "intensity",
        max: 64.0,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// A source that emits a fixed two-instance stream — the "before" the glow
    /// passthrough must reproduce exactly.
    struct Source;
    const SOURCE: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.source"),
        name: "test.source",
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
    fn fixture() -> Stream {
        Stream::new(2)
            .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
            .with(
                "tint",
                Column::Vec4(vec![[6.0, 4.0, 2.0, 1.0], [1.0, 1.0, 1.0, 1.0]]),
            )
    }
    impl NodeOp for Source {
        fn manifest(&self) -> &'static NodeManifest {
            &SOURCE
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(fixture());
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == MANIFEST.id {
                Some(&FxGlow as &dyn NodeOp)
            } else if ty == SOURCE.id {
                Some(&Source as &dyn NodeOp)
            } else {
                None
            }
        }
    }

    /// **The invariant that lets `fx.glow` live anywhere: it is byte-identical in
    /// the cook.** `source → fx.glow` produces exactly what `source` alone does —
    /// so dropping the node in a chain never changes the stream, and the glow it
    /// configures is a pure addition on the render side.
    #[test]
    fn glow_is_a_byte_identical_passthrough() {
        let mut g = Graph::new();
        let src = g.add_node("test.source");
        let glow = g.add_node(TYPE_NAME);
        g.connect(Edge {
            from: (src, 0),
            to: (glow, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let through = cook.cook(&g, &Ops, glow, 0.0).unwrap()[0]
            .as_stream()
            .clone();
        assert_eq!(
            through.get("P"),
            fixture().get("P"),
            "the glow node must not move a point"
        );
        assert_eq!(
            through.get("tint"),
            fixture().get("tint"),
            "the glow node must not touch a colour (the HDR tint survives verbatim)"
        );
    }

    #[test]
    fn no_glow_node_reads_none() {
        // The neutral point: a graph without the node never runs the pass.
        let g = Graph::new();
        assert_eq!(from_graph(&g), None);
    }

    #[test]
    fn reads_defaults_then_overrides() {
        let mut g = Graph::new();
        let n = g.add_node(TYPE_NAME);
        // Untouched → manifest defaults (white, full-saturation glow).
        assert_eq!(
            from_graph(&g),
            Some(Glow {
                threshold: 1.0,
                knee: 0.6,
                intensity: 0.8,
                radius: 1.0,
                saturation: 1.0,
                tint: [1.0, 1.0, 1.0, 1.0],
                // Os três da folha 11, todos no neutro: halo redondo, sem teto.
                stretch: 1.0,
                angle: 0.0,
                clamp: 0.0,
                // …e os três da wave dos modos + rampa, idem: `ramp_len = 0` é *sem rampa*,
                // e o passe usa o `tint` constante de sempre.
                operation: 0.0,
                source: 0.0,
            })
        );
        // A dragged slider overrides just that knob; the rest stay at default.
        g.set_param(n, "intensity", 2.5);
        g.set_param(n, "threshold", 1.5);
        g.set_param(n, "saturation", 0.0);
        g.set_param(n, "tint_r", 0.5);
        let glow = from_graph(&g).unwrap();
        assert_eq!(glow.intensity, 2.5);
        assert_eq!(glow.threshold, 1.5);
        assert_eq!(glow.saturation, 0.0, "a white bloom");
        assert_eq!(
            glow.tint,
            [0.5, 1.0, 1.0, 1.0],
            "just the red channel moved"
        );
        assert_eq!(glow.radius, 1.0, "untouched knob keeps its default");
    }

    /// **TODO PARAM DO MANIFESTO CHEGA AO `Glow`** — e o oráculo é DERIVADO.
    ///
    /// ⚠️ Este gate dizia `params.len() == 9`, e a folha 11 partiu-o ao apendar três
    /// knobs sobre código correcto. Uma CONTAGEM escrita à mão não afirma nada sobre
    /// o nó: ela não sabe se o param novo é lido, e envelhece na primeira adição. O
    /// que a célula queria dizer é *"nenhum knob nasce mudo"*, e isso mede-se
    /// mexendo em cada um e exigindo que a struct MUDE.
    ///
    /// ⚠️ **Sem controle positivo isto passaria por vácuo** se `MANIFEST.params`
    /// ficasse vazio, então o gate também exige que a varredura tenha achado gente.
    #[test]
    fn every_manifest_param_reaches_the_glow_struct() {
        let mut checked = 0usize;
        for spec in MANIFEST.params {
            let mut g = Graph::new();
            let n = g.add_node(TYPE_NAME);
            let before = from_graph(&g).expect("o nó existe");
            // Um valor que difere do default seja ele qual for.
            g.set_param(n, spec.name, spec.default + 1.25);
            let after = from_graph(&g).expect("o nó existe");
            assert_ne!(
                before, after,
                "`{}` está no manifesto e o leitor não o vê — um knob mudo",
                spec.name
            );
            checked += 1;
        }
        assert!(checked >= 9, "controle: a varredura achou {checked} params");
        assert_eq!(default_of("knee"), 0.6);
        assert_eq!(default_of("stretch"), 1.0, "o halo redondo é o neutro");
    }
}

/// A RAMPA do halo — assunto próprio, arquivo próprio.
#[cfg(test)]
#[path = "ramp_tests.rs"]
mod ramp_tests;
