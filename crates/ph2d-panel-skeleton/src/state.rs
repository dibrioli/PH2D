//! O estado do **ESQUELETO** publicado pela shell — a família inteira de uma feature, com os seus
//! statics ao lado dos seus acessores.
//!
//! ⚠️ Ele viveu dentro do painel de vetor (`state_bone.rs`) enquanto o esqueleto era uma secção de
//! lá; mudou-se com ele em 2026-09-09, sem uma linha de lei alterada.
//!
//! ⚠️ **O painel não vê o `ph2d-ecs`** (a UI vive de snapshots publicados, nunca do mundo), então o
//! que atravessa são NÚMEROS e não componentes.

use ph2d_editor_core::zones::Rect;
use std::cell::{Cell, RefCell};

thread_local! {
    /// A seleção contém pelo menos uma forma PRESA a um esqueleto? Decide se as duas saídas
    /// (Keep Pose / Release) são oferecidas — *um botão que só sabe recusar é pior que um ausente*.
    static CURRENT_IK_AUTO_SIDE: Cell<Option<usize>> = const { Cell::new(None) };
    /// ⭐⭐⭐ **Esta âncora APONTA, e com que desvio** — `Some(graus)` ⇒ ela aponta; `None` ⇒ ela
    /// alcança (ou não há âncora nenhuma).
    ///
    /// ⚠️ **UM valor para os DOIS factos, de propósito:** *«aponta»* e *«o desvio dela»* viajam
    /// juntos e não podem discordar. Com duas células, um quadro podia publicar *«aponta»* com o
    /// desvio do osso anterior — e o campo mostraria um número que não é de ninguém.
    static CURRENT_IK_AIM: Cell<Option<f64>> = const { Cell::new(None) };
    static CURRENT_ENVELOPE_MANDA: Cell<bool> = const { Cell::new(true) };
    /// ⭐ **A selecção deforma-se POR ALCANCE?** — a escolha que a fileira `Deform By` mostra.
    ///
    /// ⚠️ **Um `bool` e não o enum**, pela lei deste ficheiro: *o painel não vê o `ph2d-ecs`*, e o
    /// que atravessa são números. ⛔ E ele é `false` no nascimento porque `SkinLaw::Auto` é o
    /// `#[default]` da lei — *duas respostas ao mesmo nascimento divergem no dia em que uma mudar.*
    static CURRENT_SKIN_LAW_ENVELOPE: Cell<bool> = const { Cell::new(false) };
    static CURRENT_SKINNED: Cell<Skinned> = const { Cell::new(Skinned { vector: false, imagem: false }) };
    /// O OSSO em foco existe? Sem ele, `Length`/`Strength` não têm sujeito.
    static CURRENT_HAS_BONE: Cell<bool> = const { Cell::new(false) };
    static CURRENT_BONE_LENGTH: Cell<f64> = const { Cell::new(0.0) };
    static CURRENT_BONE_STRENGTH: Cell<f64> = const { Cell::new(1.0) };
    /// ⭐ Os SEGMENTOS e a CURVATURA do osso em foco (a F8). ⚠️ Eles decidem se as quatro fileiras
    /// da curvatura são pintadas — ver [`crate::section_campos::campos_do_osso`].
    static CURRENT_BONE_SEGMENTS: Cell<u8> = const { Cell::new(1) };
    static CURRENT_BONE_CURVE: Cell<ph2d_skeleton::bend::Bend> =
        const { Cell::new(ph2d_skeleton::bend::Bend::STRAIGHT) };
    /// ⭐ **DE ONDE vêm as alças** — o índice em [`ph2d_skeleton::bend::Handles`]: `0` autorado,
    /// `1` derivado da corrente.
    ///
    /// ⚠️ **Publicado à parte do [`CURRENT_BONE_CURVE`], e não dentro do `BoneSpec`:** o
    /// `BoneSpec` é o osso EFECTIVO que a lei consome, e o modo é uma propriedade de AUTORIA —
    /// metê-lo lá poria a lei a decidir quem a escreve.
    static CURRENT_BONE_HANDLES: Cell<Option<usize>> = const { Cell::new(None) };
}

