//! A equipa e a saída (W8) pelo caminho REAL do ecrã — `docs/MiroClone/06_equipa_e_saida.md` §2: a barra
//! e os botões pela ordem da shell (`shell_click_bar`/`shell_click_id`), o quadro pela porta do ponteiro
//! (`board_view::pointer`), o teclado pelo despacho do hero (`handle_text_input`, `handle_key`) e pelas
//! teclas do quadro (`board_keys::key`), o menu da aba pelo menu aberto.

use super::*;
use crate::documents::team::{Format, IoRequest};
use crate::interaction::{ContextMenuKind, ContextMenuRequest};
use crate::screens::hero::board_bar::fac::FacItem;
use crate::screens::hero::board_nav::team::{Hit, TeamButton, node as team_node};
use crate::screens::hero::board_nav::{self, NavButton, RowPart, button_node, row_node};
use ph2d_board_model::{
    BoardOp, ElementId, Frame, Pin, Rgba, RichText, STICKY_COLORS, Shape, Style,
};
use ph2d_host::{KeyEvent, KeyKind};

impl T {
    fn note_at(&mut self, at: [f64; 2], text: &str) -> ElementId {
        let (board, live) = self.hero.documents.active_parts().unwrap();
        let ink = Rgba([20, 20, 20, 255]);
        let el = Element::new_shape(
            board.doc.mint_id(),
            board.doc.z_on_top(),
            Shape {
                kind: ShapeType::Sticky,
                style: Style::new(Some(Rgba(STICKY_COLORS[1])), None, ink),
                text: RichText::plain(text),
            },
            [at[0], at[1], 199.0, 199.0],
        );
        let id = el.id;
        let h = live.histories.entry(board.id).or_default();
        h.apply(&mut board.doc, vec![BoardOp::Put(el)]);
        self.paint();
        id
    }

    fn frame_at(&mut self, bx: [f64; 4], name: &str) -> ElementId {
        let (board, _) = self.hero.documents.active_parts().unwrap();
        let f = Element::new_frame(
            board.doc.mint_id(),
            board.doc.z_on_top(),
            Frame {
                name: name.into(),
                style: Style::new(Some(Rgba([255; 4])), None, Rgba([0, 0, 0, 255])),
            },
            bx,
        );
        let id = f.id;
        BoardOp::Put(f).apply(&mut board.doc);
        self.paint();
        id
    }

    /// O ponto `world` no ecrã.
    fn screen(&self, world: [f64; 2]) -> (f32, f32) {
        let a = board_view::area_of(self.hero.last_canvas);
        let p = self
            .hero
            .documents
            .active_board()
            .unwrap()
            .camera
            .to_screen(a, world);
        (p[0] as f32, p[1] as f32)
    }

    /// Um clique no QUADRO pela porta da shell (o carregar e o largar).
    fn click_board(&mut self, at: (f32, f32)) {
        let (h, ts, c) = (&mut self.hero, &mut self.ts, &mut self.clock);
        for kind in [PointerKind::Down, PointerKind::Up] {
            board_view::pointer(
                h,
                inp(ts, c),
                kind,
                PointerButton::Primary,
                at.0,
                at.1,
                true,
            );
        }
        self.paint();
    }

    /// Escreve `s` no campo com o teclado e carrega Enter (o despacho do hero).
    fn write_and_enter(&mut self, s: &str) {
        for ch in s.chars() {
            let arena = Bump::new();
            let ev = self.hero.handle_text_input(ch, &arena).to_vec();
            for e in ev {
                self.hero.apply_event(e);
            }
        }
        self.hero_key(crate::interaction::KEY_ENTER);
    }

