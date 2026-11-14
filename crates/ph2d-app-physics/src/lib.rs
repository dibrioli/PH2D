//! **A metade da família `physics` que não precisa da shell** (W2/L2).
//!
//! Depende das crates do módulo e do ECS, **nunca da shell** (auditoria de velocidade
//! 2026-09-10, §4-C2). O que uma cena pede à shell viaja como **dados** num [`SceneSetup`]
//! que o roteador aplica depois (ADR-0075: systems não se chamam) — inspeccionável e
//! testável sem janela, ao contrário de um trait `SceneHost`.

// ⭐ O `SceneCtx` e as cenas que só povoam um `World` moram na folha `ph2d-physics-smoke-scenes`
// (GEF02 onda 0): re-exportados aqui para que `crate::physics_smoke_x` e
// `ph2d_app_physics::physics_smoke_x` continuem a ser o caminho de todo chamador.
pub use ph2d_physics_smoke_scenes::{SceneCtx, SceneSetup};
// As cenas do pulo (110, 111) moram na folha `ph2d-physics-smoke-jump` (13/11).
pub use ph2d_physics_smoke_jump::{physics_smoke_ledge, physics_smoke_multi_jump};
pub use ph2d_physics_smoke_scenes::{
    physics_smoke_authoring, physics_smoke_base, physics_smoke_blast, physics_smoke_brake,
    physics_smoke_brink, physics_smoke_collider, physics_smoke_collision, physics_smoke_compound,
    physics_smoke_contacts, physics_smoke_damping, physics_smoke_events, physics_smoke_fk,
    physics_smoke_foot, physics_smoke_foot_fan, physics_smoke_glide, physics_smoke_grab,
    physics_smoke_ik, physics_smoke_interact, physics_smoke_joint_break, physics_smoke_kin_pure,
    physics_smoke_kin_push, physics_smoke_kin_water, physics_smoke_lead, physics_smoke_leave,
    physics_smoke_player_bake, physics_smoke_player_carry, physics_smoke_player_crouch,
    physics_smoke_player_dash, physics_smoke_player_drop, physics_smoke_player_flank,
    physics_smoke_player_grab, physics_smoke_player_run, physics_smoke_player_slope,
    physics_smoke_player_wall, physics_smoke_probes, physics_smoke_props, physics_smoke_raft,
    physics_smoke_rail_rope, physics_smoke_rigs, physics_smoke_rod, physics_smoke_signal,
    physics_smoke_signal_ends, physics_smoke_signal_leave, physics_smoke_soft_weld,
    physics_smoke_stone, physics_smoke_surface, physics_smoke_swim, physics_smoke_terminal,
    physics_smoke_water, physics_smoke_wheel, physics_smoke_world_pin, physics_smoke_zone_force,
    physics_smoke_zones,
};
pub use ph2d_physics_smoke_scenes::{
    physics_smoke_authoring, physics_smoke_base, physics_smoke_blast, physics_smoke_brake,
    physics_smoke_brink, physics_smoke_collider, physics_smoke_collision, physics_smoke_compound,
    physics_smoke_contacts, physics_smoke_damping, physics_smoke_events, physics_smoke_fk,
    physics_smoke_foot, physics_smoke_foot_fan, physics_smoke_glide, physics_smoke_grab,
    physics_smoke_ik, physics_smoke_interact, physics_smoke_joint_break,
    physics_smoke_joint_templates, physics_smoke_kin_pure, physics_smoke_kin_push,
    physics_smoke_kin_water, physics_smoke_lead, physics_smoke_leave, physics_smoke_ledge,
    physics_smoke_multi_jump, physics_smoke_player_bake, physics_smoke_player_carry,
    physics_smoke_player_crouch, physics_smoke_player_dash, physics_smoke_player_drop,
    physics_smoke_player_flank, physics_smoke_player_grab, physics_smoke_player_run,
    physics_smoke_player_slope, physics_smoke_player_wall, physics_smoke_probes,
    physics_smoke_props, physics_smoke_raft, physics_smoke_rail_rope, physics_smoke_rigs,
    physics_smoke_rod, physics_smoke_signal, physics_smoke_signal_ends, physics_smoke_signal_leave,
    physics_smoke_soft_weld, physics_smoke_stone, physics_smoke_surface, physics_smoke_swim,
    physics_smoke_terminal, physics_smoke_water, physics_smoke_wheel, physics_smoke_world_pin,
    physics_smoke_zone_force, physics_smoke_zones,
};

// ─────────────────────────────────────────────────────────────────────────
// As cenas. ⚠️ Esta lista é ORDENADA e gerada da pasta: o roteador da shell
// continua a ser a ÚNICA fonte de QUE nível corre o quê (CLAUDE.md §5.0) —
// aqui só se declara que o ficheiro existe.
// ─────────────────────────────────────────────────────────────────────────
pub mod physics_smoke_ficha;
pub mod physics_smoke_joint_bake;
pub mod physics_smoke_joint_copy;
pub mod physics_smoke_joint_custom;
pub mod physics_smoke_joint_draw;
pub mod physics_smoke_joint_glyphs;
pub mod physics_smoke_joint_handles;
pub mod physics_smoke_joint_pair;
pub mod physics_smoke_joint_pose;
pub mod physics_smoke_joint_rig;
pub mod physics_smoke_joint_slider;
pub mod physics_smoke_kinematic;
pub mod physics_smoke_player_forgive;
pub mod physics_smoke_player_tape;
pub mod physics_smoke_pulley;
pub mod physics_smoke_pulley_break;
pub mod physics_smoke_pulley_weston;
pub mod physics_smoke_stop;

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
    canais_corpo::aplicar(sim, registry, &mut estado.conducao, doc);
    canais_mundo::aplicar(sim, ponte, &mut estado.mundo);
}
pub mod joint;
pub mod joint_anchor_drag;
// ⭐ A montagem de juntas e os gestos de criar junta moram na `ph2d-joint-assembly` (corte 13/11).
pub use ph2d_joint_assembly::{
    joint_assembly, joint_assembly_card, joint_create, joint_drop, joint_world,
};
pub mod joint_draw;
pub mod joint_rig;
pub mod joint_rig_drag;
pub mod joint_wheel;
pub mod jump_arc;

// A §11 do Inspector aplicada ao ECS, e os seus vizinhos por assunto.
pub mod physics_apply;
pub mod physics_area;
mod physics_colide;
pub mod physics_markers;
pub mod physics_seed;
pub mod physics_state;
pub mod physics_surface;

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
mod physics_smoke_joint_templates_tests;
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
pub mod physics_smoke_joint_anim;
pub mod physics_smoke_out;
pub mod physics_smoke_player;
pub mod smoke;
