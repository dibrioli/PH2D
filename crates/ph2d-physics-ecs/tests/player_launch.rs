//! **O EMPURRÃO DE FORA, pela porta do produto** (`W-Launch`) — a porta, a
//! janela e os três modos.
//!
//! ⚠️ **O oráculo é o DESLOCAMENTO, nunca um campo de estado:** o que o jogador
//! vê é o personagem sair do lugar. Um gate sobre `push_lock > 0` ficaria verde
//! com um empurrão que o freio apaga antes de chegar ao solver — que é
//! exactamente o mundo de antes desta wave.
//!
//! ⚠️ **O modo é um PAR, não um componente:** `pose_owner` pergunta ao KIND do
//! corpo antes de olhar para o `PlayerMode`, e inserir só o componente sobre um
//! corpo `Dynamic` mede o MESMO caminho três vezes (o controlo positivo da sonda
//! do teto de queda apanhou isso — `measure_terminal.rs`).

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{BodyKind, MassOverride, PhysicsBridge, PlayerInput, PlayerMode, RigidBody};
use scene_fixture::{pose, scene};

/// A velocidade do empurrão, m/s — para a DIREITA.
const PUSH: [f32; 2] = [10.0, 0.0];
/// Quanto tempo ele é dono do personagem, s.
///
/// ⚠️ **MEDIDO:** a caminhada apaga um empurrão em **9 tiques (0,15 s)** com o
/// jogador a não tocar em nada (`measure_launch.rs`), então uma janela abaixo
/// disso não compraria nada e uma muito acima mediria o resto do sistema.
const LOCK: f32 = 0.3;

/// Os três modos, com o nome que os smokes usam.
const MODES: [(Option<PlayerMode>, &str); 3] = [
    (None, "Spring"),
    (Some(PlayerMode::Kinematic), "Snap"),
    (Some(PlayerMode::Pure), "Pure"),
];

/// Uma cena plana com o personagem já assente no chão, no modo pedido.
fn standing(mode: Option<PlayerMode>) -> (SimWorld, PhysicsBridge, Entity) {
    weighing(mode, None)
}

/// A mesma cena, com a massa autorada ANTES do primeiro tique.
///
/// ⚠️ **E o "antes" é a premissa inteira, não conveniência:** o `reconcile` só
/// re-descreve um corpo **em repouso** (`at_rest && b.rest != desc`, em
/// `bridge::bodies`) — é a regra que faz de toda config editada com o relógio a
/// andar uma coisa transiente, a mesma que o `DampingOverride` documenta. Uma
/// fixture que inserisse o `MassOverride` depois de assentar mediria **o mesmo
/// número** com e sem ele, e leria isso como *"a massa não chega"*.
fn weighing(mode: Option<PlayerMode>, mass: Option<f32>) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, mut bridge, player) = scene(0.0, 0.0);
    if let Some(m) = mode {
        sim.world_mut().entity_mut(player).insert((
            m,
            RigidBody {
                kind: BodyKind::Kinematic,
            },
        ));
    }
    if let Some(m) = mass {
        sim.world_mut().entity_mut(player).insert(MassOverride(m));
    }
    for t in 1..=60_u64 {
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
    }
    (sim, bridge, player)
}

/// Empurra (ou não) e devolve quanto o personagem andou em meio segundo.
fn shove(mode: Option<PlayerMode>, push: Option<([f32; 2], f32)>, drive: f32) -> f32 {
    let (mut sim, mut bridge, player) = standing(mode);
    let x0 = pose(&sim).0;
    if let Some((v, lock)) = push {
        bridge.launch_player(player, v, lock);
    }
    for t in 61..=90_u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
    }
    pose(&sim).0 - x0
}

/// **⚠️ O GATE DA WAVE: o empurrão chega nos TRÊS modos** — e o CONTROLE é a
/// mesma cena sem empurrão nenhum.
///
/// ⚠️ **Sem o controlo dentro do mesmo gate isto não diria nada:** *"ele andou"*
/// só quer dizer alguma coisa ao lado de *"e parado ele não anda"*. E é o
/// controlo que torna visível o mundo de antes desta wave — ali um impulso
/// alcançava **1** corpo sob Spring e **ZERO** sob Snap e Pure.
#[test]
fn a_push_reaches_the_player_in_all_three_modes() {
    for (mode, tag) in MODES {
        let idle = shove(mode, None, 0.0);
        let pushed = shove(mode, Some((PUSH, LOCK)), 0.0);
        assert!(
            idle.abs() < 0.05,
            "[{tag}] o CONTROLE tem de ficar parado: {idle:.3} m"
        );
        assert!(
            pushed > 1.0,
            "[{tag}] o empurrao tem de o tirar do lugar: {pushed:.3} m"
        );
    }
}

