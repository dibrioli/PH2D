//! **Os ids da secção HUD** (TOP-20 #20).
//!
//! ⚠️ **As tabelas indexam pela ORDEM do modelo** (`ph2d_editor_core::hud_edits::HUD_NUMBERS` /
//! `HUD_TEXTS`): a linha `i` mostra o campo `i` e edita o campo `i`, e há gate a atar os
//! comprimentos. Uma segunda lista escrita à mão é o defeito que este painel já pagou.

use ph2d_a11y::NodeId;

use ph2d_tool_registry::hash_node_id;

/// Os NÚMEROS da secção, na ordem da `HUD_NUMBERS`.
pub const INSP_HUD_NUM: [NodeId; 3] = [
    hash_node_id("insp_hud_num_0"),
    hash_node_id("insp_hud_num_1"),
    hash_node_id("insp_hud_num_2"),
];

/// Os TEXTOS, na ordem da `HUD_TEXTS`.
pub const INSP_HUD_TEXT: [NodeId; 5] = [
    hash_node_id("insp_hud_text_0"),
    hash_node_id("insp_hud_text_1"),
    hash_node_id("insp_hud_text_2"),
    hash_node_id("insp_hud_text_3"),
    hash_node_id("insp_hud_text_4"),
];

/// O segmentado do `Fit` (`Keep` · `Stretch`).
pub const INSP_HUD_FIT: [NodeId; 2] = [
    hash_node_id("insp_hud_fit_0"),
    hash_node_id("insp_hud_fit_1"),
];

/// O segmentado da FONTE do rótulo (autorada · contador · relógio · etiqueta).
pub const INSP_HUD_SOURCE: [NodeId; 4] = [
    hash_node_id("insp_hud_source_0"),
    hash_node_id("insp_hud_source_1"),
    hash_node_id("insp_hud_source_2"),
    hash_node_id("insp_hud_source_3"),
];

/// A caixa **Disabled** do botão.
pub const INSP_HUD_DISABLED: NodeId = hash_node_id("insp_hud_disabled");
