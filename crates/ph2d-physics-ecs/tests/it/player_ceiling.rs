//! **O TETO É UM FATO** (`W-Ceiling`) — o `is_on_ceiling` do Godot, a última das
//! três consultas que a auditoria 09 §3.A deixou como cauda.
//!
//! ⚠️ **Estes gates moram na PONTE e não na lei, e não é arbitrário:** o
//! `head_blocked` é medido contra o mundo (uma varredura da forma), então um
//! teste que o soletrasse à mão numa fixture estaria a afirmar o que ele próprio
//! escreveu. O oráculo é uma cena com um teto de verdade e um personagem a subir
//! contra ele.
//!
//! ⚠️ **E o gate que carrega a wave é o do ASSIST DESLIGADO.** O fato nasceu
//! porque os dois sensores de cima que já existiam são assistências
//! CONDICIONAIS — o `Headroom` só é sondado agachado, o `CeilingProbe` só com a
//! quina armada —, e derivar o teto de qualquer um deles daria um readout falso
//! na maioria dos tiques, em silêncio. Um gate que só medisse a configuração
//! padrão (onde `corner_reach = 0,12` e as duas condições coincidem) seria
//! **verde exactamente sobre o defeito que esta wave existe para não ter**.

use crate::platform_water_scene as scene;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::PlayerInput;
use scene::{FLOAT, floor, subject_tuned};

/// Um teto — caixa larga e estática cuja face de BAIXO fica em `bottom`.
fn ceiling(sim: &mut SimWorld, bottom: f32) {
    sim.world_mut().spawn((
        Name::new("Ceiling"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, bottom + 0.5)),
    ));
}

/// A altura natural do pulo destes gates — a do [`PlatformPlayer::default`] não
/// alcança teto nenhum desta cena, e esta alcança com folga.
///
/// ⚠️ **As alturas de teto dos gates são RELATIVAS ao topo livre** ([`free_top`]):
/// a lei do ápice real (GEF 04) moveu o `v0` de cada altura, e uma tabela de
/// tetos fixos ficou verde ou vermelha por acaso cada vez que a lei mudou.
const H: f32 = 1.2;

fn jumper(corner_reach: f32) -> PlatformPlayer {
    jumper_h(corner_reach, H)
}

fn jumper_h(corner_reach: f32, jump_height: f32) -> PlatformPlayer {
    PlatformPlayer {
        float_height: FLOAT,
        jump_height,
        corner_reach,
        ..PlatformPlayer::default()
    }
}

/// O que uma corrida mede.
struct Run {
    /// A velocidade de cada `Bonked` publicado.
    bonks: Vec<f32>,
    /// Quantas vezes o bit `ceiling` ACENDEU (bordas de subida).
    lit: usize,
    /// O CONTROLE: sem ele um gate que mede *"bateu"* fica verde num personagem
    /// que nunca saiu do chão.
    rose: bool,
    /// O topo do collider parado no chão (a cabeça) e o mais alto que chegou.
    head: f32,
    top: f32,
    /// A maior subida por tique (m/s, diferença de posição).
    vmax: f32,
}

fn go(jump_height: f32, corner_reach: f32, ceiling_bottom: Option<f32>) -> Run {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    if let Some(b) = ceiling_bottom {
        ceiling(&mut sim, b);
    }
    let who = subject_tuned(
        &mut sim,
        true,
        FLOAT,
        Some(jumper_h(corner_reach, jump_height)),
    );
    let mut bridge = PhysicsBridge::new();
    let start = scene::y_of(&sim, "Subject");
    let (mut prev, mut top, mut vmax) = (start, start, 0.0_f32);
    let (mut bonks, mut lit, mut was_lit) = (Vec::new(), 0, false);
    for t in 1..=150u64 {
        // Segura o pulo até depois do ápice do mais alto (2,6 m ≈ 44 tiques):
        // soltar antes CORTA a subida, e a altura medida deixaria de ser a lei.
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: t <= 60,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        let now_lit = bridge.player_view(who).is_some_and(|v| v.ceiling);
        lit += usize::from(now_lit && !was_lit);
        was_lit = now_lit;
        for (_, ev) in bridge.player_events() {
            if let ph2d_platformer::PlayerEvent::Bonked { speed } = ev {
                bonks.push(*speed);
            }
        }
        let y = scene::y_of(&sim, "Subject");
        vmax = vmax.max((y - prev) * 60.0);
        top = top.max(y);
        prev = y;
    }
    let lid = scene::HALF_H + scene::RADIUS;
    Run {
        bonks,
        lit,
        rose: top > start + 0.2,
        head: start + lid,
        top: top + lid,
        vmax,
    }
}

