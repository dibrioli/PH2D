//! ⭐⭐⭐ **O conjunto de Morph States DENTRO de uma animação de States** (plano 32 W11c) — irmão
//! de [`super`] pelo teto de 600 LOC, e o corte é por ASSUNTO: ali *o que o conjunto faz às formas
//! por si*; aqui *o que o sistema de States, que já existia, consegue fazer com ele*.
//!
//! Enio, 2026-08-26: *"que eu possa usar o state morph nas animações criadas em States."*
//!
//! ⚠️ Submódulo do irmão de propósito: o harness (`world`) é **um só**.

use super::super::{create, upkeep};
use super::world;
use ph2d_ecs::{Entity, SimWorld, VecMorph};
use ph2d_vec_scene::{VecPath, VecScene};

use crate::vec_entities::{VecEntityMap, sync};

/// ⭐⭐⭐ **UMA POSE DE UI GRAVA EM QUE FORMA O CONJUNTO ESTÁ** (plano 32 W11c).
///
/// Enio, 2026-08-26: *"que eu possa usar o state morph nas animações criadas em States."*
///
/// ⚠️ **A forma que a cena MOSTRA é `sources[1]`** — o destino do último voo —, e não `sources[0]`:
/// `t = 1` no par `(A, B)` já **é** a forma B. Gravar a origem faria o `Hover` capturar a forma de
/// onde a máquina veio.
///
/// **Mutação que deve sangrar:** o `capture` gravar `sources[0]`.
#[test]
fn a_ui_pose_records_which_shape_the_set_is_showing() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    // A cena mostra a PRIMEIRA (o conjunto nasce em `[start, start]`).
    let p0 = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert_eq!(
        p0.morph_shape,
        Some(ids[0]),
        "a pose tem de gravar a forma que a cena MOSTRA"
    );

    // O motor leva-a à terceira: o par vira `(ids[0], ids[2])`.
    if let Some(mut m) = sim.world_mut().get_mut::<VecMorph>(host) {
        m.sources = [ids[0], ids[2]];
        m.t = 1.0;
    }
    let p1 = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert_eq!(
        p1.morph_shape,
        Some(ids[2]),
        "⛔ ela gravou a forma de ONDE a maquina veio, e nao a que se ve'"
    );
}

/// ⛔⛔ **UM MORPH SEM MÁQUINA grava `None`** — ele não é um conjunto de estados.
///
/// ⚠️ Um morph autorado à mão (dois operandos, `t` keyado pela timeline) não *está* numa forma:
/// dizer que está faria o `install` prendê-lo lá, **matando a curva** que a timeline conduz.
///
/// **Mutação que deve sangrar:** o `capture` largar a checagem do `VecMorphMachine`.
#[test]
fn a_hand_authored_morph_records_no_shape() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(ph2d_vec_scene::rectangle([3.0, 0.0], [4.0, 1.0]));
    let m = scene.push_path(VecPath::default());
    sync(&mut sim, &mut scene, &mut map);
    // Um Morph COMUM: componente sem máquina, que é como o botão «Morph» o cria.
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&m]))
        .insert(VecMorph::new(a, b));

    assert_eq!(
        crate::vec_ui_state_edit::capture(&sim, &scene, &map, m).morph_shape,
        None,
        "um morph autorado a' mao nao ESTA' numa forma -- prende-lo mataria o `t` da timeline"
    );

    // ⛔⛔ **E O `install` TAMBÉM NÃO O TOCA** — a outra metade, e ela nasceu de uma mutação que
    // SOBREVIVEU (2026-08-26): a guarda do `VecMorphMachine` no `install` estava **ungated**, e
    // apagá-la deixava a suíte inteira verde.
    //
    // O dano: uma pose com `morph_shape` (gravada sobre um conjunto) instalada sobre um morph
    // autorado à mão prendê-lo-ia num par degenerado — **matando a curva** que a timeline conduz.
    let host = Entity::from_bits(map[&m]);
    let before = sim.world().get::<VecMorph>(host).unwrap().clone();
    let mut pose = crate::vec_ui_state_edit::capture(&sim, &scene, &map, m);
    pose.morph_shape = Some(b);
    crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, &pose);
    assert_eq!(
        sim.world().get::<VecMorph>(host).unwrap(),
        &before,
        "⛔ o install prendeu um morph autorado a' mao numa forma -- o `t` da timeline morre"
    );
}

