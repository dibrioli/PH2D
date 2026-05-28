//! `.ph2d-painter` (canon, HR-14 versionado) + `.ph2d-painter-cache` (sidecar,
//! regenerável). ADR-0046 §2.7.
//!
//! Separação congelada pela audit 2026-05-26 C-3:
//!
//! - **Canon** = essencial pra reconstruir o canvas (history + layer_stack +
//!   brush_snapshots). Forward-compat obrigatório (v2 reader **deve** ler v1).
//! - **Sidecar** = cache (snapshots + spatial_index). Pode ser deletado/
//!   regenerado a qualquer momento. Sem migration chain.
//!
//! Caps congelados:
//! - [`PaintProject`] ≤ 12 fields
//! - [`CanvasInfo`] ≤ 8 fields
//! - [`PaintProjectCache`] ≤ 8 fields

use ph2d_painter_brush::{Brush, BrushParamsHash};
use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION;
use crate::device::LayerStack;
use crate::history::StrokeHistory;
use crate::snapshot::LayerSnapshot;

/// Magic bytes do canon (`.ph2d-painter`). 12 bytes ASCII.
pub const PAINT_PROJECT_MAGIC: [u8; 12] = *b"PH2D-PAINTER";

/// Magic bytes do sidecar (`.ph2d-painter-cache`). 18 bytes ASCII.
pub const PAINT_PROJECT_CACHE_MAGIC: [u8; 18] = *b"PH2D-PAINTER-CACHE";

/// Canon savefile — fonte de verdade pra reconstruir o canvas.
///
/// HR-14 forward-compat: writer sempre emite latest version; reader v2+
/// deve ler v1 (migration helpers em
/// [`migrate_v1_to_v2`](fn@migrate_v1_to_v2) quando v2 nascer).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaintProject {
    /// "PH2D-PAINTER" magic ASCII — identifica formato.
    pub magic: [u8; 12],
    /// HR-14 schema version (v1 = 1).
    pub version: u32,
    /// Dimensões + color profile + ppm.
    pub canvas: CanvasInfo,
    /// Stack de layers (W3 preenche; T1.8 ship com stub LayerStack default).
    pub layer_stack: LayerStack,
    /// Source of truth vetorial.
    pub history: StrokeHistory,
    /// Tabela dedup de brushes referenciados em `history` via blake3.
    ///
    /// **Audit T1.8 L2-F7 — invariante de caller:** este Vec deve manter
    /// `(BrushParamsHash, Brush)` único por hash. Caller (`ph2d-tool-painter`
    /// em T1.9+) é responsável por checar `brush_snapshots.iter().any(|(h, _)| *h == hash)`
    /// antes de push. Vec foi escolhido (vs `BTreeMap`) por (a) postcard
    /// roundtrip determinístico simples, (b) sessões típicas têm < 100
    /// brushes ⇒ lookup linear é fine. Convenção stronger (BTreeMap ou
    /// newtype `BrushSnapshotTable`) é candidate W11 polish.
    pub brush_snapshots: Vec<(BrushParamsHash, Brush)>,
    /// Wall-clock ms since epoch — created.
    pub created_at: u64,
    /// Wall-clock ms since epoch — last modified.
    pub modified_at: u64,
    /// blake3 sobre concat([magic, version, canvas, layer_stack, history,
    /// brush_snapshots, created_at, modified_at]) — integrity guard.
    pub checksum: [u8; 32],
    // === 3 slots de headroom (e.g., signature, document_metadata, embedded_palette) ===
}

impl PaintProject {
    /// Cria um project vazio com magic + version + checksum dummy. Caller
    /// deve preencher canvas/history/brushes e recomputar checksum via
    /// [`Self::recompute_checksum`].
    pub fn new(canvas: CanvasInfo) -> Self {
        let mut p = Self {
            magic: PAINT_PROJECT_MAGIC,
            version: SCHEMA_VERSION,
            canvas,
            layer_stack: LayerStack::default(),
            history: StrokeHistory::default(),
            brush_snapshots: Vec::new(),
            created_at: 0,
            modified_at: 0,
            checksum: [0u8; 32],
        };
        p.recompute_checksum();
        p
    }

