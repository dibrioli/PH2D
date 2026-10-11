//! **Facilitação** (W7, `docs/MiroClone/05_facilitacao.md`): o que fica no DOCUMENTO — o resultado de uma
//! votação (o total de cada objecto) e os carimbos de uma nota, cada um com a sua contagem. A sessão de
//! voto e o cronómetro são da sessão (`ph2d-board-edit`), nunca do ficheiro.

use serde::{Deserialize, Serialize};

use crate::Element;

/// V1 — os votos por pessoa: de 1 a 99 (o Miro: *«the maximum number of votes is 99»*), nasce 3
/// (a imagem `vote_02` do help).
pub const VOTES_MAX: u32 = 99;
pub const VOTES_DEFAULT: u32 = 3;

/// V1 — o que se escolhe ao abrir uma votação (mora no modelo porque viaja numa sessão ao vivo,
/// ADR-0182; a sessão em si é do editor).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteSetup {
    pub votes: u32,
    pub one_per_object: bool,
    /// Minutos do relógio da sessão (0 = sem relógio): armam o cronómetro do quadro.
    pub minutes: u32,
}

impl Default for VoteSetup {
    fn default() -> Self {
        Self {
            votes: VOTES_DEFAULT,
            one_per_object: false,
            minutes: 0,
        }
    }
}

/// Um carimbo — os oito ícones Lucide da galeria (o Miro põe emoji na nota; não há fonte de emoji no
/// programa, estudo §0). ⚠️ Append-only: o discriminante vai para o ficheiro.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Stamp {
    ThumbsUp,
    ThumbsDown,
    Heart,
    Star,
    Flame,
    Lightbulb,
    Question,
    Party,
}

impl Stamp {
    /// Os carimbos pela ordem da galeria.
    pub const ALL: [Stamp; 8] = [
        Stamp::ThumbsUp,
        Stamp::ThumbsDown,
        Stamp::Heart,
        Stamp::Star,
        Stamp::Flame,
        Stamp::Lightbulb,
        Stamp::Question,
        Stamp::Party,
    ];
}

/// Um carimbo numa nota e quantas vezes o deram (S1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StampCount {
    pub stamp: Stamp,
    pub count: u32,
}

/// V2 — o `+` de votar: Ø em fracção do lado do objecto, com o centro NO canto inferior direito
/// (`vote_03` do help do Miro: 49 px numa nota de 236, ±1 px do canto).
pub const VOTE_PLUS: f64 = 0.21;
/// V5 — o selo do resultado: Ø do MENOR e do MAIOR total, em fracção do lado (`vote_05`: 52 e 80 px
/// numa nota de 236 — 1 e 2 votos), linear entre os dois.
pub const VOTE_BADGE: [f64; 2] = [0.22, 0.34];
/// V5 — o centro do selo sai do canto inferior direito por esta fracção do lado, nos dois eixos
/// (`vote_05`: +6,5/+5 e +7,5/+8,5 px numa nota de 236).
pub const VOTE_BADGE_OUT: f64 = 0.03;
/// S3 — a ficha de um carimbo: a altura em fracção do lado e o vão entre fichas e até à nota. ⚠️
/// Escolha (o Miro não mostra o emoji na nota numa imagem medível): a ficha lê-se a 100 % e cabem
/// quatro de um algarismo numa nota quadrada.
pub const STAMP_CHIP_H: f64 = 0.16;
pub const STAMP_CHIP_GAP: f64 = 0.03;
/// S3 — a largura de uma ficha por altura: o ícone e as folgas, mais cada algarismo da contagem (a
/// conta é a mesma para quem desenha e para quem acerta o clique).
pub const STAMP_CHIP_BASE: f64 = 1.05;
pub const STAMP_CHIP_DIGIT: f64 = 0.36;

/// O lado de que as fracções da facilitação partem: o MENOR da caixa (a nota larga mede-se pela
/// altura, como a quadrada).
#[must_use]
pub fn mark_side(el: &Element) -> f64 {
    el.w.min(el.h)
}

