//! ⭐ **O histórico de uma tabela ao vivo** (plano 14 Q3): por chave de linha, um anel com as últimas
//! amostras numéricas (o traço da coluna «Last N s») e o último texto (o «mudou desde a amostra
//! anterior»). Quem amostra decide o ritmo; o anel só guarda.

use std::collections::{BTreeMap, VecDeque};

/// Uma linha do histórico.
#[derive(Clone, Debug, Default, PartialEq)]
struct Ring {
    samples: VecDeque<f32>,
    text: Option<String>,
    old: Option<String>,
}

/// ⭐ Os anéis de uma tabela, um por chave de linha, com `cap` amostras cada.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trails {
    cap: usize,
    rings: BTreeMap<String, Ring>,
}

/// O mínimo, o máximo e a última de uma série (o que o rato lê sobre o traço).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub min: f32,
    pub max: f32,
    pub now: f32,
}

impl Span {
    /// O resumo das amostras finitas de `samples` (`None` sem nenhuma).
    #[must_use]
    pub fn of(samples: &[f32]) -> Option<Self> {
        let mut it = samples.iter().copied().filter(|v| v.is_finite());
        let first = it.next()?;
        let (mut min, mut max, mut now) = (first, first, first);
        for v in it {
            min = min.min(v);
            max = max.max(v);
            now = v;
        }
        Some(Self { min, max, now })
    }
}

impl Trails {
    /// Anéis de `cap` amostras (no mínimo uma).
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            rings: BTreeMap::new(),
        }
    }

    /// Uma amostra: cada `(chave, texto, número)` entra no anel dela (um texto sem número limpa o
    /// traço); as chaves que não vieram saem.
    pub fn sample(&mut self, now: impl IntoIterator<Item = (String, String, Option<f32>)>) {
        let mut next = BTreeMap::new();
        for (key, text, number) in now {
            let mut ring = self.rings.remove(&key).unwrap_or_default();
            ring.old = ring.text.replace(text.clone()).filter(|t| *t != text);
            match number.filter(|v| v.is_finite()) {
                Some(v) => {
                    ring.samples.push_back(v);
                    while ring.samples.len() > self.cap.max(1) {
                        ring.samples.pop_front();
                    }
                }
                None => ring.samples.clear(),
            }
            next.insert(key, ring);
        }
        self.rings = next;
    }

    /// As amostras de `key`, a mais antiga primeiro (`None` se não é número).
    #[must_use]
    pub fn samples(&self, key: &str) -> Option<Vec<f32>> {
        let r = self.rings.get(key)?;
        (!r.samples.is_empty()).then(|| r.samples.iter().copied().collect())
    }

    /// O texto da amostra anterior de `key`, se a última o mudou.
    #[must_use]
    pub fn old(&self, key: &str) -> Option<&str> {
        self.rings.get(key)?.old.as_deref()
    }

    /// Quantas amostras cabem num anel.
    #[must_use]
    pub fn cap(&self) -> usize {
        self.cap
    }
}
