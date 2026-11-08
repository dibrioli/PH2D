//! **A ferramenta Slice, do lado da família** (`docs/PixelArt/P11_plano.md` §5): liga o ponteiro, as
//! teclas e o desenho ao gesto puro de `ph2d-pixel-slices`. A seleção é da família: sobrevive ao
//! mudar de quadro (T5), não vai ao documento nem ao desfazer.

use std::collections::BTreeMap;
use std::time::Instant;

use ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS;
use ph2d_editor_core::readout::{CHIP_H_PX, chip_width, paint_chip};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_SLICES};
use ph2d_editor_core::zones::Rect as Zona;
use ph2d_panel_pixel_slices::canal::{self, Contexto, Pedido};
use ph2d_panel_pixel_slices::familia;
use ph2d_pixel_core::slice::delete;
use ph2d_pixel_core::{SliceId, Sprite, Stroke};
use ph2d_pixel_render::{SliceMark, draw_marching_ants, draw_slice, px_per_cell, rect_edges};
use ph2d_pixel_slices::campos::quadros_com_key;
use ph2d_pixel_slices::transforma::{caixas_que_mudam, com_pixels, previa as arte};
use ph2d_pixel_slices::{Gesto, Grab, Marca, alcance, interior, no_quadro, seta, sob};
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, Point, VectorScene};
use winit::keyboard::KeyCode;
use winit::window::CursorIcon;

use crate::gesture::Mods;
use crate::input::{Button, Canvas, KeyAnswer, Press, image_point, key_of, to_screen};
use crate::{PixelApp, Tool};

#[derive(Default)]
pub(crate) struct Estado {
    pub(crate) sel: BTreeMap<u32, Vec<SliceId>>,
    gesto: Option<(u32, Gesto)>,
    /// O que o premir agarra sob o ponteiro (ou o agarrado): a seta e o realce.
    quente: Option<(u32, SliceId, Grab)>,
    transform: bool,
    /// O último clique: o segundo, depressa e na mesma célula, mostra o cartão (T2).
    clique: Option<(Instant, (i32, i32))>,
    /// O botão direito premido: a célula e o ponto; o quadro abre o menu.
    menu: Option<((i32, i32), (f32, f32))>,
    visto: Option<bool>,
    /// O documento e o desfazer depois da última cor ou do último arrasto: reescreve-se o mesmo passo.
    cor: Option<(u32, usize)>,
}

impl Estado {
    pub(crate) const fn em_curso(&self) -> bool {
        self.gesto.is_some()
    }
}

impl PixelApp {
    /// As slices selecionadas do documento `key`.
    #[must_use]
    pub fn slices_selected(&self, key: u32) -> &[SliceId] {
        self.slices.sel.get(&key).map_or(&[], Vec::as_slice)
    }

    /// A célula sob o cursor (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn hover_cell(&self) -> Option<(i32, i32)> {
        self.hover.map(|h| h.1)
    }

    /// A seta da Slice Tool em `p` fora do chrome, e o realçado: o que o premir agarra.
    #[must_use]
    pub fn slice_cursor(&self, hero: &HeroScreen, (x, y): (f32, f32)) -> Option<Quente> {
        let livre = hero.chrome_panel_at(x, y).is_none() && hero.chrome_hit(x, y).is_none();
        let q = self
            .slices
            .quente
            .filter(|_| livre && self.tool == Tool::Slice);
        q.map(|q| (seta(q.2), q.1, q.2))
    }
}

/// O ponto em células, o alcance das alças e o quadro: os MESMOS no premir e no pairar.
fn alvo(app: &PixelApp, c: &Canvas<'_>, bits: u64, key: u32, p: (f32, f32)) -> Option<Alvo> {
    let doc = app.docs.get(&key)?;
    let m = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm)?;
    let (q, f) = (image_point(m, p), no_quadro(&doc.sprite, doc.frame));
    Some(((q.x, q.y), alcance(px_per_cell(m)), f))
}
type Alvo = ((f64, f64), f64, u32);
pub type Quente = (CursorIcon, SliceId, Grab);