    fn hero_key(&mut self, keycode: u32) {
        let arena = Bump::new();
        let e = KeyEvent {
            keycode,
            modifiers: Modifiers::default(),
            kind: KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = self.hero.handle_key(e, &arena).to_vec();
        for e in ev {
            self.hero.apply_event(e);
        }
        self.paint();
    }

    fn team(&self) -> &ph2d_board_model::Team {
        &self.hero.documents.active_board().unwrap().team
    }

    fn btn(&mut self, b: TeamButton) {
        self.shell_click_id(team_node(Hit::Button(b)), &format!("{b:?}"));
    }

    /// Abre o menu da aba do quadro activo e carrega na linha `row` (o menu pintado).
    fn tab_menu(&mut self, row: NodeId) {
        let board = self.hero.documents.active().unwrap().0;
        self.hero.store.open_context_menu(ContextMenuRequest {
            x: 200.0,
            y: 40.0,
            kind: ContextMenuKind::BoardTab { board },
        });
        self.paint();
        self.shell_click_id(row, "a linha do menu");
    }
}

/// ⭐ K1/K2/K4 — `Comment` na barra curta arma a ferramenta; um clique numa nota abre um fio NOVO preso
/// a ela; escrever e `Enter` publica-o com o meu nome; a bolha aparece no quadro e anda com a nota.
#[test]
fn the_comment_tool_pins_a_thread_to_the_object_and_enter_posts_it() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    let note = t.note_at([-100.0, -100.0], "idea");
    t.shell_click_bar(Item::Fac(FacItem::Comment));
    assert!(t.hero.documents.live.team.commenting);
    let p = t.screen([0.0, 0.0]);
    t.click_board(p);
    assert_eq!(
        t.elements().len(),
        1,
        "o clique de comentar não criou nada no quadro"
    );
    t.write_and_enter("Looks good");
    let th = &t.team().threads;
    assert_eq!(th.len(), 1);
    assert_eq!(th[0].messages[0].author, "Ana");
    assert_eq!(th[0].messages[0].text, "Looks good");
    assert!(
        matches!(th[0].pin, Pin::Element { id, .. } if id == note),
        "{:?}",
        th[0].pin
    );
    let bubble = team_node(Hit::Bubble(th[0].id));
    let before = t.find(bubble).expect("a bolha está pintada");
    // A nota anda: a bolha vai com ela.
    let (board, live) = t.hero.documents.active_parts().unwrap();
    let mut el = board.doc.get(note).unwrap().clone();
    el.x += 100.0;
    live.histories
        .entry(board.id)
        .or_default()
        .apply(&mut board.doc, vec![BoardOp::Put(el)]);
    t.paint();
    let after = t.find(bubble).unwrap();
    assert!(after.0 > before.0 + 10.0, "{before:?} → {after:?}");
}

/// ⭐ K5/K6/K8 — a bolha de um fio de OUTRO abre-o e marca-o lido por mim; resolver esconde-o do quadro
/// (o cartão fecha); «Show resolved» na lista volta a mostrá-lo.
#[test]
fn a_bubble_opens_reads_and_resolve_hides_until_show_resolved() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    t.note_at([-100.0, -100.0], "idea");
    let id = {
        let b = t.hero.documents.active_board_mut().unwrap();
        b.team
            .open(Pin::Point([0.0, 0.0]), "Rui", "Why?", 0)
            .unwrap()
    };
    t.paint();
    assert!(t.team().thread(id).unwrap().unread_for("Ana"));
    t.shell_click_id(team_node(Hit::Bubble(id)), "a bolha");
    assert!(
        !t.team().thread(id).unwrap().unread_for("Ana"),
        "K6: abrir lê"
    );
    t.btn(TeamButton::CardResolve);
    assert!(t.team().thread(id).unwrap().resolved);
    assert!(
        t.find(team_node(Hit::Bubble(id))).is_none(),
        "K5: resolvido some do quadro"
    );
    assert!(
        t.find(team_node(Hit::Button(TeamButton::CardClose)))
            .is_none(),
        "e o cartão fecha"
    );
    t.shell_click_id(button_node(NavButton::Comments), "Comments");
    t.btn(TeamButton::ListShowResolved);
    assert!(
        t.find(team_node(Hit::Bubble(id))).is_some(),
        "com «Show resolved» volta"
    );
    assert!(t.find(team_node(Hit::Row(id))).is_some(), "e está na lista");
}

