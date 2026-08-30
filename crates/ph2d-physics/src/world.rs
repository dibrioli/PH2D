//! [`PhysicsWorld`] — Rapier2D wrapper sized to PH2D's 2D needs.
//!
//! All cross-frame state lives on this struct. Callers create it,
//! add bodies + colliders, then call [`PhysicsWorld::step`] once per
//! fixed-step tick (see [`ph2d_core::FixedStep`]).

pub mod blast;
/// A CONSTRUÇÃO de um corpo — irmão do `collider_build`, e pelo teto de 700 LOC.
mod body_build;
pub mod buoyancy;
/// Consultas de cena (raio/forma) — a perna da cápsula flutuante (docs dele).
pub mod cast;
pub mod character;
pub mod checkpoint;
pub mod collider_build;
pub mod contacts;
mod convenience;
pub mod damping;
pub mod defaults;
pub mod desc;
pub mod drag;
pub mod effector;
pub mod form_drag;
pub mod grab;
pub mod ik;
pub mod ik_coords;
pub mod joint_break;
pub mod joint_custom;
pub mod joint_desc;
pub mod joint_gains;
pub mod joints;
pub mod kinematic;
pub mod layers;
pub mod oneway;
/// Os colliders EXTRA de um corpo composto (docs dele).
pub mod parts;
/// Aplicar o motor de um player ao corpo (docs dele).
pub mod player;
pub mod pulley;
pub mod queries;
pub mod rope_load;
pub mod rope_route;
pub mod sensors;
pub mod shape;
mod shapes;
mod step;
pub mod sweep;
pub mod tuning;

use defaults::BodyDefaults;
use layers::LayerMatrix;
/// A porta das camadas mudou-se para o modulo delas (teto de 700 LOC); o nome fica aqui
/// porque os quatro consumidores a alcancam por `use super::groups_for`.
pub(super) use layers::groups_for;
// The descriptors and the shape vocabulary live in sibling modules (LOC),
// re-exported so callers still see `ph2d_physics::{BodyDesc, ShapeDesc, …}`.
pub use desc::{AreaEffect, BodyDesc, BodySnapshot, CombineRules, DampingDesc};
pub use shape::{CAPSULE_CAP_SEGS, ELLIPSE_SEGS, ShapeDesc, capsule_vertices, ellipse_vertices};

use crate::rmath::{Pose, Vector};
use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, RigidBody,
    RigidBodyHandle, RigidBodySet, RigidBodyType,
};
use rapier2d::geometry::{BroadPhaseBvh, ColliderHandle, ColliderSet, NarrowPhase};
use rapier2d::pipeline::PhysicsPipeline;

