//! **A ferramenta Text** (P8, `docs/PixelArt/P8_plano.md` §2 itens 8–10 e §9.3): o clique abre uma
//! caixa de texto com o canto de cima-esquerda na célula (a convenção medida no Trial,
//! `ph2d-pixel-text/tests/it/trial_text.rs`); teclar escreve e o canvas mostra o preview
//! (`Stroke::with_values`, como os filtros); Enter (clicar fora, trocar de ferramenta) carimba os
//! píxeis num passo do desfazer; Esc larga sem gravar. O texto vive aqui, nunca no documento.

use ph2d_pixel_core::raster::Cell;
use ph2d_pixel_core::{Sprite, Stroke};
use ph2d_pixel_text::{BUILTIN, FontChoice, TextMask, TextStyle};
use winit::keyboard::KeyCode;

use crate::gesture::Mods;
use crate::input::KeyAnswer;
use crate::{Options, PixelApp};

/// A caixa aberta: o documento, a âncora, o texto, o cursor (em caracteres) e o preview.
#[derive(Clone)]
pub(crate) struct Caixa {
    key: u32,
    at: Cell,
    texto: String,
    cursor: usize,
    traco: Option<Stroke>,
    /// O retângulo da caixa em células (x, y, w, h) e a célula do cursor `(x, y, altura)`.
    moldura: (i32, i32, u32, u32),
    caret: (i32, i32, i32),
}

/// A fonte resolvida: o estilo e, para uma do sistema, os bytes da face (Bold/Italic escolhem a
/// face mais perto, `ph2d_system_fonts::library::face_bytes`).
#[derive(Clone)]
pub struct Fonte {
    pub style: TextStyle,
    dados: Option<std::sync::Arc<[u8]>>,
}

/// As fontes que o Text oferece, na ordem do índice `text_font`: as de fábrica e as do sistema
/// (a mesma ordem do seletor do Vector).
#[must_use]
pub fn fontes() -> Vec<String> {
    let fabrica = BUILTIN.iter().map(|b| b.0.to_owned());
    let sistema = ph2d_system_fonts::library::pickable_families()
        .into_iter()
        .flatten();
    fabrica.chain(sistema).collect()
}

/// O índice `i` dentro da lista (os botões Previous/Next dão a volta: `u16::MAX` = a última).
#[must_use]
pub fn normaliza(i: u16) -> u16 {
    let n = u16::try_from(fontes().len()).unwrap_or(u16::MAX).max(1);
    if i == u16::MAX { n - 1 } else { i % n }
}

/// O nome da fonte `i`.
#[must_use]
pub fn nome_da_fonte(i: u16) -> String {
    fontes().into_iter().nth(usize::from(i)).unwrap_or_default()
}

/// O estilo das opções da mão.
#[must_use]
pub fn estilo(o: &Options) -> Fonte {
    let i = usize::from(o.text_font);
    let (font, dados) = match BUILTIN.get(i) {
        Some(b) => (FontChoice::Builtin(b.0.to_owned()), None),
        None => {
            let fam = nome_da_fonte(o.text_font);
            match ph2d_system_fonts::library::face_bytes(&fam, o.text_bold, o.text_italic) {
                Some((bytes, idx)) => (
                    FontChoice::System(std::path::PathBuf::from(&fam), idx),
                    Some(std::sync::Arc::from(bytes)),
                ),
                None => (FontChoice::Builtin(BUILTIN[0].0.to_owned()), None),
            }
        }
    };
    let style = TextStyle {
        font,
        px: o.text_px.max(1),
        bold: o.text_bold,
        italic: o.text_italic,
        antialias: o.text_antialias,
        hinting: o.text_hinting,
    };
    Fonte { style, dados }
}

/// O topo da caixa até à linha de base da 1.ª linha (`ascent` na escala do estilo) e a altura de
/// uma linha.
fn metricas(f: &Fonte) -> Option<(i32, i32)> {
    match (&f.style.font, &f.dados) {
        (FontChoice::System(_, index), Some(data)) => {
            let face = ph2d_pixel_text::outline::face(data, *index).ok()?;
            let m = ph2d_pixel_text::outline::face_metrics(&face, f.style.px);
            Some((m.ascent, m.ascent + m.descent))
        }
        (FontChoice::Builtin(nome), _) => {
            let p = ph2d_pixel_text::builtin(nome).ok()?;
            let lh = p.line_height().max(1);
            let s = (f.style.px / lh as u32).max(1) as i32;
            Some((p.ascent * s, lh * s))
        }
        _ => None,
    }
}

