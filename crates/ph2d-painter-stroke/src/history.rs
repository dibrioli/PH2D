//! `StrokeHistory` — coleção de [`StrokeRecord`]s com semântica de **input**
//! (o que a usuária fez), independente do **cache** de snapshots
//! (vide [`crate::snapshot`]). ADR-0046 §2.5.
//!
//! Duas politicas v1:
//!
//! - [`StrokeHistory::Full`] — desktop / iPad Pro M1+ / Android top-tier
//!   (`memory_budget.painter.stroke_history_mb ≥ 150`). Sem cap arbitrário;
//!   cresce com a sessão. Eviction NÃO acontece automática (é history,
//!   não cache); apenas snapshots são LRU.
//! - [`StrokeHistory::Ring`] — fallback low-end (Web 200 MB total,
//!   Android entry, iPad ≤ 2018). Ring buffer com cap configurável (default
//!   1000). UX deve mostrar warning honesto: "Stroke history limited on
//!   this device; export to keep full history."
//!
//! 2 slots de headroom reservados pra `Branched` (experiments) e `External`
//! (offload disk full-async) — cap total = 4.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::record::StrokeRecord;

/// History de strokes de um canvas. Source of truth da pintura vetorial.
///
/// **Audit T1.8 L4-H4 — `#[non_exhaustive]`:** permite adicionar variants
/// `Branched`/`External` futuros sem semver break em downstream `match`.
/// Construtor manual `StrokeHistory::Ring { records, cap }` continua público
/// (variants têm pub fields pra postcard) MAS `StrokeHistory::ring(cap)` é
/// o caminho oficial (clampa cap em `RING_MIN_CAP`); construtor manual com
/// `cap=0` é detectado em [`Self::push`] via `debug_assert!` + release-mode
/// fallback `effective_cap = max(cap, RING_MIN_CAP)`.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StrokeHistory {
    /// Desktop / high-end mobile. Sem cap — cresce com a sessão.
    Full(Vec<StrokeRecord>),
    /// Low-end fallback. Ring buffer com cap explícito; eviction FIFO.
    Ring {
        records: VecDeque<StrokeRecord>,
        cap: usize,
    },
    // === 2 slots de headroom ===
    // - Branched(Vec<Branch>)        — experiments / what-if (W14+)
    // - External(ExternalHistoryHandle) — offload disk full-async (W17 polish)
}

/// Default = `Full` vazio. Runtime escolhe a politica em [`Self::for_budget`].
impl Default for StrokeHistory {
    fn default() -> Self {
        Self::Full(Vec::new())
    }
}

/// Cap mínimo seguro pra `Ring` (evita `cap = 0` virar no-op silencioso).
pub const RING_MIN_CAP: usize = 1;

/// Threshold default pra escolher `Full` vs `Ring` (ADR-0046 §2.5 tabela).
/// `painter.stroke_history_mb ≥ 150` → `Full`; senão `Ring`.
pub const FULL_HISTORY_MIN_BUDGET_MB: u32 = 150;

/// Cap default para `Ring` em devices mid-tier (50..149 MB). ADR-0046 §2.5 tabela.
pub const RING_CAP_MID_TIER: usize = 5_000;

/// Cap default para `Ring` em devices low-tier (< 50 MB).
pub const RING_CAP_LOW_TIER: usize = 1_000;

impl StrokeHistory {
    /// Decide politica baseada no budget Painter (`PainterMemoryBudget::stroke_history_mb`).
    ///
    /// Tabela ADR-0046 §2.5:
    ///
    /// | Budget        | Modo          | Cap                |
    /// |---------------|---------------|---------------------|
    /// | ≥ 150 MB      | `Full`        | sem cap (subject ao budget global) |
    /// | 50-149 MB     | `Ring`        | 5000 (~30 MB)       |
    /// | < 50 MB       | `Ring`        | 1000 (~6 MB)        |
    pub fn for_budget(stroke_history_mb: u32) -> Self {
        if stroke_history_mb >= FULL_HISTORY_MIN_BUDGET_MB {
            // Audit T-durability final N-4: pré-alocar Vec capacity proporcional
            // ao budget evita 15 reallocs cumulativos (1→2→4→…→32k) cada um
            // copiando ~600B × N samples. Realloc final 16k→32k = ~10MB
            // memcpy spike que estoura frame budget 3.5ms em iPad.
            //
            // Heuristic: 50% do budget upfront. `StrokeRecord` ≈ 600B base
            // + samples inline. Budget 150 MB / 600B = 262k strokes; 50% =
            // 131k preallocated. Memory cost no startup: ~80 MB de ponteiros
            // Vec backing (sem dados reais até push).
            const RECORD_SIZE_ESTIMATE: usize = 600;
            let cap = (stroke_history_mb as usize)
                .saturating_mul(1024 * 1024)
                .saturating_div(RECORD_SIZE_ESTIMATE * 2);
            Self::Full(Vec::with_capacity(cap.max(1024)))
        } else if stroke_history_mb >= 50 {
            Self::ring(RING_CAP_MID_TIER)
        } else {
            Self::ring(RING_CAP_LOW_TIER)
        }
    }

