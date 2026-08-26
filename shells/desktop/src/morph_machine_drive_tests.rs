//! Os gates da máquina a correr — mundo, mapa e relógio, sem janela nenhuma.

use super::{MorphMachines, tick};
use ph2d_ecs::{SimWorld, VecMorph, VecMorphMachine};
use ph2d_input::{ActionState, Binding, InputMap, InputState, Key};
use ph2d_morph_machine::MorphKey;
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::preview_drive::PreviewDrive;
use crate::vec_entities::{VecEntityMap, sync};

const KEY_Z: u32 = 0x5A;

/// **O mundo de teste, montado pela PORTA REAL do produto** (`morph_set::create` + `upkeep`).
///
/// ⚠️ **Ele deixou de poder ser fabricado à mão na W11.** Antes bastava pendurar um
/// `VecMorphMachine` com a lista lá dentro; hoje a lista **são os filhos**, e um harness que os
/// dispensasse estaria a testar um mundo que o produto não sabe produzir — *uma fixtura que não
/// contém o fenómeno aprova a cura errada*.
///
/// Devolve `(sim, scene, map de paths, host, formas, InputMap)`. As `keys` são atribuídas por
/// `named`.
fn world(named: &[(usize, &str, f64)]) -> Bench {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let shapes: Vec<VecPathId> = (0..2 + named.iter().map(|n| n.0).max().unwrap_or(0))
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let x = i as f64 * 5.0;
            scene.push_path(ph2d_vec_scene::rectangle([x, -1.0], [x + 2.0, 1.0]))
        })
        .collect();
    sync(&mut sim, &mut scene, &mut map);
    let mut pending = crate::morph_set::create(&sim, &mut scene, &map, &shapes, 9);
    sync(&mut sim, &mut scene, &mut map);
    crate::morph_set::upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = ph2d_ecs::Entity::from_bits(map[&host_id]);

    let mut input = InputMap::new();
    if let Some(mut m) = sim.world_mut().get_mut::<VecMorphMachine>(host) {
        for &(ix, action, dur) in named {
            m.keys.insert(
                shapes[ix],
                MorphKey {
                    when: action.to_string(),
                    duration_s: dur,
                    ..MorphKey::default()
                },
            );
        }
    }
    for &(_, action, _) in named {
        if input.id(action).is_none() {
            input.create(action);
        }
    }
    Bench {
        sim,
        map,
        host,
        shapes,
        input,
    }
}

struct Bench {
    sim: SimWorld,
    map: VecEntityMap,
    host: ph2d_ecs::Entity,
    shapes: Vec<VecPathId>,
    input: InputMap,
}

impl Bench {
    /// Liga `action` à tecla `code`.
    fn bind(&mut self, action: &str, code: u32) {
        let id = self.input.id(action).expect("a accao existe");
        self.input
            .get_mut(id)
            .unwrap()
            .bindings
            .push(Binding::Key(Key(code)));
    }

    /// O destino que a cena mostra agora (`VecMorph::sources[1]`).
    fn showing(&self) -> VecPathId {
        self.sim.world().get::<VecMorph>(self.host).unwrap().sources[1]
    }
}

/// Um `ActionState` com uma tecla carregada NESTE tique (e solta no anterior) — é isso que faz o
/// `just_pressed` responder.
fn just_pressed(map: &InputMap, code: u32) -> ActionState {
    let mut st = ActionState::new();
    let mut dev = InputState::new();
    st.tick(map, &dev); // o tique de ANTES: a tecla ainda nao foi carregada
    dev.keyboard.handle_key_down(Key(code));
    st.tick(map, &dev);
    st
}

/// ⭐⭐ **A TECLA MORFA A FORMA** — o caminho inteiro, do mapa ao componente.
///
/// **Mutação que deve sangrar:** o `fire` nunca ser chamado — a máquina fica parada e a tecla não
/// faz nada, que é a feature inteira.
#[test]
fn the_bound_key_moves_the_morph_from_one_shape_to_the_other() {
    let mut b = world(&[(1, "jump", 0.1)]);
    b.bind("jump", KEY_Z);
    let st = just_pressed(&b.input, KEY_Z);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();

    let ran = tick(
        &mut machines,
        &mut b.sim,
        &b.map,
        &ph2d_input::Input::new(&b.input, &st),
        true,
        1.0 / 60.0,
        &mut drive,
    );
    assert_eq!(ran, 1, "a maquina tem de correr");
    assert_eq!(
        b.sim.world().get::<VecMorph>(b.host).unwrap().sources,
        [b.shapes[0], b.shapes[1]],
        "o par tem de ser o da transicao que disparou"
    );
    // Andar ate' ao fim: a forma chega na segunda.
    let quiet = ActionState::new();
    for _ in 0..30 {
        tick(
            &mut machines,
            &mut b.sim,
            &b.map,
            &ph2d_input::Input::new(&b.input, &quiet),
            true,
            1.0 / 60.0,
            &mut drive,
        );
    }
    let m = b.sim.world().get::<VecMorph>(b.host).unwrap();
    assert_eq!(
        (m.sources, m.t),
        ([b.shapes[0], b.shapes[1]], 1.0),
        "chegou, e o par NAO trocou"
    );
}

