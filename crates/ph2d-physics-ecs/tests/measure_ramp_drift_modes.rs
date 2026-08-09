//! **A DERIVA DE RAMPA NOS DOIS MODOS, VARRENDO θ** — o item 2 do §7 do plano
//! 07, e a reconferência que a §8.1 obriga.
//!
//! ⚠️ **Não é uma caixa por marcar.** A lei publicada no `ride.rs`
//! (`deriva(10 s) = 0,153 · sen θ · (1 − d)`) é um fato sobre o modo
//! **DINÂMICO** — `d` é o `spring_damping`, e o modo cinemático não tem mola
//! nenhuma para amortecer. O lado cinemático nunca foi varrido; e a cura da
//! §8.1 (2026-08-09) deu à absorção um **PISO** derivado da superfície, ou seja
//! mexeu exatamente na lei que governa um personagem cinemático numa rampa.
//! *Quem move o número que justificava uma nota tem de reconferir a nota*
//! (`CLAUDE.md` §0).
//!
//! ⚠️ **A coluna do DEFAULT não basta e a do CONTROLE não é o produto.** O
//! `spring_damping` que shipa está no TETO, onde a deriva dinâmica é `0,0000`
//! por construção — uma tabela só com ele diria *"os dois modos são iguais"*
//! sobre uma fixture que não contém o fenômeno (a lição que o
//! `player_kinematic.rs` já carrega no cabeçalho). Por isso a varredura tem as
//! DUAS: o default (o que o artista recebe) e um quarto do teto (onde o
//! dinâmico de facto sofre).
//!
//! Rodar:
//! `cargo test -p ph2d-physics-ecs --release --test measure_ramp_drift_modes -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod platform;

use ph2d_physics_ecs::{BodyKind, PlatformPlayer, PlayerMode, RigidBody};
use ph2d_platformer::RideConfig;

use platform::pose;

/// Meio segundo para a perna assentar antes de o relógio da medição começar.
const SETTLE_TICKS: u64 = 120;

fn make_kinematic(sim: &mut ph2d_ecs::SimWorld, who: ph2d_ecs::Entity) {
    let mut e = sim.world_mut().entity_mut(who);
    e.insert(PlayerMode::Kinematic);
    if let Some(mut rb) = e.get_mut::<RigidBody>() {
        rb.kind = BodyKind::Kinematic;
    }
}

/// Quanto o personagem **parado** viaja ao longo do chão em `secs` segundos.
///
/// ⚠️ Mede o **percurso HORIZONTAL** e não o do plano: os dois diferem por
/// `1/cos θ`, e a lei publicada é em `x`. Trocar de unidade a meio da varredura
/// faria a comparação com o `ride.rs` mentir 41% a 45°.
fn idle_travel(deg: f32, kinematic: bool, damping: f32, secs: u64) -> f32 {
    let (mut sim, mut bridge, who) = platform::scene(deg.to_radians(), 0.0);
    {
        let mut e = sim.world_mut().entity_mut(who);
        if let Some(mut p) = e.get_mut::<PlatformPlayer>() {
            p.spring_damping = damping;
        }
    }
    if kinematic {
        make_kinematic(&mut sim, who);
    }
    for t in 1..=SETTLE_TICKS {
        bridge.dispatch(&mut sim, true, t);
    }
    let x0 = pose(&sim).0;
    for t in SETTLE_TICKS + 1..=SETTLE_TICKS + secs * 60 {
        bridge.dispatch(&mut sim, true, t);
    }
    (pose(&sim).0 - x0).abs()
}

