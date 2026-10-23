//! W24 B14 — a menção entre réplicas e no ficheiro (módulo filho de `replica_tests`).

use super::*;

/// ⭐ W24 B14 — a menção chega AO VIVO como pessoa (não letras) e fica NO FICHEIRO: a Ana chama a Bia,
/// a réplica da Bia recebe o fio para ela (o selo e o aviso leem `for_me`), e quem abre o ficheiro
/// gravado depois vê o mesmo — por ler até a Bia o abrir; a resposta dela é para a Ana (o «Feed»).
#[test]
fn a_mention_arrives_live_and_stays_in_the_file() {
    use ph2d_board_model::Who;
    let (mut a, mut b, _) = two();
    let t =
        a.b.team
            .open_calling(
                Pin::Point([1.0, 2.0]),
                "Ana",
                "@Bia look",
                &[Who::Person("Bia".into())],
                10,
            )
            .unwrap();
    let wa = a.r.local(&mut a.b);
    b.r.receive(&mut b.b, &wa);
    let th = b.b.team.thread(t).expect("o fio chegou");
    assert_eq!(th.messages[0].mentions, vec![Who::Person("Bia".into())]);
    assert!(
        th.for_me("Bia") && th.unread_for("Bia"),
        "para a Bia, por ler"
    );
    assert!(
        !th.for_me("Rui") && !th.for_me("Ana"),
        "não é do Rui nem da autora"
    );
    let reopen = |p: &Peer| {
        let mut set = BoardSet::default();
        let id = set.create("Quadro".into());
        set.get_mut(id).unwrap().team = p.b.team.clone();
        BoardSet::from_bytes(&set.to_bytes().unwrap())
            .unwrap()
            .boards()[0]
            .team
            .clone()
    };
    let file = reopen(&b);
    let th = file.thread(t).unwrap();
    assert_eq!(th.messages[0].mentions, vec![Who::Person("Bia".into())]);
    assert!(
        th.for_me("Bia") && th.unread_for("Bia"),
        "no ficheiro, ainda por ler"
    );
    b.b.team.mark_read(t, "Bia");
    assert!(b.b.team.reply_calling(t, "Bia", "ok", &[], 11));
    let wb = b.r.local(&mut b.b);
    a.r.receive(&mut a.b, &wb);
    let th = a.b.team.thread(t).unwrap();
    assert!(
        th.for_me_at(1, "Ana"),
        "a resposta no fio dela é para a Ana"
    );
    assert!(
        !reopen(&b).thread(t).unwrap().unread_for("Bia"),
        "lido fica lido"
    );
    same(&a, &b, "a equipa convergiu");
}
