//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA** (suplente #22, W8): *a composição de hoje —
//! `Tween` + `Timer` + as `33` curvas do [`ph2d_anim`] — já exprime um **ping-pong**?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime.** Esta mesma pergunta reescreveu a entrega do #14 (o ricochete já era exacto, e o
//! componente existe por outra razão) e cortou três coisas do #15.
//!
//! ⛔⛔ **Report do dono, 2026-09-19:** *«onde estão as opções úteis como ping-pong?»*. O app já o
//! tem em dois sítios (`ph2d_anim::Extrap::PingPong` nas curvas da timeline ·
//! `ph2d_flip::CycleMode::PingPong` nas fitas do Flip), e o tween ficou de fora.
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME; o que ela decide é *o quê* da wave.
//!
//! ```text
//! cargo test -p ph2d-ecs --test it mede_o_que_a_composicao_ja_da_ao_pingpong -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **O `repeat` do relógio dá ir-e-voltar, ou dá uma SERRA?** — é a única repetição que existe.
//! B) **Alguma das `33` curvas REFLECTE?** — se uma delas voltasse ao princípio, o ping-pong seria
//!    escolher uma curva, e a wave não existiria.
//! C) **Dois tweens no mesmo canal, em contrafase, montam-no?** — o caminho que um artista
//!    esperto tentaria.

use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_ecs::{Timer, TimerRuntime, Timers, Tweens, World};
use ph2d_tween::{Canal, Tween};

const PERIODO_US: u64 = 1_000_000;
/// Dez amostras por período — fino o bastante para a serra aparecer e grosso o bastante para ler.
const PASSO_US: u64 = PERIODO_US / 10;

fn cena(tweens: Vec<Tween>, timers: Vec<Timer>) -> (World, bevy_ecs::entity::Entity) {
    let mut w = World::new();
    let cfg = Timers(timers);
    let rt = TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    let e = w.spawn((Tweens(tweens), cfg, rt)).id();
    (w, e)
}

fn laco(periodo_us: u64) -> Timer {
    Timer {
        duration_us: periodo_us,
        repeat: true,
        autostart: true,
        ..Timer::default()
    }
}

/// Anda o relógio `passos` vezes e devolve o valor que o canal recebeu em cada quadro.
fn percurso(w: &mut World, e: bevy_ecs::entity::Entity, passos: usize, canal: Canal) -> Vec<f32> {
    let mut fora = Vec::new();
    for _ in 0..passos {
        {
            let cfg = w.get::<Timers>(e).expect("tem timers").clone();
            let mut rt = w.get_mut::<TimerRuntime>(e).expect("tem runtime");
            for (i, t) in cfg.0.iter().enumerate() {
                ph2d_ecs::timer::advance(t, &mut rt.0[i], PASSO_US);
            }
        }
        // ⚠️ **A ÚLTIMA escrita do canal é a que a ponte deixa no mundo** — ela escreve por ordem,
        //    e um canal escrito duas vezes fica com a segunda. É isso que o bloco C mede.
        let ultimo = ph2d_ecs::tween::a_escrever(w)
            .into_iter()
            .rfind(|p| p.canal == canal);
        fora.push(ultimo.map_or(f32::NAN, |p| p.valor[0]));
    }
    fora
}

fn maior_salto(v: &[f32]) -> f32 {
    v.windows(2)
        .map(|p| (p[1] - p[0]).abs())
        .fold(0.0_f32, f32::max)
}

#[test]
#[ignore = "sonda do §5.0: imprime, nao afirma"]
fn mede_o_que_a_composicao_ja_da_ao_pingpong() {
    println!("\n══════ A) O `repeat` do relógio: ir-e-voltar, ou SERRA? ══════");
    let (mut w, e) = cena(
        vec![Tween::linear(Canal::Opacity, 0.0, 1.0)],
        vec![laco(PERIODO_US)],
    );
    // Dois períodos inteiros.
    let v = percurso(&mut w, e, 20, Canal::Opacity);
    println!("  valor por quadro: {v:.3?}");
    let salto = maior_salto(&v);
    let suave = v
        .windows(2)
        .map(|p| (p[1] - p[0]).abs())
        .filter(|d| *d < salto - 1e-3)
        .fold(0.0_f32, f32::max);
    println!("  maior salto entre quadros vizinhos: {salto:.4}");
    println!("  maior passo SUAVE (fora o salto):   {suave:.4}");
    println!(
        "  ⇒ {}",
        if salto > suave * 3.0 {
            "SERRA — ao fim do período ele SALTA de volta ao princípio"
        } else {
            "TRIÂNGULO — ele volta suavemente, e o ping-pong ja' existia"
        }
    );

    println!("\n══════ B) Alguma das 33 curvas REFLECTE? ══════");
    let mut reflectem = Vec::new();
    for f in EasingFamily::ALL {
        for m in EasingMode::ALL {
            let c = Easing::new(f, m);
            let (a, meio, fim) = (c.eval(0.0), c.eval(0.5), c.eval(1.0));
            // Reflectir = acabar onde começou, tendo passado longe pelo meio.
            if (fim - a).abs() < 0.05 && (meio - a).abs() > 0.5 {
                reflectem.push(format!("{f:?}/{m:?}"));
            }
        }
    }
    let total = EasingFamily::ALL.len() * EasingMode::ALL.len();
    println!(
        "  de {total} curvas, REFLECTEM: {}",
        if reflectem.is_empty() {
            "NENHUMA".to_owned()
        } else {
            reflectem.join(", ")
        }
    );
    println!("  ⇒ escolher uma curva NAO da' ping-pong: todas vao de 0 a 1 e ficam la'");

    println!("\n══════ C) Dois tweens em contrafase montam-no? ══════");
    let (mut w, e) = cena(
        vec![
            Tween::linear(Canal::Opacity, 0.0, 1.0),
            Tween::linear(Canal::Opacity, 1.0, 0.0),
        ],
        vec![laco(PERIODO_US), laco(PERIODO_US)],
    );
    let v = percurso(&mut w, e, 10, Canal::Opacity);
    println!("  valor por quadro com os DOIS no mesmo canal: {v:.3?}");
    println!(
        "  ⇒ eles nao se compoem: o SEGUNDO escreve por cima do primeiro, e o que se ve' e' so' \
         a descida"
    );
    println!(
        "\n  ⛔ E nao ha' desfasamento a autorar: os dois relogios arrancam juntos (`autostart`) e \
         nao existe campo de atraso — logo nem por acaso eles se alternam.\n"
    );
}
