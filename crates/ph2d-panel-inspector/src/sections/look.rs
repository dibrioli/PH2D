//! Os corpos dos cartões **Look** e **Advanced** da Sprite
//! ([ficha](../../../../docs/GameEngineFuncional/componentes/01_sprite.md)): o que era Texture & Atlas e Color & Tint,
//! partido por pergunta. Os ids e o despacho são os de sempre.

use super::*;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

const BRANCO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const OPACIDADE_CHEIA: f64 = 100.0; // LITERAL-PX-OK: neutro de um campo em %, não geometria

thread_local! {
    /// «Tint each corner» ligado à mão, ainda com os quatro cantos brancos.
    static CANTOS_ABERTOS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Tint e Opacity lado a lado. Self Tint só existe para quem tem filhos, ou já a usa: sem filhos dá a mesma cor.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_cores(
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
    let com_self = crate::slot_image::tem_filhos(info.entity_bits) || info.self_tint != BRANCO;
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("component.field.sprite.1"),
            tr("component.field.sprite.6"),
        ],
    );
    let [(xa, wa), (xb, wb)] = ph2d_editor_core::panel::cards::corpo::colunas_iguais(x, w);
    let fim_tint = super::color_tint::paint_tint_swatch_cell(
        Rect::new(xa, y, wa, ROW_H_PX),
        TextKey::new("component.field.sprite.1"),
        ids::INSP_SPRITE_TINT_SWATCH,
        crate::state_tint::tint_f32_to_u8(info.tint),
        info.mixed.tint,
        store,
        hit_index,
        scene,
        text_system,
        theme,
        sec,
    );
    let mut y = ph2d_editor_core::panel::cards::corpo::bar_field(
        scene,
        text_system,
        store,
        hit_index,
        ph2d_editor_core::widget::card_ink::card_body().expect("só num cartão"),
        xb,
        wb,
        y,
        tr("card.insp.opacity"),
        ids::INSP_SPRITE_OPACITY_CHIP,
        ids::INSP_SPRITE_OPACITY,
        ph2d_editor_core::widget::card_field::CardFieldSpec {
            step: 1.0,
            scrub: 1.0,
            default: Some(OPACIDADE_CHEIA),
        },
    )
    .max(fim_tint);
    if com_self {
        y = super::color_tint::paint_tint_swatch_cell(
            Rect::new(x, y, w, ROW_H_PX),
            TextKey::new("component.field.sprite.6"),
            ids::INSP_SPRITE_SELF_TINT_SWATCH,
            crate::state_tint::tint_f32_to_u8(info.self_tint),
            info.mixed.self_tint,
            store,
            hit_index,
            scene,
            text_system,
            theme,
            sec,
        );
    }
    y
}

/// Os cantos estão em uso: algum deles não é branco, ou a mão ligou a caixa.
pub(crate) fn cantos_ligados(info: &InspectorSpriteInfo) -> bool {
    info.mixed.per_corner
        || info.per_corner_tint.iter().any(|c| *c != BRANCO)
        || CANTOS_ABERTOS.with(std::cell::Cell::get)
}

/// A caixa «Tint each corner»: ligar abre os quatro; desligar devolve-os ao branco. `true` = consumido.
pub(crate) fn evento_dos_cantos(
    host: &mut dyn ph2d_editor_core::panel::PanelHostInternal,
    ev: ph2d_editor_core::interaction::WidgetEvent,
) -> bool {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    let WidgetEvent::Toggled(id) = ev else {
        return false;
    };
    if id != ids::INSP_SPRITE_CORNERS_ON {
        return false;
    }
    let Some(info) = crate::state::current_inspector_sprite() else {
        return true;
    };
    let ligado = matches!(
        host.store().checkbox(id).map(|(_, v)| v),
        Some(CheckboxValue::Checked)
    );
    CANTOS_ABERTOS.with(|c| c.set(ligado));
    if !ligado {
        for i in 0..4_u8 {
            host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                entity_bits: info.entity_bits,
                edit: ph2d_editor_core::SpriteFieldEdit::PerCornerTintAt(i, BRANCO),
            });
        }
    }
    true
}

/// O corpo do Advanced: memória e os efeitos raros. O que não decide nada (Storage, a caixa Region) não está aqui.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_advanced(
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
    let label_font = TypeToken::Sm.px();
    let row_gap = ph2d_tokens::control_gap_px();
    let mut y = super::render_source::paint_strategy_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        info,
        x,
        w,
        y,
        label_font,
    );
    // Uma área antiga, ligada pela caixa de antes, continua editável; nenhuma sprite nova chega aqui.
    let tem_area = matches!(
        store.checkbox(ids::INSP_REGION_ENABLED).map(|(_, v)| v),
        Some(CheckboxValue::Checked)
    );
    if tem_area && info.image_use != ph2d_editor_core::InspectorImageUse::Region {
        y = super::render_source_regiao::paint_region_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            info,
            x,
            w,
            y,
            label_font,
            row_gap,
        );
    }
    y = super::render_source_precision::paint_precision_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        info,
        x,
        w,
        y,
        label_font,
    );
    y = super::emissive_row::paint_emissive_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
    );
    y = super::render_source_precision::paint_precision_price(
        scene,
        text_system,
        theme,
        info,
        x,
        w,
        y,
    );
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("card.insp.silhouette"), tr("card.insp.tint_each_corner")],
    );
    let silhueta = matches!(
        store.checkbox(ids::INSP_SPRITE_TINT_FILL).map(|(_, v)| v),
        Some(CheckboxValue::Checked)
    );
    y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (
            ids::INSP_SPRITE_TINT_FILL,
            Rotulo::de_campo(
                tr("card.insp.silhouette"),
                TextKey::new("component.field.sprite.8"),
            ),
            silhueta,
        ),
        sec,
    );
    let cantos = cantos_ligados(info);
    y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (
            ids::INSP_SPRITE_CORNERS_ON,
            Rotulo::vista(TextKey::new("card.insp.tint_each_corner")),
            cantos,
        ),
        sec,
    );
    if cantos {
        y = super::color_tint::paint_per_corner_tab(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y,
            Some(info),
        );
    }
    super::render_source::paint_reimport(scene, text_system, hit_index, store, theme, info, x, w, y)
}
