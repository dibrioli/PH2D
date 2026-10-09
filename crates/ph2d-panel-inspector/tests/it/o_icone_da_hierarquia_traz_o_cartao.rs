//! ⭐⭐ **O ícone de componente da Hierarquia traz o cartão à vista** (doc 01 §7.1) — o lado do
//! Inspector: o pedido no store vira rolagem até ao PRIMEIRO cartão da família, aberto.
//!
//! Dois cartões de famílias diferentes (Audio acima de Camera, pela ordem da paleta) numa janela
//! baixa: pedir a Camera tem de rolar, e a rolagem tem de pôr o cabeçalho dela no topo do corpo.

use ph2d_component_desc::ComponentCategory;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{
    HeroSelection, InspectorAudioInfo, InspectorAudioSource, InspectorCameraFollow,
    InspectorCameraInfo, InspectorCameraLimits, InspectorGameCamera, InspectorVisibilityInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_audio, set_current_inspector_camera,
    set_current_inspector_visibility,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 420.0,
};
const NOME: &str = "Câmara";
/// A entidade que os retratos pintam — a identidade pela qual o pedido casa (onda 4, item 10a).
const ENTIDADE: u64 = 0xABCD_1234;
/// OUTRA entidade com o MESMO nome.
const HOMONIMO: u64 = 0xABCD_5678;

fn host() -> (MockPanelHost, InspectorState) {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_audio(Some(InspectorAudioInfo {
        entity_bits: 0xABCD_1234,
        source: Some(InspectorAudioSource {
            sound: "/tmp/passo.wav".into(),
            volume_db: 0.0,
            pitch: 1.0,
            looping: false,
            autoplay: false,
            max_distance: 10.0,
            attenuation: 1.0,
            non_spatialized_radius: 0.0,
            panning_strength: 1.0,
            max_polyphony: 1,
            bus_tag: 0,
            file_missing: false,
            reachable_by_signal: true,
        }),
        is_listener: false,
        listener_count: 0,
        is_active_listener: false,
        bus_labels: vec![
            "SFX".into(),
            "Music".into(),
            "Voice".into(),
            "Master".into(),
        ],
        selected_count: 1,
    }));
    set_current_inspector_camera(Some(InspectorCameraInfo {
        entity_bits: 0xABCD_1234,
        camera: InspectorGameCamera {
            height_world: 10.0,
            offset: [0.0, 0.0],
            priority: 0,
            dolly: 0.0,
            active: true,
            cull_mask: u32::MAX,
        },
        // ⚠️ Com quem segue e a cerca: o cartão fica mais alto que o corpo, e o clamp da rolagem
        // (o fim do conteúdo) não o impede de chegar ao topo — medido: sem eles parava 20 px abaixo.
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
    }));
    // ⚠️ O Inspector sabe QUEM pinta pelos retratos de identidade (nome/transform/visibilidade):
    // é o `last_entity` do sync, e é por ele que o pedido casa.
    set_current_inspector_visibility(Some(InspectorVisibilityInfo {
        entity_bits: ENTIDADE,
        visible: true,
        mixed: false,
    }));
    *h.selection_mut() = Some(HeroSelection {
        label: NOME.into(),
        kind: "ENT".into(),
        world_pos: (0.0, 0.0),
    });
    (h, InspectorState::default())
}

fn limpa() {
    set_current_inspector_visibility(None);
    set_current_inspector_audio(None);
    set_current_inspector_camera(None);
}

fn header_y(rects: &[(ph2d_a11y::NodeId, Rect)], section: ph2d_a11y::NodeId) -> Option<f32> {
    rects
        .iter()
        .find(|(id, _)| *id == section)
        .map(|(_, r)| r.y)
}

