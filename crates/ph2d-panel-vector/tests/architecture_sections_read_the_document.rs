//! ⛔⛔ **AS SECÇÕES *PATTERN* E *BRUSH* NÃO LEEM O STORE PELA PORTA DE TRÁS** (2026-08-30).
//!
//! A lei vive em `paint_sections::{live_track, live_number}` e tem gates próprios. Este mede a
//! outra metade: que os dois pintores **passam por ela** em vez de perguntarem ao `WidgetStore`
//! directamente.
//!
//! ⚠️ **Por que um gate de FONTE.** O que a lei decide é o *texto e a posição da alça* que o painel
//! desenha, e nenhum arnês deste repo lê texto pintado — o `MockPanelHost` devolve rectângulos. Um
//! gate comportamental mediria o store, que é precisamente o campo que a cura **deixou de tocar**
//! (foi assim que a 1.ª régua desta wave reprovou produto correcto). ⇒ o que resta é impedir o
//! bypass, e isso lê-se no fonte.
//!
//! ⚠️⚠️ **O gate DESCASCA comentários antes de medir.** Sem isso, documentar a cura reprova o
//! portão: o doc-comment que explica o defeito **cita** a linha `store.number_value(id)` que o
//! causava. *Um gate textual que não descasca comentários proíbe explicar o que ele defende.*

use std::path::Path;

/// Os dois pintores que a lei governa.
const FILES: [&str; 2] = ["src/paint_texture_pattern.rs", "src/paint_brush.rs"];

/// As leituras directas que a lei substitui.
const BACK_DOORS: [&str; 2] = [".number_value(", ".slider("];

/// O fonte sem comentários de linha — `//`, `///` e `//!`.
///
/// ⚠️ Não trata comentários de bloco (`/* */`): os dois ficheiros não os têm, e um descascador
/// meio-feito que os fingisse tratar seria pior que um que não os promete. Se algum aparecer, este
/// gate acusa e quem o escreveu decide.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_pattern_and_brush_sections_never_read_the_store_directly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in FILES {
        let src = std::fs::read_to_string(root.join(rel))
            .unwrap_or_else(|e| panic!("{rel}: {e} — o gate perdeu o sujeito"));
        assert!(
            !src.contains("/*"),
            "{rel} ganhou um comentário de BLOCO, e este gate não o descasca — ele passaria a \
             medir texto de comentário como se fosse código"
        );
        let code = strip_line_comments(&src);
        for porta in BACK_DOORS {
            assert!(
                !code.contains(porta),
                "{rel} lê o store por `{porta}` em vez de `live_track`/`live_number`. Essa leitura \
                 devolve SEMPRE `Some` para um widget registado, então o valor do documento vira \
                 código morto e o painel mostra o resíduo da forma ANTERIOR."
            );
        }
    }
}

/// ⭐⭐ **O CONTROLO do descascador** — sem ele, um bug no `strip_line_comments` que devolvesse
/// vazio faria o gate acima passar sobre qualquer coisa.
///
/// ⚠️ É a mesma lição que o `assert` de contagem numa prova de mutação: *um filtro que casa zero
/// imprime aprovado*.
#[test]
fn the_comment_stripper_keeps_code_and_drops_comments() {
    let src = "// .number_value( num comentario\nlet x = 1;\n    /// .slider( num doc\nlet y = 2;";
    let code = strip_line_comments(src);
    assert!(
        code.contains("let x = 1;") && code.contains("let y = 2;"),
        "comeu o código"
    );
    for porta in BACK_DOORS {
        assert!(!code.contains(porta), "não descascou `{porta}`");
    }
}