/// O que o premir agarraria sob o ponteiro, com a ferramenta na mão.
fn paira(app: &PixelApp, c: &Canvas<'_>, p: (f32, f32)) -> Option<(u32, SliceId, Grab)> {
    let bits = c.editing.filter(|_| app.tool == Tool::Slice)?;
    let key = key_of(c.world, bits)?;
    let (q, tol, frame) = alvo(app, c, bits, key, p)?;
    let s = &app.docs.get(&key)?.sprite;
    sob(s, frame, app.slices_selected(key), q, tol).map(|(id, g)| (key, id, g))
}

/// Premir com a ferramenta na mão. O botão direito é do menu da slice, nunca do gesto.
pub(crate) fn press(
    app: &mut PixelApp,
    c: &Canvas<'_>,
    bits: u64,
    key: u32,
    p: (f32, f32),
    button: Button,
    m: Mods,
) -> Press {
    let Some(doc) = app.docs.get(&key) else {
        return Press::NotMine;
    };
    let Some((q, tol, frame)) = alvo(app, c, bits, key, p) else {
        return Press::Done;
    };
    let cel = (q.0.floor() as i32, q.1.floor() as i32);
    if button != Button::Primary {
        app.slices.menu = Some((cel, p));
        return Press::Done;
    }
    let agora = Instant::now();
    let antes = app.slices.clique.replace((agora, cel));
    if antes.is_some_and(|(t, at)| {
        at == cel && agora.duration_since(t).as_nanos() < DOUBLE_CLICK_WINDOW_NS
    }) && interior(&doc.sprite, frame, cel).is_some()
    {
        canal::mostrar_propriedades();
    }
    let sel = app.slices.sel.entry(key).or_default();
    let g = Gesto::press(&doc.sprite, frame, sel, q, tol, m.shift);
    app.slices.quente = g.agarrado().map(|(id, a)| (key, id, a));
    app.slices.gesto = Some((key, g));
    Press::Done
}

/// Mover. `false`: não há gesto de slice (pairar só realça e escolhe a seta).
pub(crate) fn moved(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32)) -> bool {
    let Some(key) = app.slices.gesto.as_ref().map(|g| &g.0) else {
        app.slices.quente = paira(app, c, p);
        return false;
    };
    let q = c.editing.and_then(|bits| {
        (key_of(c.world, bits) == Some(*key))
            .then(|| to_screen(c.world, bits, app.docs.get(key)?, c.camera, c.window, c.ppm))
            .flatten()
    });
    let key = *key;
    if let (Some(m), Some((_, g))) = (q, app.slices.gesto.as_mut()) {
        let q = image_point(m, p);
        g.drag((q.x, q.y));
        app.hover = c
            .editing
            .map(|bits| (bits, (q.x.floor() as i32, q.y.floor() as i32)));
        if app.slices.transform {
            app.toca(key);
        }
    }
    true
}

/// Largar: o gesto inteiro vira UM passo, se o documento muda; com `Transform`, os píxeis vão no mesmo.
pub(crate) fn release(app: &mut PixelApp) -> bool {
    let Some((key, g)) = app.slices.gesto.take() else {
        return false;
    };
    let Some(doc) = app.docs.get_mut(&key) else {
        return true;
    };
    let frame = no_quadro(&doc.sprite, doc.frame);
    let previa = app.slices.transform.then(|| g.previa()).unwrap_or_default();
    let caixas = caixas_que_mudam(&doc.sprite, frame, &previa);
    if let Some(motivo) = crate::camadas::recusa_pintar(doc).filter(|_| !caixas.is_empty()) {
        app.recusa(motivo);
        return true;
    }
    let fundo = doc.sprite.value_of_rgba(app.bg);
    let sel = app.slices.sel.entry(key).or_default();
    if let Some(op) = g.release(&mut doc.sprite, frame, sel) {
        let op = com_pixels(&doc.sprite, op, (doc.layer, doc.frame), &caixas, fundo);
        let _ = app.commit(key, op);
    }
    true
}

