//! Os gates dos verbos do menu de um cartão ([`super`]).

use super::users_of;
use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, SpritePixels, StableId, Transform};
use ph2d_editor::interaction::drag_payload::DragPayload;
use std::collections::BTreeMap;

/// ⭐⭐⭐ **Uma PEÇA DE RECEITA não é um utilizador** (auditoria de 2026-08-30, achado nº 1).
///
/// ⛔ A varredura crua contava-a — as peças de uma receita carregam `SpritePixels` como qualquer
/// outra — e elas **não estão na cena**: o extract salta-as e a Hierarquia não lhes dá linha.
/// Consequência medida: com uma imagem usada só por uma receita, o *Select users* dizia
/// *«Selected 1 object(s)»* e **nada acendia**. É a espécie *«o consumidor que PROJECTA o valor
/// fora»* do §5.0 — o fio está completo e quem o lê descarta.
///
/// ⚠️ **A fixtura tem o fenómeno de propósito:** a MESMA textura usada por uma peça de receita e
/// por um objecto da cena. Sem o objecto da cena, um filtro que devolvesse sempre vazio passaria.
///
/// **Mutação que deve sangrar:** apagar o `.filter(|e| !is_unedited_recipe(..))`.
#[test]
fn a_recipe_piece_is_not_a_user_because_the_artist_cannot_reach_it() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);

    // A receita, com uma peça que usa a textura.
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Recipe"), MasterRoot))
        .id();
    let hidden_piece = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("RecipePiece"),
            SpritePixels(tex),
            ChildOf(master),
        ))
        .id();
    // E um objecto da CENA que usa a mesma textura.
    let on_canvas = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("OnCanvas"),
            SpritePixels(tex),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // ⚠️ A marca é DERIVADA — sem este passe a peça da receita não é peça de nada, e o gate mediria
    // um mundo que o produto nunca tem.
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
        // ⚠️ Mapa VAZIO: estes dois gates medem o caminho do `SpritePixels`, e um mapa cheio faria
        // a resposta certa sair pelo outro braço.
        &BTreeMap::new(),
    );
    assert!(
        users.contains(&on_canvas.to_bits()),
        "o objecto da cena TEM de contar"
    );
    assert!(
        !users.contains(&hidden_piece.to_bits()),
        "a peça da receita não pode contar — o artista não a vê nem a alcança"
    );
    assert_eq!(users.len(), 1, "e mais ninguém: {users:?}");
}

/// ⚠️ **Uma entidade nascida no mesmo quadro ainda não tem `StableId`** — o
/// `assign_missing_stable_ids` corre uma vez por quadro. Exigi-lo na consulta fá-la-ia desaparecer
/// da resposta **em silêncio**, que é o defeito que o `map_or(u64::MAX, …)` evita.
///
/// **Mutação que deve sangrar:** voltar a `query::<(Entity, &SpritePixels, &StableId)>()`.
#[test]
fn an_object_born_this_frame_still_counts_as_a_user() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);
    let with_id = sim
        .world_mut()
        .spawn((Transform::IDENTITY, SpritePixels(tex)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(sim.world().get::<StableId>(with_id).is_some());
    // Este nasce DEPOIS do passe do quadro — é o caso que morde.
    let newborn = sim
        .world_mut()
        .spawn((Transform::IDENTITY, SpritePixels(tex)))
        .id();
    assert!(sim.world().get::<StableId>(newborn).is_none());

    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
        // ⚠️ Mapa VAZIO: estes dois gates medem o caminho do `SpritePixels`, e um mapa cheio faria
        // a resposta certa sair pelo outro braço.
        &BTreeMap::new(),
    );
    assert!(
        users.contains(&newborn.to_bits()),
        "o recém-nascido tem de contar: {users:?}"
    );
    assert_eq!(users.len(), 2);
}

/// ⭐⭐⭐ **UMA SPRITE DE ÁTLAS conta como utilizador** — e a 1.ª versão não a via.
///
/// ⛔ O `Select users` perguntava só pelo `SpritePixels`, que é o carimbo da MINORIA: toda imagem
/// importada e todo canvas novo vivem no **átlas**. Consequência: *«Nothing is using this image»*
/// sobre uma imagem que o artista acabou de pôr na cena.
///
/// **Mutação que deve sangrar:** voltar a `query::<(Entity, &SpritePixels)>()`.
#[test]
fn an_atlas_sprite_counts_as_a_user_of_the_image_it_shows() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Imported"),
            ph2d_render::Sprite::atlas(7, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ))
        .id();
    let mut atlas = BTreeMap::new();
    atlas.insert(7u32, tex);
    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
        &atlas,
    );
    assert_eq!(users, vec![e.to_bits()], "a sprite de átlas TEM de contar");
}

