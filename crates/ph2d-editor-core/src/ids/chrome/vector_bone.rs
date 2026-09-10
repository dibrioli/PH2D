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

/// ⭐⭐⭐ **O PAINEL do esqueleto** — o rect exterior dele.
///
/// ⛔⛔ **Ele era uma SECÇÃO do painel de vetor**, e saiu de lá por ordem do dono (2026-09-09)
/// depois de uma medição: com só um osso escolhido o cabeçalho dela caía em `y = 1394 px` sobre
/// uma faixa visível de `774`, com **785 px** de secções acima a falar de coisas que um osso não
/// tem — traço, preenchimento, mistura, morph.
///
/// ⚠️ **Os ids dos controlos NÃO foram renomeados.** Um `NodeId` é o hash de uma STRING, e
/// renomear a string quebra tudo o que a referencia por nome (este ficheiro abre a dizê-lo):
/// `vector.bone.*` continua a ser o endereço, e o que mudou é **quem os pinta**.
pub const SKELETON_PANEL: NodeId = hash_node_id("skeleton.panel");

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

/// **Criar** — arrastar faz um osso; carregar num osso apenas o selecciona (é assim que se escolhe
/// onde ramificar).
pub const VECTOR_BONE_ACT_CREATE: NodeId = hash_node_id("vector.bone.action.create");

/// **Transformar** — arrastar posa o que está sob o cursor (girar · deslocar · força · IK), e
/// ⛔ nunca cria.
pub const VECTOR_BONE_ACT_TRANSFORM: NodeId = hash_node_id("vector.bone.action.transform");

/// Os dois segmentos, **índice-alinhados** com [`ph2d_tool_vector::BoneAction::ALL`]. ⚠️ Alinhar
/// por índice é o que impede a lista do painel e a do vocabulário de divergirem em silêncio.
pub const VECTOR_BONE_ACTION_IDS: [NodeId; 2] = [VECTOR_BONE_ACT_CREATE, VECTOR_BONE_ACT_TRANSFORM];

/// **Fast** — um afim por triângulo da malha guardada. É o desenho de sempre, **byte-idêntico**.
pub const VECTOR_BONE_DEFORM_FAST: NodeId = hash_node_id("vector.bone.deform.fast");

/// ⭐⭐⭐ **Smooth** — a malha é refinada NO QUADRO até o desvio caber em meio pixel de ecrã.
///
/// ⚠️ Ele responde ao report de 2026-09-10 (*«ao dobrar a articulação temos arestas retas»*), e a
/// aresta reta é o erro de aproximar um campo curvo por um afim. Medido: `9,84 px → 0,41 px` numa
/// dobra de `150°`, pagando `216 → 3 456` triângulos.
pub const VECTOR_BONE_DEFORM_SMOOTH: NodeId = hash_node_id("vector.bone.deform.smooth");

/// Os dois segmentos, **índice-alinhados** com [`ph2d_tool_vector::SkinDeform::ALL`].
pub const VECTOR_BONE_DEFORM_IDS: [NodeId; 2] =
    [VECTOR_BONE_DEFORM_FAST, VECTOR_BONE_DEFORM_SMOOTH];

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

/// ⭐⭐⭐ **Add Smart Bone** — anexa o controlo, VAZIO. ⛔ Ele não cria coisa nenhuma.
///
/// ⚠️⚠️ **Dois desenhos caíram aqui no mesmo dia, cada um por um report do dono** (2026-09-08):
/// *adoptar o clip ABERTO* (*«não há meios de selecionar nem o objeto alvo nem a animação»*) casava
/// **todo** controlo com a animação principal da cena, porque um documento novo tem **uma** acção,
/// chamada `"Main"`; e *criar uma acção com o nome do osso* fabricava duas coisas por um clique
/// (*«porque criar Bone Action no inspector e na timeline? Melhor não criar nada»*).
///
/// ⇒ quem dá sujeito ao controlo são as duas linhas do painel: o [`VECTOR_BONE_SMART_PICK`] (o
/// objecto de que ele trata) e o [`VECTOR_BONE_SMART_CLIP`] (a animação, filtrada por aquele).
pub const VECTOR_BONE_SMART_ADD: NodeId = hash_node_id("vector.bone.smart.add");

