//! ⭐ **Os dois FIOS que ligam a lei do objeto vazio ao que o artista vê** (Enio, 2026-08-26).
//!
//! A lei — *«a caixa é a união dos filhos visíveis, ou o marcador do vazio»* — tem gate próprio em
//! `group_gizmo_view`. O que **não** é alcançável de um teste é a costura: `snapshots::build_view` é
//! um closure que pede `HeroScreen` + `PresentWorld` + câmara, e o passe de pintura pede uma
//! superfície. Uma mutação que apagasse qualquer um dos dois fios deixaria a lei verde e o objeto
//! outra vez impossível de pegar.
//!
//! *Encolher o resíduo é o que se pode fazer quando o arnês não existe; fingir que ele existe, não.*

use std::fs;

/// **O gizmo é PUBLICADO** — sem isto o ramo volta a ser o `return None` de sempre.
#[test]
fn the_gizmo_pass_publishes_a_view_for_a_group_or_an_empty() {
    let src = fs::read_to_string("src/render_loop/snapshots.rs").expect("snapshots.rs");
    assert!(
        src.contains("group_gizmo_view::view("),
        "build_view deixou de publicar a caixa de um grupo/vazio — um objeto sem geometria volta \
         a nao ser agarravel por gesto nenhum (report do Enio, 2026-08-26)"
    );
    // ⚠️ O controlo NEGATIVO: aquele ramo era um `return None` com um comentário, e é exatamente
    // ele que não pode voltar.
    assert!(
        !src.contains("grupo/outro: sem gizmo próprio"),
        "o ramo antigo (`return None`) esta' de volta ao lado do novo — duas respostas para a \
         mesma pergunta, e a primeira ganha"
    );
}

/// **O anel é DESENHADO** — sem isto um objeto vazio continua sem um pixel na tela.
#[test]
fn the_paint_pass_draws_the_empty_object_ring() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    assert!(
        src.contains("empty_object_overlay::draw_empty_object_marks("),
        "o passe de pintura deixou de desenhar o anel do objeto vazio — ele fica invisivel no \
         canvas, que foi a primeira metade do report"
    );
}

/// **A RECEITA sai da tela pelo extract** — sem este fio, *Criar componente* volta a deixar dois
/// objetos empilhados quando a receita é um grupo.
#[test]
fn the_extract_asks_whether_the_entity_is_on_the_canvas() {
    let src = fs::read_to_string("src/render_loop/sim_extract.rs").expect("sim_extract.rs");
    assert!(
        src.contains("off_canvas::is_off_canvas("),
        "o extract deixou de perguntar se a entidade esta' na cena — uma receita que seja um \
         GRUPO volta a desenhar as pecas dela por cima da instancia"
    );
    // ⚠️ O controlo NEGATIVO: a leitura crua de `Visibility` era a resposta ANTIGA, e ela não
    // pode voltar ao lado da nova — duas respostas para a mesma pergunta, e a primeira ganha.
    assert!(
        !src.contains("let hidden = sim"),
        "o extract voltou a decidir a visibilidade no fio, ao lado da porta"
    );
}
