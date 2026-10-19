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
mod caption;
mod curves;
mod elbow_mid;
pub mod heads;
mod index;

use curves::{bbox_of, chords, curve, facing_jetty, midpoint, spline};

pub use cache::RouteCache;
pub use caption::{CAPTION_PAD, CaptionFrame, Ruler, upright};
pub use elbow_mid::ElbowMid;
pub use heads::marker;
pub use ph2d_vec_connect::Dir;
pub use ph2d_vec_scene::VecPath;

use ph2d_board_geom::{band_entry, inside, nearest_on_outline, ray_exit_band};
use ph2d_board_model::{Anchor, Element, End, Head, ORBIT_GAP, Route};
use ph2d_vec_connect::{
    Aabb, EndSpec, RouteInput, RouteKind, exit_point, port_side, route, side_towards,
};
use ph2d_vec_scene::VecVertex;
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

/// ⭐ **O braço da curva numa PONTA**, em fracção da DISTÂNCIA até à estação vizinha. Medido 06/10 na
/// captura do Miro que o dono mandou (a curva entre duas caixas desalinhadas: ajuste de uma cúbica
/// com as tangentes horizontais aos píxeis do traço): braço **154** para pontas a **342,6** uma da
/// outra (erro de 2,9 px, a espessura do traço) ⇒ 0,45.
/// ⚠️ A mesma captura serve também a «½ do afastamento AO LONGO da saída» (154 / 307): a 1.ª versão
/// usou essa, e com as duas pontas a sair para o MESMO lado ela caía num piso e achatava a volta (2.º
/// smoke do dono, 06/10: «a curva não é tão suave quanto no Miro»). A distância nunca colapsa. Uma
/// captura do Miro com as duas pontas para o mesmo lado separaria as duas leis.
pub const END_ARM: f64 = 0.45;

/// O braço da curva num PONTO DE AJUSTE, em fracção de cada trecho vizinho (a tangente é a do
/// Catmull-Rom: do ponto de trás ao da frente). ⭐ **0,4 MEDIDO** (06/10) na captura do Miro com pontos
/// que o dono mandou (`images/3`: seis estações e cinco meios de trecho): com braços livres trecho a
/// trecho a curva do Miro cai a < 1,1 px da cúbica com ESTAS tangentes (as direcções estão certas); a
/// melhor lei global para o comprimento é 0,35–0,40 do próprio trecho (rms 2,9–3,1 px, a espessura do
/// traço). ⛔ ⅓ (o do vectorial) fazia bico; ½ (escolhido a olho no 3.º smoke) mede 3,9 px; o
/// Catmull-Rom uniforme, o centrípeto e o cordal ficam todos atrás. Régua:
/// `docs/MiroClone/ferramentas/mede_curva_miro.py`.
pub const POINT_ARM: f64 = 0.4;

/// Até onde uma ponta de seta sai da rota (mundo), além de meio traço: a maior base ([`heads::base`],
/// a `arrow`, 25) — a folga do recorte ao ecrã e da camada de opacidade.
#[must_use]
pub fn head_reach(width: f64) -> f64 {
    heads::base(Head::Arrow) + width
}

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
    /// O comprimento do trecho junto de cada ponta `[início, fim]` — a corda entre as duas estações
    /// da ponta; no cotovelo, o último segmento. É o «último trecho» da ponta que encolhe (XA2).
    pub legs: [f64; 2],
    /// W23 B5(1) — no cotovelo, o segmento do meio de cada trecho que saiu em Z (`None` noutro
    /// trecho e noutras rotas): a pega que o arrasta na perpendicular.
    pub elbows: Vec<Option<ElbowMid>>,
}

impl Routed {
    /// Uma rota pelas `stations` (pontas e pontos de ajuste) já resolvidas, a sair por `sides`: a
    /// curva, os segmentos rectos, ou — no cotovelo — a polilinha ortogonal dada. Os ícones da barra
    /// usam-na: o desenho é a MESMA conta das setas do quadro.
    #[must_use]
    pub fn from_points(stations: Vec<[f64; 2]>, kind: Route, sides: [Dir; 2]) -> Self {
        let n = stations.len();
        let breaks: Vec<usize> = (0..n).collect();
        let verts = match kind {
            Route::Curved => curve(&stations, sides.map(Dir::vec)),
            Route::Spline => spline(&stations),
            Route::Straight | Route::Elbow => {
                stations.iter().map(|p| VecVertex::corner(*p)).collect()
            }
        };
        let legs = chords(&stations);
        Self::assemble(stations, verts, &breaks, sides, legs)
    }

