#![forbid(unsafe_code)]
//! `motion.path` — **walk a shape the artist DREW** (Motion Nodes M3 / doc 65).
//!
//! Place `count` instances at even arc-length along a **vector path**, slide them with `offset`,
//! and (optionally) turn them to face the way the curve is going. It is the vector document's
//! `motion.distribute_curve`: same distribution, but the curve is a real drawn shape instead of
//! four control-point params.
//!
//! ## The reason this node was blocked for two journeys
//!
//! The plan said *"integra `vector.*`"* — and that node family was **RETIRED** (ADR-0108). The
//! geometry moved into `ph2d-vec-scene`, a document the graph has **no reach into**: a node is
//! handed its params, its inputs and the playhead, and nothing else. That is not an oversight, it
//! is the property that lets the cook memoize, scrub and replay bit-exactly.
//!
//! So the question was never *"how do I import a curve"*. It was **"how does anything the app owns
//! get into the graph at all"** — and the answer is the [external channel](ph2d_nodegraph::external)
//! (doc 65): the app publishes named values into the `Cook`, the node reads one by name.
//!
//! **The name is the artist's.** The shell publishes every vector shape under the name it carries in
//! the Hierarchy — so the gesture is: draw a curve, call it `Track`, and type `Track` into this
//! node. There is no id to copy, no picker to build, and no second place for the two to disagree
//! about which shape they mean. Rename the shape and the node follows it — because the name IS the
//! reference, not a lookup into one.
//!
//! HR-5: arc-length is `sqrt` (the leaf `ph2d-arc-length`, shared with `motion.spline_wrap`
//! — two nodes asking where arc fraction `s` is, one answer), the tangent's angle is the Rajan `atan2`
//! (`trig.rs`). `Effect::Pure` — the layout is a pure function of the curve, the params, and
//! nothing else; the curve's own revision rides in the cook's fingerprint, so editing the shape
//! re-cooks this node and *only* the nodes downstream of it.

use ph2d_node_registry::{
    NodeRegistry, ParamGate, ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
    RegistryError,
};

/// **O `spacing` é uma DISTÂNCIA, e é a única deste nó que é.**
///
/// A unidade diz *o que o número É*, nunca como ele se mostra (doc 88), e um espaçamento de arco
/// é medido nas mesmas unidades de mundo em que a curva foi desenhada — logo `Length`, exactamente
/// como o irmão `motion.spline_wrap` declara as coordenadas dos pontos de controle dele.
///
/// ⚠️ **E os outros três ficam SEM unidade, o que é a decisão e não a omissão:** `count` é uma
/// contagem que o `ParamWidget::Int` já apresenta, `align` é um interruptor, e o `offset` é uma
/// FRAÇÃO do percurso (`0..1`, com volta) — o `spline_wrap`, que lê a mesma curva desenhada, deixa
/// as frações dele (`from`/`to`/`offset`) igualmente sem unidade, e divergir aqui faria dois nós
/// da mesma família apresentarem a mesma grandeza de dois jeitos.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "spacing",
    unit: ParamUnit::Length,
}];

/// `mode`: quem responde **"quantos?"**.
const MODE_COUNT: f32 = 0.0;
const MODE_SPACING: f32 = 1.0;

