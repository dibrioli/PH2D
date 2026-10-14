//! **O instantâneo e o dreno da secção MACRO** (ADR-0186): o que a gravação de um objecto é, e o
//! modo em que a trilha do Motion toca.
//!
//! A ponte `u8` ⇄ `ModoDaMacro` vive AQUI e só aqui (o painel não vê a `ph2d-ecs`), e a gravação é
//! lida pela porta única do formato (`ph2d_macro::codec::read`).

use ph2d_ecs::{Macro, ModoDaMacro, SimWorld};
use ph2d_editor_core::macro_edits::{
    InspectorMacroInfo, MACRO_DE_VERSAO_NOVA, MACRO_ESTRAGADA, MACRO_MODOS, MacroFieldEdit as E,
    MacroGravacao,
};

const MODOS: [ModoDaMacro; MACRO_MODOS] = [ModoDaMacro::Fiel, ModoDaMacro::Explicativo];

/// O índice que atravessa a fronteira ⇄ o modo. Fora da faixa ⇒ `None` (edição inerte).
#[must_use]
pub fn modo_de_u8(v: u8) -> Option<ModoDaMacro> {
    MODOS.get(usize::from(v)).copied()
}

#[must_use]
pub fn u8_do_modo(m: ModoDaMacro) -> u8 {
    match m {
        ModoDaMacro::Fiel => 0,
        ModoDaMacro::Explicativo => 1,
    }
}

/// A gravação que os bytes do componente guardam.
#[must_use]
pub fn gravacao(bytes: &[u8]) -> MacroGravacao {
    if bytes.is_empty() {
        return MacroGravacao::Vazia;
    }
    match ph2d_macro::codec::read(bytes) {
        Ok(doc) => MacroGravacao::Lida {
            trilhas: doc
                .tracks
                .iter()
                .map(|t| (t.family.clone(), t.events.len()))
                .collect(),
            duracao_us: doc.duration().0,
        },
        Err(ph2d_macro::CodecError::MajorMismatch { .. }) => {
            MacroGravacao::Ilegivel(MACRO_DE_VERSAO_NOVA)
        }
        Err(_) => MacroGravacao::Ilegivel(MACRO_ESTRAGADA),
    }
}

/// ⭐ **Constrói o instantâneo** da entidade escolhida, ou `None` se ela não tem o componente.
#[must_use]
pub fn build_info(sim: &SimWorld, bits: u64, selected_count: usize) -> Option<InspectorMacroInfo> {
    let m = sim
        .world()
        .get::<Macro>(ph2d_ecs::Entity::from_bits(bits))?;
    Some(InspectorMacroInfo {
        entity_bits: bits,
        modo: u8_do_modo(m.modo),
        gravacao: gravacao(&m.doc),
        selected_count,
    })
}

/// Aplica uma edição. `true` = o mundo mudou (escrever o MESMO valor não é mudança: o undo é por
/// diff).
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let Some(mut m) = sim.world_mut().get_mut::<Macro>(e) else {
        return false;
    };
    match edit {
        E::Modo(i) => {
            let Some(modo) = modo_de_u8(*i) else {
                return false;
            };
            if m.modo == modo {
                return false;
            }
            m.modo = modo;
        }
    }
    true
}

/// Aplica todas as edições do quadro. `true` = alguma coisa mudou (`|`: todas correm).
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    edits
        .iter()
        .fold(false, |mudou, (bits, edit)| apply(sim, *bits, edit) | mudou)
}

#[cfg(test)]
#[path = "macro_inspector_tests.rs"]
mod tests;
