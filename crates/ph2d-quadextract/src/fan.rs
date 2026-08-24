//! ⭐ **O LEQUE DE UM VÉRTICE**, e o único sentido em que ele se percorre.
//!
//! ⛔⛔ **O sentido é HORÁRIO SOBRE A SUPERFÍCIE, e ele é load-bearing** — é ele
//! que faz *«virar à esquerda»*, na extracção de células, ser simplesmente *«a
//! saída seguinte na lista»* ([`crate::cells`]).
//!
//! ⚠️ **Horário sobre a superfície ≠ horário no domínio**, e as duas coisas que
//! quebram a correspondência acontecem as duas: uma transição não-identidade
//! introduz um salto na direcção entre triângulos vizinhos, e um triângulo
//! **dobrado** (área negativa no domínio) inverte a ordem das suas saídas quando
//! volta à superfície. Quem consome este módulo tem de tratar as duas — ver
//! [`crate::ports`].
//!
//! # A mecânica, numa frase
//!
//! Num triângulo `(v, Q, R)` orientado positivamente na superfície, rodar em
//! sentido **anti-horário** em torno de `v` leva a direcção `v→Q` até `v→R`. Logo o
//! sentido **horário** entra pelo lado de `R` e sai pelo lado de `Q` — e é por isso
//! que o passo horário atravessa o **lado `k`** (a aresta `v→Q`) e o passo
//! anti-horário atravessa o **lado `k+2`** (a aresta `R→v`).

use crate::exact::Xf;
use crate::ingest::Topo;

/// Um canto: a face e qual dos três vértices dela.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Corner {
    pub face: u32,
    pub k: u8,
}

impl Corner {
    pub(crate) fn new(face: usize, k: usize) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self {
            face: face as u32,
            k: k as u8,
        }
    }
    pub(crate) fn f(self) -> usize {
        self.face as usize
    }
    pub(crate) fn kk(self) -> usize {
        self.k as usize
    }
}

/// O leque percorrido, mais a transição acumulada desde o primeiro canto.
pub(crate) struct Fan {
    /// Os cantos, em ordem **horária sobre a superfície**.
    pub corners: Vec<Corner>,
    /// Por canto, a transição que leva a carta do **primeiro** canto à deste.
    pub to_here: Vec<Xf>,
    /// ⭐ **A HOLONOMIA** — a transição acumulada ao dar a volta inteira. Só existe
    /// num leque fechado, e num vértice regular ela é a identidade.
    pub holonomy: Option<Xf>,
}

/// **O passo HORÁRIO** — atravessa o lado `k`.
pub(crate) fn cw(topo: &Topo, c: Corner) -> Option<(Corner, Xf)> {
    let (g, j) = topo.twin[c.f()][c.kk()]?;
    Some((
        Corner::new(g as usize, (j as usize + 1) % 3),
        topo.xf[c.f()][c.kk()],
    ))
}

/// **O passo ANTI-HORÁRIO** — atravessa o lado `k+2`.
pub(crate) fn ccw(topo: &Topo, c: Corner) -> Option<Corner> {
    let side = (c.kk() + 2) % 3;
    let (h, m) = topo.twin[c.f()][side]?;
    Some(Corner::new(h as usize, m as usize))
}

/// ⭐ **O LEQUE INTEIRO** a partir de qualquer canto dele.
///
/// ⚠️ **Recua primeiro até ao princípio.** Num leque **aberto** (vértice de bordo)
/// o primeiro canto é o único sem vizinho anti-horário, e começar no meio dele
/// daria metade das saídas — que é exactamente o modo de falha que uma malha com
/// bordo exibe e uma malha fechada esconde.
pub(crate) fn fan_of(topo: &Topo, start: Corner) -> Fan {
    let limit = topo.tris.len() + 1;
    // Recuar até ao início (ou fechar a volta).
    let mut first = start;
    for _ in 0..limit {
        match ccw(topo, first) {
            None => break,
            Some(p) => {
                if p == start {
                    first = start;
                    break;
                }
                first = p;
            }
        }
    }
    // Avançar em sentido horário, acumulando.
    let mut corners = vec![first];
    let mut to_here = vec![Xf::IDENTITY];
    let mut acc = Xf::IDENTITY;
    let mut cur = first;
    let mut holonomy = None;
    for _ in 0..limit {
        let Some((next, step)) = cw(topo, cur) else {
            break;
        };
        acc = acc.then(step);
        if next == first {
            holonomy = Some(acc);
            break;
        }
        corners.push(next);
        to_here.push(acc);
        cur = next;
    }
    Fan {
        corners,
        to_here,
        holonomy,
    }
}

/// Um canto de cada vértice — a semente de [`fan_of`].
pub(crate) fn seed_corners(topo: &Topo) -> Vec<Option<Corner>> {
    let mut seed: Vec<Option<Corner>> = vec![None; topo.verts];
    for (f, tri) in topo.tris.iter().enumerate() {
        for k in 0..3 {
            let slot = &mut seed[tri[k] as usize];
            if slot.is_none() {
                *slot = Some(Corner::new(f, k));
            }
        }
    }
    seed
}
