//! **Os ids da secção SCRIPT** (TOP-20 #16, W3).
//!
//! ⚠️ **Uma linha por propriedade DECLARADA, e não um editor para a linha aberta** — ao contrário
//! do irmão [`super::inspector_statemachine`]. Lá cada linha tem três campos de texto; aqui cada
//! uma tem UM controlo, e é o idioma de toda engine madura (o `@export` do Godot, o `go.property`
//! do Defold): o artista vê todos os números do script de uma vez.
//!
//! ⚠️⚠️ **Três tabelas de controlo com o MESMO tamanho** (número · caixa · texto), porque o tipo de
//! uma linha é o do DEFAULT declarado e muda quando o ficheiro muda. A linha `i` pinta um dos três
//! e regista só esse no `HitIndex`.
//!
//! ⭐ **O tamanho é o `ph2d_script::PROPS_MAX`**, e os dois são o MESMO facto (há gate na shell, que
//! vê as duas crates): *um modelo que aceita o que o painel não mostra produz estado inalcançável*.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O caminho do ficheiro, escrito à mão.
pub const INSP_SCRIPT_SOURCE: NodeId = hash_node_id("insp_script_source");
/// `Browse` — a shell abre o diálogo.
pub const INSP_SCRIPT_BROWSE: NodeId = hash_node_id("insp_script_browse");

/// O campo NUMÉRICO da linha `i` (uma propriedade cujo default é um `number`).
pub const INSP_SCRIPT_NUM: [NodeId; 32] = [
    hash_node_id("insp_script_num0"),
    hash_node_id("insp_script_num1"),
    hash_node_id("insp_script_num2"),
    hash_node_id("insp_script_num3"),
    hash_node_id("insp_script_num4"),
    hash_node_id("insp_script_num5"),
    hash_node_id("insp_script_num6"),
    hash_node_id("insp_script_num7"),
    hash_node_id("insp_script_num8"),
    hash_node_id("insp_script_num9"),
    hash_node_id("insp_script_num10"),
    hash_node_id("insp_script_num11"),
    hash_node_id("insp_script_num12"),
    hash_node_id("insp_script_num13"),
    hash_node_id("insp_script_num14"),
    hash_node_id("insp_script_num15"),
    hash_node_id("insp_script_num16"),
    hash_node_id("insp_script_num17"),
    hash_node_id("insp_script_num18"),
    hash_node_id("insp_script_num19"),
    hash_node_id("insp_script_num20"),
    hash_node_id("insp_script_num21"),
    hash_node_id("insp_script_num22"),
    hash_node_id("insp_script_num23"),
    hash_node_id("insp_script_num24"),
    hash_node_id("insp_script_num25"),
    hash_node_id("insp_script_num26"),
    hash_node_id("insp_script_num27"),
    hash_node_id("insp_script_num28"),
    hash_node_id("insp_script_num29"),
    hash_node_id("insp_script_num30"),
    hash_node_id("insp_script_num31"),
];

/// A CAIXA da linha `i` (um `boolean`).
pub const INSP_SCRIPT_BOOL: [NodeId; 32] = [
    hash_node_id("insp_script_bool0"),
    hash_node_id("insp_script_bool1"),
    hash_node_id("insp_script_bool2"),
    hash_node_id("insp_script_bool3"),
    hash_node_id("insp_script_bool4"),
    hash_node_id("insp_script_bool5"),
    hash_node_id("insp_script_bool6"),
    hash_node_id("insp_script_bool7"),
    hash_node_id("insp_script_bool8"),
    hash_node_id("insp_script_bool9"),
    hash_node_id("insp_script_bool10"),
    hash_node_id("insp_script_bool11"),
    hash_node_id("insp_script_bool12"),
    hash_node_id("insp_script_bool13"),
    hash_node_id("insp_script_bool14"),
    hash_node_id("insp_script_bool15"),
    hash_node_id("insp_script_bool16"),
    hash_node_id("insp_script_bool17"),
    hash_node_id("insp_script_bool18"),
    hash_node_id("insp_script_bool19"),
    hash_node_id("insp_script_bool20"),
    hash_node_id("insp_script_bool21"),
    hash_node_id("insp_script_bool22"),
    hash_node_id("insp_script_bool23"),
    hash_node_id("insp_script_bool24"),
    hash_node_id("insp_script_bool25"),
    hash_node_id("insp_script_bool26"),
    hash_node_id("insp_script_bool27"),
    hash_node_id("insp_script_bool28"),
    hash_node_id("insp_script_bool29"),
    hash_node_id("insp_script_bool30"),
    hash_node_id("insp_script_bool31"),
];

