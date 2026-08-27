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
    mark(&mut sim, None::<u64>);
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

/// ⭐⭐⭐ **Uma receita seleccionada com Shift também acende** — auditoria §1.6.
///
/// O passe lia só o **primário**, e duas rotas correntes deixam a receita seleccionada sem o ser:
/// `add_to_selection` (Shift/Ctrl-clique) e o atalho `preserves_multi` do ramo `Replace`. A linha
/// ficava realçada na Hierarquia e o canvas continuava vazio.
///
/// ⚠️ **O caso mede as DUAS coisas ao mesmo tempo** — a receita acende **e** a outra coisa
/// seleccionada continua na cena —, senão um `mark` que ignorasse o primário passaria na metade
/// que interessa.
///
/// (Mutação que o mata: `.take(1)` **logo a seguir** ao `selection.into_iter()` — o `Option<u64>`
/// de antes. ⚠️ **A 1.ª mutação que tentei SOBREVIVEU, e ela é que estava errada:** pôr o `take(1)`
/// *depois* do `filter_map(master_root_of)` é no-op, porque o objeto solto já tinha sido descartado
/// ali por não ser receita. *Uma mutação a jusante do filtro não mede o que o filtro recebeu.*
/// O chamador tem arch-gate próprio, `the_recipe_mark_is_fed_the_extra_selection_too`.)
#[test]
fn a_recipe_selected_as_an_extra_lights_up_too() {
    let (mut sim, root, piece, loose) = scene();
    // O gesto: clicar no objeto solto e depois Shift-clicar a linha da receita.
    mark(&mut sim, [loose.to_bits(), root.to_bits()]);
    assert!(
        !is_off_canvas(sim.world(), root) && !is_off_canvas(sim.world(), piece),
        "a receita ficou escondida por nao ser a selecao PRIMARIA — e a linha dela esta' realcada"
    );
    assert!(
        !is_off_canvas(sim.world(), loose),
        "acender a receita apagou o outro selecionado"
    );
    // E as duas metades continuam a valer com N: largar tudo apaga.
    mark(&mut sim, None::<u64>);
    assert!(
        is_off_canvas(sim.world(), root),
        "a receita ficou acesa depois de a selecao esvaziar"
    );
}

/// ⭐⭐⭐ **A CENA DO SMOKE, no quadro 0: a receita não tem um pixel** — auditoria §1.7.
///
/// ⛔ Este é o gate que faltava, e a sua ausência custou o report inteiro. Os textos das duas cenas
/// diziam *«receita 'Ragdoll' lá em cima»* e *«receita 'Badge' à ESQUERDA, longe das cópias»*, com
/// as coordenadas escolhidas precisamente para ela ficar visível — e desde a F4.6 aquelas
/// coordenadas têm **zero pixels**, porque nada está seleccionado no arranque. *O instrumento que
/// existe para dar o meio-caminho entregava exactamente o report que ele existe para evitar.*
///
/// ⚠️ **Nenhum gate atravessava a cena do smoke.** Os que existem medem os INGREDIENTES (que
/// `VecPathId` cada peça recebeu) numa fixtura de duas entidades feita à mão; nenhum perguntava
/// *«quantas peças desta cena desenham no quadro 0?»*. É a mesma distância entre a marca e o fim
/// que os outros sete achados têm.
///
/// (Mutação: `want.insert(root)` em vez de `want.extend(subtree(..))` ⇒ RED na 2.ª metade — o
/// PASSO 1 do smoke acenderia meia receita. ⚠️ A mutação que apaga o laço de **desmarcar**
/// sobrevive aqui de propósito: neste gate o `mark(None)` corre primeiro num mundo onde nada está
/// marcado, e quem a mata é `changing_the_selection_puts_the_recipe_back_out_of_the_scene`.)
#[test]
fn the_smoke_scene_shows_its_recipe_only_after_the_row_is_clicked() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let (master, _roots) = crate::instance_smoke::spawn_ragdoll_scene(
        &mut sim,
        &r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    );
    let recipe: Vec<Entity> = {
        let mut out = vec![master];
        let mut i = 0;
        while i < out.len() {
            if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(out[i]) {
                let kids: Vec<Entity> = kids.iter().copied().collect();
                out.extend(kids);
            }
            i += 1;
        }
        out
    };
    assert!(recipe.len() > 1, "a receita do smoke nao tem pecas");

    // O quadro 0 do smoke: ninguém escolheu nada.
    mark(&mut sim, None::<u64>);
    for &e in &recipe {
        assert!(
            is_off_canvas(sim.world(), e),
            "uma peca da receita desenha no arranque — o texto do smoke e o canvas concordam \
             outra vez, mas por acidente"
        );
    }
    // O PASSO 1 que o texto agora manda dar.
    mark(&mut sim, Some(master.to_bits()));
    for &e in &recipe {
        assert!(
            !is_off_canvas(sim.world(), e),
            "clicar na linha da receita nao a trouxe — o PASSO 1 do smoke aponta para nada"
        );
    }
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