    /// ⭐ **A curva com HASTE**: ela acaba a `stems[k]` da ponta `k` (sobre a saída, perpendicular à
    /// forma) e dali segue RECTA até à ponta — a cabeça da seta assenta sempre direita sobre a haste,
    /// por mais apertada que a curva chegue. ⛔ Sem haste, uma curva que dobra nos últimos píxeis
    /// virava a cabeça de lado (4.º smoke do dono, 06/10: o «V» torto à entrada do «Later»).
    fn curved_with_stems(
        stations: Vec<[f64; 2]>,
        sides: [Dir; 2],
        d: [[f64; 2]; 2],
        stems: [f64; 2],
    ) -> Self {
        let n = stations.len();
        let fits = if n == 2 {
            dist(stations[0], stations[1]) > stems[0] + stems[1]
        } else {
            dist(stations[0], stations[1]) > stems[0]
                && dist(stations[n - 2], stations[n - 1]) > stems[1]
        };
        let legs = chords(&stations);
        if !fits || stems == [0.0; 2] {
            let verts = curve(&stations, d);
            let breaks: Vec<usize> = (0..n).collect();
            return Self::assemble(stations, verts, &breaks, sides, legs);
        }
        let mut inner = stations.clone();
        let out = |p: [f64; 2], v: [f64; 2], s: f64| [p[0] + v[0] * s, p[1] + v[1] * s];
        inner[0] = out(stations[0], d[0], stems[0]);
        inner[n - 1] = out(stations[n - 1], d[1], stems[1]);
        let mut verts = curve(&inner, d);
        let mut breaks: Vec<usize> = (0..n).collect();
        if stems[0] > 0.0 {
            verts.insert(0, VecVertex::corner(stations[0]));
            for b in breaks.iter_mut().skip(1) {
                *b += 1;
            }
        }
        if stems[1] > 0.0 {
            verts.push(VecVertex::corner(stations[n - 1]));
            breaks[n - 1] = verts.len() - 1;
        }
        Self::assemble(stations, verts, &breaks, sides, legs)
    }

