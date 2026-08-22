#![forbid(unsafe_code)]
//! `motion.mixer` — **blend several instance streams element-wise**: the Houdini
//! "Attribute Interpolate" / "Sequence Blend" (Motion Nodes M1, streams — doc 01 §1.7 /
//! doc 30). Where `motion.combine` stacks streams end to end, this fuses them per
//! element: element `i` of the output is the average / sum / lerp of element `i` across
//! the inputs. **Blend** two layouts and a `value.lfo` morphs one into the other (a grid
//! into a ring); **Avg** blends up to four at once.
//!
//! **Algorithm — element-wise reduction over the common columns.** The count is the
//! **minimum** across the contributing inputs (the extra tail of a longer input is
//! dropped — the Sequence-Blend convention). Every column present in **all** contributing
//! inputs is reduced: **Avg** = mean, **Add** = sum (both over all non-empty inputs);
//! **Blend** = `lerp(in0, in1, blend)` with the `blend` value input (unconnected → 0.5).
//! Transcendental-free (HR-5): component arithmetic. `Effect::Pure`.
//!
//! ⚠️ **The `blend` is a FIELD, one weight per element** (doc 12's broadcast rule,
//! the same one `motion.drive` and `motion.morph` read): absent → the midpoint,
//! length-1 HELD across the stream, length-N per-element. It used to be
//! `v.first()` — a length-N field handed **element zero's number to everybody**, so
//! the one thing a per-element blend could express was an accident, and every
//! reference disagreed (Blender's *Mix* `Factor` is a field; C4D gives each field
//! layer its own Mask; our own `motion.morph` was already per-element).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `blend` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Mix modes (the `mode` param). Avg is [`MODE_AVG`] — o braço default da redução, e
/// nomeado desde a wave dos pesos porque o `ParamGate` deles precisa do número.
const MODE_ADD: i64 = 1;
/// Blend mode: `lerp(in0, in1, blend)`.
const MODE_BLEND: i64 = 2;
/// What an UNCONNECTED `blend` input means: the midpoint, which is the number
/// the node has always used and the one an artist reads off the word "Blend".
const DEFAULT_BLEND: f32 = 0.5;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mixer"),
    name: "motion.mixer",
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
        // Blend weight for the Blend mode (animatable). Optional: unconnected → 0.5.
        PortSpec {
            name: "blend",
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
        // 0 Avg · 1 Add · 2 Blend (in0→in1 by the blend input).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // ⚠️ **Apendado**: de qual entrada vem a GEOMETRIA (folha 08 linha 44).
        // `0` = **Mixed**, o nó que sempre shipou. Ver [`GEOMETRY_COLUMNS`].
        ParamSpec {
            name: "geom_from",
            default: 0.0,
        },
        // **O PESO DE CADA ENTRADA** — ver [`WEIGHTS`]. Apendados, todos `1` ⇒ literal.
        ParamSpec {
            name: "weight_0",
            default: 1.0,
        },
        ParamSpec {
            name: "weight_1",
            default: 1.0,
        },
        ParamSpec {
            name: "weight_2",
            default: 1.0,
        },
        ParamSpec {
            name: "weight_3",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **O PESO POR ENTRADA** (doc 89 folha 08 — MiniCavalry `mixer`: `wa/wb/wc/wd`; C4D dá a
/// cada camada de field a sua Strength).
///
/// A célula media o custo do que já existia: encadear `Blend` aos pares reproduz qualquer
/// combinação convexa, mas **os pesos COMPÕEM** — o peso de `c` é `w2` e o de `a` é
/// `(1−w1)(1−w2)`, então o artista que quer 0,2 / 0,3 / 0,5 tem de resolver um sistema, e
/// paga um nó por entrada.
///
/// ⚠️ **Um por PORTA, não um por contribuinte.** Este nó descarta as entradas vazias antes de
/// reduzir, então a 3ª entrada LIGADA pode ser a porta `in3`; se o peso seguisse a posição na
/// lista de contribuintes, desligar um fio **remexeria os pesos dos outros três**. O peso
/// viaja com a porta desde o `snapshot`.
///
/// ⚠️ **`Avg` normaliza, `Add` não** — e é isso que mantém os dois modos a serem o que já
/// eram: a média ponderada é `Σ wᵢ·cᵢ / Σ wᵢ` (com todos a `1`, `Σ w` é a contagem, ao bit) e
/// a soma ponderada é `Σ wᵢ·cᵢ`. Normalizar a soma seria transformá-la numa média.
///
/// ⚠️ **`Σ w = 0` não é uma divisão por zero: é a resposta ZERO.** Com todo peso a zero a
/// média ponderada é indefinida em matemática, e um `0/0` daria `NaN` — uma cena que
/// desaparece sem explicação. O que se emite é o numerador (que é zero), que se lê como
/// *"você desligou todas as entradas"* — visível e explicável.
///
/// ⚠️ **Gateados fora do `Blend`**, onde o peso já tem dono: ali a resposta a *"quanto de cada
/// um?"* é o campo `blend`, por elemento. Pintar quatro sliders ao lado dele seriam duas
/// portas para a mesma pergunta, e a segunda ganharia em silêncio.
const WEIGHTS: [&str; 4] = ["weight_0", "weight_1", "weight_2", "weight_3"];

/// Avg is mode `0` — nomeado porque o [`ParamGate`] dos pesos precisa do número.
const MODE_AVG: i64 = 0;

/// **OS QUATRO MODOS DE DOBRA apendados** (folha 08 linha 45) — `in0 op in1 op …`,
/// elemento a elemento e componente a componente, sobre TODA coluna comum.
///
/// ⚠️ **São quatro e não os oito do `field.combine`, e a diferença é MEDIDA, não
/// preguiça.** O irmão opera só na coluna `falloff`, que é `[0,1]` por contrato — e
/// dois dos oito modos dele (`Screen` e `Overlay`) são **álgebra sobre «quão longe
/// do cheio» um número está**: `1 − (1−a)(1−b)`. Este nó mistura `P`, `size`, `vel`
/// e `tint` na mesma passagem, e um `Screen` sobre uma coordenada de mundo computa
/// sem significar nada — ele leria a posição `x = 3` como *"300% do cheio"*. Os
/// quatro que ficam são **livres de unidade**: valem o mesmo num metro, num pixel e
/// numa fração. ⚠️ E o nono da lista do irmão — `Normal` — já existe aqui com outro
/// nome: é o [`MODE_BLEND`].
const MODE_SUB: i64 = 3;
const MODE_MUL: i64 = 4;
const MODE_MIN: i64 = 5;
const MODE_MAX: i64 = 6;

/// A dobra componente-a-componente de duas colunas do MESMO variant.
///
/// ⚠️ **O peso NÃO entra aqui, e isso é o desenho.** `Avg` e `Add` são reduções
/// LINEARES, e um peso é exactamente o que uma redução linear sabe absorver; um
/// `Min` ponderado não quer dizer nada (o mínimo de quê — do valor, ou do valor
/// vezes o peso, que é outra grandeza?). Os pesos ficam `ParamGate`d nos dois modos
/// que os leem, que é onde já estavam.
fn fold_col(a: &Column, b: &Column, mode: i64) -> Column {
    let f = |x: f32, y: f32| match mode {
        MODE_SUB => x - y,
        MODE_MUL => x * y,
        MODE_MIN => x.min(y),
        _ => x.max(y),
    };
    macro_rules! z {
        ($va:expr, $vb:expr, $w:literal) => {{
            $va.iter()
                .zip($vb.iter())
                .map(|(x, y)| {
                    let mut r = *x;
                    for c in 0..$w {
                        r[c] = f(x[c], y[c]);
                    }
                    r
                })
                .collect()
        }};
    }
    match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => {
            Column::Scalar(x.iter().zip(y).map(|(a, b)| f(*a, *b)).collect())
        }
        (Column::Vec2(x), Column::Vec2(y)) => Column::Vec2(z!(x, y, 2)),
        (Column::Vec3(x), Column::Vec3(y)) => Column::Vec3(z!(x, y, 3)),
        (Column::Vec4(x), Column::Vec4(y)) => Column::Vec4(z!(x, y, 4)),
        _ => a.clone(),
    }
}

/// **AS COLUNAS QUE UMA MÉDIA NÃO SABE MISTURAR.**
///
/// `P` é a posição — mediá-la é o que este nó existe para fazer, e continua a ser o
/// default. As outras duas são **IDENTIDADES**, não quantidades: `geometry_id` e
/// `texture_id` são a convenção `0 = nenhuma, m+1 = a m-ésima`, e a média de `1` e
/// `3` é `2` — **uma terceira forma, que nenhuma das duas entradas tinha**. Não é um
/// erro que dê erro: é uma peça a desenhar a arte errada.
///
/// ⚠️ **O default continua a MISTURAR, e isso é deliberado:** mudar a lei de todo
/// documento já autorado por causa de um caso que só aparece quando duas lanes
/// carregam formas DIFERENTES seria pagar com o mundo inteiro por um canto dele. O
/// que este knob dá é a cura — *escolha a lane* — e é ela que a folha 08 pedia.
const GEOMETRY_COLUMNS: [&str; 3] = ["P", "geometry_id", "texture_id"];

/// A cloned snapshot of one input.
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

/// A column truncated to the first `n` rows.
fn trunc(c: &Column, n: usize) -> Column {
    match c {
        Column::Scalar(v) => Column::Scalar(v[..n].to_vec()),
        Column::Vec2(v) => Column::Vec2(v[..n].to_vec()),
        Column::Vec3(v) => Column::Vec3(v[..n].to_vec()),
        Column::Vec4(v) => Column::Vec4(v[..n].to_vec()),
    }
}

/// Component-wise `a + b·k` (same variant, same length).
fn add_scaled(a: &Column, b: &Column, k: f32) -> Column {
    macro_rules! z {
        ($va:expr, $vb:expr, $w:literal) => {{
            $va.iter()
                .zip($vb.iter())
                .map(|(x, y)| {
                    let mut r = *x;
                    for c in 0..$w {
                        r[c] += y[c] * k;
                    }
                    r
                })
                .collect()
        }};
    }
    match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => {
            Column::Scalar(x.iter().zip(y).map(|(a, b)| a + b * k).collect())
        }
        (Column::Vec2(x), Column::Vec2(y)) => Column::Vec2(z!(x, y, 2)),
        (Column::Vec3(x), Column::Vec3(y)) => Column::Vec3(z!(x, y, 3)),
        (Column::Vec4(x), Column::Vec4(y)) => Column::Vec4(z!(x, y, 4)),
        _ => a.clone(),
    }
}

/// Component-wise scale.
fn scale(c: &Column, k: f32) -> Column {
    macro_rules! s {
        ($v:expr, $w:literal) => {{
            $v.iter()
                .map(|x| {
                    let mut r = *x;
                    for c in 0..$w {
                        r[c] *= k;
                    }
                    r
                })
                .collect()
        }};
    }
    match c {
        Column::Scalar(v) => Column::Scalar(v.iter().map(|x| x * k).collect()),
        Column::Vec2(v) => Column::Vec2(s!(v, 2)),
        Column::Vec3(v) => Column::Vec3(s!(v, 3)),
        Column::Vec4(v) => Column::Vec4(s!(v, 4)),
    }
}

/// The `blend` for element `i` — **the one broadcast rule** (doc 12), the same one
/// `motion.drive` and `motion.morph` already read: **unconnected (empty) → the
/// midpoint**, length-1 is HELD across every instance, length-N is per-element.
///
/// ⚠️ This is the P0 of doc 89 folha 08, and what it replaced was `v.first()` — a
/// length-N field handed **element zero's number to the whole stream**, so the
/// only thing a per-element blend could express was an accident. Blender's *Mix*
/// makes `Factor` a field (the diamond socket), C4D gives every field layer its
/// own Mask, and **our own `motion.morph` was already per-element** — the mixer
/// was the one place in the family where the answer collapsed to one scalar.
///
/// ⚠️ **Not clamped, deliberately.** `motion.morph` clamps to `[0, 1]` and this
/// does not, and the two are right for different reasons: morph interpolates a
/// SHAPE toward another and promises `1` is `b`, while a mixer lerp past `1` is an
/// overshoot **that has a picture** — a layout thrown past the target one, which is
/// a thing an artist asks for. Clamping here would be a silent behaviour change on
/// top of the fix, so the range stays exactly what it was.
fn blend_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => DEFAULT_BLEND,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(DEFAULT_BLEND),
    }
}

