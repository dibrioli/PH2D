//! ⛔⛔⛔ **O ALFABETO É DADO, E A LISTA É EXACTAMENTE O QUE O INTERPRETADOR LÊ.**
//!
//! # Por que este ficheiro existe
//!
//! O alfabeto deste nó vivia **só num doc-comment**, e um doc-comment não é lido pelo artista
//! nem conferido por gate nenhum. ⚠️ **A tabela em prosa estava CERTA** — varridos os 94 bytes
//! imprimíveis, exactamente 15 símbolos agem e são exactamente os 15 que ela listava. *Ela
//! estava certa e não havia como saber, que é o mesmo problema por outro lado.*
//!
//! ⇒ hoje ela é [`ls::alphabet::ALPHABET`], e estes gates são o que a impede de divergir do
//! `walk` — nos **dois** sentidos.

use ph2d_node_source_lsystem as ls;

/// ⭐ **Tudo o que está na tabela é interpretado.** Um símbolo que perca o braço no `match` fica
/// vermelho aqui, e não numa foto meses depois.
#[test]
fn every_letter_in_the_table_is_interpreted() {
    let mut medidos = 0usize;
    for l in ls::alphabet::ALPHABET {
        // ⛔⛔ **O PAR `[ ]` tem régua PRÓPRIA, e a 1.ª redacção estava errada.** Ela media
        // `[]` — empilhar e desempilhar de imediato — que é um **no-op exacto**, e o gate
        // acusava a tabela de descrever uma feature inexistente. *O trabalho do par não é
        // empilhar: é RESTAURAR*, e isso só se vê com uma viragem lá dentro que **não**
        // sobrevive ao `]`. A régua é `F[+F]F` contra `F+FF`: os mesmos módulos, sem os
        // parênteses — e o `F` final sai a direito num e virado no outro.
        if l.symbols == "[ ]" {
            medidos += 2;
            assert!(
                ls::probe_two_grammars_differ("X -> F[+F]F", "X -> F+FF"),
                "o par `[ ]` não RESTAURA o estado — `F[+F]F` e `F+FF` desenham o mesmo"
            );
            continue;
        }
        for sym in l.symbols.split(' ') {
            medidos += 1;
            assert!(
                ls::probe_symbol_acts(sym),
                "`{sym}` está na tabela do alfabeto (*{}*) e o interpretador NÃO faz nada com \
                 ele — ou o braço do `match` caiu, ou a linha da tabela descreve uma feature \
                 que não existe",
                l.does
            );
        }
    }
    assert!(
        medidos >= 9,
        "só {medidos} símbolos medidos — a tabela encolheu ou o filtro partiu"
    );
}

/// ⭐⭐ **E NADA DE FORA DELA É INTERPRETADO** — a metade que impede a tabela de ficar para trás.
///
/// ⚠️ **É esta que apanha o caso caro:** um símbolo novo ganha braço no `walk` e ninguém
/// acrescenta a linha ⇒ o artista tem uma feature que o programa não sabe explicar. A varredura
/// é sobre TODOS os bytes imprimíveis, então nenhuma lista escrita à mão a limita.
#[test]
fn nothing_outside_the_table_is_interpreted() {
    // ⛔ Os da SINTAXE ficam de fora: eles não são módulos, são a gramática da própria regra
    // (`;` separa · `->` · `:` condição · `<`/`>` contexto · `(`/`)`/`,` argumentos).
    const SINTAXE: &[u8] = b";<>:(),-";
    // ⚠️ E o `X`/`Q`: `X` é o símbolo do AXIOMA desta fixtura (age por ser reescrito, não por ser
    // interpretado) e `Q` é o módulo mudo que serve de controlo.
    const FIXTURA: &[u8] = b"XQ";

    let na_tabela: String = ls::alphabet::ALPHABET
        .iter()
        .flat_map(|l| l.symbols.chars())
        .filter(|c| *c != ' ')
        .collect();

    let mut intrusos = Vec::new();
    for b in 0x21u8..0x7f {
        if SINTAXE.contains(&b) || FIXTURA.contains(&b) {
            continue;
        }
        let c = b as char;
        if na_tabela.contains(c) {
            continue;
        }
        if ls::probe_symbol_acts(&c.to_string()) {
            intrusos.push(c);
        }
    }
    assert!(
        intrusos.is_empty(),
        "estes símbolos são INTERPRETADOS e não estão no alfabeto que o programa sabe explicar: \
         {intrusos:?} — quem lhes deu um braço no `walk` não lhes deu uma linha"
    );
}

/// ⚠️ **A legenda numa linha NÃO CABE, e o número está aqui para a próxima janela não a tentar.**
///
/// A coluna do painel de params tem **~35 caracteres** (`304 px` de inspector menos `70` de
/// rótulo, a ~6 px por caractere). ⛔ *Construí-la assim daria uma legenda elidida a um terço —
/// exactamente o que uma auditoria acusa.*
#[test]
fn the_one_line_legend_does_not_fit_the_panel_column() {
    let l = ls::alphabet::legend_one_line();
    assert!(
        l.chars().count() > 60,
        "a legenda encolheu para {} caracteres — se ela couber em ~35, a decisão da superfície \
         MUDA e este gate é o sítio onde isso se descobre: {l}",
        l.chars().count()
    );
    // E ela tem de nomear cada grupo: uma legenda que perca um símbolo não é uma legenda.
    for g in ls::alphabet::ALPHABET {
        assert!(
            l.contains(g.symbols),
            "a legenda não nomeia `{}`",
            g.symbols
        );
    }
}
