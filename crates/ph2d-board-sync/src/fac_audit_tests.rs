//! W12 — os gates da auditoria de 2 lentes (rede e interface): cada um nasceu de um caminho concreto
//! que vazava, perdia ou divergia. Módulo filho de `fac_tests` (o harness `Room` é de lá).

use super::*;

/// ⛔ Auditoria W12 (rede, F1): uma VERSÃO guardada com o modo ligado levava o quadro inteiro da autora
/// — comprimido, e por isso o gate de fuga (bytes crus) não a via. Leva o quadro como todos o vêem.
#[test]
fn a_version_saved_in_private_mode_does_not_carry_the_sealed_text() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "segredo na versão", 0.0);
    r.at(1);
    r.b(0).save_version("v".into(), 0);
    r.at(2);
    let v = r.b(1).team.versions[0].read().unwrap();
    assert!(
        !has(&shared(&v), "segredo na versão"),
        "a versão levou o selado"
    );
    let own = r.b(0).team.versions[0].read().unwrap();
    assert_eq!(
        text(&own, id),
        "",
        "a da autora é a mesma (a versão é do quadro da sala)"
    );
    let c = r.join("Dani");
    let v = r.b(c).team.versions[0].read().unwrap();
    assert!(!has(&shared(&v), "segredo na versão"));
}

/// ⛔ Auditoria W12 (rede, F2): a nota CRESCIA para os outros enquanto a autora escrevia (o comprimento
/// do texto). A caixa da minha nota escondida sai com a posição nova e o tamanho de antes; o tamanho
/// verdadeiro chega ao revelar.
#[test]
fn a_hidden_note_does_not_grow_for_the_others_but_still_moves() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "uma linha", 0.0);
    r.at(1);
    let mut el = r.b(0).doc.get(id).unwrap().clone();
    el.shape_mut().unwrap().text = RichText::plain("muitas\nlinhas\nde\nsegredo");
    el.h = 40.0;
    let _ = BoardOp::Put(el).apply(&mut r.b(0).doc);
    r.at(2);
    let seen = r.b(1).doc.get(id).unwrap().clone();
    assert_eq!(seen.h, 10.0, "a nota cresceu para o Caio");
    let mut el = r.b(0).doc.get(id).unwrap().clone();
    el.translate([30.0, 0.0]);
    let _ = BoardOp::Put(el).apply(&mut r.b(0).doc);
    r.at(3);
    let seen = r.b(1).doc.get(id).unwrap().clone();
    assert_eq!((seen.x, seen.h), (30.0, 10.0), "mexeu-se, sem crescer");
    let c = r.join("Dani");
    assert_eq!(r.b(c).doc.get(id).unwrap().h, 10.0, "o retrato também");
    r.ask(0, Ask::PrivateOff);
    r.at(3 + REVEAL_COUNTDOWN_MS);
    assert_eq!(r.b(1).doc.get(id).unwrap().h, 40.0);
    r.all_same();
}

/// ⛔ Auditoria W12 (rede, F3; interface, F1): quem NÃO lê uma nota escondida não lhe escreve o texto
/// por caminho nenhum (o desfazer, o negrito — aqui uma escrita directa no documento): a escrita
/// desfaz-se ao ser capturada, nunca vai ao cofre, e o texto da autora sai ao revelar.
#[test]
fn someone_who_cannot_read_a_note_cannot_overwrite_its_text() {
    let mut hb = board();
    let id = note(&mut hb, "", 0.0);
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    write(r.b(1), id, "do Caio");
    r.at(1);
    r.ask(0, private_on());
    write(r.b(0), id, "segredo da Bia");
    r.at(2);
    write(r.b(1), id, "");
    r.at(3);
    assert_eq!(text(r.b(1), id), "do Caio", "a escrita do Caio desfez-se");
    write(&mut r.hb, id, "da Ana");
    r.at(4);
    assert_eq!(text(&r.hb, id), "do Caio", "e a da anfitriã também");
    r.ask(0, Ask::PrivateOff);
    r.at(4 + REVEAL_COUNTDOWN_MS);
    assert_eq!(text(&r.hb, id), "segredo da Bia");
    r.all_same();
}

/// ⛔ Auditoria W12 (rede, F4): um voto dado ANTES de a nota se esconder não congela o boletim — fica,
/// pode sair, e os votos seguintes contam; só um voto NOVO nela é recusado (e quem votou sabe).
#[test]
fn a_vote_cast_before_a_note_was_hidden_does_not_freeze_the_ballot() {
    let mut hb = board();
    let (n, m) = (note(&mut hb, "n", 0.0), note(&mut hb, "m", 20.0));
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    r.ask(
        0,
        Ask::VoteStart {
            setup: VoteSetup {
                votes: 3,
                one_per_object: false,
                minutes: 0,
            },
            targets: BTreeSet::from([n, m]),
        },
    );
    r.vote(1, ballot(1, &[(n, 1)], false));
    r.ask(0, private_on());
    write(r.b(0), n, "agora privada");
    r.at(1);
    r.vote(1, ballot(1, &[(n, 1), (m, 1)], true));
    assert!(
        r.fac(0).vote.unwrap().done.contains(&11),
        "o boletim congelou"
    );
    r.vote(1, ballot(1, &[(n, 2), (m, 1)], true));
    assert!(
        r.peers[1]
            .events
            .contains(&Event::Denied(Refusal::BallotRefused))
    );
    assert_eq!(r.fac(0).vote.unwrap().used.get(&11), Some(&2));
    r.vote(1, ballot(1, &[(m, 1)], true));
    assert_eq!(
        r.fac(0).vote.unwrap().used.get(&11),
        Some(&1),
        "o voto antigo sai"
    );
}

