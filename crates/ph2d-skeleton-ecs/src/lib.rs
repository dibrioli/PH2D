#![forbid(unsafe_code)]
//! **O ESQUELETO NA CENA** (estudo 42 item 5, doc 47): o osso é uma ENTIDADE, e a ligação de uma
//! coisa a ele é um componente dessa coisa.
//!
//! # Por que o osso não é um dado dentro de um componente
//!
//! Porque a cinemática direta **já corre**: `ph2d_ecs::propagate_transforms` compõe a pose de um
//! filho com a do pai, que é a definição de FK. Uma árvore de ossos guardada dentro de um
//! componente seria uma **segunda hierarquia** — exactamente o que a ADR-0110 rejeita pelo nome — e
//! teria de reimplementar, sozinha, o undo, o save, o olho, o cadeado, o reparentar e a timeline.
//!
//! ⇒ Um osso é uma entidade com [`ph2d_ecs::Transform`] mais este [`Bone`], que carrega só o que a
//! pose não sabe dizer: **o comprimento** (onde acaba o osso) e **a força** (até onde ele manda).
//! ⛔ Ele **não** é uma forma: não tem tinta, não exporta, não entra na cena vectorial. O que se
//! vê no canvas é overlay, como a gaiola do Envelope.
//!
//! # Por que os componentes moram AQUI e não na fundação
//!
//! Porque o esqueleto serve **várias mídias** (vector, raster, 3D, Flip), e um componente por
//! mídia dentro do `ph2d-ecs` faria a fundação crescer uma vez por cliente. O precedente é a
//! [`ph2d_physics_ecs`], que possui `RigidBody`/`Collider` e os regista pela porta dela.
//!
//! ⚠️ **Registar é obrigatório e o esquecimento é MUDO:** sem a chamada a
//! [`register_skeleton_components`] o `WorldSnapshot` **descarta** estes componentes em silêncio —
//! o desenho perde o esqueleto no primeiro undo e no primeiro save, sem uma linha de erro.
//!
//! # A ligação, e o que ela NÃO guarda
//!
//! O [`SkinBind`] segue o padrão do `ph2d_ecs::VecEnvelope` no que é da casa — a fonte autorada
//! viaja em **bytes postcard**, e a shell re-escreve a forma a cada quadro.
//!
//! ⚠️⚠️ **E ele NÃO guarda pesos.** O doc do `VecVertex::corner_radius` já escreveu a razão, sobre
//! por que o raio mora dentro do vértice: *"e não num vetor paralelo ao lado dos `verts`, de
//! propósito: dezenas de operações inserem, apagam, invertem e soldam vértices, e cada uma delas
//! teria de lembrar de mexer no vetor paralelo também."* Uma tabela de pesos indexada por ordem de
//! varredura **é** esse vector paralelo. Aqui guarda-se o **BIND** (a fonte + a matriz de repouso
//! de cada osso) e o peso é derivado dele a cada quadro — então editar a forma re-pesa sozinho.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use ph2d_ecs::SimComponent;
use ph2d_ecs::StableId;
use ph2d_ecs::scene::ComponentRegistry;
/// ⚠️ **Os dois tipos da curvatura vêm da LEI, não são declarados aqui** — o mesmo motivo que já
/// traz o [`ph2d_skeleton::BendSide`] por esta fronteira: duas definições do mesmo conceito
/// divergem no primeiro campo que alguém acrescentar a uma delas.
use ph2d_skeleton::bend::{Bend, BoneSpec, Handles};

// ⭐ **A pele e a LEI dela vivem no irmão** — corte por responsabilidade imposto pelo tecto de LOC
// (ver o cabeçalho de [`skin_bind`]). ⚠️ O endereço público **não muda**: quem escrevia
// `ph2d_skeleton_ecs::SkinBind` continua a escrevê-lo, que é a mesma lei do `bend_live`.
mod skin_bind;
pub use skin_bind::{SkinBind, SkinLaw};

// ⭐⭐⭐ **A POSE DE REPOUSO** — componente próprio, pela mesma lei do [`BoneLimit`]: *a ausência é
// uma resposta*. Ver o cabeçalho de [`bone_rest`], que tem o defeito medido que ela cura.
mod bone_rest;
pub use bone_rest::BoneRest;

/// ⭐ **Os dois tipos que um campo público do [`Bone`] nomeia, re-exportados daqui.**
///
/// ⚠️ **Não é conveniência — era uma lacuna:** quem vê `bone.handles` e `bone.curve` não
/// conseguia nomear o TIPO deles sem depender da crate da lei, e escrever `Handles::Auto` é
/// exactamente o que um consumidor do componente precisa de fazer. *Um campo público cujo tipo
/// não é alcançável pelo mesmo caminho é meio campo.*
///
/// ⛔ É um re-export e nunca uma segunda definição: a lei continua a ter um dono só.
pub use ph2d_skeleton::bend::{Bend as BoneBend, Handles as BoneHandles};

