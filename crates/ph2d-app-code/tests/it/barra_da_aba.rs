//! ⭐ **A barra da própria aba** (13 §4 itens 11, 12 e 13b): o menu, o caminho até o cursor e o cartão
//! «Script options», conduzidos por cliques reais nas caixas pintadas.

use crate::{Mesa, centro};
use ph2d_app_code::commands::{CMD_TOGGLE_COMMENT, TabMenu};
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;

const BOB: &str = "local a = 1\n\nfunction update(self, dt)\n  a = a + dt\nend\n";

fn aberta() -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    (m, ids[0])
}

fn clica(m: &mut Mesa, id: ScriptId, r: Rect) {
    let (x, y) = centro(r);
    m.clica(id, x, y, Modifiers::default());
    m.pinta(id);
}

fn titulo(m: &mut Mesa, menu: TabMenu) -> Rect {
    m.docs()
        .bar_painted()
        .titles
        .iter()
        .find(|t| t.0 == menu)
        .expect("o título está pintado")
        .1
}

/// t1: clicar «Edit» abre o menu da aba; clicar «Toggle Comment» comenta a linha do cursor (um passo
/// de undo, o mesmo do Ctrl+/), e o menu fecha.
#[test]
fn o_menu_edit_da_aba_abre_e_toggle_comment_comenta_a_linha() {
    let (mut m, id) = aberta();
    let edit = titulo(&mut m, TabMenu::Edit);
    clica(&mut m, id, edit);
    let row = m
        .docs()
        .bar_painted()
        .rows
        .iter()
        .find(|r| r.0 == CMD_TOGGLE_COMMENT)
        .expect("o menu Edit aberto lista Toggle Comment")
        .1;
    clica(&mut m, id, row);
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(format!("-- {BOB}").as_str()),
        "a 1.ª linha (o cursor) comentada"
    );
    assert!(m.docs().bar_painted().rows.is_empty(), "o menu fechou");
    // Um título de cada um dos cinco menus, e cada linha corre alguma coisa (nenhuma morta).
    for menu in TabMenu::ALL {
        let rows = ph2d_app_code::commands::menu_rows(menu);
        assert!(!rows.is_empty(), "{menu:?}");
        let cmds = m.docs().commands(id.0);
        for r in rows {
            assert!(cmds.iter().any(|c| c.id == r.0), "{} sem despacho", r.1);
        }
    }
}

/// t3: o caminho diz a função que contém o cursor; clicar nela leva o cursor à linha dela.
#[test]
fn o_caminho_mostra_a_funcao_do_cursor_e_clicar_nela_salta_para_ela() {
    let (mut m, id) = aberta();
    assert!(m.docs().go_to(id.0, 3, 4), "o cursor dentro de update");
    m.pinta(id);
    let (r, line, _) = m
        .docs()
        .bar_painted()
        .crumb_fn
        .expect("o caminho tem a função");
    assert_eq!(line, 2, "update começa na linha 3");
    clica(&mut m, id, r);
    assert_eq!(m.docs().cursor_of(id).map(|c| c.0), Some(2));
    // Fora de uma função, o caminho não tem a parte da função.
    assert!(m.docs().go_to(id.0, 0, 0));
    m.pinta(id);
    assert!(m.docs().bar_painted().crumb_fn.is_none());
}

/// t4: a engrenagem abre o cartão; a caixa Strict types põe/tira o `--!strict` como o comando antigo;
/// Format on save guarda o estado; o alfinete segura o cartão a um clique fora.
#[test]
fn o_cartao_script_options_liga_as_caixas_e_o_alfinete_o_segura() {
    let (mut m, id) = aberta();
    let gear = m.docs().bar_painted().gear.expect("a engrenagem");
    clica(&mut m, id, gear);
    let caixa = |m: &mut Mesa, cmd| {
        m.docs()
            .options_card()
            .painted
            .rows
            .iter()
            .find(|r| r.0 == cmd)
            .expect("a caixa pintada")
            .1
    };
    let strict = caixa(&mut m, ph2d_app_code::commands::CMD_STRICT);
    clica(&mut m, id, strict);
    assert_eq!(m.docs().text_of(id), Some(format!("--!strict\n{BOB}")));
    let fmt = caixa(&mut m, ph2d_app_code::commands::CMD_FORMAT_ON_SAVE);
    let antes = m.docs().format_on();
    clica(&mut m, id, fmt);
    assert_eq!(m.docs().format_on(), !antes, "Format on save trocou");
    assert!(m.docs().options_card().open, "a caixa não fecha o cartão");
    assert!(
        m.docs()
            .options_card()
            .painted
            .rows
            .iter()
            .any(|r| r.0 == ph2d_app_code::commands::CMD_RESTART_ASSIST),
        "o «Restart» do cartão é o comando do Run ▸ Restart Code Assistant"
    );
    // Sem alfinete, um clique no código fecha-o; com ele, fica.
    let pin = m.docs().options_card().painted.pin.expect("o alfinete");
    clica(&mut m, id, pin);
    let code = m.docs().card_rects(id).expect("cartões")[1];
    let fora = Rect::new(code.x + code.w * 0.25, code.y + code.h * 0.5, 1.0, 1.0);
    clica(&mut m, id, fora);
    assert!(m.docs().options_card().open, "com alfinete fica");
    clica(&mut m, id, pin);
    clica(&mut m, id, fora);
    assert!(!m.docs().options_card().open, "sem alfinete fecha");
}

