//! **Os ids das §9 Sampling e §10 Material & Blend do Inspector.**
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** (2026-08-21): acrescentar as quatro
//! variantes de filtro que faltavam levou aquele arquivo a 714 contra um teto de 700. *Cortar para
//! o irmão é a cura; alargar a allowlist não é.* Mesmo padrão de `inspector_joint.rs` e
//! `inspector_player.rs`.
//!
//! As duas seções vivem juntas porque partilham a mesma lei: **a POSIÇÃO no array É a tag** do
//! enum correspondente (`FilterMode` · `RepeatMode` · `BlendMode`). O despacho deriva a tag de
//! `position(|&o| o == id)`, e a shell fecha com `from_tag`. ⛔ Nunca reordene nenhum destes
//! arrays — a ordem é o contrato.

use super::*;

/// W3 §9 Sampling — section accent color dot.
pub const INSP_LIVE_SAMPLING_COLOR: NodeId = hash_node_id("insp_live_sampling_color");

/// §10 Material & Blend — section accent color dot.
pub const INSP_LIVE_BLEND_COLOR: NodeId = hash_node_id("insp_live_blend_color");