/// ⭐ **E o lado da dobra — o SEGUNDO caso que a nota acima previu.**
///
/// ⚠️ O [`IkGoal::bend`] é público e o tipo dele só era nomeável pela crate da lei, que um
/// consumidor do componente não tem por que conhecer. *Um campo público cujo tipo não é alcançável
/// pelo mesmo caminho é meio campo* — e quem o cobrou foi a `ph2d-timeline`, ao ganhar o canal que
/// anima este bit.
///
/// ⛔ Re-export e nunca uma segunda definição: a lei continua a ter um dono só.
pub use ph2d_skeleton::BendSide;

/// **UM OSSO.** A pose dele é o [`ph2d_ecs::Transform`] da entidade; a hierarquia dela é o
/// esqueleto.
///
/// O eixo do osso é o **+X local**, de `(0,0)` a `(length, 0)` — a convenção de toda a indústria
/// (Rive, Spine, Blender), e a que faz um filho pendurado na ponta ser só um `Transform` com
/// `translation.x = length`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bone {
    /// O comprimento, em unidades **locais** do osso — logo ele herda a escala do pai, como tudo o
    /// resto da hierarquia.
    pub length: f64,
    /// ⭐ **A FORÇA** — o raio de influência, em **comprimentos deste osso** (o *Bone Strength* do
    /// Moho). `1` = ele alcança um comprimento dele a partir do próprio eixo.
    ///
    /// ⚠️ **É um múltiplo e não uma distância, de propósito:** assim a lei é adimensional e o mesmo
    /// rig desenhado dez vezes maior deforma-se igual (gate
    /// `the_same_rig_ten_times_bigger_weighs_exactly_the_same`, em `ph2d-skeleton`).
    pub strength: f64,
    /// ⭐⭐ **EM QUANTOS SUB-OSSOS ELE DOBRA** — o *Segments* do *Bendy Bone* do Blender.
    ///
    /// `1` (o nascimento) ⇒ o osso rígido de sempre, **ao bit**. Saturado em
    /// [`ph2d_skeleton::bend::MAX_SEGMENTS`], cujo tecto é MEDIDO (a tabela vive no doc da const).
    pub segments: u8,
    /// ⭐⭐ **A CURVATURA AUTORADA** — as duas alças, como deslocamento a partir do terço do eixo.
    ///
    /// [`Bend::STRAIGHT`] (o nascimento) ⇒ o osso rígido de sempre, **ao bit**, seja qual for
    /// `segments`. ⚠️ **Ela arqueia o CORPO e não mexe a ponta**: quem manda na ponta é o
    /// `length` e a rotação, e o filho está pendurado ali — se a curvatura a movesse, a corrente
    /// abria uma fenda em cada junta ao dobrar.
    pub curve: Bend,
    /// ⭐⭐⭐ **DE ONDE VÊM AS DUAS ALÇAS** — o *Handle Type* do *Bendy Bone*.
    ///
    /// [`Handles::Authored`] (o nascimento) ⇒ as alças são as que o artista escreveu, e todo rig
    /// já autorado atravessa esta linha **ao bit**. [`Handles::Auto`] ⇒ elas saem das tangentes
    /// dos ossos VIZINHOS, e a corrente inteira vira uma curva lisa.
    ///
    /// ⚠️ **A resolução do `Auto` mora na `ph2d-skeleton-live`** (só ela conhece a hierarquia), e
    /// o campo `curve` fica **intocado** — ele continua a ser o que o artista escreveu, e voltar a
    /// `Authored` devolve exactamente o que lá estava. *Um modo que sobrescreve o valor autorado é
    /// um modo que não se desliga.*
    #[serde(default)]
    pub handles: Handles,
    /// ⭐⭐⭐ **QUEM MANDA NA PONTA DA CURVA** quando as alças vêm da corrente ([`Handles::Auto`]) —
    /// o *custom handle* do Blender, pedido pelo dono em 2026-09-16.
    ///
    /// ⚠️ **Ele só tem sujeito com `Handles::Auto`**: em `Authored` as duas alças são as que o
    /// artista escreveu, e ninguém as deriva de vizinho nenhum.
    ///
    /// ⚠️ **O nascimento é [`CurveTip::Chain`]**, que é a lei que sempre existiu — logo todo rig já
    /// gravado atravessa esta linha **ao bit**.
    #[serde(default)]
    pub curve_tip: CurveTip,
}

