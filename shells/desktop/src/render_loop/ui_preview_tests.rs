//! Os gates do MODO DE PREVIEW — *entrar captura, sair devolve o mundo AO BIT, e o rato só
//! dirige aqui dentro*.

use super::*;
use ph2d_ecs::{Name, Transform};
use ph2d_ui_state::{StateSets, UiState};
use ph2d_vec_scene::{VecPath, rectangle};

const HOST: VecPathId = 1;
const CHILD: VecPathId = 2;

/// Um mundo mínimo: um hospedeiro com dois estados, e uma entidade por caminho.
fn world() -> (SimWorld, VecScene, VecEntityMap, StateSets) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::default();
    let mut map = VecEntityMap::default();

    for (id, x) in [(HOST, 0.0f32), (CHILD, 5.0)] {
        let mut p: VecPath = rectangle([0.0, 0.0], [1.0, 1.0]);
        p.id = id;
        scene.push_path(p);
        let mut t = Transform::IDENTITY;
        t.translation.x = x;
        let e = sim
            .world_mut()
            .spawn((Name(format!("p{id}")), t, ph2d_ecs::VecPathRef(id)))
            .id();
        map.insert(id, e.to_bits());
    }

    let mut states = StateSets::default();
    for (role, x) in [(StateRole::Default, 0.0), (StateRole::Hover, 40.0)] {
        let mut st = UiState::new(role);
        st.objects = vec![ObjectPose {
            translation: [x, 0.0],
            ..ObjectPose::new(HOST)
        }];
        states.set(HOST, st);
    }
    (sim, scene, map, states)
}

fn x_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> f64 {
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    f64::from(sim.world().get::<Transform>(e).unwrap().translation.x)
}

/// ⭐ **SAIR devolve o mundo ao que era, e NÃO ao estado Default.**
///
/// ⚠️ É a lei inteira da wave. A tentação barata é *"ao sair, vá para o Default"* — e ela **moveria
/// o desenho** de quem gravou o Default e depois moveu a forma. A fixture faz exactamente isso: o
/// Default está gravado em `x = 0` e o mundo está em `x = 7`, dois números que não podem coincidir
/// por acidente.
#[test]
fn leaving_restores_the_world_it_found_not_the_default_state() {
    let (mut sim, mut scene, map, states) = world();
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();

    // O artista moveu a forma DEPOIS de gravar o Default.
    let e = ph2d_ecs::Entity::from_bits(map[&HOST]);
    sim.world_mut()
        .get_mut::<Transform>(e)
        .unwrap()
        .translation
        .x = 7.0;

    assert!(pv.enter(&states, &sim, &scene, &map));
    pv.point(&mut machines, &states, &[HOST], false);
    machines.get_mut(&HOST).unwrap().advance(10.0);
    for p in machines[&HOST].pose() {
        crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, p);
    }
    assert!(
        (x_of(&sim, &map, HOST) - 40.0).abs() < 1e-9,
        "a fixture nao chegou ao Hover: x = {}",
        x_of(&sim, &map, HOST)
    );

    assert!(pv.leave(&mut machines, &mut sim, &mut scene, &map));
    assert!(
        (x_of(&sim, &map, HOST) - 7.0).abs() < 1e-9,
        "sair devolveu {} — o Default gravado e' 0 e o mundo era 7; ir para o Default MOVE o \
         desenho de quem gravou e depois mexeu",
        x_of(&sim, &map, HOST)
    );
}

