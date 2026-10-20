//! A equipa no documento (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 K, R, P).

use crate::{
    BoardId, BoardOp, BoardSet, Connector, Element, End, History, MessageId, Pin, Rgba, RichText,
    Route, SESSION_ID_BIT, Shape, ShapeType, Style, Team,
};

fn note(set: &mut BoardSet) -> (BoardId, crate::ElementId) {
    let a = set.create("Retro".into());
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([30, 30, 30, 255]);
    let el = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        Shape {
            kind: ShapeType::Sticky,
            style: Style::new(Some(Rgba([255, 249, 177, 255])), None, ink),
            text: RichText::plain("ideia"),
        },
        [100.0, 100.0, 200.0, 100.0],
    );
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    (a, id)
}

/// ⭐ K2: o pino num objecto anda, roda e escala com ele (fracção da caixa antes de rodar).
#[test]
fn a_pin_on_an_object_follows_it() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    let pin = Pin::at(&b.doc, Some(id), [150.0, 125.0]);
    assert_eq!(
        pin,
        Pin::Element {
            id,
            at: [0.25, 0.25]
        }
    );
    let t = b.team.open(pin, "Ana", "aqui", 0).unwrap();
    let mut el = b.doc.get(id).unwrap().clone();
    (el.x, el.y, el.w, el.h) = (0.0, 0.0, 400.0, 200.0);
    BoardOp::Put(el.clone()).apply(&mut b.doc);
    assert_eq!(
        b.team.thread(t).unwrap().anchor(&b.doc),
        Some([100.0, 50.0])
    );
    el.angle = std::f64::consts::FRAC_PI_2;
    BoardOp::Put(el).apply(&mut b.doc);
    let p = b.team.thread(t).unwrap().anchor(&b.doc).unwrap();
    // A meio-volta horária à volta do centro (200, 100): (100, 50) → (250, 0).
    assert!(
        (p[0] - 250.0).abs() < 1e-9 && (p[1] - 0.0).abs() < 1e-9,
        "{p:?}"
    );
}

/// K2: no vazio ou numa seta (sem caixa: a geometria é a rota), o pino fica no ponto.
#[test]
fn a_pin_on_nothing_or_on_an_arrow_stays_on_the_point() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    assert_eq!(Pin::at(&b.doc, None, [1.0, 2.0]), Pin::Point([1.0, 2.0]));
    let ink = Rgba([0, 0, 0, 255]);
    let c = Element::new_connector(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Connector::new(
            End::Free([0.0, 0.0]),
            End::Bound {
                target: id,
                anchor: crate::Anchor::Center,
            },
            Route::Straight,
            Style::new(None, Some(ink), ink),
        ),
    );
    let cid = c.id;
    BoardOp::Put(c).apply(&mut b.doc);
    assert_eq!(
        Pin::at(&b.doc, Some(cid), [5.0, 6.0]),
        Pin::Point([5.0, 6.0])
    );
}

/// ⭐ K3: o objecto apagado leva o fio; desfazer o apagar devolve-o.
#[test]
fn a_thread_leaves_with_its_object_and_comes_back_with_undo() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    let t = b
        .team
        .open(Pin::at(&b.doc, Some(id), [150.0, 150.0]), "Ana", "x", 0)
        .unwrap();
    let mut h = History::default();
    h.apply(&mut b.doc, vec![BoardOp::Delete(id)]);
    assert_eq!(b.team.thread(t).unwrap().anchor(&b.doc), None);
    h.undo(&mut b.doc);
    assert!(b.team.thread(t).unwrap().anchor(&b.doc).is_some());
}

/// ⭐ K4/K6: abrir, responder, ler — o vazio não publica; quem escreve leu; a resposta volta a pôr o
/// fio por ler para os outros.
#[test]
fn a_thread_opens_replies_and_tracks_who_read_it() {
    let mut t = Team::default();
    assert_eq!(t.open(Pin::Point([0.0; 2]), "Ana", "  \n ", 0), None);
    let id = t.open(Pin::Point([0.0; 2]), "Ana", " Olá \n", 10).unwrap();
    let th = t.thread(id).unwrap();
    assert_eq!(th.messages[0].text, "Olá");
    assert!(!th.unread_for("Ana") && th.unread_for("Rui"));
    t.mark_read(id, "Rui");
    t.mark_read(id, "Rui");
    assert_eq!(t.thread(id).unwrap().read_by, ["Ana", "Rui"]);
    assert!(!t.reply(id, "Rui", " ", 20), "a resposta vazia não publica");
    assert!(t.reply(id, "Rui", "Boa", 20));
    let th = t.thread(id).unwrap();
    assert_eq!(th.messages.len(), 2);
    assert!(th.unread_for("Ana") && !th.unread_for("Rui"));
}

