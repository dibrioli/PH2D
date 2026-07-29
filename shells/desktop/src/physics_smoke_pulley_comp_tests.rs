//! A sonda da cena 63 + o gate que mantém a mensagem dela honesta (W-Pulley W5).

use super::*;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

/// A sonda da cena 63 — o tambor e a cadernal na mesma corda.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_63 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_63() {
    let mut sim = SimWorld::new();
    build_composition(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Geared", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    println!("\n=== CENA 63 — a composicao (2 s) ===");
    println!(
        "{:>8} | {:>12} {:>10} | {:>12} {:>10}",
        "rig", "carga andou", "carga y", "contra andou", "contra y"
    );
    for (i, tag) in ["Geared", "Plain"].iter().enumerate() {
        let ly = y_of(&mut sim, &format!("{tag} Load"));
        let cy = y_of(&mut sim, &format!("{tag} Counterweight"));
        println!(
            "{tag:>8} | {:>12.3} {ly:>10.3} | {:>12.3} {cy:>10.3}",
            ly - start[i],
            cy - COUNTER_Y
        );
    }
}

/// **Os dois gestos do PISO, na cena 63 de verdade** — pelo caminho do produto
/// (`add_pulley_wheel` é a função que o botão chama).
///
/// A medição na bancada (o elevador de `measure_pulley_route_gestures`) prova o
/// mecanismo; esta prova que a CENA que o artista abre sobrevive aos dois gestos,
/// porque uma mensagem de smoke que promete algo não medido é como duas cenas desta
/// linha já afirmaram o que a medição desmentiu.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_63_floor -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_63_floor() {
    /// O maior passo que a carga engrenada dá num tique só, em 2 s.
    fn worst(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> f32 {
        let mut prev = y_of(sim, "Geared Load");
        let mut worst = 0.0f32;
        for t in 1..=120 {
            bridge.dispatch(sim, true, t);
            let now = y_of(sim, "Geared Load");
            worst = worst.max((now - prev).abs());
            prev = now;
        }
        worst
    }
    fn scene() -> (SimWorld, PhysicsBridge) {
        let mut sim = SimWorld::new();
        build_composition(sim.world_mut());
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        (sim, bridge)
    }
    fn rope_bits(sim: &mut SimWorld) -> u64 {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Geared Rope")
            .map(|(e, _)| e.to_bits())
            .expect("a corda engrenada")
    }

    // ⚠️ **O CONTROLE POSITIVO vem antes do número:** um salto igual ao do
    // controle só significa "não deu tranco" se o gesto de fato POUSOU. Sem isto,
    // um `add_pulley_wheel` que desiste em silêncio mede a cena intocada e o
    // resultado é verde sobre nada.
    fn state(sim: &mut SimWorld, bridge: &PhysicsBridge) -> (usize, f32) {
        let e = ph2d_ecs::Entity::from_bits(rope_bits(sim));
        let l0 = sim
            .world()
            .get::<ph2d_physics_ecs::PhysicsJoint>(e)
            .map_or(f32::NAN, |j| j.max_length);
        (bridge.rope_wheels(e).count(), l0)
    }

    println!("\n=== CENA 63 — os dois gestos que o piso cobre (2 s) ===");
    let (mut sim, mut bridge) = scene();
    let base = state(&mut sim, &bridge);
    let control = worst(&mut sim, &mut bridge);
    println!(
        "  controle (ninguem tocou)         roldanas={} L0={:.4}  maior salto = {control:.4} m",
        base.0, base.1
    );

    let (mut sim, mut bridge) = scene();
    let bits = rope_bits(&mut sim);
    crate::render_loop::inspector_joint_wheel::add_pulley_wheel(&mut sim, &bridge, bits);
    bridge.dispatch(&mut sim, false, 0);
    let after = state(&mut sim, &bridge);
    println!(
        "  + Add Wheel                      roldanas={} L0={:.4}  maior salto = {:.4} m{}",
        after.0,
        after.1,
        worst(&mut sim, &mut bridge),
        if after.0 == base.0 {
            "   <-- O GESTO NAO POUSOU"
        } else {
            ""
        }
    );

    let (mut sim, mut bridge) = scene();
    let e = ph2d_ecs::Entity::from_bits(rope_bits(&mut sim));
    if let Some(mut j) = sim.world_mut().get_mut::<ph2d_physics_ecs::PhysicsJoint>(e) {
        j.max_length = 2.0;
    }
    bridge.dispatch(&mut sim, false, 0);
    let after = state(&mut sim, &bridge);
    println!(
        "  + Rope Length = 2.0 (absurdo)    roldanas={} L0={:.4}  maior salto = {:.4} m{}",
        after.0,
        after.1,
        worst(&mut sim, &mut bridge),
        if (after.1 - 2.0).abs() < 1.0e-3 {
            "   <-- O PISO NAO SUBIU"
        } else {
            ""
        }
    );
    println!();
}

