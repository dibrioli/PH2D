//! P8: as opções do Text no painel PIXEL — cada uma carregada no MEIO, pelo rato de verdade, chega
//! às opções (o app refaz o preview com elas: `ph2d-app-pixel/tests/it/texto.rs`).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::{PixelOptions, PixelPanelInfo, PixelRequest, PixelTool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

fn viewport(w: f32) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w,
        h: 4000.0, // LITERAL-PX-OK: alto o bastante para o painel inteiro caber sem rolar
    }
}

/// Pinta o painel com as opções `o` e carrega no meio de `id`.
fn carrega(o: PixelOptions, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(PixelPanelInfo {
        editing: true,
        tool: PixelTool::Text,
        options: o,
        text_font_name: "cozette".to_owned(),
        ..Default::default()
    });
    let rects = host.paint::<PixelToolsPanel>(&mut st, viewport(WidgetStore::DOCK_W_MIN));
    let r = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
    let ev = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) | WidgetEvent::Toggled(c) if *c == id)),
        "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
    );
    for e in ev {
        let _ = host.apply_panel_event::<PixelToolsPanel>(&mut st, e);
    }
    host.drained_actions()
}

fn so(a: &[EditorAction], o: PixelOptions) {
    assert_eq!(a, [EditorAction::Pixel(PixelRequest::Options(Box::new(o)))]);
}

#[test]
fn each_text_option_clicked_in_the_panel_reaches_the_options() {
    let base = PixelOptions::default();
    assert!(
        !base.text_antialias && base.text_hinting,
        "as omissões do Aseprite"
    );
    let com = |f: fn(&mut PixelOptions)| {
        let mut o = base;
        f(&mut o);
        o
    };
    so(&carrega(base, ids::TEXT_BOLD), com(|o| o.text_bold = true));
    so(
        &carrega(base, ids::TEXT_ITALIC),
        com(|o| o.text_italic = true),
    );
    so(
        &carrega(base, ids::TEXT_ANTIALIAS),
        com(|o| o.text_antialias = true),
    );
    so(
        &carrega(base, ids::TEXT_HINTING),
        com(|o| o.text_hinting = false),
    );
}

/// P8b: o chip «Font» abre a lista com uma linha por fonte, na ordem publicada; carregar no meio de
/// uma linha pede as opções com essa fonte e fecha a lista.
#[test]
fn picking_a_row_of_the_font_dropdown_changes_the_font() {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    let fontes: Vec<String> = ["cozette", "tom-thumb", "unscii-8", "unscii-16", "Noto Sans"]
        .map(str::to_owned)
        .to_vec();
    set_current_pixel(PixelPanelInfo {
        editing: true,
        tool: PixelTool::Text,
        text_font_name: "cozette".to_owned(),
        text_fonts: fontes.clone(),
        ..Default::default()
    });
    let vp = viewport(WidgetStore::DOCK_W_MIN);
    let rects = host.paint::<PixelToolsPanel>(&mut st, vp);
    let chip = rects
        .iter()
        .find(|(n, _)| *n == ids::TEXT_FONT_DD)
        .map(|(_, r)| *r)
        .expect("o chip da fonte");
    for e in host.click_at(chip.x + chip.w * 0.5, chip.y + chip.h * 0.5) {
        let _ = host.apply_panel_event::<PixelToolsPanel>(&mut st, e);
    }
    let rects = host.paint::<PixelToolsPanel>(&mut st, vp);
    let linhas: Vec<_> = (0..fontes.len())
        .map(|i| {
            rects
                .iter()
                .find(|(n, _)| *n == ids::text_font_option_id(i))
                .map(|(_, r)| *r)
                .unwrap_or_else(|| panic!("a linha {i} nunca foi pintada"))
        })
        .collect();
    assert!(
        linhas.windows(2).all(|w| w[0].y < w[1].y),
        "na ordem da lista"
    );
    let r = linhas[1];
    let _ = host.drained_actions();
    for e in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = host.apply_panel_event::<PixelToolsPanel>(&mut st, e);
    }
    let o = PixelOptions {
        text_font: 1,
        ..PixelOptions::default()
    };
    so(&host.drained_actions(), o);
    assert!(
        matches!(
            host.store().get(ids::TEXT_FONT_DD),
            Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { open: false, .. })
        ),
        "a lista fecha"
    );
}

#[test]
fn the_text_size_field_is_painted_with_the_tool() {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(PixelPanelInfo {
        editing: true,
        tool: PixelTool::Text,
        ..Default::default()
    });
    let rects = host.paint::<PixelToolsPanel>(&mut st, viewport(WidgetStore::DOCK_W_MIN));
    assert!(rects.iter().any(|(n, _)| *n == ids::TEXT_SIZE));
}

/// O nome da fonte senta na coluna do VALOR, ao lado de «Font» (a largura dele é a da coluna do
/// valor, não a do painel), inteiro, no chip do dropdown — acima do Size.
#[test]
fn the_font_name_sits_in_the_value_column_next_to_its_label() {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(PixelPanelInfo {
        editing: true,
        tool: PixelTool::Text,
        text_font_name: "cozette".to_owned(),
        ..Default::default()
    });
    let w = WidgetStore::DOCK_W_MIN;
    let (rects, medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| {
        host.paint::<PixelToolsPanel>(&mut st, viewport(w))
    });
    let nome = medidos
        .iter()
        .find(|m| m.texto == "cozette")
        .expect("o nome pintado");
    assert_eq!(nome.pintado, "cozette", "inteiro");
    let r = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .expect("pintado")
    };
    let size = r(ids::TEXT_SIZE);
    let chip = r(ids::TEXT_FONT_DD);
    assert!(
        (chip.x - size.x).abs() < 1.0 && (chip.w - size.w).abs() < 1.0 && nome.largura < chip.w,
        "o chip na coluna do valor: {chip:?} (Size {size:?} · nome {} · painel {w})",
        nome.largura,
    );
    assert!(r(ids::TEXT_FONT_DD).y < size.y, "a fonte antes do tamanho");
}
