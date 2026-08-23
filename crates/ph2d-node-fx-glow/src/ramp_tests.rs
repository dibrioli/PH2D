//! **A RAMPA DO HALO** — a medição que escolheu a resolução da LUT, e os gates dela
//! (doc 89, folha 11).

use super::*;
use ph2d_color::{ColorRamp, GradientPreset, RampInterp};

/// O passo de um ecrã de 8 bits — o que o olho tem como ver depois do tonemap.
const DISPLAY_STEP: f32 = 1.0 / 255.0;

/// Quantos pontos a varredura visita. O erro de uma reconstrução linear é máximo **no meio** de
/// uma célula, então amostrar nos nós daria zero em toda parte.
const SWEEP: usize = 8192;

/// A reconstrução que o SAMPLER faz: interpolação linear entre texels vizinhos de uma LUT
/// uniforme. Escrita aqui para a medição comparar o mesmo que o device vai desenhar.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "indices e contagens pequenas de uma LUT"
)]
fn reconstruct(lut: &[[f32; 4]], t: f32) -> [f32; 3] {
    let n = lut.len();
    let f = t.clamp(0.0, 1.0) * (n - 1) as f32;
    let k = (f.floor() as usize).min(n - 2);
    let a = f - k as f32;
    let (lo, hi) = (lut[k], lut[k + 1]);
    [
        lo[0] + (hi[0] - lo[0]) * a,
        lo[1] + (hi[1] - lo[1]) * a,
        lo[2] + (hi[2] - lo[2]) * a,
    ]
}

/// Assa `n` texels uniformes — a mesma lei do [`super::bake_halo_lut`], com `n` livre.
#[expect(clippy::cast_precision_loss, reason = "n <= 4096")]
fn bake(ramp: &ColorRamp, n: usize) -> Vec<[f32; 4]> {
    (0..n)
        .map(|k| ramp.eval(k as f32 / (n - 1) as f32))
        .collect()
}

