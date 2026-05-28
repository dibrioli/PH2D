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
use subtle::ConstantTimeEq;

use crate::SCHEMA_VERSION;
use crate::device::{LayerStack, LayerStackEntry};
use crate::history::StrokeHistory;
use crate::record::MAX_SAMPLES_PER_STROKE;
use crate::snapshot::LayerSnapshot;

// =============================================================================
// Magic bytes
// =============================================================================

/// Magic bytes do canon (`.ph2d-painter`). 12 bytes ASCII.
pub const PAINT_PROJECT_MAGIC: [u8; 12] = *b"PH2D-PAINTER";

/// Magic bytes do sidecar (`.ph2d-painter-cache`). 18 bytes ASCII.
pub const PAINT_PROJECT_CACHE_MAGIC: [u8; 18] = *b"PH2D-PAINTER-CACHE";

// =============================================================================
// Caps anti-DoS pra `load()` (Audit T1.8 L5 — adversarial input)
// =============================================================================
//
// postcard NÃO enforça bounds em Vec lens — attacker forja
// `.ph2d-painter` com magic+version válidos + `history.len() = u64::MAX/24`
// → OOM no deserialize ANTES de qualquer validação. Caps abaixo são
// validados em `load()` ANTES (size) e DEPOIS (count) do deserialize.

/// Tamanho máximo do canon serializado em bytes (500 MiB). Files maiores
/// rejeitados em `load()` antes de qualquer parsing. Audit T1.8 L5-I1/I4.
///
/// Justification: ~17k strokes × 28B sample × 1024 samples (cap budget §2.10) ≈
/// 480 MB worst case. 500 MB hard cap protege contra DoS amplification.
pub const MAX_CANON_BYTES: usize = 500 * 1024 * 1024;

/// Máximo de strokes em `StrokeHistory::Full` (canon load). Budget §2.10
/// ~17-20k mas hard cap 1M cobre futuro budget bump sem mexer no cap.
pub const MAX_STROKES_PER_CANON: usize = 1_000_000;

/// Máximo de brushes em `PaintProject::brush_snapshots`. Sessão real tem
/// <100; cap 10_000 cobre uso extremo sem permitir DoS.
/// Audit T1.8 L5-I7.
pub const MAX_BRUSH_SNAPSHOTS: usize = 10_000;

/// Máximo de layers em `PaintProject::layer_stack.layers`. Spec §2.2
/// menciona 200 layers operacional; cap 1000 cobre overflow.
/// Audit T1.8 L5-I1.
pub const MAX_LAYERS: usize = 1_000;

/// Máximo de bytes em cada `LayerStackEntry::Reserved(Vec<u8>)`.
/// Reserved é placeholder W3 — não deve carregar payload real.
/// Cap 4KB previne uso como covert-channel/storage abuse.
/// Audit T1.8 L5-I9.
pub const MAX_RESERVED_PAYLOAD: usize = 4 * 1024;

/// Máximo de snapshots em `PaintProjectCache::snapshots`. Sidecar default
/// budget LRU 200 MB; snapshot típico 16 MB ⇒ ~12 snapshots simultâneos.
/// Cap 1000 cobre extremes.
pub const MAX_SNAPSHOTS_PER_CACHE: usize = 1_000;

// =============================================================================
// `PaintProject` — canon savefile (HR-14 versionado)
// =============================================================================

