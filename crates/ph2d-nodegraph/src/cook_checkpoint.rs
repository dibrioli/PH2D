//! **O PONTO DE RECUO e a MEMÓRIA DE UM NÓ** — filho do `cook` pelo tecto de LOC (lê os campos
//! privados do [`Cook`]).
//!
//! ⭐ A memória de um nó (doc 121 §9.20) é o estado que um nó guarda entre avaliações e que NÃO cabe
//! em colunas — o mundo do motor de contacto do `sim.step` (os contactos persistentes e o
//! aquecimento do solver). ⚠️ Ela viaja no [`CookCheckpoint`]: sem isso um recuo seguido de Play
//! mostraria OUTRA simulação, e o anel do recuo (ADR-0137) assenta em *«para um grafo fixo a sim é
//! função pura do tique»*.

use super::{Cook, NodeId, ScopeKey};
use crate::value::CookValue;
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

/// **O que um nó pode guardar entre avaliações** — ver [`super::EvalCtx::take_memo`].
///
/// `Clone` é a cópia que um ponto de recuo guarda; `approx_bytes` é o que o anel de recuo cobra por
/// ela (ADR-0137: um orçamento por CONTAGEM é um multiplicador — e um mundo de contacto de `4 096`
/// peças pesa megabytes, não as colunas que o acompanham).
pub trait Memo: Clone + Send + Sync + 'static {
    fn approx_bytes(&self) -> usize;
}