/// O modo das alças do osso em foco (publicado pela shell, todo quadro). `None` ⇒ sem osso.
pub fn set_current_bone_handles(v: Option<usize>) {
    CURRENT_BONE_HANDLES.with(|c| c.set(v));
}

pub(crate) fn current_bone_handles() -> Option<usize> {
    CURRENT_BONE_HANDLES.with(Cell::get)
}

/// ⭐⭐⭐ **O QUE A SELECÇÃO TEM PRESO, pelas DUAS mídias.**
///
/// ⛔⛔⛔ **Era um `bool` que só conhecia formas vectoriais, e o defeito era MUDO** (report do dono,
/// 2026-09-18: *«ainda não temos a opção de desconectar a malha do osso»*): com uma IMAGEM presa
/// escolhida ele lia `false`, e os botões *Expand* e *Release* **nem chegavam a ser pintados**.
/// ⚠️ **O cabeçalho da fase que o publica já dizia *«forma presa ou imagem com pele»*** — *um doc
/// que declara a lei que o código não implementa lê-se como auditado*.
///
/// ⚠️ **É um TIPO e não dois publicadores**, pela lei que este ficheiro já escreve para o
/// `BoneSpec`: os dois campos viajam juntos, e um publicador por campo deixaria um quadro em que
/// uma metade é desta selecção e a outra da anterior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Skinned {
    /// Há uma FORMA vectorial presa na selecção.
    pub vector: bool,
    /// Há uma IMAGEM presa na selecção.
    pub imagem: bool,
}

impl Skinned {
    /// Há alguma coisa presa — é o que decide se a fileira de saídas se pinta.
    #[must_use]
    pub fn alguma(self) -> bool {
        self.vector || self.imagem
    }
}

/// ⭐⭐⭐ **O ENVELOPE AINDA MANDA EM ALGUMA COISA?** (publicado pela shell, todo quadro).
///
/// ⛔ `false` ⇒ a cena não tem **nenhuma** forma vectorial presa, e nesse caso o campo *Strength* é
/// **provadamente inerte**: com os pesos do padrão-ouro uma imagem deforma igual a `1` e a `2`
/// (medido, coluna a coluna). ⇒ o painel esconde-o, pela mesma lei que já esconde as quatro alças
/// de curvatura num osso que não as sabe ler.
///
/// ⚠️ **O default é `true`, e a escolha é conservadora:** antes de a shell publicar o que quer que
/// seja, o campo fica **à vista**. *Esconder um controlo vivo é pior do que mostrar um inerte.*
/// A selecção deforma-se POR ALCANCE? Publicado pela shell, todo quadro.
///
/// ⚠️ **Com várias formas escolhidas, a shell publica `true` se ALGUMA delas estiver por alcance** —
/// e essa escolha é declarada: um chip que só acendesse com unanimidade deixaria o artista sem
/// saber que metade da selecção está noutra lei.
pub fn set_current_skin_law_envelope(v: bool) {
    CURRENT_SKIN_LAW_ENVELOPE.with(|c| c.set(v));
}

pub(crate) fn skin_law_envelope() -> bool {
    CURRENT_SKIN_LAW_ENVELOPE.with(Cell::get)
}

pub fn set_current_envelope_manda(v: bool) {
    CURRENT_ENVELOPE_MANDA.with(|c| c.set(v));
}

pub(crate) fn envelope_manda() -> bool {
    CURRENT_ENVELOPE_MANDA.with(Cell::get)
}

/// O que a seleção tem preso a esqueleto (publicado pela shell, todo quadro).
pub fn set_current_skinned(v: Skinned) {
    CURRENT_SKINNED.with(|c| c.set(v));
}

pub(crate) fn skinned() -> Skinned {
    CURRENT_SKINNED.with(Cell::get)
}

// ⛔⛔ **`has_skeleton` NÃO existe aqui, e a ausência é a decisão** (2026-09-09). Enquanto isto era
// uma SECÇÃO, o facto *«a cena tem esqueleto?»* atravessava para ela decidir se se pintava. Agora
// quem decide é a shell, pela porta que todo painel já tem (`panel_visible`) — e um facto publicado
// que ninguém lê é a espécie de estado morto que o §5.0 nomeia.