/// **O conjunto capturado é EXACTAMENTE o que a preview pode escrever.**
///
/// ⚠️ A `Machine` só emite ids que aparecem nos estados autorados, então capturar a união deles é
/// completo **por construção**. Este gate mede a afirmação em vez de a repetir: um id que a
/// preview escreva e que não esteja na captura fica para trás no `leave`, e o documento muda por
/// o artista ter olhado.
#[test]
fn the_snapshot_covers_every_id_the_preview_can_write() {
    let (sim, scene, map, states) = world();
    let snap = touched(&states);
    assert_eq!(snap, vec![HOST], "so' o HOST tem pose autorada");

    // Todo id que qualquer estado menciona tem de estar na captura.
    for h in states.hosts() {
        for st in states.get(h) {
            for o in &st.objects {
                assert!(snap.contains(&o.id), "o id {} ficou de fora", o.id);
            }
        }
    }
    // E a captura le' o MUNDO, nao a tabela: o CHILD nao entra porque nenhum estado o menciona.
    let _ = (sim, scene, map);
    assert!(!snap.contains(&CHILD));
}

/// **A preview NÃO liga sobre uma cena sem poses.**
///
/// ⚠️ Um modo de preview que não faz nada é indistinguível de um botão quebrado — e o artista não
/// teria como saber que o que falta é gravar um estado.
#[test]
fn the_preview_refuses_to_open_on_a_scene_with_no_states() {
    let (sim, scene, map, _) = world();
    let mut pv = UiPreview::default();
    assert!(!pv.enter(&StateSets::default(), &sim, &scene, &map));
    assert!(!pv.is_on());
}

/// **Sair de um botão para outro apaga o primeiro no MESMO passo.**
///
/// ⚠️ Um gate de um botão só nunca mostra isto: com um hospedeiro, *"o que sai volta ao Default"*
/// e *"nada mais acontece"* dão a mesma resposta.
#[test]
fn moving_from_one_host_to_another_returns_the_first_to_default() {
    let (mut sim, mut scene, map, mut states) = world();
    // Um SEGUNDO hospedeiro, com as mesmas duas poses.
    for (role, x) in [(StateRole::Default, 0.0), (StateRole::Hover, 40.0)] {
        let mut st = UiState::new(role);
        st.objects = vec![ObjectPose {
            translation: [x, 0.0],
            ..ObjectPose::new(CHILD)
        }];
        states.set(CHILD, st);
    }
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    assert!(pv.enter(&states, &sim, &scene, &map));

    pv.point(&mut machines, &states, &[HOST], false);
    machines.get_mut(&HOST).unwrap().advance(10.0);
    pv.point(&mut machines, &states, &[CHILD], false);
    for m in machines.values_mut() {
        m.advance(10.0);
    }
    for m in machines.values() {
        for p in m.pose() {
            crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, p);
        }
    }
    assert!(
        x_of(&sim, &map, HOST).abs() < 1e-9,
        "o hospedeiro que se deixou ficou aceso em x = {}",
        x_of(&sim, &map, HOST)
    );
    assert!(
        (x_of(&sim, &map, CHILD) - 40.0).abs() < 1e-9,
        "o hospedeiro novo nao acendeu"
    );
}

/// **Os dois fatos do rato derivam os três papéis** — e apertar no VAZIO não prende ninguém.
#[test]
fn the_two_mouse_facts_derive_the_role() {
    let (sim, scene, map, states) = world();
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    pv.enter(&states, &sim, &scene, &map);

    pv.point(&mut machines, &states, &[HOST], false);
    assert_eq!(pv.role_for(HOST), StateRole::Hover);
    pv.point(&mut machines, &states, &[HOST], true);
    assert_eq!(pv.role_for(HOST), StateRole::Pressed);
    pv.point(&mut machines, &states, &[], true);
    assert_eq!(
        pv.role_for(HOST),
        StateRole::Default,
        "apertar no vazio nao pode prender um hospedeiro"
    );
}