pub struct PhysicsWorld {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    gravity: Vector,
    /// Step counter — exposes a deterministic "tick number" so tests
    /// can advance N steps and report.
    step_count: u64,
    /// Sub-steps per `step()` (see [`PhysicsWorld::set_substeps`]).
    substeps: u32,
    /// The caller's tick length; `integration_parameters.dt` is this divided
    /// by [`Self::substeps`].
    base_dt: f32,
    /// World-level values every new body is born with (damping, sleep). See
    /// [`BodyDefaults`] for why a per-body rapier concept is a world setting
    /// here — and for the one rule that keeps it a single door.
    body_defaults: BodyDefaults,
    /// Which layers collide with which. See [`layers`] for why this lives on
    /// the world and why it cannot be asymmetric.
    layer_matrix: LayerMatrix,
    /// Air-drag coefficient (lumps `½·ρ·Cd`). `0.0` = vacuum, and byte-identical
    /// to a build without the feature. See [`drag`] for why this is a separate
    /// model from `body_defaults.linear_damping` rather than a tuning of it.
    air_drag: f32,
    /// Where each kinematic body was told to be by the END of this tick:
    /// `(handle, pose at the start of the tick, pose to arrive at)`.
    ///
    /// It is a field and not a local because [`PhysicsWorld::step`] is the hot
    /// path with a zero-alloc gate on it — cleared per tick, so the capacity is
    /// reached once and never allocated again. Empty in a world with no
    /// kinematic bodies, which is what keeps `step` byte-identical to the one
    /// that shipped before this existed.
    kinematic_targets: Vec<(RigidBodyHandle, Pose, Pose)>,
    /// **A aceleração que a perna de um player pede neste tique**, em m/s² —
    /// consumida por SUB-PASSO dentro de [`PhysicsWorld::step`].
    ///
    /// ⚠️ **É um `Vec` pelo mesmo motivo que `kinematic_targets` é**, e é
    /// consumido no MESMO laço pelo mesmo motivo que o `drag`: uma força
    /// aplicada uma vez por tique é integrada de um jeito e a gravidade de
    /// outro, e a diferença **não aparece na velocidade** (o impulso total é o
    /// mesmo) — só no DESLOCAMENTO. Ver o aviso de
    /// [`PhysicsWorld::queue_player_accel`].
    ///
    /// Vazio em todo mundo sem player, que é o que mantém o `step` byte-idêntico
    /// para o resto do módulo.
    player_accels: Vec<(RigidBodyHandle, [f32; 2])>,
    /// The registered **force zones** (W-Area): a sensor body and what it does to
    /// whatever overlaps it — a force in newtons, a drag that resists, or both.
    /// Derived from `BodyDesc` at spawn through the single door
    /// `effector::zone_effect`, so it is CONFIG —
    /// nothing in the step loop writes it, which is why a checkpoint restore
    /// (which swaps the body/collider arenas, not this) leaves it valid.
    ///
    /// **Sorted by handle**: a body standing in two overlapping zones sums their
    /// impulses, and the order of a float sum is exactly the kind of detail that
    /// makes a cross-OS hash drift (HR-5).
    ///
    /// The third element is the zone's own **silhouette**, copied from the same
    /// `BodyDesc` (W-AreaFalloff): the falloff measures how far out a body is *as a
    /// fraction of this shape*, so the zone has to remember what it looks like. It is
    /// carried here rather than re-read from the collider because this table is already
    /// the zone's config record, and asking rapier's `Shape` trait instead would be a
    /// second vocabulary for a silhouette `ShapeDesc` already names exactly.
    effectors: Vec<(RigidBodyHandle, desc::AreaEffect, shape::ShapeDesc)>,
    /// The live pulleys (W-Pulley) — rope constraints rapier does not have, so
    /// they are imposed from outside by a per-substep impulse pass, exactly like
    /// the drag and the force zones above it.
    ///
    /// CONFIG, not solver state: the bridge re-stamps the whole table every
    /// dispatch from the authored components, which is why it is absent from the
    /// checkpoint ring for the same reason `effectors` is — a restore keeps the
    /// table the bridge just installed rather than resurrecting one from a run
    /// that ended.
    pulleys: Vec<pulley::PulleyDesc>,
    /// As roldanas que as faixas de `pulleys` indexam — uma arena só para todas
    /// as cordas, pelo motivo escrito no [`pulley::PulleyDesc`]: um `Vec` por
    /// corda alocaria por frame e tiraria o `Copy` que deixa a tabela ser
    /// trocada com um `swap`.
    pulley_wheels: Vec<rope_route::RopeWheel>,
    /// **Quanto de corda cada guincho já recolheu**, em metros, por
    /// [`pulley::PulleyDesc::id`] (W2).
    ///
    /// ⚠️ Este é o ÚNICO estado de polia que NÃO é config, e por isso o único que
    /// o [`checkpoint::PhysicsCheckpoint`] captura: a tabela acima é reinstalada
    /// por dispatch, mas o recolhido é uma INTEGRAL de uma taxa, e um scrub que
    /// acertasse o ring sem ele veria o guincho no lugar de agora dentro de um
    /// mundo do tick de então.
    ///
    /// Chaveado por nome estável e não por índice porque a tabela é reinstalada
    /// inteira — acrescentar uma corda deslocaria os índices e um guincho passaria
    /// a recolher no lugar do vizinho. `BTreeMap` pela lei de sempre: iteração por
    /// ordem de chave, reproduzível cross-OS.
    ///
    /// **Vazio em toda cena sem tambor**, o que mantém esta wave byte-neutra.
    pulley_payout: std::collections::BTreeMap<u64, f32>,
    /// **A maior tensão que cada corda segurou neste tique**, newtons (W2) — o
    /// readout, e o número que decide a ruptura.
    ///
    /// Limpo no topo de cada `step`, como o `joint_peaks`: a carga de um tique é
    /// um fato sobre aquele tique, e o *high-water* de uma CORRIDA é acumulado
    /// pela ponte (que é quem sabe quando uma corrida começa).
    pulley_tension: std::collections::BTreeMap<u64, f32>,
    /// O mesmo, por EIXO de roldana — a resultante do desvio, que não é a tensão.
    pulley_axle: std::collections::BTreeMap<u64, f32>,
    /// **O que já rompeu.** Estado SIMULADO, não config: a ruptura nunca é
    /// escrita no componente autorado (senão desfazê-la seria trabalho do artista
    /// em vez de um Reset), e por isso ela viaja no checkpoint junto com o
    /// recolhido. Um `rebuild_from_rest` constrói um mundo novo e a desfaz.
    pulley_broken_ropes: std::collections::BTreeSet<u64>,
    /// Idem, por roldana: um eixo partido **sai da rota**.
    pulley_broken_wheels: std::collections::BTreeSet<u64>,
    /// O que rompeu no tique que acabou de rodar — canal de TRANSIÇÃO, vazio em
    /// quase todo tique, e é esse o ponto.
    pulley_breaks: Vec<rope_load::PulleyBreak>,
    /// As roldanas VIVAS da corda que o passe está resolvendo (as não-rompidas).
    /// Campo pelo mesmo motivo do `route_scratch`: o passe roda por sub-passo, e
    /// alocar aqui apareceria no gate de zero-alloc do caminho quente.
    pulley_live: Vec<rope_route::RopeWheel>,
    /// Os trechos da rota da corda que o passe está resolvendo. Buffer PRÓPRIO
    /// e persistente porque a rota escreve `N+1` trechos por corda por sub-passo
    /// — alocá-los por chamada apareceria no gate de zero-alloc do caminho
    /// quente, do mesmo jeito que os scratches de contato apareceriam.
    route_scratch: Vec<rope_route::Tangent>,
    /// The Baumgarte fraction the pulley pass corrects per sub-step. A field
    /// rather than the constant read inline **so the measured table on
    /// [`pulley::PULLEY_BIAS`] is reproducible against the PRODUCT path** and not
    /// against a second copy of the pass living in a test file — the same reason
    /// (and the same shape) as `grab_body_with` and `spawn_joint_tuned`.
    pulley_bias: f32,
    /// Quantos sub-passos de recolhimento a correção pode ficar para trás. Campo,
    /// e não a constante lida inline, pelo MESMO motivo do `pulley_bias` uma linha
    /// acima: é o que torna a tabela medida de [`pulley::PULLEY_CORRECTION_LAG`]
    /// reproduzível contra o caminho do PRODUTO em vez de contra uma segunda cópia
    /// do passe morando num arquivo de teste.
    pulley_lag: f32,
    /// The **impact peak** of each touching pair over the current tick's
    /// sub-steps (W-ImpactForce): body pair (lower handle first) → the hardest
    /// summed normal impulse it reached at any single sub-step.
    ///
    /// It is a READOUT, not simulation state — nothing in the solver reads it, so
    /// it is invisible to the C9 determinism hash (which is body poses), exactly
    /// like `contact_reports` is. It exists because the peak lives *between* the
    /// sub-steps and is gone by the time `step` returns: cleared at the start of
    /// each `step` and folded by `max` after each sub-step, so the readback finds
    /// the peak of the last tick. `BTreeMap` (not `HashMap`) so the readout is
    /// reproducible cross-OS, the same rule the whole module follows.
    ///
    /// A field and not a local for the same reason `kinematic_targets` is: `step`
    /// is the hot path with a zero-alloc gate, so the capacity is reached once and
    /// reused (`clear` keeps it). Empty for a scene where nothing touches.
    contact_peaks: std::collections::BTreeMap<contacts::PeakKey, contacts::PeakSample>,
    /// **O livro-razão da descida** (W20) — preenchido pelo gancho one-way
    /// durante o `step` e lido no topo do tique seguinte, quando a ponte
    /// pergunta se a descida ainda está a fazer trabalho. Vazio em toda cena em
    /// que ninguém está a atravessar nada.
    drop_ledger: oneway::DropLedger,
    /// Scratch for the per-sub-step fold that fills [`Self::contact_peaks`]: the
    /// tick's actively-touching COLLIDER pairs, sorted, before they are merged into
    /// per-BODY answers (W-CompoundContact).
    ///
    /// A field for the same reason its sibling is — `step` runs this once per
    /// sub-step and the zero-alloc gate measures exactly that; `Vec::clear` keeps the
    /// capacity, so a steady scene reaches it once. It carries nothing between calls:
    /// every use begins by refilling it.
    contact_scratch: Vec<contacts::ActivePair>,
    /// The **peak reaction** each joint carried over the current tick's sub-steps
    /// (W-J7): joint handle → the largest force and torque it was resisting at any
    /// single sub-step. Cleared and refilled exactly like [`Self::contact_peaks`],
    /// and for the same reason — the peak lives between the sub-steps.
    ///
    /// Unlike the contact ledger this one is *read by the simulation*: it is where
    /// a break is decided. A world with no finite threshold never compares
    /// anything, so it stays byte-identical.
    joint_peaks: std::collections::BTreeMap<joint_break::JointKey, joint_break::JointLoad>,
    /// The joints that gave way during the current tick — an EVENT channel, drained
    /// by the bridge (W-TickContacts' lesson: a transition is not derivable from the
    /// state that follows it).
    joint_breaks: Vec<joint_break::JointBreak>,
    /// **A mão** (W-Grab): o corpo que o artista está segurando enquanto a sim
    /// corre, e a tralha que o realiza. `None` no caso comum.
    ///
    /// ⚠️ Deliberadamente **fora** do [`checkpoint`](checkpoint) — e não por
    /// esquecimento: é a única entrada deste mundo que não vem do documento, então
    /// um checkpoint que a contivesse tornaria a resposta de um scrub dependente do
    /// cache. Quem garante que isso nunca acontece é a ponte (`bridge::grab`), com
    /// gate; ver [`grab`].
    grab: Option<grab::Grab>,
    /// **O campo de atração em voo** (W-Hand) — irmão exato do `grab` acima, e
    /// FORA do checkpoint pela mesma razão: um cutucão não está no documento,
    /// então nenhum replay o reproduz e um `restore` que o trouxesse de volta
    /// descreveria uma corrida que já acabou.
    attract: Option<blast::Attract>,
}

