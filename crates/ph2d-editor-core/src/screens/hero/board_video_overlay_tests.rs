//! W25 — a barra de um vídeo activo nunca fica por baixo do chrome do quadro (a barra curta, uma
//! gaveta, os painéis da direita): antes pintava-se ANTES deles e o clique ia para o botão de cima.
//! Os ajudantes são os do módulo pai, `board_video_tests.rs`.

use super::*;
use crate::screens::hero::board_nav::{self, NavButton};

const CTLS: [Ctl; 9] = [
    Ctl::Play,
    Ctl::Seek,
    Ctl::Mute,
    Ctl::Volume,
    Ctl::Fullscreen,
    Ctl::Outside,
    Ctl::Back,
    Ctl::Forward,
    Ctl::Reset,
];

/// ⭐ O vídeo activo levado a cada sítio do quadro, com a gaveta das formas ou os painéis da direita
/// abertos: nenhum botão da barra dele cai sobre outro clicável, e cada um é o que o clique alcança.
#[test]
fn the_video_bar_is_never_under_the_chrome() {
    let mut bad = Vec::new();
    let mut seen = 0;
    for which in ["formas", "direita"] {
        for w in [1366.0, 1024.0] {
            let mut t = T::new();
            t.vp_w = w;
            t.paint();
            let id = active(&mut t);
            if which == "formas" {
                t.hero.documents.live.shapes_open = true;
            } else {
                for b in [NavButton::Frames, NavButton::Comments, NavButton::Minimap] {
                    t.hero
                        .apply_event(WidgetEvent::Click(board_nav::button_node(b)));
                }
            }
            t.paint();
            let el = video(&t).unwrap();
            let home = t.hero.documents.active_board().unwrap().camera;
            for dx in [-0.45, -0.3, -0.15, 0.0, 0.15, 0.3, 0.45] {
                for dy in [-0.35, -0.2, 0.0, 0.2, 0.35] {
                    let c = t.hero.last_canvas;
                    let cam = &mut t.hero.documents.active_board_mut().unwrap().camera;
                    *cam = home;
                    cam.center_x = el.x + el.w / 2.0 - f64::from(dx * c.w) / home.zoom;
                    cam.center_y = el.y + el.h / 2.0 - f64::from(dy * c.h) / home.zoom;
                    t.paint();
                    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
                    let own: Vec<NodeId> = CTLS.iter().map(|c| board_video_bar::node(*c)).collect();
                    let mine: Vec<(Ctl, crate::zones::Rect)> = CTLS
                        .iter()
                        .filter_map(|c| {
                            t.hero
                                .hit_index
                                .rect_for(board_video_bar::node(*c))
                                .map(|r| (*c, r))
                        })
                        .collect();
                    if mine.is_empty() {
                        continue;
                    }
                    seen += 1;
                    for (c, r) in &mine {
                        let mid = (r.x + r.w / 2.0, r.y + r.h / 2.0);
                        if t.hero.hit_index.hit(mid.0, mid.1) != Some(board_video_bar::node(*c)) {
                            bad.push(format!("{which} {w} {dx},{dy}: {c:?} {r:?} tapado"));
                        }
                        for (who, o) in t.hero.hit_index.iter_registrations() {
                            let meet = r.x < o.x + o.w
                                && o.x < r.x + r.w
                                && r.y < o.y + o.h
                                && o.y < r.y + r.h;
                            if meet && !own.contains(&who) {
                                bad.push(format!(
                                    "{which} {w} {dx},{dy}: {c:?} {r:?} sobre {who:?} {o:?}"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(seen > 0, "a barra do vídeo nunca apareceu");
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