/// O osso em foco, INTEIRO. `None` ⇒ a seleção não é um osso.
///
/// ⚠️⚠️ **Ele publica o `BoneSpec` e não os dois números soltos, de propósito:** os quatro campos
/// viajam sempre juntos, e um publicador por campo deixaria um quadro em que os segmentos são de um
/// osso e a curvatura do anterior — o defeito que o publicador do *Smart Bone* já nomeia por escrito
/// duas dúzias de linhas abaixo. ⭐ E um campo novo no `BoneSpec` **não compila** aqui até alguém
/// dizer o que o painel faz com ele.
pub fn set_current_bone(v: Option<ph2d_skeleton::bend::BoneSpec>) {
    CURRENT_HAS_BONE.with(|c| c.set(v.is_some()));
    if let Some(s) = v {
        CURRENT_BONE_LENGTH.with(|c| c.set(s.length));
        CURRENT_BONE_STRENGTH.with(|c| c.set(s.strength));
        CURRENT_BONE_SEGMENTS.with(|c| c.set(s.segments));
        CURRENT_BONE_CURVE.with(|c| c.set(s.curve));
    }
}

pub(crate) fn current_bone() -> Option<ph2d_skeleton::bend::BoneSpec> {
    CURRENT_HAS_BONE
        .with(Cell::get)
        .then(|| ph2d_skeleton::bend::BoneSpec {
            length: CURRENT_BONE_LENGTH.with(Cell::get),
            strength: CURRENT_BONE_STRENGTH.with(Cell::get),
            segments: CURRENT_BONE_SEGMENTS.with(Cell::get),
            curve: CURRENT_BONE_CURVE.with(Cell::get),
        })
}

thread_local! {
    /// O osso em foco tem ÂNCORA de IK? Decide entre *Add IK* e *Remove IK*, e se os três números
    /// dela têm sujeito. ⛔ Sem isto o painel ofereceria as duas portas ao mesmo tempo, e uma delas
    /// só saberia recusar.
    static CURRENT_HAS_IK: Cell<bool> = const { Cell::new(false) };
    static CURRENT_IK_MIX: Cell<f64> = const { Cell::new(1.0) };
    static CURRENT_IK_SOFTNESS: Cell<f64> = const { Cell::new(0.0) };
    static CURRENT_IK_CHAIN: Cell<f64> = const { Cell::new(2.0) };
    /// ⭐ De que lado o joelho dobra — o ÍNDICE em `BendSide::ALL`, que é o que a fileira de
    /// segmentos precisa para saber qual acender. ⚠️ Guardar o índice e não o enum é o mesmo
    /// idioma do `VECTOR_BONE_ACTION_IDS`: quem alinha as duas listas é a POSIÇÃO.
    static CURRENT_IK_BEND: Cell<usize> = const { Cell::new(0) };
    /// ⭐ O limite de ângulo da junta em foco, em GRAUS. `None` ⇒ ela gira livremente, e o painel
    /// oferece a porta de entrada em vez dos dois números.
    static CURRENT_LIMIT: Cell<Option<(f64, f64)>> = const { Cell::new(None) };
    /// ⭐⭐⭐ **O OSSO INTELIGENTE em foco, inteiro.** `None` ⇒ ele não é um controlo.
    ///
    /// ⚠️ **Um slot só para os cinco factos**, e não cinco publicações: publicá-los por portas
    /// separadas deixaria um quadro em que a faixa é de um osso e o nome é do anterior — e o
    /// defeito seria invisível, porque cada leitura é individualmente correcta.
    static CURRENT_SMART: RefCell<Option<SmartBoneView>> = const { RefCell::new(None) };
    /// ⭐ **As acções que o DOCUMENTO tem** — a lista que o selector mostra, publicada pela shell.
    ///
    /// ⚠️ Ela é do documento e não do osso: dois ossos inteligentes escolhem de entre as mesmas
    /// acções, e uma segunda leitura no painel envelheceria no primeiro clip que ele criasse.
    static CURRENT_ACTIONS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// **O selector de acção que está ABERTO** — irmão do `PENDING_KEY_DD` do Morph, e pela mesma
    /// razão: a seção rola, e sem o passe diferido a lista seria cortada na borda dela.
    static PENDING_ACTION_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    /// ⭐⭐⭐ **QUEM PODE MANDAR NA PONTA da curva do osso em foco** — a lista que o selector mostra,
    /// publicada pela shell a partir da porta [`ph2d_app_skeleton::curve_tip::options`].
    ///
    /// ⚠️ **Ela vem inteira de lá, incluindo os RÓTULOS**: o painel não deriva a lista, porque quem
    /// a aplica (a shell) tem de ler exactamente a mesma — a posição é a escolha.
    static CURRENT_TIP: RefCell<Option<TipView>> = const { RefCell::new(None) };
    /// O selector de PONTA que está ABERTO — irmão do `PENDING_ACTION_DD`, e pela mesma razão.
    static PENDING_TIP_DD: Cell<Option<Rect>> = const { Cell::new(None) };
}

