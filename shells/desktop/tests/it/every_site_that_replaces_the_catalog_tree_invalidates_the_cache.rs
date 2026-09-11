//! ⛔⛔ **O CENSO: quem substitui a árvore de catálogos invalida a cache dela.**
//!
//! A `LibraryCache` guarda os bytes da taxonomia com a **revisão** como chave. A revisão é
//! por-árvore e uma árvore restaurada nasce em `0` ⇒ colidir com a que a cache já viu é o caso
//! **NORMAL**, não o raro. Sem `invalidate`, o quadro seguinte devolve os bytes **antigos**.
//!
//! ⚠️ **E desde 2026-08-30 o SAVE lê através da cache** (`project_save` → `capture_project`), o que
//! muda a gravidade: antes ele chamava `collect(&g.catalogs)` directamente. Um terceiro sítio que
//! substitua a árvore sem invalidar grava **a taxonomia errada no ficheiro do artista, em
//! silêncio**.
//!
//! ⚠️ Este gate é **textual** e é a única forma de o afirmar: a obrigação é do CHAMADOR, e nenhum
//! tipo a pode impor sem embrulhar a árvore num wrapper que toda a gente teria de destrancar.
//! ⛔ Ele descasca comentários antes de varrer — documentar a cura não pode reprovar o portão.

use std::path::Path;

/// Tira os comentários de linha, para uma nota que fale de `gfx.catalogs =` não contar como
/// escrita. ⛔ A lição vem da caça de 2026-08-30: um gate textual que não descasca reprova quem
/// documenta.
fn strip_comments(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_site_that_replaces_the_catalog_tree_invalidates_the_cache() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();
    let mut sites = 0usize;

    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let Ok(body) = std::fs::read_to_string(&p) else {
                continue;
            };
            let body = strip_comments(&body);
            for (n, line) in body.lines().enumerate() {
                // Uma ATRIBUIÇÃO à árvore — `gfx.catalogs = …`. ⛔ Não `catalogs:` (um campo num
                // literal) nem `catalogs()` (o leitor da crate).
                if !line.contains(".catalogs =") {
                    continue;
                }
                sites += 1;
                // A invalidação tem de estar nas 3 linhas seguintes — é o corpo do mesmo gesto.
                let near = body
                    .lines()
                    .skip(n)
                    .take(4)
                    .any(|l| l.contains("library_cache.invalidate()"));
                if !near {
                    offenders.push(format!("{}:{}", p.display(), n + 1));
                }
            }
        }
    }

    assert!(
        sites >= 2,
        "o censo achou {sites} sítios a substituir a árvore — ele varre por texto, e zero \
         significa que o padrão mudou de forma e este gate deixou de medir seja o que for"
    );
    assert!(
        offenders.is_empty(),
        "sítios que substituem a árvore de catálogos SEM invalidar a cache: {offenders:?}\n\
         Acrescente `gfx.library_cache.invalidate();` a seguir — senão o quadro seguinte usa os \
         bytes da taxonomia ANTIGA, e o save grava-os no ficheiro do artista sem dizer nada."
    );
}
