//! **Arch-gate: a âncora do motion path é DESENHADA e ARRASTADA pela `render_loop`.**
//!
//! Os gates de unidade provam a geometria (onde a âncora está na tela, o que o
//! hit-test devolve) e o documento (a porta que move a curva e reescreve as keys). Não
//! provam que o shell **desenha** a trajetória nem que costura press/move/release no
//! fluxo do ponteiro — e mutar qualquer uma dessas costuras deixa a workspace INTEIRA
//! verde, porque a `render_loop` e o `input_dispatch` precisam de janela e GPU.
//!
//! Mesmo recurso que a alça do texto em caminho (`the_textpath_handle_is_drawn_and_dragged`)
//! e o anel do pincel (`the_brush_ring_wears_the_live_dab_rotor`): quando o único
//! consumidor é o shell, a prova é sobre o FONTE.
//!
//! ⚠️ **Cada asserção nomeia o que faltou.** Um gate que deixa de encontrar por uma
//! mudança de forma passa a guardar nada, e um gate que não vê nada passa sempre.

use std::fs;

fn shell(path: &str) -> String {
    fs::read_to_string(format!("{}/src/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("ler {path}: {e}"))
}

/// O DESENHO está costurado no laço de render, e recebe a SELEÇÃO — sem ela a
/// trajetória apareceria para todo objeto animado do documento, que é espaguete.
#[test]
fn the_render_loop_draws_the_trajectory_for_the_selected_object() {
    let rl = shell("render_loop/mod.rs");
    assert!(
        rl.contains("motion_path_overlay::draw"),
        "a `render_loop` não chama `motion_path_overlay::draw` — a trajetória existe no \
         documento e não na tela"
    );
    let at = rl
        .find("motion_path_overlay::draw")
        .expect("a chamada de desenho");
    let call = &rl[at..(at + 400).min(rl.len())];
    assert!(
        call.contains("iter_selected"),
        "o desenho não recebe a seleção — a trajetória seria desenhada para todo objeto \
         com um binding Position, e um projeto com dez viraria espaguete"
    );
    assert!(
        call.contains("timeline.doc"),
        "o desenho não recebe o documento da timeline, que é onde a trajetória mora"
    );
}

/// O GESTO está costurado no ponteiro: press (arma + abre o undo), move (arrasta),
/// release (limpa + FECHA o undo).
#[test]
fn the_dispatch_wires_press_move_and_release() {
    let disp = shell("input_dispatch.rs");
    for (needle, what) in [
        (
            "self.motion_path_anchor_down(evt.x, evt.y)",
            "o PRESS (armar) não está no dispatch do ponteiro",
        ),
        (
            "self.motion_path_anchor_move(",
            "o MOVE (arrastar) não está no dispatch do ponteiro",
        ),
        (
            "self.motion_path_drag = None;",
            "o RELEASE não limpa o arrasto — a âncora ficaria colada ao cursor após soltar",
        ),
        (
            "self.timeline.history.begin(&self.timeline.doc)",
            "o press não ABRE um passo de undo",
        ),
        (
            "self.timeline.history.commit_if_changed(&self.timeline.doc)",
            "o release não FECHA o passo de undo que o press abriu — o próximo gesto o \
             herdaria e um Ctrl+Z desfaria os dois de uma vez",
        ),
    ] {
        assert!(disp.contains(needle), "{what}");
    }
}

/// ⚠️ **A ORDEM é load-bearing.** A âncora do primeiro key cai EM CIMA do sprite, onde o
/// gizmo mora — se o press dela vier depois do picking, o gizmo come o clique e o objeto
/// se MOVE onde o dedo pediu a CURVA. É a mesma razão pela qual as alças do vetor
/// precedem o picking.
///
/// Afirma a RELAÇÃO POSICIONAL, nunca uma distância em bytes: uma janela fixa é um proxy
/// que expira, e foi o que deixou dois arch-gates da `line/Vector` vermelhos ao entrar
/// uma feature no meio ([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).
#[test]
fn the_anchor_press_runs_before_the_generic_picking() {
    let disp = shell("input_dispatch.rs");
    let press = disp
        .find("self.motion_path_anchor_down(evt.x, evt.y)")
        .expect("o press da âncora");
    // A entrega ao caminho de sempre (picking de sprite + gizmo). Ancorada numa CHAMADA
    // e não num comentário: um comentário se reescreve e o gate passaria a guardar nada.
    let picking = disp
        .rfind("let _ = forward_to_hero(self.gfx.as_mut(), evt);")
        .expect("a entrega ao picking/gizmo");
    // ⚠️ **Controle do próprio ÂNCORA.** Há mais de uma chamada a `forward_to_hero` no
    // arquivo, e ancorar na errada torna esta comparação sempre verdadeira — foi
    // exatamente o que aconteceu na primeira versão deste gate, que sobreviveu à
    // mutação de mover o press para depois do gizmo. Esta é a do bloco do ADR-0112, e
    // a asserção abaixo é o que garante que continua sendo.
    let adr0112 = disp
        .find("ADR-0112: no modo **Select** a ferramenta não captura o canvas")
        .expect("o bloco do caminho de sempre");
    assert!(
        adr0112 < picking && picking - adr0112 < 2_000,
        "o `forward_to_hero` ancorado ({picking}) não é o do bloco de picking ({adr0112}) \
         — este gate está comparando o press com outra coisa e passaria sempre"
    );
    assert!(
        press < picking,
        "o press da âncora ({press}) vem DEPOIS do picking/gizmo ({picking}): apertar \
         numa âncora selecionaria/moveria o objeto em vez de puxar a curva"
    );
}

/// O arrasto escreve pela **porta única** do documento, não numa cópia da aritmética.
/// Mover a geometria sem reescrever as distâncias que as keys guardam deixa o sistema
/// estável e ERRADO: a curva nova na tela, o objeto andando os números da velha.
#[test]
fn the_drag_writes_through_the_documents_single_door() {
    let disp = shell("input_dispatch.rs");
    assert!(
        disp.contains("move_path_anchor(target, i, a)"),
        "o arrasto de ÂNCORA não escreve por `TimelineDoc::move_path_anchor` — se ele \
         mexe no `MotionPath` direto, as keys ficam com as distâncias da curva antiga"
    );
    // A alça de tangente é o outro gesto, e vai pela SUA porta única (ADR-0141, Fatia
    // 3c): moldar a curva sem reescrever as distâncias deixaria o objeto andando os
    // números da forma anterior.
    assert!(
        disp.contains("move_path_tangent(target, i, out"),
        "o arrasto de ALÇA não escreve por `TimelineDoc::move_path_tangent`"
    );
    assert!(
        !disp.contains("set_anchor("),
        "o dispatch chama `MotionPath::set_anchor` direto, contornando as portas que \
         reescrevem as keys"
    );
}
