//! **A ferramenta Text** (P8, `docs/PixelArt/P8_plano.md` §9.3): a caixa aberta (o texto vive aqui,
//! nunca no documento), o preview e o passo do Enter; o carimbo é `ph2d_pixel_text::carimbo`.

use ph2d_pixel_core::raster::Cell;
use ph2d_pixel_core::{CelKey, Sprite, Stroke};
use ph2d_pixel_text::carimbo::{self, Edit, Editing, Fonte};
use ph2d_pixel_text::{FontChoice, TextStyle};
use winit::keyboard::KeyCode;

use crate::gesture::Mods;
use crate::input::KeyAnswer;
use crate::{Options, PixelApp};

/// A caixa aberta: o documento, a âncora, o texto, o cursor (em caracteres) e o preview.
#[derive(Clone, Default)]
pub(crate) struct Caixa {
    key: u32,
    at: Cell,
    ed: Editing,
    traco: Option<Stroke>,
    /// O retângulo da caixa em células (x, y, w, h) e a célula do cursor `(x, y, altura)`.
    moldura: (i32, i32, u32, u32),
    /// O que se desenha: a moldura e a tinta com uma célula de folga em todos os lados.
    contorno: (i32, i32, u32, u32),
    caret: (i32, i32, i32),
}

/// As fontes do Text: as de fábrica e as do sistema.
#[must_use]
pub fn fontes() -> Vec<String> {
    carimbo::font_list(
        ph2d_system_fonts::library::pickable_families()
            .into_iter()
            .flatten(),
    )
}

pub(crate) fn normaliza(i: u16) -> u16 {
    carimbo::wrap_index(i, fontes().len())
}

pub(crate) fn nome_da_fonte(i: u16) -> String {
    fontes().into_iter().nth(usize::from(i)).unwrap_or_default()
}

/// O estilo das opções da mão.
#[must_use]
pub fn estilo(o: &Options) -> Fonte {
    let mut style = TextStyle::new(FontChoice::Builtin(String::new()), o.text_px.max(1));
    (style.bold, style.italic) = (o.text_bold, o.text_italic);
    (style.antialias, style.hinting) = (o.text_antialias, o.text_hinting);
    let face = ph2d_system_fonts::library::face_bytes;
    Fonte::pick(o.text_font, &fontes(), style, face)
}

/// As células que o texto carimba com `fg` sobre o cel `ck` (Antialias = por cima do que lá está).
#[must_use]
pub fn valores(
    s: &Sprite,
    ck: CelKey,
    t: &str,
    f: &Fonte,
    at: Cell,
    fg: [u8; 4],
) -> Vec<(Cell, u32)> {
    let below = |(x, y)| s.rgba_of_value(s.value_at(ck, x, y));
    let v = f.stamp(t, at, (s.width(), s.height()), fg, below);
    v.into_iter()
        .map(|(p, c)| (p, s.value_of_rgba(c)))
        .collect()
}

impl PixelApp {
    /// A caixa de texto aberta: `(chave, âncora, texto)`.
    #[must_use]
    pub fn text_box(&self) -> Option<(u32, Cell, &str)> {
        self.texto
            .as_ref()
            .map(|c| (c.key, c.at, c.ed.text.as_str()))
    }

    /// O retângulo da caixa `(x, y, w, h)` e o cursor `(x, y, altura)`, em células do sprite.
    #[must_use]
    pub fn text_box_frame(&self) -> Option<((i32, i32, u32, u32), (i32, i32, i32))> {
        self.texto.as_ref().map(|c| (c.moldura, c.caret))
    }

    /// O contorno desenhado da caixa `(x, y, w, h)`: envolve a moldura e a tinta com 1 célula de folga.
    #[must_use]
    pub fn text_box_outline(&self) -> Option<(i32, i32, u32, u32)> {
        self.texto.as_ref().map(|c| c.contorno)
    }
}

/// Abre a caixa em `at` no documento `key` (a aberta assenta antes). A camada que não se pinta recusa.
pub(crate) fn abre(app: &mut PixelApp, key: u32, at: Cell) {
    fecha(app, true);
    if let Some(motivo) = app.docs.get(&key).and_then(crate::camadas::recusa_pintar) {
        return app.recusa(motivo);
    }
    app.texto = Some(Caixa {
        key,
        at,
        ..Caixa::default()
    });
    refaz(app);
}

