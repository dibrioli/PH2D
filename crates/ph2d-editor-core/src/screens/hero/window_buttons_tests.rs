//! A faixa única: os botões da janela, a borda sob o ponteiro e o Down que arrasta ou maximiza.

use super::*;
use crate::screens::hero::menu_bar::MENU_BAR_H;
use crate::screens::hero::menu_faixa::faixa;
use ph2d_text::TextSystem;

fn bar(w: f32) -> Rect {
    Rect::new(0.0, 0.0, w, MENU_BAR_H)
}

/// Com a barra do sistema não há botões; com a do app são três, encostados à direita, − □ ×.
#[test]
fn os_tres_botoes_so_existem_na_janela_do_app_e_encostam_a_direita() {
    assert!(rects(bar(1366.0), None).is_empty());
    let b = rects(bar(1366.0), Some(false));
    let ids: Vec<_> = b.iter().map(|(id, ..)| *id).collect();
    assert_eq!(
        ids,
        [
            ids::MENUBAR_WIN_MINIMIZE,
            ids::MENUBAR_WIN_MAXIMIZE,
            ids::MENUBAR_WIN_CLOSE
        ]
    );
    let u = b.last().unwrap().2;
    assert!((u.x + u.w - 1366.0).abs() < 1e-3, "o × não encosta à borda");
    for par in b.windows(2) {
        assert!((par[0].2.x + par[0].2.w - par[1].2.x).abs() < 1e-3);
    }
    assert_eq!(b[1].1, IconId::WindowMaximize);
    assert_eq!(rects(bar(1366.0), Some(true))[1].1, IconId::WindowRestore);
}

/// O jogo e o campo afastam-se para a esquerda dos botões da janela: nada se pisa.
#[test]
fn o_jogo_fica_a_esquerda_dos_botoes_da_janela() {
    for w in [1930.0, 1366.0, 1000.0] {
        let mut ts = TextSystem::without_system_fonts();
        let f = faixa(bar(w), &mut ts, Some(false));
        let min = f.janela.first().expect("botões").2;
        let jogo = f.jogo.last().expect("o jogo cabe").2;
        assert!(jogo.x + jogo.w <= min.x, "{w}: o jogo pisa o minimizar");
        assert!(
            f.find.x + f.find.w <= f.jogo[0].2.x,
            "{w}: o campo pisa o Play"
        );
    }
}

/// As oito bordas, os cantos mais largos que a borda, e o miolo sem borda; maximizada não tem.
#[test]
fn a_borda_sob_o_ponteiro() {
    let (w, h) = (1000.0, 700.0);
    let (band, corner) = edge_band();
    let dentro = band * 0.5;
    let casos = [
        (dentro, h * 0.5, Some(Edge::West)),
        (w - dentro, h * 0.5, Some(Edge::East)),
        (w * 0.5, dentro, Some(Edge::North)),
        (w * 0.5, h - dentro, Some(Edge::South)),
        (dentro, dentro, Some(Edge::NorthWest)),
        (w - dentro, h - dentro, Some(Edge::SouthEast)),
        // O canto pega a um lado maior que a borda, ao longo dela.
        (corner - 1.0, dentro, Some(Edge::NorthWest)),
        (w - dentro, corner - 1.0, Some(Edge::NorthEast)),
        (dentro, h - corner + 1.0, Some(Edge::SouthWest)),
        (w * 0.5, h * 0.5, None),
        (band + 1.0, band + 1.0, None),
    ];
    for (x, y, want) in casos {
        assert_eq!(edge_at(x, y, w, h, false), want, "({x}, {y})");
    }
    assert_eq!(edge_at(dentro, dentro, w, h, true), None);
}

/// O Down no fundo arrasta; o segundo dentro do duplo clique maximiza; o terceiro volta a arrastar.
/// Num botão (ou fora da faixa) o Down é do editor; na borda redimensiona.
#[test]
fn o_down_arrasta_maximiza_ou_redimensiona() {
    let mut p = WindowPress::default();
    let size = (1000.0, 700.0);
    let fundo = Some(ids::MENUBAR_BACKDROP);
    let ms = 1_000_000u128;
    assert_eq!(
        p.press(500.0, 10.0, size, false, fundo, 0),
        Some(PressAction::Move)
    );
    assert_eq!(
        p.press(500.0, 10.0, size, false, fundo, 200 * ms),
        Some(PressAction::ToggleMaximize)
    );
    assert_eq!(
        p.press(500.0, 10.0, size, false, fundo, 300 * ms),
        Some(PressAction::Move)
    );
    assert_eq!(
        p.press(500.0, 10.0, size, false, fundo, 2000 * ms),
        Some(PressAction::Move)
    );
    assert_eq!(
        p.press(500.0, 10.0, size, false, Some(ids::MENUBAR_W), 2100 * ms),
        None
    );
    assert_eq!(
        p.press(1.0, 300.0, size, false, None, 0),
        Some(PressAction::Resize(Edge::West))
    );
    // Maximizada: a borda é do editor, o fundo continua a arrastar (o sistema restaura ao arrastar).
    assert_eq!(p.press(1.0, 300.0, size, true, None, 0), None);
}

/// Cada botão vira o pedido certo, e só eles.
#[test]
fn o_clique_vira_o_pedido() {
    let mut s = WidgetStore::default();
    for (id, want) in [
        (ids::MENUBAR_WIN_MINIMIZE, WindowRequest::Minimize),
        (ids::MENUBAR_WIN_MAXIMIZE, WindowRequest::ToggleMaximize),
        (ids::MENUBAR_WIN_CLOSE, WindowRequest::Close),
    ] {
        assert!(on_click(&mut s, id));
        assert_eq!(s.take_window_request(), Some(want));
        assert_eq!(s.take_window_request(), None);
    }
    assert!(!on_click(&mut s, ids::MENUBAR_W));
}
