//! Os gates de **QUEM É PAI DE QUEM** dentro de uma cópia (ADR-0164 / F5.12).
//!
//! ⚠️ **É a terceira metade da forma.** O passe estrutural sabia **materializar** o que falta e
//! **despawnar** o que sobra, e não sabia **mover**: uma peça que o artista arrastasse para outro
//! pai *dentro da receita* ficava, em toda cópia, pendurada no pai antigo — para sempre e em
//! silêncio. É a mesma família que a §F5.8 nomeia ao explicar por que a chave de emparelhamento é
//! um CAMINHO: *uma peça sob o pai errado é estável, desenha, e nada acusa.*
//!
//! ⚠️ **O oráculo é a ÁRVORE da cópia**, e nunca «o passe correu».

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

fn pass(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry, echo: &mut MasterEcho) {
    for _ in 0..2 {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        sync_instances(
            sim,
            r,
            &PhysicsBridge::new(),
            echo,
            &mut OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
        );
    }
}

/// `Robot` > `Body` > `Arm`, mais um `Head` irmão do `Body` — e uma cópia dela.
///
/// ⚠️ **O `Arm` nasce a DOIS níveis de profundidade**: com ele filho da raiz, mover--lo para o
/// `Head` mediria só *«mudou de pai»*; assim mede também que a travessia acha o pai certo quando
/// ele próprio não é a raiz.
fn scene() -> (SimWorld, ph2d_ecs::scene::ComponentRegistry, Entity, Entity) {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Robot"), MasterRoot))
        .id();
    let body = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Body"), ChildOf(master)))
        .id();
    sim.world_mut()
        .spawn((Transform::IDENTITY, Name::new("Head"), ChildOf(master)));
    sim.world_mut()
        .spawn((Transform::IDENTITY, Name::new("Arm"), ChildOf(body)));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let inst = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    (sim, r, master, inst)
}

/// A peça chamada `name` na sub-árvore de `root`.
fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(k) = sim.world().get::<Children>(e) {
            stack.extend(k.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name}");
}

fn parent_name(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<ChildOf>(e)
        .and_then(|c| sim.world().get::<Name>(c.0))
        .map_or_else(String::new, |n| n.0.clone())
}

/// ⭐⭐⭐ **Mover uma peça de lugar na RECEITA move-a em todas as cópias.**
///
/// ⛔ **O defeito que isto cura era SILENCIOSO e permanente:** o passe materializava e despawnava e
/// **nunca reparentava**, então a cópia ficava com o `Arm` pendurado no `Body` para sempre. Nada
/// acusa — a peça existe, desenha, tem os bytes certos, e só a árvore está errada.
///
/// (Mutação: apagar o bloco *«as que estão no SÍTIO ERRADO»* do `reconcile_one` ⇒ RED.)
#[test]
fn moving_a_piece_in_the_recipe_moves_it_in_every_copy() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        parent_name(&sim, piece(&sim, inst, "Arm")),
        "Body",
        "a fixtura tem de partir do braco no corpo, senao mede outra coisa"
    );

    // O artista abre a receita e arrasta o braço para a cabeça.
    let head = piece(&sim, master, "Head");
    let arm = piece(&sim, master, "Arm");
    sim.world_mut().entity_mut(arm).insert(ChildOf(head));
    pass(&mut sim, &r, &mut echo);

    assert_eq!(
        parent_name(&sim, piece(&sim, inst, "Arm")),
        "Head",
        "a copia ficou com o braco no pai ANTIGO — a forma dela deixou de ser a da receita"
    );
    // ⚠️ E é a cabeça DELA, não a da receita: uma cópia que apontasse para uma peça do mestre
    // desenharia dentro da biblioteca.
    let copy_head = piece(&sim, inst, "Head");
    assert_eq!(
        sim.world()
            .get::<ChildOf>(piece(&sim, inst, "Arm"))
            .map(|c| c.0),
        Some(copy_head),
        "o braco da copia foi parar a' cabeca do MESTRE"
    );
}