/// Edit ▸ Insert Text: a caixa no centro do sprite.
pub(crate) fn insere(app: &mut PixelApp, key: u32) {
    let Some(d) = app.docs.get(&key) else {
        return;
    };
    let at = (d.sprite.width() as i32 / 2, d.sprite.height() as i32 / 2);
    crate::input::set_tool(app, crate::Tool::Text);
    abre(app, key, at);
}

/// Recalcula o preview, a moldura e o cursor (o texto, as opções ou a cor mudaram).
pub(crate) fn refaz(app: &mut PixelApp) {
    let style = estilo(&app.options);
    let fg = app.fg;
    let stamp = app.stamp();
    let Some(c) = app.texto.as_mut() else {
        return;
    };
    let Some(doc) = app.docs.get_mut(&c.key) else {
        app.texto = None;
        return;
    };
    let s = &doc.sprite;
    let ck = (doc.layer, doc.frame);
    // Toda escrita de píxeis corta-se à selecção (o preview é o mesmo traço).
    let mut v = valores(s, ck, &c.ed.text, &style, c.at, fg);
    (c.moldura, c.caret) = style.frame(&c.ed.text, c.at, c.ed.cursor);
    let (mx, my, mw, mh) = c.moldura;
    let (mut x0, mut y0, mut x1, mut y1) = (mx, my, mx + mw as i32 - 1, my + mh as i32 - 1);
    for &((x, y), _) in &v {
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
    }
    let lado = |a: i32, b: i32| u32::try_from(b - a + 3).unwrap_or(1);
    c.contorno = (x0 - 1, y0 - 1, lado(x0, x1), lado(y0, y1));
    if let Some(m) = s.mask() {
        v.retain(|&((x, y), _)| m.get(x, y));
    }
    c.traco = (!v.is_empty()).then(|| Stroke::with_values(ck.0, ck.1, v));
    doc.stamp = stamp;
}

/// O preview do texto sobre `key`, se há.
pub(crate) fn previa(app: &PixelApp, key: u32) -> Option<Stroke> {
    app.texto
        .as_ref()
        .filter(|c| c.key == key)
        .and_then(|c| c.traco.clone())
}

/// Fecha a caixa: `grava` = carimba UM passo (texto vazio = nenhum), senão larga (`Esc`).
pub(crate) fn fecha(app: &mut PixelApp, grava: bool) -> Option<KeyAnswer> {
    let c = app.texto.take()?;
    let doc = app.docs.get(&c.key).filter(|_| grava);
    let op = c.traco.zip(doc).and_then(|(t, d)| t.op(&d.sprite));
    app.passo(c.key, op);
    let stamp = app.stamp();
    app.docs.get_mut(&c.key)?.stamp = stamp;
    Some(KeyAnswer::Quiet)
}

/// Com a caixa aberta o teclado é do texto: toda tecla premida é consumida (os atalhos de
/// ferramenta viram letras). `text` é o que o teclado escreveu (o do sistema, com acentos).
/// `None` = não há caixa.
pub(crate) fn tecla(
    app: &mut PixelApp,
    code: KeyCode,
    m: Mods,
    text: Option<&str>,
) -> Option<KeyAnswer> {
    let c = app.texto.as_mut()?;
    let e = match code {
        KeyCode::Escape => return fecha(app, false),
        KeyCode::Enter | KeyCode::NumpadEnter if !m.shift => return fecha(app, true),
        KeyCode::Enter | KeyCode::NumpadEnter => Edit::Insert("\n"),
        KeyCode::Backspace => Edit::Backspace,
        KeyCode::Delete => Edit::Delete,
        KeyCode::ArrowLeft => Edit::Left,
        KeyCode::ArrowRight => Edit::Right,
        KeyCode::Home => Edit::Home,
        KeyCode::End => Edit::End,
        _ => Edit::Insert(text.filter(|_| !m.ctrl && !m.alt).unwrap_or("")),
    };
    if c.ed.apply(e) {
        refaz(app);
    }
    Some(KeyAnswer::Quiet)
}
