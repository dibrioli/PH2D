//! **O formato do jogo e a moldura** no cartão Camera (plano GEF 17): o seletor das 15 predefinições,
//! o tamanho do `Custom` e «Frame: Always | Selected». São definições do PROJECTO que o snapshot
//! da câmera traz; irmão de [`super::camera`] pelo tecto de LOC.

use super::*;
use ph2d_editor_core::GamePreset;
use ph2d_editor_core::screens::hero::InspectorCameraInfo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// Os rótulos das fileiras — entram na medição da coluna da secção.
pub(crate) const ROTULOS: [&str; 7] = [
    "panel.inspector.camera.format",
    "panel.inspector.camera.custom_size",
    "panel.inspector.camera.frame",
    "panel.inspector.camera.icon_size",
    "panel.inspector.camera.outside",
    "panel.inspector.camera.view",
    "panel.inspector.camera.preview",
];

/// As fileiras de VISTA (onda 18 e a miniatura, A6), cada uma `(rótulo, grupo, opções, [chave off, chave on])`.
const VISTA: [(usize, ph2d_a11y::NodeId, [ph2d_a11y::NodeId; 2], [&str; 2]); 3] = [
    (
        4,
        ids::INSP_CAMERA_OUTSIDE_GROUP,
        ids::INSP_CAMERA_OUTSIDE,
        [
            "panel.inspector.camera.outside_normal",
            "panel.inspector.camera.outside_dimmed",
        ],
    ),
    (
        5,
        ids::INSP_CAMERA_VIEW_GROUP,
        ids::INSP_CAMERA_VIEW,
        [
            "panel.inspector.camera.view_free",
            "panel.inspector.camera.view_follows",
        ],
    ),
    (
        6,
        ids::INSP_CAMERA_THUMB_GROUP,
        ids::INSP_CAMERA_THUMB,
        [
            "panel.inspector.camera.preview_off",
            "panel.inspector.camera.preview_on",
        ],
    ),
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
            ph2d_editor_core::widget::key_mark::Rotulo::from(TextKey::new(ROTULOS[1]))
                .eixos(ph2d_editor_core::widget::key_mark::Eixos::Tamanho),
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
    // Num cartão, os irmãos curtos dividem a linha: Frame + Icon Size, Outside + Editor View.
    let cartao = ph2d_editor_core::widget::card_ink::card_body().is_some();
    let meia = (w - ph2d_card_stack::metrics::FIELD_GAP) * 0.5;
    let (w_par, x_dir) = if cartao { (meia, x + w - meia) } else { (w, x) };
    let topo = cur_y;
    let fim_frame = super::rows::seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w_par,
        cur_y,
        TextKey::new(ROTULOS[2]),
        ids::INSP_CAMERA_FRAME_GROUP,
        &ids::INSP_CAMERA_FRAME,
        &labels,
        selected,
    );
    let fim_icone = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x_dir,
        w_par,
        if cartao { topo } else { fim_frame },
        TextKey::new(ROTULOS[3]),
        &[ids::INSP_CAMERA_ICON_SIZE],
        4.0, // LITERAL-PX-OK: passo de 4 px de tela
        Some(ph2d_editor_core::widget::Unit::Px),
        seccao,
    );
    let mut cur_y = if cartao {
        fim_frame.max(fim_icone)
    } else {
        fim_icone
    };
    let ligado = [
        info.canvas.dim_outside,
        info.canvas.follows,
        info.canvas.preview,
    ];
    let topo_do_par = cur_y;
    for (i, ((rotulo, grupo, opcoes, chaves), on)) in VISTA.iter().zip(ligado).enumerate() {
        let labels = chaves.map(tr);
        // A 1.ª e a 2.ª vistas fazem par no cartão; a última fica sozinha.
        let (xi, wi, yi) = match i {
            0 if cartao => (x, w_par, topo_do_par),
            1 if cartao => (x_dir, w_par, topo_do_par),
            _ => (x, w, cur_y),
        };
        let fim = super::rows::seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            xi,
            wi,
            yi,
            TextKey::new(ROTULOS[*rotulo]),
            *grupo,
            opcoes,
            &labels,
            u8::from(on),
        );
        cur_y = if cartao && i < 2 { cur_y.max(fim) } else { fim };
    }
    cur_y
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
    // ⭐ A forma sai da LISTA (o doc de `dropdown_label_budget`): se o nome mais largo das 15 não cabe
    // no chip ao lado do nome, é PALETA — o nome por cima e o chip a toda a largura (a lei da
    // `paint_choice_row`). Ao lado, «16:9 · 1920 × 1080» saía «16:9 · 192…» (76,6 px, dock de 1366).
    let lado = ph2d_editor_core::property_row::caixa_do_controlo(x, w, y, seccao);
    let mais_largo = format_options()
        .iter()
        .map(|o| text_system.prefix_width(&o.label, TypeToken::Base.px()))
        .fold(0.0_f32, f32::max);
    // No cartão o rótulo vai sempre em cima e o chip enche a linha (05 §4).
    let cartao = ph2d_editor_core::widget::card_ink::card_body().is_some();
    let cabe_ao_lado =
        cartao || ph2d_editor_core::widget::dropdown_chip_width_for(mais_largo, ROW_H_PX) <= lado.w;
    let (rect, dot, fim) = if cabe_ao_lado {
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
        // O formato é do PROJECTO, não se anima: no cartão não leva marca (D17).
        (
            linha.control,
            (!cartao).then_some(linha.dot),
            linha.control.y + linha.control.h.max(ph2d_tokens::row_pitch_px()),
        )
    } else {
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            tr(ROTULOS[0]),
            x,
            y,
            font,
            w,
            resolve(ColorToken::Text2, theme),
        );
        let gy = y + font + SECTION_LABEL_TO_CONTROL_PX;
        (
            Rect::new(x, gy, w, ROW_H_PX),
            None,
            gy + ph2d_tokens::row_pitch_px(),
        )
    };
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
    if let Some(dot) = dot {
        ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    }
    fim
}
