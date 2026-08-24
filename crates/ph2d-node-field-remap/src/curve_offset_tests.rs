//! Gates do **`curve_offset`** — o deslocamento da forma autorada ao longo de si
//! mesma.
//!
//! Filho do `tests.rs` (`#[path]`), de forma que `super` é o arnês deste arquivo:
//! o `chain`/`linear`/`falloff_of` e o `Ops` moram lá. Separado pelo tecto de LOC
//! (HR-18).

use super::*;

/// Uma curva ASSIMÉTRICA — a rampa `t`, cujos extremos NÃO concordam (`0 ≠ 1`).
///
/// ⚠️ Uma curva simétrica (ou a identidade) tornaria os gates abaixo vácuos: deslocar
/// uma forma que se repete não muda nada, e um gate que não vê diferença nenhuma
/// passaria com o `curve_offset` inteiramente desligado.
fn ramp() -> Curve {
    Curve::identity()
}

/// **O ZERO É A IDENTIDADE, E O TOPO DO INTERVALO É ONDE ISSO SE MEDE.**
///
/// ⚠️ **`t = 1.0` não é um caso de canto: é o que TODA peça a máscara cheia entrega.**
/// O wrap natural (`x − floor(x)`) leva `1.0` a `0.0`, então sem a guarda ligar o nó
/// sem tocar no knob trocaria `curve(1)` por `curve(0)` em metade da cena. Este gate é
/// a guarda, e ele testa exactamente o ponto onde ela morde.
#[test]
fn a_zero_offset_is_the_identity_including_at_the_very_top() {
    let c = ramp();
    for t in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        assert_eq!(shifted(t, 0.0), t, "sem deslocamento, `t` sai intacto");
        assert_eq!(
            contour(4, t, 0.0, 4.0, Some(&c), 0.0),
            c.eval(t),
            "e a contour lê a curva no mesmo sítio de sempre"
        );
    }
    // O CONTROLE: com um deslocamento, o topo JÁ NÃO é o topo — é o que prova que a
    // igualdade acima é da guarda e não de a função ser constante.
    assert_ne!(
        shifted(1.0, 0.25),
        1.0,
        "com deslocamento o wrap age, e é por isso que o zero precisa da guarda"
    );
}

/// **O DESLOCAMENTO ANDA COM A FORMA, E DÁ A VOLTA.**
#[test]
fn the_offset_slides_the_curve_and_wraps_around() {
    // 0.25 adiante: o que estava em 0.5 passa a ser lido em 0.25.
    assert!((shifted(0.25, 0.25) - 0.5).abs() < 1e-6);
    // E o que passa do fim reentra pelo começo.
    assert!((shifted(0.9, 0.25) - 0.15).abs() < 1e-6);
}

/// **UMA VOLTA INTEIRA É A IDENTIDADE (a menos do épsilon do float), e é isso que faz
/// a faixa `−1..1` do slider ser o percurso COMPLETO** — nada além dela é alcançável.
///
/// ⚠️ O `1e-6` aqui não é preguiça: `t + 1.0` perde bits baixos de `t`, então a volta
/// não fecha ao bit. O que o gate afirma é a periodicidade, não a exactidão.
#[test]
fn one_whole_turn_lands_back_where_it_started() {
    for t in [0.1_f32, 0.37, 0.62, 0.99] {
        assert!(
            (shifted(t, 1.0) - t).abs() < 1e-6,
            "uma volta em {t} devia voltar a {t}, deu {}",
            shifted(t, 1.0)
        );
        // …e meia volta duas vezes é uma volta.
        let half = shifted(t, 0.5);
        assert!((shifted(half, 0.5) - t).abs() < 1e-6);
    }
}

/// **UM DESLOCAMENTO NEGATIVO ANDA PARA TRÁS, e não sai do intervalo.**
///
/// ⚠️ `x − floor(x)` é o `rem_euclid`, não o `%`: em Rust `(-0.25) % 1.0` é `-0.25`,
/// que como argumento de uma curva é fora do domínio.
#[test]
fn a_negative_offset_walks_backwards_and_stays_inside() {
    for t in [0.0_f32, 0.1, 0.5, 1.0] {
        let u = shifted(t, -0.25);
        assert!((0.0..=1.0).contains(&u), "t={t} saiu do intervalo: {u}");
    }
    assert!(
        (shifted(0.1, -0.25) - 0.85).abs() < 1e-6,
        "reentra pelo fim"
    );
}

/// **SÓ O MODO `Curve` OUVE O DESLOCAMENTO** — nos outros quatro ele é inerte, porque
/// ali não há tabela a deslizar: são fórmulas.
///
/// FALSIFICADO se o offset entrasse antes do `match` — as quatro formas mudariam de
/// sítio, e o knob que a folha pediu para a curva estaria a torcer o Step.
#[test]
fn only_the_curve_contour_listens_to_the_offset() {
    for mode in [0, 1, 2, 3] {
        for t in [0.0_f32, 0.3, 0.7, 1.0] {
            assert_eq!(
                contour(mode, t, 0.7, 5.0, None, 0.0),
                contour(mode, t, 0.7, 5.0, None, 0.37),
                "o modo {mode} não pode mexer-se com o offset"
            );
        }
    }
    // O CONTROLE: no modo Curve ele mexe-se mesmo.
    let c = ramp();
    assert_ne!(
        contour(4, 0.3, 0.0, 4.0, Some(&c), 0.0),
        contour(4, 0.3, 0.0, 4.0, Some(&c), 0.37),
        "e no modo Curve o offset TEM de mudar a resposta"
    );
}