fn dentro(card: Rect, r: Rect) -> bool {
    r.x >= card.x && r.y >= card.y && r.x + r.w <= card.x + card.w && r.y + r.h <= card.y + card.h
}

/// t5: toda caixa pintada do cartão (as linhas, as marcas, o alfinete e o «Restart») cabe dentro dele,
/// e as marcas, o alfinete e o «Restart» partilham a borda direita (uma grelha).
#[test]
fn o_cartao_script_options_pinta_tudo_dentro_dele_numa_grelha() {
    let (mut m, id) = aberta();
    m.docs().set_options_open(true);
    m.pinta(id);
    let p = m.docs().options_card().painted.clone();
    let card = p.card.expect("o cartão");
    let pin = p.pin.expect("o alfinete");
    assert_eq!(p.marks.len(), 3, "três marcas");
    for r in p
        .rows
        .iter()
        .map(|r| r.1)
        .chain(p.marks.iter().copied())
        .chain([pin])
    {
        assert!(dentro(card, r), "{r:?} sai de {card:?}");
    }
    let restart = p.rows.last().expect("o Restart").1;
    let borda = restart.x + restart.w;
    for r in p.marks.iter().chain([&pin]) {
        assert!(
            (r.x + r.w - borda).abs() < 0.5,
            "{r:?} fora da borda {borda}"
        );
    }
}

/// t5b (14 Q4 D): um clique real na FRASE de Format on save (a linha inteira é o alvo) alterna-a; a
/// mão e a moldura de hover sobre a linha; o interruptor repintado mostra o estado novo.
#[test]
fn clicar_a_frase_de_uma_opcao_alterna_a_e_o_interruptor_mostra_o() {
    use ph2d_editor_core::doc_surface::PointerLook;
    let (mut m, id) = aberta();
    m.docs().set_options_open(true);
    m.pinta(id);
    let p = m.docs().options_card().painted.clone();
    let fmt = ph2d_app_code::commands::CMD_FORMAT_ON_SAVE;
    let i = p.rows.iter().position(|r| r.0 == fmt).expect("a linha");
    let antes = m.docs().format_on();
    assert_eq!(p.on[i], antes, "o interruptor diz o estado");
    let (x, y) = (p.says[i].x + 1.0, p.says[i].y + p.says[i].h * 0.5);
    m.rato(id, x, y);
    assert_eq!(
        m.docs().options_card().hover,
        Some(fmt),
        "a moldura de hover"
    );
    assert_eq!(m.docs().pointer_look(id.0, x, y), Some(PointerLook::Hand));
    m.clica(id, x, y, Modifiers::default());
    m.pinta(id);
    assert_eq!(m.docs().format_on(), !antes, "a linha alternou");
    let card = m.docs().options_card();
    assert_eq!((card.painted.on[i], card.pressed), (!antes, None));
}

/// t6: o Esc fecha o cartão sem alfinete; com alfinete fica.
#[test]
fn o_esc_fecha_o_cartao_script_options_sem_alfinete() {
    let (mut m, id) = aberta();
    m.docs().set_options_open(true);
    m.pinta(id);
    m.tecla(
        id,
        ph2d_editor_core::doc_surface::Key::Escape,
        Modifiers::default(),
        None,
    );
    assert!(!m.docs().options_card().open, "o Esc fechou-o");
    m.docs().set_options_open(true);
    m.pinta(id);
    let pin = m.docs().options_card().painted.pin.expect("o alfinete");
    clica(&mut m, id, pin);
    m.tecla(
        id,
        ph2d_editor_core::doc_surface::Key::Escape,
        Modifiers::default(),
        None,
    );
    assert!(m.docs().options_card().open, "com alfinete fica");
}

