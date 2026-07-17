#![forbid(unsafe_code)]
//! ph2d-vec-blend — **interpolação de formas** (Morph / Blend).
//!
//! O problema difícil **não é interpolar**: é a **correspondência**. Dadas duas formas com
//! topologias diferentes, *qual ponto de A vira qual ponto de B?* A pesquisa
//! (`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md` §1.3) foi categórica: **ninguém
//! resolveu isso.**
//!
//! | Quem | Como resolve |
//! |---|---|
//! | **flubber** (a lib mais usada da web) | reamostra por arco, acolchoa, e faz **força bruta** sobre os deslocamentos. O autor: *"uma heurística cuja justificativa é que geralmente funciona bem"* |
//! | **GSAP MorphSVG** | heurísticas + **`shapeIndex` MANUAL** + uma ferramenta de debug cuja existência **admite que o automático erra** |
//! | **CorelDRAW Blend** | *"Map Nodes"*: o usuário **clica um nó em cada forma** |
//! | **Lottie / AE** · **Rive** | **nenhuma** — lerp por índice, exige a mesma contagem de pontos |
//!
//! O alvo aqui é o **melhor automático possível** — não há escape manual (ver a nota no corpo).
//!
//! # As duas decisões que separam este motor do flubber
//!
//! 1. **O corte é na UNIÃO das posições de âncora das duas formas** (em comprimento de arco
//!    normalizado), não numa reamostragem uniforme. Reamostrar uniformemente **arredonda as
//!    quinas** de quem tem quina — o quadrado vira um saco. Cortando na união, **as duas** formas
//!    mantêm cada uma das suas quinas, e ainda assim saem com a mesma contagem de peças, pareadas
//!    1-a-1. O preço é `n_a + n_b` âncoras na interpolada, e é um preço justo.
//! 2. **Cada peça é uma CÚBICA, não um segmento de reta.** Como a união inclui as âncoras
//!    originais, nenhuma peça atravessa uma âncora — então ela é sempre uma sub-cúbica exata de
//!    um segmento original (`subsegment`). A curva não é achatada em polilinha em lugar nenhum:
//!    o que sai é geometria de Bézier de verdade, editável.
//!
//! # O que ele NÃO faz (e a literatura sabe)
//!
//! É o **lerp de coordenadas**. Ele encolhe a forma no meio do caminho e pode auto-intersectar
//! numa rotação grande — é por isso que o GSAP tem um modo "rotational". O estado da arte é
//! Sederberg & Greenwood 1992 (deformação de trabalho mínimo) e Alexa 2000 (*as-rigid-as-
//! possible*). Ficam para depois, atrás do motor: a correspondência é o pré-requisito dos dois.

