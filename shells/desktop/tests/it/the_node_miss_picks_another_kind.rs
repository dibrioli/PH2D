//! **Arch-gate: em Edit do vetor, o clique do Node que não acerta no vetor escolhe o objecto de
//! OUTRO tipo por baixo** (dono, 05/10: *«ao clicar num objeto de outro tipo, o objeto deve ser
//! selecionado mas em modo object»*). O Select já o fazia (o clique dele vai ao pick de objecto); o
//! Node toma o clique, e o braço dele só se alcança com janela e GPU — o que se afirma é o FONTE: o
//! braço lê o `Ignored` do press e pergunta à lei pura (`vector_mode::another_kind_under`, gate na
//! crate) pela porta ÚNICA do pick.

fn dispatch() -> String {
    crate::input_text::dispatch()
}

/// Mutação que tem de sangrar: tirar a chamada do braço; ou trocar a porta do pick.
#[test]
fn the_node_arm_hands_a_miss_to_the_pick_of_another_kind() {
    let src = dispatch();
    let arm = src
        .find("None if node_mode =>")
        .expect("o braco `None if node_mode` mudou de forma");
    let press = src[arm..]
        .find("node_missed = self.vec.pen.on_press_node(")
        .map(|i| arm + i)
        .expect("o braco do Node nao le o retorno do press");
    assert!(
        src[press..].contains("== ph2d_vec_edit::PenClick::Ignored"),
        "o braco do Node nao pergunta se o press falhou"
    );
    let call = src[press..]
        .find("self.vetor_node_escolhe_outro_tipo();")
        .expect("o falhanco do Node nao chega ao pick de outro tipo");
    assert!(
        src[press..press + call].contains("if node_missed {"),
        "a escolha corre sem o falhanco"
    );
    let def = src
        .find("fn vetor_node_escolhe_outro_tipo(")
        .expect("a porta do pick de outro tipo sumiu");
    let body = &src[def..];
    let end = body.find("\n    }\n").expect("o corpo acaba");
    let body = &body[..end];
    assert!(
        body.contains("crate::hover_highlight::pick_objects_at("),
        "nao e a porta unica do pick"
    );
    assert!(
        body.contains("another_kind_under("),
        "a lei pura nao decide"
    );
}
