//! `std` (libm do sistema) × `DetMath`: variantes no MESMO processo, intercaladas, o MÍNIMO (MEDIR_VELOCIDADE.md).
use ph2d_det_math::DetMath;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 4096;

fn mede<T: Copy>(ent: &[T], f: impl Fn(T) -> f64) -> f64 {
    let t = Instant::now();
    let mut s = 0.0;
    for &v in ent {
        s += f(black_box(v));
    }
    black_box(s);
    t.elapsed().as_nanos() as f64 / ent.len() as f64
}

macro_rules! linha {
    ($nome:literal, $ent:expr, $a:expr, $b:expr) => {{
        let (mut ma, mut mb) = (f64::MAX, f64::MAX);
        for _ in 0..60 {
            ma = ma.min(mede($ent, $a));
            mb = mb.min(mede($ent, $b));
        }
        println!(
            "{:10} std {:6.1} ns · det {:6.1} ns · ×{:.2}",
            $nome,
            ma,
            mb,
            mb / ma
        );
    }};
}

#[test]
#[ignore = "medição: imprime std × det"]
fn velocidade_std_x_det() {
    let u: Vec<f32> = (0..N).map(|i| (i as f32 + 0.5) / N as f32).collect();
    let a: Vec<f32> = u.iter().map(|v| v * 6.28 - 3.14).collect();
    let p: Vec<(f32, f32)> = u
        .iter()
        .map(|&v| (v * 40.0 - 20.0, 13.0 - v * 31.0))
        .collect();
    let (ud, ad): (Vec<f64>, Vec<f64>) = (
        u.iter().map(|&v| f64::from(v)).collect(),
        a.iter().map(|&v| f64::from(v)).collect(),
    );
    let pd: Vec<(f64, f64)> = p
        .iter()
        .map(|&(x, y)| (f64::from(x), f64::from(y)))
        .collect();
    linha!(
        "hypot f32",
        &p,
        |(x, y): (f32, f32)| f64::from(x.hypot(y)),
        |(x, y): (f32, f32)| f64::from(x.det_hypot(y))
    );
    linha!(
        "hypot f64",
        &pd,
        |(x, y): (f64, f64)| x.hypot(y),
        |(x, y): (f64, f64)| x.det_hypot(y)
    );
    linha!("powf f32", &u, |v: f32| f64::from(v.powf(2.4)), |v: f32| {
        f64::from(v.det_powf(2.4))
    });
    linha!("powf f64", &ud, |v: f64| v.powf(2.4), |v: f64| v
        .det_powf(2.4));
    linha!("sin f32", &a, |v: f32| f64::from(v.sin()), |v: f32| {
        f64::from(v.det_sin())
    });
    linha!("sin f64", &ad, |v: f64| v.sin(), |v: f64| v.det_sin());
    linha!("cos f32", &a, |v: f32| f64::from(v.cos()), |v: f32| {
        f64::from(v.det_cos())
    });
    linha!("cos f64", &ad, |v: f64| v.cos(), |v: f64| v.det_cos());
    linha!("exp f32", &a, |v: f32| f64::from(v.exp()), |v: f32| {
        f64::from(v.det_exp())
    });
    linha!("exp f64", &ad, |v: f64| v.exp(), |v: f64| v.det_exp());
    linha!(
        "ln f32",
        &u,
        |v: f32| f64::from(v.ln()),
        |v: f32| f64::from(v.det_ln())
    );
    linha!(
        "atan2 f32",
        &p,
        |(x, y): (f32, f32)| f64::from(y.atan2(x)),
        |(x, y): (f32, f32)| f64::from(y.det_atan2(x))
    );
    linha!(
        "sin_cos f32",
        &a,
        |v: f32| f64::from(v.sin_cos().0),
        |v: f32| f64::from(v.det_sin_cos().0)
    );
}
