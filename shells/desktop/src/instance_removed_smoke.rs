//! ⭐⭐⭐ **`PH2D_INSTANCE_SMOKE=5` — O QUE É SÓ DESTA CÓPIA** (ADR-0164 / plano F5.10).
//!
//! # O que ela põe na tela
//!
//! Um **Robô** que é receita, e **três cópias** dele lado a lado. Cada um tem um **corpo** azul e um
//! **braço** laranja. É a cena mais simples que este módulo tem, e isso é deliberado.
//!
//! # ⚠️ Por que uma cena NOVA, e não um passo a mais na `=3`
//!
//! O report do dono (2026-09-06) foi *«não foi claro o suficiente para eu entender»* sobre os passos
//! dos órfãos, que viviam na cena do aninhamento. Medido no texto daquele smoke, cada passo pedia ao
//! artista **três** decisões de uma vez — achar a linha da RECEITA no meio de duas cópias com nome
//! parecido, saber que ali se apaga, e ler um cartão para ver o efeito.
//!
//! ⇒ aqui cada passo é **um gesto só**, o sujeito está sempre visível no canvas, e a cena ensina
//! **uma** coisa: *o que uma cópia tem de diferente da receita, e como desfazer.*
//!
//! ⚠️ **TRÊS cópias, e não duas:** com duas, *«mudou esta»* e *«mudou metade»* dão a mesma tela.
//! A do meio é a que se mexe; as das pontas são as testemunhas.
//!
//! ⚠️ **O braço fica LONGE do corpo**, com folga entre os robôs: o gesto é arrastá-lo e depois
//! apagá-lo, e uma peça encostada não se distingue do corpo num clique de canvas.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SiblingOrder, SimWorld, Transform};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Onde as três cópias aterram — e a folga entre elas é o que deixa clicar num braço sem apanhar o
/// vizinho.
const COPY_X: [f32; 3] = [-3.2, 0.0, 3.2];
const SCENE_Y: f32 = 0.0;
/// A receita fica LONGE das cópias — ver o cabeçalho da cena 2 do irmão.
const RECIPE_AT: Vec2 = Vec2::new(0.0, 3.6);
/// Onde o braço se pendura no corpo.
const ARM_AT: Vec2 = Vec2::new(1.05, 0.35);

const BODY: [f32; 4] = [0.35, 0.55, 0.85, 1.0];
const ARM: [f32; 4] = [0.95, 0.55, 0.20, 1.0];

/// Monta a cena. Devolve `(a receita, as três cópias)`.
///
/// ⚠️ **DUAS cenas a usam** — esta e a `=6` (a peça acrescentada). Elas ensinam as duas metades
/// da mesma pergunta (*«o que esta cópia tem de diferente»*) sobre o mesmo objecto, e uma
/// segunda montagem divergiria no dia em que uma delas ganhasse uma peça.
pub(crate) fn spawn_robot_scene(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> (Entity, Vec<Entity>) {
    let robot = sim
        .world_mut()
        .spawn((Transform::from_translation(RECIPE_AT), Name::new("Robot")))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Body"),
        SiblingOrder(0),
        Sprite::atlas(WHITE_TILE_KEY, [1.2, 1.6], BODY),
        ChildOf(robot),
    ));
    sim.world_mut().spawn((
        Transform::from_translation(ARM_AT),
        Name::new("Arm"),
        SiblingOrder(1),
        Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.28], ARM),
        ChildOf(robot),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (master, first) = crate::instance_verbs::make_master(sim, registry, robot, docs)
        .expect("o Robo vira receita");
    sim.world_mut()
        .entity_mut(first)
        .insert(Transform::from_translation(Vec2::new(COPY_X[0], SCENE_Y)));
    let mut copies = vec![first];
    for x in &COPY_X[1..] {
        if let Ok(c) = crate::instantiate::instantiate_master(
            sim,
            registry,
            master,
            None,
            docs,
            crate::instantiate::ArtLink::Own,
        ) {
            sim.world_mut()
                .entity_mut(c)
                .insert(Transform::from_translation(Vec2::new(*x, SCENE_Y)));
            copies.push(c);
        }
    }
    (master, copies)
}

impl crate::App {
    /// Cena 5 — ver o cabeçalho do módulo.
    ///
    /// ⚠️ **Cada passo é UM gesto**, e o passo diz **onde** ele acontece (canvas ou lista). Foi a
    /// mistura dos dois num passo só que produziu o report de 2026-09-06.
    pub(crate) fn instance_smoke_removed(&mut self) {
        let vec_entities = &mut self.vec_entities;
        let gfx = self.gfx.as_mut().expect("gfx");
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
        };
        let (_master, copies) = spawn_robot_scene(&mut gfx.sim, &gfx.component_registry, &mut docs);
        println!(
            "[instance smoke 5] montado: {} robos iguais, todos do componente 'Robot'",
            copies.len()
        );
        // ⛔⛔ **A linha que aqui esteve era FALSA** (medido em 2026-09-06): uma receita **não é
        // uma linha da cena** — a Hierarquia retira da lista tudo o que o
        // `off_canvas::is_unedited_recipe` acusa, e o `MasterRoot` também é `MasterPiece`. Ela
        // dizia *«na lista: 'Robot' é o COMPONENTE»* sobre uma linha que não está lá. ⚠️ *O texto
        // de um smoke é superfície de produto: uma frase falsa manda o dono procurar, e o report
        // que volta é indistinguível de um defeito.*
        println!(
            "[instance smoke 5] (na lista da esquerda estao SO' as tres copias — 'Robot (1)', \
             'Robot (2)' e 'Robot (3)'; o componente 'Robot' vive na biblioteca e nao aparece \
             nesta lista)"
        );
        println!(
            "[instance smoke 5] PASSO 1 (na TELA): clique na barra LARANJA do robo do MEIO — e' o \
             braco dele"
        );
        println!(
            "[instance smoke 5] PASSO 2 (na TELA): arraste esse braco um pouco para cima — so' \
             esse robo muda, e os outros dois ficam iguais"
        );
        println!(
            "[instance smoke 5] PASSO 3 (na LISTA da esquerda): botao direito na linha 'Arm' que \
             esta' acesa > 'Delete'"
        );
        println!(
            "[instance smoke 5] => o braco some SO' DESSE robo. Os outros dois continuam com o \
             deles, e o componente tambem"
        );
        println!(
            "[instance smoke 5] PASSO 4 (no cartao do topo do Inspector): aparece o botao 'Put \
             back \"Arm\"' — clique nele"
        );
        println!(
            "[instance smoke 5] => o braco volta, E VOLTA ONDE VOCE O TINHA POSTO (a sua mudanca \
             do passo 2 voltou junto)"
        );
        println!(
            "[instance smoke 5] (deu errado se: sumir o braco dos TRES robos · o botao 'Put back' \
             nao aparecer · ou o braco voltar no lugar de fabrica em vez de onde voce o pos)"
        );
    }
}

#[cfg(test)]
#[path = "instance_removed_smoke_tests.rs"]
mod tests;
