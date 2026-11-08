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
