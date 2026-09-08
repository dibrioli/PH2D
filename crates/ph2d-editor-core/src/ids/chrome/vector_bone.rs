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

/// ⭐⭐⭐ **Add Angle Limit** — dá a esta junta um arco de que ela não sai.
///
/// ⚠️ Só é pintado numa junta que ainda não tem limite — o par dele é o *Remove*, e os dois
/// excluem-se como o *Add IK* / *Remove IK*.
pub const VECTOR_BONE_LIMIT_ADD: NodeId = hash_node_id("vector.bone.limit.add");

/// **Remove Angle Limit** — a junta volta a girar livremente.
pub const VECTOR_BONE_LIMIT_REMOVE: NodeId = hash_node_id("vector.bone.limit.remove");

/// **Limit Min** — o extremo horário do arco, em GRAUS. ⚠️ O documento guarda radianos; a conversão
/// vive na shell, que é a porta onde as duas unidades se encontram.
pub const VECTOR_BONE_LIMIT_MIN: NodeId = hash_node_id("vector.bone.limit.min");

/// **Limit Max** — o extremo anti-horário, em graus. Ver [`VECTOR_BONE_LIMIT_MIN`].
pub const VECTOR_BONE_LIMIT_MAX: NodeId = hash_node_id("vector.bone.limit.max");

/// ⭐⭐⭐ **Add Smart Bone** — este osso ganha uma acção PRÓPRIA, com o nome dele.
///
/// ⚠️⚠️ **Ele CRIA a acção; ⛔ não adopta a que está aberta** — e a diferença foi um report do dono
/// (*«não há meios de selecionar nem o objeto alvo nem a animação»*, 2026-09-08). Um documento novo
/// nasce com **uma** acção chamada `"Main"`, então *adoptar a aberta* casava **todo** osso
/// inteligente com a animação principal da cena, em silêncio e sem nada na tela a dizê-lo — que é
/// exactamente o contrário do que uma acção é. É também a lei do Moho (*Create Smart Bone Action*
/// nasce com o nome do osso) e a do Blender (*Action Constraint* tem o botão **New**).
///
/// ⇒ quem escolhe outra acção é o [`VECTOR_BONE_SMART_CLIP`], que é onde *«qual animação?»* passou
/// a ter resposta na tela.
pub const VECTOR_BONE_SMART_ADD: NodeId = hash_node_id("vector.bone.smart.add");

/// **Remove Smart Bone** — o osso volta a ser um osso.
pub const VECTOR_BONE_SMART_REMOVE: NodeId = hash_node_id("vector.bone.smart.remove");

/// **Action From** — o ângulo (GRAUS) em que a acção está no princípio.
pub const VECTOR_BONE_SMART_FROM: NodeId = hash_node_id("vector.bone.smart.from");

/// **Action To** — ... e no fim. ⚠️ `To < From` percorre a acção ao contrário, e é legítimo.
pub const VECTOR_BONE_SMART_TO: NodeId = hash_node_id("vector.bone.smart.to");

/// ⭐⭐⭐ **Action** — QUAL acção este osso percorre. O chip que abre a lista das acções da timeline.
///
/// ⚠️ **É o READOUT e o gesto ao mesmo tempo** (o idioma da tecla de uma forma do Morph): o rótulo
/// do chip é o nome da acção ligada, então *«qual é?»* responde-se sem abrir nada. Um rótulo fixo
/// tipo *"Choose…"* obrigaria a abrir a lista para saber o que lá está — e foi precisamente a
/// AUSÊNCIA desta linha que fez o dono ler o osso inteligente como avariado.
///
/// ⚠️ **Registado como `Dropdown`, pintado como botão** — abrir/fechar é do dispatch genérico, e
/// registá-lo como `Button` faria o clique acender e nunca abrir lista nenhuma.
pub const VECTOR_BONE_SMART_CLIP: NodeId = hash_node_id("vector.bone.smart.clip");

/// ⭐ **Quantas acções o selector alcança** — o pool de ids é fixo porque o chrome **não cunha um
/// id em tempo de execução**.
///
/// ⚠️ **O recurso é o `ph2d_timeline::MAX_CLIPS`** (o tecto do próprio documento), e não um número
/// escolhido aqui: um pool menor esconderia acções que EXISTEM, e o artista veria uma lista que
/// mente. O gate `the_action_picker_reaches_every_clip_the_document_can_hold` mede a igualdade — e
/// vive na shell, que é a única que vê as duas crates.
pub const MAX_SMART_CLIPS: usize = 16;

/// As opções do selector de acção — uma por clip que o documento pode ter ([`MAX_SMART_CLIPS`]).
///
/// ⚠️ **Uma TABELA, não uma função de índice**, pelo mesmo motivo da [`VECTOR_BONE_BEND_IDS`]: o
/// `populate` que as regista, o `paint` que as desenha e o encaminhamento que as deixa passar
/// percorrem esta MESMA lista. Três listas escritas à mão é como um chip nasce morto sob o dedo.
pub const VECTOR_BONE_SMART_CLIP_IDS: [NodeId; MAX_SMART_CLIPS] = [
    hash_node_id("vector.bone.smart.clip_opt_0"),
    hash_node_id("vector.bone.smart.clip_opt_1"),
    hash_node_id("vector.bone.smart.clip_opt_2"),
    hash_node_id("vector.bone.smart.clip_opt_3"),
    hash_node_id("vector.bone.smart.clip_opt_4"),
    hash_node_id("vector.bone.smart.clip_opt_5"),
    hash_node_id("vector.bone.smart.clip_opt_6"),
    hash_node_id("vector.bone.smart.clip_opt_7"),
    hash_node_id("vector.bone.smart.clip_opt_8"),
    hash_node_id("vector.bone.smart.clip_opt_9"),
    hash_node_id("vector.bone.smart.clip_opt_10"),
    hash_node_id("vector.bone.smart.clip_opt_11"),
    hash_node_id("vector.bone.smart.clip_opt_12"),
    hash_node_id("vector.bone.smart.clip_opt_13"),
    hash_node_id("vector.bone.smart.clip_opt_14"),
    hash_node_id("vector.bone.smart.clip_opt_15"),
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
pub const VECTOR_BONE_VERBS: [NodeId; 9] = [
    VECTOR_BONE_BIND,
    VECTOR_BONE_EXPAND,
    VECTOR_BONE_RELEASE,
    VECTOR_BONE_IK_ADD,
    VECTOR_BONE_IK_REMOVE,
    VECTOR_BONE_LIMIT_ADD,
    VECTOR_BONE_LIMIT_REMOVE,
    VECTOR_BONE_SMART_ADD,
    VECTOR_BONE_SMART_REMOVE,
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
pub const VECTOR_BONE_FIELDS: [NodeId; 9] = [
    VECTOR_BONE_LENGTH,
    VECTOR_BONE_STRENGTH,
    VECTOR_BONE_IK_MIX,
    VECTOR_BONE_IK_SOFTNESS,
    VECTOR_BONE_IK_CHAIN,
    VECTOR_BONE_LIMIT_MIN,
    VECTOR_BONE_LIMIT_MAX,
    VECTOR_BONE_SMART_FROM,
    VECTOR_BONE_SMART_TO,
];
