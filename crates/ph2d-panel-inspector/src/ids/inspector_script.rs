//! **Os ids da secção SCRIPT** (TOP-20 #16, W3).
//!
//! ⚠️ **Uma linha por propriedade DECLARADA, e não um editor para a linha aberta** — ao contrário
//! do irmão [`super::inspector_statemachine`]. Lá cada linha tem três campos de texto; aqui cada
//! uma tem UM controlo, e é o idioma de toda engine madura (o `@export` do Godot, o `go.property`
//! do Defold): o artista vê todos os números do script de uma vez.
//!
//! ⚠️⚠️ **QUATRO tabelas de controlo com o MESMO tamanho** (número · caixa · texto · chip), porque
//! o tipo de uma linha é o do DEFAULT declarado e muda quando o ficheiro muda. A linha `i` pinta um
//! dos quatro e regista só esse no `HitIndex`.
//!
//! ⭐ **A quarta não é um tipo novo:** um enum é um `Text` cujo script declarou uma LISTA, e é a
//! lista — não o tipo — que separa o chip do campo livre. *Uma variante nova no valor custaria um
//! degrau no fio e não compraria nada.*
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

/// ⭐⭐⭐ **O CHIP da linha `i`** — uma propriedade de texto cujo script declarou uma LISTA.
///
/// ⚠️ **É a QUARTA tabela do mesmo tamanho**, pela razão do cabeçalho: o tipo de uma linha é o do
/// default declarado e muda quando o ficheiro muda. Um texto **sem** lista continua a pintar o
/// [`INSP_SCRIPT_TEXT`]; *a lista é o que separa um chip de um campo livre*.
pub const INSP_SCRIPT_ENUM: [NodeId; 32] = build_enum_chips();

/// ⭐⭐⭐ **As opções do popover — UMA tabela para TODAS as linhas, e isso é a lei.**
///
/// ⚠️⚠️ **Só um popover está aberto de cada vez**, logo uma tabela por linha custaria
/// `32 x 32 = 1024` ids para pintar, no máximo, `32`. ⛔ E a partilha tem um preço que tem de ser
/// pago com uma CERCA: se dois chips estivessem abertos, os mesmos ids seriam registados duas
/// vezes e o índice de acerto ficaria com o ÚLTIMO — o artista carregaria numa opção e escolheria
/// para a outra linha. ⇒ o pintor desenha as opções de **um** chip só, e há gate.
pub const INSP_SCRIPT_ENUM_OPT: [NodeId; 32] = build_enum_opts();

const fn build_enum_chips() -> [NodeId; 32] {
    [
        hash_node_id("insp_script_enum0"),
        hash_node_id("insp_script_enum1"),
        hash_node_id("insp_script_enum2"),
        hash_node_id("insp_script_enum3"),
        hash_node_id("insp_script_enum4"),
        hash_node_id("insp_script_enum5"),
        hash_node_id("insp_script_enum6"),
        hash_node_id("insp_script_enum7"),
        hash_node_id("insp_script_enum8"),
        hash_node_id("insp_script_enum9"),
        hash_node_id("insp_script_enum10"),
        hash_node_id("insp_script_enum11"),
        hash_node_id("insp_script_enum12"),
        hash_node_id("insp_script_enum13"),
        hash_node_id("insp_script_enum14"),
        hash_node_id("insp_script_enum15"),
        hash_node_id("insp_script_enum16"),
        hash_node_id("insp_script_enum17"),
        hash_node_id("insp_script_enum18"),
        hash_node_id("insp_script_enum19"),
        hash_node_id("insp_script_enum20"),
        hash_node_id("insp_script_enum21"),
        hash_node_id("insp_script_enum22"),
        hash_node_id("insp_script_enum23"),
        hash_node_id("insp_script_enum24"),
        hash_node_id("insp_script_enum25"),
        hash_node_id("insp_script_enum26"),
        hash_node_id("insp_script_enum27"),
        hash_node_id("insp_script_enum28"),
        hash_node_id("insp_script_enum29"),
        hash_node_id("insp_script_enum30"),
        hash_node_id("insp_script_enum31"),
    ]
}

