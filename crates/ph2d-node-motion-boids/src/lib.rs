#![forbid(unsafe_code)]
//! `motion.boids` — **emergent flocking**: `count` agents that steer by the three
//! classic local rules and coalesce into a living flock (Motion Nodes M4,
//! simulation — doc 01 §3 / doc 21). No path is authored; the swirling, splitting,
//! re-forming shoal is what *emerges* from every agent obeying its neighbours. The
//! counterpart to the deterministic `motion.verlet_rope`: same sequential substrate
//! (`pre` state), opposite character — self-organising rather than constrained.
//!
//! **Algorithm — Reynolds' boids (SIGGRAPH 1987), the gold standard.** Each agent
//! looks only at flock-mates within a perception `radius` and sums three steering
//! urges:
//! - **Separation** — steer away from crowding neighbours (inverse-square repulsion,
//!   so it bites only when they get close).
//! - **Alignment** — steer toward the average heading of neighbours.
//!
//! ## ⛔ E os três pesos NÃO viram três nós (doc 89 folha 03)
//!
//! A célula pede *"1 regra = 1 NÓ empilhável com peso próprio"* — a família POP Steer do
//! Houdini, em que separação, alinhamento e coesão são nós distintos que se empilham.
//!
//! ⛔ **RECUSADO POR NATUREZA, e o bloqueio é uma PRIMITIVA que não existe:** as três regras
//! leem uma **VIZINHANÇA** (quem está dentro do `radius`), e o catálogo não a expõe. As
//! reduções que existem — `value.reduce`, `median`, `percentile` — reduzem o stream INTEIRO,
//! não uma vizinhança; decompor exigiria construir primeiro *"o que está perto de mim"* como
//! canal, que é uma wave inteira e não uma refactoração deste nó.
//!
//! ⚠️ **E o que a decomposição compraria já está comprado:** os três pesos são params
//! **dirigíveis** (doc 58), então misturar as regras animadamente — que é o ganho prático de
//! as ter separadas — faz-se hoje com fios. O que ficaria por comprar é a EXTENSIBILIDADE (uma
//! quarta regra escrita por fora), e essa é a promessa de um substrato, não de um nó.
//! - **Cohesion** — steer toward the average position (centroid) of neighbours.
//!
//! Plus a **seek** pull toward a target point (the value inputs; unconnected → the
//! origin). Seek is a linear spring, so it doubles as the flock's tether: it grows
//! with distance and keeps the swarm bounded around its home (no fly-away, no
//! separate boundary hack). A `value.lfo` on the target herds the whole flock.
//!
//! Neighbour search is the honest `O(N²)` all-pairs scan — exact and simple for the
//! bounded counts a node produces. A spatial hash (the standard scale trick) is a
//! pure-perf follow-up (doc 21 §5); it would not change a single emitted position.
//!
//! ## Topology (the `pre` self-loop)
//!
//! Sequential like `motion.spring`/`motion.integrate`/`motion.verlet_rope`: each
//! agent's `P` and `vel` ride the `state` feedback port, auto-plumbed as the `pre`
//! self-loop on add (`out --pre--> state`). Tick 0 (`pre` = Empty) **seeds** a
//! hashed cloud of positions + velocities (stateless, reproducible — Jarzynski/
//! Olano); from tick 1 it steps. `dt` derives from the state's own `sim_t`, clamped
//! to `[0, MAX_DT]` (a loop-wrap freezes one tick, never explodes) — like
//! `motion.integrate`.
//!
//! Transcendental-free (HR-5): steering is add/multiply and one IEEE `sqrt` per
//! normalise (heading / speed clamp); the seed is the splitmix hash. Deterministic
//! → `Effect::Temporal`, replays bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod gpu;
mod hash;
use hash::hash3;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `target_*` inputs (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

mod ceiling;
use ceiling::MAX_DT;
/// Half-extent of the seed cloud (world units), centred on the target/home.
const SEED_SPREAD: f32 = 3.0;
/// The count at which the √N spread (`spread` on) reproduces the fixed
/// [`SEED_SPREAD`] — chosen so the density (agents per perception cell) it holds
/// is a lively murmuration (~22 neighbours per agent at the default radius). It
/// is the ONLY thing the reference sets: at `spread` off it is unused, and at any
/// count it is the density, never the size.
const SEED_DENSITY_REF: f32 = 64.0;
/// A boid never drifts below this fraction of `max_speed` — keeps the flock
/// gliding instead of stalling to a dead point.
const MIN_SPEED_FRAC: f32 = 0.2;
/// Below this a vector length is treated as zero (skip the normalise).
const EPS: f32 = 1e-6;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.boids"),
    name: "motion.boids",
    inputs: &[
        // The seek target / home, as two value fields (animatable). Optional:
        // unconnected reads as 0 → the origin.
        PortSpec {
            name: "target_x",
            ty: VALUE,
        },
        PortSpec {
            name: "target_y",
            ty: VALUE,
        },
        // The feedback port — auto-wired `out --pre--> state` on add.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
        // **OS OBSTÁCULOS** — pontos a evitar (modo ligado pelo `avoid`). APENDADO: os
        // três índices anteriores não se mexem. Ver [`AVOID`].
        PortSpec {
            name: "obstacle",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "count",
            default: 48.0,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
        // Perception radius: only neighbours within it steer an agent.
        ParamSpec {
            name: "radius",
            default: 2.0,
        },
        ParamSpec {
            name: "separation",
            default: 1.6,
        },
        // O `desiredseparation` de Reynolds — a SEGUNDA distância da vizinhança,
        // que o modelo publicado tem e este nó não tinha. `0` é DESLIGADO (a
        // repulsão inverse-square de sempre, sobre o raio de percepção), e é isso
        // que mantém todo bando já autorado byte-idêntico.
        ParamSpec {
            name: "separation_radius",
            default: 0.0,
        },
        ParamSpec {
            name: "alignment",
            default: 1.0,
        },
        ParamSpec {
            name: "cohesion",
            default: 0.9,
        },
        // Pull toward the target/home (a linear spring → also the tether/bound).
        ParamSpec {
            name: "seek",
            default: 1.0,
        },
        ParamSpec {
            name: "max_speed",
            default: 4.0,
        },
        // Reynolds' OTHER clamp — the steering budget (**0 = off**, the default,
        // so every flock authored before this param is byte-identical).
        ParamSpec {
            name: "max_force",
            default: 0.0,
        },
        // √N seed spread (0 = fixed, the default; 1 = density-bounded). Off is
        // byte-identical at EVERY count — it only decides whether a bigger flock
        // is a denser ball or a wider murmuration (the latter keeps the spatial
        // grid `O(N)`, which is how the count reaches the millions the GPU can do).
        // ⚠️ **360 é o disco de hoje**, e o ramo do `cos_half_fov` garante que ele
        // não custa nada — ver [`Params::cos_half_fov`].
        ParamSpec {
            name: "fov",
            default: 360.0,
        },
        ParamSpec {
            name: "speed_floor",
            default: MIN_SPEED_FRAC,
        },
        // **DESVIAR DE OBSTÁCULOS** (doc 89 folha 03 — o POP Steer Avoid Obstacle).
        // APENDADOS; `avoid = 0` desliga os três e o bando é o de sempre, ao bit.
        // Ver [`AVOID`].
        ParamSpec {
            name: "avoid",
            default: 0.0,
        },
        ParamSpec {
            name: "avoid_radius",
            default: 1.0,
        },
        ParamSpec {
            name: "lookahead",
            default: 0.0,
        },
        ParamSpec {
            name: "spread",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

mod avoid;
use avoid::{AVOID, AVOID_RADIUS, LOOKAHEAD, avoid_accel};

struct Params {
    count: usize,
    seed: u32,
    radius_sq: f32,
    separation: f32,
    /// **O `desiredseparation` de Reynolds** — a distância abaixo da qual dois
    /// agentes se empurram, em unidades de MUNDO. `0` é desligado.
    ///
    /// ⚠️ **Ele existe porque o `radius` respondia a DUAS perguntas.** Reynolds
    /// (*Steering Behaviors*, GDC 1999) e toda implementação canônica separam
    /// *"quem eu vejo"* de *"quão perto é perto demais"* — Shiffman usa
    /// `neighbordist = 50` com `desiredseparation = 25`, o POP Steer Separate do
    /// Houdini tem raio próprio, os boids de Unity/Unreal têm `separationRadius`.
    /// Aqui os dois eram o MESMO número, e o preço está medido: apertar o
    /// `radius` para fazer um bando mais LOCAL (0,5) leva a sobreposição de 82%
    /// para **100%** — a percepção e o espaço pessoal andavam juntos, e um deles
    /// tinha de ir para o lado errado.
    ///
    /// ⚠️ **E a LEI muda com ele, de propósito.** Desligado, a repulsão é
    /// `d̂/dist` sem escala própria (o `diff` do Shiffman), e o equilíbrio é um
    /// balanço mole entre pesos: medido, `0,8 → 25,6` de `separation` — **32× o
    /// peso** — compra **2,4×** de distância, e para 40 agentes de tamanho `1,0`
    /// não se sobreporem é preciso `51,2`, contra um slider que para em `6,0`.
    /// Ligado, a força é uma MOLA DE COMPRESSÃO (`(R − dist)/R`): zero na borda,
    /// máxima no contacto, e o equilíbrio **tende a `R` quando o peso sobe** — que
    /// é o que torna o número um alvo em vez de um coeficiente.
    ///
    /// ⚠️ Guardado **AO QUADRADO** e mais nada: o laço compara distâncias ao
    /// quadrado, `0` é o sinal de *desligado* nos dois, e um segundo campo com o
    /// valor cru seria a mesma grandeza em dois sítios.
    sep_radius_sq: f32,
    alignment: f32,
    cohesion: f32,
    seek: f32,
    avoid: f32,
    avoid_radius: f32,
    lookahead: f32,
    max_speed: f32,
    /// **O orçamento de DIREÇÃO** — o segundo clamp de Reynolds (*Steering
    /// Behaviors For Autonomous Characters*, GDC 1999). `max_speed` limita quão
    /// RÁPIDO um agente vai; este limita quão FORTE ele consegue virar, e o
    /// modelo canônico tem os dois: sem ele um bando com `cohesion` alta faz
    /// curvas de ângulo reto, porque a aceleração de steering não tem teto.
    ///
    /// ⚠️ **`0` é DESLIGADO, e o encoding não custa capacidade:** um orçamento
    /// de steering exatamente zero é o agente que não vira nada, e isso já é
    /// alcançável zerando os quatro pesos (`separation`/`alignment`/`cohesion`/
    /// `seek`) — então o zero está livre para carregar *sem teto*. A
    /// alternativa (default `∞`) poria `inf` na caixa numérica do artista.
    max_force: f32,
    /// √N seed spread: when on, the seed cloud grows with the count so the
    /// density stays bounded and the spatial grid stays `O(N)` — the difference
    /// between a dense ball and a murmuration at a million agents. Off (the
    /// default) is the fixed [`SEED_SPREAD`], byte-identical at every count.
    /// **O CONE DE VISÃO**, em cosseno do meio-ângulo — a segunda metade da
    /// vizinhança de Reynolds (*Steering Behaviors*, GDC 1999: a vizinhança é
    /// distância **E** ângulo; Houdini POP Steer Separate/Cohesion/Align: *"raio
    /// + ângulo de visão"*).
    ///
    /// ⚠️ **`-1` (360°) é DESLIGADO e o teste é PULADO, não satisfeito** — e a
    /// diferença é o que torna o default byte-idêntico: a comparação angular
    /// custa uma raiz por vizinho, então deixá-la correr e sempre passar pagaria
    /// o preço do cone em toda cena que não o usa. O ramo é o que a mantém em
    /// zero.
    ///
    /// ⚠️ **O QUE ELE FAZ está MEDIDO, e não é o que a folha previa.** A célula
    /// que abriu este P1 dizia *"um boid que enxerga atrás de si não vira bando,
    /// vira nuvem"*, e a sonda `measure_what_the_view_cone_does_to_the_flock`
    /// (48 agentes, 600 ticks, média de 8 SEMENTES) diz o contrário na régua
    /// canônica de *"isto é um bando?"* — o parâmetro de ordem de Vicsek:
    ///
    /// | fov | polarização | agitação °/s | spread |
    /// |----:|------------:|-------------:|-------:|
    /// | 360 |      0,7145 |         60,0 |  2,280 |
    /// | 110 |      0,2762 |        127,6 |  1,749 |
    /// |  30 |      0,2238 |        135,4 |  1,121 |
    ///
    /// Estreitar o cone **APERTA o bando e o AGITA**; ele não o polariza. É o que
    /// o smoke do Enio reportou (*"coordenado com valores altos, agitado com
    /// valores baixos"*), e é a descrição honesta do controle.
    ///
    /// ⚠️ **A causa é a vizinhança que ele ENCOLHE, e o controle prova a maior
    /// parte disso:** encolher o `radius` — uma borda de DISTÂNCIA, que não gira
    /// com o agente — reproduz o mesmo movimento (churn 60,0 → 112,1 de `r` 2,0 a
    /// 0,5). Não é a mesma coisa, e a diferença também está medida: à spread
    /// casada (~1,75) o cone cobra **127,6** contra **81,2** do raio e polariza
    /// **0,28 contra 0,45**, o resíduo que a borda ANGULAR acrescenta por girar
    /// junto com quem olha.
    cos_half_fov: f32,
    /// O PISO de velocidade, como fração de [`Params::max_speed`].
    ///
    /// ⚠️ **Fração e não velocidade absoluta, e a razão é BYTE-IDENTIDADE:** o
    /// número sempre existiu como a const `MIN_SPEED_FRAC = 0.2` e o piso vivo
    /// era `max_speed * 0.2`, então expô-lo em unidades de mundo pediria um
    /// default que **depende de outro param** — e mudaria o desenho de todo
    /// documento cujo `max_speed` não fosse o de fábrica, em silêncio. O rótulo
    /// diz *Speed Floor* justamente para o `0,2` não ser lido como m/s.
    min_speed_frac: f32,
    spread: bool,
}

impl Params {
    /// The seed cloud's half-extent: fixed, or grown with √N (bounded density).
    /// At [`SEED_DENSITY_REF`] agents the two agree, so `spread` never changes
    /// the reference-count flock — it only holds the density as the count moves.
    fn seed_half_extent(&self) -> f32 {
        if self.spread {
            SEED_SPREAD * (self.count as f32 / SEED_DENSITY_REF).sqrt()
        } else {
            SEED_SPREAD
        }
    }
}

mod stream;
use stream::{accel_col, inv_mass_col, norm, scalar_col, value_head, vec2_col};

/// Seed a hashed cloud of positions + velocities around `home` (tick 0 / a count
/// change). Stateless: a pure function of `(seed, index)`, so it reproduces.
fn seed(home: [f32; 2], p: &Params) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut pos = Vec::with_capacity(p.count);
    let mut vel = Vec::with_capacity(p.count);
    let half = p.seed_half_extent();
    for i in 0..p.count as u32 {
        pos.push([
            home[0] + (hash3(p.seed, i, 0) - 0.5) * 2.0 * half,
            home[1] + (hash3(p.seed, i, 1) - 0.5) * 2.0 * half,
        ]);
        // Initial velocities spread across ±max_speed/2 so the flock starts alive.
        vel.push([
            (hash3(p.seed, i, 2) - 0.5) * p.max_speed,
            (hash3(p.seed, i, 3) - 0.5) * p.max_speed,
        ]);
    }
    (pos, vel)
}

/// One flocking step as a pure function. `pos`/`vel` are this tick's entry state.
/// Graus de campo de visão → cosseno do meio-ângulo.
///
/// ⚠️ **`>= 360` devolve EXATAMENTE `-1`**, e o literal importa: é ele que o
/// [`step`] compara para decidir se PULA o teste angular, e um `cos(180°)` que
/// saísse `-0.99999994` deixaria o ramo ligado — o cone "desligado" passaria a
/// custar uma raiz por vizinho em toda cena, com o desenho igual e o relógio
/// não.
fn cos_half_fov(deg: f32) -> f32 {
    if deg >= 360.0 {
        return -1.0;
    }
    libm::cosf(deg.max(0.0).to_radians() * 0.5)
}

// Os oito são o estado do bando mais o mundo à volta dele; uma struct seria um
// segundo modelo dos params que o `NodeManifest` já lista (o precedente do
// `field.remap::remap`).
#[allow(clippy::too_many_arguments)]
fn step(
    pos: &[[f32; 2]],
    vel: &[[f32; 2]],
    ext_accel: &[[f32; 2]],
    inv_mass: &[f32],
    obstacles: &[[f32; 2]],
    target: [f32; 2],
    dt: f32,
    p: &Params,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let n = pos.len();
    let mut out_pos = vec![[0.0f32; 2]; n];
    let mut out_vel = vec![[0.0f32; 2]; n];
    let min_speed = p.max_speed * p.min_speed_frac;

    for i in 0..n {
        let (pi, vi) = (pos[i], vel[i]);
        // ⚠️ **MASSA INFINITA: o agente não se move — e continua a ser VISTO.** O
        // laço de vizinhos lê `pos[j]`/`vel[j]` das arrays de ENTRADA, então um
        // agente pinado por um `motion.pin_constraint` na cadeia de estado segue
        // a repelir os outros pela separação: é o *obstáculo em torno do qual a
        // multidão tem de passar* que o doc do pino nomeia, sem um kernel novo.
        // A velocidade dele vai a ZERO de propósito — um obstáculo parado que
        // guardasse heading faria o alinhamento dos vizinhos apontar para uma
        // direção que ninguém exibe.
        let w = inv_mass.get(i).copied().unwrap_or(1.0);
        if w <= 0.0 {
            out_pos[i] = pi;
            out_vel[i] = [0.0, 0.0];
            continue;
        }
        // Accumulate the three Reynolds urges over neighbours within the radius.
        let mut sep = [0.0f32; 2];
        let mut align = [0.0f32; 2];
        let mut centroid = [0.0f32; 2];
        let mut neighbours = 0u32;
        for j in 0..n {
            if j == i {
                continue;
            }
            let d = [pi[0] - pos[j][0], pi[1] - pos[j][1]];
            let dist_sq = d[0] * d[0] + d[1] * d[1];
            if dist_sq < EPS {
                continue;
            }
            // **O ESPAÇO PESSOAL É FÍSICO, NÃO PERCEPTUAL** — e é por isso que ele
            // corre ANTES do gate de percepção e do cone: um agente encostado nas
            // suas costas empurra-o, veja-o ou não. Pô-lo depois do cone faria dois
            // agentes num cone estreito atravessarem-se de frente, e pô-lo depois do
            // `radius` faria um espaço pessoal maior que a percepção ser truncado
            // **em silêncio** por outro slider.
            //
            // ⚠️ O ramo é o que mantém `separation_radius = 0` byte-idêntico, e é o
            // mesmo desenho do cone de visão abaixo: o desligado não paga a raiz que
            // a lei nova custa.
            // ⛔ **MEDIDO E REJEITADO — não refaça: a MOLA DE COMPRESSÃO.** A minha
            // primeira lei aqui foi `(R − dist)/R / dist`, escolhida para *"fazer de
            // `R` um alvo"*: zero na borda, máxima no contacto. Medida com 40
            // agentes, ela é **uniformemente PIOR** que a inverse-square (mediana
            // 0,387 contra 0,748 no mesmo peso), e a razão é aritmética — perto do
            // contacto `(R−d)/R → 1` e as duas colapsam em `1/d`, longe dele a nova
            // é estritamente menor. Ela não é um alvo; é a lei antiga atenuada.
            //
            // ⚠️ **E a lição por trás dela é FÍSICA, não afinação:** um equilíbrio
            // entre forças FINITAS cai sempre ABAIXO do raio onde a repulsão se
            // anula. Para o espaçamento pousar num número é preciso repulsão
            // infinita ali — que é uma RESTRIÇÃO, não um steering, e é exactamente o
            // que o `motion.collide` na cadeia de estado entrega (medido: mediana
            // **0,9878** de um diâmetro de 1,0). *Nenhum knob deste nó pode prometer
            // não-sobreposição, e o doc do módulo diz para onde apontar quem a quer.*
            if p.sep_radius_sq > 0.0 && dist_sq < p.sep_radius_sq {
                let inv = 1.0 / dist_sq;
                sep[0] += d[0] * inv;
                sep[1] += d[1] * inv;
            }
            if dist_sq > p.radius_sq {
                continue;
            }
            // O CONE DE VISÃO. ⚠️ O ramo é o que mantém 360° byte-idêntico: com
            // `cos_half_fov == -1` nada aqui roda, então o disco de hoje não paga
            // a raiz que a comparação angular custa.
            if p.cos_half_fov > -1.0 {
                let v_sq = vi[0] * vi[0] + vi[1] * vi[1];
                // ⚠️ **Um agente PARADO enxerga tudo, e é decisão:** sem heading
                // não há cone, e cegá-lo faria dele um retardatário permanente —
                // ele nunca mais reencontraria o bando que o deixou para trás.
                if v_sq > EPS {
                    // `d` aponta do vizinho PARA cá, então a direção do olhar
                    // para ele é `-d`. O sinal trocado põe o cone nas costas.
                    let ahead = -d[0] * vi[0] - d[1] * vi[1];
                    if ahead < p.cos_half_fov * (dist_sq * v_sq).sqrt() {
                        continue;
                    }
                }
            }
            // Separation: inverse-square repulsion (bites hardest when closest) —
            // o `diff` do Shiffman, **perceptual**, e a lei que shipa quando não há
            // espaço pessoal autorado. Com ele autorado, a separação já correu lá em
            // cima e este termo cala-se: dois autores da mesma força discordariam
            // sobre o alvo.
            if p.sep_radius_sq <= 0.0 {
                let inv = 1.0 / dist_sq;
                sep[0] += d[0] * inv;
                sep[1] += d[1] * inv;
            }
            // Alignment / cohesion accumulate neighbour heading / position.
            align[0] += vel[j][0];
            align[1] += vel[j][1];
            centroid[0] += pos[j][0];
            centroid[1] += pos[j][1];
            neighbours += 1;
        }

        let mut accel = [0.0f32; 2];
        if neighbours > 0 {
            let inv_n = 1.0 / neighbours as f32;
            // Alignment: steer toward the average heading.
            accel[0] += (align[0] * inv_n - vi[0]) * p.alignment;
            accel[1] += (align[1] * inv_n - vi[1]) * p.alignment;
            // Cohesion: steer toward the centroid.
            accel[0] += (centroid[0] * inv_n - pi[0]) * p.cohesion;
            accel[1] += (centroid[1] * inv_n - pi[1]) * p.cohesion;
        }
        // Separation: push away.
        //
        // ⚠️ **FORA do `neighbours > 0`, e a diferença é load-bearing quando há
        // espaço pessoal:** com `separation_radius > radius` um agente pode ter
        // alguém encostado e NINGUÉM na percepção, e ali a soma seria descartada —
        // dois corpos a atravessarem-se porque nenhum dos dois viu o outro. Com o
        // param desligado `sep` só tem termos quando `neighbours > 0`, então mover
        // isto para fora é **byte-idêntico** (somar `0 · separation` é somar zero).
        accel[0] += sep[0] * p.separation;
        accel[1] += sep[1] * p.separation;
        // Seek/home: a linear spring toward the target — herds AND bounds.
        accel[0] += (target[0] - pi[0]) * p.seek;
        accel[1] += (target[1] - pi[1]) * p.seek;
        // Desvio de obstáculo: MÚSCULO do agente, então entra ANTES do tecto de
        // steering — virar para não bater é exactamente o que o `max_force` orçamenta.
        if p.avoid > 0.0
            && let Some(a) = avoid_accel(pi, vi, obstacles, p.avoid_radius, p.lookahead)
        {
            accel[0] += a[0] * p.avoid;
            accel[1] += a[1] * p.avoid;
        }
        // Reynolds' STEERING BUDGET (GDC 1999) — truncate what the agent's own
        // muscle may ask for, before anything the WORLD does to it. `0` is off.
        //
        // ⚠️ The external accel is deliberately OUTSIDE this: `max_force` is how
        // hard a boid can TURN, not how much wind it can ignore. Clamping the
        // two together would let a steering budget cancel a gale — a flock that
        // resists arbitrary force is not a flock, and it would quietly undo the
        // whole point of a `force.*` reaching the state chain.
        if p.max_force > 0.0 {
            let (unit, mag) = norm(accel);
            if mag > p.max_force {
                accel = [unit[0] * p.max_force, unit[1] * p.max_force];
            }
        }
        // The EXTERNAL urge, from a `force.*` in this flock's state chain: a
        // fourth term beside the three Reynolds ones, added BEFORE the speed
        // clamp on purpose — `max_speed` still bounds the flock, which is what
        // makes `force.curl` read as Reynolds' *wander* instead of a shove.
        let ext = ext_accel.get(i).copied().unwrap_or([0.0, 0.0]);
        accel[0] += ext[0];
        accel[1] += ext[1];
        // A massa inversa escala TUDO o que chega ao agente — o impulso próprio e
        // o do mundo —, exactamente como o `motion.integrate` escala a velocidade
        // por ela: um agente pesado vira devagar e resiste ao vento. Peso `1` é o
        // bando de hoje **ao bit** (`x · 1.0` é exacto em IEEE-754).
        accel[0] *= w;
        accel[1] *= w;

        // Integrate, then clamp the speed into [min, max] so the flock glides.
        let mut nv = [vi[0] + accel[0] * dt, vi[1] + accel[1] * dt];
        let (unit, speed) = norm(nv);
        if speed > p.max_speed {
            nv = [unit[0] * p.max_speed, unit[1] * p.max_speed];
        } else if speed > EPS && speed < min_speed {
            nv = [unit[0] * min_speed, unit[1] * min_speed];
        }
        let mut np = [pi[0] + nv[0] * dt, pi[1] + nv[1] * dt];
        // NaN/∞ guard (reference parity): a diverged agent recovers at the target.
        if !(np[0].is_finite() && np[1].is_finite() && nv[0].is_finite() && nv[1].is_finite()) {
            np = target;
            nv = [0.0, 0.0];
        }
        out_pos[i] = np;
        out_vel[i] = nv;
    }
    (out_pos, out_vel)
}

/// The whole node as a pure function: seed on the first tick / a count change,
/// else step. Emits `P` + the `vel`/`sim_t` state.
fn simulate(
    target: [f32; 2],
    state: &Stream,
    obstacles: &[[f32; 2]],
    playhead: f32,
    p: &Params,
) -> Stream {
    let s_pos = vec2_col(state, "P");
    let s_vel = vec2_col(state, "vel");

    let (pos, vel) = if s_pos.len() == p.count && s_vel.len() == p.count {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        let accel = accel_col(state, p.count);
        let w = inv_mass_col(state, p.count);
        step(&s_pos, &s_vel, &accel, &w, obstacles, target, dt, p)
    } else {
        seed(target, p)
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("vel", Column::Vec2(vel))
        .with("sim_t", Column::Scalar(vec![playhead; p.count]))
}

struct MotionBoids;

impl NodeOp for MotionBoids {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS).max(1);
        let radius = ctx.param("radius").max(0.0);
        let sep_r = ctx.param("separation_radius").max(0.0);
        let p = Params {
            count,
            seed: ctx.param("seed").max(0.0).round() as u32,
            radius_sq: radius * radius,
            separation: ctx.param("separation").max(0.0),
            sep_radius_sq: sep_r * sep_r,
            alignment: ctx.param("alignment").max(0.0),
            cohesion: ctx.param("cohesion").max(0.0),
            seek: ctx.param("seek").max(0.0),
            avoid: ctx.param(AVOID).max(0.0),
            avoid_radius: ctx.param(AVOID_RADIUS).max(0.0),
            lookahead: ctx.param(LOOKAHEAD).max(0.0),
            max_speed: ctx.param("max_speed").max(0.01),
            max_force: ctx.param("max_force").max(0.0),
            // O cosseno é computado UMA vez por cook, nunca por vizinho — o laço
            // interno é `O(N²)` na CPU e um transcendental lá dentro seria o
            // custo do cone multiplicado pela vizinhança inteira.
            cos_half_fov: cos_half_fov(ctx.param("fov")),
            min_speed_frac: ctx.param("speed_floor").clamp(0.0, 1.0),
            spread: ctx.param("spread") > 0.5,
        };
        let playhead = ctx.playhead() as f32;
        let target = [
            value_head(&scalar_col(ctx.input(0), VALUE_COL)),
            value_head(&scalar_col(ctx.input(1), VALUE_COL)),
        ];
        // ⚠️ A porta dos obstáculos é lida ANTES do estado e clonada: dois `ctx.input`
        // não podem coexistir emprestados, e ela só é tocada quando o desvio está
        // ligado (com `avoid = 0` a lista fica vazia e a lei nem corre).
        let obstacles: Vec<[f32; 2]> = if p.avoid > 0.0 {
            vec2_col(ctx.input(3), "P")
        } else {
            Vec::new()
        };
        let state = ctx.input(2);
        let out = simulate(target, state, &obstacles, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionBoids))?;
    // GPU/M5 (ADR-0140): the flock on the device via the spatial grid.
    reg.register_gpu_kernel(MANIFEST.id, gpu::GPU_KERNEL);
    reg.register_grid(MANIFEST.id, gpu::GRID);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Boids",
            // Source: it mints its own agent stream, then simulates it.
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // ⚠️ **O `avoid` é o INTERRUPTOR dos outros dois** (doc 90 §2): com `avoid = 0` — o default —
    // a lista de obstáculos sai vazia e nem o raio nem a antecipação são lidos. O doc-comment do
    // `avoid.rs` dizia-o desde sempre; o painel pintava os três na mesma, e o artista não tem
    // como adivinhar que o terceiro slider liga os dois primeiros.
    reg.register_param_gates_above(MANIFEST.id, PARAM_GATES_ABOVE);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // ADR-0155: this flock CONSUMES `accel`, so a `force.*` in its state chain
    // is live instead of inert — and the diagnose stops offering to splice a
    // `motion.integrate`, which is `Temporal` and would stamp `sim_t = playhead`
    // on the way past, handing this node `dt = 0` and FREEZING the flock.
    reg.register_couplings(
        MANIFEST.id,
        &[
            ph2d_node_registry::Coupling::Consumes("accel"),
            ph2d_node_registry::Coupling::Consumes("inv_mass"),
        ],
    );
    Ok(())
}

#[path = "ui.rs"]
mod ui;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
use ui::{PARAM_GATES_ABOVE, PARAM_GROUPS, PARAM_HARD_MAX, PARAM_HINTS, PARAM_UNITS};
