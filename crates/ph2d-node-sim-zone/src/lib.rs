#![forbid(unsafe_code)]
//! **The Simulation Zone** (Motion Nodes O4, doc 48) — the highest ceiling the module's
//! re-entrancy study set out, and the last one left standing.
//!
//! ## What it is
//!
//! A zone holds a stream of live state across ticks. Whatever chain the artist wires into its
//! `state` port is the zone's **interior**: it receives last tick's state, does anything at all
//! to it, and hands it back. Then the zone emits it — and holds it for the next tick.
//!
//! ```text
//!   grid ─→ init ┌──────────┐ out ─→ output
//!                │ sim.zone │
//!        state ←─┴──────────┘
//!          ↑                      ⊙ ← the interior gets last tick's state here
//!   force.wind → sim.step → motion.cull
//! ```
//!
//! ## Why it beats the forces branch it joins (and does not replace)
//!
//! `motion.integrate`'s `forces` port (O1) is a zone with an implicit boundary: its interior
//! may only ACCUMULATE acceleration, because the integrator owns the state and everything on
//! the branch is `Pure`. A zone's interior owns nothing and may do everything — and that turns
//! nodes the library already has into things they could not be before:
//!
//! - **`motion.cull` becomes a KILL.** Outside a zone, culling drops elements from *this
//!   frame's* stream and the next frame rebuilds them. Inside, the state carries the survivors
//!   forward — so what you cull STAYS dead. Blender's zone and Houdini's POP kill, out of a
//!   filter that was already there.
//! - **`motion.combine` becomes a BIRTH.** Merge newborns into the live state and they persist,
//!   with their own history, instead of being re-emitted every frame by a stateless emitter.
//!
//! No new machinery bought either of those. The zone did.
//!
//! ## The engine underneath is the one that was already there
//!
//! The `state` port is a **feedback host** (the O1 convention: an input named `state` or
//! `forces`, of output 0's type). The editor's plumbing owns the `pre` edge — self-loop when
//! the port is bare, into the interior's head when a chain is wired in — and draws it as portal
//! badges rather than a spline. The zone adds no engine, no `Domain`, no contract: it is a node.
//!
//! ## The two things Blender's users found out the hard way
//!
//! 1. **A bare zone FREEZES its input.** Wire nothing into `state` and the plumbing self-loops
//!    it: tick 1 emits `init`, and every tick after re-emits what it emitted before. The
//!    Blender manual's users hit this as a surprise (*"even just a simulation zone that does
//!    nothing freezes the mesh"*) — so the port is called **`init`**, not `in`. It is the
//!    INITIAL state, read once. The name is the documentation.
//! 2. **An empty sim is not an unstarted one.** See `eval` below: this is what
//!    `EvalCtx::started` exists for, and getting it wrong resurrects the dead.