/// **⚠️ SEM A JANELA O EMPURRÃO É COMIDO** — e é este gate que justifica o
/// segundo argumento da porta.
///
/// Medido em `measure_launch.rs`: `13,92 m/s` no primeiro tique e `0,000` no
/// décimo, com o jogador a não tocar em nada — quem apaga é o **freio**, não o
/// direcional. Uma porta que entregasse velocidade sem a janela seria uma porta
/// que não faz nada.
///
/// **Mutação que deve sangrar:** o `drive_players` ignorar o `l.lock`.
#[test]
fn without_the_window_the_walk_eats_the_push() {
    for (mode, tag) in MODES {
        let held = shove(mode, Some((PUSH, LOCK)), 0.0);
        let eaten = shove(mode, Some((PUSH, 0.0)), 0.0);
        assert!(
            held > eaten * 1.8,
            "[{tag}] a janela tem de valer MUITO: com {held:.3} m contra {eaten:.3} m sem"
        );
    }
}

/// **Segurar a direção contrária não apaga o empurrão** — o caso em que um
/// empurrão *"que funciona"* desaparece na mão de quem joga.
///
/// ⚠️ **Ele não tem de ser IGUAL ao do jogador solto**, e pedir isso seria pedir
/// que a janela durasse para sempre: o que se afirma é que o personagem **acaba
/// à frente de onde começou**, ou seja que o empurrão sobreviveu ao dedo.
#[test]
fn holding_the_opposite_direction_does_not_erase_the_push() {
    for (mode, tag) in MODES {
        let pushed = shove(mode, Some((PUSH, LOCK)), -1.0);
        assert!(
            pushed > 0.5,
            "[{tag}] o empurrao tem de sobreviver ao dedo contrario: {pushed:.3} m"
        );
    }
}

/// **O empurrão é entregue UMA vez, mesmo num dispatch que deve vários tiques.**
///
/// ⚠️ **É a diferença inteira contra a entrada do dedo:** aquela é
/// *set-and-hold* — um dispatch que deve quatro tiques aplica a MESMA a todos
/// eles —, e um empurrão é um EVENTO. Guardado no mesmo mapa, ele seria entregue
/// quatro vezes, e o personagem sairia com quatro vezes a velocidade que quem o
/// empurrou pediu.
///
/// **Mutação que deve sangrar:** trocar o `remove` por um `get` no dreno.
#[test]
fn a_push_is_delivered_once_even_when_the_dispatch_owes_many_ticks() {
    let travel = |ticks: u64| {
        let (mut sim, mut bridge, player) = standing(None);
        let x0 = pose(&sim).0;
        bridge.launch_player(player, PUSH, LOCK);
        // UM dispatch que deve `ticks` tiques.
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, 60 + ticks);
        pose(&sim).0 - x0
    };
    let one = travel(1);
    let four = travel(4);
    // Quatro tiques andam mais que um — mas nem perto de quatro vezes o
    // empurrão: o que cresce é o TEMPO, não a velocidade entregue.
    assert!(
        four < one * 4.5,
        "quatro tiques num dispatch nao podem multiplicar o empurrao: \
         {one:.3} m em 1 contra {four:.3} m em 4"
    );
    assert!(
        four > one,
        "e mais tempo tem de andar mais: {one:.3} / {four:.3}"
    );
}

/// **A porta esvazia-se ao ser lida, e o mapa não guarda lixo.**
#[test]
fn the_pending_push_is_drained_by_the_tick_that_uses_it() {
    let (mut sim, mut bridge, player) = standing(None);
    assert_eq!(bridge.pending_launch(player), None, "nasce vazia");
    bridge.launch_player(player, PUSH, LOCK);
    assert!(
        bridge.pending_launch(player).is_some(),
        "o pedido fica pendente"
    );
    bridge.set_player_input(player, PlayerInput::default());
    bridge.dispatch(&mut sim, true, 61);
    assert_eq!(
        bridge.pending_launch(player),
        None,
        "e o tique que o usa tem de o consumir"
    );
}

/// **⚠️ E A EXPLOSÃO PASSOU A ALCANÇAR OS TRÊS MODOS** — a correção que torna
/// esta wave alcançável sem UI nova.
///
/// ⚠️ **O mundo de antes está MEDIDO e é a razão de o gate existir:** o
/// `PhysicsWorld::explode` pula todo corpo que não é `Dynamic`, então sob Snap e
/// Pure o estouro alcançava **ZERO** corpos — o botão existia, o toast dizia
/// *"0"*, e o personagem ficava parado ao lado de uma explosão.
///
/// ⚠️ **A contagem E o deslocamento**, e nenhum dos dois basta: contar corpos
/// provaria que a lista passou a incluí-lo (e não que ele se mexeu), e medir só o
/// deslocamento não distinguiria o empurrão de uma queda.
///
/// **Mutações que devem sangrar:** o `blast_players` não somar ao retorno; ele
/// não chamar o `launch_player`; a conversão ignorar a massa (aí o de pose
/// própria voa com o número de outro personagem).
#[test]
fn an_explosion_reaches_the_player_in_all_three_modes() {
    for (mode, tag) in MODES {
        let (mut sim, mut bridge, player) = standing(mode);
        let (x0, y0) = pose(&sim);
        let hit = bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], 6.0, 8.0);
        assert!(hit > 0, "[{tag}] o estouro tem de alcancar o personagem");
        for t in 61..=90_u64 {
            bridge.set_player_input(player, PlayerInput::default());
            bridge.dispatch(&mut sim, true, t);
        }
        let travel = pose(&sim).0 - x0;
        assert!(
            travel > 0.5,
            "[{tag}] e tem de o EMPURRAR, nao so' o contar: {travel:.3} m"
        );
    }
}

