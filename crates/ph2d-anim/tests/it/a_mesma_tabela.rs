//! ⭐ **AS CURVAS DA TIMELINE SÃO AS DOS NÓS** (ciclo 17, doc 126 §3.1).
//!
//! A timeline avalia em `f64` (`ph2d_anim::Easing`); os nós do Motion em `f32` pela tabela da casa
//! (`ph2d_ease`), a mesma no dispositivo. Uma key `Back Out` e um `motion.stagger` com `Back Out`
//! têm de desenhar a MESMA curva — se uma família mudar de um lado só, este gate fica vermelho.

use ph2d_anim::{Easing, EasingFamily, EasingMode};

fn da_casa(f: EasingFamily) -> i32 {
    match f {
        EasingFamily::Linear => ph2d_ease::LINEAR,
        EasingFamily::Quad => ph2d_ease::QUAD,
        EasingFamily::Cubic => ph2d_ease::CUBIC,
        EasingFamily::Quart => ph2d_ease::QUART,
        EasingFamily::Quint => ph2d_ease::QUINT,
        EasingFamily::Back => ph2d_ease::BACK,
        EasingFamily::Bounce => ph2d_ease::BOUNCE,
        EasingFamily::Sine => ph2d_ease::SINE,
        EasingFamily::Expo => ph2d_ease::EXPO,
        EasingFamily::Circ => ph2d_ease::CIRC,
        EasingFamily::Elastic => ph2d_ease::ELASTIC,
    }
}

#[test]
fn every_timeline_family_is_the_house_curve() {
    let s = ph2d_ease::Shape::DEFAULT;
    let mut pior = 0.0_f64;
    for family in EasingFamily::ALL {
        for (dir, mode) in [EasingMode::In, EasingMode::Out, EasingMode::InOut]
            .into_iter()
            .enumerate()
        {
            let e = Easing { family, mode };
            for k in 0..=1024_u16 {
                let t = f32::from(k) / 1024.0;
                let a = e.eval(f64::from(t));
                #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap, reason = "0..=2")]
                let b = f64::from(ph2d_ease::ease(da_casa(family), dir as i32, t, &s));
                let d = (a - b).abs();
                pior = pior.max(d);
                assert!(d < 2e-6, "{family:?} {mode:?} t {t}: timeline {a} × nós {b}");
            }
        }
    }
    eprintln!("a mesma tabela: pior |f64 − f32| = {pior:e}");
}
