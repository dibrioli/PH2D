//! `LayerSnapshot` — cache de layer texture tirado a cada N strokes (default 50)
//! pra acelerar undo profundo. ADR-0046 §2.6.
//!
//! Snapshots vivem **fora** de [`crate::history::StrokeHistory`] — history é
//! semântica de input, snapshots são cache derivada. Separar permite
//! invalidar cache sem tocar history (LRU eviction sob memory pressure;
//! offload disk quando suportado pela plataforma).
//!
//! Caps congelados pelo gate `painter_contract_surface::stroke_history`:
//! - [`LayerSnapshot`] ≤ 8 fields
//! - [`SnapshotStorage`] ≤ 4 variants

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::device::LayerId;

/// Snapshot de uma layer raster em um ponto da history.
///
/// Identificada por `(layer_id, at_seq)` — undo do stroke N carrega o
/// snapshot mais recente com `at_seq ≤ N` e re-roda ≤ snapshot_interval
/// strokes pra chegar ao estado pré-stroke N.
///
/// `texture_blake3` é content-addressed — snapshots idênticos (canvas
/// estável entre strokes triviais) deduplicam naturalmente.
///
/// `version` é sidecar version (NÃO HR-14 obrigatório — regenerável).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LayerSnapshot {
    /// Após qual stroke seq foi tirado o snapshot (boundary upstream).
    pub at_seq: u64,
    /// Layer de origem.
    pub layer_id: LayerId,
    /// blake3 do conteúdo da texture (dedup + integrity).
    pub texture_blake3: [u8; 32],
    /// Onde os bytes vivem (RAM ou disk).
    pub texture_data: SnapshotStorage,
    /// Wall-clock ms — UX last-used + debug.
    pub timestamp_ms: u64,
    /// Sidecar version (v1 = 1). Regenerável; v2 reader pode ignorar v1.
    pub version: u32,
    // === 2 slots de headroom ===
    // - Compressed zstd in-mem
    // - Tier (preview vs full) pra LRU policy
}

/// Storage backend do snapshot. Compactação + offload disk são W11+ polish.
///
/// **Audit T1.8 L2-F6 — Windows non-UTF-8 path note:** `PathBuf: Serialize`
/// via serde-`postcard` usa `Path::to_str()` style fallback. Em Windows,
/// paths podem conter sequências UTF-16 non-roundtrippable em UTF-8 (raro;
/// usuário renomeou pasta via API Win32 antiga) — neste caso postcard
/// retorna `Err` em save. T1.8 ship aceita o trade-off (UTF-8 path = 99.9%
/// dos casos); W14+ candidate: wrapper `OsPathBytes(Vec<u8>)` que armazena
/// raw bytes + helper de conversão lossy. Integration test
/// `snapshot_ondisk_postcard_roundtrip` cobre o caminho UTF-8 happy path.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SnapshotStorage {
    /// Bytes em RAM. RGBA8 ou RGBA16F per canvas color profile.
    InMemory(Vec<u8>),
    /// Offload disk (LRU). Filesystem path absoluto.
    /// Requer UTF-8-roundtrippable (vide nota do enum).
    OnDisk(PathBuf),
    // === 2 slots de headroom ===
    // - CompressedInMemory(Vec<u8>, CompressionAlgo)
    // - OnObjectStore(BlobRef) — cloud assets W17
}

/// Versão atual do sidecar `.ph2d-painter-cache` (NÃO é HR-14 obrigatório).
pub const SNAPSHOT_VERSION: u32 = 1;

/// Intervalo default entre snapshots em strokes (ADR-0046 §2.6).
/// 50 strokes ≈ ≤ 1 segundo p99 pra undo num canvas 4K.
pub const SNAPSHOT_STROKE_INTERVAL: u64 = 50;

impl LayerSnapshot {
    /// Construtor mínimo. `timestamp_ms` zera — caller seta com wall-clock real.
    pub fn new_in_memory(layer_id: LayerId, at_seq: u64, bytes: Vec<u8>) -> Self {
        let texture_blake3 = blake3::hash(&bytes);
        Self {
            at_seq,
            layer_id,
            texture_blake3: *texture_blake3.as_bytes(),
            texture_data: SnapshotStorage::InMemory(bytes),
            timestamp_ms: 0,
            version: SNAPSHOT_VERSION,
        }
    }

    /// `true` se o snapshot vive em RAM (não offloaded).
    pub fn is_in_memory(&self) -> bool {
        matches!(self.texture_data, SnapshotStorage::InMemory(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_in_memory_computes_blake3_correctly() {
        let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let snap = LayerSnapshot::new_in_memory(LayerId(7), 42, bytes.clone());
        let expected = blake3::hash(&bytes);
        assert_eq!(snap.texture_blake3, *expected.as_bytes());
        assert_eq!(snap.at_seq, 42);
        assert_eq!(snap.layer_id, LayerId(7));
        assert!(snap.is_in_memory());
        assert_eq!(snap.version, SNAPSHOT_VERSION);
    }

    #[test]
    fn snapshot_roundtrips_postcard() {
        let snap = LayerSnapshot::new_in_memory(LayerId(1), 100, vec![1, 2, 3]);
        let bytes = postcard::to_allocvec(&snap).unwrap();
        let back: LayerSnapshot = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(snap, back);
    }
}