/// **A VARREDURA.** Três colunas porque são três perguntas diferentes.
///
/// # O que ela mediu (2026-08-09), e as duas coisas que ela derrubou
///
/// | rampa | din@default | din@¼ | CINEMÁTICO | lei(¼) |
/// |---|---|---|---|---|
/// | 10° | 0,0000 | 0,0197 | **0,0000** | 0,0199 |
/// | 20° | 0,0000 | 0,0369 | **0,0000** | 0,0392 |
/// | 30° | 0,0000 | 0,0498 | **0,0000** | 0,0574 |
/// | 40° | 0,0000 | 0,0566 | **0,0000** | 0,0738 |
/// | 45° | 0,0000 | 0,0575 | **0,0000** | 0,0811 |
/// | 50° | 24,1617 | 24,1617 | **0,0000** | — |
/// | 60° | 16,4994 | 16,4994 | **0,0000** | — |
///
/// 1. **A deriva cinemática é `0,0000` em TODA rampa**, inclusive em 60 s — a
///    lei não tem termo em θ, ao contrário da dinâmica. É estrutural: sob Snap
///    não há mola a integrar.
/// 2. ⚠️ **A lei publicada do `ride.rs` SATURA acima de ~20°** e não é o
///    oráculo desta fixture: ela acerta a 10° e 20° (0,0197 vs 0,0199 · 0,0369
///    vs 0,0392) e erra **13% a 30°, 23% a 40° e 29% a 45°**. Ela foi ajustada
///    noutro rig (`measure_substep::rig`, 4 sub-passos) — a lição da §5.40 do
///    doc 28 do Painter, noutro módulo: *duas fixtures "grandes" diferentes dão
///    dois números incomparáveis*.
/// 3. ⚠️ **Acima do limite o cinemático NÃO escorrega** e o dinâmico sai de
///    quadro — o defeito da §8.3, atribuído em quatro passos pela sonda irmã.
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_ramp_drift_in_both_modes() {
    let ship = RideConfig::STARTING_POINT.spring_damping;
    let quarter = 0.25 * RideConfig::MAX_DAMPING;
    println!("\n=== DERIVA DE RAMPA, PARADO 10 s (percurso horizontal, m) ===");
    println!("  o limite de rampa autorado e' 45 graus; as duas ultimas linhas estao ACIMA dele");
    println!(
        "{:>7}  {:>12}  {:>12}  {:>12}  {:>10}",
        "rampa", "din@default", "din@1/4", "CINEMATICO", "lei(1/4)"
    );
    for &deg in &[10.0_f32, 20.0, 30.0, 40.0, 45.0, 50.0, 60.0] {
        let d_ship = idle_travel(deg, false, ship, 10);
        let d_quarter = idle_travel(deg, false, quarter, 10);
        let kin = idle_travel(deg, true, quarter, 10);
        // A lei do `ride.rs`, avaliada no mesmo ponto — o oráculo externo.
        let law = 0.153 * deg.to_radians().sin() * (1.0 - quarter / RideConfig::MAX_DAMPING);
        println!("{deg:>6.0}°  {d_ship:>12.4}  {d_quarter:>12.4}  {kin:>12.4}  {law:>10.4}");
    }

    // ⚠️ Dez segundos podem ESCONDER uma deriva lenta. A janela longa é o
    // controle que separa *"não deriva"* de *"deriva devagar"*.
    println!("\n=== A MESMA COISA EM 60 s (o controle da janela) ===");
    println!("{:>7}  {:>12}  {:>12}", "rampa", "din@1/4", "CINEMATICO");
    for &deg in &[30.0_f32, 45.0] {
        println!(
            "{deg:>6.0}°  {:>12.4}  {:>12.4}",
            idle_travel(deg, false, quarter, 60),
            idle_travel(deg, true, quarter, 60)
        );
    }
}

/// **O QUE ACONTECE ACIMA DO LIMITE** — o diagnóstico que a varredura obrigou,
/// e o defeito que ele nomeou (§8.3).
///
/// ⚠️ A varredura devolveu `0,0000` para o cinemático a **60°**, e o limite
/// autorado é 45: acima dele a `footing_verdict` responde `Steep`, o
/// `Footing::ground()` dá `None`, e um personagem sem chão devia **escorregar**
/// (é o que o dinâmico faz, em 16-24 m). Um zero exacto onde se espera
/// movimento é *"a fixture não contém o fenômeno"* ou um defeito — e a
/// diferença só sai da TRAJETÓRIA.
///
/// # O que estas duas sondas mediram, em ordem
///
/// 1. **Ele repousa na rampa, não paira** — a distância perpendicular do centro
///    ao plano menos o suporte da cápsula dá **5,5 cm** a 60° e 5,2 cm a 50°, a
///    folga do controlador. A mola não o segura.
/// 2. **O limite é INERTE ali** ([`measure_whether_the_slope_limit_is_honoured_on_a_steep_ramp`]):
///    trocar `max_slope_deg` de 45 para 90 para 5 dá a MESMA pose a 60° e a 80°,
///    enquanto a 45° ela muda (1,3175 contra 1,3741). *O veredito não é quem
///    congela.*
/// 3. **O rapier manda DESLIZAR** — `is_nonslip_slope = ângulo <= min_slope_slide_angle`
///    é `60 <= 45`, falso, então o `handle_slide` cai no ramo *"let it slide"*.
///    Logo o deslocamento PEDIDO é que era zero.
/// 4. **E era** (instrumentado): `stand=None vel=[0,0] wanted=[0,0]
///    grounded_was=true`. A lei RECUSA a rampa e absorve a gravidade na mesma —
///    porque a absorção dentro do [`ph2d_platformer::kinematic_advance`]
///    pergunta ao `state.grounded` (*"eu TOQUEI?"*, do controlador) enquanto a
///    da ponte pergunta ao `footing` (*"isto é CHÃO?"*, a K4).
///
/// ⇒ **duas respostas para a mesma pergunta no mesmo tique**, e a que vence é a
/// que não conhece o limite de rampa.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_what_happens_above_the_slope_limit() {
    let quarter = 0.25 * RideConfig::MAX_DAMPING;
    for &deg in &[45.0_f32, 50.0, 60.0] {
        for kinematic in [false, true] {
            let (mut sim, mut bridge, who) = platform::scene(deg.to_radians(), 0.0);
            {
                let mut e = sim.world_mut().entity_mut(who);
                if let Some(mut p) = e.get_mut::<PlatformPlayer>() {
                    p.spring_damping = quarter;
                }
            }
            if kinematic {
                make_kinematic(&mut sim, who);
            }
            let mode = if kinematic { "CINEMATICO" } else { "dinamico " };
            println!("\n=== rampa {deg:.0} deg, {mode} ===");
            println!("{:>7}  {:>10}  {:>10}", "tique", "x", "y");
            for t in 1..=300u64 {
                bridge.dispatch(&mut sim, true, t);
                if t % 60 == 0 || t <= 3 {
                    let (x, y) = pose(&sim);
                    println!("{t:>7}  {x:>10.4}  {y:>10.4}");
                }
            }
        }
    }
}

