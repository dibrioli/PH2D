#![forbid(unsafe_code)]
//! `motion.pin_constraint` — **nail elements down**: mark part of a stream as
//! immovable so the simulation flows around it while the pinned elements stay
//! driven by the upstream animation (Motion Nodes M3, simulation — doc 01 §3 /
//! doc 34). The hanging cloth's top corners, the anchor a rope swings from, the
//! obstacles a crowd has to avoid.
//!
//! **The primitive is inverse mass, not a boolean.** The gold standard is
//! Position Based Dynamics (Müller et al., 2007): every particle carries
//! `w = 1/m`, and a constraint's correction is distributed in proportion to the
//! `w`s of the particles it touches. `w = 0` is infinite mass — no force, no
//! contact and no constraint can move it. It is the same primitive as Houdini
//! Vellum's `pintoanimation`, Blender's cloth **pin group** (whose vertex weight
//! is likewise a *partial* pin) and Bullet's `invMass`; a bool would give the
//! hard pin only, and would not let a heavier-than-its-neighbours element merely
//! resist. So this node writes a per-element [`INV_MASS`] column and the
//! solvers read it.
//!
//! ```text
//! grid ── pin_constraint ──> integrate ──> output      (the pinned run holds still;
//!           (one row's worth)    ^                      the free ones fall/blow away)
//!                                └── force.wind
//! ```
//!
//! **Mind which row you are naming.** The index range is exact but it is not geometry:
//! `motion.grid` is row-major from the LOWEST y up, so its *first* `cols` elements are
//! the row at the BOTTOM of the screen and a curtain hangs from the LAST ones. To select
//! by shape instead of by index — the safer habit — put a `motion.falloff` upstream and
//! pin the region it covers.
//!
//! **Who reads it:** `motion.integrate` (the force chain's one integrator),
//! `motion.spring` and `motion.collide` — every node that takes an instance
//! stream and moves it. A missing column reads as `1.0` (free), so every graph
//! authored before this node behaves exactly as it did.
//! `motion.verlet_rope` / `motion.soft_body` / `motion.boids` are *generators*
//! (they mint their own points from params and carry state), and this doc used to
//! close with *"an upstream pin has no wire to reach them through"*. ⚠️ That was
//! true of the `in` port and **false of the STATE CHAIN**, which is a wire — and
//! which was already the wire `accel` comes in through. Wire this node into a
//! generator's feedback loop and the pin lands:
//!
//! ```text
//! rope.out --pre--> pin_constraint --> rope.state      (nails point 12 of 24;
//!                                                       `first = 12, count = 1`)
//! ```
//!
//! The `pre` lives on the edge that ENTERS the pin — that is the one that breaks
//! the cycle — and this node is `Effect::Pure`, so it does not stamp `sim_t` and
//! the solver still sees its own clock next tick. Their INTRINSIC pins (the rope's
//! head/tail, the body's top row) stack with this one.
//!
//! ⚠️ **What a pinned particle FOLLOWS is the generator's answer, not this node's,
//! and the sentence that used to live here was wrong.** It read *"an intrinsic pin
//! is clamped to an ANIMATED target, a generic one holds WHERE IT IS"* — and
//! holding where it is turned out to be a world-space FREEZE, not a pin: with
//! `motion.soft_body` neither `spacing` nor the live `anchor` could reach a
//! generically pinned row (measured 2026-08-16: the anchor moved the intrinsic pin
//! `3.0000` and the generic one `0.0000`). The law today is *a particle of infinite
//! mass follows the pose the NODE knows how to PRESCRIBE*: the soft body knows
//! (`anchor + rest[i]`), so it prescribes; the rope and the flock have no positional
//! rest shape to prescribe from, so there `pos[i]` — hold where you are — remains the
//! correct answer. This node is unchanged either way: it writes `inv_mass` and the
//! generator decides what infinite mass MEANS for its own geometry.
//!
//! **Selection** is the index range `[first, first + count)` **times** the
//! multiplicative `falloff` field the module's falloff nodes write (so a
//! `motion.falloff` upstream pins a *region* — the classic "pin what the circle
//! covers"), times `strength` (a partial pin: a heavy, sluggish element rather
//! than an immovable one). `count = 0` selects nothing and the node is the
//! identity.
//!
//! Pins **compose**: the node multiplies into whatever `inv_mass` is already on
//! the stream, exactly like the falloffs multiply into `falloff`, so two pin
//! nodes stack instead of the second erasing the first.
//!
//! `Effect::Pure`, no clock, no state — the weights are a pure function of the
//! params and the incoming field. HR-5: arithmetic only.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The **inverse-mass** column — PBD's `w = 1/m`, per element. `1` = free (the
/// default when the column is absent), `0` = pinned (infinite mass: no force,
/// contact or constraint may move it), in between = heavy. The solvers scale
/// their correction by it.
///
/// The name is the one piece of this node other crates must agree on, so it
/// lives here as a `pub const` and the readers refer to it rather than
/// re-spelling the string.
pub const INV_MASS: &str = "inv_mass";

