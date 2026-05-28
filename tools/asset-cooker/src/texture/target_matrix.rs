//! W1.T3 — Target matrix: `(Tier, AssetClass) → (Format, Encoder)`.
//!
//! Implementa ADR-0055-v4 §2 (compressão por tier) + §2.6 (per-platform matrix
//! conceitual). Tabela é a referência canônica para qual format/encoder o cooker
//! deve emitir para um (Tier × AssetClass) específico.
//!
//! **Restrições D3 (W1.T2 audit):** `Encoder::Amd` NUNCA selecionado para BC7
//! (Compressonator BC7+UltraFast R=0 silent bug). BC7 sempre via `Encoder::Bc7enc`.
//! Feature `encoder-amd` está OUT do allowlist (arch-gate `architecture_ctt_features_pinned`
//! valida) — variant `Encoder::Amd` nem está compilado neste binário.
//!
//! Multi-tier emit (W1.T6, futuro): consumidor deste módulo pode iterar Tiers e
//! chamar `target_for` para cada — gera N artifacts per source. Esta função é
//! lookup puro; W1.T6 + W1.T4 (`LogicalTextureId`) orquestram multi-emit.

use ctt::encoders::{Encoder, astcenc, bc7enc, etcpak, ispc};
use ctt::{Format, TargetFormat};

/// Newtype shadow de `ph2d_asset::TierIndex` (W1.T4, ainda vapor). Mapping
/// numérico u8 alinhado com ADR-0053 `DeviceTier` (5 variants FROZEN; ainda
/// não materializado em código, vide §Symbol Registry plano vivo).
///
/// Quando `ph2d_asset::TierIndex` materializar em W1.T4, este enum vira um
/// type alias OR a função `target_for` aceita `TierIndex` direto.
///
/// `Ord`+`PartialOrd` adicionados W1.T6 — `cook_all` retorna `BTreeMap<Tier, Vec<u8>>`
/// para iterar tiers em ordem determinística (HR-6). Declaration order canônica:
/// Desktop < Mobile < Web < LowEnd < Constrained (alinha `TierIndex::*::as_u8()`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tier {
    /// `0 — Desktop` (Windows/Linux/macOS Intel + Apple Silicon). BC7 default.
    Desktop,
    /// `1 — Mobile` (iPad Apple7+/M1+, iPhone, Android Vulkan 1.0+). ASTC default;
    /// iPad BC opcional via runtime feature query (cooker emite ambos, runtime escolhe).
    Mobile,
    /// `2 — Web` (Chrome 113+ / Safari 19+ / Firefox 145+). Adapter-dependent ladder
    /// BC → ASTC → ETC2 → RGBA8.
    Web,
    /// `3 — LowEnd` (Android low-tier sem ASTC — Mali T-series antigo). ETC2 RGBA.
    LowEnd,
    /// `4 — Constrained` (fallback uncompressed). RGBA8 raw.
    Constrained,
}

/// Classificação semântica do asset (informa qual format compactado faz sentido).
/// Brush atlases single-channel (R8) ganham mais com BC4; sprites RGBA ganham com BC7/ASTC.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AssetClass {
    /// Sprite RGBA color asset (sprites de game, UI panels coloridos).
    SpriteColor,
    /// Critical UI sprite onde qualidade de bloco fino importa (icons pequenos).
    /// ASTC 4×4 em vez de 6×6 para preservar detalhe.
    CriticalUi,
    /// Single-channel atlas (brush shape/grain, mask). R8 source → BC4 saving -50%.
    SingleChannel,
    /// Normal map RGBA (rarely used em 2D; placeholder para W4+ se relevant).
    NormalMap,
}

