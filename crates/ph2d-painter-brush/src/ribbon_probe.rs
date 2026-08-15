//! **O QUE A FITA FAZ, E O QUE ELA CUSTA** — as medições que decidem os números do
//! [`crate::line_kind::RIBBON_DAMPING_MIN`] e companhia (plano 38 W6).
//!
//! ⚠️ **O recurso desta wave não é o relógio, é TINTA.** A fita percorre caminho no TIQUE, e o
//! tique corre enquanto o botão está preso — então uma fita que nunca assenta **pinta para sempre
//! com a mão parada**. É esse número que o piso do amortecimento existe para limitar, e é ele que a
//! primeira sonda mede.
//!
//! Rodar: `cargo test -p ph2d-painter-brush --release measure_the_ribbon -- --ignored --nocapture`

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::{LineKind, RIBBON_DAMPING_MAX, RIBBON_DAMPING_MIN};
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn spec(weight: f32, friction: f32, gravity: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: LineKind::Ribbon,
        ribbon_weight: weight,
        ribbon_friction: friction,
        ribbon_gravity: gravity,
        ..Default::default()
    }
}

fn plain() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// Um traço RETO a `speed` px/s por `secs` segundos, a 60 fps, com o tique do produto entre os
/// eventos. Devolve `(dabs do traço, atraso final em px, dabs da CAUDA no pen-up)`.
fn straight(sp: BrushSpec, speed: f32, secs: f32) -> (usize, f32, usize) {
    straight_at_rate(sp, speed, secs, 1)
}

/// O mesmo gesto com `per_frame` amostras de ponteiro por quadro — um mouse de 60 Hz contra um de
/// 960 Hz. ⚠️ **O relogio do tique NAO muda**: o que muda e a finura da ESCADA que o dedo desenha.
fn straight_at_rate(sp: BrushSpec, speed: f32, secs: f32, per_frame: usize) -> (usize, f32, usize) {
    let dt = 1.0 / 60.0;
    let mut s = Stroke::new(sp, plain(), 7);
    let mut out = Vec::new();
    let mut n = 0usize;
    let start = [100.0f32, 300.0];
    s.begin(
        StrokePoint {
            pos: start,
            pressure: 1.0,
        },
        &mut out,
    );
    n += out.len();
    out.clear();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let frames = (secs / dt) as usize;
    let mut x = start[0];
    let mut last = [0.0f32, 0.0];
    for _ in 0..frames {
        out.clear();
        #[allow(clippy::cast_precision_loss)]
        for _ in 0..per_frame.max(1) {
            x += speed * dt / per_frame.max(1) as f32;
            s.extend(
                StrokePoint {
                    pos: [x, start[1]],
                    pressure: 1.0,
                },
                &mut out,
            );
            n += out.len();
            if let Some(d) = out.last() {
                last = d.center;
            }
            out.clear();
        }
        // ⚠️ **AS DUAS saidas sao lidas, e a 1ª versao lia so a segunda.** O contrato do motor e
        // que **`Stroke::tick` ABRE com `out.clear()`** (`stroke.rs:464`): o shell entrega um
        // scratch por chamada. Ler o ultimo dab so depois do tique da a resposta certa para uma
        // FITA (ali quem percorre o caminho e o tique) e **vazia** para todo outro traco, onde quem
        // emite e o `extend`. O CONTROLE denunciou: o neutro media **4860 px de atraso** num gesto
        // de 4800 px, que e o traco inteiro -- ou seja, nao media nada.
        s.tick(dt, &mut out);
        n += out.len();
        if let Some(d) = out.last() {
            last = d.center;
        }
    }
    let lag = x - last[0];
    s.finish(&mut out);
    (n, lag, out.len())
}

