//! **O bloco de presets do Wet Painter, pelo ponteiro** (W1, `docs/Painter/NewFeaturesWetPaint/02 §5`).
//!
//! Cada controlo novo é accionado como o artista o acciona — clique no chip, repintura (o popover só
//! regista as opções quando é pintado), clique na opção; dígitos e Enter numa caixa — e o efeito é lido
//! no TOOL: o pincel mudou, ou o pedido que a ponte da app vai atender ficou registado. Um id que pinta
//! mas não está no `populate`, no `event.rs` ou na tabela de opções fica morto sob o rato e reprova aqui.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_panel_painter_layers::{
    PainterLayersPanel, VistaPreset, VistaPresets, set_current_wet_presets, set_current_wet_preview,
};
use ph2d_tool_painter::ids;
use ph2d_tool_painter::{PaintMedia, PainterTool, PedidoPreset};
use ph2d_ui_testkit::MockPanelHost;
use std::sync::Arc;

const ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
const WASH: &str = "WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png";

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 1400.0)
}

fn vista(ativo: Option<&str>) -> Arc<VistaPresets> {
    let e = |rel: &str, nome: &str| VistaPreset {
        relativo: rel.to_owned(),
        nome: nome.to_owned(),
        grupo: "Watercolor · Fluid".to_owned(),
        // O Round é favorito: o menu mostra-o no topo e NÃO no grupo (cada pincel uma vez — dono 09/10).
        favorito: rel == ROUND,
    };
    Arc::new(VistaPresets {
        entradas: vec![e(ROUND, "Wet Round"), e(WASH, "Wet Wash")],
        ativo: ativo.map(str::to_owned),
        politica: 0,
        ilegiveis: 0,
    })
}

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t
}

fn painted(
    tool: &PainterTool,
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(tool.brush_settings()));
    host.paint::<PainterLayersPanel>(st, viewport())
}

