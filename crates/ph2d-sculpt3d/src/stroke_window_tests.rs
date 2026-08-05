//! **A JANELA que um `dab()` publica sobre si mesmo.**
//!
//! Irmão do `stroke_tests.rs` e não parte dele: aquele mede *a lei do traço* (o
//! que o barro faz), este mede *o que a chamada CONTA a quem está de fora* — a
//! lista de vértices que o chamador usa para subir bytes à GPU e para refitar o
//! octree. São perguntas diferentes, e a segunda tem um modo de falha que a
//! primeira não enxerga: a malha na CPU pode estar perfeita e a TELA mostrar
//! outra coisa.
//!
//! ## O report que este arquivo existe para não deixar voltar
//!
//! Enio, 2026-08-05: *"o mouse continua invertido na posição e na direção, e se
//! toco em um lado em x é esculpido do outro lado"* — e, na frase seguinte,
//! ***"isso se usar a simetria"***.
//!
//! A segunda frase é o diagnóstico inteiro. Com o espelho armado, `dab()` roda
//! [`SculptStroke::dab_core`] uma vez por cópia, e cada passagem **zerava** a
//! lista de trabalho antes de a preencher. Quem lia a janela depois do laço
//! recebia **a última cópia e só ela** — o ESPELHO. O lado que o artista tocou
//! movia-se na memória e nunca subia para o device: a mão vai à direita, a
//! esquerda deforma, e o gesto inteiro lê como espelhado. Sem simetria há uma
//! cópia só, ela é a última, e nada disso aparece — que é por que os gates
//! todos estavam verdes.

use super::*;
use ph2d_mesh::{Mesh, shapes};

fn sphere() -> Mesh {
    shapes::uv_sphere(32, 48, 1.0)
}

/// Um dab **inteiramente de um lado** do plano do espelho: centro na superfície
/// em `+X`, olhando de fora para dentro.
///
/// ⚠️ A escolha do centro é o que torna o oráculo binário em vez de um limiar:
/// a calota sob um raio de 0,5 em torno de `[1,0,0]` tem `x ≥ 0,87`, e a cópia
/// espelhada tem `x ≤ −0,87`. Nenhum vértice fica perto de zero, então
/// *"de que lado está"* não tem caso duvidoso.
fn dab_on_the_plus_x_side() -> Dab {
    Dab::at([1.0, 0.0, 0.0], 0.5, [-1.0, 0.0, 0.0])
}

/// Quantos vértices da janela caem de cada lado do plano do espelho — `(+X, −X)`.
fn sides(mesh: &Mesh, window: &[u32]) -> (usize, usize) {
    let mut out = (0usize, 0usize);
    for &v in window {
        if mesh.positions()[v as usize][0] >= 0.0 {
            out.0 += 1;
        } else {
            out.1 += 1;
        }
    }
    out
}

fn drawing_brush() -> Brush {
    Brush {
        verb: Verb::Draw,
        strength: 1.0,
        ..Brush::default()
    }
}

/// **A janela de upload descreve a CHAMADA, não a última cópia dela.**
///
/// Este é o gate do report: um dab espelhado tem de deixar obsoletos os dois
/// lados, senão o device mostra metade do que a malha diz.
#[test]
fn the_gpu_window_covers_both_sides_of_a_mirrored_dab() {
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let moved = stroke.dab(
        &mut mesh,
        &drawing_brush(),
        &dab_on_the_plus_x_side(),
        Symmetry::MIRROR_X,
    );
    assert!(moved > 0, "o dab não moveu nada — a fixture não contém o caso");

    let (right, left) = sides(&mesh, stroke.last_gpu_dirty());
    assert!(
        right > 0 && left > 0,
        "a janela de GPU cobriu apenas UM lado do espelho (+X: {right}, -X: {left}); \
         o outro deforma na memória e nunca sobe — é o \"toco de um lado e esculpe do outro\""
    );
}

/// **O mesmo para a janela do UNDO/refit**, que é a outra lista publicada.
///
/// ⚠️ Gate próprio e não uma segunda asserção no de cima: as duas listas são
/// computadas em lugares diferentes (uma é a pegada escrita, a outra o anel de
/// normais que ela invalida) e podem regredir separadamente. E é esta que o
/// octree lê — um refit que só conhece metade das cópias deixa a estrutura de
/// consulta descrevendo posições que a malha não tem mais, do lado que o
/// artista de fato tocou.
#[test]
fn the_moved_window_covers_both_sides_of_a_mirrored_dab() {
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &drawing_brush(),
        &dab_on_the_plus_x_side(),
        Symmetry::MIRROR_X,
    );

    let (right, left) = sides(&mesh, stroke.last_moved());
    assert!(
        right > 0 && left > 0,
        "a janela de vértices movidos cobriu apenas UM lado (+X: {right}, -X: {left})"
    );
}

/// **O número que `dab()` DEVOLVE é o tamanho da janela que ele PUBLICA.**
///
/// ⚠️ Este é o gate mais forte dos três, e o único que não precisa saber o que é
/// um espelho: o retorno já somava as cópias (`total += dab_core(..)`) enquanto
/// a janela guardava uma só, então a função **se contradizia** — dizia ter
/// movido `2n` vértices e listava `n`. Um chamador que confiasse no número e
/// outro que confiasse na lista discordariam sobre o mesmo dab, e foi
/// exatamente o segundo que desenhou a tela.
#[test]
fn the_count_a_dab_returns_is_the_size_of_the_window_it_publishes() {
    for sym in [Symmetry::default(), Symmetry::MIRROR_X] {
        let mut mesh = sphere();
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let moved = stroke.dab(&mut mesh, &drawing_brush(), &dab_on_the_plus_x_side(), sym);
        assert_eq!(
            moved,
            stroke.last_moved().len(),
            "com {sym:?} o dab devolveu {moved} e publicou {}",
            stroke.last_moved().len()
        );
    }
}

/// **Um dab que não move nada publica uma janela VAZIA** — inclusive depois de
/// um que moveu.
///
/// ⚠️ Sem isto o acumulador seria a porta de um defeito novo, pior que o
/// original: a janela do dab anterior sobrevivendo ao seguinte faz o chamador
/// subir bytes de uma região que ninguém tocou (barato e invisível) e, no
/// caminho do octree, refitar contra uma lista velha. O gate irmão
/// `an_idempotent_dab_does_no_work` já pede isto sem simetria; este pede com o
/// laço de cópias no meio, que é onde o `clear` mudou de lugar.
#[test]
fn a_dab_that_moves_nothing_publishes_nothing() {
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let brush = drawing_brush();
    stroke.dab(&mut mesh, &brush, &dab_on_the_plus_x_side(), Symmetry::MIRROR_X);
    assert!(!stroke.last_moved().is_empty(), "o 1º dab não moveu nada");

    // Longe do barro: a consulta não acha vértice nenhum, nas DUAS cópias.
    let far = Dab::at([0.0, 9.0, 0.0], 0.5, [0.0, -1.0, 0.0]);
    stroke.dab(&mut mesh, &brush, &far, Symmetry::MIRROR_X);
    assert!(
        stroke.last_moved().is_empty() && stroke.last_gpu_dirty().is_empty(),
        "um dab vazio herdou a janela do anterior ({} movidos, {} sujos)",
        stroke.last_moved().len(),
        stroke.last_gpu_dirty().len()
    );
}
