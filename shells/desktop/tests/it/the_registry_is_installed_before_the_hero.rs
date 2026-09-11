//! **O registry de tools é instalado ANTES do primeiro `HeroScreen::new`.**
//!
//! ## O defeito que este gate existe para impedir
//!
//! Enio, 2026-08-19: *"os botões de padding, Color equalization, Rasterize, Upscale e Painter não
//! estão funcionando"*. Cinco de onze pills da fila de Image Tools, mortos sob o rato.
//!
//! O `topbar::populate()` — que corre **dentro** do `HeroScreen::new` — dá `InteractiveState` a
//! cada pill percorrendo a fila **derivada do registry**. Sem `InteractiveState` o widget não é
//! focável: o Down nunca o arma, o Up nunca emite `Click`. E sem registry instalado, aquela
//! derivação cai no fallback de três (`trim` · `make_square` · `bgremoval`) — que é exatamente o
//! conjunto que continuou a funcionar.
//!
//! ⚠️ **A regressão foi minha, e é instrutiva:** eu tinha acabado de curar este mesmo defeito para
//! UM tool (o pill `[SHEET]`, que a lista escrita à mão não conhecia) trocando a lista por uma
//! derivada. A cura estava certa; o que eu não vi é que a derivação corria **50 linhas antes** de a
//! fonte existir. *Uma lista derivada de algo que ainda não existe é uma lista vazia com cara de
//! correta* — e troquei um tool morto por oito.
//!
//! ## Por que este gate é ESTRUTURAL, e por que o de comportamento não bastou
//!
//! Existe um gate de comportamento (`every_image_tool_pill_is_registered_and_therefore_focusable`,
//! em `ph2d-tool-registry-init`) que clica em cada pill e exige que ele responda. **Ele passou
//! durante toda a regressão** — porque instalava o registry e *depois* construía o `HeroScreen`,
//! que é a ordem cómoda de escrever um teste e o **inverso** da que o shell corria.
//!
//! *Um teste que escolhe a sua própria ordem de arranque não prova o arranque.* A ordem só existe
//! num sítio — o `init.rs` —, e é lá que ela tem de ser afirmada.

use std::path::Path;

/// O corpo do `init.rs`, sem comentários — a ordem é um facto sobre CÓDIGO, e um comentário que
/// cite `HeroScreen::new` (há dois neste ficheiro) não pode contar como a chamada.
fn init_code() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/init.rs");
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_tool_registry_is_installed_before_the_hero() {
    let code = init_code();
    let install = code.find("install_registry(").expect(
        "o `init.rs` deixou de instalar o registry de tools — sem ele a fila de Image Tools \
         inteira cai no fallback de tres pills",
    );
    let hero = code
        .find("HeroScreen::new(")
        .expect("o `init.rs` deixou de construir o HeroScreen");
    assert!(
        install < hero,
        "o registry de tools e' instalado DEPOIS do primeiro `HeroScreen::new` (bytes {install} \
         vs {hero}).\n\
         O `topbar::populate()` corre dentro do `HeroScreen::new` e deriva a fila de Image Tools \
         do registry; sem ele instalado ela cai no fallback de tres, e os outros pills nascem sem \
         `InteractiveState` — pintados, hit-registered e MORTOS sob o rato.\n\
         Mova a construcao + `install_registry` para ANTES do hero, como o registry de PAINEIS ja' \
         esta'."
    );
}

/// Controle positivo: as duas âncoras existem mesmo, e não estão a passar por coincidência de
/// `find` sobre um ficheiro que mudou de forma.
///
/// ⚠️ Sem isto, apagar o `install_registry` do `init.rs` faria o teste acima entrar em pânico com
/// uma mensagem de `expect` em vez de reprovar com a sua — e a diferença entre as duas é a única
/// coisa que diz à próxima pessoa o que aconteceu.
#[test]
fn the_two_anchors_are_real_calls_not_comments() {
    let code = init_code();
    assert!(
        code.contains("ph2d_editor::install_registry(registry)"),
        "a chamada de instalacao mudou de forma; reveja o gate irmao antes de o silenciar"
    );
    assert!(
        code.contains("HeroScreen::new(NodeId(1))"),
        "a construcao do hero mudou de forma; reveja o gate irmao antes de o silenciar"
    );
}