/// Lookup canônico: dado (Tier, AssetClass), retorna o `TargetFormat` que o
/// cooker deve passar pro `ctt::convert`.
///
/// Audit W1.T3 γ-H1 fix: retorna `TargetFormat` direto (não `Option`). Match é
/// exaustivo sobre Tier × AssetClass — todas as combinações têm format definido
/// (HDR W4+ vai adicionar variants OU outra função `target_for_hdr`).
#[must_use]
pub fn target_for(tier: Tier, asset_class: AssetClass) -> TargetFormat {
    use AssetClass::*;
    use Tier::*;
    // NB: encoders (bc7enc/astcenc/etcpak/intel) só aceitam variantes UNORM dos
    // formats compactados em `supported_formats()`. SRGB metadata é portado via
    // `Surface.color_space = ColorSpace::Srgb` — ctt grava no KTX2 container o
    // flag color-space apropriado. Passar `*_SRGB_BLOCK` aqui falharia o
    // `UnsupportedFormat` check do ctt (bc7enc.rs: `&[BC7_UNORM_BLOCK]`).
    //
    // Tier::Constrained early-return via `TargetFormat::Uncompressed(...)` —
    // W1.T6 fix: Encoder::Auto NÃO é passthrough; ctt::convert com
    // (Compressed{R8G8B8A8_UNORM, Auto}) retorna `UnsupportedFormat("no
    // compiled-in encoder supports R8G8B8A8_UNORM")`. Uncompressed variant
    // pula encoder dispatch direto.
    if matches!(tier, Tier::Constrained) {
        return TargetFormat::Uncompressed(Format::R8G8B8A8_UNORM);
    }

    let (format, encoder) = match (tier, asset_class) {
        // Desktop: BC family
        (Desktop, SpriteColor) => (Format::BC7_UNORM_BLOCK, bc7enc_encoder()),
        (Desktop, CriticalUi) => (Format::BC7_UNORM_BLOCK, bc7enc_encoder()),
        (Desktop, SingleChannel) => (Format::BC4_UNORM_BLOCK, intel_encoder()),
        (Desktop, NormalMap) => (Format::BC5_UNORM_BLOCK, intel_encoder()),

        // Mobile: ASTC family (cooker pode emitir BC7 ALÉM disto em multi-tier
        // para Apple7+/M1+ via runtime feature query — W1.T6).
        (Mobile, SpriteColor) => (Format::ASTC_6x6_UNORM_BLOCK, astcenc_encoder()),
        (Mobile, CriticalUi) => (Format::ASTC_4x4_UNORM_BLOCK, astcenc_encoder()),
        (Mobile, SingleChannel) => (Format::ASTC_6x6_UNORM_BLOCK, astcenc_encoder()),
        (Mobile, NormalMap) => (Format::ASTC_6x6_UNORM_BLOCK, astcenc_encoder()),

        // Web: prefer ASTC LDR (Safari/Chrome modern). Runtime adapter ladder
        // (BC → ASTC → ETC2 → RGBA8) é resolvida pelo renderer com base nas
        // features expostas pelo wgpu::Adapter; cooker emite ASTC como
        // primary cooked artifact pro tier Web.
        (Web, SpriteColor) => (Format::ASTC_6x6_UNORM_BLOCK, astcenc_encoder()),
        (Web, CriticalUi) => (Format::ASTC_4x4_UNORM_BLOCK, astcenc_encoder()),
        (Web, SingleChannel) => (Format::ASTC_6x6_UNORM_BLOCK, astcenc_encoder()),
        (Web, NormalMap) => (Format::ASTC_6x6_UNORM_BLOCK, astcenc_encoder()),

        // LowEnd: ETC2 fallback
        (LowEnd, SpriteColor) => (Format::ETC2_R8G8B8A8_UNORM_BLOCK, etcpak_encoder()),
        (LowEnd, CriticalUi) => (Format::ETC2_R8G8B8A8_UNORM_BLOCK, etcpak_encoder()),
        (LowEnd, SingleChannel) => (Format::ETC2_R8G8B8A8_UNORM_BLOCK, etcpak_encoder()),
        (LowEnd, NormalMap) => (Format::ETC2_R8G8B8A8_UNORM_BLOCK, etcpak_encoder()),

        // Tier::Constrained tratado via early-return acima (TargetFormat::Uncompressed).
        // Match exhaustivo: 4 tiers × 4 asset classes = 16 combos restantes acima.
        (Constrained, _) => unreachable!("Constrained handled by early-return above"),
    };

    TargetFormat::Compressed { format, encoder }
}

