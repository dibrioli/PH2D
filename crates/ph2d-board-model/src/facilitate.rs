//! **Facilitação** (W7, `docs/MiroClone/05_facilitacao.md`): o que fica no DOCUMENTO — o resultado de uma
//! votação (o total de cada objecto) e os carimbos de uma nota, cada um com a sua contagem. A sessão de
//! voto e o cronómetro são da sessão (`ph2d-board-edit`), nunca do ficheiro.

use serde::{Deserialize, Serialize};

use crate::Element;

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
