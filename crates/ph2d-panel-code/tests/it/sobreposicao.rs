//! ⭐ **Os menus da aba flutuam presos à JANELA** (foto bd_menus_transbordo a 900×560: o Run ▸ do `…`
//! cortava no fundo da aba e «Remove All Breakpoints» ficava inalcançável). O menu e os submenus
//! pintam-se por cima de tudo, dentro da janela; mais altos que ela, rolam; os submenus aninham-se em N
//! níveis (`… ▸ View ▸ Text rendering ▸`).

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::tab_bar::menu::{MenuItem, MenuPress, more_menu, more_row};
use ph2d_panel_code::tab_bar::{Crumb, TabBar, height};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const TITLES: [(u8, &str); 6] = [
    (0, "File"),
    (1, "Edit"),
    (2, "Find"),
    (3, "Go To"),
    (4, "View"),
    (5, "Run"),
];

pub(super) fn id(k: &str) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("sobre_{k}"))
}

fn cmd(k: &str) -> MenuItem {
    MenuItem::Cmd {
        id: id(k),
        name: k.into(),
        keys: String::new(),
        live: true,
    }
}

/// O View do app: Card Style ▸ e Text rendering ▸ (escolhas); o Run com as sete linhas da foto; o
/// File com `n` comandos (um menu mais alto que a janela).
fn items_of(m: u8, n: usize) -> Vec<MenuItem> {
    let sub = |k: &str, cs: [&str; 2]| MenuItem::Sub {
        id: id(k),
        name: k.into(),
        rows: MenuItem::choices(cs.map(|c| (id(c), c.to_owned(), c.ends_with('A')))),
    };
    match m {
        0 => (0..n).map(|i| cmd(&format!("File {i}"))).collect(),
        4 => vec![
            sub("Card Style", ["Cards A", "Flat B"]),
            sub("Text rendering", ["Default A", "Crisp B"]),
        ],
        5 => {
            let mut v: Vec<_> = ["Restart", "Toggle", "Continue", "Next", "Pause", "Stop"]
                .map(cmd)
                .to_vec();
            v.insert(1, MenuItem::Note("Only needed if suggestions stop".into()));
            v.push(cmd("Remove All Breakpoints"));
            v
        }
        2 => vec![
            cmd("Find A"),
            MenuItem::Cmd {
                id: id("Find B"),
                name: "Find B".into(),
                keys: String::new(),
                live: false,
            },
            MenuItem::Note("a note".into()),
            cmd("Find C"),
        ],
        _ => vec![cmd(&format!("Cmd {m}"))],
    }
}

/// A barra de `w` px no topo de uma aba, numa janela `win`; o menu do File tem `n` linhas.
pub(super) fn pinta(bar: &mut TabBar<u8>, w: f32, win: Rect, n: usize, ts: &mut TextSystem) {
    pinta_em(bar, (w, win, n), ts, 0);
}

/// [`pinta`] no relógio `now` (ns).
pub(super) fn pinta_em(
    bar: &mut TabBar<u8>,
    at: (f32, Rect, usize),
    ts: &mut TextSystem,
    now: u128,
) {
    let (w, win, n) = at;
    let (mut scene, theme) = (VectorScene::new(), Theme::default());
    let crumb = Crumb {
        script: "problemas.luau".into(),
        func: None,
        line: 22,
    };
    bar.paint(
        Rect::new(400.0, 30.0, w, height()),
        &TITLES,
        &crumb,
        &mut scene,
        ts,
        &theme,
    );
    bar.window = Some(win);
    let items = itens(bar, n);
    bar.paint_popup(&items, &mut scene, ts, &theme, (now, 0));
}

/// As linhas do menu aberto (as que [`pinta`] lhe dá).
pub(super) fn itens(bar: &TabBar<u8>, n: usize) -> Vec<MenuItem> {
    if bar.more {
        let hidden = bar.painted.hidden.clone();
        let rows = hidden.into_iter().map(|m| {
            let i = usize::from(m);
            (more_row(i), TITLES[i].1.into(), items_of(m, n))
        });
        more_menu(rows)
    } else {
        bar.open.map(|m| items_of(m, n)).unwrap_or_default()
    }
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn dentro(r: Rect, win: Rect) -> bool {
    r.x >= win.x && r.y >= win.y && r.x + r.w <= win.x + win.w && r.y + r.h <= win.y + win.h
}

/// Pairar e carregar na linha `k` do nível mais fundo aberto.
pub(super) fn row_of(bar: &TabBar<u8>, k: NodeId) -> Rect {
    let p = &bar.painted;
    let rows = p
        .deeper
        .last()
        .or(p.sub.as_ref())
        .map_or(&p.rows, |s| &s.rows);
    rows.iter().find(|r| r.0 == k).expect("a linha à vista").1
}

/// ⭐ A aba de 225 px numa janela de 560: o Run ▸ do `…` sai da aba e cabe na JANELA, todas as
/// linhas clicáveis — «Remove All Breakpoints» incluída.
#[test]
fn o_run_do_transbordo_cabe_na_janela_e_nao_na_aba() {
    let mut ts = TextSystem::without_system_fonts();
    let win = Rect::new(0.0, 0.0, 900.0, 560.0);
    let aba_fundo = 30.0 + 225.0;
    let mut bar = TabBar::<u8>::default();
    pinta(&mut bar, 200.0, win, 3, &mut ts);
    assert!(bar.painted.hidden.contains(&5), "controlo: o Run no `…`");
    let (x, y) = centro(bar.painted.more.expect("…"));
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken));
    pinta(&mut bar, 200.0, win, 3, &mut ts);
    let (x, y) = centro(row_of(&bar, more_row(5)));
    assert!(bar.hover_move(x, y));
    pinta(&mut bar, 200.0, win, 3, &mut ts);
    let sub = bar.painted.sub.clone().expect("o Run ▸");
    assert!(dentro(sub.popup, win), "dentro da janela: {sub:?}");
    assert!(
        sub.popup.y + sub.popup.h > aba_fundo,
        "controlo: o Run ▸ passa do fundo da aba ({sub:?})"
    );
    assert!(sub.arrows.is_none(), "cabe: não rola");
    assert_eq!(sub.rows.len(), 7, "as sete linhas clicáveis");
    let last = id("Remove All Breakpoints");
    let (x, y) = centro(row_of(&bar, last));
    assert!(
        bar.floats_at(x, y),
        "o rato ali é do menu, por cima do resto"
    );
    assert!(bar.hover_move(x, y));
    assert_eq!(bar.press(x, y), Some(MenuPress::Run(last)));
    assert!(!bar.is_open(), "escolher fecha tudo");
}