/// `a·(1−t) + b·t` per lane, with `t` read per element.
///
/// ⚠️ The two-term form is not stylistic: at `t = 1` the first term is `a·0.0`,
/// which IEEE-754 makes exactly zero for any finite `a`, and the second is `b·1.0`
/// — so `blend = 1` is `in1` **to the bit**, which is what the node's own doc
/// promises. `a + (b − a)·t` lands *near* `b` and is not the same number.
fn lerp_col(a: &Column, b: &Column, blend: &[f32], n: usize) -> Column {
    macro_rules! z {
        ($va:expr, $vb:expr, $w:literal, $ctor:path) => {{
            $ctor(
                (0..n)
                    .map(|i| {
                        let t = blend_at(blend, i);
                        let (x, y) = ($va[i], $vb[i]);
                        let mut r = x;
                        for c in 0..$w {
                            r[c] = x[c] * (1.0 - t) + y[c] * t;
                        }
                        r
                    })
                    .collect(),
            )
        }};
    }
    match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => Column::Scalar(
            (0..n)
                .map(|i| {
                    let t = blend_at(blend, i);
                    x[i] * (1.0 - t) + y[i] * t
                })
                .collect(),
        ),
        (Column::Vec2(x), Column::Vec2(y)) => z!(x, y, 2, Column::Vec2),
        (Column::Vec3(x), Column::Vec3(y)) => z!(x, y, 3, Column::Vec3),
        (Column::Vec4(x), Column::Vec4(y)) => z!(x, y, 4, Column::Vec4),
        // Variants disagree: summing a Vec2 into a Vec4 means nothing, so the
        // first input wins — the same arm `add_scaled` already takes.
        _ => a.clone(),
    }
}

