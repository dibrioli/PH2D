//! W1.T15 audit — Lente σ (seam / system integration): cook → decode end-to-end.
//!
//! As 6 rodadas de audit anteriores (γ/δ→T3, ε/ζ→T4, η/θ→T6, λ/μ→T7,
//! ι/κ→T11+T14, ν/ξ→T9) auditaram cada task ISOLADAMENTE. O seam entre os dois
//! lados do pipeline — o ctt encoder que `cook` usa pra EMITIR KTX2 e o parser
//! `ph2d_asset_ktx2::decode_ktx2_bytes` que o renderer W2 vai usar pra LER KTX2 —
//! nunca foi exercitado por nenhum teste do workspace. Até este gate (W1.T15) +
//! a W1.T8.1, valia:
//!
//!   - `tools/asset-cooker/` não tinha dep de PRODUÇÃO em `ph2d-asset-ktx2`
//!     (parser) — só este gate σ o usava como dev-dep. **Mudou na W1.T8.1:**
//!     `cook_tagged` precisa de `patch_premul_intent`, então asset-ktx2 virou
//!     dep normal (parser puro: ktx2 + thiserror, sem ISPC, sem ciclo — vide
//!     justificativa no `[dependencies]` do Cargo.toml).
//!   - `crates/ph2d-asset/` também não (decisão pragmática W1.T4, asset.rs).
//!   - Os testes do cooker checam só os 12 bytes de magic header.
//!   - Os testes do parser usam um `build_fixture` hand-construído, não bytes ctt.
//!
//! Logo a hipótese central da Fase 2 — "o VkFormat que o encoder ISPC grava no
//! container bate com o `ktx2::Format` que o decoder lê" — era afirmada mas não
//! travada. Este gate fecha o seam: cozinha cada (Tier, AssetClass) canônico e
//! confirma que o artefato decodifica para o FORMATO ESPERADO (não
//! `Ktx2Format::Unsupported`, que é o fallback silencioso de VkFormat divergente).
//!
//! NÃO testa: pixel correctness do bloco comprimido (precisa GPU transcode, W2);
//! cross-machine byte-determinism (D2/W1.T10 canonical runner ⏳).
//!
//! W1.T8.1 fechou o deferral kvd PH2D_PREMUL: `ctt`/`ktx2 0.5` continuam
//! READ-ONLY, mas `cook_tagged` faz patch post-hoc dos bytes (insere a key
//! PH2D_PREMUL na seção keyValueData via `ph2d_asset_ktx2::patch_premul_intent`).
//! Os testes `cook_tagged_*` abaixo travam o round-trip cook → patch → decode →
//! `premul_intent()`.
//!
//! ⚠️ ISPC encoders = global state não-thread-safe → rode com
//! `RUST_TEST_THREADS=1 cargo test -p ph2d-asset-cooker` (vide armadilha #1 do
//! módulo). Os encoders aqui crasham determinísticamente se rodados em paralelo.

use ph2d_asset_cooker::texture::{
    AssetClass, CookOptions, Tier, cook, cook_all, cook_all_tagged, cook_tagged, fixtures,
};
use ph2d_asset_ktx2::{Ktx2Format, PH2D_PREMUL_KEY, PremulIntent, decode_ktx2_bytes};

/// O fixture canônico (W1.T11) é 64×64 — as dimensões devem sobreviver o
/// round-trip cook → decode intactas (o encoder não reescala mip 0).
const FIXTURE_W: u32 = 64;
const FIXTURE_H: u32 = 64;

/// Cozinha `gradient_64x64` para um (Tier, AssetClass) e devolve o `Ktx2Image`
/// decodificado pelo parser — provando que os bytes ctt atravessam o seam.
fn cook_then_decode(tier: Tier, class: AssetClass) -> ph2d_asset_ktx2::Ktx2Image {
    let png = fixtures::gradient_64x64();
    let bytes = cook(&png, CookOptions::for_asset_class(tier, class))
        .unwrap_or_else(|e| panic!("cook {tier:?}/{class:?} failed: {e:?}"));
    decode_ktx2_bytes(&bytes).unwrap_or_else(|e| {
        panic!("decode of ctt-emitted KTX2 for {tier:?}/{class:?} failed: {e:?}")
    })
}

/// Toda decodificação deve preservar as dimensões de mip 0 e ter ≥1 mip level.
fn assert_dims_and_base(img: &ph2d_asset_ktx2::Ktx2Image, tier: Tier, class: AssetClass) {
    assert_eq!(
        (img.width, img.height),
        (FIXTURE_W, FIXTURE_H),
        "{tier:?}/{class:?}: header dims must round-trip the 64×64 source"
    );
    assert!(
        !img.mip_levels.is_empty(),
        "{tier:?}/{class:?}: decoded image must have at least the base level"
    );
    let base = img.base_level();
    assert_eq!(
        (base.width, base.height),
        (FIXTURE_W, FIXTURE_H),
        "{tier:?}/{class:?}: base level dims must equal header dims"
    );
    assert!(
        !base.data.is_empty(),
        "{tier:?}/{class:?}: base level payload must be non-empty"
    );
}

