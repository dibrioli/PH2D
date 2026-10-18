//! ⭐ **A geometria de UMA porta** — o x de um byte e o byte de um x saem do MESMO passeio pelos
//! grupos (clusters) do layout que a pintura desenhou; a pintura, o clique e o cursor nunca têm
//! contas próprias.

use crate::display::DisplayLine;
use crate::style::Ink;
use parley::Layout;

/// Uma coluna do ecrã: o grafema do buffer (byte relativo à linha) e a faixa de x que ele ocupa.
/// Os espaços de uma tab fundem-se numa coluna só (a tab).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Column {
    pub byte: usize,
    pub x0: f32,
    pub x1: f32,
}

/// Visita cada grupo mostrado, por ordem visual: (byte mostrado, x0, x1).
fn walk(layout: &Layout<Ink>, mut f: impl FnMut(usize, f32, f32) -> bool) {
    for line in layout.lines() {
        let mut x = line.metrics().offset;
        for run in line.runs() {
            for c in run.visual_clusters() {
                let adv = c.advance();
                if !f(c.text_range().start, x, x + adv) {
                    return;
                }
                x += adv;
            }
        }
    }
}

/// As colunas da linha (para o clique e para a sonda dos testes).
pub(crate) fn columns(layout: &Layout<Ink>, dl: &DisplayLine) -> Vec<Column> {
    let mut out: Vec<Column> = Vec::new();
    walk(layout, |d, x0, x1| {
        let byte = dl.owner(d);
        match out.last_mut() {
            Some(last) if last.byte == byte => last.x1 = x1,
            _ => out.push(Column { byte, x0, x1 }),
        }
        true
    });
    out
}

/// O x do byte MOSTRADO `d` (o fim do texto dá a largura).
pub(crate) fn x_of_disp(layout: &Layout<Ink>, d: usize) -> f32 {
    let mut end = 0.0_f32;
    let mut hit = None;
    walk(layout, |s, x0, x1| {
        end = end.max(x1);
        if s >= d {
            hit = Some(x0);
            return false;
        }
        true
    });
    hit.unwrap_or(end)
}

/// O x do byte do buffer `b` (relativo à linha).
pub(crate) fn x_of_byte(layout: &Layout<Ink>, dl: &DisplayLine, b: usize) -> f32 {
    x_of_disp(layout, dl.to_disp(b))
}

/// O x de onde uma faixa que começa no byte `b` se pinta: depois de uma dica nessa posição (I9).
pub(crate) fn x_of_byte_after(layout: &Layout<Ink>, dl: &DisplayLine, b: usize) -> f32 {
    x_of_disp(layout, dl.to_disp_after(b))
}

/// O byte do buffer (relativo à linha) mais próximo de `x`: o meio de cada coluna decide; um clique
/// sobre uma dica (I9) dá a posição dela, de qualquer lado.
pub(crate) fn byte_at_x(layout: &Layout<Ink>, dl: &DisplayLine, x: f32, line_len: usize) -> usize {
    let mut out = line_len;
    // (byte, x0, x1, é dica)
    let mut prev: Option<(usize, f32, f32, bool)> = None;
    let mut decide = |c: (usize, f32, f32, bool)| -> bool {
        let edge = if c.3 { c.2 } else { (c.1 + c.2) * 0.5 };
        if x < edge {
            out = c.0;
            return false;
        }
        true
    };
    let mut done = false;
    walk(layout, |d, x0, x1| {
        let byte = dl.owner(d);
        let hint = dl.is_inlay(d);
        match prev {
            Some(p) if p.0 == byte && p.3 == hint => prev = Some((byte, p.1, x1, hint)),
            Some(p) => {
                if !decide(p) {
                    done = true;
                    return false;
                }
                prev = Some((byte, x0, x1, hint));
            }
            None => prev = Some((byte, x0, x1, hint)),
        }
        true
    });
    if !done && let Some(p) = prev {
        decide(p);
    }
    out
}

/// A largura do texto mostrado.
pub(crate) fn width(layout: &Layout<Ink>) -> f32 {
    layout.full_width()
}
