//! **Arch-gate: o arrasto de KNOB é uma mão sobre a figura, e o SOLTAR a devolve.**
//!
//! ## O pedido (Enio, 2026-08-07)
//!
//! *"O mesmo mecanismo de apagar o preview deve ser aplicado quando se está mudando os parâmetros do
//! painel para shapes vivas (Size, Offset, etc.)."*
//!
//! Um arrasto de slider re-carimba a figura INTEIRA a cada quadro, pela mesma porta
//! (`restamp_shapes_preview`) e com o mesmo custo — só que sem Down/Up de canvas, então ele não passava
//! pelo `route_shape_draft`.
//!
//! ## Por que um gate de TEXTO, e por que ele afirma DOIS sítios
//!
//! O comportamento tem gate de unidade na `ph2d-tool-painter`
//! (`shape_draft_tests::a_panel_knob_drag_drafts_the_shape_and_the_release_settles_it`), mas quem
//! *responde* **há uma mão presa?** é a shell, e ela só existe com janela. As duas metades vivem em
//! lugares diferentes de propósito:
//!
//! - **o ARMAR** mora no drain de `ToolPanelEvent`, porque é o próprio edit que re-carimba: publicado
//!   um quadro depois, o primeiro quadro do arrasto pagaria o composite inteiro (na cena do report,
//!   ~300 ms) e o artista sentiria o engasgo ao *pegar* o slider;
//! - **o SOLTAR** mora no `painter_bridge::dispatch`, que roda todo quadro, porque quando o artista
//!   solta **não chega evento de painel nenhum**. Sem ele a figura ficaria fora da tela até o próximo
//!   edit — o pior modo de falha desta lei.
//!
//! ⚠️ Os dois chamam a MESMA porta com a MESMA expressão (`held_button.is_some()`); não são duas
//! respostas, é uma pergunta feita nos dois instantes em que ela decide algo.

const DRAIN: &str = include_str!("../src/render_loop/mod.rs");
const BRIDGE: &str = include_str!("../src/render_loop/painter_bridge.rs");

/// O ARMAR acontece **antes** do `handle_panel_event` — é o edit que re-carimba.
///
/// **Mutação que deve sangrar:** mover o `set_shape_draft_hold` para depois do `handle_panel_event`,
/// ou apagá-lo.
#[test]
fn the_panel_drain_publishes_the_gesture_before_the_edit_that_restamps() {
    let arm = DRAIN
        .find("p.set_shape_draft_hold(self.held_button.is_some());")
        .expect(
            "o drain de ToolPanelEvent nao publica mais o gesto — um arrasto de knob volta a \
             re-carimbar a figura inteira por quadro",
        );
    // O `handle_panel_event` que segue o armar, na MESMA vizinhança (o bloco do drain).
    let edit = DRAIN[arm..]
        .find("t.handle_panel_event(ev);")
        .expect("o edit sumiu do drain");
    assert!(
        edit < 600,
        "o `set_shape_draft_hold` nao esta imediatamente antes do `handle_panel_event` ({edit} bytes \
         de distancia) — publicar DEPOIS do edit deixa o 1o quadro do arrasto pagar o composite inteiro"
    );
}

/// O SOLTAR acontece no passe por-quadro — é o único lugar que vê a mão largar o botão.
///
/// **Mutação que deve sangrar:** apagar a chamada do bridge (o gate de unidade continua VERDE, porque
/// ele chama a porta à mão; só este vê a fiação).
#[test]
fn the_per_frame_bridge_is_what_sees_the_release() {
    assert!(
        BRIDGE.contains("painter.set_shape_draft_hold(pointer_held);"),
        "o bridge nao derruba mais a mao — ao soltar o knob a figura fica INVISIVEL ate o proximo edit"
    );
    assert!(
        BRIDGE.contains("pointer_held: bool"),
        "o bridge nao recebe mais o sinal de gesto em voo"
    );
    assert!(
        DRAIN.contains("self.held_button.is_some(),"),
        "o chamador do bridge nao passa mais o `held_button`"
    );
}

/// Controle positivo: os arquivos lidos são mesmo os do render loop do Painter, e não vazios que
/// fariam as buscas acima passarem por vácuo.
#[test]
fn the_scanned_files_are_the_painter_render_loop() {
    assert!(DRAIN.len() > 100_000 && BRIDGE.len() > 10_000);
    assert!(DRAIN.contains("painter_bridge::dispatch("));
    assert!(BRIDGE.contains("fn dispatch("));
}
