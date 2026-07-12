//! **A maquinaria do anel** — o motor comum dos balões, módulo irmão de
//! [`crate::bubbles`] (teto de 700 LOC por arquivo, HR-18).
//!
//! Os balões são todos a mesma receita: montar um ANEL de vértices no espaço de autoria
//! (`Uv`, `v` para baixo), costurar arcos com ou sem cúspide, enxertar o rabo, reajustar
//! a bbox e só então colocar no mundo. Separar isto do desenho de cada balão é o que
//! mantém as seis formas legíveis — e é onde vive a única inversão de eixo.

use crate::space::{Unit, Uv};
use crate::{VecVertex, VertexKind};

// ─────────────────────────────────────────────────────────────────────────────
// Álgebra do espaço de autoria
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn v_add(p: Uv, q: Uv) -> Uv {
    (p.0 + q.0, p.1 + q.1)
}
pub(crate) fn v_sub(p: Uv, q: Uv) -> Uv {
    (p.0 - q.0, p.1 - q.1)
}
pub(crate) fn v_mul(p: Uv, k: f64) -> Uv {
    (p.0 * k, p.1 * k)
}
pub(crate) fn v_len(p: Uv) -> f64 {
    p.0.hypot(p.1)
}
pub(crate) fn v_mid(p: Uv, q: Uv) -> Uv {
    ((p.0 + q.0) * 0.5, (p.1 + q.1) * 0.5)
}

/// Normaliza. Vetor NULO devolve `(1, 0)` em vez de `NaN` — o rabo de comprimento zero
/// degenera numa reta sobre a aresta (que é o limite correto), nunca numa forma podre.
pub(crate) fn v_norm(p: Uv) -> Uv {
    let l = v_len(p);
    if l < 1e-12 {
        (1.0, 0.0)
    } else {
        (p.0 / l, p.1 / l)
    }
}

/// Vértice em espaço de autoria: âncora + os dois handles, todos em `(u, v)`. `cusp` marca
/// a quina (handles independentes) — é o que separa o bico do rabo, que é uma cúspide de
/// 180°, dos joins tangentes da nuvem.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Uvv {
    pub(crate) a: Uv,
    pub(crate) i: Uv,
    pub(crate) o: Uv,
    pub(crate) cusp: bool,
}

/// Quina reta: handles colados na âncora.
pub(crate) fn sharp(a: Uv) -> Uvv {
    Uvv {
        a,
        i: a,
        o: a,
        cusp: true,
    }
}

/// Arco elíptico em espaço de autoria — cúbicas EXATAS (nunca polígono). Mesma convenção
/// de [`Unit::arc`]: o ângulo cresce de `+u` em direção a `+v` (para baixo), então `90°` é
/// a base; `sweep_deg` é com sinal e o sinal viaja no comprimento de handle
/// `(4/3)·tan(θ/4)`, então o arco negativo já sai percorrido ao contrário.
pub(crate) fn uv_arc(c: Uv, ru: f64, rv: f64, start_deg: f64, sweep_deg: f64) -> Vec<Uvv> {
    let start = start_deg.to_radians();
    let total = sweep_deg.to_radians();
    let n = (total.abs() / std::f64::consts::FRAC_PI_2).ceil().max(1.0);
    let seg = total / n;
    let h = (4.0 / 3.0) * (seg / 4.0).tan();
    let steps = n as usize;
    (0..=steps)
        .map(|k| {
            let ang = start + seg * k as f64;
            let (s, co) = ang.sin_cos();
            let a = (c.0 + ru * co, c.1 + rv * s);
            let t = (-ru * s * h, rv * co * h);
            Uvv {
                a,
                i: (a.0 - t.0, a.1 - t.1),
                o: (a.0 + t.0, a.1 + t.1),
                cusp: false,
            }
        })
        .collect()
}

/// Achata a junção `i → i+1` numa RETA: zera os handles do lado reto (senão a aresta nasce
/// com handles fantasmas herdados da tangente do arco vizinho, que o usuário vê e arrasta).
pub(crate) fn uv_straighten(vs: &mut [Uvv], i: usize) {
    if let Some(v) = vs.get_mut(i) {
        v.o = v.a;
        v.cusp = true;
    }
    if let Some(v) = vs.get_mut(i + 1) {
        v.i = v.a;
        v.cusp = true;
    }
}