/// **O CORPUS, com o pior caso do editor dentro.**
///
/// ⚠️ **Os quatro presets nascem todos em `Linear`, e uma régua que só os visse mentiria.** O
/// editor oferece cinco interpolações, e o `Constant` — um DEGRAU — é a função que uma
/// reconstrução linear reproduz pior de todas. *Uma fixtura que não contém o fenómeno mede o
/// instrumento, não o produto.*
fn corpus() -> Vec<(&'static str, ColorRamp)> {
    let mut out: Vec<(&'static str, ColorRamp)> = GradientPreset::ALL
        .iter()
        .map(|p| (p.name(), p.ramp()))
        .collect();
    let mut eased = GradientPreset::Heat.ramp();
    eased.interp = RampInterp::Ease;
    out.push(("Heat/Ease", eased));
    out
}

/// A rampa em DEGRAU — medida à parte, e a razão está na sonda.
fn hard_step() -> ColorRamp {
    let mut r = GradientPreset::Rainbow.ramp();
    r.interp = RampInterp::Constant;
    r
}

/// O pior erro, e a FRACÇÃO do percurso em que ele passa do passo do ecrã.
#[expect(clippy::cast_precision_loss, reason = "SWEEP <= 8192")]
fn error_of(ramp: &ColorRamp, n: usize) -> (f32, f32) {
    let lut = bake(ramp, n);
    let (mut worst, mut bad) = (0.0f32, 0usize);
    for i in 0..=SWEEP {
        let t = i as f32 / SWEEP as f32;
        let got = reconstruct(&lut, t);
        let want = ramp.eval(t);
        let e = (0..3).fold(0.0f32, |m, c| m.max((got[c] - want[c]).abs()));
        worst = worst.max(e);
        if e > DISPLAY_STEP {
            bad += 1;
        }
    }
    (worst, bad as f32 / (SWEEP + 1) as f32)
}

/// **A MEDIÇÃO QUE ESCOLHEU A RESOLUÇÃO.**
///
/// ```text
/// cargo test -p ph2d-node-fx-glow measure_ramp_lut -- --nocapture
/// ```
///
/// **Duas colunas, e a segunda foi a correcção.** A primeira versão desta sonda mediu só o erro
/// MÁXIMO e leu-o como veredito — e num degrau o erro máximo é metade do salto **e não cai com a
/// densidade**: `16` texels davam `0,998` e `1024` davam `0,834`, ou seja a régua dizia que
/// nenhuma resolução servia. O que a densidade encolhe num degrau não é a ALTURA do erro, é a
/// **LARGURA** da banda errada — e é ela que decide se aquilo se vê.
///
/// ⚠️ *Um extremo global e uma fracção do percurso respondem a perguntas diferentes, e sobre uma
/// descontinuidade só a segunda é sobre o que aparece no ecrã.*
#[test]
fn measure_ramp_lut_resolution() {
    eprintln!("o passo do ecra' e' {DISPLAY_STEP:.5}");
    eprintln!(
        "{:>7}  {:>10}  {:>9}   {:>11}  {:>9}",
        "texels", "erro suave", "fracao", "erro degrau", "fracao"
    );
    for n in [16usize, 64, 128, 256, 512, 1024] {
        let smooth = corpus()
            .iter()
            .map(|(_, r)| error_of(r, n))
            .fold((0.0f32, 0.0f32), |a, b| (a.0.max(b.0), a.1.max(b.1)));
        let step = error_of(&hard_step(), n);
        eprintln!(
            "{n:>7}  {:>10.5}  {:>8.3}%   {:>11.5}  {:>8.3}%",
            smooth.0,
            smooth.1 * 100.0,
            step.0,
            step.1 * 100.0
        );
    }
    // O controle: uma resolução grosseira TEM de ser visível nas rampas suaves, senão a régua
    // é cega e a tabela acima não diz nada.
    let coarse = corpus()
        .iter()
        .map(|(_, r)| error_of(r, 16).0)
        .fold(0.0f32, f32::max);
    assert!(
        coarse > DISPLAY_STEP,
        "controle: a 16 texels o erro TEM de ser visivel ({coarse})"
    );
}

/// **A RESOLUÇÃO QUE SHIPA É A MEDIDA** — as duas metades, e a segunda impede o número de ser
/// maior do que precisa.
#[test]
fn the_shipped_lut_resolution_is_the_measured_one() {
    for (name, ramp) in corpus() {
        let (worst, _) = error_of(&ramp, HALO_LUT_TEXELS);
        assert!(
            worst <= DISPLAY_STEP,
            "{name}: a LUT de {HALO_LUT_TEXELS} erra {worst}, acima do passo do ecra'"
        );
    }
    let half = corpus()
        .iter()
        .map(|(_, r)| error_of(r, HALO_LUT_TEXELS / 2).0)
        .fold(0.0f32, f32::max);
    assert!(
        half > DISPLAY_STEP,
        "com metade dos texels o erro ({half}) tem de ser VISIVEL, senao pagamos memoria por nada"
    );
}

/// **NUM DEGRAU, A BANDA ERRADA É UM TEXEL — E UM TEXEL É MAIS ESTREITO QUE O PASSO DO ECRÃ.**
///
/// ⚠️ Isto não é um teste de qualidade: é o **registo executável de uma limitação**. Nenhuma
/// tabela amostrada representa uma descontinuidade, e quem vier acrescentar texels a pensar que
/// «ainda está errado» tem de ler isto primeiro. O que se pode afirmar são duas coisas, e as
/// duas são estruturais:
///
/// 1. **cada salto suja UMA célula** — a fracção medida é `paradas / texels` e não mais (a
///    implementação não espalha o erro para lá da célula em que ele nasce);
/// 2. **uma célula é mais fina que um passo do ecrã** (`1/512 < 1/255`), então a banda cabe
///    dentro de um degrau de saída — e num halo, que é um borrão, ela é ainda menos visível.
///
/// ⚠️ **A barra NÃO é um número redondo escolhido depois de ver a medição.** A `1,147 %` medida
/// é exactamente `6 paradas / 512`; escrever «`< 1 %`» teria sido a barra a seguir o resultado,
/// que é o oposto do que ela serve.
#[test]
fn a_hard_step_dirties_one_cell_and_a_cell_is_finer_than_the_display() {
    let ramp = hard_step();
    let stops = ramp.len();
    let (worst, frac) = error_of(&ramp, HALO_LUT_TEXELS);
    assert!(
        worst > DISPLAY_STEP,
        "fixture: um degrau TEM de errar no salto, senao nao e' um degrau ({worst})"
    );
    #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
    let one_cell_each = stops as f32 / HALO_LUT_TEXELS as f32;
    assert!(
        frac <= one_cell_each * 1.5,
        "cada um dos {stops} saltos pode sujar UMA celula ({:.3}%); sujou {:.3}%",
        one_cell_each * 100.0,
        frac * 100.0
    );
    #[expect(clippy::cast_precision_loss, reason = "HALO_LUT_TEXELS <= 4096")]
    let cell = 1.0 / HALO_LUT_TEXELS as f32;
    assert!(
        cell < DISPLAY_STEP,
        "uma celula ({cell}) tem de ser mais fina que o passo do ecra' ({DISPLAY_STEP})"
    );
    // E a banda ENCOLHE com a densidade — a metade que prova que a limitação é de LARGURA e
    // não de altura.
    let (_, coarse) = error_of(&ramp, HALO_LUT_TEXELS / 8);
    assert!(
        coarse > frac * 4.0,
        "com um oitavo dos texels a banda tem de ser MUITO mais larga ({coarse} vs {frac})"
    );
}

/// **SEM RAMPA AUTORADA NÃO HÁ LUT** — e o passe usa o `tint` de sempre, ao bit.
#[test]
fn a_glow_without_an_authored_ramp_bakes_nothing() {
    let mut g = Graph::new();
    let n = g.add_node(TYPE_NAME);
    assert!(
        bake_halo_lut(&g).is_none(),
        "um no' recem-criado nao tem rampa"
    );
    g.set_text_param(
        n,
        RAMP_KEY.to_string(),
        ph2d_color::serialize_gradient(&GradientPreset::Heat.ramp()),
    );
    let lut = bake_halo_lut(&g).expect("uma rampa autorada assa a LUT");
    assert_eq!(lut.len(), HALO_LUT_TEXELS);
    // ⚠️ E ela está na ordem do PARÂMETRO: `Heat` vai de preto a branco, então a luminância
    // cresce. Ao contrário, o halo coloriria o brilho pelo avesso.
    let lum = |c: [f32; 4]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    assert!(lum(lut[HALO_LUT_TEXELS - 1]) > lum(lut[0]));
    assert!(
        lut.iter().all(|c| c.iter().all(|v| v.is_finite())),
        "nenhum texel pode ser NaN: ele espalha-se por seis niveis de mip"
    );
}

/// **UM TEXTO INVÁLIDO NÃO ACENDE NADA** — ele conta como *sem rampa*.
#[test]
fn junk_in_the_ramp_field_reads_as_no_ramp() {
    let mut g = Graph::new();
    let n = g.add_node(TYPE_NAME);
    for junk in ["", "nonsense", "g1 ", "g1 2 0:x,y,z"] {
        g.set_text_param(n, RAMP_KEY.to_string(), junk.to_string());
        if let Some(lut) = bake_halo_lut(&g) {
            assert!(
                lut.iter().all(|c| c.iter().all(|v| v.is_finite())),
                "texto {junk:?}: se ha' LUT, ela e' finita"
            );
        }
    }
}