/// **As acções do documento** (shell → painel, todo quadro em que a seção vive).
pub fn set_current_bone_actions(names: Vec<String>) {
    CURRENT_ACTIONS.with(|c| *c.borrow_mut() = names);
}

pub(crate) fn bone_actions() -> Vec<String> {
    CURRENT_ACTIONS.with(|c| c.borrow().clone())
}

pub(crate) fn set_pending_bone_action_dd(chip: Option<Rect>) {
    PENDING_ACTION_DD.with(|c| c.set(chip));
}

pub(crate) fn take_pending_bone_action_dd() -> Option<Rect> {
    PENDING_ACTION_DD.with(Cell::take)
}

/// ⭐⭐⭐ **O selector de PONTA do osso em foco** (shell → painel, todo quadro).
///
/// `None` ⇒ não há pergunta: não há osso em foco, ou as alças dele são autoradas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TipView {
    /// Um rótulo por linha, na ordem em que a lista as pinta.
    pub rotulos: Vec<String>,
    /// A linha ligada.
    pub ligado: usize,
    /// Quantos filhos-osso não couberam no pool de ids. `0` no caso normal.
    pub escondidos: usize,
}

/// **O selector de ponta** (shell → painel, todo quadro em que a seção vive).
pub fn set_current_bone_tip(v: Option<TipView>) {
    CURRENT_TIP.with(|c| *c.borrow_mut() = v);
}

pub(crate) fn bone_tip() -> Option<TipView> {
    CURRENT_TIP.with(|c| c.borrow().clone())
}

pub(crate) fn set_pending_bone_tip_dd(chip: Option<Rect>) {
    PENDING_TIP_DD.with(|c| c.set(chip));
}

pub(crate) fn take_pending_bone_tip_dd() -> Option<Rect> {
    PENDING_TIP_DD.with(Cell::take)
}

// ⛔⛔ **A ROLAGEM DE «REVELAR-AO-FOCAR» SAIU DAQUI, e é o painel próprio que a dissolveu**
// (2026-09-09). Ela existia porque a secção caía em `y = 1394 px` sobre uma faixa visível de `774`,
// com 785 px de secções de OUTRO assunto por cima. Neste painel o cabeçalho é a **primeira** linha
// e o corpo inteiro mede ~620 px numa coluna de 836 — *não há dobra abaixo da qual esconder-se*.
//
// ⚠️ **A LEI da aresta sobreviveu, e mudou de efeito:** um osso NOVO em foco traz a **ABA** deste
// painel para a frente (`bump_panel_z`, na shell), que é o que «revelar» quer dizer quando ele
// divide um encaixe com o Inspector. *A pergunta era boa; o que ela comandava é que era do desenho
// antigo.*

