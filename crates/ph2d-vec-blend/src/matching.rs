//! **A CORRESPONDÊNCIA** — que ponto de A vira que ponto de B.
//!
//! É o problema difícil (interpolar é o fácil), e o que separa este motor de uma rotação rígida.
//! Módulo irmão do [`super`] pelo teto de LOC; o doc do [`Correspondence`] explica a matéria.

use super::{COST_SAMPLES, MERGE_EPS, Outline, VecPath};
use crate::BlendOpts;
use kurbo::Point;

/// A correspondência escolhida: os **nós** que amarram as duas formas, e o sentido de percurso.
///
/// Cada nó é um par `(arco em A, arco em B)`, e entre dois nós o arco é mapeado **proporcional**.
/// Ou seja: a correspondência é um **mapa monótono por partes**, não uma rotação.
#[derive(Clone, Debug, PartialEq)]
pub struct Correspondence {
    /// Os nós `(u_a, u_b)`, em ordem crescente de `u_a`, cíclicos e monótonos.
    pub knots: Vec<(f64, f64)>,
    pub reversed: bool,
}

/// # Por que uma ROTAÇÃO não basta (o smoke do Enio)
///
/// A 1ª versão casava as duas formas por uma **rotação rígida**: `arco_B = arco_A − fase`. Ela
/// escolhe bem a fase, mas tem um teto **estrutural**, e o Enio bateu nele:
///
/// > O quadrado tem quina a cada **0,25** de perímetro; a estrela, vértice a cada **0,10**.
/// > Sob uma rotação, **no máximo UMA quina do quadrado cai sobre um vértice da estrela** — as
/// > outras três caem no MEIO de uma aresta dela.
///
/// O resultado: as pontas da estrela nascem do meio das arestas retas do quadrado, e o
/// intermediário sai amassado. Não é um bug de implementação — é o limite de uma rotação.
///
/// # O que este motor faz
///
/// Os **nós** são escolhidos por **programação dinâmica** (é o núcleo do Sederberg & Greenwood
/// 1992): um casamento **cíclico e monótono** entre as âncoras das duas formas, que minimiza
///
/// 1. a **distância entre as âncoras casadas**, cada forma normalizada (centro + escala) — é isto
///    que faz uma quina do quadrado procurar uma PONTA da estrela, e não um vale dela; e
/// 2. a **distorção de arco**: dois nós vizinhos devem cobrir frações de perímetro parecidas nas
///    duas formas, senão o mapa espreme metade de uma forma num canto da outra.
///
/// As âncoras que sobram (a estrela tem 10, o quadrado 4) **subdividem** a forma menor: a
/// pré-imagem de cada vértice não-casado da estrela vira um ponto novo na aresta do quadrado. É
/// exatamente a subdivisão que faltava.
#[must_use]
pub fn correspondence(a: &VecPath, b: &VecPath) -> Option<Correspondence> {
    let (oa, ob) = (Outline::of(a)?, Outline::of(b)?);
    Some(search(&oa, &ob, BlendOpts::default()))
}

/// Teto do custo da programação dinâmica (`n · m³`). Acima dele o casamento cai para a rotação
/// (um nó só) — uma forma com centenas de âncoras não tem "quinas" a casar, e o custo explodiria.
const DP_BUDGET: usize = 20_000_000;

pub(crate) fn search(oa: &Outline, ob: &Outline, opts: BlendOpts) -> Correspondence {
    let ob_rev = ob.reversed();
    let mut best: Option<(f64, Correspondence)> = None;
    for reversed in [false, true] {
        if reversed && !opts.reverse && best.is_some() && !oa.closed {
            break; // caminho aberto: o sentido de B ainda importa, mas não a rotação
        }
        let target = if reversed { &ob_rev } else { ob };
        let (cost, knots) = align(oa, target, opts.offset);
        if best.as_ref().is_none_or(|(c, _)| cost < *c) {
            best = Some((cost, Correspondence { knots, reversed }));
        }
    }
    // O toggle do usuário é a ÚLTIMA palavra sobre o sentido (o escape manual).
    let mut chosen = best.map(|(_, c)| c).unwrap_or(Correspondence {
        knots: vec![(0.0, 0.0)],
        reversed: false,
    });
    if opts.reverse {
        let target = if chosen.reversed { ob } else { &ob_rev };
        let (_, knots) = align(oa, target, opts.offset);
        chosen = Correspondence {
            knots,
            reversed: !chosen.reversed,
        };
    }
    chosen
}

