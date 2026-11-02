//! **P8 Edit ▸ Adjustments pelo caminho real** (`docs/PixelArt/P8_plano.md` §4): Brightness/
//! Contrast…, Hue/Saturation… (`Ctrl+U`) e Color Curve… (`Ctrl+M`) sobre a cena 9, pelo
//! [`panel::frame`] — irmão de `filtros.rs`. A pergunta: *o canvas mostra a cor do núcleo
//! (`filter::adjust`) sem gravar, o OK é UM passo, e um Ctrl+Z desfaz?*

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::pixel_panel::{PixelDialogKind, PixelEditAction as E, PixelRequest};
use ph2d_editor_core::zones::Rect;
use ph2d_pixel_core::filter::adjust::{HueMode, HueSaturation};
use winit::keyboard::KeyCode;

use crate::selecao::Palco;

/// Um quadrado `(100..=104, 40..=44)` pintado com `cor`, sem selecção no fim.
fn com_quadrado(cor: [u8; 4]) -> Palco {
    let mut p = Palco::limpo();
    p.app.fg = cor;
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((100, 40), (104, 44), Mods::default());
    p.tecla(KeyCode::KeyF, false, false);
    p.pede(PixelRequest::Select(
        ph2d_editor_core::pixel_panel::PixelSelectAction::Deselect,
    ));
    assert_eq!(p.rgba(102, 42), cor);
    p
}

fn aberto(p: &Palco) -> Option<PixelDialogKind> {
    let open = p
        .hero
        .store
        .context_menu()
        .is_some_and(|r| r.kind == ContextMenuKind::PixelEditDialog);
    open.then(|| p.hero.store.pixel_edit_dialog().kind)
}

fn mostrado(p: &mut Palco) -> [u8; 4] {
    let key = p.key;
    p.app.shown_rgba(key, 102, 42).expect("doc")
}

/// O artista escreve `v` no campo `id` (o commit emite `ValueChanged`), e o quadro seguinte.
fn escreve(p: &mut Palco, id: ph2d_editor_core::NodeId, v: f64) {
    p.hero.store.set_number_value(id, v);
    p.hero.apply_event(WidgetEvent::ValueChanged(id));
    p.quadro();
}

/// O OK do diálogo: UM passo, e um Ctrl+Z devolve o documento ao byte.
fn ok_um_passo(p: &mut Palco, quer: [u8; 4]) {
    let antes = p.sprite().clone();
    let n = p.passos();
    p.hero.apply_event(WidgetEvent::Click(ids::PIXEL_EDIT_OK));
    p.quadro();
    assert_eq!(aberto(p), None);
    assert_eq!(p.rgba(102, 42), quer, "o OK grava o que se via");
    assert_eq!(p.passos(), n + 1, "UM passo");
    p.tecla(KeyCode::KeyZ, true, false);
    assert_eq!(*p.sprite(), antes, "um Ctrl+Z desfaz o ajuste inteiro");
}

#[test]
fn brightness_contrast_from_the_menu_previews_the_core_lut_and_ok_is_one_step() {
    let cor = [255, 0, 0, 255];
    let mut p = com_quadrado(cor);
    let antes = p.sprite().clone();
    p.pede(PixelRequest::Edit(E::BrightnessContrast));
    assert_eq!(aberto(&p), Some(PixelDialogKind::BrightnessContrast));
    assert_eq!(mostrado(&mut p), cor, "0/0 = identidade");
    escreve(&mut p, ids::PIXEL_FILTER_BRIGHTNESS, -50.0);
    // `⌊(v/255)·(1 − 0,5)·255,5⌋`: 255 → 127, 0 → 0 (`BrightnessContrast::new(-50, 0)`).
    let quer = [127, 0, 0, 255];
    assert_eq!(mostrado(&mut p), quer);
    assert_eq!(*p.sprite(), antes, "o Preview não grava");
    ok_um_passo(&mut p, quer);
}

