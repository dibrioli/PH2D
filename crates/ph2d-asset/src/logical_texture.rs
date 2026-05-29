//! W1.T4 (ADR-0055-v4 §2 "Identidade multi-tier") — `LogicalTextureId` +
//! `LogicalTextureMap`.
//!
//! **Problema que resolve:** PH2D HR-6 manda `AssetId = blake3(bytes)` (content-
//! addressed identity). Mas cooker offline emite **N artefatos KTX2 distintos
//! por source PNG** (1 per tier conforme target matrix em
//! `tools/asset-cooker/src/texture/target_matrix.rs`). Cada artefato cooked tem
//! byte-content distinto → N AssetIds blake3 distintos. Cliente do renderer não
//! quer N AssetIds — quer **1 identidade lógica** ("essa é a textura do hero
//! sprite") e resolve qual variant carregar baseado em `DeviceTier` runtime.
//!
//! **Solução:** mapping externo `LogicalTextureId → BTreeMap<TierIndex, AssetId>`
//! — `ph2d-asset`-internal, sem amendment a `AssetDb` (que continua content-
//! addressed). Renderer faz:
//!
//! ```ignore
//! let asset_id = logical_texture_map.resolve(logical_id, current_tier)?;
//! let asset = asset_db.get(&asset_id)?;
//! ```
//!
//! `LogicalTextureId` = `blake3(source PNG bytes)`. **Distinto** de `AssetId`
//! que indexa o cooked KTX2 (cuja hash inclui tier-specific encoder output).
//! Convenção: source-hash → uma única logical ID estável across all tiers.
//!
//! Audit Lente δ W1.T3: §Symbol Registry registra esta entry como "W1.T4 cria".

use crate::{AssetId, TierIndex};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Identidade lógica de uma textura — hash blake3 dos bytes do source PNG.
///
/// Estável across tiers (mesmo source → mesmo `LogicalTextureId` em Desktop /
/// Mobile / Web / etc.). Distinto de [`AssetId`] que indexa o cooked KTX2
/// byte-content (varia per tier).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogicalTextureId([u8; 32]);

impl LogicalTextureId {
    /// Hash blake3 dos bytes do source (e.g., bytes do PNG bruto, antes do cook).
    #[must_use]
    pub fn from_source_bytes(bytes: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(bytes);
        Self(*hasher.finalize().as_bytes())
    }

    /// Constrói diretamente de um digest blake3 já calculado.
    #[must_use]
    pub const fn from_digest(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    /// Bytes raw (32) — útil pra serialização / comparação.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Representação hex de 64 chars (mesma convenção de `AssetId`).
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut s = String::with_capacity(64);
        for byte in self.0 {
            s.push_str(&format!("{byte:02x}"));
        }
        s
    }
}

impl std::fmt::Display for LogicalTextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Primeiros 8 chars hex — suficiente pra debug overlays; full hex via .to_hex().
        for byte in &self.0[..4] {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Mapping `LogicalTextureId → BTreeMap<TierIndex, AssetId>`. Renderer consulta
/// pra resolver "qual AssetId carregar pra esse logical id no tier atual?".
///
/// `BTreeMap` (não `HashMap`) por HR-6: iteration order determinístico se algum
/// path serializar este mapping (e.g., cooked manifest JSON5).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LogicalTextureMap {
    entries: BTreeMap<LogicalTextureId, BTreeMap<TierIndex, AssetId>>,
}

impl LogicalTextureMap {
    /// Construtor vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra um `AssetId` cooked para um `(LogicalTextureId, TierIndex)`. Se
    /// já existir entry para essa (logical, tier), o `AssetId` anterior é
    /// substituído (caller responsável por idempotência semantic).
    pub fn insert(
        &mut self,
        logical: LogicalTextureId,
        tier: TierIndex,
        asset_id: AssetId,
    ) -> Option<AssetId> {
        self.entries
            .entry(logical)
            .or_default()
            .insert(tier, asset_id)
    }

    /// Resolve `(logical, tier) → AssetId` lookup.
    #[must_use]
    pub fn resolve(&self, logical: LogicalTextureId, tier: TierIndex) -> Option<AssetId> {
        self.entries.get(&logical)?.get(&tier).copied()
    }