/// O campo de TEXTO da linha `i` (uma `string`).
pub const INSP_SCRIPT_TEXT: [NodeId; 32] = [
    hash_node_id("insp_script_text0"),
    hash_node_id("insp_script_text1"),
    hash_node_id("insp_script_text2"),
    hash_node_id("insp_script_text3"),
    hash_node_id("insp_script_text4"),
    hash_node_id("insp_script_text5"),
    hash_node_id("insp_script_text6"),
    hash_node_id("insp_script_text7"),
    hash_node_id("insp_script_text8"),
    hash_node_id("insp_script_text9"),
    hash_node_id("insp_script_text10"),
    hash_node_id("insp_script_text11"),
    hash_node_id("insp_script_text12"),
    hash_node_id("insp_script_text13"),
    hash_node_id("insp_script_text14"),
    hash_node_id("insp_script_text15"),
    hash_node_id("insp_script_text16"),
    hash_node_id("insp_script_text17"),
    hash_node_id("insp_script_text18"),
    hash_node_id("insp_script_text19"),
    hash_node_id("insp_script_text20"),
    hash_node_id("insp_script_text21"),
    hash_node_id("insp_script_text22"),
    hash_node_id("insp_script_text23"),
    hash_node_id("insp_script_text24"),
    hash_node_id("insp_script_text25"),
    hash_node_id("insp_script_text26"),
    hash_node_id("insp_script_text27"),
    hash_node_id("insp_script_text28"),
    hash_node_id("insp_script_text29"),
    hash_node_id("insp_script_text30"),
    hash_node_id("insp_script_text31"),
];

/// `Reset` da linha `i` — só pintado quando o valor é PRÓPRIO (larga-o, e o objecto volta ao
/// default do script).
pub const INSP_SCRIPT_RESET: [NodeId; 32] = [
    hash_node_id("insp_script_reset0"),
    hash_node_id("insp_script_reset1"),
    hash_node_id("insp_script_reset2"),
    hash_node_id("insp_script_reset3"),
    hash_node_id("insp_script_reset4"),
    hash_node_id("insp_script_reset5"),
    hash_node_id("insp_script_reset6"),
    hash_node_id("insp_script_reset7"),
    hash_node_id("insp_script_reset8"),
    hash_node_id("insp_script_reset9"),
    hash_node_id("insp_script_reset10"),
    hash_node_id("insp_script_reset11"),
    hash_node_id("insp_script_reset12"),
    hash_node_id("insp_script_reset13"),
    hash_node_id("insp_script_reset14"),
    hash_node_id("insp_script_reset15"),
    hash_node_id("insp_script_reset16"),
    hash_node_id("insp_script_reset17"),
    hash_node_id("insp_script_reset18"),
    hash_node_id("insp_script_reset19"),
    hash_node_id("insp_script_reset20"),
    hash_node_id("insp_script_reset21"),
    hash_node_id("insp_script_reset22"),
    hash_node_id("insp_script_reset23"),
    hash_node_id("insp_script_reset24"),
    hash_node_id("insp_script_reset25"),
    hash_node_id("insp_script_reset26"),
    hash_node_id("insp_script_reset27"),
    hash_node_id("insp_script_reset28"),
    hash_node_id("insp_script_reset29"),
    hash_node_id("insp_script_reset30"),
    hash_node_id("insp_script_reset31"),
];

/// `Remove` do órfão `i` — larga um valor cuja propriedade saiu do script (a divergência D2).
pub const INSP_SCRIPT_ORPHAN_REMOVE: [NodeId; 32] = [
    hash_node_id("insp_script_orphan_remove0"),
    hash_node_id("insp_script_orphan_remove1"),
    hash_node_id("insp_script_orphan_remove2"),
    hash_node_id("insp_script_orphan_remove3"),
    hash_node_id("insp_script_orphan_remove4"),
    hash_node_id("insp_script_orphan_remove5"),
    hash_node_id("insp_script_orphan_remove6"),
    hash_node_id("insp_script_orphan_remove7"),
    hash_node_id("insp_script_orphan_remove8"),
    hash_node_id("insp_script_orphan_remove9"),
    hash_node_id("insp_script_orphan_remove10"),
    hash_node_id("insp_script_orphan_remove11"),
    hash_node_id("insp_script_orphan_remove12"),
    hash_node_id("insp_script_orphan_remove13"),
    hash_node_id("insp_script_orphan_remove14"),
    hash_node_id("insp_script_orphan_remove15"),
    hash_node_id("insp_script_orphan_remove16"),
    hash_node_id("insp_script_orphan_remove17"),
    hash_node_id("insp_script_orphan_remove18"),
    hash_node_id("insp_script_orphan_remove19"),
    hash_node_id("insp_script_orphan_remove20"),
    hash_node_id("insp_script_orphan_remove21"),
    hash_node_id("insp_script_orphan_remove22"),
    hash_node_id("insp_script_orphan_remove23"),
    hash_node_id("insp_script_orphan_remove24"),
    hash_node_id("insp_script_orphan_remove25"),
    hash_node_id("insp_script_orphan_remove26"),
    hash_node_id("insp_script_orphan_remove27"),
    hash_node_id("insp_script_orphan_remove28"),
    hash_node_id("insp_script_orphan_remove29"),
    hash_node_id("insp_script_orphan_remove30"),
    hash_node_id("insp_script_orphan_remove31"),
];
