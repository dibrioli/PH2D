//! Os ids da seção **Component** do Inspector (ADR-0164 / F5).

use super::hash_node_id;
use ph2d_a11y::NodeId;

// ⛔ **Havia um `INSP_INSTANCE_SECTION` e ele SAIU** (Enio, 2026-08-27): a superfície deixou de ser
// uma seção e passou a ser um CARTÃO no topo, que não tem cabeçalho, nem recolher, nem âncora de
// nota — logo não tem o que registar. *Um id que nada regista é um id que nada resolve.*

/// ⭐ **Limpar as excepções SEM ALVO** (F5.3).
///
/// ⚠️ **Um botão, e não uma limpeza automática:** a lei do *«unused overrides»* do Unity é que
/// elas **nunca** se apagam sozinhas — sair por causa de um `Delete` no mestre é perder trabalho
/// do artista em silêncio. ⇒ o gesto existe, e é explícito.
pub const INSP_INSTANCE_CLEAR_ORPHANS: NodeId = hash_node_id("insp_instance_clear_orphans");

/// ⭐⭐⭐ **Quantas excepções SEM ALVO ganham botão próprio** (F5.3-ter).
///
/// ⚠️ **É um teto de TABELA DE IDS, e ele diz de que recurso é** — a mesma razão do
/// [`MAX_INSTANCE_APPLY_LEVELS`]: os ids são `const` para o censo do
/// `hit_indexed_ids_are_registered` os poder **ver**, e uma tabela `const` tem tamanho.
///
/// ⛔ **A LISTA continua sem tecto** (o painel rola), e a razão está escrita no modelo: *esconder
/// linhas com um botão que apaga TUDO seria esconder exactamente o que o gesto destrói*. O que o
/// tecto limita é quantas ganham o `✕` — e a linha que diz quantas ficaram sem ele é obrigatória.
///
/// **O número:** um órfão nasce de um par *(peça apagada na receita, componente overridado nela)*,
/// e apagar uma peça de uma receita é um gesto deliberado. `16` cobre a população real com folga;
/// acima disso o *Clear all* continua a alcançar todas.
pub const MAX_INSTANCE_ORPHAN_ROWS: usize = 16;

/// O `✕` de cada excepção sem alvo — um por linha, na ordem em que o cartão as pinta.
pub const INSP_INSTANCE_DROP_ORPHAN: [NodeId; MAX_INSTANCE_ORPHAN_ROWS] = [
    hash_node_id("insp_instance_drop_orphan_0"),
    hash_node_id("insp_instance_drop_orphan_1"),
    hash_node_id("insp_instance_drop_orphan_2"),
    hash_node_id("insp_instance_drop_orphan_3"),
    hash_node_id("insp_instance_drop_orphan_4"),
    hash_node_id("insp_instance_drop_orphan_5"),
    hash_node_id("insp_instance_drop_orphan_6"),
    hash_node_id("insp_instance_drop_orphan_7"),
    hash_node_id("insp_instance_drop_orphan_8"),
    hash_node_id("insp_instance_drop_orphan_9"),
    hash_node_id("insp_instance_drop_orphan_10"),
    hash_node_id("insp_instance_drop_orphan_11"),
    hash_node_id("insp_instance_drop_orphan_12"),
    hash_node_id("insp_instance_drop_orphan_13"),
    hash_node_id("insp_instance_drop_orphan_14"),
    hash_node_id("insp_instance_drop_orphan_15"),
];

/// A linha que este id representa — a leitura INVERSA da tabela, como as duas irmãs deste ficheiro.
#[must_use]
pub fn instance_drop_orphan(id: NodeId) -> Option<usize> {
    INSP_INSTANCE_DROP_ORPHAN.iter().position(|c| *c == id)
}

/// ⭐⭐⭐ **Quantas peças RECUSADAS ganham botão de devolver** (F5.10).
///
/// ⚠️ Mesmo tecto e mesma razão do [`MAX_INSTANCE_ORPHAN_ROWS`]: os ids são `const` para o censo os
/// poder **ver**, e uma tabela `const` tem tamanho. ⛔ Acima dele a saída é o *Revert* da raiz, que
/// devolve **todas** — e a linha que fica sem botão di-lo.
pub const MAX_INSTANCE_REMOVED_ROWS: usize = 16;