// ── BC7 (Desktop sprite color) ──────────────────────────────────────────

/// O seam mais crítico: o encoder bc7enc grava `BC7_*_BLOCK` e o decoder
/// precisa reconhecer esse VkFormat (não cair em `Unsupported`). UNORM vs SRGB
/// é decisão interna do ctt em função do `color_space`, então aceitamos
/// qualquer variante BC7 — o que importa é que NÃO é `Unsupported` e que
/// `is_compressed()` confirma a família.
#[test]
fn desktop_sprite_color_round_trips_as_bc7() {
    let img = cook_then_decode(Tier::Desktop, AssetClass::SpriteColor);
    assert_dims_and_base(&img, Tier::Desktop, AssetClass::SpriteColor);
    assert!(
        matches!(
            img.format,
            Ktx2Format::Bc7RgbaUnorm | Ktx2Format::Bc7RgbaUnormSrgb
        ),
        "expected a BC7 variant from the bc7enc encoder, got {:?} \
         (Unsupported means the ctt VkFormat diverged from ktx2::Format)",
        img.format
    );
    assert!(img.format.is_compressed());
}

// ── BC4 (Desktop single-channel atlas) ──────────────────────────────────

#[test]
fn desktop_single_channel_round_trips_as_bc4() {
    let img = cook_then_decode(Tier::Desktop, AssetClass::SingleChannel);
    assert_dims_and_base(&img, Tier::Desktop, AssetClass::SingleChannel);
    assert!(
        matches!(img.format, Ktx2Format::Bc4RUnorm),
        "expected BC4 from the Intel ISPC encoder, got {:?}",
        img.format
    );
    assert!(img.format.is_compressed());
}

// ── BC5 (Desktop normal map) ────────────────────────────────────────────

#[test]
fn desktop_normal_map_round_trips_as_bc5() {
    let img = cook_then_decode(Tier::Desktop, AssetClass::NormalMap);
    assert_dims_and_base(&img, Tier::Desktop, AssetClass::NormalMap);
    assert!(
        matches!(img.format, Ktx2Format::Bc5RgUnorm),
        "expected BC5 from the Intel ISPC encoder, got {:?}",
        img.format
    );
    assert!(img.format.is_compressed());
}

// ── ASTC (Mobile sprite color) ──────────────────────────────────────────

#[test]
fn mobile_sprite_color_round_trips_as_astc() {
    let img = cook_then_decode(Tier::Mobile, AssetClass::SpriteColor);
    assert_dims_and_base(&img, Tier::Mobile, AssetClass::SpriteColor);
    assert!(
        matches!(
            img.format,
            Ktx2Format::Astc6x6RgbaUnorm | Ktx2Format::Astc6x6RgbaUnormSrgb
        ),
        "expected ASTC 6×6 from the astcenc encoder, got {:?}",
        img.format
    );
    assert!(img.format.is_compressed());
}

// ── ETC2 (LowEnd fallback) ──────────────────────────────────────────────

#[test]
fn lowend_sprite_color_round_trips_as_etc2() {
    let img = cook_then_decode(Tier::LowEnd, AssetClass::SpriteColor);
    assert_dims_and_base(&img, Tier::LowEnd, AssetClass::SpriteColor);
    assert!(
        matches!(
            img.format,
            Ktx2Format::Etc2Rgba8Unorm | Ktx2Format::Etc2Rgba8UnormSrgb
        ),
        "expected ETC2 RGBA8 from the etcpak encoder, got {:?}",
        img.format
    );
    assert!(img.format.is_compressed());
}

// ── Uncompressed passthrough (Constrained tier) ─────────────────────────

/// `Tier::Constrained` faz passthrough `TargetFormat::Uncompressed(R8G8B8A8)`.
/// O seam aqui prova que mesmo o caminho NÃO-comprimido emite um container
/// válido cujo VkFormat o parser reconhece como RGBA8 (e que
/// `is_compressed()` corretamente reporta `false`).
#[test]
fn constrained_round_trips_as_uncompressed_rgba8() {
    let img = cook_then_decode(Tier::Constrained, AssetClass::SpriteColor);
    assert_dims_and_base(&img, Tier::Constrained, AssetClass::SpriteColor);
    assert!(
        matches!(
            img.format,
            Ktx2Format::Rgba8Unorm | Ktx2Format::Rgba8UnormSrgb
        ),
        "expected uncompressed RGBA8 passthrough, got {:?}",
        img.format
    );
    assert!(
        !img.format.is_compressed(),
        "Constrained passthrough must NOT report as compressed"
    );
}