fn msg_ids(t: &Team, id: crate::ThreadId) -> Vec<MessageId> {
    t.thread(id)
        .unwrap()
        .messages
        .iter()
        .map(|m| m.id)
        .collect()
}

/// ⭐ K4: só o autor edita e apaga a SUA mensagem — pelo id dela; apagar a 1.ª apaga o fio.
#[test]
fn only_the_author_edits_or_deletes_and_the_first_takes_the_thread() {
    let mut t = Team::default();
    let id = t.open(Pin::Point([0.0; 2]), "Ana", "a", 0).unwrap();
    t.reply(id, "Rui", "b", 1);
    let [m0, m1] = msg_ids(&t, id)[..] else {
        panic!("duas mensagens");
    };
    assert!(!t.edit(id, m0, "Rui", "x"), "a do outro não");
    assert!(t.edit(id, m0, "Ana", "a"), "igual não marca editada");
    assert!(!t.thread(id).unwrap().messages[0].edited);
    assert!(t.edit(id, m0, "Ana", "c"));
    assert!(t.thread(id).unwrap().messages[0].edited);
    assert!(!t.delete(id, m1, "Ana"));
    assert!(t.delete(id, m1, "Rui"));
    assert_eq!(t.thread(id).unwrap().messages.len(), 1);
    assert!(t.delete(id, m0, "Ana"));
    assert!(t.thread(id).is_none());
}

/// ⭐ A mensagem é o ID, nunca a posição: apagar uma de cima não desloca a edição de outra; um id
/// que não existe (ou de outro fio) não faz nada.
#[test]
fn a_message_is_its_id_not_its_place() {
    let mut t = Team::default();
    let a = t.open(Pin::Point([0.0; 2]), "Ana", "a", 0).unwrap();
    t.reply(a, "Rui", "b", 1);
    t.reply(a, "Ana", "c", 2);
    let b = t.open(Pin::Point([9.0; 2]), "Ana", "outro", 3).unwrap();
    let [_, m1, m2] = msg_ids(&t, a)[..] else {
        panic!("três mensagens");
    };
    let other = msg_ids(&t, b)[0];
    assert!(t.delete(a, m1, "Rui"));
    assert!(t.edit(a, m2, "Ana", "c!"));
    assert_eq!(t.thread(a).unwrap().messages[1].text, "c!");
    let before = t.clone();
    let ghost = MessageId(999);
    assert!(!t.edit(a, ghost, "Ana", "x") && !t.delete(a, ghost, "Ana"));
    assert!(!t.edit(a, other, "Ana", "x") && !t.delete(a, other, "Ana"));
    assert!(!t.delete(a, m1, "Rui"), "a apagada já não existe");
    assert_eq!(t, before);
}

/// ⭐ Fora de sessão os ids de fios, mensagens e versões saem do MESMO contador, por ordem; ele
/// grava-se, e a leitura continua acima dele.
#[test]
fn ids_out_of_a_session_are_one_counter_and_it_survives_the_file() {
    let mut set = BoardSet::default();
    let (a, _) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    let t1 = b.team.open(Pin::Point([0.0; 2]), "Ana", "x", 0).unwrap();
    b.team.reply(t1, "Rui", "y", 1);
    let t2 = b.team.open(Pin::Point([1.0; 2]), "Rui", "z", 2).unwrap();
    b.save_version("v".into(), 3);
    assert_eq!((t1, t2), (crate::ThreadId(1), crate::ThreadId(4)));
    assert_eq!(msg_ids(&b.team, t1), [MessageId(2), MessageId(3)]);
    assert_eq!(msg_ids(&b.team, t2), [MessageId(5)]);
    assert_eq!(b.team.versions[0].id, crate::VersionId(6));
    let mut back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back.get_mut(a).unwrap().team.mint(),
        7,
        "o contador viaja no ficheiro"
    );
}

