#![forbid(unsafe_code)]
//! `motion.velocity` — **para onde este elemento vai** (doc 89, folha 07 — o *compute velocity*
//! do MOPs, [houdini_mops §65](../../../docs/Motion%20Nodes/89_conferencia/07_tempo_estilisticos.md)).
//!
//! A coluna `vel` é a convenção deste repo para velocidade em **unidades por SEGUNDO**
//! (`motion.integrate` faz `d += v·dt`, e `force.drag` / `force.buoyancy` / `motion.boids` a
//! leem). Só que **quem a escrevia eram os simuladores** — um `motion.grid` movido por um
//! `value.lfo`, um `motion.path`, um objeto animado pela timeline não têm `vel` nenhuma.
//!
//! ## O que estava faltando não era a leitura, era o PRODUTOR
//!
//! Medido antes de construir: o consumo já está todo aqui, e em mais lugares do que parece.
//!
//! | quem já lê `vel` | o que faz com ela |
//! |---|---|
//! | `value.attribute` modo **Speed** | a MAGNITUDE — *"colora pela velocidade"* |
//! | `value.attribute` modo **Direction** | o ÂNGULO em graus — `→ motion.drive(Rotation)` **é** o *align to velocity* |
//! | `value.attribute` modo **lane** | `vel.x` / `vel.y` soltos |
//! | `force.drag` · `force.buoyancy` | resistem ao que se move |
//! | `motion.integrate` | o estado dela própria |
//!
//! O canal `Speed` é **oferecido no picker** do `value.attribute`, e num stream que nenhum
//! simulador tocou ele devolve **zeros** — o miss ordinário daquele nó, indistinguível de um
//! nome mal digitado. Este nó é o que o preenche.
//!
//! ⚠️ **E ele fecha um item de OUTRA família:** a folha 04 (deformers) nomeia o *Squash and
//! Stretch / Motion Stretch* como *"clássico de animação!"* e o deixa **fora de escopo** com o
//! motivo exato — *"é um deformer dirigido pela VELOCIDADE, e nenhum dos sete lê a coluna de
//! velocidade"*. Com um produtor de `vel` no catálogo, a cadeia passa a existir.
//!
//! ## Por que um nó PRÓPRIO, e não o `motion.trail` a escrever `vel`
//!
//! A célula da folha 07 diz *"o rastro tem `P` de dois ticks na mão"*, e tem — mas as linhas
//! que ele emite são **ECOS**, e um eco não tem velocidade própria: ele é onde a cabeça esteve.
//! Pôr o serviço lá também faria *"quero a velocidade"* exigir *"quero um rastro"*. O MOPs
//! nomeia a ausência do mesmo jeito: *"o **compute velocity como serviço** FALTA"*.
//!
//! ## A lei
//!
//! `vel = (P − P_anterior) / dt`, pareado por **id** quando o stream tem identidade e
//! posicionalmente quando não tem — a mesma regra do `motion.integrate`, e pelo mesmo motivo:
//! num emitter as linhas nascem e morrem, então a linha `i` de hoje não é a linha `i` de ontem.
//!
//! ⚠️ **Quem não tem passado tem velocidade ZERO**, nunca uma inventada. Uma partícula recém-
//! nascida, o primeiro tick de um cook e o tick seguinte a um reset caem todos aí (`dt` é `0.0`
//! no primeiro tick de um cook, por contrato do `EvalCtx`) — e reportar qualquer outra coisa
//! seria um deformer de motion-stretch a esticar um elemento que ainda não se moveu.
//!
//! ## O `smooth`
//!
//! Uma diferença finita é **RUIDOSA por natureza**: ela amplifica todo tremor do que alimenta a
//! posição, e um motion-stretch dirigido por ela treme junto. O `smooth` é um one-pole sobre a
//! velocidade — `v += (crua − v)/n`, a MESMA lei do `Blend` do `motion.delay` — e **`0` é a
//! diferença crua, byte a byte**.
//!
//! ⚠️ Ele filtra a VELOCIDADE, não a posição: filtrar `P` mudaria onde o elemento é desenhado,
//! que é trabalho do `motion.delay` e não deste nó. Aqui a arte não se move um pixel.
//!
//! ⚠️ **Sem `ParamHardMax`, e é medição e não esquecimento:** um one-pole com constante enorme
//! é *lento*, nunca quebrado — ele continua convergindo, e não há ponto em que o número deixe de
//! significar alguma coisa. É a mesma ausência que o `radius` do `motion.boids` carrega, pelo
//! mesmo motivo, e há gate a afirmá-la para a próxima varredura não "completar" a tabela.
//!
//! ## O que ele NÃO lê, e porquê
//!
//! ⚠️ **Não lê `falloff`.** A lei da família (*todo modificador é multiplicável por um campo de
//! peso*) vale para quem MODIFICA; este nó **MEDE**. Uma medição mascarada é uma medição que
//! mente — metade dos elementos reportaria uma velocidade que eles não têm, e o
//! `value.attribute(Speed)` a jusante não teria como saber. Quem quiser um campo sobre o
//! resultado põe um `value.attribute → value.math(Multiply)` depois, onde ele é visível.
//!
//! ⚠️ **E não declara `Coupling::Produces("vel")`**, embora escreva a coluna. O ADR-0155 marca
//! um produtor cuja coluna nada consome a jusante — e o consumidor canônico desta é o
//! `value.attribute`, que lê a coluna que o **ARTISTA nomeou num text param** e por isso não
//! declara `Consumes` nenhum (uma declaração estática seria falsa em sete dos oito canais dele).
//! Declarar a produção aqui marcaria como INERTE exatamente a cadeia que este nó existe para
//! servir: *a afirmação seria o próprio bug*, que é a lição que o `MissingSource("P")` do Boids
//! já pagou uma vez.
//!
//! Sequential (o `pre` self-loop que o editor plumba ao soltar), `Effect::Pure` (o tick entra no
//! fingerprint pela aresta `pre` consumida — o precedente do `motion.strobe`, que também lê
//! `ctx.dt()`), HR-5: só aritmética.

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// A coluna que este nó escreve — a convenção do repo, lida por `force.drag`,
/// `force.buoyancy`, `motion.integrate`, `motion.boids` e pelos três modos vetoriais do
/// `value.attribute`. **O nome é um contrato de STREAM**, como `P` e `texture_id`.
const VEL: &str = "vel";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.velocity"),
    name: "motion.velocity",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // A porta de realimentação — `out --pre--> state`, plumbada pelo editor ao soltar (a
        // convenção do nó sequencial: uma entrada chamada `state` com o tipo da saída).
        PortSpec {
            name: "state",
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
        // O one-pole sobre a velocidade, em ticks. **`0` é a diferença CRUA, byte a byte.**
        ParamSpec {
            name: "smooth",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Uma coluna `Vec2` lida até `n` elementos (ausente / do tipo errado / curta → `identity`).
/// Espelhada por-crate, como o `falloff_at` das behaviours (drop-crate: ADR-0075).
fn vec2_to_n(s: &Stream, name: &str, n: usize, identity: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}

/// As **chaves de identidade** de um stream: a coluna `id` quando existe (o emitter a carimba,
/// e ela sobrevive a nascimentos e mortes), senão `None` — e aí a identidade é POSICIONAL, o
/// que significa que uma mudança de contagem é um conjunto reconstruído, não elementos que
/// morreram. Espelho exato do `ids_of` do `motion.integrate`.
fn ids_of(s: &Stream, n: usize) -> Option<Vec<u32>> {
    match s.get("id") {
        Some(Column::Scalar(v)) => {
            let mut out: Vec<u32> = v.iter().map(|f| f.max(0.0) as u32).collect();
            out.resize(n, 0);
            Some(out)
        }
        _ => None,
    }
}

/// **Onde a linha `i` de hoje estava ontem** — o índice dela no estado, ou `None` se ela não
/// tinha ontem.
///
/// Por id quando os DOIS streams têm identidade (um gather; um id que o estado não conhece é
/// um nascimento), posicional quando algum não tem.
fn pairing(live_ids: Option<&[u32]>, state_ids: Option<&[u32]>, sn: usize) -> Pairing {
    match (live_ids, state_ids) {
        (Some(_), Some(s)) => {
            let mut map = std::collections::BTreeMap::new();
            for (j, &id) in s.iter().enumerate() {
                map.entry(id).or_insert(j);
            }
            Pairing::ById(map)
        }
        _ => Pairing::Positional(sn),
    }
}

enum Pairing {
    ById(std::collections::BTreeMap<u32, usize>),
    Positional(usize),
}

impl Pairing {
    fn prior(&self, i: usize, live_ids: Option<&[u32]>) -> Option<usize> {
        match self {
            Pairing::ById(map) => live_ids.and_then(|ids| map.get(&ids[i]).copied()),
            Pairing::Positional(sn) => (i < *sn).then_some(i),
        }
    }
}

/// **A lei, por elemento.** Devolve a velocidade em unidades por segundo.
///
/// ⚠️ `dt <= 0` e *sem passado* dão o MESMO resultado — zero —, e é de propósito: quem não tem
/// dois instantes não tem velocidade, e inventar uma seria pior que reportar que não há.
fn velocity_of(
    here: [f32; 2],
    prior: Option<([f32; 2], [f32; 2])>,
    dt: f32,
    smooth: f32,
) -> [f32; 2] {
    let Some((there, prev_vel)) = prior else {
        return [0.0, 0.0];
    };
    // ⚠️ `is_finite` e não só `> 0.0`: um `dt` NaN atravessaria uma comparação simples e
    // envenenaria a coluna inteira — e a `vel` alimenta o `motion.integrate`, onde um NaN vira
    // uma posição NaN e a peça DESAPARECE da tela sem erro nenhum.
    if dt <= 0.0 || !dt.is_finite() {
        return [0.0, 0.0];
    }
    let raw = [(here[0] - there[0]) / dt, (here[1] - there[1]) / dt];
    if smooth <= 0.0 {
        return raw;
    }
    // O one-pole do `Blend` do `motion.delay`: `v += (crua − v)/n`, com o mesmo piso de 1 que
    // torna `smooth = 1` a resposta instantânea em vez de uma divisão por zero.
    let n = smooth.max(1.0);
    [
        prev_vel[0] + (raw[0] - prev_vel[0]) / n,
        prev_vel[1] + (raw[1] - prev_vel[1]) / n,
    ]
}

/// O passo: o stream de entrada, mais a coluna `vel`.
fn step(input: &Stream, state: &Stream, dt: f32, smooth: f32) -> Stream {
    let n = input.count();
    let sn = state.count();
    let here = vec2_to_n(input, "P", n, [0.0, 0.0]);
    let there = vec2_to_n(state, "P", sn, [0.0, 0.0]);
    let prev_vel = vec2_to_n(state, VEL, sn, [0.0, 0.0]);

    let live_ids = ids_of(input, n);
    let state_ids = ids_of(state, sn);
    let pairing = pairing(live_ids.as_deref(), state_ids.as_deref(), sn);

    let mut vel = Vec::with_capacity(n);
    for (i, &p) in here.iter().enumerate() {
        let prior = pairing
            .prior(i, live_ids.as_deref())
            .map(|j| (there[j], prev_vel[j]));
        vel.push(velocity_of(p, prior, dt, smooth));
    }

    // ⚠️ O stream de ENTRADA passa inteiro adiante e a `vel` é escrita por cima: um
    // `motion.velocity` a jusante de um integrador reporta o deslocamento que de facto
    // aconteceu (depois de um `motion.collide` clampar posições, por exemplo), que é o número
    // que um motion-stretch quer — e não a velocidade que o solver *pretendia*.
    let mut out = input.clone();
    out.set(VEL, Column::Vec2(vel));
    out
}

struct MotionVelocity;

impl NodeOp for MotionVelocity {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let smooth = ctx.param("smooth");
        let dt = ctx.dt() as f32;
        let out = {
            let input = ctx.input(0);
            let state = ctx.input(1);
            step(input, state, dt, smooth)
        };
        ctx.emit(out);
    }
}

/// Registra o tipo no registry (fan-out: DIRETRIZ §3.A).
///
/// # Errors
/// Devolve [`RegistryError`] se um tipo com este id já estiver registrado.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionVelocity))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Velocity",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "smooth",
    label: "Smooth",
    min: 0.0,
    max: 32.0,
    step: 0.5,
    widget: ParamWidget::Slider,
}];

/// A unidade: uma contagem de ticks, como o `ticks` do `motion.delay`.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "smooth",
    unit: ParamUnit::Count,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
