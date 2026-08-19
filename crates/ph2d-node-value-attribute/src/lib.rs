#![forbid(unsafe_code)]
//! **`value.attribute`** — read a named COLUMN of the stream as a value field (Motion Nodes,
//! doc 50). Blender's *Named Attribute* node; Houdini's `@age`, `@id`, `@speed`.
//!
//! ## The glue that was missing
//!
//! The stream carries a dozen per-element columns — `age`, `life`, `id`, `size`, `inv_mass`,
//! whatever a node wrote — and until now **nothing could read one back out**. `value.lfo` mints a
//! global, `value.instance_field` mints a field from *identity* (index / ramp / hash), and both
//! stop there: a number an element already CARRIES was unreachable to the value graph.
//!
//! So "colour the sparks by how old they are" — the most ordinary sentence in motion graphics —
//! had no path through this library at all, however many nodes it had.
//!
//! One node fixes it, and it fixes it for every column at once: age, life, speed, mass, id,
//! anything anyone adds later. That is why it is a *named* attribute and not an enum of the
//! columns we happen to have today.
//!
//! ## The name is a TEXT param
//!
//! `NodeManifest.params` is `f32`-only and **frozen** (§6), so the column's name rides the
//! graph's text channel (`Graph::set_text_param` / `EvalCtx::text_param`) — the canonical pattern
//! for a non-`f32` param, established by `motion.expression` (doc 32). The panel renders it as a
//! text field. No contract was bumped to get a string into a node.
//!
//! ## A missing column is `0`, not a crash
//!
//! Reading an attribute nothing wrote yields zeros — the value field is still length-N, so
//! everything downstream keeps its shape. A node that errored would take the whole graph down
//! because an artist typed `ag` instead of `age`, and a node that emitted an EMPTY field would
//! silently broadcast a global zero into a per-element slot, which is worse: it looks like it
//! worked.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, ReadChannel, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{GpuKernel, StreamOp};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The text param carrying the column's name (read via `EvalCtx::text_param`).
pub const ATTR_KEY: &str = "attr";

/// 0 = the scalar column itself · 1 = the LENGTH of a Vec2 column (so `vel` reads as *speed*,
/// which is what an artist asking for "speed" means).
pub const MODE_LENGTH: i32 = 1;

/// `MODE_COMPONENT_BASE + k` = **lane `k`** of a vector column — X·Y·Z·W, and for a colour
/// column R·G·B·A.
///
/// ## Why one rung and not a case per dimension
///
/// *"Give me lane k"* has ONE answer whatever the column's width is, so the ladder does not grow
/// an arm for `Vec2`, another for `Vec3` and another for `Vec4`. That matters beyond tidiness:
/// the GPU projection replays this ladder rung for rung (`stream_op.rs`), and every arm here is
/// an arm that has to be mirrored there without drifting.
///
/// ## What this unblocks, and why it was the gap
///
/// The value domain could already READ any column by name and could only ever get a scalar or a
/// magnitude back. The COMPONENTS of a vector — a lane of `vel`, of `P`, of `tint` — were
/// unreachable, and this rung reaches them.
///
/// ⚠️ **Esta doc afirmava que a lane destravava *"turn to face where you're going"* (doc 89
/// §10.0, a linha que CINCO famílias citam), e a medição REFUTOU:** uma lane devolve `vel.x` e
/// `vel.y` como dois escalares SOLTOS, e nada no domínio de valor os junta num ângulo —
/// `value.math` faz `Add·Subtract·Multiply·Divide·Min·Max`, `value.unary` faz
/// `Abs·Negate·Sign·Floor·Fract·Square·Sqrt·Reciprocal`, e o parser de `motion.expression` tem
/// `sin`/`cos` e **não tem `atan2`** (com `ph2d-expr` FROZEN por ADR-0039). Alcançar as
/// componentes não é alcançar a DIREÇÃO; quem a alcança é [`MODE_ANGLE`, o irmão de
/// `MODE_LENGTH`](MODE_ANGLE). *Uma afirmação de ter fechado um vão é a forma mais cara de nota
/// errada: ela faz a varredura seguinte pular a linha.*
///
/// ⚠️ A lane the column does not have (`Z` of a `Vec2`) is the ladder's ORDINARY miss: zeros at
/// full length, exactly like a mistyped name. The module's fence stands — this rung adds a
/// question the node can answer, it does not change what happens when it cannot.
pub const MODE_COMPONENT_BASE: i32 = 2;

