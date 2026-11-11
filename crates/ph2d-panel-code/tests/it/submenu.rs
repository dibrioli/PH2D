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
    bar.sub = None;
    assert_eq!(bar.press(x, y), Some(MenuPress::Taken), "a mãe só abre");
    assert_eq!(bar.sub, Some(SUB), "clicar na mãe abre o submenu");
    assert!(bar.escape() && bar.open.is_some() && bar.sub.is_none());
    assert!(bar.escape() && bar.open.is_none());
    assert!(!bar.escape());
}

const S2: ph2d_a11y::NodeId = hash_node_id("t_sub2");

/// A mira (dois submenus vizinhos, como o View: Card Style ▸ e Text rendering ▸): ir na diagonal da
/// mãe de cima até ao submenu dela passa por cima da mãe de baixo SEM trocar; descer a direito troca.
#[test]
fn a_caminho_do_submenu_a_linha_de_baixo_nao_o_troca() {
    let mut ts = TextSystem::without_system_fonts();
    let its = vec![
        MenuItem::Sub {
            id: SUB,
            name: "Card Style".into(),
            rows: vec![(C1, "Sheet".into(), true), (C2, "Cards".into(), false)],
        },
        MenuItem::Sub {
            id: S2,
            name: "Text rendering".into(),
            rows: vec![(A, "Default".into(), true)],
        },
    ];
    let pinta = |bar: &mut TabBar<u8>, ts: &mut TextSystem| {
        let (mut scene, theme) = (VectorScene::new(), Theme::default());
        let crumb = Crumb {
            script: "a.luau".into(),
            func: None,
            line: 1,
        };
        let r = Rect::new(0.0, 0.0, 900.0, height());
        bar.paint(r, &[(0u8, "View")], &crumb, &mut scene, ts, &theme);
        bar.paint_popup(&its, &mut scene, ts, &theme, (0, 0));
    };
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta(&mut bar, &mut ts);
    let row = |bar: &TabBar<u8>, id| bar.painted.rows.iter().find(|r| r.0 == id).expect("mãe").1;
    let (cima, baixo) = (row(&bar, SUB), row(&bar, S2));
    let p0 = (cima.x + cima.w * 0.6, cima.y + cima.h * 0.5);
    assert!(bar.hover_move(p0.0, p0.1));
    pinta(&mut bar, &mut ts);
    let sub = bar.painted.sub.clone().expect("aberto").popup;
    // Na diagonal até ao fundo do submenu: o próximo ponto cai na mãe de baixo, dentro do triângulo.
    let alvo = (sub.x, sub.y + sub.h * 0.9);
    let t = (baixo.y + baixo.h * 0.3 - p0.1) / (alvo.1 - p0.1);
    let p1 = (p0.0 + (alvo.0 - p0.0) * t, baixo.y + baixo.h * 0.3);
    assert!(
        baixo.contains(p1.0, p1.1),
        "controlo: o ponto está na mãe de baixo"
    );
    assert!(bar.hover_move(p1.0, p1.1));
    assert_eq!(bar.sub, Some(SUB), "a mira segura o submenu");
    pinta(&mut bar, &mut ts);
    // A direito para baixo (fora do triângulo): troca.
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta(&mut bar, &mut ts);
    assert!(bar.hover_move(p0.0, p0.1));
    pinta(&mut bar, &mut ts);
    assert!(bar.hover_move(p0.0 - 30.0, baixo.y + baixo.h * 0.5));
    assert_eq!(bar.sub, Some(S2), "fora da mira troca");
}

