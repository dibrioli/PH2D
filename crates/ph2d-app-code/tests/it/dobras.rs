//! As dobras pelo caminho do produto: «Fold All» do menu, teclas, «Go to Line», a pintura. E o custo
//! por tecla das dobras, contado sem relógio (`docs/CodeEditor/02_onda1.md` §Medições (E14)).

use crate::{Mesa, ctrl};
use ph2d_app_code::commands::CMD_FOLD_ALL;
use ph2d_code_buffer::Text;
use ph2d_code_syntax::{Syntax, fold_captures_visited};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;

const BOB: &str = include_str!("../../../ph2d-code-syntax/tests/it/fixtures/bob.luau");

fn aberto(texto: &str) -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("bob.luau", texto)]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    (m, ids[0])
}

/// Ctrl+G, o número (1-based), Enter.
fn vai(m: &mut Mesa, id: ScriptId, linha_1: usize) {
    m.tecla(id, Key::Char('g'), ctrl(), None);
    m.escreve(id, &format!("{linha_1}\n"));
    m.pinta(id);
}

/// Toda dobra fechada é uma dobra que a gramática do texto de agora dá (mesmo início, mesmo fim).
fn fechadas_existem(m: &mut Mesa, id: ScriptId) {
    let t = Text::new(&m.docs().text_of(id).expect("aberto"));
    let vivas = Syntax::new(&t).folds(&t);
    for (a, b) in m.docs().closed_folds(id) {
        assert!(
            vivas.iter().any(|f| (f.start_line, f.end_line) == (a, b)),
            "a fechada {a}..{b} não é uma dobra do texto: {vivas:?}"
        );
    }
}

/// ⭐ Uma dobra fechada segue o texto: uma linha nova ACIMA dela muda o número da linha e ela
/// continua fechada; o cursor que entra nela (Go to Line) abre-a — e às de fora que a escondiam.
#[test]
fn uma_dobra_fechada_sobrevive_a_uma_edicao_acima_e_abre_quando_o_cursor_entra() {
    let (mut m, id) = aberto(BOB);
    assert!(m.docs().command(id.0, CMD_FOLD_ALL));
    m.pinta(id);
    // As de `dobras_de_bob` (ph2d-code-syntax): init, update, if, elseif, else, o if do topo.
    let todas = vec![(23, 28), (30, 52), (35, 41), (37, 38), (39, 40), (48, 50)];
    assert_eq!(m.docs().closed_folds(id), todas);

    // O cursor está na linha 0: uma linha nova acima de todas.
    assert_eq!(m.docs().cursor_of(id).map(|c| c.0), Some(0));
    m.escreve(id, "-- nota\n");
    m.pinta(id);
    let descidas: Vec<(usize, usize)> = todas.iter().map(|&(a, b)| (a + 1, b + 1)).collect();
    assert_eq!(
        m.docs().closed_folds(id),
        descidas,
        "as fechadas descem uma linha e continuam fechadas"
    );
    fechadas_existem(&mut m, id);

    // A linha 40 (1-based) = 39, dentro de update (31..53), do if (36..42) e do elseif (38..39).
    vai(&mut m, id, 40);
    assert_eq!(m.docs().cursor_of(id).map(|c| c.0), Some(39));
    assert_eq!(
        m.docs().closed_folds(id),
        vec![(24, 29), (40, 41), (49, 51)],
        "abrem-se as que escondiam o cursor; as outras ficam"
    );
}

/// Uma dobra fechada cuja estrutura deixou de existir abre-se, e escrevê-la de volta não a fecha.
/// Controlo: depois de cada passo, toda fechada é uma dobra viva (mesmo início, mesmo fim).
#[test]
fn a_fechada_cuja_estrutura_deixou_de_existir_abre_se() {
    let (mut m, id) = aberto(BOB);
    assert!(m.docs().command(id.0, CMD_FOLD_ALL));
    m.pinta(id);
    // O cabeçalho do init (linha 24, 1-based) não é escondido: o cursor fica nele.
    vai(&mut m, id, 24);
    assert!(m.docs().closed_folds(id).contains(&(23, 28)));
    // Apaga «function init(self)» inteiro: a dobra do init deixa de existir.
    m.tecla(
        id,
        Key::End,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        None,
    );
    m.tecla(id, Key::Backspace, Modifiers::default(), None);
    m.pinta(id);
    assert!(
        !m.docs().closed_folds(id).iter().any(|f| f.0 == 23),
        "a fechada sem estrutura abriu-se: {:?}",
        m.docs().closed_folds(id)
    );
    fechadas_existem(&mut m, id);
    // Escreve-a de volta: nada se fecha sozinho.
    m.escreve(id, "function init(self)");
    m.pinta(id);
    assert!(!m.docs().closed_folds(id).iter().any(|f| f.0 == 23));
    fechadas_existem(&mut m, id);
}

/// O trabalho das dobras numa tecla (tecla + quadro pintado): capturas da query de dobras.
fn trabalho_por_tecla(texto: &str, linha_1: usize) -> (u64, usize) {
    let (mut m, id) = aberto(texto);
    vai(&mut m, id, linha_1);
    m.tecla(id, Key::End, Modifiers::default(), None);
    m.pinta(id);
    let vistas = m.docs().visible_lines(id).map_or(0, |r| r.len());
    let antes = fold_captures_visited();
    const TECLAS: u64 = 5;
    for c in ['x', 'w', 'm', 'z', 'k'] {
        m.tecla(id, Key::Other, Modifiers::default(), Some(&c.to_string()));
        m.pinta(id);
    }
    ((fold_captures_visited() - antes) / TECLAS, vistas)
}

/// ⭐ **Gate de custo (sem relógio):** o trabalho das dobras por tecla, com a MESMA vista, não cresce
/// com o ficheiro. Medido antes da cura: a 7 500 linhas a query corria sobre a árvore inteira em cada
/// revisão (`6.46` ms de p50 na bancada E14) — aqui, `7 126` capturas por tecla contra `40` na cura.
#[test]
fn o_trabalho_das_dobras_por_tecla_nao_cresce_com_o_ficheiro() {
    const ALVO: usize = 31; // `  self.t = self.t + dt` (1-based) no bob; no grande, a cópia 72.
    let grande: String = BOB
        .lines()
        .cycle()
        .take(7_500)
        .map(|l| format!("{l}\n"))
        .collect();
    let por_copia = BOB.lines().count();
    let (pequeno, v_p) = trabalho_por_tecla(BOB, ALVO);
    let (grande_t, v_g) = trabalho_por_tecla(&grande, ALVO + por_copia * 72);
    eprintln!(
        "dobras por tecla: {pequeno} capturas ({por_copia} linhas) · {grande_t} (7 500) · vista {v_p}"
    );
    assert_eq!(v_p, v_g, "a mesma vista nos dois");
    assert!(pequeno > 0, "controlo: a régua conta as dobras da vista");
    assert!(
        grande_t <= pequeno * 2,
        "dobras por tecla: {pequeno} capturas a {por_copia} linhas contra {grande_t} a 7 500 (vista de {v_p} linhas)"
    );
}
