//! [`Asset`] — the decoded payload behind an [`crate::AssetId`].
//!
//! M6 ships only `ImageRgba8`. Audio, font, vector, and binary blob
//! variants land as their respective milestones (M7+ as needed). The
//! enum is intentionally non-exhaustive so adding variants doesn't
//! break downstream `match`es.
//!
//! `pixels` is wrapped in `Arc<[u8]>` (not `Vec<u8>`) so two
//! consumers — e.g. the renderer's atlas builder + an MCP tool that
//! wants to introspect the data — can share the same allocation.

use crate::prefab::PrefabDoc;
use crate::scene::SceneDoc;
use std::sync::Arc;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Asset {
    ImageRgba8 {
        width: u32,
        height: u32,
        /// Tight-packed RGBA8: `len == width * height * 4`.
        pixels: Arc<[u8]>,
    },
    /// Cooked prefab — postcard-decoded once at insert time.
    /// `Arc<PrefabDoc>` so multiple spawns share one allocation
    /// without cloning the component blobs each time.
    Prefab(Arc<PrefabDoc>),
    /// Cooked scene — same Arc-sharing strategy as Prefab.
    Scene(Arc<SceneDoc>),
    /// W1.T4 (ADR-0055-v4) — cooked GPU-compressed texture (KTX2 container).
    /// `tier` indica qual platform tier este artefato é destinado (Desktop=BC7,
    /// Mobile=ASTC, etc., per [`crate::TierIndex`] + cooker target matrix).
    ///
    /// `blob` carrega os bytes KTX2 raw — design pragmático W1.T4 (2026-05-27
    /// noite): evita adicionar dep `ph2d-asset-ktx2` em `ph2d-asset/Cargo.toml`
    /// que tinha WIP alheio do imageio fan-out paralelo. Renderer W2 decodifica
    /// via `ph2d_asset_ktx2::parse(&blob)` no upload path (não hot path; HR-3 ok).
    /// ADR-0055-v4 strategic-only é silent quanto à shape; migration para
    /// `Arc<Ktx2Image>` decode-once é refactor local (~50 LOC + 1 dep + 1 gate),
    /// não débito arquitetural — quando útil pra performance W2.
    TextureKtx2 {
        tier: crate::TierIndex,
        blob: Arc<Vec<u8>>,
    },
}

impl Asset {
    /// Convenience: rough byte cost of the decoded payload.  Used
    /// later for HR-13 budget accounting.
    pub fn byte_size(&self) -> usize {
        match self {
            Self::ImageRgba8 { pixels, .. } => pixels.len(),
            Self::Prefab(p) => {
                p.components
                    .iter()
                    .map(|c| c.data.len() + std::mem::size_of_val(c))
                    .sum::<usize>()
                    + p.children.len() * std::mem::size_of_val(&p.children[0])
                    + std::mem::size_of_val(&**p)
            }
            Self::Scene(s) => {
                s.instances
                    .iter()
                    .map(|i| {
                        i.overrides
                            .iter()
                            .map(|c| c.data.len() + std::mem::size_of_val(c))
                            .sum::<usize>()
                            + std::mem::size_of_val(i)
                    })
                    .sum::<usize>()
                    + s.relations.len() * std::mem::size_of::<crate::scene::ChildOfPair>()
                    + std::mem::size_of_val(&**s)
            }
            // W1.T4: cooked KTX2 blob — raw byte count + Arc/TierIndex overhead
            // por consistência com Prefab/Scene arms acima (audit ε-H2 / ζ-F3 fix).
            // Decoded Ktx2Image lives só no renderer cache W2; aqui Asset carries
            // só os bytes serializados.
            Self::TextureKtx2 { blob, tier } => {
                blob.len() + std::mem::size_of_val(tier) + std::mem::size_of_val(&**blob)
            }
        }
    }
}
