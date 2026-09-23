//! ⭐⭐⭐ **A secção CAMERA é PINTADA e está VIVA sob o dedo** (TOP-20 #7, W3).
//!
//! # Porque este ficheiro existe
//!
//! É a lição que o `Timers` custou, escrita como gate: ele shipou anexável e sem linha de edição, e
//! o report do dono foi *«timer sumiu do modal de componente»*. *Um componente anexável sem painel
//! é indistinguível de um que não foi anexado.*
//!
//! # ⚠️ E há um caso aqui que nenhuma outra secção tem: um interruptor que NÃO é documento
//!
//! O `Look Through` liga a vista pela câmera da cena. Ele é a metade que a W2 deixou nomeada e por
//! entregar — até este ficheiro existir, o único botão daquele motor era uma variável de ambiente.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{
    CameraFieldEdit, InspectorCameraFollow, InspectorCameraInfo, InspectorCameraLimits,
    InspectorGameCamera,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_camera};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
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

fn info(
    follow: Option<InspectorCameraFollow>,
    limits: Option<InspectorCameraLimits>,
    preview_on: bool,
) -> InspectorCameraInfo {
    InspectorCameraInfo {
        entity_bits: 0xABCD_1234,
        camera: InspectorGameCamera {
            height_world: 10.0,
            offset: [0.0, 0.0],
            priority: 0,
            dolly: 0.0,
            active: true,
            cull_mask: u32::MAX,
        },
        follow,
        limits,
        camera_count: 1,
        is_active_camera: true,
        preview_on,
        selected_count: 1,
    }
}

fn seguidor() -> InspectorCameraFollow {
    InspectorCameraFollow {
        target: "Heroi".into(),
        damping: [5.0, 5.0],
        dead_zone: [0.25, 0.25],
        lookahead: [0.0, 0.0],
        offset: [0.0, 0.0],
        target_found: true,
    }
}

fn cerca() -> InspectorCameraLimits {
    InspectorCameraLimits {
        min: [-30.0, -12.0],
        max: [30.0, 12.0],
        smaller_than_view: false,
    }
}

fn host(i: InspectorCameraInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_camera(Some(i));
    (h, InspectorState::default())
}

/// ⭐⭐⭐ **TODO campo dos três componentes é pintado com área** — a metade que o report do `Timers`
/// cobrou.
///
/// ⚠️ **A lista dos ids é varrida, não amostrada:** um campo novo que alguém acrescente e esqueça
/// de pintar cai aqui, em vez de ficar inalcançável em silêncio.
///
/// **Mutação que deve sangrar:** tirar qualquer linha de uma das três tabelas de campos.
#[test]
fn every_field_of_the_three_components_is_painted() {
    let (mut h, mut st) = host(info(Some(seguidor()), Some(cerca()), false));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for id in [
        // A câmera.
        ph2d_panel_inspector::ids::INSP_CAMERA_HEIGHT,
        ph2d_panel_inspector::ids::INSP_CAMERA_OFFSET_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_OFFSET_Y,
        ph2d_panel_inspector::ids::INSP_CAMERA_PRIORITY,
        ph2d_panel_inspector::ids::INSP_CAMERA_DOLLY,
        ph2d_panel_inspector::ids::INSP_CAMERA_ACTIVE,
        ph2d_panel_inspector::ids::INSP_CAMERA_PREVIEW,
        // Quem ela segue.
        ph2d_panel_inspector::ids::INSP_CAMERA_TARGET,
        ph2d_panel_inspector::ids::INSP_CAMERA_DAMP_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_DAMP_Y,
        ph2d_panel_inspector::ids::INSP_CAMERA_DEAD_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_DEAD_Y,
        ph2d_panel_inspector::ids::INSP_CAMERA_LOOK_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_LOOK_Y,
        ph2d_panel_inspector::ids::INSP_CAMERA_FOLLOW_OFF_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_FOLLOW_OFF_Y,
        // A cerca.
        ph2d_panel_inspector::ids::INSP_CAMERA_MIN_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_MIN_Y,
        ph2d_panel_inspector::ids::INSP_CAMERA_MAX_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_MAX_Y,
        // E a máscara — o cabeçalho dela, que nasce recolhido.
        ph2d_panel_inspector::ids::INSP_CAMERA_CULL_HEADER,
    ] {
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o campo {id:?} nao foi PINTADO com area clicavel"));
        assert!(r.w > 0.0 && r.h > 0.0, "campo {id:?} sem area: {r:?}");
    }
    set_current_inspector_camera(None);
}

/// ⭐⭐ **Uma câmera FIXA não mostra os campos de quem ela não segue.**
///
/// ⚠️ É a razão de os três componentes serem separados: o caso comum — uma câmera de sala — não
/// paga cinco controlos mortos à vista. ⛔ Um `follow_enabled` dentro da câmera daria o mesmo estado
/// com todos os campos sempre lá.
///
/// **Mutação que deve sangrar:** pintar o `follow_body` incondicionalmente.
#[test]
fn a_fixed_camera_does_not_show_the_follow_fields() {
    let (mut h, mut st) = host(info(None, None, false));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_CAMERA_HEIGHT),
        "a camera em si tem de estar la'"
    );
    for ausente in [
        ph2d_panel_inspector::ids::INSP_CAMERA_TARGET,
        ph2d_panel_inspector::ids::INSP_CAMERA_DAMP_X,
        ph2d_panel_inspector::ids::INSP_CAMERA_MIN_X,
    ] {
        assert!(
            !rects.iter().any(|(n, _)| *n == ausente),
            "o campo {ausente:?} foi pintado para uma camera que nao segue nem tem cerca"
        );
    }
    set_current_inspector_camera(None);
}

