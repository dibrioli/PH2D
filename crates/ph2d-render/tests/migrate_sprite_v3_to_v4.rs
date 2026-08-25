//! v3 → v4 migrator gate (Sprite_projeto §10.6).
//!
//! W0 created this file as a `#[ignore]`d stub reserving the canonical
//! path the spec names ("crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs").
//! **W1.T1.6 lands the real suite here:** the migrator
//! ([`Sprite::migrate_v3_to_v4`]) and the canonical load entry point
//! ([`ph2d_render::load_sprite`], ADR-0070-amendment-2 §4) now exist, so
//! the assertions below exercise the actual back-compat path against the
//! 5 frozen v3 fixtures generated in W0.T0.12 — NOT a tautology, because
//! the fixtures were serialized by the v3 `SpriteV3` schema before the
//! `Sprite` struct was bumped to v4.
//!
//! ## Why these assertions matter (the bug class they guard)
//!
//! `#[serde(default)]` on the 14 new v4 fields is documentary-only under
//! postcard (positional, non-self-describing — T0.13 empirically pinned
//! that postcard REJECTS trailing-missing fields). The wrapper-enum
//! dispatch + migrator is therefore the SOLE working back-compat path.
//! If a future edit deletes the migrator and leans on serde defaults,
//! `deserialize_v3_postcard_loads_as_v4_with_defaults` fails loudly.
//! If a future edit makes `region_filter_clip` an unconditional default
//! (the `#[serde(default)]` helper returns the Atlas value `true` for
//! everyone), `..._individual_..._region_filter_clip_false` fails.

use ph2d_render::sprite_versioned::{SpriteV3, SpriteV4, SpriteVersioned, canonical_v3_fixtures};
use ph2d_render::{LoadError, Sprite, SpriteSource, load_sprite};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| {
        panic!(
            "fixture {name} missing ({e}) — run `cargo test -p ph2d-render --test generate_v3_fixtures -- --ignored --nocapture` to bootstrap"
        )
    })
}

/// W0 carry-over: every committed fixture is a `V3` envelope, so the
/// wrapper dispatch the migrator consumes stays honest. Complements the
/// migration assertions below (this checks the discriminant; those check
/// the transform output).
#[test]
fn v3_fixtures_dispatch_through_wrapper() {
    for (name, _canonical) in canonical_v3_fixtures() {
        let bytes = read_fixture(name);
        let versioned: SpriteVersioned = postcard::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("postcard wrapper-enum dispatch on {name} failed: {e}"));
        match versioned {
            SpriteVersioned::V3(_) => {} // expected — committed fixtures are v3 envelopes
            SpriteVersioned::V4(_) | SpriteVersioned::V5(_) => {
                panic!("v3 fixture {name} dispatched as V4/V5 — discriminant/fixture regression")
            }
        }
    }
}

