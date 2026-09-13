//! Os dois gates da porta dos modais que medem o **SHELL**, e por isso ficaram aqui.
//!
//! ⚠️ **O corte foi pelo SUJEITO de cada gate, não pelo ficheiro em que ele morava** (W2): os
//! cinco que medem a função pura foram com o código para a [`ph2d_app_host::modal`]; estes dois
//! **lêem ficheiros desta crate** e mediriam zero em qualquer outra.
//!
//! ⛔⛔ **Um gate que varre um directório por prefixo de nome fica VERDE quando a família se muda**
//! — ele passa a varrer zero ficheiros e a asserção `bad.is_empty()` é trivialmente verdadeira.
//! É por isso que o irmão `every_field3d_modal_goes_through_the_door` viaja com a família para
//! `ph2d-app-field3d` **e ganha lá um piso de população**, em vez de ficar aqui a olhar para um
//! `src/` que já não tem `field3d_*` nenhum.

/// ⭐ **O LOOP LÊ O NÚMERO DESCONTADO** — e sem este gate o [`chrome_dt`] podia estar perfeito e
/// não ser usado por ninguém.
///
/// ⚠️ É a lição da W34 uma wave depois: *provar o cálculo não prova a alcançabilidade dele*. Os
/// gates acima medem a função pura; este mede que os dois relógios do chrome — os toasts e a UI
/// viva — de facto a consomem, e que o medidor de fps e a simulação **continuam** com o `wall_dt`
/// inteiro (para eles o tempo passou mesmo).
#[test]
fn the_chrome_clock_reads_the_discounted_dt() {
    // ⚠️ **O relógio do chrome mudou-se para a FASE dele** (OBRA 2 da `line/render-loop`, 2026-09-12):
    // os toasts e a UI viva andam em `fase_chrome_clock.rs`, e a simulação continua no `mod.rs`. Ler os
    // DOIS é honesto aqui porque nenhuma agulha abaixo mede ORDEM — só presença.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop");
    let src = ["mod.rs", "fase_chrome_clock.rs"]
        .map(|f| std::fs::read_to_string(dir.join(f)).expect("o loop e a fase do relógio existem"))
        .join("\n");
    // ⚠️ Comentários fora, pela razão escrita no gate da porta.
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    for needed in ["toasts.tick(ui_dt", "hero.tick_motion(ui_dt)"] {
        assert!(
            code.contains(needed),
            "o relógio do chrome tem de andar com o `ui_dt` (o `wall_dt` menos o congelamento \
             declarado) — não achei `{needed}`"
        );
    }
    // E o outro lado: quem mede o QUADRO continua com o número inteiro.
    assert!(
        code.contains("self.fixed_step.advance(wall_dt)"),
        "a simulação lê o `wall_dt` INTEIRO — para ela o tempo passou mesmo, e descontar o \
         congelamento faria a cena saltar menos do que o relógio de parede diz"
    );
}
