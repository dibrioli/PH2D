//! **Arch-gate: quem pergunta pela trajetória entrega o TAB VIVO** (Enio, 2026-07-31:
//! *"Path editável apenas em Keys: Clips"*).
//!
//! O `active_path` recusa fora da aba Keys e há gate de unidade provando as cinco portas
//! (`motion_path_overlay::tab_tests`). Mas a decisão só existe se o SHELL passar o número
//! certo, e esses cinco sítios vivem em `render_frame` e nos handlers de ponteiro — código
//! que exige janela, GPU e superfície, que **nenhum teste de unidade alcança**.
//!
//! É exatamente a forma de falha que já custou uma wave neste repo: um `true` literal ali
//! deixa **todo** gate do overlay verde, com a alça fantasma de volta na tela do artista.
//! (O irmão `the_edit_frame_only_prices_when_the_delivery_section_is_open` recusa a mesma
//! coisa pelo mesmo motivo.)
//!
//! ⚠️ E a varredura é do `src/` INTEIRO, não de uma lista de arquivos: uma lista protege
//! o sítio que alguém lembrou de listar, e o sexto nasce descoberto — que é como os cinco
//! nasceram.

use std::path::{Path, PathBuf};

/// A ÚNICA resposta aceitável para *"a trajetória é editável aqui?"* — o espelho que o
/// shell carimba do painel a cada frame (`render_loop::mod`). Um literal, uma constante
/// local ou um segundo derivado seriam uma segunda porta para a mesma pergunta.
const LIVE_TAB: &str = "self.timeline.keys_mode";

/// As portas do overlay que perguntam ao `active_path` — as quatro de agarrar e a de
/// pintar. Todas tomam o tab como PRIMEIRO argumento, de propósito: um `bool` no meio de
/// seis parâmetros é o que se esquece.
const DOORS: &[&str] = &[
    "motion_path_overlay::draw(",
    "motion_path_hit(",
    "motion_path_curve_hit(",
    "anchor_screen(",
    "tangent_screen(",
];

/// O módulo do overlay e seus testes: ali dentro os chamadores são internos (já sob a
/// decisão) e as fixtures passam literais de propósito — é o que um teste de unidade É.
fn is_the_overlay_itself(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("motion_path_overlay"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") && !is_the_overlay_itself(&p)
        {
            out.push(p);
        }
    }
}

/// O primeiro argumento de uma chamada que começa em `open` (o índice do `(`), já aparado.
fn first_arg(src: &str, open: usize) -> &str {
    let rest = &src[open + 1..];
    let end = rest.find(',').unwrap_or(rest.len());
    rest[..end].trim()
}

#[test]
fn every_caller_hands_the_overlay_the_live_keys_tab() {
    let mut files = Vec::new();
    rust_files(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        &mut files,
    );
    assert!(
        files.len() > 20,
        "controle positivo: a varredura não achou o `src/` do shell ({} arquivos)",
        files.len()
    );

    let mut seen = 0_usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        for door in DOORS {
            let mut from = 0;
            while let Some(off) = src[from..].find(door) {
                let at = from + off;
                let open = at + door.len() - 1;
                let arg = first_arg(&src, open);
                assert_eq!(
                    arg,
                    LIVE_TAB,
                    "{} chama `{door}` com `{arg}` no 1º argumento. A trajetória é do clip \
                     ATIVO e só a aba Keys o sola — fora dela quem dirige o objeto é a \
                     PILHA, e a alça oferecida edita uma curva que a aba nem nomeia. \
                     Passe `{LIVE_TAB}`: um literal aqui deixa todo gate do overlay verde \
                     com o defeito de volta na tela.",
                    path.display()
                );
                seen += 1;
                from = at + door.len();
            }
        }
    }
    // Controle positivo: sem isto, apagar as cinco chamadas (ou renomear as portas) faria
    // este gate passar afirmando exatamente nada.
    assert!(
        seen >= 4,
        "controle positivo: esperava as chamadas do shell às portas do overlay, achei {seen}"
    );
}
