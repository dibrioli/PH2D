//! ⭐ **O PDF** (E4/E5): uma página por frame, a gravação de cada uma escrita como conteúdo PDF —
//! caminhos e contornos, camadas com recorte (a opacidade de um OBJECTO num grupo de transparência:
//! compõe-se uma vez, como no Vello), as letras como contornos com o TEXTO real por baixo (`ActualText`
//! e uma camada invisível: a busca e a cópia), a sombra das notas como uma máscara de desfoque
//! gaussiano, e os links do W5 como ligações entre páginas. 1 px da gravação = 1 pt.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;

use pdf_writer::types::{
    ActionType, AnnotationType, LineCapStyle, LineJoinStyle, TextRenderingMode,
};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref, Str, TextStr};
use ph2d_vector::{Affine, BezPath, Brush, Cap, Color, DrawOp, Fill, Join, PathEl, Stroke};

use crate::glyphs::Glyphs;

/// Uma ligação (E5): a caixa do elemento em px da página (de cima), a página do alvo e o canto de
/// cima à esquerda do alvo em px DESSA página.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    pub rect: [f64; 4],
    pub page: usize,
    pub at: [f64; 2],
}

/// Uma página: a gravação, o tamanho em px (= pt) e as ligações.
pub struct PdfPage<'a> {
    pub ops: &'a [DrawOp],
    pub size: [f64; 2],
    pub links: Vec<Link>,
}

#[allow(clippy::cast_possible_truncation)]
fn f(v: f64) -> f32 {
    v as f32
}

fn rgb(c: Color) -> ([f32; 3], f32) {
    let [r, g, b, a] = c.to_rgba8().to_u8_array();
    let k = |v: u8| f32::from(v) / 255.0;
    ([k(r), k(g), k(b)], k(a))
}

fn solid(b: &Brush) -> Color {
    match b {
        Brush::Solid(c) => *c,
        _ => {
            debug_assert!(false, "o quadro só pinta cores sólidas");
            Color::BLACK
        }
    }
}

fn path(c: &mut Content, p: &BezPath, t: Affine) {
    let mut last = ph2d_vector::Point::ZERO;
    for e in p.elements() {
        match *e {
            PathEl::MoveTo(a) => {
                let a = t * a;
                c.move_to(f(a.x), f(a.y));
                last = a;
            }
            PathEl::LineTo(a) => {
                let a = t * a;
                c.line_to(f(a.x), f(a.y));
                last = a;
            }
            PathEl::QuadTo(q, a) => {
                // A quadrática do PDF não existe: a cúbica exacta que a eleva.
                let (q, a) = (t * q, t * a);
                let c1 = last + (q - last) * (2.0 / 3.0);
                let c2 = a + (q - a) * (2.0 / 3.0);
                c.cubic_to(f(c1.x), f(c1.y), f(c2.x), f(c2.y), f(a.x), f(a.y));
                last = a;
            }
            PathEl::CurveTo(a, b, d) => {
                let (a, b, d) = (t * a, t * b, t * d);
                c.cubic_to(f(a.x), f(a.y), f(b.x), f(b.y), f(d.x), f(d.y));
                last = d;
            }
            PathEl::ClosePath => {
                c.close_path();
            }
        }
    }
}

fn cm(c: &mut Content, t: Affine) {
    if t != Affine::IDENTITY {
        c.transform(t.as_coeffs().map(f));
    }
}

/// `erf` (Abramowitz & Stegun 7.1.26, erro < 1,5e-7).
fn erf(x: f64) -> f64 {
    let s = x.signum();
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-x * x).exp();
    s * y
}

/// A máscara (cinzentos = alfa) do rectângulo `[w, h]` desfocado por `σ`, sobre a caixa alargada de
/// `3σ` de cada lado: o desfoque gaussiano de uma caixa é separável — o produto de dois `erf`.
fn blur_mask(w: f64, h: f64, sigma: f64) -> (u32, u32, Vec<u8>) {
    let k = 3.0 * sigma;
    let (bw, bh) = (w + 2.0 * k, h + 2.0 * k);
    // Dois píxeis por σ chegam (é um desfoque), e o lado maior não passa de 256.
    let per = (2.0 / sigma.max(1e-9)).min(256.0 / bw.max(bh));
    let (n, m) = (
        ((bw * per).ceil() as u32).max(1),
        ((bh * per).ceil() as u32).max(1),
    );
    let s2 = sigma.max(1e-9) * std::f64::consts::SQRT_2;
    let axis = |v: f64, len: f64| 0.5 * (erf((v - k) / s2) - erf((v - k - len) / s2));
    let mut out = Vec::with_capacity((n * m) as usize);
    for j in 0..m {
        let ay = axis((f64::from(j) + 0.5) / per, h);
        for i in 0..n {
            let ax = axis((f64::from(i) + 0.5) / per, w);
            out.push((ax * ay * 255.0).round().clamp(0.0, 255.0) as u8); // CLAMP-OK: alfa em [0, 255]
        }
    }
    (n, m, out)
}

