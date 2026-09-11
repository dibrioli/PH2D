//! **A metade da família `physics` que não precisa da shell** (W2/L2).
//!
//! A auditoria de velocidade de 2026-09-10 (§4-C2) mediu o mecanismo: o ADR-0075
//! põe cada feature numa drop-crate, e **a metade da feature que fala com a `App`
//! foi ficando na shell** — 22,8 k linhas de física entre elas. Resultado: a crate
//! que TODO módulo recompila é a maior do repo (493 k linhas, 34–45 s sozinha no
//! fim de cada gate), e uma cena de física que só sabe povoar um `World` estava
//! presa lá dentro por uma razão só: ela era um **método de `App`**.
//!
//! Esta crate é o outro lado. O que vive aqui depende de `ph2d-core`, `ph2d-ecs`,
//! `ph2d-physics-ecs` e `ph2d-render` — e **de mais nada**. ⛔ Em particular, não
//! depende da shell: o `shells/desktop` é um `bin`, logo não pode ser dependência
//! de ninguém, e é essa direcção de seta que torna a extracção possível.
//!
//! ## O que uma cena PEDE à shell é DADO, nunca uma chamada
//!
//! Uma cena de smoke faz duas coisas: povoa um `World` (que é desta crate) e pede
//! à shell que enquadre a câmera, abra um painel ou escolha um objecto (que é da
//! shell). A segunda metade viaja como **dados** num [`SceneSetup`] que o roteador
//! aplica depois de a cena correr — ADR-0075 ao pé da letra: *components +
//! events/resources, systems não se chamam*.
//!
//! ⚠️ **Porque não um trait `SceneHost` com métodos:** um trait obrigaria esta
//! crate a declarar a superfície da shell (câmera, painéis, gizmo) e a shell a
//! implementá-la — o acoplamento mudava de forma, não de tamanho. Um registo de
//! intenções é inspeccionável, testável sem janela, e **não cresce** quando a
//! shell muda por dentro. É também o que deixa um gate perguntar *«esta cena pede
//! o painel certo?»* sem GPU nenhuma.

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
    /// O que a cena pede à shell. O roteador aplica-o depois de a cena correr.
    pub want: SceneSetup,
}

impl<'w> SceneCtx<'w> {
    /// Um contexto sobre `world`, com as definições vigentes e nada pedido ainda.
    pub fn new(
        world: &'w mut bevy_ecs::world::World,
        settings: ph2d_physics_ecs::PhysicsSettings,
    ) -> Self {
        Self {
            world,
            settings,
            want: SceneSetup::default(),
        }
    }

    /// Acesso directo ao mundo — o que a esmagadora maioria das cenas usa.
    pub fn world(&mut self) -> &mut bevy_ecs::world::World {
        self.world
    }
}

// ─────────────────────────────────────────────────────────────────────────
// As cenas. ⚠️ Esta lista é ORDENADA e gerada da pasta: o roteador da shell
// continua a ser a ÚNICA fonte de QUE nível corre o quê (CLAUDE.md §5.0) —
// aqui só se declara que o ficheiro existe.
// ─────────────────────────────────────────────────────────────────────────
pub mod common;
pub mod physics_smoke_authoring;
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
pub mod physics_smoke_ik;
pub mod physics_smoke_interact;
pub mod physics_smoke_joint_bake;
pub mod physics_smoke_joint_break;
pub mod physics_smoke_joint_copy;
pub mod physics_smoke_joint_custom;
pub mod physics_smoke_joint_draw;
pub mod physics_smoke_joint_glyphs;
pub mod physics_smoke_joint_handles;
pub mod physics_smoke_joint_motor;
pub mod physics_smoke_joint_pair;
pub mod physics_smoke_joint_pose;
pub mod physics_smoke_joint_rig;
pub mod physics_smoke_joint_slider;
pub mod physics_smoke_kin_pure;
pub mod physics_smoke_kin_push;
pub mod physics_smoke_kin_water;
pub mod physics_smoke_kinematic;
pub mod physics_smoke_lead;
pub mod physics_smoke_leave;
pub mod physics_smoke_ledge;
pub mod physics_smoke_multi_jump;
pub mod physics_smoke_player_bake;
pub mod physics_smoke_player_carry;
pub mod physics_smoke_player_crouch;
pub mod physics_smoke_player_dash;
pub mod physics_smoke_player_drop;
pub mod physics_smoke_player_flank;
pub mod physics_smoke_player_forgive;
pub mod physics_smoke_player_grab;
pub mod physics_smoke_player_run;
pub mod physics_smoke_player_slope;
pub mod physics_smoke_player_tape;
pub mod physics_smoke_player_wall;
pub mod physics_smoke_probes;
pub mod physics_smoke_props;
pub mod physics_smoke_pulley;
pub mod physics_smoke_pulley_break;
pub mod physics_smoke_pulley_weston;
pub mod physics_smoke_raft;
pub mod physics_smoke_rail_rope;
pub mod physics_smoke_rod;
pub mod physics_smoke_signal;
pub mod physics_smoke_signal_leave;
pub mod physics_smoke_soft_weld;
pub mod physics_smoke_stone;
pub mod physics_smoke_stop;
pub mod physics_smoke_surface;
pub mod physics_smoke_swim;
pub mod physics_smoke_terminal;
pub mod physics_smoke_water;
pub mod physics_smoke_wheel;
pub mod physics_smoke_world_pin;
pub mod physics_smoke_zone_force;
pub mod physics_smoke_zones;

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⚠️⚠️ **`routers: &[]` é a verdade MEDIDA da Fase A** — e por isso `"physics"` está na catraca
/// `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` do registo, que é o único sítio onde *«ainda não
/// saiu»* se distingue de *«alguém esqueceu»*.
///
/// ⭐ **E a distinção importa aqui mais que em qualquer outra família da rodada:** as **cenas**
/// saíram — são elas a maior parte dos 112 ficheiros desta crate —, mas o `env::var` que escolhe
/// qual delas montar ficou em [`physics_smoke`] na shell, porque ele toca a `App`. *Ter as cenas
/// não é ter o roteador*, e uma leitura rápida do tamanho desta crate concluiria o contrário.
///
/// ⇒ a entrada sai da catraca no dia em que a Fase B trouxer o `PH2D_PHYSICS_SMOKE` para cá.
///
/// [`physics_smoke`]: https://github.com/dibrioli/PH2D/blob/main/shells/desktop/src/physics/physics_smoke.rs
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "physics",
    routers: &[],
};

// ─────────────────────────────────────────────────────────────────────────
// **O que a Fase B trouxe de `shells/desktop/src/physics/`.**
// ─────────────────────────────────────────────────────────────────────────
pub mod anchor_side;
pub mod body_fk;
pub mod body_grab;
pub mod body_pose;
pub mod physics_smoke_part;
pub mod physics_smoke_pulley_comp;
pub mod physics_smoke_pulley_diff;
pub mod physics_smoke_pulley_tackle;
pub mod physics_smoke_rig;
pub mod run_stash;

// As três metades que vieram de `render_loop/`.
pub mod bridge;
pub mod inspector;
pub mod overlay;
