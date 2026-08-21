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
/// **Texture Filter segmented items — um por variante de `FilterMode`, tags `0..=6`.**
///
/// ⚠️ **A POSIÇÃO É A TAG.** O despacho (`event_ordering.rs`) faz
/// `position(|&o| o == id).map(SamplingFieldEdit::Filter)`, e a shell fecha com
/// `FilterMode::from_tag(t)` — por isso acrescentar um id aqui, na posição certa, é tudo o que uma
/// variante nova precisa. ⛔ Nunca reordene: a ordem **é** o contrato.
///
/// ⚠️ **Eram TRÊS até 2026-08-21, e as outras quatro eram inalcançáveis por gesto nenhum.** O
/// motor entrega mipmap trilinear real e anisotropia 16× desde 2026-06-18
/// (`ph2d-render/src/image_filter.rs`), o componente `TextureFilter` está no registry de cena (uma
/// tag ≥3 escrita por script sobrevivia ao save/load) — e o painel capava em 2. Pior: ele pintava
/// `.min(2)`, o que acendia **«Linear»** para as tags 3 e 5, que o renderer manda para **Nearest**.
/// *O painel dizia o oposto do que o ecrã desenhava, e capava abaixo do hardware (CLAUDE.md §0.0).*
pub const INSP_SAMPLE_FILTER: [NodeId; 7] = [
    hash_node_id("insp_sample_filter_inherit"),
    hash_node_id("insp_sample_filter_nearest"),
    hash_node_id("insp_sample_filter_linear"),
    hash_node_id("insp_sample_filter_nearest_mipmap"),
    hash_node_id("insp_sample_filter_linear_mipmap"),
    hash_node_id("insp_sample_filter_nearest_aniso"),
    hash_node_id("insp_sample_filter_linear_aniso"),
];
/// Texture Repeat segmented items (Inherit / Disabled / Enabled / Mirror
/// → tags 0/1/2/3).
pub const INSP_SAMPLE_REPEAT: [NodeId; 4] = [
    hash_node_id("insp_sample_repeat_inherit"),
    hash_node_id("insp_sample_repeat_disabled"),
    hash_node_id("insp_sample_repeat_enabled"),
    hash_node_id("insp_sample_repeat_mirror"),
];
/// UV tiling/scroll NumberInputs (W3 UvTransform): scale X/Y, offset X/Y.
pub const INSP_SAMPLE_UV_SCALE_X: NodeId = hash_node_id("insp_sample_uv_scale_x");
pub const INSP_SAMPLE_UV_SCALE_Y: NodeId = hash_node_id("insp_sample_uv_scale_y");
pub const INSP_SAMPLE_UV_OFFSET_X: NodeId = hash_node_id("insp_sample_uv_offset_x");
pub const INSP_SAMPLE_UV_OFFSET_Y: NodeId = hash_node_id("insp_sample_uv_offset_y");

/// §10 Material & Blend — section accent color dot.
pub const INSP_LIVE_BLEND_COLOR: NodeId = hash_node_id("insp_live_blend_color");
/// §10 Blend Mode segmented items, indexed by `BlendMode::tag()` (0..5):
/// Mix / Add / Subtract / Multiply / Screen / Premult. Tag 0 (Mix) =
/// detach the optional `BlendMode` component (default).
pub const INSP_SAMPLE_BLEND: [NodeId; 6] = [
    hash_node_id("insp_sample_blend_mix"),
    hash_node_id("insp_sample_blend_add"),
    hash_node_id("insp_sample_blend_subtract"),
    hash_node_id("insp_sample_blend_multiply"),
    hash_node_id("insp_sample_blend_screen"),
    hash_node_id("insp_sample_blend_premult"),
];
