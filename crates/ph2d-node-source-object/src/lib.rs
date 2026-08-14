#![forbid(unsafe_code)]
//! `source.object` — **bring an engine object into the graph** (doc 86 §2).
//!
//! The graph produces WHERE (a stream of positions); it never had a way to say
//! WHAT — no sprite, no vector, no Flip could be the thing an instancer stamps.
//! This node is that door: it names an object the app owns, and emits **one
//! instance carrying that object's appearance** — its `texture_id` (which
//! texture), `uv_rect` (its atlas cell), `size` and `tint`. Cross it with a
//! `motion.grid` through a `motion.duplicator` and the object is stamped at
//! every point.
//!
//! ## The same door `motion.path` uses, for the same reason
//!
//! A node is handed its params, its inputs and the playhead — and nothing else
//! (the property that lets the cook memoize and replay bit-exactly). So it
//! **cannot reach into the ECS**. The app publishes the object under its **name**
//! into the cook's [external channel](ph2d_nodegraph::external), and this node
//! reads that name — exactly as `motion.path` reads a drawn curve. The membrane
//! (`shells/desktop/.../motion_bridge_objects.rs`) resolves *sprite → tile
//! directly*, and *vector/Flip/group → a baked tile* (later waves); this node is
//! **media-agnostic** — it reads whatever stream the membrane published, so a
//! new media type grows the membrane, never this node.
//!
//! **The name is the artist's, and it is the whole reference.** Name a sprite
//! `Ball` in the Hierarchy and type `Ball` here — no id to copy, nowhere for the
//! two to disagree. Rename the object and the node follows it, because the name
//! IS the reference. An object that is not there (unnamed, deleted, not yet
//! resolvable) is an **empty external** → an empty stream: the node emits
//! nothing, it does not guess and it does not fail.
//!
//! `Effect::Pure` — the output is a pure function of the named external, whose
//! own content-revision rides in the cook's fingerprint, so moving/retinting the
//! object re-cooks this node and only what is downstream of it.

use ph2d_node_registry::{
    NodeRegistry, ParamHardMax, ParamHardMin, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
    RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The **text** param naming the object (the channel doc 32 opened — a
/// `ParamSpec` is f32-only and the manifest is frozen, so a string param lives
/// in the `Graph` beside the manifest, not in it; the same slot `motion.path`
/// uses for its `path` name).
const OBJECT_PARAM: &str = "object";

/// **Quando** este nó olha para o objeto — segundos à frente (ou atrás) do playhead.
///
/// A tile de um objeto Flip é assada **no shell, uma por objeto, no quadro atual do
/// app**: dois `source.object` do mesmo Flip recebiam o MESMO desenho, e nenhum nó a
/// jusante podia pedir outro. Uma cascata de cópias, cada uma num desenho diferente —
/// o par canônico com um stagger, e o `Shape Time Offset` de toda referência que tem
/// um — era **inexprimível**.
///
/// ⚠️ **Zero é o mundo de sempre, e é o próprio nome do canal:** com `0.0` este nó lê
/// o external do nome cru, exatamente como sempre leu, e o shell não assa tile nenhuma
/// a mais ([`ph2d_nodegraph::external::appearance_of`] devolve o nome). Não existe um
/// estado "deslocado de nada" para manter de acordo com o não-deslocado.
pub const TIME_OFFSET_PARAM: &str = "time_offset";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.object"),
    name: "source.object",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        // Segundos de deslocamento no relógio deste objeto. `0.0` = o quadro atual,
        // que é o único frame que este nó soube pedir até 2026-08-13.
        name: TIME_OFFSET_PARAM,
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

struct SourceObject;

impl NodeOp for SourceObject {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let name = ctx.text_param(OBJECT_PARAM).unwrap_or_default().to_string();
        // ⚠️ A chave é mintada pela porta ÚNICA que o shell também chama. As duas
        // pontas são folhas que não podem depender uma da outra, e um float
        // formatado em dois sítios é uma divergência esperando uma mudança de
        // precisão — aqui há um sítio só, e ele devolve o NOME CRU no offset zero.
        let key = ph2d_nodegraph::external::appearance_of(&name, ctx.param(TIME_OFFSET_PARAM));
        // The membrane published `(P, size, tint, uv_rect, texture_id)` under
        // this key. Clone is refcount, not a copy (columns are `Arc`); a key
        // with no published object is the empty external → an empty stream.
        let stream = ctx.external(&key).clone();
        ctx.emit(stream);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceObject))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Object",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Circle,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    // Its output carries `texture_id` (the engine object's tile). The GPU cook
    // now DRAWS this — the lowering writes the id into the instance and the
    // renderer binds the object's texture per run — so it is an OBJECT source,
    // not a blanket recusal. The shell reads this flag only for the
    // count-changing cerca: an object graph whose GPU suffix reorders / changes
    // count recuses (the texture-run partition would mis-bind).
    reg.register_object_source(MANIFEST.id);
    Ok(())
}

