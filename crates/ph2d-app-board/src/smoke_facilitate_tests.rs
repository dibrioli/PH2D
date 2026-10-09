//! ⭐ A cena 9 ENSINA o que acontece (a lição da W5b). Cada nota-lição é conduzida pelo editor real com
//! as medidas da interface (`board_view::metrics`), na ORDEM REAL (a selecção e o documento herdam-se
//! de uma lição para a outra — a lição da W6), a zoom 1 e a 0,8.

use std::time::{Duration, Instant};

use super::*;
use ph2d_board_edit::{ClusterBy, ClusterNames, Editor, Mods, Pointer, Timer, TimerRun, VoteSetup};
use ph2d_board_model::History;
use ph2d_editor_core::screens::hero::board_view::{default_style, metrics};
use ph2d_text::TextSystem;

const VIEW: [f64; 4] = [-1e4, -1e4, 1e4, 1e4];
/// Px de ecrã por unidade do mundo: zoom 1 e 0,8.
const PXS: [f64; 2] = [1.0, 1.25];

struct Run {
    ed: Editor,
    doc: BoardDoc,
    h: History,
    ts: TextSystem,
    l: Lessons,
    px: f64,
}

impl Run {
    fn new(px: f64) -> Self {
        let mut doc = BoardDoc::default();
        let l = build(&mut doc);
        Self {
            ed: Editor::new(default_style(), metrics()),
            doc,
            h: History::default(),
            ts: TextSystem::without_system_fonts(),
            l,
            px,
        }
    }

    fn at(&self, p: [f64; 2], shift: bool) -> Pointer {
        Pointer {
            world: p,
            px: self.px,
            mods: Mods {
                shift,
                ..Mods::default()
            },
        }
    }

    fn click(&mut self, p: [f64; 2], shift: bool) {
        self.drag(p, p, shift);
    }

    fn drag(&mut self, a: [f64; 2], b: [f64; 2], shift: bool) {
        let p = self.at(a, shift);
        self.ed
            .pointer_down(&mut self.doc, &mut self.h, &mut self.ts, p, VIEW);
        for k in 1..=4 {
            let t = f64::from(k) / 4.0;
            let p = self.at([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t], shift);
            self.ed.pointer_move(&mut self.doc, &mut self.ts, p);
        }
        let p = self.at(b, shift);
        self.ed.pointer_up(&mut self.doc, &mut self.h, p);
    }

    fn el(&self, id: ElementId) -> &Element {
        self.doc.get(id).expect("existe")
    }

    fn text(&self, id: ElementId) -> &str {
        self.el(id).shape().unwrap().text.as_str()
    }

    fn corner(&self, id: ElementId) -> [f64; 2] {
        let e = self.el(id);
        [e.x + e.w, e.y + e.h]
    }
}

