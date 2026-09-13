//! Widget `NodeId`s for the Upscale panel.
//!
//! Unlike the older bgremoval / padding panels (whose IDs lived in
//! `ph2d_editor_core::ids` for legacy reasons), the Upscale panel
//! derives its IDs via [`hash_node_id`] right here in the satellite —
//! no slot in editor-core needs to be allocated, keeping the tool
//! drop-in (ADR-0040 §3.8). The tool crate (`ph2d_tool_upscale::tool`)
//! re-derives the same hashes from the same string keys; both sides
//! match by construction because the hash is deterministic.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12): os ids dos controlos, que a
//! ferramenta declara em `ph2d_tool_upscale::tool::ids`, deixaram de ser re-exportados daqui — quem os
//! usa nomeia-os lá. Fica o `UPS_PANEL`, cujo slug se repete na fundação: é uma das três repetições
//! que o censo de colisões tolera por NOME (`SLUGS_REPETIDOS_TOLERADOS`), porque a cerca da
//! `line/render-loop` (`render_loop/upscale_bridge.rs`) nomeia esta cópia.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Panel root `NodeId` — used by paint to publish the panel rect and
/// by `clear_panel_rect` on hide.
pub const UPS_PANEL: NodeId = hash_node_id("panel.upscale");
