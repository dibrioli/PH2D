//! **UMA porta escreve uma expressão** (B1 do plano 12, D4 da auditoria).
//!
//! A auditoria de 2026-07-29 abriu este item com *"o documento tem dois escritores"*. A
//! medição de 2026-07-30 diz que ele já está **meio fechado**, e o outro meio é uma decisão
//! que não é minha. A tabela `(escritor, leitor)` que o plano pedia:
//!
//! | campo | ESCRITORES no produto | LEITORES |
//! |---|---|---|
//! | `clip.expr` (per-clip, ADR-0145) | **UM**: `TimelineIntent::SetBindingExpr` → `TimelineDoc::set_clip_expr` | `frame_solve` ×2 · `stack_eval` · `expr_pass::still_driven` · `snapshot` (primeiro) · `doc_extent` · `copy_clip` |
//! | `binding.expr` (global, ADR-0144) | **NENHUM** | `frame_solve` ×2 · `expr_pass` ×3 · `snapshot` (fallback) |
//!
//! Ou seja: **a autoria já é per-clip e tem uma porta só** (o ADR-0145 fechou isso). O que
//! sobrou é um canal **legado, só de leitura**: `binding.expr` não é escrito por caminho
//! nenhum do produto — ele só pode chegar de um save v15 ou da cena de smoke que o
//! exercita de propósito.
//!
//! ⚠️ **Matá-lo é decisão do Enio, não minha:** exige migrar `binding.expr` → `clip[0].expr`
//! no load, um bump de `DOC_VERSION` (que **recusa todo projeto já salvo** na versão
//! anterior) e um ADR. O que este gate faz é impedir que um SEGUNDO escritor apareça em
//! silêncio — o dia em que alguém escrever `b.expr = …` fora da exceção nomeada, a pergunta
//! do ADR volta à mesa em vez de o defeito D4 renascer.

use std::path::{Path, PathBuf};

/// A raiz do workspace: `CARGO_MANIFEST_DIR` = `shells/desktop`.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("shells/desktop/../..")
        .to_path_buf()
}

/// Todo `.rs` de produção (não-teste) sob `dir`, recursivamente.
fn sources(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            // Um arquivo de teste PODE escrever o campo (é o que fixtures fazem); o gate
            // é sobre o PRODUTO.
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with("_tests.rs") || name.contains("test") {
                continue;
            }
            if let Ok(s) = std::fs::read_to_string(&p) {
                out.push((p, s));
            }
        }
    }
}

/// **A única escrita de `binding.expr` no produto é a cena de smoke que a exercita.**
///
/// ⚠️ Controle positivo nas duas pontas: o scanner tem de ACHAR a exceção conhecida (senão
/// um rename de arquivo o deixa verde sobre nada) e tem de achar a porta per-clip.
///
/// **Mutação que deve sangrar:** qualquer `b.expr = Some(...)` novo num arquivo de produto.
#[test]
fn the_global_expression_field_has_no_product_writer() {
    let mut files = Vec::new();
    for c in [
        "crates/ph2d-timeline/src",
        "crates/ph2d-panel-timeline/src",
        "shells/desktop/src",
    ] {
        sources(&root().join(c), &mut files);
    }
    assert!(
        files.len() > 100,
        "CONTROLE: o scanner tem de estar lendo a árvore ({} arquivos)",
        files.len()
    );

    let (scenes, product) = split_by_scene(&files, |s| {
        s.contains(".expr = Some") || s.contains(".expr = None") || s.contains("b.expr =")
    });

    // CONTROLE POSITIVO: o scanner tem de ACHAR a exceção conhecida — a cena do ADR-0146
    // C4, que demonstra o canal GLOBAL de propósito (um prop-link que não obedece a strip).
    // Sem isto, um rename de arquivo deixa o gate verde sobre nada.
    assert!(
        scenes.iter().any(|w| w == "morph_fade_smoke.rs"),
        "CONTROLE: o scanner tem de achar a cena que escreve o canal global; achou {scenes:?}"
    );
    assert!(
        product.is_empty(),
        "`binding.expr` é um canal LEGADO só de leitura. Um escritor novo re-abre o defeito \
         D4 (dois escritores do mesmo fato) e a pergunta do ADR do B1 — migrar o campo e \
         bumpar `DOC_VERSION` — tem de voltar à mesa ANTES, não depois: {product:?}"
    );
}

/// Parte os arquivos que casam com `pred` em **cenas de smoke** e **produto**.
///
/// ⚠️ A distinção não é cosmética: uma cena MONTA um documento (é o papel dela), e o
/// invariante é sobre quem o edita em resposta ao artista. Mas as cenas ficam no scanner,
/// e visíveis, porque são elas que dão o controle positivo — um gate cujo scanner não acha
/// NADA passa por vacuidade.
fn split_by_scene(
    files: &[(PathBuf, String)],
    pred: impl Fn(&str) -> bool,
) -> (Vec<String>, Vec<String>) {
    let mut scenes = Vec::new();
    let mut product = Vec::new();
    for (p, _) in files.iter().filter(|(_, s)| pred(s)) {
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        if name.ends_with("_smoke.rs") {
            scenes.push(name);
        } else {
            product.push(name);
        }
    }
    scenes.sort();
    product.sort();
    (scenes, product)
}

/// **E a autoria per-clip tem UMA porta.**
///
/// A outra metade da tabela: o intent que o card levanta termina em `set_clip_expr`, e é o
/// único sítio do produto que a chama.
///
/// **Mutação que deve sangrar:** um segundo `set_clip_expr` fora do `intent_apply`.
#[test]
fn the_per_clip_expression_is_authored_through_exactly_one_door() {
    let mut files = Vec::new();
    for c in [
        "crates/ph2d-timeline/src",
        "crates/ph2d-panel-timeline/src",
        "shells/desktop/src",
    ] {
        sources(&root().join(c), &mut files);
    }

    let (scenes, product) = split_by_scene(&files, |s| s.contains("set_clip_expr("));

    assert!(
        !scenes.is_empty(),
        "CONTROLE: as cenas de smoke montam documentos com fórmula, e o scanner tem de vê-las"
    );
    // `doc.rs` DEFINE a função; `intent_apply.rs` é o único que a CHAMA no produto.
    assert_eq!(
        product,
        vec!["doc.rs".to_string(), "intent_apply.rs".to_string()],
        "a definição + UM chamador, e o chamador é o INTENT (a autoria passa pelo undo, não \
         por baixo dele). Uma segunda porta é exactamente o defeito D4: {product:?}"
    );
}