#[test]
fn every_facilitation_lesson_does_what_it_says() {
    for px in PXS {
        let mut r = Run::new(px);
        let [_l1, l2, l3, l4, l5] = r.l.notes;
        assert!(
            r.el(r.l.ideas[0]).votes() == 3 && r.el(r.l.ideas[3]).votes() == 1,
            "o resultado de antes"
        );
        // 1 — o cronómetro: `+` arma 6, Play corre, `+1m` soma, no zero soa.
        let mut t = Timer::default();
        assert_eq!(t.minutes(), 5, "o cartão abre armado a 5:00");
        t.step(true);
        let now = Instant::now();
        t.play(now);
        t.add(now, Duration::from_secs(60));
        assert_eq!(t.display(now), (7, 0));
        assert!(
            t.tick(now + Duration::from_secs(7 * 60)) && matches!(t.run(), TimerRun::Done { .. })
        );
        // 2 — votar: a sessão no quadro inteiro (nada seleccionado), o `+` no canto de uma nota.
        assert!(r.text(l2).contains("New vote") && r.text(l2).contains("Start voting"));
        assert!(r.ed.start_vote(&r.doc, VoteSetup::default()));
        let (a, b) = (r.l.ideas[2], r.l.ideas[4]);
        let pa = r.corner(a);
        r.click(pa, false);
        r.click(pa, false);
        assert_eq!(r.ed.vote().unwrap().mine(a), 2, "px {px}: o + vota");
        let minus =
            r.ed.vote_marks(&r.doc)
                .into_iter()
                .find(|m| m.id == a)
                .unwrap()
                .minus
                .unwrap()
                .0;
        r.click(minus, false);
        assert_eq!(r.ed.vote().unwrap().mine(a), 1, "px {px}: o − tira um");
        // 3 — os turnos e o fim.
        assert!(r.text(l3).contains("I'm done") && r.text(l3).contains("End for all"));
        assert!(r.ed.next_voter());
        assert_eq!(r.ed.vote().unwrap().mine(a), 0, "a seguinte começa do zero");
        let pb = r.corner(b);
        r.click(pb, false);
        r.click(pa, false);
        assert!(r.ed.end_vote(&mut r.doc, &mut r.h));
        assert_eq!((r.el(a).votes(), r.el(b).votes()), (2, 1));
        assert_eq!(
            r.el(r.l.ideas[0]).votes(),
            0,
            "o resultado de antes saiu do quadro"
        );
        // 4 — carimbos: seleccionar uma nota e carimbar; a ficha +1, Shift −1.
        assert!(r.text(l4).contains("Stamp") && r.text(l4).contains("Shift"));
        let n = r.l.ideas[6];
        r.ed.select(&r.doc, [n]);
        assert!(r.ed.stamp(&mut r.doc, &mut r.h, Stamp::Star));
        let e = r.el(n).clone();
        let chip = ph2d_board_model::stamp_chips(&e)[0].1;
        let c = [
            e.x + (chip[0] + chip[2]) / 2.0,
            e.y + (chip[1] + chip[3]) / 2.0,
        ];
        r.click(c, false);
        assert_eq!(
            r.el(n).stamps()[0].count,
            2,
            "px {px}: clicar na ficha soma"
        );
        r.click(c, true);
        assert_eq!(r.el(n).stamps()[0].count, 1, "px {px}: Shift tira");
        // 5 — agrupar: um clique no frame «Ideas» selecciona-o, e Agrupar leva as notas de dentro.
        assert!(r.text(l5).contains("Ideas") && r.text(l5).contains("By keyword"));
        r.ed.select(&r.doc, []);
        let f = r.el(r.l.ideas_frame).clone();
        r.click([f.x + 10.0 * px, f.y + 10.0 * px], false);
        assert_eq!(
            r.ed.selection().iter().copied().collect::<Vec<_>>(),
            [r.l.ideas_frame],
            "px {px}"
        );
        assert_eq!(
            r.ed.clusterable(&r.doc).len(),
            7,
            "px {px}: as sete ideias de dentro"
        );
        let names = ClusterNames {
            no_tag: "no tag".into(),
            uncategorized: "#uncategorized".into(),
        };
        assert!(r.ed.cluster(&mut r.doc, &mut r.h, ClusterBy::Keyword, &names));
        let mut titles: Vec<String> =
            r.ed.selection()
                .iter()
                .map(|id| r.el(*id).as_frame().unwrap().name.clone())
                .collect();
        titles.sort();
        assert_eq!(
            titles.len(),
            3,
            "px {px}: build · pair · o resto ({titles:?})"
        );
        assert!(titles.iter().any(|t| t.starts_with("#build")), "{titles:?}");
        assert!(titles.iter().any(|t| t.starts_with("#pair")), "{titles:?}");
        assert!(titles.contains(&"#uncategorized".to_owned()));
    }
}

/// A cena abre com o cartão do cronómetro (armado) e os selos da votação anterior à vista.
#[test]
fn the_scene_opens_with_the_timer_card() {
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    scene_facilitate(&mut hero);
    assert!(board_nav::facilitate::timer_open(&hero));
    let board = hero.documents.active_board().unwrap();
    assert_eq!(ph2d_board_edit::vote_range(&board.doc), Some([1, 3]));
}

/// ⭐ A lição 2 pelo PAINEL (auditoria W7: a cena nasce com um resultado, e o painel do resultado não
/// tinha como abrir uma votação nova — a lição ensinava um caminho que não existia): Voting mostra o
/// resultado, «New vote» abre a votação, «Start voting» começa-a; o fim dela tira o resultado antigo.
#[test]
fn the_vote_lesson_runs_from_the_panel() {
    use board_nav::facilitate::{FacButton, apply_event, fac_node, toggle_vote};
    use ph2d_editor_core::interaction::WidgetEvent;
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    scene_facilitate(&mut hero);
    toggle_vote(&mut hero);
    let click = |hero: &mut HeroScreen, b: FacButton| {
        assert!(apply_event(hero, WidgetEvent::Click(fac_node(b))), "{b:?}");
    };
    click(&mut hero, FacButton::VoteNew);
    click(&mut hero, FacButton::VoteStart);
    assert!(
        board_nav::facilitate::vote_in_session(&hero),
        "a votação abriu pelo painel"
    );
    click(&mut hero, FacButton::VoteEnd);
    let board = hero.documents.active_board().unwrap();
    assert_eq!(
        ph2d_board_edit::vote_range(&board.doc),
        None,
        "nenhum voto: o resultado antigo saiu"
    );
}
