//! ⭐⭐⭐ **`PH2D_INSTANCE_SMOKE=3` — a RECEITA DENTRO DA RECEITA** (ADR-0164 / plano F5,
//! critério 4).
//!
//! # O que ela põe na tela, e por que cada peça está lá
//!
//! Uma **Roda** que é receita; um **Carro** que *contém* uma Roda e também é receita; **dois
//! Carros** na cena; e **uma Roda solta**. Mexer na roda de um Carro passa a ter **duas** respostas
//! legítimas para *«aplicar ao mestre»* — ao Carro (todos os Carros mudam, a Roda solta não) ou à
//! Roda (toda Roda em todo o lado muda) —, e é isso que a escada do cartão oferece.
//!
//! ⚠️ **A Roda SOLTA é a testemunha, e sem ela a cena não ensina nada:** o que separa os dois
//! degraus não é o que acontece aos Carros — os dois mudam nos dois casos —, é o que acontece a
//! quem **não** está dentro de um.
//!
//! ⚠️ **DOIS Carros, e não um:** com um só, *«mudei a receita»* e *«mudei esta cópia»* dão a mesma
//! tela. É a mesma razão pela qual a cena 2 põe a receita LONGE das cópias.
//!
//! # ⛔ Como esta cena é montada, e por que a forma óbvia está errada
//!
//! ⛔ **`Make Prefab` sobre a raiz de uma cópia faz uma VARIANTE** (F5 critério 2), que *segue* a
//! base. Aninhar é **conter**: nasce um pai vazio (o Carro), a cópia da Roda passa a viver debaixo
//! dele, e só então o Carro vira receita. As duas relações leem-se parecido e são diferentes.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Onde os dois Carros e a Roda solta aterram.
const CAR_X: [f32; 2] = [-3.0, 0.4];
const LOOSE_WHEEL_X: f32 = 3.4;
const SCENE_Y: f32 = 0.0;
/// As receitas ficam LONGE — ver o cabeçalho da cena 2 do irmão.
const RECIPE_AT: Vec2 = Vec2::new(0.0, 3.6);
/// Onde a roda se pendura no carro.
const WHEEL_ON_CAR: Vec2 = Vec2::new(-0.7, -0.5);

const RIM: [f32; 4] = [0.95, 0.55, 0.20, 1.0];
const BODY: [f32; 4] = [0.35, 0.55, 0.85, 1.0];

/// Monta a cena inteira. Devolve `(receita da Roda, receita do Carro, os Carros, a Roda solta)`.
pub(crate) fn spawn_nested_scene(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> (Entity, Entity, Vec<Entity>, Entity) {
    // 1. A Roda: uma raiz com o aro pendurado.
    let wheel = sim
        .world_mut()
        .spawn((Transform::from_translation(RECIPE_AT), Name::new("Wheel")))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Rim"),
        Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], RIM),
        ChildOf(wheel),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // 2. Ela vira receita, e fica uma cópia no lugar.
    let (wheel_master, wheel_copy) = crate::instance_verbs::make_master(sim, registry, wheel, docs)
        .expect("a Roda vira receita");
    // 3. ⛔ NAO `make_master(wheel_copy)` — ver o cabeçalho. Nasce um Carro vazio e a cópia da Roda
    //    passa a viver debaixo dele.
    let car = sim
        .world_mut()
        .spawn((Transform::from_translation(RECIPE_AT), Name::new("Car")))
        .id();
    sim.world_mut()
        .entity_mut(wheel_copy)
        .insert((ChildOf(car), Transform::from_translation(WHEEL_ON_CAR)));
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Body"),
        Sprite::atlas(WHITE_TILE_KEY, [2.0, 0.7], BODY),
        ChildOf(car),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // 4. O Carro vira receita — e a sub-árvore dele CONTÉM uma cópia da Roda.
    let (car_master, first_car) =
        crate::instance_verbs::make_master(sim, registry, car, docs).expect("o Carro vira receita");
    sim.world_mut()
        .entity_mut(first_car)
        .insert(Transform::from_translation(Vec2::new(CAR_X[0], SCENE_Y)));
    let mut cars = vec![first_car];
    // 5. O segundo Carro.
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
    // 6. E a Roda SOLTA — a testemunha.
    let loose = crate::instantiate::instantiate_master(
        sim,
        registry,
        wheel_master,
        None,
        docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("uma Roda na cena");
    sim.world_mut()
        .entity_mut(loose)
        .insert(Transform::from_translation(Vec2::new(
            LOOSE_WHEEL_X,
            SCENE_Y,
        )));
    (wheel_master, car_master, cars, loose)
}

