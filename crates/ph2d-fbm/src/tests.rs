//! Os gates da lei fractal.

use super::*;

/// Um ruído de base determinístico e SEPARÁVEL, para a lei ser aritmética à vista:
/// `n(x, y) = sin_free(x) · cos_free(y)`… não — mais simples e sem transcendental:
/// devolve `x` truncado ao ciclo, o que basta para as somas serem previsíveis.
fn ramp(x: f32, _y: f32, _o: u32) -> f32 {
    x - x.floor() - 0.5
}

/// **Uma oitava é o ruído de base, sem retoque.** `sum/total` com um termo é
/// `amp·n / amp` = `n`, e é o que faz `octaves = 1` ser o neutro da soma.
#[test]
fn one_octave_is_the_base_noise_itself() {
    let s = Spec {
        octaves: 1,
        ..Spec::default()
    };
    for x in [0.1f32, 0.7, 1.3, -2.4] {
        assert!(
            (eval(s, x, 0.0, ramp) - ramp(x, 0.0, 0)).abs() < 1e-7,
            "x = {x}"
        );
    }
    // E zero oitavas não é um campo vazio, é uma: `max(1)`.
    let z = Spec {
        octaves: 0,
        ..Spec::default()
    };
    assert_eq!(eval(z, 0.3, 0.0, ramp), eval(s, 0.3, 0.0, ramp));
}

/// **A LACUNARITY escala a coordenada por oitava, e a ROUGHNESS a amplitude.**
///
/// Com um ruído de base que devolve a própria coordenada, a soma tem forma
/// fechada: `Σ aᵏ·(L^k·x)` normalizada — então os dois knobs são checáveis por
/// aritmética em vez de por eyeball.
#[test]
fn lacunarity_scales_the_coordinate_and_roughness_the_amplitude() {
    fn ident(x: f32, _y: f32, _o: u32) -> f32 {
        x
    }
    let s = Spec {
        octaves: 3,
        lacunarity: 3.0,
        roughness: 0.25,
        ty: NoiseType::Fbm,
    };
    // Σ = 1·x + 0.25·3x + 0.0625·9x = x(1 + 0.75 + 0.5625) = 2.3125x
    // total = 1 + 0.25 + 0.0625 = 1.3125
    let expected = 2.3125 / 1.3125;
    assert!((eval(s, 1.0, 0.0, ident) - expected).abs() < 1e-6);
}

/// **O NEUTRO é o fBm clássico, e ele reduz ao laço `freq*2 / amp*0.5` que a
/// família de forças tinha CRAVADO.** É este gate que torna a adoção da folha
/// byte-idêntica em vez de uma promessa.
#[test]
fn the_default_spec_reproduces_the_hardcoded_doubling_halving_loop() {
    fn base(x: f32, y: f32, _o: u32) -> f32 {
        // Um stand-in do ruído de valor: determinístico e não-linear.
        (x * 0.37 + y * 0.11).sin_like()
    }
    for oct in 1..=6u32 {
        // A lei antiga, verbatim.
        let (mut freq, mut amp, mut sum, mut norm) = (1.0f32, 1.0f32, 0.0f32, 0.0f32);
        for _ in 0..oct {
            sum += base(1.7 * freq, -0.4 * freq, 0) * amp;
            norm += amp;
            freq *= 2.0;
            amp *= 0.5;
        }
        let old = sum / norm;
        let new = eval(
            Spec {
                octaves: oct,
                ..Spec::default()
            },
            1.7,
            -0.4,
            base,
        );
        assert_eq!(
            old.to_bits(),
            new.to_bits(),
            "oitava {oct}: a folha tem de reproduzir o laco cravado AO BIT"
        );
    }
}

/// Um seno sem transcendental, só para o stand-in não ser linear (HR-5 vale para
/// o produto; aqui é fixture, mas manter a disciplina evita um oráculo de
/// plataforma).
trait SinLike {
    fn sin_like(self) -> f32;
}
impl SinLike for f32 {
    fn sin_like(self) -> f32 {
        let t = self - self.floor() - 0.5;
        4.0 * t * (1.0 - 2.0 * t.abs())
    }
}