/// The module's multiplicative selection field (written by the `motion.falloff`
/// family). Absent reads as `1` — the whole stream is inside the field.
const FALLOFF: &str = "falloff";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.pin_constraint"),
    name: "motion.pin_constraint",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // A memória do que já RASGOU. APENDADO — o índice da porta 0 não se mexe, e
        // desligada o nó é byte-idêntico. Ver [`BREAK_ABOVE`].
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
        // **A CARGA** — a cadeia de forças, de onde sai o `accel`. Ver [`BREAK_ABOVE`].
        PortSpec {
            name: "load",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // The first element of the pinned run. Rounded and clamped at eval.
        ParamSpec {
            name: "first",
            default: 0.0,
        },
        // How many consecutive elements from `first` are pinned. A dropped node
        // must SHOW something, so it lands pinning the first element (the rope
        // anchor). `0` selects nothing and the node is the identity.
        ParamSpec {
            name: "count",
            default: 1.0,
        },
        // How hard: 1 = immovable (w = 0), 0.5 = twice as heavy as its
        // neighbours, 0 = free. Blender's partial pin weight.
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
        // **A CARGA QUE RASGA O PIN.** APENDADO, default `0` = nunca rasga, ao bit.
        // Ver [`BREAK_ABOVE`].
        ParamSpec {
            name: "break_above",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// A coluna transiente que as `force.*` acumulam — **a CARGA que este pin sente**.
const ACCEL: &str = "accel";
/// A memória do que já rasgou, carregada no `pre` self-loop: `1` = rasgado.
const TORN: &str = "pin_torn";

/// **O LIMIAR DE RUPTURA** (doc 89 folha 03 — o *Breaking Threshold* dos
/// `vellumconstraints` do Houdini).
///
/// Um pin com carga acima de `break_above` **ROMPE**: a partícula volta a ter massa
/// finita e vai-se embora com o resto. `0` desliga — ver abaixo porquê é `0` e não `∞`.
///
/// ⚠️ **O que rompe é o PINO, não o material** (Enio, 2026-08-21: *"não rasga o pano,
/// os cubos não se separam"*). É também o que o *Breaking Threshold* do Vellum faz numa
/// constraint de pin — o vínculo cede, a geometria sai inteira. **Partir o tecido** é
/// outra feature e depende do solver: o `motion.soft_body` guarda a forma por
/// correspondência GLOBAL, sem ligações uma-a-uma, então ali não há aresta que se
/// quebre; num solver de arestas (`motion.verlet_rope`) a pergunta faria sentido e é
/// uma célula que ninguém abriu ainda.
///
/// ## A carga entra por uma PORTA PRÓPRIA, e o porquê foi medido num smoke reprovado
///
/// A célula dizia *"nenhum solver publica a força sentida no pin"*, e isso é verdade
/// **do solver**. Mas a carga já viaja no stream antes dele: as `force.*` acumulam em
/// `accel`. A primeira versão desta wave leu-a do próprio `in` e pôs o pin **dentro do
/// laço de força** — e o smoke voltou com *"tudo foi levado pelo vento, nada rasgou"*.
///
/// ⚠️ **MEDIDO, e desmente uma nota que eu próprio escrevi:** o `motion.integrate` lê o
/// `accel` do `state` (`ctx.input(1)`) mas o **`inv_mass` do `rest`** (`ctx.input(0)`,
/// `scalar_to_n(rest, INV_MASS, n, 1.0)`; o WGSL diz `read_rest_inv_mass`). Um pin no
/// laço escreve um `inv_mass` que **ninguém lê** — nada fica pinado, e o vento leva tudo.
///
/// ⇒ o pin tem de estar no caminho da ARTE (para o `inv_mass` chegar) e mesmo assim ver
/// a carga (que só existe na cadeia de FORÇAS). As duas coisas ao mesmo tempo pedem uma
/// porta:
///
/// ```text
/// grid ──────────────► pin_constraint ──► integrate.rest      (o inv_mass chega)
/// integrate ═pre═► force.wind ─────────► integrate.forces     (o vento move)
///                   force.wind ═pre═══► pin.load              (a carga chega ao pin)
///                                  pin ═pre═► pin.state       (a memória do rasgo)
/// ```
///
/// ⚠️ **O `pre` na aresta da carga é o que quebra o ciclo** (`pin → integrate → wind →
/// pin`): o pin julga a carga do tique ANTERIOR, que é a única que existe quando ele
/// decide. ⛔ E não há segundo vento: duplicar a força para dar carga ao pin seriam dois
/// números a dizer a mesma coisa, e eles divergem no dia em que alguém afinar um.
///
/// ⚠️ **Porta `load` desligada ⇒ carga ZERO ⇒ nada rasga** — a resposta certa (não há
/// força a puxar) e não um erro silencioso: o nó não pode inventar uma carga que ninguém
/// escreveu.
///
/// ## O rasgo é PERMANENTE, e é isso que exige a porta `state`
///
/// Sem memória, a partícula soltar-se-ia enquanto a rajada dura e voltaria a pinar
/// quando ela passasse — um **cedimento elástico**, não um rasgo. A referência rasga de
/// vez, então o que rasgou fica marcado na coluna [`TORN`] e viaja no `pre` self-loop
/// (o mesmo mecanismo do `motion.step`; o editor plumba-o ao largar o nó).
///
/// ⚠️ **Com a porta `state` desligada o rasgo é por-tique** — a marca não sobrevive ao
/// quadro. Não é um modo escondido: é a consequência de faltar o fio, e o gate
/// `a_torn_pin_stays_torn_only_when_the_state_loop_exists` mede as duas metades.
///
/// ## Porquê `0` desliga
///
/// `0` seria *"rasga a qualquer carga ≥ 0"*, ou seja **nada fica pinado** — e um pin que
/// não pina é a mesma coisa que o nó não existir. O valor mais útil naquele extremo do
/// slider é o oposto, então `0` é o DESLIGADO, que é também a identidade: todo grafo
/// autorado antes deste param lê zero e comporta-se como sempre.
const BREAK_ABOVE: &str = "break_above";

/// A carga sentida pelo elemento `i` — o módulo do `accel` acumulado. Ausente → `0`.
///
/// ⚠️ **O módulo, não o quadrado**: o limiar é uma aceleração e o slider mostra
/// unidades de mundo/s². Comparar contra o quadrado pouparia uma raiz e faria o número
/// do painel deixar de significar o que diz.
fn load_at(s: &Stream, i: usize) -> f32 {
    match s.get(ACCEL) {
        Some(Column::Vec2(v)) => v.get(i).map_or(0.0, |a| (a[0] * a[0] + a[1] * a[1]).sqrt()),
        _ => 0.0,
    }
}

/// A param as a non-negative element index/count: non-finite reads as 0.
fn as_index(v: f32) -> usize {
    if v.is_finite() && v >= 0.0 {
        v.round() as usize
    } else {
        0
    }
}

/// `x` clamped to `[0, 1]`; a non-finite value (a NaN param from a hand-edited
/// document) reads as 0 — no pin — rather than poisoning the weights.
fn clamp01(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The scalar column `name`, widened to `n` elements, `fallback` where absent.
fn scalar_or(s: &Stream, name: &str, n: usize, fallback: f32) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => v.clone(),
        _ => vec![fallback; n],
    }
}

/// The whole node: multiply the pinned run's inverse mass down toward zero.
/// Every other column (P included — the node moves nothing) rides through.
fn pin(
    input: &Stream,
    state: &Stream,
    load: &Stream,
    first: usize,
    count: usize,
    strength: f32,
    break_above: f32,
) -> Stream {
    let n = input.count();
    let falloff = scalar_or(input, FALLOFF, n, 1.0);
    let prev = scalar_or(input, INV_MASS, n, 1.0);
    // O que JÁ estava rasgado no tique anterior. Porta desligada ⇒ tudo inteiro, e o
    // rasgo passa a ser por-tique (ver [`BREAK_ABOVE`]).
    let was_torn = scalar_or(state, TORN, n, 0.0);
    let breaks = break_above > 0.0;
    // **A PRECEDÊNCIA DA CARGA, num sítio só:** a porta `load` quando ela traz `accel`,
    // senão o próprio `in`. Ver [`BREAK_ABOVE`] para os dois idiomas de fiação.
    let carga = if matches!(load.get(ACCEL), Some(Column::Vec2(v)) if !v.is_empty()) {
        load
    } else {
        input
    };
    let mut torn = Vec::with_capacity(n);
    // `first + count` cannot wrap: both are element counts (saturating on the
    // absurd param that a loaded document may carry).
    let last = first.saturating_add(count);
    let w: Vec<f32> = (0..n)
        .map(|i| {
            // ⚠️ Rasgado ANTES ou rasga AGORA — as duas metades, e a primeira é o que
            // torna o rasgo permanente. O `breaks` desligado deixa a coluna a zero,
            // então um grafo sem limiar nunca escreve nada aqui.
            let ripped = breaks && (was_torn[i] >= 0.5 || load_at(carga, i) > break_above);
            torn.push(f32::from(u8::from(ripped)));
            let selected = i >= first && i < last && !ripped;
            // The pin AMOUNT (1 = nailed): the range mask times the field times
            // the strength. Its complement is the inverse mass, multiplied into
            // whatever an upstream pin already wrote (pins compose).
            let amount = if selected {
                clamp01(strength * falloff[i])
            } else {
                0.0
            };
            prev[i] * (1.0 - amount)
        })
        .collect();

    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != INV_MASS && name != TORN {
            out.set(name.clone(), col.clone());
        }
    }
    out.set(INV_MASS, Column::Scalar(w));
    // A memória do rasgo, para o `pre` do tique seguinte. ⚠️ Escrita SEMPRE (a zeros
    // quando não há limiar): uma coluna que aparece e desaparece conforme o param faria
    // o `pre` do tique seguinte ler um stream de forma diferente, e o pareamento por
    // posição é o que este nó tem.
    out.set(TORN, Column::Scalar(torn));
    out
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`pin`], element for element.
///
/// **`params.count_`, not `params.count`.** This node has a param literally named
/// `count`, which is also the element count the engine writes into every uniform
/// block. `codegen::wgsl_field` gives the param its own field
/// (`count` → `count_`); before that, `plan::eligible` REFUSED the kernel outright
/// to avoid the shadowing, and renaming the param was never an option — it is the
/// artist's vocabulary and it lives in saved documents.
///
/// `as_index` is ported faithfully, including its guard: a non-finite or negative
/// param reads as 0, so an absurd number in a loaded document selects nothing
/// rather than wrapping into a huge range. The `first + count` sum saturates for
/// the same reason (`u32` here, `usize` on the CPU — both far past any real count).
///
/// ⚠️ **Os acessores são QUALIFICADOS PELA PORTA (`read_in_*`), e a qualificação nasce da
/// CONTAGEM de portas** (`codegen::accessor_suffix`: um nó de uma entrada usa o nome nu, um
/// de várias qualifica). Este nó tinha UMA porta quando o kernel foi escrito e passou a ter
/// **três** (`state`/`load`, a wave do `break_above`) — e a wave não reescreveu o corpo, que
/// ficou a chamar `read_falloff` num módulo onde ele passou a chamar-se `read_in_falloff`.
/// O `naga` recusou (`no definition in scope`), o `applicable` deste kernel mantinha-o fora
/// do caminho na maioria dos grafos, e o gate que o apanhou foi o
/// `every_registered_kernel_validates_across_the_whole_presence_space`. *Acrescentar uma
/// PORTA a um nó com kernel renomeia todos os acessores dele.*
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let pc_first = pc_index(params.first);\n\
        let pc_last = pc_first + pc_index(params.count_);\n\
        let pc_sel = i >= pc_first && i < pc_last;\n\
        let pc_amount = select(\n\
        \x20   0.0,\n\
        \x20   clamp(params.strength * read_in_falloff(i), 0.0, 1.0),\n\
        \x20   pc_sel);\n\
        write_inv_mass(i, read_in_inv_mass(i) * (1.0 - pc_amount));\n",
    wgsl_lib: "\
        fn pc_index(v: f32) -> u32 {\n\
            // Rust `as_index`: non-finite or negative reads as 0, else round\n\
            // half-away-from-zero (WGSL `round` is half-even).\n\
            if (!(v >= 0.0)) { return 0u; }\n\
            if (!(v < 3.4028235e38)) { return 0u; }\n\
            return u32(floor(v + 0.5));\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: INV_MASS,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: FALLOFF,
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &["first", "count", "strength"],
    count_law: None,
    variant_by_param: None,
    applicable: Some(|p| p(BREAK_ABOVE) <= 0.0),
};

