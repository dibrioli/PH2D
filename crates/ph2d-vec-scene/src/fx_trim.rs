//! **Trim Path** — revelar só um trecho do caminho. O *draw-on* de motion design.
//!
//! Três frações do comprimento TOTAL: `start`, `end` e um `offset` que gira o ponto de
//! partida ao longo do caminho. É o Trim Paths do After Effects e o Trim do Cavalry, e é
//! o efeito que casa com a timeline que já temos — keyar `end` de 0 a 1 desenha a forma.
//!
//! # Por que ele NÃO é o `marker::trim_path`
//!
//! Aquele existe e faz outra coisa: recua as pontas para dar lugar às setas, **ao longo da
//! poligonal das âncoras**, e devolve o caminho intocado se ele for fechado. Os dois desvios
//! são corretos lá (o recuo vale poucos múltiplos da largura do traço, e uma forma fechada
//! não tem ponta onde pôr seta) e são fatais aqui: revelar um círculo **é** o caso principal,
//! e medir por âncoras faria a revelação acelerar e frear conforme a densidade de pontos.
//! Este mede por [`crate::arclen`] exato.
//!
//! # O que sai
//!
//! Um contorno **ABERTO**, salvo quando o intervalo cobre a volta inteira de um fechado —
//! aí o fechado sobrevive intacto. Num fechado o intervalo **dá a volta pela emenda**
//! (é o que faz o `offset` ser útil); num aberto ele é recortado nas pontas.

use crate::arclen::{Cubic, arclen, inv_arclen, subsegment};
use crate::corner_live::segment;
use crate::{VecVertex, VertexKind};

/// Abaixo disto o pedido é "nada" — e nada é uma resposta legítima: é o primeiro quadro de
/// um draw-on. Em fração do comprimento total.
const EPS: f64 = 1e-12;

/// **Os parâmetros do Trim**, em frações do comprimento total do contorno.
///
/// Neutro em `{0, 1, 0}` — e o neutro é um **no-op byte-idêntico**, invariante que a pilha
/// inteira depende (ADR-0132: um efeito neutro não pode custar uma alocação nem mover um
/// bit, senão nenhum documento voltaria a ser `Cow::Borrowed`).
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrimSpec {
    /// Onde o trecho começa, `0..=1` do comprimento.
    pub start: f64,
    /// Onde ele acaba, `0..=1`.
    pub end: f64,
    /// Gira o ponto de partida ao longo do caminho, `0..=1`. Num contorno fechado ele dá a
    /// volta; num aberto ele desliza e o que sai do domínio é recortado.
    pub offset: f64,
}

impl Default for TrimSpec {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
            offset: 0.0,
        }
    }
}

impl TrimSpec {
    /// Este Trim não faz nada? (o caminho inteiro, sem deslocamento)
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.start <= EPS && self.end >= 1.0 - EPS && self.offset.abs() <= EPS
    }
}

/// Um pedaço a emitir: o segmento `seg` do contorno, de `t0` a `t1`.
///
/// `pub(crate)` porque o [`crate::fx_knot`] reusa a mesma máquina de corte por arco: um Trim
/// revela UM intervalo, um Knot corta os vãos das passagens de baixo (o COMPLEMENTO de vários
/// intervalos). Duas cópias do corte por arco divergiriam no 1º ajuste.
pub(crate) struct Piece {
    pub(crate) seg: usize,
    pub(crate) t0: f64,
    pub(crate) t1: f64,
}

/// **Aplica o Trim a UM contorno.** Devolve `(verts, closed)`.
///
/// Vazio (`verts.len() == 0`) é uma saída válida e esperada: é o quadro em que o draw-on
/// ainda não começou. Quem consome geometria já lida com contorno sem vértices.
#[must_use]
pub fn trim_contour(verts: &[VecVertex], closed: bool, spec: &TrimSpec) -> (Vec<VecVertex>, bool) {
    let n = verts.len();
    if n < 2 || spec.is_neutral() {
        return (verts.to_vec(), closed);
    }
    let seg_count = if closed { n } else { n - 1 };
    let segs: Vec<Cubic> = (0..seg_count).map(|i| segment(verts, i, n)).collect();
    let lens: Vec<f64> = segs.iter().map(arclen).collect();
    let total: f64 = lens.iter().sum();
    if total <= EPS {
        return (verts.to_vec(), closed);
    }

    // O intervalo pedido, em fração. Num FECHADO ele pode dar a volta pela emenda; num
    // aberto o que cai fora do domínio simplesmente não existe.
    let span = (spec.end - spec.start).clamp(0.0, 1.0);
    if span <= EPS {
        return (Vec::new(), false);
    }
    let keeps_the_loop = closed && span >= 1.0 - EPS;
    if keeps_the_loop {
        return (verts.to_vec(), true); // a volta inteira: o fechado sobrevive intacto
    }
    let lo = spec.start + spec.offset;
    let (lo, hi) = if closed {
        (lo.rem_euclid(1.0), lo.rem_euclid(1.0) + span)
    } else {
        let a = lo.clamp(0.0, 1.0);
        (a, (lo + span).clamp(0.0, 1.0))
    };
    if hi - lo <= EPS {
        return (Vec::new(), false);
    }

    let pieces = pieces_between(&segs, &lens, total, lo, hi, closed);
    (rebuild(verts, &segs, &pieces, n), false)
}