/// O topo do collider no ápice de um pulo sem teto — a régua dos tetos.
fn free_top(jump_height: f32) -> f32 {
    go(jump_height, 0.12, None).top
}

/// `(bateu alguma vez, subiu alguma vez)`.
fn run(corner_reach: f32, ceiling_bottom: Option<f32>) -> (bool, bool) {
    let r = go(H, corner_reach, ceiling_bottom);
    (r.lit > 0, r.rose)
}

/// `(velocidades das batidas, subiu alguma vez)`.
fn bonks(corner_reach: f32, ceiling_bottom: Option<f32>) -> (Vec<f32>, bool) {
    let r = go(H, corner_reach, ceiling_bottom);
    (r.bonks, r.rose)
}

/// **O FATO SOBREVIVE À ASSISTÊNCIA DE QUINA ESTAR DESLIGADA** — o gate da wave.
///
/// ⚠️ **Mutação que deve sangrar:** dar um knob ao [`ceiling_fact_wanted`] (por
/// exemplo, fazê-lo delegar ao `corner_probe_wanted`). O braço de cima continua
/// verde — é a configuração que shipa, onde as duas condições coincidem — e o de
/// baixo fica vermelho, que é exactamente o *"funciona às vezes"* que o campo
/// existe para não ser.
#[test]
fn the_ceiling_fact_survives_the_corner_assist_being_off() {
    // Com a quina ARMADA (o ponto de partida do produto).
    let (bonked_on, rose_on) = run(0.12, Some(1.9));
    assert!(rose_on, "o controle: ele tem de sair do chao");
    assert!(bonked_on, "com a quina armada o teto tem de ser relatado");

    // Com a quina DESARMADA — o mesmo mundo, a mesma subida.
    let (bonked_off, rose_off) = run(0.0, Some(1.9));
    assert!(rose_off, "o controle: ele tem de sair do chao");
    assert!(
        bonked_off,
        "um FATO nao e' opt-in: desligar a assistencia de quina nao pode apagar \
         o `is_on_ceiling`"
    );
}

/// **Sem teto não há batida** — o controle que impede o bit de nascer `true`.
#[test]
fn a_clear_sky_is_never_a_ceiling() {
    let (bonked, rose) = run(0.12, None);
    assert!(rose, "o controle: ele tem de sair do chao");
    assert!(!bonked, "sem teto nenhum o bit nao pode acender");
}

/// **De pé sob uma marquise NÃO é teto** — a fronteira contra o [`Headroom`].
///
/// ⚠️ Esta é a metade que separa as duas perguntas. *Falta de espaço para
/// levantar* é do agachar e é verdadeira o tempo todo num corredor baixo;
/// *bateu com a cabeça* é um evento da SUBIDA. Colapsá-las daria um
/// `is_on_ceiling` aceso permanentemente, que não é um evento de batida.
#[test]
fn standing_under_an_overhang_is_not_a_ceiling() {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    // Baixo o bastante para o `Headroom` reclamar, e o personagem NUNCA pula.
    ceiling(&mut sim, FLOAT + 0.6);
    let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(0.12)));
    let mut bridge = PhysicsBridge::new();

    for t in 1..=90u64 {
        bridge.set_player_input(who, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
        if let Some(v) = bridge.player_view(who) {
            assert!(
                !v.ceiling,
                "parado no chao sob uma marquise ele nao esta' a bater em nada \
                 (tique {t})"
            );
        }
    }
}

// ── E O QUE O TETO PUBLICA: `PlayerEvent::Bonked` ────────────────────────────
//
// ⚠️ **Estes gates moram AQUI, na ponte, e não ao lado do `events_between`:** o
// `speed` que o evento carrega é a velocidade de subida no tique em que o
// SENSOR relatou o bloqueio, e um teste que soletrasse as duas vistas à mão
// estaria a afirmar o número que ele próprio escreveu. O oráculo é um teto de
// verdade e uma cabeça a subir contra ele.

