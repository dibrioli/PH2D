//! Facilitar (W7) pelo caminho REAL do ecrã — `docs/MiroClone/05_facilitacao.md` §2: os botões da barra
//! curta e da de contexto pela ordem da shell (`shell_click_bar`), o cartão e o painel pelo despacho
//! sobre o que o pintor registou (`shell_click_id`), o `+` de votar pelo rato na porta da shell.

use std::time::{Duration, Instant};

use super::*;
use crate::screens::hero::board_bar::fac::{FacItem, TIMER, VOTE};
use crate::screens::hero::board_nav;
use crate::screens::hero::board_nav::facilitate::{FacButton, column, fac_node};
use ph2d_board_edit::{Timer, TimerRun};
use ph2d_board_model::{BoardOp, ElementId, Rgba, RichText, STICKY_COLORS, Shape, Stamp, Style};

impl T {
    /// Uma nota de 199 no mundo em `at`, posta no quadro activo por uma operação.
    fn put_note(&mut self, at: [f64; 2], text: &str, color: usize) -> ElementId {
        let (board, live) = self.hero.documents.active_parts().unwrap();
        let ink = Rgba([20, 20, 20, 255]);
        let el = ph2d_board_model::Element::new_shape(
            board.doc.mint_id(),
            board.doc.z_on_top(),
            Shape {
                kind: ShapeType::Sticky,
                style: Style::new(Some(Rgba(STICKY_COLORS[color])), None, ink),
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

    fn fac(&mut self, b: FacButton) {
        self.shell_click_id(fac_node(b), &format!("{b:?}"));
    }

    fn board_id(&self) -> ph2d_board_model::BoardId {
        self.hero.documents.active_board().unwrap().id
    }

    fn timer(&self) -> Option<Timer> {
        self.hero
            .documents
            .live
            .fac
            .timers
            .get(&self.board_id())
            .cloned()
    }

    fn timer_mut(&mut self) -> &mut Timer {
        let id = self.board_id();
        self.hero.documents.live.fac.timers.get_mut(&id).unwrap()
    }

    fn elem(&self, id: ElementId) -> ph2d_board_model::Element {
        self.hero
            .documents
            .active_board()
            .unwrap()
            .doc
            .get(id)
            .unwrap()
            .clone()
    }

    /// Um clique no quadro no ponto `w` do mundo, pela porta da shell.
    fn click_world(&mut self, w: [f64; 2]) {
        let b = self.hero.documents.active_board().unwrap();
        let a = board_view::area_of(self.hero.last_canvas);
        let p = b.camera.to_screen(a, w);
        let p = (p[0] as f32, p[1] as f32);
        self.drag(p, p);
    }

    fn pick(&mut self, ids: &[ElementId]) {
        board_view::select(&mut self.hero, ids.iter().copied());
        self.paint();
    }

    fn vote_mine(&self, id: ElementId) -> u32 {
        let ed = self.hero.documents.live.editor.as_ref().unwrap();
        ed.vote().map_or(0, |s| s.mine(id))
    }
}

/// ⭐ F2/F4/F5 pelo ecrã: o botão Timer abre o cartão armado a 5:00; `+` arma 6; ▶ corre; `+1m` soma;
/// ⏸ pausa; ■ pergunta e «Stop timer» repõe.
#[test]
fn the_timer_card_opens_runs_and_stops() {
    let mut t = T::new();
    t.shell_click_bar(TIMER);
    assert_eq!(t.timer().unwrap().minutes(), 5);
    t.fac(FacButton::TimerPlus);
    assert_eq!(t.timer().unwrap().minutes(), 6);
    t.fac(FacButton::TimerPlay);
    assert!(matches!(t.timer().unwrap().run(), TimerRun::Running { .. }));
    assert!(
        t.find(fac_node(FacButton::TimerMinus)).is_none(),
        "a correr, o ± não se pinta"
    );
    t.fac(FacButton::TimerAdd1);
    let left = t.timer().unwrap().left(Instant::now());
    assert!(
        left > Duration::from_secs(6 * 60 + 50),
        "6 + 1 minutos: {left:?}"
    );
    t.fac(FacButton::TimerPause);
    assert!(matches!(t.timer().unwrap().run(), TimerRun::Paused { .. }));
    t.fac(FacButton::TimerStop);
    assert!(t.timer().unwrap().confirm_stop, "o ■ pergunta");
    t.fac(FacButton::TimerConfirmStop);
    assert_eq!(t.timer().unwrap().run(), TimerRun::Set);
    t.fac(FacButton::TimerMinimize);
    assert!(
        t.find(fac_node(FacButton::TimerExpand)).is_some(),
        "a pílula"
    );
    t.fac(FacButton::TimerExpand);
    t.shell_click_bar(TIMER);
    assert!(t.timer().is_none(), "o mesmo botão fecha o cartão");
}

/// ⭐ F6: no zero a campainha é pedida UMA vez (a shell toca-a); com o altifalante desligado, nenhuma.
#[test]
fn at_zero_the_card_rings_once() {
    let mut t = T::new();
    t.shell_click_bar(TIMER);
    t.timer_mut()
        .play(Instant::now() - Duration::from_secs(301));
    t.paint();
    assert!(matches!(t.timer().unwrap().run(), TimerRun::Done { .. }));
    assert!(t.hero.documents.take_chime(), "soa no zero");
    t.paint();
    assert!(!t.hero.documents.take_chime(), "e só uma vez");
    t.fac(FacButton::TimerSound);
    t.timer_mut()
        .play(Instant::now() - Duration::from_secs(301));
    t.paint();
    assert!(
        !t.hero.documents.take_chime(),
        "com o altifalante desligado, não soa"
    );
}

/// ⭐ F7: o cartão e o painel moram no canto de cima à direita e NUNCA por cima do painel de Frames.
#[test]
fn the_timer_card_never_covers_the_frames_panel() {
    let mut t = T::new();
    t.shell_click_bar(TIMER);
    t.shell_click_bar(VOTE);
    for frames in [false, true] {
        t.hero.documents.live.nav.frames_open = frames;
        t.paint();
        let (x, _, w) = column(t.hero.last_canvas, frames, false);
        let c = t.hero.last_canvas;
        let edge = c.x + c.w - ph2d_tokens::Spacing::Md.px();
        let frames_x = edge
            - super::super::super::board_nav::frames_panel_width()
            - ph2d_tokens::Spacing::Md.px();
        assert!(
            x + w <= if frames { frames_x } else { edge } + 0.5,
            "frames={frames}"
        );
        for b in [
            FacButton::TimerClose,
            FacButton::VoteClose,
            FacButton::TimerPlay,
        ] {
            let (px, _) = t.find(fac_node(b)).unwrap_or_else(|| panic!("{b:?}"));
            assert!(
                !frames || px < frames_x,
                "{b:?} por cima do painel de Frames"
            );
        }
    }
}

/// ⭐ V1–V4 pelo ecrã: o painel abre na abertura; 2 votos; o `+` no canto da nota vota; sem votos é o
/// aviso; «I'm done» passa à seguinte; «End for all» escreve os totais num passo e mostra o resultado;
/// com a sessão aberta o resultado anterior esconde-se.
#[test]
fn a_vote_runs_from_the_panel_to_the_result() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "Faster builds", 1);
    let b = t.put_note([-100.0, -100.0], "Pair more", 2);
    t.shell_click_bar(VOTE);
    t.fac(FacButton::VotesMinus);
    t.fac(FacButton::VoteStart);
    assert!(t.hero.documents.live.render_cache.hide_votes);
    let corner = |e: &ph2d_board_model::Element| [e.x + e.w, e.y + e.h];
    let pa = corner(&t.elem(a));
    t.click_world(pa);
    t.click_world(pa);
    assert_eq!(t.vote_mine(a), 2);
    t.click_world(pa);
    assert_eq!(
        t.hero.documents.live.notice.map(|n| n.0),
        Some("board.notice.no_votes_left")
    );
    t.fac(FacButton::VoteDone);
    assert_eq!(
        t.vote_mine(a),
        0,
        "a pessoa seguinte não vê os votos da anterior"
    );
    let pb = corner(&t.elem(b));
    t.click_world(pb);
    t.fac(FacButton::VoteEnd);
    assert_eq!((t.elem(a).votes(), t.elem(b).votes()), (2, 1));
    assert!(
        !t.hero.documents.live.render_cache.hide_votes,
        "acabada, o resultado volta a ver-se"
    );
    assert!(
        t.find(fac_node(FacButton::VoteClear)).is_some(),
        "o resultado"
    );
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(t.elem(a).votes(), 0, "UM passo de desfazer");
}

/// ⭐ V6: os minutos da votação armam o cronómetro do quadro, e o zero dele acaba a votação.
#[test]
fn the_clock_ends_the_vote() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "idea", 1);
    t.shell_click_bar(VOTE);
    t.fac(FacButton::MinutesPlus);
    t.fac(FacButton::VoteStart);
    assert!(
        matches!(t.timer().unwrap().run(), TimerRun::Running { .. }),
        "o relógio arrancou"
    );
    let pa = {
        let e = t.elem(a);
        [e.x + e.w, e.y + e.h]
    };
    t.click_world(pa);
    *t.timer_mut() = Timer::with_minutes(1);
    t.timer_mut().play(Instant::now() - Duration::from_secs(61));
    t.paint();
    assert_eq!(
        t.elem(a).votes(),
        1,
        "o zero acabou a votação e escreveu o total"
    );
    assert!(t.find(fac_node(FacButton::VoteClear)).is_some());
}

