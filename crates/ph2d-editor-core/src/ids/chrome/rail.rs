//! Left rail + transform tool NodeIds (HIERARCHY_ADD, TOOL_*, RAIL_*).
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
/// ⭐⭐⭐ **O PULLDOWN DA ÁREA** — os comandos do editor que tem o canvas, num chip só.
///
/// É a metade 2 da **D2** (*cabeçalho por área*) a aterrar **onde já se paga altura**: a faixa
/// própria foi construída e revertida em 2026-08-31 (`28 px`, `−1,5` ponto de área de desenho).
///
/// ⛔⛔ **E é UM chip, não nove — o número é MEDIDO.** Com as nove entradas cruas na fila (as seis
/// vistas nomeadas e os três gestos de câmera do módulo 3D) ela precisa de **2 linhas até no iPad
/// 12,9"**, o maior dos três alvos, e ainda transborda `2` chips para o `⋯`
/// (`the_area_costs_one_chip_and_the_bar_is_still_one_line`, mutação 6).
/// *Poupar altura gastando largura não poupa nada.*
///
/// ⚠️ A face dele é uma **leitura** (qual é a vista agora), não um rótulo fixo.
pub const AREA_COMMANDS: NodeId = hash_node_id("area_commands");

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