/// ⭐⭐⭐ **DE ONDE SAI A TANGENTE DA PONTA de um osso curvado pela corrente** — a resposta à
/// pergunta que a [`crate::Bone::handles`] em `Auto` faz ao osso SEGUINTE.
///
/// # ⛔ O problema que ele resolve, e que era uma ausência declarada
///
/// Um osso com **dois filhos-osso** não tem «o seguinte»: a corrente ramifica, e escolher um deles
/// seria um sorteio que muda com a ordem de varredura — então aquele lado ficava **recto**, sempre.
/// O dono viu a cena que o demonstra e mandou (2026-09-16): *«sim, escolher qual dos vários filhos
/// manda na curva. E quero que o modo atual (ninguém manda na curva) seja uma das opções»*.
///
/// # ⚠️ Porque o filho é um [`StableId`] e não os bits dele
///
/// A mesma cerca do [`IkGoal::target`] e do [`Tendon::bone`], e pelo mesmo defeito **medido**: bits
/// de alocação não sobrevivem ao respawn do undo, e guardados DENTRO dos bytes de um componente
/// envenenam o próprio undo.
///
/// ⚠️ **E ele é resolvido por COMPARAÇÃO entre os filhos**, nunca por uma busca global: a lei
/// pergunta *«qual dos meus filhos-osso tem este id?»*, o que a mantém `O(filhos)` e — mais
/// importante — impede que um osso de outro esqueleto mande na curva deste.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveTip {
    /// **A corrente decide**, e é a lei de sempre: o ÚNICO filho-osso manda; com zero ou mais de um,
    /// a ponta fica recta.
    #[default]
    Chain,
    /// ⭐ **NINGUÉM manda** — a ponta fica recta mesmo havendo um filho só.
    ///
    /// ⚠️ É o *«modo atual»* que o dono mandou manter como opção: num osso ramificado ele é o que
    /// acontece hoje, e aqui passa a ser uma ESCOLHA em vez de uma consequência.
    Straight,
    /// ⭐⭐ **ESTE filho manda** — o *custom handle*. Um id que já não é filho-osso deste osso
    /// resolve-se como [`CurveTip::Straight`]: o osso que mandava foi apagado ou mudou de pai, e
    /// *uma referência que não se resolve não pode inventar um sorteio*.
    Bone(StableId),
}

impl Bone {
    /// ⭐ **A ÚNICA conversão para a lei pura.** Escrita com os quatro campos **por nome**, então um
    /// campo novo no [`BoneSpec`] é erro de compilação aqui — que é o ponto de ela existir uma vez
    /// só. ⛔ Um segundo sítio a montar um `BoneSpec` à mão divergiria deste no primeiro campo novo.
    #[must_use]
    pub fn spec(&self) -> BoneSpec {
        BoneSpec {
            length: self.length,
            strength: self.strength,
            segments: self.segments,
            curve: self.curve,
        }
    }

    /// ⭐⭐⭐ **O `curve` deste osso é o que se VÊ, ou um valor que ninguém lê?**
    ///
    /// `true` com [`Handles::Authored`] — as duas alças são as que o artista escreveu, e o
    /// `curve` é exactamente a curvatura desenhada. `false` com [`Handles::Auto`]: ali as alças
    /// saem das tangentes dos VIZINHOS e o `curve` fica intocado de propósito (é isso que faz
    /// voltar a `Authored` devolver o que lá estava).
    ///
    /// ⛔⛔ **Ela existe por um report do dono (2026-09-17): *«não gravou as posições dos
    /// handles»*.** A tecla `K` da linha do tempo amostra o `curve`, e num osso em `Auto` isso é
    /// o valor AUTORADO — que numa cena assim vale `[0, 0]` enquanto o artista olha para uma
    /// curva bem visível. *Ele gravou; gravou ZEROS*, e nada lho disse.
    ///
    /// ⚠️ **UMA porta, dois leitores** (a amostragem do `K` e a recusa que a explica): escrita
    /// duas vezes, o dia em que uma delas ganhasse um terceiro modo deixaria a outra a capturar
    /// em silêncio outra vez.
    #[must_use]
    pub fn handles_are_authored(&self) -> bool {
        self.handles == Handles::Authored
    }
}

impl Default for Bone {
    fn default() -> Self {
        // ⚠️ **O nascimento é o osso de SEMPRE** — `segments = 1` e a curvatura recta fazem a
        // fábrica de sub-ossos colapsar num osso só, byte-idêntico ao que existia antes de haver
        // curvatura. É isso que faz todo rig já autorado continuar a deformar-se igual.
        Self {
            length: 1.0,
            strength: 1.0,
            segments: 1,
            curve: Bend::STRAIGHT,
            handles: Handles::Authored,
            curve_tip: CurveTip::Chain,
        }
    }
}

impl SimComponent for Bone {}

