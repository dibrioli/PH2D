//! **O seletor da cor do papel aplica o papel nos quatro meios** (dono, 2026-10-06: *«o botão apply to
//! paper parece supérfluo. não seria melhor aplicar ao usar o próprio seletor de cor?»*) — do clique
//! na amostra do painel ao papel do documento: o dispatcher abre o seletor, a cor escolhida volta ao
//! painel, o painel encaminha-a pelo barramento, a ferramenta aplica o papel.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::ids::{PAINTER_BRUSH_MEDIA, PAINTER_WATERCOLOR_PAPER_COLOR_THUMB};
use ph2d_ui_testkit::MockPanelHost;

fn encaminha(host: &mut MockPanelHost, tool: &mut PainterTool) {
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

#[test]
fn o_seletor_da_cor_do_papel_aplica_o_papel_nos_quatro_meios() {
    for meio in ["0", "1", "2", "3"] {
        let mut tool = PainterTool::default();
        tool.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        tool.handle_panel_event(PanelEvent::SelectOption(PAINTER_BRUSH_MEDIA, meio.into()));
        set_current_brush(Some(tool.brush_settings()));
        set_current_dock_shows_layers(false);
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let vp = Rect::new(0.0, 0.0, 1600.0, 6000.0);
        let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
        host.open_all_sections();
        let pintados = host.paint::<PainterLayersPanel>(&mut st, vp);
        let Some((_, r)) = pintados
            .iter()
            .find(|(id, r)| *id == PAINTER_WATERCOLOR_PAPER_COLOR_THUMB && r.w > 0.0 && r.h > 0.0)
            .copied()
        else {
            panic!("meio {meio}: a cor do papel não é pintada");
        };
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            host.hit_at(x, y),
            Some(PAINTER_WATERCOLOR_PAPER_COLOR_THUMB),
            "meio {meio}: outro widget cobre a amostra"
        );
        // O clique REAL abre o seletor sobre a amostra.
        for ev in host.click_at(x, y) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        encaminha(&mut host, &mut tool);
        assert_eq!(
            tool.papel(),
            None,
            "meio {meio}: abrir o seletor aplicou o papel"
        );
        // A cor escolhida no seletor aberto volta ao painel no quadro seguinte, que a encaminha.
        host.pick_colour_in_the_open_picker([230, 200, 150, 255]);
        let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
        encaminha(&mut host, &mut tool);
        assert_eq!(
            tool.papel(),
            Some([230, 200, 150]),
            "meio {meio}: a cor escolhida no seletor não aplicou o papel"
        );
    }
}

/// A *Paint Texture* (o papel impresso dentro da tinta) só no Wet Paint — desde 28/10 é do TRAÇO e mora no cartão
/// **Wet Paint** (dono: «por traço e não para o papel todo»); o Tooth aparece SEM Kind no Wet Paint, no cartão Paper
/// (lá ele morde o papel de fábrica — esconder um controlo que age é mentir); e o clique REAL na caixa chega à
/// ferramenta. Mutações que sangram: pintar a caixa fora do Wet Paint (aparece no Digital) · voltar a esconder o
/// Tooth sem Kind · tirar o id do braço da ferramenta (o clique não liga nada).
#[test]
fn a_paint_texture_e_do_wet_paint_e_o_tooth_vive_na_seccao_paper() {
    use ph2d_tool_painter::ids::{PAINTER_WATERCOLOR_PAPER_DEPTH, PAINTER_WETPAINT_PAPER_VISUAL};
    for meio in ["0", "1", "2", "3"] {
        let mut tool = PainterTool::default();
        tool.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        tool.handle_panel_event(PanelEvent::SelectOption(PAINTER_BRUSH_MEDIA, meio.into()));
        assert_eq!(
            tool.brush_settings().paper_kind,
            0,
            "a fixture começa sem Kind"
        );
        set_current_brush(Some(tool.brush_settings()));
        set_current_dock_shows_layers(false);
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let vp = Rect::new(0.0, 0.0, 1600.0, 6000.0);
        let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
        host.open_all_sections();
        let pintados = host.paint::<PainterLayersPanel>(&mut st, vp);
        let achar = |id| {
            pintados
                .iter()
                .find(|(i, r)| *i == id && r.w > 0.0 && r.h > 0.0)
                .map(|(_, r)| *r)
        };
        let molhado = meio == "3";
        assert_eq!(
            achar(PAINTER_WETPAINT_PAPER_VISUAL).is_some(),
            molhado,
            "meio {meio}: a Paint Texture é do Wet Paint, e só dele"
        );
        // D1 W3b: o Smooth Edges do Wet Paint é da TELA — mora aqui, no cartão Paper.
        assert_eq!(
            achar(ph2d_tool_painter::ids::PAINTER_WETPAINT_SMOOTH_EDGES).is_some(),
            molhado,
            "meio {meio}: o Smooth Edges do Wet Paint mora no cartão Paper, e só no Wet Paint"
        );
        if !molhado {
            continue;
        }
        assert!(
            achar(PAINTER_WATERCOLOR_PAPER_DEPTH).is_some(),
            "o Tooth tem de aparecer sem Kind no Wet Paint (ele morde o papel de fábrica)"
        );
        let r = achar(PAINTER_WETPAINT_PAPER_VISUAL).expect("pintada");
        let antes = tool.brush_settings().wet_paper_visual;
        for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        encaminha(&mut host, &mut tool);
        assert_ne!(
            tool.brush_settings().wet_paper_visual,
            antes,
            "o clique na Paint Texture não chegou à ferramenta"
        );
    }
}

/// A Paint Texture mora no cartão **Wet Paint** (dono 28/10: é do traço), e não no Paper: o rect da caixa cai dentro do
/// cartão Wet Paint, abaixo do cabeçalho dele e acima do cabeçalho do Paper.
#[test]
fn a_paint_texture_mora_no_cartao_wet_paint() {
    use ph2d_tool_painter::ids::{
        PAINTER_WATERCOLOR_PAPER_SECTION, PAINTER_WETPAINT_PAPER_VISUAL, PAINTER_WETPAINT_SECTION,
    };
    let mut tool = PainterTool::default();
    tool.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    tool.handle_panel_event(PanelEvent::SelectOption(PAINTER_BRUSH_MEDIA, "3".into()));
    set_current_brush(Some(tool.brush_settings()));
    set_current_dock_shows_layers(false);
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let vp = Rect::new(0.0, 0.0, 1600.0, 6000.0);
    let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
    host.open_all_sections();
    let p = host.paint::<PainterLayersPanel>(&mut st, vp);
    let y = |id| {
        p.iter()
            .find(|(i, r)| *i == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| r.y)
            .unwrap_or_else(|| panic!("{id:?} não pintado"))
    };
    let (wet, caixa, papel) = (
        y(PAINTER_WETPAINT_SECTION),
        y(PAINTER_WETPAINT_PAPER_VISUAL),
        y(PAINTER_WATERCOLOR_PAPER_SECTION),
    );
    assert!(
        wet < caixa && caixa < papel,
        "a Paint Texture fora do cartão Wet Paint (Wet Paint y={wet}, caixa y={caixa}, Paper y={papel})"
    );
}