    /// Construtor seguro pra `Ring` — clampa `cap` em [`RING_MIN_CAP`].
    /// Audit T1.8 (L1-F6/L2-F2): construtor manual `StrokeHistory::Ring {
    /// cap: 0, … }` virava no-op silencioso (`len >= 0` sempre true →
    /// pingue-pongue evicta cada push). `for_budget` agora rota tudo
    /// por aqui.
    pub fn ring(cap: usize) -> Self {
        let cap = cap.max(RING_MIN_CAP);
        Self::Ring {
            records: VecDeque::with_capacity(cap),
            cap,
        }
    }

    /// Empurra um stroke novo. Em `Ring` cheio, evicta o mais antigo (FIFO).
    ///
    /// Retorna o stroke evictado (se houver) — caller pode logar ou persistir
    /// no `.ph2d-painter-cache` antes do drop.
    pub fn push(&mut self, record: StrokeRecord) -> Option<StrokeRecord> {
        match self {
            Self::Full(v) => {
                v.push(record);
                None
            }
            Self::Ring { records, cap } => {
                // Defesa em profundidade — caller pode ter deserializado
                // canon corrupto com cap=0. Audit T1.8 L1-F6/L2-F2.
                debug_assert!(
                    *cap >= RING_MIN_CAP,
                    "Ring cap < RING_MIN_CAP={} viola invariante (got {}); use StrokeHistory::ring() ou StrokeHistory::for_budget() em vez de construtor manual",
                    RING_MIN_CAP,
                    cap
                );
                let effective_cap = (*cap).max(RING_MIN_CAP);
                let evicted = if records.len() >= effective_cap {
                    records.pop_front()
                } else {
                    None
                };
                records.push_back(record);
                evicted
            }
        }
    }

    /// Pop do último stroke. Anchor pro undo.
    pub fn undo(&mut self) -> Option<StrokeRecord> {
        match self {
            Self::Full(v) => v.pop(),
            Self::Ring { records, .. } => records.pop_back(),
        }
    }

    /// Re-empurra um stroke previamente removido por `undo`. Anchor pro redo.
    ///
    /// Em `Ring`, redo NÃO restaura o cap evictado (esse stroke já foi perdido
    /// pelo eviction FIFO); apenas re-empurra `record` ao topo respeitando o cap.
    ///
    /// **Audit T1.8 L3-G5 — redo em Ring pode evictar stroke não-relacionado:**
    /// se entre `undo()` e `redo()` o caller fez `push()` de strokes novos,
    /// o redo empurra o `record` original mas evicta o stroke mais antigo do
    /// FIFO (que pode ter sido um stroke "real" do usuário, não o que estava
    /// undo-stacked). Retorno `Option<StrokeRecord>` permite caller logar
    /// warning UX nesse caso. Long-term (T1.9+) UX correto: separar
    /// undo-stack do history Ring — `StrokeHistory` é o canon do canvas,
    /// undo-stack é estado UI volátil.
    pub fn redo(&mut self, record: StrokeRecord) -> Option<StrokeRecord> {
        self.push(record)
    }

    /// Quantidade de strokes presentes.
    pub fn len(&self) -> usize {
        match self {
            Self::Full(v) => v.len(),
            Self::Ring { records, .. } => records.len(),
        }
    }

    /// `true` se vazio.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Maior `seq` presente, ou `None` se vazio. Anchor pra
    /// `CrashRecovery::scan_with_baseline` evitar replay attack (audit
    /// T-durability K-4) + evitar duplicate-replay após save (audit M-1).
    ///
    /// O(N) — `seq` é monotônico per-canvas mas iteramos pra robustez
    /// (WAL adversarial pode ter strokes fora-de-ordem).
    pub fn max_seq(&self) -> Option<u64> {
        self.iter().map(|r| r.seq).max()
    }

    /// Iterator em ordem cronológica (mais antigo → mais novo).
    ///
    /// **Audit T1.8 L4-H1 — enum dispatch (sem `Box<dyn>` alloc):** retorna
    /// tipo nominal [`StrokeHistoryIter`] em vez de `Box<dyn Iterator>`,
    /// eliminando 1 alloc por chamada + vtable indirect. Hot path
    /// (Reproject W12, snapshot rebuild) chama por frame sobre 20k strokes.
    pub fn iter(&self) -> StrokeHistoryIter<'_> {
        match self {
            Self::Full(v) => StrokeHistoryIter::Full(v.iter()),
            Self::Ring { records, .. } => StrokeHistoryIter::Ring(records.iter()),
        }
    }
}

