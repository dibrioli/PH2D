//! **A TRAVESSIA**: a árvore chata do usvg vira formas do documento.
//!
//! # A ordem das três perguntas
//!
//! 1. **Onde** — o `abs_transform` do nó, composto com a lei dos eixos ([`crate::svg_to_world`]) e
//!    com a centragem do `viewBox`. Uma só, assada pela porta que já leva gradiente, raio de quina
//!    e largura de traço junto ([`ph2d_vec_scene::bake_xform`]).
//! 2. **Com que tinta** — [`crate::paint`].
//! 3. **O que ficou de fora** — contado por espécie e devolvido em [`crate::Drawing::notes`].
//!
//! ⚠️⚠️ **O `Path::data()` do usvg 0.48 está em espaço LOCAL.** O doc dele diz *"All segments are
//! in absolute coordinates"*, e ali *absolute* quer dizer **comandos** absolutos (o `M` contra o
//! `m` do atributo `d`), não **espaço** absoluto: o `Path::new` guarda `data` intacto e calcula
//! `abs_bounding_box = bounding_box.transform(abs_transform)` — se os dados já estivessem em
//! espaço absoluto, essa linha transformaria duas vezes. Um ficheiro **sem** `transform` nenhum lê
//! igual das duas maneiras, e é por isso que a fixtura desta lei tem um `<g transform>` ANINHADO.

use crate::paint;
use crate::{Drawing, Error, Shape, SvgGroup};
use ph2d_blend_mode::BlendMode;
use ph2d_vec_scene::{Contour, FillRule, Opacity, VecPath, VecVertex, Xform, bake_xform};
use std::collections::BTreeMap;

/// **Lê e endurece.** A porta única de *"como é que este app abre um SVG com segurança?"* — o
/// tecto de bytes ANTES do parse e o resolvedor de `href` neutralizado.
///
/// ⚠️ O `image_href_resolver` por omissão do usvg chama `std::fs::read(href)` para um
/// `<image href="…">`: um ficheiro hostil com `href="/etc/passwd"` tocaria no disco. As `data:` URI
/// passam; uma string **não** é resolvida.
pub fn parse(src: &[u8]) -> Result<usvg::Tree, Error> {
    if src.is_empty() {
        return Err(Error::Parse("empty source".to_owned()));
    }
    if src.len() as u64 > crate::MAX_SVG_BYTES {
        return Err(Error::TooLarge(src.len() as u64));
    }
    let opts = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_href, _opts| None),
        },
        ..Default::default()
    };
    usvg::Tree::from_data(src, &opts).map_err(|e| Error::Parse(e.to_string()))
}

/// Como colocar o desenho no mundo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// **Um px é um px**: o mesmo divisor que dimensiona uma sprite importada.
    pub pixels_per_meter: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            pixels_per_meter: 1.0,
        }
    }
}

