//! **O ROTEADOR PREGUIÇOSO** (doc 89, folha 15) — quais entradas de um nó de selecção o cook
//! pode NÃO cozinhar, e as três condições que o autorizam.
//!
//! ⚠️ **FILHO do [`super`] e não um irmão**, pela mesma razão que o `cook_substep`: ele chama o
//! `cook_node` e lê os campos privados do [`Cook`]. O corte foi FORÇADO pelo tecto de LOC (700)
//! e a costura é por assunto — o pai fica com *como um nó se cozinha*, este com *o que se pode
//! deixar por cozinhar*.

use super::{Cook, CookError, CookValue, NodeId, OpResolver, ScopeKey, TimeFans, TimeScopes};
use crate::graph::Graph;
use std::collections::BTreeMap;

/// Quantos ramos um roteador preguiçoso pode ter — o tecto da máscara de [`LazySelect`].
///
/// ⚠️ **Um tecto e não um `Vec`**: a máscara é escrita no caminho quente de cada nó desses, uma
/// vez por cook, e uma alocação ali paga-se em toda cena que use o nó. `8` é o dobro do único
/// roteador que existe hoje (`value.switch`, quatro portas) — quem passar disto acrescenta ao
/// número, e o gate `a_lazy_router_declares_no_more_choices_than_the_mask_holds` di-lo em voz
/// alta em vez de truncar em silêncio.
pub const MAX_LAZY_CHOICES: usize = 8;

/// **UM ROTEADOR QUE PODE SALTAR AS ENTRADAS QUE NÃO ESCOLHEU** (doc 89, folha 15).
///
/// O Blender documenta a preguiça duas vezes (*"only the input that is passed through the node
/// is computed"*) e aqui o cook puxava as quatro. Medido (`measure_switch_laziness`, ramos de
/// oito oitavas sobre 4096 peças): a preguiça compra **3,90×** quando os ramos são caros **e**
/// exclusivos, e **1,03×** quando eles são o mesmo — porque nesse caso o memo já a entregou.
///
/// ## Ela é um MODO, e a razão não é conservadorismo — é uma medição
///
/// ⚠️ **Saltar um ramo muda o que o nó computa, no caso geral.** A contagem de saída do
/// `value.switch` é o **máximo** dos comprimentos de TODAS as entradas (gate
/// `the_output_count_is_decided_by_branches_nobody_chose`), então um ramo comprido que ninguém
/// escolheu ainda decide quantos elementos saem. E não há como saber isso sem cozinhar: o
/// `count_law` vive na maquinaria de GPU, que dimensiona *dispatches*; no caminho de CPU um
/// comprimento só existe depois da avaliação. ⇒ a preguiça **declara-se**, com o caminho de
/// omissão byte-idêntico, e nunca é uma optimização silenciosa do escalonador.
///
/// ## Três condições, e todas falham para o lado seguro
///
/// 1. **O `select` tem de ser UNIFORME.** Ele é uma PORTA e pode ser um campo por elemento
///    (feature documentada do nó) — e aí cada elemento escolhe um ramo diferente, ou seja
///    nenhum ramo é dispensável. O cook cozinha o `select` primeiro e só confia num valor único.
/// 2. **A sub-árvore saltada tem de ser inteiramente `Pure`.** Um ramo com `pre` ou
///    `Effect::Temporal`/`Stateful` **congela** se um tique não o cozinhar, e o artista que
///    voltasse a ele encontraria a simulação parada no passado. Quem verifica é o construtor do
///    plano (ver [`Self::skippable`]), que tem o grafo e os manifestos em mão.
/// 3. **A lei de *quais* ramos são precisos é do NÓ.** Ela viaja como ponteiro
///    ([`Self::needed`]) em vez de ser reimplementada aqui, porque o modo de mistura do
///    `value.switch` precisa de **dois** ramos e o valor do `select` decide se o par colapsa —
///    uma segunda cópia dessa lei divergiria no primeiro ajuste.
///
/// Falhar qualquer uma cozinha TUDO, que é o comportamento de sempre.
#[derive(Clone, Copy)]
pub struct LazySelect {
    /// A porta cujo valor decide.
    pub select_port: u16,
    /// A coluna escalar em que o valor do `select` viaja.
    pub select_column: &'static str,
    /// As portas candidatas, na ordem em que [`Self::needed`] as indexa.
    pub choices: &'static [u16],
    /// **A LEI de quais candidatas são precisas**, dada a selecção — a função do NÓ. Recebe a
    /// selecção uniforme e uma máscara do tamanho de [`Self::choices`], que preenche.
    pub needed: fn(f32, &mut [bool]),
    /// Quais candidatas o construtor do plano verificou serem SALTÁVEIS — sub-árvore inteiramente
    /// `Pure`, sem arestas `pre`. Indexada como [`Self::choices`].
    pub skippable: [bool; MAX_LAZY_CHOICES],
}

/// O plano de preguiça do quadro — quais nós são roteadores, e o que cada um pode saltar.
///
/// ⚠️ **Ele é ESTADO do [`Cook`] e não um argumento**, e a escolha é do mesmo tipo que a do
/// `set_external`: um argumento novo em `cook_scoped_fanned` custaria dezenas de sítios de
/// chamada, e um plano vazio (o default) é exactamente o comportamento de sempre. Quem o
/// constrói é a shell, que tem o registry e os params — o `ph2d-nodegraph` continua a não saber
/// o que é um `value.switch`.
pub type LazyBranches = BTreeMap<NodeId, LazySelect>;

