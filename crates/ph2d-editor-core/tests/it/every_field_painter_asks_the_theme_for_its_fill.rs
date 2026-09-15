//! ⭐⭐ **A metade do PINTOR: quem enche um campo pergunta à porta, nunca ao token.**
//!
//! ⛔⛔ **Report do dono, 2026-09-14:** *«caixas de input numérico sem cor de fundo»*. A causa não
//! era a paleta — era **quem a escolhia**: o `text_input` perguntava ao tema
//! (`visuals::Chrome::field_fill`) e os dois pintores irmãos, `number_input` e `text_area`,
//! enchiam com `ColorToken::Bg1`, que é o token de um **cartão de secção**. Sobre um cartão isso é
//! `0/255`, e num tema moderno a moldura de repouso é zero ⇒ nenhuma caixa.
//!
//! ⚠️ ***Uma lei com uma porta e dois consumidores fora dela não é uma lei; é uma coincidência que
//! ainda não divergiu.*** O gate irmão (`a_field_is_never_the_colour_of_what_it_sits_on`, em
//! `ph2d-tokens`) defende a COR; este defende o **caminho** até ela — os dois são precisos, porque
//! uma paleta certa lida por um pintor que não a lê não pinta nada.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// O sítio ÚNICO onde a escada clássica do preenchimento pode ser lida — a porta.
const A_PORTA: &str = "crates/ph2d-editor-core/src/widget/text_input/mod.rs";

#[test]
fn every_field_painter_asks_the_theme_for_its_fill() {
    let root = repo_root();
    let mut ficheiros = Vec::new();
    walk(
        &root.join("crates/ph2d-editor-core/src/widget"),
        &mut ficheiros,
    );
    ficheiros.sort();
    // ⚠️ **Piso de população:** um caminho errado varre zero ficheiros e o gate fica verde a medir
    // nada (a armadilha §2.7 do HOWTO, que já mordeu quatro vezes neste repo).
    assert!(
        ficheiros.len() >= 60,
        "a varredura achou so' {} ficheiros de widget — o caminho mudou?",
        ficheiros.len()
    );

    let mut fora = Vec::new();
    let mut na_porta = 0;
    for p in &ficheiros {
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        for (n, line) in src.lines().enumerate() {
            let t = line.trim();
            // ⚠️ Prosa não conta: este ficheiro e os doc-comments dos pintores NOMEIAM o defeito,
            // e um censo que não separa prosa de código mente nos dois sentidos.
            if t.starts_with("//") || !t.contains("fill_token(") {
                continue;
            }
            if rel.ends_with(A_PORTA.trim_start_matches("crates/ph2d-editor-core/")) {
                na_porta += 1;
            } else {
                fora.push(format!("{rel}:{}: {t}", n + 1));
            }
        }
    }
    assert!(
        fora.is_empty(),
        "{} pintor(es) escolhem o fundo de um campo por TOKEN em vez de perguntar a\n\
         `text_input::field_fill(state, theme)`:\n  {}\n\nNum tema moderno nao ha moldura de \
         repouso — um token que calhe ser o do cartao por baixo deixa o campo INVISIVEL, que foi \
         o report do dono de 2026-09-14.",
        fora.len(),
        fora.join("\n  ")
    );
    // ⚠️ E a porta tem de continuar a existir: `0` aqui significa que a escada clássica se perdeu
    // (o clássico deixaria de ser byte-idêntico) — não que o app ficou limpo.
    assert!(
        na_porta >= 1,
        "a escada classica do preenchimento desapareceu da porta"
    );
}
