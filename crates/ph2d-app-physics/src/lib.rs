//! **A metade da família `physics` que não precisa da shell** (W2/L2).
//!
//! Depende das crates do módulo e do ECS, **nunca da shell** (auditoria de velocidade
//! 2026-09-10, §4-C2). O que uma cena pede à shell viaja como **dados** num [`SceneSetup`]
//! que o roteador aplica depois (ADR-0075: systems não se chamam) — inspeccionável e
//! testável sem janela, ao contrário de um trait `SceneHost`.

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

// ─────────────────────────────────────────────────────────────────────────
// As cenas. ⚠️ Esta lista é ORDENADA e gerada da pasta: o roteador da shell
// continua a ser a ÚNICA fonte de QUE nível corre o quê (CLAUDE.md §5.0) —
// aqui só se declara que o ficheiro existe.
// ─────────────────────────────────────────────────────────────────────────
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
pub mod physics_smoke_ficha;
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
/// O `env::var` que escolhe a cena vive em [`smoke::armed_scene`] e o `max_level` é
/// [`smoke::CENAS`], contado ao lado do `match`.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "physics",
    routers: &[ph2d_app_host::SmokeRouter {
        env: "PH2D_PHYSICS_SMOKE",
        max_level: smoke::CENAS,
    }],
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

// ⭐⭐ O que um GESTO DE CANVAS precisa da shell, escrito em TIPOS (Fase C): os sete campos do
// `AppGfx` que ele lê, e zero métodos novos no `AppHost` (um handle desfaria a fronteira).

/// A superfície de canvas que a shell empresta a um gesto da física.
///
/// ⚠️ Os empréstimos são DISJUNTOS por construção: a shell constrói isto numa expressão só.
pub struct CanvasCtx<'a> {
    pub camera: &'a ph2d_render::Camera2d,
    pub window: ph2d_host::WindowSize,
    pub sim: &'a mut ph2d_ecs::SimWorld,
    pub bridge: &'a ph2d_physics_ecs::PhysicsBridge,
    pub present: &'a mut ph2d_ecs::PresentWorld,
    pub toasts: &'a mut ph2d_editor_core::ToastQueue,
    pub hero: Option<&'a mut ph2d_editor_core::HeroScreen>,
}

impl CanvasCtx<'_> {
    /// O ponto de MUNDO sob um pixel de ecrã — a conversão que todo gesto faz.
    #[must_use]
    pub fn to_world(&self, sx: f32, sy: f32) -> [f32; 2] {
        self.camera.screen_to_world((sx, sy), self.window)
    }
}

// As três metades que vieram de `render_loop/`.
pub mod bridge;
pub mod inspector;
pub mod overlay;

// ⭐ A AUTORIA, vinda de `shells/desktop/src/physics/` (Fase C, 2026-09-12); o prefixo
// `physics_` fica de propósito (um rename corrompe as citações em prosa).

// O assador e a ponte.
pub mod bake;
pub mod panel_bridge;

// As JUNTAS — a autoria: desenhar uma, arrastar-lhe uma âncora, montar um rig,
// agarrar uma roldana.
/// ⭐⭐ A família FISICA nos canais: o corpo, o colisor e a zona da §11 (plano 35 §A.5).
pub mod canais_corpo;
/// ⭐⭐ A família JUNTA nos canais: os campos do `PhysicsJoint` sem `PropKind` (plano 35 §3.0).
pub mod canais_junta;
/// ⭐⭐ A família MUNDO nos canais: a gravidade e o solver sobre a Cena (F30 §B).
pub mod canais_mundo;
/// O losango das linhas do painel Physics (F30 §B).
pub mod marcas_mundo;

/// ⭐ As três famílias da física nos canais (JUNTA, FISICA, MUNDO): o que vale agora…
pub fn publicar_canais(
    sim: &mut ph2d_ecs::SimWorld,
    ponte: &ph2d_physics_ecs::PhysicsBridge,
    entidades: &[u64],
) {
    canais_junta::publicar(sim, entidades);
    canais_corpo::publicar(sim, entidades);
    canais_mundo::publicar(sim, ponte);
}

/// …e o que a curva mandou neste quadro.
pub fn aplicar_canais(
    sim: &mut ph2d_ecs::SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    ponte: &mut ph2d_physics_ecs::PhysicsBridge,
    estado: &mut physics_state::PhysicsState,
    (drive, doc): (
        &mut ph2d_preview_drive::PreviewDrive,
        &ph2d_timeline::TimelineDoc,
    ),
) {
    canais_junta::aplicar(sim, drive, doc);
    canais_corpo::aplicar(sim, registry, &mut estado.conducao);
    canais_mundo::aplicar(sim, ponte, &mut estado.mundo);
}
pub mod joint;
pub mod joint_anchor_drag;
pub mod joint_create;
pub mod joint_draw;
pub mod joint_rig;
pub mod joint_rig_drag;
pub mod joint_wheel;
pub mod joint_world;

// A §11 do Inspector aplicada ao ECS, e os seus vizinhos por assunto.
pub mod physics_apply;
pub mod physics_area;
mod physics_colide;
pub mod physics_markers;
pub mod physics_seed;
pub mod physics_state;
pub mod physics_surface;

// O input do player.
/// O dedo do jogador, com o mapa do hero; a lei mora em [`ph2d_physics_ecs::player_input`].
pub mod player_input {
    pub fn resolve_player_input(
        actions: &mut ph2d_input::ActionState,
        hero: Option<&ph2d_editor_core::HeroScreen>,
        device: &ph2d_input::InputState,
    ) -> ph2d_physics_ecs::PlayerInput {
        let mapa = hero.map(|h| &h.input_map);
        ph2d_physics_ecs::player_input::resolve_player_input(actions, mapa, device)
    }
}

// `#[cfg(test)]` e mais nada; o `measure_player_tape` é instrumento, não produto.
#[cfg(test)]
mod joint_break_tests;
#[cfg(test)]
mod joint_kind_tests;
#[cfg(test)]
mod joint_lado_b_tests;
#[cfg(test)]
mod joint_motor_tests;
#[cfg(test)]
mod joint_pair_tests;
#[cfg(test)]
mod joint_paste_tests;
#[cfg(test)]
mod joint_tests;
#[cfg(test)]
mod joint_wheel_tests;
#[cfg(test)]
mod joint_world_tests;
#[cfg(test)]
mod measure_player_tape;
#[cfg(test)]
mod physics_gesture_surface_tests;
#[cfg(test)]
mod tape_second_life_tests;
#[cfg(test)]
mod topdown_finger_tests;

// A porta `apply` é a do PRODUTO (as cenas usam-na); `test-support` porque três gates da shell
// atravessam-na e `cfg(test)` é falso aqui quando a shell compila (HOWTO §2.5).
#[cfg(any(test, feature = "test-support"))]
pub mod physics_tests;

/// Veio da shell na integração de 2026-09-16 (zero `crate::` lá: é desta família).
#[cfg(test)]
mod inspector_part_tests;
pub mod physics_smoke_base;
pub mod physics_smoke_joint_anim;
pub mod physics_smoke_out;
pub mod physics_smoke_player;
pub mod physics_smoke_rigs;
pub mod smoke;