/// K1/K8 — `C` arma a ferramenta; `Shift+C` esconde (e mostra) as bolhas; `Esc` larga a ferramenta.
#[test]
fn c_arms_the_tool_and_shift_c_hides_the_bubbles() {
    let mut t = T::new();
    let id = {
        let b = t.hero.documents.active_board_mut().unwrap();
        b.team.open(Pin::Point([0.0, 0.0]), "Rui", "x", 0).unwrap()
    };
    t.paint();
    assert!(t.key(BoardKey::Char('c'), Modifiers::default(), Some("c")));
    assert!(t.hero.documents.live.team.commenting);
    assert!(t.key(BoardKey::Escape, Modifiers::default(), None));
    assert!(!t.hero.documents.live.team.commenting);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    assert!(t.key(BoardKey::Char('c'), shift, Some("C")));
    assert!(
        t.find(team_node(Hit::Bubble(id))).is_none(),
        "Shift+C esconde"
    );
    t.key(BoardKey::Char('c'), shift, Some("C"));
    assert!(t.find(team_node(Hit::Bubble(id))).is_some(), "e volta");
}

/// ⭐ K4 — só as MINHAS mensagens têm editar e apagar; editar põe o texto no compositor e `Enter`
/// grava-o marcado «(edited)»; apagar a 1.ª apaga o fio.
#[test]
fn only_my_messages_edit_and_delete() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    let id = {
        let b = t.hero.documents.active_board_mut().unwrap();
        let id = b
            .team
            .open(Pin::Point([0.0, 0.0]), "Ana", "first", 0)
            .unwrap();
        b.team.reply(id, "Rui", "second", 1);
        id
    };
    t.paint();
    t.shell_click_id(team_node(Hit::Bubble(id)), "a bolha");
    assert!(
        t.find(team_node(Hit::Edit(0))).is_some(),
        "a minha tem editar"
    );
    assert!(t.find(team_node(Hit::Edit(1))).is_none(), "a do Rui não");
    assert!(t.find(team_node(Hit::Delete(1))).is_none());
    t.shell_click_id(team_node(Hit::Edit(0)), "editar");
    assert_eq!(
        board_nav::field_text_for_tests(&t.hero, crate::ids::BOARD_COMMENT_INPUT),
        "first"
    );
    t.write_and_enter("!");
    let m = &t.team().thread(id).unwrap().messages[0];
    assert_eq!((m.text.as_str(), m.edited), ("first!", true));
    t.shell_click_id(team_node(Hit::Delete(0)), "apagar");
    assert!(t.team().thread(id).is_none(), "apagar a 1.ª apaga o fio");
}

/// ⭐ K3 — apagar o objecto leva a bolha; desfazer devolve-a.
#[test]
fn deleting_the_object_hides_its_bubble_and_undo_brings_it_back() {
    let mut t = T::new();
    let note = t.note_at([-100.0, -100.0], "idea");
    let id = {
        let b = t.hero.documents.active_board_mut().unwrap();
        let pin = Pin::at(&b.doc, Some(note), [0.0, 0.0]);
        b.team.open(pin, "Rui", "x", 0).unwrap()
    };
    t.paint();
    let bubble = team_node(Hit::Bubble(id));
    assert!(t.find(bubble).is_some());
    let (board, live) = t.hero.documents.active_parts().unwrap();
    live.histories
        .entry(board.id)
        .or_default()
        .apply(&mut board.doc, vec![BoardOp::Delete(note)]);
    t.paint();
    assert!(t.find(bubble).is_none(), "o objecto apagado leva o fio");
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    t.key(BoardKey::Char('z'), ctrl, None);
    assert!(t.find(bubble).is_some(), "o desfazer devolve-o");
}

