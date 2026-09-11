//! ⭐ **Os FIOS que ligam a lei do objeto vazio ao que o artista vê** (Enio, 2026-08-26).
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
///
/// ⚠️ **A ÂNCORA MUDOU em 2026-08-30, e a lei não.** O extract passou a fazer as **três** perguntas
/// de *«esta entidade desenha?»* por uma porta só (`off_canvas::draws_this_frame`, que chama o
/// `is_off_canvas` como primeiro termo) — a cura dos dois controlos mortos da §8 Visibility, cujo
/// mecanismo está em `the_draw_pass_asks_the_door_that_has_the_gates.rs`. Reancorar era obrigatório:
/// *mudar o modelo re-pergunta o que cada gate ainda mede.*
#[test]
fn the_extract_asks_whether_the_entity_is_on_the_canvas() {
    let src = fs::read_to_string("src/render_loop/sim_extract.rs").expect("sim_extract.rs");
    assert!(
        src.contains("off_canvas::draws_this_frame("),
        "o extract deixou de perguntar se a entidade esta' na cena — uma receita que seja um \
         GRUPO volta a desenhar as pecas dela por cima da instancia"
    );
    // ⚠️ **E a porta continua a fazer a pergunta desta lei.** Sem esta metade, um
    // `draws_this_frame` que tivesse perdido o termo do `is_off_canvas` passaria: a asserção acima
    // mede o CHAMADOR, e a lei vive no chamado.
    let door = fs::read_to_string("src/render_loop/off_canvas.rs").expect("off_canvas.rs");
    assert!(
        door.contains("!is_off_canvas(sim, entity)"),
        "a porta `draws_this_frame` deixou de perguntar pelo olho/receita"
    );
    // ⚠️ O controlo NEGATIVO: a leitura crua de `Visibility` era a resposta ANTIGA, e ela não
    // pode voltar ao lado da nova — duas respostas para a mesma pergunta, e a primeira ganha.
    assert!(
        !src.contains("let hidden = sim"),
        "o extract voltou a decidir a visibilidade no fio, ao lado da porta"
    );
}

/// ⭐⭐⭐ **O primeiro clique é de quem já está selecionado** — o fio da 4.ª volta.
///
/// A lei é pura e tem gate em `pick_order::start_on_selection`; o que **não** é alcançável de um
/// teste é o `input_dispatch`, e sem os dois fios abaixo a lei fica verde e o artista continua a
/// apanhar um filho ao tentar arrastar o pai.
#[test]
fn the_click_dispatch_starts_the_cycle_on_the_selection() {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    assert!(
        src.contains("pick_order::start_on_selection("),
        "o clique deixou de comecar o ciclo na selecao — arrastar um grupo volta a pegar um filho"
    );
    // ⚠️ E a segunda metade: mudar de seleção na HIERARQUIA tem de abrir um ciclo NOVO, senão o
    // ciclo antigo sobrevive e devolve o filho outra vez no mesmo ponto.
    assert!(
        src.contains("hero.gizmo.selection == self.cycle_pick_selection"),
        "o ciclo deixou de estar atado a' selecao — escolher o pai na Hierarquia e clicar no mesmo \
         ponto continua o ciclo antigo"
    );
}
