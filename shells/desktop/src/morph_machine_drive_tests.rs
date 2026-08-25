//! Os gates da máquina a correr — mundo, mapa e relógio, sem janela nenhuma.

use super::{MorphMachines, tick};
use ph2d_ecs::{SimWorld, VecMorph, VecMorphMachine};
use ph2d_input::{ActionState, Binding, InputMap, InputState, Key};
use ph2d_morph_machine::{MorphEdge, MorphGraph};

use crate::preview_drive::PreviewDrive;

const A: u64 = 10;
const B: u64 = 20;
const KEY_Z: u32 = 0x5A;

/// Um mundo com um Morph `A -> B` disparado pela acção `jump`, e o mapa que a liga ao `Z`.
fn scene() -> (SimWorld, ph2d_ecs::Entity, InputMap) {
    let mut sim = SimWorld::new();
    let mut m = VecMorphMachine::new(A);
    let mut e = MorphEdge::new(A, B);
    e.when = "jump".to_string();
    e.duration_s = 0.1;
    m.graph = MorphGraph {
        start: A,
        edges: vec![e],
    };
    // ⚠️ O par nasce `(A, A)`: a máquina ainda não voou, e é isso que o `pair()` dela diz.
    let ent = sim.world_mut().spawn((VecMorph::new(A, A), m)).id();

    let mut map = InputMap::new();
    let id = map.create("jump");
    map.get_mut(id)
        .unwrap()
        .bindings
        .push(Binding::Key(Key(KEY_Z)));
    (sim, ent, map)
}

/// Um `ActionState` com o `Z` carregado NESTE tique (e solto no anterior) — é isso que faz o
/// `just_pressed` responder.
fn z_just_pressed(map: &InputMap) -> ActionState {
    let mut st = ActionState::new();
    let mut dev = InputState::new();
    st.tick(map, &dev); // o tique de ANTES: a tecla ainda nao foi carregada
    dev.keyboard.handle_key_down(Key(KEY_Z));
    st.tick(map, &dev);
    st
}

/// ⭐⭐ **A TECLA MORFA A FORMA** — o caminho inteiro, do mapa ao componente.
///
/// **Mutação que deve sangrar:** o `fire` nunca ser chamado — a máquina fica parada e a tecla não
/// faz nada, que é a feature inteira.
#[test]
fn the_bound_key_moves_the_morph_from_one_shape_to_the_other() {
    let (mut sim, e, map) = scene();
    let st = z_just_pressed(&map);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();

    let ran = tick(
        &mut machines,
        &mut sim,
        &map,
        &st,
        true,
        1.0 / 60.0,
        &mut drive,
    );
    assert_eq!(ran, 1, "a maquina tem de correr");
    assert_eq!(
        sim.world().get::<VecMorph>(e).unwrap().sources,
        [A, B],
        "o par tem de ser o da seta que disparou"
    );
    // Andar ate' ao fim: a forma chega em B.
    let quiet = ActionState::new();
    for _ in 0..30 {
        tick(
            &mut machines,
            &mut sim,
            &map,
            &quiet,
            true,
            1.0 / 60.0,
            &mut drive,
        );
    }
    let m = sim.world().get::<VecMorph>(e).unwrap();
    assert_eq!(
        (m.sources, m.t),
        ([A, B], 1.0),
        "chegou, e o par NAO trocou"
    );
}

/// ⛔⛔ **COM O RELÓGIO PARADO a máquina não corre, e a tecla não faz NADA.**
///
/// ⚠️ Não é conservadorismo: a condição de uma seta é uma tecla, e a escutar durante a edição
/// carregar em `Z` morfava a forma **e** fazia o que o `Z` faz no editor — os dois, sem que nada na
/// tela explicasse.
///
/// **Mutação que deve sangrar:** largar a guarda do `playing`.
#[test]
fn with_the_clock_stopped_the_key_does_nothing() {
    let (mut sim, e, map) = scene();
    let st = z_just_pressed(&map);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();

    let ran = tick(
        &mut machines,
        &mut sim,
        &map,
        &st,
        false,
        1.0 / 60.0,
        &mut drive,
    );
    assert_eq!(ran, 0);
    assert_eq!(
        sim.world().get::<VecMorph>(e).unwrap().sources,
        [A, A],
        "o par autorado tem de ficar INTACTO"
    );
    assert!(machines.is_empty(), "e as maquinas sao LARGADAS");
}

