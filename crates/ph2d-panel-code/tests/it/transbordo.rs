//! ⭐ **A barra de menus estreita** (bc_janela_900x560: o «Run» ficava por baixo da engrenagem): os
//! títulos que não cabem antes da engrenagem saem para o transbordo `…` no fim da fila, que abre um
//! menu com eles como submenus (o idioma do VS Code e do JetBrains).

use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::tab_bar::menu::{Line, MenuItem, MenuPress, more_menu, more_row};
use ph2d_panel_code::tab_bar::{Crumb, TabBar, height};
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
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

const OFF_CMD: ph2d_a11y::NodeId = hash_node_id("t_run_off");
const VIEW_SUB: ph2d_a11y::NodeId = hash_node_id("t_view_sub");

/// O menu de cada título: um comando que vale com atalho, um sem atalho, um que NÃO vale (com atalho);
/// o Run leva a nota, o View acaba num submenu de escolhas (o idioma do app).
fn items_of(m: u8) -> Vec<MenuItem> {
    let id = if m == 5 {
        RUN_CMD
    } else {
        ph2d_tool_registry::hash_node_id_runtime(&format!("t_cmd_{m}"))
    };
    let cmd = |id, name: String, keys: &str, live| MenuItem::Cmd {
        id,
        name,
        keys: keys.into(),
        live,
    };
    let rid = |k: &str| ph2d_tool_registry::hash_node_id_runtime(&format!("t_{k}_{m}"));
    let mut out = vec![
        cmd(id, format!("Command {m}"), "F8", true),
        cmd(rid("plain"), format!("Plain {m}"), "", true),
    ];
    if m == 5 {
        out.push(MenuItem::Note("Restarts the assistant".into()));
    }
    out.push(cmd(
        if m == 5 { OFF_CMD } else { rid("off") },
        format!("Off {m}"),
        "Shift+F5",
        false,
    ));
    if m == 4 {
        out.push(MenuItem::Sub {
            id: VIEW_SUB,
            name: "Text rendering".into(),
            rows: MenuItem::choices([
                (rid("c1"), "Default".into(), true),
                (rid("c2"), "Crisp".into(), false),
            ]),
        });
    }
    out
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

/// As linhas pintadas do menu do título `m` numa barra larga (com as de cada submenu dele aberto).
fn linhas_de_topo(m: u8, ts: &mut TextSystem) -> (Vec<Line>, Vec<(ph2d_a11y::NodeId, Vec<Line>)>) {
    let mut bar = TabBar::<u8> {
        open: Some(m),
        ..TabBar::default()
    };
    pinta(&mut bar, 1600.0, ts);
    assert!(
        bar.painted.hidden.is_empty(),
        "controlo: a 1600 px nada se esconde"
    );
    let top = bar.painted.lines.clone();
    let mut subs = Vec::new();
    for p in bar.painted.sub_parents.clone() {
        bar.sub = Some(p);
        pinta(&mut bar, 1600.0, ts);
        subs.push((p, bar.painted.sub.clone().expect("o submenu").lines));
    }
    (top, subs)
}

/// ⭐ O submenu do `…` mostra o MESMO que o menu do título na barra larga: rótulo, atalho alinhado à
/// direita, inactivo a cinzento, notas; um submenu do título entra como a nota com o nome e as
/// escolhas dele. Uma linha inactiva não corre nem fecha: carregar nela é do menu.
#[test]
fn o_submenu_do_transbordo_mostra_o_mesmo_que_o_menu_do_titulo() {
    let mut ts = TextSystem::without_system_fonts();
    let w = 200.0;
    let mut bar = TabBar::<u8>::default();
    pinta(&mut bar, w, &mut ts);
    let hidden = bar.painted.hidden.clone();
    assert!(
        hidden.contains(&4) && hidden.contains(&5),
        "controlo: o View e o Run saem a {w} px ({hidden:?})"
    );
    bar.more = true;
    for m in hidden {
        let (top, subs) = linhas_de_topo(m, &mut ts);
        let esperado: Vec<Line> = top
            .iter()
            .flat_map(|l| match subs.iter().find(|s| Some(s.0) == l.id) {
                Some((_, rows)) => std::iter::once(Line {
                    id: None,
                    name: l.name.clone(),
                    keys: String::new(),
                    keys_inset: 0.0,
                    live: false,
                    check: None,
                })
                .chain(rows.iter().cloned())
                .collect(),
                None => vec![l.clone()],
            })
            .collect();
        assert!(
            top.iter().any(|l| !l.live && !l.keys.is_empty()),
            "controlo: o menu de {m} tem uma inactiva com atalho"
        );
        bar.sub = Some(more_row(m as usize));
        pinta(&mut bar, w, &mut ts);
        let sub = bar
            .painted
            .sub
            .clone()
            .expect("o submenu do título escondido");
        assert_eq!(sub.lines, esperado, "o submenu de {m} = o menu dele");
        let off = sub
            .lines
            .iter()
            .find(|l| !l.live && l.id.is_some())
            .expect("inactiva");
        let off = off.id.expect("id");
        assert!(
            sub.rows.iter().all(|r| r.0 != off),
            "a inactiva não se clica"
        );
    }
    // Carregar na inactiva do Run: nada corre, nada fecha; o rato sobre ela não a acende.
    bar.sub = Some(more_row(5));
    pinta(&mut bar, w, &mut ts);
    let sub = bar.painted.sub.clone().expect("o Run ▸");
    let i = sub
        .lines
        .iter()
        .position(|l| l.id == Some(OFF_CMD))
        .expect("Off 5");
    let y = sub.popup.y + Spacing::Sm.px() * 0.5 + height() * (i as f32 + 0.5);
    let x = sub.popup.x + sub.popup.w * 0.5;
    assert!(bar.hover_move(x, y));
    assert_ne!(bar.hover, Some(OFF_CMD), "o rato não acende a inactiva");
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken));
    assert!(bar.more && bar.sub == Some(more_row(5)), "nada fecha");
}
