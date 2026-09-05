//! ⭐⭐⭐ **`PH2D_INSTANCE_SMOKE=4` — TROCAR por um componente SEM PARENTESCO** (ADR-0164 / plano
//! F5, o último critério).
//!
//! # O que ela põe na tela, e por que cada escolha está lá
//!
//! Um **Carro** e um **Camião** que não têm nada a ver um com o outro — nenhum nasceu do outro —,
//! e **dois Carros** na cena. Trocar um deles por um Camião é a única operação deste módulo em que
//! o app **não sabe** que peça corresponde a que peça: sem elo não há resposta derivada. Os três
//! itens do menu são as três respostas honestas, e a cena existe para as tornar **visíveis num
//! olhar**.
//!
//! ⚠️⚠️ **A ORDEM DAS PEÇAS ESTÁ TROCADA ENTRE OS DOIS, e é ela que faz a cena ensinar:**
//!
//! ```text
//! Carro   → [0] Body (barra azul)     [1] Wheel (quadrado laranja)
//! Camião  → [0] Wheel (laranja)       [1] Body (barra verde)
//! ```
//!
//! ⇒ arrastar a **barra azul** e depois trocar:
//! - *por nome* leva a mexida para a barra **verde** (a homónima),
//! - *por posição* leva-a para a **roda** (quem ocupa o mesmo lugar),
//! - e o item sem adjectivo **não leva nada**.
//!
//! ⛔ **Com as peças na mesma ordem dos dois lados a cena seria muda:** os dois modos dariam o
//! mesmo resultado, e três itens de menu com uma resposta só ensinam que dois deles são enfeite.
//!
//! ⚠️ **DOIS Carros, e não um** — a mesma razão da cena 3: com um só, *«trocou este»* e *«trocou
//! tudo»* dão a mesma tela.
//!
//! ⚠️ **A mexida é uma POSIÇÃO, e não uma cor.** A excepção tem o COMPONENTE por granularidade, e
//! cor e tamanho vivem os dois no `Sprite` — uma cor levada não se distinguiria de uma cor herdada.
//! Uma peça arrastada, sim: ela fica visivelmente fora do sítio que a receita nova lhe dá.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SiblingOrder, SimWorld, Transform};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Onde os dois Carros aterram.
const CAR_X: [f32; 2] = [-2.6, 1.4];
const SCENE_Y: f32 = 0.0;
/// As receitas ficam LONGE das cópias — ver o cabeçalho da cena 2 do irmão.
const CAR_RECIPE_AT: Vec2 = Vec2::new(-2.0, 3.6);
const TRUCK_RECIPE_AT: Vec2 = Vec2::new(2.0, 3.6);

const BLUE: [f32; 4] = [0.35, 0.55, 0.85, 1.0];
const GREEN: [f32; 4] = [0.35, 0.75, 0.45, 1.0];
const ORANGE: [f32; 4] = [0.95, 0.55, 0.20, 1.0];

/// Uma peça de receita: nome, ordem entre irmãos, sítio, tamanho e cor.
///
/// ⚠️ **A ordem é DADO explícito** (`SiblingOrder`), e não a ordem em que o `spawn` calhou de
/// correr: esta cena é sobre o que o modo *por posição* lê, e deixá-la ao acaso do índice de
/// entidade faria a lição depender de um detalhe do alocador.
struct Piece {
    name: &'static str,
    order: u32,
    at: Vec2,
    size: [f32; 2],
    tint: [f32; 4],
}

fn recipe(sim: &mut SimWorld, name: &str, at: Vec2, pieces: &[Piece]) -> Entity {
    let root = sim
        .world_mut()
        .spawn((Transform::from_translation(at), Name::new(name)))
        .id();
    for p in pieces {
        sim.world_mut().spawn((
            Transform::from_translation(p.at),
            Name::new(p.name),
            SiblingOrder(p.order),
            Sprite::atlas(WHITE_TILE_KEY, p.size, p.tint),
            ChildOf(root),
        ));
    }
    root
}