/// Canon savefile — fonte de verdade pra reconstruir o canvas.
///
/// HR-14 forward-compat: writer sempre emite latest version; reader v2+
/// deve ler v1 (migration helpers em [`apply_migrations`] quando v2 nascer).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaintProject {
    /// "PH2D-PAINTER" magic ASCII — identifica formato. **Deve permanecer
    /// como primeiro field do struct** — postcard preserva declaration order
    /// e gate `paint_project_field_order_is_stable` em
    /// [`tests/forward_compat_pins.rs`](../../tests/forward_compat_pins.rs)
    /// fail-fast se reorder acontecer.
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
    /// **Audit T1.8 L2-F7 + L5-I7 — invariantes:**
    /// - Único por hash (caller responsável; `load()` enforça em runtime).
    /// - Cap [`MAX_BRUSH_SNAPSHOTS`] = 10k (load rejeita se exceder).
    /// - Caller (`ph2d-tool-painter` em T1.9+) deve verificar
    ///   `brush_snapshots.iter().any(|(h, _)| *h == hash)` antes de push.
    ///
    /// Vec foi escolhido (vs `BTreeMap`) por (a) postcard roundtrip
    /// determinístico simples, (b) sessões típicas têm <100 brushes ⇒
    /// lookup linear é fine. W11 polish: `BrushSnapshotTable` newtype
    /// wrapper enforça uniqueness no type-level.
    pub brush_snapshots: Vec<(BrushParamsHash, Brush)>,
    /// Wall-clock ms since epoch — created.
    pub created_at: u64,
    /// Wall-clock ms since epoch — last modified.
    pub modified_at: u64,
    /// blake3 sobre concat([magic, version, canvas, layer_stack, history,
    /// brush_snapshots, created_at, modified_at]) — **integrity guard ONLY,
    /// NOT a MAC**.
    ///
    /// **Audit T1.8 L5-I2 — escopo de segurança:** blake3 sem keyed-mode
    /// detecta corrupção acidental (disk bit rot, transfer truncado) mas
    /// **NÃO** detecta tamper malicioso (attacker recomputa o hash após
    /// mutar bytes). Para cloud-sync W16 ou MCP audit W13, esquema canon
    /// deve evoluir pra incluir `signature: Option<Ed25519Signature>` em
    /// um dos 3 headroom slots (v2 schema bump). Caller recebendo
    /// `.ph2d-painter` de fonte não-confiável DEVE validar provenance
    /// externamente (HTTPS TLS cert, signed manifest).
    pub checksum: [u8; 32],
    // === 3 slots de headroom ===
    // - signature: Option<Ed25519Signature>     — W16 cloud sync MAC (L5-I2)
    // - document_metadata: Option<DocMetadata>  — author/title/license
    // - embedded_palette: Option<PaletteId>     — convenience swatches
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

    /// `true` se [`Self::magic`] bate com [`PAINT_PROJECT_MAGIC`].
    /// Audit T1.8 L4-H20 — helper paralelo a [`PaintProjectCache::verify_magic`].
    pub fn verify_magic(&self) -> bool {
        self.magic == PAINT_PROJECT_MAGIC
    }

    /// Recomputa o checksum sobre todos os campos exceto o próprio checksum.
    /// Caller chama após qualquer mutação do canon.
    ///
    /// **Audit T1.8 L1-F1/L2-F5/L3-G4 — safety-first clone:** preserva
    /// invariante "panic em serialize NÃO corrompe state" via clone-then-zero
    /// (em vez de mutate-self-then-serialize que deixava janela onde
    /// `self.checksum = [0; 32]` permanecia se to_allocvec panic). Peak
    /// memory 2× serialized size; W11 follow-up streaming `blake3::Hasher`
    /// elimina ambos (vide [`docs/plans/2026-05-wave-11-carry-overs.md`]).
    pub fn recompute_checksum(&mut self) {
        let mut ghost = self.clone();
        ghost.checksum = [0u8; 32];
        if let Ok(bytes) = postcard::to_allocvec(&ghost) {
            let h = blake3::hash(&bytes);
            self.checksum = *h.as_bytes();
        }
        // Se Err OR panic do to_allocvec → self.checksum mantém valor
        // anterior (jamais corrompido).
    }

    /// `true` se o checksum gravado bate com o recomputado. Integrity guard
    /// (vide doc do field `checksum` pra escopo de segurança).
    ///
    /// **Audit T1.8 L5-I3 — constant-time comparison:** usa
    /// [`subtle::ConstantTimeEq`] em vez de `==` byte-by-byte pra evitar
    /// timing oracle quando W16 server-side reusa esta fn pra validar
    /// uploads. Custo runtime negligível (32 bytes).
    pub fn verify_checksum(&self) -> bool {
        let saved = self.checksum;
        let mut ghost = self.clone();
        ghost.checksum = [0u8; 32];
        let bytes = match postcard::to_allocvec(&ghost) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let h = blake3::hash(&bytes);
        h.as_bytes().ct_eq(&saved).into()
    }
}