    /// Recomputa o checksum sobre todos os campos exceto o próprio checksum.
    /// Caller chama após qualquer mutação do canon.
    ///
    /// **Audit T1.8 L1-F1/L2-F5 + L3-G4 — safety-first clone:** versão
    /// L1/L2 tentou zero-clone (mutate `self.checksum`, serialize, restore)
    /// mas L3-G4 mostrou janela inconsistente: se `to_allocvec` PANIC
    /// (OOM em Vec::reserve), `self.checksum = [0u8; 32]` permanece e
    /// integrity guard fica auto-satisfeito mas semanticamente vazio.
    /// Re-adotamos clone-based pra preservar atomicidade: peak memory
    /// 2× serialized size em troca de invariante "panic não corrompe
    /// state". W11 follow-up: streaming serializer pra
    /// [`blake3::Hasher`] elimina ambos clone + Vec.
    pub fn recompute_checksum(&mut self) {
        let mut ghost = self.clone();
        ghost.checksum = [0u8; 32];
        if let Ok(bytes) = postcard::to_allocvec(&ghost) {
            let h = blake3::hash(&bytes);
            self.checksum = *h.as_bytes();
        }
        // Se Err OR panic do to_allocvec → self.checksum mantém valor
        // anterior (jamais corrompido). Sentinel test
        // `paint_project_checksum_self_verifies` valida o happy path.
    }

    /// `true` se o checksum gravado bate com o recomputado. Integrity guard
    /// pra leitura de arquivos corrompidos.
    ///
    /// **Não muta `self`** — clone-based como [`Self::recompute_checksum`]
    /// pra preservar invariante anti-panic. Aloca 1× serialized size;
    /// refactor streaming Hasher W11.
    pub fn verify_checksum(&self) -> bool {
        let saved = self.checksum;
        let mut ghost = self.clone();
        ghost.checksum = [0u8; 32];
        let bytes = match postcard::to_allocvec(&ghost) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let h = blake3::hash(&bytes);
        *h.as_bytes() == saved
    }
}

/// Carrega `PaintProject` de bytes postcard, aplicando migrations conforme
/// `version`. Audit T1.8 L1-F4 — receita HR-14 forward-compat documentada.
///
/// ## Como ESTENDER quando v2 nascer
///
/// 1. Bump [`crate::SCHEMA_VERSION`] = 2 + adicione field novo em
///    `PaintProject` (ou modifique semântica).
/// 2. Implemente [`migrate_v1_to_v2`] com a transformação concreta:
///    ```ignore
///    pub fn migrate_v1_to_v2(p: &mut PaintProject) {
///        // Exemplo: adiciona timestamp_zone derivado de modified_at.
///        // O field novo já existe em PaintProject com Default;
///        // aqui você preenche com valor não-default se aplicável.
///        p.version = 2;
///        // p.timestamp_zone = TimeZone::derive_from(p.modified_at);
///        p.recompute_checksum();
///    }
///    ```
/// 3. Atualize esta fn para chamar `migrate_v1_to_v2` quando
///    `loaded.version == 1`.
/// 4. Writer SEMPRE emite latest version (`SCHEMA_VERSION`).
///
/// ## Estratégia geral
///
/// - **Adicionar field:** trivial. Use `#[serde(default)]` no field novo;
///   v1 reader ignora unknown trailing bytes, v2 reader vê field ou
///   default. Não precisa migration helper.
/// - **Remover field:** complexo. Crie struct transient `PaintProjectV1`
///   com schema antigo, deserialize nele, copie campos remanescentes
///   pra `PaintProject` corrente. Helper aqui.
/// - **Mudar semântica:** documente em migration helper; bump version
///   força readers a re-derivar dados.
pub fn load(bytes: &[u8]) -> Result<PaintProject, LoadError> {
    let mut p: PaintProject = postcard::from_bytes(bytes).map_err(|_| LoadError::Postcard)?;
    if p.magic != PAINT_PROJECT_MAGIC {
        return Err(LoadError::WrongMagic);
    }
    // Apply migrations v1 → vN sequencialmente.
    match p.version {
        0 => return Err(LoadError::UnsupportedVersion(0)),
        1 => {
            // T1.8 ship é v1 — sem migration necessária.
        }
        v if v > SCHEMA_VERSION => {
            return Err(LoadError::FutureVersion {
                file: v,
                supported: SCHEMA_VERSION,
            });
        }
        v => {
            return Err(LoadError::UnknownVersion(v));
        }
    }
    if !p.verify_checksum() {
        return Err(LoadError::ChecksumMismatch);
    }
    // Ponto de extensão futuro: chain v1 → v2 → v3 etc.
    // if p.version == 1 { migrate_v1_to_v2(&mut p); }
    p.version = SCHEMA_VERSION; // pin to current após migrations
    Ok(p)
}