/// ⭐⭐⭐ **O DESENHO**, pronto a entrar no documento — geometria em mundo, centrada na origem.
///
/// Centrada porque quem chama sabe **onde** largar e não devia ter de saber **como**: a shell
/// translada uma vez para o ponto do gesto, exactamente como faz com uma sprite.
pub fn import(src: &[u8], opts: &Options) -> Result<Drawing, Error> {
    let tree = parse(src)?;
    let mut w = Walk {
        out: Drawing::default(),
        lost: BTreeMap::new(),
    };
    // A moldura: SVG → mundo, e depois a centragem do `viewBox`.
    //
    // ⚠️ `lh` chega **negativo** (o eixo virou), então subtrair metade dele SOBE o desenho — a
    // mesma expressão serve os dois eixos, e é por isso que não há um caso especial aqui. Escrever
    // `+lh/2` para o Y (a correcção que parece óbvia) empurraria o desenho uma altura inteira para
    // baixo.
    let eixos = crate::svg_to_world(opts.pixels_per_meter);
    let [lw, lh] = eixos.apply([
        f64::from(tree.size().width()),
        f64::from(tree.size().height()),
    ]);
    let centrar = Xform([1.0, 0.0, 0.0, 1.0, -lw * 0.5, -lh * 0.5]);
    let frame = eixos.then(&centrar);
    w.out.size = [lw.abs(), lh.abs()];

    // ⚠️ Os `<text>` são contados no XML CRU: com a feature `text` desligada o usvg apaga-os da
    // árvore, então nenhuma travessia os vê. *Uma perda que a árvore não regista tem de ser lida
    // onde ela ainda existe.*
    //
    // ⛔ **Limitação NOMEADA: um `.svgz` não é UTF-8**, então este ramo não corre e um ficheiro
    // comprimido com texto entra sem a nota. Descomprimir aqui seria a **segunda** descompressão do
    // mesmo ficheiro (o usvg já a fez lá dentro) por causa de um aviso; a saída certa é a feature
    // `text` do usvg, e essa está recusada com medição no `Cargo.toml`.
    //
    // ⛔ E se um dia a feature `text` for ligada, esta contagem passa a SOMAR com o braço
    // `Node::Text` do [`Walk::group`] — quem a ligar apaga um dos dois.
    if let Ok(txt) = std::str::from_utf8(src)
        && let Ok(doc) = usvg::roxmltree::Document::parse(txt)
    {
        let n = doc
            .descendants()
            .filter(|d| d.is_element() && d.tag_name().name() == "text")
            .count();
        if n > 0 {
            w.lost.insert("<text> (esta build nao carrega fontes)", n);
        }
    }

    w.group(tree.root(), &frame, None, 1.0, BlendMode::Normal);
    w.out.notes = w
        .lost
        .iter()
        .map(|(o, n)| format!("{n} x {o}"))
        .collect::<Vec<_>>();
    Ok(w.out)
}

struct Walk {
    out: Drawing,
    /// Espécie → quantas vezes. Um mapa (e não uma lista de frases) porque a mesma perda repetida
    /// 300 vezes tem de sair numa linha com a contagem, e não em 300 linhas.
    lost: BTreeMap<&'static str, usize>,
}

impl Walk {
    fn perdeu(&mut self, o: &'static str) {
        *self.lost.entry(o).or_insert(0) += 1;
    }

    /// Percorre um grupo. `pai` é o grupo REAL mais próximo (ver [`Self::real`]).
    fn group(
        &mut self,
        g: &usvg::Group,
        frame: &Xform,
        pai: Option<usize>,
        alfa: f32,
        blend: BlendMode,
    ) {
        for n in g.children() {
            match n {
                usvg::Node::Group(inner) => {
                    if inner.clip_path().is_some() {
                        self.perdeu("clip-path");
                    }
                    if inner.mask().is_some() {
                        self.perdeu("mask");
                    }
                    if !inner.filters().is_empty() {
                        self.perdeu("filter");
                    }
                    let a = alfa * inner.opacity().get();
                    let b = traduz_blend(inner.blend_mode()).unwrap_or(blend);
                    let meu = self.real(inner, pai);
                    let antes = self.out.shapes.len();
                    self.group(inner, frame, meu.or(pai), a, b);
                    // ⛔ **A aproximação, e ela só existe quando é OBSERVÁVEL.** Um grupo com
                    // opacidade compõe-se INTEIRO e só depois desvanece; dar a mesma fracção a
                    // cada filho deixa as sobreposições entre eles à vista. Com **uma** forma lá
                    // dentro as duas contas são a mesma — e é esse o caso comum, porque o usvg
                    // embrulha num grupo toda forma que traz `opacity` própria.
                    let produzidas = self.out.shapes.len() - antes;
                    if produzidas > 1 && inner.opacity().get() < 1.0 {
                        self.perdeu("opacidade de GRUPO sobre varias formas (foi a cada uma)");
                    }
                    if produzidas > 1 && traduz_blend(inner.blend_mode()).is_some() {
                        self.perdeu("mistura de GRUPO sobre varias formas (foi a cada uma)");
                    }
                }
                usvg::Node::Path(p) => self.path(p, frame, pai, alfa, blend),
                usvg::Node::Image(_) => self.perdeu("<image> (o vector nao carrega pixels)"),
                usvg::Node::Text(_) => self.perdeu("<text> (esta build nao carrega fontes)"),
            }
        }
    }