/// A arte a meio do arrasto com `Transform`: o traço do MESMO op que o largar grava.
pub(crate) fn previa(app: &PixelApp, key: u32) -> Option<Stroke> {
    let g = app
        .slices
        .gesto
        .as_ref()
        .filter(|g| g.0 == key && app.slices.transform)?;
    let d = app
        .docs
        .get(&key)
        .filter(|d| crate::camadas::recusa_pintar(d).is_none())?;
    arte(&d.sprite, (d.layer, d.frame), &g.1, app.fundo(&d.sprite))
}

fn pedido(app: &mut PixelApp, key: u32, p: &Pedido) {
    if *p == Pedido::PassoFechou {
        app.slices.cor = None;
        return;
    }
    let cor = p.reescreve_o_passo();
    if cor && app.slices.cor == app.docs.get(&key).map(|d| (key, d.history.undo_len())) {
        app.undo(key);
    }
    let Some(d) = app.docs.get_mut(&key) else {
        return;
    };
    let frame = no_quadro(&d.sprite, d.frame);
    let sel = app.slices.sel.get(&key).cloned().unwrap_or_default();
    let t = &mut app.slices.transform;
    let r = familia::responde(p, &mut d.sprite, frame, &sel, t);
    let feito = r.op.is_some_and(|op| app.commit(key, op).is_ok());
    if let Some(nova) = r.sel {
        app.slices.sel.insert(key, nova);
    }
    if cor && feito {
        app.slices.cor = app.docs.get(&key).map(|d| (key, d.history.undo_len()));
    }
}

/// O quadro: os pedidos do painel, o que ele mostra, o menu, e quando abre (a ferramenta na mão).
pub(crate) fn frame(app: &mut PixelApp, hero: &mut HeroScreen) {
    for p in canal::take_pedidos() {
        if let Some(key) = app.chave {
            app.assenta();
            crate::viva::assenta(app);
            pedido(app, key, &p);
        }
    }
    let menu = app.slices.menu.take();
    let mut x = Contexto::default();
    if let Some((k, d)) = app.chave.and_then(|k| Some((k, app.docs.get(&k)?)))
        && app.tool == Tool::Slice
    {
        let (f, sel) = (no_quadro(&d.sprite, d.frame), app.slices_selected(k));
        x = familia::contexto(&d.sprite, f, sel, app.slices.transform);
        if let Some(m) = menu.and_then(|(cel, p)| {
            familia::menu_em(&d.sprite, f, sel, cel, hero.escala().to_logical(p))
        }) {
            canal::abrir_menu(m);
        }
    }
    let tem = x.na_mao;
    canal::publish_contexto(x);
    if app.slices.visto != Some(tem) {
        hero.panel_visibility.insert(PANEL_PIXEL_SLICES, tem);
        app.slices.visto = Some(tem);
    }
}

/// Os quadros com key das selecionadas, para a linha do tempo (§9.2): só com a ferramenta na mão.
pub(crate) fn marcas_de_quadro(app: &PixelApp, key: u32) -> Vec<u32> {
    let sel = if app.tool == Tool::Slice {
        app.slices_selected(key)
    } else {
        &[]
    };
    app.docs
        .get(&key)
        .map_or_else(Vec::new, |d| quadros_com_key(&d.sprite, sel))
}

/// `Esc` a meio: o gesto cai sem passo nenhum.
pub(crate) fn cancela(app: &mut PixelApp) -> bool {
    app.slices.gesto.take().map(|g| app.toca(g.0)).is_some()
}

