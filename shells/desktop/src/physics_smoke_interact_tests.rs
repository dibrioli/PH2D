//! **A sonda da cena 53** — os números que a mensagem afirma, medidos sobre as
//! MESMAS peças que o artista abre.
//!
//! `#[ignore]`, como a irmã da cena 52: ela imprime uma tabela, não afirma uma.
//! Roda com
//! `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_smoke_53 -- --ignored --nocapture`.
//!
//! ⚠️ Ela existe porque nesta linha **duas cenas já afirmaram coisas que a medição
//! desmentiu** (a esteira que jogava o caixote fora do mundo; a caixa de densidade
//! neutra que "ficava a meia-água" e ia ao fundo). A regra da política de UI é que
//! a sonda roda ANTES de a mensagem ser escrita.

use super::spawn_props;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{HoldMode, InteractionSettings, InteractionTool, PhysicsBridge};

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    spawn_props(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    // Um segundo de assentamento: a torre e o enxame têm de estar em REPOUSO antes
    // de qualquer medição, senão o que se mede é a queda deles.
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    (sim, bridge)
}

fn by_name(sim: &SimWorld, want: &str) -> ph2d_ecs::Entity {
    let mut q = sim
        .world()
        .try_query::<(ph2d_ecs::Entity, &Name)>()
        .unwrap();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == want)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 53 não tem `{want}`"))
}

fn pos(sim: &SimWorld, e: ph2d_ecs::Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).unwrap();
    [t.translation.x, t.translation.y]
}

/// A distância média dos seis caixotes da torre ao centro deles — o "espalhamento"
/// que *"a torre explode"* significa, e o oráculo robusto: ele não depende de qual
/// caixote se olha nem de o estouro ter batido primeiro num vizinho.
fn tower_spread(sim: &SimWorld) -> f32 {
    let ps: Vec<[f32; 2]> = (1..=6u16)
        .map(|i| pos(sim, by_name(sim, &format!("Tower {i}"))))
        .collect();
    let n = ps.len() as f32;
    let cx = ps.iter().map(|p| p[0]).sum::<f32>() / n;
    let cy = ps.iter().map(|p| p[1]).sum::<f32>() / n;
    ps.iter()
        .map(|p| ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt())
        .sum::<f32>()
        / n
}

/// O raio médio do enxame em torno de um ponto — a grandeza que "juntou"/"abriu"
/// significa, e a única que não depende de qual bolinha se olha.
fn swarm_spread(sim: &SimWorld, centre: [f32; 2]) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for i in 1..=8u16 {
        let e = by_name(sim, &format!("Bit {i}"));
        let p = pos(sim, e);
        sum += ((p[0] - centre[0]).powi(2) + (p[1] - centre[1]).powi(2)).sqrt();
        n += 1.0;
    }
    sum / n
}

