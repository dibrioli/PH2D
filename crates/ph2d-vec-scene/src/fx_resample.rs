//! **A reamostragem partilhada dos efeitos que reconstroem a curva** (Warp, Twist, Zig Zag, Sketch).
//!
//! Eles amostram o contorno por ARCO, mexem cada amostra e voltam a pôr alças. O que se perdia aí
//! é o que os FILTROS veem (F33, Bevel + Flag): uma quina que sai `Smooth` vira uma volta
//! apertada, e uma alça maior que o seu segmento — dois pontos quase colados — vira um laço. O
//! Bevel lê a direção da curva, e os dois aparecem como vincos. Medição e plano:
//! `docs/UI_New_and_Simple/F33_plano.md`.
//!
//! A lei, numa porta só:
//! - a amostra SABE se é a âncora `i`, e ao fundir duas coincidentes a âncora ganha;
//! - na âncora, as DUAS tangentes exatas ([`ArcPath::anchor_sides`]) — quina se diferem;
//! - cada alça mede o SEU vão / 3 (Hermite), nunca o vão dos dois lados.

use crate::arc_path::ArcPath;
use crate::{VecVertex, VertexKind};

/// Uma posição de arco a amostrar; `ancora` = o índice da âncora de entrada que ela É.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Amostra {
    pub s: f64,
    pub ancora: Option<usize>,
}

/// **A grade ∪ as âncoras**, ordenada, sem duplicados a menos de `merge` — e numa fusão a ÂNCORA
/// fica (é ela que carrega a quina). Num aberto a ponta final entra como a âncora `segs`.
pub(crate) fn amostras(
    ap: &ArcPath,
    grade: impl IntoIterator<Item = f64>,
    merge: f64,
    max: usize,
) -> Vec<Amostra> {
    let total = ap.total();
    let mut at: Vec<Amostra> = grade
        .into_iter()
        .map(|s| Amostra { s, ancora: None })
        .collect();
    at.extend(
        ap.anchor_arcs()
            .iter()
            .enumerate()
            .map(|(i, &s)| Amostra { s, ancora: Some(i) }),
    );
    if !ap.closed() {
        at.push(Amostra {
            s: total,
            ancora: Some(ap.anchor_arcs().len()),
        });
    }
    // Âncoras primeiro entre iguais: a fusão abaixo guarda a 1.ª de cada grupo.
    at.sort_by(|a, b| {
        a.s.total_cmp(&b.s)
            .then_with(|| b.ancora.is_some().cmp(&a.ancora.is_some()))
    });
    let mut out: Vec<Amostra> = Vec::with_capacity(at.len());
    for a in at {
        match out.last_mut() {
            Some(u) if (a.s - u.s).abs() <= merge => {
                if u.ancora.is_none() && a.ancora.is_some() {
                    *u = a;
                }
            }
            _ => out.push(a),
        }
    }
    // Num fechado, a última amostra pode coincidir com a âncora 0 (a volta inteira).
    if ap.closed()
        && out.len() > 1
        && (total - out[out.len() - 1].s).abs() <= merge
        && out[out.len() - 1].ancora.is_none()
    {
        out.pop();
    }
    // Acima do teto caem as amostras da GRADE, nunca as âncoras: cortar a lista ordenada (o que se
    // fazia) apagava o FIM do contorno. As âncoras já são a entrada — não custam memória nova.
    if out.len() > max {
        out.retain(|a| a.ancora.is_some());
    }
    out
}

/// `(ponto, tangente que chega, tangente que sai)` da amostra — iguais fora das âncoras.
pub(crate) fn lados(ap: &ArcPath, a: &Amostra) -> ([f64; 2], [f64; 2], [f64; 2]) {
    match a.ancora {
        Some(i) => ap.anchor_sides(i),
        None => {
            let (p, t) = ap.frame_at(a.s);
            (p, t, t)
        }
    }
}

/// Os dois vãos de arco da amostra `k`: `(até à anterior, até à seguinte)`. Num fechado dá a
/// volta; numa ponta de aberto o vão que falta espelha o outro.
pub(crate) fn vaos(at: &[Amostra], k: usize, closed: bool, total: f64) -> (f64, f64) {
    let m = at.len();
    let volta = at[0].s + total - at[m - 1].s;
    let tras = if k > 0 {
        at[k].s - at[k - 1].s
    } else if closed {
        volta
    } else if m > 1 {
        at[1].s - at[0].s
    } else {
        0.0
    };
    let frente = if k + 1 < m {
        at[k + 1].s - at[k].s
    } else if closed {
        volta
    } else {
        tras
    };
    (tras, frente)
}

/// Abaixo desta viragem (rad) as duas tangentes são a MESMA — o arredondamento de duas
/// avaliações de uma curva lisa, não uma quina.
const MESMA_DIRECAO: f64 = 1e-9;

/// **O vértice de Hermite**: `d_chega`/`d_sai` são a derivada da curva NOVA por unidade de arco
/// da fonte, de cada lado; cada alça é essa derivada × o seu vão / 3. `Corner` se as direções
/// diferem; senão `Smooth`, com as duas alças na mesma reta (a média das duas).
pub(crate) fn vertice(
    anchor: [f64; 2],
    d_chega: [f64; 2],
    d_sai: [f64; 2],
    (tras, frente): (f64, f64),
) -> VecVertex {
    let quina = crate::arclen::turn_between(d_chega, d_sai) > MESMA_DIRECAO;
    let (a, b) = if quina {
        (d_chega, d_sai)
    } else {
        let m = [0.5 * (d_chega[0] + d_sai[0]), 0.5 * (d_chega[1] + d_sai[1])];
        (m, m)
    };
    let (kt, kf) = (tras / 3.0, frente / 3.0);
    VecVertex {
        anchor,
        in_handle: [a[0].mul_add(-kt, anchor[0]), a[1].mul_add(-kt, anchor[1])],
        out_handle: [b[0].mul_add(kf, anchor[0]), b[1].mul_add(kf, anchor[1])],
        kind: if quina {
            VertexKind::Corner
        } else {
            VertexKind::Smooth
        },
        corner_radius: 0.0,
        id: crate::VertId::NOVO,
    }
}