/// ⛔⛔ **FORA DO MODO a máquina não corre, e a tecla não faz NADA.**
///
/// ⚠️ Não é conservadorismo: a condição é uma tecla, e a escutar durante a edição carregar em `Z`
/// morfava a forma **e** fazia o que o `Z` faz no editor — os dois, sem que nada na tela
/// explicasse.
///
/// ⚠️ **O «modo» deixou de ser o playhead na W9** (Enio, 2026-08-25): o transporte a andar **não**
/// tranca o teclado do editor. Hoje a porta é o interruptor `Preview` da seção, que o toma.
///
/// **Mutação que deve sangrar:** largar a guarda do `active`.
#[test]
fn outside_the_mode_the_key_does_nothing() {
    let mut b = world(&[(1, "jump", 0.1)]);
    b.bind("jump", KEY_Z);
    let st = just_pressed(&b.input, KEY_Z);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    let start = b.shapes[0];

    let ran = tick(
        &mut machines,
        &mut b.sim,
        &b.map,
        &ph2d_input::Input::new(&b.input, &st),
        false,
        1.0 / 60.0,
        &mut drive,
    );
    assert_eq!(ran, 0);
    assert_eq!(
        b.sim.world().get::<VecMorph>(b.host).unwrap().sources,
        [start, start],
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
    let mut b = world(&[(1, "jump", 0.1)]);
    b.bind("jump", KEY_Z);
    let st = just_pressed(&b.input, KEY_Z);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    let (s0, s1) = (b.shapes[0], b.shapes[1]);
    tick(
        &mut machines,
        &mut b.sim,
        &b.map,
        &ph2d_input::Input::new(&b.input, &st),
        true,
        1.0 / 60.0,
        &mut drive,
    );

    // A captura repõe o AUTORADO — é o que a fotografia do undo vê.
    let live = drive.substitute_authored(&mut b.sim);
    let m = b.sim.world().get::<VecMorph>(b.host).unwrap();
    assert_eq!(
        (m.sources, m.t),
        ([s0, s0], 0.0),
        "durante a fotografia o mundo tem de mostrar o que o ARTISTA desenhou"
    );
    // …e a cena volta a mostrar o que o motor escreveu.
    PreviewDrive::restore_live(&mut b.sim, &live);
    assert_eq!(
        b.sim.world().get::<VecMorph>(b.host).unwrap().sources,
        [s0, s1],
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
/// a máquina naquela forma — toda outra transição é desfeita no quadro seguinte.
///
/// **Mutação que deve sangrar:** trocar `just_pressed` por `pressed` no `morph_machine_drive`.
#[test]
fn a_held_key_fires_once_and_never_pins_the_machine() {
    const KEY_Q: u32 = 0x51;
    let mut b = world(&[(1, "jump", 0.0), (2, "dash", 0.0)]);
    b.bind("jump", KEY_Z);
    b.bind("dash", KEY_Q);
    let (s1, s2) = (b.shapes[1], b.shapes[2]);

    let mut st = ActionState::new();
    let mut dev = InputState::new();
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();

    // O `Z` desce e FICA em baixo. Dez quadros.
    dev.keyboard.handle_key_down(Key(KEY_Z));
    for _ in 0..10 {
        st.tick(&b.input, &dev);
        tick(
            &mut machines,
            &mut b.sim,
            &b.map,
            &ph2d_input::Input::new(&b.input, &st),
            true,
            1.0 / 60.0,
            &mut drive,
        );
    }
    assert_eq!(
        b.showing(),
        s1,
        "o CONTROLE: a primeira descida do Z tem de levar a' segunda forma"
    );

    // ⭐ Agora o `Q`, **com o `Z` ainda segurado**. Com `pressed`, o `jump` voltava a disparar no
    // quadro seguinte e arrastava a maquina de volta -- que e' o defeito.
    dev.keyboard.handle_key_down(Key(KEY_Q));
    for _ in 0..10 {
        st.tick(&b.input, &dev);
        tick(
            &mut machines,
            &mut b.sim,
            &b.map,
            &ph2d_input::Input::new(&b.input, &st),
            true,
            1.0 / 60.0,
            &mut drive,
        );
    }
    assert_eq!(
        b.showing(),
        s2,
        "a tecla SEGURADA pinou a maquina: o dash levou a' terceira e o jump segurado trouxe-a de \
         volta"
    );
}

/// ⭐⭐⭐ **O BOTÃO ▶ ANDA MESMO COM O MODO ACABADO DE LIGAR** — o report do Enio de 2026-08-26.
///
/// > *"na animação de States, o morph não consegue segurar os estados atribuidos no momento do
/// > Rec. Lembrando que para animações de states eventos atribuidos para Morph states não devem
/// > ser necessários, pois os estados morph são mudados com play"*
///
/// ⛔ **O mecanismo:** o mapa de máquinas é **propriedade do `tick`**, e o `tick` **esvazia-o** em
/// todo quadro fora do modo. O verbo Play corre DEPOIS do `tick` no mesmo quadro ⇒ com a
/// pré-visualização desligada ele encontrava o mapa **vazio**, ligava o modo e **não viajava**. A
/// forma só mudava ao segundo clique, e o Rec seguinte fotografava a forma ERRADA.
///
/// ⚠️ **A fixtura não dá tecla nenhuma** (`world(&[])`), e isso é metade do gate: o pedido do Enio
/// é que o Play baste — um harness com teclas atribuídas passaria por outra porta.
///
/// **Mutação que deve sangrar:** trocar o `entry(..).or_insert_with(..)` do `play` por um
/// `get_mut(..)?` — que é exactamente o código que o report descreve.
#[test]
fn the_play_button_travels_on_the_very_frame_the_mode_turns_on() {
    let mut b = world(&[]);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    let quiet = ActionState::new();

    // O quadro FORA do modo: é ele que esvazia o mapa, e é dele que o artista vem.
    tick(
        &mut machines,
        &mut b.sim,
        &b.map,
        &ph2d_input::Input::new(&b.input, &quiet),
        false,
        1.0 / 60.0,
        &mut drive,
    );
    assert!(
        machines.is_empty(),
        "a fixtura nao reproduziu o estado de que o report parte: o mapa tinha de estar vazio"
    );

    assert!(
        super::play(&mut machines, &b.sim, &b.map, b.host, 1),
        "o Play nao viajou -- e' o report do Enio: o botao so' liga o modo"
    );
    for _ in 0..60 {
        tick(
            &mut machines,
            &mut b.sim,
            &b.map,
            &ph2d_input::Input::new(&b.input, &quiet),
            true,
            1.0 / 60.0,
            &mut drive,
        );
    }
    assert_eq!(
        b.showing(),
        b.shapes[1],
        "a cena nao chegou a' forma que o Play pediu"
    );
}

/// ⭐⭐⭐ **UMA MÁQUINA QUE NASCE CONCORDA COM O QUE A CENA MOSTRA.**
///
/// ⛔ **A segunda metade do mesmo report.** Sair do modo **não** repõe a forma no mundo (o ledger
/// larga a condução e a `settle` promove o vivo a documento — é a lei do *«desfaz a corrida»*), mas
/// a máquina seguinte nascia em `graph.start()`. ⇒ com a cena a mostrar a forma B e a máquina a
/// julgar-se em A, o Play para **A** era recusado (*«chegar onde já se está não é chegar»*) e o
/// Play para **B** não fazia nada — o artista gravava sempre a mesma forma.
///
/// ⚠️ *Uma máquina que dirige o mundo tem de ser SEMEADA por ele.*
///
/// **Mutação que deve sangrar:** semear com `MorphMachine::new(&graph)` em vez de
/// `seeded(&graph, ..)`.
#[test]
fn a_machine_born_after_the_mode_reopens_agrees_with_the_canvas() {
    let mut b = world(&[]);
    let mut machines = MorphMachines::new();
    let mut drive = PreviewDrive::default();
    let quiet = ActionState::new();
    let run = |m: &mut MorphMachines, s: &mut SimWorld, d: &mut PreviewDrive, on: bool| {
        tick(
            m,
            s,
            &b.map,
            &ph2d_input::Input::new(&b.input, &quiet),
            on,
            1.0 / 60.0,
            d,
        );
    };

    run(&mut machines, &mut b.sim, &mut drive, false);
    assert!(super::play(&mut machines, &b.sim, &b.map, b.host, 1));
    for _ in 0..60 {
        run(&mut machines, &mut b.sim, &mut drive, true);
    }
    assert_eq!(b.showing(), b.shapes[1], "a fixtura nao chegou a' 2a forma");

    // O artista desliga a pre'-visualizacao. A cena FICA na forma B (a `settle` promove o vivo).
    run(&mut machines, &mut b.sim, &mut drive, false);
    assert_eq!(
        b.showing(),
        b.shapes[1],
        "sair do modo mexeu na forma -- a premissa deste gate mudou, releia a `settle`"
    );

    // E agora o Play para a forma de PARTIDA tem de andar.
    assert!(
        super::play(&mut machines, &b.sim, &b.map, b.host, 0),
        "o Play para a 1a forma foi recusado: a maquina nasceu a julgar-se la'"
    );
    for _ in 0..60 {
        run(&mut machines, &mut b.sim, &mut drive, true);
    }
    assert_eq!(
        b.showing(),
        b.shapes[0],
        "a cena nao voltou a' 1a forma pelo botao"
    );
}