#[test]
fn o_pedido_rola_ate_ao_cartao_da_familia_e_abre_o() {
    let (mut h, mut st) = host();
    h.store_mut()
        .set_collapsed(ids::INSP_LIVE_AUDIO_SECTION, false);
    h.store_mut()
        .set_collapsed(ids::INSP_LIVE_CAMERA_SECTION, true);
    let antes = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    // O 1.º cartão é o da visibilidade (o retrato de identidade também pinta).
    let topo = header_y(&antes, ids::INSP_LIVE_VISIBILITY_SECTION).expect("o 1.º cartão");
    assert!(
        header_y(&antes, ids::INSP_LIVE_CAMERA_SECTION).is_none(),
        "pré-condição: a Camera tem de começar FORA da vista, senão o teste não mede a rolagem"
    );
    h.store_mut()
        .request_inspector_reveal(ComponentCategory::Camera, ENTIDADE);
    // Dois quadros: o 1.º abre o cartão, o 2.º rola com a altura dele já medida.
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        h.store().inspector_reveal().is_none(),
        "o pedido não foi atendido"
    );
    assert!(
        !h.store().is_collapsed(ids::INSP_LIVE_CAMERA_SECTION),
        "o cartão pedido tem de ABRIR"
    );
    let scroll = h.store().panel_scroll(ids::INSP_PANEL);
    assert!(scroll > 0.0, "a Camera está abaixo do Audio e nada rolou");
    let pintado = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let depois = header_y(&pintado, ids::INSP_LIVE_CAMERA_SECTION)
        .expect("depois de rolar, a Camera tem de estar à vista");
    // ⚠️ A lei alinha a BANDA da Camera à do 1.º cartão; o cabeçalho de um cartão de componente fica
    // uns píxeis abaixo do de um cartão de identidade (medido: 9 px), então mede-se «é o de cima e
    // está a menos de um cabeçalho do topo».
    let alto = pintado
        .iter()
        .find(|(id, _)| *id == ids::INSP_LIVE_CAMERA_SECTION)
        .map_or(0.0, |(_, r)| r.h);
    assert!(
        (depois - topo).abs() < alto,
        "a Camera devia ficar no TOPO do corpo (onde o 1.º cartão estava, {topo}) e ficou em {depois}"
    );
    for id in ids::LIVE_SECTION_IDS {
        if let Some(y) = header_y(&pintado, id) {
            assert!(
                id == ids::INSP_LIVE_CAMERA_SECTION || y > depois || y < topo,
                "o cartão {id:?} ficou entre o topo e a Camera ({y} < {depois})"
            );
        }
    }
    limpa();
}

#[test]
fn o_pedido_de_outro_objecto_espera_e_o_sem_cartao_morre_sem_rolar() {
    let (mut h, mut st) = host();
    h.store_mut()
        .request_inspector_reveal(ComponentCategory::Camera, HOMONIMO);
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        h.store().inspector_reveal().is_some(),
        "o pedido de um objecto que ainda não chegou ao Inspector tem de esperar por ele"
    );
    h.store_mut()
        .request_inspector_reveal(ComponentCategory::Scripting, ENTIDADE);
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(h.store().inspector_reveal().is_none());
    assert_eq!(h.store().panel_scroll(ids::INSP_PANEL), 0.0);
    limpa();
}

/// ⭐⭐ RED-FIRST (onda 4, item 10a) — **dois objectos com o MESMO nome**, um já no Inspector: o
/// pedido do OUTRO espera pela troca. Casado pelo nome, ele era atendido no quadro de antes da
/// troca — o cartão abria e rolava sobre o objecto errado.
#[test]
fn o_pedido_de_um_homonimo_espera_pela_troca() {
    let (mut h, mut st) = host();
    h.store_mut()
        .set_collapsed(ids::INSP_LIVE_CAMERA_SECTION, true);
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    // O OUTRO «Câmara» (outra entidade, mesmo nome) pediu o cartão da Camera.
    h.store_mut()
        .request_inspector_reveal(ComponentCategory::Camera, HOMONIMO);
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let pendente = h.store().inspector_reveal().is_some();
    let fechado = h.store().is_collapsed(ids::INSP_LIVE_CAMERA_SECTION);
    let scroll = h.store().panel_scroll(ids::INSP_PANEL);
    limpa();
    assert!(
        pendente,
        "o pedido do homónimo foi atendido sobre o objecto que ainda está no Inspector"
    );
    assert!(fechado, "o cartão abriu no objecto errado");
    assert_eq!(scroll, 0.0, "o corpo rolou no objecto errado");
}
