//! ⭐ **A barra de menus estreita** (bc_janela_900x560: o «Run» ficava por baixo da engrenagem): os
//! títulos que não cabem antes da engrenagem saem para o transbordo `…` no fim da fila, que abre um
//! menu com eles como submenus (o idioma do VS Code e do JetBrains).

use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::tab_bar::menu::{MenuItem, MenuPress, more_menu, more_row};
use ph2d_panel_code::tab_bar::{Crumb, TabBar, height};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

const TITLES: [(u8, &str); 6] = [
    (0, "File"),
    (1, "Edit"),
    (2, "Find"),
    (3, "Go To"),
    (4, "View"),
    (5, "Run"),
];
const RUN_CMD: ph2d_a11y::NodeId = hash_node_id("t_run_continue");

fn items_of(m: u8) -> Vec<MenuItem> {
    let id = if m == 5 {
        RUN_CMD
    } else {
        ph2d_tool_registry::hash_node_id_runtime(&format!("t_cmd_{m}"))
    };
    vec![MenuItem::Cmd {
        id,
        name: format!("Command {m}"),
        keys: "F8".into(),
        live: true,
    }]
}

fn pinta(bar: &mut TabBar<u8>, w: f32, ts: &mut TextSystem) {
    let (mut scene, theme) = (VectorScene::new(), Theme::default());
    let crumb = Crumb {
        script: "problemas.luau".into(),
        func: Some(("update".into(), 3, 0)),
        line: 22,
    };
    let r = Rect::new(10.0, 0.0, w, height());
    bar.paint(r, &TITLES, &crumb, &mut scene, ts, &theme);
    let items = if bar.more {
        let hidden = bar.painted.hidden.clone();
        more_menu(hidden.into_iter().map(|m| {
            (
                more_row(m as usize),
                TITLES[m as usize].1.into(),
                items_of(m),
            )
        }))
    } else {
        bar.open.map(items_of).unwrap_or_default()
    };
    bar.paint_popup(&items, &mut scene, ts, &theme, (0, 0));
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

#[test]
fn a_barra_estreita_manda_os_titulos_que_nao_cabem_ao_transbordo() {
    let mut ts = TextSystem::without_system_fonts();
    let mut com_transbordo = 0;
    for w in (200..=900).step_by(10) {
        let mut bar = TabBar::<u8>::default();
        pinta(&mut bar, w as f32, &mut ts);
        let p = &bar.painted;
        let gear = p.gear.expect("a engrenagem");
        let fila: Vec<Rect> = p.titles.iter().map(|t| t.1).chain(p.more).collect();
        for t in &fila {
            assert!(
                t.x >= p.header.x && t.x + t.w <= gear.x,
                "w={w}: o título {t:?} cruza a engrenagem {gear:?} ou sai da barra {:?}",
                p.header
            );
        }
        assert_eq!(p.titles.len() + p.hidden.len(), TITLES.len(), "w={w}");
        let cabe = p.min_w <= w as f32;
        assert_eq!(
            p.more.is_some(),
            !cabe,
            "w={w}: o transbordo aparece só quando falta espaço (min_w {})",
            p.min_w
        );
        assert_eq!(p.more.is_some(), !p.hidden.is_empty(), "w={w}");
        com_transbordo += usize::from(p.more.is_some());
    }
    assert!(
        com_transbordo > 0 && com_transbordo < 71,
        "controlo: a varredura cobre as duas regiões ({com_transbordo} de 71 com transbordo)"
    );
}

#[test]
fn o_transbordo_abre_os_titulos_escondidos_e_chega_aos_itens_do_run() {
    for w in (180..=300).step_by(20) {
        abre_o_run_pelo_transbordo(w as f32);
    }
}

fn abre_o_run_pelo_transbordo(w: f32) {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = TabBar::<u8>::default();
    pinta(&mut bar, w, &mut ts);
    assert!(bar.painted.hidden.contains(&5), "o Run sai a {w} px");
    let (x, y) = centro(bar.painted.more.expect("o transbordo"));
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken));
    assert!(bar.more && bar.open.is_none(), "abre o menu do transbordo");
    pinta(&mut bar, w, &mut ts);
    let menu = bar.painted.popup.expect("o menu do transbordo");
    assert!(
        (menu.x - bar.painted.more.expect("…").x).abs() < 0.5,
        "debaixo do `…`"
    );
    let run = more_row(5);
    let parent = bar
        .painted
        .rows
        .iter()
        .find(|r| r.0 == run)
        .expect("Run ▸")
        .1;
    let (x, y) = centro(parent);
    assert!(bar.hover_move(x, y));
    assert_eq!(bar.sub, Some(run), "pairar no Run abre o submenu dele");
    pinta(&mut bar, w, &mut ts);
    let sub = bar.painted.sub.clone().expect("o submenu do Run");
    let h = bar.painted.header;
    assert!(
        sub.popup.x >= h.x && sub.popup.x + sub.popup.w <= h.x + h.w + 0.5,
        "o submenu fica dentro da aba: {sub:?}"
    );
    let tapa = sub.popup.x < menu.x + menu.w
        && menu.x < sub.popup.x + sub.popup.w
        && sub.popup.y < menu.y + menu.h
        && menu.y < sub.popup.y + sub.popup.h;
    assert!(
        !tapa,
        "o submenu nunca tapa o menu-mãe: {sub:?} sobre {menu:?}"
    );
    let row = sub
        .rows
        .iter()
        .find(|r| r.0 == RUN_CMD)
        .expect("o comando do Run")
        .1;
    let (x, y) = centro(row);
    assert!(bar.hover_move(x, y));
    assert_eq!(bar.press(x, y), Some(MenuPress::Run(RUN_CMD)));
    assert!(!bar.more && bar.open.is_none(), "escolher fecha tudo");
}

#[test]
fn o_esc_fecha_o_submenu_e_depois_o_transbordo() {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = TabBar::<u8>::default();
    pinta(&mut bar, 260.0, &mut ts);
    let (x, y) = centro(bar.painted.more.expect("o transbordo"));
    bar.press(x, y);
    pinta(&mut bar, 260.0, &mut ts);
    let (x, y) = centro(bar.painted.rows[0].1);
    bar.hover_move(x, y);
    assert!(bar.escape() && bar.more && bar.sub.is_none(), "um nível");
    assert!(bar.escape() && !bar.more, "o transbordo");
    assert!(!bar.escape());
}
