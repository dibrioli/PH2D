//! ⭐⭐ **CENSO: «de que cor é o fundo do canvas?» tem UMA porta** (report de 2026-09-02).
//!
//! Três sítios respondiam-na sozinhos — o painter do canvas, o `clear` da camada de sprites (um
//! literal) e o cartão do navegador de assets (a cor dominante do asset). Cada um estava certo
//! **sozinho**, e é por isso que nenhum gate os apanhava: só a pergunta *"e quando a cor muda?"*
//! os separa.
//!
//! ⚠️ Este gate mede o TEXTO, de propósito. Os gates de valor (`canvas_clear_tests.rs` e o
//! `the_card_shows_the_canvas_behind_the_thumbnail.rs` do painel) provam que as duas pontas
//! concordam **hoje**; este prova que nenhuma delas volta a escrever a resposta à mão.

use std::fs;
use std::path::{Path, PathBuf};

fn shell_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Todo `.rs` de produção sob `dir` (os `*_tests.rs` ficam de fora: é lá que a cerca da M14.5
/// guarda o valor legado de propósito).
fn production_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                && !p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|n| n.ends_with("_tests.rs"))
            {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// ⛔⛔ **Só as linhas de CÓDIGO contam, e isso é a lei do gate, não uma folga.**
///
/// Duas razões, e a segunda é a que quase me escapou. (1) A nota histórica no `render_loop`
/// NOMEIA o valor legado de propósito — é ela que carrega o mecanismo da regressão da M14.5 —, e
/// apagar a história para calar uma varredura seria trocar a coisa certa pela coisa medível.
/// (2) ⚠️ **A varredura ingénua mente nos DOIS sentidos:** a mesma nota também MENCIONA a porta,
/// e por isso o gate irmão abaixo ficou VERDE com o literal reposto — a prova de mutação apanhou-o.
/// Um censo textual que não sabe distinguir prosa de código acusa a prosa e absolve o código.
fn code_of(path: &Path) -> String {
    let Ok(text) = fs::read_to_string(path) else {
        return String::new();
    };
    text.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⛔ O literal que era a cópia à mão do `Bg1` do Forge.
#[test]
fn no_shell_file_writes_the_canvas_backdrop_by_hand() {
    let mut guilty = Vec::new();
    for p in production_files(&shell_src()) {
        if code_of(&p).contains("0.047, 0.047, 0.055") {
            guilty.push(p.display().to_string());
        }
    }
    assert!(
        guilty.is_empty(),
        "o fundo do canvas voltou a ser escrito a mao em: {guilty:?} — \
         a porta e' `canvas_clear::canvas_clear_rgb`"
    );
}

#[test]
fn the_sprite_layer_clear_comes_from_the_door() {
    let p = shell_src().join("render_loop").join("mod.rs");
    assert!(
        code_of(&p).contains("canvas_clear::canvas_clear_rgb"),
        "o `clear` da camada de sprites tem de sair da porta"
    );
}

/// ⭐ A outra ponta: o cartão do navegador não pinta o fundo com a cor do asset directamente.
#[test]
fn the_asset_card_asks_the_law_instead_of_painting_the_swatch() {
    // ⚠️ **A lente é a CRATE, não um ficheiro** — e a diferença mordeu em 2026-09-06: o cartão
    //    mudou-se para `src/paint/card.rs` quando o `paint.rs` cruzou o tecto de LOC, e este gate
    //    (que lia só o pai) ficou vermelho sobre código correcto. *Um censo que aponta a um
    //    ficheiro mede o sítio, não a lei; quem corta um ficheiro em dois não devia ter de saber
    //    que gate de outra pessoa aponta para ele.*
    let src = repo_root()
        .join("crates")
        .join("ph2d-panel-asset-browser")
        .join("src");
    let code: String = production_files(&src)
        .iter()
        .map(|p| code_of(p))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.is_empty(),
        "o fonte do navegador de assets nao foi lido"
    );
    assert!(
        code.contains("card_backdrop::card_backdrop"),
        "o cartao tem de perguntar a lei do fundo"
    );
    assert!(
        !code.contains("from_rgba8(swatch[0]"),
        "o cartao voltou a pintar a cor do asset como fundo — a lei vive em `card_backdrop`"
    );
}
