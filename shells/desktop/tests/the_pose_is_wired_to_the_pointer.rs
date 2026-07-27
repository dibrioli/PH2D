//! **A POSE está ligada ao ponteiro** — arch-gate sobre a costura que nenhum
//! unit test alcança (W-IK).
//!
//! A decisão (`body_pose::take_pose`) e a escrita (`write_world_pose`) têm gates
//! headless ao lado delas. O que só existe no `input_dispatch` — e precisa de
//! `App` + janela — são quatro fatos, cada um com modo de falha próprio e
//! silencioso, e são os MESMOS quatro da mão porque os dois gestos partilham a
//! máquina: press, supressão do gizmo, release antes de todo `return`, e o Move.
//!
//! ⚠️ Nada aqui afirma distância em bytes nem vizinhança de linhas — a lição de
//! `the_dispatch_is_handed_the_live_geometry`. Afirma-se *quem é chamado*, *com
//! que argumentos*, e relações de ORDEM que nenhum formato move.

use std::fs;

fn dispatch_src() -> String {
    fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs")
}

/// Igual ao irmão da mão, e a remoção de comentários é load-bearing pela mesma
/// razão: uma asserção sobre ordem de CÓDIGO que lê prosa é um gate que qualquer
/// frase pode disparar, nos dois sentidos.
fn on_mouse_input_body(src: &str) -> String {
    let i = src
        .find("pub(crate) fn on_mouse_input(")
        .expect("on_mouse_input existe");
    src[i..]
        .lines()
        .map(|l| match l.find("//") {
            Some(c) => &l[..c],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// **O press pergunta à porta, com o RELÓGIO e a porta única da ferramenta.**
///
/// ⚠️ E **não** com `self.timeline.flags.simulate_physics`, ao contrário da mão:
/// posar não dá passo nenhum, então exigir o toggle obrigaria o artista a armar a
/// simulação para *não* simular. A ausência é afirmada, não deixada implícita —
/// alguém "consertando" a assimetria por simetria a quebra.
#[test]
fn the_press_asks_the_pose_door_with_the_clock_and_the_tool() {
    let src = dispatch_src();
    let i = src
        .find("crate::body_pose::take_pose(")
        .expect("o Down tem de chamar a porta da pose");
    let call = &src[i..i + src[i..].find(");").expect("chamada sem fechamento")];
    assert!(
        call.contains("self.interaction.tool.runs_at_rest()"),
        "a condição da ferramenta tem de vir da porta única `runs_at_rest`, \
         nunca de uma comparação com `InteractionTool::Pose` escrita aqui. \
         Chamada:\n{call}"
    );
    assert!(
        call.contains("self.playhead.is_playing()"),
        "o relógio decide. Chamada:\n{call}"
    );
    assert!(
        !call.contains("simulate_physics"),
        "posar NÃO exige o toggle Physics — ver o cabeçalho do `body_pose`. \
         Chamada:\n{call}"
    );
    assert_eq!(
        src.matches("crate::body_pose::take_pose(").count(),
        1,
        "UMA porta, UM sítio: uma 2ª chamada é a 2ª cópia da regra"
    );
}

/// **Posou ⇒ o arrasto de gizmo NÃO abre.**
///
/// Os dois juntos escreveriam o MESMO `Transform` no mesmo frame, e o de trás
/// venceria em silêncio: o artista veria a cadeia dobrar e o elo pego pular de
/// volta. A supressão é a MESMA variável da mão (`grabbed`), e é isso que este
/// gate afirma — as duas metades do `||` estão dentro dela.
#[test]
fn taking_a_pose_suppresses_the_gizmo_drag() {
    let src = dispatch_src();
    let pose = src
        .find("crate::body_pose::take_pose(")
        .expect("a porta é chamada");
    // O `let grabbed = ...` que contém a chamada, achado para trás — não uma
    // contagem de bytes.
    let start = src[..pose]
        .rfind("let grabbed =")
        .expect("a pose mora dentro do `let grabbed`");
    assert!(
        start < pose,
        "a chamada da pose tem de estar dentro do binding que suprime o drag"
    );
    let after = &src[pose..];
    let end = after
        .find("opened_drag = true;")
        .expect("o pick de canvas abre o drag depois da porta");
    assert!(
        after[..end].contains("if !grabbed"),
        "o bloco que abre o drag tem de estar gateado em `!grabbed`"
    );
}

/// **O release precede todo `return`.** Relação de ORDEM, imune a formatação —
/// uma cadeia que sobrevive ao release fica colada no cursor para sempre.
#[test]
fn the_pose_release_runs_before_any_early_return() {
    let body = on_mouse_input_body(&dispatch_src());
    let release = body
        .find("self.release_body_pose();")
        .expect("o release da pose tem de estar no on_mouse_input");
    let first_return = body.find("return").expect("o handler tem early-returns");
    assert!(
        release < first_return,
        "o release tem de vir antes do 1º `return` (offsets {release} < {first_return})"
    );
}

/// **A cadeia segue o cursor no laço de Move**, ao lado dos outros `advance_*`.
#[test]
fn the_move_advances_the_pose() {
    let src = dispatch_src();
    let i = src
        .find("pub(crate) fn on_cursor_moved(")
        .expect("on_cursor_moved existe");
    let body = &src[i..];
    let end = body
        .find("pub(crate) fn on_mouse_wheel(")
        .expect("o próximo fn delimita o corpo");
    assert!(
        body[..end].contains("self.advance_body_pose();"),
        "sem isto a pose não segue o cursor — ela pega e a cadeia fica onde estava"
    );
}