/// A máscara de `texto` e onde a célula `(0, 0)` dela cai com a âncora `at`.
#[must_use]
pub fn coloca(texto: &str, f: &Fonte, at: Cell) -> Option<(TextMask, Cell)> {
    let m = match (&f.style.font, &f.dados) {
        (FontChoice::System(_, index), Some(data)) => {
            ph2d_pixel_text::rasterize_outline(data, *index, texto, &f.style).ok()?
        }
        _ => ph2d_pixel_text::rasterize(texto, &f.style).ok()?,
    };
    let (ascent, _) = metricas(f)?;
    let o = (at.0 - m.origin_x, at.1 + ascent - m.baseline);
    Some((m, o))
}

/// As células que o texto carimba com a cor `fg` sobre o cel `ck`: com Antialias a cobertura é o
/// alfa por cima do que lá está.
#[must_use]
pub fn valores(
    s: &Sprite,
    ck: ph2d_pixel_core::CelKey,
    texto: &str,
    style: &Fonte,
    at: Cell,
    fg: [u8; 4],
) -> Vec<(Cell, u32)> {
    let Some((m, o)) = coloca(texto, style, at) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for my in 0..m.h {
        for mx in 0..m.w {
            let c = m.cover[(my * m.w + mx) as usize];
            if c == 0 {
                continue;
            }
            let (x, y) = (o.0 + mx as i32, o.1 + my as i32);
            if x < 0 || y < 0 || x >= s.width() as i32 || y >= s.height() as i32 {
                continue;
            }
            let rgba = if c == u8::MAX {
                fg
            } else {
                sobre(fg, c, s.rgba_of_value(s.value_at(ck, x, y)))
            };
            out.push(((x, y), s.value_of_rgba(rgba)));
        }
    }
    out
}

/// `fg` com o alfa multiplicado por `c/255`, por cima de `dst` (alfa reto).
fn sobre(fg: [u8; 4], c: u8, dst: [u8; 4]) -> [u8; 4] {
    let a = u32::from(fg[3]) * u32::from(c) / 255;
    let da = u32::from(dst[3]) * (255 - a) / 255;
    let oa = a + da;
    if oa == 0 {
        return [0; 4];
    }
    let mut out = [0u8; 4];
    for k in 0..3 {
        out[k] = ((u32::from(fg[k]) * a + u32::from(dst[k]) * da) / oa) as u8;
    }
    out[3] = oa as u8;
    out
}

impl PixelApp {
    /// A caixa de texto aberta: `(chave, âncora, texto)`.
    #[must_use]
    pub fn text_box(&self) -> Option<(u32, Cell, &str)> {
        self.texto.as_ref().map(|c| (c.key, c.at, c.texto.as_str()))
    }

    /// O retângulo da caixa `(x, y, w, h)` e o cursor `(x, y, altura)`, em células do sprite.
    #[must_use]
    pub fn text_box_frame(&self) -> Option<((i32, i32, u32, u32), (i32, i32, i32))> {
        self.texto.as_ref().map(|c| (c.moldura, c.caret))
    }
}