/// O piso do espaçamento. Não é gosto: `count = comprimento / spacing`, então um
/// `spacing` que chega a zero pede uma contagem infinita — e o clamp que a apara
/// devolveria o teto em silêncio, com o slider a dizer outra coisa. O piso é o
/// menor passo que o `ParamUiHint` oferece, e abaixo dele a pergunta degenera.
const MIN_SPACING: f32 = 0.01;

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "count",
    max: 500.0,
}];
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The **text** param naming the shape (the channel doc 32 opened — a `ParamSpec` is f32-only, and
/// the manifest is frozen, so a string param lives in the `Graph` beside the manifest, not in it).
const PATH_PARAM: &str = "path";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.path"),
    name: "motion.path",
    inputs: &[
        // An OPTIONAL value input for the offset — so a `value.lfo` makes the set flow down the
        // curve. Same shape as `motion.distribute_curve`'s: the animation arrives through a wire,
        // which is why the node itself can stay `Pure`.
        PortSpec {
            name: "offset",
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
        ParamSpec {
            name: "count",
            default: 24.0,
        },
        // Slides the whole set along the arc, WRAPPING — a curve is a thing to walk around, not a
        // line to fall off the end of.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // `0` nada · `1` TANGENTE (a peça encara o sentido da curva) · `2` NORMAL
        // (ela encara para FORA da curva) — o `rot` sai em graus. Ver [`ALIGN_NORMAL`].
        ParamSpec {
            name: "align",
            default: 1.0,
        },
        // **Quem responde "quantos?"** — `0` a CONTAGEM, `1` o ESPAÇAMENTO.
        //
        // ⚠️ Um modo e não uma sentinela, e é a lei do `time_mode` do
        // `motion.oscillator`/`value.lfo`: `count` e `spacing` são **dois números
        // sobre a mesma grandeza**, e dois deles na tela é pior que um botão morto.
        // A sentinela (`spacing = 0` ⇒ usa `count`) foi considerada e **não cabe
        // aqui**: o `ParamGate` decide por um valor INTEIRO, então ela deixaria
        // `spacing ∈ (0, 0,5)` a pintar os dois controles com só um a mandar —
        // um knob que mente, que é o que esta casa recusa.
        ParamSpec {
            name: "mode",
            default: MODE_COUNT,
        },
        // A distância de ARCO entre vizinhos, em unidades de MUNDO.
        //
        // ⚠️ **Diverge do irmão no mesmo app, e a divergência é NOMEADA:** o
        // `pattern_along_path` do módulo Vector mede o espaçamento como FRAÇÃO da
        // largura do motivo (`1.0` encaixa borda-a-borda) porque lá o que se
        // repete é uma FORMA, que tem tamanho. Aqui o nó emite **instâncias**, e
        // ele não sabe o tamanho de nenhuma — o `size` é escrito depois, por
        // quem quiser. Uma fração de uma largura que este nó não conhece seria
        // um número sem referente; a distância de arco é o que ele pode honrar.
        ParamSpec {
            name: "spacing",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The published curve, as a polyline. The shell flattens the drawn shape **into world space** and
/// publishes its points as `P` — the graph never sees a Bézier, and does not need to.
fn polyline(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Qual das duas réguas responde *"quantos?"*.
///
/// O índice de um enum chega como `f32`, então a fronteira é o **ponto médio**
/// entre os dois — escrita a partir das duas constantes, e não como um `0.5`
/// solto que deixaria de ser o meio no dia em que um terceiro modo nascesse.
fn counts_by_spacing(mode: f32) -> bool {
    mode >= (MODE_COUNT + MODE_SPACING) * 0.5
}

/// Quantas cópias **CABEM** num arco de comprimento `length` a cada `spacing`.
///
/// ⚠️ **FLOOR, e a lei é a do irmão no mesmo app:** o `pattern_along_path` do
/// módulo Vector só coloca a cópia cuja FATIA cabe no que resta do arco
/// (`k_hi = floor(…)`), então o espaçamento entregue nunca é mais APERTADO que o
/// pedido. Com o enrolamento deste nó o vão de volta ao começo é o mesmo
/// `length / count`, então o conjunto sai uniforme e a garantia vale na volta
/// inteira.
///
/// ⚠️ **Um espaçamento maior que a curva devolve ZERO**, não uma cópia — é o
/// mesmo veredito do irmão (`if k_hi < k_lo { return Vec::new() }`), e é honesto:
/// nada cabe. O nó já trata contagem zero como *forma ausente* e emite vazio.
/// Repare que isso **cai do `floor`**, não de um caso especial.
///
/// ⚠️ **A primeira versão tinha um guard a mais (`n >= 1.0`) e a MUTAÇÃO o
/// derrubou:** trocá-lo por `n >= 0.0` não movia um número, porque `floor` de
/// qualquer coisa menor que 1 **já é 0**. Ele foi removido em vez de ganhar um
/// gate — *um guard que não consegue mudar resposta nenhuma é ruído, e um gate
/// escrito para o defender teria de mentir sobre o que prova*. O que sobra é o
/// `is_finite`, que é **defesa em camadas MEDIDA**: o `length` vem de geometria
/// real e o `spacing` já passou pelo piso, então nenhum chamador de hoje o
/// alcança — ele fica porque o param é dirigível por fio (doc 58) e custa nada.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn copies_that_fit(length: f32, spacing: f32) -> usize {
    let n = (length / spacing).floor();
    if n.is_finite() {
        // Um negativo satura em 0 no `as usize` (definido desde o Rust 1.45), e
        // um `length` negativo não existe: o `lut` é soma de comprimentos.
        (n as usize).min(RECOMMENDED_MAX_ELEMENTS)
    } else {
        0
    }
}

struct MotionPath;

impl NodeOp for MotionPath {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let align = ctx.param("align").round() as i32;
        // The offset is the param PLUS whatever a wire is putting on the `offset` input — one
        // number, so an LFO makes the set flow. (A stream with no value reads as 0.)
        let wired = match ctx.input(0).get(ph2d_nodegraph::attr::VALUE_COLUMN) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let offset = ctx.param("offset") + wired;

        // ⚠️ **O canal da GEOMETRIA, não o nome cru.** O nome cru carrega a
        // APARÊNCIA da forma (uma instância na origem, publicada pelo bake de
        // objetos DEPOIS das curvas), e ler dali dava um stream de um ponto — sem
        // arco, sem erro, sem aviso. Ver `external::curve_of`.
        let name = ctx.text_param(PATH_PARAM).unwrap_or_default().to_string();
        let pts = polyline(ctx.external(&ph2d_nodegraph::external::curve_of(&name)));
        let lut = ph2d_arc_length::lut(&pts);

        // **Quantos?** — o número que o artista digitou, ou o que o ESPAÇAMENTO
        // deriva do comprimento que o `lut` já tem na mão (`lut.last()` **É** o
        // total; o contrato está escrito na folha `ph2d-arc-length`). Nada de
        // canal novo, nada de segunda travessia: a wave inteira é uma divisão
        // sobre um número que este `eval` já calculou para amostrar.
        let count = if counts_by_spacing(ctx.param("mode")) {
            copies_that_fit(
                lut.last().copied().unwrap_or(0.0),
                ctx.param("spacing").max(MIN_SPACING),
            )
        } else {
            param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS)
        };

        // A shape that is not there (not drawn yet, renamed, deleted) is an EMPTY stream — the same
        // thing an unconnected input is. A node that cannot find its curve emits nothing; it does
        // not guess, and it does not fail.
        if lut.is_empty() || count == 0 {
            ctx.emit(Stream::new(0));
            return;
        }

        let mut pos = Vec::with_capacity(count);
        let mut rot = Vec::with_capacity(count);
        for i in 0..count {
            // Even ARC-LENGTH, not even parameter: sampling by parameter bunches the points on the
            // tight bends, and the eye reads that as "not even".
            // ⚠️ **O ENROLAMENTO é POLÍTICA DESTE NÓ, e agora está escrito aqui.**
            // Ele morava dentro do amostrador, e o `motion.spline_wrap` — o segundo
            // consumidor da mesma curva desenhada — precisa do OPOSTO (o elemento em
            // `u = 1` tem de pousar no FIM). Um amostrador não tem política de ponta;
            // um nó tem. Aqui o `offset` desliza o conjunto **e dá a volta**, que é o
            // gesto (uma marquise correndo por um caminho, não um clamp na ponta) —
            // e `s − floor(s)` é literalmente a linha que saiu do `arc::at`, então
            // isto é byte-idêntico ao que shipava.
            let s = i as f32 / count as f32 + offset;
            let (p, tangent) = ph2d_arc_length::at(&pts, &lut, s - s.floor());
            pos.push(p);
            if align != 0 {
                // ⚠️ A NORMAL é a tangente rodada um quarto de volta — `(-ty, tx)`, o
                // mesmo `un` que o irmão `motion.spline_wrap` já computa. Ela entra
                // pelos COMPONENTES e não somando 90° ao ângulo, para as duas rotas
                // atravessarem a MESMA aproximação de `atan2`: somar depois daria um
                // ângulo com dois erros diferentes conforme o modo.
                let (ax, ay) = if align == ALIGN_NORMAL {
                    (-tangent[1], tangent[0])
                } else {
                    (tangent[0], tangent[1])
                };
                rot.push(trig::deg(trig::atan2_approx(ay, ax)));
            }
        }

        let mut out = Stream::new(count).with("P", Column::Vec2(pos));
        if align != 0 {
            out = out.with("rot", Column::Scalar(rot));
        }
        ctx.emit(out);
    }
}

/// **O terceiro modo de `align`: encarar para FORA da curva** (doc 89, folha 06 ·
/// Blender GN *Curve to Points ▸ Normal*).
///
/// ⚠️ **A célula dizia «NÃO» e a razão escrita era *"a normal, que nada publica"* — e ela
/// já estava publicada:** o irmão `motion.spline_wrap` computa `un = [-ut.y, ut.x]` da
/// MESMA curva desenhada, e o doc deste nó já lhe chamava *"o segundo consumidor"*. A
/// nona célula desta folha a envelhecer sobre um facto que o código do lado já tinha.
///
/// ⚠️ **Apendado**: `0` e `1` continuam a ser *nada* e *tangente*, então o
/// `align >= 0.5` de ontem e este `!= 0` concordam em todo documento que existe.
pub const ALIGN_NORMAL: i32 = 2;

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionPath))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Path",
            category: ph2d_node_registry::NodeUiCategory::Distribute,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