/// ⭐ Em sessão (ADR-0181 §5): duas máquinas com sementes diferentes cunham ids DIFERENTES, com o bit
/// alto; o contador local não anda (ao sair, continua onde estava).
#[test]
fn two_sessions_mint_different_ids_with_the_high_bit_and_the_counter_stays() {
    let mut teams = [Team::default(), Team::default()];
    let mut ids = Vec::new();
    for (seed, t) in teams.iter_mut().enumerate() {
        t.set_session(Some(seed as u64 + 1));
        let th = t.open(Pin::Point([0.0; 2]), "Ana", "x", 0).unwrap();
        t.reply(th, "Rui", "y", 1);
        ids.push(th.0);
        ids.extend(msg_ids(t, th).iter().map(|m| m.0));
    }
    assert!(ids.iter().all(|id| id & SESSION_ID_BIT != 0), "{ids:x?}");
    let unique: std::collections::BTreeSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "{ids:x?}");
    teams[0].set_session(None);
    assert_eq!(teams[0].mint(), 1, "o contador não saltou");
}

/// ⭐ O sorteio da sessão nunca devolve um id que a equipa já tem (o que chegou de outra máquina).
#[test]
fn a_session_never_mints_an_id_the_team_already_has() {
    let mut a = Team::default();
    a.set_session(Some(5));
    let mut b = a.clone();
    let th = a.open(Pin::Point([0.0; 2]), "Ana", "x", 0).unwrap();
    b.threads = a.threads.clone();
    let taken = [th.0, msg_ids(&a, th)[0].0];
    let fresh = [b.mint(), b.mint()];
    assert!(
        fresh.iter().all(|f| !taken.contains(f)),
        "{fresh:x?} {taken:x?}"
    );
}

/// O quadro entra em sessão com o documento E a equipa, e os dois sorteios não andam juntos.
#[test]
fn a_board_session_draws_the_doc_and_the_team_apart() {
    let mut set = BoardSet::default();
    let (a, _) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    b.set_session(Some(7));
    assert!(b.doc.in_session());
    let (el, team) = (b.doc.mint_id().0, b.team.mint());
    assert!(team & SESSION_ID_BIT != 0);
    assert_ne!(el, team, "a mesma semente nos dois daria o mesmo número");
    b.set_session(None);
    assert!(!b.doc.in_session());
    assert_eq!(b.team.mint(), 1);
}

/// K5 resolver e reabrir; P1 os frames fora da apresentação.
#[test]
fn resolve_and_hidden_slides() {
    let mut t = Team::default();
    let id = t.open(Pin::Point([0.0; 2]), "Ana", "a", 0).unwrap();
    assert!(t.set_resolved(id, true));
    assert!(t.thread(id).unwrap().resolved);
    assert!(t.set_resolved(id, false));
    assert!(!t.thread(id).unwrap().resolved);
    let f = crate::ElementId(7);
    t.set_slide_hidden(f, true);
    assert!(t.hidden_slides.contains(&f));
    t.set_slide_hidden(f, false);
    assert!(t.hidden_slides.is_empty());
}

/// ⭐ R1/R3/R4: a versão guarda o documento e o rascunho, NÃO os comentários nem as outras versões;
/// restaurar é um quadro NOVO à direita, e o original não muda.
#[test]
fn a_version_restores_as_a_new_board_beside_and_leaves_the_original() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    b.sketch = true;
    b.team.open(Pin::Point([0.0; 2]), "Ana", "c", 0);
    b.save_version("v1".into(), 42);
    b.save_version("v2".into(), 43);
    BoardOp::Delete(id).apply(&mut b.doc);
    let v = &b.team.versions[0];
    assert_eq!((v.name.as_str(), v.at), ("v1", 42));
    let saved = v.read().unwrap();
    let (doc, sketch) = (saved.doc, saved.sketch);
    assert!(sketch);
    assert_eq!(doc.get(id).unwrap().shape().unwrap().text.as_str(), "ideia");
    let tail = set.create("Tail".into());
    let r = set
        .insert_after(a, "Retro — v1".into(), doc, sketch)
        .unwrap();
    let ids: Vec<_> = set.boards().iter().map(|b| b.id).collect();
    assert_eq!(ids, [a, r, tail]);
    let restored = set.get(r).unwrap();
    assert!(restored.team.threads.is_empty() && restored.team.versions.is_empty());
    assert!(restored.doc.get(id).is_some());
    assert!(
        set.get(a).unwrap().doc.get(id).is_none(),
        "o original não muda"
    );
}