use kurbo::{CubicBez, Point};
use ph2d_vec_scene::{Contour, FillRule, Paint, Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Precisão do comprimento de arco.
///
/// Ela decide **onde** a peça é cortada — não a geometria dela (subdividir uma cúbica é exato,
/// de Casteljau). Ainda assim é apertada, e por um motivo medido: a 1e-6, `t=0` devolvia A com
/// um desvio de **2,2e-6** — pequeno, mas é **viés sistemático**, não ruído, e um viés desses
/// vira "a forma tremeu" quando o `t` é animado quadro a quadro.
pub(crate) const ARCLEN_EPS: f64 = 1e-11;

/// Duas posições de arco mais próximas que isto são a MESMA — cortar entre elas produziria uma
/// peça de comprimento zero (e um vértice duplicado na saída).
pub(crate) const MERGE_EPS: f64 = 1e-9;

/// Quantas amostras a busca de correspondência usa para medir o custo de um candidato.
pub(crate) const COST_SAMPLES: usize = 64;

// A correspondência é 100% AUTOMÁTICA — não há escape manual.
//
// Houve um (`BlendOpts { offset, reverse }`, o "Rotate Match" / "Reverse Match" do painel), e os
// dois foram REMOVIDOS por serem bugs de design: o **Reverse** invertia o winding e colapsava a
// forma; o **Rotate** rodava a correspondência às cegas e produzia torção. No modelo vivo (o Blend
// do Illustrator), o ajuste é feito **editando as formas-fonte** — girar a do meio adapta os
// intermediários dos dois lados —, não um botão que gira a correspondência sem o artista ver.

/// O **compound path** (qual contorno de A vira qual contorno de B) — a camada ACIMA da
/// correspondência, num módulo irmão. É ela que faz o buraco da rosquinha sobreviver ao morph.
/// O **contorno** como cadeia de cúbicas com arco acumulado — o primitivo do motor, num módulo
/// irmão pelo teto de LOC.
mod outline;
use outline::{COINCIDENT_EPS, Outline};

mod compound;
use compound::Ring;

/// A **correspondência** (o mapa monótono entre as duas formas) — módulo irmão, pelo teto de LOC.
mod matching;
use matching::{map_backward, map_forward, search};

/// O **flow dos passos ao longo do SPINE editável** (ADR-0128) — a camada nova sobre o motor. É
/// geometria de arco pura (deslocamentos), separada da correspondência.
pub mod spine;
pub use spine::spine_offsets;

/// **O PLANO de um blend**: a correspondência já resolvida e as duas formas já cortadas, peça a
/// peça. Avaliar um `t` é, a partir daqui, só um lerp — `O(peças)`, sem busca nenhuma.
///
/// # Por que ele existe
///
/// A **correspondência é função só do par (A, B, opções)** — não do `t`. Enquanto ela era buscada
/// dentro do `morph`, um blend de 10 passos a buscava **10 vezes**, e chegava dez vezes à mesma
/// resposta. Depois que a busca de fase entrou (ela varre 256 fases contra 256 amostras), essas
/// nove buscas jogadas fora passaram a custar **5,9 ms** por blend, contra 0,18 ms do caminho da
/// DP — e o artista re-roda o blend a cada toque em Steps / Rotate.
///
/// E é a **mesma** estrutura que o **morph vivo** (o `t` animável, o próximo da fila) precisa: o
/// plano é montado quando a relação muda, e cada frame só avalia um `t`.
/// # Um plano por CONTORNO
///
/// Uma forma pode ter mais de um contorno (a rosquinha: o de fora e o buraco), e a correspondência
/// é resolvida **dentro de cada par de contornos** — o nível-2 (fase/quinas/virada) não sabe que
/// existe um buraco. Quem decide *qual* contorno de A vira *qual* de B é [`compound`].
pub struct Plan {
    /// Um por par de contornos. `links[0]` vira o contorno PRIMÁRIO do passo.
    links: Vec<Link>,
    fill_rule: FillRule,
    fill: (Option<Paint>, Option<Paint>),
    stroke: (Option<StrokeSpec>, Option<StrokeSpec>),
}

/// Um par de contornos já cortado na união, peça a peça.
struct Link {
    pieces_a: Vec<CubicBez>,
    pieces_b: Vec<CubicBez>,
    closed: bool,
}

impl Plan {
    /// Resolve a correspondência entre A e B e corta as duas na união.
    ///
    /// `None` se qualquer uma das duas degenerar (menos de 2 vértices, ou comprimento nulo).
    #[must_use]
    pub fn new(a: &VecPath, b: &VecPath) -> Option<Plan> {
        let (ra, rb) = (compound::rings(a), compound::rings(b));
        let links: Vec<Link> = compound::pair(&ra, &rb)
            .into_iter()
            .filter_map(|p| link_of(&ra, &rb, p))
            .collect();
        if links.is_empty() {
            return None;
        }
        Some(Plan {
            fill_rule: compound::fill_rule_for(links.len()),
            links,
            fill: (a.fill.clone(), b.fill.clone()),
            stroke: (a.stroke, b.stroke),
        })
    }

    /// **A forma no meio do caminho.** `t = 0` devolve A; `t = 1`, B.
    ///
    /// Um `t` **NaN** vira `0` (devolve A), e não NaN: `f64::clamp` **propaga** NaN, e o morph vivo
    /// (§4 do handoff) vai chamar isto por frame com um `t` que sai de uma curva animada — uma
    /// singularidade nessa curva não pode virar uma forma de vértices NaN, que suja a cena e o save.
    #[must_use]
    pub fn at(&self, t: f64) -> VecPath {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let mut contours = self.links.iter().map(|l| l.at(t));
        // `links` nunca é vazio (o `new` recusa), então o primário existe.
        let (verts, closed) = contours.next().unwrap_or_default();
        let mut out = VecPath {
            verts,
            closed,
            subpaths: contours
                .map(|(v, c)| Contour {
                    verts: v,
                    closed: c,
                })
                .collect(),
            fill_rule: self.fill_rule,
            ..VecPath::default()
        };
        out.fill = mix_paint(self.fill.0.as_ref(), self.fill.1.as_ref(), t);
        out.stroke = mix_stroke(self.stroke.0, self.stroke.1, t);
        out
    }
}

impl Link {
    /// Os vértices deste contorno em `t`.
    fn at(&self, t: f64) -> (Vec<VecVertex>, bool) {
        let lerped: Vec<CubicBez> = self
            .pieces_a
            .iter()
            .zip(&self.pieces_b)
            .map(|(ca, cb)| {
                CubicBez::new(
                    mix(ca.p0, cb.p0, t),
                    mix(ca.p1, cb.p1, t),
                    mix(ca.p2, cb.p2, t),
                    mix(ca.p3, cb.p3, t),
                )
            })
            .collect();
        (verts_from(&lerped, self.closed), self.closed)
    }
}

/// O [`Link`] de um par de contornos: a correspondência resolvida e os dois cortados na união.
///
/// Um lado `None` é um contorno **sem par** — o buraco que nasce ou morre. Ele não tem
/// correspondência a resolver: o lado que existe é cortado nas próprias âncoras (as peças dele são
/// os próprios segmentos), e o lado que não existe é a MESMA contagem de peças **degeneradas**,
/// todas no ponto de colapso. O lerp faz o resto — o buraco cresce do ponto, ou encolhe até ele.
fn link_of(ra: &[Ring], rb: &[Ring], pair: (Option<usize>, Option<usize>)) -> Option<Link> {
    match pair {
        (Some(i), Some(j)) => {
            let (oa, ob) = (&ra[i].outline, &rb[j].outline);
            let corr = search(oa, ob);
            // O invertido, quando existe, precisa de um dono — e o não-invertido já tem um (`rb`).
            let flipped;
            let target: &Outline = if corr.reversed {
                flipped = ob.reversed();
                &flipped
            } else {
                ob
            };
            let (pieces_a, pieces_b) = pair_up(oa, target, &corr.knots)?;
            Some(Link {
                pieces_a,
                pieces_b,
                closed: oa.closed,
            })
        }
        (Some(i), None) => collapsed(&ra[i].outline, compound::collapse_point(rb)?, true),
        (None, Some(j)) => collapsed(&rb[j].outline, compound::collapse_point(ra)?, false),
        (None, None) => None,
    }
}

/// O link de um contorno **sem par**, contra um ponto. `live_is_a` diz de que lado o contorno está.
fn collapsed(o: &Outline, point: [f64; 2], live_is_a: bool) -> Option<Link> {
    let live = o.cut(&o.anchors());
    if live.is_empty() {
        return None;
    }
    let p = Point::new(point[0], point[1]);
    let dead = vec![CubicBez::new(p, p, p, p); live.len()];
    let (pieces_a, pieces_b) = if live_is_a {
        (live, dead)
    } else {
        (dead, live)
    };
    Some(Link {
        pieces_a,
        pieces_b,
        closed: o.closed,
    })
}

/// **A forma no meio do caminho.** `t = 0` devolve A; `t = 1`, B.
///
/// `None` se qualquer uma das duas degenerar (menos de 2 vértices, ou comprimento nulo).
///
/// Para mais de um `t` do MESMO par, monte um [`Plan`] — senão a correspondência é buscada de novo
/// a cada chamada, e ela não depende do `t`.
#[must_use]
pub fn morph(a: &VecPath, b: &VecPath, t: f64) -> Option<VecPath> {
    Some(Plan::new(a, b)?.at(t))
}

/// **O PAREAMENTO**: as duas formas cortadas na UNIÃO, peça a peça, na mesma ordem.
///
/// O corte é nas âncoras de A **mais a PRÉ-IMAGEM de cada âncora de B**. É a subdivisão que faz a
/// ponta da estrela nascer: cada vértice dela que não casou com uma quina do quadrado vira um ponto
/// novo na aresta do quadrado, no lugar exato que o mapa manda.
///
/// **Isto é uma função só, e de propósito.** Os gates precisam do mesmo pareamento que o produto
/// usa; enquanto eles o reconstruíam à mão, o espelho e o original podiam divergir — e divergiram
/// (o gate omitia a normalização da origem, e por sorte isso escondia um defeito em vez de inventar
/// um). Duas portas para a mesma pergunta divergem: [[feedback_two_doors_to_the_same_question_diverge]].
fn pair_up(
    oa: &Outline,
    target: &Outline,
    knots: &[(f64, f64)],
) -> Option<(Vec<CubicBez>, Vec<CubicBez>)> {
    let mut cuts: Vec<f64> = oa.anchors();
    cuts.extend(target.anchors().iter().map(|v| map_backward(knots, *v)));
    cuts.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    cuts.dedup_by(|x, y| (*x - *y).abs() <= MERGE_EPS);
    // O fim do percurso É o começo dele (fechado): um corte em ~1 é o corte em 0.
    if cuts.last().is_some_and(|&s| s >= 1.0 - MERGE_EPS) {
        cuts.pop();
    }
    if cuts.first().is_none_or(|&s| s > MERGE_EPS) {
        cuts.insert(0, 0.0);
    }

    let pieces_a = oa.cut(&cuts);
    let cuts_b: Vec<f64> = cuts.iter().map(|u| map_forward(knots, *u)).collect();
    let pieces_b = target.cut(&cuts_b);
    (!pieces_a.is_empty() && pieces_a.len() == pieces_b.len()).then_some((pieces_a, pieces_b))
}

/// Os **N passos intermediários** entre A e B — o Blend do Illustrator.
///
/// Só os do MEIO: as duas pontas são as formas que o artista já tem. `steps = 3` devolve as
/// formas em `t = ¼, ½, ¾`.
///
/// A correspondência é buscada **uma vez** ([`Plan`]) e avaliada em cada `t`: ela é função do par,
/// não do `t`.
#[must_use]
pub fn steps(a: &VecPath, b: &VecPath, n: usize) -> Vec<VecPath> {
    let Some(plan) = Plan::new(a, b) else {
        return Vec::new();
    };
    (1..=n)
        .map(|i| plan.at(i as f64 / (n + 1) as f64))
        .collect()
}

/// Os passos intermediários de uma **CADEIA** de formas — o Blend multi-forma do Illustrator.
///
/// Para N fontes (a linha aceita 2..=5), liga `fonte[0]→fonte[1]`, `fonte[1]→fonte[2]`, … — cada
/// par consecutivo é um [`Plan`] independente, com `n` passos entre eles, unidos na fonte
/// compartilhada. É a **cadeia pairwise** do Illustrator.
///
/// As **fontes NÃO entram no resultado** — elas se desenham sozinhas (são paths reais); aqui saem
/// só os intermediários **virtuais**, na ordem da cadeia. Um par que degenera (uma forma inválida)
/// é **pulado** — a cadeia não quebra por causa de um elo.
///
/// O spine é **emergente**: as fontes estão em posições diferentes do mundo, e o lerp de
/// coordenadas move os pontos, então os passos já se distribuem entre elas numa reta. O spine
/// editável (ADR-0128, Fase C) é uma re-distribuição por cima disto.
#[must_use]
pub fn chain(shapes: &[VecPath], n: usize) -> Vec<VecPath> {
    let mut out = Vec::new();
    for pair in shapes.windows(2) {
        if let Some(plan) = Plan::new(&pair[0], &pair[1]) {
            out.extend((1..=n).map(|i| plan.at(i as f64 / (n + 1) as f64)));
        }
    }
    out
}

/// Cadeia de cúbicas → os vértices de **UM** contorno. A âncora é o `p0` de cada peça; a alça de
/// saída é o `p1` dela, e a de entrada é o `p2` da peça ANTERIOR (é o mesmo ponto do documento,
/// visto pelos dois lados).
///
/// É o construtor de **todo** contorno de um passo — o primário e cada buraco. Um 2º construtor
/// para os buracos divergiria do primário.
///
/// **Uma peça reta sai com os handles COLAPSADOS na âncora** — que é como o documento escreve uma
/// reta (e o que o resto do repo testa com `is_line`). Sem isso, o blend entre dois polígonos
/// devolveria uma forma geometricamente reta mas com alças de curva penduradas no meio de cada
/// aresta: o artista abriria o modo Node e veria controles que ele nunca pediu, a booleana a
/// trataria como curva, e o `Simplify` teria trabalho à toa.
fn verts_from(segs: &[CubicBez], closed: bool) -> Vec<VecVertex> {
    let n = segs.len();
    let straight: Vec<bool> = segs.iter().map(is_straight).collect();
    let mut verts: Vec<VecVertex> = Vec::with_capacity(n + 1);
    for (i, s) in segs.iter().enumerate() {
        let prev_i = if i == 0 { n - 1 } else { i - 1 };
        // A alça de ENTRADA mora na âncora quando não há aresta chegando (ponta de caminho
        // aberto) ou quando a que chega é RETA — nos dois casos o documento não guarda controle.
        let in_handle = if (i == 0 && !closed) || straight[prev_i] {
            s.p0
        } else {
            segs[prev_i].p2
        };
        let out_handle = if straight[i] { s.p0 } else { s.p1 };
        verts.push(VecVertex {
            anchor: xy(s.p0),
            in_handle: xy(in_handle),
            out_handle: xy(out_handle),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        });
    }
    if !closed && let Some(last) = segs.last() {
        let in_handle = if straight[n - 1] { last.p3 } else { last.p2 };
        verts.push(VecVertex {
            anchor: xy(last.p3),
            in_handle: xy(in_handle),
            out_handle: xy(last.p3),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        });
    }
    verts
}

/// A cúbica é uma RETA? (os dois controles em cima da corda — que é o que o lerp de duas retas
/// canônicas devolve, por construção).
fn is_straight(s: &CubicBez) -> bool {
    let chord = s.p3 - s.p0;
    let len = chord.hypot();
    if len <= COINCIDENT_EPS {
        return true; // peça degenerada: não há direção para desviar
    }
    [s.p1, s.p2].iter().all(|c| {
        let v = *c - s.p0;
        (v.x * chord.y - v.y * chord.x).abs() / len <= STRAIGHT_EPS
    })
}

/// O quanto um controle pode se afastar da corda e a aresta ainda contar como reta. É uma
/// tolerância **de comprimento**, e ela é frouxa de propósito: o lerp é exato, mas os `f64` que
/// chegam aqui já passaram por `inv_arclen` e por um corte — o resíduo é da ordem de 1e-12.
const STRAIGHT_EPS: f64 = 1e-9;

/// O Illustrator interpola a COR junto com a forma, e é isso que faz um blend parecer um blend
/// em vez de N cópias. Só entre dois sólidos; qualquer outra coisa (gradiente, nada) fica com o
/// lado mais próximo.
///
/// # A cor caminha em OKLab, não em sRGB
///
/// É onde superamos o Illustrator: ele interpola em **device-space** (RGB/CMYK, canal a canal), e o
/// meio de dois matizes opostos passa por um **cinza lamacento**. Em OKLab o caminho é
/// **perceptual** — a luminosidade e os eixos oponentes interpolam sem o meio-tom sujo. Preferimos
/// OKLab (cartesiano) a OKLCH (polar): o polar preserva o matiz mas força escolher o SENTIDO da
/// volta do matiz, e para um blend o caminho reto do OKLab é o esperado.
fn mix_paint(a: Option<&Paint>, b: Option<&Paint>, t: f64) -> Option<Paint> {
    match (a, b) {
        (Some(Paint::Solid(ca)), Some(Paint::Solid(cb))) => {
            Some(Paint::solid(mix_oklab(*ca, *cb, t)))
        }
        _ if t < 0.5 => a.cloned(),
        _ => b.cloned(),
    }
}

/// O TRAÇO também interpola — como a cor e a forma, é o que faz o blend parecer uma transição e não
/// N cópias. A **largura** e a **cor** caminham (largura por lerp, cor em OKLab como o fill); o resto
/// (cap/join/dash/pontas) é discreto e vem do lado mais próximo.
///
/// **Um lado só com traço** (o outro sem): a largura some **suave** (fade a 0) em vez de aparecer/
/// sumir de repente no meio — uma forma traçada que blenda para uma sem traço vê o contorno afinar.
fn mix_stroke(a: Option<StrokeSpec>, b: Option<StrokeSpec>, t: f64) -> Option<StrokeSpec> {
    match (a, b) {
        (Some(sa), Some(sb)) => Some(StrokeSpec {
            color: mix_oklab(sa.color, sb.color, t),
            width: sa.width + (sb.width - sa.width) * t,
            // cap/join/dash/pontas são discretos — o lado mais próximo (o mesmo critério do fill).
            ..(if t < 0.5 { sa } else { sb })
        }),
        // Só um lado tem traço: a largura afina até 0 (fade), com a cor/estilo do lado que existe.
        (Some(sa), None) => Some(StrokeSpec {
            width: sa.width * (1.0 - t),
            ..sa
        }),
        (None, Some(sb)) => Some(StrokeSpec {
            width: sb.width * t,
            ..sb
        }),
        (None, None) => None,
    }
}

/// Interpola duas cores no espaço **OKLab** (perceptual). O ida-e-volta sRGB→linear→OKLab→…→sRGB
/// clampa+quantiza só na fronteira do display (`to_srgb`), como manda a `ph2d-color`.
#[inline]
fn mix_oklab(a: Rgba8, b: Rgba8, t: f64) -> Rgba8 {
    use ph2d_color::{OklabColor, SrgbRgba};
    let ok = |c: Rgba8| OklabColor::from_linear(SrgbRgba([c.r, c.g, c.b, c.a]).to_linear());
    #[allow(clippy::cast_possible_truncation)]
    let m = ok(a).lerp(ok(b), t as f32).to_linear().to_srgb().0;
    Rgba8::new(m[0], m[1], m[2], m[3])
}

#[inline]
fn mix(a: Point, b: Point, t: f64) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

#[inline]
pub(crate) fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[inline]
pub(crate) fn xy(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

/// Os gates da correspondência SUAVE (o giro do quadrado→círculo) — arquivo irmão pelo teto de LOC.
#[cfg(test)]
#[path = "tests_phase.rs"]
mod tests_phase;

/// Os gates da Fase A (cadeia multi-forma + cor OKLab) — arquivo irmão pelo teto de LOC.
#[cfg(test)]
#[path = "tests_chain.rs"]
mod tests_chain;

/// Os gates do compound path (o buraco da rosquinha) — arquivo irmão pelo teto de LOC.
#[cfg(test)]
#[path = "tests_compound.rs"]
mod tests_compound;