/// t7: o menu Edit tem UMA linha com o Alt+F, «Toggle Fold», e ela faz o que a tecla faz: dobra o
/// bloco do cursor e, de novo, abre-o. «Fold All» e «Unfold All» continuam.
#[test]
fn o_menu_edit_tem_uma_linha_toggle_fold_que_faz_o_que_o_alt_f_faz() {
    use ph2d_app_code::commands::{CMD_FOLD_ALL, CMD_TOGGLE_FOLD, CMD_UNFOLD_ALL, menu_rows};
    let rows = menu_rows(TabMenu::Edit);
    let alt_f: Vec<_> = rows.iter().filter(|r| r.2 == "Alt+F").collect();
    assert_eq!(alt_f.len(), 1, "uma linha por tecla: {rows:?}");
    assert_eq!(
        (alt_f[0].0, alt_f[0].1.as_str()),
        (CMD_TOGGLE_FOLD, "Toggle Fold")
    );
    assert!(rows.iter().any(|r| r.0 == CMD_FOLD_ALL));
    assert!(rows.iter().any(|r| r.0 == CMD_UNFOLD_ALL));
    let (mut m, id) = aberta();
    assert!(m.docs().go_to(id.0, 3, 4), "o cursor dentro de update");
    let _ = m.docs().command(id.0, CMD_TOGGLE_FOLD);
    assert_eq!(m.docs().closed_folds(id), vec![(2, 4)]);
    let _ = m.docs().command(id.0, CMD_TOGGLE_FOLD);
    assert_eq!(m.docs().closed_folds(id), Vec::<(usize, usize)>::new());
}

/// Dono 10/11: títulos e linhas do menu na letra de «alive» (a mono do código: fonte, corpo, eixo).
#[test]
fn os_menus_da_barra_tem_a_letra_do_codigo() {
    use ph2d_app_code::commands::CommandState;
    use ph2d_app_code::tab_menu::{Bar, Crumb, menu_items, menu_titles};
    use ph2d_text::{PositionedLayoutItem as Item, TextSystem};
    use ph2d_tokens::{Spacing, Theme, TypeToken};
    let (mut ts, px) = (
        TextSystem::without_system_fonts(),
        ph2d_text::displayed_font_px(TypeToken::Base.px()),
    );
    let mut cache = ph2d_panel_code::LineCache::default();
    let mut mono = |ts: &mut TextSystem, t: &str| {
        let l = cache.get(ts, t, &[], px);
        let Some(Item::GlyphRun(g)) = l.lines().next().and_then(|l| l.items().next()) else {
            panic!("«{t}» moldado");
        };
        let (r, ids) = (g.run(), g.glyphs().map(|x| x.id).collect::<Vec<u32>>());
        let k = (r.font().data.id(), r.font().index, r.font_size());
        ((k, ph2d_text::run_coord_bits(r)), ids, l.width())
    };
    let (mut bar, mut scene, theme) = (
        Bar::default(),
        ph2d_vector::VectorScene::new(),
        Theme::default(),
    );
    let crumb = Crumb {
        script: "bob.luau".into(),
        func: None,
        line: 1,
    };
    let r = Rect::new(0.0, 0.0, 900.0, ph2d_app_code::tab_menu::height());
    bar.paint(r, &menu_titles(), &crumb, &mut scene, &mut ts, &theme);
    bar.open = Some(TabMenu::Edit);
    let items = menu_items(TabMenu::Edit, CommandState::default(), Default::default());
    bar.paint_popup(&items, &mut scene, &mut ts, &theme, (0, 0));
    let res = &scene.inner().encoding().resources;
    let cena: Vec<_> = res
        .glyph_runs
        .iter()
        .map(|g| {
            let gl = &res.glyphs[g.glyphs.clone()];
            let k = (g.font.data.id(), g.font.index, g.font_size);
            let eixos = res.normalized_coords[g.normalized_coords.clone()].to_vec();
            (
                (k, eixos),
                gl.iter().map(|x| x.id).collect::<Vec<u32>>(),
                g.transform.translation[0] + gl[0].x,
            )
        })
        .collect();
    let rows = ph2d_app_code::commands::menu_rows_in(TabMenu::Edit, CommandState::default());
    let titulos = bar
        .painted
        .titles
        .iter()
        .map(|(m, tr)| (m.title().to_owned(), Some(*tr)));
    for (t, caixa) in titulos.chain(rows.into_iter().map(|r| (r.1, None))) {
        let (letra, ids, w) = mono(&mut ts, &t);
        let achada = cena
            .iter()
            .find(|c| c.1 == ids)
            .unwrap_or_else(|| panic!("«{t}»: não é mono"));
        assert_eq!(achada.0, letra, "«{t}»: não é a letra de «alive»");
        if let Some(tr) = caixa {
            assert!(
                (tr.w - w - Spacing::Sm.px() * 2.0).abs() <= 1.0,
                "«{t}»: caixa {tr:?} × {w}"
            );
            assert!(
                achada.2 >= tr.x && achada.2 + w <= tr.x + tr.w + 1.0,
                "«{t}»: o alvo não cobre"
            );
        }
    }
}

