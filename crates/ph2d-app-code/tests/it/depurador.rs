//! Onda D (`docs/CodeEditor/13_plano_bancada.md` §4): o depurador pela porta do artista — o clique
//! na coluna das marcas e o F9 põem o ponto, o ponto segue as edições e chega ao `ScriptHost`, o
//! jogo parado mostra o cartão «Stopped» cujo Continue chega ao host, e os valores ao vivo são só
//! os do script da aba aberta.

use crate::{Mesa, centro};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;
use ph2d_script::{NoDoors, ScriptId};

const MOVE: &str = "function init(self)\n  self.x = 0\nend\n\nfunction update(self, dt)\n  local step = 2\n  self.x = self.x + step\nend\n";

fn aberto(texto: &str) -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("move.luau", texto)]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    (m, ids[0])
}

fn vai(m: &mut Mesa, id: ScriptId, line: usize) {
    let _ = m.docs().go_to(id.0, line, 0);
}

#[test]
fn o_clique_na_marca_e_o_f9_poem_e_tiram_o_ponto_de_parada() {
    let (mut m, id) = aberto(MOVE);
    let r = m
        .docs()
        .mark_rect(id, "  local step")
        .expect("a marca pintada");
    let (x, y) = centro(r);
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(m.docs().breakpoints_of(id), vec![5], "o clique põe o ponto");
    m.clica(id, x, y, Modifiers::default());
    assert!(
        m.docs().breakpoints_of(id).is_empty(),
        "o 2.º clique tira-o"
    );

    vai(&mut m, id, 6);
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    assert_eq!(
        m.docs().breakpoints_of(id),
        vec![6],
        "F9 na linha do cursor"
    );
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    assert!(m.docs().breakpoints_of(id).is_empty());
}

#[test]
fn o_ponto_de_parada_segue_as_edicoes_de_cima() {
    let (mut m, id) = aberto(MOVE);
    vai(&mut m, id, 6);
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    vai(&mut m, id, 0);
    m.escreve(id, "-- top\n");
    assert_eq!(
        m.docs().breakpoints_of(id),
        vec![7],
        "uma linha acima empurra-o"
    );
    m.tecla(id, Key::Char('z'), crate::ctrl(), None);
    assert_eq!(m.docs().breakpoints_of(id), vec![6], "desfazer devolve-o");
}

#[test]
fn o_ponto_de_parada_chega_ao_host_e_a_marca_diz_se_armou() {
    let (m, id) = aberto(MOVE);
    let mut m = m.com_cena(id, 1);
    m.cena();
    vai(&mut m, id, 6);
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    m.quadro();
    let h = m.host.as_ref().expect("vm");
    assert_eq!(h.breakpoints(id), &[7]);
    assert_eq!(h.armed_lines(id), &[7]);
    assert_eq!(m.docs().breakpoint_marks_of(id), vec![(6, true)]);
}

#[test]
fn parado_o_cartao_diz_onde_e_o_continue_do_botao_chega_ao_host() {
    let (m, id) = aberto(MOVE);
    let mut m = m.com_cena(id, 2);
    m.cena();
    vai(&mut m, id, 6);
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    m.quadro();
    m.cena();
    assert!(m.host.as_ref().expect("vm").is_held(), "bateu na linha 7");
    m.quadro();
    m.pinta(id);
    let (title, error, rows) = m.docs().debug_card_of(id).expect("o cartão");
    assert!(
        title.starts_with("Stopped at line 7 \u{2014} on "),
        "{title}"
    );
    assert!(error.is_none());
    assert!(rows.iter().any(|r| r == "1 more waiting"), "{rows:?}");
    assert!(rows.iter().any(|r| r == "step = 2"), "{rows:?}");
    assert!(rows.iter().any(|r| r.starts_with("self.x = ")), "{rows:?}");
    assert_eq!(m.docs().stopped_line_of(id), Some(6));
    let buttons = m.docs().debug_buttons_of(id);
    assert_eq!(buttons.len(), 2, "Continue e Next line");
    let (x, y) = centro(buttons[0]);
    m.clica(id, x, y, Modifiers::default());
    m.quadro();
    let h = m.host.as_mut().expect("vm");
    assert!(
        h.debug_frame_with(&mut m.sim, &mut NoDoors).is_some(),
        "o Continue foi pedido ao host"
    );
    let h = m.host.as_ref().expect("vm");
    let stop = h.debug_stop().expect("o 2.º objecto parou no mesmo ponto");
    assert!(stop.waiting.is_empty());
}

