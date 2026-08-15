//! [`PhysicsBridge`] — the per-frame seam between the ECS and rapier.
//!
//! Owns the transient [`PhysicsWorld`] + the `Entity -> body handle` map.
//! **Not serialized** (ADR-0131 D2): the live world is *derived* from the
//! `RigidBody`/`Collider` components each frame, exactly like Motion
//! re-cooks from its graph. The components are the truth-at-rest; the
//! world is the truth-in-motion; the pose flows into `Transform`.
//!
//! The shell owns one of these on `AppGfx.physics` and calls [`dispatch`]
//! once per frame (mirroring `motion_bridge`). It is a plain struct with
//! free-standing methods so the whole thing is drivable **headless** — the
//! `SimWorld` needs no window, so the W1 e2e gate is a unit test.
//!
//! [`dispatch`]: PhysicsBridge::dispatch

pub mod anchors;
pub mod bodies;
pub mod contacts;
mod damping;
mod diagnostics;
pub mod fk;
mod grab;
mod hold;
pub mod ik;
mod ik_lead;
mod inspect;
pub mod joint_break;
pub mod joint_desc;
mod joint_drive;
mod joint_respawn;
pub mod joints;
mod kinematic;
/// A TRADUÇÃO `PhysicsJoint` → `JointDesc` — irmão do `joints` pelo cap de 700
/// LOC, cortado por responsabilidade (docs dele).
/// A metade PEÇA do reconcile — os colliders extra de um corpo composto.
pub mod parts;
mod player;
/// O tipo do empurrão de fora (`W-Launch`) — mora no canal do player, e sobe
/// por aqui porque o módulo dono é privado.
pub use player::channel::Launch;
/// A SAÍDA do player — o readout contínuo e o canal de transições
/// (`W-PlayerOut`). Módulo irmão do `player`, cortado por RESPONSABILIDADE: ali
/// mora *o que a lei decide num tique*, aqui *o que o resto do app pode ler*.
pub mod player_out;
pub mod player_view;
mod pose_owner;
mod readback;
mod rewind;
pub mod rope;
/// As settings de MUNDO — módulo irmão pelo cap de 700 LOC.
mod settings;
pub mod signals;
mod space;
mod surfaces;
pub mod triggers;
pub mod views;

pub use kinematic::{FrozenScene, SceneAtTick};

mod dispatch;
mod tape;
pub use tape::{HeldInput, InputTape, PlayerInputAtTick, TapeWire};

use std::collections::BTreeMap;

use bevy_ecs::query::QueryState;
use parts as bridge_parts;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics::{BodyDesc, PhysicsCheckpointRing, PhysicsWorld, RigidBodyHandle};

use crate::joint::PhysicsJoint;
use joints::JointRef;

use crate::settings::PhysicsSettings;

use crate::components::{BodyKind, Collider, RigidBody};

/// The query the bridge iterates each frame. Cached (built once) because
/// `World::query` allocates a fresh `QueryState` per call — the cached
/// state is the zero-alloc idiom (HR-3), mirroring `TransformPropagationState`.
type BodyQuery = QueryState<(
    Entity,
    &'static RigidBody,
    &'static Collider,
    &'static Transform,
)>;

/// A joint waiting to be handed to the solver: which entity authored it, what
/// it is, the two body handles, and the two body ENTITIES. The entities travel
/// alongside the handles because a rewind hands out fresh handles and the only
/// durable way back to a body is its entity.
///
/// ⚠️ **O último campo é o PONTO DE MUNDO** com que este joint se prende quando
/// o lado B é o cenário (W-JointWorld) — `None` num joint corpo↔corpo. Ele viaja
/// aqui em vez de um handle porque a âncora **ainda não existe** no momento do
/// reconcile: ela é criada no flush, junto com o joint que a usa, e some com ele.
/// ⚠️ **O handle do lado B é `Option` pela MESMA razão** — num pino de mundo ele
/// ainda não existe: `None` aqui significa *"o flush cria a âncora em
/// `world_point`"*. A alternativa era um handle-fantasma que ninguém lê, e é
/// exatamente o tipo de valor que o próximo leitor acredita.
type PendingJoint = (
    Entity,
    ph2d_physics::JointDesc,
    (RigidBodyHandle, Option<RigidBodyHandle>),
    (Entity, Option<Entity>),
    Option<[f32; 2]>,
);