/// Monta a cena. Devolve `(receita do Carro, receita do Camião, os dois Carros)`.
pub(crate) fn spawn_replace_scene(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> (Entity, Entity, Vec<Entity>) {
    // 1. O Carro: a barra azul PRIMEIRO, a roda depois.
    let car = recipe(
        sim,
        "Car",
        CAR_RECIPE_AT,
        &[
            Piece {
                name: "Body",
                order: 0,
                at: Vec2::ZERO,
                size: [2.0, 0.7],
                tint: BLUE,
            },
            Piece {
                name: "Wheel",
                order: 1,
                at: Vec2::new(-0.7, -0.5),
                size: [0.5, 0.5],
                tint: ORANGE,
            },
        ],
    );
    // 2. O Camião: a roda PRIMEIRO, a barra verde depois — ver o cabeçalho.
    let truck = recipe(
        sim,
        "Truck",
        TRUCK_RECIPE_AT,
        &[
            Piece {
                name: "Wheel",
                order: 0,
                at: Vec2::new(-1.1, -0.6),
                size: [0.6, 0.6],
                tint: ORANGE,
            },
            Piece {
                name: "Body",
                order: 1,
                at: Vec2::new(0.0, 0.25),
                size: [2.6, 1.0],
                tint: GREEN,
            },
        ],
    );
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // 3. As duas viram receitas. O `make_master` deixa uma cópia no lugar; o Camião não precisa de
    //    nenhuma na cena, então a dele é apagada — ele é só a carta que a biblioteca oferece.
    let (car_master, first_car) =
        crate::instance_verbs::make_master(sim, registry, car, docs).expect("o Carro vira receita");
    let (truck_master, truck_copy) = crate::instance_verbs::make_master(sim, registry, truck, docs)
        .expect("o Camiao vira receita");
    if let Ok(em) = sim.world_mut().get_entity_mut(truck_copy) {
        em.despawn();
    }
    sim.world_mut()
        .entity_mut(first_car)
        .insert(Transform::from_translation(Vec2::new(CAR_X[0], SCENE_Y)));
    let mut cars = vec![first_car];
    // 4. O segundo Carro — a testemunha de que a troca é DESTA cópia.
    if let Ok(second) = crate::instantiate::instantiate_master(
        sim,
        registry,
        car_master,
        None,
        docs,
        crate::instantiate::ArtLink::Own,
    ) {
        sim.world_mut()
            .entity_mut(second)
            .insert(Transform::from_translation(Vec2::new(CAR_X[1], SCENE_Y)));
        cars.push(second);
    }
    (car_master, truck_master, cars)
}

/// ⭐⭐⭐ **O rótulo de um item do menu, lido da TABELA que o pinta.**
///
/// ⚠️ **Nenhum destes nomes é escrito à mão nos passos.** Uma cena que copia o rótulo passa a
/// ensinar o nome velho no dia em que alguém renomear o item — e o report que isso produz
/// (*«não achei esse botão»*) é indistinguível de uma feature ausente. *A fonte é a tabela.*
fn item(id: ph2d_editor::NodeId) -> &'static str {
    use ph2d_editor::interaction::ContextMenuKind;
    ph2d_editor::screens::hero::menu_rows::menu_rows(ContextMenuKind::AssetCard {
        cell: ph2d_editor::NodeId(1),
    })
    .iter()
    .find(|(row, _, _)| *row == id)
    .map_or("(este item saiu do menu)", |(_, label, _)| *label)
}

impl crate::App {
    /// Cena 4 — ver o cabeçalho do módulo.
    ///
    /// ⚠️ **Ela IMPRIME os passos**, como as irmãs: aqui o gesto tem uma ordem que ninguém adivinha
    /// (mexer numa peça, e só depois ir à biblioteca escolher por que trocar).
    pub(crate) fn instance_smoke_replace(&mut self) {
        let vec_entities = &mut self.vec_entities;
        let gfx = self.gfx.as_mut().expect("gfx");
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
        };
        let (_car, _truck, cars) =
            spawn_replace_scene(&mut gfx.sim, &gfx.component_registry, &mut docs);
        println!(
            "[instance smoke 4] montado: {} carro(s) na cena, e um 'Truck' que so' existe na \
             biblioteca",
            cars.len()
        );
        println!(
            "[instance smoke 4] PASSO 1: arraste a BARRA AZUL do carro da ESQUERDA um pouco para \
             cima — isso vira uma alteracao SUA daquela peca"
        );
        println!("[instance smoke 4] PASSO 2: menu 'Window' > 'Assets' abre a biblioteca");
        println!(
            "[instance smoke 4] PASSO 3: botao direito no cartao 'Truck' > '{}'",
            item(ph2d_editor::ids::CTX_MENU_ASSET_REPLACE_BY_NAME)
        );
        println!(
            "[instance smoke 4] => o carro da esquerda vira CAMIAO e a barra VERDE dele fica no \
             sitio para onde voce arrastou a azul (o carro da direita nao muda)"
        );
        println!("[instance smoke 4] --- e as outras duas maneiras ---");
        println!(
            "[instance smoke 4] PASSO 4: Ctrl+Z ate' voltar, arraste a barra azul outra vez, e \
             escolha '{}'",
            item(ph2d_editor::ids::CTX_MENU_ASSET_REPLACE_BY_TREE)
        );
        println!(
            "[instance smoke 4] => desta vez quem sobe e' a RODA: no Camiao ela e' a peca que \
             ocupa o lugar que a barra ocupa no Carro"
        );
        println!(
            "[instance smoke 4] PASSO 5: Ctrl+Z, repita, e escolha '{}' (a linha sozinha, sem a \
             segunda parte)",
            item(ph2d_editor::ids::CTX_MENU_ASSET_REPLACE)
        );
        println!(
            "[instance smoke 4] => nada do que voce mexeu e' levado, e o cartao do Inspector \
             passa a listar 'Transform - was on \"Body\"'"
        );
        println!(
            "[instance smoke 4] (deu errado se: os tres itens derem o MESMO resultado, ou se o \
             carro da direita tambem mudar)"
        );
    }
}

#[cfg(test)]
#[path = "instance_replace_smoke_tests.rs"]
mod tests;