/// A **DIREÇÃO** de uma coluna `Vec2`, em **GRAUS** — o irmão exacto do [`MODE_LENGTH`]: se
/// `Speed` responde *quão rápido*, este responde *para onde*, e as duas metades de um vetor
/// ficam ambas alcançáveis pelo domínio de valor.
///
/// ## O que ele fecha
///
/// `value.attribute(Direction de vel) → motion.drive(Rotation)` **é** o *align to velocity* de
/// todo sistema de partículas — a linha da [doc 89 §10.0] que cinco famílias (1·4·5·6·15)
/// citaram como inexprimível. Medido antes de construir: **nenhum** nó lê `vel` e escreve `rot`,
/// o `motion.look_at` mira em Point·Object·Cursor (nunca em *"para onde este elemento vai"*), e
/// nada no domínio de valor computa uma tangente inversa. É **composição de dois nós**, não um nó
/// novo — o mesmo desenho que fez de `value.noise(World) → motion.drive(Falloff)` o campo de
/// ruído.
///
/// ## Por que o degrau é NEGATIVO
///
/// A escada tem duas metades: **reduções** (o vetor inteiro vira um escalar) nos degraus baixos e
/// **lanes** em `MODE_COMPONENT_BASE + k`, aberta por construção. O espaço positivo já está
/// falado, e o `mode` é um **param que o grafo GUARDA** — renumerar `MODE_COMPONENT_BASE` para
/// abrir espaço re-apontaria em silêncio todo documento salvo (um `Opacity` gravado como `5`
/// passaria a significar outra lane). Então reduções crescem para BAIXO e lanes para cima, e as
/// duas nunca colidem.
///
/// ## Por que GRAUS, e não radianos
///
/// A coluna `rot` — a que o `motion.drive(Rotation)` escreve — **é em graus**; o lowering só
/// cruza para radianos na borda do render (`ph2d-eval-motion::lower`, *"the app's authored-angle
/// unit"*). Um canal que respondesse radianos erraria por **57×** exactamente na costura que ele
/// existe para servir, e nada na tela diria porquê
/// ([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]).
///
/// ⚠️ **`atan2` vem do `libm`, não do `std`** — o do `std` chama a libm da PLATAFORMA, e este
/// número atravessa o cozido; a física recusou o mesmo `atan2` pelo mesmo motivo
/// (`zone_force_world_at`). E o **vetor ZERO responde `0`**: quem não se move não tem direção, e
/// não pode girar.
pub const MODE_ANGLE: i32 = -1;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.attribute"),
    name: "value.attribute",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Scalar (read a scalar column) · 1 Length (read a Vec2 column's magnitude).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Lane `k` of a column, at full length — the one rung that serves every width.
///
/// Lane 0 of a SCALAR column is the column itself: a scalar is a one-lane vector, and answering
/// anything else would be inventing a distinction the data does not have.
fn component(c: &Column, k: usize, n: usize) -> Vec<f32> {
    fn lane<const W: usize>(v: &[[f32; W]], k: usize, n: usize) -> Vec<f32> {
        if k >= W || v.len() != n {
            return vec![0.0; n];
        }
        v.iter().map(|p| p[k]).collect()
    }
    match c {
        Column::Scalar(v) if k == 0 && v.len() == n => v.clone(),
        Column::Vec2(v) => lane(v, k, n),
        Column::Vec3(v) => lane(v, k, n),
        Column::Vec4(v) => lane(v, k, n),
        _ => vec![0.0; n],
    }
}

/// The named column as a length-N field. Missing / mistyped → zeros (see the module docs).
fn field(s: &Stream, name: &str, mode: i32) -> Vec<f32> {
    let n = s.count();
    match (s.get(name), mode) {
        // The component rung goes FIRST so the two arms below stay textually what they were:
        // modes 0 and 1 are byte-identical to the day before this rung existed.
        (Some(c), m) if m >= MODE_COMPONENT_BASE => {
            component(c, (m - MODE_COMPONENT_BASE) as usize, n)
        }
        // ⚠️ O `MODE_ANGLE` entra na EXCLUSÃO junto com o `MODE_LENGTH`: pedir a direção de uma
        // coluna ESCALAR não quer dizer nada, e a lei do módulo para uma pergunta que a coluna
        // não pode responder é a falha ORDINÁRIA (zeros), nunca a coluna verbatim. Sem esta
        // metade, `Direction` sobre um escalar devolveria o próprio número como se fosse um
        // ângulo — a mentira mais quieta que este nó sabe contar.
        (Some(Column::Scalar(v)), m) if m != MODE_LENGTH && m != MODE_ANGLE && v.len() == n => {
            v.clone()
        }
        (Some(Column::Vec2(v)), MODE_LENGTH) if v.len() == n => v
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1]).sqrt())
            .collect(),
        // A DIREÇÃO — ver [`MODE_ANGLE`]. Graus, porque é o que a coluna `rot` fala; `libm`,
        // porque o número atravessa o cozido; e `atan2(0, 0)` é `0`, que é a resposta certa
        // para um elemento parado (ele não tem direção, e não pode girar).
        (Some(Column::Vec2(v)), MODE_ANGLE) if v.len() == n => v
            .iter()
            .map(|p| libm::atan2f(p[1], p[0]).to_degrees())
            .collect(),
        _ => vec![0.0; n],
    }
}