/// ⭐ A versão é exactamente os bytes de um `BoardSet` de um quadro (sem o clonar): o leitor de
/// todos os formatos abre-a — o mesmo desenho de campos.
#[test]
fn a_version_is_the_bytes_of_a_one_board_set() {
    let mut set = BoardSet::default();
    let (a, _) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    b.team.open(Pin::Point([0.0; 2]), "Ana", "c", 0);
    b.camera.zoom = 3.0;
    b.save_version("v".into(), 1);
    let v = b.team.versions[0].clone();
    let mut z = flate2::write::DeflateDecoder::new(Vec::new());
    std::io::Write::write_all(&mut z, &v.bytes).unwrap();
    let raw = z.finish().unwrap();
    let mut alone = crate::Board {
        camera: crate::Camera::default(),
        team: Team::default(),
        ..set.get(a).unwrap().clone()
    };
    alone.doc = set.get(a).unwrap().doc.clone();
    let one = BoardSet::from_parts(vec![alone], a.0);
    assert_eq!(raw, one.to_bytes().unwrap());
}

/// ⭐ K9/R4: a equipa grava-se e lê-se (formato 8); duplicar o QUADRO leva-a.
#[test]
fn the_team_round_trips_and_travels_with_a_duplicated_board() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    let t = b
        .team
        .open(Pin::at(&b.doc, Some(id), [120.0, 120.0]), "Ana", "c", 5)
        .unwrap();
    b.team.reply(t, "Rui", "r", 6);
    b.team.set_resolved(t, true);
    b.team.set_slide_hidden(id, true);
    b.save_version("v".into(), 7);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    let d = set.duplicate(a, "Copy".into()).unwrap();
    assert_eq!(set.get(d).unwrap().team, set.get(a).unwrap().team);
}

/// ⭐ Um ficheiro do formato 7 (W7) continua a abrir — gravado pelo build de `74e5b7163` (o último do
/// formato 7, um teste temporário que escreveu `to_bytes`): um frame «Ideas», uma nota dentro dele com
/// 3 votos, um carimbo e uma etiqueta, um rectângulo e uma seta entre os dois. Sem equipa.
#[test]
fn a_format_7_file_still_opens_with_no_team() {
    let set = BoardSet::from_bytes(V7_BYTES).expect("o formato 7 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Retro7");
    assert_eq!(b.camera.zoom, 0.75);
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 4);
    assert_eq!(els[0].as_frame().unwrap().name, "Ideas");
    let note = els[1];
    assert_eq!(note.frame, Some(els[0].id));
    assert_eq!(note.stamps().len(), 1);
    // W13 (ADR-0183): os 3 votos da nota viram a única sessão do histórico, mostrada.
    let [p] = &b.team.polls[..] else {
        panic!("uma sessão: os votos do ficheiro");
    };
    assert_eq!(p.totals.iter().collect::<Vec<_>>(), [(&note.id, &3)]);
    assert_eq!(b.team.shown, Some(p.id));
    assert!(b.team.threads.is_empty() && b.team.versions.is_empty());
    assert_eq!(note.tags()[0].name, "urgente");
    assert!(els[3].connector().is_some());
    assert_eq!(b.doc.frames_in_panel_order()[0].id, els[0].id);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 7 (`74e5b7163`).
const V7_BYTES: &[u8] = include_bytes!("../fixtures/format_v7.bin");

/// ⭐ R4/P1 — «não apresentar» é documento: a versão leva-o (a auditoria W8 viu a restaurada voltar a
/// apresentar o frame escondido).
#[test]
fn a_version_keeps_the_frames_hidden_from_the_presentation() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    b.team.set_slide_hidden(id, true);
    b.save_version("v".into(), 0);
    let saved = b.team.versions[0].read().unwrap();
    assert!(saved.team.hidden_slides.contains(&id));
    assert!(saved.team.threads.is_empty() && saved.team.versions.is_empty());
}

/// ⭐ Os ids de GRUPO partilham o contador dos elementos: um elemento posto com um grupo (a leitura de
/// um ficheiro, uma colagem) sobe o contador acima dele — senão o próximo grupo cunhado juntava-se ao
/// importado (auditoria W8).
#[test]
fn a_put_group_raises_the_counter_past_it() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    let mut el = b.doc.get(id).unwrap().clone();
    el.groups_mut().push(crate::GroupId(50));
    BoardOp::Put(el).apply(&mut b.doc);
    assert!(b.doc.mint_group().0 > 50);
}