use ph2d_node_registry::{NodeRegistry, ParamHardMax, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{GpuKernel, StateSelect};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The initial state — read on the first tick and never again (Blender's frozen zone input,
/// named so that the surprise is impossible).
const IN_INIT: usize = 0;
/// The live state coming back from the interior. A **feedback host** port name (`state`), so
/// the editor's plumbing wires the `pre` edge and the artist never draws the loop.
const IN_STATE: usize = 1;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("sim.zone"),
    name: "sim.zone",
    inputs: &[
        PortSpec {
            name: "init",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // It consumes a `pre` edge, so the cook already treats it as sequential (it may not run
    // inside a rewritten time scope, and it never reuses a memo across ticks). `Temporal` is
    // what integrate/spring declare for the same reason.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    // ⚠️ **Este param não é lido pelo `eval` — ele é lido pelo SEQUENCIADOR**, e isso é o
    // desenho, não um descuido: um substep não é uma conta que a zona faz, é quantas vezes o
    // relógio pede que o interior dela corra ([`ph2d_nodegraph::cook::Cook::substep`]). O param
    // mora aqui porque é aqui que o artista o procura e porque o painel se auto-popula da lista;
    // ele entra no fingerprint do cook pela porta de sempre, então mudá-lo re-cozinha.
    params: &[
        ParamSpec {
            name: "substeps",
            default: 1.0,
        },
        // ── O CICLO DE VIDA (doc 89, folha 13 · o *Emitter State* do Niagara) ──
        //
        // ⚠️ **Estes QUATRO são lidos pelo `eval`**, ao contrário do `substeps` acima — e é
        // por isso que são params comuns e não metadado lateral: a cerca 2 da folha fala do
        // que o SEQUENCIADOR precisa de saber, e o preço do device está pago pelo
        // `applicable` do kernel (ver [`ZONE_KERNEL`]).
        //
        // Os defaults (`start = 0`, `duration = 0` ⇒ **para sempre**) devolvem a zona que
        // shipava — e não por promessa: com eles a maquinaria do ciclo **não corre**
        // (`Life::is_default`), e o `eval` toma exactamente o ramo de antes.
        ParamSpec {
            name: "start",
            default: 0.0,
        },
        // `0` Forever (a zona que shipava) · `1` Once · `2` Loop — ver [`life::Mode`].
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        ParamSpec {
            name: "duration",
            default: 2.0,
        },
        ParamSpec {
            name: "loop_delay",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **O teto é do RELÓGIO DE PAREDE, e a tabela é esta** (§0: meça antes de limitar). Custo por
/// QUADRO de uma zona substepada, contra o orçamento de 60 fps (16,67 ms) —
/// `measure_substeps::measure_what_a_substep_costs_per_frame`:
///
/// | partículas | sub=1 | sub=8 | sub=16 | sub=32 | sub=64 |
/// |---|---|---|---|---|---|
/// | 256 | 0,007 | 0,060 | 0,119 | 0,239 | 0,461 |
/// | 4.096 | 0,086 | 0,668 | 1,371 | 2,695 | 5,404 |
/// | 16.384 | 0,219 | 1,967 | 3,914 | 7,903 | **15,910** |
///
/// O custo é **linear em `n`** (e em partículas), então o número que decide é onde uma zona
/// pesada come o quadro: a 16.384 partículas `sub = 64` custa **95% de um quadro**. Daí o par do
/// slider dual (doc 88): a faixa CONFORTÁVEL do arrasto para em **16** — o erro cai pela metade a
/// cada dobra e já está em 0,6% ali, então acima disso o retorno não se vê — e o teto DIGITÁVEL
/// para em **64**, onde o disfuncional começa a ser medível em vez de opinável.
const MAX_SUBSTEPS: f32 = 64.0;

/// **The transients: scratch a tick writes for itself, and the zone must NOT hold.**
///
/// - `accel` is the force nodes' accumulator, spent by the step that applies it.
/// - `falloff` is a MASK — a field a `motion.falloff` computes so the modifiers after it can
///   scale their effect by it (§1.2). It describes *this tick's* authoring intent, not the
///   element.
///
/// A zone that stored them would be a loop that feeds a tick its own leftovers, and the demo
/// found out what that costs: the kill's mask (`falloff` from `motion.falloff`, consumed by
/// `motion.cull`) rode the state back around and **masked the very gravity that made it** —
/// the fringe of the seed stopped falling — and then leaked out of the zone and scaled the
/// `motion.move` that positions the scene, stretching it. Every symptom of a state that was
/// remembering scratch.
///
/// **The zone stores state, not scratch.** What the elements ARE (`P`, `vel`, `id`, `size`,
/// `tint`…) survives; what a tick wrote for its own use does not. Anything downstream that wants
/// a mask computes one — masks are cheap and a stale one is not a mask, it is a ghost.
/// ⚠️ **`hit` is the third for exactly this reason** (`sim.collide`, doc 89 folha 13): it says
/// *"this tick's collision pushed me out by this much"*, and a tick is precisely how long that
/// is true. Stored, it would report a contact on the tick AFTER the element stopped touching,
/// and every reader downstream — a colour flash, a kill, a pulse — would repeat it.
const TRANSIENTS: [&str; 3] = ["accel", "falloff", "hit"];

/// The state as the zone holds it: every column but the transients.
fn store(s: &Stream) -> Stream {
    let mut out = Stream::new(s.count());
    for (name, col) in s.columns() {
        if !TRANSIENTS.contains(&name.as_str()) {
            out.set(name.clone(), col.clone());
        }
    }
    out
}

mod life;

struct SimZone;

impl NodeOp for SimZone {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // **The whole zone is this branch, and it took two wrong answers to find the right one.**
        //
        // The question is *"has the sim started?"*, and it is NOT
        //
        // - *"is my `state` empty?"* — a sim that killed its last element hands back an EMPTY
        //   STREAM, a real answer that carries nothing. Read that as "not started" and the zone
        //   re-seeds from `init`: kill every particle and the scene RESURRECTS, one frame later,
        //   forever.
        // - *"did an edge deliver a value on `state`?"* — it always did. The interior is wired
        //   into `state` by a FORWARD edge, so the cook runs it BEFORE the zone, and on tick 1
        //   it dutifully hands back an empty stream (the absent value was its own input: the
        //   zone's previous output, through the `pre` edge).
        //
        // The state of a zone lives on the zone's OWN previous output. So that is what it asks:
        // did *I* emit anything last tick? (`EvalCtx::started` — doc 48.)
        //
        // ⚠️ **O CICLO DE VIDA é uma SEGUNDA porta ao lado desta, nunca uma substituição.**
        // Com os defaults o `is_default` corta fora a maquinaria e o que corre é o ramo acima.
        //
        // ⚠️⚠️ **E o que esse corte COMPRA teve de ser achado por mutação.** A 1.ª redacção
        // dizia *"a maquinaria não corre"* e **nenhuma mutação matava a afirmação**: para
        // `t >= 0` a lei geral concorda com este ramo tique a tique, então desligar o
        // curto-circuito deixava a suíte verde. *Uma afirmação que mutação nenhuma mata é uma
        // afirmação sobre nada.* O que ele compra é **a zona de sempre não perguntar as
        // horas**: num relógio NEGATIVO a lei do ciclo diria `Dormant`, e ela semeia. É a
        // diferença inteira, e o gate `the_default_zone_does_not_ask_the_clock_what_time_it_is`
        // é quem a prende.
        let life = life::Life::of(&|name| ctx.param(name));
        if life.is_default() {
            let held = if ctx.started() {
                ctx.input(IN_STATE)
            } else {
                ctx.input(IN_INIT)
            };
            ctx.emit(store(held));
            return;
        }
        // ⚠️ **`Nothing` é um stream VAZIO, e ele fecha o laço sozinho:** o interior lê a
        // saída anterior da zona por uma aresta atrasada, então uma vez emitido o vazio o
        // interior cozinha vazio e devolve vazio — a sim fica de facto parada, sem nenhum
        // nó a jusante precisar de saber que houve um ciclo.
        let held = match life.emit(ctx.playhead(), ctx.dt(), ctx.started()) {
            life::Emit::Nothing => {
                ctx.emit(Stream::new(0));
                return;
            }
            life::Emit::Seed => ctx.input(IN_INIT),
            life::Emit::Carry => ctx.input(IN_STATE),
        };
        ctx.emit(store(held));
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SimZone))?;
    // GPU (ADR-0135): the zone is a conditional passthrough. The PASSTHROUGH
    // kernel makes the plan claim it (no compute pass emitted); the `StateSelect`
    // tells the sequencer to forward `init` before the loop has state and `state`
    // after, stripping the SAME `TRANSIENTS` the CPU `store()` strips (one list,
    // two consumers — they cannot drift).
    reg.register_gpu_kernel(MANIFEST.id, ZONE_KERNEL);
    reg.register_state_select(
        MANIFEST.id,
        StateSelect {
            init_port: IN_INIT,
            state_port: IN_STATE,
            transients: &TRANSIENTS,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Simulation Zone",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

/// O `IntSlider` é o que faz a unidade cair em `ParamUnit::Count` sem uma 2ª declaração —
/// um substep é uma CONTAGEM, e meio substep não quer dizer nada.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "substeps",
        label: "Substeps",
        min: 1.0,
        max: 16.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    // ── O CICLO DE VIDA ──────────────────────────────────────────────────────
    ParamUiHint {
        param: "mode",
        label: "Life Cycle",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Forever", "Once", "Loop"],
        },
    },
    // ⚠️ O `start` vale nos TRÊS modos — atrasar o começo não tem nada a ver com ter fim.
    ParamUiHint {
        param: "start",
        label: "Start",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "duration",
        label: "Duration",
        min: MIN_DURATION_UI,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "loop_delay",
        label: "Loop Delay",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

/// O piso do slider da duração.
///
/// ⚠️ **DERIVADO do piso da lei, não escrito ao lado dele.** Dois literais iguais em dois
/// arquivos são duas respostas à mesma pergunta, e a que envelhece é sempre a que ninguém lê —
/// um gate a compará-los só apanharia a divergência depois de ela existir. Aqui ela não pode
/// existir. (A conversão perde ~2e-10, que é a precisão do `f32` e não uma discordância.)
#[expect(
    clippy::cast_possible_truncation,
    reason = "um piso de slider e' f32 por contrato do `ParamUiHint`"
)]
const MIN_DURATION_UI: f32 = life::MIN_DURATION as f32;

/// **Só os knobs que o modo escolhido de facto LÊ aparecem** — a lei do `count`/`spacing` do
/// `motion.path`, e a razão de o ciclo ser um modo e não uma sentinela: em `Forever` a duração
/// e o descanso não mudam um quadro, e um controle pintado que não é lido é o defeito que a
/// caça aos knobs mortos (doc 90) desta linha existiu para apagar.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "duration",
        when: "mode",
        values: &[1, 2],
    },
    ph2d_node_registry::ParamGate {
        param: "loop_delay",
        when: "mode",
        values: &[2],
    },
];

/// **O que cada número É** (doc 88): os três do ciclo são DURAÇÕES. O `substeps` é uma
/// contagem, e o `IntSlider` já a declara sem uma 2.ª entrada.
static PARAM_UNITS: &[ph2d_node_registry::ParamUnitDecl] = &[
    ph2d_node_registry::ParamUnitDecl {
        param: "start",
        unit: ph2d_node_registry::ParamUnit::Seconds,
    },
    ph2d_node_registry::ParamUnitDecl {
        param: "duration",
        unit: ph2d_node_registry::ParamUnit::Seconds,
    },
    ph2d_node_registry::ParamUnitDecl {
        param: "loop_delay",
        unit: ph2d_node_registry::ParamUnit::Seconds,
    },
];

/// O teto digitável, MEDIDO — a tabela está no doc-comment de [`MAX_SUBSTEPS`].
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "substeps",
    max: MAX_SUBSTEPS,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "life_tests.rs"]
mod life_tests;

/// **O kernel da zona — a passagem de sempre, com o preço do CICLO DE VIDA nomeado.**
///
/// A zona é um *conditional passthrough* (ADR-0135): o plano a reivindica sem emitir passe de
/// compute, e o `StateSelect` diz ao sequenciador que porta encaminhar. ⚠️ **O ciclo de vida é
/// uma decisão que o sequenciador não sabe tomar** — ele conhece «antes de haver estado» e
/// «depois», não um relógio com atraso, duração e repetição.
///
/// ⚠️ **Então o ciclo de vida é CPU-only, e o bloqueador é este:** ensinar a fase ao device
/// significaria pôr o relógio dentro do `StateSelect`, que é o metadado lateral que a cerca 2 da
/// folha 13 reserva. Com os defaults nada disso acontece — `applicable` devolve `true`, o
/// `StateSelect` decide como sempre, e a zona continua **residente na GPU**. É o precedente
/// exacto do `motion.emitter` (os modos novos dele são CPU-only, com o bloqueador escrito ao
/// lado), e o preço está na cerca 1: *um nó que o device não reivindica custa a residência do
/// laço inteiro*.
const ZONE_KERNEL: GpuKernel = GpuKernel {
    // ⚠️ **A MESMA porta que o `eval` usa** (`Life::of`) — se cada lado lesse os params à
    // mão, o device podia reivindicar uma zona que a CPU já considera ciclada.
    applicable: Some(|p| life::Life::of(p).is_default()),
    ..GpuKernel::PASSTHROUGH
};
