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
