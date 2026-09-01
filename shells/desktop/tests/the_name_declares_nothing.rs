//! ⛔⛔⛔ **O CENSO da gramática REVOGADA** — o nome não declara nada.
//!
//! O mecanismo de propriedades de uma variação foi recusado **duas vezes** pelo dono: as chaves no
//! nome (31/08) e o dado + botão *Salvar Variação…* (01/09, *«não ficou bom e não funcionou»*), e
//! está **adiado para o fim do plano**. As oito funções da gramática foram apagadas.
//!
//! ⚠️ **Este ficheiro é o que impede a primeira de voltar por uma porta esquecida** — e ela é a
//! perigosa, porque parece barata: uma função que lê `{…}` de um `Name` custa dez linhas e põe
//! renomear no caminho de uma operação estrutural. *Uma recusa sem censo é uma nota que envelhece.*

use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao li {}: {e}", p.display()))
}

/// ⚠️ **Descasca comentários** — documentar a recusa não pode fazer o portão reprovar. É a lei que
/// o gate irmão do vetor pagou: um `grep` cru vê a explicação e dá-a por implementação.
fn code(rel: &str) -> String {
    read(rel)
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⛔⛔ **Ninguém DEFINE a gramática, e ninguém a CHAMA.**
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
