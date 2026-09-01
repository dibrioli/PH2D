//! Left rail + transform tool NodeIds (HIERARCHY_ADD, TOOL_*, RAIL_*).
use super::painter::fnv_node_id_runtime;
use super::{NodeId, hash_node_id};

pub const HIERARCHY_ADD: NodeId = hash_node_id("hierarchy_add");

pub const TOOL_TRANSLATE: NodeId = hash_node_id("tool_translate");
pub const TOOL_ROTATE: NodeId = hash_node_id("tool_rotate");
pub const TOOL_SCALE: NodeId = hash_node_id("tool_scale");
pub const TOOL_PIVOT: NodeId = hash_node_id("tool_pivot");
pub const TOOL_SPACE: NodeId = hash_node_id("tool_space");
pub const TOOL_PROJECTION: NodeId = hash_node_id("tool_projection");
pub const TOOL_HOME: NodeId = hash_node_id("tool_home");
/// ⭐⭐ **O `⋯` da FILA de ferramentas** — o que não coube numa linha vive atrás dele.
///
/// ⛔ Ele existe porque a faixa **não pode crescer**: o alvo é tablet, e no iPad 11 e no mini ela
/// dobrava (`54 → 108 px`) no instante em que o pincel entrava em mãos — `−3,3` pontos de área de
/// desenho, justamente quando o ecrã faz falta
/// (`docs/UI_New_and_Simple/medicoes/06_o_orcamento_de_ecra_em_tablet.md`).
pub const TOOL_BAR_OVERFLOW: NodeId = hash_node_id("tool_bar_overflow");
/// ⭐⭐⭐ **UM PULLDOWN DA ÁREA** — os comandos do editor que tem o canvas, agrupados por LEITURA.
///
/// É a metade 2 da **D2** (*cabeçalho por área*) a aterrar **onde já se paga altura**: a faixa
/// própria foi construída e revertida em 2026-08-31 (`28 px`, `−1,5` ponto de área de desenho).
///
/// ⛔⛔ **São PULLDOWNS, nunca os comandos crus — e o número é MEDIDO.** Com as nove entradas cruas
/// na fila (as seis vistas nomeadas e os três gestos de câmera do módulo 3D) ela precisa de
/// **2 linhas até no iPad 12,9"**, o maior dos três alvos, e ainda transborda `2` chips para o `⋯`
/// (`the_area_costs_two_chips_and_the_bar_is_still_one_line`, mutação 6).
/// *Poupar altura gastando largura não poupa nada.*
///
/// ⭐ **E o ORÇAMENTO de chips da área é `3`, medido em 2026-09-01** (sonda sobre `bar_split` +
/// `horizontal_lines`, os três tablets com o módulo armado):
///
/// | alvo | largura da área | 1 chip | 2 | 3 | 4 |
/// |---|---:|---|---|---|---|
/// | iPad 12,9" | `754,0` | 1 linha | 1 | 1 | 1 |
/// | iPad 11 | `582,0` | 1 linha | 1 | 1 | **2** |
/// | iPad mini | `521,0` | 1 linha | 1 | 1 | **2** |
///
/// ⇒ o `MAX` abaixo é de **REGISTO** (quantos ids o `left_rail` cunha às cegas); quem manda de
/// facto é a **largura**, e quem a mede é o gate — que corre com o que o módulo publicou, não com
/// este número. ⛔ Passar de 3 não parte nada (o `⋯` absorve), mas custa a 2.ª linha nos dois
/// alvos pequenos, que é precisamente o que a entrega 32 existe para não pagar.
///
/// ⚠️ A face de cada um é uma **leitura** (qual é a vista agora, qual é o verbo do gizmo), não um
/// rótulo fixo — é isso que faz o chip valer o lugar dele mesmo fechado.
#[must_use]
pub fn area_menu_button(slot: u32) -> NodeId {
    fnv_node_id_runtime(&format!("area.menu.{slot}"))
}

/// Quantos pulldowns de área o `left_rail` regista às cegas — ver [`area_menu_button`].
pub const MAX_AREA_MENUS: u32 = 4;

pub const TOOL_UNDO: NodeId = hash_node_id("tool_undo");
pub const TOOL_REDO: NodeId = hash_node_id("tool_redo");
/// Show/Hide toggles for the side panels — top of the left rail.
/// `Pressed` state == panel currently visible.
pub const RAIL_SHOW_INSPECTOR: NodeId = hash_node_id("rail_show_inspector");
pub const RAIL_SHOW_HIERARCHY: NodeId = hash_node_id("rail_show_hierarchy");
/// Frosted-glass backdrop of the side rail. Painted before the
/// chips so chip clicks win; clicks on the rail's empty space
/// (between chips, around dividers) land here.
pub const RAIL_BACKDROP: NodeId = hash_node_id("rail_backdrop");
