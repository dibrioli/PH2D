//! **Os ids do ESQUELETO** (estudo 42 item 5, doc 47) — módulo irmão de [`super`] pelo teto de 700
//! LOC, com o corte por RESPONSABILIDADE: aqui vive a família que faz um desenho **dobrar** — o
//! modo que autora ossos e a seção que os liga à forma.
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING, então reordenar não quebra nada — mas
//! renomear uma string quebra tudo o que a referencia por nome, e é assim que um widget fica órfão
//! em silêncio.

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

/// ⭐⭐⭐ **Osso** — o 17.º modo. Arrastar no vazio faz um osso; o pai é o osso seleccionado, então
/// arrasto-arrasto-arrasto é uma cadeia.
///
/// ⚠️ Ele fica no FIM da fileira, ao lado da Moldura, e a vizinhança diz o porquê: os dois são os
/// únicos modos que produzem algo que **não é uma forma** — aquele um lugar onde as formas moram,
/// este algo que as **move**.
pub const VECTOR_MODE_BONE: NodeId = hash_node_id("vector.mode.bone");

/// O cabeçalho da seção **SKELETON**. ⚠️ Tem de entrar em [`super::VECTOR_SECTIONS`], senão o
/// chevron pinta, clica e **não dobra** (o `dispatch` consulta aquela lista antes de disparar o
/// toggle) — dívida que o Text on Path e o Pattern on Path já pagaram.
pub const VECTOR_SECTION_BONE: NodeId = hash_node_id("vector.section.bone");

/// **Bind** — prende as formas seleccionadas ao esqueleto. Não move um pixel (a pose de repouso é a
/// identidade por construção), e é o gesto que separa um desenho de um personagem.
pub const VECTOR_BONE_BIND: NodeId = hash_node_id("vector.bone.bind");

/// **Keep Pose** — solta as formas e fica com a geometria deformada de AGORA (o *Expand* do
/// envelope). Par do de baixo: adivinhar qual dos dois o artista quer é que não.
pub const VECTOR_BONE_EXPAND: NodeId = hash_node_id("vector.bone.expand");

/// **Release** — solta as formas e devolve o que o artista DESENHOU.
pub const VECTOR_BONE_RELEASE: NodeId = hash_node_id("vector.bone.release");

/// **Length** — o comprimento do osso seleccionado, em unidades locais dele.
pub const VECTOR_BONE_LENGTH: NodeId = hash_node_id("vector.bone.length");

/// **Strength** — o raio de influência, em **comprimentos deste osso** (o *Bone Strength* do Moho).
///
/// ⚠️ Múltiplo e não distância, de propósito: é o que torna a lei adimensional, e o mesmo rig
/// desenhado dez vezes maior deforma-se igual.
pub const VECTOR_BONE_STRENGTH: NodeId = hash_node_id("vector.bone.strength");

/// ⭐⭐⭐ **O grupo alternável CRIAR × TRANSFORMAR** (Enio, 2026-09-07: *«do modo como está fica
/// confuso para o usuário»*) — o cabeçalho do par.
pub const VECTOR_BONE_ACTION: NodeId = hash_node_id("vector.bone.action");

/// **Criar** — arrastar faz um osso; carregar num osso apenas o selecciona (é assim que se escolhe
/// onde ramificar).
pub const VECTOR_BONE_ACT_CREATE: NodeId = hash_node_id("vector.bone.action.create");

/// **Transformar** — arrastar posa o que está sob o cursor (girar · deslocar · força · IK), e
/// ⛔ nunca cria.
pub const VECTOR_BONE_ACT_TRANSFORM: NodeId = hash_node_id("vector.bone.action.transform");

/// Os dois segmentos, **índice-alinhados** com [`ph2d_tool_vector::BoneAction::ALL`]. ⚠️ Alinhar
/// por índice é o que impede a lista do painel e a do vocabulário de divergirem em silêncio.
pub const VECTOR_BONE_ACTION_IDS: [NodeId; 2] = [VECTOR_BONE_ACT_CREATE, VECTOR_BONE_ACT_TRANSFORM];

/// ⭐⭐⭐ **Add IK** — dá a este osso uma ÂNCORA: um alvo que a corrente persegue a cada quadro.
///
/// ⚠️ Só é pintado num osso que ainda **não** tem uma — duas âncoras a puxar a mesma corrente é
/// estado inalcançável, e o gesto recusa-o também.
pub const VECTOR_BONE_IK_ADD: NodeId = hash_node_id("vector.bone.ik.add");

/// **Remove IK** — apaga a âncora e o alvo com ela.
pub const VECTOR_BONE_IK_REMOVE: NodeId = hash_node_id("vector.bone.ik.remove");

/// **Mix** — quanto da restrição vale (`0..1`). O *Mix* do Spine, o *Influence* do Blender.
pub const VECTOR_BONE_IK_MIX: NodeId = hash_node_id("vector.bone.ik.mix");

/// **Softness** — a que fracção do alcance a corrente começa a abrandar, para o joelho não estalar
/// ao esticar. O *Softness* do Spine.
pub const VECTOR_BONE_IK_SOFTNESS: NodeId = hash_node_id("vector.bone.ik.softness");