/// Costura um arco no fim do anel **FUNDINDO** o vértice compartilhado: quem chega traz o
/// `in_handle`, quem sai traz o `out_handle`. Concatenar ingenuamente (pulando o primeiro
/// do arco novo) perderia a tangente de saída; concatenar sem pular duplicaria a âncora.
pub(crate) fn stitch(ring: &mut Vec<Uvv>, arc: &[Uvv], cusp: bool) {
    let Some((first, rest)) = arc.split_first() else {
        return;
    };
    if let Some(last) = ring.last_mut() {
        last.o = first.o;
        last.cusp = cusp;
        ring.extend_from_slice(rest);
    } else {
        ring.extend_from_slice(arc);
    }
}

/// Fecha o anel: o último vértice coincide com o primeiro — funde o handle de entrada nele
/// e descarta o duplicado.
pub(crate) fn seal(ring: &mut Vec<Uvv>, cusp: bool) {
    if ring.len() < 2 {
        return;
    }
    let last = ring.pop().expect("o anel tem ao menos dois vertices");
    if let Some(first) = ring.first_mut() {
        first.i = last.i;
        first.cusp = cusp;
    }
}

/// Rotaciona `k` quartos de volta em torno do centro da caixa, no espaço de autoria (`+u`
/// vai para `+v`, isto é, o giro é horário na TELA). É o que permite UM caminho de código
/// para os quatro lados do rabo: constrói-se sempre com o rabo saindo à DIREITA e gira-se a
/// forma inteira no fim.
pub(crate) fn rot_q(p: Uv, k: u8) -> Uv {
    let (du, dv) = (p.0 - 0.5, p.1 - 0.5);
    let (du, dv) = match k % 4 {
        0 => (du, dv),
        1 => (-dv, du),
        2 => (-du, -dv),
        _ => (dv, -du),
    };
    (0.5 + du, 0.5 + dv)
}

/// O setor por onde o rabo sai, a partir do bico (o teste `dz` do OOXML, limpo):
/// `0` = direita · `1` = BASE · `2` = esquerda · `3` = topo.
pub(crate) fn sector(tip: Uv) -> u8 {
    let (du, dv) = (tip.0 - 0.5, tip.1 - 0.5);
    if du.abs() >= dv.abs() {
        if du >= 0.0 { 0 } else { 2 }
    } else if dv >= 0.0 {
        1
    } else {
        3
    }
}

