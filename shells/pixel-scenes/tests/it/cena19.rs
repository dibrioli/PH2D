//! **A sonda da cena 19** (P8): o que a mensagem ao dono promete sai daqui — o estado com que a
//! cena abre (Background branca + o boneco numa camada transparente activa, sem selecção) e um passo
//! de cada família: o Gradient radial com Bayer 4, o Text «PH2D» na fonte de pixel, a Convolution
//! blur-3x3 (Preview não grava, OK grava) e o Alpha −50 do Hue/Saturation. Depois, um teste por
//! nome para cada diálogo de Edit ▸ Adjustments/FX e o Outline SEM selecção (pedido do menu →
//! diálogo → OK), e a borracha nas duas camadas.

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input;
use ph2d_app_pixel::smoke_efeitos::{FIGURE, PANTS, RAMP, SHIRT, SIZE, SKIN, boneco};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{
    PixelEditAction as E, PixelGradientDither, PixelRequest, PixelSelectAction,
};
use ph2d_editor_core::zones::Rect;
use ph2d_pixel_core::Sprite;
use ph2d_pixel_core::filter::convolution::KERNELS;
use winit::keyboard::KeyCode;

use crate::selecao::Palco;

const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
/// A zona livre de cima à direita (x, y, w, h), onde não há boneco.
const ZONA: (i32, i32, u32, u32) = (32, 6, 26, 20);

fn celulas() -> impl Iterator<Item = (i32, i32)> {
    let (w, h) = (SIZE.0 as i32, SIZE.1 as i32);
    (0..h).flat_map(move |y| (0..w).map(move |x| (x, y)))
}

/// As células do cel activo que mudaram de `antes` para o documento de agora.
fn mudadas(p: &Palco, antes: &Sprite) -> Vec<(i32, i32)> {
    let d = p.app.doc(p.key).expect("doc");
    let k = (d.layer, d.frame);
    celulas()
        .filter(|&(x, y)| antes.value_at(k, x, y) != d.sprite.value_at(k, x, y))
        .collect()
}

fn mudaram(p: &Palco, antes: &Sprite) -> usize {
    mudadas(p, antes).len()
}

/// Uma tecla pelo caminho da shell: primeiro a caixa de texto, depois os atalhos.
fn tecla_texto(p: &mut Palco, code: KeyCode, text: Option<&str>) {
    let m = Mods::default();
    if input::text_key(&mut p.app, p.sim.world(), Some(p.bits), code, m, text).is_none() {
        input::key_with(&mut p.app, p.sim.world(), Some(p.bits), code, m);
    }
    p.quadro();
}

fn clica(p: &mut Palco, id: ph2d_editor_core::NodeId) {
    p.hero.apply_event(WidgetEvent::Click(id));
    p.quadro();
}

fn escreve(p: &mut Palco, id: ph2d_editor_core::NodeId, v: f64) {
    p.hero.store.set_number_value(id, v);
    p.hero.apply_event(WidgetEvent::ValueChanged(id));
    p.quadro();
}

/// A camada Background (a de baixo) e a do boneco (a activa).
fn camadas(p: &Palco) -> (u32, u32) {
    let d = p.app.doc(p.key).expect("doc");
    (d.sprite.layers()[0].id.0, d.layer.0)
}

/// A [`ZONA`] seleccionada pelo marquee (o gesto do artista); devolve a máscara.
fn seleciona_zona(p: &mut Palco) -> ph2d_pixel_core::selection::Mask {
    let (sx, sy, sw, sh) = ZONA;
    p.ferramenta(Tool::RectMarquee);
    p.arrasta(
        (sx, sy),
        (sx + sw as i32 - 1, sy + sh as i32 - 1),
        Mods::default(),
    );
    let m = p.mascara().expect("o marquee selecciona");
    let n = celulas().filter(|&(x, y)| m.get(x, y)).count();
    assert_eq!(n, (sw * sh) as usize);
    m
}