/// Default `ColorSpace` semanticamente correto para um asset class.
/// Audit W1.T3 γ-H2 fix: `SpriteColor`/`CriticalUi` são color data (sRGB);
/// `SingleChannel`/`NormalMap` são linear data (mask/grain/normal). Default
/// universal sRGB causava shader-visible gamma bug em normal maps cookados.
#[must_use]
pub fn default_color_space_for(asset_class: AssetClass) -> ctt::ColorSpace {
    match asset_class {
        AssetClass::SpriteColor | AssetClass::CriticalUi => ctt::ColorSpace::Srgb,
        AssetClass::SingleChannel | AssetClass::NormalMap => ctt::ColorSpace::Linear,
    }
}

// Audit W1.T6 θ-M2 fix: conversion `Tier → ph2d_asset::TierIndex`. Necessária
// pra CLI / Painter W3 / asset-pipeline brush W3.T1 / qualquer caller que
// queira registrar `cook_all` output no `LogicalTextureMap`. Sem essa glue
// pública, cada consumer reimplementaria conversão isoladamente (risco de
// quebrar HR-6 BTree ordering). Ordering preservada: `Tier::Desktop → 0`,
// ..., `Tier::Constrained → 4` (alinha com `TierIndex::DESKTOP`..`CONSTRAINED`).
impl From<Tier> for ph2d_asset::TierIndex {
    fn from(value: Tier) -> Self {
        match value {
            Tier::Desktop => ph2d_asset::TierIndex::DESKTOP,
            Tier::Mobile => ph2d_asset::TierIndex::MOBILE,
            Tier::Web => ph2d_asset::TierIndex::WEB,
            Tier::LowEnd => ph2d_asset::TierIndex::LOW_END,
            Tier::Constrained => ph2d_asset::TierIndex::CONSTRAINED,
        }
    }
}

// Encoder constructors with Settings::default() per W1.T2.1 D1 — features
// pinadas (encoder-bc7enc + encoder-astcenc + encoder-etcpak + encoder-intel,
// encoder-amd OMITIDO). Arch-gate `architecture_ctt_features_pinned` valida.

#[inline]
fn bc7enc_encoder() -> Encoder {
    Encoder::Bc7enc(bc7enc::Bc7encSettings::default())
}

#[inline]
fn intel_encoder() -> Encoder {
    Encoder::Intel(ispc::IspcSettings::default())
}

#[inline]
fn astcenc_encoder() -> Encoder {
    Encoder::Astcenc(astcenc::AstcencSettings::default())
}

