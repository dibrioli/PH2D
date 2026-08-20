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
    /// **Imagem de 16 bits por canal** — plano
    /// [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
    /// W1.2. Variante IRMÃ do [`Asset::ImageRgba8`], que é o ponto de extensão que o cabeçalho
    /// deste ficheiro reservou de propósito (`#[non_exhaustive]`).
    ///
    /// ⚠️ **`pixels` são bits de MEIO-FLOAT em espaço LINEAR**, não inteiros de 16 bits e não sRGB
    /// — é o que a textura `Rgba16Float` consome sem conversão nenhuma. As duas razões estão em
    /// [`ph2d_imageio::precision`]: o `Rgba16Unorm` depende de uma feature opcional do adapter, e
    /// não existe variante sRGB de formato de 16 bits algum (o hardware converte ao amostrar
    /// `Rgba8UnormSrgb`, e **não** converte um `Rgba16Float`).
    ///
    /// ⚠️ **`#[non_exhaustive]` corta nos dois sentidos.** Um `match` numa crate de fora tem braço
    /// `_` e portanto aceita esta variante **em silêncio, sem a tratar**. Quem consumir pixels tem
    /// de passar por [`Asset::image_rgba8`], que converte, ou tratar o ramo explicitamente — o
    /// compilador **não** o vai lembrar.
    ImageRgba16 {
        width: u32,
        height: u32,
        /// Tight-packed meio-float linear: `len == width * height * 4`.
        pixels: Arc<[u16]>,
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
    /// via `ph2d_asset_ktx2::decode_ktx2_bytes(&blob)` no upload path (não hot path; HR-3 ok).
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
            // ⚠️ `len()` conta ELEMENTOS, e aqui cada um são 2 bytes. Sem o `× 2` esta variante
            // reportaria metade do custo real ao orçamento HR-13 — e um orçamento que subestima
            // não avisa antes de estourar.
            Self::ImageRgba16 { pixels, .. } => pixels.len() * 2,
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

    /// A precisão desta imagem, ou `None` se o asset não for imagem descomprimida.
    ///
    /// ⚠️ `TextureKtx2` devolve `None` de propósito: uma textura cozida é BC/ASTC/ETC2 e a sua
    /// precisão depende do tier resolvido — dizer "RGBA8" dela seria a mesma mentira que a linha
    /// `Format` do Inspector contava antes do plano 17 §5.
    #[must_use]
    pub fn precision(&self) -> Option<ph2d_imageio::Precision> {
        match self {
            Self::ImageRgba8 { .. } => Some(ph2d_imageio::Precision::Rgba8),
            Self::ImageRgba16 { .. } => Some(ph2d_imageio::Precision::Rgba16),
            _ => None,
        }
    }

    /// **Os pixels em RGBA8, convertendo se preciso** — a porta única por onde um consumidor que
    /// só sabe 8 bits alcança uma imagem de 16 sem a ignorar em silêncio.
    ///
    /// ⚠️ **`Cow` e não `Vec`:** o caso de 8 bits (que é quase todos) **não copia nada**. Devolver
    /// `Vec` faria toda a leitura de toda a imagem do app pagar uma cópia por causa de uma
    /// variante que quase nenhuma imagem usa.
    ///
    /// ⚠️ O ramo de 16 bits **perde** (é o sentido que perde, plano 18 §3.2). Quem quiser os 16
    /// bits tem de os pedir pelo `match`; esta função é a rede de segurança, não o caminho bom.
    #[must_use]
    pub fn image_rgba8(&self) -> Option<(u32, u32, std::borrow::Cow<'_, [u8]>)> {
        match self {
            Self::ImageRgba8 {
                width,
                height,
                pixels,
            } => Some((*width, *height, std::borrow::Cow::Borrowed(&**pixels))),
            Self::ImageRgba16 {
                width,
                height,
                pixels,
            } => Some((
                *width,
                *height,
                std::borrow::Cow::Owned(ph2d_imageio::rgba16_to_rgba8(pixels)),
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_imageio::Precision;

    fn rgba8(pixels: &[u8]) -> Asset {
        Asset::ImageRgba8 {
            width: (pixels.len() / 4) as u32,
            height: 1,
            pixels: Arc::from(pixels.to_vec().into_boxed_slice()),
        }
    }

    /// ⚠️ **`len()` conta elementos, não bytes.** Sem o `× 2` no braço de 16 bits, o orçamento
    /// HR-13 subestimaria em metade exactamente a variante que custa o dobro.
    #[test]
    fn sixteen_bits_costs_twice_the_bytes_of_eight() {
        let eight = rgba8(&[1, 2, 3, 4]);
        let sixteen = Asset::ImageRgba16 {
            width: 1,
            height: 1,
            pixels: Arc::from(vec![0u16; 4].into_boxed_slice()),
        };
        assert_eq!(eight.byte_size(), 4);
        assert_eq!(
            sixteen.byte_size(),
            8,
            "um pixel de 16 bits sao 8 bytes; `pixels.len()` sozinho diria 4"
        );
    }

    /// A porta de 8 bits **não copia** quando já é 8 bits, e converte quando não é.
    #[test]
    fn the_eight_bit_door_borrows_instead_of_copying_when_it_can() {
        let original = [10u8, 20, 30, 255];
        let asset = rgba8(&original);
        let (w, h, px) = asset.image_rgba8().expect("uma imagem tem pixels");
        assert_eq!((w, h), (1, 1));
        assert!(
            matches!(px, std::borrow::Cow::Borrowed(_)),
            "o caso de 8 bits copiou — toda leitura do app passaria a pagar por uma variante que \
             quase nenhuma imagem usa"
        );
        assert_eq!(&*px, &original);
    }

    /// **A ida-e-volta pela porta**: 8 → 16 → porta de 8 devolve os bytes originais. Fecha o laço
    /// com a lei exaustiva do `ph2d_imageio::precision`, agora atravessando o `Asset`.
    #[test]
    fn a_sixteen_bit_asset_comes_back_through_the_door_unchanged() {
        let original = [10u8, 20, 30, 255, 0, 128, 255, 7];
        let halves = ph2d_imageio::rgba8_to_rgba16(&original);
        let asset = Asset::ImageRgba16 {
            width: 2,
            height: 1,
            pixels: Arc::from(halves.into_boxed_slice()),
        };
        let (w, h, px) = asset.image_rgba8().expect("uma imagem tem pixels");
        assert_eq!((w, h), (2, 1));
        assert!(
            matches!(px, std::borrow::Cow::Owned(_)),
            "o caso de 16 bits tem MESMO de converter"
        );
        assert_eq!(&*px, &original);
    }

    /// ⚠️ **Uma textura cozida não tem precisão a declarar** — ela é BC/ASTC/ETC2 e depende do
    /// tier. Dizer "RGBA8" dela é a mentira que o plano 17 §5 removeu do Inspector.
    #[test]
    fn a_cooked_texture_declares_no_precision() {
        assert_eq!(rgba8(&[0, 0, 0, 0]).precision(), Some(Precision::Rgba8));
        let cooked = Asset::TextureKtx2 {
            tier: crate::TierIndex::new(0).expect("o tier 0 existe"),
            blob: Arc::new(vec![0u8; 8]),
        };
        assert_eq!(cooked.precision(), None);
        assert!(
            cooked.image_rgba8().is_none(),
            "a porta de 8 bits nao pode inventar pixels de um blob comprimido"
        );
    }
}
