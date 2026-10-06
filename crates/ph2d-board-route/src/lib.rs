//! **As rotas das setas do Quadro** (MiroClone, W2) — de uma [`Connector`] (a relação guardada no
//! documento) à linha que se desenha: onde cada ponta encosta, por que lado sai, e o caminho pelos
//! pontos de ajuste do artista — a curva do Miro (sai e entra perpendicular, passa suave pelos
//! pontos), segmentos rectos, ou o cotovelo do roteador reusado (`ph2d-vec-connect`) que só contorna
//! as SUAS duas formas. As pontas de seta são as do catálogo vectorial.
//!
//! ⛔ Nenhuma rota reage a OUTRA forma (ordem do dono, 06/10: «setas não se reajustam sozinhas»; o
//! desvio automático que aqui esteve está nas recusas do plano §6). A rota NÃO vive no documento:
//! deriva-se, e a [`RouteCache`] refá-la só quando a seta ou uma das suas formas mudou.
//!
//! ⚠️ O roteador nomeia as direcções num mundo Y-para-CIMA (`Dir::North` = `+y`); o quadro é Y para
//! baixo, e a geometria não se importa — `North` aqui é «para baixo no ecrã». Nada vira: só o nome.

mod cache;
mod index;

pub use cache::RouteCache;
pub use ph2d_vec_connect::Dir;
pub use ph2d_vec_scene::VecPath;

use ph2d_board_geom::{inside, nearest_on_outline, ray_exit};
use ph2d_board_model::{Anchor, Element, End, Head, Route};
use ph2d_vec_connect::{
    Aabb, EndSpec, RouteInput, RouteKind, exit_point, port_side, route, side_towards,
};
use ph2d_vec_scene::{Marker, VecVertex, end_tangent, trim_path};
use ph2d_vector::BezPath;

/// ⭐ **O recuo da rota em cotovelo antes de poder dobrar** (o *jetty*), em unidades do mundo.
/// Medido no oráculo (Excalidraw 0.18.1, editor montado, 2026-10-06): a seta que sai pela direita e
/// tem de dar a volta avança **40** antes de dobrar, nas duas pontas — com caixas de 140×90
/// (`saidas/seta_cotovelo_volta.editor.json`) e de 280×180 (`…_volta_grande`): é FIXO, não
/// proporcional à caixa. A folga às duas formas da seta é `MARGIN_K × JETTY` (a lei do roteador).
pub const JETTY: f64 = 40.0;

/// O afastamento entre setas paralelas (as que ligam o MESMO par de formas): a razão do vectorial
/// (`SPREAD_STEP / JETTY_MAX` = 0,35) sobre o recuo do quadro.
pub const SPREAD_STEP: f64 = 0.35 * JETTY;

/// ⭐ **O braço da curva numa PONTA** — metade da distância até ao ponto seguinte, medida ao longo do
/// eixo por onde a seta sai. Medido 06/10 na captura do Miro que o dono mandou (a curva entre duas
/// caixas desalinhadas: ajuste de uma cúbica com as tangentes horizontais aos píxeis do traço): braço
/// **154** para um afastamento horizontal de **307** (erro de 2,9 px, a espessura do traço).
pub const END_ARM: f64 = 0.5;

/// O braço da curva num PONTO DE AJUSTE (e o piso do da ponta): ⅓ do trecho — o spline cardinal
/// que passa pelos pontos sem escapar deles (o `DEFAULT_CURVE_ARM` do conector vectorial). O piso
/// faz a curva de duas pontas que saem para o MESMO lado (o «não» por baixo) dar a volta em vez de
/// colapsar.
pub const POINT_ARM: f64 = 1.0 / 3.0;

/// ⭐ **O tamanho da ponta de seta**, em múltiplos da largura do traço sobre a caixa do catálogo
/// (`Marker`: comprimento 4·w). Medido no oráculo (`saidas/seta_reta_ligada.svg`, `strokeWidth 2`,
/// ponta `arrow`): as duas riscas recuam **23,49** ao longo da linha ⇒ `23,49 / (4 · 2) = 2,94`.
/// ⚠️ A abertura é a do catálogo (26,6° de meia-abertura contra os ~20° do Excalidraw): uma lei de
/// pontas para o app inteiro vale mais que copiar o ângulo.
pub const HEAD_SCALE: f64 = 2.94;

/// A largura de quebra do rótulo de uma seta (mundo): a de uma caixa de nascença do editor
/// (`ph2d_board_edit::CLICK_SIZE`, gate lá) — um rótulo é um título curto, não um parágrafo.
pub const LABEL_WRAP: f64 = 160.0;

