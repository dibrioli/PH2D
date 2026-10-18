//! I14 (Onda 2, `docs/CodeEditor/03_onda2.md`): a paleta `Ctrl+K` com os prefixos — `@` os símbolos
//! do script activo, `:` a linha, `>` só comandos — pelo caminho do produto (o hero com a família
//! instalada, a abertura e o dreno da shell); e o `--!strict` por ficheiro (o comando e a barra).

use crate::{Mesa, ctrl};
use ph2d_app_code::commands::CMD_STRICT;
use ph2d_editor_core::doc_surface::{DocSymbolKind, DocumentSurface, Key};
use ph2d_editor_core::screens::hero::global_palette;
use ph2d_script::ScriptId;

const BOB: &str = "-- bob\nph2d.property(\"speed\", 2)\nlocal LIMITE = 3\nfunction init(self)\n  self.t = 0\nend\n\nfunction update(self, dt)\n  local s = dt\nend\n";

fn aberta(texto: &str) -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("bob.luau", texto)]);
    m.pede(ids[0]);
    (m, ids[0])
}

/// Abre o `Ctrl+K`, escreve `q` e carrega Enter (as portas da shell: `open`, o teclado no store, o
/// `command_palette_top`, o dreno).
fn paleta(m: &mut Mesa, q: &str) {
    global_palette::open(&mut m.hero);
    for c in q.chars() {
        m.hero.store.command_palette_push_char(c);
    }
    if let Some(id) = m.hero.store.command_palette_top() {
        m.hero.store.set_command_pick(id);
        m.hero.store.close_command_palette();
    }
    global_palette::drain(&mut m.hero);
}

#[test]
fn os_simbolos_e_as_linhas_do_script_activo() {
    let (mut m, id) = aberta(BOB);
    let p = m.docs().places(id.0).expect("a aba de código tem sítios");
    let got: Vec<(String, DocSymbolKind, usize, usize)> = p
        .symbols
        .iter()
        .map(|s| (s.name.clone(), s.kind, s.line, s.col))
        .collect();
    assert_eq!(
        got,
        [
            ("speed".to_owned(), DocSymbolKind::Property, 1, 15),
            ("LIMITE".to_owned(), DocSymbolKind::Variable, 2, 6),
            ("init".to_owned(), DocSymbolKind::Function, 3, 9),
            ("update".to_owned(), DocSymbolKind::Function, 7, 9),
        ],
        "pela ordem do texto; o `s` de dentro do `update` não é símbolo"
    );
    assert_eq!(p.lines, 11);
}

#[test]
fn arroba_leva_ao_inicio_do_nome_e_dois_pontos_a_linha() {
    let (mut m, id) = aberta(BOB);
    paleta(&mut m, "@upd");
    assert_eq!(m.docs().cursor_of(id), Some((7, BOB.find("update(").unwrap())));
    paleta(&mut m, "@");
    assert_eq!(
        m.docs().cursor_of(id),
        Some((3, BOB.find("init(").unwrap())),
        "`@` + Enter: o 1.º símbolo MOSTRADO (o cartão das funções vem primeiro)"
    );
    paleta(&mut m, ":4");
    assert_eq!(m.docs().cursor_of(id), Some((3, BOB.find("function init").unwrap())));
    paleta(&mut m, ":5:3");
    assert_eq!(m.docs().cursor_of(id), Some((4, BOB.find("  self.t").unwrap() + 2)));
    paleta(&mut m, ":99");
    assert_eq!(
        m.docs().cursor_of(id),
        Some((10, BOB.len())),
        "além do fim: a última linha"
    );
    // `>` só comandos: um símbolo não casa, um comando da aba sim.
    let q = m.hero.store.command_palette_top();
    assert_eq!(q, None, "fechada");
    global_palette::open(&mut m.hero);
    for c in ">update".chars() {
        m.hero.store.command_palette_push_char(c);
    }
    assert_eq!(m.hero.store.command_palette_top(), None, "`>update` não é um símbolo");
    m.hero.store.close_command_palette();
    paleta(&mut m, "@nada_disto");
    assert_eq!(m.docs().cursor_of(id), Some((10, BOB.len())), "nada casa: o cursor fica");
}

#[test]
fn strict_poe_e_tira_a_linha_num_passo_e_o_rotulo_diz_o_que_faz() {
    let (mut m, id) = aberta(BOB);
    let rotulo = |m: &mut Mesa| {
        m.docs()
            .commands(id.0)
            .into_iter()
            .find(|c| c.id == CMD_STRICT)
            .unwrap()
            .label
    };
    assert_eq!(rotulo(&mut m), "Turn On Strict Types (--!strict)");
    assert!(m.docs().command(id.0, CMD_STRICT));
    assert_eq!(m.docs().text_of(id), Some(format!("--!strict\n{BOB}")));
    assert_eq!(rotulo(&mut m), "Turn Off Strict Types (--!strict)");
    m.tecla(id, Key::Char('z'), ctrl(), Some("z"));
    assert_eq!(m.docs().text_of(id).as_deref(), Some(BOB), "um passo de undo");
    // Uma diretiva depois do comentário do topo conta, e o comando tira-a.
    let com = format!("-- bob\n--!strict\n{}", &BOB[7..]);
    let (mut m, id) = aberta(&com);
    assert!(m.docs().command(id.0, CMD_STRICT));
    assert_eq!(m.docs().text_of(id).as_deref(), Some(BOB));
}
