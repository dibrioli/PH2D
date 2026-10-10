//! ⭐ **O SVG** (E5): a gravação de uma página escrita em SVG 1.1 — caminhos, contornos, camadas com
//! recorte (`clip-path`, nunca o `ns1:clip-path` do serializador do Excalidraw: o librsvg e o Inkscape
//! não o recortam, estudo §3), as letras como contornos (com o texto em `aria-label`) e a sombra das
//! notas pelo `feGaussianBlur` (o desfoque gaussiano que o Vello aproxima).

use std::fmt::Write as _;

use ph2d_vector::{Affine, BezPath, Brush, Cap, Color, DrawOp, Fill, Join, PathEl, Stroke};

use crate::glyphs::Glyphs;

/// Um número curto e exacto o bastante (milésimos de px), sem zeros à direita.
pub(crate) fn num(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    let r = if r == 0.0 { 0.0 } else { r };
    let mut s = format!("{r:.3}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

fn d(path: &BezPath) -> String {
    let mut s = String::new();
    for e in path.elements() {
        let _ = match *e {
            PathEl::MoveTo(p) => write!(s, "M{} {}", num(p.x), num(p.y)),
            PathEl::LineTo(p) => write!(s, "L{} {}", num(p.x), num(p.y)),
            PathEl::QuadTo(a, p) => {
                write!(s, "Q{} {} {} {}", num(a.x), num(a.y), num(p.x), num(p.y))
            }
            PathEl::CurveTo(a, b, p) => write!(
                s,
                "C{} {} {} {} {} {}",
                num(a.x),
                num(a.y),
                num(b.x),
                num(b.y),
                num(p.x),
                num(p.y)
            ),
            PathEl::ClosePath => write!(s, "Z"),
        };
    }
    s
}

fn transform(t: Affine) -> String {
    if t == Affine::IDENTITY {
        return String::new();
    }
    let [a, b, c, dd, e, f] = t.as_coeffs();
    format!(
        " transform=\"matrix({} {} {} {} {} {})\"",
        num(a),
        num(b),
        num(c),
        num(dd),
        num(e),
        num(f)
    )
}

/// `#rrggbb` e a opacidade (`None` = opaca).
fn paint(c: Color) -> (String, Option<String>) {
    let [r, g, b, a] = c.to_rgba8().to_u8_array();
    let hex = format!("#{r:02x}{g:02x}{b:02x}");
    (hex, (a < 255).then(|| num(f64::from(a) / 255.0)))
}

/// A cor de um pincel. Só o sólido existe no quadro; outro tipo cai na 1.ª paragem (e grita em debug).
fn solid(b: &Brush) -> Color {
    match b {
        Brush::Solid(c) => *c,
        Brush::Gradient(g) => {
            debug_assert!(false, "o quadro não pinta gradientes");
            g.stops.first().map_or(Color::BLACK, |s| s.color.to_alpha_color())
        }
        Brush::Image(_) => {
            debug_assert!(false, "o quadro não pinta imagens");
            Color::BLACK
        }
    }
}

pub(crate) fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn stroke_attrs(st: &Stroke) -> String {
    let cap = match st.start_cap {
        Cap::Butt => "butt",
        Cap::Round => "round",
        Cap::Square => "square",
    };
    let join = match st.join {
        Join::Bevel => "bevel",
        Join::Miter => "miter",
        Join::Round => "round",
    };
    let mut s = format!(
        " stroke-width=\"{}\" stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\" stroke-miterlimit=\"{}\"",
        num(st.width),
        num(st.miter_limit)
    );
    if !st.dash_pattern.is_empty() {
        let dash: Vec<String> = st.dash_pattern.iter().map(|&v| num(v)).collect();
        let _ = write!(
            s,
            " stroke-dasharray=\"{}\" stroke-dashoffset=\"{}\"",
            dash.join(" "),
            num(st.dash_offset)
        );
    }
    s
}

/// ⭐ Escreve a página (`ops` gravadas por [`crate::paint_page`], `size` em px) em SVG.
#[must_use]
pub fn write(ops: &[DrawOp], size: [f64; 2]) -> String {
    let mut glyphs = Glyphs::default();
    let [w, h] = size;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{0}\" height=\"{1}\" viewBox=\"0 0 {0} {1}\">\n",
        num(w),
        num(h)
    );
    let mut next = 0usize;
    for op in ops {
        match op {
            DrawOp::Fill {
                rule,
                transform: t,
                brush,
                path,
            } => fill(&mut out, &d(path), *rule, *t, solid(brush), None),
            DrawOp::Stroke {
                style,
                transform: t,
                brush,
                path,
            } => {
                let (hex, a) = paint(solid(brush));
                let a = a.map_or(String::new(), |a| format!(" stroke-opacity=\"{a}\""));
                let _ = writeln!(
                    out,
                    "<path d=\"{}\" fill=\"none\" stroke=\"{hex}\"{a}{}{}/>",
                    d(path),
                    stroke_attrs(style),
                    transform(*t)
                );
            }
            DrawOp::Layer {
                rule,
                alpha,
                transform: t,
                clip,
                ..
            } => {
                next += 1;
                let rule = if *rule == Fill::EvenOdd { " clip-rule=\"evenodd\"" } else { "" };
                let _ = writeln!(
                    out,
                    "<clipPath id=\"c{next}\"><path d=\"{}\"{rule}{}/></clipPath>",
                    d(clip),
                    transform(*t)
                );
                let op = if *alpha < 1.0 {
                    format!(" opacity=\"{}\"", num(f64::from(*alpha)))
                } else {
                    String::new()
                };
                let _ = writeln!(out, "<g clip-path=\"url(#c{next})\"{op}>");
            }
            DrawOp::Pop => out.push_str("</g>\n"),
            DrawOp::Glyphs(run) => {
                let label = format!(" aria-label=\"{}\"", escape(&run.text));
                let path = glyphs.run_path(run);
                fill(&mut out, &d(&path), Fill::NonZero, run.transform, run.brush, Some(&label));
            }
            DrawOp::BlurredRect {
                transform: t,
                rect,
                color,
                radius,
                std_dev,
            } => {
                next += 1;
                let k = 3.0 * std_dev;
                let (hex, a) = paint(*color);
                let a = a.map_or(String::new(), |a| format!(" fill-opacity=\"{a}\""));
                let _ = writeln!(
                    out,
                    "<filter id=\"b{next}\" filterUnits=\"userSpaceOnUse\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"><feGaussianBlur stdDeviation=\"{}\"/></filter>",
                    num(rect.x0 - k),
                    num(rect.y0 - k),
                    num(rect.width() + 2.0 * k),
                    num(rect.height() + 2.0 * k),
                    num(*std_dev)
                );
                let _ = writeln!(
                    out,
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{hex}\"{a} filter=\"url(#b{next})\"{}/>",
                    num(rect.x0),
                    num(rect.y0),
                    num(rect.width()),
                    num(rect.height()),
                    num(*radius),
                    transform(*t)
                );
            }
        }
    }
    out.push_str("</svg>\n");
    out
}

fn fill(out: &mut String, path: &str, rule: Fill, t: Affine, c: Color, extra: Option<&str>) {
    if path.is_empty() {
        return;
    }
    let (hex, a) = paint(c);
    let a = a.map_or(String::new(), |a| format!(" fill-opacity=\"{a}\""));
    let rule = if rule == Fill::EvenOdd { " fill-rule=\"evenodd\"" } else { "" };
    let _ = writeln!(
        out,
        "<path d=\"{path}\" fill=\"{hex}\"{a}{rule}{}{}/>",
        transform(t),
        extra.unwrap_or("")
    );
}
