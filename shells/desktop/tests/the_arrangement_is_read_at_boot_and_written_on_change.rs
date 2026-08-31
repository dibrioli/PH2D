//! ⭐⭐ **A ARRUMAÇÃO ESTÁ LIGADA NAS DUAS PONTAS** — e é este gate que impede o ficheiro de ficar
//! órfão.
//!
//! ⛔ O `layout_persist` pode estar inteiro, testado e correcto, e o artista continuar a perder a
//! arrumação — basta ninguém o **chamar**. Um módulo de persistência sem as duas chamadas é uma
//! feature que passa a suíte e não existe no produto.
//!
//! ⚠️ **É de FONTE porque a alternativa toca no disco:** as duas metades vivem no arranque
//! (`init.rs`) e no hook de ponteiro (`forwarding.rs`), e exercitá-las de verdade escreveria em
//! `~/.ph2d/`. O que elas fazem já é medido em `src/layout_persist_tests.rs`; o que falta aqui é
//! *alguém chama*.

use std::fs;

fn src(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} existe"))
}

/// ⭐ **A leitura, ANTES do primeiro quadro.**
///
/// ⚠️ Instalar depois faria o primeiro quadro desenhar a arrumação de omissão e saltar para a do
/// artista no seguinte — o mesmo defeito que as preferências de UI já tinham nomeado, e a razão de
/// as duas viverem no mesmo sítio do arranque.
#[test]
fn the_arrangement_is_installed_before_the_first_frame() {
    let init = src("src/init.rs");
    assert!(
        init.contains("layout_persist::install("),
        "ninguém instala a arrumação gravada no arranque — o artista arruma o app e perde tudo ao \
         fechar"
    );
    let (at_install, at_first_frame) = (
        init.find("layout_persist::install(")
            .expect("a chamada existe"),
        // O hero é devolvido ao fim do bloco de arranque; instalar depois disso é tarde.
        init.find("Some(hero)").unwrap_or(usize::MAX),
    );
    assert!(
        at_install < at_first_frame,
        "a arrumação é instalada DEPOIS de o hero sair do arranque"
    );
}

/// ⭐ **A escrita, no mesmo hook dos outros dois inquilinos.**
#[test]
fn the_arrangement_is_written_when_it_changes() {
    let fw = src("src/forwarding.rs");
    assert!(
        fw.contains("persist::layout_if_changed(hero)"),
        "ninguém grava a arrumação quando ela muda"
    );
    // ⚠️ Os três inquilinos partilham o hook **de propósito** (mesma forma: dono vivo → projecção →
    // mudança detectada → escrita best-effort). Um deles fora dele seria um quarto sítio a lembrar.
    for sibling in ["persist::palettes_if_changed", "persist::prefs_if_changed"] {
        assert!(
            fw.contains(sibling),
            "controlo positivo: `{sibling}` saiu do hook e este gate mediria outro sítio"
        );
    }
}

/// ⛔ **A escrita passa pela DECISÃO com nome**, nunca por uma condição escrita à mão no hook.
///
/// ⚠️ A 1.ª versão tinha a condição inline e ela dizia o **contrário** do comentário ao lado: com o
/// espelho a arrancar em `None`, `c.get() != Some(h)` é sempre verdade ⇒ o ficheiro era reescrito
/// no arranque de toda sessão. Uma decisão com nome é o que a torna gateável.
#[test]
fn the_write_decision_has_a_name() {
    let fwp = src("src/forwarding_persist.rs");
    assert!(
        fwp.contains("layout_persist::should_save("),
        "a decisão de gravar voltou para dentro do hook — ela deixa de ser gateável, e foi assim \
         que ela nasceu errada"
    );
}
