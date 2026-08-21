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
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let pc_first = pc_index(params.first);\n\
        let pc_last = pc_first + pc_index(params.count_);\n\
        let pc_sel = i >= pc_first && i < pc_last;\n\
        let pc_amount = select(\n\
        \x20   0.0,\n\
        \x20   clamp(params.strength * read_falloff(i), 0.0, 1.0),\n\
        \x20   pc_sel);\n\
        write_inv_mass(i, read_inv_mass(i) * (1.0 - pc_amount));\n",
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
mod tests {
    use super::*;

    /// A stream of `n` elements at the origin, with the given optional fields.
    fn stream(n: usize, falloff: Option<Vec<f32>>, inv_mass: Option<Vec<f32>>) -> Stream {
        let mut s = Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n]));
        if let Some(f) = falloff {
            s.set(FALLOFF, Column::Scalar(f));
        }
        if let Some(w) = inv_mass {
            s.set(INV_MASS, Column::Scalar(w));
        }
        s
    }

    fn weights(s: &Stream) -> Vec<f32> {
        match s.get(INV_MASS) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("no inv_mass column"),
        }
    }

    /// The range is what gets pinned: inside it the inverse mass is 0 (infinite
    /// mass), outside it stays 1 (free). FALSIFIED if the node pinned the whole
    /// stream (the bug that would freeze every sim downstream).
    #[test]
    fn the_index_range_is_what_gets_pinned() {
        let out = pin(
            &stream(5, None, None),
            &Stream::new(0),
            &Stream::new(0),
            1,
            2,
            1.0,
            0.0,
        );
        assert_eq!(weights(&out), vec![1.0, 0.0, 0.0, 1.0, 1.0]);
    }

    /// `strength` is a PARTIAL pin (Blender's pin weight): half strength leaves
    /// half the inverse mass, i.e. an element twice as heavy as its neighbours.
    #[test]
    fn strength_is_a_partial_pin() {
        let out = pin(
            &stream(2, None, None),
            &Stream::new(0),
            &Stream::new(0),
            0,
            1,
            0.25,
            0.0,
        );
        assert_eq!(weights(&out), vec![0.75, 1.0]);
    }

    /// The `falloff` field scales the pin, so an upstream falloff pins a REGION:
    /// full field = nailed, half = heavy, zero = untouched.
    #[test]
    fn the_falloff_field_scales_the_pin() {
        let out = pin(
            &stream(3, Some(vec![1.0, 0.5, 0.0]), None),
            &Stream::new(0),
            &Stream::new(0),
            0,
            3,
            1.0,
            0.0,
        );
        assert_eq!(weights(&out), vec![0.0, 0.5, 1.0]);
    }

    /// Two pins COMPOSE (multiply) instead of the second erasing the first —
    /// the falloff family's rule. Two half-pins on the same element leave a
    /// quarter of the inverse mass.
    #[test]
    fn pins_compose_multiplicatively() {
        let once = pin(
            &stream(1, None, None),
            &Stream::new(0),
            &Stream::new(0),
            0,
            1,
            0.5,
            0.0,
        );
        let twice = pin(&once, &Stream::new(0), &Stream::new(0), 0, 1, 0.5, 0.0);
        assert_eq!(weights(&twice), vec![0.25]);
    }

    /// `count = 0` (or a zero strength) selects nothing: every element stays
    /// free, and an upstream weight rides through untouched.
    #[test]
    fn an_empty_selection_is_the_identity() {
        assert_eq!(
            weights(&pin(
                &stream(3, None, None),
                &Stream::new(0),
                &Stream::new(0),
                0,
                0,
                1.0,
                0.0
            )),
            vec![1.0; 3]
        );
        assert_eq!(
            weights(&pin(
                &stream(3, None, None),
                &Stream::new(0),
                &Stream::new(0),
                0,
                3,
                0.0,
                0.0
            )),
            vec![1.0; 3]
        );
        let carried = stream(2, None, Some(vec![0.0, 0.5]));
        assert_eq!(
            weights(&pin(
                &carried,
                &Stream::new(0),
                &Stream::new(0),
                0,
                0,
                1.0,
                0.0
            )),
            vec![0.0, 0.5]
        );
    }

    /// A non-finite param never poisons the weights (a hand-edited document can
    /// carry any `f32`): the element stays free rather than going NaN.
    #[test]
    fn a_non_finite_strength_reads_as_free() {
        let out = pin(
            &stream(1, None, None),
            &Stream::new(0),
            &Stream::new(0),
            0,
            1,
            f32::NAN,
            0.0,
        );
        assert_eq!(weights(&out), vec![1.0]);
    }

    /// Cooks through the registry: the weights land on the stream and every
    /// other column (the positions the node must NOT touch) passes through.
    #[test]
    fn registers_and_cooks_the_weight_column() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.pin.test.src"),
            name: "motion.pin.test.src",
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
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionPinConstraint),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.pin.test.src");
        let p = g.add_node("motion.pin_constraint");
        g.set_param(p, "count", 2.0);
        g.connect(Edge {
            from: (src, 0),
            to: (p, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, p, 0.0).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 3, "count preserved");
        assert_eq!(weights(s), vec![0.0, 0.0, 1.0], "the first two are pinned");
        match s.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[1], [1.0, 0.0], "positions ride through"),
            _ => panic!("P"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Doc 89 folha 03 — o LIMIAR DE RUPTURA. Ver [`BREAK_ABOVE`] para o mecanismo.
    // ─────────────────────────────────────────────────────────────────────────

    /// Um stream de `n` elementos com a carga (`accel`) escrita à mão.
    fn loaded(n: usize, accel: &[[f32; 2]]) -> Stream {
        Stream::new(n)
            .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
            .with(ACCEL, Column::Vec2(accel.to_vec()))
    }

    fn inv_mass_of(s: &Stream) -> Vec<f32> {
        match s.get(INV_MASS) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("inv_mass"),
        }
    }

    /// **`0` NÃO RASGA NADA — e é a identidade, com carga ou sem ela.**
    ///
    /// ⚠️ O braço COM carga é o que importa: um nó que comparasse `carga > 0` soltaria
    /// todo pin de toda cena que tenha uma força, em silêncio.
    #[test]
    fn a_zero_threshold_never_tears_even_under_load() {
        let sem = pin(
            &loaded(2, &[[0.0, 0.0]; 2]),
            &Stream::new(0),
            &loaded(2, &[[0.0, 0.0]; 2]),
            0,
            2,
            1.0,
            0.0,
        );
        let com = pin(
            &loaded(2, &[[99.0, 0.0]; 2]),
            &Stream::new(0),
            &loaded(2, &[[99.0, 0.0]; 2]),
            0,
            2,
            1.0,
            0.0,
        );
        assert_eq!(inv_mass_of(&sem), vec![0.0, 0.0], "pinado");
        assert_eq!(
            inv_mass_of(&com),
            vec![0.0, 0.0],
            "e continua pinado sob carga"
        );
    }

    /// **ACIMA DO LIMIAR O PIN SOLTA; ABAIXO, SEGURA.**
    ///
    /// Carga `3` e `6` contra um limiar de `5`: a primeira segura, a segunda rasga.
    /// FALSIFICADO por uma comparação invertida, ou por um limiar que agisse sobre o
    /// stream todo em vez de por elemento.
    #[test]
    fn the_pin_tears_only_where_the_load_exceeds_the_threshold() {
        let out = pin(
            &loaded(2, &[[3.0, 0.0], [6.0, 0.0]]),
            &Stream::new(0),
            &loaded(2, &[[3.0, 0.0], [6.0, 0.0]]),
            0,
            2,
            1.0,
            5.0,
        );
        assert_eq!(inv_mass_of(&out), vec![0.0, 1.0], "só o segundo rasgou");
    }

    /// **A CARGA É O MÓDULO, não uma componente** — uma força só em Y rasga tanto como
    /// uma só em X.
    #[test]
    fn the_load_is_the_magnitude_of_the_accumulated_force() {
        let out = pin(
            &loaded(3, &[[6.0, 0.0], [0.0, 6.0], [3.0, 4.0]]),
            &Stream::new(0),
            &loaded(3, &[[6.0, 0.0], [0.0, 6.0], [3.0, 4.0]]),
            0,
            3,
            1.0,
            4.9,
        );
        // 6, 6 e 5 — os três passam de 4,9.
        assert_eq!(inv_mass_of(&out), vec![1.0, 1.0, 1.0]);
    }

    /// **SEM `accel` NO STREAM A CARGA É ZERO E NADA RASGA** — a resposta certa (não há
    /// força a puxar), não um erro silencioso.
    ///
    /// ⚠️ É também o que documenta a ORDEM de fiação: um pin posto ANTES das forças não
    /// vê carga nenhuma, porque a coluna ainda não existe.
    #[test]
    fn a_stream_without_accel_carries_no_load() {
        let out = pin(
            &stream(2, None, None),
            &Stream::new(0),
            &Stream::new(0),
            0,
            2,
            1.0,
            0.01,
        );
        assert_eq!(inv_mass_of(&out), vec![0.0, 0.0], "sem carga, nada rasga");
    }

    /// **O RASGO É PERMANENTE — mas SÓ com o laço de estado, e o gate mede as duas
    /// metades.**
    ///
    /// ⚠️ Sem o `pre` a marca não sobrevive ao quadro e o pin volta a segurar quando a
    /// rajada passa: um **cedimento elástico**, não um rasgo. Não é um modo escondido; é
    /// a consequência de faltar o fio, e está escrito no doc do param.
    #[test]
    fn a_torn_pin_stays_torn_only_when_the_state_loop_exists() {
        let rajada = loaded(1, &[[9.0, 0.0]]);
        let calmo = loaded(1, &[[0.0, 0.0]]);
        // Tique 1: rasga.
        let t1 = pin(&rajada, &Stream::new(0), &rajada, 0, 1, 1.0, 5.0);
        assert_eq!(inv_mass_of(&t1), vec![1.0], "rasgou");
        // Tique 2 COM o laço: a marca chega, e ele continua solto mesmo sem carga.
        let com = pin(&calmo, &t1, &calmo, 0, 1, 1.0, 5.0);
        assert_eq!(inv_mass_of(&com), vec![1.0], "rasgado é rasgado");
        // Tique 2 SEM o laço: ele volta a pinar — o cedimento elástico.
        let sem = pin(&calmo, &Stream::new(0), &calmo, 0, 1, 1.0, 5.0);
        assert_eq!(inv_mass_of(&sem), vec![0.0], "sem memória, ele re-pina");
        assert_ne!(
            inv_mass_of(&com),
            inv_mass_of(&sem),
            "e as duas leis diferem"
        );
    }

    /// **A MEMÓRIA SAI SEMPRE, mesmo a zeros** — uma coluna que aparece e desaparece
    /// conforme o param faria o `pre` do tique seguinte ler um stream de outra forma.
    #[test]
    fn the_tear_memory_is_always_written() {
        for limiar in [0.0_f32, 5.0] {
            let out = pin(
                &loaded(2, &[[0.0, 0.0]; 2]),
                &Stream::new(0),
                &loaded(2, &[[0.0, 0.0]; 2]),
                0,
                2,
                1.0,
                limiar,
            );
            match out.get(TORN) {
                Some(Column::Scalar(v)) => assert_eq!(v.len(), 2, "limiar {limiar}"),
                _ => panic!("a coluna do rasgo tem de sair sempre (limiar {limiar})"),
            }
        }
    }

    /// **O LIMIAR LIGADO RECUSA O DEVICE, E O DESLIGADO NÃO.**
    #[test]
    fn the_tear_refuses_the_device_and_the_default_does_not() {
        let f = GPU_KERNEL.applicable.expect("o kernel declara a recusa");
        assert!(f(&|_: &str| 0.0), "sem limiar: o device continua a valer");
        assert!(
            !f(&|n: &str| if n == BREAK_ABOVE { 5.0 } else { 0.0 }),
            "com limiar: o rasgo é estado, e o kernel é um mapa sem memória"
        );
    }

    /// **A CARGA VEM DA PORTA `load`; SEM ELA, DO PRÓPRIO `in`.**
    ///
    /// ⚠️ Os dois idiomas: com o `motion.integrate` o pin tem de estar no caminho da
    /// arte (é de lá que o `inv_mass` é lido) e a carga chega-lhe pela porta; com um
    /// GERADOR (`motion.soft_body`) ele cabe dentro da cadeia de estado e o `in` já a
    /// traz. Uma precedência, não duas fontes.
    #[test]
    fn the_load_falls_back_to_the_nodes_own_input() {
        let carga = loaded(1, &[[9.0, 0.0]]);
        let limpo = stream(1, None, None);
        // Pela PORTA: o `in` não tem carga nenhuma.
        let pela_porta = pin(&limpo, &Stream::new(0), &carga, 0, 1, 1.0, 5.0);
        assert_eq!(inv_mass_of(&pela_porta), vec![1.0], "a porta manda");
        // Pelo `in`: a porta está vazia.
        let pelo_in = pin(&carga, &Stream::new(0), &Stream::new(0), 0, 1, 1.0, 5.0);
        assert_eq!(inv_mass_of(&pelo_in), vec![1.0], "o recuo funciona");
        // E a PORTA VENCE o `in` quando as DUAS trazem carga — uma precedência, não uma
        // soma. ⚠️ A precedência é sobre a COLUNA e não sobre o fio: uma porta ligada a
        // um stream SEM `accel` não é fonte de carga nenhuma, e é indistinguível de uma
        // porta desligada — que é o que o braço `limpo` acima já prova.
        let leve = loaded(1, &[[1.0, 0.0]]);
        let manda = pin(&carga, &Stream::new(0), &leve, 0, 1, 1.0, 5.0);
        assert_eq!(
            inv_mass_of(&manda),
            vec![0.0],
            "a carga da PORTA (1) está abaixo do limiar; a do `in` (9) não pode falar"
        );
    }
}