/// ⭐⭐ **E O `install` DEVOLVE a forma** — a chegada põe o par em `(shape, shape)`, que é exacta.
///
/// **Mutação que deve sangrar (1):** o `install` não escrever nada — o `Hover` animaria e a
/// chegada deixaria a forma no penúltimo `t`.
///
/// **Mutação que deve sangrar (2):** ele escrever mesmo com `morph_shape == None` — uma pose
/// gravada antes de o objecto ser um conjunto passaria a mandá-lo para a primeira forma.
#[test]
fn installing_a_pose_puts_the_set_exactly_on_its_shape() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    let mut pose = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    pose.morph_shape = Some(ids[2]);
    crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, &pose);
    let m = sim.world().get::<VecMorph>(host).expect("o morph fica");
    assert_eq!(
        (m.sources, m.t),
        ([ids[2], ids[2]], 0.0),
        "a chegada tem de por o par na forma EXACTA"
    );

    // ⛔ E `None` NÃO escreve: ele é *"esta pose não se pronuncia"*.
    let before = sim.world().get::<VecMorph>(host).unwrap().clone();
    pose.morph_shape = None;
    crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, &pose);
    assert_eq!(
        sim.world().get::<VecMorph>(host).unwrap(),
        &before,
        "`None` e' «nao me pronuncio» -- escrever por causa dele poria uma pose antiga a mandar"
    );
}