/// ⭐⭐ **O QUE A MÁQUINA ESCREVE NÃO ENTRA NO UNDO** — e são os DOIS campos.
///
/// ⛔ **O `Driver::MorphT` sozinho não bastava:** ele cobre o `t` e **só** o `t`. Sem o
/// `MorphPair`, trocar de par durante a reprodução entraria no undo como se o artista tivesse
/// re-ligado as fontes à mão.
///
/// **Mutação que deve sangrar:** não registar o par no ledger.
#[test]
fn both_fields_the_machine_writes_are_preview_and_not_document() {
    let (mut sim, e, map) = scene();
    let st = z_just_pressed(&map);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    tick(
        &mut machines,
        &mut sim,
        &map,
        &st,
        true,
        1.0 / 60.0,
        &mut drive,
    );

    // A captura repõe o AUTORADO — é o que a fotografia do undo vê.
    let live = drive.substitute_authored(&mut sim);
    let m = sim.world().get::<VecMorph>(e).unwrap();
    assert_eq!(
        (m.sources, m.t),
        ([A, A], 0.5),
        "durante a fotografia o mundo tem de mostrar o que o ARTISTA desenhou"
    );
    // …e a cena volta a mostrar o que o motor escreveu.
    PreviewDrive::restore_live(&mut sim, &live);
    assert_eq!(
        sim.world().get::<VecMorph>(e).unwrap().sources,
        [A, B],
        "e depois da fotografia a cena volta ao que o motor mostrava"
    );
}

/// ⚠️ **Uma tecla SEGURADA não re-dispara.** Com `pressed` em vez de `just_pressed`, a máquina
/// saltaria a cadeia inteira num piscar de olhos.
///
/// **Mutação que deve sangrar:** trocar `just_pressed` por `pressed`.
#[test]
fn a_held_key_fires_once_and_not_every_frame() {
    let mut sim = SimWorld::new();
    let mut m = VecMorphMachine::new(A);
    const C: u64 = 30;
    let mut e1 = MorphEdge::new(A, B);
    e1.when = "jump".to_string();
    e1.duration_s = 0.0; // instantanea: sem isto a 2a seta nunca teria hipotese de disparar
    let mut e2 = MorphEdge::new(B, C);
    e2.when = "jump".to_string();
    e2.duration_s = 0.0;
    m.graph = MorphGraph {
        start: A,
        edges: vec![e1, e2],
    };
    let ent = sim.world_mut().spawn((VecMorph::new(A, A), m)).id();
    let mut map = InputMap::new();
    let id = map.create("jump");
    map.get_mut(id)
        .unwrap()
        .bindings
        .push(Binding::Key(Key(KEY_Z)));

    let mut st = ActionState::new();
    let mut dev = InputState::new();
    dev.keyboard.handle_key_down(Key(KEY_Z));
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    // Dez quadros com a tecla SEGURADA o tempo todo.
    for _ in 0..10 {
        st.tick(&map, &dev);
        tick(
            &mut machines,
            &mut sim,
            &map,
            &st,
            true,
            1.0 / 60.0,
            &mut drive,
        );
    }
    assert_eq!(
        sim.world().get::<VecMorph>(ent).unwrap().sources,
        [A, B],
        "a tecla segurada disparou a cadeia inteira -- ela tem de disparar UMA vez"
    );
}

/// **Uma máquina cuja entidade morreu some junto** — senão o mapa cresceria para sempre.
#[test]
fn a_machine_whose_object_died_is_dropped() {
    let (mut sim, e, map) = scene();
    let st = z_just_pressed(&map);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    tick(
        &mut machines,
        &mut sim,
        &map,
        &st,
        true,
        1.0 / 60.0,
        &mut drive,
    );
    assert_eq!(machines.len(), 1);

    sim.world_mut().despawn(e);
    let quiet = ActionState::new();
    tick(
        &mut machines,
        &mut sim,
        &map,
        &quiet,
        true,
        1.0 / 60.0,
        &mut drive,
    );
    assert!(machines.is_empty(), "a maquina sobreviveu ao objecto");
}