// =============================================================================
// `CanvasInfo` + `ColorProfile`
// =============================================================================

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
/// 2. Test [`color_profile_srgb_discriminant_is_zero`](../../tests/forward_compat_pins.rs)
///    (T1.8) pinning byte de serialização — fail-fast se discriminant drift.
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

// =============================================================================
// `PaintProjectCache` — sidecar (regenerável)
// =============================================================================

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
    /// Audit T1.8 L5-I3 — usa constant-time eq pra simetria com canon.
    pub fn matches_canon(&self, canon_blake3: &[u8; 32]) -> bool {
        self.source_blake3.ct_eq(canon_blake3).into()
    }

    /// `true` se [`Self::magic`] bate com [`PAINT_PROJECT_CACHE_MAGIC`].
    /// Audit T1.8 L4-H20 — paralelo a [`PaintProject::verify_magic`].
    pub fn verify_magic(&self) -> bool {
        self.magic == PAINT_PROJECT_CACHE_MAGIC
    }
}

/// Placeholder pra R-tree serializado (W14 Inspector materializa).
pub type SerializedRTree = Vec<u8>;

// =============================================================================
// `save` + `load` + migrations
// =============================================================================

/// Serializa `PaintProject` para bytes postcard, recomputando o checksum
/// automaticamente para evitar emit com checksum stale.
///
/// **Audit T1.8 L4-H6 — symmetric helper a [`load`].** Caller que usa
/// `postcard::to_allocvec(&p)` direto pode emit bytes com checksum stale
/// se esqueceu de [`PaintProject::recompute_checksum`]. Este helper
/// garante que o bytes sempre representam state consistente.
///
/// Clona o project pra recompute (não muta `p`). Caller que prefere zero-clone
/// pode chamar `p.recompute_checksum()` + `postcard::to_allocvec(p)` direto.
pub fn save(p: &PaintProject) -> Result<Vec<u8>, SaveError> {
    let mut p = p.clone();
    p.recompute_checksum();
    postcard::to_allocvec(&p).map_err(SaveError::Serialization)
}

