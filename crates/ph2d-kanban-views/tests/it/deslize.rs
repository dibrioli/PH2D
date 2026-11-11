//! InputMap onda 4 item 4: o deslize de lado do trackpad (`Views::wheel_sideways`) só mexe onde o conteúdo
//! rola na horizontal. MEDIDO nas áreas reais da aba (1600×1000, 1280×720) e numa estreita: o Calendário (os
//! quatro modos), o Dashboard e a gaveta da Linha do tempo arrumam-se à largura (nada passa da borda direita)
//! ⇒ o deslize não muda NADA (nem o vertical); controlo: a grelha da Linha do tempo rola de lado.

use crate::support::World;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::CivilDate;
use ph2d_kanban_views::calendar::Mode;
use ph2d_kanban_views::dashboard::Chart;
use ph2d_kanban_views::{ViewKind, Views, wheel_px};
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

const AREAS: [Rect; 3] = [
    Rect {
        x: 308.0,
        y: 28.0,
        w: 988.0,
        h: 890.0,
    },
    Rect {
        x: 297.0,
        y: 28.0,
        w: 690.0,
        h: 604.0,
    },
    Rect {
        x: 220.0,
        y: 28.0,
        w: 460.0,
        h: 560.0,
    },
];

fn pinta(v: &mut Views, w: &World, ts: &mut ph2d_text::TextSystem, a: Rect) {
    for _ in 0..2 {
        let cx = w.cx();
        v.paint(
            &mut VectorScene::new(),
            Affine::IDENTITY,
            a,
            &w.set,
            w.id,
            ts,
            Theme::Light,
            &cx,
        );
    }
}

/// O que a vista pintou (rects), para medir se passa da borda direita da área.
fn pecas(v: &Views) -> Vec<Rect> {
    let mut out = Vec::new();
    match v.kind {
        ViewKind::Calendar => {
            let dias = (2026..=2026)
                .flat_map(|y| (9..=12).flat_map(move |m| (1..=31).map(move |d| (y, m, d))));
            out.extend(dias.filter_map(|(y, m, d)| v.calendar.cell_rect(CivilDate::new(y, m, d)?)));
            out.extend(v.calendar.agenda_rows().into_iter().map(|r| r.0));
        }
        ViewKind::Dashboard => {
            for c in Chart::ALL {
                let n = v.dashboard.painted_bars(c).map_or(0, <[_]>::len);
                let ps = (0..n).filter_map(|i| v.dashboard.bar_at(c, i));
                out.extend(ps.map(|(x, y)| Rect::new(x, y, 0.0, 0.0)));
            }
        }
        _ => {
            let p = v.timeline.painted();
            out.extend(p.drawer);
            out.extend(p.drawer_cards.iter().map(|c| c.1));
        }
    }
    out
}

/// Desliza de lado nos dois sentidos sobre `p`; devolve se o estado inteiro (`Debug`) mudou.
fn desliza(v: &mut Views, w: &World, p: (f32, f32)) -> bool {
    let antes = format!("{v:?}");
    let cx = w.cx();
    let tomou =
        [-48.0, 48.0, -96.0].map(|dx| v.wheel_sideways(&w.set, w.id, &cx, p, wheel_px(dx)).taken);
    let mudou = format!("{v:?}") != antes;
    assert_eq!(tomou.contains(&true), mudou, "tomado = mexeu");
    mudou
}

#[test]
fn o_deslize_de_lado_so_mexe_onde_o_conteudo_rola_na_horizontal() {
    let (w, mut ts) = (
        World::scene1(),
        ph2d_text::TextSystem::without_system_fonts(),
    );
    let modos = [Mode::Month, Mode::Week, Mode::Day, Mode::Agenda];
    for a in AREAS {
        let mid = (a.x + a.w / 2.0, a.y + a.h / 2.0);
        for vista in modos
            .map(Some)
            .into_iter()
            .chain([None, Some(Mode::Month)])
            .enumerate()
        {
            let mut v = Views::default();
            v.kind = [ViewKind::Calendar; 4]
                .into_iter()
                .chain([ViewKind::Dashboard, ViewKind::Timeline])
                .nth(vista.0)
                .unwrap();
            v.calendar.mode = vista.1.unwrap_or_default();
            pinta(&mut v, &w, &mut ts, a);
            let ps = pecas(&v);
            let direita = ps.iter().map(|r| r.x + r.w).fold(f32::MIN, f32::max);
            eprintln!(
                "{:?} {:?} @{}: {} peças, borda direita {direita:.1} / {:.1}",
                v.kind,
                vista.1,
                a.w,
                ps.len(),
                a.x + a.w
            );
            assert!(
                !ps.is_empty() && direita <= a.x + a.w + 0.5,
                "{:?} cabe na largura {a:?}",
                v.kind
            );
            let sobre = match v.timeline.painted().drawer {
                Some(r) if v.kind == ViewKind::Timeline => (r.x + r.w / 2.0, r.y + r.h / 2.0),
                _ => mid,
            };
            assert!(
                !desliza(&mut v, &w, sobre),
                "{:?} {:?}: nada",
                v.kind,
                vista.1
            );
        }
        let mut v = Views::default();
        v.kind = ViewKind::Timeline;
        pinta(&mut v, &w, &mut ts, a);
        let g = v.timeline.painted().ruler.expect("régua").grid;
        let x0 = v.timeline.scroll_x;
        assert!(
            desliza(&mut v, &w, (g.x + g.w / 3.0, g.y + g.h / 2.0)),
            "controlo: a grelha rola @{}",
            a.w
        );
        assert!(v.timeline.scroll_x != x0, "controlo: o scroll x mudou");
    }
}
