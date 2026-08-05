//! **O que uma altura agachada ABAIXO do piso geométrico faz** (W18, a medição
//! que abre a wave).
//!
//! `cargo test -p ph2d-physics-ecs --test measure_crouch_floor -- --ignored --nocapture`
//!
//! ⚠️ **Sonda, não gate.** O handoff da W15 deixou este item aberto com a cura já
//! nomeada (*"um readout, não um clamp"*) e **sem número**: *"quem escrever `0,20`
//! numa cápsula de `0,50` vê o corpo enterrado, sem nada a dizer por quê"*. Uma
//! afirmação sobre o que o artista vê tem de ser MEDIDA antes de virar UI.
//!
//! # ⚠️ E a medição REFUTOU a premissa — o corpo não enterra, ele SATURA
//!
//! O solver de contato segura a cápsula exatamente tangente, e o que sobra é
//! **1 mm** na altura mais extrema (`0,10`) — o `normalized_allowed_linear_error`
//! do rapier, o mesmo 1,3 mm que a W2a já mediu e declarou *"não é o que ninguém
//! viu"*. A pose é **perfeitamente estável** (0,0000 m de variação num segundo):
//! nem tremor, nem afundamento, nem nada na tela.
//!
//! **O defeito verdadeiro é outro, e é pior de encontrar:** abaixo do piso o
//! slider é **MORTO**. Medido na rampa, onde o piso é `half_height + radius/cos θ`:
//!
//! | rampa | piso | autorado `0,50` | autorado `0,30` |
//! |---|---|---|---|
//! | 0° | 0,500 | folga 0,002 | 0,000 |
//! | 30° | 0,531 | **0,027** | **0,027** |
//! | 45° | 0,583 | **0,059** | **0,058** |
//!
//! Duzentos milímetros de curso de slider, **um milímetro** de resposta — e a
//! 45° já é assim a partir de `0,50`, que num plano parece perfeitamente bom.
//! É o modo de falha exato de um botão que MENTE, e é o mesmo que fez a perna de
//! pé ganhar o `Fit to Collider (needs > 0.50 m)`: a diferença é que o card do
//! agachar não tem controle nenhum que o resolva.

#[path = "platform_crouch_rig.rs"]
mod rig_fixture;

use rig_fixture::{BODY_HALF, FLOAT_HEIGHT, crouch_right, pose, rig};

/// A que altura o corpo assenta agachado, varrendo a altura autorada por cima e
/// por baixo do piso geométrico da cápsula (`half_height + radius = 0,5`).
#[test]
#[ignore]
fn measure_what_a_crouch_below_the_floor_does() {
    eprintln!("== o piso geometrico do agachar ==");
    eprintln!("  cápsula: half_height 0,3 + radius 0,2 => meia-altura {BODY_HALF:.2}");
    eprintln!("  de pé: float_height {FLOAT_HEIGHT:.2}");
    eprintln!();
    eprintln!("  autorado |  centro  |  base do corpo  |  enterrado");
    eprintln!("  ---------|----------|-----------------|-----------");
    for h in [0.80_f32, 0.60, 0.50, 0.45, 0.40, 0.30, 0.20, 0.10] {
        let mut r = rig(h, None);
        // Tempo de sobra para o transiente da mola assentar.
        r.run(0, 180, crouch_right());
        let (_, y) = pose(&r.sim);
        let base = y - BODY_HALF;
        eprintln!(
            "     {h:.2}   |   {y:.3}  |      {base:+.3}      |  {}",
            if base < -0.001 {
                format!("{:.0} mm", -base * 1000.0)
            } else {
                "nao".to_string()
            }
        );
    }
}

/// **E numa RAMPA**, onde o piso é maior (`half_height + radius / cos θ`).
///
/// ⚠️ A célula do plano mede `0,50`; a de 45° mede `0,583`. Se a saturação
/// acompanhar o piso da RAMPA, então o número que o readout tem de mostrar é o
/// `min_float_height` que a §14 já computa — e não um `half_height + radius`
/// escrito à parte.
#[test]
#[ignore]
fn measure_the_floor_on_a_ramp() {
    use ph2d_ecs::{Name, Transform};
    for deg in [0.0_f32, 30.0, 45.0] {
        let slope = deg.to_radians();
        eprintln!(
            "  -- rampa {deg:.0}° (piso previsto {:.3}) --",
            0.3 + 0.2 / slope.cos()
        );
        for h in [0.80_f32, 0.60, 0.50, 0.30] {
            let mut r = rig(h, None);
            // Inclina o chão DESTE rig — a fixture é a mesma, só a pose do
            // chão muda; uma segunda cena seria uma segunda resposta a *"como é
            // um personagem num chão"*.
            let floor = {
                let mut q = r
                    .sim
                    .world_mut()
                    .try_query::<(ph2d_ecs::Entity, &Name)>()
                    .unwrap();
                let mut f = None;
                for (e, n) in q.iter(r.sim.world()) {
                    if n.as_str() == "Floor" {
                        f = Some(e);
                    }
                }
                f.expect("o chao existe")
            };
            if let Some(mut t) = r.sim.world_mut().get_mut::<Transform>(floor) {
                t.rotation = slope;
            }
            r.run(0, 240, crouch_right());
            let (x, y) = pose(&r.sim);
            // A altura do corpo acima da superfície, medida ao longo da NORMAL.
            let top = 0.5 / slope.cos() + x * slope.tan();
            eprintln!(
                "     autorado {h:.2} => folga acima do chao {:.3}",
                (y - top) * slope.cos()
            );
        }
    }
}

/// **E o que o artista VÊ enquanto anda** — a altura é estável, ou o corpo
/// treme?
///
/// ⚠️ A pergunta importa porque a cura é um READOUT: se o defeito fosse um
/// tremor visível, o artista o encontraria sozinho e um aviso seria redundante.
/// Se o corpo assenta quieto e ENTERRADO, o app sabe uma coisa que a tela não
/// diz.
#[test]
#[ignore]
fn measure_whether_a_buried_crouch_is_visibly_wrong() {
    for h in [0.60_f32, 0.30] {
        let mut r = rig(h, None);
        let mut t = r.run(0, 120, crouch_right());
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for _ in 0..60 {
            t = r.run(t, 1, crouch_right());
            let (_, y) = pose(&r.sim);
            lo = lo.min(y);
            hi = hi.max(y);
        }
        eprintln!(
            "  altura {h:.2}: centro varia {:.4} m ao longo de 1 s (de {lo:.3} a {hi:.3})",
            hi - lo
        );
    }
}
