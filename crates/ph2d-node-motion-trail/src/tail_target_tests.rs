//! **A LEI DA PONTA DA CAUDA** — os gates do report de 2026-08-08
//! (*"sliders mal balanceados; Saturação 0.9 já fica quase todo dessaturado"*).
//!
//! Os cinco knobs de decaimento são ALVOS na ponta, não taxas por tick. Estes gates
//! afirmam as três propriedades que isso compra, e cada um nasceu VERMELHO sobre a lei
//! anterior:
//!
//! 1. o slider é **LINEAR no que se vê** (o alvo autorado É o que a ponta mede);
//! 2. o número **NÃO SE MOVE** quando o Length ou o Spacing mudam;
//! 3. e o ponto neutro atravessa a derivação **ao bit**.
//!
//! ⚠️ Arquivo IRMÃO do `lib_tests.rs` por ASSUNTO — aquele mede a mecânica do anel
//! (cadência, janela, promoção), este mede a lei do decaimento. Segue FILHO por `#[path]`,
//! então `use super::*` alcança `step`, `Decay`, `per_tick` e `rate_for`, todos privados.

use super::*;

/// Um ponto que anda, em cor saturada e com as duas colunas que a cauda desbota.
fn dot(x: f32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[x, 0.0]]))
        .with("tint", Column::Vec4(vec![[0.9, 0.15, 0.05, 1.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]]))
}

/// A idade NOMINAL do eco mais velho — a que a regra de sobrevivência permite.
fn nominal_span(length: f32, spacing: f32) -> u32 {
    ((generations(length, 1) - 1) * spacing_of(spacing)) as u32
}

/// Roda a cauda até ela encher **e até a fase em que o eco mais velho alcançou a idade
/// nominal**, devolvendo a saída ali.
///
/// ⚠️ Com `spacing > 1` a idade do mais velho **CICLA**: a cadência de promoção (uma
/// cabeça a cada `s` ticks) e a de descarte (uma linha por tick) não são travadas uma na
/// outra. Medido no anel: 13↔14 a `sp 2`, 17..20 a `sp 4`, 49..56 a `sp 8` — amplitude
/// `s − 1`, com a CONTAGEM estável. Isso é do anel e **precede esta wave** (a lei antiga
/// oscilava junto, por `rate^(s−1)`); o alvo é o que a ponta alcança no TOPO do ciclo, e é
/// essa fase que este helper procura.
fn run(length: f32, spacing: f32, decay: Decay) -> Stream {
    let want = nominal_span(length, spacing);
    let mut state = Stream::new(0);
    let mut settled = None;
    // Encher + uma volta inteira do ciclo, com folga.
    for t in 0..(want * 3 + 64) {
        state = step(&dot(t as f32), &state, length, decay, spacing);
        if t >= want * 2 {
            let oldest = ages(&state, AGE).iter().copied().fold(0.0f32, f32::max) as u32;
            if oldest == want {
                settled = Some(state.clone());
            }
        }
    }
    settled.unwrap_or_else(|| {
        panic!("len {length} sp {spacing}: o eco mais velho nunca alcancou a idade {want}")
    })
}

const LUMA: [f32; 3] = [0.213, 0.715, 0.072];

/// A grandeza que o `Tail Saturation` NOMEIA: quão longe da luma os canais estão.
fn spread(c: [f32; 4]) -> f32 {
    let l = LUMA[0] * c[0] + LUMA[1] * c[1] + LUMA[2] * c[2];
    (c[0] - l).abs() + (c[1] - l).abs() + (c[2] - l).abs()
}

/// `(alfa, tamanho, saturação)` na ponta, como FRAÇÃO da cabeça viva — as três grandezas
/// que os knobs multiplicativos nomeiam.
fn tail_over_head(s: &Stream) -> [f32; 3] {
    let t = match s.get("tint") {
        Some(Column::Vec4(v)) => v.clone(),
        _ => panic!("tint"),
    };
    let z = match s.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("size"),
    };
    let n = t.len() - 1;
    [
        t[0][3] / t[n][3],
        z[0][0] / z[n][0],
        spread(t[0]) / spread(t[n]),
    ]
}

/// As sete configurações que a sonda mediu — o default do nó, a esteira do smoke, o teto
/// do Length e um espaçamento largo.
const CONFIGS: [(f32, f32); 7] = [
    (2.0, 1.0),
    (4.0, 1.0),
    (8.0, 1.0),
    (16.0, 1.0),
    (8.0, 2.0),
    (6.0, 4.0),
    (8.0, 8.0),
];

