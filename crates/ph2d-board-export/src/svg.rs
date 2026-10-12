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
            g.stops
                .first()
                .map_or(Color::BLACK, |s| s.color.to_alpha_color())
        }
        Brush::Image(_) => {
            debug_assert!(false, "o quadro não pinta imagens");
            Color::BLACK
        }
    }
}

/// O texto num atributo: os caracteres de controlo (que o XML não aceita — um `.excalidraw` importado
/// pode trazê-los, auditoria W8) viram espaço, e os quatro especiais escapam-se.
pub(crate) fn escape(s: &str) -> String {
    let clean: String = s
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    clean
        .replace('&', "&amp;")
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
    write_linked(ops, size, &[])
}

/// [`write`] com as ligações (E5): cada `(caixa, alvo)` em px da página vira um `<a>` sobre a caixa
/// para uma `<view>` com a caixa do alvo — a ligação interna do SVG 1.1.
#[must_use]
pub fn write_linked(ops: &[DrawOp], size: [f64; 2], links: &[([f64; 4], [f64; 4])]) -> String {
    write_with_uris(ops, size, links, &[])
}

/// W18 — [`write_linked`] com as ligações para FORA (`uris`: a caixa em px da página e o endereço — um
/// vídeo do quadro leva o dele): um `<a href>` transparente sobre a caixa.
#[must_use]
pub fn write_with_uris(
    ops: &[DrawOp],
    size: [f64; 2],
    links: &[([f64; 4], [f64; 4])],
    uris: &[([f64; 4], String)],
) -> String {
    let mut glyphs = Glyphs::default();
    let [w, h] = size;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{0}\" height=\"{1}\" viewBox=\"0 0 {0} {1}\">\n",
        num(w),
        num(h)
    );
    let mut next = 0usize;
    let mut images = Images::default();
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
                let rule = if *rule == Fill::EvenOdd {
                    " clip-rule=\"evenodd\""
                } else {
                    ""
                };
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
            DrawOp::Image(op) => images.place(&mut out, op),
            DrawOp::Embed(op) => images.embed(&mut out, op.transform, &op.source),
            DrawOp::Glyphs(run) => {
                let before = glyphs.len();
                let placed = glyphs.place(run);
                // Os contornos novos definem-se uma vez; cada letra é um `<use>`.
                if glyphs.len() > before {
                    out.push_str("<defs>");
                    for i in before..glyphs.len() {
                        let _ = write!(out, "<path id=\"g{i}\" d=\"{}\"/>", d(glyphs.path(i)));
                    }
                    out.push_str("</defs>\n");
                }
                let (hex, a) = paint(run.brush);
                let a = a.map_or(String::new(), |a| format!(" fill-opacity=\"{a}\""));
                let _ = write!(
                    out,
                    "<g fill=\"{hex}\"{a}{} aria-label=\"{}\">",
                    transform(run.transform),
                    escape(&run.text)
                );
                for (i, [x, y]) in placed {
                    let _ = write!(
                        out,
                        "<use xlink:href=\"#g{i}\" x=\"{}\" y=\"{}\"/>",
                        num(x),
                        num(y)
                    );
                }
                out.push_str("</g>\n");
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
    for (i, (from, to)) in links.iter().enumerate() {
        let _ = writeln!(
            out,
            "<view id=\"link{i}\" viewBox=\"{} {} {} {}\"/><a xlink:href=\"#link{i}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#000000\" fill-opacity=\"0\"/></a>",
            num(to[0]),
            num(to[1]),
            num(to[2] - to[0]),
            num(to[3] - to[1]),
            num(from[0]),
            num(from[1]),
            num(from[2] - from[0]),
            num(from[3] - from[1])
        );
    }
    for (bx, url) in uris {
        let _ = writeln!(
            out,
            "<a xlink:href=\"{0}\" href=\"{0}\"><rect x=\"{1}\" y=\"{2}\" width=\"{3}\" height=\"{4}\" fill=\"#000000\" fill-opacity=\"0\"/></a>",
            escape(url),
            num(bx[0]),
            num(bx[1]),
            num(bx[2] - bx[0]),
            num(bx[3] - bx[1])
        );
    }
    out.push_str("</svg>\n");
    out
}