/// Column names present in every contributing snapshot, in the first input's order.
fn common_columns(snaps: &[&Snap]) -> Vec<String> {
    let Some(first) = snaps.first() else {
        return Vec::new();
    };
    first
        .cols
        .iter()
        .map(|(n, _)| n.clone())
        .filter(|n| snaps.iter().all(|s| s.column(n).is_some()))
        .collect()
}

/// Reduce the contributing inputs into one stream. `blend` is only used in Blend mode;
/// `weights` is aligned to `contributing` and carries **each snapshot's own port weight**
/// ([`WEIGHTS`]).
fn mix(
    mode: i64,
    contributing: &[&Snap],
    blend: &[f32],
    weights: &[f32],
    geom_from: Option<&Snap>,
) -> Stream {
    if contributing.is_empty() {
        return Stream::new(0);
    }
    let count = contributing.iter().map(|s| s.count).min().unwrap_or(0);
    let mut out = Stream::new(count);
    if count == 0 {
        return out;
    }
    let total_w: f32 = weights.iter().sum();
    for name in common_columns(contributing) {
        let cols: Vec<Column> = contributing
            .iter()
            .map(|s| trunc(s.column(&name).unwrap(), count))
            .collect();
        // A soma PONDERADA, que os dois modos de redução partilham. ⚠️ Com todos os pesos a
        // `1` ela é a soma de antes **ao bit**: `x·1.0` é `x` em IEEE-754 para todo `x`
        // finito, e o primeiro termo passa pelo mesmo `scale` que os outros.
        let weighted_sum = || {
            cols.iter()
                .zip(weights)
                .skip(1)
                .fold(scale(&cols[0], weights[0]), |acc, (c, w)| {
                    add_scaled(&acc, c, *w)
                })
        };
        // A GEOMETRIA vem de UMA lane quando o artista a escolhe — ver
        // [`GEOMETRY_COLUMNS`]. Ela sai antes da mistura, não depois: misturar e
        // depois deitar fora daria o mesmo número e o dobro do trabalho.
        if let Some(src) = geom_from
            && GEOMETRY_COLUMNS.contains(&name.as_str())
            && let Some(c) = src.column(&name)
        {
            out.set(name, trunc(c, count));
            continue;
        }
        let mixed = match mode {
            MODE_BLEND if cols.len() >= 2 => lerp_col(&cols[0], &cols[1], blend, count),
            MODE_ADD => weighted_sum(),
            // As quatro dobras apendadas. Com UMA entrada elas são a identidade —
            // `fold` sobre uma lista de um devolve o elemento —, que é a resposta
            // certa e a mesma que os outros modos dão.
            MODE_SUB | MODE_MUL | MODE_MIN | MODE_MAX => cols
                .iter()
                .skip(1)
                .fold(cols[0].clone(), |acc, c| fold_col(&acc, c, mode)),
            _ => {
                // Avg (and Blend with a single input): the WEIGHTED mean over the inputs.
                // ⚠️ `Σ w = 0` emite o numerador (zero) em vez de `0/0 = NaN` — ver
                // [`WEIGHTS`].
                let sum = weighted_sum();
                if total_w == 0.0 {
                    sum
                } else {
                    scale(&sum, 1.0 / total_w)
                }
            }
        };
        out.set(name, mixed);
    }
    out
}

