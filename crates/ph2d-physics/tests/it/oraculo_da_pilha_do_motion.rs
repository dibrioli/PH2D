//! ⭐⭐⭐ **O ORÁCULO DA PILHA** — a MESMA cena `=114` do Motion, corrida por um solver de impulsos
//! sequenciais com `λ` acumulado (doc 109 §8.14, ordem do dono de 2026-09-15: *«encomendas o motor
//! de contacto novo»*).
//!
//! ⚠️ **Passo 1 do `CLAUDE.md` §0.9 é a TRIAGEM, e ela parou na primeira porta ABERTA:** o solver
//! que a medição do doc 109 §8.14 encomendou **já está na árvore** — `rapier2d` 0.35.3,
//! Apache-2.0, a dependência que a Física usa desde o ADR-0131. ⇒ ele corre-se como oráculo em vez
//! de se escrever um, e o que esta bancada mede é **o alvo**: o que «assentar» vale em número.
//!
//! ⛔ **Isto NÃO é um gate do produto** — o Motion não usa este solver hoje. É a referência contra a
//! qual a obra encomendada se mede, e o corpus de onde as barras dela vão sair.
//!
//! As duas réguas são as do `ph2d-app-motion` (`motion_state_pilha_demo_tremor.rs`): o `|Δângulo|`
//! MEDIANO por tique de cada peça na janela assente, e o ângulo LÍQUIDO que ela percorre.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

/// As constantes da cena `=114`, lidas de `motion_state_pilha_demo.rs`.
const LADO: f32 = 0.11;
/// A meia extensão da caixa do `Square` (o `[√2, 1]` que o `source.shape` declara, sobre `size/2`).
const MEIA: f32 = LADO * std::f32::consts::SQRT_2 / 2.0;
const GAP: f32 = 0.32;
const ALTURA: f32 = -0.35;
const TACA_Y: f32 = -1.2;
const TACA_R: f32 = 1.8;
const GRAVIDADE: f32 = 4.0;
/// A gravidade que o `rapier` aplica por omissão — o `gravity_scale` converte para a da cena.
const G_RAPIER: f32 = 9.81;

fn desc(body_type: RigidBodyType, x: f32, y: f32, shape: ShapeDesc) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.05,
        friction: 0.6,
        layer: 0,
        is_sensor: false,
        gravity_scale: GRAVIDADE / G_RAPIER,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// A taça: um anel de caixas FIXAS tangentes por dentro ao círculo `(0, TACA_Y)` de raio `TACA_R`.
///
/// ⚠️ O `ShapeDesc` desta casa não tem forma côncava, e uma taça é côncava — 64 segmentos dão um
/// erro de sagitta de `TACA_R · (1 − cos(π/64)) ≈ 0,002`, que é `2,6 %` do lado de uma peça.
fn taca(w: &mut PhysicsWorld) {
    const SEGS: usize = 64;
    let meia_corda = TACA_R * std::f32::consts::PI / SEGS as f32;
    for k in 0..SEGS {
        let a = std::f32::consts::TAU * k as f32 / SEGS as f32;
        let (s, c) = a.sin_cos();
        // O centro do segmento fica FORA do círculo por meia espessura, para a face interior dele
        // cair sobre o raio.
        let esp = 0.1_f32;
        let r = TACA_R + esp;
        w.spawn_body(BodyDesc {
            rotation: a,
            ..desc(
                RigidBodyType::Fixed,
                c * r,
                TACA_Y + s * r,
                ShapeDesc::Cuboid {
                    half_x: esp,
                    half_y: meia_corda,
                },
            )
        });
    }
}

/// As 25 peças, na grelha da cena.
fn pecas(w: &mut PhysicsWorld, atrito: f32) -> Vec<RigidBodyHandle> {
    let mut hs = Vec::new();
    for r in 0..5 {
        for c in 0..5 {
            let x = (c as f32 - 2.0) * GAP;
            let y = ALTURA + (r as f32 - 2.0) * GAP;
            hs.push(w.spawn_body(BodyDesc {
                friction: atrito,
                ..desc(
                    RigidBodyType::Dynamic,
                    x,
                    y,
                    ShapeDesc::Cuboid {
                        half_x: MEIA,
                        half_y: MEIA,
                    },
                )
            }));
        }
    }
    hs
}