    /// `verts` = o caminho; `breaks[k]` = o índice do vértice da estação `k`.
    fn assemble(
        stations: Vec<[f64; 2]>,
        verts: Vec<VecVertex>,
        breaks: &[usize],
        sides: [Dir; 2],
        legs: [f64; 2],
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
            legs,
            elbows: Vec::new(),
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

/// O comprimento da HASTE recta debaixo de cada ponta (ver `Routed::curved_with_stems`): o que a
/// ponta ocupa ao longo da linha no tamanho a que o trecho `legs[k]` a encolhe. Sem ponta, sem haste.
fn stems(heads: [Head; 2], legs: [f64; 2]) -> [f64; 2] {
    [0, 1].map(|k| heads::along(heads[k], heads::size(heads[k], legs[k])))
}

/// ⭐ **A seta desenhada**: a ponta com o bico NA ponta da rota (a linha encosta no contorno, a seta
/// aponta para a forma), cada uma no seu tamanho encolhido pelo trecho junto dela (XA2,
/// [`heads::size`]), e a linha recuada o que a ponta tapa.
#[must_use]
pub fn drawn(r: &Routed, heads: [Head; 2], width: f64) -> Drawn {
    let sizes = [0, 1].map(|k| heads::size(heads[k], r.legs[k]));
    drawn_sized(r, heads, width, sizes)
}

/// [`drawn`] com a escala FIXA `scale` do catálogo (os ÍCONES da barra: a ponta do quadro, em 16 px,
/// não cabia no botão — foto da cena 3, 06/10).
#[must_use]
pub fn drawn_at(r: &Routed, heads: [Head; 2], width: f64, scale: f64) -> Drawn {
    let sizes = heads.map(|h| heads::size_for_scale(h, scale, width));
    drawn_sized(r, heads, width, sizes)
}

fn drawn_sized(r: &Routed, heads: [Head; 2], width: f64, sizes: [f64; 2]) -> Drawn {
    use ph2d_vector::{ParamCurve, PathSeg};
    let full = ph2d_vec_render::build_bezpath(&r.path);
    let segs: Vec<PathSeg> = full.segments().collect();
    if segs.is_empty() {
        return Drawn {
            line: BezPath::new(),
            heads: Vec::new(),
        };
    }
    let cut = [0, 1].map(|k| chord_cut(&segs, k == 1, heads::inset(heads[k], sizes[k])));
    let mut heads_out = Vec::new();
    for (k, h) in heads.into_iter().enumerate() {
        if h == Head::None {
            continue;
        }
        let (tip, far) = if k == 0 {
            (segs[0].start(), segs[segs.len() - 1].end())
        } else {
            (segs[segs.len() - 1].end(), segs[0].start())
        };
        // Para onde a ponta APONTA de volta: o ponto da linha à distância que ela ocupa. Uma linha
        // inteira mais curta aponta para a outra ponta (⛔ antes saltava a ponta: a seta curta ficava
        // sem ela).
        let aim = chord_cut(&segs, k == 1, heads::along(h, sizes[k]))
            .map_or(far, |(i, t)| segs[i].eval(t));
        let dir = [tip.x - aim.x, tip.y - aim.y];
        heads_out.extend(heads::build(h, [tip.x, tip.y], dir, width, sizes[k]));
    }
    let line = match cut {
        [Some(a), Some(b)] if a.0 < b.0 || (a.0 == b.0 && a.1 < b.1) => sub_path(&segs, a, b),
        _ => BezPath::new(),
    };
    Drawn {
        line,
        heads: heads_out,
    }
}

/// ⭐ **Onde a curva fica a `r` (em linha recta) da sua ponta** — a ponta de início (`from_end =
/// false`) ou a de fim: `(segmento, t)`. A linha corta-se AÍ e a cabeça aponta da ponta para AÍ — a
/// linha acaba no centro da base dela e é a MESMA curva que a guia da selecção desenha. ⛔ Recuar a
/// âncora ao longo da tangente (a versão anterior) mudava a curva e separava o traço da guia (3.º
/// smoke do dono, 06/10); o `trim_path` do vectorial recua pela corda das âncoras e entortava-a.
/// `None` = a curva inteira fica mais perto da ponta do que `r`.
fn chord_cut(segs: &[ph2d_vector::PathSeg], from_end: bool, r: f64) -> Option<(usize, f64)> {
    use ph2d_vector::ParamCurve;
    let n = segs.len();
    let tip = if from_end {
        segs[n - 1].end()
    } else {
        segs[0].start()
    };
    if r <= 0.0 {
        return Some(if from_end { (n - 1, 1.0) } else { (0, 0.0) });
    }
    let order: Vec<usize> = if from_end {
        (0..n).rev().collect()
    } else {
        (0..n).collect()
    };
    for i in order {
        let s = &segs[i];
        let far = if from_end { s.start() } else { s.end() };
        if (far - tip).hypot() < r {
            continue;
        }
        // Bissecção: de perto da ponta (`near`) a longe (`far`).
        let (mut lo, mut hi) = if from_end { (1.0, 0.0) } else { (0.0, 1.0) };
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if (s.eval(mid) - tip).hypot() < r {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        return Some((i, hi));
    }
    None
}

/// O pedaço da curva de `a` a `b` (`(segmento, t)`), cortando os segmentos das pontas.
fn sub_path(segs: &[ph2d_vector::PathSeg], a: (usize, f64), b: (usize, f64)) -> BezPath {
    use ph2d_vector::ParamCurve;
    let mut out = BezPath::new();
    for (i, seg) in segs.iter().enumerate().take(b.0 + 1).skip(a.0) {
        let t0 = if i == a.0 { a.1 } else { 0.0 };
        let t1 = if i == b.0 { b.1 } else { 1.0 };
        let piece = seg.subsegment(t0..t1);
        if out.elements().is_empty() {
            out.move_to(piece.start());
        }
        out.push(piece.as_path_el());
    }
    out
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
/// largada sobre `el` em `p` prende-se ao CENTRO quando cai no miolo, e em ÓRBITA de um ponto do
/// contorno quando cai na faixa de largura `band` (mundo) junto a ele: por dentro o ponto do contorno
/// mais perto, por fora o próprio CURSOR (o `fixedPoint` do Excalidraw de hoje, W23 A10) — colado ao
/// meio de um lado quando passa a menos de `band` dele (o encaixe do Excalidraw, XB5). As duas param a [`gap`] da forma (W9, decisão do dono 10/10: o quadro
/// inteiro adopta o vão). `None` = `p` está fora da forma e da faixa.
#[must_use]
pub fn anchor_for(el: &Element, p: [f64; 2], band: f64) -> Option<Anchor> {
    let (q, d) = nearest_on_outline(el, p)?;
    let within = inside(el, p);
    // Por fora a faixa conta também o vão: a ponta presa está LÁ, e agarrá-la (com zoom, a faixa em
    // px fica mais fina que o vão) não a pode soltar (auditoria W9).
    if !within && d > band + gap(el) {
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
    let focus = if within { q } else { p };
    Some(Anchor::Orbit(
        mid.map_or_else(|| normalized(el, focus), |m| m.0),
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

    /// O FOCO da ponta (XB1): o ponto dela (solta), o ponto fixo ou em órbita, o centro.
    fn focus(&self) -> [f64; 2] {
        match *self {
            Resolved::Point(p) => p,
            Resolved::Shape(el, Anchor::Fixed(uv) | Anchor::Orbit(uv)) => fixed_world(el, uv),
            Resolved::Shape(..) => self.center(),
        }
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
        Resolved::Shape(el, Anchor::Center) => center_exit(el, other, kind, prev, spread),
        Resolved::Shape(el, Anchor::Orbit(uv)) => {
            let focus = fixed_world(el, uv);
            let g = gap(el);
            if kind == RouteKind::Straight {
                // XB1: na recta da outra estação ao foco, onde ela entra na faixa.
                let p = band_entry(el, other, focus, g);
                return (p, port_side(aabb(el), p, prev));
            }
            // A curva do Miro e o cotovelo saem PERPENDICULARES: do ponto do contorno (o foco que lá
            // está), ou pela face que olha para a outra ponta (um foco no miolo), com o mesmo vão.
            let tol = ON_OUTLINE * (1.0 + el.w.max(el.h));
            match nearest_on_outline(el, focus) {
                Some((q, d)) if d <= tol => {
                    let side = port_side(aabb(el), q, prev);
                    let n = ph2d_board_geom::outline_normal(el, q).unwrap_or(side.vec());
                    ([q[0] + n[0] * g, q[1] + n[1] * g], side)
                }
                _ => center_exit(el, other, kind, prev, spread),
            }
        }
    }
}

/// ⭐ **O vão de uma ponta presa** (W9, XB1): [`ORBIT_GAP`] mais meio traço da forma — 5,5 · 6 · 7
/// para a forma de traço 1 · 2 · 4, seja qual for o traço da seta (`B_seta_sw*_forma_sw*`).
#[must_use]
pub fn gap(el: &Element) -> f64 {
    ORBIT_GAP + el.style().stroke_width.max(0.0) / 2.0
}

/// A ponta presa ao CENTRO (a semântica do Miro: a face que olha para a outra ponta), a [`gap`] da
/// forma — decisão do dono (10/10): o quadro inteiro pára antes da forma, como o Excalidraw.
fn center_exit(
    el: &Element,
    other: [f64; 2],
    kind: RouteKind,
    prev: Option<Dir>,
    spread: f64,
) -> ([f64; 2], Dir) {
    let g = gap(el);
    exit_point(aabb(el), true, other, kind, prev, spread, |from, ray| {
        ray_exit_band(el, from, ray, g)
    })
}

/// ⭐ **A rota de uma seta** entre duas pontas resolvidas, pelos `waypoints` do artista, com a
/// histerese `prev` e o afastamento `spread` das paralelas. Só as duas formas da seta contam (o
/// cotovelo contorna-as); nenhuma outra.
pub(crate) fn compute(
    ends: [&Resolved<'_>; 2],
    kind: Route,
    waypoints: &[[f64; 2]],
    mids: &[Option<f64>],
    prev: [Option<Dir>; 2],
    spread: f64,
    heads: [Head; 2],
) -> Routed {
    let rk = match kind {
        Route::Straight | Route::Spline => RouteKind::Straight,
        Route::Elbow | Route::Curved => RouteKind::Orthogonal,
    };
    // Cada ponta sai rumo à estação VIZINHA: o 1.º ponto de ajuste, ou a outra ponta — o FOCO dela numa
    // rota recta (XB1: a recta entre os dois focos), o centro nas que escolhem a face (o Miro).
    let other = |e: &Resolved<'_>| {
        if rk == RouteKind::Straight {
            e.focus()
        } else {
            e.center()
        }
    };
    let toward0 = waypoints.first().copied().unwrap_or(other(ends[1]));
    let toward1 = waypoints.last().copied().unwrap_or(other(ends[0]));
    let (p0, d0) = exit(ends[0], toward0, rk, prev[0], spread);
    let (p1, d1) = exit(ends[1], toward1, rk, prev[1], -spread);
    let mut stations = vec![p0];
    stations.extend_from_slice(waypoints);
    stations.push(p1);
    match kind {
        Route::Curved => {
            // A curva encaixa na NORMAL do contorno (num canto vivo, na direcção do lado).
            let normal = |e: &Resolved<'_>, p: [f64; 2], side: Dir| match *e {
                Resolved::Shape(el, _) => {
                    ph2d_board_geom::outline_normal(el, p).unwrap_or(side.vec())
                }
                Resolved::Point(_) => side.vec(),
            };
            let d = [normal(ends[0], p0, d0), normal(ends[1], p1, d1)];
            let stems = stems(heads, chords(&stations));
            return Routed::curved_with_stems(stations, [d0, d1], d, stems);
        }
        Route::Straight | Route::Spline => {
            return Routed::from_points(stations, kind, [d0, d1]);
        }
        Route::Elbow => {}
    }
    // O cotovelo: cada trecho entre estações pelo roteador; por obstáculo, só a forma da PONTA em
    // que o trecho encosta (6.º smoke do dono, 06/10: contornar a outra fazia voltas que saltavam).
    // Um ponto de ajuste é uma ponta solta no meio (sai rumo à estação seguinte).
    let wall = |e: &Resolved<'_>| matches!(e, Resolved::Shape(..)).then(|| e.bbox());
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
    let mut elbows = Vec::with_capacity(last);
    for i in 0..last {
        let (a, b) = (stations[i], stations[i + 1]);
        let da = if i == 0 { d0 } else { toward(a, b) };
        let db = if i + 1 == last { d1 } else { toward(b, a) };
        let walls: Vec<Aabb> = [
            (i == 0).then_some(ends[0]),
            (i + 1 == last).then_some(ends[1]),
        ]
        .into_iter()
        .flatten()
        .filter_map(wall)
        .collect();
        let jetty = facing_jetty(a, da, b, db);
        let mut leg = route(&RouteInput {
            start: EndSpec { at: a, dir: da },
            end: EndSpec { at: b, dir: db },
            kind: RouteKind::Orthogonal,
            jetty,
            obstacles: &walls,
            spread,
            self_loop,
        });
        let frac = mids.get(i).copied().flatten();
        elbows.push(elbow_mid::place(&mut leg, da, db, jetty, frac));
        pts.extend(leg.into_iter().skip(1));
        breaks.push(pts.len() - 1);
    }
    let verts = pts.iter().map(|p| VecVertex::corner(*p)).collect();
    let legs = chords(&pts);
    let mut routed = Routed::assemble(stations, verts, &breaks, [d0, d1], legs);
    routed.elbows = elbows;
    routed
}

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
        End::Bound { target, anchor } => match doc
            .get(target)
            .filter(|el| ph2d_board_geom::shape_of(el).is_some())
        {
            Some(el) => Resolved::Shape(el, anchor),
            None => Resolved::Point(fallback.unwrap_or([0.0, 0.0])),
        },
    }
}

#[cfg(test)]
mod caption_tests;
#[cfg(test)]
mod jitter_probe;
#[cfg(test)]
mod oracle_tests;
#[cfg(test)]
mod tests;