/// Erros de [`load`].
///
/// **Audit T1.8 L3-G9:** `#[non_exhaustive]` permite adicionar variants
/// novos sem semver break — downstream `match` precisa de `_ =>` arm.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// Bytes não são um `PaintProject` postcard válido.
    Postcard,
    /// Magic bytes não batem com `PH2D-PAINTER`.
    WrongMagic,
    /// Version 0 não é suportada (file pré-v1 não existe).
    UnsupportedVersion(u32),
    /// File version > SCHEMA_VERSION — reader atual desatualizado vs arquivo.
    FutureVersion { file: u32, supported: u32 },
    /// Version não reconhecida (gap na migration chain).
    /// **Audit T1.8 L3-G1:** unreachable em SCHEMA_VERSION=1 (todas versions
    /// no range são `0` ou `1` ou `>1`); preservado pra evolução v2+ quando
    /// gaps na migration chain forem possíveis. `#[allow(dead_code)]` no
    /// variant evita warning até alguma migration concreta exercitar.
    #[allow(dead_code)]
    UnknownVersion(u32),
    /// Checksum não bate — arquivo corrompido OU adulterado.
    ChecksumMismatch,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postcard => write!(f, "postcard deserialization failed"),
            Self::WrongMagic => write!(f, "magic bytes mismatch — not a .ph2d-painter file"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version {} (must be ≥ 1)", v),
            Self::FutureVersion { file, supported } => write!(
                f,
                "file version {} is newer than reader (supports up to {})",
                file, supported
            ),
            Self::UnknownVersion(v) => write!(f, "unknown version {} in migration chain", v),
            Self::ChecksumMismatch => write!(f, "checksum mismatch — file corrupted or tampered"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Metadados do canvas — dimensões + color profile + scale físico.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanvasInfo {
    /// Largura em pixels.
    pub width: u32,
    /// Altura em pixels.
    pub height: u32,
    /// Color profile (ADR-0051 enum FROZEN 8). Stub local até `ph2d-color::ColorProfile`
    /// materializar via T-color (ADR-0051) — vide [`ColorProfile`].
    pub color_profile: ColorProfile,
    /// Pixels per millimeter (scale físico). 11.811 = 300 DPI; 3.937 = 100 DPI.
    pub ppm: f32,
    // === 4 slots de headroom (created_dpi_hint, gamut_clip_policy,
    //                         author_intent, embedded_icc_blob_ref) ===
}

impl Default for CanvasInfo {
    fn default() -> Self {
        // 1024×1024 @ 300 DPI sRGB — default seguro pra prototyping.
        Self {
            width: 1024,
            height: 1024,
            color_profile: ColorProfile::default(),
            ppm: 11.811,
        }
    }
}

/// Color profile do canvas — STUB local até `ph2d-color::ColorProfile`
/// materializar via T-color (ADR-0051).
///
/// **Audit T1.8 L1-F10 — HR-14 forward-compat pin obrigatório:** quando
/// T-color materializar o enum FROZEN 8 em `ph2d-color` e este stub for
/// removido, o discriminant `Srgb` DEVE permanecer `= 0` literal. postcard
/// serializa variant como `varint(discriminant)`; mudar `Srgb` pra `= 5`
/// (por exemplo) faria todos `.ph2d-painter` v1 com `ColorProfile::Srgb`
/// deserializarem como variant errada SEM erro. Mitigação:
///
/// 1. Gate textual a ser adicionado em `painter_contract_surface` quando
///    `ph2d-color::ColorProfile` for criado: assert que `Srgb = 0` aparece
///    literal na source.
/// 2. Test [`color_profile_pin`](tests/forward_compat_pins.rs) (T1.8)
///    pinning byte de serialização — fail-fast se discriminant drift.
///
/// ADR-0051 §2.2 prevê 7 variants v1 + 1 slot reserved (FROZEN em 8):
/// sRGB, DisplayP3, Rec2020, ProPhoto, AdobeRGB, LinearSrgb (working),
/// Hdr10 (`SmpteSt2084` PQ), + 1 reserved. T1.8 expõe apenas o default
/// `Srgb` — quando T-color nascer, esse enum é deletado daqui e
/// `ph2d-color::ColorProfile` re-exportado por `pub use`.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ColorProfile {
    /// Display sRGB — default web/desktop (gamut compatibility universal).
    /// **DISCRIMINANT 0 RESERVADO:** vide docstring do enum.
    #[default]
    Srgb = 0,
}

/// Sidecar cache — pode ser deletado a qualquer momento sem perda de dados.
/// `source_blake3` aponta pro `.ph2d-painter` correspondente; mismatch =
/// cache descartado silenciosamente (regenerável).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaintProjectCache {
    /// "PH2D-PAINTER-CACHE" magic ASCII.
    pub magic: [u8; 18],
    /// Sidecar version. Sem migration chain — v2+ pode ignorar antigos.
    pub version: u32,
    /// blake3 do `.ph2d-painter` sibling no momento da geração. Invalida cache
    /// se mismatch.
    pub source_blake3: [u8; 32],
    /// Layer snapshots cacheados (LRU eviction sob memory pressure).
    pub snapshots: Vec<LayerSnapshot>,
    /// R-tree de bboxes pra lookup espacial (ADR-0048 Inspector W14).
    /// `None` enquanto W14 não materializar o tipo concreto — placeholder
    /// `Vec<u8>` aceita o slot sem comprometer o ABI postcard.
    pub spatial_index: Option<SerializedRTree>,
    /// Wall-clock ms — gerado em.
    pub generated_at: u64,
    // === 2 slots de headroom ===
}

