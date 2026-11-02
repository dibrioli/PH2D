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

/// P8c: a largura da lista de fontes é a lei da porta (dono 25/10: o nome INTEIRO, só desloca à
/// esquerda se não cabe, nunca sai da região) sobre a largura de cada linha COMO SE PINTA — a prévia
/// na escala inteira dela, não o rótulo medido na fonte da interface.
#[test]
fn the_font_list_fits_the_widest_painted_row_and_no_wider() {
    use ph2d_editor_core::pixel_panel::font_preview::{self, PixelFontPreview};
    use ph2d_editor_core::widget::popover_width_for;
    use ph2d_tokens::{ROW_H_PX, Spacing};
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    let fontes: Vec<String> = ["cozette", "tom-thumb", "x", "Noto Sans"]
        .map(str::to_owned)
        .to_vec();
    // «x» desenha-se numa prévia larga (150 × 8 células → 2× na linha de 22 = 300 px): MUITO mais
    // larga que o rótulo «x» na fonte da interface; as outras ficam sem prévia (rótulo).
    let (pw, ph) = (150_u32, 8_u32);
    font_preview::publish(
        "x".to_owned(),
        PixelFontPreview {
            w: pw,
            h: ph,
            cover: vec![u8::MAX; (pw * ph) as usize],
        },
    );
    let p = font_preview::with_preview("x", |p| p.cloned()).expect("publicada");
    #[expect(
        clippy::cast_precision_loss,
        reason = "largura de uma máscara de teste"
    )]
    let pintada = pw as f32 * p.scale_in(ROW_H_PX);
    set_current_pixel(PixelPanelInfo {
        editing: true,
        tool: PixelTool::Text,
        text_font_name: "cozette".to_owned(),
        text_fonts: fontes.clone(),
        ..Default::default()
    });
    let vp = Rect {
        x: 0.0,
        y: 0.0,
        w: 1920.0, // LITERAL-PX-OK: uma janela de ecrã inteiro (o painel à direita, a lista com região)
        h: 1080.0, // LITERAL-PX-OK: idem
    };
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
    let lista = popover_width_for(pintada, false);
    assert!(
        lista > chip.w,
        "a prévia é mais larga que o chip: {lista} × {chip:?}"
    );
    for i in 0..fontes.len() {
        let r = rects
            .iter()
            .find(|(n, _)| *n == ids::text_font_option_id(i))
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("a linha {i} nunca foi pintada"));
        assert!(
            r.w >= pintada && r.w <= lista && lista - r.w < Spacing::Md.px(),
            "a linha {i} {r:?}: cabe a prévia de {pintada} px, nunca mais larga que a lei ({lista})"
        );
        assert!(
            r.x >= vp.x && r.x + r.w <= vp.x + vp.w,
            "a linha {i} {r:?} dentro da região {vp:?}"
        );
        let ancorada = (r.x - chip.x).abs() < Spacing::Md.px();
        let encostada = (vp.x + vp.w - (r.x + r.w)).abs() < Spacing::Md.px();
        assert!(
            ancorada || (chip.x + lista > vp.x + vp.w && encostada),
            "no chip, ou deslocada à esquerda só o que não cabe: {r:?} × {chip:?}"
        );
    }
}