/// ⭐⭐⭐ **ATÉ ONDE ESTA JUNTA DOBRA** — o *IK Limits* do Blender, o *Angle constraints* do Moho.
///
/// Sem ele o cotovelo dobra para trás e o joelho hiperextende: a corrente alcança o alvo por um
/// caminho que um corpo não faz, e o rig lê-se como um barbante em vez de um membro.
///
/// # ⚠️ Ele mora no OSSO, e o Godot põe-no na RESTRIÇÃO — a diferença é deliberada
///
/// O `SkeletonModification2DCCDIK` guarda `constraint_angle_min`/`_max` **por junta da
/// modificação**, logo o limite existe enquanto existir aquela IK. Aqui ele é um componente do
/// **osso**, que é o modelo do Blender e do Moho, e a razão é que a afirmação *«este cotovelo não
/// dobra para trás»* é sobre a ANATOMIA e não sobre um solver: ela vale quando o artista gira o
/// osso à mão, vale sem IK nenhuma, e não pode evaporar quando ele carrega em *Remove IK*.
///
/// ⭐ E porque o osso é uma **entidade**, o limite ganha undo, save, olho, cadeado e timeline sem
/// uma linha de código própria — a mesma razão de tudo o resto deste módulo.
///
/// # A grandeza
///
/// Os dois ângulos são a rotação **LOCAL** do osso, em radianos — o mesmo espaço do
/// [`ph2d_ecs::Transform::rotation`] dele, que numa hierarquia já é *«quanto este osso está virado
/// em relação ao pai»*. ⛔ Um limite em espaço de MUNDO mudaria de significado quando o artista
/// girasse o ombro, que é exactamente o contrário do que uma junta é.
///
/// ⚠️ **A faixa de nascimento é a VOLTA INTEIRA**, e isso é o no-op exacto: pendurar o componente
/// não move a pose um bit. Quem aperta é o artista.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoneLimit {
    /// O extremo mais **horário** da faixa, em radianos locais.
    pub min: f64,
    /// O extremo mais **anti-horário**, em radianos locais.
    ///
    /// ⚠️ `max < min` não é estado inválido a evitar com uma guarda: a lei
    /// ([`ph2d_skeleton::clamp_to_limit`]) trava a junta no **centro** do que estiver escrito. Um
    /// `f64::clamp` com os limites trocados **aborta o processo**, e um ficheiro editado à mão
    /// chega lá.
    pub max: f64,
}

impl Default for BoneLimit {
    fn default() -> Self {
        Self {
            min: -ph2d_skeleton::FULL_TURN / 2.0,
            max: ph2d_skeleton::FULL_TURN / 2.0,
        }
    }
}

impl SimComponent for BoneLimit {}

/// ⭐⭐⭐ **O OSSO INTELIGENTE** — girar este osso PERCORRE uma animação inteira.
///
/// É o *Smart Bone* do Moho e o *Action Constraint* do Blender: o artista grava uma acção (um
/// clip da timeline) e diz *«quando este osso vai de A a B, a acção vai do princípio ao fim»*. Um
/// osso passa a ser um **controlo**, e não só uma peça que roda.
///
/// # ⭐⭐ Para que serve, e por que não se resolve com pesos
///
/// O caso canónico é a **correcção**: um cotovelo dobrado a 120° amassa a manga, e nenhum ajuste de
/// força do osso arruma isso — a deformação certa naquele ângulo é uma pose AUTORADA, não uma
/// interpolação. O segundo é o **controlo composto**: um osso solto que não deforma nada por si e
/// abre uma boca, fecha uma mão, vira uma cabeça de perfil.
///
/// # As referências
///
/// | ferramenta | como |
/// |---|---|
/// | **Moho** | *Smart Bone* — o osso tem uma acção própria, e o ângulo dele é o tempo dela |
/// | **Blender** | *Action Constraint* — `Target` + canal + `Range Min/Max` → `Action Frame Start/End` |
/// | **Godot** (MIT) | `AnimationNodeBlendSpace1D` — um valor dirige pontos de mistura num eixo |
///
/// ⚠️ **A nossa forma é a do Blender e a do Moho** (um ângulo → o TEMPO de uma acção), e não a do
/// Godot (um valor → a MISTURA de N animações). A diferença importa: misturar duas poses precisa
/// que elas existam as duas e sejam compatíveis; percorrer um clip precisa de **um** clip, e é o
/// que o artista já sabe fazer nesta casa — ele grava na timeline que já existe.
///
/// # ⚠️ O clip é nomeado pelo NOME, nunca pelo índice
///
/// É a lei desta casa (*referência durável entre objectos é o NOME*): reordenar ou apagar clips
/// mexe em todos os índices, e um índice guardado passaria a apontar para a animação do vizinho —
/// em silêncio, que é o pior modo de falha.
///
/// # ⚠️⚠️ O gesto NÃO CRIA NADA — o painel é que escolhe (2026-09-08)
///
/// *Add Smart Bone* anexa o componente **vazio**; quem lhe dá sujeito são as duas linhas da secção
/// Skeleton — o ***Pick Object*** (o alvo) e o selector ***Action*** (a animação, filtrada por ele).
/// ⛔ **Dois desenhos anteriores caíram, cada um por um report do dono no mesmo dia:** *adoptar o
/// clip ABERTO* casava todo controlo com a animação principal da cena (um documento novo tem **uma**
/// acção, `"Main"`), e *criar uma acção com o nome do osso* fabricava duas coisas por um clique
/// (*«porque criar Bone Action no inspector e na timeline? Melhor não criar nada»*).
///
/// # ⚠️ E um controlo NÃO percorre a acção que está ABERTA
///
/// Ali o artista está a **gravá-la**. Os dois escreveriam o mesmo objecto no mesmo quadro, e o
/// passe do controlo corre **depois** do da timeline ⇒ arrastar o playhead não moveria nada e a
/// pose acabada de pôr seria reposta antes de ser vista. É a lei do Moho — dentro de uma acção o
/// relógio é o do editor, não o ângulo do osso.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SmartBone {
    /// O nome do clip que este osso percorre. Vazio ⇒ inerte (o osso é um osso normal).
    pub clip: String,
    /// ⭐⭐⭐ **O NOME do objecto de que este controlo trata.** Vazio ⇒ nenhum escolhido.
    ///
    /// ⚠️⚠️ **Ele NÃO muda o que a acção faz** — ela corre inteira, como sempre. O que ele muda é a
    /// LISTA que o painel oferece: só as acções que animam este objecto. Report do dono
    /// (2026-09-08): *«é necessário um botão de picker para selecionar o objeto … só deve aparecer
    /// as animações relacionadas ao objeto selecionado»*.
    ///
    /// ⚠️ **NOME e não bits**, pela lei da referência durável: o undo respawna tudo com bits novos,
    /// e bits dentro dos bytes de um componente envenenam o próprio undo.
    ///
    /// ⚠️ **Um alvo que já não resolve não esconde nada** — a lista volta a mostrar todas as
    /// acções. Um filtro sobre um objecto apagado deixaria o artista com um selector vazio e sem
    /// gesto que o cure, que é pior que filtro nenhum.
    pub target: String,
    /// O ângulo local (radianos) em que a acção está no **princípio**.
    pub from: f64,
    /// ... e no **fim**.
    ///
    /// ⚠️ `to == from` é faixa nula, e a lei devolve o princípio da acção — ⛔ nunca uma divisão por
    /// zero. Uma faixa invertida (`to < from`) é legítima: ela percorre a acção **ao contrário**,
    /// que é o que o artista quer quando o controlo dele gira para o outro lado.
    pub to: f64,
}

