//! **O interop DTCG pergunta ao ARTISTA onde o arquivo fica** — arch-gate sobre a costura que
//! nenhum teste de unidade alcança (plano UI/UX W9).
//!
//! # Por que sobre o fonte
//!
//! Um diálogo nativo **bloqueia** e precisa de uma janela: nenhum gate consegue clicar num. Então
//! a política do import (que modo recebe, o que se mantém dos outros, por que porta se escreve)
//! foi cortada para uma função pura-de-I/O — a `install`, que o `tokens_bridge_dtcg_tests` dirige
//! — e o que sobra nas duas funções públicas é **escolher o caminho e mover bytes**. Esta metade
//! não tem outro modo de ser afirmada.
//!
//! # O que ele fecha
//!
//! Um caminho FIXO. É a tentação barata (`PH2D_TOKENS_PATH`, um `ph2d.tokens.json` no CWD), e ela
//! shipa uma feature que só o autor sabe usar: o artista carrega o botão, nada aparece na tela, e
//! o arquivo nasce numa pasta que ele não escolheu. O projeto já tem esse precedente vivo e
//! nomeado — o Ctrl+S global grava num path de env — e ele está na lista de *aberto*, não na de
//! como se faz.

use std::fs;

const SRC: &str = "src/render_loop/tokens_bridge_dtcg.rs";

fn source() -> String {
    fs::read_to_string(SRC).unwrap_or_else(|e| panic!("{SRC}: {e}"))
}

/// O corpo de uma `fn` de topo, do nome até a linha `}` sem indentação.
fn body_of(src: &str, name: &str) -> String {
    let head = src
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("o {SRC} nao tem uma `fn {name}` — o gate mudou de alvo"));
    let rest = &src[head..];
    let end = rest.find("\n}\n").map_or(rest.len(), |i| i + 2);
    rest[..end].to_string()
}

/// **As duas metades perguntam ao artista onde o arquivo fica.**
#[test]
fn both_halves_open_a_native_file_dialog() {
    let src = source();
    for (name, verb) in [("export", "save_file"), ("import", "pick_file")] {
        let body = body_of(&src, name);
        assert!(
            body.contains("rfd::FileDialog"),
            "a `fn {name}` nao abre um dialogo — um caminho fixo faz o arquivo nascer numa pasta \
             que o artista nao escolheu"
        );
        assert!(
            body.contains(verb),
            "a `fn {name}` tem de usar `{verb}` — salvar e escolher sao dois dialogos diferentes"
        );
    }
}

/// **Desistir do diálogo não é um erro a anunciar.**
///
/// ⚠️ Um toast por cada vez que o artista carrega Escape num seletor de arquivo é ruído que ele
/// aprende a ignorar — e no dia em que houver um erro real ele já não olha para lá.
#[test]
fn cancelling_the_dialog_is_silent() {
    let src = source();
    for name in ["export", "import"] {
        let body = body_of(&src, name);
        let cancel = body
            .find("else {")
            .expect("o `let Some(path) = ... else` do dialogo");
        let arm = &body[cancel..body[cancel..].find('}').map_or(body.len(), |i| cancel + i)];
        assert!(
            !arm.contains("toast") && !arm.contains("Toast"),
            "a `fn {name}` anuncia a desistencia do dialogo: {arm:?}"
        );
    }
}

/// **A POLÍTICA não mora aqui** — ela é a `install`, e é o que o gate de comportamento dirige.
///
/// ⚠️ Sem esta metade, alguém "simplifica" o módulo enfiando a escrita da camada dentro do braço
/// do diálogo — e a wave inteira passa a ser provada por um gate que só consegue afirmar que o
/// codec funciona, com a costura de fora. É exactamente onde as waves desta linha falham.
#[test]
fn the_install_policy_is_a_function_of_its_own() {
    let src = source();
    assert!(
        src.contains("pub(crate) fn install("),
        "a politica de instalacao tem de ser uma `fn install` propria — ela e' a unica metade \
         que um teste consegue dirigir"
    );
    let body = body_of(&src, "import");
    assert!(
        body.contains("install("),
        "o `import` tem de delegar para a `install`"
    );
    assert!(
        !body.contains("set_color_overrides") && !body.contains("set_num_overrides"),
        "o `import` escreve a camada DIRECTAMENTE — a politica saiu da funcao que os gates \
         dirigem, e a proxima mudanca dela nao sera' medida por ninguem"
    );
}

/// ⚠️ **CONTROLE POSITIVO** — sem ele, um arquivo renomeado deixaria os três gates acima verdes
/// por não medirem nada.
#[test]
fn the_gate_is_reading_the_product() {
    let src = source();
    assert!(
        src.len() > 2000,
        "o {SRC} tem {} bytes — o gate esta' a medir um arquivo que nao e' o produto",
        src.len()
    );
    assert!(src.contains("ph2d_tokens_dtcg::export"));
    assert!(src.contains("ph2d_tokens_dtcg::import"));
}
