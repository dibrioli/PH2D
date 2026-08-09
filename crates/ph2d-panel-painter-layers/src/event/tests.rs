//! Regression tests for the Brush-section dropdown-option decoders. The Stroke Method decoder once
//! used `0..7`, which silently dropped the PH2D shape extensions (Ellipse = 7, then Polygon = 8) —
//! clicking them in the dropdown did nothing.
//!
//! ⚠️ **E o gate reincidiu junto com o produto**, porque ele também trazia um literal (`0..9`): o
//! `FreeHand` (9) nunca foi coberto, e quando o `GridStamp` (10) chegou o decodificador voltou a
//! largar a última opção com a suíte **verde**. Uma varredura que escolhe o próprio fim não pode
//! provar que o fim está certo — as duas pontas agora perguntam ao enum (`StrokeMethod::COUNT`), e
//! quem prende o `COUNT` ao número REAL de métodos é o gate irmão em `ph2d-painter-brush`.

use super::decode::{
    decode_brush_preset_option, decode_shape_follow_option, decode_stroke_method_option,
    decode_texture_kind_option, decode_texture_mapping_option, decode_texture_ramp_alpha_option,
};
use super::*;
use ph2d_editor_core::ids::{
    PAINTER_SHAPE_FOLLOW_MODES, painter_brush_preset_option_id,
    painter_brush_stroke_method_option_id, painter_brush_texture_kind_option_id,
    painter_brush_texture_mapping_option_id, painter_brush_texture_ramp_alpha_option_id,
    painter_shape_follow_option_id,
};
use ph2d_tool_painter::{RampAlphaMode, StrokeMethod, TextureKind, TextureMapping};

#[test]
fn paper_kind_option_ids_round_trip_and_dont_collide_with_grain() {
    use super::decode::decode_paper_kind_option;
    use ph2d_editor_core::ids::painter_paper_kind_option_id;
    for k in 0..TextureKind::COUNT {
        assert_eq!(
            decode_paper_kind_option(painter_paper_kind_option_id(k)),
            Some(k)
        );
        // The Paper slot + the Grain slot share the TextureKind enum but must NOT collide on ids.
        assert!(
            decode_paper_kind_option(core_ids::painter_brush_texture_kind_option_id(k)).is_none()
        );
    }
}

#[test]
fn every_preset_option_id_round_trips() {
    // Both presets (Digital = 0, Watercolor = 1) must decode back to themselves.
    for i in 0u8..core_ids::PAINTER_BRUSH_PRESET_COUNT {
        assert_eq!(
            decode_brush_preset_option(painter_brush_preset_option_id(i)),
            Some(i),
            "preset option id {i} did not decode back"
        );
    }
    assert_eq!(
        decode_brush_preset_option(core_ids::PAINTER_BRUSH_PRESET),
        None,
        "the chip id is not an option id"
    );
}

#[test]
fn every_stroke_method_option_id_round_trips() {
    // Todo método: a faixa é a do ENUM. Um literal aqui só prova o que o autor lembrou de contar.
    for m in 0u8..StrokeMethod::COUNT {
        let id = painter_brush_stroke_method_option_id(m);
        assert_eq!(
            decode_stroke_method_option(id),
            Some(m),
            "o id da opção do método {m} não decodifica de volta — clicar nela no dropdown não faz \
             NADA (a regressão Ellipse = 7 / Polygon = 8 / GridStamp = 10)"
        );
    }
    // A foreign id decodes to None (not a false match).
    assert_eq!(
        decode_stroke_method_option(core_ids::PAINTER_BRUSH_STROKE_METHOD),
        None,
        "the chip id is not an option id"
    );
}

