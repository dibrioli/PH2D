//! Os gates da geometria do cartão lateral (F10): dentro da área, fora da seleção, espelhado na
//! borda direita, longe da ④ — pela MESMA conta que o pintor ([`super::super::tool_bars::placed_card`]).

use super::*;
use crate::interaction::{BarButton, BarFace, BarFamily};
use crate::screens::hero::float_bar::tool_bars::{blocks, placed, placed_card as placed_in_bars};

const AREA: Rect = Rect::new(300.0, 80.0, 1200.0, 800.0);

fn inside(a: Rect, r: Rect) -> bool {
    r.x >= a.x - 1e-3
        && r.y >= a.y - 1e-3
        && r.x + r.w <= a.x + a.w + 1e-3
        && r.y + r.h <= a.y + a.h + 1e-3
}

fn bars(sel: Rect, n_fields: usize, actions: usize) -> ToolBars {
    let b = |id| BarButton {
        id: NodeId(id),
        face: BarFace::Icon(crate::icons::IconId::Select),
        label: String::new(),
        on: false,
        family: BarFamily::Neutral,
    };
    let mut card: Vec<ToolRailEntry> = (0..n_fields)
        .map(|i| ToolRailEntry::number(NodeId(9_000 + i as u64), "Field", "Field", None))
        .collect();
    card.push(
        ToolRailEntry::compound(NodeId(9_100), "Viewed", "From above", "Viewed")
            .showing(["From above", "From below"]),
    );
    ToolBars {
        column: vec![vec![b(1), b(2), b(3)], vec![b(4)]],
        actions: vec![(0..actions as u64).map(|i| b(100 + i)).collect()],
        selection: Some(sel),
        card,
        ..ToolBars::default()
    }
}

/// A seleção varrida pela área, bordas e meio-fora incluídos.
fn selections() -> Vec<Rect> {
    let mut out = Vec::new();
    for fx in [-0.05_f32, 0.0, 0.1, 0.35, 0.5, 0.7, 0.85, 0.95] {
        for fy in [-0.05_f32, 0.0, 0.1, 0.4, 0.7, 0.9] {
            for (w, h) in [(60.0, 40.0), (180.0, 120.0)] {
                out.push(Rect::new(AREA.x + AREA.w * fx, AREA.y + AREA.h * fy, w, h));
            }
        }
    }
    out
}

/// Cada cartão colocado da varredura, com as barras e a lista do [`placed`].
fn sweep(mut each: impl FnMut(Rect, &ToolBars, &[(BarButton, Rect)], &PlacedCard)) -> usize {
    let mut ts = TextSystem::without_system_fonts();
    let mut n = 0;
    for sel in selections() {
        for (fields, actions) in [(1, 3), (4, 6), (7, 10)] {
            let b = bars(sel, fields, actions);
            let all = placed(&b, AREA, false, false, false);
            let card = placed_in_bars(&b, AREA, &all, RailButtonSize::Small, &mut ts)
                .expect("com campos e seleção há cartão");
            each(sel, &b, &all, &card);
            n += 1;
        }
    }
    n
}

#[test]
fn the_card_never_leaves_the_area() {
    let n = sweep(|sel, _, _, c| {
        assert!(
            inside(AREA, c.panel),
            "sel {sel:?}: o cartão {:?} sai da área",
            c.panel
        );
        assert!(inside(c.panel, c.content), "os chips saem do painel");
    });
    assert!(n >= 200, "controlo: só {n} casos");
}

#[test]
fn the_card_never_covers_the_selection_when_a_side_has_room() {
    sweep(|sel, _, _, c| {
        let room_r = AREA.x + AREA.w - (sel.x + sel.w) >= c.panel.w + card_gap();
        let room_l = sel.x - AREA.x >= c.panel.w + card_gap();
        if room_r || room_l {
            assert!(
                !overlaps(sel, c.panel),
                "sel {sel:?}: o cartão {:?} cobre-a",
                c.panel
            );
        }
    });
}

