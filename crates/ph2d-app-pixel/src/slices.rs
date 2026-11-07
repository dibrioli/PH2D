//! **A ferramenta Slice, do lado da família** (`docs/PixelArt/P11_plano.md` §5): liga o ponteiro, as
//! teclas e o desenho ao gesto puro de `ph2d-pixel-slices`. A seleção é da família: sobrevive ao
//! mudar de quadro (T5), não vai ao documento nem ao desfazer.

use std::collections::BTreeMap;

use ph2d_editor_core::readout::{CHIP_H_PX, chip_width, paint_chip};
use ph2d_editor_core::zones::Rect as Zona;
use ph2d_pixel_core::slice::delete;
use ph2d_pixel_core::{SliceId, Sprite};
use ph2d_pixel_render::{SliceMark, draw_marching_ants, draw_slice, px_per_cell, rect_edges};
use ph2d_pixel_slices::{Gesto, Marca, no_quadro};
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, Point, VectorScene};
use winit::keyboard::KeyCode;

use crate::gesture::Mods;
use crate::input::{Button, Canvas, KeyAnswer, Press, image_point, key_of, to_screen};
use crate::{PixelApp, Tool};

/// O alcance das alças em píxeis de ECRÃ: o mesmo a qualquer zoom.
const GRAB_PX: f64 = 6.0; // LITERAL-PX-OK: alcance do ponteiro sobre uma alça, em píxeis de ecrã
/// Mas nunca menos que a célula que o ponteiro está a tocar (T4: o píxel da linha agarra-a).
const GRAB_MIN_CELLS: f64 = 0.5;

#[derive(Default)]
pub(crate) struct Estado {
    sel: BTreeMap<u32, Vec<SliceId>>,
    gesto: Option<(u32, Gesto)>,
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
}

/// Premir com a ferramenta na mão. O botão direito é do menu da slice (por vir), nunca do gesto.
pub(crate) fn press(
    app: &mut PixelApp,
    c: &Canvas<'_>,
    bits: u64,
    key: u32,
    p: (f32, f32),
    button: Button,
    m: Mods,
) -> Press {
    if button != Button::Primary {
        return Press::Done;
    }
    let Some(doc) = app.docs.get(&key) else {
        return Press::NotMine;
    };
    let Some(screen) = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm) else {
        return Press::Done;
    };
    let q = image_point(screen, p);
    let tol = (GRAB_PX / px_per_cell(screen)).max(GRAB_MIN_CELLS);
    let frame = no_quadro(&doc.sprite, doc.frame);
    let sel = app.slices.sel.entry(key).or_default();
    let g = Gesto::press(&doc.sprite, frame, sel, (q.x, q.y), tol, m.shift);
    app.slices.gesto = Some((key, g));
    Press::Done
}

/// Mover. `false`: não há gesto de slice.
pub(crate) fn moved(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32)) -> bool {
    let Some((key, _)) = app.slices.gesto.as_ref() else {
        return false;
    };
    let q = c.editing.and_then(|bits| {
        (key_of(c.world, bits) == Some(*key))
            .then(|| to_screen(c.world, bits, app.docs.get(key)?, c.camera, c.window, c.ppm))
            .flatten()
    });
    if let (Some(m), Some((_, g))) = (q, app.slices.gesto.as_mut()) {
        let q = image_point(m, p);
        g.drag((q.x, q.y));
    }
    true
}

/// Largar: o gesto inteiro vira UM passo, se o documento muda.
pub(crate) fn release(app: &mut PixelApp) -> bool {
    let Some((key, g)) = app.slices.gesto.take() else {
        return false;
    };
    let Some(doc) = app.docs.get_mut(&key) else {
        return true;
    };
    let frame = no_quadro(&doc.sprite, doc.frame);
    let sel = app.slices.sel.entry(key).or_default();
    if let Some(op) = g.release(&mut doc.sprite, frame, sel) {
        let _ = app.commit(key, op);
    }
    true
}

/// `Esc` a meio: o gesto cai sem passo nenhum.
pub(crate) fn cancela(app: &mut PixelApp) -> bool {
    app.slices.gesto.take().is_some()
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
    if app.tool != Tool::Slice || ctrl || shift || app.slices_selected(key).is_empty() {
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
    for k in marcas(app, key) {
        let mark = SliceMark {
            shape: k.shape,
            color: k.color,
            selected: k.selected,
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
