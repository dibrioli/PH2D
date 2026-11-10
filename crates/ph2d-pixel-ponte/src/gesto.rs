//! **O gesto de cada ferramenta** (Aseprite): o que o premir começa, o que mover faz, quando o
//! gesto acaba. Toda forma é redesenhada INTEIRA a cada movimento sobre o mesmo [`Stroke`], que a
//! vista mostra por cima do documento e que vira UM passo do desfazer ao acabar. Saiu de
//! `ph2d-app-pixel` (P12, `docs/PixelArt/P12_plano.md`): nada aqui vê o estado da família.

use ph2d_editor_core::pixel_panel::PixelShape;
use ph2d_pixel_core::brush::{Brush, BrushShape};
use ph2d_pixel_core::fill::{Bucket, BucketSample};
use ph2d_pixel_core::ink::Ink;
use ph2d_pixel_core::raster::{Cell, curve_path, line_cells, polygon_cells, square_box};
use ph2d_pixel_core::symmetry::{Mirrors, Symmetry, Tiling};
use ph2d_pixel_core::{Paint, Sprite, Stroke};

use ph2d_editor_core::pixel_panel::{PixelOptions as Options, PixelTool as Tool};

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
pub enum Kind {
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
    /// P8: o Gradient arrastado desde `from` (`ph2d_app_pixel::degrade`).
    Gradient {
        from: Cell,
    },
    /// P8: o Blur/Jumble (o estado vive em o `PixelApp::mancha` da família).
    Smudge,
    /// Arrastar um eixo da simetria (vertical = o do espelho H).
    Axis {
        vertical: bool,
    },
}

/// A tinta do chrome no vocabulário do núcleo.
pub fn ink(i: ph2d_editor_core::pixel_panel::PixelInk) -> Ink {
    use ph2d_editor_core::pixel_panel::PixelInk as I;
    match i {
        I::Simple => Ink::Simple,
        I::AlphaCompositing => Ink::AlphaCompositing,
        I::CopyAlphaColor => Ink::CopyAlphaColor,
        I::LockAlpha => Ink::LockAlpha,
        I::Shading => Ink::Shading,
        // O padrão entra no traço (`Stroke::with_dither`); a tinta é a Simple.
        I::Dither => Ink::Simple,
    }
}

/// P13: o padrão do ink Dither para o traço da ferramenta `tool` (`None` = sem padrão; a borracha
/// e as que escrevem a cor pronta nunca o levam).
#[must_use]
pub fn pincel_dither(
    o: &ph2d_editor_core::pixel_panel::PixelOptions,
    tool: Tool,
) -> Option<ph2d_pixel_core::dither::PincelDither> {
    use ph2d_editor_core::pixel_panel::{PixelGradientDither as G, PixelInk};
    use ph2d_pixel_core::dither::{Dither, PincelDither};
    let pinta = tool.paints()
        && !matches!(
            tool,
            Tool::Eraser | Tool::Gradient | Tool::Blur | Tool::Jumble | Tool::Push
        );
    let matriz = match o.dither_matrix {
        G::Bayer2 => Dither::Bayer2,
        G::None | G::Bayer4 => Dither::Bayer4,
        G::Bayer8 => Dither::Bayer8,
    };
    (pinta && o.ink == PixelInk::Dither).then_some(PincelDither {
        nivel: o.dither_level.min(PincelDither::MAX),
        matriz,
    })
}

/// A tinta do núcleo no vocabulário do chrome (um espaço guardado que volta).
pub fn ink_out(i: Ink) -> ph2d_editor_core::pixel_panel::PixelInk {
    use ph2d_editor_core::pixel_panel::PixelInk as I;
    match i {
        Ink::Simple => I::Simple,
        Ink::AlphaCompositing => I::AlphaCompositing,
        Ink::CopyAlphaColor => I::CopyAlphaColor,
        Ink::LockAlpha => I::LockAlpha,
        Ink::Shading => I::Shading,
    }
}