impl PaintProjectCache {
    /// Cria um cache vazio pra um canon dado. `source_blake3` deve bater com
    /// `canon_blake3` calculado pelo caller (não recomputado aqui — depende
    /// do canon na disk file final, não do struct in-mem).
    pub fn new(source_blake3: [u8; 32]) -> Self {
        Self {
            magic: PAINT_PROJECT_CACHE_MAGIC,
            version: crate::snapshot::SNAPSHOT_VERSION,
            source_blake3,
            snapshots: Vec::new(),
            spatial_index: None,
            generated_at: 0,
        }
    }

    /// `true` se o cache foi gerado a partir do canon dado pelo seu `blake3`.
    pub fn matches_canon(&self, canon_blake3: &[u8; 32]) -> bool {
        &self.source_blake3 == canon_blake3
    }
}

/// Placeholder pra R-tree serializado (W14 Inspector materializa).
pub type SerializedRTree = Vec<u8>;

/// Migration v1 → v2: hoje no-op (v2 ainda não existe). Quando v2 nascer,
/// caller chama esta função pra adaptar canon antigo, depois `recompute_checksum`.
pub fn migrate_v1_to_v2(_p: &mut PaintProject) {
    // v2 inexistente em T1.8. Helper stub pra documentar o contrato HR-14.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_project_new_initializes_magic_and_version() {
        let p = PaintProject::new(CanvasInfo::default());
        assert_eq!(p.magic, PAINT_PROJECT_MAGIC);
        assert_eq!(p.version, SCHEMA_VERSION);
        assert!(p.history.is_empty());
        assert!(p.brush_snapshots.is_empty());
    }

    #[test]
    fn paint_project_checksum_self_verifies() {
        let p = PaintProject::new(CanvasInfo::default());
        assert!(p.verify_checksum(), "fresh project checksum must verify");
    }

    #[test]
    fn paint_project_checksum_detects_tamper() {
        let mut p = PaintProject::new(CanvasInfo::default());
        p.modified_at = 12345; // mutate WITHOUT recompute → checksum stale
        assert!(!p.verify_checksum(), "tampered checksum must not verify");
        p.recompute_checksum();
        assert!(
            p.verify_checksum(),
            "after recompute, checksum verifies again"
        );
    }

    #[test]
    fn paint_project_cache_matches_canon_by_blake3() {
        let canon_hash = [42u8; 32];
        let cache = PaintProjectCache::new(canon_hash);
        assert!(cache.matches_canon(&canon_hash));
        let other_hash = [13u8; 32];
        assert!(!cache.matches_canon(&other_hash));
    }

    #[test]
    fn canvas_info_default_is_1024_300dpi_srgb() {
        let c = CanvasInfo::default();
        assert_eq!(c.width, 1024);
        assert_eq!(c.height, 1024);
        assert_eq!(c.color_profile, ColorProfile::Srgb);
        // 11.811 px/mm ≈ 300 DPI (25.4 mm/inch × 11.811 ≈ 300).
        assert!((c.ppm - 11.811).abs() < 0.001);
    }
}