/// **O PISO DO AMORTECIMENTO** — quanta tinta uma fita deixa com a MÃO PARADA e o botão preso.
///
/// ⚠️ A pergunta não é *"quanto tempo ela balança?"* e sim *"quantos dabs ela deposita sem o artista
/// fazer nada?"* — o segundo é o que o artista vê e o que o orçamento paga.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_ribbon_settling_ink() {
    let dt = 1.0 / 60.0;
    println!("[ribbon] mao PARADA, botao preso por 30 s (1800 tiques), peso 1,0 (tau 0,25 s)");
    println!(
        "{:>9} {:>7}  {:>10} {:>12}",
        "friction", "zeta", "dabs", "silencio s"
    );
    for f in [0.0f32, 0.02, 0.05, 0.08, 0.15, 0.3, 0.5, 1.0] {
        let zeta = RIBBON_DAMPING_MIN + f * (RIBBON_DAMPING_MAX - RIBBON_DAMPING_MIN);
        // ⚠️ A fita tem de estar DESLOCADA para haver o que assentar: 400 px de gesto rápido, e só
        // depois a mão para. Uma fita que nasce sob o dedo não balança, e a sonda mediria zero.
        let mut s = Stroke::new(spec(1.0, f, 0.0), plain(), 7);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [100.0, 300.0],
                pressure: 1.0,
            },
            &mut out,
        );
        for i in 1..=20 {
            #[allow(clippy::cast_precision_loss)]
            let x = 100.0 + (i as f32) * 20.0;
            s.extend(
                StrokePoint {
                    pos: [x, 300.0],
                    pressure: 1.0,
                },
                &mut out,
            );
            s.tick(dt, &mut out);
        }
        // A mão PARA aqui: nenhum `extend` a mais, só o relógio.
        let mut parked = 0usize;
        // Quantos TIQUES depois de a mão parar o último dab caiu — o "tempo até o silêncio".
        let mut silence = 0usize;
        let mut t = 0usize;
        for _ in 0..1800 {
            s.tick(dt, &mut out);
            t += 1;
            if !out.is_empty() {
                silence = t;
            }
            parked += out.len();
        }
        println!("{f:>9.2} {zeta:>7.2}  {parked:>10} {:>12.2}", silence as f32 * dt);
    }
    println!("[ribbon] leitura: o piso e o menor amortecimento cujo silencio chega em tempo de gesto.");
}

/// **A LEI** — o atraso cresce com a VELOCIDADE do gesto, que é o que *pesar* significa.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_ribbon_lag_against_speed() {
    println!("[ribbon] traco reto de 2 s, peso 0,45 (tau = 0,1125 s), friction 0,30");
    println!("{:>10} {:>10} {:>12}", "px/s", "atraso px", "atraso/tau");
    for v in [300.0f32, 600.0, 1200.0, 2400.0, 4800.0] {
        let (_, lag, _) = straight(spec(0.45, 0.30, 0.0), v, 2.0);
        println!("{v:>10.0} {lag:>10.1} {:>12.3}", lag / (v * 0.1125));
    }
    println!("[ribbon] leitura: `atraso/tau` proximo de 1 => a fita atrasa `velocidade x tau`.");
}

/// **O CUSTO** — quanto uma fita cobra por segundo de traço, contra o kill de 8 ms/evento.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_ribbon_dab_budget() {
    println!("[ribbon] dabs de 2 s de traco + a CAUDA do pen-up, por peso");
    println!(
        "{:>7} {:>8}  {:>9} {:>10} {:>9}",
        "weight", "friction", "dabs", "atraso px", "cauda"
    );
    for w in [0.0f32, 0.25, 0.45, 0.7, 1.0] {
        for f in [0.05f32, 0.30, 1.0] {
            let (n, lag, tail) = straight(spec(w, f, 0.0), 2400.0, 2.0);
            println!("{w:>7.2} {f:>8.2}  {n:>9} {lag:>10.1} {tail:>9}");
        }
    }
    println!("[ribbon] leitura: o controle e `weight 0` (a fita desarmada, o traco de sempre).");
}

/// **A GRAVIDADE** — quanto a fita PENDE em repouso, para o teto ser um número e não um palpite.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_ribbon_hang() {
    let dt = 1.0 / 60.0;
    println!("[ribbon] mao PARADA por 3 s: quanto a fita desce sob a gravidade");
    println!("{:>7} {:>9}  {:>12} {:>12}", "weight", "gravity", "queda px", "previsto g*tau2");
    for w in [0.25f32, 0.45, 1.0] {
        for g in [0.25f32, 0.5, 1.0] {
            let sp = spec(w, 0.5, g);
            let tau = sp.ribbon_lag_s();
            let predicted = sp.ribbon_gravity_px_s2() * tau * tau;
            let mut s = Stroke::new(sp, plain(), 7);
            let mut out = Vec::new();
            let at = [200.0f32, 300.0];
            s.begin(
                StrokePoint {
                    pos: at,
                    pressure: 1.0,
                },
                &mut out,
            );
            let mut last = at;
            for _ in 0..180 {
                s.extend(
                    StrokePoint {
                        pos: at,
                        pressure: 1.0,
                    },
                    &mut out,
                );
                s.tick(dt, &mut out);
                if let Some(d) = out.last() {
                    last = d.center;
                }
            }
            println!(
                "{w:>7.2} {g:>9.2}  {:>12.1} {predicted:>12.1}",
                last[1] - at[1]
            );
        }
    }
}