thread_local! {
    /// ⭐ **O VERBO do arrasto que está armado** — o ÍNDICE em `ph2d_tool_vector::BoneAction::ALL`.
    /// `None` ⇒ a ferramenta Osso não está na mão, e a fileira *Criar × Transformar* não tem sujeito.
    ///
    /// ⚠️ **Um ÍNDICE e não o enum**, pelo mesmo motivo do lado da dobra: é o que mantém este painel
    /// sem depender da crate da ferramenta de vector — e quem alinha as duas listas é a POSIÇÃO.
    static BONE_TOOL: Cell<Option<usize>> = const { Cell::new(None) };
}

/// **O verbo do arrasto** (shell → painel). `None` fora da ferramenta Osso.
pub fn set_current_bone_tool(v: Option<usize>) {
    BONE_TOOL.with(|c| c.set(v));
}

thread_local! {
    /// ⭐⭐⭐ **OS DOIS NÚMEROS DO PINCEL DE PESO** (raio · quanto), publicados pela shell a partir
    /// do `VectorDrawConfig`.
    ///
    /// ⚠️ **O sujeito deles é o PINCEL e não o osso**, e é por isso que não viajam no `BoneSpec`:
    /// eles valem antes de haver osso nenhum em foco, e um campo sem sujeito é a classe de
    /// controlo morto que o `CLAUDE.md` §5.0 nomeia — *ao contrário*.
    static BONE_WEIGHT: Cell<(f64, f64)> = const { Cell::new((0.0, 0.0)) };
}

/// **Os dois números do pincel de peso** (shell → painel, todo quadro).
pub fn set_current_bone_weight(raio: f64, quanto: f64) {
    BONE_WEIGHT.with(|c| c.set((raio, quanto)));
}

pub(crate) fn bone_weight() -> (f64, f64) {
    BONE_WEIGHT.with(Cell::get)
}

pub(crate) fn bone_tool() -> Option<usize> {
    BONE_TOOL.with(Cell::get)
}

/// A âncora do osso em foco e os três números dela (`mix`, `softness`, `chain`). `None` ⇒ ele não
/// tem uma, e o painel oferece a porta de entrada.
/// ⭐⭐⭐ **QUE LADO O `Auto` ESTÁ A DERIVAR** (report do dono, 2026-09-18: *«IK Bend não funcionou
/// com Auto IK e trocando CCw por CW»*).
///
/// ⛔⛔⛔ **MEDIDO: com o `Chain` de fábrica (`2`) o `Auto` e o `Cw` são a MESMA pose, ao bit**
/// (distância `0,0000` entre as três rotações; com `Chain = 3` ela é `2,9991`). A cena do osso
/// captura `Cw`, logo o artista clica em **dois** dos quatro chips e não vê nada mudar — e isso é
/// indistinguível de um controlo partido. *Só o `Ccw` move (`3,0000`).*
///
/// ⚠️ **A cura não é esconder o `Auto`:** ele significa *«deriva o lado da pose que chega»*, e
/// coincide **nesta** pose, não sempre. ⇒ o chip **diz** qual lado está a derivar, e o artista vê,
/// sem clicar, que pedir esse lado não vai mudar nada.
///
/// `None` ⇒ não se sabe (sem âncora, ou a corrente não tem lado), e o chip fica com o rótulo nu.
pub fn set_current_bone_ik_auto_side(v: Option<usize>) {
    CURRENT_IK_AUTO_SIDE.with(|c| c.set(v));
}

pub(crate) fn current_bone_ik_auto_side() -> Option<usize> {
    CURRENT_IK_AUTO_SIDE.with(Cell::get)
}

/// ⭐⭐⭐ **A âncora em foco APONTA?** — `Some(desvio em GRAUS)` ou `None` (ela alcança, ou não
/// existe). Publicado pela shell a partir da porta [`ph2d_skeleton_live::goal::aponta`].
///
/// ⚠️ **GRAUS aqui e radianos no documento**, a mesma lei do limite da junta duas fileiras abaixo:
/// o documento guarda o arco no espaço do `Transform::rotation` e o artista pensa em graus. A
/// conversão vive na SHELL, num sítio só.
pub fn set_current_bone_aim(v: Option<f64>) {
    CURRENT_IK_AIM.with(|c| c.set(v));
}