/// Dono 10/11: o fio da barra (`cards::bar_rule`) pinta-se; a linha 1 respira abaixo dele.
#[test]
fn um_fio_separa_a_barra_do_codigo_e_a_linha_1_respira_abaixo_dele() {
    let (mut m, id) = aberta();
    let p = m.pinta(id).inner().encoding().path_data.clone();
    let f = ph2d_panel_code::cards::bar_rule(m.docs().bar_painted().header);
    let has = |x: f32, y: f32| p.as_chunks::<2>().0.contains(&[x.to_bits(), y.to_bits()]);
    assert!(has(f.x, f.y) && has(f.x + f.w, f.y + f.h), "o fio {f:?}");
    let mid = m.docs().glyph_center(id, 0).expect("a linha 1").1;
    assert!(mid > f.y + f.h + ph2d_tokens::Spacing::Md.px(), "{mid}");
}

/// ⭐ `View ▸ Card Style ▸` e `View ▸ Text rendering ▸` (dono 10/11 e 11/11) são submenus: as escolhas
/// saíram das linhas soltas; pairar na mãe abre-o à direita com o ✓ no em vigor; escolher muda o estilo
/// do quadro / a letra do código, fecha tudo e o ✓ segue; o Esc da aba fecha um nível de cada vez.
/// (A mecânica fina — outra linha, a mira, clicar fora, a posição — está em `ph2d-panel-code`, `submenu`.)
#[test]
fn os_submenus_do_view_escolhem_o_estilo_e_a_letra() {
    use ph2d_app_code::commands::{CMD_CARD_CARDS, CMD_CARD_SHEET, CMD_CARD_STRIP};
    use ph2d_app_code::tab_menu::{ROW_CARD_STYLE, ROW_TEXT_RENDERING};
    use ph2d_code_debugcard::CardStyle::{Cards, Sheet, Strip};
    use ph2d_panel_code::code_text::{ROWS, mode, set_mode};
    use ph2d_tokens::TextRendering;
    let (mut m, id) = aberta();
    let abre = |m: &mut Mesa, mae| {
        let view = titulo(m, TabMenu::View);
        clica(m, id, view);
        let p = m.docs().bar_painted().clone();
        assert!(p.sub.is_none(), "fechado até pairar");
        let (x, y) = centro(p.rows.iter().find(|r| r.0 == mae).expect("a mãe").1);
        m.rato(id, x, y);
        m.pinta(id);
        let sub = m.docs().bar_painted().sub.clone().expect("pairar abre");
        let menu = p.popup.expect("o menu");
        assert!(sub.popup.x >= menu.x + menu.w - 0.5, "à direita");
        (p, sub)
    };
    let esc = |m: &mut Mesa| {
        m.tecla(
            id,
            ph2d_editor_core::doc_surface::Key::Escape,
            Modifiers::default(),
            None,
        );
        m.pinta(id);
        m.docs().bar_painted().clone()
    };
    let estilos = vec![CMD_CARD_SHEET, CMD_CARD_CARDS, CMD_CARD_STRIP];
    for (mae, ids) in [
        (ROW_CARD_STYLE, estilos),
        (ROW_TEXT_RENDERING, ROWS.map(|r| r.0).to_vec()),
    ] {
        let (p, sub) = abre(&mut m, mae);
        assert!(
            ids.iter().all(|c| p.rows.iter().all(|r| r.0 != *c)),
            "nada solto"
        );
        assert_eq!(sub.rows.iter().map(|r| r.0).collect::<Vec<_>>(), ids);
        assert_eq!(sub.checked, [ids[0]], "de fábrica");
        let _ = (esc(&mut m), esc(&mut m));
        for i in (0..ids.len()).rev() {
            let (_, sub) = abre(&mut m, mae);
            let (x, y) = centro(sub.rows[i].1);
            m.rato(id, x, y);
            clica(&mut m, id, sub.rows[i].1);
            let q = m.docs().bar_painted();
            assert!(q.popup.is_none() && q.sub.is_none(), "escolher fecha tudo");
            if mae == ROW_TEXT_RENDERING {
                assert_eq!(mode(), TextRendering::ALL[i]);
            } else {
                assert_eq!(m.docs().card_style, [Sheet, Cards, Strip][i]);
            }
            assert_eq!(abre(&mut m, mae).1.checked, [ids[i]], "o ✓ segue");
            let um = esc(&mut m);
            assert!(
                um.sub.is_none() && um.popup.is_some(),
                "o Esc fecha só o submenu"
            );
            assert!(esc(&mut m).popup.is_none(), "o 2.º Esc fecha o menu");
        }
    }
    set_mode(TextRendering::Default);
}
