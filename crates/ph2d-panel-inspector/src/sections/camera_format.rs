//! **O formato do jogo e a moldura** no cartão Camera (plano GEF 17): o seletor das 15 predefinições,
//! o tamanho do `Custom` e «Frame: Always | When Selected». São definições do PROJECTO que o snapshot
//! da câmera traz; irmão de [`super::camera`] pelo tecto de LOC.

use super::*;
use ph2d_editor_core::GamePreset;
use ph2d_editor_core::screens::hero::InspectorCameraInfo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// Os rótulos das três fileiras — entram na medição da coluna da secção.
pub(crate) const ROTULOS: [&str; 4] = [
    "panel.inspector.camera.format",
    "panel.inspector.camera.custom_size",
    "panel.inspector.camera.frame",
    "panel.inspector.camera.icon_size",
];

/// As 15 entradas do seletor, na ordem de `GamePreset::ALL` (a posição é a tag).
pub(crate) fn format_options() -> Vec<DropdownOption<u8>> {
    ids::INSP_CAMERA_FORMAT_OPT
        .iter()
        .zip(GamePreset::ALL)
        .map(|(&id, p)| DropdownOption::new(id, p.to_u8(), tr(p.label_key()).to_owned()))
        .collect()
}

/// Pinta as fileiras do formato e da moldura. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn format_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCameraInfo,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = format_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        info,
        seccao,
    );
    if info.format.preset == GamePreset::Custom {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            TextKey::new(ROTULOS[1]),
            &[ids::INSP_CAMERA_CUSTOM_W, ids::INSP_CAMERA_CUSTOM_H],
            1.0, // LITERAL-PX-OK: passo de um píxel do jogo
            Some(ph2d_editor_core::widget::Unit::Px),
            seccao,
        );
    }
    let labels = [
        tr("panel.inspector.camera.frame_always"),
        tr("panel.inspector.camera.frame_selected"),
    ];
    let selected = u8::from(info.frame == ph2d_editor_core::CameraFrameShow::Selected);
    let cur_y = super::rows::seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        TextKey::new(ROTULOS[2]),
        ids::INSP_CAMERA_FRAME_GROUP,
        &ids::INSP_CAMERA_FRAME,
        &labels,
        selected,
    );
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        TextKey::new(ROTULOS[3]),
        &[ids::INSP_CAMERA_ICON_SIZE],
        4.0, // LITERAL-PX-OK: passo de 4 px de tela
        Some(ph2d_editor_core::widget::Unit::Px),
        seccao,
    )
}

/// O seletor do formato — um chip, como o do barramento do áudio. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn format_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCameraInfo,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr(ROTULOS[0]),
        seccao,
    );
    let rect = linha.control;
    hit_index.register(ids::INSP_CAMERA_FORMAT_PICK, rect);
    let open = matches!(
        store.get(ids::INSP_CAMERA_FORMAT_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let sel = info.format.preset.to_u8();
    let mut dd = Dropdown::new(ids::INSP_CAMERA_FORMAT_PICK, "", format_options())
        .open(open)
        .visual(store.dropdown_visual(ids::INSP_CAMERA_FORMAT_PICK));
    dd.select(sel);
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    // ⚠️ O popover pinta-se no passe diferido (`crate::popovers`), senão sairia por baixo da secção
    // seguinte.
    if open {
        crate::state_popovers::set_pending_camera_format_dd(Some((sel, rect)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, linha.dot);
    y + ph2d_tokens::row_pitch_px()
}