/// ⭐ O dono (10/10): *«não temos undo para Resolve»*. Resolver e mover a bolha entram na MESMA pilha
/// do desenho, pela ordem em que se fizeram: o desfazer do documento pára diante de um passo da
/// equipa (que o `undo_team` desfaz), e o refazer devolve-os pela ordem inversa.
#[test]
fn resolve_and_move_undo_in_order_with_the_drawing() {
    let mut set = BoardSet::default();
    let (a, id) = note(&mut set);
    let b = set.get_mut(a).unwrap();
    let t = b.team.open(Pin::Point([0.0, 0.0]), "Ana", "x", 0).unwrap();
    let mut h = History::default();
    let mut moved = b.doc.get(id).unwrap().clone();
    moved.x += 10.0;
    h.apply(&mut b.doc, vec![BoardOp::Put(moved)]);
    assert!(h.apply_team(&mut b.team, crate::TeamOp::Resolve(t, true)));
    assert!(
        !h.apply_team(&mut b.team, crate::TeamOp::Resolve(t, true)),
        "nada mudou: não é passo"
    );
    let to = Pin::Element { id, at: [0.5, 0.5] };
    assert!(h.apply_team(&mut b.team, crate::TeamOp::Pin(t, to)));
    // Desfazer: o documento não passa por cima dos passos da equipa.
    assert!(!h.undo(&mut b.doc));
    assert!(h.undo_team(&mut b.team));
    assert_eq!(b.team.thread(t).unwrap().pin, Pin::Point([0.0, 0.0]));
    assert!(!h.undo(&mut b.doc));
    assert!(h.undo_team(&mut b.team));
    assert!(!b.team.thread(t).unwrap().resolved);
    assert!(!h.undo_team(&mut b.team), "o seguinte é do documento");
    assert!(h.undo(&mut b.doc));
    assert_eq!(b.doc.get(id).unwrap().x, 100.0);
    // Refazer pela ordem inversa.
    assert!(h.redo(&mut b.doc) && h.redo_team(&mut b.team) && h.redo_team(&mut b.team));
    let th = b.team.thread(t).unwrap();
    assert!(th.resolved && th.pin == to);
    assert!(!h.can_redo());
}

/// Um passo da equipa cujo fio já saiu (apagar a 1.ª mensagem não se desfaz) gasta-se sem efeito.
#[test]
fn a_team_step_of_a_gone_thread_is_spent() {
    let mut team = Team::default();
    let t = team.open(Pin::Point([0.0, 0.0]), "Ana", "x", 0).unwrap();
    let mut h = History::default();
    assert!(h.apply_team(&mut team, crate::TeamOp::Resolve(t, true)));
    let first = team.thread(t).unwrap().messages[0].id;
    assert!(team.delete(t, first, "Ana"));
    assert!(h.undo_team(&mut team));
    assert!(!h.can_undo() && !h.can_redo());
}

/// Gravados pelo build do formato 15 (`401153313a`): um fio do Rui («@Enio look at this») e a resposta
/// do Enio («ok @Rui») — o texto tinha as letras, não a menção.
const V15_BYTES: &[u8] = include_bytes!("../fixtures/format_v15.bin");

/// ⭐ W23 B14 — o ficheiro do formato 15 lê-se, as mensagens SEM menções (as letras «@Enio» de um
/// ficheiro antigo não chamam ninguém: ninguém as escolheu), e o 16 vai e volta com elas.
#[test]
fn format_15_reads_with_no_mentions_and_16_keeps_them() {
    assert_eq!(postcard::take_from_bytes::<u32>(V15_BYTES).unwrap().0, 15);
    let mut set = crate::BoardSet::from_bytes(V15_BYTES).unwrap();
    let b = set.boards()[0].id;
    let board = set.get_mut(b).unwrap();
    let t = &board.team.threads[0];
    assert_eq!(t.messages[0].text, "@Enio look at this");
    assert!(t.messages.iter().all(|m| m.mentions.is_empty()));
    let id = t.id;
    assert!(board.team.reply_calling(
        id,
        "Rui",
        "@Enio and @everyone",
        &[crate::Who::Person("Enio".into()), crate::Who::Everyone],
        1_760_000_200,
    ));
    let back = crate::BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    let m = back.boards()[0].team.threads[0].messages.last().unwrap();
    assert_eq!(m.mentions.len(), 2);
}