/// O pincel, a simetria, o ladrilho e a tinta das opções, para o documento `doc`; `pincel` = o
/// copiado da selecção (Ctrl+B), que toma o lugar da forma.
pub fn paint(
    o: &Options,
    s: &Sprite,
    axis2: Option<(i32, i32)>,
    pincel: Option<&(std::sync::Arc<ph2d_pixel_core::pattern::PatternBrush>, bool)>,
) -> Paint {
    use ph2d_editor_core::pixel_panel::PixelBrushShape as S;
    use ph2d_editor_core::pixel_panel::PixelPatternAlign as A;
    use ph2d_pixel_core::pattern::PatternAlign;
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
            pattern: pincel.map(|(p, _)| p.clone()),
            pattern_align: match o.pattern_align {
                A::Source => PatternAlign::Source,
                A::Destination => PatternAlign::Destination,
                A::Paint => PatternAlign::Paint,
            },
            image_color: pincel.is_none_or(|&(_, imagem)| imagem),
        },
        symmetry: symmetry(o, s, axis2),
        tiling: Tiling {
            x: o.tile_x,
            y: o.tile_y,
        },
        pixel_perfect: o.pixel_perfect,
        ink: ink(o.ink),
    }
}

/// A simetria das opções, com os eixos que o artista arrastou (ou no meio).
pub fn symmetry(o: &Options, sprite: &Sprite, axis2: Option<(i32, i32)>) -> Symmetry {
    let mirrors = Mirrors {
        h: o.sym_h,
        v: o.sym_v,
        d1: o.sym_d1,
        d2: o.sym_d2,
    };
    let (w, h) = (sprite.width(), sprite.height());
    let mut s = Symmetry::centered(mirrors, w, h);
    if let Some((ax, ay)) = axis2 {
        s.axis2_x = ax;
        s.axis2_y = ay;
    }
    s
}

/// As opções do balde.
pub fn bucket(o: &Options) -> Bucket {
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

/// A caixa `from → to` com os modificadores (`Shift` = quadrado, `Ctrl` = a partir do centro),
/// ↖ ↘ — a forma viva nasce dela.
pub fn boxed(from: Cell, to: Cell, m: Mods) -> (Cell, Cell) {
    let to = if m.shift {
        square_box(from.0, from.1, to.0, to.1)
    } else {
        to
    };
    let from = if m.ctrl {
        (2 * from.0 - to.0, 2 * from.1 - to.1)
    } else {
        from
    };
    (
        (from.0.min(to.0), from.1.min(to.1)),
        (from.0.max(to.0), from.1.max(to.1)),
    )
}

fn collect(f: impl FnOnce(&mut dyn FnMut(i32, i32))) -> Vec<Cell> {
    let mut v = Vec::new();
    f(&mut |x, y| v.push((x, y)));
    v
}

/// **Redesenha a forma** do gesto no traço: contorno com o pincel, interior cheio.
pub fn reshape(
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
                    ph2d_pixel_guia::direcao_guiada(sprite.guias(), *from, *to)
                } else {
                    *to
                };
                (
                    collect(|cb| line_cells(from.0, from.1, to.0, to.1, cb)),
                    Vec::new(),
                )
            }
            // As formas do Vector (P6b): a do catálogo com os valores do painel.
            _ => {
                let Some(forma) = PixelShape::of_tool(tool) else {
                    return;
                };
                let (a, b) = boxed(*from, *to, m);
                ph2d_pixel_formas::cells(forma, &o.shapes.get(forma), a, b, o.shape_fill)
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

/// O nome do passo de um traço na Undo History: o da FERRAMENTA, não do gesto (Aseprite O11).
#[must_use]
pub fn rotulo(tool: Tool) -> ph2d_pixel_core::Label {
    use ph2d_pixel_core::Label;
    Label::new(match tool {
        Tool::Pencil => "pixel.history.pencil",
        Tool::Eraser => "pixel.history.eraser",
        Tool::Spray => "pixel.history.spray",
        Tool::Line => "pixel.history.line",
        Tool::Curve => "pixel.history.curve",
        Tool::RECTANGLE => "pixel.history.rectangle",
        Tool::ELLIPSE => "pixel.history.ellipse",
        Tool::Shape(_) => "pixel.history.shape",
        Tool::Bucket => "pixel.history.paint_bucket",
        Tool::Gradient => "pixel.history.gradient",
        Tool::Blur => "pixel.history.blur",
        Tool::Jumble => "pixel.history.jumble",
        Tool::Push => "pixel.history.push",
        Tool::Contour => "pixel.history.contour",
        Tool::Polygon => "pixel.history.polygon",
        Tool::Text => "pixel.history.text",
        Tool::Transform => "pixel.history.move",
        Tool::RectMarquee | Tool::EllipseMarquee => "pixel.history.marquee",
        Tool::Lasso | Tool::PolygonLasso => "pixel.history.lasso",
        Tool::MagicWand => "pixel.history.magic_wand",
        Tool::Slice => "pixel.history.slice",
        Tool::Eyedropper | Tool::Hand => "pixel.history.draw",
    })
}
