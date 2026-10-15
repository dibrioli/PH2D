//! A secção Shape Color com as duas camadas (prompt 38) pinta e regista os controlos de cada estado —
//! uma linha não pintada não regista hit rect, logo nenhum clique a alcança — e nenhum se sobrepõe.
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_painter::{BrushSettings, PANEL_RAMP_STOPS, ids};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

/// O corpo do painel pintado com `brush` e a secção Shape Color aberta: cada id com o seu rect.
fn hits(brush: BrushSettings) -> Vec<(NodeId, Rect)> {
    crate::state::set_current_brush(Some(brush));
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    host.store_mut()
        .set_collapsed(ids::PAINTER_SHAPE_RAMP_SECTION, false);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 360.0, 8000.0);
    let layout = HeroLayout::for_viewport(viewport);
    {
        let mut ctx = PaintCtx {
            host: &mut host,
            layout: &layout,
            slot: layout
                .slot_rects(ph2d_editor_core::screens::slot::SlotSet::ANY_DOCK)
                .get(ph2d_editor_core::screens::slot::Slot::RightTop),
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        let rect = Rect::new(0.0, 0.0, 320.0, 8000.0);
        crate::paint_brush::paint_brush_body(&mut ctx, Theme::default(), rect, 0.0);
    }
    crate::state::set_current_brush(None);
    host.hit_index_mut().iter_registrations().collect()
}

