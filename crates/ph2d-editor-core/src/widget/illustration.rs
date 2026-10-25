//! ⭐ **A ILUSTRAÇÃO de uma dica** (GameEngineFuncional N1, `docs/GameEngineFuncional/05_N1_plano.md` §4):
//! um desenho pequeno descrito em DADOS — rectângulos, discos, cápsulas, linhas com seta e o contorno
//! tracejado de uma forma de colisão —, para a dica do menu Add mostrar o que o objecto É, como as do
//! Fusion 360.
//!
//! ⚠️ **Dados e não pixels:** as coordenadas são da MOLDURA (`0..1`, x para a direita, y para BAIXO) e as
//! cores são `ColorToken` — o desenho segue o tema e a escala, e nenhuma família traz imagens para o
//! chrome. Os raios e meias-alturas são fracções da ALTURA da moldura (um disco continua redondo numa
//! moldura larga).

use crate::paint::{fill_circle, fill_rounded_rect, resolve, stroke_polyline};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Affine, BezPath, Fill, Stroke, VectorScene};

/// Um traço do desenho. Ver o cabeçalho do módulo para as unidades.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mark {
    /// Rectângulo cheio; `round` é o raio dos cantos em fracção da altura da moldura.
    Rect {
        at: [f32; 2],
        size: [f32; 2],
        fill: ColorToken,
        round: f32,
    },
    /// Disco cheio de centro `c` e raio `r` (fracção da altura).
    Disc {
        c: [f32; 2],
        r: f32,
        fill: ColorToken,
    },
    /// Cápsula vertical de centro `c`: meia-altura do tronco `half_h` e raio `r` (fracções da altura).
    Capsule {
        c: [f32; 2],
        half_h: f32,
        r: f32,
        fill: ColorToken,
    },
    /// Linha de `a` a `b`; com `arrow`, uma ponta de seta em `b`.
    Line {
        a: [f32; 2],
        b: [f32; 2],
        stroke: ColorToken,
        arrow: bool,
    },
    /// Contorno TRACEJADO de um rectângulo — a forma de colisão, que não se vê no jogo.
    Dashed {
        at: [f32; 2],
        size: [f32; 2],
        stroke: ColorToken,
    },
}

// ⚠️ `PartialEq` sobre `f32` sem `Eq` partiria o `Eq` de quem guarda um `&'static [Mark]` (a `AddEntry`
// compara-se com `==` em todo o lado). Os desenhos são constantes autoradas, nunca `NaN`.
impl Eq for Mark {}

/// A ponta da seta, em fracção da altura da moldura.
const ARROW_HEAD: f32 = 0.07; // LITERAL-PX-OK: fracção da moldura, não um comprimento de UI
/// O traço e o vão do tracejado, em múltiplos da largura do traço.
const DASH_ON: f64 = 3.0; // LITERAL-PX-OK: múltiplo do traço, não um comprimento de UI
const DASH_OFF: f64 = 2.5; // LITERAL-PX-OK: múltiplo do traço, não um comprimento de UI

/// ⭐ **Pinta `marks` dentro de `frame`**, por ordem (o último por cima).
pub fn paint_illustration(scene: &mut VectorScene, frame: Rect, marks: &[Mark], theme: Theme) {
    let px = |p: [f32; 2]| (frame.x + p[0] * frame.w, frame.y + p[1] * frame.h);
    let h = frame.h;
    let width = StrokeToken::Thick.px();
    for mark in marks {
        match *mark {
            Mark::Rect {
                at,
                size,
                fill,
                round,
            } => {
                let (x, y) = px(at);
                fill_rounded_rect(
                    scene,
                    Rect::new(x, y, size[0] * frame.w, size[1] * frame.h),
                    round * h,
                    resolve(fill, theme),
                );
            }
            Mark::Disc { c, r, fill } => {
                let (x, y) = px(c);
                fill_circle(scene, x, y, r * h, resolve(fill, theme));
            }
            Mark::Capsule { c, half_h, r, fill } => {
                let (x, y) = px(c);
                let (rr, hh) = (r * h, half_h * h);
                fill_rounded_rect(
                    scene,
                    Rect::new(x - rr, y - hh - rr, rr * 2.0, (hh + rr) * 2.0),
                    rr,
                    resolve(fill, theme),
                );
            }
            Mark::Line {
                a,
                b,
                stroke,
                arrow,
            } => {
                let (pa, pb) = (px(a), px(b));
                let color = resolve(stroke, theme);
                stroke_polyline(scene, &[pa, pb], width, color);
                if arrow {
                    paint_arrow_head(scene, pa, pb, ARROW_HEAD * h, color);
                }
            }
            Mark::Dashed { at, size, stroke } => {
                let (x, y) = px(at);
                let (w, hh) = (size[0] * frame.w, size[1] * frame.h);
                let mut path = BezPath::new();
                path.move_to((x as f64, y as f64));
                path.line_to(((x + w) as f64, y as f64));
                path.line_to(((x + w) as f64, (y + hh) as f64));
                path.line_to((x as f64, (y + hh) as f64));
                path.close_path();
                let wd = width as f64;
                let style = Stroke::new(wd).with_dashes(0.0, [DASH_ON * wd, DASH_OFF * wd]);
                scene.inner_mut().stroke(
                    &style,
                    Affine::IDENTITY,
                    resolve(stroke, theme),
                    None,
                    &path,
                );
            }
        }
    }
}

/// Um triângulo cheio com a ponta em `tip`, apontado na direcção `from → tip`.
fn paint_arrow_head(
    scene: &mut VectorScene,
    from: (f32, f32),
    tip: (f32, f32),
    len: f32,
    color: ph2d_vector::Color,
) {
    let (dx, dy) = (tip.0 - from.0, tip.1 - from.1);
    let n = (dx * dx + dy * dy).sqrt();
    if n <= f32::EPSILON || len <= 0.0 {
        return;
    }
    let (ux, uy) = (dx / n, dy / n);
    let (bx, by) = (tip.0 - ux * len, tip.1 - uy * len);
    let half = len * 0.5;
    let mut path = BezPath::new();
    path.move_to((tip.0 as f64, tip.1 as f64));
    path.line_to(((bx - uy * half) as f64, (by + ux * half) as f64));
    path.line_to(((bx + uy * half) as f64, (by - ux * half) as f64));
    path.close_path();
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
}

#[cfg(test)]
#[path = "illustration/tests.rs"]
mod tests;
