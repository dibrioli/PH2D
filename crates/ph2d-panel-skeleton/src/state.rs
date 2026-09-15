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
    static CURRENT_SKINNED: Cell<bool> = const { Cell::new(false) };
    /// O OSSO em foco existe? Sem ele, `Length`/`Strength` não têm sujeito.
    static CURRENT_HAS_BONE: Cell<bool> = const { Cell::new(false) };
    static CURRENT_BONE_LENGTH: Cell<f64> = const { Cell::new(0.0) };
    static CURRENT_BONE_STRENGTH: Cell<f64> = const { Cell::new(1.0) };
    /// ⭐ Os SEGMENTOS e a CURVATURA do osso em foco (a F8). ⚠️ Eles decidem se as quatro fileiras
    /// da curvatura são pintadas — ver [`crate::section::campos_do_osso`].
    static CURRENT_BONE_SEGMENTS: Cell<u8> = const { Cell::new(1) };
    static CURRENT_BONE_CURVE: Cell<ph2d_skeleton::bend::Bend> =
        const { Cell::new(ph2d_skeleton::bend::Bend::STRAIGHT) };
}

/// A seleção tem forma presa a esqueleto (publicado pela shell, todo quadro).
pub fn set_current_skinned(v: bool) {
    CURRENT_SKINNED.with(|c| c.set(v));
}

pub(crate) fn skinned() -> bool {
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

pub(crate) fn bone_tool() -> Option<usize> {
    BONE_TOOL.with(Cell::get)
}

thread_local! {
    /// ⭐⭐⭐ **A CENA TEM UMA IMAGEM PRESA?** — o sujeito da fileira *Deform*.
    ///
    /// ⛔⛔ **Ela NÃO é a [`skinned`], e a distinção foi achada antes do smoke:** aquela pergunta é
    /// *«a SELECÇÃO é uma forma presa?»*, e a resposta dela varre **caminhos vectoriais** — uma
    /// imagem presa é uma *sprite*, logo nunca lá aparece. Ligar a fileira àquela pergunta
    /// deixá-la-ia **viva e inalcançável**: pintada em código, nunca na tela.
    ///
    /// ⚠️ **E a pergunta certa é sobre a CENA, não sobre a selecção**, porque a escolha é GLOBAL:
    /// ela vive na ferramenta e vale para toda imagem presa. Uma fileira que só aparecesse com a
    /// imagem escolhida prometeria uma propriedade por-objecto que não existe.
    static HAS_SKINNED_IMAGE: Cell<bool> = const { Cell::new(false) };

    /// ⭐ **COMO a pele é desenhada** — o ÍNDICE em `ph2d_tool_vector::SkinDeform::ALL`.
    ///
    /// ⚠️ **Um ÍNDICE e não o enum**, pela MESMA razão do verbo do arrasto: é o que mantém este
    /// painel sem depender da crate da ferramenta de vector, e quem alinha as duas listas é a
    /// POSIÇÃO. ⛔ Sem `Option`: ao contrário do verbo, esta pergunta tem **sempre** resposta —
    /// alguma coisa está a desenhar a pele, e nenhum segmento aceso mentiria.
    static SKIN_DEFORM: Cell<usize> = const { Cell::new(0) };
}

/// **A cena tem alguma imagem presa ao esqueleto?** (shell → painel).
pub fn set_current_skinned_image(v: bool) {
    HAS_SKINNED_IMAGE.with(|c| c.set(v));
}

pub(crate) fn skinned_image() -> bool {
    HAS_SKINNED_IMAGE.with(Cell::get)
}

/// **Como a pele é desenhada** (shell → painel), o report das arestas retas de 2026-09-10.
pub fn set_current_skin_deform(v: usize) {
    SKIN_DEFORM.with(|c| c.set(v));
}

pub(crate) fn skin_deform() -> usize {
    SKIN_DEFORM.with(Cell::get)
}

/// A âncora do osso em foco e os três números dela (`mix`, `softness`, `chain`). `None` ⇒ ele não
/// tem uma, e o painel oferece a porta de entrada.
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
