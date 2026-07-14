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
//! Então o alvo aqui é o honesto: **bom automático + escape manual óbvio** ([`BlendOpts`]).
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

use kurbo::{CubicBez, ParamCurve, ParamCurveArclen, Point};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecVertex, VertexKind};

/// Precisão do comprimento de arco.
///
/// Ela decide **onde** a peça é cortada — não a geometria dela (subdividir uma cúbica é exato,
/// de Casteljau). Ainda assim é apertada, e por um motivo medido: a 1e-6, `t=0` devolvia A com
/// um desvio de **2,2e-6** — pequeno, mas é **viés sistemático**, não ruído, e um viés desses
/// vira "a forma tremeu" quando o `t` é animado quadro a quadro.
const ARCLEN_EPS: f64 = 1e-11;

/// Duas posições de arco mais próximas que isto são a MESMA — cortar entre elas produziria uma
/// peça de comprimento zero (e um vértice duplicado na saída).
const MERGE_EPS: f64 = 1e-9;

/// Quantas amostras a busca de correspondência usa para medir o custo de um candidato.
const COST_SAMPLES: usize = 64;

/// O **escape manual** — o `shapeIndex` do GSAP, o *Map Nodes* do Corel.
///
/// O automático acerta a maioria das vezes e erra em algumas; quando erra, a forma "gira" ou
/// "vira do avesso" no meio do caminho. Estes dois campos são a saída, e existem porque **toda**
/// ferramenta séria do mercado teve de ter uma.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BlendOpts {
    /// Roda a correspondência em `offset` âncoras de B. `0` = o que o automático escolheu.
    pub offset: i32,
    /// Inverte o sentido de percurso de B. É o que conserta a forma que sai do avesso.
    pub reverse: bool,
}

/// O contorno de uma forma como cadeia de cúbicas, com o comprimento de arco acumulado.
struct Outline {
    segs: Vec<CubicBez>,
    /// `cum[i]` = arco até o INÍCIO de `segs[i]`; `cum[n]` = o total.
    cum: Vec<f64>,
    total: f64,
    closed: bool,
}

impl Outline {
    /// A partir do contorno externo da geometria **COZIDA** (é ela que está na tela — mesma
    /// escolha da booleana: o raio de quina vivo já entra assado).
    fn of(path: &VecPath) -> Option<Outline> {
        let cooked = path.cooked();
        let verts = &cooked.verts;
        if verts.len() < 2 {
            return None;
        }
        let n = verts.len();
        let last = if cooked.closed { n } else { n - 1 };
        let mut segs = Vec::with_capacity(last);
        for i in 0..last {
            let a = &verts[i];
            let b = &verts[(i + 1) % n];
            segs.push(CubicBez::new(
                pt(a.anchor),
                pt(a.out_handle),
                pt(b.in_handle),
                pt(b.anchor),
            ));
        }
        let mut cum = Vec::with_capacity(segs.len() + 1);
        let mut total = 0.0;
        for s in &segs {
            cum.push(total);
            total += s.arclen(ARCLEN_EPS);
        }
        cum.push(total);
        if total <= MERGE_EPS {
            return None; // forma degenerada: não há o que percorrer
        }
        Some(Outline {
            segs,
            cum,
            total,
            closed: cooked.closed,
        })
    }

    /// As posições (arco normalizado, em [0,1)) das âncoras ORIGINAIS.
    fn anchors(&self) -> Vec<f64> {
        self.cum[..self.segs.len()]
            .iter()
            .map(|c| c / self.total)
            .collect()
    }

    /// O ponto no arco normalizado `s`.
    fn at(&self, s: f64) -> Point {
        let (i, t) = self.locate(s);
        self.segs[i].eval(t)
    }