impl Default for SmartBone {
    fn default() -> Self {
        Self {
            clip: String::new(),
            target: String::new(),
            // ⚠️ Um quarto de volta — **um número de produto, escolhido, e não um tecto de
            // recurso**: é a amplitude que um dial de controlo pede sem obrigar o artista a dar meia
            // volta com o rato. ⛔ `0..0` seria faixa nula e o controlo nasceria morto, e a lei
            // devolveria sempre o princípio da acção.
            //
            // ⛔⛔ **A nota anterior dizia «o mesmo valor de nascimento do `BoneLimit`, e pela mesma
            // razão», e as duas metades eram falsas** (auditoria de 2026-09-08): o `Default` do
            // `BoneLimit` é a **volta inteira** (o quarto de volta é o valor do GESTO *Add Angle
            // Limit*, noutra crate), e a razão citada era a de um ARRASTO — e estes dois números
            // não têm arrasto nenhum, são dois campos digitados em graus.
            from: 0.0,
            to: ph2d_skeleton::FULL_TURN / 4.0,
        }
    }
}

impl SimComponent for SmartBone {}

/// ⭐⭐⭐ **A ÂNCORA** — a restrição de cinemática inversa que **FICA**.
///
/// Ela mora no osso da PONTA da corrente, que é o modelo do Blender (*Inverse Kinematics* é uma
/// *bone constraint* no último osso) e o que faz undo, save, olho, cadeado e timeline valerem aqui
/// sem uma linha de código próprio — pela mesma razão de o osso ser uma entidade.
///
/// # ⭐⭐ Por que ela não é o arrasto que já existia
///
/// O arrasto da ponta **posa** — acaba quando o dedo levanta. Uma restrição **persiste**: o artista
/// anima UM objecto (a mão) e o braço inteiro segue-o, para sempre e através da timeline. É a
/// diferença entre *«um editor de esqueletos»* e *«um editor de animação»*, e as quatro referências
/// entregam-na assim:
///
/// | Referência | onde vive | o quanto | a corrente | a suavidade |
/// |---|---|---|---|---|
/// | Blender | *bone constraint* na ponta | *Influence* `0..1` | *Chain Length* (`0` = tudo) | — |
/// | Spine | `IkConstraint` | *Mix* `0..1` | lista de ossos | *Softness* |
/// | Rive | `IKConstraint` | *Strength* `0..100` | *Bone Count* | — |
/// | Moho | *bone constraint* | — | a cadeia do osso | — |
///
/// ⚠️ **O ALVO é uma entidade qualquer**, e é isso que o torna poderoso: ele tem `Transform`, logo
/// a timeline anima-o, o gizmo move-o, o undo desfá-lo e a Hierarquia mostra-o — tudo de graça.
///
/// ⛔ **O alvo NUNCA pode ser descendente da corrente.** Mover o osso moveria o alvo, que moveria o
/// osso: o laço realimenta-se e a corrente vibra. O passe **recusa** esse caso em vez de o resolver,
/// porque não há resposta certa para ele.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IkGoal {
    /// A identidade durável do objecto que a ponta persegue. [`StableId::NONE`] ⇒ a restrição está
    /// inerte (o alvo foi apagado), e o passe deixa a corrente em paz.
    ///
    /// ⚠️ Mesma cerca do [`Tendon::bone`], e pelo mesmo defeito medido: bits de alocação não
    /// sobrevivem ao respawn do undo.
    pub target: StableId,
    /// Quantos ossos a corrente tem, **contados da ponta para cima**. `0` = até à raiz do esqueleto.
    ///
    /// ⚠️ **Não tem tecto escrito, e é de propósito:** quem o limita é a corrente REAL
    /// (`min(chain, comprimento da cadeia)`), que é um número derivado do documento e não um palpite
    /// — um valor absurdo vindo de um ficheiro é aparado pela árvore, não por uma constante.
    ///
    /// ⭐ O valor de NASCIMENTO é **2** — o membro, o par que a lei fechada resolve exactamente. ⛔ O
    /// `0` do Blender (*«até à raiz»*) é o default dele e é a queixa nº 1 documentada da feature: ao
    /// primeiro arrasto o esqueleto inteiro dobra.
    pub chain: u32,
    /// ⭐ **A MISTURA** (`0..1`) — o *Mix* do Spine. `0` = a restrição está desligada e a pose é a
    /// que o artista autorou; `1` = a corrente obedece ao alvo.
    ///
    /// ⚠️ Ela mistura **ângulos**, não posições — ver [`ph2d_skeleton::blend_angle`]: interpolar
    /// posições encurtaria os ossos.
    pub mix: f64,
    /// ⭐ **A SUAVIDADE** — o *Softness* do Spine, em **fracção do alcance da corrente**.
    ///
    /// ⚠️ **Fracção e não distância, de propósito** — a mesma decisão do [`Bone::strength`]: assim a
    /// lei é adimensional e o mesmo rig desenhado dez vezes maior abranda no mesmo sítio. `0` = o
    /// corte a seco de sempre, **ao bit**.
    pub softness: f64,
    /// ⭐⭐⭐ **DE QUE LADO O JOELHO DOBRA** — o `flip_bend_direction` do Godot, o `bendDirection`
    /// do Spine. Ver [`ph2d_skeleton::BendSide`] para o mecanismo e para por que a resposta 2D é um
    /// interruptor e não o *pole target* do Blender.
    ///
    /// ⚠️ **O valor de nascimento é CAPTURADO, não escolhido:** o `add` mede de que lado a corrente
    /// já está e grava-o. É a mesma lei que o *Remove IK* pagou — *o que volta é o AUTORADO* —, e é
    /// o que faz a âncora nascer **byte-idêntica** à pose que o artista posou à mão, e ficar lá.
    ///
    /// ⚠️ [`ph2d_skeleton::BendSide::Keep`] continua a ser um estado legítimo (é o que um ficheiro
    /// gravado antes desta wave recebe, e é o comportamento que ele tinha): ali o lado sai da pose,
    /// e a corrente **inverte** ao passar pela recta.
    pub bend: ph2d_skeleton::BendSide,
}