/// ⭐⭐ **Uma peça que o artista ACRESCENTOU não é mexida** — ela não tem elo, e o passe não lhe
/// toca (F5.11). *Só o que a receita deu é que a receita arruma.*
///
/// ⚠️ **É uma CERCA, e não um gate com mutação:** hoje ele é verdadeiro por construção — o bloco
/// percorre a árvore do MESTRE e só toca no que o mapa `have` conhece, e `have` só tem peças COM
/// elo. Ele existe para que uma reescrita que passasse a percorrer a sub-árvore da **cópia**
/// reprove aqui. *Uma cerca que se declara cerca não finge ser uma medição.*
#[test]
fn an_added_piece_keeps_the_parent_the_artist_gave_it() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let copy_head = piece(&sim, inst, "Head");
    let hat = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Hat"), ChildOf(copy_head)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    // A receita mexe-se por baixo — e a peça do artista fica onde ele a pôs.
    let head = piece(&sim, master, "Head");
    let arm = piece(&sim, master, "Arm");
    sim.world_mut().entity_mut(arm).insert(ChildOf(head));
    pass(&mut sim, &r, &mut echo);

    assert_eq!(
        sim.world().get::<ChildOf>(hat).map(|c| c.0),
        Some(copy_head),
        "o passe arrumou uma peca que a receita nunca deu"
    );
}

/// ⭐⭐⭐ **E a RAIZ da cópia nunca é reparentada** — ela é um objecto da cena, e onde ela está é
/// decisão do artista.
///
/// ⛔ A raiz tem elo como qualquer peça (ele aponta para o `MasterRoot`), então um bloco que
/// varresse *todas* as peças com elo arrastaria a cópia para dentro da biblioteca — que é onde o
/// mestre vive. *É a mesma excepção do `ROOT_IS_ITS_OWN`, um nível acima: a pose e o lugar da raiz
/// são dela.*
///
/// ⚠️⚠️ **A guarda explícita do passe é REDUNDANTE hoje, e a mutação disse-o:** tirá-la deixa este
/// gate VERDE, porque o pai da raiz do mestre **nunca** está no mapa `have` — ele fica *acima* do
/// mestre, e o mapa só tem peças de *dentro* dele. ⇒ a linha fica como **cerca legível** (ela diz a
/// lei no sítio onde alguém a leria) e a redundância está escrita ao lado dela. *Uma linha que
/// mutação nenhuma mata ou é dívida ou é cerca — e a diferença é dizê-lo.*
#[test]
fn the_root_of_a_copy_is_never_reparented_into_the_library() {
    let (mut sim, r, _master, inst) = scene();
    let group = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Group")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    sim.world_mut().entity_mut(inst).insert(ChildOf(group));
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        sim.world().get::<ChildOf>(inst).map(|c| c.0),
        Some(group),
        "o passe arrastou a raiz da copia para outro sitio — o lugar dela e' do artista"
    );
}

