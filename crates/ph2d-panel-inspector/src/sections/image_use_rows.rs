//! ⭐ **O uso da imagem à vista** ([plano GEF 15](../../../../docs/GameEngineFuncional/15_uso_da_imagem_a_vista_plano.md)
//! D1/D2): logo abaixo da linha Image, o seletor «Use» (Still · Repeated · Animated · Region) e, por baixo, só os
//! campos do uso escolhido — parada: o quadro no ecrã e «‹ n / N ›»; repetida: Width/Height; animada: H/V Frames e a
//! secção Animation ali mesmo (sem a folha com as células, dono 27/10); região (plano GEF 16): «Edit Region…» e o
//! tamanho da área.

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;
use ph2d_editor_core::{InspectorDraw, InspectorImageUse};
use ph2d_i18n::{tr, tr_with};

const USOS: [&str; 4] = [
    "panel.inspector.image.use_still",
    "panel.inspector.image.use_repeated",
    "panel.inspector.image.use_animated",
    "panel.inspector.image.use_region",
];

/// Pinta o seletor e os campos do uso de `info`; `anim` é a secção Animation (e a animação aberta) de uma sprite
/// animada. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_image_use(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
    anim: Option<(&InspectorAnimInfo, usize)>,
) -> f32 {
    let mut y = if crate::event_image_use::use_is_fixed(info) {
        super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            tr("panel.inspector.image.use_background"),
            ColorToken::Text3,
        )
    } else {
        let labels = USOS.map(tr);
        let selected = InspectorImageUse::ALL
            .iter()
            .position(|u| *u == info.image_use)
            .unwrap_or(0);
        super::rows::seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y,
            TextKey::new("panel.inspector.image.use"),
            ids::INSP_IMAGE_USE_GROUP,
            &ids::INSP_IMAGE_USE,
            &labels,
            selected as u8,
        )
    };
    match info.image_use {
        InspectorImageUse::Still if crate::event_image_use::sheet_cells(info) > 1 => {
            super::sheet_thumb::paint_still_frame(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                y,
                info,
            )
        }
        InspectorImageUse::Still => y,
        InspectorImageUse::Repeated => {
            paint_size_rows(scene, text_system, theme, hit_index, store, x, w, y, info)
        }
        InspectorImageUse::Region => {
            y = super::anim_card::button_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                y,
                ids::INSP_IMAGE_EDIT_REGION,
                tr("panel.inspector.image.edit_region"),
            );
            match info.draw {
                InspectorDraw::Region { px } => super::rows::aviso(
                    scene,
                    text_system,
                    theme,
                    x,
                    w,
                    y,
                    &tr_with(
                        "panel.inspector.image.region_px",
                        &[("w", &px[0]), ("h", &px[1])],
                    ),
                    ColorToken::Text2,
                ),
                _ => y,
            }
        }
        InspectorImageUse::Animated => {
            let sec = super::sprite_pose::seccao(text_system);
            y = super::sprite_pose::paint_sheet_grid_rows(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                y,
                info.hframes,
                info.vframes,
                false,
                sec,
            );
            match anim {
                Some((an, selected)) => super::anim::paint_anim_section(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x,
                    w,
                    y,
                    an,
                    selected,
                ),
                None => y,
            }
        }
    }
}

/// **Width / Height** de uma sprite que repete (plano GEF 14 D5): o uso É a caixa Tiled de antes.
#[allow(clippy::too_many_arguments)]
fn paint_size_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
) -> f32 {
    const NOMES: [&str; 2] = [
        "panel.inspector.image.width_m",
        "panel.inspector.image.height_m",
    ];
    if !matches!(info.draw, InspectorDraw::Tiled { .. }) {
        return y;
    }
    let sec = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &NOMES.map(tr));
    let mut y = y;
    for (id, nome) in ids::INSP_IMAGE_TILED_SIZE.into_iter().zip(NOMES) {
        y = super::sprite_pose::number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y,
            sec,
            TextKey::new(nome).into(),
            id,
        );
    }
    y
}