#[test]
fn hue_saturation_by_ctrl_u_previews_the_core_color_and_respects_the_channels() {
    let cor = [200, 100, 50, 255];
    let mut p = com_quadrado(cor);
    p.tecla(KeyCode::KeyU, true, false);
    assert_eq!(aberto(&p), Some(PixelDialogKind::HueSaturation), "Ctrl+U");
    p.hero
        .apply_event(WidgetEvent::Click(ids::PIXEL_FILTER_HUE_MODES[1].1));
    escreve(&mut p, ids::PIXEL_FILTER_HUE, 120.0);
    escreve(&mut p, ids::PIXEL_FILTER_SATURATION, -30.0);
    let hs = HueSaturation {
        mode: HueMode::Hsv,
        hue: 120,
        saturation: -30,
        lightness: 0,
        alpha: 0,
    };
    let [r, g, b] = hs.rgb([200, 100, 50]);
    assert_ne!([r, g, b], [200, 100, 50]);
    assert_eq!(mostrado(&mut p), [r, g, b, 255]);
    // O canal R desligado fica com o original no que se mostra.
    p.hero
        .apply_event(WidgetEvent::Click(ids::PIXEL_FILTER_CHANNELS[0].1));
    p.quadro();
    assert_eq!(mostrado(&mut p), [200, g, b, 255]);
    ok_um_passo(&mut p, [200, g, b, 255]);
}

#[test]
fn color_curve_by_ctrl_m_with_a_point_added_by_the_press_changes_the_preview() {
    let cor = [200, 100, 50, 255];
    let mut p = com_quadrado(cor);
    p.tecla(KeyCode::KeyM, true, false);
    assert_eq!(aberto(&p), Some(PixelDialogKind::ColorCurve), "Ctrl+M");
    assert_eq!(mostrado(&mut p), cor, "sem pontos = a diagonal");
    // O premir na caixa como o despacho o regista (o rato real prova-se no editor-core): a caixa
    // pintada, o id activo e o ponteiro; o quadro da família acrescenta o ponto.
    let caixa = Rect::new(0.0, 0.0, 255.0, 255.0);
    p.hero.hit_index.register(ids::PIXEL_FILTER_CURVE, caixa);
    p.hero.store.note_pointer(64.0, 64.0);
    p.hero.store.set_active(Some(ids::PIXEL_FILTER_CURVE));
    p.quadro();
    p.hero.store.set_active(None);
    p.quadro();
    let pts = p
        .hero
        .store
        .pixel_edit_dialog()
        .filter
        .curve
        .points()
        .to_vec();
    assert_eq!(pts, vec![(64, 191)], "o ponto acrescentado no premir");
    // Rectas com `⌊y⌋`: acima de 64 vai de (64,191) a (255,255); abaixo, de (0,0) a (64,191).
    let f = |v: i32| {
        if v <= 64 {
            191 * v / 64
        } else {
            191 + (64 * (v - 64)).div_euclid(191)
        }
    };
    let quer = [200, 100, 50].map(|v| u8::try_from(f(v)).expect("u8"));
    let quer = [quer[0], quer[1], quer[2], 255];
    assert_ne!(quer, cor);
    assert_eq!(mostrado(&mut p), quer);
    ok_um_passo(&mut p, quer);
}

/// O campo Alpha do Hue/Saturation (rodada 3 `B4`): `-50` mostra o alfa `255 → 127` sem gravar, o
/// OK é UM passo e o valor fica lembrado ao reabrir.
#[test]
fn hue_saturation_alpha_field_previews_half_alpha_and_is_remembered() {
    let cor = [200, 100, 50, 255];
    let mut p = com_quadrado(cor);
    p.tecla(KeyCode::KeyU, true, false);
    assert_eq!(aberto(&p), Some(PixelDialogKind::HueSaturation), "Ctrl+U");
    escreve(&mut p, ids::PIXEL_FILTER_ALPHA, -50.0);
    let quer = [200, 100, 50, 127];
    assert_eq!(mostrado(&mut p), quer);
    ok_um_passo(&mut p, quer);
    p.tecla(KeyCode::KeyU, true, false);
    assert_eq!(
        p.hero.store.number_value(ids::PIXEL_FILTER_ALPHA),
        Some(-50.0),
        "lembrado na sessão"
    );
}