    /// Lista todos os tiers cookados para um `LogicalTextureId`. Útil em
    /// editor UI ("essa textura tem N variants disponíveis").
    #[must_use]
    pub fn available_tiers(&self, logical: LogicalTextureId) -> Vec<TierIndex> {
        self.entries
            .get(&logical)
            .map(|m| m.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Total de logical textures registradas.
    #[must_use]
    pub fn logical_count(&self) -> usize {
        self.entries.len()
    }

    /// Total de (logical, tier) entries. Diferencia de `logical_count` quando
    /// algumas textures têm múltiplos tiers cookados.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries.values().map(BTreeMap::len).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_texture_id_from_bytes_is_deterministic() {
        let id_a = LogicalTextureId::from_source_bytes(b"hello world");
        let id_b = LogicalTextureId::from_source_bytes(b"hello world");
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn logical_texture_id_distinguishes_inputs() {
        let id_a = LogicalTextureId::from_source_bytes(b"abc");
        let id_b = LogicalTextureId::from_source_bytes(b"abd");
        assert_ne!(id_a, id_b);
    }

    #[test]
    fn logical_texture_id_to_hex_length_64() {
        let id = LogicalTextureId::from_source_bytes(b"x");
        assert_eq!(id.to_hex().len(), 64);
    }

    #[test]
    fn map_insert_and_resolve_round_trip() {
        let mut map = LogicalTextureMap::new();
        let logical = LogicalTextureId::from_source_bytes(b"hero.png");
        let asset_desktop = AssetId::from_bytes(b"cooked-bc7-bytes");
        let asset_mobile = AssetId::from_bytes(b"cooked-astc-bytes");

        assert_eq!(map.insert(logical, TierIndex::DESKTOP, asset_desktop), None);
        assert_eq!(map.insert(logical, TierIndex::MOBILE, asset_mobile), None);

        assert_eq!(
            map.resolve(logical, TierIndex::DESKTOP),
            Some(asset_desktop)
        );
        assert_eq!(map.resolve(logical, TierIndex::MOBILE), Some(asset_mobile));
        assert_eq!(map.resolve(logical, TierIndex::WEB), None);
    }

    #[test]
    fn map_overwrite_returns_previous() {
        let mut map = LogicalTextureMap::new();
        let logical = LogicalTextureId::from_source_bytes(b"sprite.png");
        let asset_v1 = AssetId::from_bytes(b"old-cooked");
        let asset_v2 = AssetId::from_bytes(b"new-cooked");

        assert_eq!(map.insert(logical, TierIndex::DESKTOP, asset_v1), None);
        assert_eq!(
            map.insert(logical, TierIndex::DESKTOP, asset_v2),
            Some(asset_v1)
        );
        assert_eq!(map.resolve(logical, TierIndex::DESKTOP), Some(asset_v2));
    }

    #[test]
    fn map_available_tiers_lists_in_order() {
        let mut map = LogicalTextureMap::new();
        let logical = LogicalTextureId::from_source_bytes(b"sprite.png");

        map.insert(logical, TierIndex::WEB, AssetId::from_bytes(b"w"));
        map.insert(logical, TierIndex::DESKTOP, AssetId::from_bytes(b"d"));
        map.insert(logical, TierIndex::MOBILE, AssetId::from_bytes(b"m"));

        // BTreeMap ordering → 0 < 1 < 2 → Desktop, Mobile, Web.
        let tiers = map.available_tiers(logical);
        assert_eq!(
            tiers,
            vec![TierIndex::DESKTOP, TierIndex::MOBILE, TierIndex::WEB]
        );
    }

    #[test]
    fn map_counts() {
        let mut map = LogicalTextureMap::new();
        let hero = LogicalTextureId::from_source_bytes(b"hero.png");
        let villain = LogicalTextureId::from_source_bytes(b"villain.png");

        assert_eq!(map.logical_count(), 0);
        assert_eq!(map.entry_count(), 0);

        map.insert(hero, TierIndex::DESKTOP, AssetId::from_bytes(b"a"));
        map.insert(hero, TierIndex::MOBILE, AssetId::from_bytes(b"b"));
        map.insert(villain, TierIndex::DESKTOP, AssetId::from_bytes(b"c"));

        assert_eq!(map.logical_count(), 2);
        assert_eq!(map.entry_count(), 3);
    }

    #[test]
    fn map_round_trips_via_postcard() {
        let mut map = LogicalTextureMap::new();
        let logical = LogicalTextureId::from_source_bytes(b"hero.png");
        map.insert(logical, TierIndex::DESKTOP, AssetId::from_bytes(b"cooked"));

        let bytes = postcard::to_allocvec(&map).unwrap();
        let decoded: LogicalTextureMap = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(
            decoded.resolve(logical, TierIndex::DESKTOP),
            map.resolve(logical, TierIndex::DESKTOP)
        );
    }
}