/// W14 — as imagens: cada ficheiro definido UMA vez (`<image>` em `<defs>`, os bytes como chegaram
/// num `data:`; sem MIME, o nível em PNG), cada desenho um `<use>` com o afim do píxel natural.
///
/// W15 — a preto e branco: o `<use>` leva o filtro [`GRAY_FILTER`] (definido uma vez) e os bytes
/// continuam os originais (um SVG embebido fica vector); os píxeis de um nível já vêm cinzentos.
#[derive(Default)]
struct Images {
    defined: std::collections::BTreeMap<(usize, usize), usize>,
    gray: bool,
}

/// O preto e branco da casa em SVG: `saturate 0` é a matriz da luminância Rec. 709, e em `linearRGB`
/// (dito, não confiado à omissão) corre sobre a luz linear — a lei de `ph2d_board_render::to_gray`.
const GRAY_FILTER: &str = "<defs><filter id=\"gray\" color-interpolation-filters=\"linearRGB\"><feColorMatrix type=\"saturate\" values=\"0\"/></filter></defs>";

impl Images {
    fn place(&mut self, out: &mut String, op: &ph2d_vector::ImageOp) {
        let level = [op.image.width(), op.image.height()];
        match &op.source {
            Some(s) => {
                let k = Affine::scale_non_uniform(
                    f64::from(level[0]) / f64::from(s.size[0].max(1)),
                    f64::from(level[1]) / f64::from(s.size[1].max(1)),
                );
                self.embed(out, op.transform * k, s);
            }
            None => {
                let px = op.image.rgba();
                let key = (px.as_ptr() as usize, px.len());
                self.put(out, key, level, op.transform, false, || {
                    format!("data:image/png;base64,{}", b64(&png_of(&op.image)))
                });
            }
        }
    }

    /// Os bytes de `s` com `t` do píxel natural ao ecrã (W16: o export SVG não descodifica nada).
    fn embed(&mut self, out: &mut String, t: Affine, s: &ph2d_vector::ImageSource) {
        let key = (s.bytes.as_ptr() as usize, s.bytes.len());
        self.put(out, key, s.size, t, s.gray, || {
            format!("data:{};base64,{}", s.mime, b64(&s.bytes))
        });
    }

    fn put(
        &mut self,
        out: &mut String,
        key: (usize, usize),
        size: [u32; 2],
        t: Affine,
        gray: bool,
        href: impl FnOnce() -> String,
    ) {
        let n = self.defined.len();
        let id = *self.defined.entry(key).or_insert_with(|| {
            let href = href();
            let _ = writeln!(
                out,
                "<defs><image id=\"i{n}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" xlink:href=\"{href}\"/></defs>",
                size[0], size[1]
            );
            n
        });
        if gray && !self.gray {
            self.gray = true;
            let _ = writeln!(out, "{GRAY_FILTER}");
        }
        let filter = if gray { " filter=\"url(#gray)\"" } else { "" };
        let _ = writeln!(out, "<use xlink:href=\"#i{id}\"{}{filter}/>", transform(t));
    }
}

fn b64(b: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(b)
}

/// Os píxeis de um nível em PNG (a imagem cujos bytes não são de um MIME que o SVG mostra).
fn png_of(img: &ph2d_vector::StableImage) -> Vec<u8> {
    let mut out = Vec::new();
    let mut e = png::Encoder::new(&mut out, img.width(), img.height());
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    if let Ok(mut w) = e.write_header() {
        let _ = w.write_image_data(img.rgba());
    }
    out
}

fn fill(out: &mut String, path: &str, rule: Fill, t: Affine, c: Color, extra: Option<&str>) {
    if path.is_empty() {
        return;
    }
    let (hex, a) = paint(c);
    let a = a.map_or(String::new(), |a| format!(" fill-opacity=\"{a}\""));
    let rule = if rule == Fill::EvenOdd {
        " fill-rule=\"evenodd\""
    } else {
        ""
    };
    let _ = writeln!(
        out,
        "<path d=\"{path}\" fill=\"{hex}\"{a}{rule}{}{}/>",
        transform(t),
        extra.unwrap_or("")
    );
}