/// Erros de [`save`]. `#[non_exhaustive]` permite expansão futura.
#[non_exhaustive]
#[derive(Debug)]
pub enum SaveError {
    /// postcard falhou ao serializar (geralmente OOM em alloc do Vec).
    Serialization(postcard::Error),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "postcard serialization failed: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}

/// Carrega `PaintProject` de bytes postcard, aplicando migrations conforme
/// `version` e validando todos os caps anti-DoS.
///
/// **Audit T1.8 L5 — defesa em profundidade na ordem:**
///
/// 1. **Size precheck** ([`MAX_CANON_BYTES`]) — rejeita bytes excessivos
///    ANTES de qualquer parsing (mitiga I-1/I-4 DoS amplification).
/// 2. **postcard deserialize** — pode falhar em bytes malformados.
/// 3. **Magic check** — fail-fast em arquivos não-PH2D.
/// 4. **Version range** — `0` (não-suportado), `>SCHEMA_VERSION`
///    (future, requer reader novo), gaps na chain.
/// 5. **Integrity** ([`PaintProject::verify_checksum`]) — fail se hash
///    não bate.
/// 6. **Cap validation** ([`validate_caps_post_deserialize`]) — runtime
///    enforcement de bounds que postcard NÃO enforça (history/points/
///    brush_snapshots/layer counts).
/// 7. **Brush snapshots dedup check** — rejeita duplicate hashes
///    (mitiga L5-I7 confusion attack).
/// 8. **Migrations chain** ([`apply_migrations`]) — empurra v1→v2→…
///    sequencialmente conforme `version`.
/// 9. **Post-migration cap revalidation** — caps podem ter mudado por
///    migration (defesa contra I-5).
/// 10. **Pin version** ao `SCHEMA_VERSION` atual.
///
/// ## Como ESTENDER quando v2 nascer
///
/// 1. Bump [`crate::SCHEMA_VERSION`] = 2 + adicione field novo em
///    `PaintProject`. Use `#[serde(default)]` no field novo se quiser
///    forward-compat de v1 reader (ignora trailing).
/// 2. Adicione case em [`apply_migrations`] para `v == 1`.
/// 3. Writer SEMPRE emite latest version (`SCHEMA_VERSION`).
///
/// ## OBRIGAÇÕES DO MIGRATOR
///
/// CADA `migrate_vN_to_vN+1` DEVE:
///
/// - Preservar invariantes de cap (re-checar lengths após preenchimento default).
/// - NÃO confiar em fields attacker-controlled para derivar estado
///   authentication (signatures, MACs) — derivar só de constantes ou
///   campos cryptographically verified.
/// - Após chain completa, `load` re-roda [`validate_caps_post_deserialize`]
///   antes de devolver o `PaintProject` (defesa contra I-5).
pub fn load(bytes: &[u8]) -> Result<PaintProject, LoadError> {
    // (1) Size precheck — mitiga I-1/I-4 DoS amplification.
    if bytes.len() > MAX_CANON_BYTES {
        return Err(LoadError::TooLarge {
            size: bytes.len(),
            max: MAX_CANON_BYTES,
        });
    }

    // (2) Deserialize.
    let mut p: PaintProject = postcard::from_bytes(bytes).map_err(LoadError::Deserialization)?;

    // (3) Magic.
    if !p.verify_magic() {
        return Err(LoadError::WrongMagic);
    }

    // (4) Version range.
    match p.version {
        0 => return Err(LoadError::UnsupportedVersion(0)),
        v if v > SCHEMA_VERSION => {
            return Err(LoadError::FutureVersion {
                file: v,
                supported: SCHEMA_VERSION,
            });
        }
        _ => {} // OK pra migration chain
    }

    // (5) Integrity.
    if !p.verify_checksum() {
        return Err(LoadError::IntegrityMismatch);
    }

    // (6) Cap validation post-deserialize.
    validate_caps_post_deserialize(&p)?;

    // (7) Brush snapshots dedup runtime enforcement.
    validate_brush_snapshots_unique(&p)?;

    // (8) Migration chain v1 → vN.
    apply_migrations(&mut p)?;

    // (9) Re-validate caps post-migration (caps podem ter mudado).
    validate_caps_post_deserialize(&p)?;

    // (10) Pin version ao reader corrente.
    p.version = SCHEMA_VERSION;
    Ok(p)
}

/// Valida caps em runtime que postcard NÃO enforça (Audit T1.8 L5-I1/I7/I9).
/// Chamado por [`load`] e pelos migration helpers (defesa em profundidade).
pub fn validate_caps_post_deserialize(p: &PaintProject) -> Result<(), LoadError> {
    // History stroke count cap.
    if p.history.len() > MAX_STROKES_PER_CANON {
        return Err(LoadError::CapExceeded {
            kind: "history.strokes",
            got: p.history.len(),
            max: MAX_STROKES_PER_CANON,
        });
    }

    // Per-stroke sample count cap.
    for rec in p.history.iter() {
        if rec.points.len() > MAX_SAMPLES_PER_STROKE {
            return Err(LoadError::CapExceeded {
                kind: "stroke.points",
                got: rec.points.len(),
                max: MAX_SAMPLES_PER_STROKE,
            });
        }
    }

    // Brush snapshots count cap (L5-I7).
    if p.brush_snapshots.len() > MAX_BRUSH_SNAPSHOTS {
        return Err(LoadError::CapExceeded {
            kind: "brush_snapshots",
            got: p.brush_snapshots.len(),
            max: MAX_BRUSH_SNAPSHOTS,
        });
    }

    // Layer stack count cap.
    if p.layer_stack.layers.len() > MAX_LAYERS {
        return Err(LoadError::CapExceeded {
            kind: "layer_stack.layers",
            got: p.layer_stack.layers.len(),
            max: MAX_LAYERS,
        });
    }

    // Per-LayerStackEntry::Reserved payload cap (L5-I9).
    for entry in &p.layer_stack.layers {
        let LayerStackEntry::Reserved(bytes) = entry;
        if bytes.len() > MAX_RESERVED_PAYLOAD {
            return Err(LoadError::CapExceeded {
                kind: "layer_stack_entry.reserved",
                got: bytes.len(),
                max: MAX_RESERVED_PAYLOAD,
            });
        }
    }

    Ok(())
}

/// Valida que `brush_snapshots` não tem entradas duplicadas pelo hash.
/// Audit T1.8 L5-I7 — attacker pode forjar duas brushes com mesmo hash
/// mas params diferentes pra confusion attack (preview-vs-render divergence).
fn validate_brush_snapshots_unique(p: &PaintProject) -> Result<(), LoadError> {
    // HR-5 + ADR-0022 — BTreeSet cross-OS determinístico (HashSet hash order
    // diverge entre platforms).
    use std::collections::BTreeSet;
    let mut seen: BTreeSet<&BrushParamsHash> = BTreeSet::new();
    for (h, _brush) in &p.brush_snapshots {
        if !seen.insert(h) {
            return Err(LoadError::DuplicateBrushHash(*h));
        }
    }
    Ok(())
}

/// Aplica migration chain v1 → vN sequencialmente. T1.8 ship é no-op
/// (SCHEMA_VERSION=1). Audit T1.8 L3-G1/L5-I5/L5-I10/L4-H14: helper VIVO
/// (sempre chamado por `load`), pra futuro dev não-esquecer de descomentar.
pub fn apply_migrations(_p: &mut PaintProject) -> Result<(), LoadError> {
    // Em SCHEMA_VERSION=1 não há migration. Quando v2 nascer:
    // if _p.version == 1 { migrate_v1_to_v2(_p)?; }
    // if _p.version == 2 { migrate_v2_to_v3(_p)?; }
    // ...
    Ok(())
}

/// Migration v1 → v2 — STUB até v2 nascer. Audit T1.8 L1-F4/L4-H14:
/// `#[doc(hidden)]` esconde de rustdoc downstream até estar pronto;
/// retorna `Ok(())` (no-op) em vez de panic.
///
/// Quando v2 for criada, esta fn DEVE:
/// 1. Adicionar/preencher novos fields no `p`.
/// 2. Setar `p.version = 2`.
/// 3. Chamar `p.recompute_checksum()`.
/// 4. NOT recursively call validate_caps_post_deserialize (load faz no fim).
#[doc(hidden)]
pub fn migrate_v1_to_v2(_p: &mut PaintProject) -> Result<(), LoadError> {
    Ok(())
}

// =============================================================================
// `LoadError`
// =============================================================================

/// Erros de [`load`].
///
/// **Audit T1.8 L3-G9:** `#[non_exhaustive]` permite adicionar variants
/// novos sem semver break — downstream `match` precisa de `_ =>` arm.
#[non_exhaustive]
#[derive(Debug)]
pub enum LoadError {
    /// Bytes excedem [`MAX_CANON_BYTES`] (anti-DoS precheck). Audit T1.8 L5-I1.
    TooLarge { size: usize, max: usize },
    /// postcard falhou ao deserializar. Audit T1.8 L4-H5 — preserva causa
    /// interna em vez de string genérica.
    Deserialization(postcard::Error),
    /// Magic bytes não batem com `PH2D-PAINTER`.
    WrongMagic,
    /// Version 0 não é suportada (file pré-v1 não existe).
    UnsupportedVersion(u32),
    /// File version > SCHEMA_VERSION — reader atual desatualizado vs arquivo.
    FutureVersion { file: u32, supported: u32 },
    /// Version não reconhecida (gap na migration chain).
    /// **Não-instanciado em SCHEMA_VERSION=1** — preservado para v2+ quando
    /// migrations criarem gaps. Removível em semver-minor (`#[non_exhaustive]`).
    /// Audit T1.8 L3-G1/L4-H9: `#[doc(hidden)]` esconde de rustdoc public até
    /// ser atingível.
    #[doc(hidden)]
    UnknownVersion(u32),
    /// Integrity hash mismatch — file corrupted (use signed canon pra
    /// tamper-detection; vide doc do field `checksum`). Audit T1.8 L5-I2:
    /// renomeado de `ChecksumMismatch` pra remover palavra "tampered" que
    /// mentia escopo (blake3 sem MAC não detecta tamper malicioso).
    IntegrityMismatch,
    /// Cap excedido (defesa anti-DoS em runtime). Audit T1.8 L5-I1/I7/I9.
    CapExceeded {
        kind: &'static str,
        got: usize,
        max: usize,
    },
    /// Brush snapshots têm duplicate hash (confusion attack defense).
    /// Audit T1.8 L5-I7.
    DuplicateBrushHash(BrushParamsHash),
}

// PartialEq manual — postcard::Error não impl PartialEq, então derive falha.
impl PartialEq for LoadError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TooLarge { size: a, max: am }, Self::TooLarge { size: b, max: bm }) => {
                a == b && am == bm
            }
            (Self::Deserialization(_), Self::Deserialization(_)) => true, // opaque equality
            (Self::WrongMagic, Self::WrongMagic) => true,
            (Self::UnsupportedVersion(a), Self::UnsupportedVersion(b)) => a == b,
            (
                Self::FutureVersion {
                    file: a,
                    supported: as_,
                },
                Self::FutureVersion {
                    file: b,
                    supported: bs,
                },
            ) => a == b && as_ == bs,
            (Self::UnknownVersion(a), Self::UnknownVersion(b)) => a == b,
            (Self::IntegrityMismatch, Self::IntegrityMismatch) => true,
            (
                Self::CapExceeded {
                    kind: a,
                    got: ag,
                    max: am,
                },
                Self::CapExceeded {
                    kind: b,
                    got: bg,
                    max: bm,
                },
            ) => a == b && ag == bg && am == bm,
            (Self::DuplicateBrushHash(a), Self::DuplicateBrushHash(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge { size, max } => write!(
                f,
                "canon file too large: {size} bytes > {max} bytes hard cap (anti-DoS)"
            ),
            Self::Deserialization(e) => write!(f, "postcard deserialization failed: {e}"),
            Self::WrongMagic => write!(f, "magic bytes mismatch — not a .ph2d-painter file"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version {v} (must be ≥ 1)"),
            Self::FutureVersion { file, supported } => write!(
                f,
                "file version {file} is newer than reader (supports up to {supported})"
            ),
            Self::UnknownVersion(v) => write!(f, "unknown version {v} in migration chain"),
            Self::IntegrityMismatch => write!(
                f,
                "integrity hash mismatch — file corrupted (use signed canon for tamper-detection)"
            ),
            Self::CapExceeded { kind, got, max } => {
                write!(f, "cap exceeded on {kind}: got {got}, max {max}")
            }
            Self::DuplicateBrushHash(_) => {
                write!(
                    f,
                    "duplicate brush_snapshots hash (confusion attack defense)"
                )
            }
        }
    }
}

