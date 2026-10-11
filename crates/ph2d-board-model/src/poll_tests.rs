//! O histórico das votações (W13, ADR-0183).

use std::collections::BTreeMap;

use crate::{BoardSet, ElementId, PollId, Team};

fn totals(v: &[(u64, u32)]) -> BTreeMap<ElementId, u32> {
    v.iter().map(|&(id, n)| (ElementId(id), n)).collect()
}

/// ⭐ Três votações seguidas ficam as três, «Voting 1..3», a mais nova mostrada e em cima; o
/// instantâneo guarda só os `> 0`.
#[test]
fn three_votes_in_a_row_are_three_sessions_and_the_newest_is_shown() {
    let mut t = Team::default();
    let a = t.add_poll(totals(&[(1, 2), (2, 1), (3, 0)]), "Ana", 2, 100);
    let b = t.add_poll(totals(&[(5, 1)]), "Rui", 1, 200);
    let c = t.add_poll(totals(&[(4, 3)]), "", 2, 300);
    let nums: Vec<_> = t.polls_newest_first().map(|p| (p.id, p.number)).collect();
    assert_eq!(nums, [(c, 3), (b, 2), (a, 1)]);
    assert_eq!(t.shown, Some(c));
    let pa = t.poll(a).unwrap();
    assert_eq!(pa.totals, totals(&[(1, 2), (2, 1)]), "o zero não entra");
    assert_eq!((pa.total(), pa.voters, pa.by.as_str(), pa.at), (3, 2, "Ana", 100));
    assert_eq!(pa.name, "", "sem nome: o ecrã pinta «Voting N»");
}

/// O ranking: o mais votado primeiro, empate pelo id mais antigo.
#[test]
fn a_session_ranks_most_voted_first_and_ties_by_the_older_id() {
    let mut t = Team::default();
    let id = t.add_poll(totals(&[(9, 2), (3, 2), (5, 4)]), "", 1, 0);
    let r = t.poll(id).unwrap().ranked();
    assert_eq!(r, [(ElementId(5), 4), (ElementId(3), 2), (ElementId(9), 2)]);
}

/// Renomear apara as pontas e diz se mudou; apagar a mostrada desliga o selo; o número seguinte é
/// 1 + o maior VIVO.
#[test]
fn rename_trims_delete_turns_the_badge_off_and_numbers_follow_the_living() {
    let mut t = Team::default();
    let a = t.add_poll(totals(&[(1, 1)]), "", 1, 0);
    let b = t.add_poll(totals(&[(2, 1)]), "", 1, 0);
    assert!(t.rename_poll(a, "  Retro  "));
    assert_eq!(t.poll(a).unwrap().name, "Retro");
    assert!(!t.rename_poll(a, "Retro"), "o mesmo nome não muda nada");
    assert!(!t.rename_poll(PollId(99), "x"));
    assert!(t.remove_poll(b));
    assert_eq!(t.shown, None, "a mostrada saiu: o quadro não mostra nada");
    assert!(!t.remove_poll(b));
    let c = t.add_poll(totals(&[(3, 1)]), "", 1, 0);
    assert_eq!(t.poll(c).unwrap().number, 2);
    t.show_poll(Some(a));
    assert_eq!(t.shown_poll().map(|p| p.id), Some(a));
    t.show_poll(Some(b));
    assert_eq!(t.shown, None, "mostrar uma que não existe é não mostrar nenhuma");
}

/// ⭐ ADR-0183 §8: a cópia do quadro nasce sem histórico (Miro), a gravação leva-o, e a versão nunca.
#[test]
fn the_history_survives_the_file_but_not_a_duplicate_nor_a_version() {
    let mut set = BoardSet::default();
    let a = set.create("Retro".into());
    let b = set.get_mut(a).unwrap();
    b.team.add_poll(totals(&[(1, 2)]), "Ana", 1, 50);
    b.save_version("v".into(), 60);
    let saved = b.team.versions[0].read().unwrap();
    assert!(saved.team.polls.is_empty() && saved.team.shown.is_none());
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    let d = set.duplicate(a, "Copy".into()).unwrap();
    let copy = &set.get(d).unwrap().team;
    assert!(copy.polls.is_empty() && copy.shown.is_none());
    assert_eq!(set.get(a).unwrap().team.polls.len(), 1, "o original fica com o seu");
}

/// Os ids das sessões saem do contador da equipa, partilhado com fios e versões.
#[test]
fn a_session_id_never_collides_with_a_thread_or_a_version() {
    let mut set = BoardSet::default();
    let a = set.create("R".into());
    let b = set.get_mut(a).unwrap();
    b.save_version("v".into(), 1);
    let p = b.team.add_poll(totals(&[(1, 1)]), "", 1, 0);
    assert_ne!(p.0, b.team.versions[0].id.0);
}

/// ⭐ Um ficheiro do formato 10 (W11–W12) continua a abrir e os votos viram UMA sessão — gravado pelo
/// build de `02f3fc596` (o último do formato 10; teste temporário que escreveu `to_bytes`, apagado):
/// «Retro10» com «ideia A» (3 votos, id 1), «ideia B» (1 voto, id 2) e uma caixa sem votos (id 3); um
/// fio de 2 mensagens em A; a versão «v1» (que leva os votos dentro, formato 10). «Outro»: uma nota sem
/// votos.
#[test]
fn a_format_10_file_opens_and_its_votes_become_one_shown_session() {
    let set = BoardSet::from_bytes(V10_BYTES).expect("o formato 10 lê-se");
    let [a, o] = set.boards() else {
        panic!("dois quadros");
    };
    assert_eq!((a.name.as_str(), a.doc.live_len()), ("Retro10", 3));
    let [p] = &a.team.polls[..] else {
        panic!("uma sessão: a dos votos do ficheiro");
    };
    assert_eq!(p.totals, totals(&[(1, 3), (2, 1)]));
    assert_eq!((p.number, p.at, p.by.as_str(), p.voters), (1, 0, "", 0));
    assert_eq!(a.team.shown, Some(p.id), "o quadro mostra-os como antes");
    assert_eq!(a.team.threads[0].messages.len(), 2);
    assert!(
        a.team.threads.iter().all(|t| t.id.0 != p.id.0)
            && a.team.versions.iter().all(|v| v.id.0 != p.id.0),
        "o id da sessão não colide com os do ficheiro"
    );
    let saved = a.team.versions[0].read().expect("a versão do formato 10 abre");
    assert_eq!(saved.doc.live_len(), 3);
    assert!(
        o.team.polls.is_empty() && o.team.shown.is_none(),
        "sem votos, sem sessão"
    );
    let bytes = set.to_bytes().unwrap();
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 11);
    assert_eq!(BoardSet::from_bytes(&bytes).unwrap(), set, "regravado no 11, igual");
}

/// Gravados pelo build do formato 10 (`02f3fc596`).
const V10_BYTES: &[u8] = include_bytes!("../fixtures/format_v10.bin");