/// O smoke do dono 11/11 («o submenu trocou no caminho»): o rato REAL vai em passos de 1–2 px, com
/// tremor (passos só para baixo, até um para trás). A mira parte da âncora, não do passo anterior: o
/// caminho inteiro até «Cards» segura o submenu; parado na mãe de baixo mais do que a pausa, troca.
#[test]
fn a_diagonal_com_tremor_da_mao_segura_o_submenu_e_a_pausa_troca() {
    use ph2d_panel_code::tab_bar::menu::AIM_REST_NS;
    let mut ts = TextSystem::without_system_fonts();
    let its = vec![
        MenuItem::Sub {
            id: SUB,
            name: "Card Style".into(),
            rows: vec![
                (C1, "Sheet".into(), true),
                (C2, "Cards".into(), false),
                (A, "Strip".into(), false),
            ],
        },
        MenuItem::Sub {
            id: S2,
            name: "Text rendering".into(),
            rows: vec![(hash_node_id("t_d"), "Default".into(), true)],
        },
    ];
    let pinta = |bar: &mut TabBar<u8>, ts: &mut TextSystem| {
        let (mut scene, theme) = (VectorScene::new(), Theme::default());
        let crumb = Crumb {
            script: "a.luau".into(),
            func: None,
            line: 1,
        };
        let r = Rect::new(0.0, 0.0, 900.0, height());
        bar.paint(r, &[(0u8, "View")], &crumb, &mut scene, ts, &theme);
        bar.paint_popup(&its, &mut scene, ts, &theme, (0, 0));
    };
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta(&mut bar, &mut ts);
    let row = |bar: &TabBar<u8>, id| bar.painted.rows.iter().find(|r| r.0 == id).expect("mãe").1;
    let (cima, baixo) = (row(&bar, SUB), row(&bar, S2));
    let p0 = (cima.x + cima.w * 0.5, cima.y + cima.h * 0.8);
    let mut t = 1_000_000_000u128;
    assert!(bar.hover_move_at(p0.0, p0.1, t));
    pinta(&mut bar, &mut ts);
    let sub = bar.painted.sub.clone().expect("aberto");
    let alvo = sub.rows[2].1;
    let alvo = (alvo.x + alvo.w * 0.3, alvo.y + alvo.h * 0.5);
    let (mut x, mut y) = p0;
    let mut na_mae_de_baixo = 0;
    let mut k = 0u32;
    while x < alvo.0 {
        // Passos de mão: a diagonal média, mas em rajadas — só para baixo, só para a direita, um recuo.
        let (dx, dy) = match k % 5 {
            0 => (0.0, 2.0),
            1 => (2.0, 0.0),
            2 => (-1.0, 1.0),
            _ => (3.0, 1.0),
        };
        let slope = (alvo.1 - p0.1) / (alvo.0 - p0.0);
        (x, y) = (x + dx, (y + dy).min(p0.1 + (x + dx - p0.0) * slope + 4.0));
        k += 1;
        t += 8_000_000;
        assert!(bar.hover_move_at(x, y, t));
        na_mae_de_baixo += usize::from(baixo.contains(x, y));
        assert_eq!(
            bar.sub,
            Some(SUB),
            "o tremor em ({x}, {y}) trocou o submenu"
        );
        pinta(&mut bar, &mut ts);
    }
    assert!(
        na_mae_de_baixo > 3,
        "controlo: o caminho passa pela mãe de baixo"
    );
    // Parado na mãe de baixo mais do que a pausa: troca.
    let mut bar = TabBar::<u8> {
        open: Some(0),
        ..TabBar::default()
    };
    pinta(&mut bar, &mut ts);
    assert!(bar.hover_move_at(p0.0, p0.1, t));
    pinta(&mut bar, &mut ts);
    let ey = baixo.y + baixo.h * 0.3;
    let em = (p0.0 + (alvo.0 - p0.0) * (ey - p0.1) / (alvo.1 - p0.1), ey);
    assert!(baixo.contains(em.0, em.1), "controlo: na mãe de baixo");
    assert!(bar.hover_move_at(em.0, em.1, t + 8_000_000));
    assert_eq!(bar.sub, Some(SUB), "controlo: a caminho ainda mira");
    assert!(bar.hover_move_at(em.0 + 1.0, em.1, t + 8_000_000 + AIM_REST_NS + 1));
    assert_eq!(bar.sub, Some(S2), "a pausa acaba a mira");
    // Uma folga de meia linha abaixo do submenu: mirar logo abaixo da última escolha ainda segura.
    let novo = |bar: &mut TabBar<u8>, ts: &mut TextSystem| {
        *bar = TabBar::<u8> {
            open: Some(0),
            ..TabBar::default()
        };
        pinta(bar, ts);
        assert!(bar.hover_move_at(p0.0, p0.1, t));
        pinta(bar, ts);
    };
    novo(&mut bar, &mut ts);
    let canto = (sub.popup.x, sub.popup.y + sub.popup.h + height() * 0.4);
    let ey = baixo.y + baixo.h * 0.5;
    let fx = p0.0 + (canto.0 - p0.0) * (ey - p0.1) / (canto.1 - p0.1);
    assert!(baixo.contains(fx, ey), "controlo: na mãe de baixo");
    assert!(bar.hover_move_at(fx, ey, t + 8_000_000));
    assert_eq!(bar.sub, Some(SUB), "a folga da borda segura a mira");
    // Dentro do submenu a âncora vem com o rato: voltar dele à mãe de baixo troca logo.
    novo(&mut bar, &mut ts);
    assert!(bar.hover_move_at(alvo.0, alvo.1, t + 8_000_000));
    assert_eq!(bar.hover, Some(A), "controlo: dentro do submenu");
    assert!(bar.hover_move_at(em.0, em.1, t + 16_000_000));
    assert_eq!(
        bar.sub,
        Some(S2),
        "sair do submenu para a mãe de baixo troca"
    );
}

/// Clicar fora do menu e do submenu fecha os dois (o clique é da barra: nada corre).
#[test]
fn clicar_fora_fecha_o_menu_e_o_submenu() {
    let mut ts = TextSystem::without_system_fonts();
    let mut bar = abre(900.0, &mut ts);
    assert!(bar.sub.is_some(), "controlo: aberto");
    assert_eq!(bar.press(860.0, 400.0), Some(MenuPress::Taken));
    assert!(bar.open.is_none() && bar.sub.is_none());
}