/// **A cena 63 diz a verdade** — os dois números da mensagem, medidos pelo
/// caminho do produto.
///
/// O oráculo é a DIFERENÇA entre os dois rigs: a carga e o contrapeso são os
/// mesmos nos dois e os dois têm cadernal móvel, então a única coisa que pode
/// explicar um subir e o outro cair é o segundo diâmetro do tambor MULTIPLICANDO
/// a vantagem que a cadernal já dava.
#[test]
fn the_composition_scene_says_what_happens() {
    let mut sim = SimWorld::new();
    build_composition(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Geared", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    let geared = y_of(&mut sim, "Geared Load") - start[0];
    let plain = y_of(&mut sim, "Plain Load") - start[1];
    /// Folga sobre os números da mensagem — a sim é determinística, então ela
    /// existe só para o gate não ser um fingerprint que qualquer afinação de
    /// solver quebra.
    const SLACK: f32 = 0.3;
    // O SINAL antes da magnitude: as duas cargas são iguais e as duas cadernais
    // também, então andarem para lados opostos é o que só a engrenagem explica.
    assert!(
        geared > 0.0 && plain < 0.0,
        "as MESMAS massas, nas MESMAS talhas, tinham de andar para lados opostos; \
         engrenada {geared:.3} m e comum {plain:.3} m"
    );
    assert!(
        (geared - MEASURED_GEARED_RISE).abs() < SLACK,
        "a mensagem diz que a carga engrenada SOBE {MEASURED_GEARED_RISE:.2} m; ela \
         andou {geared:.3} m"
    );
    assert!(
        (plain + MEASURED_PLAIN_DROP).abs() < SLACK,
        "a mensagem diz que o controle CAI {MEASURED_PLAIN_DROP:.2} m ate o chao; ele \
         andou {plain:.3} m"
    );
    // ⚠️ **A RAZÃO é o oráculo mais forte da cena**, e ela não depende de nenhuma
    // massa: a corda é inextensível, então o lado do contrapeso TEM de andar
    // `2·(R/r)` vezes o que a carga anda, seja qual for a força que a move. Uma
    // afinação de solver move os dois números juntos e deixa a razão de pé.
    let counter = y_of(&mut sim, "Geared Counterweight") - COUNTER_Y;
    let ratio = -counter / geared;
    let mech = 2.0 * R_IN / R_OUT;
    assert!(
        (ratio - mech).abs() < 0.5,
        "a corda e inextensivel: o contrapeso tinha de andar {mech:.1}x o que a \
         carga anda. Carga {geared:.3} m, contrapeso {counter:.3} m, razao \
         {ratio:.2}"
    );
}

/// **O contrapeso do rig engrenado NUNCA alcança o tambor** — o degenerado que o
/// W1 nomeou (um corpo que passa da própria roldana não tem tangente), e que já
/// derrubou a primeira versão da cena 62.
///
/// Ele é o gate que protege a cena de si mesma: a vantagem 8 faz o contrapeso
/// andar **oito vezes** o que a carga anda, então a pista dele é o recurso escasso
/// desta montagem, não uma folga qualquer.
#[test]
fn the_counterweight_never_reaches_its_drum() {
    let mut sim = SimWorld::new();
    build_composition(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut lowest = f32::INFINITY;
    let mut highest = f32::NEG_INFINITY;
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
        for tag in ["Geared", "Plain"] {
            let cy = y_of(&mut sim, &format!("{tag} Counterweight"));
            lowest = lowest.min(cy);
            highest = highest.max(cy);
        }
    }
    // ⚠️ **A barra é o CÍRCULO do tambor, não o centro dele.** A rota tangencia
    // a circunferência, então a âncora entra em degeneração ao cruzar
    // `DRUM_Y − R_IN` — usar o raio do CORPO aqui mediria a coisa errada por
    // 0,25 m e deixaria passar exatamente a falha que este gate existe para
    // pegar.
    assert!(
        highest < DRUM_Y - R_IN,
        "algum contrapeso chegou a {highest:.2}, contra o circulo do tambor que \
         comeca em {:.2}: a rota degenera e a cena passa a mostrar outra coisa",
        DRUM_Y - R_IN
    );
    assert!(
        lowest > 0.0,
        "algum contrapeso desceu a {lowest:.2} e atravessou o chao — a cena tem de \
         caber na pista que ela mesma da"
    );
}

/// **A cena CABE no quadro que ela mesma define** — e este gate existe porque a
/// cena NÃO cabia no quadro padrão.
///
/// ⚠️ **O report que o abriu:** *"selecionar Geared Rope Drum não mostra três
/// alças âmbar"*. A medição inocentou o publicador na primeira sonda (ele devolve
/// as três para aquele tambor) e a causa estava no ENQUADRAMENTO: a câmera padrão
/// mostra `y ∈ [−5, +5]` e o tambor está em **y = 10** — as alças eram desenhadas
/// cinco metros acima do topo da tela.
///
/// ⚠️ **A lição é sobre a ORIGEM dos números.** O `DRUM_Y` foi escolhido por
/// motivo FÍSICO (a pista de que o contrapeso precisa, W5) e um número desses não
/// sabe nada sobre o que cabe na tela; as duas coisas têm de ser reconciliadas por
/// alguém, e um gate é quem lembra. Uma cena de smoke **enquadra o que ela pede
/// que se olhe** — senão ela mostra outra coisa e o artista relata a feature como
/// quebrada.
///
/// O oráculo é a POSE de tudo que a cena spawna com `Transform`, contra o
/// retângulo vertical que a câmera cobre. A largura fica de fora de propósito: ela
/// depende do aspecto da janela, que a cena não conhece.
#[test]
fn the_scene_fits_the_frame_it_sets() {
    let mut sim = SimWorld::new();
    build_composition(sim.world_mut());
    let (top, bottom) = (
        CAMERA_CENTRE[1] + CAMERA_HEIGHT * 0.5,
        CAMERA_CENTRE[1] - CAMERA_HEIGHT * 0.5,
    );
    let worst = crate::physics_smoke_pulley::outside_frame(
        sim.world_mut(),
        CAMERA_CENTRE,
        CAMERA_HEIGHT,
        // O chão é uma faixa de 32 m e o `Transform` dele é o CENTRO: ele existe
        // para ser atravessado, não para ser visto inteiro.
        &["Floor"],
    );
    assert!(
        worst.is_none(),
        "a cena spawna algo fora do quadro que ela define ({bottom:.1}..{top:.1}): \
         {worst:?} — o artista o seleciona na Hierarquia e as alças dele sao \
         desenhadas fora da tela"
    );
    // E o quadro tem de ser APERTADO o bastante para valer a pena: uma altura
    // enorme faria o gate passar mostrando tudo do tamanho de um grão.
}

/// **As alças só existem em REPOUSO, então a cena que as pede nasce PAUSADA.**
///
/// ⚠️ **Este gate nasceu do segundo report do mesmo sintoma** (*"nada visível
/// ainda"*, depois de a hipótese do enquadramento ser corrigida). O publicador é
/// `at_rest`-gated — `at_rest = !playhead.is_playing()` — porque durante o play o
/// overlay desenha a geometria do SOLVER e estas alças autoram a geometria
/// AUTORADA. A cena 63 pede que o artista agarre três alças **e nascia tocando**:
/// elas não podiam existir.
///
/// A medição, pelo mesmo publicador: com a cena real e o tambor selecionado, ele
/// devolve **3** alças em repouso e **0** tocando.
///
/// ⚠️ **`PAUSED_SCENES` é uma ENUMERAÇÃO escrita à mão**, e uma enumeração é o que
/// a cena seguinte nasce fora. As cenas de alça de joint (43-47) estão lá; a minha
/// não estava, e nada me disse. O gate irmão em `shells/desktop/tests/` é quem
/// fecha a classe — este aqui pina a cena.
#[test]
fn the_scene_that_asks_for_handles_starts_paused() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/physics_smoke.rs"))
        .expect("physics_smoke.rs");
    let start = src.find("const PAUSED_SCENES").expect("a lista sumiu");
    let list = &src[start..start + 400];
    let end = list.find("];").expect("a lista não fecha");
    assert!(
        list[..end].contains("\"63\""),
        "a cena 63 manda AGARRAR alças, e alça de roldana é rest-only: nascendo \
         tocando ela não publica nenhuma"
    );
}