/// Corre `2,9 s` a 60 Hz e devolve, na janela `2,0..2,9`, `(balanço pior, giro líquido pior)` em
/// GRAUS — as duas réguas do Motion, termo a termo.
fn corre(atrito: f32) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    taca(&mut w);
    let hs = pecas(&mut w, atrito);
    let graus = 180.0 / std::f32::consts::PI;
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<f32>::new());
    let mut liquido = vec![0.0_f32; hs.len()];
    for k in 0..=174_u64 {
        w.step();
        let ang: Vec<f32> = hs
            .iter()
            .map(|h| w.body_pose(*h).map_or(0.0, |p| p.rotation.angle() * graus))
            .collect();
        if k >= 120 && anterior.len() == ang.len() {
            passos.push(
                ang.iter()
                    .zip(&anterior)
                    .map(|(a, b)| (a - b).abs())
                    .collect(),
            );
            for i in 0..ang.len() {
                liquido[i] += ang[i] - anterior[i];
            }
        }
        anterior = ang;
    }
    let mediana = |v: &[f32]| {
        let mut s = v.to_vec();
        s.sort_by(f32::total_cmp);
        s[s.len() / 2]
    };
    let balanco = (0..hs.len())
        .map(|i| mediana(&passos.iter().map(|l| l[i]).collect::<Vec<_>>()))
        .fold(0.0_f32, f32::max);
    (balanco, liquido.iter().fold(0.0_f32, |a, v| a.max(v.abs())))
}

/// **SONDA — o ALVO.** O que um solver com `λ` acumulado entrega na mesma pilha.
///
/// ```text
/// cargo test -p ph2d-physics --test it oraculo_da_pilha -- --ignored --nocapture
/// ```
#[test]
#[ignore = "bancada de referencia, nao um gate do produto"]
fn oraculo_da_pilha_do_motion() {
    eprintln!("\n  atrito das peças | balanço pior (°/tique) | giro líquido pior (°)");
    eprintln!("  -----------------|------------------------|----------------------");
    for atrito in [0.0_f32, 0.3, 0.6] {
        let (b, g) = corre(atrito);
        eprintln!("  {atrito:>16.2} | {b:>22.4} | {g:>21.2}");
    }
    eprintln!("\n  ⚠️ o Motion, na mesma cena, lê 3,79..5,69 e 24,30..28,36 (doc 109 §8.8).");
}

/// **SONDA — O TECTO DE OBJECTOS do oráculo**, que é o número que DECIDE o desenho da obra
/// encomendada (`CLAUDE.md` §0.0: *nunca deixe o fallback definir o produto*).
///
/// ⚠️⚠️ O Motion é **GPU-resident por omissão** e a [auditoria 98] mediu **4,19 M objectos em
/// 3,85 ms** no dispositivo. Um solver de corpo rígido é CPU e tem ilhas, contactos persistentes e
/// broad-phase: se a obra encomendada for *«passar a sim do Motion a usar este solver»*, o tecto do
/// módulo passa a ser ESTE número. ⇒ mede-se ANTES de escrever a espec, não depois.
///
/// ```text
/// cargo test -p ph2d-physics --test it probe_o_tecto_do_oraculo -- --ignored --nocapture --release
/// ```
#[test]
#[ignore = "sonda de medicao — corra em RELEASE"]
fn probe_o_tecto_do_oraculo() {
    eprintln!("\n  peças | ms por tique | quadro de 16,7 ms");
    eprintln!("  ------|--------------|------------------");
    for lado in [10_usize, 20, 40, 80] {
        let n = lado * lado;
        let mut w = PhysicsWorld::new();
        taca_grande(&mut w, lado);
        for r in 0..lado {
            for c in 0..lado {
                let x = (c as f32 - lado as f32 / 2.0) * MEIA * 2.2;
                let y = ALTURA + (r as f32) * MEIA * 2.2;
                w.spawn_body(BodyDesc {
                    friction: 0.6,
                    ..desc(
                        RigidBodyType::Dynamic,
                        x,
                        y,
                        ShapeDesc::Cuboid {
                            half_x: MEIA,
                            half_y: MEIA,
                        },
                    )
                });
            }
        }
        // Deixa assentar, e só então mede — um tique de queda livre não tem contactos.
        for _ in 0..180 {
            w.step();
        }
        let t = std::time::Instant::now();
        for _ in 0..60 {
            w.step();
        }
        let ms = t.elapsed().as_secs_f64() * 1e3 / 60.0;
        eprintln!(
            "  {n:>5} | {ms:>9.3} ms | {:>17}",
            if ms <= 16.7 { "cabe" } else { "⛔ ESTOURA" }
        );
    }
    eprintln!("\n  ⚠️ o dispositivo faz 4,19 M objectos em 3,85 ms (auditoria 98).");
}

/// Uma taça larga o bastante para `lado²` peças.
fn taca_grande(w: &mut PhysicsWorld, lado: usize) {
    let meia = MEIA * 2.2 * lado as f32;
    for (x, y, hx, hy) in [
        (0.0, ALTURA - 0.2, meia, 0.1_f32),
        (-meia, ALTURA + meia, 0.1, meia),
        (meia, ALTURA + meia, 0.1, meia),
    ] {
        w.spawn_body(desc(
            RigidBodyType::Fixed,
            x,
            y,
            ShapeDesc::Cuboid {
                half_x: hx,
                half_y: hy,
            },
        ));
    }
}