/// ⛔ Auditoria W12 (rede, F5): um texto às claras (de quem ainda não sabia do modo) que passa à frente
/// do selado da anfitriã: quem entra depois converge, e depois de revelar todos têm o mesmo.
#[test]
fn a_newcomer_converges_when_a_clear_text_overtakes_the_hosts_sealed_one() {
    let mut hb = board();
    let id = note(&mut hb, "", 0.0);
    let mut r = Room::new(hb, &["Bia"]);
    if let Some(e) = r.host.ask(&mut r.hb, private_on(), 0) {
        panic!("{e:?}");
    }
    write(&mut r.hb, id, "da Ana");
    let out = r.host.tick(&mut r.hb, 0);
    let p = &mut r.peers[0];
    let b = p.b.as_mut().unwrap();
    write(b, id, "da Bia às claras");
    let up = p.g.tick(b, 0);
    p.up.extend(up.into_iter().map(|m| m.encode()));
    r.route(out);
    r.pump();
    let c = r.join("Caio");
    r.all_same();
    assert!(!r.heard(c, "da Ana"));
    r.host_ask(Ask::PrivateOff);
    r.at(REVEAL_COUNTDOWN_MS);
    r.all_same();
}

/// ⛔ Auditoria W12 (rede, F7): um boletim cuja soma não cabe (dava a volta e passava) é recusado.
#[test]
fn a_ballot_whose_sum_overflows_is_refused() {
    let mut hb = board();
    let (a, b) = (note(&mut hb, "a", 0.0), note(&mut hb, "b", 20.0));
    let mut r = Room::new(hb, &["Bia"]);
    r.ask(
        0,
        Ask::VoteStart {
            setup: VoteSetup::default(),
            targets: BTreeSet::from([a, b]),
        },
    );
    r.vote(0, ballot(1, &[(a, u32::MAX), (b, 2)], false));
    assert_eq!(r.host.fac(0, &r.hb).vote.unwrap().used.get(&10), None);
    assert!(
        r.peers[0]
            .events
            .contains(&Event::Denied(Refusal::BallotRefused))
    );
}

/// ⛔ Prova de mutação W12 (M13): a corrida da virada — a Bia ainda não sabe que o modo ligou e o texto
/// dela sai às claras; o anfitrião, que sabe, põe-no no cofre (o filtro do servidor, como o Scrumlr):
/// o Caio não o ouve até revelar, e depois todos o têm.
#[test]
fn a_clear_text_written_before_hearing_of_the_mode_goes_to_the_vault() {
    let mut hb = board();
    let id = note(&mut hb, "", 0.0);
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    if let Some(e) = r.host.ask(&mut r.hb, private_on(), 0) {
        panic!("{e:?}");
    }
    let p = &mut r.peers[0];
    let b = p.b.as_mut().unwrap();
    write(b, id, "escrito na virada");
    let up = p.g.tick(b, 0);
    assert!(
        up.iter().any(|m| matches!(m, Msg::Writes(_))),
        "controlo: a Bia escreveu às claras"
    );
    p.up.extend(up.into_iter().map(|m| m.encode()));
    r.pump();
    assert!(!r.heard(1, "escrito na virada"), "o Caio ouviu-o");
    assert_eq!(r.fac(1).hidden, BTreeSet::from([id]));
    r.host_ask(Ask::PrivateOff);
    r.at(REVEAL_COUNTDOWN_MS);
    assert_eq!(text(r.b(1), id), "escrito na virada");
    r.all_same();
}

/// ⛔ Prova de mutação W12 (M14): o selado que ia no fio quando a ligação caiu (capturado, nunca
/// entregue) volta ao COFRE no reenvio — nunca às claras a todos.
#[test]
fn a_sealed_write_lost_on_the_wire_is_resent_to_the_vault() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let p = &mut r.peers[0];
    let b = p.b.as_mut().unwrap();
    let id = note(b, "perdido no fio", 0.0);
    let lost = p.g.tick(b, 0);
    assert!(
        lost.iter().any(|m| matches!(m, Msg::Sealed(_))),
        "controlo: saiu selado"
    );
    r.drop(0);
    r.back(0);
    r.at(1);
    assert!(!r.heard(1, "perdido no fio"), "o reenvio saiu às claras");
    assert_eq!(r.fac(1).hidden, BTreeSet::from([id]));
    r.ask(0, Ask::PrivateOff);
    r.at(1 + REVEAL_COUNTDOWN_MS);
    assert_eq!(text(r.b(1), id), "perdido no fio");
    r.all_same();
}
