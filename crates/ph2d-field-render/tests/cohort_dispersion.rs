//! ⭐⭐ **DISPERSAR AS COORTES COMPRA ALGUMA COISA?** — a varredura da
//! [`ph2d_field_render::PHASE`] (W89).
//!
//! # A pergunta
//!
//! Toda região compilada no mesmo quadro tem a mesma folga em todas as direcções, então sai da
//! caixa no **mesmo quadro seguinte**: as falhas chegam em **lote**, e o lote auto-sustenta-se
//! (quem recompila junto volta a expirar junto). Deslocar o **centro** da caixa dentro da folga que
//! a inflação já pagou mantém o volume — logo o preço por amostra — e muda quanto cada região ainda
//! pode derivar. *Se a coorte é o mecanismo, dispersá-la baixa o máximo sem mexer na média.*
//!
//! ⚠️⚠️ **A 1.ª corrida desta varredura mediu OUTRA COISA.** Ela correu antes de a árvore sair da
//! fita, quando um despejo custava `270–365 ms`: as cinco amplitudes davam `~290 ms` de máximo
//! porque **o máximo era a tempestade**, não a coorte. *Uma cura medida numa fixtura onde o defeito
//! dominante é outro lê-se como inútil.* ⇒ esta corre no **regime**, com o defeito grande já curado.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test cohort_dispersion -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
use ph2d_field_render::{Orbit, TapeCache};

fn circulo(n: usize) -> FieldDoc {
    let c: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![c], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
            }),
            mods: Vec::new(),
        }],
        NodeId(0),
    )
    .expect("extrusão")
}

#[test]
#[ignore = "sonda; roda com --nocapture"]
fn measure_what_dispersing_the_cohorts_buys() {
    const QUADROS: usize = 90;
    const REGIME: usize = 40;
    const GRAUS: f64 = 2.0;
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = circulo(168);
    let amps = [0.0f32, 0.3, 0.5, 0.8];
    let mut tabela: Vec<Vec<(f64, f64, f64, usize)>> = vec![Vec::new(); amps.len()];
    for _ronda in 0..3 {
        for (ai, amp) in amps.iter().enumerate() {
            let cache = TapeCache::with_phase(*amp);
            let mut ms: Vec<f64> = Vec::new();
            let mut compila = 0usize;
            for i in 0..QUADROS {
                let cam = Orbit {
                    rotation: Orbit::from_yaw_pitch(
                        0.72 + (i as f64 * GRAUS).to_radians() as f32,
                        0.52,
                    )
                    .rotation,
                    ..Orbit::default()
                };
                let c0 =
                    ph2d_field_eval::hybrid::FLOAT_TAPES.load(std::sync::atomic::Ordering::Relaxed);
                let t0 = std::time::Instant::now();
                let _ = ph2d_field_render::trace_cached_for_test(
                    &doc,
                    &reg,
                    &cam,
                    426,
                    240,
                    false,
                    Some(&cache),
                );
                ms.push(t0.elapsed().as_secs_f64() * 1000.0);
                if i >= REGIME {
                    compila += ph2d_field_eval::hybrid::FLOAT_TAPES
                        .load(std::sync::atomic::Ordering::Relaxed)
                        - c0;
                }
            }
            let mut r: Vec<f64> = ms[REGIME..].to_vec();
            let media = r.iter().sum::<f64>() / r.len() as f64;
            r.sort_by(f64::total_cmp);
            tabela[ai].push((r[r.len() / 2], media, r[r.len() - 1], compila));
        }
    }
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("fase | mediana | média | MÁXIMO | compilações no regime");
    for (ai, amp) in amps.iter().enumerate() {
        println!(
            "{amp:4.1} | {:7.1} | {:5.1} | {:6.1} | {:8}",
            med(tabela[ai].iter().map(|r| r.0).collect()),
            med(tabela[ai].iter().map(|r| r.1).collect()),
            med(tabela[ai].iter().map(|r| r.2).collect()),
            med(tabela[ai].iter().map(|r| r.3 as f64).collect()) as usize
        );
    }
}