/// ⭐⭐⭐ **TROCAR dois níveis na receita deixa a cópia com uma ÁRVORE, e não com um laço.**
///
/// A receita passa de `Body > Head` para `Head > Body`, e na cópia o `Head` ainda está debaixo do
/// `Body` no instante em que o passe corre. *Uma hierarquia com ciclo não é uma árvore — a
/// travessia de transformes não termina, e o sintoma é um app que congela, não um teste vermelho.*
///
/// ⚠️⚠️ **E este gate CORRIGIU uma afirmação minha.** Eu escrevi que a pré-ordem do mestre *«é o
/// que impede o ciclo»* e a mutação que inverte a travessia **sobreviveu**: os alvos são calculados
/// do mestre **antes** de qualquer escrita, logo as atribuições são independentes e o estado final
/// é o mesmo em qualquer ordem — o ciclo existe **entre dois `insert`** e ninguém o observa.
/// *A propriedade é real; a razão que eu tinha escrito ao lado dela não era.*
///
/// ⚠️ **A 1.ª fixtura também não produzia o fenómeno** (os dois eram irmãos): sem o aninhamento, o
/// alvo do movimento não é descendente de quem se move, e não há ciclo nenhum a evitar.
///
/// (Mutação: o `moves.push` a virar no-op ⇒ RED — é o mesmo que mata o gate de cima.)
#[test]
fn swapping_two_levels_in_the_recipe_never_makes_a_cycle() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    let (m_body, m_head) = (piece(&sim, master, "Body"), piece(&sim, master, "Head"));
    // ⚠️⚠️ **A fixtura tem de ANINHAR primeiro, e a 1.ª redacção não o fazia** — com o `Body` e o
    // `Head` IRMÃOS a mutação da ordem sobreviveu, porque um ciclo só se fecha quando o alvo do
    // movimento é **descendente** de quem se move. *Uma fixtura que não produz o fenómeno absolve
    // a linha que o impede.*
    sim.world_mut().entity_mut(m_head).insert(ChildOf(m_body));
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        parent_name(&sim, piece(&sim, inst, "Head")),
        "Body",
        "a copia tem de partir com a cabeca DENTRO do corpo, senao nao ha' ciclo a evitar"
    );

    // E agora a troca: a cabeça sobe à raiz e o corpo passa a pendurar-se nela.
    sim.world_mut().entity_mut(m_head).insert(ChildOf(master));
    sim.world_mut().entity_mut(m_body).insert(ChildOf(m_head));
    pass(&mut sim, &r, &mut echo);

    let (body, head) = (piece(&sim, inst, "Body"), piece(&sim, inst, "Head"));
    assert_eq!(
        sim.world().get::<ChildOf>(head).map(|c| c.0),
        Some(inst),
        "a cabeca da copia nao subiu para a raiz"
    );
    assert_eq!(
        sim.world().get::<ChildOf>(body).map(|c| c.0),
        Some(head),
        "o corpo da copia nao desceu para a cabeca"
    );
    // ⚠️ E a árvore continua a ser uma árvore: subir do braço tem de chegar à raiz num número
    // finito de passos. *Um ciclo lê-se como um app que congela, não como um teste vermelho.*
    let mut e = piece(&sim, inst, "Arm");
    for _ in 0..8 {
        match sim.world().get::<ChildOf>(e) {
            Some(c) => e = c.0,
            None => break,
        }
    }
    assert_eq!(
        e, inst,
        "a subida do braco nao chegou a' raiz em 8 passos — ha' ciclo"
    );
}

/// ⭐⭐⭐ **REORDENAR entre irmãos dentro de uma cópia FICA** — e é isso que a guarda do arrasto
/// deixa passar de propósito (F5.12).
///
/// ⚠️ **Este gate existe porque eu afirmei isto num doc antes de o medir.** A guarda do gesto só
/// recusa quando o **pai** muda, com a justificação de que a ordem *«viaja no `SiblingOrder`, que É
/// componente registado, e vira excepção da cópia como qualquer outro valor»*. Uma frase dessas ao
/// lado de código é uma promessa ao próximo leitor — e uma promessa sem régua envelhece sozinha.
///
/// ⛔ Se ela fosse falsa, a guarda estaria a deixar passar um gesto que se desfaz sozinho — o
/// defeito exacto que ela existe para impedir, pela porta que ela deixou aberta.
#[test]
fn reordering_a_piece_inside_a_copy_sticks_as_an_override() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (body, head) = (piece(&sim, inst, "Body"), piece(&sim, inst, "Head"));
    let before = sim
        .world()
        .get::<ph2d_ecs::SiblingOrder>(head)
        .map(|s| s.0)
        .expect("a peca tem ordem");

    // O artista arrasta o `Head` para cima do `Body` — mesmo pai, outra ordem.
    let swapped = sim
        .world()
        .get::<ph2d_ecs::SiblingOrder>(body)
        .map(|s| s.0)
        .expect("a peca tem ordem")
        .saturating_sub(1);
    assert_ne!(before, swapped, "a fixtura tem de MUDAR a ordem");
    sim.world_mut()
        .entity_mut(head)
        .insert(ph2d_ecs::SiblingOrder(swapped));
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);

    assert_eq!(
        sim.world().get::<ph2d_ecs::SiblingOrder>(head).map(|s| s.0),
        Some(swapped),
        "a ordem que o artista deu foi reescrita pela receita — a guarda do arrasto esta' a deixar \
         passar um gesto que se desfaz sozinho"
    );
    // ⚠️ E a receita **não** se mexeu: o que é da cópia fica na cópia.
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::SiblingOrder>(piece(&sim, master, "Head"))
            .map(|s| s.0),
        Some(before),
        "reordenar dentro de uma copia subiu a' receita"
    );
}