// ── Full matrix via cook_all (W1.T6) ────────────────────────────────────

/// Fecha o seam no nível de batch: cada um dos 5 tiers que `cook_all` emite
/// para `SpriteColor` deve decodificar para um formato CONHECIDO (nunca
/// `Unsupported`). Isto cobre a integração cook_all → asset → ktx2 completa
/// num único gate, exatamente o "final integration check" do W1.T15.
#[test]
fn cook_all_every_tier_decodes_to_a_known_format() {
    let png = fixtures::gradient_64x64();
    let artifacts = cook_all(&png, AssetClass::SpriteColor).expect("cook_all sprite color");
    assert_eq!(artifacts.len(), 5, "cook_all emits one artifact per tier");

    for (tier, bytes) in &artifacts {
        let img = decode_ktx2_bytes(bytes)
            .unwrap_or_else(|e| panic!("cook_all artifact for {tier:?} failed to decode: {e:?}"));
        assert_eq!(
            (img.width, img.height),
            (FIXTURE_W, FIXTURE_H),
            "{tier:?}: dims must round-trip"
        );
        assert!(
            !matches!(img.format, Ktx2Format::Unsupported(_)),
            "{tier:?}: decoded to Unsupported({}) — the encoder's VkFormat is not \
             in the decoder's known set; the cook→decode seam is broken for this tier",
            match img.format {
                Ktx2Format::Unsupported(raw) => raw,
                _ => 0,
            }
        );
    }
}

// ── W1.T8.1 PH2D_PREMUL tag round-trip (cook_tagged) ────────────────────

/// End-to-end seam for the patcher: `cook_tagged` (Desktop/SpriteColor,
/// default `Straight` alpha) must stamp the KVD so the parser reports
/// `Straight` instead of the pre-T8.1 `Unspecified`, while the format +
/// dims still round-trip unchanged. Exercises the BC7 (non-empty,
/// multi-mip-capable) artifact, not just a 1×1 synthetic.
#[test]
fn cook_tagged_desktop_sprite_color_reports_straight_intent() {
    let png = fixtures::gradient_64x64();
    let opts = CookOptions::for_asset_class(Tier::Desktop, AssetClass::SpriteColor);

    // Baseline: plain `cook` carries no tag.
    let plain = cook(&png, opts).expect("plain cook");
    let plain_img = decode_ktx2_bytes(&plain).expect("plain decodes");
    assert_eq!(
        plain_img.premul_intent(),
        PremulIntent::Unspecified,
        "untagged cook must stay Unspecified (regression guard for the deferral)"
    );

    // Tagged: the patcher inserts PH2D_PREMUL.
    let tagged = cook_tagged(&png, opts).expect("cook_tagged");
    let img = decode_ktx2_bytes(&tagged).expect("tagged decodes");

    assert_eq!(
        img.premul_intent(),
        PremulIntent::Straight,
        "cook_tagged must stamp the source alpha intent (Straight)"
    );
    assert_eq!(
        img.kvd.get(PH2D_PREMUL_KEY).map(Vec::as_slice),
        Some(&[0u8][..]),
        "PH2D_PREMUL value byte must encode Straight (0)"
    );
    // Format + dims survive the KVD insertion + offset shift untouched.
    assert_eq!(
        img.format, plain_img.format,
        "tag must not change the format"
    );
    assert_eq!((img.width, img.height), (FIXTURE_W, FIXTURE_H));
    // Base mip payload is byte-identical to the untagged cook.
    assert_eq!(
        img.base_level().data.as_ref(),
        plain_img.base_level().data.as_ref(),
        "tagging must not perturb the mip data"
    );
}

/// `cook_all_tagged` stamps every tier's artifact; each decodes to a known
/// format AND reports the `Straight` intent (the `for_asset_class` default).
#[test]
fn cook_all_tagged_every_tier_carries_premul_intent() {
    let png = fixtures::gradient_64x64();
    let artifacts = cook_all_tagged(&png, AssetClass::SpriteColor).expect("cook_all_tagged");
    assert_eq!(artifacts.len(), 5, "one tagged artifact per tier");

    for (tier, bytes) in &artifacts {
        let img = decode_ktx2_bytes(bytes)
            .unwrap_or_else(|e| panic!("tagged artifact for {tier:?} failed to decode: {e:?}"));
        assert!(
            !matches!(img.format, Ktx2Format::Unsupported(_)),
            "{tier:?}: tagged artifact decoded to Unsupported — seam broken"
        );
        assert_eq!(
            img.premul_intent(),
            PremulIntent::Straight,
            "{tier:?}: cook_all_tagged must carry the PH2D_PREMUL Straight tag"
        );
    }
}
