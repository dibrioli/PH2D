//! Os gates do MODO de edição de receita (F4.6).
//!
//! ⚠️ **O oráculo é `is_off_canvas`**, e não a marca: um gate sobre a marca mede o mecanismo que eu
//! escolhi, e não o fim que a frase promete (*«a receita aparece enquanto se mexe nela»*) — a lição
//! de 26/08.

use super::mark;
use crate::render_loop::off_canvas::is_off_canvas;
use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform, Visibility};

/// Uma receita de duas peças, e uma entidade solta que nunca participa.
fn scene() -> (SimWorld, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    let piece = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Box"), ChildOf(root)))
        .id();
    let loose = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Loose")))
        .id();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    (sim, root, piece, loose)
}

/// ⭐⭐⭐ **A receita sai da cena, e VOLTA enquanto está selecionada.**
///
/// As duas verdades que se contradiziam: esconder sempre torna a forma do mestre impossível de
/// mudar; desenhar sempre põe dois objetos empilhados.
///
/// (Mutação: `is_off_canvas` ignorar o `MasterEditing` ⇒ RED na 2.ª metade.)
#[test]
fn the_recipe_comes_back_while_it_is_being_edited() {
    let (mut sim, root, piece, _) = scene();
    mark(&mut sim, None);
    assert!(
        is_off_canvas(sim.world(), root) && is_off_canvas(sim.world(), piece),
        "a receita esta' na cena sem ninguem a editar — dois objetos empilhados"
    );
    // O gesto: escolher uma PEÇA dela na Hierarquia.
    mark(&mut sim, Some(piece.to_bits()));
    assert!(
        !is_off_canvas(sim.world(), root) && !is_off_canvas(sim.world(), piece),
        "a receita nao voltou ao escolher uma peca dela — a forma do mestre fica inalcancavel"
    );
}

/// ⚠️⚠️ **As DUAS metades do passe** — marcar sem desmarcar deixa a receita visível para sempre
/// depois de o artista mudar de selecção.
///
/// (Mutação: apagar o laço do `difference` inverso ⇒ RED.)
#[test]
fn changing_the_selection_puts_the_recipe_back_out_of_the_scene() {
    let (mut sim, root, piece, loose) = scene();
    mark(&mut sim, Some(root.to_bits()));
    assert!(!is_off_canvas(sim.world(), piece));
    mark(&mut sim, Some(loose.to_bits()));
    assert!(
        is_off_canvas(sim.world(), root) && is_off_canvas(sim.world(), piece),
        "a receita ficou visivel depois de o artista mudar de selecao"
    );
}

/// ⛔ **Escolher uma coisa qualquer não acende receita nenhuma**, e o olho do artista continua a
/// valer por cima do modo.
#[test]
fn a_loose_object_lights_nothing_and_the_eye_still_wins() {
    let (mut sim, root, _, loose) = scene();
    mark(&mut sim, Some(loose.to_bits()));
    assert!(!is_off_canvas(sim.world(), loose), "o objeto solto sumiu");
    // O olho fechado esconde mesmo a receita que está a ser editada — ele é autoria do artista.
    mark(&mut sim, Some(root.to_bits()));
    sim.world_mut()
        .entity_mut(root)
        .insert(Visibility::hidden());
    assert!(
        is_off_canvas(sim.world(), root),
        "o modo de edicao passou por cima do olho da Hierarquia"
    );
}
