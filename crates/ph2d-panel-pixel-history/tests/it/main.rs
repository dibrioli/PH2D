//! A janela UNDO HISTORY por gesto real: cada linha pintada responde no seu rect e pede ir até ela.

use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_history::canal::{self, HistoryRequest, HistoryView, Row};
use ph2d_panel_pixel_history::{PixelHistoryPanel, PixelHistoryPanelState, ids};
use ph2d_pixel_core::StepState::{Done, Undone};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 1400.0);
const BAIXA: Rect = Rect::new(0.0, 0.0, 1600.0, 360.0);

fn vista(n: usize, current: usize) -> HistoryView {
    let rows = (0..n)
        .map(|i| Row {
            label: if i == 0 {
                "pixel.history.initial"
            } else {
                "pixel.history.pencil"
            },
            state: if i <= current { Done } else { Undone },
        })
        .collect();
    HistoryView {
        rows,
        current,
        bytes: 5530,
    }
}

fn mesa(v: HistoryView, janela: Rect) -> Mesa<PixelHistoryPanel> {
    let _ = canal::take_requests();
    canal::publish_view(v);
    let mut m = Mesa::nova(PixelHistoryPanelState);
    m.host.set_panel_visible(PixelHistoryPanel::ID, true);
    m.assenta_em(janela);
    m
}

#[test]
fn history_window_rows_answer_a_real_pointer() {
    let mut m = mesa(vista(5, 2), JANELA);
    let mut fundos = Vec::new();
    for i in 0..5 {
        let r = m.rect(ids::row(i)).expect("linha pintada");
        fundos.push(r.y);
        let _ = m.clica(ids::row(i));
        m.assenta_em(JANELA);
        assert_eq!(canal::take_requests(), vec![HistoryRequest::GoTo(i)]);
    }
    assert!(
        fundos.windows(2).all(|w| w[0] < w[1]),
        "as linhas não descem: {fundos:?}"
    );
}

#[test]
fn history_window_hides_without_rows() {
    let m = mesa(HistoryView::default(), JANELA);
    assert!(m.rect(ids::row(0)).is_none());
    // Fechada não se prova aqui: a mesa reabre o painel a cada quadro (`paint_with_layout`).
}

/// Uma lista longa numa janela baixa: a primeira linha responde e a última fica fora (rola).
#[test]
fn history_window_long_list_scrolls() {
    let m = mesa(vista(60, 59), BAIXA);
    let primeira = m.rect(ids::row(0)).expect("a primeira");
    assert!(primeira.y < BAIXA.h);
    assert!(
        m.rect(ids::row(59))
            .is_none_or(|r| r.y >= BAIXA.h || r.y + r.h > BAIXA.h),
        "60 linhas couberam em {} px",
        BAIXA.h
    );
}

#[test]
fn history_window_title_shows_pretty_size() {
    assert_eq!(canal::title(0), "Undo History (0 bytes)");
    assert_eq!(canal::title(5530), "Undo History (5.4K)");
    assert_eq!(canal::title(1100), "Undo History (1.1K)");
    assert_eq!(canal::title(1_572_864), "Undo History (1.5M)");
}

#[test]
fn history_view_of_follows_the_core() {
    use ph2d_pixel_core::{Background, ColorMode, History, NewSprite, Paint, Sprite, Stroke};
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut h = History::default();
    for y in 0..3 {
        let mut st = Stroke::begin(&s, l, f, 0xff00_00ff, Paint::default(), (0, y));
        st.to(&s, 7, y);
        let op = st.op(&s).expect("op");
        h.commit_labeled(
            &mut s,
            op,
            ph2d_pixel_core::Label::new("pixel.history.pencil"),
        )
        .expect("commit");
    }
    assert!(h.undo(&mut s));
    let v = HistoryView::of(&h);
    assert_eq!(v.current, 2);
    assert_eq!(v.rows.len(), 4);
    assert_eq!(v.rows[3].state, Undone);
    assert_eq!(v.bytes, h.bytes());
}
