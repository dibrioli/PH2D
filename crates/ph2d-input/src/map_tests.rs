//! Gates do MAPA — e todos eles fazem **a mesma pergunta**: *o que acontece à gravação de ontem?*
//!
//! ⛔ É a LEI Nº 1 do plano, vista do lado do id: a fita grava a **acção resolvida**, então tudo o
//! que mexa no significado de um [`ActionId`] reescreve o passado em silêncio.

use super::*;
use crate::action::{Binding, InputAction};
use crate::keyboard::Key;

/// ⭐ **REORDENAR o painel não pode mexer nos ids.** É o que um índice na lista não conseguiria
/// prometer — e um índice é o candidato mais tentador para quem escreve este tipo em cinco minutos.
#[test]
fn reordering_the_panel_never_changes_an_id() {
    let mut m = InputMap::new();
    let jump = m.create("jump");
    let fire = m.create("fire");

    // O painel apaga e recria a acção do meio noutra posição -- o gesto de "mover para baixo".
    let removed = m.remove(jump).expect("jump existia");
    m.insert(removed);

    assert_eq!(m.id("jump"), Some(jump), "o id de `jump` sobreviveu a' mudanca de posicao");
    assert_eq!(m.id("fire"), Some(fire));
    assert_ne!(jump, fire, "e os dois continuam distintos");
}

/// ⭐ **RENOMEAR também não.** É o que um hash do nome não conseguiria prometer — e o hash é o
/// segundo candidato tentador, porque este repo usa exactamente esse truque noutro domínio (o nome
/// estável de uma entidade). Aqui ele estaria errado, e a diferença é a fita.
#[test]
fn renaming_an_action_never_changes_its_id() {
    let mut m = InputMap::new();
    let id = m.create("jump");
    m.get_mut(id).expect("existe").name = "pular".to_string();

    assert_eq!(m.id("pular"), Some(id), "o id seguiu o nome novo");
    assert_eq!(m.id("jump"), None, "e o nome velho deixou de responder");
}

/// ⛔ **Um id apagado nunca volta a ser atribuído.** Se voltasse, uma fita gravada antes da remoção
/// passaria a accionar a acção seguinte que alguém criasse — um defeito que só aparece semanas
/// depois, na gravação de outra pessoa.
#[test]
fn a_removed_id_is_never_handed_out_again() {
    let mut m = InputMap::new();
    let a = m.create("a");
    let b = m.create("b");
    m.remove(a);
    m.remove(b);
    let c = m.create("c");

    assert_ne!(c, a, "o id de `a` foi reutilizado");
    assert_ne!(c, b, "o id de `b` foi reutilizado");
}

/// ⚠️ **A armadilha clássica do contador**: um mapa que chega pronto (um ficheiro, uma fixtura) tem
/// de fazer o contador **subir**. Sem isto, a próxima acção criada nasce com um id **já em uso**, e
/// as duas passam a ser a mesma coisa para a fita.
#[test]
fn adopting_a_prebuilt_action_pushes_the_counter_past_it() {
    let mut m = InputMap::new();
    m.insert(InputAction::new(ActionId(41), "from_disk"));
    let fresh = m.create("fresh");

    assert!(
        fresh.0 > 41,
        "o contador ficou em {} e vai colidir com o id 41 que veio do disco",
        fresh.0
    );
}

/// Um nome repetido devolve a acção que já existe — duas linhas com o mesmo nome tornariam
/// [`InputMap::id`] uma pergunta sem resposta única.
#[test]
fn creating_the_same_name_twice_returns_the_same_action() {
    let mut m = InputMap::new();
    let first = m.create("jump");
    let again = m.create("jump");

    assert_eq!(first, again);
    assert_eq!(m.len(), 1, "e nao nasceu uma segunda linha no painel");
}

/// `insert` sobre um id que já existe **substitui**, e não duplica — é o caminho de um mapa
/// recarregado por cima de outro.
#[test]
fn inserting_over_an_existing_id_replaces_it() {
    let mut m = InputMap::new();
    let id = m.create("jump");
    let mut edited = InputAction::new(id, "jump");
    edited.bindings.push(Binding::Key(Key(0x20)));
    m.insert(edited);

    assert_eq!(m.len(), 1);
    assert_eq!(m.get(id).expect("existe").bindings.len(), 1);
}

/// A ordem que o painel mostra é a ordem em que o autor criou — e é estável, porque é uma `Vec`.
#[test]
fn the_panel_order_is_the_authoring_order() {
    let mut m = InputMap::new();
    for n in ["move_left", "move_right", "jump"] {
        m.create(n);
    }
    let names: Vec<&str> = m.actions().iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["move_left", "move_right", "jump"]);
}