#[test]
fn the_card_sits_right_of_the_selection_and_mirrors_left_at_the_right_edge() {
    let mut ts = TextSystem::without_system_fonts();
    let mid = Rect::new(AREA.x + 400.0, AREA.y + 300.0, 120.0, 90.0);
    let b = bars(mid, 3, 0);
    let all = placed(&b, AREA, false, false, false);
    let c = placed_in_bars(&b, AREA, &all, RailButtonSize::Small, &mut ts).expect("cartão");
    assert!(
        (c.panel.x - (mid.x + mid.w + card_gap())).abs() < 1e-3,
        "à direita, com a folga"
    );
    assert!(
        (c.panel.y - mid.y).abs() < 1e-3,
        "alinhado ao topo da seleção"
    );
    let edge = Rect::new(AREA.x + AREA.w - 130.0, AREA.y + 300.0, 120.0, 90.0);
    let b = bars(edge, 3, 0);
    let all = placed(&b, AREA, false, false, false);
    let c = placed_in_bars(&b, AREA, &all, RailButtonSize::Small, &mut ts).expect("cartão");
    assert!(
        (c.panel.x + c.panel.w - (edge.x - card_gap())).abs() < 1e-3,
        "na borda direita espelha para a esquerda ({:?})",
        c.panel
    );
}

#[test]
fn the_card_never_collides_with_the_selection_actions_or_the_column() {
    let mut desviou = 0;
    sweep(|sel, b, all, c| {
        for r in blocks(b, all.len()) {
            if r.start >= r.end {
                continue;
            }
            let Some(p) = panel_of(all[r].iter().map(|(_, x)| *x)) else {
                continue;
            };
            assert!(
                !overlaps(p, c.panel),
                "sel {sel:?}: o cartão {:?} cobre a barra {p:?}",
                c.panel
            );
        }
        if card_rect(AREA, sel, [c.panel.w, c.panel.h], &[]) != c.panel {
            desviou += 1;
        }
    });
    // Controlo: a varredura tem casos em que o lugar ingénuo batia numa barra (a ④ cai por baixo
    // da seleção no topo da área) — sem eles o gate não mede o desvio.
    assert!(desviou > 0, "controlo: nenhum cartão precisou de desviar");
}

#[test]
fn the_card_puts_one_field_per_line() {
    let mut ts = TextSystem::without_system_fonts();
    let sel = Rect::new(AREA.x + 400.0, AREA.y + 300.0, 120.0, 90.0);
    let b = bars(sel, 4, 0);
    let all = placed(&b, AREA, false, false, false);
    let c = placed_in_bars(&b, AREA, &all, RailButtonSize::Small, &mut ts).expect("cartão");
    let slots = entry_rects(
        &c.rail,
        c.content,
        RailButtonSize::Small,
        RailAxis::Horizontal,
    );
    assert_eq!(slots.len(), 5);
    for w in slots.windows(2) {
        assert!(
            w[1].rect.y > w[0].rect.y,
            "dois campos na mesma linha: {w:?}"
        );
    }
    for s in &slots {
        assert!(
            inside(c.content, s.rect),
            "o campo {:?} sai do rect dos chips",
            s.rect
        );
    }
}

#[test]
fn no_fields_or_no_selection_means_no_card() {
    let mut ts = TextSystem::without_system_fonts();
    let sel = Rect::new(AREA.x + 400.0, AREA.y + 300.0, 120.0, 90.0);
    let mut b = bars(sel, 2, 0);
    b.selection = None;
    assert!(placed_in_bars(&b, AREA, &[], RailButtonSize::Small, &mut ts).is_none());
    let mut b = bars(sel, 0, 0);
    b.card.clear();
    assert!(placed_in_bars(&b, AREA, &[], RailButtonSize::Small, &mut ts).is_none());
}