/// Abre a caixa em `at` no documento `key` (a aberta assenta antes). A camada que não se pinta recusa.
pub(crate) fn abre(app: &mut PixelApp, key: u32, at: Cell) {
    confirma(app);
    if let Some(motivo) = app.docs.get(&key).and_then(crate::camadas::recusa_pintar) {
        return app.recusa(motivo);
    }
    app.texto = Some(Caixa {
        key,
        at,
        texto: String::new(),
        cursor: 0,
        traco: None,
        moldura: (at.0, at.1, 0, 0),
        caret: (at.0, at.1, 0),
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
    let v = valores(s, ck, &c.texto, &style, c.at, fg);
    c.traco = (!v.is_empty()).then(|| Stroke::with_values(ck.0, ck.1, v));
    let (_, lh) = metricas(&style).unwrap_or((0, 1));
    let linhas: Vec<&str> = c.texto.split('\n').collect();
    let largura = |t: &str| {
        coloca(t, &style, c.at).map_or(0, |(m, o)| (o.0 + m.w as i32 - c.at.0).max(0) as u32)
    };
    let w = linhas.iter().map(|l| largura(l)).max().unwrap_or(0);
    c.moldura = (c.at.0, c.at.1, w, (lh * linhas.len() as i32) as u32);
    let antes: String = c.texto.chars().take(c.cursor).collect();
    let fila = antes.matches('\n').count() as i32;
    let resto = antes.rsplit('\n').next().unwrap_or("");
    // A largura do avanço: a caneta depois de `resto` (um ponto de medida garante o avanço final).
    let x = if resto.is_empty() {
        0
    } else {
        let com = coloca(&format!("{resto}."), &style, c.at);
        let ponto = coloca(".", &style, c.at);
        match (com, ponto) {
            (Some((a, oa)), Some((b, ob))) => (oa.0 + a.w as i32) - (ob.0 + b.w as i32),
            _ => 0,
        }
    };
    c.caret = (c.at.0 + x, c.at.1 + fila * lh, lh);
    doc.stamp = stamp;
}

/// O preview do texto sobre `key`, se há.
pub(crate) fn previa(app: &PixelApp, key: u32) -> Option<Stroke> {
    app.texto
        .as_ref()
        .filter(|c| c.key == key)
        .and_then(|c| c.traco.clone())
}

/// Carimba o texto: UM passo (texto vazio = nenhum). `false` = não havia caixa.
pub(crate) fn confirma(app: &mut PixelApp) -> bool {
    let Some(c) = app.texto.take() else {
        return false;
    };
    let op = c
        .traco
        .as_ref()
        .zip(app.docs.get(&c.key))
        .and_then(|(t, d)| t.op(&d.sprite));
    app.passo(c.key, op);
    toca(app, c.key);
    true
}

/// Larga a caixa sem gravar (`Esc`).
pub(crate) fn cancela(app: &mut PixelApp) -> bool {
    let Some(c) = app.texto.take() else {
        return false;
    };
    toca(app, c.key);
    true
}

fn toca(app: &mut PixelApp, key: u32) {
    let stamp = app.stamp();
    if let Some(d) = app.docs.get_mut(&key) {
        d.stamp = stamp;
    }
}

/// O byte do caracter `n` de `s`.
fn byte(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map_or(s.len(), |(i, _)| i)
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
    match code {
        KeyCode::Escape => {
            cancela(app);
            return Some(KeyAnswer::Quiet);
        }
        KeyCode::Enter | KeyCode::NumpadEnter if !m.shift => {
            confirma(app);
            return Some(KeyAnswer::Quiet);
        }
        KeyCode::Enter | KeyCode::NumpadEnter => {
            let i = byte(&c.texto, c.cursor);
            c.texto.insert(i, '\n');
            c.cursor += 1;
        }
        KeyCode::Backspace if c.cursor > 0 => {
            c.cursor -= 1;
            let i = byte(&c.texto, c.cursor);
            c.texto.remove(i);
        }
        KeyCode::Delete if c.cursor < c.texto.chars().count() => {
            let i = byte(&c.texto, c.cursor);
            c.texto.remove(i);
        }
        KeyCode::ArrowLeft => c.cursor = c.cursor.saturating_sub(1),
        KeyCode::ArrowRight => c.cursor = (c.cursor + 1).min(c.texto.chars().count()),
        KeyCode::Home => c.cursor = 0,
        KeyCode::End => c.cursor = c.texto.chars().count(),
        _ => {
            let novo: String = text
                .filter(|_| !m.ctrl && !m.alt)
                .unwrap_or("")
                .chars()
                .filter(|ch| !ch.is_control())
                .collect();
            if novo.is_empty() {
                return Some(KeyAnswer::Quiet);
            }
            let i = byte(&c.texto, c.cursor);
            c.texto.insert_str(i, &novo);
            c.cursor += novo.chars().count();
        }
    }
    refaz(app);
    Some(KeyAnswer::Quiet)
}