impl Default for IkGoal {
    fn default() -> Self {
        Self {
            target: StableId::NONE,
            chain: DEFAULT_CHAIN,
            mix: 1.0,
            softness: 0.0,
            // ⚠️ `Keep` aqui e **capturado** no `add`: o default de um componente é o que um
            // ficheiro sem o campo recebe, e para esse a resposta honesta é *«o que ele fazia»*.
            bend: ph2d_skeleton::BendSide::Keep,
        }
    }
}

/// Quantos ossos uma âncora nova governa — **o membro**.
///
/// ⭐ Dois é a corrente que a lei dos cossenos resolve **fechada e exacta** (sem iteração nenhuma),
/// e é a anatomia de um braço e de uma perna. ⛔ O `0` do Blender (*«até à raiz»*) é o default dele
/// e a queixa nº 1 da feature: um clique e o esqueleto inteiro passa a dobrar.
pub const DEFAULT_CHAIN: u32 = 2;

impl SimComponent for IkGoal {}

/// ⭐ **ESTE OBJECTO É UMA ÂNCORA DE IK** — a marca que o alvo carrega.
///
/// # ⚠️ Por que uma marca, e não uma pergunta derivada
///
/// A pergunta *«alguma âncora nomeia este objecto?»* é derivável do [`IkGoal`], e **o preço a
/// tornaria O(n²)**: quem precisa dela é o `group_gizmo_view::empty_objects`, que **varre o mundo
/// inteiro a cada quadro** e faria uma varredura de âncoras por entidade visitada.
///
/// ⛔ E ela **tem de ser registada**: sem isso o `WorldSnapshot` descarta-a em silêncio, e a marca
/// evapora no primeiro Ctrl+Z — o anel de objecto vazio voltaria a aparecer por cima do losango,
/// **só depois de desfazer**, que é o pior modo de falha possível (intermitente e sem erro).
///
/// # ⭐⭐ O que ela compra
///
/// Um alvo é um objecto com `Transform` e sem pixels ⇒ ele responde *sim* ao
/// `group_gizmo_view::is_empty_object`, e ganharia **um segundo anel** concêntrico com o losango,
/// mais um disco a **disputar o clique** com a alça. ⚠️ É o defeito literal que o report do dono de
/// 2026-09-06 (*«alguns bones têm círculos grandes e pequenos»*) já custou uma vez, e o doc daquela
/// cura escreveu a lei: *uma família nova que ganhe alças próprias e não venha a esta lista nasce
/// com duas caixas sobre o mesmo objecto*.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IkTarget;

