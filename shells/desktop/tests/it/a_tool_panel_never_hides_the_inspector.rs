//! ⭐⭐⭐ **NENHUM BRIDGE ESCONDE O INSPECTOR** — a metade de FONTE do report de 2026-09-08.
//!
//! > *«algumas ferramentas ou painéis não criam abas»* · *«inspector … apaga ao ser estreitado sem
//! > que o painel seja recolhido»* — Enio.
//!
//! Cinco bridges tinham um interruptor de flanco `panel_visibility.insert("inspector", !active)`:
//! o modelo de *takeover* anterior às abas. A metade de COMPORTAMENTO vive em
//! `ph2d-panel-registry-init/tests/it/a_slot_with_two_panels_shows_tabs.rs`
//! (`a_tool_panel_joins_the_inspector_as_a_tab_instead_of_replacing_it`); esta afirma que **ninguém
//! voltou a escrevê-lo**, que é a metade que um gate de comportamento não vê: um bridge NOVO com a
//! linha velha passaria naquele e escondia o Inspector na mesma.
//!
//! ⚠️ **Lê só CÓDIGO.** A explicação de por que a linha saiu está nos comentários dos próprios
//! bridges e contém a string proibida — um censo textual que não separa prosa de código mente nos
//! dois sentidos, e esta linha já pagou isso três vezes.

use std::fs;
use std::path::Path;

/// O ficheiro sem comentários de linha **e sem espaço nenhum** — ver o cabeçalho.
///
/// ⛔⛔ **A 1.ª redacção deste gate era CEGA à forma que o `rustfmt` escreve**, e deixou passar a
/// SEXTA superfície no mesmo dia em que nasceu: o `painter_bridge` tinha
///
/// ```text
/// hero.panel_visibility
///     .insert("inspector", !painter_is_active);
/// ```
///
/// — a mesma chamada, partida em duas linhas porque o receptor é longo. Um `contains` sobre o
/// texto cru procura uma sequência que a formatação decide, não uma que o código tem.
///
/// ⚠️ *É a QUARTA vez que esta linha paga a mesma lição — um censo textual tem de conhecer TODAS as
/// formas do que lê* (w25 o `cargo fmt`, w27 a prosa, w33 o doc-comment, e agora a quebra de
/// linha). ⇒ o espaço sai todo antes da comparação.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn no_bridge_hides_the_inspector_to_take_its_slot() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop");
    let mut scanned = 0usize;
    let mut guilty: Vec<String> = Vec::new();
    for e in fs::read_dir(&dir)
        .expect("a pasta dos bridges existe")
        .flatten()
    {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        scanned += 1;
        if code_only(&src).contains("panel_visibility.insert(\"inspector\"") {
            guilty.push(p.file_name().unwrap_or_default().to_string_lossy().into());
        }
    }
    // ⚠️ **O controlo do INSTRUMENTO:** uma pasta renomeada devolveria zero ficheiros e este gate
    //    leria como aprovado sobre nada.
    assert!(
        scanned >= 10,
        "só {scanned} bridges varridos em {dir:?} — a varredura perdeu o alvo"
    );
    guilty.sort();
    assert!(
        guilty.is_empty(),
        "estes bridges escondem o Inspector para lhe tomar o encaixe: {guilty:?}\ncura: não \
         esconda ninguém — os dois são OCUPANTES do mesmo encaixe e a fileira de abas nasce \
         sozinha. Quem fica à frente é decidido pelo `slot_tabs::reconcile_z`."
    );
}
