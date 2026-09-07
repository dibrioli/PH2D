//! O estado do **ESQUELETO** (estudo 42 item 5) publicado pela shell — irmão de `state.rs` pelo
//! teto de 600 LOC daquele arquivo, e coeso pelo mesmo critério do `state_envelope`: a família
//! inteira de uma feature, com os seus statics ao lado dos seus acessores.
//!
//! ⚠️ **O painel não vê o `ph2d-ecs`** (a UI vive de snapshots publicados, nunca do mundo), então o
//! que atravessa são NÚMEROS e não componentes.

use std::cell::Cell;

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
}

/// A âncora do osso em foco e os três números dela (`mix`, `softness`, `chain`). `None` ⇒ ele não
/// tem uma, e o painel oferece a porta de entrada.
pub fn set_current_bone_ik(v: Option<(f64, f64, f64)>) {
    CURRENT_HAS_IK.with(|c| c.set(v.is_some()));
    if let Some((mix, softness, chain)) = v {
        CURRENT_IK_MIX.with(|c| c.set(mix));
        CURRENT_IK_SOFTNESS.with(|c| c.set(softness));
        CURRENT_IK_CHAIN.with(|c| c.set(chain));
    }
}

pub(crate) fn current_bone_ik() -> Option<(f64, f64, f64)> {
    CURRENT_HAS_IK.with(Cell::get).then(|| {
        (
            CURRENT_IK_MIX.with(Cell::get),
            CURRENT_IK_SOFTNESS.with(Cell::get),
            CURRENT_IK_CHAIN.with(Cell::get),
        )
    })
}