/// O *Put back* de cada peça recusada, na ordem em que o cartão as pinta.
pub const INSP_INSTANCE_RESTORE_PIECE: [NodeId; MAX_INSTANCE_REMOVED_ROWS] = [
    hash_node_id("insp_instance_restore_piece_0"),
    hash_node_id("insp_instance_restore_piece_1"),
    hash_node_id("insp_instance_restore_piece_2"),
    hash_node_id("insp_instance_restore_piece_3"),
    hash_node_id("insp_instance_restore_piece_4"),
    hash_node_id("insp_instance_restore_piece_5"),
    hash_node_id("insp_instance_restore_piece_6"),
    hash_node_id("insp_instance_restore_piece_7"),
    hash_node_id("insp_instance_restore_piece_8"),
    hash_node_id("insp_instance_restore_piece_9"),
    hash_node_id("insp_instance_restore_piece_10"),
    hash_node_id("insp_instance_restore_piece_11"),
    hash_node_id("insp_instance_restore_piece_12"),
    hash_node_id("insp_instance_restore_piece_13"),
    hash_node_id("insp_instance_restore_piece_14"),
    hash_node_id("insp_instance_restore_piece_15"),
];

/// A linha que este id representa — a leitura INVERSA da tabela, como as irmãs deste ficheiro.
#[must_use]
pub fn instance_restore_piece(id: NodeId) -> Option<usize> {
    INSP_INSTANCE_RESTORE_PIECE.iter().position(|c| *c == id)
}

/// ⭐⭐⭐ **Quantas peças ACRESCENTADAS ganham botão de aplicar** (F5.11).
///
/// ⚠️ Mesmo tecto e mesma razão das duas irmãs acima: os ids são `const` para o censo os poder
/// **ver**, e uma tabela `const` tem tamanho. ⛔ Acima dele a saída é o *Apply to Master* do menu
/// da linha, que alcança **uma** peça de cada vez sem passar por esta tabela — e a linha que fica
/// sem botão di-lo.
pub const MAX_INSTANCE_ADDED_ROWS: usize = 16;

/// O *Add … to …* de cada peça acrescentada, na ordem em que o cartão as pinta.
pub const INSP_INSTANCE_APPLY_ADDED: [NodeId; MAX_INSTANCE_ADDED_ROWS] = [
    hash_node_id("insp_instance_apply_added_0"),
    hash_node_id("insp_instance_apply_added_1"),
    hash_node_id("insp_instance_apply_added_2"),
    hash_node_id("insp_instance_apply_added_3"),
    hash_node_id("insp_instance_apply_added_4"),
    hash_node_id("insp_instance_apply_added_5"),
    hash_node_id("insp_instance_apply_added_6"),
    hash_node_id("insp_instance_apply_added_7"),
    hash_node_id("insp_instance_apply_added_8"),
    hash_node_id("insp_instance_apply_added_9"),
    hash_node_id("insp_instance_apply_added_10"),
    hash_node_id("insp_instance_apply_added_11"),
    hash_node_id("insp_instance_apply_added_12"),
    hash_node_id("insp_instance_apply_added_13"),
    hash_node_id("insp_instance_apply_added_14"),
    hash_node_id("insp_instance_apply_added_15"),
];

/// A linha que este id representa — a leitura INVERSA da tabela, como as irmãs deste ficheiro.
#[must_use]
pub fn instance_apply_added(id: NodeId) -> Option<usize> {
    INSP_INSTANCE_APPLY_ADDED.iter().position(|c| *c == id)
}

/// ⭐ **Quantas FILEIRAS o cartão endereça.**
///
/// ⚠️ **É `1` desde 2026-09-01**, quando o mecanismo de propriedades foi adiado: a família oferece
/// UMA fileira, a das versões. A tabela continua a ser bidimensional porque o pintor e o censo de
/// ids a percorrem assim, e porque é ela que volta a crescer quando a feature for retomada.
pub const MAX_INSTANCE_AXES: usize = 1;

/// Quantos valores por eixo. ⚠️ Mesmo teto e mesma razão do irmão do vetor.
pub const MAX_INSTANCE_AXIS_VALUES: usize = 8;

