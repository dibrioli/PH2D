//! ⭐ **O teclado nos menus da aba** (o idioma dos menus de sistema e do VS Code): ↑/↓ saltam as
//! inactivas e as notas e dão a volta, Home/End, → abre o submenu e realça a 1.ª linha, ← fecha-o, no
//! topo ←/→ passam ao título vizinho (o `…` incluído), Enter corre, Esc fecha um nível; num menu que
//! rola, o realce arrasta a rolagem.

use super::sobreposicao::{id, itens, pinta, pinta_em, row_of};
use ph2d_editor_core::doc_surface::Key;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::tab_bar::menu::{AUTO_ROLL_ROW_NS, MenuPress, more_row};
use ph2d_panel_code::tab_bar::{TabBar, height};
use ph2d_text::TextSystem;

const WIN: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 560.0,
};

fn tecla(bar: &mut TabBar<u8>, k: Key) -> Option<MenuPress> {
    let items = itens(bar, 3);
    bar.key(&items, k)
}

fn aberta(m: u8) -> (TabBar<u8>, TextSystem) {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = TabBar::<u8> {
        open: Some(m),
        ..TabBar::default()
    };
    pinta(&mut bar, 1600.0, WIN, 3, &mut ts);
    (bar, ts)
}

/// ↑/↓ saltam a linha inactiva e a nota e dão a volta; Home/End; Enter corre a realçada e fecha.
#[test]
fn as_setas_saltam_as_inactivas_e_as_notas_e_dao_a_volta() {
    let (mut bar, _ts) = aberta(2);
    let (a, c) = (Some(id("Find A")), Some(id("Find C")));
    let mut seq = Vec::new();
    for k in [
        Key::Down,
        Key::Down,
        Key::Down,
        Key::Up,
        Key::End,
        Key::Home,
    ] {
        assert_eq!(tecla(&mut bar, k), Some(MenuPress::Taken), "{k:?}");
        seq.push(bar.hover);
    }
    assert_eq!(
        seq,
        [a, c, a, c, c, a],
        "Find B (inactiva) e a nota nunca acendem"
    );
    tecla(&mut bar, Key::Down);
    assert_eq!(
        tecla(&mut bar, Key::Enter),
        Some(MenuPress::Run(id("Find C")))
    );
    assert!(!bar.is_open(), "correr fecha o menu");
    assert_eq!(
        tecla(&mut bar, Key::Down),
        None,
        "menu fechado: a tecla segue"
    );
}

/// → abre o submenu da linha realçada e realça a 1.ª dele; ← fecha-o e volta à mãe; Enter também
/// abre; o Esc fecha um nível e deixa a mãe acesa.
#[test]
fn a_seta_direita_abre_o_submenu_e_a_esquerda_fecha() {
    let (mut bar, mut ts) = aberta(4);
    let (style, cards) = (id("Card Style"), id("Cards A"));
    tecla(&mut bar, Key::Down);
    assert_eq!(bar.hover, Some(style));
    assert!(bar.chain().is_empty(), "controlo: realçar não abre");
    tecla(&mut bar, Key::Right);
    assert_eq!((bar.chain(), bar.hover), (vec![style], Some(cards)));
    pinta(&mut bar, 1600.0, WIN, 3, &mut ts);
    tecla(&mut bar, Key::Down);
    assert_eq!(bar.hover, Some(id("Flat B")), "↓ move-se no submenu");
    tecla(&mut bar, Key::Left);
    assert_eq!((bar.chain(), bar.hover), (vec![], Some(style)));
    tecla(&mut bar, Key::Enter);
    assert_eq!((bar.chain(), bar.hover), (vec![style], Some(cards)));
    tecla(&mut bar, Key::Escape);
    assert_eq!((bar.chain(), bar.hover), (vec![], Some(style)));
    assert!(bar.is_open());
    tecla(&mut bar, Key::Escape);
    assert!(!bar.is_open());
}

