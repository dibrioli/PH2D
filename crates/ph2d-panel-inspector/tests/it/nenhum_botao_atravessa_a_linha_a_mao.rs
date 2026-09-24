//! ⭐⭐⭐ **NENHUM BOTÃO DE ACÇÃO DO INSPECTOR ESCOLHE SOZINHO ATRAVESSAR A LINHA.**
//!
//! ⛔⛔ **Report do dono, 2026-09-24, foto do cartão da junta com uma seta:** *«esses botões que
//! atravessam de lado a lado talvez fiquem melhor na coluna do lado direito»*. Eram
//! `Rect::new(x, y, w, h)` — a linha inteira — em `29` sítios deste painel. Hoje a porta
//! [`ph2d_editor_core::property_row::caixa_do_botao`] decide: a coluna do valor quando o rótulo lá
//! cabe, a linha inteira quando não cabe (e é por isso que nenhum botão ficou cortado).
//!
//! ⚠️ **Porque é textual:** a decisão é de uma porta que devolve um RECT; um sítio que o escreva à
//! mão pinta igual e nenhum gate de pixel o separa do que a porta mandaria a uma largura em que o
//! rótulo não cabe. ⇒ o censo procura o idioma: um `Rect::new(x, …, w, …)` que acaba num
//! `paint_button`.
//!
//! ⏳ **O limite, nomeado:** os pares `+ Add … | x Remove …` repartem a linha pelo
//! `segment_rects_for` e NÃO passam por aqui — dois rótulos desses nunca cabem na coluna (já cortam
//! à largura inteira no degrau estreito), logo a porta mandá-los-ia atravessar na mesma.

use std::fs;
use std::path::PathBuf;

fn sections_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sections")
}

/// `let <var> = Rect::new(x, <y>, w, <h>);` seguido, dentro de ~30 linhas, de um `paint_button`
/// que usa `<var>`.
fn botoes_a_mao(src: &str) -> usize {
    let linhas: Vec<&str> = src.lines().collect();
    let mut n = 0;
    for (i, l) in linhas.iter().enumerate() {
        let t = l.trim();
        let Some(resto) = t.strip_prefix("let ") else {
            continue;
        };
        let Some((var, rhs)) = resto.split_once(" = ") else {
            continue;
        };
        let rhs = rhs.replace(' ', "");
        if !(rhs.starts_with("Rect::new(x,") && rhs.contains(",w,")) {
            continue;
        }
        let depois = linhas[i + 1..(i + 30).min(linhas.len())].join("\n");
        if depois.contains("paint_button(") && depois.contains(var.trim()) {
            n += 1;
        }
    }
    n
}

fn censo() -> (usize, usize, Vec<(String, usize)>) {
    let (mut ficheiros, mut pela_porta) = (0, 0);
    let mut acusados = Vec::new();
    for entry in fs::read_dir(sections_dir()).expect("sections/ existe") {
        let path = entry.expect("entrada legível").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let nome = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("nome utf-8")
            .to_string();
        if nome.ends_with("_tests.rs") {
            continue;
        }
        ficheiros += 1;
        let src = fs::read_to_string(&path).expect("ficheiro legível");
        pela_porta += src.matches("property_row::caixa_do_botao(").count();
        let n = botoes_a_mao(&src);
        if n > 0 {
            acusados.push((nome, n));
        }
    }
    acusados.sort();
    (ficheiros, pela_porta, acusados)
}

/// **Todo botão de acção de linha inteira passa pela porta.**
///
/// **Mutação que deve sangrar:** repor o `Rect::new(x, yy, w, h)` num dos `29` sítios — é a foto.
#[test]
fn nenhum_botao_atravessa_a_linha_a_mao() {
    let (ficheiros, pela_porta, acusados) = censo();
    // ⚠️ **Pisos MEDIDOS em 2026-09-24:** `70` ficheiros de produto e `29` botões pela porta. O
    //    segundo é o que prova que a agulha ainda acha o idioma: uma porta renomeada deixaria os
    //    acusados a zero e ler-se-ia como aprovação.
    assert!(
        ficheiros >= 60,
        "a varredura leu {ficheiros} ficheiro(s) em sections/ — deixou de alcançar o directório"
    );
    assert!(
        pela_porta >= 25,
        "só {pela_porta} botão(ões) passam pela `caixa_do_botao` — a porta mudou de nome ou os \
         botões voltaram a ser escritos à mão"
    );
    assert!(
        acusados.is_empty(),
        "estes botões atravessam a linha por um `Rect::new(x, …, w, …)` escrito à mão (report do \
         dono de 2026-09-24): {acusados:?}\n\
         cura: `ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, y, h, rótulo)` — \
         a coluna do valor quando o rótulo cabe, a linha inteira quando não cabe."
    );
}

/// O extractor apanha o idioma e não apanha o que só se parece com ele.
#[test]
fn o_extractor_separa_o_idioma() {
    let mau = "let rect = Rect::new(x, yy, w, h);\nlet b = Button::new(id, \"A\");\n\
               paint_button(&b, rect, scene, ts, theme);";
    assert_eq!(botoes_a_mao(mau), 1);
    // Um rect de linha inteira que NÃO é de botão (um fundo, um separador) não conta.
    let fundo = "let rect = Rect::new(x, yy, w, h);\nfill_rect(scene, rect, cor);";
    assert_eq!(botoes_a_mao(fundo), 0);
    // A porta não conta.
    let bom = "let rect = ph2d_editor_core::property_row::caixa_do_botao(ts, x, w, yy, h, &l);\n\
               paint_button(&b, rect, scene, ts, theme);";
    assert_eq!(botoes_a_mao(bom), 0);
}