/// **O KERNEL CARREGA O PARAM E A LEI, E A GUARDA DO ZERO ESTÁ NO DEVICE TAMBÉM.**
///
/// ⚠️ Sem a guarda no WGSL, o device e a CPU discordariam **em toda peça a máscara
/// cheia** com o knob em zero — a divergência mais cara possível, porque acontece no
/// estado default e ninguém a procuraria ali.
#[test]
fn the_kernel_carries_the_offset_and_the_same_zero_guard() {
    assert!(
        GPU_KERNEL.params.contains(&"curve_offset"),
        "o uniforme tem de carregar o deslocamento: {:?}",
        GPU_KERNEL.params
    );
    assert!(
        GPU_KERNEL.wgsl.contains("params.curve_offset"),
        "e o corpo tem de o passar à contour"
    );
    assert!(
        GPU_KERNEL
            .wgsl_lib
            .contains("if (offset == 0.0) { return t; }"),
        "a guarda do zero tem de existir no device como existe na CPU"
    );
    assert!(
        GPU_KERNEL
            .wgsl_lib
            .contains("rm_curve_sample(rm_shifted(t, offset))"),
        "e o deslocamento tem de entrar SÓ no braço da curva"
    );
}

/// **SEM CURVA AUTORADA, O DESLOCAMENTO CONTINUA A AGIR — e isto é o bug do smoke.**
///
/// ⚠️ **O estado em que o nó NASCE é o estado sem curva.** A primeira versão escrevia
/// `curve.map_or(t, |c| c.eval(shifted(t, off)))`: com `None` o `map_or` devolvia `t` e
/// o `shifted` nunca corria, então o knob estava morto exactamente onde o artista o
/// encontra primeiro. Uma curva ausente É a identidade, e a identidade deslocada é um
/// **dente de serra** — uma resposta, não um no-op.
///
/// ⚠️ **O gate anterior não viu isto porque o FIXTURE dele tinha curva.** `Some(&ramp)`
/// e `None` tomam ramos diferentes do `map_or`, e eu só exercitava um. *Um fixture só
/// prova o que contém* — a terceira vez que esta linha paga essa lei.
#[test]
fn the_offset_still_acts_when_no_curve_is_authored() {
    // `t = 0.3` deslocado 0,35 ⇒ 0,65. Sem a cura, saía 0,3.
    let got = contour(4, 0.3, 0.0, 4.0, None, 0.35);
    assert!(
        (got - 0.65).abs() < 1e-6,
        "sem curva, o offset tem de agir: {got}"
    );
    // …e a volta continua a existir: 0,8 + 0,35 reentra em 0,15.
    let wrapped = contour(4, 0.8, 0.0, 4.0, None, 0.35);
    assert!((wrapped - 0.15).abs() < 1e-6, "e ele dá a volta: {wrapped}");
    // O CONTROLE: sem deslocamento, sem curva, é o passthrough EXACTO de sempre.
    for t in [0.0_f32, 0.3, 0.8, 1.0] {
        assert_eq!(contour(4, t, 0.0, 4.0, None, 0.0), t);
    }
}

/// **A CPU E O DEVICE CONCORDAM NO CASO SEM CURVA — e antes NÃO concordavam.**
///
/// O `fill_curve_lut` de uma string ausente escreve a **identidade** (`out[k] = t`), e o
/// WGSL amostra-a em `rm_shifted(t, offset)`. Ou seja: o device sempre deslocou. A CPU
/// é que saía antes. Este gate compara os dois pela LUT que o device de facto recebe.
#[test]
fn the_device_and_the_cpu_agree_with_no_curve_authored() {
    let mut lut = vec![0.0_f32; LUT_RESOLUTION as usize];
    fill_curve_lut("", &mut lut);
    let sample = |t: f32| {
        #[expect(clippy::cast_precision_loss, reason = "resolução pequena")]
        let last = (lut.len() - 1) as f32;
        let x = t.clamp(0.0, 1.0) * last;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "índice"
        )]
        let i0 = x.floor() as usize;
        let i1 = (i0 + 1).min(lut.len() - 1);
        #[expect(clippy::cast_precision_loss, reason = "índice pequeno")]
        let f = x - i0 as f32;
        lut[i0] * (1.0 - f) + lut[i1] * f
    };
    for off in [0.0_f32, 0.35, -0.2] {
        for t in [0.0_f32, 0.25, 0.7, 1.0] {
            let cpu = contour(4, t, 0.0, 4.0, None, off);
            let gpu = sample(shifted(t, off));
            assert!(
                (cpu - gpu).abs() < 1e-4,
                "off={off} t={t}: CPU {cpu} contra device {gpu}"
            );
        }
    }
}
