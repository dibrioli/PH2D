//! **A POSE** — tudo o que um objeto pode ter de diferente entre dois estados, e nada mais.

use crate::role::StateRole;
use ph2d_fx_op::FxOp;
use ph2d_stroke_width::WidthStops;
use ph2d_vec_scene::{Paint, StrokeSpec, VecPath, VecPathId};
use serde::{Deserialize, Serialize};

/// Onde um objeto está, como ele se parece, e que forma ele tem — num estado.
///
/// ⚠️ **O transform é DECOMPOSTO em T/R/S, e não é uma matriz.** Interpolar matrizes é o erro
/// clássico: o caminho entre duas rotações passa por dentro (a forma **encolhe** no meio) e uma
/// escala negativa vira cisalhamento. Aqui a representação **apaga o caso especial** — não existe
/// matriz para alguém lerpar por engano.
///
/// ⚠️ **A TINTA é campo de primeira classe, e não vive dentro da geometria.** Foi a autoria que
/// expôs isto: um botão que só muda de cor no hover tem a MESMA forma nos dois estados, e uma
/// tinta que morasse lá dentro **não teria por onde viajar** — o par de formas idênticas nem
/// entra no casamento. São dois fatos independentes sobre um objeto, e um estado precisa de
/// poder autorar um sem o outro.
///
/// ⚠️ **A geometria é a FONTE autorada, e a autoria grava-a sempre.** Ela chegou a ser opcional
/// *"porque a forma raramente muda"* — e o preço dessa frase foi que uma edição de nó, um Fillet
/// ou um Chamfer **não animavam**: o campo existia, o motor sabia casá-lo, e ninguém o
/// preenchia. `None` sobrou para o caso honesto de não haver forma nenhuma a que se agarrar (o
/// objeto sumiu do documento).
///
/// ⚠️ **E o que poupa os 0,64 ms por objeto (ver o doc da crate) é a IGUALDADE, não a ausência:**
/// duas formas idênticas não constroem `Plan` nenhum. O par só-de-cor continua a custar zero.
///
/// ⚠️ **Fonte, nunca cozido.** É a fonte que o modo Node edita e é ela que a chegada devolve ao
/// documento; guardar a cozida assaria o raio de quina e a pilha de efeitos, e o artista
/// perderia as alças no primeiro Show. Quem coze é o Blend, para o CAMINHO — a costura
/// fonte≠cozido do ADR-0121, no nível do estado.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectPose {
    /// **A identidade, e a única chave de casamento.** Nunca o nome, nunca o índice.
    pub id: VecPathId,
    pub translation: [f64; 2],
    /// Radianos. Interpolada pelo **arco mais curto** — ver [`super::Transition::at`].
    pub rotation: f64,
    pub scale: [f64; 2],
    /// `0` = invisível, `1` = opaco. É o canal que carrega o fade de quem entra e de quem sai.
    pub opacity: f32,
    /// O preenchimento neste estado. `None` = sem preenchimento (não *"herda"*: um estado que
    /// autora um objeto autora a tinta dele).
    pub fill: Option<Paint>,
    /// O traço neste estado.
    pub stroke: Option<StrokeSpec>,
    /// A forma, quando este estado a autora. `None` ⇒ a cena manda, e nenhum `Plan` é construído.
    pub geometry: Option<VecPath>,
    /// **O perfil de largura viva** (ADR-0148), quando o traço tem um. `None` = largura uniforme.
    ///
    /// ⚠️ **É o único canal de forma que não mora no `VecPath`** — ele é um componente ECS —, e
    /// é por isso que tem campo próprio aqui em vez de viajar dentro da `geometry`. Sem ele o
    /// Width Tool seria a única ferramenta de desenho cuja edição não animaria entre dois
    /// estados, e a ausência não teria nome nenhum.
    pub width: Option<WidthStops>,
    /// **A pilha de FX raster** (blur, glow, sombra, ajuste de cor) neste estado. Vazia = sem
    /// filtro.
    ///
    /// ⚠️ **Pela mesma razão do [`Self::width`]**: é um canal de APARÊNCIA que não vive no
    /// `VecPath` — ele é um componente ECS (`ph2d_ecs::VecFilter`) —, então a pose tem de o
    /// carregar por si. Sem ele, um blur seria o único efeito do editor incapaz de diferir entre
    /// *Default* e *Hover*, e a seção Filters do painel ficaria muda dentro de um botão animado.
    ///
    /// ⚠️ **E o vazio NÃO é "herda"**, é *sem filtro* — a mesma lei do `fill`. Quando um dos
    /// lados não conhece um degrau que o outro tem, quem resolve é o alinhamento
    /// (`ph2d_fx_op::mix_stacks`), que o faz crescer do neutro em vez de o omitir.
    pub filters: Vec<FxOp>,
    /// **O verbo PRÓPRIO desta forma dentro da booleana viva que a consome** — o código de
    /// `PathfinderOp` com que ela dobra sobre o resultado das anteriores. `None` = ela não se
    /// pronuncia, e **herda o do grupo**.
    ///
    /// ⚠️ **`None` significa aqui exatamente o que significa no componente** (`ph2d_ecs::VecBoolOp`),
    /// e essa coincidência é o desenho: um vocabulário próprio nesta ponta obrigaria alguém a
    /// manter duas leis de herança em dia. É o oposto do [`Self::fill`], cujo `None` teve de ganhar
    /// um doc a dizer que *não* é "herda".
    ///
    /// ⚠️ **Terceiro membro da família do [`Self::width`] e do [`Self::filters`]**, e pela mesma
    /// razão que os dois existem: é um canal que NÃO vive no `VecPath` — é um componente ECS —,
    /// então a pose tem de o carregar por si.
    pub bool_op: Option<u8>,
    /// **O verbo do GRUPO booleano acima desta forma**, se houver algum. `None` = ela não é
    /// operando de booleana viva nenhuma neste estado, e então nada há a escrever.
    ///
    /// ⚠️ **São dois fatos diferentes, e por isso dois campos.** O de cima é *"que verbo ESTA
    /// forma manda"*; este é *"em que operação ela está metida"* — e o segundo é o que faz a
    /// receita inteira do grupo (as quatro de conjunto **e** as quatro receitas) mudar entre dois
    /// estados. Um campo só teria de escolher qual dos dois carregar, e a escolha calada é como
    /// um `Trim` autorado no Hover não anima nada.
    ///
    /// ⚠️ **Ele repete-se em cada operando do mesmo grupo, e a redundância é deliberada:** o grupo
    /// é uma entidade **sem `VecPathId`** e a pose é chaveada por caminho, então ele não tem slot
    /// próprio. Gravá-lo em quem o grupo governa é a única chave que já existe — e como a captura
    /// lê os operandos todos do mesmo sítio, os valores não podem divergir.
    ///
    /// ⚠️ **Ausência NUNCA desfaz o grupo.** `None` é *"não sei de grupo nenhum"*, e a escrita
    /// simplesmente não acontece; interpretá-lo como *"remova o `VecBoolGroup`"* faria uma pose
    /// gravada antes da booleana **destruir** a booleana no primeiro Show.
    pub bool_group_op: Option<u8>,
    /// ⭐⭐⭐ **EM QUE FORMA o conjunto de estados do Morph está, nesta pose** (plano 32 W11c).
    ///
    /// `None` = este objecto não é um conjunto de estados, ou a pose não se pronuncia sobre ele.
    ///
    /// # A pergunta que ele responde
    ///
    /// Enio, 2026-08-26: *"Assegure-se que esse sistema de states em morph seja integrado e
    /// completamente compatível com o sistema de States previamente existente, ou seja, que eu
    /// possa usar o state morph nas animações criadas em States."*
    ///
    /// ⇒ um botão que veste um conjunto de Morph States pode **mudar de forma no `Hover`**: a pose
    /// grava *que forma*, e a transição interpola até lá como interpola tudo o resto.
    ///
    /// ⚠️ **QUARTO membro da família do [`Self::width`], [`Self::filters`] e [`Self::bool_op`]**, e
    /// pela razão que os três já documentam: é um canal que **não vive no `VecPath`** — ele é
    /// estado de um componente ECS —, então a pose tem de o carregar por si. Sem ele, um conjunto
    /// de estados seria o único objecto do editor incapaz de diferir entre *Default* e *Hover*.
    ///
    /// ⚠️ **É a FORMA (`VecPathId`), nunca o índice na lista.** A lista é derivada dos filhos
    /// (W11a) e muda quando o artista arrasta um para dentro ou para fora — um índice guardado
    /// passaria a apontar para outra forma **sem que nada mudasse na pose**. *Um índice guardado é
    /// uma afirmação sobre uma lista que pode ter mudado.*
    ///
    /// ⚠️ **`None` aqui significa «não me pronuncio», e não «volta ao início»** — a mesma leitura
    /// do [`Self::bool_op`], e o oposto do [`Self::fill`] (cujo `None` teve de ganhar um doc a
    /// dizer que *não* é «herda»). Uma pose gravada sobre um objecto que ainda não era um conjunto
    /// não pode passar a mandá-lo para a primeira forma no dia em que ele virar um.
    pub morph_shape: Option<VecPathId>,
}

