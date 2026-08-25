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

/// ⭐⭐ **O KERNEL DA GPU ESPELHA AS TRÊS CONSTANTES, e agora há quem o diga.**
///
/// ⛔ O WGSL é uma STRING: a razão entre camadas, o ganho e o passo de fase vivem lá como
/// números soltos (`/ 1.618034`, `* 0.5`, `* 0.618034`), copiados à mão dos `const` acima.
/// Até 2026-08-25 **nenhum gate ligava os dois lados** — mudar a constante em Rust deixava a
/// GPU a calcular outro mar, e a paridade CPU/GPU só reprovaria numa máquina com adapter, em
/// gates `#[ignore]` que o CI nunca corre.
///
/// ⚠️ **Este gate é TEXTUAL de propósito**: ele corre sem GPU, em todo o lado, e apanha a
/// divergência no instante em que ela é escrita — que é a única altura em que ela é barata.
/// *Uma lei escrita em dois sítios ainda não é uma lei; aqui a segunda cópia é obrigatória
/// (é outra linguagem), então o que se constrói é a PONTE.*
#[test]
fn the_gpu_kernel_mirrors_the_three_spectrum_constants() {
    let wgsl = GPU_KERNEL.wgsl;
    for (needle, name) in [
        (format!("/ {WAVE_LACUNARITY};"), "WAVE_LACUNARITY"),
        (format!("* {WAVE_GAIN};"), "WAVE_GAIN"),
        (format!("* {PHASE_STEP})"), "PHASE_STEP"),
    ] {
        assert!(
            wgsl.contains(&needle),
            "o WGSL nao contem `{needle}` -- a constante {name} de Rust e a copia da GPU \
             divergiram, e o mar sai diferente nos dois caminhos"
        );
    }
    // CONTROLE: a razão NÃO é inteira — é isso que impede a soma de se repetir.
    assert!(
        (WAVE_LACUNARITY - WAVE_LACUNARITY.round()).abs() > 0.1,
        "uma razao inteira torna a soma de senos exactamente periodica ({WAVE_LACUNARITY})"
    );
}

/// ⭐⭐⭐ **O MAR NÃO SE REPETE** — o report do Enio de 2026-08-25: *«há dois formatos de onda
/// juntas mas regulares e não irregulares»*.
///
/// ⛔ **Nenhuma régua anterior podia ver isto.** A variedade de alturas de crista mede o
/// ESPALHAMENTO; um desenho que se repete três vezes tem exactamente o mesmo espalhamento de
/// um que nunca se repete. *«Irregular» não é «as cristas têm alturas diferentes» — é «a
/// sequência não volta»*, e são duas propriedades distintas que uma régua só lia como uma.
///
/// A lei: com razão entre camadas `r`, a camada `k` tem comprimento `λ/rᵏ` e completa `rᵏ`
/// ciclos sobre uma distância `λ`. Com `r` INTEIRO isso é um número inteiro de ciclos para
/// toda a camada ⇒ **soma exactamente periódica**. Medido com `r = 2`: `0,000008` da
/// amplitude de diferença entre `x` e `x + λ` — o mesmo desenho, repetido.
///
/// ⚠️ **O CONTROLO é a senoide única**, e ele tem de dar ZERO: um seno **é** periódico por
/// definição, e uma régua que o acusasse estaria a medir outra coisa.
#[test]
fn the_summed_sea_does_not_repeat_itself() {
    let (amp, lambda, speed) = (0.25_f32, 2.5_f32, 0.7_f32);
    let worst_at = |waves: f32| {
        (0..1024)
            .map(|i| {
                let x = -4.0 + 4.0 * i as f32 / 1023.0;
                let a = surface_at(x, 1.3, 0.0, amp, lambda, speed, waves);
                let b = surface_at(x + lambda, 1.3, 0.0, amp, lambda, speed, waves);
                (a - b).abs() / amp
            })
            .fold(0.0_f32, f32::max)
    };
    let plain = worst_at(1.0);
    assert!(
        plain < 1e-3,
        "CONTROLE: um seno E' periodico por definicao, e a regua tem de o dizer ({plain:.6})"
    );
    for waves in [2.0_f32, 3.0, 4.0] {
        let w = worst_at(waves);
        assert!(
            w > 0.5,
            "com {waves} camadas o mar repete-se a cada comprimento de onda \
             (diferenca {w:.6} da amplitude) -- a razao entre camadas e' inteira?"
        );
    }
}
