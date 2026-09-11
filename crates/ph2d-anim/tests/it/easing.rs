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

/// **O modo está morto exactamente onde o catálogo diz que está** — o gate que faz do
/// [`ph2d_anim::EasingFamily::uses_mode`] uma MEDIÇÃO em vez de uma afirmação.
///
/// Ele decide uma UI: um seletor de curva que oferecesse os três modos de uma família cujo
/// `eval` os ignora pintaria dois controlos que não fazem nada. A pergunta *"esta família usa o
/// modo?"* é respondida UMA vez, pelo enum, e é aqui que ela é conferida contra as curvas.
///
/// ⚠️ As duas metades importam. Sem a segunda (*"onde diz `true`, os modos DIFEREM"*), o
/// predicado podia responder `false` para tudo e o gate ficava verde com o seletor inteiro
/// escondido.
#[test]
fn the_mode_is_dead_exactly_where_the_catalogue_says_it_is() {
    use ph2d_anim::{Easing, EasingFamily, EasingMode};
    for f in EasingFamily::ALL {
        // A maior distância entre duas das três curvas desta família, sobre 101 amostras.
        let mut spread = 0.0_f64;
        for a in EasingMode::ALL {
            for b in EasingMode::ALL {
                for i in 0..=100 {
                    let u = f64::from(i) / 100.0;
                    let d = (Easing::new(f, a).eval(u) - Easing::new(f, b).eval(u)).abs();
                    spread = spread.max(d);
                }
            }
        }
        if f.uses_mode() {
            assert!(
                spread > 1e-6,
                "{}: uses_mode() diz que o modo importa, e as tres curvas coincidem (spread {spread:.3e}) \
                 -- o seletor ofereceria tres chips identicos",
                f.label()
            );
        } else {
            assert!(
                spread == 0.0,
                "{}: uses_mode() diz que o modo e' inerte, e as curvas DIFEREM (spread {spread:.3e}) \
                 -- esconder os chips esconderia uma escolha real",
                f.label()
            );
        }
    }
}

/// **Nenhuma família se chama como outra.** Um catálogo com dois rótulos iguais é um seletor em
/// que duas linhas dizem a mesma palavra e fazem coisas diferentes.
#[test]
fn every_family_has_its_own_name() {
    use ph2d_anim::EasingFamily;
    for a in EasingFamily::ALL {
        for b in EasingFamily::ALL {
            assert!(
                a == b || a.label() != b.label(),
                "duas familias partilham o rotulo {:?}",
                a.label()
            );
        }
    }
}
