#![forbid(unsafe_code)]
//! `fx.drop_shadow` — a **drop shadow** as a per-instance stream FX (Motion Nodes
//! M4, doc 01 §3): every element casts a coloured ghost of itself, offset along a
//! direction, drawn behind the whole layout.
//!
//! **The reference is the Photoshop / After Effects layer effect** — Angle,
//! Distance, Opacity, Colour (+ Size/Spread, see below) — and the defaults are
//! theirs: **35 % opacity**, black, thrown down-and-right.
//!
//! ```text
//!   out = [ every element's shadow ] ++ [ every element, verbatim ]
//!              (behind, in block)            (on top)
//! ```
//!
//! **Whole-layout, not per-element.** The shadows are one block, ahead of the
//! elements, so every shadow is behind every element — Photoshop's layer shadow.
//! Interleaving (shadow, element, shadow, element, …) would let one element's shadow
//! fall ON its neighbour, which reads as dirt, not depth.
//!
//! **`Direction` is the direction the shadow FALLS** (AE's name and its meaning).
//! Photoshop calls the same dial `Angle` and points it at the *light* instead, which
//! is the opposite vector; the label here says which one it is, so nothing has to be
//! remembered. Degrees — the app's one authored-angle unit — measured
//! counter-clockwise from `+x`, in the y-up world of the Motion canvas, so the
//! default **315°** throws the shadow down-and-right.
//!
//! **A shadow is a COLOUR, not a dark copy.** Its RGB comes from the swatch (black by
//! default); only its ALPHA is inherited — `swatch.a × element.a × falloff` — so a
//! half-transparent element casts a half-transparent shadow, and a `falloff` region
//! decides *which* elements cast at all. (A `tint`-darkened copy would carry the
//! element's hue and give a red ball a red shadow.)
//!
//! ## A MACIEZ, e o que a cerca contra ela dizia (doc 89, folha 11)
//!
//! Este cabeçalho dizia, palavra por palavra: *"What is deliberately NOT here:
//! blur … rather than a fake softness built from a stack of ghosts."* A cerca
//! tinha DUAS razões, e só uma delas continua de pé:
//!
//! - **O BORRÃO RASTER continua fora, e agora com o mecanismo escrito.** O passe do
//!   Motion compõe **aditivamente** (`One`/`One`, `motion_fx.rs`), e **um halo
//!   escuro não pode ser somado** — um borrão de verdade exigiria um passe ANTES do
//!   de sprites, que é decisão de renderer e não um param deste nó.
//! - **A «pilha de fantasmas» era sobre ENCADEAR o nó**, e a própria conferência
//!   mediu porque aquilo é ruim: um *smear* ao longo de UMA direção, sem alargamento
//!   perpendicular, com o alfa a compor multiplicativamente (`0,35² = 0,1225` na 2ª
//!   ordem). **Um disco de UM passe não tem nenhum dos três defeitos**, e é o que o
//!   `softness` faz — ver [`soft`], que traz os números e o que a aproximação custa.
//!
//! ⚠️ **A primeira-parte também discordava da cerca:** o nosso próprio
//! `ph2d-ecs::vec_filter_kinds` "Drop Shadow" tem `Radius` — o módulo Vector shipou
//! a sombra MACIA enquanto este nó a recusava.
//!
//! Em `softness = 0` o nó é a sombra **hard-edged** de sempre, ao bit: um tap, um
//! alfa, o caminho literal — o flat-design / long-shadow, honestamente o que é.
//!
//! Transcendental-free (HR-5): the direction goes through the parabolic `cos/sin`
//! leaf. `Effect::Pure`. Like every ghost FX it duplicates `id`s, so place it
//! downstream of anything that pairs state by id — conventionally before the Output.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod copies;
mod soft;
mod trig;
use copies::{falloff_at, positions, tile, tints};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// How many rows one element becomes com a sombra DURA: o fantasma + ele próprio.
///
/// ⚠️ **Já não é a contagem do nó** — com `softness > 0` são `soft::TAPS + 1`, e é
/// `cast` quem a calcula a partir dos taps que vai de facto emitir. Esta const fica
/// por ser o caso neutro que os gates do teto usam como referência — e por isso é
/// `cfg(test)`: um número que só o teste lê não pertence ao binário.
#[cfg(test)]
const COPIES: usize = 2;