/// **O SLIDER É LINEAR NO QUE SE VÊ** — o alvo autorado É o que a ponta mede.
///
/// ⚠️ Nasceu VERMELHO. Sob a lei anterior o knob ERA a taxa por tick, então `0.5` produzia
/// `0.5^span` na ponta: 0.25 no `length 3`, 0.0078 no default do nó, 0.0000 na esteira do
/// smoke. O oráculo aqui é o próprio número que o artista digitou — é isso que "linear"
/// significa, e é o que um slider tem de entregar.
#[test]
fn the_tail_lands_exactly_on_the_authored_target() {
    for target in [0.9f32, 0.5, 0.25, 1.0, 1.5] {
        for (len, sp) in CONFIGS {
            let d = Decay {
                alpha_max: 1.0,
                fade: target,
                shrink: target,
                saturation: target,
                ..Decay::NEUTRAL
            };
            let got = tail_over_head(&run(len, sp, d));
            for (name, v) in ["alfa", "tamanho", "saturacao"].iter().zip(got) {
                assert!(
                    (v - target).abs() <= target * 1e-3 + 1e-5,
                    "len {len} sp {sp}: o `{name}` autorado {target} mediu {v} na ponta"
                );
            }
        }
    }
}

/// **O NÚMERO NÃO SE MOVE QUANDO OUTRO KNOB SE MOVE** — o gate do report do Enio.
///
/// ⚠️ Nasceu VERMELHO, e este é o defeito CARO. Sob a lei anterior o mesmo `0.90`
/// autorado produzia, na ponta, **0.90 · 0.73 · 0.48 · 0.21 · 0.25 · 0.17 · 0.07** ao
/// longo destas sete configurações — sete desenhos para um número só. O valor certo do
/// knob era função de OUTROS DOIS, que é a definição de um bug de design.
#[test]
fn the_authored_target_does_not_move_when_the_length_or_the_spacing_moves() {
    let d = Decay {
        alpha_max: 1.0,
        saturation: 0.9,
        ..Decay::NEUTRAL
    };
    let measured: Vec<f32> = CONFIGS
        .iter()
        .map(|&(len, sp)| tail_over_head(&run(len, sp, d))[2])
        .collect();
    let (lo, hi) = measured
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(
        hi - lo < 2e-3,
        "o mesmo 0.9 tinha de dar a mesma ponta em toda configuracao, e deu {measured:?}"
    );
    assert!(
        (lo - 0.9).abs() < 2e-3,
        "e ela e o 0.9 autorado: {measured:?}"
    );
}

/// **OS ÂNGULOS SÃO TOTAIS PERCORRIDOS PELA CAUDA**, não passos por tick.
///
/// ⚠️ Nasceu VERMELHO: `spin 9` sobre a esteira do smoke percorria **180°** e sobre
/// `spacing 8` percorria **504°** — o mesmo número, uma volta e meia de diferença.
#[test]
fn the_angles_are_totals_across_the_tail() {
    for (len, sp) in CONFIGS {
        let out = run(
            len,
            sp,
            Decay {
                alpha_max: 1.0,
                spin: 90.0,
                ..Decay::NEUTRAL
            },
        );
        match out.get("rot") {
            Some(Column::Scalar(v)) => {
                let total = v[0] - v[v.len() - 1];
                assert!(
                    (total - 90.0).abs() < 1e-2,
                    "len {len} sp {sp}: a cauda tinha de percorrer 90 deg e percorreu {total}"
                );
            }
            _ => panic!("len {len} sp {sp}: sem coluna `rot` a cauda nao girou"),
        }
    }
}

/// **O PONTO NEUTRO ATRAVESSA A DERIVAÇÃO AO BIT.**
///
/// É o que mantém todo grafo já autorado sem knobs de decaimento byte-idêntico: `powf(1,
/// y)` é `1` EXATO em IEEE-754 para todo `y`, então a raiz `span`-ésima da identidade é a
/// identidade — não "1.0 a menos de um ulp".
#[test]
fn the_neutral_survives_the_derivation_to_the_bit() {
    for span in [0u32, 1, 2, 7, 20, 31, 56, 495] {
        let n = Decay::NEUTRAL.per_tick(span);
        assert_eq!(n.fade, 1.0, "span {span}");
        assert_eq!(n.shrink, 1.0, "span {span}");
        assert_eq!(n.saturation, 1.0, "span {span}");
        assert_eq!(n.hue_shift, 0.0, "span {span}");
        assert_eq!(n.spin, 0.0, "span {span}");
    }
}

/// **UM ALVO DE ZERO É UMA RAMPA, NÃO UM PENHASCO.**
///
/// ⚠️ `0^(1/span)` é ZERO, então sem o piso o primeiro eco já colapsaria e a cauda inteira
/// seria invisível — o slider morreria justamente na ponta que o artista mais usa
/// (*"fade to nothing"*). O piso é UM NÍVEL de 8 bits, o número do renderer.
#[test]
fn a_target_of_zero_is_a_ramp_not_a_cliff() {
    let out = run(
        8.0,
        1.0,
        Decay {
            alpha_max: 1.0,
            fade: 0.0,
            ..Decay::NEUTRAL
        },
    );
    let a = match out.get("tint") {
        Some(Column::Vec4(v)) => v.iter().map(|c| c[3]).collect::<Vec<_>>(),
        _ => panic!("tint"),
    };
    // Um penhasco daria `[0, 0, …, 0, 1]`: estritamente crescente é o que o separa.
    assert!(
        a.windows(2).all(|w| w[1] > w[0]),
        "a cauda tinha de ser uma rampa e mediu {a:?}"
    );
    assert!(
        a[0] <= TARGET_FLOOR + 1e-6,
        "e ela chega ao invisivel na ponta: {a:?}"
    );
    assert!(
        a[a.len() - 2] > 0.05,
        "sem colapsar o eco vizinho da cabeca: {a:?}"
    );
}

