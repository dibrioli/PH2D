//! **A ponte da timeline declara o que escreveu** — o alcance que um teste não atravessa.
//!
//! ⚠️ Os gates da LEI vivem em `shells/desktop/src/timeline_preview_tests.rs` e correm o `apply`
//! REAL do `ph2d-timeline`. O que eles não alcançam é o `timeline_bridge::run`: ele pede nove
//! pedaços de estado do shell (o relógio activo, a fila de intents, o autokey, o emissor de
//! sinais) e não tem chamador nenhum fora do laço de quadro. Este arch-gate cobre essa costura.
//!
//! ⛔ Sem ele, tirar as duas linhas da ponte deixaria a lei **verde e o produto errado** — que é
//! exactamente o estado em que a §11 e o solver estiveram até 2026-08-23.

mod sculpt_source;
use sculpt_source::{function_body, source};

/// **O censo corre ANTES do apply, e a declaração DEPOIS** — a ordem é a asserção inteira.
///
/// ⚠️ Invertida, o «antes» seria o «depois» e nada ficaria declarado: a comparação daria sempre
/// igual, o ledger ficaria vazio, e o defeito voltava inteiro **com a suíte verde**.
#[test]
fn the_bridge_measures_before_the_apply_and_declares_after() {
    let body = function_body(&source("render_loop/timeline_bridge.rs"), "run");
    let census = body
        .find("state_of_bindings")
        .expect("a ponte precisa de fotografar as poses antes do apply");
    let apply = body
        .find("apply_scene(")
        .expect("o ramo Arrange (o comum) continua a aplicar a cena");
    let declare = body
        .find("declare_timeline_writes")
        .expect("…e de declarar o que o apply escreveu");
    assert!(
        census < apply,
        "o censo esta' DEPOIS do apply: ele fotografaria a pose que a curva acabou de escrever, e \
         a comparacao daria sempre igual"
    );
    assert!(
        apply < declare,
        "a declaracao esta' ANTES do apply: nada teria mudado ainda"
    );
}

/// **UMA declaração para os TRÊS ramos.** O `container`, o `solo` e o Arrange escrevem os mesmos
/// componentes; uma cópia por braço é como a do ramo menos usado fica para trás — e o Arrange, que
/// é o comum, seria o único a funcionar.
#[test]
fn one_declaration_serves_the_three_apply_branches() {
    let body = function_body(&source("render_loop/timeline_bridge.rs"), "run");
    assert_eq!(
        body.matches("declare_timeline_writes").count(),
        1,
        "a declaracao esta' duplicada por braco — a que ficar para tras e' a que ninguem repara"
    );
    for branch in ["apply_container(", "apply_active_clip(", "apply_scene("] {
        assert!(
            body.contains(branch),
            "o ramo `{branch}` desapareceu da ponte"
        );
    }
}