/// **Os três tipos são três imagens, e a retificação é POR OITAVA.**
#[test]
fn the_three_types_rectify_per_octave_not_at_the_end() {
    fn alt(x: f32, _y: f32, o: u32) -> f32 {
        // Sinais opostos por oitava: o `|·|` por-oitava soma, o final cancelaria.
        if o.is_multiple_of(2) { x } else { -x }
    }
    let two = Spec {
        octaves: 2,
        lacunarity: 1.0,
        roughness: 1.0,
        ty: NoiseType::Fbm,
    };
    assert!(
        eval(two, 1.0, 0.0, alt).abs() < 1e-7,
        "com sinais opostos o fBm CANCELA"
    );
    let turb = Spec {
        ty: NoiseType::Turbulence,
        ..two
    };
    assert!(
        (eval(turb, 1.0, 0.0, alt) - 1.0).abs() < 1e-6,
        "e a turbulencia SOMA -- e a prova de que o `abs` e por oitava"
    );
    let ridge = Spec {
        ty: NoiseType::Ridged,
        ..two
    };
    assert!(
        eval(ridge, 1.0, 0.0, alt).abs() < 1e-6,
        "e o ridged de |n| = 1 e (1-1)^2 = 0"
    );
}

/// **A costura FECHA, e fecha C¹.** O fim do laço tem de dar o mesmo número que o
/// começo — e a derivada nas duas pontas tem de ser a mesma, senão a volta dá um
/// tranco.
#[test]
fn the_time_loop_closes_on_the_same_number_and_the_same_slope() {
    let l = 4.0f32;
    let (a0, _, w0) = loop_times(0.0, l);
    assert_eq!(
        (a0, w0),
        (0.0, 0.0),
        "no comeco o peso e zero, entao o valor e campo(a0)"
    );
    // Perto do fim o peso vai a 1, entao o valor e campo(b1) — e `b1` tem de ser
    // o instante do COMECO (`a0`), que e o que faz a costura fechar no MESMO
    // numero. ⚠️ A primeira versao deste gate comparava `b1` com `b0` (`−L`) e
    // reprovava sobre um produto certo: `b0` e onde a mistura COMECA a olhar, nao
    // onde ela termina.
    let (_, b1, w1) = loop_times(l - 1e-4, l);
    assert!(w1 > 0.9999, "o peso fecha em 1, deu {w1}");
    assert!(
        (b1 - a0).abs() < 1e-3,
        "a segunda amostra no fim e o comeco: {b1} contra {a0}"
    );
    // A periodicidade: t e t+L dao o MESMO par.
    for t in [0.3f32, 1.9, 3.7] {
        let (a, b, w) = loop_times(t, l);
        let (a2, b2, w2) = loop_times(t + l, l);
        assert!((a - a2).abs() < 1e-4 && (b - b2).abs() < 1e-4 && (w - w2).abs() < 1e-5);
    }
    // Sem laço: os dois instantes coincidem e o peso e zero — o consumidor pode
    // pular a segunda amostra, que e o que torna `loop_len = 0` byte-identico.
    assert_eq!(loop_times(7.25, 0.0), (7.25, 7.25, 0.0));
}

/// **A FAIXA NATURAL É A DA RETIFICAÇÃO** — bipolar quando a soma tem sinal,
/// unipolar quando ela é retificada por oitava.
///
/// ⚠️ **É o gate que fixa a assimetria que a armadilha explora.** Se as três
/// respondessem `[-1,1]`, o `Min/Max` dos nós que consomem esta folha entregaria
/// metade da faixa pedida em dois dos três tipos — sem erro, sem aviso, e sem
/// nenhum número do painel ter mudado.
#[test]
fn the_natural_range_follows_the_rectification() {
    assert_eq!(NoiseType::Fbm.natural_range(), (-1.0, 1.0));
    assert_eq!(NoiseType::Turbulence.natural_range(), (0.0, 1.0));
    assert_eq!(NoiseType::Ridged.natural_range(), (0.0, 1.0));
}