/// Traduz o intervalo de FRAÇÃO para a lista de pedaços de segmento que ele cobre.
///
/// Num fechado `hi` pode passar de 1: o percurso dá a volta e o índice de segmento continua
/// a andar módulo `seg_count`. É por isso que a caminhada é por *comprimento acumulado a
/// partir de `lo`*, e não por "de que segmento é `lo`, de que segmento é `hi`".
pub(crate) fn pieces_between(
    segs: &[Cubic],
    lens: &[f64],
    total: f64,
    lo: f64,
    hi: f64,
    closed: bool,
) -> Vec<Piece> {
    let (s_lo, s_hi) = (lo * total, hi * total);
    let mut out = Vec::new();
    let count = segs.len();
    // Onde `lo` cai: o segmento e o comprimento já andado até o início dele.
    let mut walked = 0.0;
    let mut idx = 0usize;
    let mut guard = 0usize;
    while idx < count && walked + lens[idx] <= s_lo {
        walked += lens[idx];
        idx += 1;
    }
    if idx >= count {
        return out; // `lo` caiu no fim exato de um aberto
    }
    let mut cursor = s_lo;
    while cursor < s_hi - EPS {
        guard += 1;
        if guard > count * 2 + 4 {
            break; // defesa: um `total` degenerado não pode virar laço infinito
        }
        let i = idx % count;
        let seg_start = walked;
        let seg_end = walked + lens[i];
        let piece_end = seg_end.min(s_hi);
        if piece_end > cursor + EPS && lens[i] > EPS {
            out.push(Piece {
                seg: i,
                t0: inv_arclen(&segs[i], cursor - seg_start),
                t1: inv_arclen(&segs[i], piece_end - seg_start),
            });
        }
        cursor = piece_end;
        walked = seg_end;
        idx += 1;
        if idx >= count && !closed {
            break;
        }
    }
    out
}

/// Remonta os vértices a partir da corrente de cúbicas cortadas.
///
/// O `kind` do vértice **original** sobrevive quando o corte cai exatamente sobre ele
/// (`t == 0` ou `t == 1`); um ponto de CORTE nasce `Corner`, porque não há suavidade a
/// preservar de um lado que deixou de existir. O `corner_radius` sai zero em todos: o
/// estágio da quina já correu (ADR-0132 §3), e a pilha trabalha sobre geometria plana.
pub(crate) fn rebuild(
    verts: &[VecVertex],
    segs: &[Cubic],
    pieces: &[Piece],
    n: usize,
) -> Vec<VecVertex> {
    if pieces.is_empty() {
        return Vec::new();
    }
    let cut: Vec<Cubic> = pieces
        .iter()
        .map(|p| subsegment(&segs[p.seg], p.t0, p.t1))
        .collect();
    let kind_at = |p: &Piece, head: bool| -> VertexKind {
        let whole = if head { p.t0 <= EPS } else { p.t1 >= 1.0 - EPS };
        if !whole {
            return VertexKind::Corner;
        }
        let v = if head { p.seg } else { (p.seg + 1) % n };
        verts[v].kind
    };
    let mut out = Vec::with_capacity(cut.len() + 1);
    out.push(VecVertex {
        anchor: cut[0][0],
        in_handle: cut[0][0], // ponta: não há nada a chegar
        out_handle: cut[0][1],
        kind: kind_at(&pieces[0], true),
        corner_radius: 0.0,
    });
    for w in 0..cut.len() - 1 {
        out.push(VecVertex {
            anchor: cut[w][3],
            in_handle: cut[w][2],
            out_handle: cut[w + 1][1],
            kind: kind_at(&pieces[w], false),
            corner_radius: 0.0,
        });
    }
    let last = cut.len() - 1;
    out.push(VecVertex {
        anchor: cut[last][3],
        in_handle: cut[last][2],
        out_handle: cut[last][3], // ponta
        kind: kind_at(&pieces[last], false),
        corner_radius: 0.0,
    });
    out
}

#[cfg(test)]
#[path = "fx_trim_tests.rs"]
mod tests;