/// **Uma cabeça que bate publica UM evento, com a velocidade que ela trazia.**
///
/// O teto 1,9 fica 0,7 m abaixo do topo livre: uma batida cheia (a tabela do
/// `the_bonk_fades_…` põe-na entre 3,12 e 3,78 m/s), não um quase.
///
/// ⚠️ **Mutação que deve sangrar:** derivar o evento do NÍVEL do bit em vez da
/// BORDA (`after.ceiling` sem o `&& !before.ceiling`). O bit fica de pé nos
/// tiques 8 **e** 9 — ele só se apaga quando a subida morre —, então o nível
/// publica DOIS eventos para uma batida, e um consumidor de som toca duas vezes.
#[test]
fn the_head_that_hits_publishes_one_bonk_carrying_the_speed_it_climbed_with() {
    let (bonks, rose) = bonks(0.12, Some(1.9));
    assert!(rose, "o controle: ele tem de sair do chao");
    assert_eq!(bonks.len(), 1, "uma batida, um evento: {bonks:?}");
    assert!(
        (3.0..5.0).contains(&bonks[0]),
        "a velocidade publicada nao e' a subida que ele trazia: {:?}",
        bonks[0]
    );
}

/// **Sem teto não há batida** — o controle que impede o evento de nascer de um
/// bit que acende sozinho.
#[test]
fn a_clear_sky_never_publishes_a_bonk() {
    let (bonks, rose) = bonks(0.12, None);
    assert!(rose, "o controle: ele tem de sair do chao");
    assert!(bonks.is_empty(), "sem teto nenhum: {bonks:?}");
}

/// **A batida é SILENCIOSA na margem, e isso é medição, não desenho.**
///
/// O sensor olha um tique à frente (`rel_up * dt`), então há uma janela em que
/// ele diz *bloqueado* e o contato é marginal — a pergunta honesta é **quão
/// alto** o evento grita ali. Com `H` = 1,2 a cabeça parte de 1,40 e o topo
/// livre é 2,60 (= cabeça + H, a lei do ápice real); a velocidade publicada desce
/// continuamente até o teto sair do alcance:
///
/// | teto − livre | m/s   |   | teto − livre | m/s   |
/// |--------------|-------|---|--------------|-------|
/// | −0,80        | 3,777 |   | −0,10        | 1,003 |
/// | −0,60        | 3,123 |   | −0,05        | 0,676 |
/// | −0,40        | 2,469 |   | −0,01        | 0,349 |
/// | −0,20        | 1,412 |   | +0,01        | —     |
///
/// ⚠️ **E o corte cai onde a GEOMETRIA manda:** 1 cm abaixo do topo livre ainda
/// bate, 1 cm acima já não. Reproduzir: `measure_the_bonk_across_ceiling_heights`.
#[test]
fn the_bonk_fades_to_nothing_as_the_ceiling_leaves_reach() {
    let free = free_top(H);
    let hard = bonks(0.12, Some(free - 0.6)).0;
    let graze = bonks(0.12, Some(free - 0.01)).0;
    let clear = bonks(0.12, Some(free + 0.01)).0;
    assert_eq!(hard.len(), 1, "{hard:?}");
    assert_eq!(graze.len(), 1, "{graze:?}");
    assert!(clear.is_empty(), "teto fora de alcance: {clear:?}");
    assert!(
        graze[0] < hard[0] * 0.25,
        "a margem tinha de ser silenciosa: {graze:?} contra {hard:?}"
    );
}

/// A SONDA da tabela acima — varre a altura do teto RELATIVA ao topo livre.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_bonk_across_ceiling_heights() {
    let (free, head) = (free_top(H), go(H, 0.12, None).head);
    println!("H={H} cabeca={head:.4} topo livre={free:.4}");
    println!("teto-livre | bonks m/s");
    for k in [
        -1.15_f32, -0.8, -0.6, -0.4, -0.3, -0.2, -0.1, -0.05, -0.03, -0.01, 0.01, 0.05,
    ] {
        let b = free + k;
        let r = go(H, 0.12, Some(b));
        println!("{:+.2} ({b:.3}) | {:?} lit={}", b - free, r.bonks, r.lit);
    }
}

