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
    decode_shape_follow_option, decode_stroke_method_option, decode_texture_kind_option,
    decode_texture_mapping_option, decode_texture_ramp_alpha_option,
};
use ph2d_editor_core::ids as core_ids;
use ph2d_tool_painter::ids::{
    PAINTER_SHAPE_FOLLOW_MODES, painter_brush_stroke_method_option_id,
    painter_brush_texture_kind_option_id, painter_brush_texture_mapping_option_id,
    painter_brush_texture_ramp_alpha_option_id, painter_shape_follow_option_id,
};
use ph2d_tool_painter::{RampAlphaMode, StrokeMethod, TextureKind, TextureMapping};

#[test]
fn paper_kind_option_ids_round_trip_and_dont_collide_with_grain() {
    use super::decode::decode_paper_kind_option;
    use ph2d_tool_painter::ids::painter_paper_kind_option_id;
    for k in 0..TextureKind::COUNT {
        assert_eq!(
            decode_paper_kind_option(painter_paper_kind_option_id(k)),
            Some(k)
        );
        // The Paper slot + the Grain slot share the TextureKind enum but must NOT collide on ids.
        assert!(
            decode_paper_kind_option(
                ph2d_tool_painter::ids::painter_brush_texture_kind_option_id(k)
            )
            .is_none()
        );
    }
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
        decode_texture_kind_option(ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_KIND),
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
        decode_texture_mapping_option(ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_MAPPING),
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
        decode_shape_follow_option(ph2d_tool_painter::ids::PAINTER_SHAPE_FOLLOW),
        None,
        "the chip id is not an option id"
    );
    // Disjoint from the sibling Shape Kind option-id space (both live in the Shape section).
    assert_eq!(
        decode_shape_follow_option(ph2d_tool_painter::ids::painter_shape_kind_option_id(2)),
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
        decode_texture_ramp_alpha_option(
            ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE
        ),
        None,
        "the chip id is not an option id"
    );
}

/// B39.1 (DF-70, T1-16): with Monochrome on the panel shows ONE row (Gray = the engine's
/// `red_out`), but the slider edit used the last picked tab — Gray's Red bar wrote the Blue row
/// and nothing on the canvas moved.
#[test]
fn a_barra_do_gray_no_monochrome_escreve_a_linha_que_o_motor_le() {
    use ph2d_tool_painter::{AdjustmentKind, AdjustmentParams, LayerKind, LayerStack};
    let mut stack = LayerStack::new();
    let id = stack
        .add_adjustment(AdjustmentKind::ChannelMixer)
        .expect("layer");
    if let Some(l) = stack.get_mut(id)
        && let LayerKind::Adjustment(adj) = &mut l.kind
        && let AdjustmentParams::ChannelMixer(mx) = &mut adj.params
    {
        mx.monochromatic = true;
    }
    crate::state::set_active_mixer_channel(id.0, 2);
    assert_eq!(
        super::mixer_edit_output(&stack, id),
        0,
        "Monochrome: the Gray bars must write red_out (the row the engine reads), not the Blue tab"
    );
    if let Some(l) = stack.get_mut(id)
        && let LayerKind::Adjustment(adj) = &mut l.kind
        && let AdjustmentParams::ChannelMixer(mx) = &mut adj.params
    {
        mx.monochromatic = false;
    }
    assert_eq!(
        super::mixer_edit_output(&stack, id),
        2,
        "colour: the picked tab"
    );
}

/// B39.2 (DF-71): the "−" sent the SELECTED id even after that point was gone — the 2nd click did
/// nothing.
#[test]
fn o_menos_do_falloff_apaga_dois_pontos_seguidos() {
    use ph2d_tool_painter::FalloffPoint;
    let ponto = |id, x| FalloffPoint {
        id,
        x,
        y: 1.0 - x,
        ..crate::paint_brush::FALLBACK_BRUSH.falloff_points[0]
    };
    let mut b = crate::paint_brush::FALLBACK_BRUSH;
    b.falloff_points[..4].copy_from_slice(&[
        ponto(0, 0.0),
        ponto(1, 0.3),
        ponto(2, 0.6),
        ponto(3, 1.0),
    ]);
    b.falloff_len = 4;
    crate::state::set_current_brush(Some(b));
    crate::state::set_selected_falloff_point(Some(2));
    assert_eq!(
        super::falloff_remove_target(),
        Some(2),
        "1st click: the selected point"
    );
    // The tool drops point 2 and republishes the brush.
    b.falloff_points[2] = ponto(3, 1.0);
    b.falloff_len = 3;
    crate::state::set_current_brush(Some(b));
    assert_eq!(
        super::falloff_remove_target(),
        Some(1),
        "2nd click: the gone point 2 is no target — the first interior point is"
    );
    crate::state::set_selected_falloff_point(None);
    crate::state::set_current_brush(None);
}

/// B39.2 (DF-72): after "−" on the last stop the selection pointed past the end; the panel showed
/// the (clamped) last stop but R/G/B wrote the missing one — the sliders went dead.
#[test]
fn o_rgb_do_gradient_map_depois_de_tirar_a_ultima_paragem_mexe_a_paragem_vista() {
    use ph2d_tool_painter::{AdjustmentKind, AdjustmentParams, LayerKind, LayerStack};
    let mut stack = LayerStack::new();
    let id = stack
        .add_adjustment(AdjustmentKind::GradientMap)
        .expect("layer");
    let n = match stack.get(id).map(|l| &l.kind) {
        Some(LayerKind::Adjustment(adj)) => match &adj.params {
            AdjustmentParams::GradientMap(g) => g.stops.len(),
            _ => 0,
        },
        _ => 0,
    };
    assert!(n >= 2);
    crate::state::set_selected_gradient_stop(id.0, n);
    assert_eq!(
        super::gradient_edit_stop(&stack, id),
        n - 1,
        "the stop the panel shows"
    );
}

/// B39.3 (DF-76): closing the Painter closed only the brush-colour picker; a picker opened by any
/// other Painter swatch (paper, ramp stop, shape layer, impasto…) stayed floating on the canvas.
#[test]
fn fechar_o_painter_fecha_o_seletor_de_qualquer_amostra_dele() {
    use ph2d_editor_core::panel::{Panel, PanelHostInternal};
    use ph2d_ui_testkit::MockPanelHost;
    let sw = ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB;
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    assert_eq!(
        super::picker::try_toggle_shared_picker(&mut host, sw),
        Some(true)
    );
    assert_eq!(host.store().picker_target(), Some(sw));
    let fechado = |host: &mut MockPanelHost| {
        let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 360.0, 800.0);
        let layout = ph2d_editor_core::screens::HeroLayout::for_viewport(viewport);
        let mut scene = ph2d_vector::VectorScene::new();
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        host.set_panel_visible(crate::PainterLayersPanel::ID, false);
        let mut ctx = ph2d_editor_core::panel::PaintCtx {
            host,
            layout: &layout,
            slot: viewport,
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        crate::paint::paint(&mut crate::PainterLayersPanelState, &mut ctx);
    };
    fechado(&mut host);
    assert_eq!(host.store().picker_target(), None, "the Painter is closed");
    // A picker the Painter did NOT open (the Inspector's) stays.
    let alheio = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
    host.store_mut().set_picker_target(Some(alheio));
    fechado(&mut host);
    assert_eq!(host.store().picker_target(), Some(alheio));
}