fn base() -> BrushSettings {
    let mut stops = [[0.0f32; 6]; PANEL_RAMP_STOPS];
    stops[0] = [0.0, 1.0, 0.0, 0.0, 1.0, 0.0];
    stops[1] = [1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    BrushSettings {
        shape_color_ramp_enabled: true,
        shape_color_ramp_stops: stops,
        shape_color_ramp_stop_count: 2,
        shape_falloff_ramp_stops: stops,
        shape_falloff_ramp_stop_count: 2,
        shape_falloff_color: false,
        ..crate::paint_brush::FALLBACK_BRUSH
    }
}

#[test]
fn as_duas_camadas_pintam_os_controlos_de_cada_estado() {
    let tem = |h: &[(NodeId, Rect)], id| h.iter().any(|(i, _)| *i == id);
    let so_textura = hits(base());
    for id in [
        ids::PAINTER_SHAPE_TEXTURE_COLOR,
        ids::PAINTER_SHAPE_FALLOFF_COLOR,
        ids::PAINTER_SHAPE_COLOR_SWAP,
        ids::PAINTER_SHAPE_RAMP_BW,
        ids::PAINTER_SHAPE_RAMP_SWATCH,
    ] {
        assert!(tem(&so_textura, id), "Texture Color só: falta {id:?}");
    }
    for id in [
        ids::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH,
        ids::PAINTER_SHAPE_COLOR_BLEND,
        ids::PAINTER_SHAPE_FALLOFF_OPACITY,
    ] {
        assert!(
            !tem(&so_textura, id),
            "Texture Color só: {id:?} não devia estar pintado"
        );
    }
    let ambas = hits(BrushSettings {
        shape_falloff_color: true,
        ..base()
    });
    let novos = [
        ids::PAINTER_SHAPE_TEXTURE_COLOR,
        ids::PAINTER_SHAPE_FALLOFF_COLOR,
        ids::PAINTER_SHAPE_COLOR_SWAP,
        ids::PAINTER_SHAPE_FALLOFF_RAMP_ADD,
        ids::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH,
        ids::PAINTER_SHAPE_COLOR_BLEND,
        ids::PAINTER_SHAPE_TEXTURE_OPACITY,
        ids::PAINTER_SHAPE_FALLOFF_OPACITY,
        ids::PAINTER_SHAPE_RAMP_SWATCH,
    ];
    for id in novos {
        assert!(tem(&ambas, id), "as duas camadas: falta {id:?}");
    }
    let rects: Vec<Rect> = novos
        .iter()
        .filter_map(|id| ambas.iter().find(|(i, _)| i == id).map(|(_, r)| *r))
        .collect();
    for (a, ra) in rects.iter().enumerate() {
        for rb in &rects[a + 1..] {
            let cruza = ra.x < rb.x + rb.w
                && rb.x < ra.x + ra.w
                && ra.y < rb.y + rb.h
                && rb.y < ra.y + ra.h;
            assert!(
                !cruza,
                "dois controlos das camadas sobrepõem-se: {ra:?} × {rb:?}"
            );
        }
    }
    // Sem textura no Shape (a resposta REAL da ferramenta): o cartão da Texture Color inteiro sem
    // clique (2.º smoke do dono, 15/10: «deve ficar apagado e inativo completamente»), e sem Blend.
    let sem_textura = hits(BrushSettings {
        shape_falloff_color: true,
        inercias: ph2d_tool_painter::PainterTool::default()
            .brush_settings()
            .inercias,
        ..base()
    });
    assert!(
        tem(&sem_textura, ids::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH),
        "sem textura: a Falloff vive"
    );
    for id in [
        ids::PAINTER_SHAPE_TEXTURE_COLOR,
        ids::PAINTER_SHAPE_RAMP_SWATCH,
        ids::PAINTER_SHAPE_RAMP_ADD,
        ids::PAINTER_SHAPE_TEXTURE_OPACITY,
        ids::PAINTER_SHAPE_COLOR_BLEND,
    ] {
        assert!(
            !tem(&sem_textura, id),
            "sem textura: {id:?} não pode receber clique"
        );
    }
    let bw = hits(BrushSettings {
        shape_color_ramp_bw: true,
        shape_falloff_color: true,
        ..base()
    });
    assert!(
        tem(&bw, ids::PAINTER_SHAPE_RAMP_SWATCH),
        "B&W: o tom tem a sua rampa"
    );
    // Com B&W os DOIS cartões ficam (dono 15/10: «B&W está fechando um dos color ramps. Não permita»).
    for id in [
        ids::PAINTER_SHAPE_TEXTURE_COLOR,
        ids::PAINTER_SHAPE_FALLOFF_COLOR,
        ids::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH,
        ids::PAINTER_SHAPE_COLOR_BLEND,
        ids::PAINTER_SHAPE_FALLOFF_OPACITY,
    ] {
        assert!(tem(&bw, id), "B&W: {id:?} tem de continuar no painel");
    }
}

/// **A pré-visualização da ponta pinta as MESMAS camadas** (2.º smoke do dono, 15/10: com o Shape sem
/// textura ela mostrava a rampa da Texture Color pela cobertura): sem textura, a cor é a da Falloff.
#[test]
fn a_previa_da_ponta_pinta_as_camadas_sem_textura() {
    let mut verde = [[0.0f32; 6]; PANEL_RAMP_STOPS];
    verde[0] = [0.0, 0.0, 1.0, 0.0, 1.0, 0.0];
    verde[1] = [1.0, 0.0, 1.0, 0.0, 1.0, 1.0];
    let b = BrushSettings {
        shape_falloff_color: true,
        shape_falloff_ramp_stops: verde,
        inercias: ph2d_tool_painter::PainterTool::default()
            .brush_settings()
            .inercias,
        ..base()
    };
    crate::state::set_current_brush(Some(b));
    let cov = vec![200u8; 16];
    let cores = crate::paint_shape_previa::camadas_da_previa(
        &b,
        ph2d_tool_painter::TextureKind::None,
        &[1.0; 64],
        None,
        4,
        &cov,
    );
    crate::state::set_current_brush(None);
    let c = cores.expect("a Falloff Color colore a pré-visualização")[0];
    assert!(
        c[1] > 0.8 && c[0] < 0.2,
        "sem textura a prévia tem de ser a Falloff (verde): {c:?}"
    );
}