/// **A DERIVA CINEMÁTICA NÃO TEM TERMO EM θ** — e é isso que a separa da
/// dinâmica, cuja lei publicada é `∝ sen θ`.
///
/// ⚠️ **A barra é uma RAZÃO contra o dinâmico no mesmo ponto**, e não um
/// literal: o resíduo cinemático é *onde o último sweep o deixou* (a §7.5 mediu
/// a folga do pé a variar de 1,0 a 4,5 cm com a aproximação), então um valor
/// absoluto seria derrubado por qualquer mudança de cápsula. O que é estrutural
/// — e o que este gate afirma — é que o dinâmico **cresce com a rampa** e o
/// cinemático **não**.
#[test]
fn the_kinematic_drift_does_not_grow_with_the_ramp() {
    let quarter = 0.25 * RideConfig::MAX_DAMPING;
    let (shallow_d, steep_d) = (
        idle_travel(15.0, false, quarter, 10),
        idle_travel(45.0, false, quarter, 10),
    );
    let (shallow_k, steep_k) = (
        idle_travel(15.0, true, quarter, 10),
        idle_travel(45.0, true, quarter, 10),
    );

    // O CONTROLE: a fixture tem de conter o fenômeno nos dois extremos, e a
    // razão dinâmica tem de mostrar o crescimento que a lei prevê (sen 45 /
    // sen 15 = 2,73).
    assert!(
        shallow_d > 0.005 && steep_d > 0.02,
        "a fixture tem de CONTER o fenomeno: o dinamico derivou {shallow_d:.4} a 15 deg \
         e {steep_d:.4} a 45 deg"
    );
    // ⚠️ **A barra é MEDIDA (2,00x), não derivada da lei publicada (2,73x).**
    // A minha primeira versão pediu `> 2,0 x` porque `sen45/sen15 = 2,73`, e
    // ela falhou por 0,0001 sobre um produto correto: a lei do `ride.rs` foi
    // ajustada noutra fixture e SATURA acima de ~20 graus (ver o cabecalho da
    // varredura). Uma barra tomada de um numero medido noutra fixture nao e uma
    // barra medida.
    assert!(
        steep_d > 1.6 * shallow_d,
        "o dinamico tem de CRESCER com a rampa (medido 2,00x): {shallow_d:.4} -> {steep_d:.4}"
    );

    // E o cinemático, no mesmo par de rampas, não cresce.
    assert!(
        steep_k < 0.25 * steep_d,
        "sob Snap a rampa ingreme nao pode derivar como o dinamico: \
         {steep_k:.4} contra {steep_d:.4}"
    );
    assert!(
        steep_k < 3.0 * shallow_k.max(1.0e-4),
        "e a deriva cinematica nao pode crescer com theta: {shallow_k:.4} -> {steep_k:.4}"
    );
}

/// **A ABLAÇÃO QUE ATRIBUI** — o limite de rampa está a ser HONRADO a 60°?
///
/// ⚠️ Teoria não decide isto. Se variar `max_slope_deg` de 45 para 90 (tudo
/// caminhável) deixar a trajetória IDÊNTICA, então o veredito já era `Ground` e
/// o limite é inerte na descida; se ela mudar, o veredito funciona e quem
/// congela o personagem é outra coisa.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_whether_the_slope_limit_is_honoured_on_a_steep_ramp() {
    println!("\n=== O LIMITE DE RAMPA E' HONRADO? (cinematico, 5 s parado) ===");
    println!("{:>7}  {:>10}  {:>12}  {:>12}", "rampa", "limite", "x", "y");
    for &deg in &[45.0_f32, 60.0, 80.0] {
        for &limit in &[45.0_f32, 90.0, 5.0] {
            let (mut sim, mut bridge, who) = platform::scene(deg.to_radians(), 0.0);
            {
                let mut e = sim.world_mut().entity_mut(who);
                if let Some(mut p) = e.get_mut::<PlatformPlayer>() {
                    p.max_slope_deg = limit;
                }
            }
            make_kinematic(&mut sim, who);
            for t in 1..=300u64 {
                bridge.dispatch(&mut sim, true, t);
            }
            let (x, y) = pose(&sim);
            println!("{deg:>6.0}°  {limit:>9.0}°  {x:>12.4}  {y:>12.4}");
        }
    }
}