impl std::error::Error for LoadError {}

// =============================================================================
// Tests
// =============================================================================

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
    fn paint_project_verify_magic() {
        let mut p = PaintProject::new(CanvasInfo::default());
        assert!(p.verify_magic());
        p.magic = *b"WRONGMAGIC!!";
        assert!(!p.verify_magic());
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
    fn paint_project_cache_verify_magic() {
        let cache = PaintProjectCache::new([0u8; 32]);
        assert!(cache.verify_magic());
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

    #[test]
    fn save_recomputes_checksum_automatically() {
        let mut p = PaintProject::new(CanvasInfo::default());
        p.modified_at = 12345; // mutate WITHOUT recompute
        assert!(!p.verify_checksum(), "stale before save()");
        let bytes = save(&p).expect("save");
        let p2 = load(&bytes).expect("load");
        assert!(
            p2.verify_checksum(),
            "after save+load, fresh checksum verifies"
        );
        assert_eq!(p2.modified_at, 12345);
    }

    #[test]
    fn load_rejects_oversized_canon() {
        // Bytes > MAX_CANON_BYTES rejeitados ANTES de parsing.
        let huge = vec![0u8; MAX_CANON_BYTES + 1];
        let err = load(&huge).expect_err("oversized canon should fail");
        assert!(matches!(err, LoadError::TooLarge { .. }));
    }
}
