//! **Propriedades do vocabulário de easing** que consumidores dependem — hoje, a única que
//! atravessa crates: o ESPELHO no tempo.
/// **O espelho no TEMPO de um easing é trocar `In` por `Out`** — a identidade que deixa um
/// fade percorrido de trás para frente *sentir* o easing autorado (Enio, 2026-08-01).
///
/// ⚠️ Ela é MEDIDA, não suposta: `1 − f_espelhado(1 − u) == f(u)` nas 11 famílias × 3 modos,
/// 101 amostras cada. `InOut` é o seu próprio espelho, e `Linear` também — e é por isso que
/// o `smoothstep` de fábrica dos fades não muda de forma nenhuma quando a direção inverte.
#[test]
fn mirroring_an_easing_in_time_is_swapping_in_for_out() {
    use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};
    let mut worst = 0.0_f64;
    let mut where_ = String::new();
    for f in [
        F::Linear,
        F::Sine,
        F::Quad,
        F::Cubic,
        F::Quart,
        F::Quint,
        F::Expo,
        F::Circ,
        F::Back,
        F::Elastic,
        F::Bounce,
    ] {
        for m in M::ALL {
            let e = Easing::new(f, m);
            for i in 0..=100 {
                let u = f64::from(i) / 100.0;
                let d = (e.eval(u) - (1.0 - e.mirrored().eval(1.0 - u))).abs();
                if d > worst {
                    worst = d;
                    where_ = format!("{f:?} {m:?} u={u}");
                }
            }
        }
    }
    assert!(
        worst < 1e-12,
        "o espelho não é In↔Out: {worst:e} em {where_}"
    );
    // …e o modo simétrico é o seu PRÓPRIO espelho — sem esta metade, um `mirrored` que
    // trocasse InOut por In passaria na asserção acima em toda família simétrica.
    assert_eq!(
        Easing::new(F::Quad, M::InOut).mirrored(),
        Easing::new(F::Quad, M::InOut)
    );
    assert_eq!(
        Easing::new(F::Quad, M::In).mirrored(),
        Easing::new(F::Quad, M::Out)
    );
}
