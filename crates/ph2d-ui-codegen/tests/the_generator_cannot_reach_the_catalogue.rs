//! **ARCH-GATE: o gerador não alcança o catálogo de widgets** (plano UI/UX W8b).
//!
//! ⚠️ A contenção é **estrutural, não disciplinar**, e é a wave inteira numa linha: sem alcance ao
//! `ph2d-editor-core`, esta crate **não CONSEGUE** ter opinião sobre o que um `Slider` é. O
//! identificador do tipo chega pronto no `RowSpec`, resolvido por quem é dono do enum
//! (`WidgetKind::ident`).
//!
//! Sem isto, o caminho fácil existe: alguém acrescenta a dep e escreve um `match code { 4 =>
//! "Slider", … }` aqui dentro — uma **segunda resposta** a *"quais são os tipos vestíveis?"*, que
//! drifta do enum no dia em que um tipo entra, **em silêncio** (um número desconhecido não falha,
//! ele só não casa).
//!
//! É o mesmo gate que o `ph2d-paint-gpu` usa contra o `ph2d-painter-brush`.

use std::fs;

fn manifest() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("o Cargo.toml da propria crate e legivel")
}

/// **CONTROLE POSITIVO.** O manifesto é o que este gate pensa que é — sem isto, um rename de
/// arquivo o deixaria verde afirmando coisas sobre um arquivo vazio.
#[test]
fn the_manifest_this_gate_reads_is_the_right_one() {
    let m = manifest();
    assert!(
        m.contains("name = \"ph2d-ui-codegen\""),
        "este gate esta a ler outro manifesto"
    );
    assert!(
        m.contains("[dependencies]"),
        "o manifesto perdeu a secao que este gate examina"
    );
}

/// **Nenhuma dependência alcança o catálogo — nem direta, nem por um vizinho que o carregue.**
#[test]
fn the_codegen_crate_depends_on_nothing() {
    let m = manifest();
    let deps: Vec<&str> = m
        .lines()
        .skip_while(|l| l.trim() != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();
    assert!(
        deps.is_empty(),
        "o gerador ganhou uma dep — e a primeira delas e' o caminho para uma segunda tabela \
         `codigo -> nome`: {deps:?}"
    );
}

/// **E o código não nomeia o catálogo** — nem em `use`, nem em texto.
///
/// ⚠️ A segunda metade não é redundante: uma tabela `match code { 4 => "Slider" }` **não precisa
/// da dep** para existir. Ela compilaria sozinha aqui dentro, e seria exatamente a divergência que
/// este gate existe para impedir.
#[test]
fn the_generator_never_spells_a_widget_type_itself() {
    let raw = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("o lib.rs e legivel");
    // ⚠️ **Só o CÓDIGO**, e a linha abaixo é o gate a aprender com o próprio erro: a primeira
    // versão varria o arquivo inteiro e disparou sobre a **prosa** — o doc do `RowSpec` cita
    // `"Slider"` como exemplo do que o campo carrega. Um comentário pode nomear um tipo; código
    // não pode. *Um gate que falha sobre a documentação de si mesmo está a medir a coisa errada.*
    let src: String = raw
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for kind in [
        "Slider",
        "Toggle",
        "Checkbox",
        "ProgressBar",
        "SectionHeader",
        "ListItem",
        "Divider",
    ] {
        assert!(
            !src.contains(&format!("\"{kind}\"")),
            "o gerador escreveu o nome de um tipo do catalogo ({kind}) — a segunda tabela nasceu"
        );
    }
}