const fn build_enum_opts() -> [NodeId; 32] {
    [
        hash_node_id("insp_script_enum_opt_00"),
        hash_node_id("insp_script_enum_opt_01"),
        hash_node_id("insp_script_enum_opt_02"),
        hash_node_id("insp_script_enum_opt_03"),
        hash_node_id("insp_script_enum_opt_04"),
        hash_node_id("insp_script_enum_opt_05"),
        hash_node_id("insp_script_enum_opt_06"),
        hash_node_id("insp_script_enum_opt_07"),
        hash_node_id("insp_script_enum_opt_08"),
        hash_node_id("insp_script_enum_opt_09"),
        hash_node_id("insp_script_enum_opt_10"),
        hash_node_id("insp_script_enum_opt_11"),
        hash_node_id("insp_script_enum_opt_12"),
        hash_node_id("insp_script_enum_opt_13"),
        hash_node_id("insp_script_enum_opt_14"),
        hash_node_id("insp_script_enum_opt_15"),
        hash_node_id("insp_script_enum_opt_16"),
        hash_node_id("insp_script_enum_opt_17"),
        hash_node_id("insp_script_enum_opt_18"),
        hash_node_id("insp_script_enum_opt_19"),
        hash_node_id("insp_script_enum_opt_20"),
        hash_node_id("insp_script_enum_opt_21"),
        hash_node_id("insp_script_enum_opt_22"),
        hash_node_id("insp_script_enum_opt_23"),
        hash_node_id("insp_script_enum_opt_24"),
        hash_node_id("insp_script_enum_opt_25"),
        hash_node_id("insp_script_enum_opt_26"),
        hash_node_id("insp_script_enum_opt_27"),
        hash_node_id("insp_script_enum_opt_28"),
        hash_node_id("insp_script_enum_opt_29"),
        hash_node_id("insp_script_enum_opt_30"),
        hash_node_id("insp_script_enum_opt_31"),
    ]
}

/// ⭐⭐⭐ **O campo X da linha `i`** — uma propriedade cujo default é um `ph2d.vec2`.
///
/// ⚠️⚠️ **São DOIS campos numa fileira só, e é isso que a wave compra:** duas propriedades
/// (`pos_x`, `pos_y`) já exprimiam uma posição — o que faltava era ela ser UMA coisa, com um
/// `Reset` só e uma edição só (ver o `SetVec2`).
pub const INSP_SCRIPT_VEC2_X: [NodeId; 32] = [
    hash_node_id("insp_script_vec2x0"),
    hash_node_id("insp_script_vec2x1"),
    hash_node_id("insp_script_vec2x2"),
    hash_node_id("insp_script_vec2x3"),
    hash_node_id("insp_script_vec2x4"),
    hash_node_id("insp_script_vec2x5"),
    hash_node_id("insp_script_vec2x6"),
    hash_node_id("insp_script_vec2x7"),
    hash_node_id("insp_script_vec2x8"),
    hash_node_id("insp_script_vec2x9"),
    hash_node_id("insp_script_vec2x10"),
    hash_node_id("insp_script_vec2x11"),
    hash_node_id("insp_script_vec2x12"),
    hash_node_id("insp_script_vec2x13"),
    hash_node_id("insp_script_vec2x14"),
    hash_node_id("insp_script_vec2x15"),
    hash_node_id("insp_script_vec2x16"),
    hash_node_id("insp_script_vec2x17"),
    hash_node_id("insp_script_vec2x18"),
    hash_node_id("insp_script_vec2x19"),
    hash_node_id("insp_script_vec2x20"),
    hash_node_id("insp_script_vec2x21"),
    hash_node_id("insp_script_vec2x22"),
    hash_node_id("insp_script_vec2x23"),
    hash_node_id("insp_script_vec2x24"),
    hash_node_id("insp_script_vec2x25"),
    hash_node_id("insp_script_vec2x26"),
    hash_node_id("insp_script_vec2x27"),
    hash_node_id("insp_script_vec2x28"),
    hash_node_id("insp_script_vec2x29"),
    hash_node_id("insp_script_vec2x30"),
    hash_node_id("insp_script_vec2x31"),
];