#[test]
fn deserialize_v3_postcard_loads_as_v4_with_defaults() {
    let bytes = read_fixture("sprite_v3_atlas.postcard");
    let m = load_sprite(&bytes).expect("v3 atlas fixture loads");
    let sprite = m.sprite;

    // ⭐ **A escada sobe DOIS degraus agora** (v3 → v4 → v5, ADR-0164 F1 passo 6) — e o que ela
    // NÃO faz é o mais importante: nenhum dos três componentes nasce, porque uma grelha de uma
    // célula, uma janela desligada e cantos brancos são indistinguíveis da ausência.
    // *Migrar é preservar o que foi autorado, não materializar defaults.*
    assert_eq!(m.grid, None, "1x1 nao materializa componente");
    assert_eq!(
        m.region, None,
        "region_enabled=false nao materializa componente"
    );
    assert_eq!(
        m.corner_tint, None,
        "cantos brancos nao materializam componente"
    );

    // Forwarded v3 fields.
    assert_eq!(sprite.version, 5);
    assert_eq!(sprite.source, SpriteSource::Atlas { key: 0 });
    assert_eq!(sprite.size, [10.0, 10.0]);
    assert_eq!(sprite.tint, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(sprite.anchor, [0.0, 0.0]);

    // New v4 fields → benign identity defaults.
    assert_eq!(sprite.self_tint, [1.0; 4]);
    assert!(!sprite.tint_fill);
    assert_eq!(sprite.opacity, 1.0);
    assert!(!sprite.flip_x);
    assert!(!sprite.flip_y);
    assert!(sprite.centered);
    assert_eq!(sprite.offset, [0.0, 0.0]);
}

#[test]
fn deserialize_v3_individual_sprite_loads_with_region_filter_clip_false() {
    let bytes = read_fixture("sprite_v3_individual.postcard");
    let m = load_sprite(&bytes).expect("v3 individual fixture loads");
    assert_eq!(m.sprite.version, 5);
    assert_eq!(m.sprite.source, SpriteSource::Individual { texture_id: 42 });
    assert_eq!(m.sprite.tint, [1.0, 1.0, 1.0, 0.5]);
    // ⚠️ **O `region_filter_clip` deixou de existir num sprite SEM região** (ADR-0164 F1 passo
    // 6): a escolha Atlas/Individual mudou-se para os construtores do `SpriteRegion`, e um v3
    // nunca tem região — logo não há bool nenhum a carregar. O gate que a mede agora é
    // `the_migrator_still_picks_the_clip_from_the_source`, abaixo.
    assert_eq!(m.region, None, "um v3 nao tem regiao");
}

/// ⚠️ **A escolha Atlas→clip / Individual→sem-clip SOBREVIVEU ao corte** — ela mudou de casa
/// (para os construtores do [`ph2d_ecs::SpriteRegion`]), e este gate afirma-a onde ela vive
/// agora. Sem ele o corte teria apagado, em silêncio, uma lei que dois testes v3 mediam.
#[test]
fn the_migrator_still_picks_the_clip_from_the_source() {
    assert!(
        ph2d_ecs::SpriteRegion::for_atlas([0.0; 4]).filter_clip,
        "no atlas ha' vizinhos de verdade: o clamp defende"
    );
    assert!(
        !ph2d_ecs::SpriteRegion::individual([0.0; 4]).filter_clip,
        "numa textura propria nao ha' vizinho, e o clamp so' cortaria borda"
    );
}

/// Drive ALL 5 frozen fixtures through `load_sprite`, asserting the
/// forwarded fields survive verbatim and the conditional
/// `region_filter_clip` branch matches the source variant. Covers the
/// atlas / atlas_with_anchor / individual / premultiplied / max_size
/// coverage matrix (anatomia / Lens D D25).
#[test]
fn all_v3_fixtures_migrate_preserving_fields_and_region_branch() {
    for (name, v3) in canonical_v3_fixtures() {
        let bytes = read_fixture(name);
        let m = load_sprite(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        let sprite = m.sprite;
        // ⭐ Nenhum dos três componentes nasce de um v3 — ver o gate acima.
        assert_eq!(m.grid, None, "{name}: grid");
        assert_eq!(m.region, None, "{name}: region");
        assert_eq!(m.corner_tint, None, "{name}: corner_tint");

        assert_eq!(sprite.version, 5, "{name}: version");
        assert_eq!(sprite.source, v3.source, "{name}: source");
        assert_eq!(sprite.size, v3.size, "{name}: size");
        assert_eq!(sprite.tint, v3.tint, "{name}: tint");
        assert_eq!(sprite.anchor, v3.anchor, "{name}: anchor");

        // `premultiplied` is `#[serde(skip)]` — never on the wire, so a
        // wire-loaded sprite is ALWAYS `false`, even for the
        // `sprite_v3_premultiplied` fixture whose in-memory canonical
        // carries `true`. The runtime flag is rebuilt from texture-store
        // context at the extract boundary, not from the blob.
        assert!(
            !sprite.premultiplied,
            "{name}: wire-loaded premultiplied must be false (serde skip)"
        );

        // v4 identity defaults hold for every migrated sprite.
        assert_eq!(sprite.self_tint, [1.0; 4], "{name}: self_tint");
        assert!(!sprite.tint_fill, "{name}: tint_fill");
        assert_eq!(sprite.opacity, 1.0, "{name}: opacity");
        assert!(sprite.centered, "{name}: centered");
    }
}

/// The PURE migrator (no wire round-trip) must NOT drop a `premultiplied`
/// flag a caller set in memory — `#[serde(skip)]` only erases it on the
/// wire. This pins the migrator-as-value-transform contract distinct
/// from the wire-load behavior asserted above.
#[test]
fn migrate_v3_to_v4_preserves_in_memory_premultiplied_flag() {
    let v3 = SpriteV3 {
        source: SpriteSource::Individual { texture_id: 1 },
        size: [64.0, 64.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        anchor: [0.0, 0.0],
        premultiplied: true,
    };
    let v4 = Sprite::migrate_v3_to_v4(v3);
    assert!(
        v4.premultiplied,
        "pure migrator preserves an in-memory premultiplied flag"
    );
    assert!(!v4.region_filter_clip, "Individual → false");
    assert_eq!(
        v4.version, 4,
        "o migrador v3->v4 carimba 4, nao o VERSION vivo"
    );
    // E o degrau seguinte preserva a bandeira e sobe a versao.
    let m = Sprite::migrate_v4_to_v5(v4);
    assert!(m.sprite.premultiplied, "o 2o degrau tambem a preserva");
    assert_eq!(m.sprite.version, 5);
}

/// ⭐ **UM BLOB v4 AUTORADO PARTE-SE EM QUATRO** (ADR-0164 F1 passo 6) — a afirmação central da
/// migração, e a única que mede o caso que importa: um ficheiro real, com grelha, janela e
/// degradê **autorados**, tem de os devolver como componentes, sem perder um bit.
///
/// ⚠️ O envelope é construído com o espelho CONGELADO [`SpriteV4`], e não com a `Sprite` viva:
/// são estes os bytes que já estão em disco. Construí-lo com o tipo vivo mediria a minha ideia
/// do formato, não o formato.
#[test]
fn an_authored_v4_blob_splits_into_the_three_components() {
    let original = SpriteV4 {
        version: 4,
        source: SpriteSource::Atlas { key: 3 },
        size: [20.0, 40.0],
        tint: [0.9, 0.8, 0.7, 0.6],
        anchor: [0.0, 0.0],
        premultiplied: false,
        self_tint: [0.5, 0.5, 0.5, 1.0],
        per_corner_tint: [
            [1.0, 0.0, 0.0, 1.0], // TL red
            [0.0, 1.0, 0.0, 1.0], // TR green
            [0.0, 0.0, 1.0, 1.0], // BL blue
            [1.0, 1.0, 0.0, 1.0], // BR yellow
        ],
        tint_fill: true,
        opacity: 0.5,
        flip_x: true,
        flip_y: true,
        centered: false,
        offset: [3.0, -4.0],
        hframes: 4,
        vframes: 2,
        frame: 5,
        region_enabled: true,
        region_rect: [1.0, 2.0, 8.0, 8.0],
        region_filter_clip: false,
    };

    let bytes = postcard::to_allocvec(&SpriteVersioned::V4(original)).expect("v4 serializes");
    let m = load_sprite(&bytes).expect("v4 round-trips");

    // Os 13 que FICAM na Sprite, verbatim.
    assert_eq!(m.sprite.version, 5, "a versao sobe");
    assert_eq!(m.sprite.source, original.source);
    assert_eq!(m.sprite.size, original.size);
    assert_eq!(m.sprite.tint, original.tint);
    assert_eq!(m.sprite.self_tint, original.self_tint);
    assert!(m.sprite.tint_fill);
    assert_eq!(m.sprite.opacity, 0.5);
    assert!(m.sprite.flip_x && m.sprite.flip_y);
    assert!(!m.sprite.centered);
    assert_eq!(m.sprite.offset, [3.0, -4.0]);

    // E os 7 que SAEM, cada um no componente dele.
    assert_eq!(
        m.grid,
        Some(ph2d_ecs::SpriteGrid {
            hframes: 4,
            vframes: 2,
            frame: 5
        })
    );
    assert_eq!(
        m.region,
        Some(ph2d_ecs::SpriteRegion {
            rect: [1.0, 2.0, 8.0, 8.0],
            // ⚠️ **O bool GRAVADO, não o derivado da fonte.** Este é um sprite de Atlas, cujo
            // construtor escolheria `true` — mas o ficheiro diz `false`, e migrar é preservar
            // bytes, não recalcular a escolha. (Mutação: derivar do `source` ⇒ RED.)
            filter_clip: false,
        })
    );
    assert_eq!(
        m.corner_tint,
        Some(ph2d_ecs::SpriteCornerTint(original.per_corner_tint))
    );
}

/// ⚠️ **Um `region_rect` autorado com `enabled = false` é DESCARTADO** — e é a decisão certa: ele
/// era o estado que ninguém conseguia ler (*"há janela ou não há?"* com duas respostas). Guardá-lo
/// anexaria uma região desligada, e uma região desligada deixou de existir.
#[test]
fn a_disabled_region_rect_does_not_survive_the_split() {
    let mut v4 = SpriteV4 {
        version: 4,
        source: SpriteSource::Individual { texture_id: 5 },
        size: [10.0, 10.0],
        tint: [1.0; 4],
        anchor: [0.0; 2],
        premultiplied: false,
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        tint_fill: false,
        opacity: 1.0,
        flip_x: false,
        flip_y: false,
        centered: true,
        offset: [0.0; 2],
        hframes: 1,
        vframes: 1,
        frame: 0,
        region_enabled: false,
        region_rect: [1.0, 2.0, 8.0, 8.0], // autorado, e DESLIGADO
        region_filter_clip: true,
    };
    assert_eq!(Sprite::migrate_v4_to_v5(v4).region, None);
    // Controle: ligado, ela sobrevive — senão este gate passaria com um migrador que
    // descartasse SEMPRE a região.
    v4.region_enabled = true;
    assert!(Sprite::migrate_v4_to_v5(v4).region.is_some());
}

/// ⚠️ **Uma grelha 1×1 com `frame != 0` NÃO é «indistinguível da ausência»** — o frame autorado é
/// autoria, e descartá-lo perderia informação. A régua é [`ph2d_ecs::SpriteGrid::is_single`], que
/// pergunta pelos TRÊS números, não só pelos dois da grelha.
#[test]
fn a_single_cell_grid_with_an_authored_frame_still_materialises() {
    let single = ph2d_ecs::SpriteGrid::SINGLE;
    assert!(single.is_single());
    assert!(
        !ph2d_ecs::SpriteGrid { frame: 3, ..single }.is_single(),
        "um frame autorado nao e' o neutro"
    );
}

/// Bytes that are not a valid `SpriteVersioned` envelope (truncated, or
/// a pre-wrapper v2-era blob) surface as `LoadError::Deserialize` — never
/// a panic, never silent corruption (ADR-0070-amendment-2 §4). The error
/// preserves the postcard cause via `source()`.
#[test]
fn load_sprite_rejects_garbage_bytes_as_error_not_panic() {
    use std::error::Error;
    // A multi-byte varint discriminant far beyond the 2 declared variants.
    let garbage = [0xFFu8, 0xFF, 0xFF, 0xFF, 0xFF];
    let err = load_sprite(&garbage).expect_err("garbage must not load");
    assert!(matches!(err, LoadError::Deserialize(_)));
    assert!(!err.to_string().is_empty(), "Display is non-empty");
    assert!(
        err.source().is_some(),
        "postcard cause preserved in source()"
    );
}

#[test]
fn load_sprite_rejects_empty_bytes() {
    assert!(matches!(load_sprite(&[]), Err(LoadError::Deserialize(_))));
}

/// W1.T1.6 trailing-byte cap. Raw `postcard::from_bytes` silently accepts
/// a valid V3 prefix followed by hostile padding (pinned by the W0 test
/// `versioned_load_silently_ignores_trailing_bytes_after_valid_v3`).
/// `load_sprite` is the defense layer that REJECTS it via `take_from_bytes`
/// + a leftover check — a single-sprite blob must be exactly one envelope.
#[test]
fn load_sprite_rejects_trailing_bytes_after_valid_envelope() {
    let canonical = read_fixture("sprite_v3_atlas.postcard");
    let mut padded = canonical.clone();
    padded.extend_from_slice(&[0xAAu8; 100]); // 100 hostile trailing bytes
    let err = load_sprite(&padded).expect_err("trailing bytes must be rejected");
    match err {
        LoadError::TrailingBytes { consumed, total } => {
            assert_eq!(total, padded.len(), "total = full input length");
            assert_eq!(consumed, canonical.len(), "consumed = exactly the envelope");
            assert_eq!(total - consumed, 100, "100 hostile trailing bytes detected");
        }
        other => panic!("expected TrailingBytes, got {other:?}"),
    }
    // And the exact (unpadded) blob still loads cleanly — the cap rejects
    // only the surplus, never a well-formed envelope.
    assert!(
        load_sprite(&canonical).is_ok(),
        "exact envelope still loads"
    );
}