/// **O passo que alcança TODO desenho no fps mais rápido do domínio.**
///
/// Não é um número escolhido: a faixa de `fps` que a tira oferece é `[1, 120]`
/// (`ph2d-panel-flip-frames`), então um quadro no extremo rápido dura `1/120 s` —
/// um passo maior deixaria desenhos inalcançáveis pelo stepper num objeto a 120 fps,
/// que é precisamente o objeto em que o offset é mais fino.
const OFFSET_STEP: f32 = 1.0 / 120.0;

/// **O teto MEDIDO, derivado do domínio — não um palpite de segurança.**
///
/// A magnitude do offset não consome recurso nenhum: a tile assada tem o mesmo
/// tamanho deslocada ou não, e *quantas* tiles existem é limitado por construção
/// (o bake despeja por quadro o que ninguém pediu, então o conjunto vivo é a
/// contagem de nós `source.object`, não a de offsets já visitados).
///
/// O que existe é um limite de **SIGNIFICADO**: passado o vão da própria animação o
/// desenho segura, e o offset deixa de mostrar coisa nova. O maior vão que um
/// documento pode expressar é a exposição máxima de uma chave — `HOLD_MAX = 999`
/// quadros — no fps mais lento que a tira oferece (`1`), o que dá **999 s**. Além
/// disso não há Flip alcançável com outro desenho para mostrar, em nenhum fps.
const OFFSET_HARD: f32 = 999.0;

/// Duas rows: o nome do objeto (o picker) e QUANDO olhar para ele.
///
/// ⚠️ A faixa confortável do arrasto é ±2 s — 48 desenhos a 24 fps, que é a escala de
/// uma cascata de cópias —, e o *disfuncional* começa muito mais longe
/// ([`OFFSET_HARD`]), que é a divisão que o doc 88 B2 instalou: o slider é o gesto,
/// a caixa de texto é o alcance.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: OBJECT_PARAM,
        label: "Object",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    ParamUiHint {
        param: TIME_OFFSET_PARAM,
        label: "Time Offset",
        min: -2.0,
        max: 2.0,
        step: OFFSET_STEP,
        widget: ParamWidget::Slider,
    },
];

/// O offset é tempo de RELÓGIO, e é a unidade que o diz. Um número nu aqui leria
/// como *quadros* — que é a outra lei, a que este nó deliberadamente não tem.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: TIME_OFFSET_PARAM,
    unit: ParamUnit::Seconds,
}];

/// O piso e o teto digitáveis são simétricos porque o offset é uma DIREÇÃO: uma cópia
/// pode mostrar um desenho anterior tão legitimamente quanto um posterior.
static PARAM_HARD_MIN: &[ParamHardMin] = &[ParamHardMin {
    param: TIME_OFFSET_PARAM,
    min: -OFFSET_HARD,
}];

static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: TIME_OFFSET_PARAM,
    max: OFFSET_HARD,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
