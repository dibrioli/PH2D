//! ⭐ **O BLOCO GERADO TEM DE DESCREVER A ÁRVORE** — o gate de staleness do `ph2d-app-sync`.
//!
//! ⚠️ **Por que ele existe:** o bloco é *append-only por construção* — cinco linhas vão acrescentar
//! a família delas — e a única coisa que garante que ele não é editado à mão (ou esquecido) é este
//! gate. Sem ele, uma família nova que ninguém sincronizasse ficaria **fora do registo** com a
//! árvore inteira verde: a crate compila, os testes dela passam, e ela simplesmente não existe para
//! a shell.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .to_path_buf()
}

/// ⚠️ **O `rustfmt` INDENTA o marcador de fecho, e o bloco gerado não sabe disso.**
///
/// O `// <ph2d-app-sync:end>` está dentro de uma função, logo o `cargo fmt` põe-lhe quatro espaços
/// à frente — e a fatia «entre os marcadores» passa a acabar nesses quatro espaços, que o gerador
/// nunca escreveu. *Um gate que compara texto gerado com texto formatado compara duas coisas
/// diferentes*, e a leitura errada é «o bloco está velho, corra o sync» — que não cura nada,
/// porque o sync escreve exactamente o que já lá está.
///
/// ⇒ a comparação é feita sobre a indentação do fecho aparada. ⛔ **Não se apara o corpo inteiro**:
/// a indentação DENTRO do bloco é significativa, e apará-la deixaria passar um bloco gerado com o
/// recuo errado.
fn between<'a>(src: &'a str, begin: &str, end: &str) -> &'a str {
    let b = src
        .find(begin)
        .unwrap_or_else(|| panic!("marcador {begin} ausente"));
    let e = src
        .find(end)
        .unwrap_or_else(|| panic!("marcador {end} ausente"));
    assert!(e > b, "marcadores fora de ordem: {begin} depois de {end}");
    let corpo = b + begin.len();
    let inicio = src[corpo..]
        .find('\n')
        .map(|i| corpo + i + 1)
        .expect("quebra de linha depois do marcador");
    // o recuo que o `rustfmt` pôs no marcador de fecho não faz parte do bloco gerado
    let fatia = &src[inicio..e];
    fatia.rfind('\n').map_or(fatia, |i| {
        if fatia[i + 1..].trim().is_empty() {
            &fatia[..=i]
        } else {
            fatia
        }
    })
}

#[test]
fn the_generated_blocks_match_a_scan_of_the_tree() {
    let root = root();
    let found = ph2d_app_sync::scan_app_crates(&root.join("crates"));
    assert!(
        !found.is_empty(),
        "a varredura não achou família nenhuma em crates/ph2d-app-* — um gate que varre zero \
         ficheiros afirma-se verde sobre nada"
    );

    let lib = std::fs::read_to_string(root.join("crates/ph2d-app-registry-init/src/lib.rs"))
        .expect("lib.rs");
    assert_eq!(
        between(&lib, ph2d_app_sync::RS_BEGIN, ph2d_app_sync::RS_END),
        ph2d_app_sync::render_rs(&found),
        "o corpo de `register_all_app_families` não descreve a árvore — corra \
         `cargo run -p ph2d-app-sync`"
    );

    let toml = std::fs::read_to_string(root.join("crates/ph2d-app-registry-init/Cargo.toml"))
        .expect("Cargo.toml");
    for (b, e, esperado, nome) in [
        (
            ph2d_app_sync::TOML_DEPS_BEGIN,
            ph2d_app_sync::TOML_DEPS_END,
            ph2d_app_sync::render_deps(&found),
            "[dependencies]",
        ),
        (
            ph2d_app_sync::TOML_DEFAULT_BEGIN,
            ph2d_app_sync::TOML_DEFAULT_END,
            ph2d_app_sync::render_default(&found),
            "default",
        ),
        (
            ph2d_app_sync::TOML_FEATURES_BEGIN,
            ph2d_app_sync::TOML_FEATURES_END,
            ph2d_app_sync::render_features(&found),
            "[features]",
        ),
    ] {
        assert_eq!(
            between(&toml, b, e),
            esperado,
            "o bloco {nome} não descreve a árvore — corra `cargo run -p ph2d-app-sync`"
        );
    }
}

/// ⛔⛔ **TODA família varrida tem de estar no `default`.**
///
/// ⚠️ Esta é a recusa medida da auditoria §9 escrita como gate. Uma família fora do `default` não
/// dá erro nenhum: o `cargo check` da shell passa, o gate de fecho passa, o CI passa — e as cenas
/// de smoke dela, mais os gates que vivem dentro delas, **deixam de ser compilados em silêncio**.
/// *O que muda o tecto é a crate; a feature é para bissectar.*
#[test]
fn every_family_is_on_by_default() {
    let root = root();
    let found = ph2d_app_sync::scan_app_crates(&root.join("crates"));
    let toml = std::fs::read_to_string(root.join("crates/ph2d-app-registry-init/Cargo.toml"))
        .expect("Cargo.toml");
    let bloco = between(
        &toml,
        ph2d_app_sync::TOML_DEFAULT_BEGIN,
        ph2d_app_sync::TOML_DEFAULT_END,
    );
    for c in &found {
        let slug = ph2d_app_sync::feature_slug(c);
        assert!(
            bloco.contains(&format!("\"{slug}\"")),
            "a família `{c}` existe na árvore e NÃO está no `default` — as cenas de smoke dela e \
             os gates que vivem nelas deixam de compilar, sem uma linha vermelha"
        );
    }
}