impl Cook {
    /// **Declara quais nós podem saltar entradas neste quadro** (doc 89, folha 15) — ver
    /// [`LazySelect`].
    ///
    /// Chamado pela shell antes do cook do quadro, como o [`Self::set_external`]. Um plano vazio
    /// é o caminho de sempre, byte-idêntico: nenhum nó salta nada.
    ///
    /// ⚠️ **Substitui, não acumula.** Um nó que deixou de ser preguiçoso (o artista desligou o
    /// modo, ou o ramo passou a ter estado) tem de sair do plano no quadro em que isso acontece
    /// — acumular deixaria a preguiça viva sobre uma condição que já não se verifica, que é a
    /// forma deste desenho falhar em silêncio.
    pub fn set_lazy_branches(&mut self, plan: LazyBranches) {
        self.lazy = plan;
    }

    /// O plano de preguiça em vigor — para a shell e os gates o poderem ler de volta.
    #[must_use]
    pub fn lazy_branches(&self) -> &LazyBranches {
        &self.lazy
    }

    /// **A MÁSCARA DO QUADRO** — que candidatas deste nó ficam por cozinhar.
    ///
    /// ⚠️ **O `select` é cozido AQUI, antes das outras portas**, e é essa ordem que torna a
    /// decisão possível: quais ramos são precisos é função do VALOR dele. Cozinhá-lo duas vezes
    /// não custa — a segunda, dentro do laço de portas do pai, é um acerto de memo —, e é a do
    /// laço que empurra a revisão dele para a impressão digital, uma vez só.
    // ⚠️ **Oito argumentos, pelo mesmo motivo que o `cook_node`** — esta função é uma extensão
    // dele e recebe exactamente o contexto que ele tem em mão para o chamar de volta. Um struct
    // aqui seria um segundo empacotamento do mesmo conjunto, e teria de ser desfeito na linha
    // seguinte para a recursão o poder consumir.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn lazy_skip_mask(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        node: NodeId,
        in_playhead: f64,
        in_key: ScopeKey,
        scopes: &TimeScopes,
        fans: &TimeFans,
    ) -> Result<[bool; MAX_LAZY_CHOICES], CookError> {
        let mut skip = [false; MAX_LAZY_CHOICES];
        let Some(lazy) = self.lazy.get(&node).copied() else {
            return Ok(skip);
        };
        if lazy.choices.len() > MAX_LAZY_CHOICES {
            return Ok(skip);
        }
        // Sem aresta no `select`, o nó lê o campo VAZIO — que é `0.0` em todo índice, ou seja o
        // ramo 0. Isso é uniforme por construção, e é o caso comum de um roteador cuja escolha
        // não é animada.
        let selected = match graph.input_edge(node, lazy.select_port as usize) {
            Some((src, src_port, false)) => {
                self.cook_node(graph, ops, src, in_playhead, in_key, scopes, fans)?;
                uniform_scalar(&self.cur_output(src, in_key, src_port), lazy.select_column)
            }
            // Uma aresta `pre` no `select` lê o tique anterior; ela é uniforme ou não pelo mesmo
            // critério, e o valor está no instantâneo.
            Some((src, src_port, true)) => {
                uniform_scalar(&self.prev_output(src, src_port), lazy.select_column)
            }
            None => Some(0.0),
        };
        if let Some(s) = selected {
            let mut needed = [false; MAX_LAZY_CHOICES];
            (lazy.needed)(s, &mut needed[..lazy.choices.len()]);
            for k in 0..lazy.choices.len() {
                // Só salta o que a lei do nó dispensa **e** o construtor do plano declarou
                // saltável. As duas metades, sempre — uma sozinha é o defeito que congela uma
                // simulação num ramo que ninguém está a ver, e há prova de mutação a dizê-lo.
                skip[k] = !needed[k] && lazy.skippable[k];
            }
        }
        Ok(skip)
    }
}

/// **O valor do `select`, quando ele é o MESMO para todo elemento** — `None` quando não é.
///
/// ⚠️ **É a primeira das três condições da preguiça** (ver [`LazySelect`]): o `select` é uma
/// porta e pode ser um campo por elemento, e aí cada elemento escolhe um ramo diferente — ou
/// seja, nenhum ramo é dispensável. A régua é a do próprio nó: um campo **vazio** lê `0` em todo
/// índice e um campo de **um** valor difunde-o (a regra 1→N), então os dois são uniformes; um
/// campo mais longo só o é se todos os valores forem iguais.
///
/// ⚠️ **Compara por BITS e não por `==`**: dois `NaN` não são iguais, e um campo inteiro de
/// `NaN` é tão uniforme quanto um de zeros — recusá-lo mandaria cozinhar tudo por causa de uma
/// aritmética que já estava partida a montante.
fn uniform_scalar(v: &CookValue, column: &str) -> Option<f32> {
    let s = match v {
        CookValue::Instances(s) => s,
        CookValue::Empty => return Some(0.0),
        // ⚠️ Um valor OPACO é apagado pelo tipo — não há como ler um escalar dele, e adivinhar
        // seria a preguiça a decidir sobre uma coisa que ela não consegue ver. Cozinha tudo.
        CookValue::Opaque(_) => return None,
    };
    match s.get(column) {
        Some(crate::attr::Column::Scalar(xs)) => match xs.split_first() {
            None => Some(0.0),
            Some((first, rest)) => rest
                .iter()
                .all(|x| x.to_bits() == first.to_bits())
                .then_some(*first),
        },
        // Sem a coluna, o nó lê o campo vazio — `0.0` em todo índice.
        _ => Some(0.0),
    }
}
