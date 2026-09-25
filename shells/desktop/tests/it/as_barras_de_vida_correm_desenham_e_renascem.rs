//! ⭐⭐⭐ **As barras de vida CORREM no quadro, são DESENHADAS e RENASCEM no rebobinar** (plano 28,
//! W4).
//!
//! # ⚠️ Porque é um gate de TEXTO
//!
//! A ponte da barra (`ph2d_app_components::health_bar_bridge`) tem os gates dela, e todos entram
//! **abaixo** da shell: a chamada mora no `motores_do_quadro`, o desenho no `present` e o rebobinar
//! no `fase_fabrica_e_morte` — três sítios que pedem a `App` ou o `AppGfx` inteiros e não são
//! alcançáveis de um teste. *Um motor com a lei certa e a shell a não o ligar lê-se como um motor sem
//! a lei* — a forma que esta casa já pagou quatro vezes. ⇒ a régua lê os três, com `include_str!`
//! para que mudar um ficheiro de sítio NÃO compile.

const MOTORES: &str = include_str!("../../src/render_loop/motores_do_quadro.rs");
const PRESENT: &str = include_str!("../../src/render_loop/present.rs");
const MORTE: &str = include_str!("../../src/render_loop/fase_fabrica_e_morte.rs");

/// ⭐⭐ **O `correm` chama as barras, e DEPOIS dos motores que falam** — elas só lêem, e o que
/// lêem (a vida) foi escrito pelo passo da física que corre antes desta janela.
///
/// **Mutação que deve sangrar:** apagar a chamada `barras(…)` do `correm`.
#[test]
fn o_quadro_corre_as_barras() {
    let corpo = &MOTORES[MOTORES
        .find("pub(super) fn correm(")
        .expect("o `correm` existe")..];
    let fim = corpo.find("\n}\n").expect("o `correm` fecha");
    let corpo = &corpo[..fim];
    assert!(
        corpo.contains("barras(sim, health_bars, relogio)"),
        "o `correm` já não chama as barras — a ponte tem a lei e a tela não a vê"
    );
    assert!(
        MOTORES.contains("health_bars.frame(sim, dt)"),
        "a função `barras` já não entrega o quadro à ponte"
    );
}

/// ⭐⭐ **As faixas entram no slot `extra` do passe de sprites** — e contam para o atalho
/// `so_fantasmas`, senão uma cena só com barras (nenhum outro produtor vivo) não as desenharia.
///
/// **Mutações que devem sangrar:** apagar o `push` das barras · tirá-las do `so_fantasmas`.
#[test]
fn o_present_desenha_as_faixas() {
    assert!(
        PRESENT.contains("&health_bars.instances"),
        "o present já não lê as instâncias das barras"
    );
    let atalho = PRESENT
        .find("let so_fantasmas")
        .expect("o atalho do `extra` existe");
    let fim_atalho = PRESENT[atalho..].find(';').expect("o atalho fecha") + atalho;
    assert!(
        PRESENT[atalho..fim_atalho].contains("barras.is_empty()"),
        "as barras não contam para o atalho — uma cena só com barras não as desenharia"
    );
    assert!(
        PRESENT.contains("for i in barras {"),
        "as faixas não entram no `extra`"
    );
}

/// ⭐⭐ **Rebobinar é RENASCER, e os rastos também** — o `renascer_a_corrida` é a porta com dois
/// chamadores (o recomeço e o rebobinar), e é nela que o rasto nasce outra vez colado à vida.
///
/// **Mutação que deve sangrar:** apagar o `health_bars.rewind()` do `renascer_a_corrida`.
#[test]
fn o_rebobinar_faz_os_rastos_renascerem() {
    let corpo = &MORTE[MORTE
        .find("fn renascer_a_corrida(")
        .expect("a porta do renascimento existe")..];
    let fim = corpo.find("\n}\n").expect("a porta fecha");
    assert!(
        corpo[..fim].contains("health_bars.rewind()"),
        "o renascimento já não repõe os rastos — um golpe que ninguém deu escorreria no rebobinar"
    );
}