/// **A AFIM LEVA AS DUAS PONTAS ÀS DUAS PONTAS** — para qualquer faixa natural e
/// qualquer alvo. O oráculo é a definição, não um valor a olho.
#[test]
fn the_affine_maps_both_ends_onto_both_ends() {
    for natural in [(-1.0f32, 1.0f32), (0.0, 1.0), (-3.0, 7.0)] {
        for (min, max) in [(0.0f32, 1.0f32), (-5.0, 5.0), (2.0, 3.5), (10.0, -10.0)] {
            let (gain, off) = gain_offset_for_range(natural, min, max);
            let at = |v: f32| v * gain + off;
            assert!((at(natural.0) - min).abs() < 1e-5, "{natural:?} -> {min}");
            assert!((at(natural.1) - max).abs() < 1e-5, "{natural:?} -> {max}");
        }
    }
}

/// **PARA UM CAMPO BIPOLAR ELA É A CONTA QUE O ARTISTA JÁ FAZIA, e para um
/// unipolar é a que ele NÃO faria** — as duas metades da armadilha, num gate.
#[test]
fn it_agrees_with_the_artists_arithmetic_only_where_that_arithmetic_is_right() {
    let (min, max) = (2.0f32, 6.0f32);
    // Bipolar: a conta de cabeça — amplitude = (max−min)/2, centro = (min+max)/2.
    let (gain, off) = gain_offset_for_range(NoiseType::Fbm.natural_range(), min, max);
    assert!((gain - 2.0).abs() < 1e-6, "ganho bipolar: {gain}");
    assert!((off - 4.0).abs() < 1e-6, "centro bipolar: {off}");
    // Unipolar: a conta de cabeça daria 2 e 4 outra vez, e entregaria [2, 6]
    // deslocado — a resposta certa é ganho 4 e piso 2.
    let (gain_u, off_u) = gain_offset_for_range(NoiseType::Ridged.natural_range(), min, max);
    assert!((gain_u - 4.0).abs() < 1e-6, "ganho unipolar: {gain_u}");
    assert!((off_u - 2.0).abs() < 1e-6, "piso unipolar: {off_u}");
    // ⚠️ **O CONTROLE, e a medição que nomeia o custo da armadilha.** A conta do
    // artista aplicada a um campo unipolar acerta o TOPO por acidente (o topo
    // natural é `1` nos dois casos) e erra o PISO por metade da faixa pedida: o
    // campo passa a viver em `[4, 6]` quando o painel diz `[2, 6]`. É por isso que
    // ela é silenciosa — quem olha o pico vê o número certo.
    let wrong_floor = 0.0 * gain + off;
    assert!(
        (wrong_floor - min).abs() > (max - min) * 0.49,
        "a armadilha tem de existir na fixture: piso {wrong_floor} contra {min}"
    );
    assert!(
        (1.0 * gain + off - max).abs() < 1e-6,
        "e o TOPO acerta — e' isso que a torna invisivel"
    );
}

/// **UMA FAIXA NATURAL DEGENERADA NÃO DIVIDE POR ZERO** — ganho zero e o piso em
/// `min`, nunca `inf`.
#[test]
fn a_degenerate_natural_range_is_not_an_infinity() {
    let (gain, off) = gain_offset_for_range((3.0, 3.0), 1.0, 9.0);
    assert_eq!(gain, 0.0);
    assert_eq!(off, 1.0);
    assert!(gain.is_finite() && off.is_finite());
}

/// **`min > max` INVERTE, e sai da fórmula sem um `if`** — o *"Min>Max
/// auto-invertido"* que a Cavalry documenta como default inteligente.
#[test]
fn a_reversed_range_inverts_without_a_branch() {
    let (gain, off) = gain_offset_for_range((-1.0, 1.0), 5.0, 1.0);
    assert!(gain < 0.0, "o ganho tem de sair negativo: {gain}");
    let (floor, ceil) = (-1.0f32, 1.0f32); // as pontas da faixa NATURAL
    assert!(
        (floor * gain + off - 5.0).abs() < 1e-6,
        "o piso natural vai ao MAIOR"
    );
    assert!(
        (ceil * gain + off - 1.0).abs() < 1e-6,
        "e o topo natural ao MENOR"
    );
}