pub(crate) fn current_bone_aim() -> Option<f64> {
    CURRENT_IK_AIM.with(Cell::get)
}

pub fn set_current_bone_ik(v: Option<(f64, f64, f64, ph2d_skeleton::BendSide)>) {
    CURRENT_HAS_IK.with(|c| c.set(v.is_some()));
    if let Some((mix, softness, chain, bend)) = v {
        CURRENT_IK_MIX.with(|c| c.set(mix));
        CURRENT_IK_SOFTNESS.with(|c| c.set(softness));
        CURRENT_IK_CHAIN.with(|c| c.set(chain));
        // ⚠️ A posição na lista da LEI, nunca um número escrito aqui: uma variante nova acende o
        // segmento certo sem ninguém se lembrar deste ficheiro.
        let i = ph2d_skeleton::BendSide::ALL
            .iter()
            .position(|s| *s == bend)
            .unwrap_or(0);
        CURRENT_IK_BEND.with(|c| c.set(i));
    }
}

/// O limite da junta em foco, em GRAUS (`min`, `max`). `None` ⇒ ela não tem um.
///
/// ⚠️ **Graus e não radianos**, e a conversão fica na SHELL: o documento guarda o arco em radianos
/// (o mesmo espaço do `Transform::rotation`) e o artista pensa em graus. Duas unidades num campo
/// só é como um número passa a significar outra coisa sem ninguém dar por isso.
pub fn set_current_bone_limit(v: Option<(f64, f64)>) {
    CURRENT_LIMIT.with(|c| c.set(v));
}

pub(crate) fn current_bone_limit() -> Option<(f64, f64)> {
    CURRENT_LIMIT.with(Cell::get)
}

/// ⭐⭐⭐ **O OSSO INTELIGENTE em foco** (shell → painel) — a faixa, a acção e o alvo, de uma vez.
///
/// ⚠️ **Graus e não radianos**, pela mesma razão do limite: o documento guarda o ângulo no espaço
/// do `Transform` e o artista pensa em graus. A conversão vive na SHELL.
#[derive(Clone, Debug, PartialEq)]
pub struct SmartBoneView {
    /// O ângulo (GRAUS) em que a acção está no princípio.
    pub from: f64,
    /// ... e no fim.
    pub to: f64,
    /// O NOME da acção ligada. Vazio ⇒ nenhuma.
    pub clip: String,
    /// O NOME do objecto de que este controlo trata. Vazio ⇒ nenhum escolhido.
    pub target: String,
    /// O *Pick Object* está ARMADO — o clique seguinte escolhe o alvo.
    ///
    /// ⚠️ Ele muda o RÓTULO do botão, e é essa a diferença entre um gesto modal que se percebe e um
    /// clique que parece não ter feito nada.
    pub picking: bool,
}

/// Publica o osso inteligente em foco. `None` ⇒ ele não é um controlo.
pub fn set_current_bone_smart(v: Option<SmartBoneView>) {
    CURRENT_SMART.with(|c| *c.borrow_mut() = v);
}

pub(crate) fn current_bone_smart() -> Option<SmartBoneView> {
    CURRENT_SMART.with(|c| c.borrow().clone())
}

pub(crate) fn current_bone_ik() -> Option<(f64, f64, f64, usize)> {
    CURRENT_HAS_IK.with(Cell::get).then(|| {
        (
            CURRENT_IK_MIX.with(Cell::get),
            CURRENT_IK_SOFTNESS.with(Cell::get),
            CURRENT_IK_CHAIN.with(Cell::get),
            CURRENT_IK_BEND.with(Cell::get),
        )
    })
}

/// **O estado RETIDO do painel** — vazio, e a ausência é a decisão.
///
/// ⚠️ Tudo o que este painel mostra é **publicado pela shell por quadro** (os statics acima): o
/// osso em foco, a âncora, o limite, o controlo e a lista de acções. Guardar aqui uma cópia daria
/// uma segunda resposta a *«qual osso está aceso?»*, e as duas divergiriam no primeiro clique.
#[derive(Default)]
pub struct SkeletonPanelState;
