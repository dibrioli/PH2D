//! Os gates da RECUSA de uma peça (ADR-0164 / F5.10) — o *Removed GameObject* do Unity.
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::tests`]**, e o corte foi imposto pelo
//! `shell_files_respect_hr18_loc_cap` (699 de 600). Lá mede-se *«a forma de uma cópia segue a da
//! receita»*; aqui *«e o que acontece quando o artista diz não»*. São as duas metades da mesma
//! função, e a fronteira entre elas é exactamente a marca.
//!
//! ⚠️ **O oráculo é sempre a ÁRVORE das três partes** — a cópia que recusou, a irmã e a receita.
//! Medir só a primeira não distingue *«esta cópia perdeu a peça»* de *«a receita perdeu a peça»*.

use super::tests::*;
use crate::instance_sync::MasterEcho;
use ph2d_ecs::{ChildOf, Children, Entity, Name, SimWorld, Transform};

fn tint_of(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world()
        .get::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint
}

/// A peça `Box` desta cópia, e o `StableId` da peça do MESTRE de que ela nasceu.
fn box_of(sim: &SimWorld, inst: Entity) -> (Entity, u64) {
    let e = sim
        .world()
        .get::<Children>(inst)
        .and_then(|k| k.iter().copied().next())
        .expect("a copia tem a peca");
    let sid = sim
        .world()
        .get::<ph2d_ecs::InstanceOf>(e)
        .expect("elo")
        .master;
    (e, sid)
}

/// ⭐⭐⭐ **Uma cópia pode recusar uma peça, e SÓ ELA a perde** (F5.10).
///
/// ⚠️ As três metades são precisas: a cópia que recusou perde a peça, a **irmã** continua com ela,
/// e a **receita** fica intacta. Sem a segunda isto seria apagar no mestre com outro nome; sem a
/// terceira seria um *Delete* disfarçado.
///
/// (Mutação: o `reconcile` a ignorar o `removed` numa das duas metades ⇒ RED.)
#[test]
fn a_piece_the_copy_refused_leaves_that_copy_and_stays_in_the_others() {
    let (mut sim, r, master, inst) = scene();
    let sister = instantiate(&mut sim, &r, master);
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (piece, sid) = box_of(&sim, inst);
    assert_eq!(names(&sim, inst), ["Box"], "a fixtura comecava sem a peca");

    assert_eq!(
        crate::instance_structure::refuse_pieces(&mut sim, &[piece.to_bits()]),
        1
    );
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);

    assert!(
        names(&sim, inst).is_empty(),
        "a copia que recusou continua com a peca: {:?}",
        names(&sim, inst)
    );
    assert_eq!(names(&sim, sister), ["Box"], "a IRMA perdeu a peca tambem");
    assert_eq!(names(&sim, master), ["Box"], "a RECEITA perdeu a peca");
    assert!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(inst)
            .is_some_and(|o| o.removed.contains(&sid)),
        "a decisao nao ficou guardada na copia"
    );
}

/// ⭐⭐⭐ **Devolver a peça traz a EXCEPÇÃO com ela** — e sem uma linha de código sobre poses.
///
/// ⚠️ É a prova de que a recusa entra pela mesma maquinaria do mestre a apagar: o passe sepulta as
/// excepções da peça ao tirá-la (`entomb`) e **exuma-as** ao pô-la de volta. *O gesto escreve um id;
/// quem faz o trabalho é o passe.*
///
/// (Mutação: o `refuse_pieces` a despawnar a peça em vez de marcar ⇒ a excepção não é sepultada e
/// não volta ⇒ RED.)
#[test]
fn putting_a_refused_piece_back_brings_its_exception_with_it() {
    let (mut sim, r, _master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (piece, sid) = box_of(&sim, inst);
    // A excepção do artista: a peça desta cópia fica vermelha.
    sim.world_mut()
        .get_mut::<ph2d_render::Sprite>(piece)
        .expect("sprite")
        .tint = [1.0, 0.0, 0.0, 1.0];
    pass(&mut sim, &r, &mut echo);

    crate::instance_structure::refuse_pieces(&mut sim, &[piece.to_bits()]);
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    assert!(names(&sim, inst).is_empty(), "a peca nao saiu");

    assert!(crate::instance_structure::restore_piece(
        &mut sim,
        inst.to_bits(),
        sid
    ));
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    let (back, _) = box_of(&sim, inst);
    assert_eq!(
        tint_of(&sim, back),
        [1.0, 0.0, 0.0, 1.0],
        "a peca voltou com o valor da RECEITA — a excepcao do artista evaporou"
    );
}

/// ⛔ **A LEI DO PASSE não mudou: um `despawn` CRU continua a ser desfeito.**
///
/// ⚠️ É o que separa a feature de um buraco. A forma de uma cópia é a da receita, e só o **gesto**
/// tem como dizer *«de propósito»* — a marca. *A guarda vive no gesto; a lei fica no passe.*
///
/// (Mutação: o `reconcile` a saltar toda peça ausente, e não só as marcadas ⇒ RED.)
#[test]
fn a_raw_despawn_is_still_undone_by_the_pass() {
    let (mut sim, r, _master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (piece, _) = box_of(&sim, inst);
    sim.world_mut().entity_mut(piece).despawn();
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        names(&sim, inst),
        ["Box"],
        "a peca apagada POR FORA nao voltou — a lei do passe mudou sem ninguem a declarar"
    );
}

/// ⛔ **A recusa NUNCA se limpa sozinha** — a lei dos órfãos, aplicada à outra metade.
///
/// Se a receita apagar a peça e alguém desfizer, ela volta ao mestre — e a cópia que a recusou
/// **continua a recusá-la**. *Sair por causa de um gesto de outra pessoa é perder trabalho do
/// artista em silêncio.*
///
/// (Mutação: o `reconcile` a podar do `removed` o que o mestre já não tem ⇒ RED.)
#[test]
fn the_refusal_outlives_the_recipe_deleting_and_restoring_the_piece() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (piece, sid) = box_of(&sim, inst);
    crate::instance_structure::refuse_pieces(&mut sim, &[piece.to_bits()]);
    pass(&mut sim, &r, &mut echo);

    // A receita apaga a peça… e alguém desfaz (a peça volta com o MESMO `StableId`).
    let master_box = sim
        .world()
        .get::<Children>(master)
        .and_then(|k| k.iter().copied().next())
        .expect("a receita tem a peca");
    sim.world_mut().entity_mut(master_box).despawn();
    pass(&mut sim, &r, &mut echo);
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        ph2d_ecs::StableId(sid),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(master),
    ));
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);

    assert!(
        names(&sim, inst).is_empty(),
        "a copia recebeu de volta uma peca que ela tinha recusado"
    );
}

/// ⭐⭐ **O *Revert* da RAIZ traz de volta TODAS as peças recusadas** — a saída de quem passou do
/// tecto de botões do cartão, e a que o artista já conhece.
///
/// ⚠️ **Só na raiz**, e a razão é a mesma dos overrides: uma peça recusada não está na cena, logo
/// não há linha nela para clicar. O escopo é o que a mão aponta, e aqui ele apontou a cópia inteira.
///
/// (Mutação: o `revert_all_overrides` não limpar o `removed` ⇒ RED.)
#[test]
fn reverting_the_copy_puts_every_refused_piece_back() {
    let (mut sim, r, _master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (piece, _sid) = box_of(&sim, inst);
    crate::instance_structure::refuse_pieces(&mut sim, &[piece.to_bits()]);
    pass(&mut sim, &r, &mut echo);
    assert!(names(&sim, inst).is_empty(), "a peca nao saiu");

    let done = crate::instance_revert::revert_all_overrides(&mut sim, &mut echo, inst)
        .expect("a raiz e' uma instancia");
    assert_eq!(done.pieces_back, 1, "o Revert nao contou a peca que voltou");
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        names(&sim, inst),
        ["Box"],
        "a peca recusada nao voltou com o Revert — e o cartao tem tecto de botoes"
    );
}

/// ⭐⭐ **Devolver UMA peça deixa as outras recusadas** — a lei irmã da do `✕` dos órfãos.
///
/// ⚠️ Sem este gate, um *Put back* que limpasse o conjunto inteiro passaria: com **uma** peça
/// recusada as duas leis dão a mesma tela, e é por isso que a fixtura tem **duas**. *Uma fixtura de
/// um elemento não pode medir se um gesto é singular.*
///
/// (Mutação: o `restore_piece` a chamar `removed.clear()` ⇒ RED.)
#[test]
fn putting_one_piece_back_leaves_the_others_refused() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let master = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Badge"),
            ph2d_ecs::MasterRoot,
        ))
        .id();
    for n in ["Box", "Tag"] {
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new(n),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(master),
        ));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let inst = instantiate(&mut sim, &r, master);
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);

    let pieces: Vec<Entity> = sim
        .world()
        .get::<Children>(inst)
        .expect("a copia tem pecas")
        .iter()
        .copied()
        .collect();
    assert_eq!(pieces.len(), 2, "a fixtura precisa de DUAS pecas");
    let bits: Vec<u64> = pieces.iter().map(|e| e.to_bits()).collect();
    assert_eq!(crate::instance_structure::refuse_pieces(&mut sim, &bits), 2);
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    assert!(names(&sim, inst).is_empty(), "as duas tinham de sair");

    let first = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .expect("a copia guarda as decisoes")
        .removed
        .iter()
        .copied()
        .next()
        .expect("uma peca recusada");
    assert!(crate::instance_structure::restore_piece(
        &mut sim,
        inst.to_bits(),
        first
    ));
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        names(&sim, inst).len(),
        1,
        "o *Put back* de uma peca devolveu as DUAS — ele e' o botao de baixo com outro rotulo"
    );
}
