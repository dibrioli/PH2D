//! **A varredura das paredes** (W13) — os números que a wave escreveu saíram
//! daqui, não da cabeça de ninguém (CLAUDE.md §0).
//!
//! `cargo test -p ph2d-physics-ecs --test measure_wall -- --ignored --nocapture`
//!
//! ⚠️ **Sondas, não gates:** elas imprimem e não afirmam. Um gate que
//! cronometrasse estas tabelas mediria a máquina; o que fica gateado é a
//! PROPRIEDADE (o `platform_wall`), e o que fica aqui é a tabela que escolheu
//! cada constante.

#[path = "platform_wall_rig.rs"]
mod rig_fixture;

use ph2d_physics_ecs::PlayerInput;
use rig_fixture::{Rig, START_Y, into_wall, pose, rig};

/// Empurra contra a parede por `ticks` e devolve a altura.
fn slide_for(r: &mut Rig, ticks: u64) -> f32 {
    r.bridge.set_player_input(r.player, into_wall());
    let mut t = 0;
    for _ in 0..ticks {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    pose(&r.sim).1
}

/// **A COLA que o escorregamento existe para substituir.**
///
/// ⚠️ Esta é a tabela que derrubou a primeira versão da lei. Ela era um TETO
/// (*"não caia mais rápido do que isto"*), escrita por raciocínio; a linha
/// `desligado` mostra que **o personagem pressionado contra uma parede não
/// cai** — logo não há queda a frear, e o knob seria um número que não faz nada.
///
/// As duas causas são do produto que já shipa: o **atrito**
/// (`DEFAULT_FRICTION = 0,5`) contra a normal que o controle aéreo sustenta, e a
/// **gravidade do ÁPICE** (`peak_gravity = 0,5`), que corta metade do peso
/// justamente na janela de velocidade em que o personagem colado vive — um
/// estado auto-reforçante.
#[test]
#[ignore]
fn measure_the_stick_and_the_slide() {
    eprintln!("== 1 segundo empurrando contra a parede (queda a partir de {START_Y:.1}) ==");
    for speed in [0.0_f32, 1.0, 3.0, 6.0, 12.0] {
        let y = slide_for(&mut rig(speed, 0.0), 60);
        let label = if speed == 0.0 { " (desligado)" } else { "" };
        eprintln!(
            "  slide_speed {speed:5.1} -> y {y:7.3}  desceu {:6.3} m{label}",
            START_Y - y
        );
    }
    eprintln!(
        "\n⚠️ A linha do 0.0 e' a COLA: sem a assistencia ele NAO desce.\n\
         E' por isso que a lei DEFINE a velocidade em vez de a limitar."
    );
}

/// **O silêncio do controle aéreo depois de um pulo de parede.**
///
/// ⚠️ **A tabela que escolheu o `jump_lockout = 0,2 s`.** As duas colunas medem
/// coisas diferentes e as duas importam: a ALTURA diz quanto do pulo autorado
/// sobrevive (o atrito da raspagem come o topo), e o AFASTAMENTO diz se o
/// personagem chega à parede seguinte — que é a razão de existir do gesto.
///
/// ## Medido (2026-08-05, altura autorada 2,0 m)
///
/// | `jump_lockout` | subiu | % de 2,0 | afastou |
/// |---|---|---|---|
/// | 0,00 s | 1,621 m | 81% | 0,462 m |
/// | 0,05 s | 1,833 m | 92% | 0,737 m |
/// | 0,10 s | 1,921 m | 96% | 1,137 m |
/// | **0,20 s** | **1,932 m** | **97%** | **1,737 m** |
/// | 0,30 s | 1,932 m | 97% | 2,237 m |
/// | 0,50 s | 1,932 m | 97% | 3,437 m |
///
/// ⚠️ **Uma frase minha morreu nesta tabela:** eu escrevi que *"o afastamento
/// satura, e é onde a constante mora"*. Ele **não satura** — cresce linear,
/// porque enquanto o controle está calado nada freia a horizontal. Quem satura
/// é a **ALTURA**, em 0,10-0,20 s. A constante sai dali: é onde a assistência
/// para de perder o pulo, e cada décimo além disso é controle tirado do jogador
/// comprando alcance.
#[test]
#[ignore]
fn measure_the_wall_jump_lockout() {
    eprintln!(
        "== pulo de parede, altura autorada 2,0 m, com o jogador a SEGURAR a direcao da parede =="
    );
    for lockout in [0.0_f32, 0.05, 0.1, 0.2, 0.3, 0.5] {
        let mut r = rig(3.0, 2.0);
        r.player_cfg(|p| p.wall_jump_lockout = lockout);
        // Agarra-se.
        r.bridge.set_player_input(r.player, into_wall());
        let mut t = 0;
        for _ in 0..30 {
            t += 1;
            r.bridge.dispatch(&mut r.sim, true, t);
        }
        let (x0, y0) = pose(&r.sim);
        // Pula, segurando o botão (soltar cedo corta o pulo — `cut_gravity`) e
        // continuando a empurrar para a parede, que é o que um jogador faz.
        r.bridge.set_player_input(
            r.player,
            PlayerInput {
                drive: 1.0,
                jump: true,
                down: false,
            },
        );
        let (mut peak, mut far) = (y0, x0);
        for k in 0..45 {
            t += 1;
            r.bridge.dispatch(&mut r.sim, true, t);
            if k == 24 {
                r.bridge.set_player_input(r.player, into_wall());
            }
            let (x, y) = pose(&r.sim);
            peak = peak.max(y);
            far = far.min(x);
        }
        eprintln!(
            "  lockout {lockout:4.2}s -> subiu {:5.3} m ({:3.0}% de 2,0)   afastou {:5.3} m",
            peak - y0,
            (peak - y0) / 2.0 * 100.0,
            x0 - far
        );
    }
    eprintln!(
        "\n⚠️ Sem silencio o controle aereo puxa o personagem de volta e o atrito\n\
         come o topo do pulo.\n\
         ⚠️ QUEM SATURA E' A ALTURA (97% ja' em 0,10-0,20 s). O AFASTAMENTO nao\n\
         satura -- ele cresce LINEAR, porque enquanto o controle esta' calado nada\n\
         freia a horizontal. Logo a constante nao sai de um joelho no afastamento:\n\
         ela sai de onde a ALTURA para de ser perdida, e depois disso cada decimo\n\
         de segundo e' controle tirado do jogador em troca de alcance."
    );
}
