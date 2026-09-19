//! **CADA FERRAMENTA DE IMAGEM DECLARA O QUE FAZ AO VALOR DO PIXEL.**
//!
//! Auditoria completa:
//! [`docs/Sprite_projeto/19`](../../../docs/Sprite_projeto/19_auditoria_precisao_por_ferramenta.md).
//! Enio, 2026-08-20: *"auditoria completa com cada tool"*.
//!
//! # A pergunta única
//!
//! > O valor de um pixel de saída é CALCULADO a partir de mais de um pixel de entrada, ou de
//! > aritmética sobre a cor?
//!
//! - **Não** ⇒ a ferramenta move/copia/seleciona, e perder precisão nela é **gratuito**.
//! - **Sim** ⇒ preservar exige aritmética de 16 bits — código novo. Converter o resultado de volta
//!   para cima seria pior: o rótulo diria 16 sobre valores que passaram por 8.
//!
//! # As TRÊS classes, e por que duas não bastavam
//!
//! ⚠️ A primeira versão deste gate tinha duas listas — «preserva» e «não preserva» — e estava
//! **errada**. A auditoria leu os algoritmos e encontrou duas ferramentas **condicionais**:
//!
//! | classe | ferramentas |
//! |---|---|
//! | **sempre geométrica** | Trim Transparency · Make Square · Padding |
//! | **condicional** | Upscale (só `Nearest`) · BG-Removal (só sem despill) |
//! | **nunca** | Rasterize · Equalize Sizes · Color Equalization · Painter |
//!
//! *A resposta era do ALGORITMO e não do nome:* o `Upscale` tem três modos e só o `Nearest` é
//! replicação pura; o `BG-Removal` copia R, G e B verbatim e só calcula o alfa — excepto quando o
//! despill reescreve as bordas macias.
//!
//! ⚠️ **`Real Size` não aparece em lista nenhuma**, e isso é um facto e não um esquecimento: ela
//! repõe a escala do `Transform` a ±1 e **nunca commita textura**. Não pode perder o que não toca.

use std::fs;
use std::path::{Path, PathBuf};

/// A chamada que compromete um resultado geométrico preservando a precisão.
const GEOMETRIC_DOOR: &str = "commit_geometric_edit(";
/// A chamada do funil normal, que converte para 8 bits e avisa.
const EIGHT_BIT_DOOR: &str = "commit_edited_texture(";
/// Quem CONSTRÓI o buffer de 16 bits — a geometria pura que as cinco partilham.
///
/// ⚠️ **Isto é um ENDEREÇO DE FIAÇÃO, e ele já se mudou uma vez:** a lei vivia em
/// `crate::precision_geometry` e saiu da shell para a folha [`ph2d-sprite-precision`] em
/// 2026-09-19, quando a catraca `the_shell_only_shrinks` reprovou. *Um gate que cita um endereço
/// falha ALTO no dia da mudança, que é a espécie barata* — a cara é a que fica verde a medir nada.
const PRECISION_DOOR: &str = "ph2d_sprite_precision::";

fn handler(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/hero_intents/image_edit")
        .join(format!("{name}.rs"))
}

