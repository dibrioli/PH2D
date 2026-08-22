//! Gates do alinhamento de pilhas — irmão por `#[path]`.

use super::*;

fn blur(radius: f32) -> FxOp {
    FxOp {
        radius,
        ..FxOp::new(FxOp::BLUR)
    }
}

/// **O CASO QUE O ENIO NOMEOU**: o filtro entra depois de o *Default* já ter sido gravado, e o
/// estado que não o conhece parte do **valor zero** em vez de o fazer saltar.
///
/// ⚠️ As três asserções são uma só propriedade vista de três ângulos, e a do MEIO é a que
/// importa: um `t = 0` correto com um meio errado seria um degrau que aparece de uma vez no
/// primeiro quadro — exactamente o defeito que esta lei existe para impedir.
#[test]
fn a_filter_added_after_the_fact_grows_from_zero_instead_of_snapping() {
    let before: Vec<FxOp> = Vec::new(); // o Default, gravado antes de o blur existir
    let after = vec![blur(20.0)]; // o Hover, gravado depois

    let at = |t: f64| mix_stacks(&before, &after, t);

    assert_eq!(at(0.0).len(), 1, "o degrau tem de EXISTIR na partida");
    assert!(
        at(0.0)[0].radius.abs() < 1e-6,
        "o lado que nao conhece o filtro tem de o aplicar com valor ZERO, nao omiti-lo"
    );
    assert!(
        (at(0.5)[0].radius - 10.0).abs() < 1e-4,
        "o meio nao esta' entre as pontas: o filtro salta em vez de crescer"
    );
    assert!(
        (at(1.0)[0].radius - 20.0).abs() < 1e-4,
        "a chegada e' o valor autorado"
    );
}

/// **E o simétrico: retirado depois, ele ENCOLHE até zero** — a mesma lei lida ao contrário. Sem
/// ela, sair de um estado com blur para um sem blur apagaria o desfoque de um quadro para o
/// outro.
#[test]
fn a_filter_missing_on_the_far_side_shrinks_to_zero() {
    let from = vec![blur(20.0)];
    let to: Vec<FxOp> = Vec::new();
    let mid = mix_stacks(&from, &to, 0.5);
    assert_eq!(mid.len(), 1);
    assert!(
        (mid[0].radius - 10.0).abs() < 1e-4,
        "o degrau que sai tem de encolher pelo meio"
    );
    assert!(mix_stacks(&from, &to, 1.0)[0].radius.abs() < 1e-6);
}

/// **A ORDEM da pilha é preservada** — casar por TIPO faria um degrau saltar de posição no meio
/// do hover, e `Shadow → Blur` não desenha o que `Blur → Shadow` desenha.
#[test]
fn the_stack_order_is_index_aligned_never_reshuffled_by_kind() {
    let from = vec![blur(10.0), FxOp::new(FxOp::GLOW)];
    let to = vec![blur(30.0), FxOp::new(FxOp::GLOW)];
    let mid = mix_stacks(&from, &to, 0.5);
    assert_eq!(mid[0].kind, FxOp::BLUR, "o 1o degrau trocou de tipo");
    assert_eq!(mid[1].kind, FxOp::GLOW, "o 2o degrau trocou de tipo");
    assert!((mid[0].radius - 20.0).abs() < 1e-4);
}

/// **Um degrau TROCADO por outro tipo entra do próprio neutro** — não há meio-termo entre um
/// Blur e um Glow, e inventar um daria ao device um degrau que ninguém pode autorar.
#[test]
fn a_kind_swapped_in_place_enters_from_its_own_neutral() {
    let from = vec![blur(20.0)];
    let to = vec![FxOp {
        radius: 8.0,
        ..FxOp::new(FxOp::GLOW)
    }];
    let start = mix_stacks(&from, &to, 0.0);
    assert_eq!(
        start[0].kind,
        FxOp::GLOW,
        "o tipo e' DISCRETO: vai ao destino"
    );
    assert!(
        start[0].radius.abs() < 1e-6,
        "e o valor dele parte do NEUTRO -- nao herda o raio do Blur que estava ali"
    );
}

/// **O neutro tem o `kind` e mais nada** — é o contrato de que a lei toda depende.
///
/// ⚠️ E ele NÃO é o `FxOp::new`, que nasce visível de propósito: o controlo positivo abaixo é o
/// que separa os dois, e sem ele este gate passaria com os dois construtores trocados.
#[test]
fn the_neutral_is_zeroed_and_is_not_the_visible_default() {
    let n = FxOp::neutral(FxOp::GLOW);
    assert_eq!(n.kind, FxOp::GLOW);
    assert!(
        n.enabled,
        "neutro nao e' desligado -- e' ligado e sem efeito"
    );
    assert!(n.radius.abs() < 1e-6 && n.grow.abs() < 1e-6 && n.opacity.abs() < 1e-6);
    assert!(
        FxOp::new(FxOp::GLOW).radius.abs() > 1e-6,
        "controlo positivo: o default do catalogo nasce VISIVEL, senao este gate nao separa nada"
    );
}