impl SimComponent for IkTarget {}

/// Um osso a que **esta** coisa está presa, com a pose de repouso dele no instante do bind.
///
/// ⭐ O nome é o do Rive, que chama a mesma coisa exactamente assim — um *Tendon* liga um osso a
/// uma pele.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tendon {
    /// ⭐⭐⭐ **A IDENTIDADE DURÁVEL do osso** — o [`StableId`], nunca o `Entity::to_bits()`.
    ///
    /// ⚠️⚠️ **A 1.ª redacção guardava os bits da entidade, e MEDIU-SE que a pele morria em
    /// silêncio no primeiro Ctrl+Z** (gate `a_skin_survives_the_respawn_that_undo_and_save_do`,
    /// `0 de 2` tendões a resolver): o undo e o salvar são a mesma máquina, ela **despawna e
    /// re-spawna no mesmo mundo**, e o `to_bits` é um **id de alocação** — a geração sobe e os bits
    /// deixam de nomear nada. O sintoma não é um erro: a forma fica com a última geometria boa e
    /// deixa de responder aos ossos.
    ///
    /// ⛔ E os bits guardados eram também um **pânico à espera**: o `Entity::from_bits` do
    /// `bevy_ecs` aborta o processo com bits que nunca vieram de um `to_bits`, e é exactamente
    /// isso que um ficheiro gravado noutra sessão entrega.
    ///
    /// ⭐ **O tipo é a cerca**, e não o nome do campo: com `StableId` aqui, escrever um
    /// `e.to_bits()` neste sítio é **erro de compilação**. Foi a lição do anel da ponta — *um valor
    /// que muda de significado e mantém a forma não avisa ninguém*.
    ///
    /// A porta de escrita é o [`ph2d_ecs::stable_id_of`] e a de leitura o
    /// [`ph2d_ecs::entity_of_stable_id`]; é a mesma identidade por que a `PhysicsJoint` nomeia os
    /// corpos dela desde a wave das instâncias.
    pub bone: StableId,
    /// ⭐ **O TENDÃO** — o afim `osso → coisa` no instante em que se ligou, em `[a,b,c,d,e,f]`.
    ///
    /// É daqui que sai TUDO: o eixo de repouso (`rest·(0,0)` até `rest·(length,0)`, que é o que a
    /// distância mede) e a matriz da pose (`S⁻¹ ∘ B ∘ rest⁻¹`). ⚠️ E é por ele ser o composto
    /// `coisa⁻¹ ∘ osso` que a pose de repouso é a **identidade** sem uma guarda escrita à mão.
    pub rest: [f64; 6],
}