/// O casamento cíclico monótono entre as âncoras de `oa` e `ob`, e o custo dele.
///
/// `offset` roda o nó inicial em B — é o **escape manual** (o `shapeIndex` do GSAP / *Map Nodes*
/// do Corel), agora com um significado exato: *"case a minha primeira âncora com a k-ésima dela"*.
pub(crate) fn align(oa: &Outline, ob: &Outline, offset: i32) -> (f64, Vec<(f64, f64)>) {
    let (ua, ub) = (oa.anchors(), ob.anchors());
    let (n, m) = (ua.len(), ub.len());
    if n == 0 || m == 0 {
        return (f64::INFINITY, vec![(0.0, 0.0)]);
    }
    // Caminho aberto: as pontas são as pontas, e o mapa é a identidade de arco.
    if !oa.closed || !ob.closed {
        return (travel_cost(oa, ob, &[(0.0, 0.0)]), vec![(0.0, 0.0)]);
    }
    // Fora do orçamento (ou uma forma com uma âncora só): cai na ROTAÇÃO — um nó, e o mapa vira
    // `arco_B = arco_A − fase`, que é exatamente o comportamento antigo.
    if n.min(m) < 2 || n.saturating_mul(m.pow(3)) > DP_BUDGET {
        return rotation_only(oa, ob, offset);
    }

    // As âncoras, NORMALIZADAS (centro + escala): o custo compara FORMA, não posição no mundo.
    let (pa, pb) = (normalized(oa), normalized(ob));
    // A forma com MENOS âncoras é a que casa todas as suas; as que sobram na outra subdividem.
    let swapped = n > m;
    let (ua, ub, pa, pb) = if swapped {
        (ub, ua, pb, pa)
    } else {
        (ua, ub, pa, pb)
    };
    let (n, m) = (ua.len(), ub.len());

    let mut best: Option<(f64, Vec<usize>)> = None;
    let start = offset.rem_euclid(i32::try_from(m).unwrap_or(1)) as usize;
    for c0 in 0..m {
        // O escape manual força o 1º nó; sem ele, todos os inícios são candidatos.
        let c = (c0 + start) % m;
        if offset != 0 && c0 != 0 {
            break;
        }
        if let Some((cost, js)) = dp_from(&ua, &ub, &pa, &pb, c)
            && best.as_ref().is_none_or(|(bc, _)| cost < *bc)
        {
            best = Some((cost, js));
        }
    }
    let Some((cost, js)) = best else {
        return rotation_only(oa, ob, offset);
    };
    let mut knots: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            if swapped {
                (ub[js[i]], ua[i])
            } else {
                (ua[i], ub[js[i]])
            }
        })
        .collect();
    knots.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
    (cost, knots)
}

/// A rotação de um nó só (o comportamento antigo) — o degradado seguro.
fn rotation_only(oa: &Outline, ob: &Outline, offset: i32) -> (f64, Vec<(f64, f64)>) {
    let (ua, ub) = (oa.anchors(), ob.anchors());
    let mut best = (f64::INFINITY, vec![(0.0, 0.0)]);
    for (i, &u) in ua.iter().enumerate() {
        for (j, &v) in ub.iter().enumerate() {
            if offset != 0 && (i != 0 || j != offset.rem_euclid(ub.len() as i32) as usize) {
                continue;
            }
            let knots = vec![(u, v)];
            let cost = travel_cost(oa, ob, &knots);
            if cost < best.0 {
                best = (cost, knots);
            }
        }
    }
    best
}

/// O custo de percurso sob um mapa (o critério do flubber): o quanto os pontos andam.
fn travel_cost(oa: &Outline, ob: &Outline, knots: &[(f64, f64)]) -> f64 {
    (0..COST_SAMPLES)
        .map(|k| {
            let u = k as f64 / COST_SAMPLES as f64;
            (oa.at(u) - ob.at(map_forward(knots, u))).hypot2()
        })
        .sum()
}

/// As âncoras normalizadas: centro na origem, escala RMS 1. O custo passa a comparar **forma**.
fn normalized(o: &Outline) -> Vec<Point> {
    let pts: Vec<Point> = o.segs.iter().map(|s| s.p0).collect();
    let n = pts.len() as f64;
    let cx = pts.iter().map(|p| p.x).sum::<f64>() / n;
    let cy = pts.iter().map(|p| p.y).sum::<f64>() / n;
    let rms = (pts
        .iter()
        .map(|p| (p.x - cx).powi(2) + (p.y - cy).powi(2))
        .sum::<f64>()
        / n)
        .sqrt()
        .max(MERGE_EPS);
    pts.iter()
        .map(|p| Point::new((p.x - cx) / rms, (p.y - cy) / rms))
        .collect()
}