fn deflate(bytes: &[u8]) -> Vec<u8> {
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(6));
    let _ = z.write_all(bytes);
    z.finish().unwrap_or_default()
}

/// O texto em WinAnsi (o da Helvetica dos 14 de base) — o que não cabe vira `?`; o `ActualText` leva
/// o texto inteiro.
fn win_ansi(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| match c as u32 {
            0x20..=0x7e | 0xa0..=0xff => c as u32 as u8,
            _ => b'?',
        })
        .collect()
}

/// A chave de uma máscara de sombra (o rect em bits e a cor) e o nome e objecto dela.
type MaskKey = (u64, u64, u64, [u8; 4]);

/// Um grupo de transparência por escrever: a ref, o conteúdo, o tamanho da página, os nomes que usa.
type PendingForm = (Ref, Vec<u8>, [f64; 2], BTreeSet<String>);

/// O que um documento partilha entre páginas: as refs, os estados de alfa, as máscaras, os grupos.
struct Doc {
    pdf: Pdf,
    next: i32,
    alphas: BTreeMap<u8, (String, Ref)>,
    masks: BTreeMap<MaskKey, (String, Ref)>,
    /// Os grupos de transparência por escrever.
    forms: Vec<PendingForm>,
    /// O objecto de cada nome de recurso (estados de alfa, máscaras, grupos, glifos): ⛔ cada conteúdo
    /// declara SÓ os nomes que usa — com o dicionário inteiro em cada grupo, N objectos translúcidos
    /// davam N grupos × N entradas (auditoria W8: 2 000 rectângulos a 50 % = 85 MB).
    refs: BTreeMap<String, Ref>,
    groups: usize,
    glyphs: Glyphs,
    /// O XObject de cada contorno de glifo já escrito (índice da cache → nome).
    glyph_forms: BTreeMap<usize, String>,
}

impl Doc {
    fn id(&mut self) -> Ref {
        self.next += 1;
        Ref::new(self.next)
    }

    /// O nome do estado com fill/stroke alfa `a` (`None` = opaco).
    fn alpha(&mut self, a: f32) -> Option<String> {
        if a >= 1.0 {
            return None;
        }
        let key = (a * 255.0).round() as u8;
        if !self.alphas.contains_key(&key) {
            let r = self.id();
            self.alphas.insert(key, (format!("A{key}"), r));
            self.refs.insert(format!("A{key}"), r);
        }
        Some(self.alphas[&key].0.clone())
    }

    /// O XObject do contorno `i` (escrito uma vez): só o caminho e o `f` — a cor é a do estado.
    fn glyph_form(&mut self, i: usize) -> String {
        if let Some(n) = self.glyph_forms.get(&i) {
            return n.clone();
        }
        let r = self.id();
        let p = self.glyphs.path(i).clone();
        let mut c = Content::new();
        path(&mut c, &p, Affine::IDENTITY);
        c.fill_nonzero();
        let body = c.finish();
        let bb = if p.elements().is_empty() {
            [0.0; 4]
        } else {
            let b = ph2d_vector::Shape::bounding_box(&p);
            [b.x0, b.y0, b.x1, b.y1]
        };
        let mut form = self.pdf.form_xobject(r, &body);
        form.bbox(Rect::new(f(bb[0]), f(bb[1]), f(bb[2]), f(bb[3])));
        form.finish();
        let name = format!("H{i}");
        self.glyph_forms.insert(i, name.clone());
        self.refs.insert(name.clone(), r);
        name
    }

