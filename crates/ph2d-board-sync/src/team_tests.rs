//! A casca de um fio estacionado (`TeamState::parked`) pela porta de quem recebe.

use super::*;


fn at(n: u64) -> Lamport {
    Lamport {
        n,
        ..Lamport::default()
    }
}

/// ⭐ O fio esvaziado (a última mensagem apagada) estaciona; o «resolvido» e a nova posição que chegam
/// ENTRETANTO entram na casca (nada fica pendente) e voltam com a mensagem seguinte.
#[test]
fn a_parked_thread_takes_its_pin_and_resolved_while_it_waits() {
    let mut team = Team::default();
    let t = team.open(Pin::Point([0.0; 2]), "Rui", "hi", 1).unwrap();
    let m0 = team.thread(t).unwrap().messages[0].id;
    let mut s = TeamState::default();
    s.receive(&mut team, &[(at(1), TeamWrite::MessageGone(t, m0))]);
    assert!(team.thread(t).is_none(), "o fio vazio estacionou");
    let moved = Pin::Point([40.0, 50.0]);
    s.receive(
        &mut team,
        &[(at(2), TeamWrite::Resolved(t, true)), (at(3), TeamWrite::Pin(t, moved))],
    );
    assert_eq!(s.pending(), 0, "a casca recebeu as duas escritas");
    let mut other = Team::default();
    let back = other.open(Pin::Point([0.0; 2]), "Ana", "again", 4).unwrap();
    let mut m = other.thread(back).unwrap().messages[0].clone();
    m.id = MessageId(m0.0 + 1);
    s.receive(&mut team, &[(at(4), TeamWrite::Message(t, m))]);
    let th = team.thread(t).expect("voltou com a mensagem");
    assert!(th.resolved);
    assert_eq!(th.pin, moved);
}