impl PhysicsWorld {
    /// Default integration step. 60 Hz lockstep matches the rest of
    /// the engine ([`ph2d_core::FixedStep::DEFAULT_HZ`]).
    pub const DEFAULT_DT: f32 = 1.0 / 60.0;

    /// Default gravity (Y-up world per SKILL §11.1, so down is -y).
    /// 9.81 m/s² Earth-standard.
    pub const DEFAULT_GRAVITY_Y: f32 = -9.81;

    /// Integration sub-steps per tick. **Not rapier's default of 1** —
    /// chosen by measurement (Enio, 2026-07-18: *"observa-se alguma
    /// interpenetração dos objetos dinâmicos com o chão"*).
    ///
    /// A body landing at 9.4 m/s travels 157 mm in a 60 Hz tick, so on the
    /// tick it first touches it is **already 83 mm inside the floor** — and
    /// no solver knob moves that number, because it is not a solver failure,
    /// it is `velocity × dt`. Measured: contact damping, corrective-velocity
    /// ceiling, extra solver iterations and even CCD all left the depth at
    /// 83.2 mm. (CCD does nothing here because nothing *tunnels* — 83 mm of
    /// overlap on a 560 mm body is not a missed collision.)
    ///
    /// Sub-stepping is the only lever on the depth, and it is linear:
    /// 1→83 mm, 2→73, 4→31, 8→8.8. Four is the knee — Box2D v3 ships the
    /// same default for the same reason — and costs 264 µs for 500 bodies
    /// against HR-4's 1.5 ms.
    pub const DEFAULT_SUBSTEPS: u32 = 4;

