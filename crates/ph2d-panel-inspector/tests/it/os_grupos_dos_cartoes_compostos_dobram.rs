//! ⭐⭐ **Os grupos dos cartões compostos dobram pelo CLIQUE** (doc CodeEditor 12, oráculo Godot) —
//! câmera, paralaxe e HUD: o clique no cabeçalho fecha o grupo, as linhas dele saem do índice de
//! acerto, a dobra sobrevive à troca de objecto, e um segundo clique devolve as linhas.
//!
//! ⚠️ Pelo despacho do produto, nunca pelo `set_collapsed`: a máscara da câmera tinha um gate que a
//! abria à mão e o clique no cabeçalho dela estava morto.
//!
//! **Mutações que devem sangrar:** tirar o `mark_collapsible_section` de um grupo · pintar o corpo
//! sem perguntar `Grupo::aberto` · registar o cabeçalho sem rect.

use ph2d_a11y::NodeId;
use ph2d_editor_core::hud_edits::InspectorHudInfo;
use ph2d_editor_core::parallax_edits::{CameraDoJogo, InspectorParallaxInfo};
use ph2d_editor_core::screens::hero::{
    InspectorCameraFollow, InspectorCameraInfo, InspectorCameraLimits, InspectorGameCamera,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, ids, set_current_inspector_camera, set_current_inspector_hud,
    set_current_inspector_parallax,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn pintados(h: &mut MockPanelHost, st: &mut InspectorState) -> Vec<(NodeId, Rect)> {
    h.settle_section_folds();
    h.paint::<InspectorPanel>(st, VIEWPORT)
        .into_iter()
        .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
        .collect()
}

fn clica(h: &mut MockPanelHost, st: &mut InspectorState, id: NodeId, t: u128) {
    let r = pintados(h, st)
        .into_iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| r)
        .unwrap_or_else(|| panic!("o cabecalho {id:?} nao foi pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, t));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, t + SEC / 100)));
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
}

/// O caso inteiro de UM grupo: aberto por omissão → clique fecha e esconde `filhos` → outro
/// objecto e volta, continua fechado → clique reabre. `troca` publica o objecto `0` ou `1`.
fn o_grupo_dobra(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    grupo: NodeId,
    filhos: &[NodeId],
    troca: &dyn Fn(u8),
) {
    let tem = |r: &[(NodeId, Rect)], id: NodeId| r.iter().any(|(n, _)| *n == id);
    let r = pintados(h, st);
    for &f in filhos {
        assert!(
            tem(&r, f),
            "{f:?} devia estar visivel com o grupo {grupo:?} aberto (omissao)"
        );
    }
    clica(h, st, grupo, SEC);
    assert!(
        h.store().is_collapsed(grupo),
        "o clique no grupo {grupo:?} nao o fechou — controlo morto"
    );
    let r = pintados(h, st);
    assert!(
        tem(&r, grupo),
        "o cabecalho {grupo:?} sumiu com o grupo fechado"
    );
    for &f in filhos {
        assert!(
            !tem(&r, f),
            "{f:?} continua clicavel com o grupo {grupo:?} fechado"
        );
    }
    troca(1);
    let _ = pintados(h, st);
    troca(0);
    let r = pintados(h, st);
    assert!(
        filhos.iter().all(|&f| !tem(&r, f)),
        "a dobra do grupo {grupo:?} nao sobreviveu a troca de objecto"
    );
    clica(h, st, grupo, 3 * SEC);
    let r = pintados(h, st);
    for &f in filhos {
        assert!(
            tem(&r, f),
            "{f:?} nao voltou depois de reabrir o grupo {grupo:?}"
        );
    }
}

fn camera(bits: u64) -> InspectorCameraInfo {
    InspectorCameraInfo {
        entity_bits: bits,
        camera: InspectorGameCamera {
            height_world: 10.0,
            offset: [0.0, 0.0],
            priority: 0,
            dolly: 0.0,
            active: true,
            cull_mask: u32::MAX,
        },
        follow: Some(InspectorCameraFollow {
            target: "Heroi".into(),
            damping: [5.0, 5.0],
            dead_zone: [0.25, 0.25],
            lookahead: [0.0, 0.0],
            offset: [0.0, 0.0],
            target_found: true,
        }),
        limits: Some(InspectorCameraLimits {
            min: [-30.0, -12.0],
            max: [30.0, 12.0],
            smaller_than_view: false,
        }),
        camera_count: 1,
        is_active_camera: true,
        preview_on: false,
        selected_count: 1,
        format: ph2d_editor_core::GameFormat::default(),
        frame: ph2d_editor_core::CameraFrameShow::Always,
    }
}

#[test]
fn os_grupos_da_camera_e_a_mascara_dobram_pelo_clique() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let troca = |n: u8| set_current_inspector_camera(Some(camera(0xCA00 + u64::from(n))));
    troca(0);
    o_grupo_dobra(
        &mut h,
        &mut st,
        ids::INSP_CAMERA_FOLLOW_GROUP,
        &[ids::INSP_CAMERA_TARGET, ids::INSP_CAMERA_DAMP_X],
        &troca,
    );
    o_grupo_dobra(
        &mut h,
        &mut st,
        ids::INSP_CAMERA_LIMITS_GROUP,
        &[ids::INSP_CAMERA_MIN_X, ids::INSP_CAMERA_MAX_Y],
        &troca,
    );
    // A máscara nasce RECOLHIDA: abre-se primeiro, depois corre o mesmo caso.
    clica(&mut h, &mut st, ids::INSP_CAMERA_CULL_HEADER, SEC);
    o_grupo_dobra(
        &mut h,
        &mut st,
        ids::INSP_CAMERA_CULL_HEADER,
        &ids::INSP_CAMERA_CULL_BIT[..2],
        &troca,
    );
    set_current_inspector_camera(None);
}

fn paralaxe(bits: u64) -> InspectorParallaxInfo {
    InspectorParallaxInfo {
        entity_bits: bits,
        factor: [0.35, 0.6],
        repeat: Some([7.0, 3.0]),
        motion: Some([0.35, -0.2]),
        limits: Some(([-30.0, -12.0], [30.0, 14.0])),
        camera: CameraDoJogo::Activa,
        atravessa: false,
        pre_visualizacao: true,
        outro_motor: false,
        e_neutra: false,
        selected_count: 1,
    }
}

#[test]
fn os_grupos_da_paralaxe_dobram_pelo_clique() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let troca = |n: u8| set_current_inspector_parallax(Some(paralaxe(0xBA00 + u64::from(n))));
    troca(0);
    for (grupo, filhos) in [
        (
            ids::INSP_PARALLAX_REPEAT_GROUP,
            &[ids::INSP_PARALLAX_TILE_X][..],
        ),
        (
            ids::INSP_PARALLAX_MOTION_GROUP,
            &[ids::INSP_PARALLAX_VEL_Y][..],
        ),
        (
            ids::INSP_PARALLAX_LIMITS_GROUP,
            &[ids::INSP_PARALLAX_MIN_X, ids::INSP_PARALLAX_MAX_Y][..],
        ),
    ] {
        o_grupo_dobra(&mut h, &mut st, grupo, filhos, &troca);
    }
    // ⭐ O factor é a parte PRINCIPAL: sem cabeçalho, nunca dobra com um grupo.
    clica(&mut h, &mut st, ids::INSP_PARALLAX_REPEAT_GROUP, 5 * SEC);
    let r = pintados(&mut h, &mut st);
    assert!(r.iter().any(|(n, _)| *n == ids::INSP_PARALLAX_K_X));
    set_current_inspector_parallax(None);
}

fn hud(bits: u64) -> InspectorHudInfo {
    InspectorHudInfo {
        entity_bits: bits,
        has_button: true,
        signal: "bateu".into(),
        has_counter: true,
        counter_name: "pontos".into(),
        ..InspectorHudInfo::default()
    }
}

#[test]
fn os_grupos_do_hud_dobram_pelo_clique() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let troca = |n: u8| set_current_inspector_hud(Some(hud(0xD00 + u64::from(n))));
    troca(0);
    o_grupo_dobra(
        &mut h,
        &mut st,
        ids::INSP_HUD_BUTTON_GROUP,
        &[ids::INSP_HUD_DISABLED],
        &troca,
    );
    o_grupo_dobra(
        &mut h,
        &mut st,
        ids::INSP_HUD_COUNTER_GROUP,
        &[ids::INSP_HUD_COUNTER_KEEP],
        &troca,
    );
    set_current_inspector_hud(None);
}
