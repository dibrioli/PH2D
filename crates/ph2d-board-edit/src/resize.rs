//! A geometria dos gestos de caixa (sem estado): a caixa de criar, a moldura depois de puxar uma
//! pega, e os elementos que seguem a moldura (as notas levam a letra).

use ph2d_board_model::Element;

use crate::gesture::{MIN_SIDE, is_note, rect};
use crate::{Dir, Frame, Mods};

/// O ângulo é múltiplo de 90°?
pub(crate) fn axis_angle(a: f64) -> bool {
    let (s, c) = a.sin_cos();
    s.abs() < 1e-9 || c.abs() < 1e-9
}

/// A caixa de criação de `start` a `p`: `Shift` = quadrada, `Alt` = centrada em `start`.
pub(crate) fn drag_box(start: [f64; 2], p: [f64; 2], m: Mods) -> [f64; 4] {
    let mut d = [p[0] - start[0], p[1] - start[1]];
    if m.shift {
        let side = d[0].abs().max(d[1].abs());
        d = [side.copysign(d[0]), side.copysign(d[1])];
    }
    let (a, b) = if m.alt {
        (
            [start[0] - d[0], start[1] - d[1]],
            [start[0] + d[0], start[1] + d[1]],
        )
    } else {
        (start, [start[0] + d[0], start[1] + d[1]])
    };
    let r = rect(a, b);
    [
        r[0],
        r[1],
        (r[2] - r[0]).max(MIN_SIDE),
        (r[3] - r[1]).max(MIN_SIDE),
    ]
}

/// ⭐ **A moldura depois de arrastar a pega `dir` até `l`** (local, relativo ao centro antes de
/// rodar). O lado oposto fica fixo (com `Alt`, o centro); `Shift` mantém a proporção.
pub(crate) fn resize_frame(f: Frame, dir: Dir, l: [f64; 2], m: Mods) -> Frame {
    let (sx, sy) = dir.sign();
    let (hw, hh) = (f.w / 2.0, f.h / 2.0);
    // O ponto fixo e a extensão pedida em cada eixo que esta pega mexe.
    let anchor = if m.alt {
        [0.0, 0.0]
    } else {
        [-sx * hw, -sy * hh]
    };
    let reach = |axis: usize, s: f64| -> f64 {
        if s == 0.0 {
            return if axis == 0 { f.w } else { f.h };
        }
        let d = l[axis] - anchor[axis];
        if m.alt { 2.0 * d.abs() } else { d.abs() }
    };
    let (mut w, mut h) = (reach(0, sx).max(MIN_SIDE), reach(1, sy).max(MIN_SIDE));
    if m.shift && f.w > 0.0 && f.h > 0.0 {
        let k = match (sx != 0.0, sy != 0.0) {
            (true, true) => (w / f.w).max(h / f.h),
            (true, false) => w / f.w,
            _ => h / f.h,
        };
        (w, h) = ((f.w * k).max(MIN_SIDE), (f.h * k).max(MIN_SIDE));
    }
    // O centro novo (local): do ponto fixo para o lado onde o dedo está.
    let centre = |axis: usize, s: f64, size: f64| -> f64 {
        if s == 0.0 || m.alt {
            return if s == 0.0 { 0.0 } else { anchor[axis] };
        }
        let toward = (l[axis] - anchor[axis]).signum();
        let toward = if toward == 0.0 { s } else { toward };
        anchor[axis] + toward * size / 2.0
    };
    let c = [centre(0, sx, w), centre(1, sy, h)];
    Frame {
        center: f.point(c),
        w,
        h,
        angle: f.angle,
    }
}

/// Os elementos de uma moldura `old` que passou a `new`. Um elemento só segue a moldura (é ela);
/// vários escalam as posições e os tamanhos com ela (a moldura de vários não roda).
pub(crate) fn resized(originals: &[Element], old: Frame, new: Frame) -> Vec<Element> {
    // A letra de uma nota escala com ela (a nota inteira cresce, não só a caixa).
    let font = |el: &mut Element, o: &Element| {
        if is_note(o) && o.w > 0.0 {
            el.style_mut().font_size = o.style().font_size * el.w / o.w;
        }
    };
    if let [one] = originals {
        let mut el = one.clone();
        el.w = new.w;
        el.h = new.h;
        el.x = new.center[0] - new.w / 2.0;
        el.y = new.center[1] - new.h / 2.0;
        font(&mut el, one);
        return vec![el];
    }
    let (kx, ky) = (
        new.w / old.w.max(f64::EPSILON),
        new.h / old.h.max(f64::EPSILON),
    );
    let o0 = [old.center[0] - old.w / 2.0, old.center[1] - old.h / 2.0];
    let n0 = [new.center[0] - new.w / 2.0, new.center[1] - new.h / 2.0];
    originals
        .iter()
        .map(|o| {
            let mut el = o.clone();
            let c = o.center();
            let nc = [n0[0] + (c[0] - o0[0]) * kx, n0[1] + (c[1] - o0[1]) * ky];
            let quarter = o.angle.sin().abs() > 0.5;
            let (fx, fy) = if quarter { (ky, kx) } else { (kx, ky) };
            el.w = (o.w * fx).max(MIN_SIDE);
            el.h = (o.h * fy).max(MIN_SIDE);
            el.x = nc[0] - el.w / 2.0;
            el.y = nc[1] - el.h / 2.0;
            font(&mut el, o);
            el
        })
        .collect()
}
