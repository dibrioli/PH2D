//! ⭐⭐⭐ **A LEI DO LÓBULO, contra o ORÁCULO** — a forma fechada do [`super::lobe_shrink`] medida
//! contra a quadratura que a define.
//!
//! ⚠️ **Este ficheiro VIAJOU com a função** (de `ph2d-app-field3d/src/render_light_lobe_tests.rs`), e
//! o corte é o que a HOWTO manda: o que fica lá é o que exercita o **CÉU** daquele módulo (a lei de
//! ler a rampa na direcção média, que é de lá); o que veio é o que exercita o **COEFICIENTE**, que é
//! de cá.

/// O factor de encolhimento do lóbulo, por quadratura — o ORÁCULO da forma fechada.
///
/// # A distribuição
///
/// É a do *split-sum* (Karis), que é a que o `mx_environment_prefilter` assume: `N = V = R`, amostras
/// `h` do GGX, `L = 2(N·h)h − N`, peso `N·L`, e as amostras com `N·L ≤ 0` **descartadas** — é isso
/// que faz `c` deixar de ser trivial quando o lóbulo passa do hemisfério.
///
/// ⇒ `c(α) = Σ (N·L)² / Σ (N·L)`.
fn lobe_shrink_by_quadrature(alpha: f32) -> f32 {
    let a2 = f64::from(alpha).powi(2);
    let (mut num, mut den) = (0.0f64, 0.0f64);
    const N: usize = 1 << 16;
    for i in 0..N {
        // ⚠️ Quadratura do ponto médio sobre `ξ`, não um sorteio: uma sonda que é o ORÁCULO de um
        // gate não pode ter ruído de Monte Carlo maior do que a barra que ela vai justificar.
        let xi = (i as f64 + 0.5) / N as f64;
        // Amostragem de importância do GGX: `cos²θ_h = (1 − ξ) / (1 + (α² − 1)ξ)`.
        let cos2 = (1.0 - xi) / (1.0 + (a2 - 1.0) * xi);
        let ndl = 2.0 * cos2 - 1.0;
        if ndl <= 0.0 {
            continue;
        }
        num += ndl * ndl;
        den += ndl;
    }
    if den <= 0.0 {
        return 1.0;
    }
    (num / den) as f32
}

/// ⭐⭐⭐ **A FORMA FECHADA CONCORDA COM A QUADRATURA**, e os dois controlos de fronteira.
///
/// ⚠️ **A vizinhança de `α = 1` é varrida DE PROPÓSITO**: é ali que o numerador e o denominador vão
/// os dois a zero como `k³` e o cancelamento come a precisão. Sem esses pontos, o gate ficaria verde
/// sobre a única região onde a forma fechada não se pode usar crua.
///
/// **Mutação que deve sangrar:** apagar o ramo do limite (`|k| < 1e-3 → 2/3`).
#[test]
fn a_forma_fechada_do_lobulo_concorda_com_a_quadratura() {
    let mut pior = 0.0f32;
    let mut onde = 0.0f32;
    // ⚠️ O varrimento inclui `0`, `1` e a vizinhança fina de `1` — os três casos de fronteira.
    let mut alphas: Vec<f32> = (0..=100).map(|i| i as f32 / 100.0).collect();
    alphas.extend([0.999, 0.9995, 0.9999, 1.0, 0.001, 0.0005]);
    for alpha in alphas {
        let nosso = super::lobe_shrink(alpha);
        let oraculo = lobe_shrink_by_quadrature(alpha);
        let d = (nosso - oraculo).abs();
        if d > pior {
            pior = d;
            onde = alpha;
        }
    }
    assert!(
        pior < 2.0e-4,
        "a forma fechada afasta-se da quadratura em {pior:.2e} (pior em α = {onde}) — a barra é 2e-4"
    );
    // ⭐ **Os dois controlos que não são coincidência**, afirmados: o lóbulo que colapsa na
    // espelhada, e o hemisfério cosseno.
    assert!(
        (super::lobe_shrink(0.0) - 1.0).abs() < 1.0e-6,
        "α = 0 tem de devolver a própria direcção espelhada"
    );
    assert!(
        (super::lobe_shrink(1.0) - 2.0 / 3.0).abs() < 1.0e-6,
        "α = 1 é o hemisfério cosseno, e o coeficiente dele é 2/3"
    );
    // ⛔ **E é MONÓTONO**: um lóbulo mais largo nunca encolhe menos. Sem isto, uma forma fechada com
    // um sinal trocado num ramo passaria no desvio médio e daria um céu que clareia com a rugosidade.
    let mut anterior = 1.0f32;
    for i in 0..=100 {
        let c = super::lobe_shrink(i as f32 / 100.0);
        assert!(
            c <= anterior + 1.0e-6,
            "o encolhimento subiu em α = {} ({anterior} → {c})",
            i as f32 / 100.0
        );
        anterior = c;
    }
}

/// ⭐⭐ **O `EnvLobe::of` É a composição que os dois consumidores escreviam à mão** — e este gate é a
/// razão de ele existir.
///
/// ⚠️ A régua não é «devolve dois números»: é que eles são **exactamente** `lobe_shrink` dos `alphas`
/// que a própria crate corta. Sem esta igualdade, a porta seria uma terceira redacção.
///
/// **Mutação que deve sangrar:** trocar `main` por `coat` na porta.
#[test]
fn a_porta_do_lobulo_e_o_que_os_consumidores_escreviam() {
    use crate::wgsl::{EnvLobe, alphas};

    // ⚠️ Dois materiais com verniz de rugosidade DIFERENTE da principal — senão `main == coat` e uma
    // troca dos dois campos ficaria invisível.
    for (rugosidade, verniz) in [(0.1_f32, 0.8_f32), (0.9, 0.2), (0.3, 0.3)] {
        let m = crate::OpenPbr {
            specular_roughness: rugosidade,
            coat_weight: 1.0,
            coat_roughness: verniz,
            ..Default::default()
        };
        let s = m.prepare();
        let (a_main, a_coat) = alphas(&s);
        let lobe = EnvLobe::of(&s);
        assert_eq!(
            lobe.main,
            super::lobe_shrink(a_main),
            "o `main` da porta não é o encolhimento do α principal (rug {rugosidade})"
        );
        assert_eq!(
            lobe.coat,
            super::lobe_shrink(a_coat),
            "o `coat` da porta não é o encolhimento do α do verniz (verniz {verniz})"
        );
    }

    // ⭐ **O CONTROLO**: com rugosidades diferentes os dois campos TÊM de ser diferentes — senão as
    // igualdades acima passariam sobre uma porta que devolvesse o mesmo número duas vezes.
    //
    // ⚠️⚠️ **E o SENTIDO da fixtura é medido, não escolhido:** a 1.ª redacção pôs a principal LISA
    // (`0,1`) sob um verniz RUGOSO (`0,9`) e os dois campos leram `0,667` e `0,692` — porque o
    // verniz **ALARGA** o `α` principal (é o que o `main_alpha` diz de si mesmo) e ali ele satura em
    // `1`. *Uma fixtura que separa dois números tem de os separar no sentido em que a lei os mexe.*
    let s = crate::OpenPbr {
        specular_roughness: 0.9,
        coat_weight: 1.0,
        coat_roughness: 0.05,
        ..Default::default()
    }
    .prepare();
    let lobe = EnvLobe::of(&s);
    assert!(
        (lobe.main - lobe.coat).abs() > 0.05,
        "controlo: a fixtura tem de separar os dois α ({lobe:?})"
    );
}
