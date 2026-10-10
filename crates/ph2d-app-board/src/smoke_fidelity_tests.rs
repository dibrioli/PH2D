//! ⭐ A cena 11 ENSINA o que acontece, e cada lição conduz-se pelo caminho que ela manda: a cena é a do
//! smoke (`scene_fidelity`, nunca uma cópia — lição da W8), o ficheiro da lição 2 abre pela MESMA porta
//! do menu da aba (`io::import_file`), e a seta da lição 4 desenha-se com a tecla `A` e o rato no QUADRO.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_board_model::{Anchor, STICKY_SIDE};
use ph2d_board_route::RouteCache;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use serde_json::Value;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn area(r: Rect) -> [f64; 4] {
    [
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.w),
        f64::from(r.h),
    ]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

struct Screen {
    hero: HeroScreen,
    ts: TextSystem,
    clock: u128,
    l: Lessons,
}

impl Screen {
    fn new() -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut hero = HeroScreen::new(NodeId(1));
        let l = scene_fidelity(&mut hero);
        let mut s = Self {
            hero,
            ts: TextSystem::without_system_fonts(),
            clock: 1_000_000_000_000,
            l,
        };
        s.paint();
        s.paint();
        s
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.ts);
    }

    fn board(&self) -> &ph2d_board_model::Board {
        self.hero.documents.active_board().unwrap()
    }

    /// A tecla `k` (com o texto que ela escreve, como a shell a entrega).
    fn board_key(&mut self, k: BoardKey, text: Option<&str>) {
        let m = Modifiers::default();
        board_keys::key(&mut self.hero, &mut self.ts, k, m, true, text, None);
        board_keys::key(&mut self.hero, &mut self.ts, k, m, false, None, None);
        self.paint();
    }

    /// Arrastar no QUADRO de `a` a `b` (mundo), com `mods` seguras depois de carregar (como a mão
    /// segura o `Ctrl` enquanto chega e larga).
    fn drag_board(&mut self, a: [f64; 2], b: [f64; 2], mods: Modifiers) {
        let r = area(self.hero.last_canvas);
        let cam = self.board().camera;
        let steps = 8;
        for i in 0..=steps + 1 {
            let (kind, t) = match i {
                0 => (PointerKind::Down, 0.0),
                i if i <= steps => (PointerKind::Move, f64::from(i) / f64::from(steps)),
                _ => (PointerKind::Up, 1.0),
            };
            let w = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            let p = cam.to_screen(r, w);
            let (x, y) = (p[0] as f32, p[1] as f32);
            assert!(self.hero.chrome_hit(x, y).is_none(), "nenhum controlo em {w:?}");
            self.clock += 50_000_000;
            let inp = Input {
                text: &mut self.ts,
                mods: if kind == PointerKind::Down {
                    Modifiers::default()
                } else {
                    mods
                },
                now_ns: self.clock,
            };
            // O caminho da shell: o mover tem porta própria.
            if kind == PointerKind::Move {
                board_view::pointer_move(&mut self.hero, inp, x, y);
            } else {
                board_view::pointer(&mut self.hero, inp, kind, PointerButton::Primary, x, y, true);
            }
        }
        self.paint();
    }
}

/// As notas-lição cabem (a altura de nascença) e a cena inteira está à vista ao abrir — a CAIXA dela,
/// com a margem do que os painéis tapam.
#[test]
fn the_lessons_fit_and_the_scene_is_in_view() {
    let s = Screen::new();
    let b = s.board();
    for id in s.l.lessons {
        assert_eq!(b.doc.get(id).unwrap().h, STICKY_SIDE, "a lição {} cresceu", id.0);
    }
    let a = area(s.hero.last_canvas);
    let v = s.l.view;
    for p in [[v[0], v[1]], [v[2], v[3]]] {
        let q = b.camera.to_screen(a, p);
        assert!(
            q[0] >= a[0] - 1.0
                && q[1] >= a[1] - 1.0
                && q[0] <= a[0] + a[2] + 1.0
                && q[1] <= a[1] + a[3] + 1.0,
            "a caixa da cena sai da área: {q:?} em {a:?}"
        );
    }
}

/// Os ficheiros que as lições mandam abrir EXISTEM (o caminho lido do próprio texto da lição).
#[test]
fn the_files_the_lessons_name_exist() {
    let mut seen = 0;
    for i in 1..=4 {
        let text = ph2d_i18n::tr(&format!("board.smoke.fidelity.l{i}"));
        for w in text.split_whitespace() {
            let path = w.trim_end_matches([',', '.', ';']);
            if path.starts_with("docs/") {
                seen += 1;
                assert!(
                    std::path::Path::new(&format!("{ROOT}/{path}")).is_file(),
                    "a lição {i} nomeia {path}, que não existe"
                );
            }
        }
    }
    assert_eq!(seen, 2, "a foto (lição 1) e o ficheiro (lição 2)");
}