/// **A SONDA que dá os números ao handoff** — quanto cada modo anda com o
/// estouro do produto.
///
/// Rode: `cargo test -p ph2d-physics-ecs --test player_launch --release
/// -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_what_the_explosion_now_does() {
    println!("\n== o estouro (raio 6, impulso 8) ao lado do personagem ==");
    for (mode, tag) in MODES {
        let (mut sim, mut bridge, player) = standing(mode);
        let (x0, y0) = pose(&sim);
        let hit = bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], 6.0, 8.0);
        for t in 61..=90_u64 {
            bridge.set_player_input(player, PlayerInput::default());
            bridge.dispatch(&mut sim, true, t);
        }
        println!(
            "  {tag:<8} -> {hit} corpo(s), andou {:>6.3} m em meio segundo",
            pose(&sim).0 - x0
        );
    }
}

/// **Um personagem é UM corpo na contagem** — e este gate existe porque a
/// primeira versão o contava duas vezes.
///
/// ⚠️ O `PhysicsWorld::explode` já conta o player DINÂMICO (ele é um corpo
/// `Dynamic` como outro qualquer), então somar a varredura de players por cima
/// fazia o toast dizer *"2 corpos"* para um personagem sozinho numa cena. O que
/// a varredura acrescenta à contagem é só quem o estouro **não alcançava**.
///
/// **Mutação que deve sangrar:** contar todos os empurrados em vez de só os de
/// pose própria.
#[test]
fn a_dynamic_player_is_counted_once() {
    let (sim, mut bridge, _) = standing(None);
    let (x0, y0) = pose(&sim);
    assert_eq!(
        bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], 6.0, 8.0),
        1,
        "um personagem sozinho na cena e' UM corpo atingido"
    );
}

/// **⚠️ E O ESTOURO É RESISTIDO PELA MASSA** — a lei que o próprio
/// `PhysicsWorld::explode` declara (*a folha voa e o caixote resiste*), agora
/// também para quem tem pose própria.
///
/// ⚠️ **Este gate existe porque uma mutação SOBREVIVEU:** trocar a massa real
/// por `1.0` deixou os seis gates verdes, porque o personagem das fixturas pesa
/// **1,0 kg** — a fixture tornava as duas respostas idênticas. Um personagem
/// pesado é o que as separa.
///
/// ⚠️ **E ele mede o modo SNAP**, que é onde a conversão vive: sob Spring quem
/// divide pela massa é o solver, e o gate estaria a testar o rapier.
#[test]
fn a_heavier_character_is_moved_less_by_the_same_blast() {
    let travel = |mass: Option<f32>| {
        let (mut sim, mut bridge, player) = weighing(Some(PlayerMode::Kinematic), mass);
        let (x0, y0) = pose(&sim);
        bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], 6.0, 8.0);
        for t in 61..=90_u64 {
            bridge.set_player_input(player, PlayerInput::default());
            bridge.dispatch(&mut sim, true, t);
        }
        pose(&sim).0 - x0
    };
    let light = travel(None);
    let heavy = travel(Some(8.0));
    assert!(
        heavy < light * 0.5,
        "oito vezes a massa tem de andar MUITO menos: {light:.3} m contra {heavy:.3} m"
    );
    assert!(heavy > 0.0, "e ainda assim sair do lugar: {heavy:.3} m");
}

/// **SONDA: quanto a massa autorada muda o estouro, nos três modos.**
///
/// ⚠️ **A primeira versão desta sonda dizia que a massa NÃO chegava** (o mesmo
/// `7,106 m` com e sem `MassOverride(8)`), e a culpa era da fixture: ela
/// inseria o componente **depois** de o personagem assentar, e o `reconcile` só
/// re-descreve um corpo **em repouso**. Autorada antes do primeiro tique, a
/// mesma explosão move-o `0,218 m` em vez de `7,106`.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_how_the_authored_mass_changes_the_blast() {
    for (mode, tag) in MODES {
        for m in [None, Some(8.0_f32)] {
            let (mut sim, mut bridge, player) = weighing(mode, m);
            let (x0, y0) = pose(&sim);
            let before = x0;
            bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], 6.0, 8.0);
            for t in 61..=90_u64 {
                bridge.set_player_input(player, PlayerInput::default());
                bridge.dispatch(&mut sim, true, t);
            }
            println!(
                "  {tag:<8} MassOverride {:?}  ->  andou {:.3} m",
                m,
                pose(&sim).0 - before
            );
        }
    }
}
