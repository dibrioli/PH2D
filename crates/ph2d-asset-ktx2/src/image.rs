use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{Ktx2Format, PH2D_PREMUL_KEY};

// ── image + mip types ───────────────────────────────────────────────

/// One mip level of the decoded pyramid. `data` is the raw bytes in
/// the declared [`Ktx2Format`] — uncompressed for RGBA*, compressed
/// blocks for BC / ASTC / ETC2. The decoder makes one heap allocation
/// per mip; the `Arc<[u8]>` lets the caller share the bytes between
/// the asset DB and the renderer without re-copying.
#[derive(Debug, Clone)]
pub struct MipLevel {
    /// Width of THIS mip in pixels (mip 0 == header width, mip N is
    /// `max(1, width >> N)`).
    pub width: u32,
    /// Height of THIS mip in pixels.
    pub height: u32,
    /// Raw payload — interpretation depends on [`Ktx2Image::format`].
    pub data: Arc<[u8]>,
}

/// A fully decoded KTX2 file. Header dimensions are mip 0 (the
/// largest level). Cubemap faces and array layers are NOT yet
/// flattened — Fase 1 rejects multi-layer / multi-face inputs to keep
/// the surface tight; the limits are deliberately conservative and
/// will be relaxed in Fase 2 if the asset pipeline needs them.
/// `#[non_exhaustive]` (W1.T9 audit Lente ν-7): adding a field in Fase 2
/// must stay additive for any future external consumer (today there are
/// none — decode goes through [`decode_ktx2_bytes`](crate::decode_ktx2_bytes), not struct literals
/// outside this crate). Within this crate, struct-literal construction
/// remains allowed.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Ktx2Image {
    pub format: Ktx2Format,
    pub width: u32,
    pub height: u32,
    /// Mip pyramid from level 0 (largest) to level N-1. Always at
    /// least one entry.
    pub mip_levels: Vec<MipLevel>,
    /// W1.T9 — KTX2 `keyValueData` preserved per spec §3.10.8. Empty se
    /// container não tem kvd OR caller construiu via struct literal sem
    /// passar kvd. Bounded por [`MAX_KVD_ENTRIES`](crate::MAX_KVD_ENTRIES) + [`MAX_KVD_VALUE_BYTES`](crate::MAX_KVD_VALUE_BYTES)
    /// no parser. BTreeMap (não HashMap) garante iteration ordering
    /// determinístico (HR-6).
    pub kvd: BTreeMap<String, Vec<u8>>,
}

impl Ktx2Image {
    /// Shorthand for `&self.mip_levels[0]` — the largest, full-
    /// resolution level. Always present: the decoder rejects files
    /// with zero mip levels as `InvalidContainer`, so this never
    /// panics for an `Ktx2Image` produced by [`decode_ktx2_bytes`](crate::decode_ktx2_bytes).
    #[must_use]
    pub fn base_level(&self) -> &MipLevel {
        &self.mip_levels[0]
    }

    /// W1.T9 — sum of mip level payload bytes. HR-13 budget accounting
    /// helper: used by `ph2d-asset::Asset::TextureKtx2.byte_size()` so the
    /// memory budget aggregator can size cooked textures sem extra parse.
    /// Não conta kvd ou Arc/Vec overhead — pure payload.
    #[must_use]
    pub fn byte_size_estimate(&self) -> usize {
        self.mip_levels.iter().map(|m| m.data.len()).sum()
    }

    /// W1.T9 — read [`PremulIntent`] from `kvd[PH2D_PREMUL_KEY]`. Tri-state:
    /// `[0] = Straight`, `[1] = Premultiplied`, key ausente OR malformed →
    /// `Unspecified`. Renderer pode usar `Unspecified` para defer decision
    /// pra source asset metadata OR conservative default.
    ///
    /// NB W1.T8 deferral: ctt 0.4.0 cooker NÃO emite kvd. Cooked KTX2 hoje
    /// always retorna `Unspecified` aqui. API serve future cooker integration
    /// (W1.T8.1 OR upstream ctt PR).
    #[must_use]
    pub fn premul_intent(&self) -> PremulIntent {
        match self.kvd.get(PH2D_PREMUL_KEY).map(|v| v.as_slice()) {
            Some([0]) => PremulIntent::Straight,
            Some([1]) => PremulIntent::Premultiplied,
            _ => PremulIntent::Unspecified,
        }
    }
}

/// W1.T9 — tri-state alpha intent flag carried via KTX2 `PH2D_PREMUL` kvd key.
///
/// - `Straight` — RGB components encode non-premultiplied color. Renderer
///   deve premultiplicar antes de compositing.
/// - `Premultiplied` — RGB já contém color * alpha. Renderer composita direto.
/// - `Unspecified` — key ausente; caller decide default (conservative:
///   tratar como Straight; aggressive: assume Premultiplied per ctt convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// W1.T15 audit Lente π-2: enum de metadata forward-compat (alpha-intent tagging);
// uma intent futura (ex. AssociatedAlpha/Coverage) é plausível. Fence agora —
// zero consumidor externo, custo ergonômico zero.
#[non_exhaustive]
pub enum PremulIntent {
    Straight,
    Premultiplied,
    Unspecified,
}