/// Só fica chamado quem o texto mostra — à nascença e depois de editar.
#[test]
fn a_mention_lives_only_while_the_text_shows_it() {
    let mut team = Team::default();
    let who = [crate::Who::Person("Ana".into()), crate::Who::Person("Rui".into())];
    let t = team
        .open_calling(Pin::Point([0.0; 2]), "Enio", "hi @Ana", &who, 1)
        .unwrap();
    let m = &team.thread(t).unwrap().messages[0];
    assert_eq!(m.mentions, vec![crate::Who::Person("Ana".into())], "o Rui não está no texto");
    let mid = m.id;
    assert!(team.edit(t, mid, "Enio", "hi everyone"));
    assert!(team.thread(t).unwrap().messages[0].mentions.is_empty());
}

/// ⭐ A porta «é para mim»: chama-me (ou a toda a gente), ou responde num fio onde eu escrevi — e
/// nunca uma mensagem minha.
#[test]
fn a_message_is_for_me_when_it_calls_me_or_answers_me() {
    let mut team = Team::default();
    let ana = [crate::Who::Person("Ana".into())];
    let t = team
        .open_calling(Pin::Point([0.0; 2]), "Rui", "@Ana see", &ana, 1)
        .unwrap();
    let th = team.thread(t).unwrap();
    assert!(th.for_me_at(0, "Ana"));
    assert!(!th.for_me_at(0, "Rui"), "a minha nunca é para mim");
    assert!(!th.for_me_at(0, "Bia"), "nem me chama nem escrevi");
    assert!(team.reply(t, "Bia", "agree", 2));
    let th = team.thread(t).unwrap();
    assert!(th.for_me_at(1, "Rui"), "a resposta num fio onde escrevi");
    assert!(!th.for_me_at(1, "Ana"), "a Ana não escreveu, só foi chamada");
    assert!(team.reply_calling(t, "Rui", "@everyone done", &[crate::Who::Everyone], 3));
    let th = team.thread(t).unwrap();
    assert!(th.for_me_at(2, "Zé"), "toda a gente");
    assert!(th.for_me("Ana") && th.for_me("Zé"));
    assert_eq!(team.people("Rui", &["Zé".to_owned()]), vec!["Ana", "Bia", "Zé"]);
}

/// A porta «é para mim» nas bordas: a MINHA resposta num fio onde escrevi não é para mim; uma
/// mensagem minha POSTERIOR não torna a anterior para mim; fora do fio, nada.
#[test]
fn my_own_reply_and_my_later_reply_are_never_for_me() {
    let mut team = Team::default();
    let t = team.open(Pin::Point([0.0; 2]), "Bia", "plan?", 1).unwrap();
    assert!(team.reply(t, "Rui", "ok", 2));
    assert!(team.reply(t, "Bia", "thanks", 3));
    assert!(team.reply(t, "Bia", "and more", 4));
    let th = team.thread(t).unwrap();
    assert!(!th.for_me_at(0, "Rui"), "a do início não fica minha por eu ter respondido depois");
    assert!(th.for_me_at(2, "Rui"), "a resposta da Bia depois da minha");
    assert!(!th.for_me_at(3, "Bia"), "a minha 2.ª mensagem num fio onde já escrevi");
    assert!(!th.for_me_at(99, "Rui"), "fora do fio");
}

/// ⭐ A lista das menções é CANÓNICA (ordenada, sem repetidos) à nascença e depois de editar com
/// menções novas — cada par chega à mesma lista seja qual for a ordem em que as escolheu.
#[test]
fn the_mentions_are_one_sorted_list_without_repeats() {
    use crate::Who;
    let mut team = Team::default();
    let who = [
        Who::Everyone,
        Who::Person("Rui".into()),
        Who::Person("Ana".into()),
        Who::Person("Ana".into()),
    ];
    let t = team
        .open_calling(Pin::Point([0.0; 2]), "Enio", "@Rui @Ana @everyone", &who, 1)
        .unwrap();
    let canonical = vec![Who::Person("Ana".into()), Who::Person("Rui".into()), Who::Everyone];
    let m = &team.thread(t).unwrap().messages[0];
    assert_eq!(m.mentions, canonical);
    let mid = m.id;
    let more = [Who::Person("Bia".into()), Who::Person("Bia".into()), Who::Person("Zé".into())];
    assert!(team.edit_calling(t, mid, "Enio", "@Bia and @Ana", &more));
    assert_eq!(
        team.thread(t).unwrap().messages[0].mentions,
        vec![Who::Person("Ana".into()), Who::Person("Bia".into())],
        "o Zé não está no texto, a Bia uma vez só"
    );
}
