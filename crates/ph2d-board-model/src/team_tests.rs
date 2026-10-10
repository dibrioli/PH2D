//! A equipa no documento (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 K, R, P).

use crate::{
    BoardId, BoardOp, BoardSet, Connector, Element, End, History, Pin, Rgba, RichText, Route,
    Shape, ShapeType, Style, Team,
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
    assert_eq!(pin, Pin::Element { id, at: [0.25, 0.25] });
    let t = b.team.open(pin, "Ana", "aqui", 0).unwrap();
    let mut el = b.doc.get(id).unwrap().clone();
    (el.x, el.y, el.w, el.h) = (0.0, 0.0, 400.0, 200.0);
    BoardOp::Put(el.clone()).apply(&mut b.doc);
    assert_eq!(b.team.thread(t).unwrap().anchor(&b.doc), Some([100.0, 50.0]));
    el.angle = std::f64::consts::FRAC_PI_2;
    BoardOp::Put(el).apply(&mut b.doc);
    let p = b.team.thread(t).unwrap().anchor(&b.doc).unwrap();
    // A meio-volta horária à volta do centro (200, 100): (100, 50) → (250, 0).
    assert!((p[0] - 250.0).abs() < 1e-9 && (p[1] - 0.0).abs() < 1e-9, "{p:?}");
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
    assert_eq!(Pin::at(&b.doc, Some(cid), [5.0, 6.0]), Pin::Point([5.0, 6.0]));
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

/// ⭐ K4: só o autor edita e apaga a SUA mensagem; apagar a 1.ª apaga o fio.
#[test]
fn only_the_author_edits_or_deletes_and_the_first_takes_the_thread() {
    let mut t = Team::default();
    let id = t.open(Pin::Point([0.0; 2]), "Ana", "a", 0).unwrap();
    t.reply(id, "Rui", "b", 1);
    assert!(!t.edit(id, 0, "Rui", "x"), "a do outro não");
    assert!(t.edit(id, 0, "Ana", "a"), "igual não marca editada");
    assert!(!t.thread(id).unwrap().messages[0].edited);
    assert!(t.edit(id, 0, "Ana", "c"));
    assert!(t.thread(id).unwrap().messages[0].edited);
    assert!(!t.delete(id, 1, "Ana"));
    assert!(t.delete(id, 1, "Rui"));
    assert_eq!(t.thread(id).unwrap().messages.len(), 1);
    assert!(t.delete(id, 0, "Ana"));
    assert!(t.thread(id).is_none());
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
    let (doc, sketch) = v.read().unwrap();
    assert!(sketch);
    assert_eq!(doc.get(id).unwrap().shape().unwrap().text.as_str(), "ideia");
    let tail = set.create("Tail".into());
    let r = set.insert_after(a, "Retro — v1".into(), doc, sketch).unwrap();
    let ids: Vec<_> = set.boards().iter().map(|b| b.id).collect();
    assert_eq!(ids, [a, r, tail]);
    let restored = set.get(r).unwrap();
    assert!(restored.team.threads.is_empty() && restored.team.versions.is_empty());
    assert!(restored.doc.get(id).is_some());
    assert!(set.get(a).unwrap().doc.get(id).is_none(), "o original não muda");
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
    assert_eq!(b.team, Team::default());
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 4);
    assert_eq!(els[0].as_frame().unwrap().name, "Ideas");
    let note = els[1];
    assert_eq!(note.frame, Some(els[0].id));
    assert_eq!((note.votes(), note.stamps().len()), (3, 1));
    assert_eq!(note.tags()[0].name, "urgente");
    assert!(els[3].connector().is_some());
    assert_eq!(b.doc.frames_in_panel_order()[0].id, els[0].id);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set, "relido e regravado no formato actual, fica igual");
}

/// Gravados pelo build do formato 7 (`74e5b7163`).
const V7_BYTES: &[u8] = include_bytes!("../fixtures/format_v7.bin");