/// ⭐⭐⭐ **EDITAR um componente pela biblioteca põe a SELECÇÃO na receita** (report do Enio,
/// 2026-09-05: *«não tem como editar o componente»*).
///
/// ⚠️ **É a selecção que faz a receita existir na tela** — ela não está na cena e volta enquanto
/// está seleccionada (a marca derivada `MasterEditing`). ⇒ este gate mede o **único** efeito que o
/// verbo tem, e é ele que o liga a toda a maquinaria que já existia.
///
/// **Mutação que deve sangrar:** o braço `EditPrefab` não escrever o `select_out` (o item passa a
/// comer o clique e a dizer que editou, sem nada acontecer — a 1.ª espécie de controlo morto).
#[test]
fn editing_a_prefab_from_the_library_selects_the_recipe() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let mut echo = crate::instance_sync::MasterEcho::default();
    let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };

    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let stable_id = sim.world().get::<StableId>(master).expect("id").0;

    let mut select_out = None;
    let acted = super::drain(
        DragPayload::Prefab { stable_id },
        ph2d_editor::action_bus::AssetCardAction::EditPrefab,
        &mut sim,
        &r,
        &mut echo,
        &mut gizmo,
        &mut toasts,
        &mut docs,
        [0.0, 0.0],
        &BTreeMap::new(),
        &mut select_out,
    );
    assert!(acted, "o verbo nao agiu");
    assert_eq!(
        select_out,
        Some(master.to_bits()),
        "a seleccao nao foi para a receita — o item come o clique e nada aparece"
    );
}

/// ⛔ **Uma IMAGEM recusa com o FACTO, e a recusa FALA** — a tabela do menu é plana, e um item que
/// come o clique em silêncio é pior que um ausente (a lei das outras três recusas deste ficheiro).
#[test]
fn editing_an_image_is_refused_out_loud() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let mut echo = crate::instance_sync::MasterEcho::default();
    let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };

    let mut select_out = None;
    let acted = super::drain(
        DragPayload::Image { asset: [7; 32] },
        ph2d_editor::action_bus::AssetCardAction::EditPrefab,
        &mut sim,
        &r,
        &mut echo,
        &mut gizmo,
        &mut toasts,
        &mut docs,
        [0.0, 0.0],
        &BTreeMap::new(),
        &mut select_out,
    );
    assert!(!acted);
    assert!(
        select_out.is_none(),
        "uma imagem nao tem receita a seleccionar"
    );
    assert!(
        !toasts.is_empty(),
        "a recusa foi MUDA — o artista carrega e conclui que o app esta' partido"
    );
}

// ── A TROCA por um componente sem parentesco (plano F5, o último critério) ───────────────────
//
// ⚠️ **Estes gates atravessam a shell**, e não o construtor do mapa: o mecanismo tem gates próprios
// em `instance_swap_match_tests.rs`. O que se mede aqui é a **terceira pergunta** do §5.0 — *o
// leitor decide, ou entrega a alguém que descarta?* — sobre o único verbo deste menu cujo sujeito
// não é o cartão.

/// Duas receitas sem antepassado comum e uma cópia da primeira na cena.
fn two_recipes_and_a_copy(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
) -> (u64, ph2d_ecs::Entity) {
    let mut make = |name: &str| {
        let root = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new(name), MasterRoot))
            .id();
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new("Body"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(root),
        ));
        root
    };
    let car = make("Car");
    let truck = make("Truck");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let copy = crate::instantiate::instantiate_master(
        sim,
        r,
        car,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    (sim.world().get::<StableId>(truck).expect("id").0, copy)
}

