//! **Os ids da secção TAGS** (TOP-20 #9, W3a) — os controlos, que são do PAINEL.
//!
//! ⚠️ O cabeçalho colapsável e o ponto de cor **não** vivem aqui: eles são da tabela
//! `ph2d_editor_core::ids::LIVE_SECTIONS`, que é o que faz uma secção nascer viva nos quatro
//! sítios que ninguém liga entre si. Aqui ficam só os ids que esta crate pinta e despacha.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Os CHIPS** — um por tag do objecto, até ao cap de [`ph2d_ecs::tags::TAGS_MAX`].
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, e há gate a prendê-los: *um modelo que
/// aceita o que o painel não mostra produz estado inalcançável por gesto nenhum* — a lei que o
/// `ANIM_TAGS_MAX` pagou e que o `INSP_TIMER_ROW` repete.
pub const INSP_TAGS_CHIP: [NodeId; 16] = [
    hash_node_id("insp_tags_chip_00"),
    hash_node_id("insp_tags_chip_01"),
    hash_node_id("insp_tags_chip_02"),
    hash_node_id("insp_tags_chip_03"),
    hash_node_id("insp_tags_chip_04"),
    hash_node_id("insp_tags_chip_05"),
    hash_node_id("insp_tags_chip_06"),
    hash_node_id("insp_tags_chip_07"),
    hash_node_id("insp_tags_chip_08"),
    hash_node_id("insp_tags_chip_09"),
    hash_node_id("insp_tags_chip_10"),
    hash_node_id("insp_tags_chip_11"),
    hash_node_id("insp_tags_chip_12"),
    hash_node_id("insp_tags_chip_13"),
    hash_node_id("insp_tags_chip_14"),
    hash_node_id("insp_tags_chip_15"),
];

/// **O chip da caixa de escolha** — o que abre a lista das tags do projecto.
pub const INSP_TAGS_PICK: NodeId = hash_node_id("insp_tags_pick");

/// **As opções da caixa de escolha** — as tags que o objecto ainda NÃO tem, filtradas pelo texto.
///
/// ⛔⛔ **Este array NÃO é o cap de um modelo — a árvore não tem cap** (medido: `9 344` tags abrem
/// em `9,2 ms`). Ele é o que a lista consegue mostrar de uma vez, e o recurso é o POPOVER: ele é
/// preso à altura da coluna do Inspector e rola.
///
/// **MEDIDO** (`tests/it/a_lista_de_tags_cabe_no_popover.rs`, linha de `22,0 px`):
///
/// | janela | região do popover | linhas visíveis |
/// |---|---:|---:|
/// | workstation `2560×1440` | `1376,0 px` | **62** |
/// | portátil `1600×900` | `836,0 px` | 38 |
/// | tablet `1280×800` | `736,0 px` | 33 |
///
/// ⇒ **`64`**, que é o ecrã maior mais dois — sem isso, rolar seria a única forma de ver o que a
/// busca já tinha encontrado, que é o oposto do que uma busca faz. ⚠️ **Quem mexer na altura da
/// linha move este teto**, e o gate reprova com a tabela nova impressa.
///
/// ⚠️ **E o que passa daqui é DITO, nunca cortado em silêncio**: a linha final da lista conta as
/// que sobram e manda escrever mais — *um chooser que esconde metade dos resultados ensina que a
/// tag não existe*.
pub const INSP_TAGS_OPT: [NodeId; 64] = [
    hash_node_id("insp_tags_opt_00"),
    hash_node_id("insp_tags_opt_01"),
    hash_node_id("insp_tags_opt_02"),
    hash_node_id("insp_tags_opt_03"),
    hash_node_id("insp_tags_opt_04"),
    hash_node_id("insp_tags_opt_05"),
    hash_node_id("insp_tags_opt_06"),
    hash_node_id("insp_tags_opt_07"),
    hash_node_id("insp_tags_opt_08"),
    hash_node_id("insp_tags_opt_09"),
    hash_node_id("insp_tags_opt_10"),
    hash_node_id("insp_tags_opt_11"),
    hash_node_id("insp_tags_opt_12"),
    hash_node_id("insp_tags_opt_13"),
    hash_node_id("insp_tags_opt_14"),
    hash_node_id("insp_tags_opt_15"),
    hash_node_id("insp_tags_opt_16"),
    hash_node_id("insp_tags_opt_17"),
    hash_node_id("insp_tags_opt_18"),
    hash_node_id("insp_tags_opt_19"),
    hash_node_id("insp_tags_opt_20"),
    hash_node_id("insp_tags_opt_21"),
    hash_node_id("insp_tags_opt_22"),
    hash_node_id("insp_tags_opt_23"),
    hash_node_id("insp_tags_opt_24"),
    hash_node_id("insp_tags_opt_25"),
    hash_node_id("insp_tags_opt_26"),
    hash_node_id("insp_tags_opt_27"),
    hash_node_id("insp_tags_opt_28"),
    hash_node_id("insp_tags_opt_29"),
    hash_node_id("insp_tags_opt_30"),
    hash_node_id("insp_tags_opt_31"),
    hash_node_id("insp_tags_opt_32"),
    hash_node_id("insp_tags_opt_33"),
    hash_node_id("insp_tags_opt_34"),
    hash_node_id("insp_tags_opt_35"),
    hash_node_id("insp_tags_opt_36"),
    hash_node_id("insp_tags_opt_37"),
    hash_node_id("insp_tags_opt_38"),
    hash_node_id("insp_tags_opt_39"),
    hash_node_id("insp_tags_opt_40"),
    hash_node_id("insp_tags_opt_41"),
    hash_node_id("insp_tags_opt_42"),
    hash_node_id("insp_tags_opt_43"),
    hash_node_id("insp_tags_opt_44"),
    hash_node_id("insp_tags_opt_45"),
    hash_node_id("insp_tags_opt_46"),
    hash_node_id("insp_tags_opt_47"),
    hash_node_id("insp_tags_opt_48"),
    hash_node_id("insp_tags_opt_49"),
    hash_node_id("insp_tags_opt_50"),
    hash_node_id("insp_tags_opt_51"),
    hash_node_id("insp_tags_opt_52"),
    hash_node_id("insp_tags_opt_53"),
    hash_node_id("insp_tags_opt_54"),
    hash_node_id("insp_tags_opt_55"),
    hash_node_id("insp_tags_opt_56"),
    hash_node_id("insp_tags_opt_57"),
    hash_node_id("insp_tags_opt_58"),
    hash_node_id("insp_tags_opt_59"),
    hash_node_id("insp_tags_opt_60"),
    hash_node_id("insp_tags_opt_61"),
    hash_node_id("insp_tags_opt_62"),
    hash_node_id("insp_tags_opt_63"),
];

/// **O campo do nome de uma tag NOVA** — e também a BUSCA que filtra a lista acima.
///
/// ⭐⭐ **Um campo, duas perguntas, e isso é o desenho, não uma economia**: escrever `Fly` estreita
/// a lista *e* arma o `Create "Fly"`. Dois campos fariam o artista escrever o nome duas vezes para
/// descobrir que a tag já existia — que é exactamente o gesto que a busca existe para evitar.
pub const INSP_TAGS_NEW: NodeId = hash_node_id("insp_tags_new");

/// **O botão que cria a tag escrita no campo acima e marca o objecto com ela**, num gesto.
pub const INSP_TAGS_CREATE: NodeId = hash_node_id("insp_tags_create");