/// ⭐ A sessão é do SEU quadro: noutra aba não há `+`; ao voltar, ela volta.
#[test]
fn a_vote_session_stays_with_its_board() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "idea", 1);
    t.shell_click_bar(VOTE);
    t.fac(FacButton::VoteStart);
    let first = t.board_id();
    t.hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_NEW));
    t.paint();
    assert!(
        t.hero
            .documents
            .live
            .editor
            .as_ref()
            .unwrap()
            .vote()
            .is_none()
    );
    t.hero.documents.activate(Some(first));
    t.paint();
    let pa = {
        let e = t.elem(a);
        [e.x + e.w, e.y + e.h]
    };
    t.click_world(pa);
    assert_eq!(t.vote_mine(a), 1, "a sessão voltou com o quadro");
}

/// ⭐ S2 e C1 pela barra de contexto: com UMA nota há Carimbo e não há Agrupar (controlo morto); a
/// galeria dá o carimbo e fecha-se; com duas, Agrupar › By color faz os frames.
#[test]
fn stamp_and_cluster_from_the_note_bar() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "a", 1);
    let b = t.put_note([-100.0, -100.0], "b", 2);
    t.pick(&[a]);
    assert!(
        t.find(node(Item::Fac(FacItem::Cluster))).is_none(),
        "uma nota não se agrupa"
    );
    t.shell_click_bar(Item::Fac(FacItem::Stamps));
    t.shell_click_bar(Item::Fac(FacItem::Stamp(2)));
    assert_eq!(t.elem(a).stamps()[0].stamp, Stamp::Heart);
    assert!(
        t.find(node(Item::Fac(FacItem::Stamp(2)))).is_none(),
        "a galeria fecha-se"
    );
    t.pick(&[a, b]);
    t.shell_click_bar(Item::Fac(FacItem::Cluster));
    t.shell_click_bar(Item::Fac(FacItem::ClusterBy(0)));
    let frames = t
        .elements()
        .iter()
        .filter(|e| e.as_frame().is_some())
        .count();
    assert_eq!(frames, 2, "duas cores, dois frames");
}