struct MotionMixer;

impl NodeOp for MotionMixer {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i64;
        // ⚠️ The WHOLE column, not `v.first()`: the field is a per-element answer
        // and reading one row of it was the P0 of doc 89 folha 08.
        let blend: Vec<f32> = match ctx.input(4).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        // ⚠️ O peso é lido junto com a porta e viaja com ela pelo filtro — ver [`WEIGHTS`]:
        // uma entrada vazia sai da lista, e um peso indexado pela POSIÇÃO na lista passaria
        // a valer para outra porta.
        let ws: Vec<f32> = WEIGHTS.iter().map(|w| ctx.param(w)).collect();
        // Snapshot the four stream inputs, one at a time.
        // ⚠️ O índice da PORTA viaja com o snapshot, e não só o peso: o `geom_from`
        // nomeia a porta que o artista vê (A..D), e uma porta vazia sai da lista —
        // um índice contado na LISTA apontaria para outra porta.
        let snaps: Vec<(usize, Snap, f32)> = (0..4usize)
            .map(|k| (k, snapshot(ctx.input(k)), ws[k]))
            .filter(|(_, s, _)| s.count > 0)
            .collect();
        // Blend uses only the first two inputs; Avg/Add use all non-empty.
        let taken = if mode == MODE_BLEND { 2 } else { snaps.len() };
        let contributing: Vec<&Snap> = snaps.iter().take(taken).map(|(_, s, _)| s).collect();
        let weights: Vec<f32> = snaps.iter().take(taken).map(|(_, _, w)| *w).collect();
        // `0` = Mixed. Uma porta escolhida que esteja VAZIA cai de volta na mistura
        // — a alternativa seria a cena desaparecer por causa de um fio que faltava.
        let want = ctx.param("geom_from").round() as i64;
        let geom_from = (want >= 1)
            .then(|| {
                snaps
                    .iter()
                    .take(taken)
                    .find(|(port, _, _)| *port + 1 == want as usize)
                    .map(|(_, s, _)| s)
            })
            .flatten();
        ctx.emit(mix(mode, &contributing, &blend, &weights, geom_from));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMixer))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Mixer",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// ⚠️ **A faixa é `0..2` e o `1` fica no MEIO** — o literal é o centro do curso, e a metade de
