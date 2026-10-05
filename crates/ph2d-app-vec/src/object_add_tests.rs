use super::*;

/// ⭐⭐ GATE (escolha do dono, 05/10) — **a entrada NÃO cria nada: arma a ferramenta** — nenhuma
/// entidade nasce (a Hierarquia e o canvas ficam como estavam), e o Edit espera a 1.ª forma.
///
/// (Mutação: tirar o `vec.edit.arm()` ⇒ RED — a ferramenta nunca viria à mão.)
#[test]
fn the_entry_arms_the_tool_and_creates_nothing() {
    let mut vec = crate::state::VecState::default();
    assert!(!vec.edit.armed());
    assert!(add(VECTOR, &mut vec), "é desta família");
    assert!(vec.edit.armed(), "a ferramenta não ficou pedida");
    assert!(vec.entities.is_empty(), "nasceu uma forma");
}

/// **UMA entrada**, e uma de outra família não é desta — nada se arma.
#[test]
fn one_entry_and_the_entry_of_another_family_is_not_ours() {
    assert_eq!(ENTRIES, &[VECTOR]);
    let mut vec = crate::state::VecState::default();
    assert!(!add(ph2d_editor_core::object_add::EMPTY, &mut vec));
    assert!(!vec.edit.armed());
}