    /// Um grupo é REAL para a Hierarquia quando o ficheiro lhe deu um `id`.
    ///
    /// ⚠️ O usvg **fabrica** grupos para embrulhar opacidade, clip e filtro; eles não têm `id` e
    /// não são nada que o artista tenha desenhado. Derivar do `id` separa os dois sem uma lista.
    fn real(&mut self, g: &usvg::Group, pai: Option<usize>) -> Option<usize> {
        if g.id().is_empty() {
            return None;
        }
        self.out.groups.push(SvgGroup {
            name: g.id().to_owned(),
            parent: pai,
        });
        Some(self.out.groups.len() - 1)
    }

    fn path(
        &mut self,
        p: &usvg::Path,
        frame: &Xform,
        pai: Option<usize>,
        alfa: f32,
        blend: BlendMode,
    ) {
        if !p.is_visible() {
            return;
        }
        let mut contornos = contornos(p.data());
        if contornos.is_empty() {
            return;
        }
        let (verts, closed) = contornos.remove(0);
        let (fill, perdeu_f) = match p.fill() {
            Some(f) => {
                let (t, l) = paint::tinta(f.paint(), f.opacity().get());
                (Some(t), l)
            }
            None => (None, None),
        };
        let (stroke, perdeu_s) = match p.stroke() {
            Some(s) => {
                let (t, l) = paint::traco(s);
                (Some(t), l)
            }
            None => (None, None),
        };
        for o in [perdeu_f, perdeu_s].into_iter().flatten() {
            self.perdeu(o);
        }
        if p.paint_order() == usvg::PaintOrder::StrokeAndFill {
            self.perdeu("paint-order=stroke (o documento pinta sempre o traco por cima)");
        }
        let mut path = VecPath {
            verts,
            closed,
            fill,
            stroke,
            subpaths: contornos
                .into_iter()
                .map(|(verts, closed)| Contour { verts, closed })
                .collect(),
            fill_rule: match p.fill().map(usvg::Fill::rule) {
                Some(usvg::FillRule::EvenOdd) => FillRule::EvenOdd,
                Some(usvg::FillRule::NonZero) | None => FillRule::NonZero,
            },
            opacity: Opacity::new(alfa),
            blend,
            ..VecPath::default()
        };
        // ⚠️ UMA porta leva tudo ao mundo: âncoras, handles, geometria do gradiente, raio de quina
        // e a LARGURA do traço. Escalar a largura à mão aqui seria a segunda lei.
        let x = xform_de(p.abs_transform()).then(frame);
        bake_xform(&mut path, &x);
        self.out.shapes.push(Shape {
            path,
            name: p.id().to_owned(),
            group: pai,
        });
    }
}

/// O `mix-blend-mode` do SVG no vocabulário do app. `None` para `Normal` — quem herda o modo do
/// pai precisa de distinguir *"este nó não pediu nada"* de *"este nó pediu Normal"*.
fn traduz_blend(b: usvg::BlendMode) -> Option<BlendMode> {
    Some(match b {
        usvg::BlendMode::Normal => return None,
        usvg::BlendMode::Multiply => BlendMode::Multiply,
        usvg::BlendMode::Screen => BlendMode::Screen,
        usvg::BlendMode::Overlay => BlendMode::Overlay,
        usvg::BlendMode::Darken => BlendMode::Darken,
        usvg::BlendMode::Lighten => BlendMode::Lighten,
        usvg::BlendMode::ColorDodge => BlendMode::ColorDodge,
        usvg::BlendMode::ColorBurn => BlendMode::ColorBurn,
        usvg::BlendMode::HardLight => BlendMode::HardLight,
        usvg::BlendMode::SoftLight => BlendMode::SoftLight,
        usvg::BlendMode::Difference => BlendMode::Difference,
        usvg::BlendMode::Exclusion => BlendMode::Exclusion,
        usvg::BlendMode::Hue => BlendMode::Hue,
        usvg::BlendMode::Saturation => BlendMode::Saturation,
        usvg::BlendMode::Color => BlendMode::Color,
        usvg::BlendMode::Luminosity => BlendMode::Luminosity,
    })
}

/// O afim do usvg no vocabulário do documento. `x' = sx·x + kx·y + tx` dos dois lados — o que muda
/// é a ORDEM dos seis números, e trocá-la transpõe a matriz em silêncio.
fn xform_de(t: usvg::Transform) -> Xform {
    Xform([
        f64::from(t.sx),
        f64::from(t.ky),
        f64::from(t.kx),
        f64::from(t.sy),
        f64::from(t.tx),
        f64::from(t.ty),
    ])
}