/// ⭐⭐⭐ **O QUADRO INTEIRO: ▶ Play, Rec, e a transição de States MORFA** — o report do Enio de
/// 2026-08-26, reproduzido pela composição que o produto corre.
///
/// > *"na animação de States, o morph não consegue segurar os estados atribuidos no momento do Rec.
/// > Lembrando que para animações de states eventos atribuidos para Morph states não devem ser
/// > necessários, pois os estados morph são mudados com play"*
///
/// ⛔⛔ **Nenhum dos três gates acima podia ver este defeito**, e a razão é a lição desta linha:
/// eles medem `capture` e `install` — as duas metades **certas** — e o defeito vivia na
/// **COMPOSIÇÃO**, no braço do despacho que abre a máquina. *Um gate de unidade é cego à fiação da
/// shell.*
///
/// ⚠️ **A fixtura não atribui uma única tecla**, e é metade do gate: o pedido é que o ▶ baste.
///
/// **Mutação que deve sangrar:** qualquer uma das três — o `play` voltar ao `get_mut`, o `open`
/// semear com `MorphMachine::new`, ou o `apply_ui_steps` não escrever.
#[test]
fn play_records_and_the_ui_transition_morphs_the_set() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    let mut machines = crate::morph_machine_drive::MorphMachines::new();
    let mut drive = crate::preview_drive::PreviewDrive::default();
    let input = ph2d_input::InputMap::new();
    let quiet = ph2d_input::ActionState::new();
    let morph_frame = |m: &mut _, s: &mut SimWorld, d: &mut _, on: bool| {
        crate::morph_machine_drive::tick(
            m,
            s,
            &map,
            &ph2d_input::Input::new(&input, &quiet),
            on,
            1.0 / 60.0,
            d,
        );
    };

    // (1) O papel Default: a cena está na 1ª forma, e é isso que se grava.
    let rest = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert_eq!(rest.morph_shape, Some(ids[0]));

    // (2) ▶ Play na 3ª linha, vindo de FORA do modo — que é de onde o artista vem.
    morph_frame(&mut machines, &mut sim, &mut drive, false);
    assert!(crate::morph_machine_drive::play(
        &mut machines,
        &sim,
        &map,
        host,
        2
    ));
    for _ in 0..60 {
        morph_frame(&mut machines, &mut sim, &mut drive, true);
    }

    // (3) Rec no papel Hover: ele tem de fotografar a forma NOVA.
    let hover = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert_eq!(
        hover.morph_shape,
        Some(ids[2]),
        "⛔ o Rec fotografou a MESMA forma do Default -- e' o report do Enio"
    );

    // (4) Fora do modo, a transição Default -> Hover tem de morfar de verdade.
    morph_frame(&mut machines, &mut sim, &mut drive, false);
    let mut states = ph2d_ui_state::StateSets::default();
    for (role, p) in [
        (ph2d_ui_state::StateRole::Default, &rest),
        (ph2d_ui_state::StateRole::Hover, &hover),
    ] {
        let mut st = ph2d_ui_state::UiState::new(role);
        st.objects = vec![p.clone()];
        states.set(host_id, st);
    }
    let mut ui = crate::render_loop::ui_state_bridge::UiMachines::new();
    let mut cooked = crate::render_loop::ui_state_bridge::Cooked::default();
    crate::render_loop::ui_state_bridge::request(
        &mut ui,
        &states,
        host_id,
        ph2d_ui_state::StateRole::Hover,
    );
    crate::render_loop::ui_state_bridge::dispatch(
        &mut ui,
        &mut states,
        &mut sim,
        &mut scene,
        &map,
        0.05,
        &mut cooked,
    );
    assert_eq!(
        cooked.morph_steps.len(),
        1,
        "a transicao nao publicou passo"
    );
    crate::morph_machine_drive::apply_ui_steps(&mut sim, &map, &cooked.morph_steps, &mut drive);
    let m = sim.world().get::<VecMorph>(host).unwrap();
    assert_eq!(
        m.sources,
        [ids[0], ids[2]],
        "o mundo nao ficou entre as DUAS formas gravadas"
    );
    assert!(
        m.t > 0.0 && m.t < 1.0,
        "a meio da transicao o t tem de estar entre as pontas: {}",
        m.t
    );
}

/// ⛔⛔ **A POSE DE UM CONJUNTO NÃO CARREGA GEOMETRIA** — ela é DERIVADA.
///
/// A forma de um conjunto de Morph States é reescrita por `morph_live::recook` em todo quadro, a
/// partir do par e do `t`. Gravá-la na pose não guarda trabalho do artista nenhum e custa duas
/// coisas: o `install` escreve-a para o `recook` a apagar **no mesmo quadro**, e a `Transition`
/// **casa duas geometrias** para animar um canal que ninguém lê — `Plan::new` custa **13 079×** um
/// passo, e é por isso que o `plans_built` existe.
///
/// ⚠️ **A TINTA fica**, e a assimetria é o gate: o `recook` escreve os vértices e **não** o
/// `fill`/`stroke`, então a cor do conjunto é autorada e anima como a de qualquer forma.
///
/// **Mutação que deve sangrar:** largar a guarda `!derived_geometry` no `capture`.
#[test]
fn a_set_pose_carries_no_geometry_and_costs_no_plan() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    // O CONTROLE vem primeiro: uma forma NORMAL grava a geometria dela.
    let leaf = crate::vec_ui_state_edit::capture(&sim, &scene, &map, ids[0]);
    assert!(
        leaf.geometry.is_some(),
        "uma forma normal tem de gravar a geometria -- a fixtura perdeu a premissa"
    );

    let a = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert!(
        a.geometry.is_none(),
        "a pose do conjunto gravou geometria DERIVADA"
    );
    // Cozer o conjunto noutra forma NÃO pode fazer a pose passar a carregá-la.
    if let Some(mut m) = sim.world_mut().get_mut::<VecMorph>(host) {
        m.sources = [ids[0], ids[2]];
        m.t = 1.0;
    }
    let xf = crate::vec_transform::build(&sim, &map);
    crate::morph_live::recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut crate::morph_live::MorphPlans::default(),
    );
    let b = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert!(
        b.geometry.is_none(),
        "e nem depois de o recook a reescrever"
    );
    assert_eq!(
        ph2d_ui_state::Transition::new(&[a], &[b]).plans_built(),
        0,
        "a transicao casou duas geometrias que o recook apaga no mesmo quadro"
    );
}