fn body(name: &str) -> String {
    let p = handler(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {p:?}: {e}"))
}

/// **As que preservam — sempre ou condicionalmente — passam pela porta que preserva.**
#[test]
fn the_preserving_tools_go_through_the_preserving_door() {
    for name in [
        "trim_transparency",
        "make_square",
        "padding",
        "upscale",
        "bgremoval",
    ] {
        let src = body(name);
        assert!(
            src.contains(GEOMETRIC_DOOR),
            "`{name}` nao passa por `{GEOMETRIC_DOOR}` — a auditoria \
             `docs/Sprite_projeto/19` diz que ela pode preservar, entao converter 16 bits para 8 \
             ali e' uma perda gratuita."
        );
        assert!(
            src.contains(PRECISION_DOOR),
            "`{name}` chama a porta mas nao constroi o buffer de 16 bits — o `None` fa-la cair no \
             caminho de 8 bits em silencio, e o gate acima passaria na mesma (a porta e' \
             `{PRECISION_DOOR}`)"
        );
    }
}

/// **As duas CONDICIONAIS declaram a condição, no próprio sítio.**
///
/// ⚠️ Sem isto, alguém que apagasse o `if` passaria a prometer preservação num caminho que
/// reescreve valores — o `Upscale · Lanczos3` a dizer `RGBA16` sobre pixels filtrados, o
/// `BG-Removal` com despill a dizer o mesmo sobre bordas recalculadas. *Uma preservação
/// condicional sem a condição à vista é uma mentira à espera de um refactor.*
#[test]
fn the_conditional_tools_name_the_condition_that_gates_them() {
    for (name, marker, why) in [
        (
            "upscale",
            "UpscaleAlgorithm::Nearest",
            "so' o modo Nearest replica; Lanczos3 e Xbr filtram",
        ),
        (
            "bgremoval",
            "despill",
            "o despill reescreve RGB nas bordas macias",
        ),
    ] {
        let src = body(name);
        assert!(
            src.contains(marker),
            "`{name}` preserva sem nomear a condicao `{marker}` — {why}"
        );
    }
}

/// **As que resamplam ou recolorem NÃO usam a porta que preserva.**
///
/// ⚠️ É a metade que impede a mentira. Se o Upscale passasse por ali, o Inspector diria `RGBA16`
/// sobre uma imagem cujos valores foram calculados em 8 bits — *o rótulo tem de prometer o que o
/// modelo entrega*, e esta é a terceira vez que esta wave paga essa lei.
#[test]
fn the_resampling_tools_do_not_claim_to_preserve_precision() {
    for name in ["rasterize", "color_equalization", "equalize_sizes"] {
        let p = handler(name);
        if !p.exists() {
            continue; // um handler que mudou de nome nao e' um handler que mente
        }
        let src = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {p:?}: {e}"));
        assert!(
            !src.contains(GEOMETRIC_DOOR),
            "`{name}` passa por `{GEOMETRIC_DOOR}`, mas ela RESAMPLA ou RECOLORE: os valores \
             atravessam 8 bits e o Inspector passaria a dizer RGBA16 sobre uma imagem que ja' \
             perdeu a precisao. Se ela deixou de resamplar, mova-a para o gate irmao."
        );
    }
}

/// **Controle positivo:** as duas portas existem mesmo, e são distintas.
///
/// ⚠️ Sem isto, renomear qualquer uma delas faria os dois gates acima passarem por não encontrarem
/// nada — verdes sobre um aparelho morto.
#[test]
fn both_doors_are_real_and_distinct() {
    let funnel = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/hero_intents/texture_edit.rs"),
    )
    .expect("texture_edit.rs");
    assert!(
        funnel.contains(&format!(
            "pub(crate) fn {}",
            GEOMETRIC_DOOR.trim_end_matches('(')
        )),
        "a porta geometrica `{GEOMETRIC_DOOR}` deixou de existir com esse nome"
    );
    assert!(
        funnel.contains(&format!(
            "pub(crate) fn {}",
            EIGHT_BIT_DOOR.trim_end_matches('(')
        )),
        "o funil de 8 bits `{EIGHT_BIT_DOOR}` deixou de existir com esse nome"
    );
    assert_ne!(GEOMETRIC_DOOR, EIGHT_BIT_DOOR);
}

/// **Controle positivo do TERCEIRO endereço** — e ele é mais forte que os outros dois.
///
/// ⚠️ O `PRECISION_DOOR` é uma string procurada em ficheiros: se a crate mudar de nome, o gate
/// `the_preserving_tools_go_through_the_preserving_door` passa a procurar algo que não existe em
/// lado nenhum e fica **verde a medir nada** — a lei que o irmão acima já escreve para as duas
/// portas. Aqui a referência é ao SÍMBOLO, logo uma renomeação deixa de **compilar**, e quem a
/// consertar tem a const à vista três linhas acima.
#[test]
fn the_precision_door_is_real() {
    let _: fn(&[u16], u32, u32, [u32; 4], u32, u32, u32, u32) -> Option<Vec<u16>> =
        ph2d_sprite_precision::blit_rgba16;
    let _: fn(&[u16], u32, u32, u32, u32) -> Option<Vec<u16>> =
        ph2d_sprite_precision::replicate_rgba16;
    let _: fn(&[u16], &[u8]) -> Option<Vec<u16>> = ph2d_sprite_precision::apply_alpha8_to_rgba16;
}