/// ⭐⭐ **Os chips do cartão, um por `(eixo, valor)`.**
///
/// ⚠️ **Uma fileira por PERGUNTA**, e não uma lista de nomes: com `Size` e `State` a família tem
/// quatro versões e **duas** perguntas, e uma fileira plana obriga o artista a ler os nomes para
/// descobrir a estrutura. No modo plano a família devolve **um** eixo chamado `Variant`, que é
/// exactamente a fileira de antes — *duas modalidades, uma tabela*.
///
/// ⚠️ **Tabela `const`, e não um hash em tempo de execução** — é o idioma dos irmãos deste
/// directório, e é o que deixa o censo de ids do `hit_indexed_ids_are_registered` **ver** estes 32.
pub const INSP_INSTANCE_AXIS_OPTION: [[NodeId; MAX_INSTANCE_AXIS_VALUES]; MAX_INSTANCE_AXES] = [[
    hash_node_id("insp_instance_axis_0_0"),
    hash_node_id("insp_instance_axis_0_1"),
    hash_node_id("insp_instance_axis_0_2"),
    hash_node_id("insp_instance_axis_0_3"),
    hash_node_id("insp_instance_axis_0_4"),
    hash_node_id("insp_instance_axis_0_5"),
    hash_node_id("insp_instance_axis_0_6"),
    hash_node_id("insp_instance_axis_0_7"),
]];

/// O `(eixo, valor)` que este id representa — a leitura INVERSA da tabela.
///
/// ⚠️ **Uma porta, e não uma varredura em cada chamador**: o painel, o despachante e os gates fazem
/// a mesma pergunta, e a escada escrita três vezes é a doença que a coluna de catálogos pagou.
#[must_use]
pub fn instance_axis_option(id: NodeId) -> Option<(usize, usize)> {
    INSP_INSTANCE_AXIS_OPTION
        .iter()
        .enumerate()
        .find_map(|(a, row)| row.iter().position(|c| *c == id).map(|v| (a, v)))
}

/// ⭐⭐⭐ **Quantos DEGRAUS da escada do *Aplicar* o cartão endereça** (F5 critério 4).
///
/// ⚠️ **É um teto de TABELA DE IDS, e ele diz de que recurso é** — o mesmo do
/// [`MAX_INSTANCE_AXIS_VALUES`]: os ids são `const` para o censo do
/// `hit_indexed_ids_are_registered` os poder **ver**, e uma tabela `const` tem um tamanho.
///
/// ⛔ **Não é o limite de aninhamento do produto.** Uma cena com nove receitas encaixadas continua
/// a funcionar; o que acontece é que o 9.º degrau fica **fora do cartão**, e a fileira diz quantos
/// ficaram — ⛔ escrito, nunca truncado em silêncio. O *Aplicar ao mestre* do menu (o degrau mais
/// externo) alcança-se sempre, porque não passa por esta tabela.
pub const MAX_INSTANCE_APPLY_LEVELS: usize = 8;

/// Os botões da escada, um por degrau — do mais **externo** para o mais **interno**.
pub const INSP_INSTANCE_APPLY_LEVEL: [NodeId; MAX_INSTANCE_APPLY_LEVELS] = [
    hash_node_id("insp_instance_apply_level_0"),
    hash_node_id("insp_instance_apply_level_1"),
    hash_node_id("insp_instance_apply_level_2"),
    hash_node_id("insp_instance_apply_level_3"),
    hash_node_id("insp_instance_apply_level_4"),
    hash_node_id("insp_instance_apply_level_5"),
    hash_node_id("insp_instance_apply_level_6"),
    hash_node_id("insp_instance_apply_level_7"),
];

/// O degrau que este id representa — a leitura INVERSA da tabela.
///
/// ⚠️ **Uma porta, e não uma varredura em cada chamador** — o pintor, o despachante e os gates
/// fazem a mesma pergunta, e é o precedente do [`instance_axis_option`] logo acima.
#[must_use]
pub fn instance_apply_level(id: NodeId) -> Option<usize> {
    INSP_INSTANCE_APPLY_LEVEL.iter().position(|c| *c == id)
}