#[test]
fn scene_19_opens_ready_and_each_p8_family_takes_one_step() {
    let mut p = Palco::cena(19);
    // O estado com que abre: sem selecção, a camada do boneco activa e transparente por cima da
    // Background branca.
    assert!(p.mascara().is_none(), "nasce sem selecção");
    let (fundo, activa) = camadas(&p);
    assert_ne!(fundo, activa, "o boneco não está na Background");
    let s = p.sprite();
    assert_eq!(s.layers().len(), 2);
    assert!(s.layers()[0].background && !s.layers()[1].background);
    let f = p.app.doc(p.key).expect("doc").frame;
    let bg_px = s.rgba_of_value(s.value_at((s.layers()[0].id, f), 63, 47));
    assert_eq!(bg_px, [255; 4], "a Background é branca");
    assert_eq!(
        p.rgba(63, 47),
        TRANSPARENT,
        "a camada do boneco é transparente fora dele"
    );
    assert_eq!(p.app.tool, Tool::Gradient);
    assert_eq!((p.app.fg, p.app.bg), (RAMP[0], RAMP[5]));
    let pal = p.sprite().palette().to_vec();
    assert!(pal.ends_with(&RAMP), "a rampa no fim da paleta");
    let boneco_n = celulas().filter(|&(x, y)| boneco(x, y).is_some()).count();
    for (c, cor) in [((13, 9), SKIN), ((13, 20), SHIRT), ((10, 35), PANTS)] {
        assert_eq!(p.rgba(c.0, c.1), cor);
    }
    eprintln!(
        "[sonda cena 19] abre: 2 camadas (Background branca + boneco transparente activa), sem \
         selecção · boneco = {boneco_n} células · paleta = {} cores (rampa de {} no fim)",
        pal.len(),
        RAMP.len()
    );

    // Uma selecção na zona livre pelo marquee (o gesto do artista), para o Gradient.
    let (sx, sy, sw, sh) = ZONA;
    let m = seleciona_zona(&mut p);
    p.ferramenta(Tool::Gradient);

    // Gradient radial + Bayer 4 do centro da selecção ao canto: UM passo, só dentro da selecção.
    p.opcoes(|o| {
        o.gradient_radial = true;
        o.gradient_dither = PixelGradientDither::Bayer4;
    });
    let antes = p.sprite().clone();
    let n = p.passos();
    let centro = (sx + sw as i32 / 2, sy + sh as i32 / 2);
    p.arrasta(
        centro,
        (sx + sw as i32 - 1, sy + sh as i32 - 1),
        Mods::default(),
    );
    assert_eq!(p.passos(), n + 1, "o gradiente é UM passo");
    let todas = mudadas(&p, &antes);
    let pintadas = todas.len();
    let fora = todas.iter().filter(|&&(x, y)| !m.get(x, y)).count();
    assert_eq!(fora, 0, "nada fora da selecção");
    let cores: std::collections::BTreeSet<[u8; 4]> = celulas()
        .filter(|&(x, y)| m.get(x, y))
        .map(|(x, y)| p.rgba(x, y))
        .collect();
    assert!(pintadas > 0 && cores.len() >= 2);
    eprintln!(
        "[sonda cena 19] Gradient radial Bayer 4: {pintadas} células, {} cores, 1 passo",
        cores.len()
    );

    // Text «PH2D» na fonte de pixel (a de fábrica), na zona livre de baixo: Enter = UM passo.
    p.pede(PixelRequest::Select(PixelSelectAction::Deselect));
    p.ferramenta(Tool::Text);
    assert_eq!(p.app.options.text_font, 0, "a fonte de pixel de fábrica");
    p.app.fg = [0, 0, 0, 255];
    let antes = p.sprite().clone();
    let n = p.passos();
    p.press((26, 34), Mods::default());
    p.release();
    assert!(p.app.text_box().is_some(), "a caixa abriu");
    for ch in "PH2D".chars() {
        let mut b = [0u8; 4];
        tecla_texto(&mut p, KeyCode::KeyA, Some(ch.encode_utf8(&mut b)));
    }
    assert_eq!(*p.sprite(), antes, "teclar só pré-visualiza");
    tecla_texto(&mut p, KeyCode::Enter, None);
    assert!(
        p.app.text_box().is_none() && p.app.floating(),
        "o 1.º Enter solta"
    );
    assert_eq!(*p.sprite(), antes, "solto ainda não grava");
    tecla_texto(&mut p, KeyCode::Enter, None);
    assert_eq!(p.passos(), n + 1, "o texto é UM passo");
    let texto = mudaram(&p, &antes);
    assert!(texto > 0);
    eprintln!("[sonda cena 19] Text «PH2D» (fonte de pixel): {texto} células, 1 passo");

    // Convolution Matrix blur-3x3: o Preview não grava, o OK grava UM passo.
    p.tecla(KeyCode::F9, false, false);
    let i = KERNELS
        .iter()
        .position(|k| k.name == "blur-3x3")
        .expect("blur");
    clica(&mut p, ids::pixel_filter_kernel_id(i).expect("na escada"));
    let antes = p.sprite().clone();
    let n = p.passos();
    let key = p.key;
    let previa = celulas()
        .filter(|&(x, y)| p.app.shown_rgba(key, x, y) != Some(p.rgba(x, y)))
        .count();
    assert!(previa > 0, "o Preview mostra");
    assert_eq!(*p.sprite(), antes, "o Preview não grava");
    clica(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(p.passos(), n + 1, "o OK é UM passo");
    let blur = mudaram(&p, &antes);
    assert_eq!(blur, previa, "o OK grava o que se via");
    eprintln!(
        "[sonda cena 19] Convolution blur-3x3: Preview mostra {previa} células sem gravar; OK grava {blur}, 1 passo"
    );

    // Hue/Saturation com Alpha −50: o alfa 255 vira 127.
    p.tecla(KeyCode::KeyU, true, false);
    p.hero
        .store
        .set_number_value(ids::PIXEL_FILTER_ALPHA, -50.0);
    p.hero
        .apply_event(WidgetEvent::ValueChanged(ids::PIXEL_FILTER_ALPHA));
    p.quadro();
    let n = p.passos();
    clica(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(p.passos(), n + 1);
    let camisa = p.rgba(13, 20);
    assert_eq!(camisa[3], 127, "o alfa da camisa: {camisa:?}");
    let a127 = celulas().filter(|&(x, y)| p.rgba(x, y)[3] == 127).count();
    eprintln!(
        "[sonda cena 19] Hue/Saturation Alpha −50: camisa {camisa:?}; {a127} células com alfa 127"
    );
}

/// Abre a caixa em `at` e escreve `t` (sem Enter).
fn caixa(p: &mut Palco, at: (i32, i32), t: &str) {
    p.ferramenta(Tool::Text);
    p.app.fg = [0, 0, 0, 255];
    p.press(at, Mods::default());
    p.release();
    tecla_texto(p, KeyCode::KeyA, Some(t));
}

/// O contorno da caixa do Text envolve a tinta com UMA célula de folga em todos os lados.
#[test]
fn the_text_box_outline_wraps_the_ink_with_one_cell_of_margin() {
    let mut p = Palco::cena(19);
    p.pede(PixelRequest::Select(PixelSelectAction::Deselect));
    caixa(&mut p, (26, 34), "PH2D");
    let key = p.key;
    let tinta: Vec<(i32, i32)> = celulas()
        .filter(|&(x, y)| p.app.shown_rgba(key, x, y) != Some(p.rgba(x, y)))
        .collect();
    let (ox, oy, ow, oh) = p.app.text_box_outline().expect("contorno");
    let x0 = tinta.iter().map(|c| c.0).min().expect("tinta");
    let x1 = tinta.iter().map(|c| c.0).max().expect("tinta");
    let y0 = tinta.iter().map(|c| c.1).min().expect("tinta");
    let y1 = tinta.iter().map(|c| c.1).max().expect("tinta");
    eprintln!("[sonda cena 19] tinta x {x0}..={x1} y {y0}..={y1} · contorno ({ox},{oy}) {ow}×{oh}");
    assert!(ox <= x0 - 1 && oy <= y0 - 1, "folga à esquerda e em cima");
    assert!(ox + ow as i32 - 1 >= x1 + 1, "folga à direita");
    assert!(oy + oh as i32 - 1 >= y1 + 1, "folga em baixo");
    assert_eq!(ox + ow as i32 - 1, x1 + 1, "exactamente uma à direita");
}

/// O Text corta-se à selecção como o Fill e o Gradient: o preview e o carimbo só dentro dela.
#[test]
fn the_text_is_clipped_to_the_selection_in_preview_and_stamp() {
    let mut p = Palco::cena(19);
    let m = seleciona_zona(&mut p);
    let antes = p.sprite().clone();
    caixa(&mut p, (50, 10), "PH2D");
    let key = p.key;
    let previa: Vec<(i32, i32)> = celulas()
        .filter(|&(x, y)| p.app.shown_rgba(key, x, y) != Some(p.rgba(x, y)))
        .collect();
    assert!(!previa.is_empty(), "há tinta dentro");
    assert!(
        previa.iter().all(|&(x, y)| m.get(x, y)),
        "o preview corta-se"
    );
    tecla_texto(&mut p, KeyCode::Enter, None);
    tecla_texto(&mut p, KeyCode::Enter, None);
    let gravadas = mudadas(&p, &antes);
    assert_eq!(gravadas, previa, "o carimbo é o preview");
    // Fora da selecção nada: na mesma âncora sem selecção a tinta passa da borda.
    let mut q = Palco::cena(19);
    q.pede(PixelRequest::Select(PixelSelectAction::Deselect));
    let antes = q.sprite().clone();
    caixa(&mut q, (50, 10), "PH2D");
    tecla_texto(&mut q, KeyCode::Enter, None);
    tecla_texto(&mut q, KeyCode::Enter, None);
    let sem = mudadas(&q, &antes);
    assert!(
        sem.iter().any(|&(x, y)| !m.get(x, y)),
        "sem selecção passava da borda"
    );
    eprintln!(
        "[sonda cena 19] Text em (50,10): {} células com selecção · {} sem",
        gravadas.len(),
        sem.len()
    );
}

/// A família publica a área livre e o retângulo do sprite no ecrã a cada quadro (o diálogo de
/// filtro foge dele): os cantos são os das células (0,0) e (64,48).
#[test]
fn the_sprite_rect_on_screen_is_published_for_the_dialogs() {
    let mut p = Palco::cena(19);
    p.quadro();
    let (_, (x, y, w, h)) = p.hero.store.pixel_edit_dialog().avoid.expect("publicado");
    let a = crate::comum::at(&p.app, &p.sim, &p.cam, p.bits, 0, 0);
    let b = crate::comum::at(&p.app, &p.sim, &p.cam, p.bits, 63, 47);
    let cel = (b.0 - a.0) / 63.0;
    assert!((x as f32 - (a.0 - cel * 0.5)).abs() <= 1.0, "x {x} × {a:?}");
    assert!((y as f32 - (a.1 - cel * 0.5)).abs() <= 1.0, "y {y} × {a:?}");
    assert!((w as f32 - cel * 64.0).abs() <= 2.0 && (h as f32 - cel * 48.0).abs() <= 2.0);
}

/// O pedido do menu Edit `a` (o que o clique do menu emite) abre o diálogo SEM selecção, `ajusta`
/// mexe nos campos, o OK grava UM passo; devolve as células que mudaram na camada do boneco.
fn pelo_menu_sem_selecao(a: E, ajusta: impl FnOnce(&mut Palco)) -> (Palco, Vec<(i32, i32)>) {
    let mut p = Palco::cena(19);
    assert!(p.mascara().is_none());
    let (_, activa) = camadas(&p);
    p.pede(PixelRequest::Edit(a));
    assert!(
        p.hero.store.pixel_filter_preview().is_some(),
        "{a:?}: o diálogo abriu"
    );
    ajusta(&mut p);
    let antes = p.sprite().clone();
    let n = p.passos();
    clica(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(p.passos(), n + 1, "{a:?}: o OK é UM passo");
    assert_eq!(camadas(&p).1, activa);
    let m = mudadas(&p, &antes);
    eprintln!("[sonda cena 19] {a:?} sem selecção: {} células", m.len());
    (p, m)
}

fn no_boneco(m: &[(i32, i32)]) -> usize {
    m.iter().filter(|&&(x, y)| boneco(x, y).is_some()).count()
}

#[test]
fn scene_19_brightness_contrast_acts_on_the_figure_without_a_selection() {
    let (p, m) = pelo_menu_sem_selecao(E::BrightnessContrast, |p| {
        escreve(p, ids::PIXEL_FILTER_BRIGHTNESS, -50.0);
    });
    assert!(no_boneco(&m) > 0);
    assert_ne!(p.rgba(13, 9), SKIN, "a pele escureceu");
}

#[test]
fn scene_19_hue_saturation_acts_on_the_figure_without_a_selection() {
    let (p, m) = pelo_menu_sem_selecao(E::HueSaturation, |p| {
        escreve(p, ids::PIXEL_FILTER_HUE, 120.0);
    });
    assert!(no_boneco(&m) > 0);
    assert_ne!(p.rgba(13, 20), SHIRT, "a camisa mudou de matiz");
}

#[test]
fn scene_19_color_curve_acts_on_the_figure_without_a_selection() {
    let (p, m) = pelo_menu_sem_selecao(E::ColorCurve, |p| {
        // Um ponto (64, 191) premido na caixa, como em `ajustes.rs`.
        p.hero
            .hit_index
            .register(ids::PIXEL_FILTER_CURVE, Rect::new(0.0, 0.0, 255.0, 255.0));
        p.hero.store.note_pointer(64.0, 64.0);
        p.hero.store.set_active(Some(ids::PIXEL_FILTER_CURVE));
        p.quadro();
        p.hero.store.set_active(None);
        p.quadro();
    });
    assert!(no_boneco(&m) > 0);
    assert_ne!(p.rgba(10, 35), PANTS, "as calças clarearam");
}

#[test]
fn scene_19_invert_acts_on_the_figure_without_a_selection() {
    let (p, m) = pelo_menu_sem_selecao(E::Invert, |_| {});
    assert!(no_boneco(&m) > 0);
    let [r, g, b, _] = SHIRT;
    assert_eq!(p.rgba(13, 20), [255 - r, 255 - g, 255 - b, 255]);
}

#[test]
fn scene_19_despeckle_acts_on_the_figure_without_a_selection() {
    let (_, m) = pelo_menu_sem_selecao(E::Despeckle, |_| {});
    assert!(!m.is_empty(), "a mediana come as pontas dos braços");
}

#[test]
fn scene_19_convolution_acts_on_the_figure_without_a_selection() {
    let i = KERNELS
        .iter()
        .position(|k| k.name == "blur-3x3")
        .expect("blur");
    let (_, m) = pelo_menu_sem_selecao(E::ConvolutionMatrix, |p| {
        clica(p, ids::pixel_filter_kernel_id(i).expect("na escada"));
    });
    assert!(no_boneco(&m) > 0);
}

/// O Outline na camada transparente desenha o anel À VOLTA do boneco com a cor de frente; o boneco
/// fica.
#[test]
fn scene_19_outline_rings_the_figure_on_the_transparent_layer() {
    let (p, m) = pelo_menu_sem_selecao(E::Outline, |_| {});
    assert_eq!(no_boneco(&m), 0, "o boneco fica");
    assert!(!m.is_empty());
    assert_eq!(p.rgba(9, 8), RAMP[0], "à esquerda da cabeça");
    assert_eq!(p.rgba(13, 5), RAMP[0], "por cima da cabeça");
    assert_eq!(p.rgba(13, 9), SKIN);
    assert_eq!(p.rgba(40, 40), TRANSPARENT, "longe fica transparente");
    let r = FIGURE;
    let perto = |(x, y): (i32, i32)| {
        x >= r.x - 1 && y >= r.y - 1 && x <= r.x + r.w as i32 && y <= r.y + r.h as i32
    };
    assert!(m.iter().all(|&c| perto(c)), "só a volta do boneco");
}

/// A borracha (lei do Aseprite): na camada do boneco apaga para transparente; na Background pinta a
/// cor de fundo.
#[test]
fn scene_19_eraser_clears_the_figure_layer_and_paints_bg_on_the_background() {
    let mut p = Palco::cena(19);
    p.ferramenta(Tool::Eraser);
    p.arrasta((13, 20), (13, 20), Mods::default());
    assert_eq!(p.rgba(13, 20), TRANSPARENT, "camada normal: apaga");
    let (fundo, _) = camadas(&p);
    p.pede(PixelRequest::LayerSelect(fundo));
    assert_eq!(camadas(&p).1, fundo);
    p.ferramenta(Tool::Eraser);
    p.arrasta((40, 40), (40, 40), Mods::default());
    assert_eq!(p.rgba(40, 40), RAMP[5], "Background: a cor de fundo");
}