impl ObjectPose {
    /// Uma pose neutra: na origem, sem giro, escala 1, opaca, sem forma própria.
    #[must_use]
    pub fn new(id: VecPathId) -> Self {
        Self {
            id,
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            opacity: 1.0,
            fill: None,
            stroke: None,
            geometry: None,
            width: None,
            filters: Vec::new(),
            bool_op: None,
            bool_group_op: None,
            morph_shape: None,
        }
    }

    /// **Esta pose e a outra descrevem a mesma coisa?**
    ///
    /// ⚠️ É esta pergunta — e não um limiar — que decide se o par ENTRA na transição. Um objeto
    /// que não muda entre dois estados não é interpolado, não constrói `Plan` e não aparece na
    /// conta: *não animar* é mais barato e mais correto que *animar de x para x*.
    #[must_use]
    pub fn is_same_as(&self, other: &Self) -> bool {
        self == other
    }
}

/// Um ESTADO: o papel que ele desempenha, e a pose de cada objeto que ele autora.
///
/// ⚠️ **O papel é a CHAVE do estado; o id é a chave do OBJETO.** São duas perguntas diferentes e
/// cada uma tem uma resposta só — *que estado é este?* responde-se com o
/// [`StateRole`], *quem é quem entre dois estados?* com o [`ObjectPose::id`]. Nenhuma das duas é
/// um nome livre, e é isso que faz uma animação sobreviver a renomear e a reordenar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    pub role: StateRole,
    pub objects: Vec<ObjectPose>,
}

impl UiState {
    #[must_use]
    pub fn new(role: StateRole) -> Self {
        Self {
            role,
            objects: Vec::new(),
        }
    }

    /// A pose de `id` neste estado, se ele a autora.
    ///
    /// ⚠️ Busca LINEAR de propósito: um estado de UI tem dezenas de objetos, não milhares, e um
    /// mapa aqui custaria uma ordem de iteração que o `serde` teria de preservar para o arquivo
    /// ser determinista. Se um dia a conta crescer, o índice nasce em [`super::Transition::new`],
    /// que roda **uma vez por par** — nunca por frame.
    #[must_use]
    pub fn pose(&self, id: VecPathId) -> Option<&ObjectPose> {
        self.objects.iter().find(|p| p.id == id)
    }
}