/// **As PEÇAS** (W-Compound): quem carrega `Collider` e **não** `RigidBody`. O
/// `Without` é a definição inteira — uma entidade com os dois é um CORPO, e a
/// `BodyQuery` já a pegou.
type PartQuery = QueryState<
    (Entity, &'static Collider, &'static Transform),
    bevy_ecs::query::Without<RigidBody>,
>;

/// A query das SUPERFÍCIES (`W-Surface`) — quem carrega uma, seja corpo ou
/// peça. Cacheada pelo mesmo motivo zero-alloc das irmãs.
type SurfaceQuery = QueryState<(Entity, &'static crate::WalkSurface)>;

/// The joint query, cached for the same zero-alloc reason as [`BodyQuery`].
/// The `Transform` is the anchor — see `bridge::joints` for the policy.
type JointQuery = QueryState<(Entity, &'static PhysicsJoint, &'static Transform)>;

/// A live rapier body owned by the bridge, keyed by its ECS entity.
#[derive(Copy, Clone)]
pub(super) struct BodyRef {
    pub(super) handle: RigidBodyHandle,
    /// Cached so `readback` knows which bodies to read (only dynamic ones
    /// move; static bodies keep their authored pose).
    kind: BodyKind,
    /// The spawn description captured when this body was FIRST created —
    /// i.e. the **rest state at tick 0**. rapier cannot rewind, so a
    /// backwards clock replays from here (see
    /// [`PhysicsBridge::rewind_to`]). Without it, Reset could not put the
    /// ball back where it started: the live `Transform` has already been
    /// overwritten by the readback.
    pub(super) rest: BodyDesc,
}

/// O semeio de um joint: a entidade, as duas âncoras body-local e — só para uma
/// polia — o comprimento da corda, que sai da ROTA que as roldanas desenham.
type JointSeed = (Entity, [f32; 2], [f32; 2], Option<f32>);

/// The ECS ↔ rapier bridge. One per document, held on `AppGfx.physics`.
pub struct PhysicsBridge {
    world: PhysicsWorld,
    /// `BTreeMap`, not `HashMap`, on purpose: iteration is by `Entity`
    /// order — deterministic per run and cross-OS (entity allocation is
    /// sequential) — so the sim's body order and the hash never depend on a
    /// randomised hasher seed (HR-5). The repo's disallowed-`HashMap` lint
    /// is the standing structural guard for this.
    bodies: BTreeMap<Entity, BodyRef>,
    /// Last fixed tick the world was stepped **to** — the physics analog
    /// of the Motion pump's `last_cooked_tick`. The play/scrub decision in
    /// [`dispatch`](PhysicsBridge::dispatch) reads it.
    last_stepped: u64,
    /// Built lazily on first dispatch (needs `&mut World`); `None` after
    /// [`rebuild`](PhysicsBridge::rebuild) so it re-binds to the world.
    query: Option<BodyQuery>,
    /// The joints this bridge holds, keyed by the entity that authors each one.
    /// `BTreeMap` for the determinism reason `bodies` documents.
    joints: BTreeMap<Entity, JointRef>,
    /// A sessão de POSE viva (W-IK), ou `None` fora de um gesto de IK.
    ///
    /// Fora do checkpoint e fora de tudo que é persistido, de propósito: uma
    /// árvore de multibody é **ferramenta**, não estado da cena — ela nasce num
    /// press e morre num release, e o que sobrevive ao gesto é o `Transform`
    /// autorado que o chamador escreveu.
    pub(super) ik: Option<ik::IkSession>,
    /// A sessão de CINEMÁTICA DIRETA viva (W-FK), ou `None` fora do gesto.
    ///
    /// Irmã do `ik` e, como ela, fora do checkpoint e de tudo que é persistido:
    /// um gesto de pose é **ferramenta**, e o que sobrevive a ele é o
    /// `Transform` autorado que o chamador escreveu.
    pub(super) fk: Option<fk::FkSession>,
    joint_query: Option<JointQuery>,
    /// W-Compound: a query das peças e a tabela delas.
    part_query: Option<PartQuery>,
    /// W-Surface: a query das superfícies e a tabela delas — ver
    /// `bridge::surfaces` para por que ela é reconstruída a cada dispatch.
    surface_query: Option<SurfaceQuery>,
    surfaces: surfaces::Surfaces,
    parts: std::collections::BTreeMap<Entity, bridge_parts::PartRef>,
    /// Scratch da varredura de peças — zero-alloc em regime, como o `seen`.
    part_seen: Vec<Entity>,
    /// A query das roldanas. Separada da dos joints porque uma roldana é uma
    /// ENTIDADE própria (W-Pulley W1) — a corda a aponta pelo nome.
    wheel_query: Option<rope::WheelQuery>,
    // Reusable scratch — cleared+refilled each frame so the steady-state
    // hot path never reallocates (HR-3; proven by the capacity gate).
    /// Entities carrying physics components this frame (for stale detection).
    seen: Vec<Entity>,
    /// Name-hash → body entity, rebuilt each frame a joint needs resolving.
    names: BTreeMap<u64, Entity>,
    joints_seen: Vec<Entity>,
    joints_to_spawn: Vec<PendingJoint>,
    joints_to_remove: Vec<Entity>,
    /// **Joints que só mudaram de NÚMERO** — os que ganham o `data` reescrito no
    /// lugar em vez de remover-e-inserir (`PhysicsWorld::retune_joint`).
    ///
    /// ⚠️ A separação é o que salva o ring: uma mudança estrutural o invalida
    /// (um checkpoint indexa as arenas), e antes disto **todo edit de parâmetro
    /// contava como estrutural** — arrastar um slider limpava o cache de scrub a
    /// cada quadro, e um param keyframado o limparia a cada TICK. Scratch:
    /// limpo por reconcile.
    joints_to_retune: Vec<(Entity, ph2d_physics::JointDesc)>,
    /// Joints whose body-local anchors were just derived from their world
    /// `Transform` (the seed) — written back after the query borrow releases so
    /// a body move never re-derives them again. Scratch: cleared per reconcile.
    /// O terceiro elemento é o semeio de POLIA — as duas roldanas e o
    /// comprimento da corda — presente só para uma [`crate::JointKind::Pulley`]
    /// que ainda não foi ancorada. Viaja no mesmo scratch das âncoras porque é o
    /// MESMO sentinela (`anchored`) que governa os dois: uma polia com roldanas
    /// em `[0,0]` é uma que nunca foi semeada, e o semeio acontece uma vez, das
    /// poses de REPOUSO.
    joints_to_seed: Vec<JointSeed>,
    /// **As roldanas cujo eixo local acaba de ser semeado** (W-Pulley W3), com o
    /// local derivado da pose de REPOUSO do corpo em que estão montadas.
    ///
    /// Scratch, limpo a cada colheita. Irmão exato do `joints_to_seed` e pelo
    /// MESMO motivo: a conversão mundo→local acontece uma vez e é persistida,
    /// senão um move de corpo re-derivaria e o eixo deslizaria pelo bloco.
    wheels_to_seed: Vec<(Entity, [f32; 2])>,
    /// A tabela de polias a instalar neste dispatch, reconstruída inteira todo
    /// frame (uma polia não é dona de nada nas arenas do rapier, então não há
    /// diff de spawn/remove a fazer — e é por isso que ela nunca invalida o ring
    /// de checkpoints).
    pulleys_to_install: Vec<ph2d_physics::world::pulley::PulleyDesc>,
    /// **A arena de roldanas** que as faixas de `pulleys_to_install` indexam —
    /// uma lista só para todas as cordas do mundo, pelo motivo escrito no
    /// `PulleyDesc`: um `Vec` por corda alocaria por frame.
    pulley_wheels_to_install: Vec<ph2d_physics::world::rope_route::RopeWheel>,
    /// As roldanas colhidas do mundo neste dispatch, ORDENADAS por
    /// `(corda, order, desempate por nome)` — a chave inteira é estável através
    /// do undo, ao contrário dos bits de entidade, que são id de ALOCAÇÃO.
    ///
    /// Scratch: limpa e repreenchida por dispatch, então o regime não realoca.
    rope_wheels: Vec<rope::RopeWheelRow>,
    /// **O ângulo de cada roldana**, radianos, na MESMA ordem da arena.
    ///
    /// ⚠️ **Estado VIVO, e é por isso que ele mora aqui e não no componente**: um
    /// ângulo serializado seria um passo de undo por frame (o `canonicalize`
    /// ordena pelas bytes do componente), a mesma razão pela qual velocidade e
    /// sono nunca entraram no `RigidBody`. Um replay o reintegra sozinho.
    wheel_spin: Vec<f32>,
    /// O ângulo por ENTIDADE, para ele sobreviver ao reconcile — a arena é
    /// reconstruída todo dispatch, e sem esta memória toda roda voltaria a zero a
    /// cada frame.
    wheel_spin_by_entity: std::collections::BTreeMap<Entity, f32>,
    /// A entidade de cada roldana da arena, na MESMA ordem — é por ela que o
    /// desenho e as alças sabem QUEM é a roda sob o cursor. Paralela em vez de um
    /// campo dentro do `RopeWheel` porque aquele tipo é do motor, que não conhece
    /// entidade nenhuma (e não deve conhecer).
    wheel_entities: Vec<Entity>,
    /// **O que o artista escolheu sobre o LADO**, na MESMA ordem da arena.
    ///
    /// ⚠️ Paralela e não lida direto das linhas colhidas porque a arena passa por
    /// um FILTRO que elas não passam: um eixo que cedeu não é instalado. Enquanto o
    /// override era lido por `zip` contra as linhas cruas, o primeiro eixo partido
    /// de uma corda **deslocava** todos os overrides seguintes para a roldana
    /// vizinha — o defeito nascia só depois de uma ruptura, e só com um abraço
    /// autorado, que é por que ele viveu calado. O override tem de andar pelo MESMO
    /// filtro que a roda que ele governa.
    wheel_wraps: Vec<crate::WrapSide>,
    /// Os trechos que a resolução de LADO escreve. Scratch pelo mesmo motivo.
    route_scratch: Vec<ph2d_physics::world::rope_route::Tangent>,
    /// **A lista de polias VIVAS**, reconstruída todo reconcile — a fonte única
    /// de que tanto a tabela do solver quanto as views de desenho saem. Uma
    /// polia não vive no `ImpulseJointSet`, então sem este registro ela seria
    /// invisível na tela.
    pulley_records: Vec<views::PulleyRecord>,
    /// (joint, derived world pivot) pairs — `sync_joint_pivots` writes each into
    /// the joint's `Transform.translation` so the anchor dot follows the body.
    /// Scratch: taken out and refilled per sync, so the steady state never allocs.
    joints_to_sync: Vec<(Entity, [f32; 2])>,
    /// Where each kinematic body stood when this dispatch began, so a
    /// multi-tick dispatch can spread its move across the ticks it owes
    /// instead of teleporting it on the first one. Scratch: cleared and
    /// refilled per dispatch, so the steady state never reallocates.
    kin_start: Vec<(Entity, [f32; 3])>,
    /// Ancestor chain buffer for `space::world_transform` / `write_world_pose`.
    /// Persistent so the per-body, per-frame conversion allocates nothing
    /// (HR-3; `hot_path_no_alloc` is the gate).
    pub(super) chain: Vec<Transform>,
    /// A ordem em que o readback escreve: `(profundidade, entidade, x, y, rot)`.
    ///
    /// ⚠️ **Ancestral ANTES de descendente, e isso é correção, não arrumação.**
    /// `Transform` é LOCAL, então o readback converte a pose de MUNDO do solver
    /// contra o pai VIGENTE; escrito antes do pai, o filho é convertido contra uma
    /// pose que o mesmo passe vai substituir em seguida. O mapa de corpos é um
    /// `BTreeMap<Entity>` e a ordem de `Entity` **não** é a de spawn — medido, um
    /// par pai/filho itera **o filho primeiro**. Scratch: limpo a cada readback.
    pub(super) readback_order: Vec<(u32, Entity, f32, f32, f32)>,
    /// New entities to spawn a body for, this frame.
    to_spawn: Vec<(Entity, BodyDesc, BodyKind)>,
    /// Entities whose body must be removed this frame (component gone).
    to_remove: Vec<Entity>,
    /// The bodies a settle pass is about to walk. Collected first because
    /// `follow_authored_pose` borrows the `chain` scratch mutably while
    /// `self.bodies` is being iterated; retained for the same zero-alloc reason
    /// as every other buffer here.
    to_settle: Vec<(Entity, RigidBodyHandle)>,
    /// The world's authored settings, kept so a rewind can rebuild a fresh
    /// world that still has them: `PhysicsWorld::new` starts from the engine
    /// defaults, so anything not carried here is **silently reset** by a scrub
    /// backwards. (This field used to be gravity alone, which is exactly that
    /// bug scoped to nine other knobs.)
    settings: PhysicsSettings,
    /// Sparse cache of past states, so scrubbing backwards replays at most
    /// `STRIDE` steps instead of `target` of them (W1.5). Purely an
    /// accelerator: on a miss the rest-pose rebuild below runs, which is the
    /// path that shipped in W1.
    ring: PhysicsCheckpointRing,
    /// Every `world.step()` this bridge has ever run. The scrub gate is a
    /// COUNT, not a stopwatch: the claim being defended is "a scrub replays
    /// at most `STRIDE` steps regardless of how far in it lands", and steps
    /// are exactly that quantity — deterministic, and immune to the machine
    /// the test happens to run on.
    steps_taken: u64,
    /// **Trigger state** (W7): each sensor entity → the entities inside it,
    /// rebuilt every dispatch (`bridge::triggers`). Empty without sensors, so a
    /// non-trigger scene pays nothing. `BTreeMap` for the determinism reason
    /// `bodies` documents.
    triggers: BTreeMap<Entity, Vec<Entity>>,
    /// O conjunto do dispatch ANTERIOR — a baseline de que `trigger_events`
    /// sai (W-Signal). Irmão de `contact_since`.
    trigger_since: BTreeMap<Entity, Vec<Entity>>,
    /// Quem ENTROU num sensor neste dispatch, drenado pelo shell.
    trigger_events: Vec<triggers::TriggerEvent>,
    /// Quem SAIU de um sensor neste dispatch (W-SignalLeave) — o espelho do
    /// campo acima, preenchido pelo MESMO diff.
    trigger_exits: Vec<triggers::TriggerEvent>,
    /// `trigger_since` descreve o dispatch imediatamente anterior? Falso
    /// depois de toda descontinuidade do relógio — ver `diff_trigger_entries`.
    triggers_continuous: bool,
    /// The pairs touching this frame (`bridge::contacts`). A flat list, not a map:
    /// a contact is a RELATIONSHIP with no owner, unlike a trigger, which is asked
    /// about one sensor. Cleared and refilled per dispatch; empty in free fall.
    contacts: Vec<contacts::BodyContact>,
    /// What was touching at the END of the previous TICK, with where and how hard it
    /// last hit — the memory that turns a per-tick set into TRANSITIONS
    /// (`bridge::contacts`). `BTreeMap` for the determinism reason `bodies`
    /// documents; the key order is the order `Ended` events come out in.
    contact_since: BTreeMap<(Entity, Entity), contacts::ContactMemo>,
    /// The transitions of this dispatch — cleared at its start, appended to per tick.
    contact_events: Vec<contacts::ContactEvent>,
    /// The joints that gave way in this dispatch (`bridge::joint_break`). Same
    /// shape and the same lifetime as `contact_events`, and for the same reason:
    /// a break is a transition, and a dispatch can owe several ticks.
    joint_breaks: Vec<joint_break::JointBreakEvent>,
    /// The hardest each joint has been pulled since the clock last restarted
    /// (`bridge::joint_break`) — the number a break threshold is TUNED against.
    /// Unlike `joint_breaks` this is not per-dispatch: it is the memory of the run.
    joint_peaks: BTreeMap<Entity, ph2d_physics::JointLoad>,
    /// O mesmo, para as CORDAS (W-Pulley W2): elas não vivem no `joints`, então
    /// o laço de cima não passa por elas. Newtons; só a linear, porque uma corda
    /// não transmite torque.
    pulley_peaks: BTreeMap<Entity, f32>,
    /// The live begin-flashes (`bridge::contacts`) — the visible half. Seeded from
    /// `Began` transitions and decayed in SIM ticks; PERSISTS across dispatches (a
    /// flash outlives the tick it was born in), so it is not cleared per dispatch, only
    /// aged and dropped, or cleared by a discontinuity.
    flashes: Vec<contacts::ContactFlash>,
    /// Whether `contact_since` describes the tick immediately before this one.
    ///
    /// False after any discontinuous clock move, which makes the next forward tick adopt
    /// the set in SILENCE instead of reporting a scrub as a hundred collisions. Starts
    /// TRUE over an empty baseline, so the first stepped tick does report what it finds
    /// — the Unity reading, argued in `accumulate_contact_events`.
    contacts_continuous: bool,
    /// **O que o dedo do jogador está fazendo**, por player (W3).
    ///
    /// ⚠️ Fica AQUI e não no componente por lei do módulo: entrada não é config,
    /// e um campo que muda por tick dentro de um componente faria o
    /// `canonicalize` do undo ver cada frame como um passo (o mesmo motivo pelo
    /// qual velocidade e sono nunca entraram no `RigidBody`).
    ///
    /// ⚠️ **Set-and-hold, e é uma escolha de ESCOPO, não o desenho final:** a
    /// shell escreve a cada frame, e a ponte lê. A W7 troca a FONTE por uma fita
    /// por tick — o player volta a ser função de `(tick, fita)` e o scrub volta a
    /// ser bit-exato —, sem que a lei precise mudar uma linha. Até lá, um scrub
    /// para trás replaya com a entrada de HOJE, e essa é a dívida nomeada.
    ///
    /// `BTreeMap` pelo motivo determinístico que o `bodies` documenta.
    player_input: BTreeMap<Entity, ph2d_platformer::PlayerInput>,
    /// **O EMPURRÃO DE FORA que ainda não foi entregue** (`W-Launch`).
    ///
    /// ⚠️ **Mapa PRÓPRIO, e não um campo do `player_input`, porque a semântica é
    /// oposta:** a entrada do dedo é *set-and-hold* (um dispatch que deve vários
    /// tiques aplica a MESMA a todos eles, que é o que segurar uma tecla quer
    /// dizer), e um empurrão é um **evento**. Guardado ali, um dispatch que
    /// devesse quatro tiques entregaria o empurrão **quatro vezes**.
    ///
    /// Drenado pelo `drive_players` no primeiro tique que o vê — `BTreeMap` pelo
    /// motivo determinístico que o `bodies` documenta.
    player_launch: BTreeMap<Entity, player::channel::Launch>,
    /// **Os estados de pulo nos tiques ÂNCORA** (W7) — o irmão do `ring`.
    ///
    /// Ver `bridge::tape`: um seed do ring devolve o mundo do tique T e este
    /// mapa devolve a memória do controlador do MESMO tique. Sem ele a resposta
    /// para um tick dependeria de o cache ter o âncora.
    state_ring: tape::PlayerStateRing,
    /// **O estado VIVO do controlador**, por player (W4/W14) — a fase do pulo, a
    /// borda dos botões e o arranque.
    ///
    /// ⚠️ **Um mapa, não um por assunto**, e a razão é o ring da fita: este
    /// valor é o que o `record_player_states` guarda por tique âncora, então um
    /// segundo mapa teria de ser acrescentado àquele ring à mão — e esquecê-lo é
    /// um scrub que devolve o mundo de um tique e a memória do controlador de
    /// outro, sem erro e sem aviso (ver [`ph2d_platformer::PlayerState`]).
    ///
    /// ⚠️ Aqui e não no componente pela lei do módulo: um campo que muda por
    /// tick faria o `canonicalize` do undo ver cada frame como um passo. A W7 o
    /// torna DERIVADO da fita, e aí ele deixa de ser guardado.
    player_state: BTreeMap<Entity, ph2d_platformer::PlayerState>,
    /// **A plataforma que cada player está ATRAVESSANDO agora** (W12).
    ///
    /// ⚠️ **Uma forma, não um relógio, e não "todas as one-way":** a descida
    /// mira a plataforma em que o personagem estava, e só ela — é isso que faz
    /// duas plataformas empilhadas se comportarem como o artista espera (ele
    /// desce da de cima e **pousa** na de baixo, em vez de atravessar as duas).
    ///
    /// ⚠️ E é `ColliderHandle`, não corpo: a W-Compound deu a um corpo várias
    /// formas, então um cenário pode ser UM corpo estático com dez plataformas.
    /// Guardar o corpo faria descer de uma delas dissolver as outras nove.
    player_drop: BTreeMap<Entity, ph2d_physics::ColliderHandle>,

    /// **O que os sensores de cada player olharam no último tique** (`W-Probes`)
    /// — a leitura que o overlay desenha.
    ///
    /// ⚠️ **Um `Vec`, e reusado**: a lista é reconstruída em todo tique de player
    /// (`drive_players` limpa e reenche), então o `Vec` guarda a capacidade e o
    /// caminho quente não aloca. `BTreeMap` seria a estrutura errada aqui — não
    /// há nada a procurar por entidade, o consumidor desenha a lista inteira, e
    /// a ORDEM já é determinística porque o laço que a enche itera o
    /// `BTreeMap<Entity>` dos corpos.
    ///
    /// ⚠️ **Ela é LIMPA quando a física fica em `hold`** (o toggle Physics
    /// desmarcado), pela razão exacta que limpa os contatos: sem passo não há
    /// sensor, e marcas de um tique que já não corre descreveriam um mundo que o
    /// artista pode desmontar com a mão.
    player_probes: Vec<player_view::ProbeMark>,

    /// **De que CORPO é cada pedaço dessa lista, e de onde ele foi lançado** —
    /// o que [`PhysicsBridge::reanchor_player_probes`] precisa para pousar as
    /// marcas onde o corpo ESTÁ.
    ///
    /// ⚠️ **Não é atribuição por gosto: é a âncora.** A leitura é gravada
    /// ANTES do `step` (é ela que a lei consome para decidir o tique), e o
    /// `readback` publica a pose DEPOIS — então a marca descreve a origem de um
    /// tique atrás do corpo que o artista vê. Sem esta faixa não há como
    /// perguntar *"quanto este corpo andou desde que o sensor olhou?"*, porque
    /// a lista publicada é plana e uma cena pode ter vários players.
    ///
    /// `(corpo, início, fim, origem do cast)`. Reusado como o `player_probes`,
    /// e pela mesma razão.
    player_probe_anchors: Vec<(RigidBodyHandle, usize, usize, [f32; 2])>,

    /// **O que cada player ESTAVA a fazer no fim do último tique** — o readout
    /// contínuo (`W-PlayerOut`, ver `bridge::player_out`).
    ///
    /// ⚠️ **Ele é também a MEMÓRIA de que o canal de eventos precisa:** uma
    /// transição é a diferença entre duas vistas consecutivas, e é por esta
    /// tabela ser reescrita **por TIQUE** — e não por quadro — que um dispatch
    /// que deve três tiques não perde o pulo do meio.
    ///
    /// ⚠️ **Ela é LIMPA nas duas descontinuidades** (scrub e `hold`), pela razão
    /// exacta que limpa `player_probes` e os contatos: sem passo não há lei, e
    /// publicar *"no chão, a 4 m/s"* com a física desarmada seria descrever uma
    /// corrida que acabou.
    player_views: BTreeMap<Entity, ph2d_platformer::PlayerView>,

    /// **As TRANSIÇÕES deste dispatch** — mesma forma e mesmo tempo de vida que
    /// `contact_events`, e pela mesma razão: um evento é uma borda, e um
    /// dispatch pode dever vários tiques.
    player_events: Vec<(Entity, ph2d_platformer::PlayerEvent)>,
}

impl Default for PhysicsBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsBridge {
    pub fn new() -> Self {
        Self {
            world: PhysicsWorld::new(),
            player_probes: Vec::new(),
            player_views: BTreeMap::new(),
            player_events: Vec::new(),
            player_probe_anchors: Vec::new(),
            bodies: BTreeMap::new(),
            last_stepped: 0,
            query: None,
            joints: BTreeMap::new(),
            ik: None,
            fk: None,
            joint_query: None,
            part_query: None,
            surface_query: None,
            surfaces: surfaces::Surfaces::default(),
            parts: std::collections::BTreeMap::new(),
            part_seen: Vec::new(),
            wheel_query: None,
            seen: Vec::new(),
            names: BTreeMap::new(),
            joints_seen: Vec::new(),
            joints_to_spawn: Vec::new(),
            joints_to_remove: Vec::new(),
            joints_to_retune: Vec::new(),
            joints_to_seed: Vec::new(),
            wheels_to_seed: Vec::new(),
            pulleys_to_install: Vec::new(),
            pulley_wheels_to_install: Vec::new(),
            rope_wheels: Vec::new(),
            wheel_entities: Vec::new(),
            wheel_wraps: Vec::new(),
            wheel_spin: Vec::new(),
            wheel_spin_by_entity: std::collections::BTreeMap::new(),
            route_scratch: Vec::new(),
            pulley_records: Vec::new(),
            joints_to_sync: Vec::new(),
            kin_start: Vec::new(),
            chain: Vec::new(),
            readback_order: Vec::new(),
            to_spawn: Vec::new(),
            to_remove: Vec::new(),
            to_settle: Vec::new(),
            settings: PhysicsSettings::default(),
            ring: PhysicsCheckpointRing::new(),
            steps_taken: 0,
            triggers: BTreeMap::new(),
            trigger_since: BTreeMap::new(),
            trigger_events: Vec::new(),
            trigger_exits: Vec::new(),
            triggers_continuous: true,
            contacts: Vec::new(),
            contact_since: BTreeMap::new(),
            contact_events: Vec::new(),
            joint_breaks: Vec::new(),
            joint_peaks: BTreeMap::new(),
            pulley_peaks: BTreeMap::new(),
            flashes: Vec::new(),
            contacts_continuous: true,
            player_input: BTreeMap::new(),
            player_launch: BTreeMap::new(),
            state_ring: BTreeMap::new(),
            player_state: BTreeMap::new(),
            player_drop: BTreeMap::new(),
        }
    }

    /// Throw away the derived world. Call on project load / undo restore —
    /// entity bits are recycled there, so the handle map dangles. The world
    /// is rebuilt from components on the next [`dispatch`](Self::dispatch)
    /// (runtime-truth: the components are the truth, the world is derived).
    pub fn rebuild(&mut self) {
        self.world = PhysicsWorld::new();
        // A fresh world starts from the ENGINE defaults, not this document's
        // settings — re-push them or a project load quietly reverts every knob.
        self.settings.apply_to(&mut self.world);
        self.bodies.clear();
        self.joints.clear();
        self.last_stepped = 0;
        self.query = None; // re-bind to the (possibly fresh) world
        self.joint_query = None;
        self.wheel_query = None;
        self.ring.clear(); // cached states belong to the document being left
        // Entity bits are recycled here, so a held input would start driving
        // SOMEONE ELSE — the same trap that made joint anchors travel by NAME.
        self.clear_player_input();
    }
}