/// `Esc` limpa a seleção; `Delete` apaga as slices inteiras, 1 desfazer (G2, G3). As setas não são
/// daqui: mudam de quadro (T2).
pub(crate) fn tecla(
    app: &mut PixelApp,
    key: u32,
    code: KeyCode,
    ctrl: bool,
    shift: bool,
) -> Option<KeyAnswer> {
    if app.tool != Tool::Slice || ctrl || shift {
        return None;
    }
    if code == KeyCode::Escape && canal::esc_fecha_menu() {
        return Some(KeyAnswer::Quiet);
    }
    if app.slices_selected(key).is_empty() {
        return None;
    }
    match code {
        KeyCode::Escape => {}
        KeyCode::Delete | KeyCode::Backspace => {
            let op = delete(&app.docs.get(&key)?.sprite, app.slices_selected(key))?;
            let _ = app.commit(key, op);
        }
        _ => return None,
    }
    app.slices.sel.remove(&key);
    Some(KeyAnswer::Quiet)
}

/// O desfazer pode tirar uma slice da seleção.
pub(crate) fn poda(e: &mut Estado, key: u32, s: &Sprite) {
    if let Some(v) = e.sel.get_mut(&key) {
        v.retain(|id| s.slice(*id).is_some());
    }
}

/// O que se desenha de cada slice: selecionada só com a ferramenta na mão.
fn marcas(app: &PixelApp, key: u32) -> Vec<Marca> {
    let Some(doc) = app.docs.get(&key) else {
        return Vec::new();
    };
    let g = app.slices.gesto.as_ref().filter(|(k, _)| *k == key);
    let sel = if app.tool == Tool::Slice {
        app.slices_selected(key)
    } else {
        &[]
    };
    let frame = no_quadro(&doc.sprite, doc.frame);
    ph2d_pixel_slices::marcas(&doc.sprite, frame, g.map(|(_, g)| g), sel)
}

/// As slices por cima da arte, com qualquer ferramenta (F3, T8), e o elástico do gesto.
pub(crate) fn desenha(app: &PixelApp, key: u32, m: Affine, linha: f64, scene: &mut VectorScene) {
    let Some(s) = app.docs.get(&key).map(|d| &d.sprite) else {
        return;
    };
    let canvas = s.bounds();
    let quente = app
        .slices
        .quente
        .filter(|q| q.0 == key && app.tool == Tool::Slice);
    for k in marcas(app, key) {
        let mark = SliceMark {
            shape: k.shape,
            color: k.color,
            selected: k.selected,
            hot: quente.filter(|q| q.1 == k.id).map(|q| q.2),
        };
        draw_slice(scene, &mark, canvas, m, linha);
    }
    if let Some((_, g)) = app.slices.gesto.as_ref().filter(|(k, _)| *k == key)
        && let Some(r) = g.elastico(canvas)
    {
        let fase = crate::draw::fase_das_formigas(app);
        draw_marching_ants(scene, &rect_edges(r, canvas), m, fase, linha);
    }
}

/// O nome das selecionadas, na tela (decisão do dono, §9.2): uma ficha por cima da caixa.
pub(crate) fn nomes(
    app: &PixelApp,
    c: &Canvas<'_>,
    area: Zona,
    theme: Theme,
    (scene, text): (&mut VectorScene, &mut ph2d_text::TextSystem),
) {
    let Some((bits, key)) = c.editing.and_then(|b| Some((b, key_of(c.world, b)?))) else {
        return;
    };
    let Some(m) = app
        .docs
        .get(&key)
        .and_then(|d| to_screen(c.world, bits, d, c.camera, c.window, c.ppm))
    else {
        return;
    };
    for k in marcas(app, key).into_iter().filter(|k| k.selected) {
        let (b, name) = (k.shape.bounds, k.name);
        let q = m * Point::new(f64::from(b.x), f64::from(b.y));
        let w = chip_width(text, &name, 0.0);
        let meio = CHIP_H_PX * 0.5;
        let cx = (q.x as f32 + w * 0.5).max(area.x + w * 0.5);
        let cy = (q.y as f32 - meio).max(area.y + meio);
        paint_chip(text, scene, &name, [cx, cy], 0.0, theme);
    }
}