fn novo(
    tool: &PainterTool,
    ativo: Option<&str>,
) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_wet_presets(Some(vista(ativo)));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = painted(tool, &mut host, &mut st);
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn entrega(host: &mut MockPanelHost, tool: &mut PainterTool) {
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

fn clica(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    r: Rect,
) {
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    entrega(host, tool);
}

/// Abre o chip `chip` e clica a opção `opcao` — os dois passos de um dropdown.
fn escolhe(tool: &mut PainterTool, ativo: Option<&str>, chip: NodeId, opcao: NodeId) {
    let (mut host, mut st, rects) = novo(tool, ativo);
    let c = rect_of(&rects, chip).unwrap_or_else(|| panic!("o chip {chip:?} não está pintado"));
    clica(&mut host, &mut st, tool, c);
    let rects = painted(tool, &mut host, &mut st);
    let o = rect_of(&rects, opcao)
        .unwrap_or_else(|| panic!("clicar o chip {chip:?} não abriu a opção {opcao:?}"));
    clica(&mut host, &mut st, tool, o);
}

fn digita(tool: &mut PainterTool, id: NodeId, texto: &str) {
    let (mut host, mut st, _) = novo(tool, Some(ROUND));
    for ev in host.type_into_number(id, texto) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    entrega(&mut host, tool);
}

#[test]
fn o_size_px_e_o_diametro_do_anel() {
    let mut t = molhado();
    digita(&mut t, ids::PAINTER_WETPAINT_VOL_SIZE, "40");
    assert_eq!(t.brush_settings().size_px, 20.0, "Size px 40 = raio 20");
    let (_, _, rects) = novo(&t, Some(ROUND));
    assert!(
        rect_of(&rects, ids::PAINTER_BRUSH_SIZE_SLIDER).is_none(),
        "no Wet Paint o Size normalizado do topo sai: um só controlo do raio"
    );
}

#[test]
fn o_water_e_um_volume_e_mexe_no_knob() {
    let mut t = molhado();
    digita(&mut t, ids::PAINTER_WETPAINT_WATER, "0.3");
    assert!(
        (t.wet_knobs().water - 0.3).abs() < 1e-9,
        "{}",
        t.wet_knobs().water
    );
}

/// **A Opacity do Wet Paint** (a claridade do Rebelle, 10/10): o campo e o valor rápido chegam ao pincel
/// — a Opacity DELE, não a força partilhada entre os meios.
#[test]
fn a_opacity_e_um_volume_do_wet_paint() {
    let mut t = molhado();
    let forca = t.brush_settings().strength;
    digita(&mut t, ids::PAINTER_WETPAINT_VOL_OPACITY, "0.5");
    assert!(
        (t.brush_settings().wet_opacidade - 0.5).abs() < 1e-6,
        "{}",
        t.brush_settings().wet_opacidade
    );
    assert_eq!(
        t.brush_settings().strength,
        forca,
        "a força partilhada não mexe"
    );
    let i100 = ids::VOL_QUICK_UNIT.iter().position(|&v| v == 1.0).unwrap() as u8;
    escolhe(
        &mut t,
        Some(ROUND),
        ids::PAINTER_WETPAINT_VOL_QUICK[2],
        ids::painter_wetpaint_vol_quick_option_id(2, i100),
    );
    assert_eq!(t.brush_settings().wet_opacidade, 1.0, "o rápido «100 %»");
}

#[test]
fn os_valores_rapidos_mudam_o_volume() {
    let mut t = molhado();
    let i100 = ids::VOL_QUICK_SIZE_PX
        .iter()
        .position(|&v| v == 100.0)
        .unwrap() as u8;
    escolhe(
        &mut t,
        Some(ROUND),
        ids::PAINTER_WETPAINT_VOL_QUICK[0],
        ids::painter_wetpaint_vol_quick_option_id(0, i100),
    );
    assert_eq!(
        t.brush_settings().size_px,
        50.0,
        "o rápido «100 px» é o diâmetro"
    );
    let i50 = ids::VOL_QUICK_UNIT.iter().position(|&v| v == 0.5).unwrap() as u8;
    escolhe(
        &mut t,
        Some(ROUND),
        ids::PAINTER_WETPAINT_VOL_QUICK[1],
        ids::painter_wetpaint_vol_quick_option_id(1, i50),
    );
    assert!(
        (t.wet_knobs().water - 0.5).abs() < 1e-6,
        "{}",
        t.wet_knobs().water
    );
}

#[test]
fn escolher_um_preset_no_navegador_pede_esse_preset() {
    for secao in [0u8, 2] {
        let mut t = molhado();
        let alvo = if secao == 0 { ROUND } else { WASH };
        escolhe(
            &mut t,
            Some(ROUND),
            ids::PAINTER_WETPAINT_PRESET,
            ids::painter_wetpaint_preset_option_id(secao, alvo),
        );
        assert_eq!(
            t.take_wet_preset_request(),
            Some(PedidoPreset::Escolher(alvo.to_owned())),
            "secção {secao}"
        );
    }
}

#[test]
fn os_botoes_do_preset_pedem_o_que_dizem() {
    for (id, pedido) in [
        (
            ids::PAINTER_WETPAINT_PRESET_FAVORITE,
            PedidoPreset::AlternarFavorito,
        ),
        (
            ids::PAINTER_WETPAINT_PRESET_SAVE_NEW,
            PedidoPreset::GuardarComoNovo,
        ),
        (
            ids::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT,
            PedidoPreset::GuardarComoPadrao,
        ),
        (ids::PAINTER_WETPAINT_PRESET_RESET, PedidoPreset::Repor),
    ] {
        let mut t = molhado();
        let (mut host, mut st, rects) = novo(&t, Some(ROUND));
        let r =
            rect_of(&rects, id).unwrap_or_else(|| panic!("{pedido:?}: o botão não está pintado"));
        clica(&mut host, &mut st, &mut t, r);
        assert_eq!(t.take_wet_preset_request(), Some(pedido));
    }
}

#[test]
fn a_politica_dos_volumes_e_um_pedido() {
    let mut t = molhado();
    escolhe(
        &mut t,
        Some(ROUND),
        ids::PAINTER_WETPAINT_PRESET_POLICY,
        ids::painter_wetpaint_policy_option_id(2),
    );
    assert_eq!(t.take_wet_preset_request(), Some(PedidoPreset::Politica(2)));
}

#[test]
fn sem_preset_activo_so_ha_guardar_como_novo() {
    let t = molhado();
    let (_, _, rects) = novo(&t, None);
    assert!(rect_of(&rects, ids::PAINTER_WETPAINT_PRESET_SAVE_NEW).is_some());
    for id in [
        ids::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT,
        ids::PAINTER_WETPAINT_PRESET_RESET,
        ids::PAINTER_WETPAINT_PRESET_FAVORITE,
    ] {
        assert!(
            rect_of(&rects, id).is_none(),
            "{id:?} sem preset activo seria um botão sem alvo"
        );
    }
}

/// ⭐ L1 (`docs/Painter/Pinceis/03_MOTORES.md` §2): o navegador abre nos QUATRO meios — fora do Wet Paint sem
/// os volumes vivos (o Size e a Strength têm as fileiras deles no corpo) e com o pincel de partida do meio
/// («Basic brush») por baixo. Era `fora_do_wet_paint_o_bloco_nao_aparece`, a decisão que a L1 revogou.
#[test]
fn fora_do_wet_paint_o_navegador_aparece_sem_os_volumes() {
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
    ] {
        let mut t = PainterTool::default();
        t.set_paint_media(meio);
        let (_, _, rects) = novo(&t, Some(ROUND));
        assert!(
            rect_of(&rects, ids::PAINTER_WETPAINT_PRESET).is_some(),
            "{meio:?}"
        );
        assert!(
            rect_of(&rects, ids::PAINTER_WETPAINT_PRESET_SAVE_NEW).is_some(),
            "{meio:?}"
        );
        assert!(
            rect_of(&rects, ids::PAINTER_WETPAINT_PRESET_POLICY).is_some(),
            "{meio:?}"
        );
        for vol in [
            ids::PAINTER_WETPAINT_VOL_SIZE,
            ids::PAINTER_WETPAINT_VOL_OPACITY,
        ] {
            assert!(
                rect_of(&rects, vol).is_none(),
                "{meio:?}: volume do Wet fora dele"
            );
        }
        assert!(
            rect_of(&rects, ids::PAINTER_BRUSH_PRESET).is_some(),
            "{meio:?}: o pincel de partida do meio fica"
        );
    }
}