    /// (índice do segmento, `t` local) do arco normalizado `s`.
    fn locate(&self, s: f64) -> (usize, f64) {
        let arc = s.clamp(0.0, 1.0) * self.total;
        // O último segmento absorve o fim exato (`s == 1`).
        let i = match self.cum[1..].partition_point(|&c| c <= arc) {
            i if i >= self.segs.len() => self.segs.len() - 1,
            i => i,
        };
        let local = (arc - self.cum[i]).max(0.0);
        let seg = &self.segs[i];
        // O comprimento já está no acumulado — recalculá-lo aqui seria um `arclen` por consulta.
        let len = self.cum[i + 1] - self.cum[i];
        let t = if len <= MERGE_EPS {
            0.0
        } else {
            seg.inv_arclen(local.min(len), ARCLEN_EPS)
        };
        (i, t.clamp(0.0, 1.0))
    }

    /// A cadeia de cúbicas que sai de cortar este contorno nas posições `cuts`.
    ///
    /// **O percurso é CÍCLICO, e isso não é detalhe.** As posições de B saem de `wrap(s − fase)`,
    /// então elas são uma **rotação** da ordem crescente, não a ordem crescente: exatamente uma
    /// peça atravessa a origem do contorno. (Nunca *mais* de uma, e nunca uma âncora: a origem de
    /// B **é** uma âncora de B, e toda âncora de B está em `cuts` — é o que garante que cada peça
    /// caiba num único segmento e seja uma sub-cúbica EXATA.)
    ///
    /// Devolve **uma peça por corte** — sempre. É o que faz A e B saírem pareados 1-a-1.
    fn cut(&self, cuts: &[f64]) -> Vec<CubicBez> {
        let m = cuts.len();
        let mut out = Vec::with_capacity(m);
        for k in 0..m {
            let s0 = cuts[k];
            // O corte seguinte, ciclicamente. Se ele "voltou" (é a peça que fecha o contorno), o
            // fim dela é o fim do percurso.
            let next = cuts[(k + 1) % m];
            let s1 = if next <= s0 + MERGE_EPS { 1.0 } else { next };
            let (i0, t0) = self.locate(s0);
            let (i1, t1) = self.locate(s1);
            // O corte do fim pode ter caído no COMEÇO do segmento seguinte (t≈0): a peça é do
            // segmento de `s0`, até o fim DELE.
            let t1 = if i1 == i0 { t1 } else { 1.0 };
            if t1 - t0 <= MERGE_EPS {
                out.push(self.segs[i0].subsegment(t0..t0)); // peça degenerada: preserva o PAREAMENTO
                continue;
            }
            out.push(self.segs[i0].subsegment(t0..t1));
        }
        out
    }

    /// O contorno percorrido ao CONTRÁRIO (a saída do "forma vira do avesso").
    fn reversed(&self) -> Outline {
        let segs: Vec<CubicBez> = self
            .segs
            .iter()
            .rev()
            .map(|s| CubicBez::new(s.p3, s.p2, s.p1, s.p0))
            .collect();
        let mut cum = Vec::with_capacity(segs.len() + 1);
        let mut total = 0.0;
        for s in &segs {
            cum.push(total);
            total += s.arclen(ARCLEN_EPS);
        }
        cum.push(total);
        Outline {
            segs,
            cum,
            total,
            closed: self.closed,
        }
    }
}

/// A correspondência escolhida: em que fase o contorno de B é lido, e em que sentido.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Correspondence {
    /// O arco normalizado de A **casa** com `(arco de A) − phase` em B.
    pub phase: f64,
    pub reversed: bool,
}

/// Acha a correspondência que **minimiza a distância total percorrida** — o critério do flubber,
/// e o único que se sustenta sem perguntar nada ao usuário.
///
/// Os candidatos NÃO são "todos os deslocamentos de uma reamostragem" (o O(n²) do flubber sobre
/// pontos que ele mesmo inventou): são os **alinhamentos âncora-com-âncora** (`n_a × n_b`), nos
/// dois sentidos. É o mesmo custo e casa quina com quina, que é onde o olho olha.
///
/// Num caminho ABERTO não há liberdade de rotação (as pontas são as pontas): só o sentido.
#[must_use]
pub fn correspondence(a: &VecPath, b: &VecPath) -> Option<Correspondence> {
    let (oa, ob) = (Outline::of(a)?, Outline::of(b)?);
    Some(search(&oa, &ob))
}