struct MotionPinConstraint;

impl NodeOp for MotionPinConstraint {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let first = as_index(ctx.param("first"));
        let count = as_index(ctx.param("count"));
        let strength = ctx.param("strength");
        let break_above = ctx.param(BREAK_ABOVE).max(0.0);
        // ⚠️ O estado é clonado ANTES do input 0: os dois `ctx.input` não podem
        // coexistir emprestados. Uma coluna escalar, e só no tique em que há limiar.
        let state = if break_above > 0.0 {
            match ctx.input(1).get(TORN) {
                Some(Column::Scalar(v)) => {
                    Stream::new(v.len()).with(TORN, Column::Scalar(v.clone()))
                }
                _ => Stream::new(0),
            }
        } else {
            Stream::new(0)
        };
        // A carga, clonada pela mesma razão que o estado: dois `ctx.input` não podem
        // coexistir emprestados, e ela só é tocada quando há limiar.
        let load = if break_above > 0.0 {
            match ctx.input(2).get(ACCEL) {
                Some(Column::Vec2(v)) => Stream::new(v.len()).with(ACCEL, Column::Vec2(v.clone())),
                _ => Stream::new(0),
            }
        } else {
            Stream::new(0)
        };
        let out = pin(
            ctx.input(0),
            &state,
            &load,
            first,
            count,
            strength,
            break_above,
        );
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionPinConstraint))?;
    // ADR-0155: a pin PRODUCES `inv_mass` (pins by zeroing it); inert without a
    // solver (integrate/sim.step/spring/collide) downstream to read it.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Produces("inv_mass")],
    );
    // ⚠️ **Mais a RECUSA quando o limiar está ligado**: o rasgo é ESTADO no `pre`, e o
    // kernel deste nó é um mapa por-elemento sem memória. Reimplementá-lo ali seria a
    // segunda cópia de uma lei — a mesma porta que o `motion.combine`/`motion.cull`
    // usam. Desligado (o default) nada recua.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Pin Constraint",
            // It authors a weight FIELD the sims read — the falloff family's
            // category, not a transform (it moves nothing itself).
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `first`/`count` — MEDIDO, e o achado é que este eixo NÃO tem custo** (doc 88
/// A1 · §0), enquanto os sliders ficam nos 4.096 que cobrem a autoria confortável.
///
/// Os dois params são ÍNDICES dentro do stream de entrada — eles escolhem QUAIS linhas ficam
/// pinadas, não quantas existem —, então o custo do nó é o da entrada e é **plano no eixo medido**.
/// Medido pela porta do produto (`measure_the_count_ceiling`, fonte de 200.000 instâncias):
///
/// | pins pedidos | cook |
/// |---|---|
/// | 4.096 | 0,534 ms |
/// | 40.000 | 0,490 ms |
/// | 200.000 | 0,516 ms |
///
/// Sem inclinação não há teto de recurso a derivar: o limite honesto é o comprimento do stream,
/// que o `eval` já respeita ao fatiar. O hard max acompanha os geradores lineares — quem pina uma
/// grade de um milhão precisa poder digitar o índice de um milhão.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "first",
        max: 1_000_000.0,
    },
    ParamHardMax {
        param: "count",
        max: 1_000_000.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "first",
        label: "First",
        min: 0.0,
        max: 4096.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 0.0,
        // ⚠️ **Um IntSlider cujo curso passa da largura do track não consegue
        // selecionar todo inteiro.** O track mede ~154 px, então os 4.096 de
        // antes moviam **27 pins por pixel**: pinar 2, 3 ou 5 era inexprimível
        // com a mão, num nó cujo default é **1**. O teto de recurso não mudou —
        // ele está no `PARAM_HARD_MAX` acima (um milhão, medido plano), e é lá
        // que quem pina uma grade inteira digita.
        max: 128.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    }, // ⚠️ **`0` é DESLIGADO, e o rótulo diz isso** — ver [`BREAK_ABOVE`]. O topo da
    // faixa é a aceleração que uma `force.*` típica entrega (o default do
    // `force.attractor` é 5, o do `force.wind` da mesma ordem): acima disso o slider
    // seria curso morto, porque nenhuma carga do catálogo lá chega.
    ParamUiHint {
        param: "break_above",
        label: "Break Above (0 = never)",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
