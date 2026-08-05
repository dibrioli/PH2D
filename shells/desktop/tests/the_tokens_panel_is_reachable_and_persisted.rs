//! **Arch-gate: o painel de TOKENS existe no app que corre** (plano UI/UX W6, degrau 1).
//!
//! # Porque é um arch-gate
//!
//! Os gates de `ph2d-panel-tokens` provam o painel; os de `ph2d-tokens` provam a camada. **Todos
//! passariam com o painel fora do registro do binário** — foi exactamente o que aconteceu com o de
//! física no 1º smoke do W2b: a shell põe `default-features = false` no `ph2d-panel-registry-init`
//! e re-enumera os painéis na lista `default` DELA, então ligar a feature só na crate de registry
//! **não alcança ninguém**. A tecla alterna a visibilidade, o z-order pergunta o id, recebe `None`
//! e não pinta — sem erro, sem warning.
//!
//! A metade que estes gates cobrem vive em `Cargo.toml` e no laço de frame, que exige janela;
//! nenhum teste de unidade a alcança.

use std::fs;

fn manifest() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).expect("Cargo.toml")
}

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// **O arquivo de projeto inteiro, como FAMÍLIA** — todo `src/project*.rs` que não é suíte.
///
/// ⚠️ **Um arch-gate ancorado num NOME de arquivo é um proxy que expira**, e este expirou: a
/// metade do LOAD morava em `project.rs` quando o gate nasceu, e a `line/sculpt3d` a moveu para
/// o irmão `project_load.rs` (um split de LOC — o corpo do `project_load_from` não cabia mais).
/// O produto seguiu certo e só o endereço envelheceu; a linha da tabela de cor continua a ser
/// executada, uma linha acima da da timeline. É a MESMA cicatriz que a `line/Vector` já pagou
/// duas vezes em 23/07 (a janela de 400 bytes e a distância de 1200), com a mesma cura: **afirme
/// a propriedade, nunca o endereço.**
///
/// ⚠️ **A varredura exclui `*_tests.rs` de propósito:** uma fixture que citasse a chamada faria o
/// gate ficar VERDE sem o produto a fazer nada — verde por documentação de si mesmo, a armadilha
/// que o gate do `[frame]` do Flip pagou. Conferido ao escrever: hoje a agulha aparece só em
/// `project.rs` (o save) e `project_load.rs` (o load), em nenhuma suíte.
fn project_family() -> String {
    let dir = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    let mut all = String::new();
    let mut seen = 0usize;
    for e in fs::read_dir(&dir).expect("src/") {
        let p = e.expect("entry").path();
        let Some(n) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if n.starts_with("project") && n.ends_with(".rs") && !n.ends_with("_tests.rs") {
            all.push_str(&fs::read_to_string(&p).expect("le o arquivo"));
            seen += 1;
        }
    }
    // Controle positivo: uma varredura que não achou nada estaria VERDE por vácuo em todo
    // `assert!(contains)` que a consome.
    assert!(
        seen >= 2,
        "a familia project*.rs encolheu para {seen} arquivo(s) — a varredura esta' a olhar para o lugar errado"
    );
    all
}

/// **O painel está na lista `default` DA SHELL** — a metade que o W2b pagou com um smoke.
#[test]
fn the_shell_compiles_the_tokens_panel_into_its_registry() {
    let m = manifest();
    assert!(
        m.contains("panel-tokens         = [\"ph2d-panel-registry-init/panel-tokens\"]"),
        "falta a feature `panel-tokens` que liga o registry-init"
    );
    let default_start = m.find("\ndefault = [").expect("a lista default da shell");
    let default_end = default_start + m[default_start..].find("\n]").expect("fim da lista");
    assert!(
        m[default_start..default_end].contains("\"panel-tokens\""),
        "a feature existe e a lista `default` da shell NAO a liga — o painel ficaria fora do \
         registro do binario, com a tecla a alternar a visibilidade de um painel que nao existe"
    );
}

/// **A tecla `T` alterna a visibilidade do painel** — um painel de MUNDO não é tool-gated, então
/// sem abridor próprio é feature que ninguém alcança.
#[test]
fn the_t_key_toggles_the_tokens_panel() {
    let s = src("input_handlers.rs");
    let at = s
        .find("KeyCode::KeyT =>")
        .expect("o braço da tecla T mudou de forma — reancore este gate");
    let block = &s[at..at + 400];
    assert!(
        block.contains("is_panel_visible(\"tokens\")") && block.contains("insert(\"tokens\""),
        "a tecla `T` deixou de alternar o painel de tokens"
    );
    // ⚠️ E o scaffold de debug que ela era não pode voltar: um braço que empurra um toast E
    // alterna o painel faria a tecla ter dois donos.
    //
    // ⚠️ **A âncora é a CHAMADA, não a frase.** A 1ª versão procurava o literal
    // `"Toast key (T) pressed"` e nasceu VERMELHA — porque o doc-comment que esta mesma wave
    // escreveu, a explicar de onde a tecla veio, CITA a frase. *Um oráculo que casa com a
    // documentação de si mesmo não está a olhar para o produto* (a lição que o `[frame]` do Flip
    // pagou em §5.48); só o código tem a chamada.
    assert!(
        !s.contains("Toast::info(\"Toast key"),
        "o scaffold de debug da tecla T voltou — ela tem um dono agora"
    );
}

/// **A ponte corre no laço de frame**, e é ela que faz a cor escolhida no picker chegar à camada.
///
/// ⚠️ Sem esta chamada o painel pinta, o picker abre, a cor é escolhida — e nada muda. Os sete
/// gates de seam e os oito da camada ficam VERDES.
#[test]
fn the_frame_loop_runs_the_tokens_bridge() {
    let s = src("render_loop/mod.rs");
    assert!(
        s.contains("tokens_bridge::dispatch(hero)"),
        "o laço de frame não chama a ponte de tokens"
    );
}

/// **A tabela autorada viaja no arquivo, nos DOIS sentidos.**
///
/// ⚠️ Meia fiação é pior que nenhuma: só o save deixaria a tabela num arquivo que ninguém lê; só o
/// load deixaria o artista a re-vestir o app e a perder tudo ao fechar — um knob que ESQUECE.
#[test]
fn the_authored_table_travels_in_the_file_both_ways() {
    let s = project_family();
    assert!(
        s.contains("tokens: crate::project_tokens::collect()"),
        "o save não grava a tabela autorada"
    );
    assert!(
        s.contains("crate::project_tokens::install(&file.tokens)"),
        "o load não instala a tabela do arquivo"
    );
}
