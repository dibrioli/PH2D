//! O estado do **ESQUELETO** (estudo 42 item 5) publicado pela shell — irmão de `state.rs` pelo
//! teto de 600 LOC daquele arquivo, e coeso pelo mesmo critério do `state_envelope`: a família
//! inteira de uma feature, com os seus statics ao lado dos seus acessores.
//!
//! ⚠️ **O painel não vê o `ph2d-ecs`** (a UI vive de snapshots publicados, nunca do mundo), então o
//! que atravessa são NÚMEROS e não componentes.

use ph2d_editor_core::zones::Rect;
use std::cell::{Cell, RefCell};

thread_local! {
    /// A seleção contém pelo menos uma forma PRESA a um esqueleto? Decide se as duas saídas
    /// (Keep Pose / Release) são oferecidas — *um botão que só sabe recusar é pior que um ausente*.
    static CURRENT_SKINNED: Cell<bool> = const { Cell::new(false) };
    /// A CENA tem esqueleto? É o que torna a seção útil fora do modo Osso — e, sem ele, ela só
    /// aparece na ferramenta que faz ossos. *Uma seção que fala de algo que não existe é ruído.*
    static CURRENT_HAS_SKELETON: Cell<bool> = const { Cell::new(false) };
    /// O OSSO em foco existe? Sem ele, `Length`/`Strength` não têm sujeito.
    static CURRENT_HAS_BONE: Cell<bool> = const { Cell::new(false) };
    static CURRENT_BONE_LENGTH: Cell<f64> = const { Cell::new(0.0) };
    static CURRENT_BONE_STRENGTH: Cell<f64> = const { Cell::new(1.0) };
}

/// A seleção tem forma presa a esqueleto (publicado pela shell, todo quadro).
pub fn set_current_skinned(v: bool) {
    CURRENT_SKINNED.with(|c| c.set(v));
}

pub(crate) fn skinned() -> bool {
    CURRENT_SKINNED.with(Cell::get)
}

/// A cena tem pelo menos um osso (publicado pela shell, todo quadro).
pub fn set_current_has_skeleton(v: bool) {
    CURRENT_HAS_SKELETON.with(|c| c.set(v));
}

pub(crate) fn has_skeleton() -> bool {
    CURRENT_HAS_SKELETON.with(Cell::get)
}

/// O osso em foco e os dois números dele. `None` ⇒ a seleção não é um osso.
pub fn set_current_bone(v: Option<(f64, f64)>) {
    CURRENT_HAS_BONE.with(|c| c.set(v.is_some()));
    if let Some((length, strength)) = v {
        CURRENT_BONE_LENGTH.with(|c| c.set(length));
        CURRENT_BONE_STRENGTH.with(|c| c.set(strength));
    }
}

pub(crate) fn current_bone() -> Option<(f64, f64)> {
    CURRENT_HAS_BONE.with(Cell::get).then(|| {
        (
            CURRENT_BONE_LENGTH.with(Cell::get),
            CURRENT_BONE_STRENGTH.with(Cell::get),
        )
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