/// Onde começa (canto superior esquerdo, mundo) o bloco do rótulo de altura `block_h` centrado no
/// meio `mid` da rota — a MESMA conta para quem o desenha e para quem o edita.
#[must_use]
pub fn label_origin(mid: [f64; 2], block_h: f64) -> [f64; 2] {
    [mid[0] - LABEL_WRAP / 2.0, mid[1] - block_h / 2.0]
}

/// O que a [`RouteCache`] guarda de cada seta: a rota já desenhável.
#[derive(Clone, Debug, PartialEq)]
pub struct Routed {
    /// As ESTAÇÕES (mundo): a ponta de início, os pontos de ajuste, a ponta de fim.
    pub stations: Vec<[f64; 2]>,
    /// O caminho que se desenha, sem as pontas de seta.
    pub path: VecPath,
    /// O lado por onde cada ponta saiu `[início, fim]` (a histerese do próximo cálculo).
    pub sides: [Dir; 2],
    /// A caixa da rota `[x0, y0, x1, y1]` (o recorte ao ecrã e a selecção por arrasto).
    pub bbox: [f64; 4],
    /// O meio da rota por comprimento: onde vive o rótulo.
    pub mid: [f64; 2],
    /// O meio (por comprimento) de cada TRECHO entre duas estações: as pegas que, arrastadas, criam
    /// um ponto de ajuste novo ali (as bolinhas cheias do Miro).
    pub leg_mids: Vec<[f64; 2]>,
}

impl Routed {
    /// Uma rota pelas `stations` (pontas e pontos de ajuste) já resolvidas, a sair por `sides`: a
    /// curva, os segmentos rectos, ou — no cotovelo — a polilinha ortogonal dada. Os ícones da barra
    /// usam-na: o desenho é a MESMA conta das setas do quadro.
    #[must_use]
    pub fn from_points(stations: Vec<[f64; 2]>, kind: Route, sides: [Dir; 2]) -> Self {
        let n = stations.len();
        let breaks: Vec<usize> = (0..n).collect();
        let verts = if kind == Route::Curved {
            curve(&stations, sides.map(Dir::vec))
        } else {
            stations.iter().map(|p| VecVertex::corner(*p)).collect()
        };
        Self::assemble(stations, verts, &breaks, sides)
    }

    /// `verts` = o caminho; `breaks[k]` = o índice do vértice da estação `k`.
    fn assemble(
        stations: Vec<[f64; 2]>,
        verts: Vec<VecVertex>,
        breaks: &[usize],
        sides: [Dir; 2],
    ) -> Self {
        let path = VecPath {
            verts,
            closed: false,
            ..VecPath::default()
        };
        let leg_mids = breaks
            .windows(2)
            .map(|w| {
                midpoint(&VecPath {
                    verts: path.verts[w[0]..=w[1]].to_vec(),
                    closed: false,
                    ..VecPath::default()
                })
            })
            .collect();
        Self {
            bbox: bbox_of(&path),
            mid: midpoint(&path),
            stations,
            path,
            sides,
            leg_mids,
        }
    }

    /// As duas pontas `[início, fim]` no mundo.
    #[must_use]
    pub fn ends(&self) -> [[f64; 2]; 2] {
        [self.stations[0], self.stations[self.stations.len() - 1]]
    }

    /// Os vértices do caminho (as dobras do cotovelo; numa curva, as estações).
    #[must_use]
    pub fn polyline(&self) -> Vec<[f64; 2]> {
        self.path.verts.iter().map(|v| v.anchor).collect()
    }

    /// Os pontos de ajuste (as estações do meio).
    #[must_use]
    pub fn waypoints(&self) -> &[[f64; 2]] {
        &self.stations[1..self.stations.len() - 1]
    }

    /// A distância de `p` à linha desenhada (o acerto do clique).
    #[must_use]
    pub fn distance(&self, p: [f64; 2]) -> f64 {
        use ph2d_vector::{ParamCurveNearest, Point};
        ph2d_vec_render::build_bezpath(&self.path)
            .segments()
            .map(|s| s.nearest(Point::new(p[0], p[1]), 1e-6).distance_sq)
            .fold(f64::INFINITY, f64::min)
            .sqrt()
    }
}

/// O desenho de uma seta: a linha (já recuada onde há ponta cheia ou vazada) e as pontas.
#[derive(Clone, Debug)]
pub struct Drawn {
    pub line: BezPath,
    /// `(contorno, preenchida?)` — preenchida se pinta, senão traça.
    pub heads: Vec<(BezPath, bool)>,
}

