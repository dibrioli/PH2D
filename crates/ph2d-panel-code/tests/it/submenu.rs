//! ⭐ **O submenu em cascata da barra da aba** (dono 10/11, `View ▸ Text rendering ▸`): a geometria e a
//! mecânica, sobre a barra sozinha (o caminho pela aba inteira está em `ph2d-app-code`, `barra_da_aba`).

use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::tab_bar::menu::{MenuItem, MenuPress};
use ph2d_panel_code::tab_bar::{Crumb, TabBar, height};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

const A: ph2d_a11y::NodeId = hash_node_id("t_cmd_a");
const SUB: ph2d_a11y::NodeId = hash_node_id("t_sub");
const C1: ph2d_a11y::NodeId = hash_node_id("t_c1");
const C2: ph2d_a11y::NodeId = hash_node_id("t_c2");

fn items() -> Vec<MenuItem> {
    vec![
        MenuItem::Cmd {
            id: A,
            name: "Toggle Peek".into(),
            keys: String::new(),
            live: true,
        },
        MenuItem::Sub {
            id: SUB,
            name: "Text rendering".into(),
            rows: vec![
                (C1, "Default".into(), true),
                (C2, "Crisp Heavy".into(), false),
            ],
        },
    ]
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// Pinta a barra em `w` de largura (1.º título `first`) e o menu aberto.
fn pinta_com(bar: &mut TabBar<u8>, w: f32, first: &str, ts: &mut TextSystem) {
    let (mut scene, theme) = (VectorScene::new(), Theme::default());
    let crumb = Crumb {
        script: "a.luau".into(),
        func: None,
        line: 1,
    };
    let titles = [(0u8, first), (1u8, "View")];
    bar.paint(
        Rect::new(0.0, 0.0, w, height()),
        &titles,
        &crumb,
        &mut scene,
        ts,
        &theme,
    );
    bar.paint_popup(&items(), &mut scene, ts, &theme, (0, 0));
}

fn abre(w: f32, ts: &mut TextSystem) -> TabBar<u8> {
    abre_com(w, "File", ts)
}

fn abre_com(w: f32, first: &str, ts: &mut TextSystem) -> TabBar<u8> {
    let mut bar = TabBar::<u8>::default();
    pinta_com(&mut bar, w, first, ts);
    let view = bar.painted.titles[1].1;
    let (x, y) = centro(view);
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken));
    pinta_com(&mut bar, w, first, ts);
    let parent = bar
        .painted
        .rows
        .iter()
        .find(|r| r.0 == SUB)
        .expect("a linha-mãe")
        .1;
    let (x, y) = centro(parent);
    assert!(bar.hover_move(x, y));
    pinta_com(&mut bar, w, first, ts);
    bar
}

#[test]
fn o_submenu_abre_ao_lado_com_o_check_so_no_em_vigor() {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = abre(900.0, &mut ts);
    let menu = bar.painted.popup.expect("o menu");
    let sub = bar.painted.sub.clone().expect("o submenu");
    assert!(
        (sub.popup.x - (menu.x + menu.w)).abs() < 0.5,
        "à direita, encostado"
    );
    let parent = bar
        .painted
        .rows
        .iter()
        .find(|r| r.0 == SUB)
        .expect("a linha-mãe")
        .1;
    assert!(
        (sub.rows[0].1.y - parent.y).abs() < 0.5,
        "a 1.ª escolha na altura da mãe"
    );
    assert_eq!(sub.checked, [C1]);
    let (x, y) = centro(sub.rows[1].1);
    assert!(bar.hover_move(x, y));
    assert_eq!(bar.hover, Some(C2));
    assert_eq!(bar.sub, Some(SUB), "pairar no submenu mantém-no aberto");
    assert_eq!(bar.press(x, y), Some(MenuPress::Run(C2)));
    assert!(
        bar.open.is_none() && bar.sub.is_none(),
        "escolher fecha tudo"
    );
}

#[test]
fn sem_espaco_a_direita_o_submenu_abre_a_esquerda_ou_encosta_a_borda() {
    let mut ts = TextSystem::without_system_fonts();
    let long = "A very long first menu title";
    let wide = abre_com(900.0, long, &mut ts);
    let (menu, sub) = (
        wide.painted.popup.expect("o menu"),
        wide.painted.sub.expect("sub"),
    );
    let w = menu.x + menu.w + sub.popup.w * 0.5;
    let bar = abre_com(w, long, &mut ts);
    let (menu, sub) = (
        bar.painted.popup.expect("o menu"),
        bar.painted.sub.expect("sub"),
    );
    assert!(
        (sub.popup.x + sub.popup.w - menu.x).abs() < 0.5,
        "à esquerda, encostado: {sub:?}"
    );
    let narrow = abre(
        abre(900.0, &mut ts).painted.popup.expect("menu").w + 60.0,
        &mut ts,
    );
    let (h, sub) = (narrow.painted.header, narrow.painted.sub.expect("sub"));
    assert!(
        sub.popup.x >= h.x - 0.5 && sub.popup.x + sub.popup.w <= h.x + h.w + 0.5,
        "dentro da aba"
    );
}

#[test]
fn a_linha_mae_nao_corre_nada_e_o_esc_fecha_um_nivel() {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = abre(900.0, &mut ts);
    let parent = bar
        .painted
        .rows
        .iter()
        .find(|r| r.0 == SUB)
        .expect("a linha-mãe")
        .1;
    let (x, y) = centro(parent);
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken), "a mãe só abre");
    assert!(bar.escape() && bar.open.is_some() && bar.sub.is_none());
    assert!(bar.escape() && bar.open.is_none());
    assert!(!bar.escape());
}