/// ⭐⭐⭐ **Carregar em `Look Through` chega ao barramento** — com o gesto REAL.
///
/// ⚠️ **Um `WidgetEvent::Toggled` sintético passa sobre uma caixa ausente do `populate`**, que é a
/// família dos dez chips do impasto. Aqui o `Down`+`Up` cai no rectângulo que a pintura registou.
///
/// **Mutação que deve sangrar:** tirar o `INSP_CAMERA_PREVIEW` do `populate_camera`.
#[test]
fn pressing_look_through_reaches_the_bus_with_a_real_pointer() {
    let (mut h, mut st) = host(info(Some(seguidor()), None, false));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let (_, r) = rects
        .iter()
        .find(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_CAMERA_PREVIEW)
        .copied()
        .expect("o `Look Through` nao foi pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre `Look Through` nao virou evento nenhum — ele esta' desenhado e nao existe \
         para o dispatcher (falta o registo no populate)"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let acoes = h.drained_actions();
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorCameraEdit {
                edit: CameraFieldEdit::Preview(true),
                ..
            }
        )),
        "carregar em `Look Through` com a pre-visualizacao DESLIGADA tem de mandar `Preview(true)`; \
         o que chegou foi {acoes:?}"
    );
    set_current_inspector_camera(None);
}

/// ⭐⭐ **O valor da caixa vem do SNAPSHOT** — ligada, o clique manda DESLIGAR.
///
/// ⚠️ É a lei que a §11 pagou com um report: ler o store faria o primeiro clique depois de trocar de
/// objecto mandar o valor do objecto **anterior**.
///
/// **Mutação que deve sangrar:** trocar `!info.preview_on` por `true`.
#[test]
fn the_toggle_reads_the_snapshot_not_the_store() {
    let (mut h, mut st) = host(info(Some(seguidor()), None, true));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let (_, r) = rects
        .iter()
        .find(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_CAMERA_PREVIEW)
        .copied()
        .expect("o `Look Through` nao foi pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let acoes = h.drained_actions();
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorCameraEdit {
                edit: CameraFieldEdit::Preview(false),
                ..
            }
        )),
        "com a pre-visualizacao LIGADA o clique tem de mandar `Preview(false)`; chegou {acoes:?}"
    );
    set_current_inspector_camera(None);
}

/// ⭐ **Os 32 bits da máscara existem sob o dedo** — depois de a sub-secção abrir.
///
/// ⚠️ **A grade nasce RECOLHIDA**, então este gate ABRE-A primeiro: medir com ela fechada leria
/// *«a máscara está morta»* sobre um painel correcto.
#[test]
fn the_cull_mask_grid_is_alive_once_opened() {
    let (mut h, mut st) = host(info(None, None, false));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_collapsed(ph2d_panel_inspector::ids::INSP_CAMERA_CULL_HEADER, false);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (bit, &id) in ph2d_panel_inspector::ids::INSP_CAMERA_CULL_BIT
        .iter()
        .enumerate()
    {
        assert!(
            rects.iter().any(|(n, _)| *n == id),
            "o bit {bit} da mascara nao chegou ao indice de acerto"
        );
    }
    set_current_inspector_camera(None);
}

/// ⭐⭐⭐ **O DOLLY (plano 24, W5) mostra o que a câmera TEM e o que se escreve chega ao barramento**
/// — a semente e o dreno, as duas metades que um gate de pintura não vê.
///
/// ⚠️ A fixtura usa `0,4` e não o `0` de fábrica: *um corpus no NEUTRO de um knob não testa esse
/// knob*, e a semente esquecida mostraria exactamente o `0` do `populate`.
///
/// **Mutações que devem sangrar:** tirar a linha do dolly do `sync_sections` · tirar o braço do
/// `event_camera` · mapear o id a outra variante.
#[test]
fn o_dolly_mostra_a_camera_e_chega_ao_barramento() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::PanelHostInternal as _;
    let id = ph2d_panel_inspector::ids::INSP_CAMERA_DOLLY;
    let mut i = info(None, None, false);
    i.camera.dolly = 0.4;
    let (mut h, mut st) = host(i);
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let lido = h.store().number_value(id).expect("o dolly esta' registado");
    assert!(
        (lido - 0.4).abs() < 1e-6,
        "o dolly mostra {lido} e a camera tem 0,4"
    );
    h.store_mut().set_number_value(id, 0.25);
    let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::ValueChanged(id));
    let edits: Vec<_> = h
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorCameraEdit { edit, .. } => Some(edit),
            _ => None,
        })
        .collect();
    assert_eq!(edits, vec![ph2d_editor_core::CameraFieldEdit::Dolly(0.25)]);
    set_current_inspector_camera(None);
}