#[inline]
fn etcpak_encoder() -> Encoder {
    Encoder::Etcpak(etcpak::EtcpakSettings::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract `(format, encoder_kind_str)` from a TargetFormat. Para
    /// `Uncompressed` retorna `kind = "Uncompressed"` (sem encoder dispatch).
    /// TargetFormat não tem Debug; helper isola match + retorna shapes assertable.
    fn extract(target: TargetFormat) -> (Format, &'static str) {
        match target {
            TargetFormat::Compressed { format, encoder } => {
                let kind = match encoder {
                    Encoder::Auto => "Auto",
                    Encoder::Bc7enc(_) => "Bc7enc",
                    Encoder::Intel(_) => "Intel",
                    Encoder::Etcpak(_) => "Etcpak",
                    Encoder::Astcenc(_) => "Astcenc",
                };
                (format, kind)
            }
            TargetFormat::Uncompressed(format) => (format, "Uncompressed"),
        }
    }

    #[test]
    fn desktop_sprite_color_uses_bc7_bc7enc() {
        let (format, kind) = extract(target_for(Tier::Desktop, AssetClass::SpriteColor));
        assert_eq!(format, Format::BC7_UNORM_BLOCK);
        assert_eq!(kind, "Bc7enc");
    }

    #[test]
    fn mobile_single_channel_uses_astc_6x6() {
        let (format, kind) = extract(target_for(Tier::Mobile, AssetClass::SingleChannel));
        assert_eq!(format, Format::ASTC_6x6_UNORM_BLOCK);
        assert_eq!(kind, "Astcenc");
    }

    #[test]
    fn desktop_single_channel_uses_bc4_intel() {
        let (format, kind) = extract(target_for(Tier::Desktop, AssetClass::SingleChannel));
        assert_eq!(format, Format::BC4_UNORM_BLOCK);
        assert_eq!(kind, "Intel");
    }

    #[test]
    fn constrained_falls_back_uncompressed() {
        let (format, kind) = extract(target_for(Tier::Constrained, AssetClass::SpriteColor));
        assert_eq!(format, Format::R8G8B8A8_UNORM);
        // W1.T6 fix: Tier::Constrained agora retorna TargetFormat::Uncompressed
        // (não Compressed+Encoder::Auto que falhava com UnsupportedFormat).
        assert_eq!(kind, "Uncompressed");
    }

    #[test]
    fn lowend_uses_etc2_etcpak() {
        let (format, kind) = extract(target_for(Tier::LowEnd, AssetClass::SpriteColor));
        assert_eq!(format, Format::ETC2_R8G8B8A8_UNORM_BLOCK);
        assert_eq!(kind, "Etcpak");
    }

    /// W1.T3 γ-H2 fix: color/UI classes → sRGB; data classes → Linear. Gamma
    /// bug em normal map cooked como sRGB ASTC era shader-visible em runtime.
    #[test]
    fn default_color_space_distinguishes_color_vs_data_classes() {
        assert_eq!(default_color_space_for(AssetClass::SpriteColor), ctt::ColorSpace::Srgb);
        assert_eq!(default_color_space_for(AssetClass::CriticalUi), ctt::ColorSpace::Srgb);
        assert_eq!(default_color_space_for(AssetClass::SingleChannel), ctt::ColorSpace::Linear);
        assert_eq!(default_color_space_for(AssetClass::NormalMap), ctt::ColorSpace::Linear);
    }

    /// Audit W1.T6 η-L2: arch-gate cross-crate ordering — `Tier::Desktop as u8 == 0`,
    /// ..., `Tier::Constrained as u8 == 4`. Mesmo mapping de `ph2d_asset::TierIndex::*::as_u8()`.
    /// Se alguém reordenar Tier no enum, este test pega antes de quebrar HR-6
    /// (postcard wire format de cooked-asset metadata depende dessa alinhamento).
    #[test]
    fn tier_ordering_matches_ph2d_asset_tier_index() {
        let pairs = [
            (Tier::Desktop, ph2d_asset::TierIndex::DESKTOP),
            (Tier::Mobile, ph2d_asset::TierIndex::MOBILE),
            (Tier::Web, ph2d_asset::TierIndex::WEB),
            (Tier::LowEnd, ph2d_asset::TierIndex::LOW_END),
            (Tier::Constrained, ph2d_asset::TierIndex::CONSTRAINED),
        ];
        for (tier, expected_ti) in pairs {
            let converted: ph2d_asset::TierIndex = tier.into();
            assert_eq!(
                converted, expected_ti,
                "Tier::{tier:?} MUST map to ph2d_asset::TierIndex::{} (u8={})",
                expected_ti.name(),
                expected_ti.as_u8()
            );
        }
        // Verifica ordering canônica BTreeMap (HR-6 deterministic).
        assert!(Tier::Desktop < Tier::Mobile);
        assert!(Tier::Mobile < Tier::Web);
        assert!(Tier::Web < Tier::LowEnd);
        assert!(Tier::LowEnd < Tier::Constrained);
    }

    /// D3 guard: nenhum entry da matrix DEVE retornar Encoder::Auto para BC7 (que
    /// poderia dispatcher para Compressonator se feature ativada). Asserção
    /// estrutural: BC7 paths sempre devolvem Bc7enc.
    #[test]
    fn bc7_paths_never_dispatch_via_auto() {
        for tier in [Tier::Desktop, Tier::Mobile, Tier::Web, Tier::LowEnd] {
            for class in [
                AssetClass::SpriteColor,
                AssetClass::CriticalUi,
                AssetClass::SingleChannel,
                AssetClass::NormalMap,
            ] {
                let (format, kind) = extract(target_for(tier, class));
                if matches!(format, Format::BC7_UNORM_BLOCK) {
                    assert_eq!(
                        kind, "Bc7enc",
                        "D3 violation: BC7 format dispatched via {kind} for ({tier:?}, {class:?}); MUST be Bc7enc"
                    );
                }
            }
        }
    }
}