/// Regista os componentes que a `ph2d-skeleton-ecs` possui. A shell chama isto uma vez no arranque,
/// ao lado do `register_ecs_components` e do `register_physics_components`.
///
/// ⚠️ **Sem esta chamada o `WorldSnapshot` descarta-os em SILÊNCIO** — é o bug
/// `Locked`/`GroupedChildren`/`VecPathRef` que a física já pagou, e aqui ele apareceria como *"o
/// personagem perdeu o esqueleto ao desfazer"*.
pub fn register_skeleton_components(reg: &mut ComponentRegistry) {
    // `register_default`: um osso de comprimento 1 e força 1 é um osso legítimo, então a paleta do
    // Inspector pode pendurá-lo.
    reg.register_default::<Bone>("ph2d::skeleton::Bone");
    // `register`: uma pele sem a fonte autorada dentro não é uma pele, é uma forma prestes a sumir
    // — ela chega pelo GESTO (*Bind*) e nunca por um botão de "acrescentar componente".
    reg.register::<SkinBind>("ph2d::skeleton::Skin");
    // `register` pela MESMA razão: uma âncora sem alvo não governa nada. Ela chega pelo gesto
    // (*Add IK*), que cria o alvo no mesmo passo — pendurá-la por paleta daria uma restrição inerte
    // que o artista não teria como completar.
    reg.register::<IkGoal>("ph2d::skeleton::IkGoal");
    // ⚠️ **Registada, e o esquecimento seria INTERMITENTE:** a marca do alvo evaporaria no primeiro
    // Ctrl+Z e o anel de objecto vazio voltaria por cima do losango — um defeito que só aparece
    // depois de desfazer, que é o pior modo de falha que há.
    reg.register_default::<IkTarget>("ph2d::skeleton::IkTarget");
    // `register_default`: a faixa de nascimento é a volta inteira, que **não apara nada** — logo a
    // paleta do Inspector pode pendurá-lo sem mover a pose, e o artista aperta-o depois.
    reg.register_default::<BoneLimit>("ph2d::skeleton::BoneLimit");
    // ⭐ `register_default` desde 2026-09-08, e a razão MUDOU com o desenho: enquanto o gesto lhe
    // dava a acção, pendurá-lo por paleta daria um controlo inerte e sem caminho de conclusão —
    // hoje o gesto **não cria nada** (ordem do dono) e as duas linhas da secção Skeleton
    // (*Pick Object* · *Action*) completam-no. Nascer vazio é um no-op exacto: o `drive` salta um
    // clip vazio. ⚠️ O `Attach::Authored` do descritor e este `register_default` são **as duas
    // metades da mesma decisão**, e o gate `every_offered_component_can_be_constructed` reprova
    // quem mexer numa só.
    reg.register_default::<SmartBone>("ph2d::skeleton::SmartBone");
    // ⛔⛔ **`register` e NUNCA `register_default`, e a ausência do default É a cura desta wave:** o
    // valor neutro de uma pose é a **identidade**, que é exactamente o byte que mandava a arte
    // presa 20 unidades para longe. Um repouso que não diz QUAL pose não é um repouso — ele chega
    // com o osso (o gesto de o criar) ou com o verbo *Set Rest Pose*, e um osso sem ele faz o
    // verbo recusar em voz alta em vez de adivinhar.
    reg.register::<BoneRest>("ph2d::skeleton::BoneRest");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **Esta contagem existe para doer** (espelha `registers_every_physics_component`): um
    /// componente do esqueleto que salte o registo é descartado de todo snapshot em silêncio. Se
    /// acrescentar um, suba o número.
    #[test]
    fn registers_every_skeleton_component() {
        let mut reg = ComponentRegistry::new();
        register_skeleton_components(&mut reg);
        assert_eq!(reg.len(), 7);
        assert!(reg.get_by_name("ph2d::skeleton::Bone").is_some());
        assert!(reg.get_by_name("ph2d::skeleton::Skin").is_some());
        assert!(reg.get_by_name("ph2d::skeleton::IkGoal").is_some());
        assert!(reg.get_by_name("ph2d::skeleton::IkTarget").is_some());
        // ⚠️ **Nomeado, e não só contado:** um repouso que salte o registo evapora no primeiro
        // Ctrl+Z, e o sintoma é *«o botão de voltar ao repouso deixou de funcionar»* — depois de
        // desfazer, que é o pior modo de falha que há.
        assert!(reg.get_by_name("ph2d::skeleton::BoneRest").is_some());
    }

    /// ⭐ **O NOME CANÓNICO NÃO DIZ "VECTOR"** — e é a metade destrutiva-depois desta wave.
    ///
    /// O `ComponentBlob` é endereçado por `blake3(nome canónico)`, então trocar o nome depois de
    /// existirem projectos gravados faria cada um deles **perder o esqueleto em silêncio** ao
    /// abrir. Medido em 2026-09-06: os dois `.ph2dproj` da máquina do dono são de 26/08, onze dias
    /// antes de os ossos existirem ⇒ nenhum tem esqueleto, e a troca custou zero.
    ///
    /// Este gate impede que alguém devolva a palavra ao nome sem reabrir aquela conta.
    #[test]
    fn the_canonical_names_belong_to_the_module_not_to_one_medium() {
        let mut reg = ComponentRegistry::new();
        register_skeleton_components(&mut reg);
        for d in reg.iter() {
            assert!(
                d.canonical_name.starts_with("ph2d::skeleton::"),
                "`{}` nao mora no modulo - um nome por midia volta a prender o esqueleto a ela",
                d.canonical_name
            );
            assert!(
                !d.canonical_name.to_ascii_lowercase().contains("vec"),
                "`{}` ainda diz VECTOR, e o esqueleto serve raster, 3D e Flip",
                d.canonical_name
            );
        }
    }

    /// Um osso nasce legítimo: comprimento 1, força 1 — e a força é um MÚLTIPLO, o que faz a lei
    /// ser adimensional.
    #[test]
    fn a_fresh_bone_is_one_long_and_reaches_one_of_itself() {
        let b = Bone::default();
        assert_eq!(b.length, 1.0);
        assert_eq!(b.strength, 1.0);
    }
}