/// **Só a régua escolhida aparece.** `count` e `spacing` respondem à MESMA
/// pergunta, e pintar os dois faria o artista perguntar qual deles manda — o
/// precedente exacto é o `time_mode` do `motion.oscillator` (Seconds × BPM).
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "count",
        when: "mode",
        values: &[0],
    },
    ParamGate {
        param: "spacing",
        when: "mode",
        values: &[1],
    },
];

/// Param UI hints (M1.P1). The **path name** is a `ParamWidget::Source` — a picker of the
/// shapes the app has published (doc 65), so the artist picks the shape they drew by NAME
/// instead of typing its exact internal name. It rides the same text channel underneath
/// (doc 32); the raw text field is still there as the escape (a forward reference is legal).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: PATH_PARAM,
        label: "Shape",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    ParamUiHint {
        param: "mode",
        label: "Count By",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Number", "Spacing"],
        },
    },
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 240.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    // A faixa é a que a MÃO percorre; o teto digitável é o `ParamHardMax`, e
    // acima dele nada quebra — só não cabe cópia nenhuma, que é a resposta certa.
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: MIN_SPACING,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -1.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Era um `Toggle`, e virou um seletor de TRÊS** — apendado: `0` e `1` guardam o
    // que sempre guardaram (nada · tangente), e um documento de ontem abre igual.
    ParamUiHint {
        param: "align",
        label: "Align To Path",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "Tangent", "Normal"],
        },
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