impl crate::App {
    /// Cena 3 — ver o cabeçalho do módulo.
    ///
    /// ⚠️ **Ela IMPRIME os passos.** É a lei do irmão: *um subsistema sem cena de smoke própria
    /// recebe sempre o mesmo report — «não funcionou» — sem o meio caminho*; e aqui o gesto tem
    /// uma ordem que ninguém adivinha (mexer numa peça **de dentro** de uma cópia).
    pub(crate) fn instance_smoke_nested(&mut self) {
        let vec_entities = &mut self.vec_entities;
        let gfx = self.gfx.as_mut().expect("gfx");
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
        };
        let (_wheel, _car, cars, _loose) =
            spawn_nested_scene(&mut gfx.sim, &gfx.component_registry, &mut docs);
        println!(
            "[instance smoke 3] montado: {} carro(s) + 1 roda solta a' direita",
            cars.len()
        );
        println!(
            "[instance smoke 3] PASSO 1: clique na roda LARANJA do carro da ESQUERDA (a peca de \
             dentro da copia, nao o carro inteiro)"
        );
        println!("[instance smoke 3] PASSO 2: arraste-a um pouco — isso vira uma EXCEPCAO dela");
        println!(
            "[instance smoke 3] PASSO 3: no cartao do topo do Inspector aparecem DOIS botoes — \
             'Apply as override in \"Car\"' e 'Apply to \"Wheel\"'"
        );
        println!(
            "[instance smoke 3] PASSO 4: 'Apply as override in \"Car\"' => os DOIS carros mudam e \
             a roda solta NAO"
        );
        println!(
            "[instance smoke 3] PASSO 5: Ctrl+Z ate' voltar, repita, e escolha 'Apply to \
             \"Wheel\"' => a roda solta muda TAMBEM"
        );
        // ⭐⭐⭐ **A segunda metade da cena** (F5 critério 3) — ela não precisa de peças novas: a
        // carroçaria já lá está, e é a peça certa porque **não tem física** (a pose de um corpo
        // dinâmico é do solver, e arrastá-la não produz excepção nenhuma). O caminho inteiro está
        // gateado em `moving_a_body_then_deleting_it_in_the_recipe_leaves_one_named_orphan`.
        println!("[instance smoke 3] --- e para ver as ALTERACOES QUE FICAM SEM DONO ---");
        println!(
            "[instance smoke 3] PASSO 6: arraste a CARROCARIA azul de um dos carros (nao a roda)"
        );
        println!(
            "[instance smoke 3] PASSO 7: na lista da esquerda, abra a receita 'Car' e apague a \\
             linha 'Body' dela"
        );
        println!(
            "[instance smoke 3] PASSO 8: o cartao passa a dizer, por extenso, 'Transform - was on \\
             \"Body\"' -- e o botao ao lado apaga exactamente essa"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance_sync::{MasterEcho, sync_instances};
    use ph2d_physics_ecs::PhysicsBridge;

    fn build() -> (
        SimWorld,
        ph2d_ecs::scene::ComponentRegistry,
        Vec<Entity>,
        Entity,
    ) {
        let mut sim = SimWorld::new();
        let r = crate::init::build_component_registry();
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        let (_w, _c, cars, loose) = spawn_nested_scene(
            &mut sim,
            &r,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
        );
        (sim, r, cars, loose)
    }

    fn rim_of(sim: &SimWorld, root: Entity) -> Entity {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            if sim.world().get::<Name>(e).is_some_and(|n| n.0 == "Rim") {
                return e;
            }
            if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
                stack.extend(kids.iter().copied());
            }
        }
        panic!("a cena nao montou um aro debaixo de {root:?}");
    }

    /// ⭐⭐⭐ **A CENA ENSINA O QUE ACONTECE** — os dois botões que os passos impressos prometem
    /// existem, com estes nomes, nesta ordem.
    ///
    /// ⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente* —
    /// a ausente não é acreditada. Os `println!` dos passos dizem *«aparecem DOIS botões»*, e sem
    /// este gate a única coisa que os desmentiria era o Enio à frente do ecrã.
    #[test]
    fn the_printed_steps_describe_the_ladder_the_scene_produces() {
        let (mut sim, _r, cars, _loose) = build();
        let rim = rim_of(&sim, cars[0]);
        let names: Vec<String> = crate::instance_apply_deep::apply_levels(&mut sim, rim)
            .into_iter()
            .map(|l| l.name)
            .collect();
        assert_eq!(
            names,
            ["Car", "Wheel"],
            "os passos impressos prometem estes dois"
        );
    }

    /// ⭐⭐⭐ **O CAMINHO DO SMOKE DOS ÓRFÃOS existe nesta cena** (F5 critério 3) — mexer na
    /// carroçaria de um Carro da cena e apagá-la na receita deixa **uma** excepção sem alvo, e ela
    /// sabe que era a `Body`.
    ///
    /// ⛔ *Uma cena de smoke que ensina o contrário do que acontece é pior que uma cena ausente* —
    /// e os passos que eu escrevo ao dono só valem o que este gate mede. ⚠️ A carroçaria é a peça
    /// certa para o gesto **porque não tem física**: a pose de um corpo dinâmico é do solver, e
    /// arrastá-la **não** produz excepção nenhuma (condição (b) da refutação 1).
    #[test]
    fn moving_a_body_then_deleting_it_in_the_recipe_leaves_one_named_orphan() {
        let (mut sim, r, cars, _loose) = build();
        let bridge = PhysicsBridge::new();
        let mut echo = MasterEcho::default();
        let run = |sim: &mut SimWorld, echo: &mut MasterEcho| {
            let (mut sc, mut mp) = crate::instance_docs::empty_docs();
            sync_instances(
                sim,
                &r,
                &bridge,
                echo,
                &mut crate::instance_docs::OwnedDocs {
                    vec_scene: &mut sc,
                    vec_entities: &mut mp,
                },
            );
        };
        run(&mut sim, &mut echo);

        // PASSO 1 — o artista arrasta a carroçaria do carro da esquerda.
        let body = named(&sim, cars[0], "Body");
        sim.world_mut()
            .entity_mut(body)
            .insert(Transform::from_translation(Vec2::new(0.3, 0.2)));
        run(&mut sim, &mut echo);
        let root_of = |sim: &SimWorld| {
            sim.world()
                .get::<ph2d_ecs::ObjectInstance>(cars[0])
                .cloned()
                .unwrap_or_default()
        };
        assert_eq!(
            root_of(&sim).overrides.len(),
            1,
            "arrastar a carrocaria nao virou excepcao — o resto do smoke nao mede nada"
        );

        // PASSO 2 — e apaga a carroçaria NA RECEITA.
        let master = ph2d_ecs::master_root_of(sim.world(), named(&sim, cars[0], "Body"))
            .or_else(|| {
                let mut q = sim
                    .world_mut()
                    .query_filtered::<Entity, bevy_ecs::prelude::With<ph2d_ecs::MasterRoot>>();
                q.iter(sim.world())
                    .find(|&e| sim.world().get::<Name>(e).is_some_and(|n| n.0 == "Car"))
            })
            .expect("a receita do Carro");
        let master_body = named(&sim, master, "Body");
        sim.world_mut().entity_mut(master_body).despawn();
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        run(&mut sim, &mut echo);

        let o = root_of(&sim);
        assert_eq!(o.overrides.len(), 0, "a chave ficou a apontar para o nada");
        assert_eq!(
            o.orphans.len(),
            1,
            "a excepcao foi perdida em vez de guardada"
        );
        assert_eq!(
            o.orphans.values().next().expect("o orfao").piece_name,
            "Body",
            "o painel nao vai poder dizer QUAL"
        );
    }

    /// A entidade chamada `name` na sub-árvore de `root` (a raiz incluída).
    fn named(sim: &SimWorld, root: Entity, name: &str) -> Entity {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            if sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
                return e;
            }
            if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
                stack.extend(kids.iter().copied());
            }
        }
        panic!("nao ha' {name:?} debaixo de {root:?}");
    }

    /// ⭐⭐ **A Roda SOLTA é mesmo uma testemunha** — ela segue a receita da Roda e **não** está
    /// dentro de Carro nenhum. Sem isto os dois degraus da cena seriam indistinguíveis.
    #[test]
    fn the_loose_wheel_follows_the_wheel_and_no_car() {
        let (mut sim, r, cars, loose) = build();
        let bridge = PhysicsBridge::new();
        let mut echo = MasterEcho::default();
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        sync_instances(
            &mut sim,
            &r,
            &bridge,
            &mut echo,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
        );
        let loose_rim = rim_of(&sim, loose);
        let names: Vec<String> = crate::instance_apply_deep::apply_levels(&mut sim, loose_rim)
            .into_iter()
            .map(|l| l.name)
            .collect();
        assert_eq!(
            names,
            ["Wheel"],
            "a roda solta nao pode estar dentro de um Carro"
        );
        assert_eq!(cars.len(), 2, "a cena promete DOIS carros");
    }
}