/// O campo Y da linha `i` — ver [`INSP_SCRIPT_VEC2_X`].
pub const INSP_SCRIPT_VEC2_Y: [NodeId; 32] = [
    hash_node_id("insp_script_vec2y0"),
    hash_node_id("insp_script_vec2y1"),
    hash_node_id("insp_script_vec2y2"),
    hash_node_id("insp_script_vec2y3"),
    hash_node_id("insp_script_vec2y4"),
    hash_node_id("insp_script_vec2y5"),
    hash_node_id("insp_script_vec2y6"),
    hash_node_id("insp_script_vec2y7"),
    hash_node_id("insp_script_vec2y8"),
    hash_node_id("insp_script_vec2y9"),
    hash_node_id("insp_script_vec2y10"),
    hash_node_id("insp_script_vec2y11"),
    hash_node_id("insp_script_vec2y12"),
    hash_node_id("insp_script_vec2y13"),
    hash_node_id("insp_script_vec2y14"),
    hash_node_id("insp_script_vec2y15"),
    hash_node_id("insp_script_vec2y16"),
    hash_node_id("insp_script_vec2y17"),
    hash_node_id("insp_script_vec2y18"),
    hash_node_id("insp_script_vec2y19"),
    hash_node_id("insp_script_vec2y20"),
    hash_node_id("insp_script_vec2y21"),
    hash_node_id("insp_script_vec2y22"),
    hash_node_id("insp_script_vec2y23"),
    hash_node_id("insp_script_vec2y24"),
    hash_node_id("insp_script_vec2y25"),
    hash_node_id("insp_script_vec2y26"),
    hash_node_id("insp_script_vec2y27"),
    hash_node_id("insp_script_vec2y28"),
    hash_node_id("insp_script_vec2y29"),
    hash_node_id("insp_script_vec2y30"),
    hash_node_id("insp_script_vec2y31"),
];

/// ⭐⭐⭐ **A AMOSTRA de cor da linha `i`** — ela abre o selector da casa.
///
/// ⚠️ **Ela é registada como amostra do selector** (`register_picker_swatch`), que é o que faz o
/// clique abrir o selector partilhado e a cor escolhida voltar por `widget_color(id)` — o mesmo
/// caminho das amostras de tinta do Sprite.
///
/// ⛔⛔ **Um id POR LINHA, e a razão está medida noutra linha desta casa:** em 19/09 o painel do
/// modelador teve CINCO fileiras de cor a partilharem um id, e mexer numa mudava as cinco. *Duas
/// amostras com o mesmo id não são duas amostras.*
pub const INSP_SCRIPT_COLOR: [NodeId; 32] = [
    hash_node_id("insp_script_color0"),
    hash_node_id("insp_script_color1"),
    hash_node_id("insp_script_color2"),
    hash_node_id("insp_script_color3"),
    hash_node_id("insp_script_color4"),
    hash_node_id("insp_script_color5"),
    hash_node_id("insp_script_color6"),
    hash_node_id("insp_script_color7"),
    hash_node_id("insp_script_color8"),
    hash_node_id("insp_script_color9"),
    hash_node_id("insp_script_color10"),
    hash_node_id("insp_script_color11"),
    hash_node_id("insp_script_color12"),
    hash_node_id("insp_script_color13"),
    hash_node_id("insp_script_color14"),
    hash_node_id("insp_script_color15"),
    hash_node_id("insp_script_color16"),
    hash_node_id("insp_script_color17"),
    hash_node_id("insp_script_color18"),
    hash_node_id("insp_script_color19"),
    hash_node_id("insp_script_color20"),
    hash_node_id("insp_script_color21"),
    hash_node_id("insp_script_color22"),
    hash_node_id("insp_script_color23"),
    hash_node_id("insp_script_color24"),
    hash_node_id("insp_script_color25"),
    hash_node_id("insp_script_color26"),
    hash_node_id("insp_script_color27"),
    hash_node_id("insp_script_color28"),
    hash_node_id("insp_script_color29"),
    hash_node_id("insp_script_color30"),
    hash_node_id("insp_script_color31"),
];
