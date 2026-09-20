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

/// O segmentado do `Fit` (`Keep` · `Stretch` · `Expand`).
///
/// ⛔⛔⛔ **UM id por SEGMENTO, e é a contagem deste array que decide o que o artista VÊ.**
///
/// Report do dono, 2026-09-20: *«só tem as opções de keep e Stretch. Não expand»* — e o painter já
/// passava os **três** rótulos. O que faltava era o terceiro id: o segmentado pinta um segmento por
/// entrada daqui, logo o rótulo a mais era **simplesmente ignorado**.
///
/// ⚠️⚠️ **E o gate que eu tinha era CEGO a isto:** ele contava os RÓTULOS no fonte do pintor
/// (`3 == 3`) e nunca olhava para este array. *Uma régua que conta rótulos não vê quantos
/// SEGMENTOS são pintados* — é a mesma forma do `SignalVerb::ALL` que foi a `9` com o array de ids
/// parado em `8`, e do chip do `Density` da escultura.
///
/// ⭐ Ele é lido por **TRÊS** consumidores — o pintor, o `populate` (que os regista) e o despacho
/// do clique —, logo uma entrada a mais cura os três de uma vez, e uma a menos mata os três.
pub const INSP_HUD_FIT: [NodeId; 3] = [
    hash_node_id("insp_hud_fit_0"),
    hash_node_id("insp_hud_fit_1"),
    hash_node_id("insp_hud_fit_2"),
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

/// A caixa **Keep on restart** do contador — *«outra vida, mesma pontuação»*.
pub const INSP_HUD_COUNTER_KEEP: NodeId = hash_node_id("insp_hud_counter_keep");
