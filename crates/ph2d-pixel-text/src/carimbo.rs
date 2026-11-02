//! **O carimbo do texto** (P8, a ferramenta Text do Pixel): onde a máscara cai com a âncora no
//! canto de cima-esquerda da caixa (a convenção medida no Trial, `tests/it/trial_text.rs`), as
//! células que acende com a cobertura, a mistura AA por cima da cor de baixo, a moldura e o
//! cursor da caixa, e a edição do texto (cursor em caracteres). O app só liga.

use crate::{FontChoice, TextMask, TextStyle};
use std::sync::Arc;

/// Uma célula `(x, y)`.
pub type Cell = (i32, i32);

/// A fonte resolvida: o estilo e, para uma do sistema, os bytes da face (`FontChoice::System`
/// diz o índice; o caminho é só o nome).
#[derive(Clone, Debug)]
pub struct Fonte {
    pub style: TextStyle,
    dados: Option<Arc<[u8]>>,
}

impl Fonte {
    /// Uma fonte de fábrica (ou de ficheiro, lida por [`crate::rasterize`]).
    #[must_use]
    pub fn new(style: TextStyle) -> Self {
        Self { style, dados: None }
    }

    /// Uma face do sistema já em memória (os bytes do ficheiro e o índice da face).
    #[must_use]
    pub fn with_bytes(mut style: TextStyle, name: &str, bytes: Vec<u8>, index: u32) -> Self {
        style.font = FontChoice::System(name.into(), index);
        Self {
            style,
            dados: Some(Arc::from(bytes)),
        }
    }

    /// Do topo da caixa à linha de base da 1.ª linha (`ascent` na escala do estilo) e a altura de
    /// uma linha.
    #[must_use]
    pub fn metrics(&self) -> Option<(i32, i32)> {
        match (&self.style.font, &self.dados) {
            (FontChoice::System(_, index), Some(data)) => {
                let face = crate::outline::face(data, *index).ok()?;
                let m = crate::outline::face_metrics(&face, self.style.px);
                Some((m.ascent, m.ascent + m.descent))
            }
            (FontChoice::Builtin(nome), _) => {
                let p = crate::builtin(nome).ok()?;
                let lh = p.line_height().max(1);
                let s = (self.style.px / lh as u32).max(1) as i32;
                Some((p.ascent * s, lh * s))
            }
            _ => None,
        }
    }

    /// A máscara de `texto` e onde a célula `(0, 0)` dela cai com a âncora `at`.
    #[must_use]
    pub fn place(&self, texto: &str, at: Cell) -> Option<(TextMask, Cell)> {
        let m = match (&self.style.font, &self.dados) {
            (FontChoice::System(_, index), Some(data)) => {
                crate::rasterize_outline(data, *index, texto, &self.style).ok()?
            }
            _ => crate::rasterize(texto, &self.style).ok()?,
        };
        let (ascent, _) = self.metrics()?;
        let o = (at.0 - m.origin_x, at.1 + ascent - m.baseline);
        Some((m, o))
    }

    /// As células acesas dentro de `w × h`, com a cobertura `1..=255`.
    #[must_use]
    pub fn cells(&self, texto: &str, at: Cell, (w, h): (u32, u32)) -> Vec<(Cell, u8)> {
        let Some((m, o)) = self.place(texto, at) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for my in 0..m.h {
            for mx in 0..m.w {
                let c = m.cover[(my * m.w + mx) as usize];
                let (x, y) = (o.0 + mx as i32, o.1 + my as i32);
                if c > 0 && x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
                    out.push(((x, y), c));
                }
            }
        }
        out
    }

    /// A moldura da caixa `(x, y, w, h)` e o cursor `(x, y, altura)` com o cursor no caracter
    /// `cursor` (Shift+Enter = `\n`).
    #[must_use]
    pub fn frame(
        &self,
        texto: &str,
        at: Cell,
        cursor: usize,
    ) -> ((i32, i32, u32, u32), (i32, i32, i32)) {
        let (_, lh) = self.metrics().unwrap_or((0, 1));
        let largura = |t: &str| {
            self.place(t, at)
                .map_or(0, |(m, o)| (o.0 + m.w as i32 - at.0).max(0) as u32)
        };
        let linhas: Vec<&str> = texto.split('\n').collect();
        let w = linhas.iter().map(|l| largura(l)).max().unwrap_or(0);
        let antes: String = texto.chars().take(cursor).collect();
        let fila = antes.matches('\n').count() as i32;
        let resto = antes.rsplit('\n').next().unwrap_or("");
        // O avanço da caneta depois de `resto`: um ponto de medida a seguir guarda o avanço final.
        let x = match (self.place(&format!("{resto}."), at), self.place(".", at)) {
            (Some((a, oa)), Some((b, ob))) if !resto.is_empty() => {
                (oa.0 + a.w as i32) - (ob.0 + b.w as i32)
            }
            _ => 0,
        };
        let moldura = (at.0, at.1, w, (lh * linhas.len() as i32) as u32);
        (moldura, (at.0 + x, at.1 + fila * lh, lh))
    }
}

/// A cor que o texto deixa numa célula de cobertura `c` sobre `dst`: cobertura cheia = `fg` tal e
/// qual, parcial (Antialias) = [`over`]. A lei do carimbo e do texto solto que assenta.
#[must_use]
pub fn tinge(fg: [u8; 4], c: u8, dst: [u8; 4]) -> [u8; 4] {
    if c == u8::MAX { fg } else { over(fg, c, dst) }
}