/// ⭐⭐⭐ **Pick Object** — arma o gesto de duas mãos que diz de que objecto este controlo trata.
///
/// ⚠️ **O clique seguinte vale no CANVAS e na HIERARQUIA**, e é de graça: as duas superfícies
/// escrevem a MESMA selecção, então quem resolve o pick é *«a selecção mudou para outra coisa»* —
/// não um segundo caminho de acerto que divergiria do primeiro no dia em que um deles mudasse.
///
/// ⚠️ **Ele é o READOUT e o gesto**: o rótulo diz o nome do objecto escolhido. Um rótulo fixo
/// obrigaria a abrir outra coisa para saber o que lá está.
pub const VECTOR_BONE_SMART_PICK: NodeId = hash_node_id("vector.bone.smart.pick");

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
pub const VECTOR_BONE_VERBS: [NodeId; 10] = [
    VECTOR_BONE_BIND,
    VECTOR_BONE_EXPAND,
    VECTOR_BONE_RELEASE,
    VECTOR_BONE_IK_ADD,
    VECTOR_BONE_IK_REMOVE,
    VECTOR_BONE_LIMIT_ADD,
    VECTOR_BONE_LIMIT_REMOVE,
    VECTOR_BONE_SMART_ADD,
    VECTOR_BONE_SMART_REMOVE,
    VECTOR_BONE_SMART_PICK,
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

/// ⭐⭐⭐ **OS TRÊS CONTROLOS DESTA SEÇÃO CUJO SUJEITO É A SELECÇÃO DE FORMAS**, e não um osso.
///
/// ⚠️ Eles são a **excepção declarada** de [`needs_focused_bone`]: o *Bind* prende as formas
/// escolhidas ao esqueleto, e o par *Keep Pose* / *Release* solta-as. Nenhum dos três pergunta qual
/// osso está aceso.
pub const VECTOR_BONE_ON_SELECTION: [NodeId; 3] =
    [VECTOR_BONE_BIND, VECTOR_BONE_EXPAND, VECTOR_BONE_RELEASE];

/// ⭐⭐⭐ **O SUJEITO DESTE CONTROLO É O OSSO EM FOCO?** — a pergunta que decide se o dreno tem com
/// que trabalhar, e se o silêncio dele precisa de ser explicado.
///
/// ⛔⛔ **Ela é DERIVADA das tabelas, e a derivação é a cura.** O braço da shell que diz *«nenhum
/// osso em foco»* era uma disjunção escrita à mão: nasceu com **dois** verbos (os da âncora),
/// ficaram **oito** quando a auditoria de 2026-09-08 a apanhou, e os campos e as duas fileiras de
/// chips nunca lá entraram. *Uma cura escrita para os verbos que existiam não segue os que vêm* — e
/// o sintoma de um verbo que morre calado é indistinguível de uma rota cortada, que é o report que
/// esta seção já pagou quatro vezes.
///
/// ⇒ acrescentar um id a qualquer das quatro tabelas põe-no **automaticamente** do lado certo; quem
/// age sobre as FORMAS declara-o em [`VECTOR_BONE_ON_SELECTION`], e é a única lista à mão que resta.
#[must_use]
pub fn needs_focused_bone(id: NodeId) -> bool {
    if VECTOR_BONE_ON_SELECTION.contains(&id) {
        return false;
    }
    VECTOR_BONE_VERBS.contains(&id)
        || VECTOR_BONE_FIELDS.contains(&id)
        || VECTOR_BONE_BEND_IDS.contains(&id)
        || VECTOR_BONE_SMART_CLIP_IDS.contains(&id)
}
