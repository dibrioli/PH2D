//! ⭐⭐ **A ARRUMAÇÃO ESTÁ LIGADA NAS DUAS PONTAS** — e é este gate que impede o ficheiro de ficar
//! órfão.
//!
//! ⛔ O `layout_persist` pode estar inteiro, testado e correcto, e o artista continuar a perder a
//! arrumação — basta ninguém o **chamar**. Um módulo de persistência sem as duas chamadas é uma
//! feature que passa a suíte e não existe no produto.
//!
//! ⚠️ **É de FONTE porque a alternativa toca no disco:** as duas metades vivem no arranque
//! (`init.rs`) e no **quadro** (`render_loop`), e exercitá-las de verdade escreveria em `~/.ph2d/`.
//! O que elas fazem já é medido em `src/layout_persist_tests.rs`; o que falta aqui é *alguém
//! chama*, e **de onde**.

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

/// ⭐⭐⭐ **A escrita corre no QUADRO, e NÃO no hook de ponteiro.**
///
/// ⛔⛔ **Este gate mudou de forma porque a 1.ª versão estava no sítio errado, e o Enio apanhou-o:**
/// *«não funcionou. Voltou ao zero.»* A detecção vivia no `forward_to_hero`, com os outros dois
/// inquilinos da persistência — e os dois gestos que arrumam o app **não passam por lá**:
///
/// | gesto | por que escapava |
/// |---|---|
/// | arrastar a **borda** de uma coluna | o `dock_seam_move`/`_up` faz `return` no `input_dispatch` |
/// | largar uma **aba** noutro encaixe | é resolvido DENTRO do `paint`, depois do hook |
///
/// ⇒ *um detector no caminho de um gesto só vê os gestos que passam por ele; o quadro vê todos,
/// porque é onde o estado assenta.* O gate afirma as duas metades: **está no quadro** e **não está
/// no hook**.
#[test]
fn the_arrangement_is_detected_on_the_frame_and_not_on_the_pointer_hook() {
    let frame = src("src/render_loop/mod.rs");
    assert!(
        frame.contains("layout_persist::save_if_changed(hero)"),
        "ninguém grava a arrumação no quadro — a borda de uma coluna e a largada de uma aba não \
         passam pelo hook de ponteiro, e voltam ao zero ao reabrir o app"
    );
    assert!(
        frame.contains("paint_hero_screen("),
        "controlo positivo: o quadro mudou de ficheiro e este gate mediria outro sítio"
    );

    let fw = src("src/forwarding.rs");
    assert!(
        !fw.contains("layout_if_changed"),
        "a detecção voltou para o hook de ponteiro — os dois gestos que arrumam o app escapam-lhe"
    );
    // ⚠️ Os outros dois inquilinos FICAM no hook, e está certo: a escolha de paleta e a de carácter
    // são cliques que atravessam o hero. O controlo impede este gate de os arrastar consigo.
    for sibling in ["persist::palettes_if_changed", "persist::prefs_if_changed"] {
        assert!(
            fw.contains(sibling),
            "`{sibling}` saiu do hook de ponteiro — ele não tem o problema que mudou a arrumação"
        );
    }
}

/// ⛔ **A escrita passa pela DECISÃO com nome**, nunca por uma condição escrita à mão.
///
/// ⚠️ A 1.ª versão tinha a condição inline e ela dizia o **contrário** do comentário ao lado: com o
/// espelho a arrancar em `None`, `c.get() != Some(h)` é sempre verdade ⇒ o ficheiro era reescrito
/// no arranque de toda sessão. Uma decisão com nome é o que a torna gateável.
#[test]
fn the_write_decision_has_a_name() {
    let lp = src("src/layout_persist.rs");
    assert!(
        lp.contains("should_save(previous, h)"),
        "a decisão de gravar voltou para dentro do detector — ela deixa de ser gateável, e foi \
         assim que ela nasceu errada"
    );
}
