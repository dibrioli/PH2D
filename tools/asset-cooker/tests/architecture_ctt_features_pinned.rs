//! Arch-gate W1.T2.1 (D1) — `ctt` dep deve declarar `default-features = false`
//! + exatamente as features auditadas em W1.T2.
//!
//! Razão: W1.T2 audit Lente A HIGH#3 — auto-dispatch order do ctt é
//! feature-gated; ativar/desativar features muda silenciosamente qual
//! encoder backend é escolhido para um dado formato, divergindo bytes
//! de output (quebra HR-6 content-addressed).
//!
//! Razão D3: encoder-amd (Compressonator) tem bug R=0 silent em BC7+UltraFast
//! Linux/macOS (Lente A HIGH#2). Feature deve ESTAR AUSENTE da lista.
//!
//! Vide docs/audits/ctt-source-audit-2026-05-27-CONSOLIDATED.md.
//!
//! Implementação usa crate `toml` (não parser manual `str::find`) — audit
//! meta-session β HIGH-2 mostrou que parser manual falha em formatos TOML
//! válidos (campos em ordem diferente, arrays multi-linha).

use std::collections::BTreeSet;
use std::fs;

const REQUIRED_FEATURES: &[&str] = &[
    "encoder-bc7enc",
    "encoder-astcenc",
    "encoder-etcpak",
    "encoder-intel",
    "ispc-prebuilt",
];

const FORBIDDEN_FEATURES: &[&str] = &["encoder-amd"];

#[test]
fn ctt_features_pinned_and_amd_excluded() {
    let manifest_path = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read_to_string(&manifest_path).expect("read tools/asset-cooker/Cargo.toml");

    let parsed: toml::Value = toml::from_str(&raw).expect("parse tools/asset-cooker/Cargo.toml");

    let ctt = parsed
        .get("dependencies")
        .and_then(|d| d.get("ctt"))
        .expect("W1.T2.1 D1 — ctt dep MUST be declared in [dependencies]");

    let ctt_table = ctt.as_table().expect(
        "W1.T2.1 D1 — ctt dep MUST use object form `ctt = { version = ..., features = [...] }` \
         (string-only form `ctt = \"0.4.0\"` cannot pin features)",
    );

    let default_features = ctt_table
        .get("default-features")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    assert!(
        !default_features,
        "W1.T2.1 D1 — ctt dep MUST declare `default-features = false` to prevent silent \
         encoder-dispatch drift. Audit Lente A HIGH#3."
    );

    let features: BTreeSet<&str> = ctt_table
        .get("features")
        .and_then(|v| v.as_array())
        .expect("W1.T2.1 D1 — ctt dep MUST declare explicit `features = [...]` allowlist")
        .iter()
        .map(|v| v.as_str().expect("ctt feature entries must be strings"))
        .collect();

    let required: BTreeSet<&str> = REQUIRED_FEATURES.iter().copied().collect();
    let missing: Vec<&&str> = required.difference(&features).collect();
    assert!(
        missing.is_empty(),
        "W1.T2.1 D1 — required ctt features missing: {missing:?}\n\
         Got: {features:?}\nRequired (exact set): {required:?}"
    );

    let extras: Vec<&&str> = features.difference(&required).collect();
    assert!(
        extras.is_empty(),
        "W1.T2.1 D1 — unexpected ctt features beyond audited allowlist: {extras:?}\n\
         Got: {features:?}\nAllowed: {required:?}\n\
         Any addition requires audit update + W1.T2.1 amendment."
    );

    for forbidden in FORBIDDEN_FEATURES {
        assert!(
            !features.contains(forbidden),
            "W1.T2.1 D3 — forbidden ctt feature `{forbidden}` MUST NOT be enabled. \
             Compressonator backend has known BC7+UltraFast R=0 silent bug on Linux/macOS \
             (Lente A HIGH#2; ctt-compressonator.rs:296-300 acknowledges). BC7 is covered \
             by encoder-bc7enc; BC1-5 by encoder-intel."
        );
    }
}