/// ⛔⛔ **DESCONECTAR UMA FORMA TIRA-A DOS ESTADOS GRAVADOS** — senão ela é puxada de volta.
///
/// ⚠️ **O mecanismo:** um estado grava a **sub-árvore** (`members`), então as formas-membro entram
/// com a pose **LOCAL** que têm dentro do conjunto (o `align` põe-nas todas no mesmo ponto). O ⊘
/// tira a forma do conjunto e devolve-lhe o `Transform` de mundo — mas a pose antiga continua na
/// tabela, e o `install` do próximo Show/hover **reescreve-lhe o `Transform`**: a forma solta
/// **salta para a origem do conjunto**, no meio de uma animação que já não é sobre ela.
///
/// ⚠️ **A família é PRÉ-EXISTENTE** (reparentar um filho para fora de um widget faz o mesmo), mas
/// aqui ela está a **um clique** de distância, num botão da mesma secção. É o mesmo argumento do
/// `retain_hosts`, um nível abaixo: *uma forma que sai leva as poses dela*.
///
/// ⛔ **O Dissolve não precisa disto** — ele apaga o path do conjunto, e o `dispatch` já deixa cair
/// a tabela inteira do hospedeiro (`retain_hosts`). Só o ⊘ de UMA forma vazava.
///
/// **Mutação que deve sangrar:** o `disconnect_row` não chamar `forget_object_in_all_states`.
#[test]
fn disconnecting_a_shape_takes_it_out_of_the_recorded_states() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;

    let mut states = ph2d_ui_state::StateSets::default();
    let mut st = ph2d_ui_state::UiState::new(ph2d_ui_state::StateRole::Default);
    st.objects = crate::vec_ui_state_edit::members(&sim, &scene, &map, host_id)
        .into_iter()
        .map(|id| crate::vec_ui_state_edit::capture(&sim, &scene, &map, id))
        .collect();
    states.set(host_id, st);
    assert!(
        states
            .role(host_id, ph2d_ui_state::StateRole::Default)
            .is_some_and(|s| s.objects.iter().any(|p| p.id == ids[1])),
        "a fixtura tem de gravar a forma-membro -- senao nao ha' o que vazar"
    );

    // ⚠️ **Pela porta do PRODUTO** (`disconnect_row`), e nao chamando as duas metades a' mao: um
    // gate que as chamasse provaria que elas funcionam, nao que o ⊘ as chama -- que e' o defeito.
    let host = Entity::from_bits(map[&host_id]);
    assert_eq!(
        crate::morph_set::disconnect_row(&mut sim, &map, &mut states, host, 1),
        None,
        "com tres formas o ⊘ tira uma e NAO dissolve o conjunto"
    );
    assert!(
        !states
            .role(host_id, ph2d_ui_state::StateRole::Default)
            .is_some_and(|s| s.objects.iter().any(|p| p.id == ids[1])),
        "⛔ a forma solta ficou na tabela -- o proximo Show puxa-a para a origem do conjunto"
    );
    // O CONTROLE: as OUTRAS ficam, e o hospedeiro tambem.
    let left = states
        .role(host_id, ph2d_ui_state::StateRole::Default)
        .map(|s| s.objects.len())
        .unwrap_or(0);
    assert_eq!(left, 3, "so' a desconectada podia sair (host + 2 formas)");
}