/// ⭐ Lição 1 — o ficheiro do dono abre com as três setas RECTAS (a curva dele com dois pontos),
/// presas em órbita, e as pontas onde o excalidraw.com as gravou (≤ 0,05 px, mais o deslize da cena).
#[test]
fn lesson_1_the_owners_arrows_are_straight_and_stop_where_his_did() {
    let s = Screen::new();
    let b = s.board();
    let file: Value = serde_json::from_str(OWNER_FILE).unwrap();
    let theirs: Vec<&Value> = file["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "arrow")
        .collect();
    assert_eq!(s.l.owner.len(), 3);
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    for (a, id) in theirs.iter().zip(&s.l.owner) {
        let c = b.doc.get(*id).unwrap().connector().unwrap();
        assert_eq!(c.route, Route::Spline);
        assert!(c.waypoints.is_empty());
        let pts = a["points"].as_array().unwrap();
        let at = |k: usize| {
            [
                a["x"].as_f64().unwrap() + pts[k][0].as_f64().unwrap() + OWNER_SHIFT[0],
                a["y"].as_f64().unwrap() + pts[k][1].as_f64().unwrap() + OWNER_SHIFT[1],
            ]
        };
        let r = routes.get(*id).unwrap();
        for (k, want) in [(0, at(0)), (1, at(pts.len() - 1))] {
            assert!(
                matches!(c.ends()[k], End::Bound { anchor: Anchor::Orbit(_), .. }),
                "{:?}",
                c.ends()[k]
            );
            let e = dist(r.ends()[k], want);
            assert!(e <= 0.05, "{}: {e:.3}", a["id"]);
        }
    }
}

/// ⭐ Lição 2 — o MESMO ficheiro pela porta do menu da aba (`io::import_file`) dá as mesmas setas.
#[test]
fn lesson_2_the_tab_menu_opens_the_same_drawing() {
    let s = Screen::new();
    let path = std::path::PathBuf::from(format!(
        "{ROOT}/docs/MiroClone/bv.excalidraw"
    ));
    let (name, imp) = crate::io::import_file(&path).unwrap();
    assert_eq!(name, "bv");
    let opened: Vec<&Element> = imp
        .doc
        .live_in_z_order()
        .into_iter()
        .filter(|e| e.connector().is_some())
        .collect();
    assert_eq!(opened.len(), 3);
    for (e, id) in opened.iter().zip(&s.l.owner) {
        assert_eq!(e.connector(), s.board().doc.get(*id).unwrap().connector());
    }
}

/// ⭐ Lição 3 — cada seta curta TEM ponta, e ela mede `min(25, L/2)` (a profundidade da risca dele a
/// 20°). ⛔ Antes da W9, as de 10 e 20 ficavam sem ponta.
#[test]
fn lesson_3_every_short_arrow_has_a_head_half_its_length() {
    let s = Screen::new();
    let b = s.board();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    for (id, l) in s.l.short.iter().zip(SHORT) {
        let c = b.doc.get(*id).unwrap().connector().unwrap();
        let r = routes.get(*id).unwrap();
        let d = ph2d_board_route::drawn(r, c.heads, c.style.stroke_width);
        assert_eq!(d.heads.len(), 1, "a seta de {l} tem ponta");
        let p = &d.heads[0].0;
        let els = p.elements();
        let first = match els[0] {
            ph2d_vector::PathEl::MoveTo(q) => q,
            _ => panic!("a ponta começa por um MoveTo"),
        };
        let tip = r.ends()[1];
        let depth = tip[0] - first.x;
        let want = 25f64.min(l / 2.0) * 20f64.to_radians().cos();
        assert!((depth - want).abs() < 1e-6, "L {l}: {depth} contra {want}");
    }
}

/// ⭐ Lição 4 — `A` e arrastar de «Start here» para DENTRO de «Drop inside»: a seta presa às duas pára a
/// `5 + traço/2` da caixa (o vão do Excalidraw, decisão do dono 10/10); com `Ctrl` ao largar, solta.
#[test]
fn lesson_4_a_drawn_arrow_stops_before_the_box_and_ctrl_leaves_it_loose() {
    let mut s = Screen::new();
    s.board_key(BoardKey::Char('a'), Some("a"));
    let (from, to) = {
        let b = s.board();
        (
            b.doc.get(s.l.from).unwrap().center(),
            b.doc.get(s.l.to).unwrap().center(),
        )
    };
    let before = s.board().doc.live_len();
    s.drag_board(from, to, Modifiers::default());
    let b = s.board();
    assert_eq!(b.doc.live_len(), before + 1, "uma seta nova");
    let arrow = b
        .doc
        .live_in_z_order()
        .into_iter()
        .rev()
        .find(|e| e.connector().is_some())
        .unwrap();
    let c = arrow.connector().unwrap();
    assert!(matches!(c.end, End::Bound { target, .. } if target == s.l.to), "{:?}", c.end);
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let tip = routes.get(arrow.id).unwrap().ends()[1];
    let to_el = b.doc.get(s.l.to).unwrap();
    let (_, d) = ph2d_board_geom::nearest_on_outline(to_el, tip).unwrap();
    let g = ph2d_board_route::gap(to_el);
    assert!(!ph2d_board_geom::inside(to_el, tip), "a ponta fica FORA da caixa");
    assert!((d - g).abs() < 1e-3, "pára a {d:.3} da caixa, a lei diz {g}");
    // Com Ctrl ao largar: solta.
    s.board_key(BoardKey::Char('a'), Some("a"));
    s.drag_board(from, to, Modifiers { ctrl: true, ..Modifiers::default() });
    let loose = s
        .board()
        .doc
        .live_in_z_order()
        .into_iter()
        .rev()
        .find(|e| e.connector().is_some())
        .unwrap()
        .connector()
        .unwrap()
        .end;
    assert!(matches!(loose, End::Free(_)), "{loose:?}");
}
