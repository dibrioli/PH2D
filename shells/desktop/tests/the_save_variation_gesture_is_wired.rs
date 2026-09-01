//! ⭐⭐⭐ **A FIAÇÃO do gesto «Salvar Variação»** — textual de propósito.
//!
//! # ⛔⛔⛔ Porque este gate é textual
//!
//! Medido na auditoria multiagêntica de 2026-08-31: **quatro mutações da fiação sobreviveram com
//! 6 407 testes verdes**. Um gate unitário chama a função directamente, então ele fica verde com a
//! função nunca CHAMADA de sítio nenhum — e é a chamada que faz o gesto existir para o artista.
//!
//! ⚠️ **Cada linha aqui é uma mutação que sobreviveria.** Um `seam_*` prova o caminho que o dedo
//! percorre; isto prova que os outros caminhos ainda existem.

use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao li {}: {e}", p.display()))
}

/// ⚠️ **Descasca comentários** — documentar a cura não pode fazer o portão passar. É a lei que o
/// gate irmão do vetor pagou: um `grep` cru vê a explicação e dá-a por implementação.
fn code(rel: &str) -> String {
    read(rel)
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O gesto CHEGA à lei** — o dreno chama a porta de escrita.
///
/// (Mutação: apagar o braço `InspectorSaveVariation` do dreno ⇒ RED. Nenhum gate unitário o vê.)
#[test]
fn the_drain_calls_the_save_door() {
    let s = code("src/render_loop/mod.rs");
    for needle in [
        "EditorAction::InspectorSaveVariation",
        "crate::variant_save::save_variation(",
        "crate::variant_save::rename_value(",
    ] {
        assert!(
            s.contains(needle),
            "`render_loop/mod.rs` nao tem `{needle}` — o gesto morre no barramento"
        );
    }
}

/// ⭐⭐⭐ **O cartão SABE que há o que gravar** — sem estes dois campos o botão nunca aparece, e o
/// selector de propriedade nasce vazio.
///
/// (Mutação: `pending: 0` fixo no construtor ⇒ RED aqui; o `with_nothing_to_save…` fica VERDE,
/// porque ele mede a ausência.)
#[test]
fn the_card_builder_fills_pending_and_declared() {
    let s = code("src/render_loop/inspector_properties.rs");
    for needle in ["pending", "declared", "ObjectInstance"] {
        assert!(
            s.contains(needle),
            "o construtor do cartao nao enche `{needle}`"
        );
    }
}

/// ⛔⛔ **A DECLARAÇÃO não volta ao nome** — o censo que impede a lei revogada de regressar por
/// uma porta esquecida.
///
/// Enio, 2026-09-01: *«Vamos tirar do nome o mecanismo de criação de variações»*. As oito funções
/// da gramática foram apagadas; isto garante que ninguém as reescreve.
///
/// ⚠️ **A régua olha a DEFINIÇÃO na lei, e a CHAMADA na shell** — e não a palavra solta. A 1.ª
/// versão procurava `display_name` cru e acusou `d.display_name`, que é o campo do descritor de
/// componente: *um gate que parseia o fonte tem de saber todas as formas do que parseia, senão
/// acusa o inocente e cega-se ao culpado.*
#[test]
fn nothing_parses_braces_out_of_a_name_any_more() {
    const DEAD: [&str; 8] = [
        "parse_combo",
        "with_value",
        "variant_name",
        "display_name",
        "declared_axes",
        "chip_label",
        "row_label",
        "hidden_count",
    ];
    // Lado A — a lei não as DEFINE.
    let law = code("../../crates/ph2d-editor-core/src/screens/hero/variant_axes.rs");
    for dead in DEAD {
        assert!(
            !law.contains(&format!("fn {dead}")),
            "`variant_axes` voltou a DEFINIR `{dead}` — a gramatica das chaves foi REVOGADA"
        );
    }
    // Lado B — e ninguém as CHAMA pelo módulo dela.
    for rel in [
        "src/render_loop/inspector_properties.rs",
        "src/render_loop/inspector_instance.rs",
        "src/render_loop/hierarchy_rename.rs",
        "src/render_loop/mod.rs",
        "src/instance_verbs.rs",
        "src/variant_save.rs",
    ] {
        let s = code(rel);
        for dead in DEAD {
            assert!(
                !s.contains(&format!("variant_axes::{dead}")),
                "`{rel}` voltou a chamar `variant_axes::{dead}`"
            );
        }
    }
}
