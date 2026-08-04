//! **Os ids dos COMPONENTES** (plano UI/UX W5) — irmão de [`super::vector_anchors`] pelo teto de
//! LOC, e o corte é o assunto: aqui mora o prefab (mestre, instância, override).
//!
//! # Quatro verbos, e cada um aparece onde faz sentido
//!
//! *Create Component* só com uma forma comum selecionada · *Place Instance* só com um mestre ·
//! *Detach* e *Reset Overrides* só com uma instância. ⚠️ A alternativa — quatro botões sempre
//! pintados, três deles inertes — é o botão-morto que este repo persegue: um botão que não faz
//! nada é pior que um botão que falta, porque ensina o artista a duvidar dos outros.
//!
//! # A W5b acrescentou a metade que faltava: as DIFERENÇAS
//!
//! A W5a shipou o modelo do override (`OverrideSlot::Fill` / `Hidden`) com gates — e **nenhuma
//! porta que o produzisse**. O *Reset Overrides* era, literalmente, um botão que nunca podia ser
//! preciso: a única coisa capaz de criar um override era um teste. É a forma exata de
//! [[feedback_a_capability_without_a_door_passes_every_gate]], e a W5b é a porta.
//!
//! A porta é uma **LISTA de peças** — uma linha por peça do mestre, com um interruptor de
//! visibilidade e uma swatch de cor. ⚠️ E a lista é a **sub-árvore INTEIRA** do mestre, nunca as
//! peças visíveis: esconder uma peça tirar-lhe-ia a própria linha, e o gesto não teria volta.
//!
//! *Update Main* e *Swap* fecham o resto da lista do plano.

use ph2d_a11y::NodeId;

use super::painter::fnv_node_id_runtime;
use crate::ids::hash_node_id;

/// O cabeçalho da seção **Component**.
pub const VECTOR_SECTION_COMPONENT: NodeId = hash_node_id("vector.section.component");

/// **Create Component** — a forma selecionada (e a sub-árvore dela) vira um mestre.
pub const VECTOR_COMPONENT_CREATE: NodeId = hash_node_id("vector.component.create");
/// **Place Instance** — põe uma cópia derivada do mestre selecionado.
pub const VECTOR_COMPONENT_PLACE: NodeId = hash_node_id("vector.component.place");
/// **Detach** — a instância deixa de derivar: o que estava na tela vira geometria dela.
pub const VECTOR_COMPONENT_DETACH: NodeId = hash_node_id("vector.component.detach");
/// **Reset Overrides** — a instância volta a ser exactamente o mestre.
pub const VECTOR_COMPONENT_RESET: NodeId = hash_node_id("vector.component.reset");

/// **Update Main** — as diferenças desta instância passam a ser o MESTRE, e as irmãs herdam.
///
/// ⚠️ Ele **absorve o que o mestre sabe guardar**, que hoje é a COR. O `Hidden` não sobe: um
/// mestre não tem *"peça escondida"*, e a única maneira de a esconder lá seria apagá-la — e apagar
/// arte não é o que *"atualizar o mestre"* significa para ninguém. A recusa é por espécie, dentro
/// da porta, e não uma condição no botão: um botão que some quando a instância tem um `Hidden`
/// esconderia também as cores que ele PODE absorver.
pub const VECTOR_COMPONENT_UPDATE_MAIN: NodeId = hash_node_id("vector.component.update_main");

/// **Swap Main** — arma o conta-gotas: o próximo clique num MESTRE religa esta instância a ele.
///
/// O idioma de pick modal desta linha (o *Pick Path* do pattern/texto, o conta-gotas de corpo do
/// joint): arma, e o clique seguinte no canvas resolve. Um dropdown de componentes seria a segunda
/// resposta a *"qual é o mestre?"* — e teria de listar por NOME, que é justamente o endereço que a
/// W5a recusou.
pub const VECTOR_COMPONENT_SWAP: NodeId = hash_node_id("vector.component.swap");

/// Quantas peças do mestre a lista endereça.
///
/// ⚠️ **Isto é um teto de TABELA DE IDS, e ele diz de que recurso é:** o `populate` regista os
/// ids por-linha num laço e o roteador varre o mesmo intervalo para resolver um clique — as duas
/// pontas precisam de um número finito. Não é um teto do MESTRE: um componente pode ter as peças
/// que quiser, e as que passam daqui continuam a desenhar, a herdar e a ser editáveis **no
/// mestre**.
///
/// ⚠️ E o excedente **não é silencioso** — o painel escreve quantas peças ficaram de fora (a lei
/// dos caps que não mentem). O número é o mesmo, e pela mesma razão, que o `MAX_CONTAINERS` da
/// timeline: uma tabela fixa varrida por clique.
pub const MAX_INSTANCE_PIECES: usize = 16;

/// O interruptor de visibilidade da peça `row` **nesta instância** (`OverrideSlot::Hidden`).
#[must_use]
pub fn vector_instance_piece_show_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.instance.piece.show.{row}"))
}

/// Quantos EIXOS de variant a seção endereça (plano UI/UX W5c).
///
/// ⚠️ **Teto de TABELA DE IDS, e ele diz de que recurso é** — o mesmo que o
/// [`MAX_INSTANCE_PIECES`]: o `populate` regista `AXES × VALUES` chips num laço e o roteador varre
/// o mesmo intervalo. Não é teto do catálogo: um conjunto pode ter as versões que quiser, e as que
/// passam daqui continuam a existir, a desenhar, e a ser alcançáveis pelo **Swap Main** — que é um
/// conta-gotas e não tem lista nenhuma.
///
/// ⚠️ E o excedente **é escrito** no painel, nunca truncado em silêncio.
pub const MAX_VARIANT_AXES: usize = 4;

/// Quantos VALORES por eixo a seção endereça (plano UI/UX W5c).
pub const MAX_VARIANT_VALUES: usize = 8;

/// O chip do valor `value` do eixo `axis` da instância selecionada.
///
/// ⚠️ **Um chip por VALOR, e não um dropdown**, porque um eixo de variant tem tipicamente dois a
/// quatro valores — e a fileira segmentada mostra-os todos ao mesmo tempo, que é o que deixa o
/// artista ver o catálogo em vez de o abrir. É o widget que este painel já usa para *"qual
/// destes?"* com poucos candidatos, e ele quebra em linhas sozinho quando não cabem.
#[must_use]
pub fn vector_variant_option_id(axis: usize, value: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.variant.{axis}.{value}"))
}

/// A swatch de cor da peça `row` **nesta instância** (`OverrideSlot::Fill`).
///
/// ⚠️ Ela é alvo de PICKER (`register_picker_swatch`), como a swatch de Fill da forma: o clique
/// ABRE o picker e a cor é lida do alvo dele no frame seguinte. Registá-la como botão faria o
/// picker nunca abrir.
#[must_use]
pub fn vector_instance_piece_colour_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.instance.piece.colour.{row}"))
}
