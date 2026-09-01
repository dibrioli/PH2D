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

/// ⭐⭐⭐ **O CAMPO que renomeia o valor vigente**, aberto por um clique no chip aceso.
///
/// # ⛔⛔⛔ O clique no chip aceso não fazia NADA, e era o gesto que faltava
///
/// Report do Enio (2026-08-31, a quarta vez): ele escrevia `{Size=Big}` no nome da **cópia** para
/// dar nome ao valor, e o modelo ignorava-o — correctamente, porque uma propriedade é do
/// COMPONENTE. *O defeito é que autorar o valor obrigava a seleccionar OUTRO objecto do que aquele
/// que se está a olhar.*
///
/// ⭐ **O gesto não é novo: era um clique MORTO.** Carregar no valor já aceso era um no-op
/// silencioso («o artista carregou no botão que diz onde ele já está»), e é exactamente onde ele
/// aponta quando quer mudar o nome do valor. ⇒ um id só, porque só se edita um de cada vez.
pub const INSP_INSTANCE_VALUE_EDIT: NodeId = hash_node_id("insp_instance_value_edit");

/// ⭐⭐⭐ **SALVAR VARIAÇÃO** — o botão que aparece quando a cópia escolhida tem modificações.
///
/// Enio, 2026-09-01: *«Ao criar e modificar uma instância surge no card um botão do tipo "Salvar
/// Variação". Daí o fluxo acontece da forma mais inteligente possível, com o momento de colocar o
/// nome que vai gerar o botão seletor da variação.»*
///
/// ⚠️ **Só existe quando há o que gravar.** Sem modificação não há versão a criar, e um botão que
/// não faz nada é a espécie que a caça aos knobs mortos nomeia.
pub const INSP_INSTANCE_SAVE_VARIATION: NodeId = hash_node_id("insp_instance_save_variation");

/// ⭐⭐ **A propriedade escolhida no formulário** — as que a família já tem, mais *«Nova…»*.
///
/// ⚠️ **`MAX_INSTANCE_AXES + 1`**: o `+1` é a entrada *«Nova propriedade…»*, que é o que torna
/// criar a PRIMEIRA propriedade e criar a SEGUNDA o mesmo gesto — as duas precisam das mesmas três
/// respostas, e uma porta só é o que impede as duas de divergirem.
pub const INSP_INSTANCE_SAVE_PROP: [NodeId; MAX_INSTANCE_AXES + 1] = [
    hash_node_id("insp_instance_save_prop_0"),
    hash_node_id("insp_instance_save_prop_1"),
    hash_node_id("insp_instance_save_prop_2"),
    hash_node_id("insp_instance_save_prop_3"),
    hash_node_id("insp_instance_save_prop_new"),
];

/// O nome da propriedade NOVA (`Color`) — só existe com *«Nova…»* escolhida.
pub const INSP_INSTANCE_SAVE_NEW_PROP: NodeId = hash_node_id("insp_instance_save_new_prop");

/// ⭐⭐⭐ **Como se chama o que JÁ EXISTE** (`Normal`) — só com *«Nova…»* escolhida, e obrigatório.
///
/// ⚠️ Nascer uma propriedade põe TODA receita da família a declarar um valor nela. Sem esta
/// pergunta a fileira nova nasceria com um botão em branco — e uma fileira de um valor só nem
/// sequer é oferecida, então o artista veria o gesto não fazer nada.
pub const INSP_INSTANCE_SAVE_EXISTING: NodeId = hash_node_id("insp_instance_save_existing");

/// O nome desta versão (`Big`) — **é ele que vira o botão seletor**.
pub const INSP_INSTANCE_SAVE_VALUE: NodeId = hash_node_id("insp_instance_save_value");

/// Confirmar o formulário.
pub const INSP_INSTANCE_SAVE_CONFIRM: NodeId = hash_node_id("insp_instance_save_confirm");

/// Desistir. ⚠️ **Existe de propósito:** um formulário sem saída obriga o artista a gravar algo
/// para se ver livre dele.
pub const INSP_INSTANCE_SAVE_CANCEL: NodeId = hash_node_id("insp_instance_save_cancel");

/// ⭐⭐⭐ **Quantos EIXOS de propriedade o cartão endereça** — `Size`, `State`, …
///
/// ⚠️ Teto de **TABELA DE IDS**, como o irmão abaixo: uma família pode declarar os eixos que
/// quiser, e o excedente é **escrito** no cartão.
pub const MAX_INSTANCE_AXES: usize = 4;

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
pub const INSP_INSTANCE_AXIS_OPTION: [[NodeId; MAX_INSTANCE_AXIS_VALUES]; MAX_INSTANCE_AXES] = [
    [
        hash_node_id("insp_instance_axis_0_0"),
        hash_node_id("insp_instance_axis_0_1"),
        hash_node_id("insp_instance_axis_0_2"),
        hash_node_id("insp_instance_axis_0_3"),
        hash_node_id("insp_instance_axis_0_4"),
        hash_node_id("insp_instance_axis_0_5"),
        hash_node_id("insp_instance_axis_0_6"),
        hash_node_id("insp_instance_axis_0_7"),
    ],
    [
        hash_node_id("insp_instance_axis_1_0"),
        hash_node_id("insp_instance_axis_1_1"),
        hash_node_id("insp_instance_axis_1_2"),
        hash_node_id("insp_instance_axis_1_3"),
        hash_node_id("insp_instance_axis_1_4"),
        hash_node_id("insp_instance_axis_1_5"),
        hash_node_id("insp_instance_axis_1_6"),
        hash_node_id("insp_instance_axis_1_7"),
    ],
    [
        hash_node_id("insp_instance_axis_2_0"),
        hash_node_id("insp_instance_axis_2_1"),
        hash_node_id("insp_instance_axis_2_2"),
        hash_node_id("insp_instance_axis_2_3"),
        hash_node_id("insp_instance_axis_2_4"),
        hash_node_id("insp_instance_axis_2_5"),
        hash_node_id("insp_instance_axis_2_6"),
        hash_node_id("insp_instance_axis_2_7"),
    ],
    [
        hash_node_id("insp_instance_axis_3_0"),
        hash_node_id("insp_instance_axis_3_1"),
        hash_node_id("insp_instance_axis_3_2"),
        hash_node_id("insp_instance_axis_3_3"),
        hash_node_id("insp_instance_axis_3_4"),
        hash_node_id("insp_instance_axis_3_5"),
        hash_node_id("insp_instance_axis_3_6"),
        hash_node_id("insp_instance_axis_3_7"),
    ],
];

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