/// **E a cena 63 sobrevive aos dois gestos do PISO** — a afirmação que a mensagem
/// faz, no rig que o artista abre.
///
/// A bancada (`ph2d-physics-ecs/tests/pulley_route_floor.rs`) prova o mecanismo com
/// um elevador de 12 m; esta prova a CENA, cuja corda mede 80,9 m e cujo tambor é
/// diferencial — geometria que nenhuma fixture de bancada tem.
///
/// ⚠️ **O controle positivo vem PRIMEIRO, e é o que torna o número legível:** um
/// salto igual ao do controle só significa *não trancou* se o gesto POUSOU. Sem a
/// contagem de roldanas e sem o `L0`, um `add_pulley_wheel` que desistisse em
/// silêncio mediria a cena intocada — verde sobre nada.
#[test]
fn the_composition_scene_survives_the_floors_two_gestures() {
    fn scene() -> (SimWorld, PhysicsBridge) {
        let mut sim = SimWorld::new();
        build_composition(sim.world_mut());
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        (sim, bridge)
    }
    fn rope(sim: &mut SimWorld) -> ph2d_ecs::Entity {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Geared Rope")
            .map(|(e, _)| e)
            .expect("a corda engrenada")
    }
    fn l0(sim: &mut SimWorld) -> f32 {
        let e = rope(sim);
        sim.world()
            .get::<ph2d_physics_ecs::PhysicsJoint>(e)
            .map_or(f32::NAN, |j| j.max_length)
    }
    fn worst(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> f32 {
        let mut prev = y_of(sim, "Geared Load");
        let mut worst = 0.0f32;
        for t in 1..=120 {
            bridge.dispatch(sim, true, t);
            let now = y_of(sim, "Geared Load");
            worst = worst.max((now - prev).abs());
            prev = now;
        }
        worst
    }

    let (mut sim, mut bridge) = scene();
    let wheels_before = bridge.rope_wheels(rope(&mut sim)).count();
    let control = worst(&mut sim, &mut bridge);
    assert!(
        control < 0.05,
        "o CONTROLE ja tranca ({control:.4} m) — a fixture nao mede o defeito"
    );

    // (a) Acrescentar uma roldana pelo caminho do PRODUTO.
    let (mut sim, mut bridge) = scene();
    let bits = rope(&mut sim).to_bits();
    crate::render_loop::inspector_joint_wheel::add_pulley_wheel(&mut sim, &bridge, bits);
    bridge.dispatch(&mut sim, false, 0);
    let wheels_after = bridge.rope_wheels(rope(&mut sim)).count();
    assert!(
        wheels_after == wheels_before + 1,
        "o gesto nao pousou: {wheels_before} roldanas antes, {wheels_after} depois — \
         o numero abaixo mediria a cena intocada"
    );
    let added = worst(&mut sim, &mut bridge);
    assert!(
        added < control * 1.5,
        "Add Wheel trancou a cena: maior salto {added:.4} m contra {control:.4} do controle"
    );

    // (b) E um comprimento impossivel digitado na row.
    let (mut sim, mut bridge) = scene();
    let e = rope(&mut sim);
    if let Some(mut j) = sim.world_mut().get_mut::<ph2d_physics_ecs::PhysicsJoint>(e) {
        j.max_length = 2.0;
    }
    bridge.dispatch(&mut sim, false, 0);
    let floored = l0(&mut sim);
    assert!(
        floored > 10.0,
        "o piso nao subiu: a row ficou em {floored:.4} m sobre uma corda de \
         {MEASURED_GEARED_ROPE_LENGTH:.1} m"
    );
    let typed = worst(&mut sim, &mut bridge);
    assert!(
        typed < control * 1.5,
        "um comprimento impossivel trancou a cena: maior salto {typed:.4} m contra \
         {control:.4} do controle"
    );
}