struct ValueAttribute;

impl NodeOp for ValueAttribute {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i32;
        let out = {
            let name = ctx.text_param(ATTR_KEY).unwrap_or("").to_string();
            let input = ctx.input(0);
            let v = field(input, &name, mode);
            Stream::new(v.len()).with("v", Column::Scalar(v))
        };
        ctx.emit(out);
    }
}

/// The artist-facing channels (plan §1.1): a human word, the column it reads, and HOW —
/// the scalar itself (`0`), a Vec2's MAGNITUDE (`MODE_LENGTH`, so `vel` reads as *speed*)
/// or one LANE of a vector column (`MODE_COMPONENT_BASE + k`, which is how "Opacity"
/// reaches the alpha inside `tint`). Custom keeps the arbitrary-column reach the node was
/// built for.
///
/// ⚠️ **This list used to claim *"seven of them + Custom = 8 = the segmented selector's
/// ceiling"*, and the sentence was TRUE the day it was written — `MAX_ENUM_OPTIONS` really
/// was 8.** It stopped being true **inside this same line**, at the shapes wave
/// (`525946b58`, *"35 das 43 formas do `source.shape` eram INALCANÇÁVEIS"*), which raised the
/// cap to **48** and made `CHANNELS_EXTRA_BASE` derive from it; the row also **wraps at four
/// columns**, growing its own height, so eight + Custom paints as three rows rather than
/// overflowing. Nobody reconciled this sentence with the number its own line had moved —
/// which is the §0 rule biting at home: *quem move o número que tornava algo inalcançável tem
/// de reconferir a nota*. The executable form now lives in the shell
/// (`the_channel_picker_fits_the_panels_ceiling`), where the table and the cap meet.
///
/// ⚠️ **`pub` porque um gate FORA desta crate a lê** — o
/// `every_non_scalar_column_is_reachable_or_deliberately_hidden` do shell varre o registry
/// inteiro e cruza esta tabela com a denylist do picker. Ele não caberia aqui: esta crate
/// não vê os outros nós, e o `registry-init` que os vê não pode ser dependência dela.
///
/// ⚠️ **Every entry must name a column something WRITES** — an entry is a promise that a
/// word yields a quantity, and a word that resolves to nothing takes the module's ordinary
/// miss (zeros at full length) with no way for the artist to tell it apart from a typo.
/// The gate that holds this builds the stream a producer leaves and reads it back through
/// the entry itself, rather than comparing the table against a second list of names.
pub const READ_CHANNELS: &[ReadChannel] = &[
    // ⚠️ **A POSIÇÃO — a coluna mais básica do stream, e a ÚNICA que não tinha entrada.**
    // Todo gerador escreve `P` e todo transform a lê; ela estava ausente desta lista por
    // omissão, não por decisão (nenhuma cerca a mencionava). O relato veio do Enio a olhar
    // o painel: *"em Custom temos um P — de onde vem e o que significa?"*.
    //
    // ⚠️ **E o `Custom…` NÃO a alcançava, o que é a metade que faz destas linhas uma cura e
    // não um atalho.** O picker de coluna viva escreve o nome com o **modo 0** (escalar), e
    // uma `Vec2` lida em modo 0 cai no `_` da escada: **zeros no comprimento cheio**, em
    // silêncio, indistinguível de um nome mal digitado. Ou seja: quem digitasse `P` à mão
    // recebia zeros e nada na tela dizia porquê. É exactamente por isso que `Speed`, `Size`
    // e `Direction` são entradas — **uma coluna `Vec2` só é alcançável POR UMA ENTRADA**, e
    // uma que não tenha é inalcançável pelo artista, ainda que o cook a leia.
    //
    // As quatro são as duas leituras que um vetor tem, nas duas bases: cartesiana (as
    // lanes) e polar. ⚠️ `Angle` é *onde a peça ESTÁ* (em torno da origem do mundo);
    // `Direction`, ali abaixo, é *para onde ela VAI*. E `Radius`/`Angle` dizem no rótulo o
    // que `Distance` esconderia: a referência é a **origem**, porque não há segunda entrada
    // de onde ser distante.
    //
    // ⚠️ Trap 1 medido antes de escrever: as lanes a composição **não dá** (o `Custom` é
    // escalar-only); o `Radius` daria em **seis** nós (`X·X + Y·Y → value.unary(Sqrt)`); e o
    // `Angle` a composição **não consegue de todo** — `ph2d-expr` está FROZEN sem `atan2` e
    // nada no domínio de valor computa uma tangente inversa (a mesma razão que fez o
    // `MODE_ANGLE` nascer).
    ReadChannel {
        label: "Position X",
        column: "P",
        mode: MODE_COMPONENT_BASE,
    },
    ReadChannel {
        label: "Position Y",
        column: "P",
        mode: MODE_COMPONENT_BASE + 1,
    },
    ReadChannel {
        label: "Radius",
        column: "P",
        mode: MODE_LENGTH,
    },
    ReadChannel {
        label: "Angle",
        column: "P",
        mode: MODE_ANGLE,
    },
    ReadChannel {
        label: "Speed",
        column: "vel",
        mode: MODE_LENGTH,
    },
    // ⚠️ **A OUTRA METADE DO MESMO VETOR** (doc 89 §10.0, a linha que cinco famílias citaram):
    // `Speed` responde *quão rápido* e descarta *para onde*, e era essa a ausência — não a da
    // coluna, que sempre esteve lá. Com esta entrada,
    // `value.attribute(Direction) → motion.drive(Rotation)` é o *align to velocity*, em dois nós.
    // A unidade é GRAUS porque é o que o `rot` do outro lado fala; ver [`MODE_ANGLE`].
    ReadChannel {
        label: "Direction",
        column: "vel",
        mode: MODE_ANGLE,
    },
    // ⚠️ **`tint` lane 3, not a column called `"opacity"`.** This entry used to name a
    // column that NOTHING in the library writes: `motion.drive`'s opacity channel writes
    // `tint` (`CH_OPACITY => "tint"`, lane 3) and `lower_to_instances` reads the alpha from
    // exactly there — `RenderInstance.opacity` is hardcoded to `1.0`, there is no
    // per-instance opacity surface. So picking "Opacity" took the module's ORDINARY MISS
    // and handed back zeros at full length, in silence, indistinguishable from a typo:
    // reading back the opacity a `motion.drive` had just written was inexpressible.
    //
    // The lane rung (`MODE_COMPONENT_BASE`, W0-A) is what makes the honest answer sayable —
    // before it, a `Vec4` column had no reachable channel at all.
    ReadChannel {
        label: "Opacity",
        column: "tint",
        mode: MODE_COMPONENT_BASE + 3,
    },
    ReadChannel {
        label: "Rotation",
        column: "rot",
        mode: 0,
    },
    ReadChannel {
        label: "Size",
        column: "size",
        mode: MODE_LENGTH,
    },
    ReadChannel {
        label: "Age",
        column: "age",
        mode: 0,
    },
    ReadChannel {
        label: "Life",
        column: "life",
        mode: 0,
    },
    ReadChannel {
        label: "Seed",
        column: "seed",
        mode: 0,
    },
    // ⚠️ **O peso de um CAMPO** — a coluna que as cinco `field.*` escrevem (`field.box`,
    // `field.combine`, `field.index_range`, `field.radial_sweep`, `field.remap`) e que o
    // `motion.falloff` também produz. Ela era consumida por SEIS `motion.*`
    // (`Consumes("falloff")`: step · delay · slit_scan · spline_wrap · drop_shadow ·
    // rgb_split) e **por nenhum nó do domínio de valor** — então *"quanta influência este
    // campo tem AQUI?"* era uma pergunta que a arte podia sentir e o grafo não podia dizer.
    //
    // É o que faltava para o portão espacial da folha 12 (`SUPERAR:` item 3): com esta
    // entrada, `field.box → value.attribute(Falloff) → value.math(Multiply, pulse.level) →
    // pulse.compare` é *"dispare só quem está dentro da caixa"* — a combinação que nenhuma
    // referência tem (o C4D tem Fields e zero eventos; o Niagara tem eventos e zero campos
    // componíveis). O gate da cadeia vive na `ph2d-node-registry-init`.
    ReadChannel {
        label: "Falloff",
        column: "falloff",
        mode: 0,
    },
    // ⚠️ **O CONTATO** — a coluna que o `sim.collide` escreve (doc 89, folha 13 P1): quão fundo
    // a colisão deste tique empurrou o elemento de volta, e `0` onde nada tocou. Sem esta
    // entrada o canal existiria e **nenhum nó do domínio de VALOR poderia lê-lo**, que é a
    // mesma metade faltante que o `Falloff` tinha: a arte sentia e o grafo não sabia dizer.
    //
    // É ela que torna a colisão COMPONÍVEL, e a cadeia inteira é de nós que já existem —
    // `sim.collide → value.attribute(Hit) → value.math → motion.drive(<canal>)` marca quem
    // tocou, e com `motion.drive(Falloff) → motion.cull` **dentro da zona** ele MORRE ao
    // tocar (o `sim.collision_pulse` da linha 98 do doc 63, sem um nó novo).
    ReadChannel {
        label: "Hit",
        column: "hit",
        mode: 0,
    },
    // ⚠️ **A VIZINHANÇA** — as duas colunas que o `motion.proximity` escreve (doc 89, folha
    // 03 linhas 42 e 61). Elas são a metade que faltava para os três modos do *Push Apart*
    // do C4D serem COMPOSIÇÃO em vez de um param `mode` dentro do solver: com `Overlap`,
    // `proximity → value.attribute(Overlap) → value.math(Subtract) → motion.drive(Size,
    // Multiply)` **é** o modo Scale, e `→ motion.drive(Falloff) → motion.cull(invert)` é o
    // Hide. Sem estas entradas as colunas existiriam e só um `Custom` com o nome digitado à
    // mão as alcançaria — a mesma metade faltante que o `Falloff` e o `Hit` tinham.
    //
    // `Neighbours` é uma CONTAGEM (não uma densidade normalizada): quem quiser a fracção
    // compõe um `value.map_range`, e um número já dividido por um máximo que o nó escolheu
    // seria uma decisão de apresentação assada no dado.
    ReadChannel {
        label: "Neighbours",
        column: "neighbours",
        mode: 0,
    },
    ReadChannel {
        label: "Overlap",
        column: "overlap",
        mode: 0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    // ONE picker: the artist chooses a named channel; the panel writes both the column
    // (`attr`) and the `mode` behind it. "mode" gets no row of its own — folded in.
    ParamUiHint {
        param: ATTR_KEY,
        label: "Read",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Channels {
            mode_param: "mode",
            channels: READ_CHANNELS,
        },
    },
];

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueAttribute))?;
    // ADR-0136: the column's NAME is a text param — dynamic, inexpressible as a
    // static binding — so the projection runs in the sequencer's machinery
    // (`StreamOp::Project`), which resolves the name against the live stream's
    // column map and replays [`field`]'s exact ladder (copy / length / zeros).
    reg.register_gpu_kernel(MANIFEST.id, GpuKernel::PASSTHROUGH);
    reg.register_stream_op(
        MANIFEST.id,
        StreamOp::Project {
            text_param: ATTR_KEY,
            mode_param: "mode",
        },
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Attribute",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