/// A full turn, in the degrees the param stores — the `trig` leaf speaks **cycles**.
const DEGREES_PER_TURN: f32 = 360.0;

/// Hard ceiling on the emitted element count (`2 × count`, an untrusted upstream).
/// Over budget the FX turns itself off (the input, verbatim) rather than shadowing
/// half the layout.
///
/// ⚠️ **O RECURSO É TEMPO, e o número é MEDIDO** — o mesmo teto do `motion.trail`, pelo mesmo
/// motivo (linhas emitidas no caminho de CPU) e com a mesma tabela ao lado dele. Medido pela
/// porta do produto (`ph2d-node-registry-init/tests/measure_instance_ceiling.rs`): este nó
/// custa **~10–15 ns por linha emitida**, então o teto antigo de `65_536` valia ~0,7 ms — um
/// vigésimo de um quadro de 60 fps —, e este vale ~3 ms. ⚠️ **Nenhum dos três nós que
/// carregavam este literal trazia uma medição**, e a justificativa escrita era uma CONTAGEM
/// (*"131k quads"*), não um custo.
///
/// ⚠️ **Os três têm de andar juntos** — gate `the_three_instance_ceilings_agree` na
/// `ph2d-node-registry-init`, a única crate que vê os três. Drop-crates não podem depender umas
/// das outras (ADR-0075), então a const é copiada como o `falloff_at` das behaviours; o que a
/// mantém honesta é o gate.
pub const MAX_INSTANCES: usize = 262_144;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.drop_shadow"),
    name: "fx.drop_shadow",
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
        // Where the shadow FALLS (degrees, ccw from +x). 315° = down-and-right.
        ParamSpec {
            name: "direction",
            default: 315.0,
        },
        ParamSpec {
            name: "distance",
            default: 0.2,
        },
        // The shadow's colour. `a` IS the opacity (Photoshop's default: 35 % black).
        ParamSpec {
            name: "r",
            default: 0.0,
        },
        ParamSpec {
            name: "g",
            default: 0.0,
        },
        ParamSpec {
            name: "b",
            default: 0.0,
        },
        // Apendado (doc 89 folha 11). `0` = a sombra hard-edged de sempre, ao bit.
        ParamSpec {
            name: "softness",
            default: 0.0,
        },
        ParamSpec {
            name: "a",
            default: 0.35,
        },
        // Apendado (doc 89 folha 11). `0` = `Sink`, e é o mundo de antes deste param **ao
        // bit**: a coluna nem chega a ser escrita. Ver [`SHADOW_BLEND_LABELS`].
        ParamSpec {
            name: SHADOW_BLEND,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// O param que escolhe **com que modo a SOMBRA se mistura** (doc 89 folha 11).
///
/// ⚠️ **A célula precificava isto como *"uma coluna de convenção de stream"*, e a fundação já
/// tinha shipado por outra wave.** A coluna por-linha existe (`ph2d_eval_motion::lower`), o
/// `motion.trail` e o `motion.strobe` já a escrevem, e o que faltava aqui era **um param**.
/// *Um item cujo custo era a fundação muda de preço no dia em que a fundação chega.*
pub const SHADOW_BLEND: &str = "shadow_blend";

/// A coluna de convenção que o `ph2d-eval-motion` lê por LINHA — o mesmo nome do param do
/// sink, porque é a mesma grandeza.
pub const BLEND_COLUMN: &str = "blend";

/// Os modos que esta sombra oferece, na ordem das tags.
///
/// ⚠️ **Copiados, e não importados, porque um nó é uma FOLHA** — este crate não pode depender
/// do `motion.output` nem do `motion.trail`, e é a mesma razão por que aqueles também os
/// escrevem à mão. Quem impede as três listas de divergir é o gate
/// `the_row_operators_speak_the_sinks_vocabulary` (na shell, o único sítio que vê os quatro
/// lados) — e este nó entra nele como **uma linha**.
///
/// ⚠️ **O `Multiply` é o default do Photoshop para uma sombra**, e é por isso que ele é o
/// motivo desta célula existir: uma sombra que MULTIPLICA escurece o que está por baixo em vez
/// de a cobrir, que é o que uma sombra faz no mundo. O default aqui continua `Sink` porque o
/// default de um param apendado **reduz** ao mundo de antes; a escolha é do artista.
pub const SHADOW_BLEND_LABELS: [&str; 7] = [
    // ⚠️ O primeiro NÃO é um modo: é a ausência de escolha.
    "Sink",
    "Normal",
    "Add",
    "Subtract",
    "Multiply",
    "Screen",
    "Premultiplied",
];

/// O valor de coluna de um `shadow_blend` autorado — `None` quando ele é *o do sink*.
///
/// ⚠️ A coluna guarda o índice do dropdown tal e qual (o rótulo `Sink` é o `0`, e daí em
/// diante o índice JÁ é `modo + 1`): uma segunda aritmética entre o dropdown e a coluna seria
/// a segunda porta que diverge da primeira.
fn shadow_blend_tag(v: f32) -> Option<f32> {
    if !v.is_finite() || v < 0.5 {
        return None;
    }
    let top = (SHADOW_BLEND_LABELS.len() - 1) as f32;
    Some(v.round().clamp(1.0, top))
}

/// The world-space offset of every shadow: `distance` along `direction`.
fn offset(direction_deg: f32, distance: f32) -> [f32; 2] {
    let (cos, sin) = trig::cos_sin_cycles(direction_deg / DEGREES_PER_TURN);
    [cos * distance, sin * distance]
}

/// Onde cada elemento projecta os seus fantasmas.
///
/// Sem maciez é **UM** ponto, o offset — o caminho LITERAL, o nó que sempre shipou,
/// e um `softness` não-finito (documento carregado, edição por MCP) conta como
/// desligado em vez de envenenar as posições. Com maciez são [`soft::TAPS`] pontos
/// num disco em torno desse mesmo offset.
fn taps(off: [f32; 2], softness: f32) -> Vec<[f32; 2]> {
    if !(softness.is_finite() && softness > 0.0) {
        return vec![off];
    }
    soft::disc(softness)
        .iter()
        .map(|o| [off[0] + o[0], off[1] + o[1]])
        .collect()
}

/// One evaluation: the shadows (behind, in a block), then the elements verbatim.
fn cast(
    input: &Stream,
    direction_deg: f32,
    distance: f32,
    color: [f32; 4],
    softness: f32,
    shadow_blend: f32,
) -> Stream {
    let n = input.count();
    let off = offset(direction_deg, distance);
    let pts = taps(off, softness);
    // ⚠️ **O teto conta as linhas que este cook vai de facto emitir**, e é por isso
    // que a maciez o move: `TAPS + 1` por elemento em vez de `2`. O portão é o mesmo
    // que já existia; o que mudou foi ele passar a saber o número certo.
    let copies = pts.len() + 1;
    // Nothing to shadow, no budget, or a fully transparent shadow colour: forward the
    // input verbatim rather than paying for invisible quads. A junk alpha (NaN / ∞ — a
    // loaded document, an MCP edit) counts as "off": it would otherwise poison every
    // shadow's alpha.
    let dead = !color[3].is_finite() || color[3] <= 0.0;
    if n == 0 || n.saturating_mul(copies) > MAX_INSTANCES || dead {
        return input.clone();
    }
    let p = positions(input);
    let base = tints(input);
    let split = pts.len() > 1;

    let mut pos = Vec::with_capacity(n * copies);
    let mut tint = Vec::with_capacity(n * copies);
    // The shadows: the swatch's colour, carrying the element's own transparency.
    // ⚠️ Em BLOCOS por tap (todo o tap 0, depois todo o tap 1, …), a mesma razão
    // de `tile`: a ordem do stream é a ordem de desenho, e blocos mantêm cada
    // fantasma atrás de cada elemento.
    for q in &pts {
        for i in 0..n {
            pos.push([p[i][0] + q[0], p[i][1] + q[1]]);
            let a = color[3] * base[i][3] * falloff_at(input, i);
            let a = if split { soft::per_tap_alpha(a) } else { a };
            tint.push([color[0], color[1], color[2], a]);
        }
    }
    // The elements themselves, verbatim and LAST, so they paint over their shadows.
    for i in 0..n {
        pos.push(p[i]);
        tint.push(base[i]);
    }

    let mut out = Stream::new(n * copies);
    for (name, col) in input.columns() {
        if name != "P" && name != "tint" {
            out.set(name.clone(), tile(col, copies));
        }
    }
    out.set("P", Column::Vec2(pos));
    out.set("tint", Column::Vec4(tint));
    // **O MODO DA SOMBRA** (doc 89 folha 11) — só as linhas do FANTASMA o levam.
    //
    // ⚠️ **No `Sink` a coluna nem é tocada**, e é isso que faz o default ser byte-idêntico em
    // vez de meramente equivalente: o `tile` acima já copiou o `blend` que o artista tivesse
    // posto a montante, e escrever `0` por cima seria apagá-lo.
    //
    // ⚠️ **E os ELEMENTOS mantêm o que traziam.** Uma sombra que impusesse o próprio modo às
    // peças que a projectam seria o nó a decidir sobre linhas que não são dele — e o rastro a
    // montante já pode ter escolhido ali.
    if let Some(tag) = shadow_blend_tag(shadow_blend) {
        let own = input.get(BLEND_COLUMN);
        let mut blend = vec![tag; n * copies];
        for (i, slot) in blend.iter_mut().skip(n * pts.len()).enumerate() {
            *slot = match own {
                Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(0.0),
                _ => 0.0,
            };
        }
        out.set(BLEND_COLUMN, Column::Scalar(blend));
    }
    out
}

/// **A sombra que herda o modo do SINK** — o que este nó era antes do [`SHADOW_BLEND`].
///
/// ⚠️ **Nomeado em vez de herdado por um default**, do mesmo jeito que o `unlimited` do
/// `sim.step`: uma premissa herdada em silêncio **inverte de sentido** no dia em que o default
/// se move, e o gate continua verde a testar o oposto do que diz. Os gates que de facto medem
/// o modo chamam a [`cast`] direto.
#[cfg(test)]
fn sink_blend(
    input: &Stream,
    direction_deg: f32,
    distance: f32,
    color: [f32; 4],
    softness: f32,
) -> Stream {
    cast(input, direction_deg, distance, color, softness, 0.0)
}

struct FxDropShadow;

impl NodeOp for FxDropShadow {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (direction, distance) = (ctx.param("direction"), ctx.param("distance"));
        let color = [
            ctx.param("r"),
            ctx.param("g"),
            ctx.param("b"),
            ctx.param("a"),
        ];
        let out = cast(
            ctx.input(0),
            direction,
            distance,
            color,
            ctx.param("softness"),
            ctx.param(SHADOW_BLEND),
        );
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxDropShadow))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Drop Shadow",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    // CPU-only: this node reads `falloff` only at eval runtime (no GPU kernel), so the
    // diagnoser cannot derive the role from a `ColumnBinding` — declare it (ADR-0155).
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

