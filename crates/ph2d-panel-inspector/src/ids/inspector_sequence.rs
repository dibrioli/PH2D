//! **Os ids da secção SEQUENCE** (TOP-20 #19, W3).
//!
//! ⚠️⚠️ **O `16` do array de opções é o `ph2d_timeline::MAX_CONTAINERS`, e os dois são o MESMO
//! facto** — a mesma lei do `PROPS_MAX` do script e do `STATES_MAX` do cérebro, e escrito da mesma
//! maneira: o literal aqui, o gate na SHELL, que é a única crate que vê a timeline e este painel ao
//! mesmo tempo. ⛔ *Importar a `ph2d-timeline` para dentro do painel seria pôr o documento da
//! animação na closure de compilação de um painel que, por desenho, não o conhece* — o snapshot
//! traz-lhe os nomes prontos.
//!
//! *Uma cutscene que o documento aceita e o selector não consegue endereçar é uma cutscene que o
//! artista vê na aba Containers e não consegue escolher aqui* — e o chrome não sabe cunhar um id em
//! runtime.

use ph2d_a11y::NodeId;

use ph2d_tool_registry::hash_node_id;

/// **A cutscene deste objecto — o CHIP do selector.** As entradas dele são [`INSP_SEQ_OPT`].
///
/// ⚠️ **O que o `Dropdown` guarda é o `open`, e nunca a escolha**: quem é dono da escolha é o
/// snapshot, que o quadro seguinte relê da cena. Escrever a escolha aqui abriria a segunda porta
/// para o mesmo estado, e ela mentiria exactamente no caso em que a shell recusasse a edição.
pub const INSP_SEQ_PICK: NodeId = hash_node_id("insp_seq_pick");

/// As entradas do selector — uma por container do documento, pela ordem dele.
///
/// ⚠️ **Elas são BOTÕES** (as linhas do popover), e o despachante decide pelo `is_focusable`: sem
/// registo no `populate`, o clique numa opção é engolido **em silêncio**, com o rectângulo pintado
/// na mesma. É a doença que esta crate já pagou sete vezes.
pub const INSP_SEQ_OPT: [NodeId; 16] = [
    hash_node_id("insp_seq_opt_0"),
    hash_node_id("insp_seq_opt_1"),
    hash_node_id("insp_seq_opt_2"),
    hash_node_id("insp_seq_opt_3"),
    hash_node_id("insp_seq_opt_4"),
    hash_node_id("insp_seq_opt_5"),
    hash_node_id("insp_seq_opt_6"),
    hash_node_id("insp_seq_opt_7"),
    hash_node_id("insp_seq_opt_8"),
    hash_node_id("insp_seq_opt_9"),
    hash_node_id("insp_seq_opt_10"),
    hash_node_id("insp_seq_opt_11"),
    hash_node_id("insp_seq_opt_12"),
    hash_node_id("insp_seq_opt_13"),
    hash_node_id("insp_seq_opt_14"),
    hash_node_id("insp_seq_opt_15"),
];

/// ⭐ **O botão que LARGA a cutscene** — o caminho de volta ao estado vazio.
///
/// ⚠️ **Sem ele o selector é uma porta de sentido único:** escolher é um clique e desescolher seria
/// impossível, porque uma lista de opções não tem a opção *«nenhuma»* sem alguém a inventar — e
/// inventá-la como linha do popover faria a posição das opções deixar de ser o índice do container,
/// que é a lei que este array declara.
pub const INSP_SEQ_CLEAR: NodeId = hash_node_id("insp_seq_clear");