/// Iterator nominal sobre [`StrokeHistory`]. Audit T1.8 L4-H1: enum dispatch
/// elimina `Box<dyn>` por chamada de `iter()`.
pub enum StrokeHistoryIter<'a> {
    Full(std::slice::Iter<'a, StrokeRecord>),
    Ring(std::collections::vec_deque::Iter<'a, StrokeRecord>),
}

impl<'a> Iterator for StrokeHistoryIter<'a> {
    type Item = &'a StrokeRecord;
    fn next(&mut self) -> Option<&'a StrokeRecord> {
        match self {
            Self::Full(it) => it.next(),
            Self::Ring(it) => it.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Full(it) => it.size_hint(),
            Self::Ring(it) => it.size_hint(),
        }
    }
}

impl ExactSizeIterator for StrokeHistoryIter<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Full(it) => it.len(),
            Self::Ring(it) => it.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_color::OklchColor;
    use ph2d_painter_brush::BrushHandle;

    use crate::device::LayerId;

    fn rec(seq: u64) -> StrokeRecord {
        StrokeRecord::new(
            seq,
            BrushHandle::default(),
            [0u8; 32],
            LayerId::default(),
            OklchColor::default(),
        )
    }

    #[test]
    fn for_budget_picks_full_at_or_above_150() {
        let h = StrokeHistory::for_budget(150);
        assert!(matches!(h, StrokeHistory::Full(_)));
        let h = StrokeHistory::for_budget(1_024);
        assert!(matches!(h, StrokeHistory::Full(_)));
    }

    #[test]
    fn for_budget_picks_mid_ring_50_to_149() {
        let h = StrokeHistory::for_budget(100);
        assert!(matches!(
            h,
            StrokeHistory::Ring {
                cap: RING_CAP_MID_TIER,
                ..
            }
        ));
    }

    #[test]
    fn for_budget_picks_low_ring_below_50() {
        let h = StrokeHistory::for_budget(0);
        assert!(matches!(
            h,
            StrokeHistory::Ring {
                cap: RING_CAP_LOW_TIER,
                ..
            }
        ));
        let h = StrokeHistory::for_budget(49);
        assert!(matches!(
            h,
            StrokeHistory::Ring {
                cap: RING_CAP_LOW_TIER,
                ..
            }
        ));
    }

    #[test]
    fn push_full_grows_unbounded() {
        let mut h = StrokeHistory::default();
        for i in 0..10 {
            assert!(h.push(rec(i)).is_none());
        }
        assert_eq!(h.len(), 10);
    }

    #[test]
    fn push_ring_evicts_when_full() {
        let mut h = StrokeHistory::Ring {
            records: VecDeque::with_capacity(3),
            cap: 3,
        };
        assert!(h.push(rec(0)).is_none());
        assert!(h.push(rec(1)).is_none());
        assert!(h.push(rec(2)).is_none());
        // 4º empurra evicta o 0º.
        let evicted = h.push(rec(3));
        assert!(evicted.is_some());
        assert_eq!(evicted.unwrap().seq, 0);
        assert_eq!(h.len(), 3);
    }

    #[test]
    fn undo_pops_last_full() {
        let mut h = StrokeHistory::default();
        h.push(rec(0));
        h.push(rec(1));
        h.push(rec(2));
        assert_eq!(h.undo().unwrap().seq, 2);
        assert_eq!(h.undo().unwrap().seq, 1);
        assert_eq!(h.undo().unwrap().seq, 0);
        assert!(h.undo().is_none());
    }

    #[test]
    fn undo_pops_last_ring() {
        let mut h = StrokeHistory::Ring {
            records: VecDeque::with_capacity(2),
            cap: 2,
        };
        h.push(rec(0));
        h.push(rec(1));
        assert_eq!(h.undo().unwrap().seq, 1);
        assert_eq!(h.undo().unwrap().seq, 0);
        assert!(h.undo().is_none());
    }

    #[test]
    fn redo_restores_to_top() {
        let mut h = StrokeHistory::default();
        h.push(rec(0));
        let popped = h.undo().unwrap();
        h.redo(popped);
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn iter_yields_chronological_order() {
        let mut h = StrokeHistory::default();
        for i in 0..5 {
            h.push(rec(i));
        }
        let seqs: Vec<_> = h.iter().map(|r| r.seq).collect();
        assert_eq!(seqs, vec![0, 1, 2, 3, 4]);
    }
}