/// **OS DEFAULTS REPRODUZEM O RASTRO QUE JÁ SHIPAVA**, e não foram escolhidos.
///
/// No default do nó (`length 8`, `spacing 1`, span 7) as taxas derivadas dos alvos
/// `0.10`/`0.65` são as MESMAS `0.72`/`0.94` que o manifesto trazia como taxas — o rastro
/// no default não se move um pixel, e o que muda é que ele CONTINUA o mesmo quando o
/// Length e o Spacing andam.
#[test]
fn the_defaults_reproduce_the_shipped_rates_at_the_default_length() {
    let d = Decay::new(0.10, 0.65).per_tick(7);
    assert!((d.fade - 0.72).abs() < 2e-3, "fade derivado: {}", d.fade);
    assert!(
        (d.shrink - 0.94).abs() < 2e-3,
        "shrink derivado: {}",
        d.shrink
    );

    // E os defaults do MANIFESTO são esses alvos — um gate que lesse `Decay::new` e não o
    // manifesto ficaria verde com o produto shipando outro número.
    let of = |n: &str| {
        MANIFEST
            .params
            .iter()
            .find(|p| p.name == n)
            .expect("param")
            .default
    };
    assert_eq!(of("fade"), 0.10);
    assert_eq!(of("shrink"), 0.65);
}

/// **A IDADE DO ECO MAIS VELHO CICLA COM O ESPAÇAMENTO** — e isso é do ANEL, não da lei
/// do alvo.
///
/// ⚠️ Pinado aqui para ninguém ler o tremor da ponta como regressão desta wave: a
/// promoção acontece uma vez a cada `s` ticks e o descarte uma vez por tick, então a
/// idade do mais velho varre uma faixa de amplitude `s − 1` enquanto a CONTAGEM fica
/// parada. A lei antiga oscilava junto (por `rate^(s−1)`, um fator de 5× no `fade 0.8`
/// com `sp 8`) — só que lá a cauda inteira já era invisível e ninguém a via tremer.
///
/// Curar isto é a outra metade — o decaimento como função da IDADE, que exige guardar o
/// valor de nascimento por linha (~28 B/linha) e é o item da CURVA de cauda do doc 88.
#[test]
fn the_oldest_echo_ages_in_a_cycle_when_the_spacing_is_wide() {
    for (len, sp) in [(8.0f32, 1.0f32), (8.0, 2.0), (6.0, 4.0), (8.0, 8.0)] {
        let want = nominal_span(len, sp);
        let mut state = Stream::new(0);
        let (mut lo, mut hi, mut counts) = (u32::MAX, 0u32, Vec::new());
        for t in 0..(want * 3 + 64) {
            state = step(&dot(t as f32), &state, len, Decay::NEUTRAL, sp);
            if t >= want * 2 {
                let a = ages(&state, AGE);
                let oldest = a.iter().copied().fold(0.0f32, f32::max) as u32;
                lo = lo.min(oldest);
                hi = hi.max(oldest);
                counts.push(a.len());
            }
        }
        assert_eq!(
            hi - lo,
            spacing_of(sp) as u32 - 1,
            "len {len} sp {sp}: a faixa da idade e `s-1`, e mediu {lo}..{hi}"
        );
        assert_eq!(
            hi, want,
            "len {len} sp {sp}: o topo do ciclo e a idade nominal"
        );
        assert!(
            counts.windows(2).all(|w| w[0] == w[1]),
            "len {len} sp {sp}: a CONTAGEM nao cicla — so a idade: {counts:?}"
        );
    }
}

/// **UM ALVO VINDO DE UM DOCUMENTO NÃO ENVENENA A CAUDA.**
///
/// `NaN`/infinito caem na identidade em vez de espalhar `NaN` por toda linha carregada, e
/// um negativo cai no piso — a cauda fica invisível, nunca com sinal trocado.
#[test]
fn a_junk_target_falls_back_instead_of_poisoning_the_tail() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            rate_for(bad, 1.0 / 7.0),
            1.0,
            "{bad} tinha de virar identidade"
        );
        assert_eq!(step_for(bad, 7), 0.0, "{bad} tinha de virar zero");
    }
    let r = rate_for(-5.0, 1.0 / 7.0);
    assert!(r > 0.0 && r < 1.0, "um alvo negativo cai no piso: {r}");
}