/// cima é o que separa uma média ponderada de uma média: dar peso `2` a uma entrada é dizer
/// *"esta conta o dobro"*, e sem isso o knob só saberia apagar.
macro_rules! weight_hint {
    ($p:expr, $l:expr) => {
        ParamUiHint {
            param: $p,
            label: $l,
            min: 0.0,
            max: 2.0,
            step: 0.01,
            widget: ParamWidget::Slider,
        }
    };
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 6.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // ⚠️ Os quatro últimos são APENDADOS (folha 08 linha 45): `0..2` ficam
            // onde estavam, então todo documento já autorado lê o mesmo modo. Ver
            // [`fold_col`] para porque são QUATRO e não os oito do `field.combine`.
            labels: &["Avg", "Add", "Blend", "Subtract", "Multiply", "Min", "Max"],
        },
    },
    // De qual entrada vem a GEOMETRIA. `Mixed` é o nó de sempre.
    ParamUiHint {
        param: "geom_from",
        label: "Geometry From",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Mixed", "In 0", "In 1", "In 2", "In 3"],
        },
    },
    weight_hint!(WEIGHTS[0], "Weight 0"),
    weight_hint!(WEIGHTS[1], "Weight 1"),
    weight_hint!(WEIGHTS[2], "Weight 2"),
    weight_hint!(WEIGHTS[3], "Weight 3"),
];

/// Os pesos só aparecem onde são lidos — ver [`WEIGHTS`]: no `Blend` quem responde
/// *"quanto de cada um?"* é o campo `blend`, e um segundo controle para a mesma pergunta
/// ganharia em silêncio.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: WEIGHTS[0],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
    ParamGate {
        param: WEIGHTS[1],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
    ParamGate {
        param: WEIGHTS[2],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
    ParamGate {
        param: WEIGHTS[3],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "blend_field_tests.rs"]
mod blend_field_tests;

#[cfg(test)]
#[path = "weights_tests.rs"]
mod weights_tests;
