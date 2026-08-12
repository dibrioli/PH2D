//! **PLANAR, medido antes de decidir a forma** (`W-Glide`, plano 08 §4.6).
//!
//! ⚠️ **O plano escreveu a suspeita e mandou conferi-la:** *"planar é um
//! multiplicador de gravidade sob botão, na queda — se for isso, ele não é uma
//! wave: é um campo"*. Esta sonda é a conferência, e ela tem de responder a duas
//! perguntas, não uma:
//!
//! 1. **Uma ESCALA de gravidade produz um planeio?** O módulo já tem essa forma
//!    (as fases do pulo: `extra = escala − 1`), e ela é a mais barata que existe
//!    — um campo, um braço no `match`, zero superfície nova.
//! 2. **Ou é preciso uma VELOCIDADE ALVO?** O módulo também já tem essa forma (o
//!    `wall_slide`, um `boost` que põe `rel_up` em `−slide_speed`), e o
//!    doc-comment dela guarda o precedente que interessa: **a versão-TETO foi
//!    morta por medição**.
//!
//! ⚠️ **A diferença entre as duas não é de gosto, é OBSERVÁVEL:** sob uma escala
//! a velocidade continua a CRESCER — só mais devagar —, então *quão rápido se
//! chega ao chão depende de QUANTO se caiu*; sob um alvo ela ASSENTA, e a
//! descida passa a ser um número que o jogador conhece. Um planeio existe para
//! **atravessar um vão**, e atravessar um vão exige saber onde se vai aterrar.
//!
//! ⚠️ **E há uma terceira coluna que decide sozinha:** o ALCANCE horizontal. É
//! para isso que se plana, e é a única coluna que fala a língua do artista.
//!
//! Rode: `cargo test -p ph2d-physics-ecs --test measure_glide --release
//! -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use scene_fixture::{pose, scene};

/// A altura de onde se larga, em metros acima do repouso.
const DROP: f32 = 8.0;

/// Uma cena plana com o personagem NO AR, a `DROP` metros acima do repouso.
///
/// ⚠️ **Solto do alto e não a pular**, e é o regime que a pergunta pede: um
/// planeio é o que se faz **na queda**, e um arco de pulo traria a fase de
/// subida junto — que tem gravidade própria (`takeoff_gravity` / `cut_gravity`)
/// e envenenaria a coluna.
fn falling(fall_gravity: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
        p.fall_gravity = fall_gravity;
    }
    let y = pose(&sim).1;
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(player) {
        t.translation.y = y + DROP;
    }
    (sim, bridge, player)
}

/// Deixa cair, com `drive` fixo, e devolve `(PICO da descida, segundos no ar,
/// alcance horizontal)`.
///
/// ⚠️ **A coluna do pico MENTIU DUAS VEZES, e a segunda é mais interessante que
/// a primeira.** Ela nasceu como *"a velocidade ao tocar"* e reportava
/// `−0,10 m/s` para uma queda de oito metros **em todas as escalas** — a perna é
/// uma MOLA e apanha o corpo antes de o pé encostar, então o tique em que a
/// queda acaba já traz a velocidade **assentada**.
///
/// ⚠️ **Trocada pelo PICO, ela saiu NÃO-MONOTÓNICA** (`0,25 → −6,26` mas
/// `0,10 → −8,29`), e uma gravidade menor não pode dar uma queda mais rápida. A
/// coluna de PROFUNDIDADE é quem responde: o pico acontece sempre a **~7,85 m**
/// de uma queda de 8 — isto é, **na aterragem**. Sob gravidade baixa o corpo
/// chega devagar e o que domina o pico é a **perna a puxá-lo para a altura de
/// repouso**, não a queda.
///
/// ⚠️ **Isso é um fato do módulo, não desta wave** (as seis linhas o mostram,
/// incluindo o default de hoje), e é por isso que a coluna se chama
/// `pico NA ATERRAGEM`: *uma queda mais lenta não aterra mais macio*. Quem
/// responde *"quão depressa se estava a cair"* é a sonda irmã, que mede por
/// PROFUNDIDADE e para antes de a perna entrar.
fn drop_it(fall_gravity: f32, drive: f32) -> (f32, f32, f32, f32) {
    let (mut sim, mut bridge, player) = falling(fall_gravity);
    let (x0, y0) = pose(&sim);
    let mut tick = 0_u64;
    let mut prev_y = y0;
    let mut peak = 0.0_f32;
    let mut peak_at = 0.0_f32;
    for i in 1..=600_u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        let (x, y) = pose(&sim);
        let v = (y - prev_y) * 60.0;
        prev_y = y;
        if v < peak {
            peak = v;
            peak_at = y0 - y;
        }
        // Tocou o chão quando parou de descer depois de ter descido.
        if v > -0.05 && y < y0 - 1.0 {
            return (peak, i as f32 / 60.0, (x - x0).abs(), peak_at);
        }
    }
    (peak, 10.0, (pose(&sim).0 - x0).abs(), peak_at)
}

