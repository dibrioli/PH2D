//! **A sonda da cena 19** (P8): o que a mensagem ao dono promete sai daqui — o estado com que a
//! cena abre e um passo de cada família: o Gradient radial com Bayer 4 na selecção, o Text «PH2D»
//! na fonte de pixel, a Convolution blur-3x3 (Preview não grava, OK grava) e o Alpha −50 do
//! Hue/Saturation.

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input;
use ph2d_app_pixel::smoke_efeitos::{PANTS, RAMP, SELECTION, SHIRT, SIZE, SKIN, boneco};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{PixelGradientDither, PixelRequest, PixelSelectAction};
use ph2d_pixel_core::Sprite;
use ph2d_pixel_core::filter::convolution::KERNELS;
use winit::keyboard::KeyCode;

use crate::selecao::Palco;

fn celulas() -> impl Iterator<Item = (i32, i32)> {
    let (w, h) = (SIZE.0 as i32, SIZE.1 as i32);
    (0..h).flat_map(move |y| (0..w).map(move |x| (x, y)))
}

/// As células que mudaram de `antes` para o documento de agora.
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

#[test]
fn scene_19_opens_ready_and_each_p8_family_takes_one_step() {
    let mut p = Palco::cena(19);
    // O estado com que abre.
    let (sx, sy, sw, sh) = SELECTION;
    let m = p.mascara().expect("a selecção nasce feita");
    let na_mascara = celulas().filter(|&(x, y)| m.get(x, y)).count();
    assert_eq!(na_mascara, (sw * sh) as usize);
    assert_eq!(p.app.tool, Tool::Gradient);
    assert_eq!((p.app.fg, p.app.bg), (RAMP[0], RAMP[5]));
    let pal = p.sprite().palette().to_vec();
    assert!(pal.ends_with(&RAMP), "a rampa no fim da paleta");
    let boneco_n = celulas().filter(|&(x, y)| boneco(x, y).is_some()).count();
    for (c, cor) in [((13, 9), SKIN), ((13, 20), SHIRT), ((10, 35), PANTS)] {
        assert_eq!(p.rgba(c.0, c.1), cor);
    }
    assert_eq!(p.rgba(63, 47), [255; 4], "fundo branco");
    eprintln!(
        "[sonda cena 19] abre: selecção {sw}×{sh} em ({sx},{sy}) = {na_mascara} células · boneco = \
         {boneco_n} células · paleta = {} cores (rampa de {} no fim)",
        pal.len(),
        RAMP.len()
    );

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
    assert!(p.app.text_box().is_none());
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
    let m = p.mascara().expect("selecção");
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
    let gravadas = mudadas(&p, &antes);
    assert_eq!(gravadas, previa, "o carimbo é o preview");
    // Fora da selecção nada: na mesma âncora sem selecção a tinta passa da borda.
    let mut q = Palco::cena(19);
    q.pede(PixelRequest::Select(PixelSelectAction::Deselect));
    let antes = q.sprite().clone();
    caixa(&mut q, (50, 10), "PH2D");
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