/// ⭐ R1/R2/R3 — pelo menu da aba: «Versions…» abre o painel com o nome pronto; `Enter` guarda; mexer
/// no quadro; «Restore» abre a versão num quadro NOVO à direita, e o original fica como está.
#[test]
fn a_version_saves_from_the_tab_menu_and_restores_as_a_new_board() {
    let mut t = T::new();
    let note = t.note_at([-100.0, -100.0], "before");
    let original = t.hero.documents.active().unwrap();
    t.tab_menu(crate::ids::CTX_MENU_BOARD_VERSIONS);
    assert_eq!(
        board_nav::field_text_for_tests(&t.hero, crate::ids::BOARD_VERSION_INPUT),
        "Version 1"
    );
    t.hero_key(crate::interaction::KEY_ENTER);
    assert_eq!(t.team().versions.len(), 1);
    assert_eq!(t.team().versions[0].name, "Version 1");
    let (board, live) = t.hero.documents.active_parts().unwrap();
    live.histories
        .entry(board.id)
        .or_default()
        .apply(&mut board.doc, vec![BoardOp::Delete(note)]);
    t.paint();
    t.shell_click_id(team_node(Hit::Restore(0)), "Restore");
    let now = t.hero.documents.active().unwrap();
    assert_ne!(now, original, "um quadro novo, activo");
    let tabs: Vec<_> = t
        .hero
        .documents
        .boards()
        .boards()
        .iter()
        .map(|b| b.id)
        .collect();
    assert_eq!(tabs, [original, now], "logo à direita");
    assert!(
        t.hero
            .documents
            .boards()
            .get(now)
            .unwrap()
            .doc
            .get(note)
            .is_some()
    );
    assert!(
        t.hero
            .documents
            .boards()
            .get(original)
            .unwrap()
            .doc
            .get(note)
            .is_none(),
        "o original não muda"
    );
}

/// ⭐ P1–P6 — «Present» (o botão do canto): começa no 1.º diapositivo, a sala inteira toma o clique,
/// `→`/`Espaço`/`←` andam, nas pontas pára, `Esc` sai e a vista de antes volta; o olho do painel de
/// Frames tira um frame da apresentação.
#[test]
fn present_walks_the_frames_and_esc_gives_the_view_back() {
    let mut t = T::new();
    t.hero.motion.set_reduced_motion(true);
    let a = t.frame_at([0.0, 0.0, 1600.0, 900.0], "One");
    let b = t.frame_at([2000.0, 0.0, 1600.0, 900.0], "Two");
    let c = t.frame_at([4000.0, 0.0, 800.0, 800.0], "Three");
    let before = t.hero.documents.active_board().unwrap().camera;
    t.shell_click_id(button_node(NavButton::Frames), "Frames");
    t.shell_click_id(row_node(b, RowPart::Eye), "o olho de Two");
    assert!(t.team().hidden_slides.contains(&b));
    t.shell_click_id(button_node(NavButton::Present), "Present");
    let show = |t: &T| t.hero.documents.live.team.show.clone().unwrap();
    assert_eq!(show(&t).slides, [a, c], "o escondido não entra");
    assert_eq!(show(&t).at, 0);
    // A sala toma o clique: no sítio das abas o que está é a apresentação.
    let tab = (t.hero.last_viewport.x + 40.0, t.hero.last_viewport.y + 10.0);
    assert_eq!(
        t.hero.hit_index.hit(tab.0, tab.1),
        Some(team_node(Hit::Button(TeamButton::ShowSurface)))
    );
    t.key(BoardKey::Right, Modifiers::default(), None);
    assert_eq!(show(&t).at, 1);
    t.key(BoardKey::Space, Modifiers::default(), None);
    assert_eq!(show(&t).at, 1, "na ponta pára");
    // P3 — com reduced motion a câmara É a do diapositivo (a margem medida).
    let cam = t.hero.documents.active_board().unwrap().camera;
    assert_eq!([cam.center_x, cam.center_y], [4400.0, 400.0]);
    t.btn(TeamButton::ShowPrev);
    assert_eq!(show(&t).at, 0);
    t.key(BoardKey::Escape, Modifiers::default(), None);
    assert!(t.hero.documents.live.team.show.is_none());
    assert_eq!(
        t.hero.documents.active_board().unwrap().camera,
        before,
        "P6: a vista de antes"
    );
}

