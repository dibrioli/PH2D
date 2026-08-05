//! **O passe de desenho publica o que as rows DIRIGEM, e depois dos tokens** (plano UI/UX W8b.3).
//!
//! # Por que um arch-gate
//!
//! A decisão mora dentro de `render_loop`, cujo laço exige `AppGfx` — janela e GPU. **Nenhum teste
//! de unidade a alcança**, e sem ela os nove gates do resolvedor ficam verdes sobre um produto em
//! que o slider não move um pixel: o `resolve` devolve a resposta certa e ninguém a consome.
//!
//! # A ORDEM é lei, e não arrumação
//!
//! A opacidade tem de entrar **depois** dos tokens porque ela desvanece *o que de fato vai ser
//! desenhado*. Invertida, o token cobriria a cor já desvanecida com o alfa dele e o slider ficaria
//! inerte **em toda forma bindada** — inerte só ALI, que é a pior forma de um controle falhar: ele
//! funciona na cena de teste e não funciona na arte do artista.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// Controle positivo: uma varredura vazia tornaria todo gate abaixo verde por vácuo.
fn body() -> &'static str {
    assert!(
        SRC.len() > 10_000,
        "o fonte do render_loop nao foi lido — os gates abaixo seriam verdes por vacuo"
    );
    SRC
}

/// **O passe de desenho RESOLVE e APLICA os drives.**
#[test]
fn the_draw_pass_resolves_and_applies_the_drives() {
    let s = body();
    assert!(
        s.contains("vec_widget_drive::resolve("),
        "o passe de desenho nao resolve os drives — o painel responde e a arte nao muda"
    );
    assert!(
        s.contains("vec_widget_drive::apply("),
        "os drives sao resolvidos e nunca aplicados — trabalho que ninguem consome"
    );
}

/// **E na ORDEM certa: os tokens primeiro, a opacidade depois.**
#[test]
fn the_opacity_lands_after_the_tokens() {
    let s = body();
    let tokens = s
        .find("vec_view.bound = crate::vec_bindings::resolve(")
        .expect("a publicacao dos tokens sumiu do render_loop");
    let drives = s
        .find("crate::vec_widget_drive::apply(")
        .expect("a aplicacao dos drives sumiu do render_loop");
    assert!(
        tokens < drives,
        "a opacidade foi aplicada ANTES dos tokens — o token cobriria a cor ja' desvanecida e o \
         slider ficaria inerte em toda forma bindada"
    );
}

/// **O clique do conta-gotas PRENDE** — o outro lado do gesto, no dispatch.
///
/// ⚠️ Sem esta metade o botão arma um modal que resolve e não escreve nada: o realce acende, o
/// clique consome o press, e o vínculo nunca nasce.
#[test]
fn the_pick_click_binds() {
    const DISPATCH: &str = include_str!("../src/input_dispatch.rs");
    assert!(DISPATCH.len() > 10_000, "controle positivo");
    let arm = DISPATCH
        .find("PathPick::WidgetBind(widget)")
        .expect("o braco do pick de vinculo sumiu do dispatch");
    let window = &DISPATCH[arm..(arm + 400).min(DISPATCH.len())];
    assert!(
        window.contains("vec_widget_edit::bind("),
        "o clique do conta-gotas nao prende nada"
    );
}