/// **O que cada número É** (doc 88, Wave A · doc 89 folha 11) — nunca como ele é mostrado.
///
/// ⚠️ **A família `fx.*` tinha ZERO dos quatro canais de side-metadata** (`param_units` ·
/// `hard_max` · `hard_min` · `sections`), medido por grep nas três crates. Não era gap de
/// capacidade: era o número a viajar sem dizer o que é, e a fronteira de display já existia.
///
/// A `distance` é um comprimento de MUNDO — o painel resolve a face (`px` ou `m`) do
/// `ProjectSettings::display_unit`, e o store fica em metros. A `direction` é o `Angle` que o
/// widget já desenhava e que a UNIDADE não dizia: *o widget é como se mostra, a unidade é o
/// que o número É*, e um `Angle` que só existe no widget some assim que alguém ler o param
/// por outra porta.
///
/// ⚠️ **As quatro do swatch ficam de fora de propósito** — `r`/`g`/`b`/`a` são frações de cor,
/// e `Ratio` ali seria um rótulo que não ajuda ninguém a ler o número.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "distance",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "direction",
        unit: ParamUnit::Angle,
    },
    ParamUnitDecl {
        param: "softness",
        unit: ParamUnit::Length,
    },
];

/// O teto que a MÃO percorre fica no slider; o que a MÁQUINA aceita alcança-se por DIGITAÇÃO
/// (o soft/hard do Blender, doc 88 §11). ⚠️ O curso de antes é este número — nada ficou
/// inalcançável, só deixou de ser o que o dedo varre.
///
/// A `distance` parava em `2,0` mundo, que é uma sombra curta; o *long shadow* do design plano
/// — o look que este nó de propósito entrega hard-edged — pede uma dezena de unidades.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "distance",
        max: 40.0,
    },
    // ⚠️ **O recurso da maciez é a CONTAGEM DE TAPS, não o raio** — 16 taps num disco
    // grande deixam de se sobrepor e a penumbra vira bandas. O slider para onde a
    // aproximação ainda lê como penumbra; acima disso o artista está a pedir um
    // borrão, que é a rota de renderer que a cerca nomeia (ver o cabeçalho).
    ParamHardMax {
        param: "softness",
        max: 4.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ **PRIMEIRO na lista, e não no fim onde foi apendado ao manifesto**: o modo é a
    // pergunta que decide o que as outras significam (uma sombra que MULTIPLICA não se lê como
    // uma que cobre), e a ordem do painel é a das perguntas, não a da história do arquivo.
    ParamUiHint {
        param: SHADOW_BLEND,
        label: "Shadow Blend",
        min: 0.0,
        max: (SHADOW_BLEND_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &SHADOW_BLEND_LABELS,
        },
    },
    ParamUiHint {
        param: "direction",
        label: "Direction",
        min: 0.0,
        max: DEGREES_PER_TURN,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "distance",
        label: "Distance",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "softness",
        label: "Softness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "r",
        label: "Color",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["r", "g", "b", "a"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK_35: [f32; 4] = [0.0, 0.0, 0.0, 0.35];

    fn pair(tint: [f32; 4]) -> Stream {
        Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [2.0, 0.0]]))
            .with("tint", Column::Vec4(vec![tint, tint]))
            .with("size", Column::Vec2(vec![[0.5, 0.5], [0.5, 0.5]]))
    }

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }
    fn ts(s: &Stream) -> Vec<[f32; 4]> {
        match s.get("tint").unwrap() {
            Column::Vec4(v) => v.clone(),
            _ => panic!("tint"),
        }
    }

    /// The layout: shadows FIRST (they must draw behind), elements LAST and verbatim.
    /// At `direction = 0°` the shadow falls along `+x`. FALSIFIED by any order that
    /// puts the shadows on top, and by any implementation that moves the element.
    #[test]
    fn the_shadows_are_one_block_behind_the_untouched_elements() {
        let out = sink_blend(&pair([1.0, 1.0, 1.0, 1.0]), 0.0, 0.5, BLACK_35, 0.0);
        assert_eq!(out.count(), 4, "a shadow + an element, per element");

        let p = ps(&out);
        assert_eq!(p[0], [0.5, 0.0], "shadow of element 0, thrown +x");
        assert_eq!(
            p[1],
            [2.5, 0.0],
            "shadow of element 1 — still in the SHADOW block"
        );
        assert_eq!(p[2], [0.0, 0.0], "the elements, where they always were");
        assert_eq!(p[3], [2.0, 0.0]);

        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 4, "size rode along onto the shadows"),
            _ => panic!("size"),
        }
    }

    /// A shadow is the COLOUR you asked for, carrying only the element's alpha —
    /// never a darkened copy of it. FALSIFIED by tinting the copy (a red element
    /// would cast a red shadow) and by ignoring the element's transparency (a ghost
    /// element would cast a solid shadow).
    #[test]
    fn the_shadow_is_a_colour_and_inherits_only_the_transparency() {
        let out = sink_blend(&pair([1.0, 0.0, 0.0, 0.5]), 0.0, 0.5, BLACK_35, 0.0);
        let t = ts(&out);
        assert_eq!(
            t[0][0..3],
            [0.0, 0.0, 0.0],
            "a RED element casts a BLACK shadow"
        );
        // 0.35 (the swatch) × 0.5 (the element's own alpha).
        assert!((t[0][3] - 0.175).abs() < 1e-6, "alpha = {}", t[0][3]);
        assert_eq!(
            t[2],
            [1.0, 0.0, 0.0, 0.5],
            "the element keeps its own colour"
        );
    }

    /// `direction` is DEGREES in a y-up world, so the default 315° throws the shadow
    /// down-and-right (`+x`, `−y`). FALSIFIED by a radians/cycles mix-up (315 rad is
    /// somewhere else entirely) and by the sign flip that throws it up-and-left.
    #[test]
    fn the_default_direction_throws_the_shadow_down_and_right() {
        let off = offset(315.0, 1.0);
        // cos 315° = +1/√2, sin 315° = −1/√2 (the parabolic leaf is ~0.1 % off).
        let diag = std::f32::consts::FRAC_1_SQRT_2;
        assert!((off[0] - diag).abs() < 0.01, "x = {}", off[0]);
        assert!((off[1] + diag).abs() < 0.01, "y = {}", off[1]);
        // A full turn is the same direction (the leaf wraps).
        let wrapped = offset(315.0 + DEGREES_PER_TURN, 1.0);
        assert!((wrapped[0] - off[0]).abs() < 1e-5);
    }

    /// `falloff` decides WHICH elements cast — the shadow fades, the element does not.
    #[test]
    fn falloff_picks_the_casters() {
        let src = pair([1.0, 1.0, 1.0, 1.0]).with("falloff", Column::Scalar(vec![0.0, 1.0]));
        let t = ts(&sink_blend(&src, 0.0, 0.5, BLACK_35, 0.0));
        assert_eq!(t[0][3], 0.0, "element 0 casts nothing");
        assert_eq!(t[1][3], 0.35, "element 1 casts at full opacity");
        assert_eq!(t[2], [1.0; 4], "the non-caster is itself untouched");
    }

    /// The effect turns ITSELF off rather than shadowing half the layout: a fully
    /// transparent swatch, an empty stream, or an over-budget one forwards the input.
    #[test]
    fn a_transparent_swatch_or_an_over_budget_stream_forwards_the_input() {
        let src = pair([1.0, 1.0, 1.0, 1.0]);
        let off = sink_blend(&src, 0.0, 0.5, [0.0, 0.0, 0.0, 0.0], 0.0);
        assert_eq!(off.count(), 2);
        assert_eq!(ps(&off), ps(&src), "verbatim");

        let huge = Stream::new(MAX_INSTANCES); // 2 × over the ceiling
        assert_eq!(
            sink_blend(&huge, 0.0, 0.5, BLACK_35, 0.0).count(),
            MAX_INSTANCES
        );
        assert_eq!(
            sink_blend(&Stream::new(0), 0.0, 0.5, BLACK_35, 0.0).count(),
            0
        );
    }
}

#[cfg(test)]
#[path = "softness_tests.rs"]
mod softness_tests;

/// O modo da sombra — assunto próprio, arquivo próprio (o corte do irmão acima).
#[cfg(test)]
#[path = "blend_tests.rs"]
mod blend_tests;
