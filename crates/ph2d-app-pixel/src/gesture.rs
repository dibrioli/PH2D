//! **O gesto de cada ferramenta** (Aseprite): o que o premir começa, o que mover faz, quando o
//! gesto acaba. Toda forma é redesenhada INTEIRA a cada movimento sobre o mesmo [`Stroke`], que a
//! vista mostra por cima do documento e que vira UM passo do desfazer ao acabar.

use ph2d_pixel_core::brush::{Brush, BrushShape};
use ph2d_pixel_core::fill::{Bucket, BucketSample};
use ph2d_pixel_core::ink::Ink;
use ph2d_pixel_core::raster::{
    Cell, curve_path, ellipse_cells, ellipse_filled_cells, line_cells, polygon_cells,
    rounded_rect_cells, snap_pixel_angle, square_box,
};
use ph2d_pixel_core::symmetry::{Mirrors, Symmetry, Tiling};
use ph2d_pixel_core::{Paint, Sprite, Stroke};

use crate::Tool;
use crate::state::{Doc, Options};

/// Os modificadores do Aseprite no ponteiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mods {
    /// Conta-gotas rápido.
    pub alt: bool,
    /// Formas: quadrado/círculo · Linha: ângulo de pixel art · Lápis: reta desde o último ponto.
    pub shift: bool,
    /// Formas: a partir do centro.
    pub ctrl: bool,
}

impl Mods {
    /// Os modificadores da janela.
    #[must_use]
    pub fn of(m: winit::keyboard::ModifiersState) -> Self {
        Self {
            alt: m.alt_key(),
            shift: m.shift_key(),
            ctrl: m.control_key() || m.super_key(),
        }
    }
}

/// O que o gesto em curso é.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Lápis, borracha: traço livre.
    Free,
    Spray,
    /// Linha, retângulo(s), elipse(s): da âncora ao cursor.
    Box {
        from: Cell,
        to: Cell,
    },
    /// A curva: arrasta-se a corda; depois o controlo segue o cursor e um clique acaba.
    Curve {
        from: Cell,
        to: Cell,
        ctrl: Option<Cell>,
    },
    /// O traço livre que fecha e enche ao largar.
    Contour(Vec<Cell>),
    /// Os vértices até agora (o último segue o cursor).
    Polygon(Vec<Cell>),
    /// O balde já encheu no premir.
    Fill,
    /// Arrastar um eixo da simetria (vertical = o do espelho H).
    Axis {
        vertical: bool,
    },
}

/// A tinta do chrome no vocabulário do núcleo.
pub(crate) fn ink(i: ph2d_editor_core::pixel_panel::PixelInk) -> Ink {
    use ph2d_editor_core::pixel_panel::PixelInk as I;
    match i {
        I::Simple => Ink::Simple,
        I::AlphaCompositing => Ink::AlphaCompositing,
        I::CopyAlphaColor => Ink::CopyAlphaColor,
        I::LockAlpha => Ink::LockAlpha,
        I::Shading => Ink::Shading,
    }
}

/// O pincel, a simetria, o ladrilho e a tinta das opções, para o documento `doc`.
pub(crate) fn paint(o: &Options, doc: &Doc) -> Paint {
    use ph2d_editor_core::pixel_panel::PixelBrushShape as S;
    let shape = match o.shape {
        S::Circle => BrushShape::Circle,
        S::Square => BrushShape::Square,
        S::Line => BrushShape::Line,
    };
    Paint {
        brush: Brush {
            shape,
            size: o.size,
            angle: o.angle,
        },
        symmetry: symmetry(o, doc),
        tiling: Tiling {
            x: o.tile_x,
            y: o.tile_y,
        },
        pixel_perfect: o.pixel_perfect,
        ink: ink(o.ink),
    }
}

/// A simetria das opções, com os eixos que o artista arrastou (ou no meio).
pub(crate) fn symmetry(o: &Options, doc: &Doc) -> Symmetry {
    let mirrors = Mirrors {
        h: o.sym_h,
        v: o.sym_v,
        d1: o.sym_d1,
        d2: o.sym_d2,
    };
    let (w, h) = (doc.sprite.width(), doc.sprite.height());
    let mut s = Symmetry::centered(mirrors, w, h);
    if let Some((ax, ay)) = doc.axis2 {
        s.axis2_x = ax;
        s.axis2_y = ay;
    }
    s
}

/// As opções do balde.
pub(crate) fn bucket(o: &Options) -> Bucket {
    Bucket {
        tolerance: o.tolerance,
        contiguous: o.contiguous,
        eight: o.eight,
        sample: if o.sample_visible {
            BucketSample::Visible
        } else {
            BucketSample::Layer
        },
    }
}

/// A caixa `from → to` com os modificadores: `Shift` = quadrado, `Ctrl` = a partir do centro.
fn boxed(from: Cell, to: Cell, m: Mods) -> (Cell, Cell) {
    let to = if m.shift {
        square_box(from.0, from.1, to.0, to.1)
    } else {
        to
    };
    if m.ctrl {
        ((2 * from.0 - to.0, 2 * from.1 - to.1), to)
    } else {
        (from, to)
    }
}

fn collect(f: impl FnOnce(&mut dyn FnMut(i32, i32))) -> Vec<Cell> {
    let mut v = Vec::new();
    f(&mut |x, y| v.push((x, y)));
    v
}

/// **Redesenha a forma** do gesto no traço: contorno com o pincel, interior cheio.
pub(crate) fn reshape(
    stroke: &mut Stroke,
    sprite: &Sprite,
    tool: Tool,
    kind: &Kind,
    o: &Options,
    m: Mods,
) {
    let (outline, interior) = match kind {
        Kind::Box { from, to } => match tool {
            Tool::Line => {
                let to = if m.shift {
                    snap_pixel_angle(*from, *to)
                } else {
                    *to
                };
                (
                    collect(|cb| line_cells(from.0, from.1, to.0, to.1, cb)),
                    Vec::new(),
                )
            }
            Tool::Rectangle | Tool::FilledRectangle => {
                let (a, b) = boxed(*from, *to, m);
                let r = o.corner_radius;
                let filled = tool == Tool::FilledRectangle;
                let out = collect(|cb| rounded_rect_cells(a.0, a.1, b.0, b.1, r, false, cb));
                let inner = if filled {
                    collect(|cb| rounded_rect_cells(a.0, a.1, b.0, b.1, r, true, cb))
                } else {
                    Vec::new()
                };
                (out, inner)
            }
            _ => {
                let (a, b) = boxed(*from, *to, m);
                let out = collect(|cb| ellipse_cells(a.0, a.1, b.0, b.1, cb));
                let inner = if tool == Tool::FilledEllipse {
                    collect(|cb| ellipse_filled_cells(a.0, a.1, b.0, b.1, cb))
                } else {
                    Vec::new()
                };
                (out, inner)
            }
        },
        Kind::Curve { from, to, ctrl } => {
            let c = ctrl.unwrap_or(((from.0 + to.0) / 2, (from.1 + to.1) / 2));
            (curve_path(from.0, from.1, c.0, c.1, to.0, to.1), Vec::new())
        }
        Kind::Polygon(pts) | Kind::Contour(pts) => polygon_cells(pts),
        _ => return,
    };
    stroke.set_shape(sprite, &outline, &interior);
}