fn search(oa: &Outline, ob: &Outline) -> Correspondence {
    let closed = oa.closed && ob.closed;
    let ob_rev = ob.reversed();
    let mut best = Correspondence {
        phase: 0.0,
        reversed: false,
    };
    let mut best_cost = f64::INFINITY;
    for reversed in [false, true] {
        let target = if reversed { &ob_rev } else { ob };
        let phases: Vec<f64> = if closed {
            let (pa, pb) = (oa.anchors(), target.anchors());
            pa.iter()
                .flat_map(|sa| pb.iter().map(move |sb| wrap(sa - sb)))
                .collect()
        } else {
            vec![0.0] // caminho aberto: a ponta é a ponta
        };
        for phase in phases {
            let cost = travel(oa, target, phase);
            if cost < best_cost {
                best_cost = cost;
                best = Correspondence { phase, reversed };
            }
        }
    }
    best
}

/// A distância total que os pontos percorrem sob esta correspondência (soma dos quadrados —
/// ela pune o ponto que atravessa a forma inteira, que é exatamente o que faz o morph "girar").
fn travel(oa: &Outline, ob: &Outline, phase: f64) -> f64 {
    (0..COST_SAMPLES)
        .map(|k| {
            let u = k as f64 / COST_SAMPLES as f64;
            let (pa, pb) = (oa.at(u), ob.at(wrap(u - phase)));
            (pa - pb).hypot2()
        })
        .sum()
}

#[inline]
fn wrap(s: f64) -> f64 {
    let s = s % 1.0;
    if s < 0.0 { s + 1.0 } else { s }
}

/// **A forma no meio do caminho.** `t = 0` devolve A; `t = 1`, B.
///
/// `None` se qualquer uma das duas degenerar (menos de 2 vértices, ou comprimento nulo).
#[must_use]
pub fn morph(a: &VecPath, b: &VecPath, t: f64, opts: BlendOpts) -> Option<VecPath> {
    let (oa, ob) = (Outline::of(a)?, Outline::of(b)?);
    let auto = search(&oa, &ob);
    let reversed = auto.reversed != opts.reverse; // XOR: o toggle do usuário sobre o automático
    let target = if reversed { ob.reversed() } else { ob };

    // O escape manual: rodar `offset` âncoras de B é somar à fase a distância de arco entre a
    // âncora 0 e a âncora `offset` — é o `shapeIndex` do GSAP, com o automático como base.
    let anchors_b = target.anchors();
    let phase = if oa.closed && target.closed && !anchors_b.is_empty() {
        let n = anchors_b.len() as i32;
        let k = opts.offset.rem_euclid(n) as usize;
        wrap(auto.phase + anchors_b[k] - anchors_b[0])
    } else {
        auto.phase
    };

    // O corte é na UNIÃO: as âncoras de A, mais as de B trazidas para o arco de A. É esta união
    // que faz as quinas das DUAS formas sobreviverem — reamostrar uniformemente (o flubber)
    // arredonda a quina de quem tem quina.
    let mut cuts: Vec<f64> = oa.anchors();
    cuts.extend(anchors_b.iter().map(|sb| wrap(sb + phase)));
    cuts.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    cuts.dedup_by(|x, y| (*x - *y).abs() <= MERGE_EPS);
    // O fim do percurso É o começo dele (fechado): um corte em ~1 é o corte em 0, e mantê-lo
    // criaria uma peça de comprimento zero na costura.
    if cuts.last().is_some_and(|&s| s >= 1.0 - MERGE_EPS) {
        cuts.pop();
    }
    // O percurso sempre começa na ponta/origem.
    if cuts.first().is_none_or(|&s| s > MERGE_EPS) {
        cuts.insert(0, 0.0);
    }

    let pieces_a = oa.cut(&cuts);
    let cuts_b: Vec<f64> = cuts.iter().map(|s| wrap(s - phase)).collect();
    let pieces_b = target.cut(&cuts_b);
    if pieces_a.is_empty() || pieces_a.len() != pieces_b.len() {
        return None;
    }

    let t = t.clamp(0.0, 1.0);
    let lerped: Vec<CubicBez> = pieces_a
        .iter()
        .zip(&pieces_b)
        .map(|(ca, cb)| {
            CubicBez::new(
                mix(ca.p0, cb.p0, t),
                mix(ca.p1, cb.p1, t),
                mix(ca.p2, cb.p2, t),
                mix(ca.p3, cb.p3, t),
            )
        })
        .collect();

    let mut out = path_from(&lerped, oa.closed);
    out.fill = mix_paint(a.fill.as_ref(), b.fill.as_ref(), t);
    out.stroke = if t < 0.5 { a.stroke } else { b.stroke };
    Some(out)
}