/// Corre o verbo do cartão com a selecção dada, e devolve `(agiu, o que ele DISSE)`.
///
/// ⚠️ **A voz vem inteira, e não como `!toasts.is_empty()`.** Cada caminho vazio deste verbo diz
/// uma coisa diferente e as três são accionáveis; um gate que só conta toasts deixa colapsá-las
/// numa só sem sangrar — e a que sobreviveria seria a que não diz o que fazer.
fn run_replace(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    verb: ph2d_editor::action_bus::AssetCardAction,
    asset: DragPayload,
    selected: Option<u64>,
) -> (bool, String) {
    let mut echo = crate::instance_sync::MasterEcho::default();
    let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
    gizmo.replace_selection(selected);
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let mut select_out = None;
    let acted = super::drain(
        asset,
        verb,
        sim,
        r,
        &mut echo,
        &mut gizmo,
        &mut toasts,
        &mut docs,
        [0.0, 0.0],
        &BTreeMap::new(),
        &mut select_out,
    );
    let said: Vec<&str> = toasts.iter().map(|t| t.message.as_str()).collect();
    (acted, said.join(" | "))
}

fn master_of(sim: &SimWorld, e: ph2d_ecs::Entity) -> u64 {
    sim.world()
        .get::<ph2d_ecs::InstanceOf>(e)
        .expect("a copia tem elo")
        .master
}

/// ⭐⭐⭐ **O clique no item chega ao MUNDO** — a cópia passa a ser do outro componente.
///
/// ⚠️ Um seam de painel prova que o clique chega ao **barramento**; ele não prova que alguém do
/// outro lado o lê. É a espécie *«o dreno de UM BRAÇO SÓ»* do §5.0, e o `match` do
/// [`super::drain`] não termina em `_ => {}` só porque o compilador o proíbe — o do barramento
/// termina, e é por isso que o censo textual irmão existe.
///
/// **Mutação que deve sangrar:** o braço a devolver `false` sem chamar o `swap`.
#[test]
fn replacing_the_selection_makes_the_copy_belong_to_the_other_component() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (truck_id, copy) = two_recipes_and_a_copy(&mut sim, &r);
    assert_ne!(
        master_of(&sim, copy),
        truck_id,
        "a fixtura ja' comecava la'"
    );

    let (acted, spoke) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelectionByName,
        DragPayload::Prefab {
            stable_id: truck_id,
        },
        Some(copy.to_bits()),
    );
    assert!(acted, "o verbo nao mexeu no documento");
    assert!(
        spoke.contains("Truck") && spoke.contains("Replaced"),
        "a voz nao diz o que aconteceu nem a quem: {spoke:?}"
    );
    assert_eq!(
        master_of(&sim, copy),
        truck_id,
        "o elo da copia nao mudou — o clique morreu a um passo do efeito"
    );
}

/// ⭐⭐ **O sujeito é a RAIZ da cópia, e não a entidade clicada.**
///
/// ⚠️ O artista escolhe uma PEÇA de dentro da cópia tantas vezes quantas escolhe a raiz — clicar no
/// canvas dá a peça que está debaixo do rato. Exigir a raiz faria o gesto falhar sem dizer porquê,
/// que é o modo de falha mais caro deste menu.
///
/// **Mutação que deve sangrar:** trocar o `instance_root_of` pelos bits escolhidos.
#[test]
fn picking_a_piece_inside_the_copy_replaces_the_whole_copy() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (truck_id, copy) = two_recipes_and_a_copy(&mut sim, &r);
    let piece = sim
        .world()
        .get::<ph2d_ecs::Children>(copy)
        .expect("a copia tem pecas")
        .iter()
        .copied()
        .next()
        .expect("uma peca");
    assert_ne!(piece, copy, "a fixtura tinha de dar uma PECA");

    let (acted, _) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelection,
        DragPayload::Prefab {
            stable_id: truck_id,
        },
        Some(piece.to_bits()),
    );
    assert!(acted);
    assert_eq!(
        master_of(&sim, copy),
        truck_id,
        "escolher uma peca de dentro nao alcancou a copia"
    );
}

/// ⛔ **Sem nada escolhido a recusa DIZ o que fazer** — o sujeito deste verbo não está no cartão,
/// então «não aconteceu nada» seria a leitura de um item partido.
#[test]
fn replacing_with_nothing_picked_says_what_to_pick() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (truck_id, _copy) = two_recipes_and_a_copy(&mut sim, &r);
    let (acted, spoke) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelection,
        DragPayload::Prefab {
            stable_id: truck_id,
        },
        None,
    );
    assert!(!acted, "sem sujeito nao ha' edicao");
    assert!(
        spoke.contains("Pick the copy"),
        "a recusa nao diz O QUE FAZER — «nada aconteceu» le-se como item partido: {spoke:?}"
    );
}

