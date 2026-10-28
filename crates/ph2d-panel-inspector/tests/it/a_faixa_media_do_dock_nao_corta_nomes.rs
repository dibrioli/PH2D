//! ⭐⭐⭐ **NUMA LARGURA MÉDIA DO DOCK, UM NOME NÃO É CORTADO PARA UMA SECÇÃO VIZINHA CABER.**
//!
//! Report do dono 24/10 (`docs/CodeEditor/ferramentas/faixa_dos_rotulos_espremidos/`): com a câmera
//! seleccionada e o dock a `≈ 240–270 px`, o `Transform` e o `Camera Follow` punham os dois campos
//! lado a lado e o NOME elidido («Posit…», «Da…»); a `230` empilhavam com o nome inteiro e a `280`
//! cabia tudo. A coluna do nome é UMA por painel, e o tecto dela vinha da secção de nomes curtos
//! (`Camera Limits`: `Min`/`Max`) que cedia por si — a cedência dela cortava o nome das outras, que
//! é o que a §6-ter proíbe a uma secção (*«nunca abaixo do que o rótulo precisa»*).

use ph2d_editor_core::screens::hero::{
    InspectorCameraFollow, InspectorCameraInfo, InspectorCameraLimits, InspectorGameCamera,
    InspectorTransformInfo,
};
use ph2d_editor_core::screens::layout::ChromeBands;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, set_current_inspector_camera, set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

/// Os nomes das linhas da cena do report — os que o artista lê à esquerda de um campo.
const NOMES: &[&str] = &[
    "Position X / Y",
    "Rotation",
    "Scale X / Y",
    "Skew X / Y",
    "Height",
    "Offset",
    "Priority",
    "Dolly",
    "Active",
    "Look Through",
    "Target",
    "Damping",
    "Dead Zone",
    "Lookahead",
    "Min",
    "Max",
];

fn camera() -> InspectorCameraInfo {
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
        icon_px: ph2d_editor_core::game_format::CAMERA_ICON_PX,
    }
}

/// Os nomes da cena cortados a uma largura de dock — pela pintura do REGISTO (a que corre a coluna
/// do painel, [`ph2d_editor_core::widget::ColunaDoPainel`]), no 3.º quadro, onde ela convergiu.
///
/// ⛔ Pelo `paint_with_layout` cada secção responde sozinha, e a régua passava verde sobre o report:
/// o corte vinha da coluna do PAINEL.
fn cortados(dock: f32) -> Vec<String> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut painel = ph2d_editor_core::panel::ErasedPanel::new::<InspectorPanel>();
    set_current_inspector_camera(Some(camera()));
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [2000.0, 800.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    let bandas = ChromeBands {
        right_dock_w: dock,
        ..ChromeBands::DEFAULT
    };
    let medidos = host.medindo_a_pintura_do_registo_com_bandas(&mut painel, VIEWPORT, bandas);
    set_current_inspector_camera(None);
    set_current_inspector_transform(None);
    let mut v: Vec<String> = medidos
        .into_iter()
        .filter(|m| m.pintado != m.texto && NOMES.contains(&m.texto.as_str()))
        .map(|m| format!("{:?} → {:?} ({:.1} px)", m.texto, m.pintado, m.largura))
        .collect();
    v.sort();
    v.dedup();
    v
}

/// ⭐⭐⭐ **Da largura em que a linha já empilha até à largura de omissão, nenhum nome da cena corta.**
///
/// ⚠️ O degrau é de `1 px`: a faixa do report tem `≈ 40 px`, e uma régua de passos largos saltaria
/// as bordas dela. O piso é a largura em que o dono viu os nomes inteiros (`≈ 230`).
#[test]
fn nenhum_nome_da_cena_da_camera_corta_entre_230_e_320() {
    let mut falhas = Vec::new();
    for dock in 230..=320 {
        #[expect(clippy::cast_precision_loss, reason = "px inteiros pequenos")]
        let c = cortados(dock as f32);
        if !c.is_empty() {
            falhas.push(format!("dock {dock}: {}", c.join(" · ")));
        }
    }
    assert!(
        falhas.is_empty(),
        "{} larguras cortam nomes:\n  {}",
        falhas.len(),
        falhas.join("\n  ")
    );
}