#[test]
#[ignore = "sonda de medição: imprime os números da cena 53"]
fn probe_smoke_53() {
    // ── A PRANCHA: os três modos, a atitude depois de 1 s pendurada pela ponta.
    println!("PRANCHA pega pela PONTA, 1 s:");
    for (hold, slack) in [
        (HoldMode::Spring, 0.0_f32),
        (HoldMode::Rigid, 0.0),
        (HoldMode::Rope, 1.5),
    ] {
        let (mut sim, mut bridge) = scene();
        let plank = by_name(&sim, "Plank");
        let grab_at = pos(&sim, plank);
        let tip = [grab_at[0] - 1.2, grab_at[1]];
        let settings = InteractionSettings {
            hold,
            slack,
            ..InteractionSettings::default()
        };
        assert!(bridge.grab_with(plank, tip, settings.hold_spec()));
        // Levanta a ponta **2,5 m** e segura. ⚠️ A 1ª medição levantou 1 m e a corda
        // de slack 1,5 não ficou esticada: a prancha não saiu do chão e o número
        // reportado era a folga sobrando, não o alcance da lei. Uma altura MAIOR que
        // o slack é o que faz a corda dizer algo.
        let target = [tip[0], tip[1] + 2.5];
        for t in 61..=121u64 {
            bridge.move_grab(target);
            bridge.dispatch(&mut sim, true, t);
        }
        let spin = sim.world().get::<Transform>(plank).unwrap().rotation;
        // A distância do PONTO DE PEGA ao cursor — não a do centro, que dista meia
        // prancha por geometria e não diz nada sobre a lei.
        let c = pos(&sim, plank);
        let (sn, cs) = (spin.sin(), spin.cos());
        let held = [c[0] - 1.2 * cs, c[1] - 1.2 * sn];
        let trail = ((held[0] - target[0]).powi(2) + (held[1] - target[1]).powi(2)).sqrt();
        println!(
            "  {hold:?} slack {slack:.1} -> giro {spin:.3} rad · ponto de pega a \
             {trail:.3} m do cursor"
        );
    }

    // ── A TORRE: o estouro no pé dela.
    {
        let (mut sim, mut bridge) = scene();
        let s = InteractionSettings {
            tool: InteractionTool::Explode,
            ..InteractionSettings::default()
        };
        let c = s.clamped();
        let spread_before = tower_spread(&sim);
        let hit = bridge.explode([0.0, 0.3], c.blast_radius, c.blast_impulse);
        // ⚠️ O oráculo é o ESPALHAMENTO depois de 1 s, e a 1ª versão desta sonda
        // media a velocidade de um caixote 1 tick depois do estouro: dentro de uma
        // pilha ele bate nos vizinhos no MESMO tick, então o número reportado era o
        // da colisão e saía invertido (0,13 m/s no de baixo contra 24,57 no de
        // cima). O que o artista vê é a torre se abrindo.
        for t in 61..=121u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        println!(
            "TORRE: estouro raio {:.1} impulso {:.1} em (0, 0.3) -> {hit} corpos\n  \
               espalhamento (dist media ao centro da pilha) {:.2} -> {:.2} m",
            c.blast_radius,
            c.blast_impulse,
            spread_before,
            tower_spread(&sim),
        );
        // E um estouro FORA do alcance.
        let (mut sim2, mut bridge2) = scene();
        let far = bridge2.explode([40.0, 0.3], c.blast_radius, c.blast_impulse);
        bridge2.dispatch(&mut sim2, true, 61);
        println!("  fora do alcance -> {far} corpos");
    }

    // ── O ENXAME: o campo, puxando e repelindo.
    for force in [50.0_f32, -20.0, -50.0] {
        let (mut sim, mut bridge) = scene();
        let centre = [6.65_f32, 1.0];
        let before = swarm_spread(&sim, centre);
        let s = InteractionSettings {
            tool: InteractionTool::Attract,
            attract_radius: 4.0,
            attract_force: force,
            ..InteractionSettings::default()
        };
        bridge.attract(&s, centre);
        for t in 61..=121u64 {
            bridge.move_attract(centre);
            bridge.dispatch(&mut sim, true, t);
        }
        let after = swarm_spread(&sim, centre);
        println!(
            "ENXAME: forca {force:+.0} raio 4,0 por 1 s -> raio medio {before:.2} -> {after:.2} m"
        );
    }

    // ── O MURO MÓVEL: arrastá-lo TOCANDO leva o collider (o bug do fantasma).
    {
        let (mut sim, mut bridge) = scene();
        let ledge = by_name(&sim, "Ledge");
        let witness = by_name(&sim, "Witness");
        let w0 = pos(&sim, witness);
        // O gesto que o artista faz: o gizmo escreve o `Transform` do estático.
        let down = pos(&sim, ledge)[1] - 0.8;
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(ledge) {
            t.translation.y = down;
        }
        for t in 61..=121u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let w1 = pos(&sim, witness);
        println!(
            "MURO MOVEL: 'Ledge' desceu 0,80 m -> a testemunha desceu {:.3} m \
             (de y={:.3} para y={:.3})",
            w0[1] - w1[1],
            w0[1],
            w1[1]
        );
    }
}
