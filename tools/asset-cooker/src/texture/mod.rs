//! W1.T3 — Texture cooking pipeline (ADR-0055-v4).
//!
//! Module entry para texture compression cooking via `ctt` v0.4.0 (multi-encoder
//! unificado offline). Single source (PNG) → multi-tier KTX2 artifacts.
//!
//! API:
//!
//! - [`cook`] — função lib API: bytes PNG + opções → KTX2 bytes.
//! - [`CookOptions`] — config: tier alvo + format override + alpha intent.
//! - [`Tier`] — newtype shadow de `ph2d_asset::TierIndex` (W1.T4); 5 variants
//!   alinhadas com ADR-0053 (Desktop / Mobile / Web / LowEnd / Constrained).
//!
//! Submódulos:
//!
//! - [`target_matrix`] — tabela `(Tier, AssetClass) → (Format, Encoder)` (ADR-0055-v4 §2 + §2.6).
//! - [`cook`][mod@cook] — implementação da função de cooking + decode + ctt::convert glue.
//!
//! Disciplinas operacionais herdadas do W1.T2 audit (CONSOLIDATED.md):
//!
//! - **D1**: `ctt` features pinadas em Cargo.toml (`default-features = false` +
//!   allowlist exata); arch-gate `architecture_ctt_features_pinned` valida.
//! - **D3**: `encoder-amd` ausente do build (Compressonator BC7+UltraFast R=0
//!   silent bug). BC7 via bc7enc-rdo; BC1-5 via Intel ISPC.
//! - **D2 + D4**: determinismo + snapshot test ⏳ aguardam W1.T10 (canonical
//!   runner CI workflow) + W1.T11.5 (Git LFS setup).

pub mod cook;
pub mod target_matrix;

pub use cook::{CookOptions, TextureCookError, cook};
pub use target_matrix::{AssetClass, Tier, target_for};
