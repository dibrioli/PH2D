//! ⭐ **O menu da barra e o que fica FORA dele** (auditoria final 12/11): um carregar fora de todos os
//! níveis (um painel, o chrome) ou a perda de foco fecha a cadeia toda e o teclado deixa de ir ao menu;
//! o rato a sair da janela larga o realce e a mira sem fechar; um submenu cuja linha-mãe rolou para
//! fora da vista fecha (nada aberto fica sem se ver).

use ph2d_editor_core::doc_surface::Key;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::tab_bar::menu::{MenuItem, MenuPress};
use ph2d_panel_code::tab_bar::{Crumb, TabBar, height};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

const SUB: ph2d_a11y::NodeId = hash_node_id("f_sub");

/// O submenu na 1.ª linha e `n` comandos por baixo.
fn items(n: usize) -> Vec<MenuItem> {
    let sub = MenuItem::Sub {
        id: SUB,
        name: "Text rendering".into(),
        rows: MenuItem::choices(vec![
            (hash_node_id("f_c1"), "Default".into(), true),
            (hash_node_id("f_c2"), "Crisp Heavy".into(), false),
        ]),
    };
    let cmds = (0..n).map(|i| MenuItem::Cmd {
        id: ph2d_tool_registry::hash_node_id_runtime(&format!("f_cmd_{i}")),
        name: format!("Command {i}"),
        keys: String::new(),
        live: true,
    });
    std::iter::once(sub).chain(cmds).collect()
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn pinta(bar: &mut TabBar<u8>, n: usize, ts: &mut TextSystem) {
    let (mut scene, theme) = (VectorScene::new(), Theme::default());
    let crumb = Crumb {
        script: "a.luau".into(),
        func: None,
        line: 1,
    };
    let titles = [(0u8, "File"), (1u8, "View")];
    let header = Rect::new(0.0, 0.0, 900.0, height());
    bar.paint(header, &titles, &crumb, &mut scene, ts, &theme);
    bar.paint_popup(&items(n), &mut scene, ts, &theme, (0, 0));
}

/// O menu View aberto numa janela de `win_h` de altura, com o submenu aberto pelo rato.
fn aberto(n: usize, win_h: f32, ts: &mut TextSystem) -> TabBar<u8> {
    let mut bar = TabBar::<u8> {
        window: Some(Rect::new(0.0, 0.0, 900.0, win_h)),
        ..TabBar::default()
    };
    pinta(&mut bar, n, ts);
    let (x, y) = centro(bar.painted.titles[1].1);
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken));
    pinta(&mut bar, n, ts);
    let parent = bar.painted.rows.iter().find(|r| r.0 == SUB).expect("mãe").1;
    let (x, y) = centro(parent);
    assert!(bar.hover_move_at(x, y, 1));
    pinta(&mut bar, n, ts);
    assert!(
        bar.sub == Some(SUB) && bar.painted.sub.is_some(),
        "controlo: o submenu aberto e pintado"
    );
    bar
}

/// Um carregar num painel (o aviso do hero) fecha o menu e o submenu; depois disso ↓ e Enter não
/// são do menu (seguem ao painel em foco).
#[test]
fn o_aviso_de_fora_fecha_a_cadeia_e_o_teclado_deixa_de_ir_ao_menu() {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = aberto(3, 560.0, &mut ts);
    let it = items(3);
    assert!(
        bar.key(&it, Key::Down).is_some(),
        "controlo: aberto, ↓ é do menu"
    );
    assert!(bar.dismiss(), "havia um menu aberto");
    assert!(!bar.is_open() && bar.chain().is_empty(), "a cadeia toda");
    assert_eq!(bar.key(&it, Key::Down), None, "↓ já não é do menu");
    assert_eq!(bar.key(&it, Key::Enter), None, "Enter não corre nada");
    assert!(!bar.dismiss(), "fechado: nada a fechar");
}

/// O rato sai da janela: o realce do rato e a âncora da mira largam-se; o menu e o submenu ficam
/// (o teclado continua).
#[test]
fn o_rato_a_sair_larga_o_realce_e_a_mira_sem_fechar() {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = aberto(3, 560.0, &mut ts);
    assert!(
        bar.hover.is_some() && bar.last.is_some() && bar.moved_at > 0,
        "controlo"
    );
    bar.pointer_left();
    assert_eq!((bar.hover, bar.last, bar.moved_at), (None, None, 0));
    assert!(bar.is_open() && bar.sub == Some(SUB), "o menu fica aberto");
    assert!(
        bar.key(&items(3), Key::Down).is_some(),
        "o teclado continua"
    );
}

/// A linha-mãe rola para fora da vista (a roda no menu de topo): o submenu fecha, não fica uma
/// cadeia aberta que nada mostra.
#[test]
fn o_submenu_fecha_quando_a_linha_mae_rola_para_fora_da_vista() {
    let mut ts = TextSystem::without_system_fonts();
    let n = 30;
    let mut bar = aberto(n, 300.0, &mut ts);
    let popup = bar.painted.popup.expect("o menu");
    assert!(bar.painted.arrows.is_some(), "controlo: o menu rola");
    let (x, y) = centro(popup);
    assert!(bar.wheel(x, y - height(), -height() * 4.0));
    pinta(&mut bar, n, &mut ts);
    assert!(
        !bar.painted.rows.iter().any(|r| r.0 == SUB),
        "controlo: a mãe saiu da vista"
    );
    assert_eq!(bar.sub, None, "a cadeia fecha com a mãe fora da vista");
    assert!(bar.painted.sub.is_none() && bar.is_open(), "o menu fica");
}