/// ⭐ Um menu mais alto que a janela ocupa-a e rola: a roda e a seta ▼ levam ao último item.
#[test]
fn o_menu_mais_alto_que_a_janela_rola_ate_ao_ultimo() {
    let mut ts = TextSystem::without_system_fonts();
    let win = Rect::new(0.0, 0.0, 900.0, 200.0);
    let n = 30;
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta(&mut bar, 600.0, win, n, &mut ts);
    let menu = bar.painted.popup.expect("o File");
    assert!(dentro(menu, win), "dentro da janela: {menu:?}");
    let (up, down) = bar.painted.arrows.expect("rola: as setas");
    let last = id(&format!("File {}", n - 1));
    assert!(
        bar.painted.rows.iter().all(|r| r.0 != last),
        "controlo: escondido"
    );
    let (x, y) = centro(down);
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken), "a seta ▼");
    pinta(&mut bar, 600.0, win, n, &mut ts);
    assert!(bar.scroll[0] > 0.0, "a seta rolou");
    let (x, y) = centro(menu);
    for _ in 0..n {
        assert!(bar.wheel(x, y, -height()));
    }
    pinta(&mut bar, 600.0, win, n, &mut ts);
    let r = row_of(&bar, last);
    assert!(
        r.y >= up.y + up.h && r.y + r.h <= down.y + 0.5,
        "à vista: {r:?}"
    );
    let (x, y) = centro(r);
    assert!(bar.hover_move(x, y));
    assert_eq!(bar.press(x, y), Some(MenuPress::Run(last)));
}

/// Abre `… ▸ View ▸ Text rendering ▸` (ou o View de topo com `top`) e carrega no «Crisp».
fn crisp(top: bool, ts: &mut TextSystem) -> Option<MenuPress> {
    let win = Rect::new(0.0, 0.0, 900.0, 560.0);
    let w = if top { 1600.0 } else { 200.0 };
    let mut bar = TabBar::<u8>::default();
    pinta(&mut bar, w, win, 3, ts);
    let at = if top {
        bar.painted
            .titles
            .iter()
            .find(|t| t.0 == 4)
            .expect("View")
            .1
    } else {
        bar.painted.more.expect("…")
    };
    let (x, y) = centro(at);
    bar.press(x, y);
    pinta(&mut bar, w, win, 3, ts);
    let path = if top { vec![] } else { vec![more_row(4)] };
    for p in path.into_iter().chain([id("Text rendering")]) {
        let (x, y) = centro(row_of(&bar, p));
        assert!(bar.hover_move(x, y));
        pinta(&mut bar, w, win, 3, ts);
    }
    let levels = 1 + bar.painted.deeper.len() + usize::from(bar.painted.sub.is_some());
    assert_eq!(levels, if top { 2 } else { 3 }, "os níveis abertos");
    let (x, y) = centro(row_of(&bar, id("Crisp B")));
    assert!(bar.hover_move(x, y));
    bar.press(x, y)
}

/// ⭐ O View ▸ do `…` é o View de topo: Text rendering ▸ é um submenu de verdade (o 3.º nível), e a
/// escolha dá o MESMO comando.
#[test]
fn o_view_do_transbordo_aninha_os_submenus_como_o_de_topo() {
    let mut ts = TextSystem::without_system_fonts();
    let topo = crisp(true, &mut ts);
    assert_eq!(
        topo,
        Some(MenuPress::Run(id("Crisp B"))),
        "controlo: o de topo"
    );
    assert_eq!(crisp(false, &mut ts), topo, "o do `…` dá o mesmo");
}

/// O Esc fecha um nível de cada vez, do mais fundo ao menu.
#[test]
fn o_esc_fecha_os_tres_niveis_um_a_um() {
    let mut bar = TabBar::<u8> {
        more: true,
        sub: Some(more_row(4)),
        deeper: vec![id("Text rendering")],
        ..TabBar::default()
    };
    assert!(bar.escape() && bar.deeper.is_empty() && bar.sub.is_some());
    assert!(bar.escape() && bar.sub.is_none() && bar.more);
    assert!(bar.escape() && !bar.is_open());
}