/// **A tabela que decide a forma da wave.**
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_a_gravity_scale_does_to_a_fall() {
    println!("\n== uma QUEDA de {DROP:.1} m, por escala de gravidade ==");
    println!(
        "  fall_gravity   pico NA ATERRAGEM   a que profundidade   segundos no ar   alcance lateral"
    );
    for g in [2.0_f32, 1.0, 0.5, 0.25, 0.1, 0.05] {
        let (v, t, dx, at) = drop_it(g, 1.0);
        println!("  {g:>12.2}   {v:>17.2}   {at:>18.2}   {t:>14.2}   {dx:>15.2}");
    }
    println!("\n(o default do modulo e' 2.00; `1.00` e' a gravidade do mundo, byte a byte)");
    println!(
        "(o pico acontece sempre a ~7.85 m de 8 -- e' a PERNA a apanhar o corpo, nao a queda)"
    );
}

/// **A velocidade ao longo da queda** — a coluna que separa *escala* de *alvo*.
///
/// ⚠️ **É esta tabela que responde à pergunta, e não a de cima:** uma escala
/// pequena parece um planeio numa queda curta e deixa de parecer numa longa,
/// porque a velocidade nunca assenta. Se as três colunas de profundidade
/// discordarem, a escala **não** é um planeio.
#[test]
#[ignore = "sonda de medicao"]
fn measure_whether_the_descent_ever_settles() {
    println!("\n== a velocidade de descida, por profundidade ==");
    println!("  fall_gravity   apos 1 m   apos 3 m   apos 6 m   assentou?");
    for g in [2.0_f32, 1.0, 0.5, 0.25, 0.1, 0.05] {
        let (mut sim, mut bridge, player) = falling(g);
        let y0 = pose(&sim).1;
        let mut tick = 0_u64;
        let mut prev_y = y0;
        let mut marks = [0.0_f32; 3];
        let mut next = 0_usize;
        for _ in 0..600 {
            bridge.set_player_input(player, PlayerInput::default());
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
            let y = pose(&sim).1;
            let v = (y - prev_y) * 60.0;
            prev_y = y;
            let fallen = y0 - y;
            while next < 3 && fallen >= [1.0, 3.0, 6.0][next] {
                marks[next] = v;
                next += 1;
            }
            if next == 3 {
                break;
            }
        }
        // "Assentou" = a velocidade parou de crescer entre 3 m e 6 m.
        let settled = (marks[2] - marks[1]).abs() < 0.10;
        println!(
            "  {g:>12.2}   {:>8.2}   {:>8.2}   {:>8.2}   {}",
            marks[0],
            marks[1],
            marks[2],
            if settled { "SIM" } else { "nao" }
        );
    }
}

/// **O que o ALVO daria**, pela lei que o `wall_slide` já usa.
///
/// ⚠️ **Não há código novo aqui, e é o ponto:** um alvo é uma constante por
/// construção, então esta tabela é aritmética — ela existe para ficar **ao lado**
/// da de cima, na mesma unidade, e tornar a comparação uma leitura em vez de um
/// argumento.
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_a_speed_target_would_give() {
    println!("\n== uma DESCIDA a velocidade alvo (a lei do wall_slide) ==");
    println!("  alvo m/s   v apos 1 m   v apos 6 m   segundos p/ {DROP:.0} m   alcance a 6 m/s");
    for target in [1.0_f32, 2.0, 3.0, 4.0] {
        let t = DROP / target;
        println!(
            "  {target:>8.2}   {:>10.2}   {:>10.2}   {t:>16.2}   {:>15.2}",
            -target,
            -target,
            6.0 * t
        );
    }
    println!("\n(a velocidade e' a MESMA em toda profundidade -- e' isso que um alvo E')");
}

