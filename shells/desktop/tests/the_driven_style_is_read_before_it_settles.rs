//! ⭐⭐⭐ **O QUINTO FACTO QUE UMA CURVA ESCREVE NÃO PRECISA DO LEDGER — ele tem cura própria.**
//!
//! ⚠️ **A auditoria de 2026-09-08 deixou-o aberto como *«o ledger cobre 4 das 5 escritas do
//! `write_prop`»*, e a medição desta wave DISSOLVE o item:** a quinta é a opacidade de um caminho
//! vectorial (`ph2d_ecs::VecDrivenStyle`), e ela é protegida por **duas** coisas que juntas são mais
//! fortes do que o ledger:
//!
//! 1. **Ela é DESREGISTADA, e não por esquecimento:** o componente não deriva `Serialize`, e o
//!    `ComponentRegistry::register_default` exige-o — *uma linha de registo não compila*. Logo ela
//!    nunca entra no snapshot, nunca vai ao save, e nunca pode virar um passo de undo. O ledger
//!    existe para impedir exactamente isso nos outros quatro, que **são** registados.
//! 2. **Ela volta ao autorado TODO QUADRO** (`vec_driven_style::settle_to_authored`), e não apenas
//!    quando um motor é desligado — que é o que o `release_to_authored` do ledger faz. Um controlo
//!    removido deixa resíduo por **um** quadro, no máximo.
//!
//! ⛔ Acrescentá-la ao ledger seria pôr no memo um facto que nunca esteve na fotografia.
//!
//! ⚠️⚠️ **O que PODE regredir em silêncio é a ORDEM** — e ela não tinha gate nenhum. O doc da função
//! escreve a lei (*«depois de a projecção deste quadro ser lida, nunca antes»*) e a razão: repor
//! ANTES apagaria a curva **deste** quadro, e a forma deixaria de desvanecer com a suíte inteira
//! verde.

const LOOP: &str = include_str!("../src/render_loop/mod.rs");

/// **O fonte sem comentários** — sem isto, uma nota que cita a chamada conta como chamada.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **LER, APLICAR, e SÓ ENTÃO repor o autorado.**
#[test]
fn the_driven_style_settles_after_the_frame_has_read_it() {
    let src = code_only(LOOP);
    let resolve = src
        .find("vec_driven_style::resolve(")
        .expect("a projecção da aparência conduzida vive no quadro");
    let apply = src
        .find("vec_driven_style::apply(")
        .expect("ela é aplicada à vista do quadro");
    let settle = src.find("vec_driven_style::settle_to_authored(").expect(
        "o componente NUNCA volta ao autorado — apagar uma track de opacidade deixa a forma \
             congelada no último valor da curva, para sempre e sem gesto de volta",
    );
    assert!(
        resolve < apply && apply < settle,
        "a ordem do quadro partiu-se (resolve {resolve}, apply {apply}, settle {settle}) — repor o \
         autorado ANTES de a projecção ser lida apaga a curva DESTE quadro, e a forma deixa de \
         desvanecer com a suite inteira verde"
    );
}
