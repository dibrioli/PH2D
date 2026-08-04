//! **PULAR** (W4) — os gates de COMPORTAMENTO, com o rapier de verdade.
//!
//! A lei pura tem gates próprios na `ph2d-platformer` (dada uma altura, qual
//! impulso; dada uma fase, qual gravidade). Estes fazem a outra pergunta, a que
//! só a simulação responde: *o personagem de fato SOBE a altura autorada, o
//! toque curto dá um pulo curto, ele não pula duas vezes no ar, e a perna não o
//! puxa de volta no instante da decolagem?*
//!
//! ⚠️ **Nenhum número aqui foi escolhido** — cada barra saiu da sonda
//! `measure_the_jump` (`-- --ignored --nocapture`), que imprime a tabela.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use scene_fixture::{pose, scene};

/// Dirige a cena e devolve a **altura máxima acima do repouso** e a pose final.
///
/// `hold_ticks` é por quantos ticks o botão fica preso a partir do tick 30 (o
/// personagem assenta antes) — `0` significa *nunca apertou*.
fn jump_and_watch(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    player: Entity,
    hold_ticks: u64,
    total: u64,
) -> (f32, f32) {
    // Assenta primeiro: a mola tem de estar em repouso, senão a altura medida
    // carrega a oscilação inicial em vez do pulo.
    let mut tick = 0_u64;
    for _ in 0..30 {
        tick += 1;
        bridge.dispatch(sim, true, tick);
    }
    let (_, rest_y) = pose(sim);

    let mut peak = rest_y;
    for i in 0..total {
        let held = i < hold_ticks;
        bridge.set_player_input(
            player,
            PlayerInput {
                jump: held,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
        let (_, y) = pose(sim);
        peak = peak.max(y);
    }
    let (_, end_y) = pose(sim);
    (peak - rest_y, end_y - rest_y)
}

/// A cena padrão destes gates: chão plano, player em cima.
fn flat() -> (SimWorld, PhysicsBridge, Entity) {
    scene(0.0, 0.0)
}

/// **O gate da wave: a altura AUTORADA é a altura ALCANÇADA.**
///
/// ⚠️ Com os multiplicadores em `1.0` — que é o CONTROLE, e é o único regime em
/// que a promessa `jump_height` vale ao pé da letra: os seis multiplicadores
/// existem precisamente para dobrá-la, então medir a altura com o perfil de
/// produto (`peak 0,5`, `fall 2`) e chamar o resultado de "a altura autorada"
/// seria pinar um número que a lei nunca prometeu.
#[test]
fn the_authored_height_is_the_height_reached() {
    let (mut sim, mut bridge, player) = flat();
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        p.jump_height = 2.0;
        p.takeoff_gravity = 1.0;
        p.peak_gravity = 1.0;
        p.fall_gravity = 1.0;
        p.cut_gravity = 1.0;
    }
    let (peak, _) = jump_and_watch(&mut sim, &mut bridge, player, 200, 200);
    eprintln!("altura autorada 2,00 -> pico medido {peak:.4} m");
    assert!(
        (peak - 2.0).abs() < 0.15,
        "o pico tem de ficar na altura autorada: {peak:.4} contra 2,00"
    );
}

/// **O toque CURTO dá um pulo mais BAIXO** — a altura variável.
///
/// ⚠️ O oráculo é a RAZÃO entre dois pulos da MESMA cena, não uma altura
/// absoluta: a razão é o que o artista percebe, e ela é imune a qualquer deriva
/// do assentamento inicial que uma altura absoluta carregaria junto.
#[test]
fn a_short_tap_gives_a_lower_jump() {
    let full = {
        let (mut sim, mut bridge, player) = flat();
        jump_and_watch(&mut sim, &mut bridge, player, 200, 200).0
    };
    let tap = {
        let (mut sim, mut bridge, player) = flat();
        // Solta depois de 5 ticks — bem dentro da subida.
        jump_and_watch(&mut sim, &mut bridge, player, 5, 200).0
    };
    eprintln!(
        "pulo cheio {full:.4} m · toque curto {tap:.4} m · razao {:.3}",
        tap / full
    );
    assert!(
        tap < full * 0.75,
        "soltar cedo tem de cortar o pulo: {tap:.4} contra {full:.4}"
    );
    assert!(
        tap > 0.15,
        "e ainda tem de ser um PULO, nao um tremor: {tap:.4} m"
    );
}

/// **Segurar não re-pula, e não há pulo duplo no ar.**
///
/// ⚠️ As duas metades num gate só porque são a MESMA lei (a decolagem é na
/// BORDA e exige chão) vista de dois lados; separá-las daria dois gates que
/// passam ou falham sempre juntos.
#[test]
fn holding_does_not_re_jump_and_there_is_no_double_jump() {
    // (a) segurar o tempo todo: um pulo, e ele volta ao chão.
    let (mut sim, mut bridge, player) = flat();
    let (peak, end) = jump_and_watch(&mut sim, &mut bridge, player, 400, 400);
    eprintln!("segurando 400 ticks: pico {peak:.4} m, fim {end:.4} m");
    assert!(
        peak < 4.0,
        "segurar o botao nao pode empilhar impulsos: pico {peak:.4} m"
    );
    assert!(
        end.abs() < 0.2,
        "e ele tem de VOLTAR ao chao: {end:.4} m acima do repouso"
    );

    // (b) tamborilar no ar: nenhum toque depois da decolagem acrescenta altura.
    let drummed = {
        let (mut sim, mut bridge, player) = flat();
        let mut tick = 0_u64;
        for _ in 0..30 {
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
        }
        let (_, rest_y) = pose(&sim);
        let mut peak = rest_y;
        for i in 0..200_u64 {
            // Segura nos 3 primeiros ticks e depois pulsa a cada 4 — cada pulso
            // é uma BORDA nova, que um pulo duplo aceitaria.
            let held = i < 3 || i % 4 == 0;
            bridge.set_player_input(
                player,
                PlayerInput {
                    jump: held,
                    ..PlayerInput::default()
                },
            );
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
            let (_, y) = pose(&sim);
            peak = peak.max(y);
        }
        peak - rest_y
    };
    eprintln!("tamborilando no ar: pico {drummed:.4} m");
    // ⚠️ A barra é o pulo COMPLETO, não ele mais uma folga: um toque no ar
    // acrescenta um `v0` INTEIRO (`√(2g·h)` = 6,26 m/s com esta config), então
    // um pulo duplo passaria o segurado com sobra — não chegaria perto dele.
    // O tamborilado medido fica em 0,83 (ele é um pulo CORTADO, porque o dedo
    // solta no tick 3), e essa distância é o que dá dentes ao gate.
    assert!(
        drummed < peak,
        "tamborilar no ar nao pode subir mais que segurar: {drummed:.4} contra {peak:.4}"
    );
}

/// **A queda é mais rápida que a subida** — o `fall_gravity`, o número que todo
/// platformer carrega.
///
/// ⚠️ O oráculo é o TEMPO de cada metade do arco, não uma velocidade num
/// instante: uma velocidade instantânea depende de onde o tick caiu, e a
/// assimetria do arco é o que o jogador sente.
#[test]
fn the_fall_is_faster_than_the_rise() {
    let (mut sim, mut bridge, player) = flat();
    let mut tick = 0_u64;
    for _ in 0..30 {
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    let (_, rest_y) = pose(&sim);

    let mut rise = 0_u32;
    let mut fall = 0_u32;
    let mut peaked = false;
    let mut prev = rest_y;
    for i in 0..300_u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                jump: i < 200,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        let (_, y) = pose(&sim);
        if y > prev + 1.0e-4 && !peaked {
            rise += 1;
        } else if y < prev - 1.0e-4 {
            peaked = true;
            if y - rest_y > 0.05 {
                fall += 1;
            }
        }
        prev = y;
    }
    eprintln!("subida {rise} ticks · queda {fall} ticks");
    assert!(rise > 5 && fall > 5, "o arco tem de existir: {rise}/{fall}");
    assert!(
        fall < rise,
        "a queda tem de ser mais RAPIDA que a subida: {fall} contra {rise} ticks"
    );
}

/// ⚠️ **A perna CALA enquanto o pulo sobe** — o defeito que este módulo
/// existiria para ter.
///
/// No instante da decolagem o raio **ainda vê o chão** (o personagem não saiu do
/// `cling_distance`), então uma mola viva puxaria de volta exatamente o que o
/// boost acabou de dar. O oráculo tem DUAS metades e nenhuma basta sozinha:
/// alargar o alcance da perna **não muda nada** (a comparação) *e* **o pulo é um
/// pulo** (o piso absoluto).
///
/// ⚠️ A segunda metade nasceu de uma mutação: com a perna VIVA no ar o pulo é
/// estrangulado a **0,1471 m** — mas os DOIS lados são estrangulados igual, então
/// a comparação sozinha fica **verde sobre dois doentes**, que é precisamente a
/// forma de gate que este repo já pagou três vezes.
#[test]
fn the_leg_falls_silent_while_the_jump_climbs() {
    let normal = {
        let (mut sim, mut bridge, player) = flat();
        jump_and_watch(&mut sim, &mut bridge, player, 200, 200).0
    };
    let clingy = {
        let (mut sim, mut bridge, player) = flat();
        {
            let mut e = sim.world_mut().entity_mut(player);
            let mut p = e.get_mut::<PlatformPlayer>().unwrap();
            // O sensor passa a ver o chão por 6 m — mais alto que o pulo inteiro.
            p.cling_distance = 6.0;
        }
        jump_and_watch(&mut sim, &mut bridge, player, 200, 200).0
    };
    eprintln!("pulo normal {normal:.4} m · com cling de 6 m {clingy:.4} m");
    assert!(
        normal > 1.5 && clingy > 1.5,
        "os dois tem de ser PULOS de verdade (a perna calou): {normal:.4} / {clingy:.4}"
    );
    assert!(
        (clingy - normal).abs() < 0.2,
        "alargar o alcance da perna nao pode estrangular o pulo: {clingy:.4} contra {normal:.4}"
    );
}

/// **A tabela** — de onde saem as barras acima e os números da cena `=83`.
///
/// `cargo test -p ph2d-physics-ecs --test platform_jump measure_the_jump -- --ignored --nocapture`
///
/// ⚠️ Ela reporta a altura acima do **TOPO DO CHÃO** (`y = 0,5` nesta fixture),
/// que é a régua que a cena de smoke precisa — o pico acima do REPOUSO, que os
/// gates usam, está 0,9 m acima dela e responderia a outra pergunta.
///
/// ⚠️ E a última coluna é a que decide o desenho da cena: uma saliência só é
/// **alcançável** se o topo dela couber em `pico − float_height`, porque o
/// personagem tem de pousar PAIRANDO sobre ela, não encostado.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_jump() {
    const FLOOR_TOP: f32 = 0.5;
    eprintln!("jump_height | pico acima do chao | saliencia alcancavel");
    for h in [1.0_f32, 1.5, 2.0, 3.0, 4.0] {
        let (mut sim, mut bridge, player) = flat();
        {
            let mut e = sim.world_mut().entity_mut(player);
            let mut p = e.get_mut::<PlatformPlayer>().unwrap();
            p.jump_height = h;
        }
        let (peak_over_rest, _) = jump_and_watch(&mut sim, &mut bridge, player, 200, 200);
        let peak = scene_fixture::FLOAT_HEIGHT + peak_over_rest;
        eprintln!(
            "{h:>10.1} | {peak:>18.2} | {:>20.2}",
            peak - scene_fixture::FLOAT_HEIGHT
        );
    }
    eprintln!("(o topo do chao desta fixture esta em y = {FLOOR_TOP:.1})");
}

/// **Um corpo sem o componente NÃO pula** — o controle da wave inteira.
#[test]
fn a_body_without_the_behaviour_ignores_the_button() {
    let (mut sim, mut bridge, player) = flat();
    sim.world_mut()
        .entity_mut(player)
        .remove::<PlatformPlayer>();
    let (peak, _) = jump_and_watch(&mut sim, &mut bridge, player, 200, 200);
    assert!(
        peak < 0.05,
        "sem o componente o botao nao faz nada: {peak:.4} m"
    );
}