/// V2 — o centro e o Ø do `+` de votar, no referencial da caixa do elemento (origem no canto superior
/// esquerdo, antes de rodar).
#[must_use]
pub fn vote_plus(el: &Element) -> ([f64; 2], f64) {
    ([el.w, el.h], mark_side(el) * VOTE_PLUS)
}

/// V5 — o centro e o Ø do selo de `count` votos, quando os totais do quadro vão de `lo` a `hi`, no
/// referencial da caixa.
#[must_use]
pub fn vote_badge(el: &Element, count: u32, [lo, hi]: [u32; 2]) -> ([f64; 2], f64) {
    let s = mark_side(el);
    let t = if hi > lo {
        f64::from(count.saturating_sub(lo)) / f64::from(hi - lo)
    } else {
        1.0
    };
    let d = s * (VOTE_BADGE[0] + (VOTE_BADGE[1] - VOTE_BADGE[0]) * t.clamp(0.0, 1.0));
    let out = s * VOTE_BADGE_OUT;
    ([el.w + out, el.h + out], d)
}

/// Quantos algarismos tem `n` (a largura da ficha).
#[must_use]
pub fn digits(n: u32) -> usize {
    n.checked_ilog10().map_or(1, |l| l as usize + 1)
}

/// S3 — as fichas dos carimbos de `el`, por baixo dela, da esquerda, em filas que param antes do selo
/// do voto (o maior) e do `+` (que tapavam a 4.ª ficha e lhe roubavam o clique — auditoria W7):
/// `(carimbo, [x0, y0, x1, y1])` no referencial da caixa.
#[must_use]
pub fn stamp_chips(el: &Element) -> Vec<(StampCount, [f64; 4])> {
    let s = mark_side(el);
    let (h, gap) = (s * STAMP_CHIP_H, s * STAMP_CHIP_GAP);
    let reach = el.w - s * (VOTE_BADGE[1] / 2.0 - VOTE_BADGE_OUT) - gap;
    let (mut x, mut y) = (0.0, el.h + gap);
    let mut out = Vec::new();
    for c in el.stamps() {
        let w = h * (STAMP_CHIP_BASE + STAMP_CHIP_DIGIT * digits(c.count) as f64);
        if x > 0.0 && x + w > reach {
            (x, y) = (0.0, y + h + gap);
        }
        out.push((*c, [x, y, x + w, y + h]));
        x += w + gap;
    }
    out
}

impl Element {
    /// O total do último resultado de votação neste objecto (V4; `0` = nenhum).
    #[must_use]
    pub fn votes(&self) -> u32 {
        self.org.as_ref().map_or(0, |o| o.votes)
    }

    pub fn set_votes(&mut self, n: u32) {
        if n > 0 || self.org.is_some() {
            self.org_mut().votes = n;
        }
    }

    /// Os carimbos, pela ordem em que nasceram (S3).
    #[must_use]
    pub fn stamps(&self) -> &[StampCount] {
        self.org.as_ref().map_or(&[], |o| &o.stamps)
    }

    /// Soma `delta` ao carimbo `s` (nasce no fim; a 0 sai). `true` = mudou.
    pub fn add_stamp(&mut self, s: Stamp, delta: i64) -> bool {
        if delta == 0 || (delta < 0 && !self.stamps().iter().any(|c| c.stamp == s)) {
            return false;
        }
        let list = &mut self.org_mut().stamps;
        let at = list.iter().position(|c| c.stamp == s).unwrap_or_else(|| {
            list.push(StampCount { stamp: s, count: 0 });
            list.len() - 1
        });
        let n = (i64::from(list[at].count) + delta).clamp(0, i64::from(u32::MAX));
        if n == 0 {
            list.remove(at);
        } else {
            list[at].count = u32::try_from(n).unwrap_or(u32::MAX);
        }
        true
    }
}

#[cfg(test)]
#[path = "facilitate_tests.rs"]
mod tests;