/// **O PISO DO ATRASO** — abaixo de que `τ` a fita deixa de desenhar algo que o traço comum já não
/// desenhe. É a sonda que o doc de [`crate::line_kind::RIBBON_LAG_MIN_S`] cita.
///
/// ⚠️ **A régua é o DAB, não o pixel** — o produto emite tinta a cada `spacing`, então um
/// deslocamento menor que um espaçamento **não tem onde pousar**: a fita e o neutro carimbam nos
/// mesmos lugares. Por isso a coluna que decide é `atraso / espaçamento`, e o piso é onde ela cruza
/// a unidade.
///
/// ⚠️ **E ela mede o DELTA contra o neutro, nunca o atraso absoluto** — o traço comum também termina
/// atrás do dedo (até um espaçamento, porque o último dab só sai quando o caminho o alcança), e
/// atribuir esse resto à fita infla toda linha da tabela.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_ribbon_floor_of_visibility() {
    const V: f32 = 2400.0; // px/s — um gesto rápido de verdade, onde o atraso é maior
    let spacing_px = 12.0 * 0.1 * 2.0; // radius * spacing * 2 (o passo do percurso é em DIÂMETROS)
    // O CONTROLE: o mesmo gesto sem fita. O que ele deixa para trás não é da fita.
    let mut neutral = spec(0.0, 0.30, 0.0);
    neutral.line_kind = LineKind::None;
    let (_, base_lag, _) = straight(neutral, V, 2.0);
    println!("[ribbon] gesto reto a {V:.0} px/s por 2 s; espacamento = {spacing_px:.2} px");
    println!("[ribbon] CONTROLE (sem fita): o ultimo dab fica {base_lag:.2} px atras do dedo");
    println!(
        "{:>8} {:>9} {:>11} {:>10} {:>11} {:>9} {:>9}",
        "weight", "tau s", "atraso 60Hz", "em dabs", "atraso 960Hz", "2*z*v*tau", "n@dt cap"
    );
    for w in [1.0f32, 0.4, 0.16, 0.08, 0.04, 0.02, 0.008, 0.004] {
        let sp = spec(w, 0.30, 0.0);
        let tau = sp.ribbon_lag_s();
        let (_, lag, _) = straight(sp, V, 2.0);
        let (_, lag_fast, _) = straight_at_rate(sp, V, 2.0, 16);
        let delta = (lag - base_lag).max(0.0);
        let delta_fast = (lag_fast - base_lag).max(0.0);
        // A lei do regime permanente de uma mola de 2ª ordem dirigida a velocidade constante:
        // `x = v*t - L` com aceleracao nula da `L = 2*zeta*v*tau`. Ela e o oraculo da tabela.
        let law = 2.0 * sp.ribbon_damping() * V * tau;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((crate::line_kind::RIBBON_MAX_STEP_S
            / (crate::line_kind::RIBBON_SUBSTEP_FRACTION * tau))
            .ceil() as usize)
            .max(1);
        println!(
            "{w:>8.3} {tau:>9.4} {delta:>11.2} {:>10.2} {delta_fast:>11.2} {law:>9.2} {n:>9}",
            delta / spacing_px
        );
    }
    println!("[ribbon] leitura: o piso e a linha onde `em dabs` cruza 1,0 -- abaixo dela a fita");
    println!("[ribbon] carimba onde o neutro carimba, e o slider inteiro nao desenha nada.");
}

/// **QUEM EMITE NUM TRAÇO DE FITA** — o `extend` ou o tique?
///
/// ⚠️ **A sonda de atraso NÃO consegue ver isto**: ela lê o último dab depois das DUAS chamadas,
/// então se o `extend` carimba no dedo e o tique carimba na fita, o segundo sobrescreve a leitura e
/// a tabela sai correta sobre um traço que alterna entre dois lugares.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_who_emits_on_a_ribbon_stroke() {
    let dt = 1.0 / 60.0;
    for (name, mut sp) in [
        ("None (controle)", spec(0.0, 0.30, 0.0)),
        ("Ribbon 0,45", spec(0.45, 0.30, 0.0)),
    ] {
        if name.starts_with("None") {
            sp.line_kind = LineKind::None;
        }
        let mut s = Stroke::new(sp, plain(), 7);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [100.0, 300.0],
                pressure: 1.0,
            },
            &mut out,
        );
        let (mut ext, mut tik) = (0usize, 0usize);
        let (mut ext_last, mut tik_last) = ([0.0f32, 0.0], [0.0f32, 0.0]);
        for i in 1..=30 {
            out.clear();
            #[allow(clippy::cast_precision_loss)]
            let x = 100.0 + (i as f32) * 40.0;
            s.extend(
                StrokePoint {
                    pos: [x, 300.0],
                    pressure: 1.0,
                },
                &mut out,
            );
            ext += out.len();
            if let Some(d) = out.last() {
                ext_last = d.center;
            }
            s.tick(dt, &mut out);
            tik += out.len();
            if let Some(d) = out.last() {
                tik_last = d.center;
            }
        }
        println!(
            "[quem-emite] {name:<16} extend={ext:>5} dabs (ultimo x={:.1}) | tique={tik:>5} dabs (ultimo x={:.1}) | dedo x=1300.0",
            ext_last[0], tik_last[0]
        );
    }
    println!("[quem-emite] leitura: numa FITA so o TIQUE pode emitir. Se as duas colunas tiverem");
    println!("[quem-emite] dabs, o caminho alterna entre o dedo e a fita -- e isso desenha ESPICULAS.");
}

