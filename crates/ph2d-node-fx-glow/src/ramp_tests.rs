//! **A RAMPA DO HALO** — a medição que escolheu a resolução da LUT, e os gates dela
//! (doc 89, folha 11). A régua é a do degradê assado ([`ph2d_color::ramp_lut::regua`]): o erro no
//! espaço GRAVADO (sRGB de 8 bits), e a reconstrução é a do SAMPLER que lê a textura.

use super::*;
use ph2d_color::GradientPreset;
use ph2d_color::ramp_lut::regua::{DISPLAY_STEP, corpus, degrau, erro_no_ecra, pior};

/// Bits de fracção do peso da filtragem linear — o que as duas placas da estação dão
/// (`subTexelPrecisionBits = 8`, RTX 5060 Ti e RADV); o D3D11 exige 8, o Vulkan só 4.
const BITS_DO_PESO: i32 = 8;

/// O que o SAMPLER devolve no `fs_composite` (`bloom.wgsl`): a tabela guardada em `Rgba16Float`,
/// o endereço no centro do texel (`lut_u`: o índice contínuo é `t·(n−1)`, a lei de
/// [`ph2d_color::ramp_lut::bake`]) e o peso quantizado a [`BITS_DO_PESO`].
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "índices de uma tabela pequena"
)]
fn sampler(lut: &[[f32; 4]], t: f32) -> [f32; 4] {
    le_no_indice(lut, t.clamp(0.0, 1.0) * (lut.len() - 1) as f32)
}

/// ⛔ O endereço de ANTES (corrigido em 11/10): `textureSample` em `u = t` cai no índice contínuo
/// `t·n − ½` — meia célula ao lado da lei do `bake`, que põe a amostra `k` em `k/(n−1)`.
#[expect(clippy::cast_precision_loss, reason = "n <= 4096")]
fn sampler_antes(lut: &[[f32; 4]], t: f32) -> [f32; 4] {
    let n = lut.len() as f32;
    le_no_indice(lut, (t.clamp(0.0, 1.0) * n - 0.5).clamp(0.0, n - 1.0))
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "índices de uma tabela pequena"
)]
fn le_no_indice(lut: &[[f32; 4]], x: f32) -> [f32; 4] {
    let meio = |v: f32| ph2d_color::half_to_f32(ph2d_color::f32_to_half(v));
    let k = (x.floor() as usize).min(lut.len() - 2);
    let passos = 2f32.powi(BITS_DO_PESO);
    let a = ((x - k as f32) * passos).round() / passos;
    std::array::from_fn(|c| {
        let (lo, hi) = (meio(lut[k][c]), meio(lut[k + 1][c]));
        lo + (hi - lo) * a
    })
}

/// **A MEDIÇÃO QUE ESCOLHEU A RESOLUÇÃO** —
/// `bash scripts/cargo-test-narrow.sh ph2d-node-fx-glow measure_ramp_lut -- --nocapture`.
///
/// Três colunas por resolução: o sampler de hoje, o de ANTES (o endereço meia célula ao lado) e a
/// fracção do percurso que o degrau suja. ⚠️ Num degrau o erro MÁXIMO é metade do salto a
/// qualquer densidade; o que a densidade encolhe é a LARGURA da banda errada.
#[test]
fn measure_ramp_lut_resolution() {
    eprintln!("passo do ecra' {DISPLAY_STEP:.5} (sRGB)");
    eprintln!(
        "{:>7}  {:>10}  {:>12}  {:>13}",
        "texels", "sampler", "antes (½ cel)", "degrau fracao"
    );
    for n in [256usize, 512, 1024, 2048, 4096] {
        let (_, f) = erro_no_ecra(&degrau(), n, sampler);
        eprintln!(
            "{n:>7}  {:>10.5}  {:>12.5}  {:>12.3}%",
            pior(n, sampler),
            pior(n, sampler_antes),
            f * 100.0
        );
    }
    for (nome, r) in corpus() {
        let (w, _) = erro_no_ecra(&r, HALO_LUT_TEXELS, sampler);
        eprintln!("  @{HALO_LUT_TEXELS} {nome:<24} {w:.5}");
    }
    assert!(pior(16, sampler) > DISPLAY_STEP, "controlo: a régua vê 16 texels");
}

/// **A RESOLUÇÃO QUE SHIPA É A MEDIDA** — invisível no corpus inteiro, lida pelo sampler, e
/// metade dela visível (senão pagávamos memória por nada).
#[test]
fn the_shipped_lut_resolution_is_the_measured_one() {
    for (nome, r) in corpus() {
        let (w, _) = erro_no_ecra(&r, HALO_LUT_TEXELS, sampler);
        assert!(
            w <= DISPLAY_STEP,
            "{nome}: a LUT de {HALO_LUT_TEXELS} erra {w}, acima do passo do ecra'"
        );
    }
    let meia = pior(HALO_LUT_TEXELS / 2, sampler);
    assert!(
        meia > DISPLAY_STEP,
        "com metade dos texels o erro ({meia}) tem de ser VISIVEL"
    );
}

/// **NUM DEGRAU, A BANDA ERRADA É UMA CÉLULA POR SALTO — e uma célula é mais fina que o passo do
/// ecrã.** Registo executável de uma limitação: nenhuma tabela amostrada representa uma
/// descontinuidade; o que se afirma é que cada salto suja UMA célula e que ela encolhe com a
/// densidade.
#[test]
fn a_hard_step_dirties_one_cell_and_a_cell_is_finer_than_the_display() {
    let ramp = degrau();
    let stops = ramp.len();
    let (worst, frac) = erro_no_ecra(&ramp, HALO_LUT_TEXELS, sampler);
    assert!(
        worst > DISPLAY_STEP,
        "fixture: um degrau TEM de errar no salto ({worst})"
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
    assert!(cell < DISPLAY_STEP, "uma celula ({cell}) mais fina que o passo");
    let (_, coarse) = erro_no_ecra(&ramp, HALO_LUT_TEXELS / 8, sampler);
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

/// **A LUT assa-se UMA vez por texto**: o mesmo texto devolve a mesma tabela, o texto mexido assa
/// outra, e o lixo não apaga a última boa.
#[test]
fn the_halo_lut_bakes_once_per_text() {
    let mut g = Graph::new();
    let n = g.add_node(TYPE_NAME);
    let texto = |p: GradientPreset| ph2d_color::serialize_gradient(&p.ramp());
    g.set_text_param(n, RAMP_KEY.to_string(), texto(GradientPreset::Heat));
    let a = bake_halo_lut(&g).expect("assa");
    let b = bake_halo_lut(&g).expect("assa");
    assert!(Arc::ptr_eq(&a, &b), "o mesmo texto nao reassa");
    g.set_text_param(n, RAMP_KEY.to_string(), texto(GradientPreset::Ice));
    let c = bake_halo_lut(&g).expect("assa");
    assert!(!Arc::ptr_eq(&a, &c) && a != c, "o texto mexido assa outra");
    g.set_text_param(n, RAMP_KEY.to_string(), "lixo".to_string());
    assert!(bake_halo_lut(&g).is_none(), "lixo e' sem rampa");
}
