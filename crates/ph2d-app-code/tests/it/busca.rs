//! ⭐ **E9 — buscar/substituir e ir para linha**, pela barra pintada e pelo teclado.

use crate::{Mesa, centro, ctrl};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_panel_code::FindHit;

fn linhas(n: usize) -> String {
    (1..=n)
        .map(|i| format!("local vida{i} = vida + {i}\n"))
        .collect()
}

/// Substituir tudo é UM passo de desfazer — e o clique é no botão PINTADO.
#[test]
fn substituir_tudo_e_um_passo_de_undo() {
    let original = linhas(12);
    let (mut m, ids) = Mesa::new(&[("bob.luau", &original)]);
    m.pede(ids[0]);
    m.tecla(ids[0], Key::Char('h'), ctrl(), Some("h"));
    m.escreve(ids[0], "vida");
    assert_eq!(m.docs().find_count_text(ids[0]).as_deref(), Some("1 of 24"));
    m.tecla(ids[0], Key::Tab, Modifiers::default(), None);
    m.escreve(ids[0], "hp");
    m.pinta(ids[0]);
    let r = m
        .docs()
        .find_target(ids[0], FindHit::ReplaceAll)
        .expect("o botão foi pintado");
    let (x, y) = centro(r);
    m.clica(ids[0], x, y, Modifiers::default());
    let novo = m.docs().text_of(ids[0]).expect("aberto");
    assert_eq!(novo, original.replace("vida", "hp"));
    assert_eq!(
        m.docs().find_count_text(ids[0]).as_deref(),
        Some("No results"),
        "a busca vazia di-lo no ecrã"
    );
    m.tecla(ids[0], Key::Escape, Modifiers::default(), None);
    m.tecla(ids[0], Key::Char('z'), ctrl(), Some("z"));
    assert_eq!(
        m.docs().text_of(ids[0]),
        Some(original),
        "um Ctrl+Z desfaz as 24 trocas"
    );
}

/// Uma busca sem resultado diz «No results»; com resultados, «k of n» segue o Enter.
#[test]
fn a_contagem_no_ecra() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", &linhas(3))]);
    m.pede(ids[0]);
    m.tecla(ids[0], Key::Char('f'), ctrl(), Some("f"));
    m.escreve(ids[0], "zzz");
    assert_eq!(
        m.docs().find_count_text(ids[0]).as_deref(),
        Some("No results")
    );
    m.tecla(ids[0], Key::Char('a'), ctrl(), Some("a"));
    m.escreve(ids[0], "local");
    assert_eq!(m.docs().find_count_text(ids[0]).as_deref(), Some("1 of 3"));
    m.tecla(ids[0], Key::Enter, Modifiers::default(), None);
    m.tecla(ids[0], Key::Enter, Modifiers::default(), None);
    assert_eq!(m.docs().find_count_text(ids[0]).as_deref(), Some("2 of 3"));
    assert_eq!(
        m.docs().cursor_of(ids[0]).map(|c| c.0),
        Some(1),
        "o Enter leva o cursor à ocorrência"
    );
}

/// Ctrl+G 40 ⇒ o cursor na linha 40 e a vista mostra-a.
#[test]
fn ctrl_g_40_poe_o_cursor_na_40_e_a_vista_mostra_a() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", &linhas(300))]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    assert!(
        !m.docs()
            .visible_lines(ids[0])
            .expect("aberto")
            .contains(&39),
        "controlo: a 40 começa fora da vista"
    );
    m.tecla(ids[0], Key::Char('g'), ctrl(), Some("g"));
    m.escreve(ids[0], "40");
    m.tecla(ids[0], Key::Enter, Modifiers::default(), None);
    m.pinta(ids[0]);
    assert_eq!(m.docs().cursor_of(ids[0]).map(|c| c.0), Some(39));
    assert!(
        m.docs()
            .visible_lines(ids[0])
            .expect("aberto")
            .contains(&39)
    );
}

/// O trabalho da busca numa tecla escrita no TEXTO (tecla + quadro pintado): bytes lidos pela busca.
/// Nos dois casos procurou-se `vida`; `aberta = false` fecha a barra com o Esc antes das teclas.
fn bytes_por_tecla(texto: &str, aberta: bool) -> u64 {
    let (mut m, ids) = Mesa::new(&[("bob.luau", texto)]);
    let id = ids[0];
    m.pede(id);
    m.tecla(id, Key::Char('f'), ctrl(), Some("f"));
    m.escreve(id, "vida");
    m.pinta(id);
    if aberta {
        // A barra fica aberta e o teclado volta ao texto (um clique no código).
        let (x, y) = (crate::AREA.x + crate::AREA.w * 0.6, crate::AREA.y + crate::AREA.h * 0.4);
        m.clica(id, x, y, Modifiers::default());
    } else {
        m.tecla(id, Key::Escape, Modifiers::default(), None);
    }
    m.pinta(id);
    let antes = ph2d_code_buffer::find_bytes_scanned();
    const TECLAS: u64 = 5;
    for c in ['x', 'w', 'm', 'z', 'k'] {
        m.tecla(id, Key::Other, Modifiers::default(), Some(&c.to_string()));
        m.pinta(id);
    }
    if aberta {
        assert!(m.docs().find_count_text(id).is_some(), "controlo: a barra continua aberta");
    }
    (ph2d_code_buffer::find_bytes_scanned() - antes) / TECLAS
}

/// ⭐ **Gate de custo (sem relógio), item 10 da auditoria:** a busca FECHADA não lê nada por tecla;
/// ABERTA, o que ela lê por tecla não cresce com o ficheiro (a busca segue as edições em vez de
/// varrer o texto inteiro a cada revisão).
#[test]
fn a_busca_por_tecla_nao_cresce_com_o_ficheiro_e_fechada_nao_custa_nada() {
    let (pequeno, grande) = (linhas(50), linhas(7_500));
    let fechada = bytes_por_tecla(&grande, false);
    let (ap, ag) = (bytes_por_tecla(&pequeno, true), bytes_por_tecla(&grande, true));
    eprintln!("busca por tecla: fechada {fechada} B · aberta {ap} B (50 linhas) · {ag} B (7 500)");
    assert_eq!(fechada, 0, "a busca fechada leu {fechada} bytes por tecla");
    assert!(ap > 0, "controlo: a régua conta a busca aberta");
    assert!(ag <= ap * 2, "aberta: {ap} B por tecla a 50 linhas contra {ag} B a 7 500");
}