/// O catálogo de pontas que desenha cada [`Head`].
#[must_use]
pub fn marker(h: Head) -> Marker {
    match h {
        Head::None => Marker::None,
        Head::Arrow => Marker::Open,
        Head::Triangle => Marker::Triangle,
        Head::Diamond => Marker::Diamond,
        Head::DiamondOpen => Marker::DiamondOpen,
        Head::Circle => Marker::Circle,
        Head::CircleOpen => Marker::CircleOpen,
        Head::Bar => Marker::Bar,
    }
}

/// ⭐ **A seta desenhada**: a ponta com o bico NA ponta da rota (a linha encosta no contorno, a seta
/// aponta para a forma) e a linha recuada o que a ponta tapa (`Marker::inset`).
#[must_use]
pub fn drawn(r: &Routed, heads: [Head; 2], width: f64) -> Drawn {
    drawn_at(r, heads, width, HEAD_SCALE)
}

/// [`drawn`] com o tamanho da ponta `scale` (os ÍCONES da barra: a ponta do quadro, em 16 px, não
/// cabia no botão — foto da cena 3, 06/10).
#[must_use]
pub fn drawn_at(r: &Routed, heads: [Head; 2], width: f64, scale: f64) -> Drawn {
    let ms = heads.map(marker);
    let inset = |m: Marker| m.inset(scale) * width;
    let line = trim_path(&r.path, inset(ms[0]), inset(ms[1]))
        .map_or_else(BezPath::new, |p| ph2d_vec_render::build_bezpath(&p));
    let mut out = Vec::new();
    for (i, m) in ms.into_iter().enumerate() {
        let Some((tip, dir)) = end_tangent(&r.path, i == 0) else {
            continue;
        };
        if let Some(head) = m.build(tip, dir, width, scale, 0.0) {
            let closed = head.closed;
            out.push((
                ph2d_vec_render::build_bezpath(&head),
                closed && m.is_filled(),
            ));
        }
    }
    Drawn { line, heads: out }
}

/// O sítio do mundo de um ponto fixo `[u, v]` da caixa de `el` (antes de rodar).
#[must_use]
pub fn fixed_world(el: &Element, [u, v]: [f64; 2]) -> [f64; 2] {
    el.rotate([el.x + u * el.w, el.y + v * el.h])
}

/// O ponto `q` (mundo) na caixa de `el` antes de rodar, normalizado a `0..1`.
#[must_use]
pub fn normalized(el: &Element, q: [f64; 2]) -> [f64; 2] {
    let [x, y] = el.unrotate(q);
    [
        (x - el.x) / el.w.max(f64::EPSILON),
        (y - el.y) / el.h.max(f64::EPSILON),
    ]
}

/// ⭐ **A regra da ligação** (a semântica do Miro, pesquisa 01 §4.1, no gesto do tldraw): uma ponta
/// largada sobre `el` em `p` prende-se ao CENTRO quando cai no miolo, e a um PONTO FIXO quando cai
/// na faixa de largura `band` (mundo) junto ao contorno, por dentro ou por fora — o ponto do
/// contorno mais perto, colado ao meio de um lado quando passa a menos de `band` dele. `None` =
/// `p` está fora da forma e da faixa.
#[must_use]
pub fn anchor_for(el: &Element, p: [f64; 2], band: f64) -> Option<Anchor> {
    let (q, d) = nearest_on_outline(el, p)?;
    let within = inside(el, p);
    if !within && d > band {
        return None;
    }
    if within && d > band {
        return Some(Anchor::Center);
    }
    let mid = [[0.5, 0.0], [1.0, 0.5], [0.5, 1.0], [0.0, 0.5]]
        .into_iter()
        .map(|uv| (uv, fixed_world(el, uv)))
        .filter(|(_, m)| dist(*m, q) <= band)
        .filter(|(_, m)| nearest_on_outline(el, *m).is_some_and(|(_, e)| e < ON_OUTLINE))
        .min_by(|a, b| dist(a.1, q).total_cmp(&dist(b.1, q)));
    Some(Anchor::Fixed(
        mid.map_or_else(|| normalized(el, q), |m| m.0),
    ))
}

/// Um meio de lado está NO contorno (e não no vazio, como o meio do lado esquerdo de um triângulo).
const ON_OUTLINE: f64 = 1e-6;

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Uma ponta já resolvida contra o documento.
#[derive(Clone, Debug)]
pub(crate) enum Resolved<'a> {
    /// Um ponto (solta, ou presa a uma forma que já não existe).
    Point([f64; 2]),
    Shape(&'a Element, Anchor),
}

