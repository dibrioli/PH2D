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
    /// O tique de cada `Bonked`.
    bonk_ticks: Vec<u64>,
    /// A subida de cada tique (m/s, diferença de posição; índice = tique − 1).
    vy: Vec<f32>,
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
    go_as(false, jump_height, corner_reach, ceiling_bottom)
}

/// Põe o player em Snap (`PlayerMode::Kinematic` + corpo cinemático) — o gesto da §14 escreve os
/// dois por uma porta.
fn snap(sim: &mut SimWorld, who: ph2d_ecs::Entity) {
    let mut e = sim.world_mut().entity_mut(who);
    e.insert(ph2d_physics_ecs::PlayerMode::Kinematic);
    if let Some(mut rb) = e.get_mut::<RigidBody>() {
        rb.kind = BodyKind::Kinematic;
    }
}

/// A corrida, dinâmica (`kinematic = false`) ou sob Snap.
fn go_as(kinematic: bool, jump_height: f32, corner_reach: f32, ceiling_bottom: Option<f32>) -> Run {
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
    if kinematic {
        snap(&mut sim, who);
    }
    let mut bridge = PhysicsBridge::new();
    let mut start = scene::y_of(&sim, "Subject");
    let (mut prev, mut top, mut vmax) = (start, start, 0.0_f32);
    // Sob Snap o corpo assenta noutra altura que a do spawn: a cabeça mede-se assente.
    let rest_tick = if kinematic { 59 } else { 0 };
    let (mut bonks, mut lit, mut was_lit) = (Vec::new(), 0, false);
    let (mut bonk_ticks, mut vy) = (Vec::new(), Vec::new());
    for t in 1..=150u64 {
        // Segura o pulo até depois do ápice do mais alto (2,6 m ≈ 44 tiques):
        // soltar antes CORTA a subida, e a altura medida deixaria de ser a lei.
        bridge.set_player_input(
            who,
            PlayerInput {
                // Sob Snap o corpo assenta devagar (medido: apertar no tique 20 subia 0,36 m a H = 1,0;
                // no 60, 0,96): o aperto vem depois de assentar.
                jump: if kinematic {
                    (60..=120).contains(&t)
                } else {
                    t <= 60
                },
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
                bonk_ticks.push(t);
            }
        }
        let y = scene::y_of(&sim, "Subject");
        if t == rest_tick {
            (start, prev, top, vmax) = (y, y, y, 0.0);
        }
        vy.push((y - prev) * 60.0);
        vmax = vmax.max((y - prev) * 60.0);
        top = top.max(y);
        prev = y;
    }
    let lid = scene::HALF_H + scene::RADIUS;
    Run {
        bonks,
        bonk_ticks,
        vy,
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
// `speed` que o evento carrega é a subida PRETENDIDA no tique do choque (a ponte
// carimba-a nos dois caminhos, `bridge::ceiling_fact`), e um teste que soletrasse
// as duas vistas à mão estaria a afirmar o número que ele próprio escreveu. O oráculo é um teto de
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
/// Jump Height 1,0–2,6 (passo 0,05) × fundo do teto a cabeça + 0,02, + 0,05 e
/// de cabeça + 0,10 até o topo livre (passo 0,05), com a quina armada e
/// desarmada: exactamente UMA batida
/// (o bit acende uma vez E um `Bonked`), nenhuma com o teto fora do alcance
/// (topo livre + 0,05 e + 0,10), a subida nunca acima do `v0` da lei, e a
/// velocidade publicada nunca CRESCE quando o teto sobe.
///
/// ⚠️ **O gate que estava verde era de UM ponto** (teto 1,9 com 1,3146 m): os
/// tetos baixos — os primeiros ~0,3 m da subida, onde a perna ainda vê o chão —
/// não eram medidos, e ali a porta do teto lia o sensor cru como *no chão*.
///
/// ⚠️ **E os tetos DENTRO do tique da decolagem** (+ 0,02 e + 0,05, Bug #12b): a
/// sonda corre antes de a lei lançar e não pergunta; quem os vê é o teto medido
/// DEPOIS do passo, pelos contactos (`bridge::ceiling_fact`).
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
            let mut offsets = vec![0.02_f32, 0.05];
            offsets.extend(
                (0..)
                    .map(|k| 0.10 + 0.05 * k as f32)
                    .take_while(|o| free.head + o < free.top),
            );
            for b in offsets.into_iter().map(|o| free.head + o) {
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

/// **O teto DENTRO do tique da decolagem é batido** (Bug #12b) — cabeça + 0,05.
///
/// A sonda de antes do passo não pergunta (`rel_up` < 0 antes do lançamento) e o
/// solver encosta a cabeça no teto no mesmo tique. O facto medido depois do passo
/// publica UMA batida, no tique do pulo ou no seguinte, com a subida que o pulo
/// pretendia (o `v0` da lei, não o resto que sobrou).
#[test]
fn a_ceiling_inside_the_takeoff_tick_is_bonked_with_the_launch_speed() {
    let g = -ph2d_physics::PhysicsWorld::DEFAULT_GRAVITY_Y;
    let v0 = ph2d_platformer::launch_speed(&jumper(0.12).config().jump, g, H, false, 1.0 / 60.0);
    for reach in [0.12_f32, 0.0] {
        let head = go(H, reach, None).head;
        let mut sim = SimWorld::new();
        floor(&mut sim, 0.0);
        ceiling(&mut sim, head + 0.05);
        let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(reach)));
        let mut bridge = PhysicsBridge::new();
        let (mut jumped, mut bonked) = (None, Vec::new());
        for t in 1..=90u64 {
            bridge.set_player_input(
                who,
                PlayerInput {
                    jump: t >= 30,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, t);
            for (_, ev) in bridge.player_events() {
                match ev {
                    ph2d_platformer::PlayerEvent::Jumped { .. } => jumped = jumped.or(Some(t)),
                    ph2d_platformer::PlayerEvent::Bonked { speed } => bonked.push((t, *speed)),
                    _ => {}
                }
            }
        }
        let jt = jumped.expect("o controle: o pulo tem de sair");
        assert_eq!(
            bonked.len(),
            1,
            "q={reach}: uma batida, um evento: {bonked:?}"
        );
        let (bt, speed) = bonked[0];
        assert!(
            bt == jt || bt == jt + 1,
            "q={reach}: a batida no tique do pulo ({jt}) ou no seguinte, nao em {bt}"
        );
        assert!(
            (speed - v0).abs() < 0.25 * v0,
            "q={reach}: a velocidade publicada e' a subida pretendida ({v0:.3}), nao {speed:.3}"
        );
    }
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

/// **Sob Snap o teto também é um facto, uma vez por batida** (Bug #12b, o caminho do movimento).
///
/// O corpo cinemático não gera contacto no solver: quem vê o teto dentro do tique da decolagem é
/// o `apply_kin_moves` (subida pedida, cortada contra uma normal de teto). Fundo do teto a
/// cabeça + 0,02, + 0,05 e de + 0,10 até o topo livre, em quatro alturas de pulo, quina armada e
/// desarmada: exactamente um `Bonked` e uma borda do bit; nada a topo livre + 0,10; `speed`
/// nunca cresce quando o teto sobe. De pé sob a marquise, sob Snap, não é batida.
///
/// ⚠️ **Mutação que deve sangrar:** desligar o reporte do corte (`let cut = false && …` em
/// `player_kinmove`) → os tetos a + 0,02/+ 0,05 ficam sem batida.
#[test]
fn under_snap_every_ceiling_in_reach_is_bonked_once() {
    let mut bad = Vec::new();
    for h in [1.0_f32, 1.2, 1.8, 2.6] {
        for reach in [0.12_f32, 0.0] {
            let free = go_as(true, h, reach, None);
            if !free.rose {
                bad.push(format!(
                    "h={h} q={reach}: sob Snap nao saiu do chao (cabeca {:.3} topo {:.3} vy {:.3})",
                    free.head, free.top, free.vmax
                ));
                continue;
            }
            let mut offsets = vec![0.02_f32, 0.05];
            offsets.extend(
                (0..)
                    .map(|k| 0.10 + 0.10 * k as f32)
                    .take_while(|o| free.head + o < free.top - 0.02),
            );
            let mut last = f32::INFINITY;
            for o in offsets {
                let r = go_as(true, h, reach, Some(free.head + o));
                if r.bonks.len() != 1 || r.lit != 1 {
                    bad.push(format!(
                        "h={h} q={reach} cabeca+{o:.2} (livre {:.3}): {:?} lit={}",
                        free.top - free.head,
                        r.bonks,
                        r.lit
                    ));
                } else if r.bonks[0] > last + 1e-3 {
                    bad.push(format!(
                        "h={h} q={reach} cabeca+{o:.2}: {:.3} > {last:.3} (nao esvanece)",
                        r.bonks[0]
                    ));
                }
                last = r.bonks.first().copied().unwrap_or(last);
            }
            let r = go_as(true, h, reach, Some(free.top + 0.10));
            if !r.bonks.is_empty() || r.lit != 0 {
                bad.push(format!(
                    "h={h} q={reach} livre+0,10: {:?} lit={}",
                    r.bonks, r.lit
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{} falhas:\n{}", bad.len(), bad.join("\n"));

    // De pé sob a marquise, sob Snap: nunca acende.
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    ceiling(&mut sim, FLOAT + 0.6);
    let who = subject_tuned(&mut sim, true, FLOAT, Some(jumper(0.12)));
    snap(&mut sim, who);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=90u64 {
        bridge.set_player_input(who, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
        assert!(
            !bridge.player_view(who).is_some_and(|v| v.ceiling),
            "sob Snap, parado sob a marquise nao bate (tique {t})"
        );
        assert!(
            !bridge
                .player_events()
                .iter()
                .any(|(_, e)| matches!(e, ph2d_platformer::PlayerEvent::Bonked { .. })),
            "sob Snap, parado sob a marquise nao publica Bonked (tique {t})"
        );
    }
}

/// **`Bonked.speed` é UM critério nos dois caminhos** — a subida PRETENDIDA do tique do choque.
///
/// Até ao choque a corrida com teto é a corrida livre, então a subida pretendida do tique da
/// batida é a que a corrida livre ANDOU nesse tique (diferença de posição; o atrito da gravidade
/// dentro do tique fica abaixo de `g·dt/2` ≈ 0,08 m/s). A régua antiga da sonda (a vista, antes do
/// motor) é a do tique ANTERIOR — `g·dt` ≈ 0,16 m/s acima. Teto ao alcance da sonda (topo livre
/// − 0,6 e − 0,3) e dentro do tique da decolagem (cabeça + 0,02), dinâmico e Snap.
///
/// ⚠️ **Mutação que deve sangrar:** não recarimbar o `Bonked` da sonda em
/// `arm_ceiling_fact` (`if false && view.ceiling`) → sob Snap os tetos da sonda publicam a vista
/// (3,286 contra 3,123). Dinâmico as duas réguas coincidem fora da decolagem: a gravidade é do
/// solver e o motor da subida é zero (`takeoff_gravity = 1`).
#[test]
fn the_bonk_speed_is_the_intended_rise_on_both_paths() {
    let mut bad = Vec::new();
    for kinematic in [false, true] {
        let free = go_as(kinematic, H, 0.12, None);
        for b in [free.top - 0.6, free.top - 0.3, free.head + 0.02] {
            let r = go_as(kinematic, H, 0.12, Some(b));
            let (Some(&speed), Some(&t)) = (r.bonks.first(), r.bonk_ticks.first()) else {
                bad.push(format!("snap={kinematic} teto={b:.3}: sem batida"));
                continue;
            };
            let want = free.vy[t as usize - 1];
            if (speed - want).abs() > 0.1 {
                bad.push(format!(
                    "snap={kinematic} teto={b:.3} tique {t}: speed {speed:.3} contra a subida \
                     pretendida {want:.3}"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
