//! ⭐⭐⭐ **As QUATRO secções que estreavam a catraca do [`super::toda_seccao_viva_chega_a_pixel`]**
//! — `Factory` · `Projectile` · `State Machine` · `Top-Down`.
//!
//! # ⛔⛔ Porque é que elas NÃO vivem no ficheiro do censo
//!
//! Ele **salta-se a si próprio** (os literais da catraca contam como semeadura — ver o cabeçalho
//! dele), logo um gate escrito lá dentro seria invisível à conta e as quatro continuariam a ler-se
//! como órfãs. *A régua e o que ela mede não podem morar no mesmo sítio.*
//!
//! # ⚠️ A régua é um CAMPO e não o cabeçalho da secção
//!
//! Um cabeçalho pintado com o corpo vazio é exactamente o defeito que a `Factory` já pagou: as
//! secções dela shiparam **fora** da `LIVE_SECTIONS`, com o chevron a prometer uma dobra que não
//! podia acontecer.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

const ENTITY: u64 = 0x5EC0_0001;

/// ⭐⭐⭐ **As quatro secções que nenhum gate pintava, cada uma semeada e medida no PIXEL.**
///
/// ⚠️ **A régua é um CAMPO e não o cabeçalho:** um cabeçalho pintado com o corpo vazio é
/// exactamente o defeito que a `Factory` já pagou (as secções dela shiparam fora da
/// `LIVE_SECTIONS`, com o chevron a prometer uma dobra impossível).
///
/// **Mutações que devem sangrar:** tirar a chamada de uma destas secções do pintor · devolver cedo
/// no braço `None` do instantâneo.
#[test]
fn as_quatro_seccoes_que_estreavam_a_catraca_chegam_a_pixel() {
    use ph2d_editor_core::factory_edits::{
        InspectorFactory, InspectorFactoryInfo, InspectorLifecycle, InspectorSpawnWhere,
    };
    use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
    use ph2d_editor_core::statemachine_edits::{
        InspectorStateMachineInfo, InspectorStateRow, InspectorTransitionRow,
    };
    use ph2d_editor_core::topdown_edits::{
        InspectorFacing, InspectorMoveDirections, InspectorTopDownInfo, InspectorViewpoint,
    };
    use ph2d_panel_inspector::{
        ids, set_current_inspector_factory, set_current_inspector_projectile,
        set_current_inspector_statemachine, set_current_inspector_topdown,
    };

    /// Pinta com o instantâneo semeado e diz se `id` chegou a pixel.
    fn pinta_e_ve(semeia: impl FnOnce(), limpa: impl FnOnce(), id: ph2d_a11y::NodeId) -> bool {
        let mut h = MockPanelHost::with_panel::<InspectorPanel>();
        let mut st = InspectorState::default();
        semeia();
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let achou = rects.iter().any(|(n, _)| *n == id);
        limpa();
        achou
    }

    let fabrica = InspectorFactoryInfo {
        entity_bits: ENTITY,
        factory: Some(InspectorFactory {
            recipe: "Bala".into(),
            recipe_found: true,
            on_signal: "tiro".into(),
            spawn_where: InspectorSpawnWhere::Here,
            area: [2.0, 2.0],
            tag: String::new(),
            pick_random: false,
            burst: 1,
            alive_max: 16,
            total_max: 0,
            on_spawned: "nasceu".into(),
            on_exhausted: String::new(),
            seed: 7,
            aim_from_spawner: true,
            alive: 3,
        }),
        lifecycle: Some(InspectorLifecycle {
            lifetime_s: Some(2.0),
            on_death: "morreu".into(),
            outside_margin: Some(1.0),
        }),
        is_spawned: false,
        has_game_camera: true,
        clock_playing: true,
        selected_count: 1,
    };
    assert!(
        pinta_e_ve(
            || set_current_inspector_factory(Some(fabrica.clone())),
            || set_current_inspector_factory(None),
            ids::INSP_FACTORY_RECIPE,
        ),
        "a seccao FACTORY nao chega a pixel"
    );

    let projectil = InspectorProjectileInfo {
        entity_bits: ENTITY,
        initial_speed: 7.0,
        acceleration: 0.0,
        max_speed: 20.0,
        gravity: 0.0,
        bounciness: 0.0,
        max_bounces: 0,
        range: 4.0,
        face_velocity: true,
        homing_accel: 0.0,
        homing_target: String::new(),
        homing_target_missing: false,
        body_is_kinematic: true,
        has_body: true,
        clock_playing: true,
        flight_over: false,
        selected_count: 1,
    };
    assert!(
        pinta_e_ve(
            || set_current_inspector_projectile(Some(projectil.clone())),
            || set_current_inspector_projectile(None),
            ids::INSP_PJ_SPEED,
        ),
        "a seccao PROJECTILE nao chega a pixel"
    );

    let cerebro = InspectorStateMachineInfo {
        entity_bits: ENTITY,
        states: vec![
            InspectorStateRow {
                name: "Fechada".into(),
                on_enter: "fecha".into(),
                on_exit: String::new(),
                has_exit: false,
            },
            InspectorStateRow {
                name: "Aberta".into(),
                on_enter: "abre".into(),
                on_exit: String::new(),
                has_exit: false,
            },
        ],
        transitions: vec![InspectorTransitionRow {
            from: 0,
            on: "toque".into(),
            to: 1,
        }],
        initial: 0,
        current: Some(0),
        clock_playing: true,
        selected_count: 1,
    };
    assert!(
        pinta_e_ve(
            || set_current_inspector_statemachine(Some(cerebro.clone())),
            || set_current_inspector_statemachine(None),
            ids::INSP_SM_STATE_NAME,
        ),
        "a seccao STATE MACHINE nao chega a pixel"
    );

    let mover = InspectorTopDownInfo {
        entity_bits: ENTITY,
        speed: 6.0,
        acceleration: 40.0,
        deceleration: 40.0,
        directions: InspectorMoveDirections::Eight,
        viewpoint: InspectorViewpoint::TopDown,
        viewpoint_angle_deg: 0.0,
        facing: InspectorFacing::Movement,
        turn_speed_deg: 720.0,
        min_slide_angle_deg: 5.0,
        max_slides: 4,
        default_controls: true,
        knockback_recovery: 24.0,
        body_is_kinematic: true,
        has_body: true,
        conflicts_with_platformer: false,
        clock_playing: true,
        selected_count: 1,
    };
    assert!(
        pinta_e_ve(
            || set_current_inspector_topdown(Some(mover.clone())),
            || set_current_inspector_topdown(None),
            ids::INSP_TD_SPEED,
        ),
        "a seccao TOP-DOWN nao chega a pixel"
    );
}