    fn mask(&mut self, w: f64, h: f64, sigma: f64, color: Color) -> String {
        let key = (
            w.to_bits(),
            h.to_bits(),
            sigma.to_bits(),
            color.to_rgba8().to_u8_array(),
        );
        if let Some((n, _)) = self.masks.get(&key) {
            return n.clone();
        }
        let (img, smask) = (self.id(), self.id());
        let (nw, nh, alpha) = blur_mask(w, h, sigma);
        let ([r, g, b], a) = rgb(color);
        let scaled: Vec<u8> = alpha
            .iter()
            .map(|&v| (f32::from(v) * a).round() as u8)
            .collect();
        let rgb8 = [r, g, b].map(|v| (v * 255.0).round() as u8);
        let mut im = self.pdf.image_xobject(img, &rgb8);
        im.width(1).height(1).bits_per_component(8);
        im.color_space().device_rgb();
        im.s_mask(smask);
        im.finish();
        let packed = deflate(&scaled);
        let mut sm = self.pdf.image_xobject(smask, &packed);
        sm.filter(Filter::FlateDecode);
        sm.width(nw as i32).height(nh as i32).bits_per_component(8);
        sm.color_space().device_gray();
        sm.finish();
        let name = format!("M{}", self.masks.len());
        self.masks.insert(key, (name.clone(), img));
        self.refs.insert(name.clone(), img);
        name
    }
}

/// Uma camada aberta: o conteúdo dela, se é um grupo (alfa < 1) o alfa, e os nomes que ela usa.
struct Open {
    content: Content,
    group: Option<f32>,
    used: BTreeSet<String>,
}

impl Open {
    fn new(group: Option<f32>) -> Self {
        Self {
            content: Content::new(),
            group,
            used: BTreeSet::new(),
        }
    }
}

fn stroke_style(c: &mut Content, st: &Stroke) {
    c.set_line_width(f(st.width));
    c.set_line_cap(match st.start_cap {
        Cap::Butt => LineCapStyle::ButtCap,
        Cap::Round => LineCapStyle::RoundCap,
        Cap::Square => LineCapStyle::ProjectingSquareCap,
    });
    c.set_line_join(match st.join {
        Join::Bevel => LineJoinStyle::BevelJoin,
        Join::Miter => LineJoinStyle::MiterJoin,
        Join::Round => LineJoinStyle::RoundJoin,
    });
    c.set_miter_limit(f(st.miter_limit));
    if !st.dash_pattern.is_empty() {
        c.set_dash_pattern(st.dash_pattern.iter().map(|&v| f(v)), f(st.dash_offset));
    }
}