/// **AS TRÊS CANDIDATAS, no instante em que o dedo aperta.**
///
/// ⚠️ **As leis abaixo NÃO são produto** — não existe planeio ainda. São as três
/// formas candidatas escritas como one-liners, lado a lado, para que a escolha
/// seja uma LEITURA e não um argumento. Cada uma é a forma que este módulo já
/// tem em algum lugar:
///
/// | forma | onde já vive | o que faz |
/// |---|---|---|
/// | **escala** | as fases do pulo (`extra = escala − 1`) | acelera menos |
/// | **alvo** | o `wall_slide` (`boost` até `−slide_speed`) | **põe** a velocidade |
/// | **teto** | ⚠️ **em lugar nenhum** | põe a velocidade **só se for mais rápida** |
///
/// ⚠️ **A terceira não estava escrita em parte nenhuma, e o doc do `wall_slide`
/// explica por quê:** a versão-teto dele foi **morta por medição** — *"com o
/// atrito default o personagem não cai, e um teto nunca dispararia"*. Isso é
/// verdade **da PAREDE**, onde há atrito. **No ar livre não há**, então a
/// objeção não viaja — e a forma que ela matou lá é candidata legítima aqui.
///
/// ⚠️ **A coluna que decide é `Δv`, o salto de velocidade no tique do aperto:**
/// um planeio é uma assistência à QUEDA, e uma assistência que **inverte** um
/// corpo que sobe não é um planeio, é um botão de *descer agora*.
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_each_candidate_does_at_the_moment_the_finger_presses() {
    /// A velocidade alvo das duas candidatas que a usam, m/s.
    const TARGET: f32 = 2.0;

    println!("\n== as tres candidatas, no tique do aperto (alvo/teto = {TARGET:.1} m/s) ==");
    println!("  momento          rel_up   escala 0.1   ALVO      TETO");
    for (nome, v) in [
        ("subindo forte", 8.0_f32),
        ("subindo devagar", 2.0),
        ("no apice", 0.0),
        ("caindo devagar", -1.0),
        ("caindo rapido", -12.0),
    ] {
        // ESCALA: nao mexe na velocidade, so' na aceleracao ⇒ Δv do tique = 0.
        let escala = 0.0_f32;
        // ALVO: poe a velocidade, sempre (a lei do `wall_slide`).
        let alvo = -TARGET - v;
        // TETO: poe a velocidade SO' se ela for mais rapida que o teto.
        let teto = if v < -TARGET { -TARGET - v } else { 0.0 };
        println!("  {nome:<15}  {v:>6.1}   {escala:>10.1}   {alvo:>+6.1}   {teto:>+6.1}");
    }
    println!("\n(Δv = o salto de velocidade que a lei impoe NO TIQUE do aperto)");
    println!("(um planeio assiste a QUEDA -- inverter quem sobe seria outro botao)");
}

/// **A CENA: quanto se atravessa por queda, com e sem o planeio.**
///
/// ⚠️ **Esta sonda existe para DIMENSIONAR a cena de smoke**, e o precedente é a
/// da beirada: a primeira versão daquela cena pôs o patamar numa altura que o
/// corpo nunca alcançava, porque foi calculada com o número do ar livre em vez
/// do medido. Um vão escolhido a olho é um vão que ou os dois atravessam ou
/// nenhum atravessa — e nos dois casos a cena não mostra a feature.
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_gap_a_glide_crosses() {
    println!("\n== o VAO que se atravessa, por queda (correndo, dedo no pulo) ==");
    println!("  queda   sem planeio   com planeio 2 m/s   com 1 m/s");
    for queda in [1.0_f32, 1.5, 2.0, 3.0] {
        let mut col = [0.0_f32; 3];
        for (i, teto) in [0.0_f32, 2.0, 1.0].into_iter().enumerate() {
            let (mut sim, mut bridge, player) = scene(0.0, 0.0);
            if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
                p.glide_fall_speed = teto;
            }
            // ⚠️ **A altura é escrita ANTES do primeiro dispatch, e é a regra do
            // W2a:** *a pose de repouso é a pose AUTORADA no tique 0*, então um
            // `Transform` escrito com o relógio já a andar **não chega ao
            // solver**. A primeira versão desta sonda assentava primeiro e
            // levantava depois — e reportou o MESMO número para quedas de 1 e de
            // 3 metros, que é a assinatura de uma queda que nunca aconteceu.
            let (x0, y_rest) = pose(&sim);
            if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(player) {
                t.translation.y = y_rest + queda;
            }
            let mut tick = 0_u64;
            for _ in 0..900 {
                bridge.set_player_input(
                    player,
                    PlayerInput {
                        drive: 1.0,
                        jump: true,
                        ..PlayerInput::default()
                    },
                );
                tick += 1;
                bridge.dispatch(&mut sim, true, tick);
                if pose(&sim).1 <= y_rest + 0.02 {
                    break;
                }
            }
            col[i] = pose(&sim).0 - x0;
        }
        println!(
            "  {queda:>5.1}   {:>10.2}   {:>17.2}   {:>9.2}",
            col[0], col[1], col[2]
        );
    }
    println!("\n(o vao da cena tem de cair ENTRE a 1a coluna e a 2a -- so' assim");
    println!(" um atravessa e o outro nao, com o mesmo gesto)");
}