/// No menu de topo ←/→ passam ao título vizinho, o `…` incluído (com a volta), e a 1.ª linha do
/// menu novo acende-se; → numa linha que não abre submenu também passa ao título seguinte.
#[test]
fn no_topo_as_setas_passam_ao_titulo_vizinho_e_ao_transbordo() {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta(&mut bar, 200.0, WIN, 3, &mut ts);
    let first_hidden = *bar.painted.hidden.first().expect("controlo: há `…`");
    tecla(&mut bar, Key::Left);
    assert!(
        bar.more && bar.open.is_none(),
        "← no 1.º título dá a volta ao `…`"
    );
    pinta(&mut bar, 200.0, WIN, 3, &mut ts);
    assert_eq!(bar.hover, Some(more_row(usize::from(first_hidden))));
    tecla(&mut bar, Key::Right);
    assert_eq!(
        bar.chain().len(),
        1,
        "→ na linha de um título escondido abre o submenu dele"
    );
    pinta(&mut bar, 200.0, WIN, 3, &mut ts);
    tecla(&mut bar, Key::Right);
    assert_eq!(
        (bar.open, bar.more),
        (Some(0), false),
        "→ num comando: o título a seguir ao `…`"
    );
    pinta(&mut bar, 200.0, WIN, 3, &mut ts);
    assert_eq!(bar.hover, Some(id("File 0")));
    if bar.painted.titles.len() > 1 {
        tecla(&mut bar, Key::Right);
        assert_eq!(bar.open, Some(bar.painted.titles[1].0));
    }
}

/// Num menu mais alto que a janela, o End leva a rolagem até à última linha e o Home de volta.
#[test]
fn o_realce_por_teclado_arrasta_a_rolagem() {
    let mut ts = TextSystem::without_system_fonts();
    let (win, n) = (Rect::new(0.0, 0.0, 900.0, 200.0), 30);
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta(&mut bar, 600.0, win, n, &mut ts);
    let (up, down) = bar.painted.arrows.expect("controlo: rola");
    let last = id(&format!("File {}", n - 1));
    let items = itens(&bar, n);
    bar.key(&items, Key::End);
    pinta(&mut bar, 600.0, win, n, &mut ts);
    assert_eq!(bar.hover, Some(last));
    let r = row_of(&bar, last);
    assert!(
        r.y >= up.y + up.h - 0.5 && r.y + r.h <= down.y + 0.5,
        "à vista: {r:?}"
    );
    bar.key(&items, Key::Home);
    pinta(&mut bar, 600.0, win, n, &mut ts);
    assert_eq!(bar.scroll[0], 0.0, "o Home volta ao topo");
}

/// ⭐ O rato PARADO na seta ▼ de um menu que rola: o menu rola sozinho a passo constante pelo
/// relógio ([`AUTO_ROLL_ROW_NS`]); sai da seta, pára; na ▲, volta.
#[test]
fn o_rato_parado_na_seta_rola_o_menu_pelo_relogio() {
    let mut ts = TextSystem::without_system_fonts();
    let at = (600.0, Rect::new(0.0, 0.0, 900.0, 200.0), 30);
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta_em(&mut bar, at, &mut ts, 0);
    let (up, down) = bar.painted.arrows.expect("controlo: rola");
    let menu = bar.painted.popup.expect("o File");
    let c = |r: Rect| (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let ms = 1_000_000u128;
    let (x, y) = c(down);
    assert!(bar.hover_move_at(x, y, 0));
    assert!(
        bar.auto_rolling(),
        "na seta: rola sozinho (quem pinta pede quadros)"
    );
    pinta_em(&mut bar, at, &mut ts, 250 * ms);
    let want = (250 * ms) as f32 / AUTO_ROLL_ROW_NS as f32 * height();
    assert!(
        (bar.scroll[0] - want).abs() < 0.5,
        "{} ≈ {want}",
        bar.scroll[0]
    );
    pinta_em(&mut bar, at, &mut ts, 500 * ms);
    let s = bar.scroll[0];
    assert!((s - want * 2.0).abs() < 0.5, "a passo constante: {s}");
    let (x, y) = (menu.x + menu.w * 0.5, menu.y + menu.h * 0.5);
    assert!(bar.hover_move_at(x, y, 500 * ms));
    assert!(!bar.auto_rolling(), "fora da seta: pára");
    pinta_em(&mut bar, at, &mut ts, 900 * ms);
    assert_eq!(bar.scroll[0], s, "parado");
    let (x, y) = c(up);
    assert!(bar.hover_move_at(x, y, 900 * ms));
    pinta_em(&mut bar, at, &mut ts, 1_000 * ms);
    assert!(bar.scroll[0] < s, "a ▲ volta");
}