fn page_content(doc: &mut Doc, ops: &[DrawOp], size: [f64; 2]) -> (Vec<u8>, BTreeSet<String>) {
    let mut stack = vec![Open::new(None)];
    // Cada camada aberta: `Some(alfa)` = um grupo (um conteúdo novo na pilha); `None` = só o recorte.
    let mut levels: Vec<Option<f32>> = Vec::new();
    // O PDF sobe; a gravação desce: vira-se uma vez, e 1 px = 1 pt.
    stack[0]
        .content
        .transform([1.0, 0.0, 0.0, -1.0, 0.0, f(size[1])]);
    for op in ops {
        match op {
            DrawOp::Fill {
                rule,
                transform,
                brush,
                path: p,
            } => {
                let a = doc.alpha(rgb(solid(brush)).1);
                let top = stack.last_mut().expect("a página");
                top.used.extend(a.clone());
                fill(
                    &mut top.content,
                    p,
                    *rule,
                    *transform,
                    solid(brush),
                    a.as_deref(),
                );
            }
            DrawOp::Stroke {
                style,
                transform,
                brush,
                path: p,
            } => {
                let ([r, g, b], a) = rgb(solid(brush));
                let a = doc.alpha(a);
                let top = stack.last_mut().expect("a página");
                top.used.extend(a.clone());
                let c = &mut top.content;
                c.save_state();
                cm(c, *transform);
                c.set_stroke_rgb(r, g, b);
                if let Some(a) = &a {
                    c.set_parameters(Name(a.as_bytes()));
                }
                stroke_style(c, style);
                path(c, p, Affine::IDENTITY);
                c.stroke();
                c.restore_state();
            }
            DrawOp::Layer {
                rule,
                alpha,
                transform,
                clip,
                ..
            } => {
                let c = &mut stack.last_mut().expect("a página").content;
                c.save_state();
                path(c, clip, *transform);
                if *rule == Fill::EvenOdd {
                    c.clip_even_odd();
                } else {
                    c.clip_nonzero();
                }
                c.end_path();
                if *alpha < 1.0 {
                    stack.push(Open::new(Some(*alpha)));
                    levels.push(Some(*alpha));
                } else {
                    levels.push(None);
                }
            }
            DrawOp::Pop => {
                let Some(level) = levels.pop() else {
                    debug_assert!(false, "Pop sem camada");
                    continue;
                };
                if level.is_some() {
                    let open = stack.pop().expect("o grupo");
                    let body = open.content.finish().to_vec();
                    let r = doc.id();
                    doc.forms.push((r, body, size, open.used));
                    let name = format!("G{}", doc.groups);
                    doc.groups += 1;
                    doc.refs.insert(name.clone(), r);
                    let a = open.group.and_then(|g| doc.alpha(g));
                    let top = stack.last_mut().expect("a página");
                    top.used.extend(a.clone());
                    top.used.insert(name.clone());
                    let c = &mut top.content;
                    if let Some(a) = &a {
                        c.set_parameters(Name(a.as_bytes()));
                    }
                    c.x_object(Name(name.as_bytes()));
                }
                stack.last_mut().expect("a página").content.restore_state();
            }
            DrawOp::Glyphs(run) => {
                let placed = doc.glyphs.place(run);
                let names: Vec<String> = placed.iter().map(|(i, _)| doc.glyph_form(*i)).collect();
                let ([r, g, b], alpha) = rgb(run.brush);
                let a = doc.alpha(alpha);
                let top = stack.last_mut().expect("a página");
                top.used.extend(a.clone());
                top.used.extend(names.iter().cloned());
                top.used.insert("F0".to_owned());
                let c = &mut top.content;
                c.begin_marked_content_with_properties(Name(b"Span"))
                    .properties()
                    .actual_text(TextStr(&run.text));
                // Cada letra é o XObject do seu contorno (tirado uma vez), pintado com a cor do estado.
                c.save_state();
                cm(c, run.transform);
                c.set_fill_rgb(r, g, b);
                if let Some(a) = &a {
                    c.set_parameters(Name(a.as_bytes()));
                }
                for (name, (_, [x, y])) in names.iter().zip(&placed) {
                    c.save_state();
                    c.transform([1.0, 0.0, 0.0, 1.0, f(*x), f(*y)]);
                    c.x_object(Name(name.as_bytes()));
                    c.restore_state();
                }
                c.restore_state();
                if let Some(g) = run.glyphs.first() {
                    let tm = run.transform
                        * Affine::translate((f64::from(g.x), f64::from(g.y)))
                        * Affine::FLIP_Y;
                    c.begin_text();
                    c.set_text_rendering_mode(TextRenderingMode::Invisible);
                    c.set_font(Name(b"F0"), run.size);
                    c.set_text_matrix(tm.as_coeffs().map(f));
                    c.show(Str(&win_ansi(&run.text)));
                    c.end_text();
                }
                c.end_marked_content();
            }
            DrawOp::BlurredRect {
                transform,
                rect,
                color,
                std_dev,
                ..
            } => {
                let name = doc.mask(rect.width(), rect.height(), *std_dev, *color);
                let k = 3.0 * std_dev;
                let (x0, y0) = (rect.x0 - k, rect.y0 - k);
                let (w, h) = (rect.width() + 2.0 * k, rect.height() + 2.0 * k);
                let top = stack.last_mut().expect("a página");
                top.used.insert(name.clone());
                let c = &mut top.content;
                c.save_state();
                cm(c, *transform);
                // O quadrado unitário da imagem sobe; aqui desce: a linha 0 da máscara fica em cima.
                c.transform([f(w), 0.0, 0.0, f(-h), f(x0), f(y0 + h)]);
                c.x_object(Name(name.as_bytes()));
                c.restore_state();
            }
        }
    }
    debug_assert!(levels.is_empty(), "camada por fechar");
    let page = stack.swap_remove(0);
    (page.content.finish().to_vec(), page.used)
}

fn fill(c: &mut Content, p: &BezPath, rule: Fill, t: Affine, color: Color, alpha: Option<&str>) {
    if p.elements().is_empty() {
        return;
    }
    let ([r, g, b], _) = rgb(color);
    c.save_state();
    cm(c, t);
    c.set_fill_rgb(r, g, b);
    if let Some(a) = alpha {
        c.set_parameters(Name(a.as_bytes()));
    }
    path(c, p, Affine::IDENTITY);
    if rule == Fill::EvenOdd {
        c.fill_even_odd();
    } else {
        c.fill_nonzero();
    }
    c.restore_state();
}

