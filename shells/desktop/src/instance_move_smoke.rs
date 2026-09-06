//! ⭐⭐⭐ **`PH2D_INSTANCE_SMOKE=7` — MUDAR UMA PEÇA DE LUGAR NO COMPONENTE** (ADR-0164 / F5.12).
//!
//! # O que ela põe na tela
//!
//! Um **Robot** que é receita e **três cópias** dele. Cada um tem um **corpo** azul, uma **cabeça**
//! verde por cima, e um **braço** laranja pendurado no CORPO.
//!
//! # ⚠️ Por que uma cena própria, e não um passo na `=6`
//!
//! A `=5` e a `=6` ensinam *o que uma cópia tem de diferente*; esta ensina **o contrário** — o que
//! a receita manda em todas ao mesmo tempo. E ela precisa de uma peça a DOIS níveis de
//! profundidade (o braço no corpo, com uma cabeça irmã para onde ir), que as outras duas não têm.
//!
//! # ⚠️ O passo mora na LISTA, e por isso a cena DESENHA a lista no stdout
//!
//! Reparentar é um gesto de Hierarquia — não há como fazê-lo no canvas. E foi exactamente aí que o
//! report de 2026-09-06 doeu: *achar a linha certa entre quatro conjuntos de nomes iguais* é uma
//! decisão que o dono não tem como tomar sozinho. ⇒ a cena imprime a árvore como ela aparece,
//! com uma seta em cada uma das duas linhas do gesto. *Um passo que manda procurar não é um passo.*

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SiblingOrder, SimWorld, Transform};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Onde as três cópias aterram.
const COPY_X: [f32; 3] = [-3.2, 0.0, 3.2];
const SCENE_Y: f32 = 0.0;
/// A receita fica LONGE das cópias — ver o cabeçalho da cena 2 do irmão.
const RECIPE_AT: Vec2 = Vec2::new(0.0, 4.2);
/// A cabeça pousa em cima do corpo.
const HEAD_AT: Vec2 = Vec2::new(0.0, 1.15);
/// O braço pendura-se ao lado de quem o carrega — o mesmo local nos dois pais possíveis, e é isso
/// que faz o salto ser exactamente a altura da cabeça.
const ARM_AT: Vec2 = Vec2::new(1.05, 0.0);

const BODY: [f32; 4] = [0.35, 0.55, 0.85, 1.0];
const HEAD: [f32; 4] = [0.40, 0.78, 0.45, 1.0];
const ARM: [f32; 4] = [0.95, 0.55, 0.20, 1.0];

/// Monta a cena. Devolve `(a receita, as três cópias)`.
pub(crate) fn spawn_move_scene(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> (Entity, Vec<Entity>) {
    let robot = sim
        .world_mut()
        .spawn((Transform::from_translation(RECIPE_AT), Name::new("Robot")))
        .id();
    let body = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Body"),
            SiblingOrder(0),
            Sprite::atlas(WHITE_TILE_KEY, [1.2, 1.6], BODY),
            ChildOf(robot),
        ))
        .id();
    sim.world_mut().spawn((
        Transform::from_translation(HEAD_AT),
        Name::new("Head"),
        SiblingOrder(1),
        Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.7], HEAD),
        ChildOf(robot),
    ));
    // ⚠️ **O braço nasce no CORPO** — é o que o gesto vai mudar.
    sim.world_mut().spawn((
        Transform::from_translation(ARM_AT),
        Name::new("Arm"),
        SiblingOrder(0),
        Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.28], ARM),
        ChildOf(body),
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
    /// Cena 7 — ver o cabeçalho do módulo.
    pub(crate) fn instance_smoke_move(&mut self) {
        let vec_entities = &mut self.vec_entities;
        let gfx = self.gfx.as_mut().expect("gfx");
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
        };
        let (master, copies) = spawn_move_scene(&mut gfx.sim, &gfx.component_registry, &mut docs);
        // ⛔⛔⛔ **A RECEITA TEM DE FICAR ABERTA, senão o PASSO 1 nomeia linhas que não existem.**
        //
        // Uma receita **não é uma linha da cena**: desde 2026-08-30 a Hierarquia retira da lista
        // tudo o que o `off_canvas::is_unedited_recipe` acusa, e o `MasterRoot` também é
        // `MasterPiece` — logo a receita INTEIRA sai. A marca que a traz de volta é derivada da
        // **selecção**, então escolhê-la aqui é o que põe as quatro linhas na lista.
        //
        // ⚠️ **E é por isso que ela também se VÊ na tela** (o mesmo `MasterEditing` manda nas duas
        // perguntas): a cena tem **quatro** robôs, e o de cima é o componente. O texto di-lo — um
        // robô a mais que ninguém explicou lê-se como defeito.
        let master_bits = master.to_bits();
        println!(
            "[instance smoke 7] montado: {} robos iguais + o COMPONENTE (o de CIMA, sozinho)",
            copies.len()
        );
        println!(
            "[instance smoke 7] o componente ja' esta' ABERTO — e' por isso que ele aparece na \
             lista E na tela (o robo de cima, sozinho); escolher outra coisa fecha-o"
        );
        println!("[instance smoke 7] a LISTA da esquerda comeca assim:");
        println!("[instance smoke 7]     Robot          <- o COMPONENTE (a linha ja' ACESA)");
        println!("[instance smoke 7]       Body");
        println!("[instance smoke 7]         Arm        <- (1) ARRASTE ESTA LINHA");
        println!("[instance smoke 7]       Head         <- (2) E LARGUE EM CIMA DESTA");
        println!("[instance smoke 7]     Robot (1)      (as tres copias vem depois)");
        println!(
            "[instance smoke 7] PASSO 1 (na LISTA da esquerda): arraste o 'Arm' de dentro do \
             'Robot' ACESO e largue-o em cima do 'Head' logo abaixo"
        );
        println!(
            "[instance smoke 7] => nos TRES robos da tela o braco laranja sobe do corpo para a \
             cabeca, ao mesmo tempo"
        );
        println!(
            "[instance smoke 7] PASSO 2 (na LISTA): tente fazer o mesmo dentro de um robo \
             NUMERADO — arraste o 'Arm' dele para o 'Head' dele"
        );
        println!(
            "[instance smoke 7] => o app recusa e diz onde fazer: o lugar de uma peca e' do \
             componente"
        );
        println!(
            "[instance smoke 7] (deu errado se: as linhas 'Body'/'Head'/'Arm' do 'Robot' aceso nao \
             estiverem na lista · so' um robo mudar · nenhum mudar · o braco voltar sozinho para o \
             corpo · ou o passo 2 mexer no robo numerado sem dizer nada)"
        );
        // ⚠️ **Depois dos `println!`, e não antes** — o `gfx` está emprestado ao `docs` até aqui.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(master_bits));
        }
    }
}

#[cfg(test)]
#[path = "instance_move_smoke_tests.rs"]
mod tests;