/// A face apagada do [`Memo`] que o `Cook` guarda por `(nó, escopo)`.
pub(crate) trait DynMemo: Send + Sync {
    fn bytes(&self) -> usize;
    fn clone_box(&self) -> Box<dyn DynMemo>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Memo> DynMemo for T {
    fn bytes(&self) -> usize {
        self.approx_bytes()
    }
    fn clone_box(&self) -> Box<dyn DynMemo> {
        Box::new(self.clone())
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// A snapshot of the **simulation state** carried across ticks — the `pre`-edge
/// feedback ([`Cook::prev_outputs`]), the sequential tick counter and the nodes'
/// [`Memo`]s — captured by [`Cook::checkpoint`] and reinstated by
/// [`Cook::restore`] (plan §1.4, M2.N2). Enough to reproduce any later frame by
/// restoring and re-cooking forward: GGPO's *"buffer sufficient to restore"*, and
/// bit-exact here because the cook is deterministic (no transcendentals, hashed
/// RNG — ADR-0032, HR-5).
///
/// **Why this is all of it:** every sequential node carries its recurrence in
/// stream columns on its `pre` self-loop (ADR-0032), and the one kind of state
/// that does not fit a column — a node's [`Memo`] — rides here too. The memo
/// cache and the revision clock are derivable and are deliberately excluded (the
/// cache is stale for a rewound clock; the revision clock stays live so a restore
/// reads as a change and redraws).
///
/// The clone of the columns shares their `Arc`s; the memos are DEEP copies (a
/// node mutates its own in place), which is why a graph with memos is
/// checkpointed sparsely (`ph2d-eval-motion`, `MEMO_A_CADA`).
#[derive(Clone, Default)]
pub struct CookCheckpoint {
    prev_outputs: BTreeMap<NodeId, Vec<CookValue>>,
    tick: u64,
    /// The clock the last tick closed on — restored with the state, so a replayed tick takes
    /// exactly the `dt` it took the first time.
    prev_playhead: Option<f64>,
    memo: BTreeMap<(NodeId, ScopeKey), Arc<dyn DynMemo>>,
}

/// O que o `malloc` acrescenta a cada pedido (cabeçalho e arredondamento, glibc) — a régua
/// `ph2d-eval-motion/tests/anel_memoria.rs` mede `bytes + 16 · blocos`.
const CABECALHO: usize = 16;

fn bloco(n: usize) -> usize {
    if n == 0 { 0 } else { n + CABECALHO }
}

/// Os nós de uma `BTreeMap<K, V>` de `n` entradas: as folhas do `std` têm `11` lugares (pai,
/// índice, contagem e as duas filas), cheias como as deixa um `collect` ordenado — e um clone tem
/// a forma do original —; acima de uma folha, um nó interno (mais `12` ponteiros) por `11` folhas.
fn peso_do_mapa<K, V>(n: usize) -> usize {
    const LUGARES: usize = 11;
    if n == 0 {
        return 0;
    }
    let folha = (12 + LUGARES * (size_of::<K>() + size_of::<V>())).next_multiple_of(8);
    let folhas = n.div_ceil(LUGARES);
    let internos = if folhas > 1 {
        folhas.div_ceil(LUGARES)
    } else {
        0
    };
    folhas * bloco(folha) + internos * bloco(folha + (LUGARES + 1) * size_of::<usize>())
}

/// O que UM valor de saída retém num ponto de recuo: o mapa de colunas do `Stream` e os nomes
/// (copiados pelo clone) e cada coluna (o `Arc` e os dados — partilhados com o cozimento só até ao
/// tique seguinte, que escreve colunas novas). Um `Opaque` é apagado de tipo e conta `0`.
fn peso_do_valor(v: &CookValue) -> usize {
    let CookValue::Instances(s) = v else {
        return 0;
    };
    let colunas: usize = s
        .shared_columns()
        .map(|(nome, c)| {
            bloco(nome.len())
                + bloco(2 * size_of::<usize>() + size_of::<crate::attr::Column>())
                + bloco(c.approx_bytes())
        })
        .sum();
    peso_do_mapa::<String, Arc<crate::attr::Column>>(s.shared_columns().count()) + colunas
}

impl CookCheckpoint {
    /// ⭐ **O heap que este ponto de recuo RETÉM, com o cabeçalho do `malloc`** — o que um anel com
    /// orçamento em bytes cobra por ele (ADR-0137): os mapas que o clone copia, os vetores de
    /// saídas, os nomes e as colunas, e cada [`Memo`] (o `Arc` da cópia e o que ela declara). Gate:
    /// `ph2d-eval-motion/tests/anel_memoria.rs` (`±25 %`; antes desta conta um ponto leve declarava
    /// só os dados — `8 B` — e retinha `~970`, doc 132 §7). ⚠️ Uma coluna partilhada por dois
    /// pontos conta nos dois: erra para CIMA, o lado que um orçamento pode errar.
    pub fn approx_bytes(&self) -> usize {
        let saidas: usize = self
            .prev_outputs
            .values()
            .map(|vs| {
                bloco(size_of_val(vs.as_slice())) + vs.iter().map(peso_do_valor).sum::<usize>()
            })
            .sum();
        let memos: usize = self
            .memo
            .values()
            .map(|m| bloco(2 * size_of::<usize>() + size_of_val(&**m)) + m.bytes())
            .sum();
        peso_do_mapa::<NodeId, Vec<CookValue>>(self.prev_outputs.len())
            + saidas
            + peso_do_mapa::<(NodeId, ScopeKey), Arc<dyn DynMemo>>(self.memo.len())
            + memos
    }
}

impl Cook {
    /// Capture the current simulation state — the `pre` feedback + the sequential
    /// tick + the nodes' memos — for later [`Self::restore`] (plan §1.4, M2.N2).
    /// Take it at the point in the tick loop where cooking `target` would
    /// reproduce a specific frame: i.e. **before that frame's `cook`**, which is
    /// exactly the state left by the previous frame's [`Self::advance_tick`]. Then
    /// a scrub is `restore(nearest checkpoint ≤ target)` followed by `cook;
    /// advance_tick` forward to `target` — bit-exact, because it walks the
    /// identical cook path as forward playback (GGPO save/load/advance).
    pub fn checkpoint(&self) -> CookCheckpoint {
        CookCheckpoint {
            prev_outputs: self.prev_outputs.clone(),
            prev_playhead: self.prev_playhead,
            tick: self.tick,
            memo: self
                .memo
                .iter()
                .map(|(k, m)| (*k, Arc::from(m.clone_box())))
                .collect(),
        }
    }

    /// Reinstate a [`Self::checkpoint`]ed simulation state, so the next `cook`
    /// reproduces the frame that checkpoint was taken before. The memo cache is
    /// **cleared** — its entries are stale for a rewound clock (a sequential
    /// node's fingerprint keys on the tick, a `Temporal` node's on the playhead,
    /// both of which just jumped), so a stale hit would serve a future frame
    /// (GGPO's *"invalidate the forward memo"*). The monotonic revision clock is
    /// **kept** so the recompute reads as a change downstream and the scene
    /// redraws. Scope lanes are dropped with the cache; the nodes' memos come
    /// back as deep copies (the checkpoint stays valid for the next restore).
    pub fn restore(&mut self, cp: &CookCheckpoint) {
        self.prev_outputs = cp.prev_outputs.clone();
        self.prev_playhead = cp.prev_playhead;
        self.tick = cp.tick;
        self.memo = cp.memo.iter().map(|(k, m)| (*k, m.clone_box())).collect();
        self.cache.clear();
        self.live_keys.clear();
    }

    /// **Algum nó guarda memória?** — quem regista pontos de recuo espaça-os quando sim, porque cada
    /// um é uma cópia funda dela (ver [`CookCheckpoint`]).
    #[must_use]
    pub fn has_memo(&self) -> bool {
        !self.memo.is_empty()
    }
}