/// ⭐⭐⭐ **O 2.º REPORT DO ENIO (2026-08-26), reproduzido**: a máquina de teclas **larga** enquanto
/// o sistema de States age.
///
/// > *"em states Default gravei Morph States em wide, em hover gravei Morph states em tall. Ao ligar
/// > o preview Default não segurou wide e está em tall. No hover há uma transição tall - wide - tall.
/// > Ao sair de hover o mesmo acontece: tall - wide - tall."*
///
/// ⛔⛔ **O mecanismo:** dois motores a escrever o MESMO `VecMorph`. A W11c ordenou-os **dentro** do
/// quadro, e isso só resolve os instantes em que a transição **fala** — ela cala-se nas pontas, de
/// propósito. No REPOUSO e na CHEGADA quem escrevia era a máquina de teclas, parada onde o ▶ a
/// deixou. Os três sintomas do report saem daí, um a um.
///
/// ⚠️ **A fixtura é o estado exacto do report**: máquina parada em `tall`, mundo posto em `wide` pelo
/// `install` do `Default`, e o modo do Morph **ligado**.
///
/// **Mutação que deve sangrar:** o `drives` devolver `morph_preview` (ignorando o `ui_state_live`).
#[test]
fn the_key_machine_lets_go_while_the_ui_states_act() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    let mut machines = crate::morph_machine_drive::MorphMachines::new();
    let mut drive = crate::preview_drive::PreviewDrive::default();
    let input = ph2d_input::InputMap::new();
    let quiet = ph2d_input::ActionState::new();
    let tick = |m: &mut _, s: &mut SimWorld, d: &mut _, on: bool| {
        crate::morph_machine_drive::tick(
            m,
            s,
            &map,
            &ph2d_input::Input::new(&input, &quiet),
            on,
            1.0 / 60.0,
            d,
        );
    };
    let showing = |s: &SimWorld| s.world().get::<VecMorph>(host).unwrap().sources[1];

    // O ▶ leva o conjunto a` 3a forma e a maquina fica parada la' -- o estado em que o Rec do
    // `Hover` acontece.
    tick(&mut machines, &mut sim, &mut drive, false);
    assert!(crate::morph_machine_drive::play(
        &mut machines,
        &sim,
        &map,
        host,
        2
    ));
    for _ in 0..60 {
        tick(&mut machines, &mut sim, &mut drive, true);
    }
    assert_eq!(showing(&sim), ids[2], "a fixtura nao chegou a' 3a forma");

    // Agora o sistema de States poe a cena no `Default` (a 1a forma), como o `ui_preview::enter` faz.
    let mut rest = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    rest.morph_shape = Some(ids[0]);
    crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, &rest);
    assert_eq!(showing(&sim), ids[0]);

    // ⛔ E o quadro seguinte, com o modo do Morph AINDA ligado, nao pode repor a forma da maquina.
    for _ in 0..10 {
        tick(
            &mut machines,
            &mut sim,
            &mut drive,
            crate::morph_machine_drive::drives(true, true),
        );
    }
    assert_eq!(
        showing(&sim),
        ids[0],
        "⛔ a maquina de teclas repos a forma dela por cima do Default -- e' o report do Enio"
    );

    // ⭐ E ao largar o sistema de States, a maquina volta SEM SALTO: ela renasce semeada pelo mundo.
    for _ in 0..10 {
        tick(
            &mut machines,
            &mut sim,
            &mut drive,
            crate::morph_machine_drive::drives(true, false),
        );
    }
    assert_eq!(
        showing(&sim),
        ids[0],
        "a maquina voltou e saltou para a forma antiga -- ela tem de nascer onde os States a deixaram"
    );
}

/// ⭐⭐ **O que acontece à tabela quando o CONJUNTO muda por baixo dela** — irmão por LOC (HR-18),
/// e o corte é por assunto: aqui em cima *o que os States conseguem fazer com um conjunto*; ali, *o
/// que os repara quando uma forma sai dele*.
#[cfg(test)]
#[path = "morph_set_states_repair_tests.rs"]
mod repair_tests;
