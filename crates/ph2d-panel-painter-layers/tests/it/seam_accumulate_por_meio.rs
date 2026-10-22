//! **O Accumulate e o Space Attenuation aparecem só onde o meio os oferece** (doc 46 §1).
//! A lei é a `BrushSettings::accumulate_offered`, a mesma que o traço pergunta; este gate prova que
//! a TELA a segue, meio a meio.

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_tool_painter::ids::{PAINTER_BRUSH_ACCUMULATE, PAINTER_BRUSH_SPACE_ATTEN};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn pintados(media: PaintMedia) -> Vec<NodeId> {
    pintados_com(media, |_| {})
}

fn pintados_com(media: PaintMedia, ajusta: impl Fn(&mut PainterTool)) -> Vec<NodeId> {
    let mut t = PainterTool::default();
    t.set_paint_media(media);
    ajusta(&mut t);
    set_current_brush(Some(t.brush_settings()));
    set_current_dock_shows_layers(false);
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let vp = Rect::new(0.0, 0.0, 1600.0, 6000.0);
    let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
    host.open_all_sections();
    host.paint::<PainterLayersPanel>(&mut st, vp)
        .into_iter()
        .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn o_accumulate_e_o_space_attenuation_so_onde_o_meio_os_oferece() {
    for (media, oferece) in [
        (PaintMedia::Digital, true),
        (PaintMedia::Watercolor, false),
        (PaintMedia::Impasto, false),
        (PaintMedia::WetPaint, false),
    ] {
        let ids = pintados(media);
        for (id, nome) in [
            (PAINTER_BRUSH_ACCUMULATE, "Accumulate"),
            (PAINTER_BRUSH_SPACE_ATTEN, "Space Attenuation"),
        ] {
            assert_eq!(
                ids.contains(&id),
                oferece,
                "{media:?}: o {nome} {} pintado",
                if oferece { "NÃO é" } else { "é" }
            );
        }
    }
}

/// **Com o Solid o Accumulate e o Space Attenuation saem do painel** (decisão do dono, 2026-10-06): a
/// mancha obedece ao tecto do traço, e com o Accumulate ligado o traço passava dele (`0,744` contra a
/// mancha a `0,400`). No Digital, onde o meio os oferece; e voltam ao desmarcar o Solid.
#[test]
fn com_solid_o_accumulate_sai_do_painel() {
    for solid in [true, false] {
        let ids = pintados_com(PaintMedia::Digital, |t| {
            if solid {
                t.toggle_style_solid();
            }
        });
        for (id, nome) in [
            (PAINTER_BRUSH_ACCUMULATE, "Accumulate"),
            (PAINTER_BRUSH_SPACE_ATTEN, "Space Attenuation"),
        ] {
            assert_eq!(
                ids.contains(&id),
                !solid,
                "Digital com Solid {solid}: o {nome} {} pintado",
                if solid { "é" } else { "NÃO é" }
            );
        }
    }
}

/// **A Shape Color aparece em TODO meio** (prompt 41, dono 15/10). No Wet Paint esteve escondida
/// (doc 46 §1: «a mistura K–M homogeneíza o carimbo no primeiro passo») — análise que a régua refutou:
/// nos 12 pincéis de fábrica a cor por célula sobrevive aos 10 s da água (×3,4–18 o desvio de G−R do
/// traço sem rampa, `wetpaint/shape_color_tests.rs::sonda_shape_color_no_wet_paint`).
#[test]
fn a_rampa_da_shape_aparece_em_todo_meio() {
    use ph2d_tool_painter::ids::{PAINTER_SHAPE_RAMP_ENABLE, PAINTER_SHAPE_RAMP_SECTION};
    for (media, oferece) in [
        (PaintMedia::Digital, true),
        (PaintMedia::Watercolor, true),
        (PaintMedia::Impasto, true),
        (PaintMedia::WetPaint, true),
    ] {
        let ids = pintados(media);
        for (id, nome) in [
            (PAINTER_SHAPE_RAMP_SECTION, "cabeçalho Shape Color"),
            (PAINTER_SHAPE_RAMP_ENABLE, "Use Color Ramp da Shape"),
        ] {
            assert_eq!(
                ids.contains(&id),
                oferece,
                "{media:?}: o {nome} {} pintado",
                if oferece { "NÃO é" } else { "é" }
            );
        }
    }
}

/// **As Grain Colors aparecem nos QUATRO meios** (dono 22/10) — o valor do grão indexa a cor em cada
/// um (no Digital e no Impasto o grão do carimbo, na Aquarela o mapa de granulação, no Wet Paint o
/// grão do dab; `grain_colors_tests` na ferramenta). Com um Grain escolhido (sem ele a rampa não se
/// pinta); onde o meio não lê o grão a linha esmaece (`Dependente::CorDoGrao*`).
#[test]
fn a_rampa_do_grain_aparece_nos_quatro_meios() {
    use ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE;
    for (media, oferece) in [
        (PaintMedia::Digital, true),
        (PaintMedia::Watercolor, true),
        (PaintMedia::Impasto, true),
        (PaintMedia::WetPaint, true),
    ] {
        let ids = pintados_com(media, |t| t.set_brush_texture_kind(1));
        assert_eq!(
            ids.contains(&PAINTER_BRUSH_TEXTURE_RAMP_ENABLE),
            oferece,
            "{media:?}: o Use Color Ramp do Grain {} pintado",
            if oferece { "NÃO é" } else { "é" }
        );
    }
}