/// ⭐ Nenhum botão da facilitação mora numa zona que a SHELL trata antes (o gate da W6, alargado).
#[test]
fn no_shell_gesture_zone_sits_on_a_facilitation_button() {
    let mut t = T::new();
    t.hero.view.rulers_visible = true;
    let a = t.put_note([-400.0, -100.0], "a", 1);
    t.shell_click_bar(TIMER);
    t.shell_click_bar(VOTE);
    let mut found = Vec::new();
    for b in [
        FacButton::TimerPlus,
        FacButton::TimerPlay,
        FacButton::TimerClose,
        FacButton::VoteStart,
        FacButton::VotesPlus,
    ] {
        found.push((format!("{b:?}"), t.find(fac_node(b)).unwrap()));
    }
    t.pick(&[a]);
    t.shell_click_bar(Item::Fac(FacItem::Stamps));
    for i in 0..Stamp::ALL.len() {
        let it = Item::Fac(FacItem::Stamp(i));
        found.push((format!("{it:?}"), t.find(node(it)).unwrap()));
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

// ─── Curas da auditoria W7 ──────────────────────────────────────────────────────────────────────

/// Carregar outro projecto (os ids de quadro recomeçam) e apagar um quadro levam a facilitação dele:
/// nada cai sobre o quadro novo com o mesmo id, e a campainha não toca por um quadro que já não existe.
#[test]
fn loading_or_deleting_a_board_takes_its_facilitation() {
    let mut t = T::new();
    t.put_note([-400.0, -100.0], "idea", 1);
    t.shell_click_bar(TIMER);
    t.shell_click_bar(VOTE);
    t.fac(FacButton::VoteStart);
    let mut set = ph2d_board_model::BoardSet::default();
    let id = set.create("other".into());
    crate::screens::hero::document_tabs::load(&mut t.hero, set);
    t.hero.documents.activate(Some(id));
    t.paint();
    assert!(t.timer().is_none() && !board_nav::facilitate::vote_in_session(&t.hero));
    t.shell_click_bar(TIMER);
    t.timer_mut()
        .play(Instant::now() - Duration::from_secs(301));
    t.hero.documents.remove(id);
    t.paint();
    assert!(!t.hero.documents.take_chime(), "o quadro apagado não toca");
}

/// F1/F6: com a aba Cena activa, o cronómetro de um quadro chega a zero e TOCA.
#[test]
fn the_chime_rings_with_the_scene_tab_open() {
    let mut t = T::new();
    t.shell_click_bar(TIMER);
    t.timer_mut()
        .play(Instant::now() - Duration::from_secs(301));
    t.hero.documents.activate(None);
    t.paint();
    assert!(t.hero.documents.take_chime());
}

/// V6 + desfazer: o zero do relógio NÃO acaba a votação a meio de um arrasto (o desfazer dele
/// guardaria o antes); acaba logo a seguir.
#[test]
fn the_clock_waits_for_the_gesture_to_end() {
    let mut t = T::new();
    let a = t.put_note([-400.0, -100.0], "idea", 1);
    t.shell_click_bar(VOTE);
    t.fac(FacButton::MinutesPlus);
    t.fac(FacButton::VoteStart);
    let pa = {
        let e = t.elem(a);
        [e.x + e.w, e.y + e.h]
    };
    t.click_world(pa);
    *t.timer_mut() = Timer::with_minutes(1);
    t.timer_mut().play(Instant::now() - Duration::from_secs(61));
    let b = t.hero.documents.active_board().unwrap();
    let area = board_view::area_of(t.hero.last_canvas);
    let s = b.camera.to_screen(area, [-300.0, 0.0]);
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    let p = PointerButton::Primary;
    board_view::pointer(
        h,
        inp(ts, c),
        PointerKind::Down,
        p,
        s[0] as f32,
        s[1] as f32,
        true,
    );
    board_view::pointer_move(h, inp(ts, c), s[0] as f32 + 40.0, s[1] as f32);
    t.paint();
    assert!(
        board_nav::facilitate::vote_in_session(&t.hero),
        "a meio do gesto, ainda aberta"
    );
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    board_view::pointer(
        h,
        inp(ts, c),
        PointerKind::Up,
        p,
        s[0] as f32 + 40.0,
        s[1] as f32,
        true,
    );
    t.paint();
    assert!(!board_nav::facilitate::vote_in_session(&t.hero));
    assert_eq!(t.elem(a).votes(), 1);
}

/// V6: o ■ confirmado larga o relógio da votação (o cartão volta a ser só um cronómetro).
#[test]
fn stopping_the_timer_releases_the_vote() {
    let mut t = T::new();
    t.put_note([-400.0, -100.0], "idea", 1);
    t.shell_click_bar(VOTE);
    t.fac(FacButton::MinutesPlus);
    t.fac(FacButton::VoteStart);
    assert!(t.find(fac_node(FacButton::VoteAdd1)).is_some());
    t.fac(FacButton::TimerStop);
    t.fac(FacButton::TimerConfirmStop);
    assert!(
        t.find(fac_node(FacButton::VoteAdd1)).is_none(),
        "sem relógio, sem «+1 min»"
    );
    t.timer_mut()
        .play(Instant::now() - Duration::from_secs(400));
    t.paint();
    assert!(
        board_nav::facilitate::vote_in_session(&t.hero),
        "o zero já não a acaba"
    );
}

/// V4: o resultado oferece «New vote»; «See all results» nunca sai da área do quadro.
#[test]
fn the_result_panel_starts_a_new_vote_and_fits_the_area() {
    let mut t = T::new();
    let ids: Vec<ElementId> = (0..60)
        .map(|i| {
            t.put_note(
                [
                    f64::from(i % 10) * 220.0 - 1200.0,
                    f64::from(i / 10) * 220.0,
                ],
                "x",
                1,
            )
        })
        .collect();
    for (i, id) in ids.iter().enumerate() {
        let (board, live) = t.hero.documents.active_parts().unwrap();
        let mut el = board.doc.get(*id).unwrap().clone();
        el.set_votes(1 + i as u32);
        let h = live.histories.entry(board.id).or_default();
        h.apply(&mut board.doc, vec![BoardOp::Put(el)]);
    }
    t.shell_click_bar(VOTE);
    t.fac(FacButton::VoteSeeAll);
    let c = t.hero.last_canvas;
    for b in [
        FacButton::VoteSeeAll,
        FacButton::VoteClear,
        FacButton::VoteNew,
    ] {
        let (_, y) = t.find(fac_node(b)).unwrap_or_else(|| panic!("{b:?}"));
        assert!(y < c.y + c.h, "{b:?} fora da área");
    }
    t.fac(FacButton::VoteNew);
    assert!(t.find(fac_node(FacButton::VoteStart)).is_some());
}

/// O fundo do cartão toma o clique: com a ferramenta Nota, clicar no mostrador não faz uma nota.
#[test]
fn a_click_on_the_card_never_reaches_the_board() {
    let mut t = T::new();
    t.shell_click_bar(TIMER);
    t.shell_click_bar(Item::Tool(Tool::Shape(ShapeType::Sticky)));
    let (x, y) = t
        .find(fac_node(FacButton::Surface))
        .expect("o fundo está registado");
    let before = t.elements().len();
    let arena = Bump::new();
    let on_canvas = t.hero.chrome_panel_at(x, y).is_none() && t.hero.chrome_hit(x, y).is_none();
    assert!(!on_canvas, "o cartão não é quadro");
    for kind in [PointerKind::Down, PointerKind::Up] {
        for e in t.hero.handle_pointer(ptr(kind, x, y), &arena).to_vec() {
            t.hero.apply_event(e);
        }
    }
    t.paint();
    assert_eq!(t.elements().len(), before);
}

/// F7: com a busca aberta, a coluna desce para baixo dela; nos limites, o ± que não mexeria não se
/// pinta (controlo morto).
#[test]
fn the_card_avoids_the_search_and_hides_dead_steps() {
    let mut t = T::new();
    t.shell_click_bar(TIMER);
    let (_, y0) = t.find(fac_node(FacButton::TimerPlay)).unwrap();
    t.hero.documents.live.nav.search_open = true;
    t.paint();
    let (_, y1) = t.find(fac_node(FacButton::TimerPlay)).unwrap();
    assert!(y1 > y0 + btn_px() * 2.0, "{y0} → {y1}");
    t.hero.documents.live.nav.search_open = false;
    for _ in 0..4 {
        t.fac(FacButton::TimerMinus);
    }
    assert_eq!(t.timer().unwrap().minutes(), 1);
    assert!(
        t.find(fac_node(FacButton::TimerMinus)).is_none(),
        "a 1 minuto, sem −"
    );
}