impl Resolved<'_> {
    fn bbox(&self) -> Aabb {
        match self {
            Resolved::Point(p) => Aabb::new(*p, *p),
            Resolved::Shape(el, _) => aabb(el),
        }
    }

    fn center(&self) -> [f64; 2] {
        self.bbox().center()
    }
}

pub(crate) fn aabb(el: &Element) -> Aabb {
    let [x0, y0, x1, y1] = el.aabb();
    Aabb::new([x0, y0], [x1, y1])
}

/// Por onde a ponta `me` sai rumo a `other`.
fn exit(
    me: &Resolved<'_>,
    other: [f64; 2],
    kind: RouteKind,
    prev: Option<Dir>,
    spread: f64,
) -> ([f64; 2], Dir) {
    match *me {
        Resolved::Point(p) => {
            exit_point(Aabb::new(p, p), false, other, kind, prev, spread, |_, _| {
                None
            })
        }
        Resolved::Shape(el, Anchor::Fixed(uv)) => {
            let p = fixed_world(el, uv);
            (p, port_side(aabb(el), p, prev))
        }
        Resolved::Shape(el, Anchor::Center) => {
            exit_point(aabb(el), true, other, kind, prev, spread, |from, ray| {
                ray_exit(el, from, ray)
            })
        }
    }
}

