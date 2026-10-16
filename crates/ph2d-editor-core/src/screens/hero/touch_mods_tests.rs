//! Os gates do canhoto e do widget dos modificadores (F5).

use super::*;
use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{BarButton, BarFace, BarFamily, ToolBars, WidgetEvent};

const AREA: Rect = Rect::new(300.0, 80.0, 1200.0, 800.0);

fn bars() -> ToolBars {
    let b = |id| BarButton {
        id: crate::NodeId(id),
        face: BarFace::Icon(crate::icons::IconId::Select),
        label: String::new(),
        on: false,
        family: BarFamily::Draw,
    };
    ToolBars {
        column: vec![vec![b(1), b(2)], vec![b(3)]],
        ..ToolBars::default()
    }
}

/// ⭐ **Canhoto: a coluna ① e o widget passam para a metade DIREITA da área** — e, o controlo,
/// destro ficam na esquerda.
#[test]
fn left_handed_moves_the_column_and_the_widget_to_the_right() {
    let mid = AREA.x + AREA.w * 0.5;
    for (left_handed, right_side) in [(true, true), (false, false)] {
        let col =
            crate::screens::hero::float_bar::tool_bars::placed(&bars(), AREA, false, left_handed);
        assert!(!col.is_empty());
        for (_, r) in &col {
            assert_eq!(
                r.x > mid,
                right_side,
                "canhoto={left_handed}: coluna em {r:?}"
            );
        }
        for r in rects(AREA, true, left_handed) {
            assert_eq!(
                r.x > mid,
                right_side,
                "canhoto={left_handed}: widget em {r:?}"
            );
        }
    }
}

/// **O widget não tapa a coluna ①** (ela desce até ao fundo num iPad mini).
#[test]
fn the_widget_sits_beside_the_column_never_under_it() {
    let col = crate::screens::hero::float_bar::tool_bars::placed(&bars(), AREA, false, false);
    let right = col.iter().map(|(_, r)| r.x + r.w).fold(0.0, f32::max);
    for r in rects(AREA, true, false) {
        assert!(
            r.x >= right,
            "o widget ({r:?}) começa antes do fim da coluna ({right})"
        );
    }
}

/// ⭐ **Um clique num botão trava e outro destrava; *Ver ▸ Modo foco* pede o foco à shell.**
#[test]
fn a_click_locks_a_modifier_and_the_focus_row_asks_for_focus() {
    let mut hero = HeroScreen::new(crate::NodeId(1));
    assert!(
        !hero.store.touch().widget_visible(),
        "nasceu à vista sem toque"
    );
    assert!(hero.apply_event(WidgetEvent::Click(ids::TOUCH_MOD_SHIFT)));
    assert!(hero.store.touch().locked().shift, "o clique não travou");
    assert!(hero.store.touch().widget_visible(), "travado e escondido");
    assert!(hero.apply_event(WidgetEvent::Click(ids::TOUCH_MOD_SHIFT)));
    assert!(
        !hero.store.touch().locked().shift,
        "o 2.º clique não destravou"
    );
    let _ = hero.bus.drain().count();
    assert!(hero.apply_event(WidgetEvent::Click(ids::MENUBAR_VIEW_FOCUS)));
    assert!(
        hero.bus
            .drain()
            .any(|a| matches!(a, EditorAction::FocusMode)),
        "a linha do modo foco não chegou à shell"
    );
}

/// ⭐ **F8: *Ver ▸ Enquadrar tudo / seleção* pedem o enquadramento à shell** — o tudo pela MESMA
/// variante do botão VIEW e do beliscão rápido —, e fecham o menu. As linhas vivem na tabela do *Ver*
/// e no registo da barra (sem `InteractiveState` uma linha nasce morta).
#[test]
fn the_view_menu_rows_ask_the_shell_to_frame() {
    use crate::screens::hero::ViewFocusKind;
    let rows: Vec<_> = crate::screens::hero::menu_rows::menu_rows(
        crate::interaction::ContextMenuKind::MenuBarView,
    )
    .iter()
    .map(|r| r.0)
    .collect();
    for (id, kind) in [
        (ids::MENUBAR_VIEW_FRAME_ALL, ViewFocusKind::All),
        (
            ids::MENUBAR_VIEW_FRAME_SELECTED,
            ViewFocusKind::FrameSelected,
        ),
    ] {
        assert!(rows.contains(&id), "a linha não está no menu *Ver*");
        assert!(
            crate::screens::hero::menu_bar::OWN_ROWS.contains(&id),
            "a linha não está registada"
        );
        let mut hero = HeroScreen::new(crate::NodeId(1));
        assert!(hero.apply_event(WidgetEvent::Click(id)));
        let got: Vec<_> = hero.bus.drain().collect();
        assert!(
            got.contains(&EditorAction::SetViewFocus { kind }),
            "{kind:?}: {got:?}"
        );
    }
}