fn pt(p: usvg::tiny_skia_path::Point) -> [f64; 2] {
    [f64::from(p.x), f64::from(p.y)]
}

/// Fecha o contorno em construção, fundindo o vértice repetido de um `Z` explícito.
fn fecha(cur: &mut Vec<VecVertex>, fechado: &mut bool, out: &mut Vec<(Vec<VecVertex>, bool)>) {
    if cur.len() >= 2 {
        if *fechado
            && let (Some(u), Some(p)) = (cur.last().copied(), cur.first().copied())
            && dist2(u.anchor, p.anchor) < COINCIDENTE
        {
            cur.pop();
            if let Some(f) = cur.first_mut() {
                f.in_handle = u.in_handle;
            }
        }
        out.push((std::mem::take(cur), *fechado));
    } else {
        cur.clear();
    }
    *fechado = false;
}

/// Quão perto dois pontos têm de estar para serem o MESMO ponto de um `Z`.
///
/// É uma distância AO QUADRADO em unidades de SVG: `1e-9` de unidade é mil vezes mais fino do que
/// a precisão com que qualquer editor escreve um `d`, e grosso o bastante para apanhar o `f32` com
/// que o usvg guarda os pontos.
const COINCIDENTE: f64 = 1e-18;

/// Os CONTORNOS de um caminho, em vértices cúbicos.
///
/// ⚠️ **Uma quadrática vira uma cúbica EXACTA** (elevação de grau: os dois controlos ficam a ⅔ do
/// caminho entre cada extremo e o controlo quadrático) — não é uma aproximação, e é por isso que
/// não há tolerância nenhuma aqui.
///
/// ⚠️ **O `Z` do SVG fecha com uma RECTA até ao início.** Quando o ficheiro já tinha voltado ao
/// ponto inicial à mão, o vértice repetido é fundido — e o handle de ENTRADA dele passa para o
/// primeiro vértice, senão a curva com que o caminho regressa desapareceria.
fn contornos(data: &usvg::tiny_skia_path::Path) -> Vec<(Vec<VecVertex>, bool)> {
    use usvg::tiny_skia_path::PathSegment as S;
    /// O peso da elevação de grau: uma quadrática `(a, c, p)` é a cúbica cujos controlos estão a
    /// dois terços do caminho entre cada extremo e `c`. É identidade algébrica, não ajuste.
    const DOIS_TERCOS: f64 = 2.0 / 3.0;
    let mut out: Vec<(Vec<VecVertex>, bool)> = Vec::new();
    let mut cur: Vec<VecVertex> = Vec::new();
    let mut fechado = false;
    for seg in data.segments() {
        match seg {
            S::MoveTo(p) => {
                fecha(&mut cur, &mut fechado, &mut out);
                cur.push(VecVertex::corner(pt(p)));
            }
            S::LineTo(p) => cur.push(VecVertex::corner(pt(p))),
            S::QuadTo(c, p) => {
                let a = cur.last().map_or_else(|| pt(c), |v| v.anchor);
                let (c, p) = (pt(c), pt(p));
                let c1 = [
                    a[0] + DOIS_TERCOS * (c[0] - a[0]),
                    a[1] + DOIS_TERCOS * (c[1] - a[1]),
                ];
                let c2 = [
                    p[0] + DOIS_TERCOS * (c[0] - p[0]),
                    p[1] + DOIS_TERCOS * (c[1] - p[1]),
                ];
                empurra(&mut cur, c1, c2, p);
            }
            S::CubicTo(c1, c2, p) => empurra(&mut cur, pt(c1), pt(c2), pt(p)),
            S::Close => {
                fechado = true;
                fecha(&mut cur, &mut fechado, &mut out);
            }
        }
    }
    fecha(&mut cur, &mut fechado, &mut out);
    out
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}

/// Um segmento cúbico: o handle de SAÍDA é do vértice que já lá está, o de ENTRADA é do novo.
fn empurra(cur: &mut Vec<VecVertex>, c1: [f64; 2], c2: [f64; 2], p: [f64; 2]) {
    if let Some(u) = cur.last_mut() {
        u.out_handle = c1;
    }
    let mut v = VecVertex::corner(p);
    v.in_handle = c2;
    cur.push(v);
}
