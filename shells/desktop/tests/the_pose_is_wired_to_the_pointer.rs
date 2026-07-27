//! **Os gestos de POSE estão ligados ao ponteiro** — arch-gate sobre a costura
//! que nenhum unit test alcança (W-IK, W-FK, W-JointTools).
//!
//! As decisões (`body_pose::take_pose`, `body_fk::take_fk`) e a escrita
//! (`write_world_pose`) têm gates headless ao lado delas. O que só existe no
//! `input_dispatch` — e precisa de `App` + janela — são cinco fatos, cada um com
//! modo de falha próprio e silencioso: quem responde *qual gesto é este*, o
//! press, a supressão do gizmo, o release antes de todo `return`, e o Move.
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

/// O fonte sem espaço em branco nenhum.
///
/// ⚠️ **Load-bearing:** uma asserção sobre uma cadeia de métodos escrita como
/// literal fica refém de onde o rustfmt decidiu quebrar a linha — o gate passa a
/// falhar (ou, pior, a passar) por causa de um `cargo fmt`. Normalizar é o que
/// torna a afirmação sobre o CÓDIGO em vez de sobre o formato dele.
fn squeezed(src: &str) -> String {
    src.chars().filter(|c| !c.is_whitespace()).collect()
}

/// A chamada que começa em `needle`, até o `);` que a fecha.
fn call_at(src: &str, needle: &str) -> String {
    let i = src
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} ausente"));
    let rest = &src[i..];
    rest[..rest.find(");").expect("chamada sem fechamento")].to_string()
}

/// **Qual gesto o press abre é perguntado UMA vez, à porta que também decide o
/// alcance do arrasto.**
///
/// É dessa unicidade que sai a lei do Alt (*sempre o rig inteiro*): as duas
/// metades — suprimir o gesto e alargar o arrasto — saem da mesma condição, e a
/// mutação que este gate mata é a shell comparar `self.interaction.joint` com
/// `JointTool::Ik` na mão, que reintroduz a segunda cópia da regra e faz o Alt
/// funcionar só num dos dois caminhos.
#[test]
fn the_press_asks_one_door_which_gesture_this_is() {
    let src = squeezed(&dispatch_src());
    assert_eq!(
        src.matches("self.interaction.joint.gesture(self.modifiers.alt_key())")
            .count(),
        1,
        "UMA pergunta por press, e o Alt entra NELA — é ele que suprime o gesto"
    );
    assert_eq!(
        src.matches(".gesture(").count(),
        1,
        "uma 2ª chamada de `gesture` é a 2ª cópia da regra do Alt"
    );
    // E o alcance do arrasto vem da IRMÃ dela, nos dois sítios de press (a alça
    // do gizmo e o pick de canvas) — não de um `alt_key()` solto decidindo rig.
    assert_eq!(
        src.matches("drag_reach(self.modifiers.alt_key())").count(),
        2,
        "os dois sítios de press têm de perguntar o alcance à mesma porta"
    );
}

/// **O press pergunta às duas portas de pose, com o RELÓGIO.**
///
/// ⚠️ E **não** com `self.timeline.flags.simulate_physics`, ao contrário da mão:
/// posar não dá passo nenhum, então exigir o toggle obrigaria o artista a armar a
/// simulação para *não* simular. A ausência é afirmada, não deixada implícita —
/// alguém "consertando" a assimetria por simetria a quebra.
#[test]
fn the_press_asks_both_pose_doors_with_the_clock() {
    let src = dispatch_src();
    for (needle, gesture) in [
        ("crate::body_pose::take_pose(", "JointGesture::Ik"),
        ("crate::body_fk::take_fk(", "JointGesture::Fk"),
    ] {
        let call = call_at(&src, needle);
        assert!(
            call.contains("gesture == Some(ph2d_physics_ecs::") && call.contains(gesture),
            "a condição de `{needle}` tem de vir da resposta da porta única, \
             nunca de uma comparação com o modo escrita aqui. Chamada:\n{call}"
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
            src.matches(needle).count(),
            1,
            "UMA porta, UM sítio: uma 2ª chamada de `{needle}` é a 2ª cópia da regra"
        );
    }
}

/// **A FK recebe o MUNDO e o CURSOR**, que a IK não precisa.
///
/// O pivô da FK é a âncora da junta do pai, que só o `SimWorld` sabe onde está,
/// e o gesto é *grab-relative* — sem o ponto do press ele teleportaria o elo
/// para o ângulo do cursor no primeiro Move.
#[test]
fn the_fk_press_is_given_the_world_and_the_grab_point() {
    let call = call_at(&dispatch_src(), "crate::body_fk::take_fk(");
    assert!(
        call.contains("&gfx.sim"),
        "sem o mundo não há como achar a âncora da junta. Chamada:\n{call}"
    );
    assert!(
        call.contains("world_pos"),
        "sem o ponto do press o gesto não é grab-relative. Chamada:\n{call}"
    );
}

/// **Posou ⇒ o arrasto de gizmo NÃO abre.**
///
/// Os dois juntos escreveriam o MESMO `Transform` no mesmo frame, e o de trás
/// venceria em silêncio: o artista veria a cadeia dobrar e o elo pego pular de
/// volta. A supressão é a MESMA variável da mão (`grabbed`), e é isso que este
/// gate afirma — as três metades do `||` estão dentro dela.
#[test]
fn taking_a_pose_suppresses_the_gizmo_drag() {
    let src = dispatch_src();
    for needle in ["crate::body_pose::take_pose(", "crate::body_fk::take_fk("] {
        let at = src.find(needle).expect("a porta é chamada");
        // O `let grabbed = ...` que contém a chamada, achado para trás — não uma
        // contagem de bytes.
        let start = src[..at]
            .rfind("let grabbed =")
            .unwrap_or_else(|| panic!("`{needle}` tem de morar dentro do `let grabbed`"));
        assert!(start < at);
        let after = &src[at..];
        let end = after
            .find("opened_drag = true;")
            .expect("o pick de canvas abre o drag depois das portas");
        assert!(
            after[..end].contains("if !grabbed"),
            "o bloco que abre o drag tem de estar gateado em `!grabbed`"
        );
    }
}

/// **O release precede todo `return`.** Relação de ORDEM, imune a formatação —
/// uma cadeia que sobrevive ao release fica colada no cursor para sempre.
#[test]
fn the_pose_release_runs_before_any_early_return() {
    let body = on_mouse_input_body(&dispatch_src());
    let first_return = body.find("return").expect("o handler tem early-returns");
    for needle in ["self.release_body_pose();", "self.release_body_fk();"] {
        let release = body
            .find(needle)
            .unwrap_or_else(|| panic!("`{needle}` tem de estar no on_mouse_input"));
        assert!(
            release < first_return,
            "`{needle}` tem de vir antes do 1º `return` ({release} < {first_return})"
        );
    }
}

/// **A cadeia segue o cursor no laço de Move**, ao lado dos outros `advance_*`.
#[test]
fn the_move_advances_both_poses() {
    let src = dispatch_src();
    let i = src
        .find("pub(crate) fn on_cursor_moved(")
        .expect("on_cursor_moved existe");
    let body = &src[i..];
    let end = body
        .find("pub(crate) fn on_mouse_wheel(")
        .expect("o próximo fn delimita o corpo");
    for needle in ["self.advance_body_pose();", "self.advance_body_fk();"] {
        assert!(
            body[..end].contains(needle),
            "sem `{needle}` o gesto pega e a cadeia fica onde estava"
        );
    }
}
