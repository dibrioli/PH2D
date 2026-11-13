//! Os gates da FAIXA de gravidade dentro do tique (GEF 04): o tique divide-se onde a velocidade
//! cruza um limiar, então o pico nunca desce quando a Jump Height sobe — nem no pulo cortado.
use super::*;
use crate::feel::Feel;

const G: f32 = 9.81;
const DT: f32 = 1.0 / 60.0;

fn ground() -> GroundSample {
    GroundSample {
        grip: 1.0,
        distance: 0.9,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
        brink: crate::Brink::NONE,
    }
}

fn cfg_of(feel: Option<Feel>, h: f32) -> JumpConfig {
    let mut c = JumpConfig::STARTING_POINT;
    if let Some(f) = feel {
        let g = f.gravity();
        c.takeoff_gravity = g.takeoff_gravity;
        c.takeoff_speed = g.takeoff_speed;
        c.peak_gravity = g.peak_gravity;
        c.peak_speed = g.peak_speed;
        c.fall_gravity = g.fall_gravity;
        c.cut_gravity = g.cut_gravity;
    }
    c.jump_height = h;
    c
}

/// O pico do pulo sob a lei pura e o integrador do mundo (`world_steps` passos de Euler
/// semi-implícito, o motor como impulso no topo do tique): botão preso `held` tiques
/// (`None` = o pulo cheio).
fn peak(cfg: &JumpConfig, held: Option<u32>) -> f32 {
    let n = cfg.world_steps.max(1);
    let h = DT / n as f32;
    let (mut st, mut v, mut p, mut top) = (JumpState::default(), 0.0_f32, 0.0_f32, 0.0_f32);
    for tick in 0..2000_u32 {
        let press = held.is_none_or(|k| tick < k);
        let foot = ground();
        let s = jump_step(
            cfg,
            st,
            (tick == 0).then_some(&foot),
            v,
            press,
            false,
            None,
            [0.0, -G],
            [0.0, 1.0],
            DT,
            crate::Buoyed::DRY,
        );
        st = s.state;
        v += s.motor.accel[1] * DT + s.motor.boost[1];
        for _ in 0..n {
            v -= G * h;
            p += v * h;
        }
        top = top.max(p);
        if v < 0.0 && p < top - 0.5 {
            break;
        }
    }
    top
}

const FEELS: [Option<Feel>; 4] = [
    None,
    Some(Feel::Floaty),
    Some(Feel::Snappy),
    Some(Feel::Heavy),
];

/// ⭐ A VARREDURA de monotonia (Jump Height 0,5–3,0 a cada 1 cm, 4 sensações, pulo cheio e
/// cortado nos tiques 1..5): o pico nunca desce quando a altura sobe (0,1 mm). Com a faixa lida
/// no início do tique o cortado caía ~0,1 m a cada degrau (1,15 → 1,20).
#[test]
fn o_pico_nunca_desce_quando_a_altura_sobe() {
    let mut falhas = Vec::new();
    for feel in FEELS {
        for held in [None, Some(1), Some(2), Some(3), Some(4), Some(5)] {
            let mut antes: Option<(f32, f32)> = None;
            for i in 0..=250 {
                let h = 0.5 + i as f32 * 0.01;
                let top = peak(&cfg_of(feel, h), held);
                if let Some((ha, ta)) = antes
                    && top < ta - 1.0e-4
                {
                    falhas.push(format!(
                        "{feel:?} {held:?}: {ha:.2}→{h:.2} pico {ta:.4}→{top:.4}"
                    ));
                }
                antes = Some((h, top));
            }
        }
    }
    for h in [1.15_f32, 1.20, 1.50, 1.55] {
        eprintln!(
            "cortado no tique 3, h={h}: pico {:.4}",
            peak(&cfg_of(None, h), Some(3))
        );
    }
    assert!(
        falhas.is_empty(),
        "o pico faz serra ({}): {:?}",
        falhas.len(),
        &falhas[..falhas.len().min(12)]
    );
}

/// O pulo cheio da lei pura sobe a Jump Height (≤ 1 mm) com o tique dividido.
#[test]
fn o_pulo_cheio_da_lei_sobe_a_jump_height() {
    let mut falhas = Vec::new();
    for feel in FEELS {
        for i in 0..=25 {
            let h = 0.5 + i as f32 * 0.1;
            let top = peak(&cfg_of(feel, h), None);
            if (top - h).abs() > 1.0e-3 {
                falhas.push(format!("{feel:?} h={h:.2}: {top:.5}"));
            }
        }
    }
    assert!(falhas.is_empty(), "o ápice da lei mente: {falhas:?}");
}

/// A divisão é exacta: um tique que cruza o limiar muda a velocidade pelo integral dos trechos.
#[test]
fn o_tique_divide_se_no_limiar() {
    let c = cfg_of(None, 2.0);
    let ps = c.peak_speed;
    // Meio tique acima do limiar sob o corte, meio abaixo sob o ápice.
    let v = ps + c.cut_gravity * G * DT * 0.5;
    let s = jump_config::tick_gravity_scale(&c, true, v, G, DT);
    let want = 0.5 * (c.cut_gravity + c.peak_gravity);
    assert!((s - want).abs() < 1.0e-4, "escala média {s} vs {want}");
    // Longe de qualquer limiar é a faixa pura.
    assert_eq!(
        jump_config::tick_gravity_scale(&c, true, 20.0, G, DT),
        c.cut_gravity
    );
}