#[test]
fn os_valores_ao_vivo_sao_so_os_da_aba_aberta() {
    let (mut m, ids) = Mesa::new(&[("move.luau", MOVE), ("other.luau", MOVE)]);
    m.pede(ids[1]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    let mut m = m.com_cena(ids[0], 1).com_cena(ids[1], 1);
    m.quadro();
    for _ in 0..3 {
        m.cena();
        m.host.as_mut().expect("vm").debug_end_frame();
        m.quadro();
    }
    let ghosts = m.docs().ghosts_of(ids[0]);
    assert!(
        ghosts
            .iter()
            .any(|(l, t)| *l == 5 && t.contains("step = 2")),
        "{ghosts:?}"
    );
    assert!(m.docs().ghosts_of(ids[1]).is_empty(), "a aba de trás não");
    let (title, _, _) = m.docs().debug_card_of(ids[0]).expect("de quem são");
    assert!(title.starts_with("Live values: "), "{title}");
}

// ── Auditoria do fecho (08/11) ────────────────────────────────────────────────────────────────

#[test]
fn o_clique_abaixo_da_ultima_linha_nao_poe_ponto_de_parada() {
    let (mut m, id) = aberto(MOVE);
    let r = m
        .docs()
        .mark_rect(id, "  self.x = self.x")
        .expect("a marca");
    let (x, _) = centro(r);
    // Três linhas abaixo desta acaba o texto («end» e a vazia); seis abaixo já não é linha.
    m.clica(id, x, r.y + r.h * 6.5, Modifiers::default());
    assert!(
        m.docs().breakpoints_of(id).is_empty(),
        "abaixo do fim não há linha"
    );
    m.clica(id, x, r.y + r.h * 2.5, Modifiers::default());
    assert_eq!(
        m.docs().breakpoints_of(id),
        vec![8],
        "a última linha (vazia) é linha"
    );
}

/// O motor respondeu pelo script e não armou NADA (aqui: o módulo não carrega): a marca é oca,
/// mesmo com a lista do que armou vazia (antes, «vazia» lia-se «ainda não sei» e pintava cheio).
#[test]
fn o_ponto_que_o_motor_nao_armou_e_oco_mesmo_sem_nada_armado() {
    let (m, id) = aberto("function update(self, dt)\n  local step = 2\nend\n\nlocal x = = 1\n");
    let mut m = m.com_cena(id, 1);
    m.cena();
    vai(&mut m, id, 3);
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    m.quadro();
    let armed = m.host.as_ref().expect("vm").armed_lines(id).to_vec();
    assert!(armed.is_empty(), "{armed:?}");
    assert_eq!(
        m.docs().breakpoint_marks_of(id),
        vec![(3, false)],
        "o motor respondeu e não armou: oco"
    );
}

#[test]
fn parado_em_ecra_cheio_o_palco_cede_ao_editor_e_o_f8_continua() {
    let (m, id) = aberto(MOVE);
    let mut m = m.com_cena(id, 2);
    m.hero.view.palco_do_jogo = true;
    m.cena();
    vai(&mut m, id, 6);
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    m.quadro();
    m.cena();
    assert!(m.host.as_ref().expect("vm").is_held());
    m.quadro();
    assert!(!m.hero.view.palco_do_jogo, "o editor volta a pintar-se");
    assert!(m.hero.view.palco_largado, "a janela do jogo fecha");
    m.tecla(id, Key::F(8), Modifiers::default(), None);
    m.quadro();
    let h = m.host.as_mut().expect("vm");
    assert!(
        h.debug_frame_with(&mut m.sim, &mut NoDoors).is_some(),
        "o F8 parado é o Continue"
    );
}

#[test]
fn o_menu_run_so_deixa_continuar_parado_e_o_f8_e_de_uma_linha_so() {
    use ph2d_app_code::GameStop;
    use ph2d_app_code::commands::{CMD_CONTINUE, CMD_NEXT_LINE, CMD_NEXT_PROBLEM, row_state};
    assert_eq!(row_state(CMD_CONTINUE, GameStop::Running), (false, false));
    assert_eq!(row_state(CMD_NEXT_LINE, GameStop::Running), (false, false));
    assert_eq!(row_state(CMD_NEXT_PROBLEM, GameStop::Running), (true, true));
    assert_eq!(row_state(CMD_CONTINUE, GameStop::Line), (true, true));
    assert_eq!(row_state(CMD_NEXT_LINE, GameStop::Line), (true, true));
    assert_eq!(row_state(CMD_NEXT_PROBLEM, GameStop::Line), (true, false));
    assert_eq!(row_state(CMD_CONTINUE, GameStop::Error), (true, true));
    assert_eq!(
        row_state(CMD_NEXT_LINE, GameStop::Error),
        (false, false),
        "num erro não"
    );
}

#[test]
fn a_engrenagem_e_a_funcao_do_caminho_tem_dica() {
    let (mut m, id) = aberto(MOVE);
    vai(&mut m, id, 6);
    m.pinta(id);
    let tips = m.docs().bar_painted().tips.clone();
    assert!(tips.iter().any(|t| t.1 == "Script options"), "{tips:?}");
    assert!(tips.iter().any(|t| t.1 == "Go to update"), "{tips:?}");
}

/// Plano 14 (decisão do dono 13): `View ▸ Card Style` troca a aparência do Stopped (Sheet por
/// omissão → Cards → Strip → Sheet), o ▸ de uma tabela abre-a, e o botão direito no cabeçalho da
/// Sheet esconde a coluna Type — tudo por cliques reais nas caixas pintadas.
#[test]
fn o_card_style_troca_o_stopped_e_o_cabecalho_esconde_a_coluna_type() {
    use ph2d_app_code::commands::{CMD_CARD_CARDS, CMD_CARD_SHEET, CMD_CARD_STRIP};
    use ph2d_code_debugcard::CardStyle::{Cards, Sheet, Strip};
    const T: &str = "function update(self, dt)\n  self.pos = { x = 1 }\n  local s = 2 -- @\nend\n";
    let (m, id) = aberto(T);
    let mut m = m.com_cena(id, 1);
    m.cena();
    vai(&mut m, id, 2);
    m.tecla(id, Key::F(9), Modifiers::default(), None);
    let passo = |m: &mut Mesa| {
        m.quadro();
        m.pinta(id);
        m.docs().debug_view_of(id).expect("o quadro").clone()
    };
    passo(&mut m);
    m.cena();
    let v = passo(&mut m);
    assert_eq!(v.style, Sheet, "Sheet por omissão");
    let keys = |v: &ph2d_panel_code::debug_card::DebugCardView| {
        let (lay, rows, _) = v.painted.table.clone().expect("a Sheet pinta a tabela");
        let at: Vec<String> = lay.rows.iter().map(|s| rows[s.index].key.clone()).collect();
        (lay, at)
    };
    let (lay, at) = keys(&v);
    assert!(
        lay.col("type").is_some() && at.contains(&"#self".into()),
        "{at:?}"
    );
    assert!(!at.contains(&"self.pos.x".into()), "a tabela dobrada");
    let (x, y) = centro(v.chevron_of("self.pos").expect("o ▸"));
    m.clica(id, x, y, Modifiers::default());
    let (lay, at) = keys(&passo(&mut m));
    assert!(at.contains(&"self.pos.x".into()), "o ▸ abriu-a");
    // O botão direito no cabeçalho (o do último quadro: a tabela aberta fê-lo crescer) abre o
    // menu das colunas; «Type» esconde-a.
    let (x, y) = centro(lay.header);
    m.clica_com(
        id,
        x,
        y,
        Modifiers::default(),
        ph2d_host::PointerButton::Secondary,
    );
    m.pinta(id);
    let menu = m
        .docs()
        .context_painted()
        .expect("o menu das colunas")
        .clone();
    let (x, y) = centro(menu.rows.iter().find(|r| r.0 == "Type").expect("Type").1);
    m.clica(id, x, y, Modifiers::default());
    assert!(
        keys(&passo(&mut m)).0.col("type").is_none(),
        "Type escondida"
    );
    // Cards e Strip pintam caixas de valor (clicar copia), sem tabela; e de volta à Sheet.
    for (cmd, style) in [
        (CMD_CARD_CARDS, Cards),
        (CMD_CARD_STRIP, Strip),
        (CMD_CARD_SHEET, Sheet),
    ] {
        assert!(m.docs().command(id.0, cmd));
        let v = passo(&mut m);
        assert_eq!(v.style, style);
        assert_eq!(v.painted.table.is_none(), style != Sheet);
        assert_eq!(v.painted.values.is_empty(), style == Sheet, "{style:?}");
    }
}
