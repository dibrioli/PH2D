//! Os gates da cena 112 (`W-Glide`) — o VÃO, medido nesta geometria.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate corre os DOIS lados: um
//! gate que só afirmasse *"o da direita atravessa"* passaria numa cena cujo vão
//! fosse estreito o bastante para os dois atravessarem.

use super::{
    GAP, GLIDE, LANDING_END, LANE_A, LANE_B, LANE_SPAN, TAKEOFF_END, TAKEOFF_TOP, build_glide_scene,
};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// A cena montada, com o relógio pronto a andar.
fn rig() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    let _ = build_glide_scene(sim.world_mut());
    (sim, PhysicsBridge::new())
}

/// Onde está o personagem chamado `tag`.
fn at(sim: &SimWorld, tag: &str) -> (f32, f32) {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == tag {
            return (t.translation.x, t.translation.y);
        }
    }
    panic!("o personagem {tag} tem de existir");
}

/// Corre para a direita e devolve a pose de cada um **no instante em que o
/// destino dele foi DECIDIDO** — aterrou no patamar, ou caiu no poço.
///
/// ⚠️ **Medir "onde ele está depois de N tiques" é o que a primeira versão
/// fazia, e ela reprovou o produto CORRETO.** O planador atravessava o vão,
/// aterrava na raia-x 20,37 — e depois **continuava a andar** com o dedo preso
/// até sair pela outra ponta do patamar e cair. O gate reportava *"ele caiu"*
/// sobre uma travessia bem-sucedida.
///
/// ⚠️ **O gesto SEGURA o pulo desde o início**, então os dois pulam antes de
/// sair da beira — e é honesto: os DOIS fazem o mesmo, o teclado é um só, e o
/// que a cena contrasta continua a ser o único número que difere.
fn run_both(hold_jump: bool) -> ((f32, f32), (f32, f32)) {
    let (mut sim, mut bridge) = rig();
    let mut ids: Vec<(ph2d_ecs::Entity, String)> = {
        let mut q = sim
            .world_mut()
            .try_query::<(ph2d_ecs::Entity, &Name)>()
            .unwrap();
        q.iter(sim.world())
            .filter(|(_, n)| n.as_str() == "No Glide" || n.as_str() == "Glide")
            .map(|(e, n)| (e, n.as_str().to_string()))
            .collect()
    };
    ids.sort_by(|a, b| a.1.cmp(&b.1));
    let mut done: [Option<(f32, f32)>; 2] = [None, None];
    // ⚠️ **A altura anterior de cada um**, e ela é o que torna a pergunta
    // *"aterrou?"* respondível — ver o comentário no predicado abaixo.
    let mut prev: [f32; 2] = [f32::INFINITY; 2];
    for tick in 1..=600_u64 {
        for (e, _) in &ids {
            bridge.set_player_input(
                *e,
                PlayerInput {
                    drive: 1.0,
                    jump: hold_jump,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, tick);
        for (tag, slot) in [("Glide", 0_usize), ("No Glide", 1)] {
            if done[slot].is_some() {
                continue;
            }
            let p = at(&sim, tag);
            let fell = p.1 < super::PIT_TOP + 2.0;
            // ⚠️ **Aterrar é estar À ALTURA de repouso E PARADO**, e a segunda
            // metade é a que faltava: um corpo a cair para o poço **ATRAVESSA** a
            // altura de repouso do patamar a caminho do fundo, e a primeira
            // versão deste predicado registava essa passagem como uma aterragem
            // — o gate reprovava um vão CORRETO dizendo que o sem-planeio tinha
            // atravessado.
            let still = (p.1 - prev[slot]).abs() < 0.01;
            let landed = (p.1 - (super::LANDING_TOP + super::FLOAT)).abs() < 0.15 && still;
            prev[slot] = p.1;
            if fell || landed {
                done[slot] = Some(p);
            }
        }
        if done.iter().all(Option::is_some) {
            break;
        }
    }
    let fin = |slot: usize, tag: &str| {
        done[slot].unwrap_or_else(|| panic!("o destino de {tag} nunca foi decidido"))
    };
    (fin(1, "No Glide"), fin(0, "Glide"))
}

/// **⚠️ O GATE DA CENA: o vão fica ENTRE o que cada um alcança.**
///
/// ⚠️ **É esta a medição que dimensiona a cena, e ela tem de sair DESTA
/// geometria** — a sonda `measure_the_gap_a_glide_crosses` larga o personagem
/// **parado**, e quem sai a correr de um patamar já leva a velocidade toda. Usar
/// o número dela aqui repetiria o erro que a cena da beirada cometeu (o patamar
/// alto foi calculado com o número do ar livre, e o corpo nunca lá chegava).
///
/// As duas metades importam:
///
/// - o **sem planeio** tem de CAIR no poço — senão a cena não mostra falha;
/// - o **com planeio** tem de ATERRAR no patamar — senão ela não mostra a
///   feature.
#[test]
fn the_gap_is_between_what_each_one_reaches() {
    // Tempo de sobra para os dois resolverem: quem cai no poço já lá está, e
    // quem atravessa já aterrou.
    let (plain, glide) = run_both(true);

    // ⚠️ O `x` é medido dentro da PRÓPRIA raia, senão as duas colunas não são
    // comparáveis.
    let plain_x = plain.0 - LANE_A;
    let glide_x = glide.0 - LANE_B;

    assert!(
        plain.1 < super::PIT_TOP + 2.0,
        "sem planeio ele tem de acabar no POCO (y = {:.2}, o poco esta' em {:.2}); \
         se ele atravessa, o vao de {GAP:.2} m e' estreito de mais e a cena nao \
         mostra falha nenhuma",
        plain.1,
        super::PIT_TOP
    );
    assert!(
        glide.1 > super::LANDING_TOP - 0.5,
        "planando ele tem de ATERRAR no patamar (x = {glide_x:.2}, y = {:.2}); se ele \
         cai, o vao de {GAP:.2} m e' largo de mais e a cena nao mostra a feature",
        glide.1
    );
    assert!(
        glide_x > TAKEOFF_END + GAP,
        "e tem de aterrar do lado de DENTRO da borda do patamar: {glide_x:.2} contra {:.2}",
        TAKEOFF_END + GAP
    );
    assert!(
        plain_x < TAKEOFF_END + GAP,
        "e o outro tem de ficar AQUEM dela: {plain_x:.2} contra {:.2}",
        TAKEOFF_END + GAP
    );
}

/// **Sem o dedo, os DOIS caem** — o passo 2 do roteiro, e o controlo da cena.
///
/// ⚠️ **Sem este gate a cena podia estar a mostrar duas raias com geometrias
/// diferentes** em vez de um número diferente: se o da direita atravessasse
/// mesmo sem apertar nada, o que a cena mostra não seria o planeio.
#[test]
fn without_the_finger_both_of_them_fall_in() {
    let (plain, glide) = run_both(false);
    for (tag, p) in [("No Glide", plain), ("Glide", glide)] {
        assert!(
            p.1 < super::PIT_TOP + 2.0,
            "sem o dedo, {tag} tem de cair no poco: y = {:.2}",
            p.1
        );
    }
}

/// **As duas raias não se tocam** — a geometria de uma não pode alcançar a
/// outra.
#[test]
fn the_two_lanes_do_not_reach_each_other() {
    // ⚠️ Em tempo de COMPILAÇÃO, e uma vez só: a segunda asserção que esta
    // função tinha repetia a primeira em runtime sobre os mesmos dois `const`,
    // e o clippy tinha razão em a chamar de constante.
    const _: () = assert!(LANE_SPAN > LANDING_END);
    const _: () = assert!(LANE_B - LANE_A > LANDING_END);
}

/// **A aritmética que a mensagem imprime está certa** — em tempo de compilação.
#[test]
fn the_scene_prints_the_numbers_it_builds() {
    const _: () = assert!(TAKEOFF_TOP > super::LANDING_TOP);
    const _: () = assert!(super::PIT_TOP < super::LANDING_TOP);
    const _: () = assert!(TAKEOFF_END + GAP < LANDING_END);
    const _: () = assert!(GLIDE > 0.0);
    assert!(super::GLIDE_SMOKE_MESSAGE.contains("O VAO (W-Glide)"));
}

/// **A SONDA que dimensiona o vão** — onde cada um cruza o nível de aterragem.
///
/// ⚠️ **Ela mede ESTA geometria, com ESTE gesto**, e existe porque as duas
/// tentativas anteriores de escolher o vão falharam pelo mesmo motivo: o número
/// veio de outra fixture. A primeira veio da sonda que larga o personagem
/// **parado** (4,18 m sem planeio) — mas quem corre de um patamar **e pula**
/// leva a velocidade toda e vai muito mais longe.
///
/// Rode: `cargo test -p ph2d-host-desktop --release --bins
/// where_each_one_crosses -- --ignored --nocapture`
#[test]
#[ignore = "sonda de dimensionamento"]
fn where_each_one_crosses_the_landing_level() {
    // ⚠️ **Sem patamar de aterragem no caminho**, senão o que se mede é onde ele
    // aterrou e não até onde ele chegava.
    let mut sim = SimWorld::new();
    let _ = build_glide_scene(sim.world_mut());
    // Apaga os dois patamares de aterragem.
    let doomed: Vec<ph2d_ecs::Entity> = {
        let mut q = sim
            .world_mut()
            .try_query::<(ph2d_ecs::Entity, &Name)>()
            .unwrap();
        q.iter(sim.world())
            .filter(|(_, n)| n.as_str().ends_with("Landing"))
            .map(|(e, _)| e)
            .collect()
    };
    for e in doomed {
        sim.world_mut().despawn(e);
    }
    let mut bridge = PhysicsBridge::new();
    let ids: Vec<(ph2d_ecs::Entity, String)> = {
        let mut q = sim
            .world_mut()
            .try_query::<(ph2d_ecs::Entity, &Name)>()
            .unwrap();
        q.iter(sim.world())
            .filter(|(_, n)| n.as_str() == "No Glide" || n.as_str() == "Glide")
            .map(|(e, n)| (e, n.as_str().to_string()))
            .collect()
    };
    let mut crossed: [Option<f32>; 2] = [None, None];
    for tick in 1..=900_u64 {
        for (e, _) in &ids {
            bridge.set_player_input(
                *e,
                PlayerInput {
                    drive: 1.0,
                    jump: true,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, tick);
        for (tag, lane, slot) in [("No Glide", LANE_A, 0_usize), ("Glide", LANE_B, 1)] {
            if crossed[slot].is_none() {
                let p = at(&sim, tag);
                if p.1 <= super::LANDING_TOP {
                    crossed[slot] = Some(p.0 - lane);
                }
            }
        }
        if crossed.iter().all(Option::is_some) {
            break;
        }
    }
    println!("\n== onde cada um cruza o nivel de aterragem (correndo, pulo preso) ==");
    println!("  a beira do patamar de saida esta' em x = {TAKEOFF_END:.2}");
    for (tag, slot) in [("sem planeio", 0_usize), ("planando", 1)] {
        match crossed[slot] {
            Some(x) => println!(
                "  {tag:<12} cruza em x = {x:>6.2}  (vao atravessado: {:>6.2} m)",
                x - TAKEOFF_END
            ),
            None => println!("  {tag:<12} nunca cruzou"),
        }
    }
    println!("\n(o GAP da cena tem de ficar ENTRE os dois vaos atravessados)");
}

/// **SONDA de diagnóstico: onde é que ele fica, PARADO e sem dedo nenhum.**
///
/// Report do smoke: *"os players ficam dando pulinhos discretos sozinhos (sem
/// input)"*. A pergunta que separa as duas curas opostas é a AMPLITUDE — um pulo
/// de verdade sobe metros, uma mola a oscilar sobe milímetros.
#[test]
#[ignore = "sonda de diagnostico"]
fn where_the_idle_player_stands() {
    let (mut sim, mut bridge) = rig();
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    println!("\n== o personagem PARADO, sem input nenhum (cena 112) ==");
    // ⚠️ **A CADÊNCIA do produto, e não um tique por quadro:** o app calcula
    // `target = round(tempo / dt)` a cada QUADRO, e um monitor a 144 Hz produz
    // dois ou três quadros para o mesmo tique. É a única variável que o probe
    // anterior não tinha.
    let mut time = 0.0_f64;
    for f in 1..=720_u64 {
        time += 1.0 / 144.0;
        let t = (time * 60.0).round() as u64;
        bridge.dispatch(&mut sim, true, t);
        let (_, y) = at(&sim, "Glide");
        lo = lo.min(y);
        hi = hi.max(y);
        if f % 48 == 0 {
            println!("  quadro={f:>3} tique={t:>3}  y={y:>8.4}");
        }
    }
    println!("  amplitude ao longo de 300 tiques: {:.4} m", hi - lo);
    println!(
        "  (o topo do patamar esta' em {TAKEOFF_TOP:.2}; o repouso e' {:.2})",
        TAKEOFF_TOP + super::FLOAT
    );
}

/// **SONDA de diagnóstico: o personagem PARADO perto da BEIRA.**
///
/// ⚠️ **É a hipótese que a fixture do probe irmão não continha:** ele mede o
/// corpo no MEIO do patamar, onde os três raios do leque de pés vêem o mesmo
/// chão. Na beira eles vêem chãos diferentes, e uma perna cuja força salta entre
/// dois valores é exactamente o que *"pulinhos discretos sozinhos"* descreve.
#[test]
#[ignore = "sonda de diagnostico"]
fn how_still_he_stands_near_the_edge() {
    println!("\n== o personagem PARADO, sem input, a VARIAS distancias da beira ==");
    println!("   (a beira do patamar de saida esta' em x = {TAKEOFF_END:.2})");
    for back in [1.50_f32, 0.50, 0.30, 0.20, 0.15, 0.10, 0.05, 0.00] {
        let (mut sim, mut bridge) = rig();
        {
            let e = {
                let mut q = sim
                    .world_mut()
                    .try_query::<(ph2d_ecs::Entity, &Name)>()
                    .unwrap();
                q.iter(sim.world())
                    .find(|(_, n)| n.as_str() == "Glide")
                    .map(|(e, _)| e)
                    .expect("player")
            };
            let mut t = sim.world_mut().get_mut::<Transform>(e).expect("t");
            t.translation.x = LANE_B + TAKEOFF_END - back;
        }
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        let mut last = 0.0_f32;
        for t in 1..=240_u64 {
            bridge.dispatch(&mut sim, true, t);
            let (_, y) = at(&sim, "Glide");
            // Ignora o assentar inicial: só os últimos 180 tiques.
            if t > 60 {
                lo = lo.min(y);
                hi = hi.max(y);
            }
            last = y;
        }
        println!(
            "  recuado {back:>4.2} m da beira  ->  amplitude {:>7.4} m   (y final {last:>7.3})",
            hi - lo
        );
    }
}

/// **SONDA de diagnóstico: o dedo PRESO no pulo, em chão firme.**
///
/// ⚠️ **É o gesto que o próprio roteiro manda fazer** (passo 3: *"corra e SEGURE
/// o pulo"*), e quem falha o vão aterra no poço com o dedo ainda preso. Se um
/// botão SEGURADO re-dispara o pulo, o que se vê é o personagem a saltitar no
/// fundo do poço — *"pulinhos discretos"*.
#[test]
#[ignore = "sonda de diagnostico"]
fn how_many_times_a_held_button_jumps() {
    let (mut sim, mut bridge) = rig();
    let e = {
        let mut q = sim
            .world_mut()
            .try_query::<(ph2d_ecs::Entity, &Name)>()
            .unwrap();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Glide")
            .map(|(e, _)| e)
            .expect("player")
    };
    println!("\n== o dedo PRESO no pulo, em chao firme (300 tiques) ==");
    let mut prev = at(&sim, "Glide").1;
    let mut rises = 0_u32;
    let mut was_rising = false;
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for t in 1..=300_u64 {
        bridge.set_player_input(
            e,
            PlayerInput {
                jump: true,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        let y = at(&sim, "Glide").1;
        let rising = y > prev + 1.0e-4;
        if rising && !was_rising {
            rises += 1;
            println!("  t={t:>3}  SUBIDA #{rises} comeca em y={y:.4}");
        }
        was_rising = rising;
        prev = y;
        lo = lo.min(y);
        hi = hi.max(y);
    }
    println!("  subidas contadas: {rises}   amplitude: {:.4} m", hi - lo);
}
