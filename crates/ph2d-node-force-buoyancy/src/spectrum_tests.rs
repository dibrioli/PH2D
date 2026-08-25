//! Os gates do **ESPECTRO** do mar (doc 89, folha 02).

use super::*;

const LEVEL: f32 = 0.0;
const AMP: f32 = 0.6;
const LAMBDA: f32 = 3.0;
const SPEED: f32 = 1.0;

/// ⭐ **`waves = 1` É A SENOIDE DE SEMPRE, AO BIT** — e o ramo é que o garante.
#[test]
fn a_single_wave_is_the_old_sine_bit_for_bit() {
    for k in 0..400 {
        let x = k as f32 * 0.05 - 10.0;
        let t = k as f32 * 0.017;
        let (h, sl) = sea_at(x, t, LEVEL, AMP, LAMBDA, SPEED, 1);
        let phase = (x - SPEED * t) / LAMBDA;
        let (cos, sin) = cos_sin_cycles(phase);
        assert_eq!(
            h.to_bits(),
            (LEVEL + AMP * sin).to_bits(),
            "altura em x={x}"
        );
        assert_eq!(
            sl.to_bits(),
            (AMP * (std::f32::consts::TAU / LAMBDA) * cos).to_bits(),
            "inclinacao em x={x}"
        );
    }
}

/// ⭐⭐ **O ESPECTRO É UM MAR, e não uma senoide maior.**
///
/// A régua é a que separa as duas coisas: uma senoide tem **um** comprimento, então a
/// distância entre cristas vizinhas é sempre a mesma. Somando camadas as cristas deixam de
/// ser equidistantes — é isso que faz um mar ler como mar.
#[test]
fn the_spectrum_breaks_the_single_wavelength() {
    let crests = |waves: i32| -> Vec<f32> {
        let mut out = Vec::new();
        let mut prev = sea_at(-12.0, 0.0, LEVEL, AMP, LAMBDA, SPEED, waves).1;
        for k in 1..2400 {
            let x = k as f32 * 0.01 - 12.0;
            let s = sea_at(x, 0.0, LEVEL, AMP, LAMBDA, SPEED, waves).1;
            // Uma crista é onde a inclinação passa de positiva a negativa.
            if prev > 0.0 && s <= 0.0 {
                out.push(x);
            }
            prev = s;
        }
        out
    };
    let spread = |v: &[f32]| -> f32 {
        let gaps: Vec<f32> = v.windows(2).map(|w| w[1] - w[0]).collect();
        let (lo, hi) = gaps
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), g| (a.min(*g), b.max(*g)));
        hi - lo
    };
    let one = crests(1);
    let four = crests(4);
    assert!(
        one.len() > 5,
        "a senoide tem cristas que medir: {}",
        one.len()
    );
    assert!(
        spread(&one) < 0.05,
        "CONTROLE: uma senoide tem cristas EQUIDISTANTES (dispersao {:.4})",
        spread(&one)
    );
    assert!(
        four.len() > one.len(),
        "o espectro tem MAIS cristas: {} contra {}",
        four.len(),
        one.len()
    );
    assert!(
        spread(&four) > 0.2,
        "e elas deixam de ser equidistantes (dispersao {:.4})",
        spread(&four)
    );
}

/// ⭐⭐ **O DESENCONTRO DE FASE VARIA A ALTURA DAS CRISTAS** — e este gate existe porque a
/// justificação anterior foi refutada por ele.
///
/// ⛔ **A primeira versão dizia «sem o desencontro todas as cristas coincidem» e uma
/// mutação que o apagava SOBREVIVEU — duas vezes.** A premissa estava errada em dois
/// pontos: `fase = 0` é o cruzamento por ZERO de um seno, não a crista; e com comprimentos
/// **harmónicos** as camadas completam números de ciclos diferentes, logo **nunca** cristam
/// juntas, com ou sem deslocamento.
///
/// ⭐ **O que ele compra, medido:** o pico da superfície sobe de `0,7563` (em fase) para
/// `1,0251` — cristas mais VARIADAS em altura para a mesma energia, que é o que separa um
/// mar de um padrão. A referência da comparação é a alternativa refutada, computada aqui.
#[test]
fn the_octaves_never_all_crest_at_once() {
    // A alternativa refutada: as mesmas quatro camadas, todas com a mesma fase.
    let aligned_at = |x: f32| -> f32 {
        let (mut h, mut a, mut l) = (0.0_f32, AMP, LAMBDA);
        for _ in 0..4 {
            let (_, sin) = cos_sin_cycles(x / l);
            h += a * sin;
            a *= 0.5;
            l /= 2.0;
        }
        h
    };
    let peak = |f: &dyn Fn(f32) -> f32| -> f32 {
        (0..8000)
            .map(|k| f(k as f32 * 0.005 - 20.0).abs())
            .fold(0.0_f32, f32::max)
    };
    let shipped = peak(&|x| sea_at(x, 0.0, 0.0, AMP, LAMBDA, SPEED, 4).0);
    let aligned = peak(&aligned_at);
    let stacked = AMP + AMP * 0.5 + AMP * 0.25 + AMP * 0.125;
    println!("pico shipado {shipped:.4} · pico em fase {aligned:.4} · empilhado {stacked:.4}");
    assert!(
        aligned > stacked * 0.5,
        "CONTROLE: a alternativa em fase tem de produzir um mar de facto ({aligned:.4})"
    );
    assert!(
        shipped > aligned * 1.2,
        "o desencontro de fase tinha de VARIAR mais as cristas: pico {shipped:.4} contra \
         {aligned:.4} em fase -- ele esta' a faltar"
    );
    // E os dois ficam abaixo do empilhamento total: nenhuma soma de harmónicos o alcança.
    assert!(
        shipped < stacked,
        "nem o desencontro empilha tudo: {shipped:.4} contra {stacked:.4}"
    );
}

/// ⚠️ **Cada camada é METADE da anterior**, então a soma é limitada: um mar de 4 ondas
/// nunca passa de `2×` a amplitude autorada, e não de `4×`.
#[test]
fn the_spectrum_is_bounded_by_twice_the_authored_amplitude() {
    let mut hi = f32::MIN;
    for k in 0..4000 {
        let x = k as f32 * 0.01 - 20.0;
        hi = hi.max(sea_at(x, 0.0, LEVEL, AMP, LAMBDA, SPEED, 4).0.abs());
    }
    assert!(hi <= AMP * 2.0, "o mar passou do dobro: {hi:.4}");
    assert!(
        hi > AMP,
        "e ele de facto cresceu acima de uma onda so': {hi:.4}"
    );
}

/// O param é totalizado e aparado: um fio pode entregar qualquer coisa.
#[test]
fn the_wave_count_is_totalised() {
    assert_eq!(wave_count(1.0), 1);
    assert_eq!(wave_count(0.0), 1, "menos de uma onda e' uma onda");
    assert_eq!(wave_count(-9.0), 1);
    assert_eq!(wave_count(99.0), MAX_WAVES, "o tecto morde");
    assert_eq!(wave_count(f32::NAN), 1);
    assert_eq!(MANIFEST.param_default(WAVES), Some(1.0));
}