/// ⛔ **Uma imagem não é um componente**, e a quarta recusa da tabela plana também fala.
#[test]
fn an_image_cannot_replace_a_copy_and_it_says_so() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (_truck_id, copy) = two_recipes_and_a_copy(&mut sim, &r);
    let before = master_of(&sim, copy);
    let (acted, spoke) = run_replace(
        &mut sim,
        &r,
        ph2d_editor::action_bus::AssetCardAction::ReplaceSelectionByTree,
        DragPayload::Image { asset: [3; 32] },
        Some(copy.to_bits()),
    );
    assert!(!acted);
    assert!(
        spoke.contains("not a component"),
        "a recusa nao nomeia o FACTO: {spoke:?}"
    );
    assert_eq!(master_of(&sim, copy), before, "a copia mexeu-se");
}

/// ⭐⭐⭐ **Os três itens são TRÊS LEIS, e o modo sai do VERBO.**
///
/// ⚠️ O `match verb` do [`super::replace_selection`] termina em `_ =>`, então um modo novo cairia no
/// *«não leves nada»* **em silêncio** — a espécie *«o dreno de um braço só»* do §5.0, aqui dentro de
/// uma função em vez de num barramento. Este gate mede as três células de uma vez: com a ordem dos
/// irmãos TROCADA entre as duas receitas, cada modo re-chaveia a excepção para uma peça diferente.
///
/// **Mutação que deve sangrar:** colapsar dois braços do `match verb`.
#[test]
fn the_three_replace_items_are_three_different_laws() {
    use ph2d_editor::action_bus::AssetCardAction as A;

    let kid = |sim: &SimWorld, root: ph2d_ecs::Entity, name: &str| -> u64 {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
                return sim.world().get::<StableId>(e).expect("id").0;
            }
            if let Some(k) = sim.world().get::<ph2d_ecs::Children>(e) {
                stack.extend(k.iter().copied());
            }
        }
        panic!("nao achei {name:?}");
    };

    for (verb, want) in [
        (A::ReplaceSelectionByName, "Body"),
        (A::ReplaceSelectionByTree, "Wheel"),
    ] {
        let mut sim = SimWorld::new();
        let r = crate::init::build_component_registry();
        // ⚠️ A ordem dos irmãos ao CONTRÁRIO — o único arranjo em que os dois modos discordam.
        let mut make = |name: &str, pieces: [&str; 2]| {
            let root = sim
                .world_mut()
                .spawn((Transform::IDENTITY, Name::new(name), MasterRoot))
                .id();
            for p in pieces {
                sim.world_mut().spawn((
                    Transform::IDENTITY,
                    Name::new(p),
                    ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
                    ChildOf(root),
                ));
            }
            root
        };
        let car = make("Car", ["Body", "Wheel"]);
        let truck = make("Truck", ["Wheel", "Body"]);
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        let copy = crate::instantiate::instantiate_master(
            &mut sim,
            &r,
            car,
            None,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
            crate::instantiate::ArtLink::Own,
        )
        .expect("instanciou");
        // A excepção do artista, escrita à mão na chave: é ela que o re-key move.
        let car_body = kid(&sim, car, "Body");
        let truck_id = sim.world().get::<StableId>(truck).expect("id").0;
        let type_id = 7u64;
        sim.world_mut()
            .entity_mut(copy)
            .insert(ph2d_ecs::ObjectInstance {
                overrides: std::collections::BTreeSet::from([ph2d_ecs::OverrideKey {
                    piece: car_body,
                    type_id,
                }]),
                ..Default::default()
            });

        let (acted, _) = run_replace(
            &mut sim,
            &r,
            verb,
            DragPayload::Prefab {
                stable_id: truck_id,
            },
            Some(copy.to_bits()),
        );
        assert!(acted, "{verb:?} nao agiu");
        let landed = sim
            .world()
            .get::<ph2d_ecs::ObjectInstance>(copy)
            .expect("instancia")
            .overrides
            .iter()
            .map(|k| k.piece)
            .collect::<Vec<_>>();
        assert_eq!(
            landed,
            vec![kid(&sim, truck, want)],
            "{verb:?} nao levou a excepcao para a peca {want:?} — os modos colapsaram"
        );
    }
}