/// ⭐ Escreve as páginas num PDF.
#[must_use]
pub fn write(pages: &[PdfPage<'_>]) -> Vec<u8> {
    let mut doc = Doc {
        pdf: Pdf::new(),
        next: 0,
        alphas: BTreeMap::new(),
        masks: BTreeMap::new(),
        forms: Vec::new(),
        refs: BTreeMap::new(),
        groups: 0,
        glyphs: Glyphs::default(),
        glyph_forms: BTreeMap::new(),
    };
    let (catalog, tree, font) = (doc.id(), doc.id(), doc.id());
    let page_ids: Vec<Ref> = pages.iter().map(|_| doc.id()).collect();
    let mut contents = Vec::new();
    for p in pages {
        contents.push(page_content(&mut doc, p.ops, p.size));
    }
    doc.pdf.catalog(catalog).pages(tree);
    doc.pdf
        .pages(tree)
        .kids(page_ids.iter().copied())
        .count(i32::try_from(pages.len()).unwrap_or(i32::MAX));
    doc.pdf
        .type1_font(font)
        .base_font(Name(b"Helvetica"))
        .encoding_predefined(Name(b"WinAnsiEncoding"));
    for (key, (_, r)) in &doc.alphas {
        let a = f32::from(*key) / 255.0;
        doc.pdf
            .ext_graphics(*r)
            .non_stroking_alpha(a)
            .stroking_alpha(a);
    }
    let refs = std::mem::take(&mut doc.refs);
    // Os recursos de UM conteúdo: só os nomes que ele usa.
    let resources = |res: &mut pdf_writer::writers::Resources<'_>, used: &BTreeSet<String>| {
        let mut g = res.ext_g_states();
        for n in used.iter().filter(|n| n.starts_with('A')) {
            if let Some(r) = refs.get(n) {
                g.pair(Name(n.as_bytes()), *r);
            }
        }
        g.finish();
        let mut x = res.x_objects();
        for n in used.iter().filter(|n| !n.starts_with('A') && *n != "F0") {
            if let Some(r) = refs.get(n) {
                x.pair(Name(n.as_bytes()), *r);
            }
        }
        x.finish();
        if used.contains("F0") {
            res.fonts().pair(Name(b"F0"), font);
        }
    };
    for (r, body, [w, h], used) in std::mem::take(&mut doc.forms) {
        let packed = deflate(&body);
        let mut form = doc.pdf.form_xobject(r, &packed);
        form.filter(Filter::FlateDecode);
        form.bbox(Rect::new(0.0, 0.0, f(w), f(h)));
        form.group().transparency();
        resources(&mut form.resources(), &used);
        form.finish();
    }
    for (i, (p, (body, used))) in pages.iter().zip(contents).enumerate() {
        let content = doc.id();
        let annots: Vec<Ref> = p.links.iter().map(|_| doc.id()).collect();
        let [w, h] = p.size;
        let mut page = doc.pdf.page(page_ids[i]);
        page.media_box(Rect::new(0.0, 0.0, f(w), f(h)));
        page.parent(tree);
        page.contents(content);
        page.annotations(annots.iter().copied());
        resources(&mut page.resources(), &used);
        page.finish();
        let packed = deflate(&body);
        doc.pdf.stream(content, &packed).filter(Filter::FlateDecode);
        for (a, l) in annots.iter().zip(&p.links) {
            let (Some(&dest), Some(target)) = (page_ids.get(l.page), pages.get(l.page)) else {
                continue;
            };
            let bx = l.rect;
            let mut an = doc.pdf.annotation(*a);
            an.subtype(AnnotationType::Link);
            an.rect(Rect::new(f(bx[0]), f(h - bx[3]), f(bx[2]), f(h - bx[1])));
            an.border(0.0, 0.0, 0.0, None);
            // Para o sítio do alvo (na mesma página ou noutra), com o zoom de quem lê.
            let top = target.size[1] - l.at[1];
            an.action()
                .action_type(ActionType::GoTo)
                .destination()
                .page(dest)
                .xyz(f(l.at[0]), f(top), None);
        }
    }
    doc.pdf.finish()
}
