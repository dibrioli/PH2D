//! Os gates da máquina a correr — mundo, mapa e relógio, sem janela nenhuma.

use super::{MorphMachines, tick};
use ph2d_ecs::{SimWorld, VecMorph, VecMorphMachine};
use ph2d_input::{ActionState, Binding, InputMap, InputState, Key};
use ph2d_morph_machine::{MorphGraph, MorphState};

use crate::preview_drive::PreviewDrive;

const A: u64 = 10;
const B: u64 = 20;
const KEY_Z: u32 = 0x5A;

/// Um mundo com duas formas — `A` (o começo) e `B`, alcançada pela acção `jump` — e o mapa que a
/// liga ao `Z`.
fn scene() -> (SimWorld, ph2d_ecs::Entity, InputMap) {
    let mut sim = SimWorld::new();
    let mut m = VecMorphMachine::new(&[A, B]);
    let mut b = MorphState::new(B);
    b.when = "jump".to_string();
    b.duration_s = 0.1;
    m.graph = MorphGraph {
        states: vec![MorphState::new(A), b],
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

/// ⛔⛔ **FORA DO MODO a máquina não corre, e a tecla não faz NADA.**
///
/// ⚠️ Não é conservadorismo: a condição de uma seta é uma tecla, e a escutar durante a edição
/// carregar em `Z` morfava a forma **e** fazia o que o `Z` faz no editor — os dois, sem que nada na
/// tela explicasse.
///
/// ⚠️ **O «modo» deixou de ser o playhead na W9** (Enio, 2026-08-25): o transporte a andar **não**
/// tranca o teclado do editor, então com ele o conflito ficava exactamente onde estava. Hoje a
/// porta é o interruptor `Preview` da seção, que toma o teclado.
///
/// **Mutação que deve sangrar:** largar a guarda do `active`.
#[test]
fn outside_the_mode_the_key_does_nothing() {
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

/// ⚠️⚠️ **Uma tecla SEGURADA não re-dispara — e sob o modelo por-forma o dano é PINAR.**
///
/// ⛔⛔ **Este gate MUDOU de fixtura na W10, e a razão é o achado.** Ele media uma cadeia
/// `A --jump--> B --jump--> C`: com `pressed` em vez de `just_pressed`, a máquina saltava a cadeia
/// inteira num piscar de olhos. **Essa cadeia deixou de ser exprimível** — uma tecla nomeia UMA
/// forma —, e a mutação `just_pressed -> pressed` passaria a **SOBREVIVER**: o segundo disparo é
/// recusado por já se estar em `B`, e nada observável muda.
///
/// ⇒ *o dano mudou de forma, e a régua tem de o seguir*: com `pressed`, uma tecla segurada **PINA**
/// a máquina naquela forma — toda outra transição é desfeita no quadro seguinte. É isso que este
/// gate mede agora.
///
/// **Mutação que deve sangrar:** trocar `just_pressed` por `pressed` no `morph_machine_drive`.
#[test]
fn a_held_key_fires_once_and_never_pins_the_machine() {
    const C: u64 = 30;
    const KEY_Q: u32 = 0x51;
    let mut sim = SimWorld::new();
    let mut m = VecMorphMachine::new(&[A]);
    let mut b = MorphState::new(B);
    b.when = "jump".to_string();
    b.duration_s = 0.0; // instantanea: o gate mede o DISPARO, nao a duracao
    let mut c = MorphState::new(C);
    c.when = "dash".to_string();
    c.duration_s = 0.0;
    m.graph = MorphGraph {
        states: vec![MorphState::new(A), b, c],
    };
    let ent = sim.world_mut().spawn((VecMorph::new(A, A), m)).id();

    let mut map = InputMap::new();
    let j = map.create("jump");
    map.get_mut(j)
        .unwrap()
        .bindings
        .push(Binding::Key(Key(KEY_Z)));
    let d = map.create("dash");
    map.get_mut(d)
        .unwrap()
        .bindings
        .push(Binding::Key(Key(KEY_Q)));

    let mut st = ActionState::new();
    let mut dev = InputState::new();
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    let run = |st: &mut ActionState,
               dev: &InputState,
               sim: &mut SimWorld,
               machines: &mut MorphMachines,
               drive: &mut PreviewDrive,
               n: usize| {
        for _ in 0..n {
            st.tick(&map, dev);
            tick(machines, sim, &map, st, true, 1.0 / 60.0, drive);
        }
    };

    // O `Z` desce e FICA em baixo. Dez quadros.
    dev.keyboard.handle_key_down(Key(KEY_Z));
    run(&mut st, &dev, &mut sim, &mut machines, &mut drive, 10);
    assert_eq!(
        sim.world().get::<VecMorph>(ent).unwrap().sources[1],
        B,
        "o CONTROLE: a primeira descida do Z tem de levar a B"
    );

    // ⭐ Agora o `Q`, **com o `Z` ainda segurado**. Com `pressed`, o `jump` voltava a disparar no
    // quadro seguinte e arrastava a maquina de volta a B -- que e' o defeito.
    dev.keyboard.handle_key_down(Key(KEY_Q));
    run(&mut st, &dev, &mut sim, &mut machines, &mut drive, 10);
    assert_eq!(
        sim.world().get::<VecMorph>(ent).unwrap().sources[1],
        C,
        "a tecla SEGURADA pinou a maquina: o dash levou a C e o jump segurado trouxe-a de volta"
    );
}