/// Os **N passos intermediários** entre A e B — o Blend do Illustrator.
///
/// Só os do MEIO: as duas pontas são as formas que o artista já tem. `steps = 3` devolve as
/// formas em `t = ¼, ½, ¾`.
#[must_use]
pub fn steps(a: &VecPath, b: &VecPath, n: usize, opts: BlendOpts) -> Vec<VecPath> {
    (1..=n)
        .filter_map(|i| morph(a, b, i as f64 / (n + 1) as f64, opts))
        .collect()
}

/// Cadeia de cúbicas → `VecPath`. A âncora é o `p0` de cada peça; a alça de saída é o `p1` dela,
/// e a de entrada é o `p2` da peça ANTERIOR (é o mesmo ponto do documento, visto pelos dois
/// lados).
fn path_from(segs: &[CubicBez], closed: bool) -> VecPath {
    let n = segs.len();
    let mut verts: Vec<VecVertex> = Vec::with_capacity(n + 1);
    for (i, s) in segs.iter().enumerate() {
        let prev = if i == 0 {
            if closed { segs[n - 1] } else { *s }
        } else {
            segs[i - 1]
        };
        let in_handle = if i == 0 && !closed { s.p0 } else { prev.p2 };
        verts.push(VecVertex {
            anchor: xy(s.p0),
            in_handle: xy(in_handle),
            out_handle: xy(s.p1),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        });
    }
    if !closed && let Some(last) = segs.last() {
        verts.push(VecVertex {
            anchor: xy(last.p3),
            in_handle: xy(last.p2),
            out_handle: xy(last.p3),
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        });
    }
    VecPath {
        verts,
        closed,
        ..VecPath::default()
    }
}

/// O Illustrator interpola a COR junto com a forma, e é isso que faz um blend parecer um blend
/// em vez de N cópias. Só entre dois sólidos; qualquer outra coisa (gradiente, nada) fica com o
/// lado mais próximo.
fn mix_paint(a: Option<&Paint>, b: Option<&Paint>, t: f64) -> Option<Paint> {
    match (a, b) {
        (Some(Paint::Solid(ca)), Some(Paint::Solid(cb))) => Some(Paint::solid(Rgba8::new(
            mix_u8(ca.r, cb.r, t),
            mix_u8(ca.g, cb.g, t),
            mix_u8(ca.b, cb.b, t),
            mix_u8(ca.a, cb.a, t),
        ))),
        _ if t < 0.5 => a.cloned(),
        _ => b.cloned(),
    }
}

#[inline]
fn mix_u8(a: u8, b: u8, t: f64) -> u8 {
    (f64::from(a) + (f64::from(b) - f64::from(a)) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[inline]
fn mix(a: Point, b: Point, t: f64) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

#[inline]
fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[inline]
fn xy(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