/// **Com a preview DESLIGADA o rato não dirige nada** — é o interruptor inteiro num gate.
#[test]
fn the_mouse_drives_nothing_while_the_preview_is_off() {
    let (_, _, _, states) = world();
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    pv.point(&mut machines, &states, &[HOST], false);
    assert!(
        machines.is_empty(),
        "o rato criou uma maquina fora do modo de preview"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// A HIERARQUIA (W7h) — um hospedeiro DENTRO de outro.
// ─────────────────────────────────────────────────────────────────────────────

const MENU: VecPathId = 10;
const ITEM: VecPathId = 11;

/// Um menu que CONTÉM um item, os dois hospedeiros — a cena que o §10 do handoff da W7c nomeia
/// (*"um menu que abre com sub-estados"*).
///
/// ⚠️ O aninhamento é ECS de verdade (`ChildOf`), porque é assim que `members` decide quem
/// pertence a quem — uma fixture que só pusesse os dois no mapa não conteria o fenômeno.
fn nested() -> (SimWorld, VecScene, VecEntityMap, StateSets) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::default();
    let mut map = VecEntityMap::default();

    let mut ents = Vec::new();
    for id in [MENU, ITEM] {
        // ⚠️ `push_path` **reescreve o id** (ele é quem os cunha), então declarar `p.id` aqui não
        // basta: a fixture da wave anterior fazia isso e os ids que ela nomeava nunca chegavam à
        // cena — `members` devolvia vazio e o aninhamento não existia.
        let made = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        if let Some(p) = scene.path_mut(made) {
            p.id = id;
        }
        let e = sim
            .world_mut()
            .spawn((
                Name(format!("p{id}")),
                Transform::IDENTITY,
                ph2d_ecs::VecPathRef(id),
            ))
            .id();
        map.insert(id, e.to_bits());
        ents.push(e);
    }
    sim.world_mut()
        .entity_mut(ents[1])
        .insert(ph2d_ecs::ChildOf(ents[0]));

    let mut states = StateSets::default();
    for (host, x) in [(MENU, 40.0), (ITEM, 70.0)] {
        for (role, tx) in [(StateRole::Default, 0.0), (StateRole::Hover, x)] {
            let mut st = UiState::new(role);
            st.objects = vec![ObjectPose {
                translation: [tx, 0.0],
                ..ObjectPose::new(host)
            }];
            states.set(host, st);
        }
    }
    (sim, scene, map, states)
}

/// Instala tudo o que as máquinas têm a dizer, no fim das transições.
fn settle(machines: &mut UiMachines, sim: &mut SimWorld, scene: &mut VecScene, map: &VecEntityMap) {
    for m in machines.values_mut() {
        m.advance(10.0);
    }
    for m in machines.values() {
        for p in m.pose() {
            crate::vec_ui_state_edit::install(sim, scene, map, p);
        }
    }
}

/// ⭐ **O menu NÃO fecha quando o cursor entra num item dele.**
///
/// ⚠️ É a lei inteira da hierarquia, e o defeito que ela pina vê-se a olho: `point` mandava o
/// hospedeiro que se DEIXA para o `Default`, e um ANCESTRAL do novo *hot* contava como deixado.
/// O menu fechava com o cursor dentro dele.
#[test]
fn an_ancestor_stays_lit_while_the_cursor_is_in_its_descendant() {
    let (mut sim, mut scene, map, states) = nested();
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    assert!(pv.enter(&states, &sim, &scene, &map));

    // O cursor entra no MENU (só o fundo dele é tocado), e depois desce para o ITEM.
    pv.point(&mut machines, &states, &[MENU], false);
    settle(&mut machines, &mut sim, &mut scene, &map);
    pv.point(&mut machines, &states, &[ITEM, MENU], false);
    settle(&mut machines, &mut sim, &mut scene, &map);

    assert!(
        (x_of(&sim, &map, ITEM) - 70.0).abs() < 1e-9,
        "o item sob o cursor nao acendeu (x = {})",
        x_of(&sim, &map, ITEM)
    );
    assert!(
        (x_of(&sim, &map, MENU) - 40.0).abs() < 1e-9,
        "o MENU fechou com o cursor DENTRO dele (x = {}) — um ancestral do hot foi tratado \
         como deixado",
        x_of(&sim, &map, MENU)
    );
}

/// **E sair da árvore inteira apaga os dois** — sem esta metade o ancestral ficaria aceso para
/// sempre, e um menu que nunca fecha é pior que um que fecha cedo.
#[test]
fn leaving_the_whole_tree_returns_every_ancestor_to_default() {
    let (mut sim, mut scene, map, states) = nested();
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    assert!(pv.enter(&states, &sim, &scene, &map));

    pv.point(&mut machines, &states, &[MENU], false);
    pv.point(&mut machines, &states, &[ITEM, MENU], false);
    pv.point(&mut machines, &states, &[], false);
    settle(&mut machines, &mut sim, &mut scene, &map);

    assert!(x_of(&sim, &map, ITEM).abs() < 1e-9, "o item ficou aceso");
    assert!(
        x_of(&sim, &map, MENU).abs() < 1e-9,
        "sair da arvore inteira tem de apagar o ancestral tambem (x = {})",
        x_of(&sim, &map, MENU)
    );
}

/// ⭐ **O INTERIOR ganha, e não o menor `VecPathId`.**
///
/// ⚠️ O `host_under` antigo devolvia UM hospedeiro varrendo `states.hosts()` — um `BTreeMap` —,
/// então com dois hospedeiros aninhados o vencedor era decidido por **qual id era menor**, e não
/// por qual era o mais interno. O doc dizia *"o de cima é o que o artista vê"*, o que é verdade
/// entre PICKS e falso entre HOSPEDEIROS para um pick só.
#[test]
fn the_innermost_host_wins_not_the_smaller_id() {
    let (sim, scene, map, states) = nested();
    // O pick é o ITEM: ele pertence à sub-árvore do MENU **e** à própria.
    let chain = host_under(&sim, &scene, &map, &states, &[ITEM]);
    assert_eq!(
        chain.first().copied(),
        Some(ITEM),
        "o hospedeiro mais INTERNO tem de vir primeiro — a cadeia veio {chain:?}"
    );
    assert!(
        chain.contains(&MENU),
        "o ancestral tem de estar na cadeia, senao `point` nao tem como o poupar"
    );
}

/// ⭐ **O ancestral que fica aceso NÃO é re-animado** — a segunda camada, e ela é sobre CUSTO.
///
/// ⚠️ Ela precisa de gate PRÓPRIO porque a pose visível **não** a denuncia: pedir `Default` e
/// logo `Hover` no mesmo quadro deixa o menu em `Hover` na mesma, e o gate de comportamento
/// acima fica verde [[feedback_layered_defenses_need_per_layer_gates]]. O que se perde é
/// trabalho: `Machine::go_to` constrói uma [`ph2d_ui_state::Transition`] a cada chamada, e o
/// doc daquela crate mede o casamento em **0,64 ms por par com geometria** — 20 objetos numa
/// troca seriam 12,79 ms, 77% de um quadro, para não mover um vértice.
///
/// ⇒ o filtro de *quem se deixa* tem de olhar a CADEIA inteira, e não só o hospedeiro anterior.
#[test]
fn an_ancestor_that_stays_lit_is_not_re_animated() {
    let (sim, scene, map, states) = nested();
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    assert!(pv.enter(&states, &sim, &scene, &map));

    pv.point(&mut machines, &states, &[MENU], false);
    for m in machines.values_mut() {
        m.advance(10.0);
    }
    assert!(
        !machines[&MENU].is_animating(),
        "a fixture nao contem o fenomeno: o menu tinha de estar ASSENTE em Hover"
    );

    pv.point(&mut machines, &states, &[ITEM, MENU], false);
    assert!(
        !machines[&MENU].is_animating(),
        "o ancestral foi re-animado — ele recebeu um pedido de Default que o pedido seguinte \
         desfez, e cada um desses constroi uma Transition"
    );
    assert!(
        machines[&ITEM].is_animating(),
        "o item sob o cursor tinha de estar a animar — sem isso o gate acima e' vacuo"
    );
}
