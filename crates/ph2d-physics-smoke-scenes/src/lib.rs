//! **As cenas de smoke da física que só povoam um `World`** (GEF02 onda 0, 09/11).
//!
//! Saíram da `ph2d-app-physics` (tecto de LOC) com o [`SceneCtx`]: o conjunto é FECHADO — nenhuma
//! destas cenas nomeia outra coisa da família. A `ph2d-app-physics` re-exporta tudo, e o roteador
//! (`ph2d_app_physics::smoke`) continua a ser a única fonte de QUE nível corre o quê.

/// O que uma cena pede à shell **depois** de povoar o mundo.
///
/// Tudo aqui é `Option`/`Vec` vazio por omissão: uma cena que não peça nada deixa
/// a shell exactamente como estava, e é isso que torna o registo seguro de passar
/// a **todas** as cenas, inclusive as 44 que só povoam o mundo.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct SceneSetup {
    /// Centro da câmera, em unidades de mundo.
    pub camera_center: Option<[f32; 2]>,
    /// Altura visível da câmera, em unidades de mundo (o «zoom» enquadrado).
    pub camera_height_world: Option<f32>,
    /// Painéis a abrir, pela chave que o `panel_visibility` da shell usa.
    ///
    /// ⚠️ A chave é `&'static str` **de propósito** — é o que a shell já usa, e
    /// mantê-la assim evita que esta crate tenha de conhecer o enum de painéis
    /// dela (que é exactamente a dependência que não pode existir).
    pub panels: Vec<&'static str>,
    /// A entidade que a cena quer deixar escolhida, em bits.
    ///
    /// ⚠️ Bits e não `Entity` porque o consumidor é o gizmo da shell, que guarda
    /// bits; e porque uma cena que escolhe alguém acabou de o criar, logo os bits
    /// são válidos no mesmo quadro.
    pub select: Option<u64>,
    /// Definições de mundo que a cena quer impor (gravidade, sub-passos, …).
    pub settings: Option<ph2d_physics_ecs::PhysicsSettings>,
    /// O laço do transporte que a cena quer armado, em segundos `(início, fim)`.
    ///
    /// ⚠️ `f64` porque é o que o `Playhead` guarda — estreitar para `f32` aqui
    /// poria uma conversão no meio de um pedido que é só dados.
    ///
    /// ⚠️ **Pedido e não escrita directa**: o `Playhead` é da shell, e uma cena
    /// que lhe tocasse a meio da construção decidiria a ordem em que as coisas
    /// acontecem — que é precisamente o que o roteador existe para decidir.
    pub playhead_loop: Option<(f64, f64)>,
    /// O corpo cujo *readout* de jogador a cena quer ver impresso, em bits.
    ///
    /// ⚠️ Isto era `self.physics.player_readout_log = Some(bits)` — um campo do
    /// `PhysicsState`, que vive na shell. Como pedido, a cena declara-o e o
    /// roteador escreve-o, e a cena deixa de precisar do estado da shell.
    pub player_readout_log: Option<u64>,
}

/// O contexto que o roteador passa a uma cena: o mundo para povoar, as definições
/// **vigentes** para quem quer mudar só uma, e o registo do que pedir à shell.
///
/// ⚠️ **As definições entram por VALOR e saem pelo `want`.** Uma cena que quisesse
/// lê-las por referência mutável estaria a escrever no mundo da shell a meio da
/// construção — e o roteador deixaria de poder decidir a ordem em que as coisas
/// se aplicam. Ler o que está lá e declarar o que se quer são dois actos, e o
/// segundo é reversível.
pub struct SceneCtx<'w> {
    /// O mundo ECS que a cena povoa.
    pub world: &'w mut bevy_ecs::world::World,
    /// As definições de física vigentes no momento em que a cena corre.
    pub settings: ph2d_physics_ecs::PhysicsSettings,
    /// O documento da timeline, para as cenas que AUTORAM uma track.
    ///
    /// ⚠️⚠️ **Esta é a única coisa que uma cena escreve DURANTE, e não por
    /// pedido** — e a razão é que uma track não é um facto sobre a cena, é
    /// conteúdo autorado: o que se guarda não caberia num `SceneSetup` sem o
    /// transformar num segundo formato de timeline.
    ///
    /// ⭐ **Ela só é alcançável porque a `ph2d-timeline` é uma crate-motor IRMÃ**
    /// (decisão do integrador, 11/09): não há ciclo — a `ph2d-timeline` não
    /// depende de nenhuma `ph2d-app-*` nem da shell. *Uma crate-motor irmã não é
    /// a shell*, que é o ADR-0075 a funcionar.
    pub timeline: &'w mut ph2d_timeline::TimelineDoc,
    /// O que a cena pede à shell. O roteador aplica-o depois de a cena correr.
    pub want: SceneSetup,
}

impl<'w> SceneCtx<'w> {
    /// Um contexto sobre `world`, com as definições vigentes e nada pedido ainda.
    pub fn new(
        world: &'w mut bevy_ecs::world::World,
        settings: ph2d_physics_ecs::PhysicsSettings,
        timeline: &'w mut ph2d_timeline::TimelineDoc,
    ) -> Self {
        Self {
            world,
            settings,
            timeline,
            want: SceneSetup::default(),
        }
    }

    /// Acesso directo ao mundo — o que a esmagadora maioria das cenas usa.
    pub fn world(&mut self) -> &mut bevy_ecs::world::World {
        self.world
    }
}

// As cenas, ORDENADAS.
pub mod physics_smoke_authoring;
pub mod physics_smoke_base;
pub mod physics_smoke_blast;
pub mod physics_smoke_brake;
pub mod physics_smoke_brink;
pub mod physics_smoke_collider;
pub mod physics_smoke_collision;
pub mod physics_smoke_compound;
pub mod physics_smoke_contacts;
pub mod physics_smoke_damping;
pub mod physics_smoke_events;
pub mod physics_smoke_fk;
pub mod physics_smoke_foot;
pub mod physics_smoke_foot_fan;
pub mod physics_smoke_glide;
pub mod physics_smoke_grab;
pub mod physics_smoke_grow;
pub mod physics_smoke_ik;
pub mod physics_smoke_interact;
pub mod physics_smoke_joint_break;
pub mod physics_smoke_joint_motor;
pub mod physics_smoke_kin_pure;
pub mod physics_smoke_kin_push;
pub mod physics_smoke_kin_water;
pub mod physics_smoke_lead;
pub mod physics_smoke_leave;
pub mod physics_smoke_player_bake;
pub mod physics_smoke_player_carry;
pub mod physics_smoke_player_crouch;
pub mod physics_smoke_player_dash;
pub mod physics_smoke_player_drop;
pub mod physics_smoke_player_flank;
pub mod physics_smoke_player_grab;
pub mod physics_smoke_player_run;
pub mod physics_smoke_player_slope;
pub mod physics_smoke_player_wall;
pub mod physics_smoke_probes;
pub mod physics_smoke_props;
pub mod physics_smoke_raft;
pub mod physics_smoke_rail_rope;
pub mod physics_smoke_rigs;
pub mod physics_smoke_rod;
pub mod physics_smoke_signal;
pub mod physics_smoke_signal_ends;
pub mod physics_smoke_signal_leave;
pub mod physics_smoke_soft_weld;
pub mod physics_smoke_stone;
pub mod physics_smoke_surface;
pub mod physics_smoke_swim;
pub mod physics_smoke_terminal;
pub mod physics_smoke_water;
pub mod physics_smoke_wheel;
pub mod physics_smoke_world_pin;
pub mod physics_smoke_zone_force;
pub mod physics_smoke_zones;
