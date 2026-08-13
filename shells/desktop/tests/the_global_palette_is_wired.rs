//! **Arch-gate: a paleta de comandos global está FIADA — a tecla, a ordem do `match`, e o dreno.**
//!
//! ⚠️ **Um gate de unidade é cego a isto.** O modelo e a execução vivem em `ph2d-editor-core` e têm
//! sete gates lá; todos ficariam VERDES com uma shell que nunca abre a paleta e nunca drena o pick —
//! a feature existiria e não alcançaria nada que o artista toca, que é literalmente o estado em que
//! a UI viva ficou durante uma jornada inteira antes do `the_chrome_reads_the_ui_clock`.
//!
//! A shell é um **binário**: `shells/desktop/tests/` não a pode importar, então estes gates lêem o
//! FONTE. Cada um traz um **controle positivo** — um ficheiro renomeado deixaria a varredura vazia
//! e o gate passaria por vácuo, que é a falha silenciosa que o `keyboard.rs` partido já produziu.

const INPUT_HANDLERS: &str = include_str!("../src/input_handlers.rs");
const MAIN: &str = include_str!("../src/main.rs");
const GLUE: &str = include_str!("../src/global_palette_input.rs");

/// ⚠️ **CONTROLE POSITIVO.** Sem ele, um dono que se mude transforma os gates abaixo em varreduras
/// vazias — falha alta em vez de verde por vácuo.
#[test]
fn the_scanned_files_are_the_real_ones() {
    assert!(
        INPUT_HANDLERS.contains("match code {"),
        "o `input_handlers.rs` deixou de ter o match de teclas"
    );
    assert!(
        MAIN.contains("self.run_render_frame();"),
        "o `main.rs` deixou de ter o corpo do frame"
    );
    assert!(
        GLUE.contains("pub(crate) fn open_global_palette"),
        "a cola da paleta global mudou de dono"
    );
}

/// **`Ctrl+K` abre a paleta, e o arm está ACIMA do `K` nu.**
///
/// A ordem é a lei: o arm do `K` não olha para os modificadores (ele arma o keyframe da timeline),
/// então um arm guardado colocado ABAIXO nunca seria alcançado — o `match` do Rust é ordenado, e o
/// `KeyCode::KeyK` sem guarda casa primeiro.
///
/// *Mutação que sangra:* mover o arm do `Ctrl+K` para depois do `KeyCode::KeyK =>`.
#[test]
fn ctrl_k_opens_the_palette_and_its_arm_precedes_the_bare_k() {
    let guarded = INPUT_HANDLERS
        .find("KeyCode::KeyK if cmd_chord")
        .expect("o arm do Ctrl+K não existe: a paleta global não tem tecla");
    let bare = INPUT_HANDLERS
        .find("KeyCode::KeyK => {")
        .expect("o arm do K nu não existe — este gate deixou de afirmar o que diz");
    assert!(
        guarded < bare,
        "o arm do `Ctrl+K` está DEPOIS do `K` nu ({guarded} > {bare}): o `match` é ordenado, \
         então ele nunca é alcançado e a paleta não abre"
    );
    let body = &INPUT_HANDLERS[guarded..bare];
    assert!(
        body.contains("open_global_palette"),
        "o arm do `Ctrl+K` não abre a paleta global"
    );
}

/// **O pick é DRENADO no frame** — sem isto a escolha do artista fica no store para sempre.
///
/// *Mutação que sangra:* apagar a chamada ⇒ escolher um comando não faz nada, e nenhum gate de
/// `editor-core` repara.
#[test]
fn the_frame_drains_the_global_palette_pick() {
    let at = MAIN
        .find("self.global_palette_drain();")
        .expect("o frame não drena o pick da paleta global: a escolha nunca é executada");
    let render = MAIN
        .find("self.run_render_frame();")
        .expect("o `main.rs` deixou de ter o render do frame");
    assert!(
        at < render,
        "o dreno corre DEPOIS do render: o comando escolhido só apareceria no quadro seguinte"
    );
}

/// ⭐ **Os DOIS consumidores do canal do pick tomam CONDICIONALMENTE.**
///
/// Há uma paleta de nós (Motion) e uma de comandos (chrome) sobre **um** canal. Com um `take`
/// incondicional, qual dos dois recebe o pick passa a ser a ordem em que os drenos correm no frame —
/// um facto invisível que muda quando alguém reordena o laço, e cujo sintoma é *«às vezes não faz
/// nada»*. O gate afirma a propriedade sobre os dois ficheiros de dreno.
///
/// *Mutação que sangra:* voltar qualquer um dos dois a `take_command_pick()`.
#[test]
fn both_pick_consumers_take_conditionally() {
    const MOTION: &str = include_str!("../src/render_loop/motion_bridge_library.rs");
    for (name, src) in [
        ("global_palette_input.rs", GLUE),
        ("motion_bridge_library.rs", MOTION),
    ] {
        assert!(
            src.contains("take_command_pick_if"),
            "o dreno em `{name}` toma o pick INCONDICIONALMENTE: a ordem dos drenos passa a \
             decidir quem recebe o comando"
        );
        assert!(
            !src.contains("take_command_pick()"),
            "o `{name}` ainda tem um `take_command_pick()` incondicional ao lado do condicional"
        );
    }
}
