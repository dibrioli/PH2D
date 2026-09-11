//! **Dentro de um fluxo o arrasto REORDENA, e não escreve `Transform`** — arch-gate sobre a
//! costura que nenhum unit test alcança (plano UI/UX W2, ADR-0153 corolário).
//!
//! A LEI é gateada onde ela mora: os testes do `layout_reorder` dirigem um `SimWorld` REAL
//! headless e provam qual slot um ponto pede, que o arrastado sai da régua, e que fora de um fluxo
//! o gesto não existe. O que eles **não podem tocar** é o `advance_gizmo_drag`, que precisa de
//! `App` + `HeroScreen` + janela — e é lá que se decidem as duas metades que faltam:
//!
//! 1. **O braço existe, e pergunta ao `flow_parent`.** Sem ele o arrasto cai no caminho de sempre
//!    e escreve a pose derivada: a forma salta de volta para a fila (o layout re-resolve no frame
//!    seguinte) e o artista fica com um gesto que parece quebrado.
//! 2. **Ele NÃO escreve `Transform`, e esta é a metade cara.** A escrita seria *invisível* — a
//!    posição é derivada —, mas o undo deste editor regista por **DIFF do mundo ECS**, então cada
//!    frame de arrasto viraria um passo de undo sobre um número que ninguém lê. Um gate que só
//!    afirmasse (1) ficaria VERDE sobre exactamente esse defeito.
//!
//! ⚠️ Nada aqui afirma distância em bytes nem vizinhança de linhas: a lição de
//! `the_dispatch_is_handed_the_live_geometry` (2026-07-23) é que um proxy posicional expira na
//! wave seguinte. O que se afirma é *que pergunta é feita* e *o que o braço faz com a resposta*.

use std::fs;

fn source() -> String {
    fs::read_to_string("src/input_dispatch/gizmo_drag.rs").expect("gizmo_drag.rs")
}

/// O corpo do braço do fluxo: da pergunta até o `} else {` que abre o caminho de sempre.
fn flow_arm(src: &str) -> String {
    let i = src.find("crate::layout_reorder::flow_parent(").expect(
        "o arrasto NAO pergunta se a forma esta' dentro de um fluxo — dentro de uma moldura \
             que empilha ele escreve a pose DERIVADA, e a forma salta de volta para a fila",
    );
    let after = &src[i..];
    let end = after
        .find("} else {")
        .expect("o braco do fluxo nao fecha num `} else {`");
    after[..end].to_string()
}

/// **O braço chama a porta de reordenar.** Perguntar e não agir seria pior que não perguntar: o
/// arrasto viraria um no-op mudo.
#[test]
fn the_flow_arm_reorders_through_the_one_door() {
    let arm = flow_arm(&source());
    assert!(
        arm.contains("crate::layout_reorder::drop_at("),
        "o braco do fluxo pergunta e nao AGE — o arrasto viraria um no-op mudo:\n{arm}"
    );
}

/// **E ele NÃO escreve `Transform`.**
///
/// ⚠️ A mutação que este gate existe para matar é a mais barata de todas: acrescentar a escrita
/// "por segurança" no braço novo. Ela não muda um pixel (a pose é derivada) e enche a fila de
/// undo de passos que o artista não pediu.
#[test]
fn the_flow_arm_never_writes_the_authored_transform() {
    let arm = flow_arm(&source());
    assert!(
        !arm.contains("get_mut::<Transform>"),
        "o braco do fluxo escreve a pose AUTORADA — ela e' invisivel (o layout re-resolve), mas o \
         undo regista por DIFF, entao cada frame de arrasto vira um passo espurio:\n{arm}"
    );
}

/// **Só o TRANSLATE muda de significado.**
///
/// ⚠️ Escalar um filho dentro de um fluxo continua a ser escalar — o tamanho dele é uma ENTRADA da
/// disposição, e o layout re-flui em volta. Sem esta guarda um `ScaleCorner` seria engolido pelo
/// reordenar, e a alça de canto ficaria morta dentro de toda moldura que empilha.
#[test]
fn only_a_translate_becomes_a_reorder() {
    let src = source();
    let i = src
        .find("crate::layout_reorder::flow_parent(")
        .expect("o braco do fluxo");
    // A condição do braço é o que vem entre o `} else if` anterior e a pergunta.
    let head_start = src[..i]
        .rfind("} else if")
        .expect("o braco do fluxo nao e' um `else if`");
    let head = &src[head_start..i];
    assert!(
        head.contains("GizmoDragKind::Translate"),
        "o braco do fluxo engole TODO gesto — um scale de canto ficaria morto dentro de qualquer \
         moldura que empilha:\n{head}"
    );
}