/// Escolher no navegador FORA do Wet Paint pede o preset (a ponte troca o meio ao aplicá-lo).
#[test]
fn escolher_no_navegador_fora_do_wet_paint_pede_o_preset() {
    let mut t = PainterTool::default();
    escolhe(
        &mut t,
        None,
        ids::PAINTER_WETPAINT_PRESET,
        ids::painter_wetpaint_preset_option_id(2, WASH),
    );
    assert_eq!(
        t.take_wet_preset_request(),
        Some(PedidoPreset::Escolher(WASH.to_owned()))
    );
}

/// O preview VIVO ocupa a sua faixa por baixo do Favorite (e os botões descem com ela); sem quadro
/// publicado não há faixa nenhuma. Com ou sem biblioteca.
#[test]
fn o_preview_vivo_tem_a_sua_faixa_no_bloco() {
    let t = molhado();
    set_current_wet_preview(None);
    let (_, _, sem) = novo(&t, Some(ROUND));
    set_current_wet_preview(Some((Arc::new(vec![200; 256 * 128 * 4]), 256, 128)));
    let (_, _, com) = novo(&t, Some(ROUND));
    let y = |r: &[(NodeId, Rect)]| rect_of(r, ids::PAINTER_WETPAINT_PRESET_SAVE_NEW).unwrap().y;
    assert!(
        y(&com) > y(&sem) + 40.0,
        "a faixa do preview não ocupou lugar: {} → {}",
        y(&sem),
        y(&com)
    );
    set_current_wet_preview(None);
}

/// **Cada pincel aparece UMA vez no menu** (dono 09/10, captura: o «Recent» e o grupo repetiam os quatro
/// nomes). Aberto o chip: o Round (favorito) está nos Favoritos e não no grupo; o Wash está só no grupo; a
/// secção 1 (o antigo «Recent») não pinta nada.
#[test]
fn cada_pincel_aparece_uma_vez_no_menu() {
    let t = molhado();
    let (mut host, mut st, rects) = novo(&t, Some(ROUND));
    let chip = rect_of(&rects, ids::PAINTER_WETPAINT_PRESET).expect("o chip do Preset");
    for ev in host.click_at(chip.x + chip.w * 0.5, chip.y + chip.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    let rects = painted(&t, &mut host, &mut st);
    let pintado = |secao: u8, rel: &str| {
        rect_of(&rects, ids::painter_wetpaint_preset_option_id(secao, rel)).is_some()
    };
    assert!(pintado(0, ROUND), "o favorito no topo");
    assert!(pintado(2, WASH), "o resto no grupo");
    assert!(!pintado(2, ROUND), "o favorito repetido no grupo");
    assert!(!pintado(0, WASH), "um não-favorito nos Favoritos");
    for rel in [ROUND, WASH] {
        assert!(!pintado(1, rel), "a secção «Recent» voltou: {rel}");
    }
}