/// **ONDE A FITA SALTA** — uma espícula é um segmento reto entre dois dabs distantes, então basta
/// perguntar por saltos maiores que o espaçamento e reportar em que FASE do traço eles caem.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_where_the_ribbon_jumps() {
    let dt = 1.0 / 60.0;
    let spacing_px = 12.0 * 0.1 * 2.0;
    let mut s = Stroke::new(spec(0.45, 0.30, 0.0), plain(), 7);
    let mut out = Vec::new();
    let mut prev: Option<[f32; 2]> = None;
    let mut worst = (0.0f32, String::new());
    let mut jumps = 0usize;
    let mut scan = |out: &[crate::stroke::Dab], fase: &str, prev: &mut Option<[f32; 2]>| {
        for d in out {
            if let Some(p) = *prev {
                let g = ((d.center[0] - p[0]).powi(2) + (d.center[1] - p[1]).powi(2)).sqrt();
                if g > spacing_px * 2.0 {
                    jumps += 1;
                    if g > worst.0 {
                        worst = (g, format!("{fase} de {p:?} para {:?}", d.center));
                    }
                }
            }
            *prev = Some(d.center);
        }
    };
    s.begin(
        StrokePoint {
            pos: [100.0, 300.0],
            pressure: 1.0,
        },
        &mut out,
    );
    scan(&out, "begin", &mut prev);
    // Um gesto CURVO: a espicula do report aparece nos extremos, e uma reta nao os tem.
    for i in 1..=60 {
        out.clear();
        #[allow(clippy::cast_precision_loss)]
        let t = i as f32 / 60.0;
        let x = 100.0 + t * 900.0;
        let y = 300.0 + (t * 12.0).sin() * 160.0;
        s.extend(StrokePoint { pos: [x, y], pressure: 1.0 }, &mut out);
        scan(&out, "extend", &mut prev);
        s.tick(dt, &mut out);
        scan(&out, "tique", &mut prev);
    }
    out.clear();
    s.finish(&mut out);
    println!("[salto] o pen-up emitiu {} dabs (a CAUDA)", out.len());
    scan(&out, "finish", &mut prev);
    println!("[salto] espacamento = {spacing_px:.2} px; saltos > 2x isso: {jumps}");
    println!("[salto] pior: {:.1} px -- {}", worst.0, worst.1);
}

/// **ONDE A TINTA ACABA** — a fita tem de parar onde a FITA parou, nunca no cursor.
///
/// ⚠️ **O oráculo NÃO é o salto entre dabs.** Uma espícula é uma corrida RETA de dabs, espaçados
/// normalmente, numa direção que o gesto não tem — logo o `measure_where_the_ribbon_jumps` mede
/// zero sobre ela. Quem a denuncia é a POSIÇÃO final da tinta contra as duas candidatas.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_where_the_ink_ends_after_pen_up() {
    let dt = 1.0 / 60.0;
    let mut s = Stroke::new(spec(0.45, 0.30, 0.0), plain(), 7);
    let mut out = Vec::new();
    s.begin(StrokePoint { pos: [100.0, 300.0], pressure: 1.0 }, &mut out);
    let mut antes = [100.0f32, 300.0];
    let mut x = 100.0f32;
    for _ in 0..30 {
        out.clear();
        x += 40.0;
        s.extend(StrokePoint { pos: [x, 300.0], pressure: 1.0 }, &mut out);
        s.tick(dt, &mut out);
        if let Some(d) = out.last() {
            antes = d.center;
        }
    }
    out.clear();
    s.finish(&mut out);
    let depois = out.last().map_or(antes, |d| d.center);
    println!("[fim] dedo soltou em x={x:.1}");
    println!("[fim] ultimo dab ANTES do pen-up  : x={:.1}  (a fita, atrasada)", antes[0]);
    println!("[fim] ultimo dab DEPOIS do pen-up : x={:.1}  ({} dabs de cauda)", depois[0], out.len());
    println!("[fim] leitura: se o depois pousar NO DEDO, algo arrastou a tinta ate o cursor em");
    println!("[fim] linha reta -- e essa reta E a espicula. A fita tem de parar onde ela parou.");
}