/// **Chain** — quantos ossos a âncora governa, contados da ponta para cima. `0` = até à raiz.
pub const VECTOR_BONE_IK_CHAIN: NodeId = hash_node_id("vector.bone.ik.chain");

/// ⭐⭐⭐ **O cabeçalho de PARA QUE LADO O JOELHO DOBRA** — o `flip_bend_direction` do Godot, o
/// `bendDirection` do Spine.
pub const VECTOR_BONE_IK_BEND: NodeId = hash_node_id("vector.bone.ik.bend");

/// **Auto** — o lado sai da pose que a corrente tem. ⚠️ É o comportamento de sempre, e é ele que
/// **inverte** o joelho quando o membro passa pela posição esticada: medido, uma corrente a
/// `−99,498744` volta do mesmo alvo a `+99,498744`.
pub const VECTOR_BONE_IK_BEND_AUTO: NodeId = hash_node_id("vector.bone.ik.bend.auto");

/// **CCW** — o joelho fica travado do lado anti-horário da recta raiz→alvo.
pub const VECTOR_BONE_IK_BEND_CCW: NodeId = hash_node_id("vector.bone.ik.bend.ccw");

/// **CW** — ... do lado horário.
pub const VECTOR_BONE_IK_BEND_CW: NodeId = hash_node_id("vector.bone.ik.bend.cw");

/// Os três segmentos, **índice-alinhados** com [`ph2d_skeleton::BendSide::ALL`] — o mesmo idioma do
/// [`VECTOR_BONE_ACTION_IDS`], e pela mesma razão: alinhar por índice é o que impede a fileira do
/// painel e o vocabulário da lei de divergirem em silêncio.
///
/// ⭐⭐ **Ela é uma tabela de pleno direito, com os MESMOS TRÊS consumidores das outras duas** — o
/// registo (`populate_bone`), o encaminhamento (`event_clicks::is_shell_click`) e a pintura. ⚠️ A
/// 1.ª redacção economizou-a, pendurando os três ids na [`VECTOR_BONE_VERBS`] para herdar o
/// encaminhamento — e o gate `table_driven_chips_are_registered_too` reprovou, **com razão**: ele
/// exige que o `populate` tenha um laço sobre a tabela que o `paint` itera, e sem esse laço a
/// próxima fileira que alguém acrescente nasce **morta sob o dedo**. *Herdar a rota de outra tabela
/// funciona e apaga a regra que protege quem vier a seguir.*
pub const VECTOR_BONE_BEND_IDS: [NodeId; 3] = [
    VECTOR_BONE_IK_BEND_AUTO,
    VECTOR_BONE_IK_BEND_CCW,
    VECTOR_BONE_IK_BEND_CW,
];

/// ⭐⭐⭐ **OS VERBOS DA SEÇÃO SKELETON — uma tabela, dois consumidores.**
///
/// Todo botão desta seção mexe no **MUNDO** (um componente de uma entidade), logo o clique dele é
/// da SHELL: o painel tem de o **encaminhar** ao barramento em vez de o consumir. Quem regista os
/// widgets ([`ph2d_panel_vector`]'s `populate_bone`) e quem decide o que atravessa
/// (`event_clicks::is_shell_click`) lêem **esta** lista.
///
/// ⛔⛔ **Ela existe porque a mesma rota morreu QUATRO vezes nesta linha.** Eram duas listas
/// escritas à mão, e o modo de falha é o pior que há: o botão **pinta**, **acende sob o rato** e o
/// clique **morre dentro do painel** — indistinguível, do lado de fora, de um verbo que recusou.
/// Foi o bug #29 (três rotas de uma vez, com o gate de registo VERDE), e voltou em 2026-09-07 com
/// o *Add IK* (report do dono: *«Add IK não funciona»*), acrescentado à seção sem vir aqui.
///
/// ⇒ *Uma lista escrita à mão ao lado de outra é duas respostas à mesma pergunta, e a que o artista
/// vê é a que envelhece.* Com uma tabela só, acrescentar um verbo liga-o nos dois sítios.
pub const VECTOR_BONE_VERBS: [NodeId; 5] = [
    VECTOR_BONE_BIND,
    VECTOR_BONE_EXPAND,
    VECTOR_BONE_RELEASE,
    VECTOR_BONE_IK_ADD,
    VECTOR_BONE_IK_REMOVE,
];

/// ⭐⭐⭐ **OS CAMPOS NUMÉRICOS DA SEÇÃO SKELETON — a mesma tabela, os mesmos dois consumidores.**
///
/// Todos moram num componente de uma entidade, logo o VALOR é da **shell**: o painel tem de o
/// encaminhar (`event::is_shell_number_field`) em vez de o guardar. Fora da lista, o campo aceita
/// teclas e **não fala com ninguém** — a forma mais cara de um controlo nascer morto, porque parece
/// vivo.
///
/// ⚠️ Ela existe pela mesma razão da [`VECTOR_BONE_VERBS`], e o custo já foi pago: o Z-index
/// pagou-o uma vez, e o *Add IK* pagou-o outra na família ao lado.
pub const VECTOR_BONE_FIELDS: [NodeId; 5] = [
    VECTOR_BONE_LENGTH,
    VECTOR_BONE_STRENGTH,
    VECTOR_BONE_IK_MIX,
    VECTOR_BONE_IK_SOFTNESS,
    VECTOR_BONE_IK_CHAIN,
];