/// O peso da distorção de arco no custo. As duas parcelas são adimensionais e da mesma ordem
/// (distâncias normalizadas ~1; frações de perímetro ~1/n), então `1.0` é um empate honesto — e
/// não um botão a calibrar.
const STRETCH_WEIGHT: f64 = 1.0;

/// A programação dinâmica, com o 1º nó fixado em `(0, c)`: escolhe `j_1 < j_2 < … < j_{n-1}`
/// (ciclicamente crescentes) minimizando distância-entre-casados + distorção-de-arco.
fn dp_from(
    ua: &[f64],
    ub: &[f64],
    pa: &[Point],
    pb: &[Point],
    c: usize,
) -> Option<(f64, Vec<usize>)> {
    let (n, m) = (ua.len(), ub.len());
    if n > m {
        return None;
    }
    let du = |i: usize| wrap_pos(ua[(i + 1) % n] - ua[i]);
    let dv = |o0: usize, o1: usize| wrap_pos(ub[(c + o1) % m] - ub[(c + o0) % m]);
    let pair = |i: usize, o: usize| (pa[i] - pb[(c + o) % m]).hypot2();
    let stretch = |i: usize, o0: usize, o1: usize| {
        let d = du(i) - dv(o0, o1);
        STRETCH_WEIGHT * d * d
    };

    // dp[i][o] = melhor custo de casar a âncora `i` de A com o offset `o` de B (a partir de c).
    let inf = f64::INFINITY;
    let mut dp = vec![vec![inf; m]; n];
    let mut from = vec![vec![0usize; m]; n];
    dp[0][0] = pair(0, 0);
    for i in 1..n {
        for o in i..=(m - (n - i)) {
            for o0 in (i - 1)..o {
                let prev = dp[i - 1][o0];
                if prev.is_infinite() {
                    continue;
                }
                let cost = prev + pair(i, o) + stretch(i - 1, o0, o);
                if cost < dp[i][o] {
                    dp[i][o] = cost;
                    from[i][o] = o0;
                }
            }
        }
    }
    // Fecha o ciclo: do último nó de volta ao primeiro (a volta completa é `m` offsets).
    let mut best: Option<(f64, usize)> = None;
    for (o, cost) in dp[n - 1].iter().enumerate().skip(n - 1) {
        if cost.is_infinite() {
            continue;
        }
        let total = cost + stretch(n - 1, o, m);
        if best.as_ref().is_none_or(|(bc, _)| total < *bc) {
            best = Some((total, o));
        }
    }
    let (cost, mut o) = best?;
    let mut js = vec![0usize; n];
    for i in (0..n).rev() {
        js[i] = (c + o) % m;
        if i > 0 {
            o = from[i][o];
        }
    }
    Some((cost, js))
}

/// Delta de arco cíclico, sempre **positivo** (uma volta completa vale 1).
#[inline]
fn wrap_pos(d: f64) -> f64 {
    let w = wrap(d);
    if w <= MERGE_EPS { 1.0 } else { w }
}

/// O mapa **monótono por partes** A → B: entre dois nós, o arco caminha proporcional.
///
/// Com um nó só ele é exatamente a rotação de antes (`arco_B = arco_A − fase`) — o caso
/// degradado continua sendo o comportamento que já estava provado.
pub(crate) fn map_forward(knots: &[(f64, f64)], u: f64) -> f64 {
    let k = knots.len();
    if k == 0 {
        return wrap(u);
    }
    if k == 1 {
        return wrap(u - knots[0].0 + knots[0].1);
    }
    let u = wrap(u);
    // O trecho que contém `u`: o último nó com `u_a <= u` (ciclicamente).
    let i = knots
        .iter()
        .rposition(|(ua, _)| *ua <= u + MERGE_EPS)
        .unwrap_or(k - 1);
    let (ua0, ub0) = knots[i];
    let (ua1, ub1) = knots[(i + 1) % k];
    let span_a = wrap_pos(ua1 - ua0);
    let span_b = wrap_pos(ub1 - ub0);
    let f = (wrap_pos(u - ua0) / span_a).clamp(0.0, 1.0);
    let f = if wrap(u - ua0) <= MERGE_EPS { 0.0 } else { f };
    wrap(ub0 + f * span_b)
}

/// O inverso do [`map_forward`] (B → A) — os nós lidos ao contrário.
pub(crate) fn map_backward(knots: &[(f64, f64)], v: f64) -> f64 {
    let flipped: Vec<(f64, f64)> = {
        let mut f: Vec<(f64, f64)> = knots.iter().map(|(a, b)| (*b, *a)).collect();
        f.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
        f
    };
    map_forward(&flipped, v)
}

#[inline]
pub(crate) fn wrap(s: f64) -> f64 {
    let s = s % 1.0;
    if s < 0.0 { s + 1.0 } else { s }
}