/// **A ponta que os dois gates acima não tocavam: todo método que o MENU OFERECE decodifica.**
///
/// O round-trip prova que a faixa do decodificador cobre o enum; este prova que ela cobre a
/// **lista que o artista de fato vê**. Eram duas listas em módulos diferentes — a do
/// `stroke_method_offer` (o que se pinta) e a faixa do `decode` (o que se entende) —, e nada as
/// obrigava a concordar: a do Digital ganhou o `GridStamp` e a do decodificador não, então a última
/// opção do dropdown pintava, respondia ao mouse e **não fazia nada**.
///
/// ⚠️ A fixture é o pincel **DIGITAL** de propósito: é o ramo com o superset (o único que oferece o
/// método 10), e um `None` cairia no ramo genérico, que não contém o fenômeno.
///
/// **Mutação que deve sangrar:** faixa literal `0..10` no `decode_stroke_method_option`.
#[test]
fn every_method_the_menu_offers_can_be_decoded() {
    let digital = ph2d_tool_painter::PainterTool::default().brush_settings();
    let offered = crate::stroke_method_offer::offered_stroke_methods(Some(&digital));
    assert!(
        offered.contains(&10),
        "controle positivo: o menu do Digital deixou de oferecer o Grid Stamp (10) — este gate \
         estaria passando por vácuo sobre a lista que ele existe para cobrir"
    );
    for &m in offered {
        let id = painter_brush_stroke_method_option_id(m);
        assert_eq!(
            decode_stroke_method_option(id),
            Some(m),
            "o menu oferece o método {m} e o decodificador não o entende — clicar nessa opção não \
             faz NADA, e o único vestígio é um `unhandled event: Click(..)` no log"
        );
    }
}

#[test]
fn stroke_method_option_ids_are_distinct() {
    let ids: Vec<_> = (0u8..StrokeMethod::COUNT)
        .map(painter_brush_stroke_method_option_id)
        .collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "option ids {i} and {j} collide");
        }
    }
}

#[test]
fn every_texture_kind_option_id_round_trips() {
    // All kinds (`0..=COUNT-1`, includes None) must decode back — the range MUST include the last.
    for k in 0u8..TextureKind::COUNT {
        let id = painter_brush_texture_kind_option_id(k);
        assert_eq!(
            decode_texture_kind_option(id),
            Some(k),
            "texture kind {k} did not decode back (last-value-dropped regression)"
        );
    }
    assert_eq!(
        decode_texture_kind_option(core_ids::PAINTER_BRUSH_TEXTURE_KIND),
        None,
        "the chip id is not an option id"
    );
}

#[test]
fn every_texture_mapping_option_id_round_trips() {
    for m in 0u8..TextureMapping::COUNT {
        let id = painter_brush_texture_mapping_option_id(m);
        assert_eq!(
            decode_texture_mapping_option(id),
            Some(m),
            "texture mapping {m} did not decode back"
        );
    }
    assert_eq!(
        decode_texture_mapping_option(core_ids::PAINTER_BRUSH_TEXTURE_MAPPING),
        None,
        "the chip id is not an option id"
    );
}

#[test]
fn every_shape_follow_option_id_round_trips() {
    // The Shape Follow dropdown (Off/Rake/Flow) — a clicked popover option must decode back to its wire
    // value, or that option's click is silently dropped. The chip id itself must NOT decode as an option.
    for &(v, _) in &PAINTER_SHAPE_FOLLOW_MODES {
        assert_eq!(
            decode_shape_follow_option(painter_shape_follow_option_id(v)),
            Some(v),
            "shape follow {v} did not decode back"
        );
    }
    assert_eq!(
        decode_shape_follow_option(core_ids::PAINTER_SHAPE_FOLLOW),
        None,
        "the chip id is not an option id"
    );
    // Disjoint from the sibling Shape Kind option-id space (both live in the Shape section).
    assert_eq!(
        decode_shape_follow_option(core_ids::painter_shape_kind_option_id(2)),
        None,
        "a Shape Kind option id must not decode as a Follow option"
    );
}

#[test]
fn every_ramp_alpha_option_id_round_trips() {
    // All 3 alpha actions (Off / Strength / Sprite) must decode back — the range MUST include the last.
    for m in 0u8..RampAlphaMode::COUNT {
        let id = painter_brush_texture_ramp_alpha_option_id(m);
        assert_eq!(
            decode_texture_ramp_alpha_option(id),
            Some(m),
            "ramp alpha action {m} did not decode back (last-value-dropped regression)"
        );
    }
    assert_eq!(
        decode_texture_ramp_alpha_option(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE),
        None,
        "the chip id is not an option id"
    );
}