/// `fg` com o alfa multiplicado por `c/255`, por cima de `dst` (alfa reto).
#[must_use]
pub fn over(fg: [u8; 4], c: u8, dst: [u8; 4]) -> [u8; 4] {
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

/// Uma tecla de edição da caixa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit<'a> {
    Insert(&'a str),
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
}

/// O texto e o cursor (em caracteres) da caixa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Editing {
    pub text: String,
    pub cursor: usize,
}

impl Editing {
    fn byte(&self, n: usize) -> usize {
        self.text
            .char_indices()
            .nth(n)
            .map_or(self.text.len(), |(i, _)| i)
    }

    /// Aplica `e`; `false` = nada mudou (os caracteres de controlo não entram).
    pub fn apply(&mut self, e: Edit<'_>) -> bool {
        let n = self.text.chars().count();
        let antes = (self.text.len(), self.cursor);
        match e {
            Edit::Insert(s) => {
                let novo: String = s
                    .chars()
                    .filter(|c| *c == '\n' || !c.is_control())
                    .collect();
                let i = self.byte(self.cursor);
                self.text.insert_str(i, &novo);
                self.cursor += novo.chars().count();
            }
            Edit::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                let i = self.byte(self.cursor);
                self.text.remove(i);
            }
            Edit::Delete if self.cursor < n => {
                let i = self.byte(self.cursor);
                self.text.remove(i);
            }
            Edit::Left => self.cursor = self.cursor.saturating_sub(1),
            Edit::Right => self.cursor = (self.cursor + 1).min(n),
            Edit::Home => self.cursor = 0,
            Edit::End => self.cursor = n,
            _ => {}
        }
        (self.text.len(), self.cursor) != antes
    }
}

/// As famílias do sistema mais usadas em pixel art: as instaladas sobem para logo depois das de
/// fábrica, nesta ordem (casam sem olhar a maiúsculas, espaços nem hífens).
pub const PIXEL_POPULAR: [&str; 12] = [
    "Press Start 2P",
    "Silkscreen",
    "VT323",
    "Pixelify Sans",
    "m5x7",
    "m3x6",
    "Monogram",
    "Pixel Operator",
    "PixelOperator",
    "Terminus",
    "Tamzen",
    "Spleen",
];

fn chave(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// As fontes do Text, na ordem do índice: as de fábrica, as do sistema de [`PIXEL_POPULAR`] que
/// existem (na ordem dela) e depois as restantes de `system` (na ordem do seletor do Vector).
#[must_use]
pub fn font_list(system: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut resto: Vec<String> = system.into_iter().collect();
    let mut v: Vec<String> = crate::BUILTIN.iter().map(|b| b.0.to_owned()).collect();
    for p in PIXEL_POPULAR.map(chave) {
        while let Some(k) = resto.iter().position(|f| chave(f) == p) {
            v.push(resto.remove(k));
        }
    }
    v.extend(resto);
    v
}

/// Quantas das primeiras de [`font_list`] são de pixel (de fábrica ou de [`PIXEL_POPULAR`]).
#[must_use]
pub fn pixel_popular_count(list: &[String]) -> usize {
    let pop = PIXEL_POPULAR.map(chave);
    crate::BUILTIN.len()
        + list
            .iter()
            .skip(crate::BUILTIN.len())
            .take_while(|f| pop.contains(&chave(f)))
            .count()
}

/// O índice `i` numa lista de `n` (Previous/Next dão a volta: `u16::MAX` = a última).
#[must_use]
pub fn wrap_index(i: u16, n: usize) -> u16 {
    let n = u16::try_from(n).unwrap_or(u16::MAX).max(1);
    if i == u16::MAX { n - 1 } else { i % n }
}

impl Fonte {
    /// A fonte `i` de `list` ([`font_list`]) com o resto do estilo de `style`: de fábrica, ou a face
    /// que `load(família, bold, italic)` devolve (sem ela, a 1.ª de fábrica).
    #[must_use]
    pub fn pick(
        i: u16,
        list: &[String],
        mut style: TextStyle,
        load: impl FnOnce(&str, bool, bool) -> Option<(Vec<u8>, u32)>,
    ) -> Self {
        let i = usize::from(i);
        let nome = crate::BUILTIN.get(i).map_or(crate::BUILTIN[0].0, |b| b.0);
        style.font = FontChoice::Builtin(nome.to_owned());
        let sistema = list.get(i).filter(|_| i >= crate::BUILTIN.len());
        match sistema.and_then(|f| Some((f, load(f, style.bold, style.italic)?))) {
            Some((f, (bytes, index))) => Self::with_bytes(style, f, bytes, index),
            None => Self::new(style),
        }
    }

    /// As células que `texto` carimba com `fg`: a cobertura cheia é `fg`, a parcial (Antialias) é
    /// `fg` por cima de `below(célula)` ([`over`]).
    #[must_use]
    pub fn stamp(
        &self,
        texto: &str,
        at: Cell,
        size: (u32, u32),
        fg: [u8; 4],
        below: impl Fn(Cell) -> [u8; 4],
    ) -> Vec<(Cell, [u8; 4])> {
        let cells = self.cells(texto, at, size);
        cells
            .into_iter()
            .map(|(p, c)| (p, tinge(fg, c, below(p))))
            .collect()
    }
}