/// ⭐ **A rota de uma seta** entre duas pontas resolvidas, pelos `waypoints` do artista, com a
/// histerese `prev` e o afastamento `spread` das paralelas. Só as duas formas da seta contam (o
/// cotovelo contorna-as); nenhuma outra.
pub(crate) fn compute(
    ends: [&Resolved<'_>; 2],
    kind: Route,
    waypoints: &[[f64; 2]],
    prev: [Option<Dir>; 2],
    spread: f64,
) -> Routed {
    let rk = match kind {
        Route::Straight => RouteKind::Straight,
        Route::Elbow | Route::Curved => RouteKind::Orthogonal,
    };
    // Cada ponta sai rumo à estação VIZINHA: o 1.º ponto de ajuste, ou a outra ponta.
    let toward0 = waypoints.first().copied().unwrap_or(ends[1].center());
    let toward1 = waypoints.last().copied().unwrap_or(ends[0].center());
    let (p0, d0) = exit(ends[0], toward0, rk, prev[0], spread);
    let (p1, d1) = exit(ends[1], toward1, rk, prev[1], -spread);
    let mut stations = vec![p0];
    stations.extend_from_slice(waypoints);
    stations.push(p1);
    if kind != Route::Elbow {
        return Routed::from_points(stations, kind, [d0, d1]);
    }
    // O cotovelo: cada trecho entre estações pelo roteador, com só as DUAS formas da seta por
    // obstáculo. Um ponto de ajuste é uma ponta solta no meio (sai rumo à estação seguinte).
    let walls: Vec<Aabb> = ends
        .iter()
        .filter(|e| matches!(e, Resolved::Shape(..)))
        .map(|e| e.bbox())
        .collect();
    let self_loop = match (ends[0], ends[1]) {
        (Resolved::Shape(a, _), Resolved::Shape(b, _)) if a.id == b.id && waypoints.is_empty() => {
            Some(aabb(a))
        }
        _ => None,
    };
    let toward = |from: [f64; 2], to: [f64; 2]| {
        side_towards([to[0] - from[0], to[1] - from[1]], 0.0, 0.0, None)
    };
    let last = stations.len() - 1;
    let mut pts: Vec<[f64; 2]> = vec![p0];
    let mut breaks = vec![0];
    for i in 0..last {
        let (a, b) = (stations[i], stations[i + 1]);
        let da = if i == 0 { d0 } else { toward(a, b) };
        let db = if i + 1 == last { d1 } else { toward(b, a) };
        let leg = route(&RouteInput {
            start: EndSpec { at: a, dir: da },
            end: EndSpec { at: b, dir: db },
            kind: RouteKind::Orthogonal,
            jetty: JETTY,
            obstacles: &walls,
            spread,
            self_loop,
        });
        pts.extend(leg.into_iter().skip(1));
        breaks.push(pts.len() - 1);
    }
    let verts = pts.iter().map(|p| VecVertex::corner(*p)).collect();
    Routed::assemble(stations, verts, &breaks, [d0, d1])
}

/// ⭐ **A curva do Miro** pelas estações: em cada PONTA a tangente é a da saída (perpendicular à
/// face, `d[k]` aponta para FORA da forma) e o braço [`END_ARM`] da distância ao vizinho ao longo
/// dela (no mínimo [`POINT_ARM`] do trecho); num ponto de ajuste a tangente é a do Catmull-Rom (do
/// vizinho de trás ao da frente) e o braço [`POINT_ARM`] do trecho.
fn curve(st: &[[f64; 2]], d: [[f64; 2]; 2]) -> Vec<VecVertex> {
    let n = st.len();
    let sub = |a: [f64; 2], b: [f64; 2]| [a[0] - b[0], a[1] - b[1]];
    let unit = |v: [f64; 2]| {
        let l = v[0].hypot(v[1]);
        if l < 1e-12 {
            [0.0, 0.0]
        } else {
            [v[0] / l, v[1] / l]
        }
    };
    // A tangente de PERCURSO em cada estação (no fim, entra contra a saída da forma).
    let tangent = |i: usize| -> [f64; 2] {
        if i == 0 {
            d[0]
        } else if i == n - 1 {
            [-d[1][0], -d[1][1]]
        } else {
            unit(sub(st[i + 1], st[i - 1]))
        }
    };
    let arm = |i: usize, toward: usize| -> f64 {
        let chord = dist(st[i], st[toward]);
        let floor = POINT_ARM * chord;
        if i == 0 || i == n - 1 {
            let t = tangent(i);
            let along = sub(st[toward], st[i]);
            (END_ARM * (along[0] * t[0] + along[1] * t[1]).abs()).max(floor)
        } else {
            floor
        }
    };
    (0..n)
        .map(|i| {
            let t = tangent(i);
            let mut v = VecVertex::corner(st[i]);
            if i + 1 < n {
                let a = arm(i, i + 1);
                v.out_handle = [st[i][0] + t[0] * a, st[i][1] + t[1] * a];
            }
            if i > 0 {
                let a = arm(i, i - 1);
                v.in_handle = [st[i][0] - t[0] * a, st[i][1] - t[1] * a];
            }
            v
        })
        .collect()
}

fn bbox_of(path: &VecPath) -> [f64; 4] {
    use ph2d_vector::Shape as _;
    let r = ph2d_vec_render::build_bezpath(path).bounding_box();
    [r.x0, r.y0, r.x1, r.y1]
}

/// O ponto a meio comprimento do caminho desenhado.
fn midpoint(path: &VecPath) -> [f64; 2] {
    use ph2d_vector::{ParamCurve, ParamCurveArclen};
    let bp = ph2d_vec_render::build_bezpath(path);
    let segs: Vec<_> = bp.segments().collect();
    let lens: Vec<f64> = segs.iter().map(|s| s.arclen(ARCLEN_ACCURACY)).collect();
    let mut left = lens.iter().sum::<f64>() / 2.0;
    for (s, l) in segs.iter().zip(&lens) {
        if left <= *l {
            let p = s.eval(s.inv_arclen(left, ARCLEN_ACCURACY));
            return [p.x, p.y];
        }
        left -= l;
    }
    path.verts.first().map_or([0.0, 0.0], |v| v.anchor)
}

/// A precisão do comprimento de arco (mundo): um centésimo de unidade, abaixo de um px a qualquer
/// zoom de trabalho.
const ARCLEN_ACCURACY: f64 = 1e-2;

/// O lado da ponta `which` de uma seta presa a um ponto fixo, para o oráculo (o `fixedPoint` do
/// Excalidraw diz a face pela coordenada que vale 0 ou 1).
#[must_use]
pub fn fixed_side(el: &Element, uv: [f64; 2]) -> Dir {
    port_side(aabb(el), fixed_world(el, uv), None)
}

/// A ponta `end` resolvida contra o documento; `fallback` = onde ela estava, se a forma sumiu.
pub(crate) fn resolve<'a>(
    doc: &'a ph2d_board_model::BoardDoc,
    end: End,
    fallback: Option<[f64; 2]>,
) -> Resolved<'a> {
    match end {
        End::Free(p) => Resolved::Point(p),
        End::Bound { target, anchor } => match doc.get(target).filter(|el| el.shape().is_some()) {
            Some(el) => Resolved::Shape(el, anchor),
            None => Resolved::Point(fallback.unwrap_or([0.0, 0.0])),
        },
    }
}

#[cfg(test)]
mod oracle_tests;
#[cfg(test)]
mod tests;
