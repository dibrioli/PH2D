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
    let cargo_toml = fs::read_to_string(env!("CARGO_MANIFEST_DIR").to_string() + "/Cargo.toml")
        .expect("read tools/asset-cooker/Cargo.toml");

    let ctt_line_start = cargo_toml
        .find("\nctt = {")
        .expect("ctt dep must use object form `ctt = { ... }` (not `ctt = \"x.y.z\"`)");
    let ctt_block_end = cargo_toml[ctt_line_start..]
        .find("] }")
        .expect("ctt dep block must close with `] }`");
    let ctt_block = &cargo_toml[ctt_line_start..ctt_line_start + ctt_block_end + 3];

    assert!(
        ctt_block.contains("default-features = false"),
        "W1.T2.1 D1 — ctt dep MUST declare `default-features = false` to prevent silent \
         encoder-dispatch drift. Audit Lente A HIGH#3.\nFound block:\n{ctt_block}"
    );

    for feature in REQUIRED_FEATURES {
        assert!(
            ctt_block.contains(&format!("\"{feature}\"")),
            "W1.T2.1 D1 — required ctt feature `{feature}` missing from pinned allowlist.\n\
             Block:\n{ctt_block}"
        );
    }

    for feature in FORBIDDEN_FEATURES {
        assert!(
            !ctt_block.contains(&format!("\"{feature}\"")),
            "W1.T2.1 D3 — forbidden ctt feature `{feature}` MUST NOT be enabled. \
             Compressonator backend has known BC7+UltraFast R=0 silent bug on Linux/macOS \
             (Lente A HIGH#2; ctt-compressonator.rs:296-300 acknowledges). BC7 is covered \
             by encoder-bc7enc; BC1-5 by encoder-intel.\nBlock:\n{ctt_block}"
        );
    }
}