/// P2 — sem frames, «Present» avisa e não abre.
#[test]
fn present_without_frames_tells_and_does_not_start() {
    let mut t = T::new();
    t.shell_click_id(button_node(NavButton::Present), "Present");
    assert!(t.hero.documents.live.team.show.is_none());
    assert_eq!(
        t.hero.documents.live.notice.map(|n| n.0),
        Some("board.present.no_frames")
    );
}

/// ⭐ E1–E4 — «Export…» do menu da aba abre o diálogo; escolher o PDF e «Export…» faz o pedido à
/// família do quadro; sem selecção a área «Selection» não se pinta. «Open an Excalidraw file…» pede a
/// importação.
#[test]
fn export_and_import_ask_the_family_for_a_file() {
    let mut t = T::new();
    t.note_at([-100.0, -100.0], "idea");
    t.tab_menu(crate::ids::CTX_MENU_BOARD_EXPORT);
    assert!(
        t.find(team_node(Hit::Button(TeamButton::ScopeSelection)))
            .is_none()
    );
    assert!(
        t.find(team_node(Hit::Button(TeamButton::SizeLarge)))
            .is_some(),
        "PNG: os tamanhos"
    );
    t.btn(TeamButton::ExportPdf);
    assert!(
        t.find(team_node(Hit::Button(TeamButton::SizeLarge)))
            .is_none(),
        "PDF: sem tamanhos"
    );
    t.btn(TeamButton::ExportGo);
    match t.hero.documents.take_io() {
        Some(IoRequest::Export { choice, .. }) => assert_eq!(choice.format, Format::Pdf),
        other => panic!("{other:?}"),
    }
    assert!(
        t.hero.documents.live.team.export.is_none(),
        "o diálogo fecha"
    );
    t.tab_menu(crate::ids::CTX_MENU_BOARD_IMPORT);
    assert_eq!(t.hero.documents.take_io(), Some(IoRequest::Import));
}

/// ⭐ Nenhum botão da equipa mora numa zona que a SHELL trata antes (o gate da W6/W7, alargado).
#[test]
fn no_shell_gesture_zone_sits_on_a_team_button() {
    let mut t = T::new();
    t.hero.view.rulers_visible = true;
    t.frame_at([-400.0, -300.0, 800.0, 600.0], "One");
    let id = {
        let b = t.hero.documents.active_board_mut().unwrap();
        b.team.open(Pin::Point([0.0, 0.0]), "Rui", "x", 0).unwrap()
    };
    t.paint();
    let mut found = vec![(
        "bubble".to_owned(),
        t.find(team_node(Hit::Bubble(id))).unwrap(),
    )];
    t.shell_click_id(team_node(Hit::Bubble(id)), "a bolha");
    for b in [TeamButton::CardResolve, TeamButton::CardClose] {
        found.push((format!("{b:?}"), t.find(team_node(Hit::Button(b))).unwrap()));
    }
    t.tab_menu(crate::ids::CTX_MENU_BOARD_EXPORT);
    for b in [
        TeamButton::ExportPng,
        TeamButton::ExportGo,
        TeamButton::ScopeFrames,
    ] {
        found.push((format!("{b:?}"), t.find(team_node(Hit::Button(b))).unwrap()));
    }
    for b in [NavButton::Present, NavButton::Comments] {
        found.push((format!("{b:?}"), t.find(button_node(b)).unwrap()));
    }
    let layout = t.hero.last_layout.unwrap();
    for (what, p) in found {
        assert!(
            layout.dock_seam_at(p).is_none(),
            "{what}: a costura da doca rouba-o"
        );
        assert!(
            layout.dock_reopen_at(p).is_none(),
            "{what}: o reabrir da doca rouba-o"
        );
        let band = t.hero.rulers_live() && crate::ruler::hit(t.hero.last_canvas, p).is_some();
        assert!(!band, "{what}: a faixa da régua rouba-o");
    }
}