/// Autoria → MUNDO: gira `k` quartos e atravessa a fronteira ([`Unit::p`]) de uma vez.
pub(crate) fn place(u: &Unit, k: u8, vs: &[Uvv]) -> Vec<VecVertex> {
    vs.iter()
        .map(|w| VecVertex {
            anchor: u.p(rot_q(w.a, k)),
            in_handle: u.p(rot_q(w.i, k)),
            out_handle: u.p(rot_q(w.o, k)),
            kind: if w.cusp {
                VertexKind::Corner
            } else {
                VertexKind::Smooth
            },
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// bbox exata + refit (a alavanca de contenção, em espaço de autoria)
// ─────────────────────────────────────────────────────────────────────────────

/// As raízes em `(0,1)` da derivada de uma cúbica escalar — onde a curva tem extremo
/// INTERNO. Sem elas a bbox é a das âncoras, que **subestima** a forma (a barriga de cada
/// bolha da nuvem vive justamente aí) e o refit deixaria a nuvem transbordando.
pub(crate) fn extrema(p0: f64, p1: f64, p2: f64, p3: f64, out: &mut Vec<f64>) {
    let a = 3.0 * (-p0 + 3.0 * p1 - 3.0 * p2 + p3);
    let b = 6.0 * (p0 - 2.0 * p1 + p2);
    let c = 3.0 * (p1 - p0);
    if a.abs() < 1e-12 {
        if b.abs() > 1e-12 {
            out.push(-c / b);
        }
        return;
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return;
    }
    let s = disc.sqrt();
    out.push((-b + s) / (2.0 * a));
    out.push((-b - s) / (2.0 * a));
}

/// Uma cúbica escalar em `t` (de Casteljau expandido).
pub(crate) fn cubic_at(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// A bbox VERDADEIRA de um anel fechado em espaço de autoria.
pub(crate) fn ring_bbox(vs: &[Uvv]) -> ([f64; 2], [f64; 2]) {
    let axis = |p: Uv, k: usize| if k == 0 { p.0 } else { p.1 };
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for w in vs {
        for k in 0..2 {
            lo[k] = lo[k].min(axis(w.a, k));
            hi[k] = hi[k].max(axis(w.a, k));
        }
    }
    let n = vs.len();
    let mut roots: Vec<f64> = Vec::with_capacity(2);
    for i in 0..n {
        let (x, y) = (vs[i], vs[(i + 1) % n]);
        for k in 0..2 {
            let (p0, p1, p2, p3) = (axis(x.a, k), axis(x.o, k), axis(y.i, k), axis(y.a, k));
            roots.clear();
            extrema(p0, p1, p2, p3, &mut roots);
            for &t in &roots {
                if t > 0.0 && t < 1.0 {
                    let val = cubic_at(p0, p1, p2, p3, t);
                    lo[k] = lo[k].min(val);
                    hi[k] = hi[k].max(val);
                }
            }
        }
    }
    (lo, hi)
}

/// Mapeia (afim, exato sobre as Béziers) a bbox verdadeira do anel no retângulo `lo..hi`.
pub(crate) fn refit(vs: &mut [Uvv], lo: [f64; 2], hi: [f64; 2]) {
    let (blo, bhi) = ring_bbox(vs);
    let (mut scale, mut shift) = ([1.0_f64; 2], [0.0_f64; 2]);
    for k in 0..2 {
        let span = bhi[k] - blo[k];
        if span > 1e-9 {
            scale[k] = (hi[k] - lo[k]) / span;
            shift[k] = lo[k] - blo[k] * scale[k];
        } else {
            shift[k] = (lo[k] + hi[k]) * 0.5 - blo[k];
        }
    }
    let map = |p: Uv| (p.0 * scale[0] + shift[0], p.1 * scale[1] + shift[1]);
    for w in vs.iter_mut() {
        w.a = map(w.a);
        w.i = map(w.i);
        w.o = map(w.o);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// O rabo (compartilhado pelos dois balões de fala)
// ─────────────────────────────────────────────────────────────────────────────

/// Comprimento do handle da raiz (× `L`).
pub(crate) const ROOT_LEN: f64 = 0.30;
/// Mistura do handle da raiz entre "na direção do bico" (0) e "na normal do corpo" (1).
/// **É o parâmetro que faz ou quebra o rabo:** em `1.0` (a escolha "óbvia") um rabo
/// inclinado sai perpendicular ao corpo e depois tem de voltar para o bico — um gancho em
/// S. `0.35` é o ponto doce.
pub(crate) const ROOT_BIAS: f64 = 0.35;
/// Comprimento do handle do bico (× `L`) — a agudez do afunilamento.
pub(crate) const TIP_PULL: f64 = 0.45;

/// Os dois flancos do rabo. O caminho CHEGA em `pa` e SAI de `pb`; `na`/`nb` são as normais
/// externas do corpo nesses pontos; `t` é o bico. Devolve `(out de pa, o vértice do bico,
/// in de pb)`.
///
/// Os dois handles do bico COINCIDEM (`spread = 0`): a ponta é uma agulha — a cúspide de
/// 180° do rabo de quadrinho. Os dois ramos se separam só na segunda ordem, e para lados
/// OPOSTOS do eixo (porque `pa` e `pb` estão em lados opostos), então eles nunca se cruzam.
///
/// **Os handles são limitados pela CORDA até o bico** (metade dela — a regra clássica da
/// Bézier). Sem esse teto, um rabo curto e muito inclinado (bico no canto + faixa fina: a
/// base é clampada longe do bico, e o eixo fica quase paralelo à aresta) produz um handle de
/// raiz que **passa do bico** — a cúspide vira um laço, e o contorno se auto-intersecta. O
/// teto é inerte na configuração normal (o comprimento pedido já é menor que meia corda) e
/// só morde exatamente no caso degenerado.
pub(crate) fn tail_flanks(pa: Uv, pb: Uv, t: Uv, na: Uv, nb: Uv) -> (Uv, Uvv, Uv) {
    let m = v_mid(pa, pb);
    let axis = v_sub(t, m);
    let l = v_len(axis);
    let dir = v_norm(axis);
    let (chord_a, chord_b) = (v_len(v_sub(t, pa)), v_len(v_sub(t, pb)));
    let ra = v_norm(v_add(
        v_mul(v_norm(v_sub(t, pa)), 1.0 - ROOT_BIAS),
        v_mul(na, ROOT_BIAS),
    ));
    let rb = v_norm(v_add(
        v_mul(v_norm(v_sub(t, pb)), 1.0 - ROOT_BIAS),
        v_mul(nb, ROOT_BIAS),
    ));
    let pull = (TIP_PULL * l).min(0.5 * chord_a.min(chord_b));
    let back = v_sub(t, v_mul(dir, pull));
    (
        v_add(pa, v_mul(ra, (ROOT_LEN * l).min(0.5 * chord_a))),
        Uvv {
            a: t,
            i: back,
            o: back,
            cusp: true,
        },
        v_add(pb, v_mul(rb, (ROOT_LEN * l).min(0.5 * chord_b))),
    )
}