    /// Contact spring frequency, Hz. **Not rapier's default of 30** — that
    /// governs how fast the remaining overlap is pushed back out, which is
    /// the other half of what the artist sees: 30 Hz took 9 frames to become
    /// invisible, 120 Hz takes 1.
    ///
    /// Raising *damping* instead (rapier's usual advice for a stiffer look)
    /// was measured and goes the wrong way here — 5.0 is already overdamped,
    /// and 20 stretched the recovery from 9 frames to 30. Damping is left at
    /// rapier's tuned default.
    pub const DEFAULT_CONTACT_HZ: f32 = 120.0;

    pub fn new() -> Self {
        let integration_parameters = IntegrationParameters {
            dt: Self::DEFAULT_DT / Self::DEFAULT_SUBSTEPS as f32,
            // ⚠️ **O `damping_ratio` tem de vir escrito, e não do `..default()`.** Os dois
            // coeficientes viraram UM struct na rapier 0.31, então preencher só a frequência
            // não é possível — e o `5.0` aqui é exactamente o que o default da 0.28 dava, que
            // é o número que o doc acima diz ter sido MEDIDO (subir para 20 esticou a
            // recuperação de 9 para 30 quadros). *Uma constante que era implícita passou a ser
            // nossa; escrevê-la é o que impede o dia em que o upstream a muda por baixo.*
            contact_softness: rapier2d::dynamics::SpringCoefficients {
                natural_frequency: Self::DEFAULT_CONTACT_HZ,
                damping_ratio: 5.0,
            },
            // ⛔ **Um padrão da rapier mudou na 0.31 e ele MUDA O TATO: as iterações
            // internas de estabilização caíram de `2` para `1`.** Fica fixado no valor de
            // sempre, pelo critério da casa: *um padrão que muda o TATO é conteúdo autorado
            // — há cenas ajustadas contra ele —, enquanto um que muda a ARQUITECTURA do
            // solver se aceita, porque não há como o fixar.* Este é uma constante, e fixá-lo
            // custa uma linha.
            //
            // ⭐ **O que elas compram está MEDIDO, e não é o que se supõe.** A leitura
            // óbvia é *«mais estabilização = pilha em repouso mais quieta»*. O que o gate
            // `the_same_gesture_hangs_only_the_one_with_a_reach` mostra é outra coisa: ele
            // põe um personagem contra uma parede **sem beirada ao alcance**, e ele tem de
            // **escorregar**. Com `1` iteração ele deixou de escorregar de todo
            // (`1,727 → 1,750` em dois segundos, contra os `0,15 m` que a barra mede); com
            // `2` o escorregar volta e o gate fica verde. ⇒ estas iterações são
            // **load-bearing para o ATRITO**, não só para o tremor de uma pilha.
            // *Uma parede que segura quem não se agarrou não é economia de cálculo: é outra
            // regra de jogo.*
            //
            // ⛔⛔ **E uma hipótese REFUTADA, registada para não voltar:** a 0.29 introduziu
            // um `friction_model` configurável cujo padrão (`Simplified`) resolve uma
            // restrição de atrito por grupo de **4 contatos** em vez de uma por contacto — o
            // candidato óbvio para explicar a mudança no escorregar contra a parede. **Ele é
            // `#[cfg(feature = "dim3")]`**: não existe em 2D, e nunca nos alcançou. *O
            // compilador matou a hipótese antes da medição — e é por isso que se escreve o
            // palpite em código em vez de o assumir.*
            num_internal_stabilization_iterations: 2,
            // ⛔⛔ **SETE constantes da 0.35 mudam o TATO, e ficam fixadas no valor de sempre.**
            // O critério é o mesmo do bloco: *um padrão que muda o tato é conteúdo autorado — há
            // cenas ajustadas contra ele —, e um que muda a ARQUITECTURA do solver aceita-se,
            // porque não há como o fixar.* Estas são constantes; fixá-las custa uma linha cada.
            //
            // ⚠️ **`static_contact_softness` é um CONCEITO NOVO, e é o mais perigoso da lista.**
            // Até aqui **todo** contacto usava os nossos `DEFAULT_CONTACT_HZ`. A 0.35 partiu-os em
            // dois grupos: dinâmico↔dinâmico (`contact_softness`) e **contra corpos fixos**
            // (`static_contact_softness`, cujo padrão é 60 Hz / 10). O chão e as paredes são
            // fixos ⇒ ajustar só o primeiro deixaria **o contacto que mais importa** a metade da
            // rigidez que o slider «Contact Hz» do painel promete. *Uma lei escrita num sítio
            // quando o modelo tem dois não é uma lei.*
            static_contact_softness: rapier2d::dynamics::SpringCoefficients {
                natural_frequency: Self::DEFAULT_CONTACT_HZ,
                damping_ratio: 5.0,
            },
            // 0.001 → 0.005: os corpos afundariam **5×** mais uns nos outros em repouso.
            normalized_allowed_linear_error: 0.001,
            // 10 → 3: uma sobreposição funda sairia **3,3× mais devagar**.
            normalized_max_corrective_velocity: 10.0,
            // 0,002 → 0,02: contatos detectados **10× mais cedo**; corpos parecendo flutuar um
            // fio acima da superfície.
            normalized_prediction_distance: 0.002,
            // ⚠️ **Sem tecto → 400 u/s.** A 0.35 passou a limitar a velocidade linear. Em queda
            // livre a `9,81` isso levaria ~41 s a morder, mas o `blast::explode` divide impulso
            // por massa: um corpo leve com uma explosão forte era ilimitado e passaria a bater no
            // tecto **sem aviso**. `Real::MAX` desarma-o, que é o que sempre valeu aqui.
            normalized_max_linear_velocity: rapier2d::math::Real::MAX,
            // Duas reduções de trabalho no narrow phase que a 0.35 liga por omissão. Elas mudam a
            // TRAJECTÓRIA dentro da tolerância do solver — e portanto o hash `physics_ecs_c9`.
            // Ficam desligadas: o custo que elas poupam não foi medido, e o que elas mexem foi.
            contact_clustering: false,
            contact_recycling: false,
            // ⛔⛔ **DOIS campos novos ficam no padrão da 0.35, e a recusa é MEDIDA.**
            //
            // `friction_in_bias_pass: false` é a regra nova *«sem atrito ao aplicar o viés»*, e o
            // doc da rapier diz o que ela compra: *«load-bearing for tall stacks: friction
            // reacting to bias velocities pumps their coherent lean mode until they topple»*.
            // ⚠️ Ligá-lo a `true` **cura um gate** desta crate (`compound::a_part_moved_while_the
            // _clock_runs_takes_its_collider_along`) — e o gate é sobre uma peça composta a levar
            // o collider, não sobre atrito. ⇒ *seria comprar de volta um defeito que eles
            // corrigiram, para curar um sintoma que não é o dele.* Bissectado: dos dois campos,
            // este é o único que muda alguma coisa aqui.
            //
            // `warmstart_joints: false` fica pela razão oposta — medido, ligá-lo **não muda um
            // único gate**. Ele é capacidade nova (*«melhora a convergência de conjuntos de
            // juntas rígidas»*), e ligar o que não se mediu a precisar não é afinação, é ruído.
            //
            // ⚠️ *A régua desta decisão não é «o teste fica verde»: é «o que a plataforma
            // corrigiu de propósito, aceita-se».*
            ..IntegrationParameters::default()
        };
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_pipeline: PhysicsPipeline::new(),
            integration_parameters,
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            gravity: Vector::new(0.0, Self::DEFAULT_GRAVITY_Y),
            step_count: 0,
            substeps: Self::DEFAULT_SUBSTEPS,
            base_dt: Self::DEFAULT_DT,
            // ⚠️ `ours()`, não `rapier()`: os limiares de adormecer deixaram de ser os dela na
            // 0.35. Ver o doc de `BodyDefaults::ours`.
            body_defaults: BodyDefaults::ours(),
            layer_matrix: LayerMatrix::all(),
            air_drag: 0.0,
            kinematic_targets: Vec::new(),
            player_accels: Vec::new(),
            effectors: Vec::new(),
            pulleys: Vec::new(),
            pulley_wheels: Vec::new(),
            pulley_payout: std::collections::BTreeMap::new(),
            pulley_tension: std::collections::BTreeMap::new(),
            pulley_axle: std::collections::BTreeMap::new(),
            pulley_broken_ropes: std::collections::BTreeSet::new(),
            pulley_broken_wheels: std::collections::BTreeSet::new(),
            pulley_breaks: Vec::new(),
            pulley_live: Vec::new(),
            route_scratch: Vec::new(),
            pulley_bias: pulley::PULLEY_BIAS,
            pulley_lag: pulley::PULLEY_CORRECTION_LAG,
            contact_peaks: std::collections::BTreeMap::new(),
            drop_ledger: oneway::DropLedger::default(),
            contact_scratch: Vec::new(),
            joint_peaks: std::collections::BTreeMap::new(),
            joint_breaks: Vec::new(),
            grab: None,
            attract: None,
        }
    }

    /// The air-drag coefficient. `0.0` = vacuum.
    pub fn air_drag(&self) -> f32 {
        self.air_drag
    }

    /// Set the air-drag coefficient (lumps `½·ρ·Cd` — "how thick is the air").
    ///
    /// Unlike [`Self::set_body_defaults`], this touches no body: drag is a
    /// property of the WORLD, applied as a force each substep, so there is
    /// nothing to stamp and nothing to wake.
    pub fn set_air_drag(&mut self, k: f32) {
        self.air_drag = k.max(0.0);
    }

    /// Put a freshly inserted collider on its layer — the one place a `(layer,
    /// matrix)` pair becomes rapier `InteractionGroups`, shared with
    /// [`Self::set_layer_matrix`] so spawning and re-filtering can never
    /// disagree about what a layer means.
    fn stamp_layer(&mut self, handle: ColliderHandle, layer: usize) {
        let groups = groups_for(layer, self.layer_matrix);
        if let Some(c) = self.colliders.get_mut(handle) {
            c.set_collision_groups(groups);
        }
    }

    /// Stamp the world defaults onto a body that was just inserted — the single
    /// place every spawn path funnels through, so "every body in this world
    /// carries this world's defaults" is true by construction rather than by a
    /// list of call sites someone has to keep in sync.
    fn stamp_defaults(&mut self, handle: RigidBodyHandle) {
        if let Some(body) = self.bodies.get_mut(handle) {
            self.body_defaults.apply_to(body);
        }
    }

    /// Override the integration dt. Must match the caller's
    /// FixedStep — mismatched dt destroys both accuracy and
    /// determinism.
    pub fn set_dt(&mut self, dt: f32) {
        self.base_dt = dt;
        self.integration_parameters.dt = dt / self.substeps as f32;
    }

    /// The **tick** length — what one [`PhysicsWorld::step`] advances, and
    /// what must match the caller's `FixedStep`.
    ///
    /// Deliberately not the integrator's dt: `step()` runs
    /// [`PhysicsWorld::substeps`] internal integrations of `dt/substeps`
    /// each. One name, one meaning — a `dt()` that silently became the
    /// sub-step length would quietly disagree with the Playhead.
    pub fn dt(&self) -> f32 {
        self.base_dt
    }

    /// The integrator's own step: [`Self::dt`] divided by [`Self::substeps`].
    pub fn substep_dt(&self) -> f32 {
        self.integration_parameters.dt
    }

    /// Integration sub-steps per [`PhysicsWorld::step`].
    pub fn substeps(&self) -> u32 {
        self.substeps
    }

    /// How many kinematic aims are queued for the next [`PhysicsWorld::step`]
    /// — the per-sub-step replay list. Exists so the refusal in
    /// [`PhysicsWorld::set_next_kinematic_pose`] is a claim a test can check
    /// rather than a comment; the effect it guards is cost, and cost is not
    /// visible in a pose.
    ///
    /// [`step`]: PhysicsWorld::step
    #[doc(hidden)]
    #[must_use]
    pub fn kinematic_aim_count(&self) -> usize {
        self.kinematic_targets.len()
    }

    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Teleport a body to `(x, y, rotation)` (world units, radians) and
    /// reset its velocity — the "settle to the authored pose" operation
    /// the bridge uses while paused, so a body sits exactly where the
    /// artist placed it before play. `wake` requests the body be woken
    /// (pass `true` when the pose actually changed).
    pub fn set_body_pose(
        &mut self,
        handle: RigidBodyHandle,
        x: f32,
        y: f32,
        rotation: f32,
        wake: bool,
    ) {
        if let Some(b) = self.bodies.get_mut(handle) {
            b.set_position(Pose::new(Vector::new(x, y), rotation), wake);
            b.set_linvel(Vector::ZERO, wake);
            b.set_angvel(0.0, wake);
        }
    }

    /// **Quantos corpos a ARENA tem** — incluindo os que não têm entidade
    /// (âncoras de mundo, a tralha da mão).
    ///
    /// ⚠️ **Não confundir com `PhysicsBridge::body_count`**, que conta o mapa
    /// entidade→corpo da ponte. Um gate de vazamento de âncora perguntado àquele
    /// **não pode falhar**: a âncora não tem entidade, então ela nunca aparece
    /// lá — e o gate ficaria verde exatamente sobre o defeito que ele alega
    /// pegar.
    #[must_use]
    pub fn arena_body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Remove a body and its attached colliders (used when the ECS entity
    /// carrying it is despawned). No-op if the handle is already gone.
    pub fn remove_body(&mut self, handle: RigidBodyHandle) {
        // A removed body is no longer a zone. Left behind, the entry would keep
        // querying a dead collider handle every substep — harmless today (the
        // lookup fails), and exactly the kind of stale table that stops being
        // harmless the moment the arena reuses the index.
        self.effectors.retain(|(h, _, _)| *h != handle);
        // A pulley whose body is gone has no branch to pull on; `branch` would
        // already answer `None`, but a table that keeps naming dead handles is
        // the stale table the comment above is about.
        self.pulleys
            .retain(|p| p.body_a != handle && p.body_b != handle);
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    /// Spawn a body explicitly (advanced — used when you need a
    /// non-circle / non-cuboid shape or non-default body type).
    pub fn insert_body(&mut self, body: RigidBody) -> RigidBodyHandle {
        let handle = self.bodies.insert(body);
        self.stamp_defaults(handle);
        handle
    }

    pub fn bodies(&self) -> &RigidBodySet {
        &self.bodies
    }
    pub fn bodies_mut(&mut self) -> &mut RigidBodySet {
        &mut self.bodies
    }
    pub fn colliders(&self) -> &ColliderSet {
        &self.colliders
    }
    pub fn colliders_mut(&mut self) -> &mut ColliderSet {
        &mut self.colliders
    }

    /// Convenience: read the position + orientation of a body.
    pub fn body_pose(&self, handle: RigidBodyHandle) -> Option<Pose> {
        self.bodies.get(handle).map(|b| *b.position())
    }

    /// **A massa que o solver tem para este corpo**, em kg.
    ///
    /// ⚠️ **Ela responde para TODA espécie de corpo, o que não é óbvio:** um
    /// corpo cinemático tem massa INFINITA *efetiva* (o rapier zera a
    /// inversa-massa), mas `mass()` continua a devolver a massa dos colliders —
    /// medido em `1,0000` para Dynamic, Kinematic e Fixed no doc do `fluid_at`,
    /// que é quem primeiro precisou do fato. É isso que deixa o estouro converter
    /// impulso em velocidade para um personagem de pose própria com a MESMA lei
    /// que usa para um caixote (`W-Launch`).
    #[must_use]
    pub fn body_mass(&self, handle: RigidBodyHandle) -> f32 {
        self.bodies.get(handle).map_or(0.0, RigidBody::mass)
    }

    /// Iterate every dynamic body's snapshot, sorted by handle index
    /// for stable order across runs / OSes.
    pub fn body_snapshots(&self) -> Vec<BodySnapshot> {
        let mut out: Vec<BodySnapshot> = self
            .bodies
            .iter()
            .map(|(handle, body)| {
                let pos = body.position();
                let lin = body.linvel();
                BodySnapshot {
                    handle_index: handle.into_raw_parts().0,
                    x: pos.translation.x,
                    y: pos.translation.y,
                    rotation: pos.rotation.angle(),
                    linvel_x: lin.x,
                    linvel_y: lin.y,
                    angvel: body.angvel(),
                }
            })
            .collect();
        out.sort_by_key(|s| s.handle_index);
        out
    }

    /// blake3 digest over the sorted body snapshots — the C9 cross-OS
    /// gate in CI compares this hash across Linux / macOS / Windows.
    /// MUST be byte-identical or HR-5 is violated.
    pub fn deterministic_hash(&self) -> [u8; 32] {
        let snapshots = self.body_snapshots();
        let mut hasher = blake3::Hasher::new();
        for s in &snapshots {
            hasher.update(&s.handle_index.to_le_bytes());
            hasher.update(&s.x.to_bits().to_le_bytes());
            hasher.update(&s.y.to_bits().to_le_bytes());
            hasher.update(&s.rotation.to_bits().to_le_bytes());
            hasher.update(&s.linvel_x.to_bits().to_le_bytes());
            hasher.update(&s.linvel_y.to_bits().to_le_bytes());
            hasher.update(&s.angvel.to_bits().to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ⚠️ **Este âncora deixou de citar `nalgebra` na subida para a rapier 0.35**, e não por
// arrumação: a matemática da rapier passou a ser `glam`, e o `nalgebra` continua na árvore
// **só** para o código SIMD e os jacobianos de multibody dela — não é mais a nossa superfície.
// O que sobra é o que sempre foi o ponto: manter a superfície de tipos da rapier viva mesmo
// que a lib pare de a usar, sem que o "unused import" a apague.
#[allow(dead_code)]
fn _force_imports_alive() {
    let _ = std::mem::size_of::<crate::rmath::Pose>();
    let _ = std::mem::size_of::<RigidBodyType>();
}

#[cfg(test)]
mod tests;