/// **TODO teto ao alcance é batido UMA vez, em TODA altura de pulo** (Bug #12).
///
/// Jump Height 1,0–2,6 (passo 0,05) × fundo do teto da cabeça + 0,10 até o topo
/// livre (passo 0,05), com a quina armada e desarmada: exactamente UMA batida
/// (o bit acende uma vez E um `Bonked`), nenhuma com o teto fora do alcance
/// (topo livre + 0,05 e + 0,10), a subida nunca acima do `v0` da lei, e a
/// velocidade publicada nunca CRESCE quando o teto sobe.
///
/// ⚠️ **O gate que estava verde era de UM ponto** (teto 1,9 com 1,3146 m): os
/// tetos baixos — os primeiros ~0,3 m da subida, onde a perna ainda vê o chão —
/// não eram medidos, e ali a porta do teto lia o sensor cru como *no chão*.
///
/// ⚠️ **ABERTO (Bug #12b, `BUGS_physics.md`): a cabeça + 0,05 reprova nas 33
/// alturas.** É outro mecanismo: o teto está dentro do PRÓPRIO tique da
/// decolagem (o sensor corre antes da lei lançar, com `rel_up` < 0, e no tique
/// seguinte o solver já matou a subida), por isso a grelha começa em + 0,10.
#[test]
fn every_ceiling_in_reach_is_bonked_once_at_every_jump_height() {
    let g = -ph2d_physics::PhysicsWorld::DEFAULT_GRAVITY_Y;
    let mut bad = Vec::new();
    for i in 0..=32 {
        let h = 1.0 + 0.05 * i as f32;
        let law = jumper_h(0.12, h).config().jump;
        let v0 = ph2d_platformer::launch_speed(&law, g, h, false, 1.0 / 60.0);
        for reach in [0.12_f32, 0.0] {
            let free = go(h, reach, None);
            let mut last = f32::INFINITY;
            let mut b = free.head + 0.10;
            while b < free.top {
                let r = go(h, reach, Some(b));
                if r.bonks.len() != 1 || r.lit != 1 {
                    bad.push(format!(
                        "h={h:.2} q={reach} teto={b:.3} (livre {:.3}): {:?} lit={}",
                        free.top, r.bonks, r.lit
                    ));
                } else if r.bonks[0] > last + 1e-3 {
                    bad.push(format!(
                        "h={h:.2} q={reach} teto={b:.3}: {:.3} > {last:.3} (nao esvanece)",
                        r.bonks[0]
                    ));
                }
                if r.vmax > v0 + 1e-3 {
                    bad.push(format!(
                        "h={h:.2} q={reach} teto={b:.3}: vy {:.3} > v0 {v0:.3}",
                        r.vmax
                    ));
                }
                last = r.bonks.first().copied().unwrap_or(last);
                b += 0.05;
            }
            for over in [0.05_f32, 0.10] {
                let r = go(h, reach, Some(free.top + over));
                if !r.bonks.is_empty() || r.lit != 0 {
                    bad.push(format!(
                        "h={h:.2} q={reach} teto livre+{over}: {:?} lit={}",
                        r.bonks, r.lit
                    ));
                }
            }
            if !free.rose || free.vmax > v0 + 1e-3 {
                bad.push(format!(
                    "h={h:.2} q={reach} sem teto: subiu={} vy {:.3} v0 {v0:.3}",
                    free.rose, free.vmax
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{} falhas:\n{}", bad.len(), bad.join("\n"));
}

/// **E a batida chega ao BARRAMENTO com nome** — a quarta condição da política
/// de UI do plano (*a sequência leva a algum lugar*).
///
/// ⚠️ Sem este gate a wave entregaria um evento que ninguém pode ouvir: o
/// `player_signal_name` é um `match` exaustivo, então o compilador obriga a
/// nomear o variant novo — e um nome que ninguém verifica pode estar escrito e
/// não sair, porque a saída é **opt-in por-player** (`PlayerSignals`) e o opt-in
/// é a metade que um `match` não pode garantir.
#[test]
fn the_bonk_reaches_the_signal_bus_with_a_name() {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    ceiling(&mut sim, 1.9);
    let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(0.12)));
    sim.world_mut()
        .entity_mut(who)
        .insert(ph2d_physics_ecs::PlayerSignals);
    let mut bridge = PhysicsBridge::new();

    let mut names: Vec<String> = Vec::new();
    for t in 1..=120u64 {
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: t <= 8,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        names.extend(
            bridge
                .signal_events(&sim, &ph2d_tags::TagTree::new())
                .into_iter()
                .map(|s| s.name),
        );
    }
    assert_eq!(
        names.iter().filter(|n| *n == "player.bonked").count(),
        1,
        "a batida tem de sair pelo barramento, uma vez: {names:?}"
    );
}
