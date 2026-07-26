//! **A FIAÇÃO da LUT do filme** (plano 26 §9.6.5) — o que os gates de kernel do
//! [`crate::height_film_aa_tests`] *não podem* provar.
//!
//! Aqueles medem a paridade da fórmula sobre geometrias construídas à mão. Estes medem o PRODUTO:
//!
//! 1. **a estrada rápida é de fato TOMADA** no laço real dos dois kernels — a lição do ADR-0120, onde
//!    um caminho rápido virou código morto com todos os gates verdes porque ninguém contou quantas
//!    vezes ele disparou;
//! 2. **os bytes que o depósito escreve** batem com o oráculo das nove amostras REAIS, sobre um dab
//!    VARRIDO — o único fixture que exercita banda, calota e straddle de uma vez, e o único lugar onde
//!    um `w`/`d`/base trocados de lugar na fiação apareceriam.
//!
//! ⚠️ O oráculo do (2) recomputa a geometria do kernel (`sweep_residual`), **nunca a expansão da LUT**:
//! ele é a referência, não um espelho do caminho rápido.

use crate::height::{HeightDab, HeightFields, accumulate_dab_height};
use crate::height_film::FilmAa;
use crate::height_film_lut::take_lut_counts;
use crate::{BrushSpec, Falloff};

/// Um pincel de impasto com Smooth Edges — o `film_aa_wanted` exige `deposits_height()`.
fn spec(falloff: Falloff, radius: f32) -> BrushSpec {
    BrushSpec {
        radius_px: radius,
        falloff,
        impasto: true,
        impasto_depth: 0.5,
        impasto_smooth_edges: true,
        ..Default::default()
    }
}

/// O mesmo pincel, **achatado e girado** — a geometria em que a versão euclidiana desta LUT errava em
/// silêncio, e que o `raio × minor` da admissibilidade existe para medir.
fn flattened(falloff: Falloff, radius: f32) -> BrushSpec {
    BrushSpec {
        dab_flatten: 0.45,
        dab_angle_deg: 31,
        ..spec(falloff, radius)
    }
}

/// Os cinco planos canvas-shaped que o depósito escreve.
struct Planes {
    height: Vec<f32>,
    paint: Vec<f32>,
    grain: Vec<u8>,
    film: Vec<u8>,
    radius: Vec<f32>,
}

impl Planes {
    fn new(n: usize) -> Self {
        Self {
            height: vec![0.0; n],
            paint: vec![0.0; n],
            grain: vec![crate::height::NO_GRAIN; n],
            film: vec![0; n],
            radius: vec![0.0; n],
        }
    }

    fn fields(&mut self) -> HeightFields<'_> {
        HeightFields {
            height: &mut self.height,
            paint: &mut self.paint,
            grain: &mut self.grain,
            film: &mut self.film,
            radius: &mut self.radius,
        }
    }
}

/// Um dab varrido: a corda aponta a 27° e mede 1/5 do raio, a ordem do produto a spacing 0,1.
fn swept_dab(s: &BrushSpec, radius: f32, centre: [f32; 2]) -> HeightDab<'static> {
    let back = radius * 0.2;
    let (ux, uy) = (0.891_f32, 0.454_f32);
    HeightDab {
        center: centre,
        radius,
        coverage: 1.0,
        footprint: s.dab_footprint([1.0, 0.0]),
        prev_center: Some([centre[0] + back * ux, centre[1] + back * uy]),
        shape: None,
        grain: None,
        grain_image: None,
    }
}

/// Deposita um dab e devolve `(planos, (hits, straddles))`.
///
/// ⚠️ A trava é do CONTADOR, não do depósito: os testes correm em paralelo e os contadores são
/// globais, então sem ela um gate lê os disparos de outro (foi o que a 1ª rodada de mutações mediu).
fn deposit(s: &BrushSpec, radius: f32, side: u32, centre: [f32; 2]) -> (Planes, (usize, usize)) {
    let n = (side as usize) * (side as usize);
    let mut planes = Planes::new(n);
    let dab = swept_dab(s, radius, centre);
    let _guard = crate::height_film_lut::lock_counts();
    let _ = take_lut_counts(); // descarta o que uma corrida anterior deixou
    let _ = accumulate_dab_height(&mut planes.fields(), side, side, s, &dab, None);
    let counts = take_lut_counts();
    (planes, counts)
}

/// **A estrada rápida é TOMADA no depósito real — e RECUSADA onde a regra manda.**
///
/// ⚠️ Este é o gate que o ADR-0120 pagou caro para existir: sem contar os disparos, uma LUT que nunca
/// é chamada deixa toda a suíte verde e o produto exatamente como estava.
#[test]
fn the_deposit_takes_the_lut_road_and_the_inadmissible_brush_does_not() {
    // Admissível: Smooth, redondo, raio 100.
    let (_, (hits, straddles)) = deposit(&spec(Falloff::Smooth, 100.0), 100.0, 256, [128.0, 128.0]);
    assert!(
        hits > 1000,
        "a LUT tem de ser tomada no laço real do depósito: {hits} texels"
    );
    // E o straddle tem de ter DISPARADO — senão o fallback é código morto e o gate do épsilon do
    // kernel estaria medindo uma região que o produto nunca devolve ao caminho exato.
    assert!(
        straddles > 0,
        "a fronteira calota↔banda tem de devolver texels ao caminho exato: {straddles}"
    );

    // `Constant` sai pela cláusula da família (degrau interage com a grade E é mais lento).
    let (_, (hard_hits, _)) = deposit(&spec(Falloff::Constant, 100.0), 100.0, 256, [128.0, 128.0]);
    assert_eq!(hard_hits, 0, "um bico duro não pode tomar a LUT");

    // Raio 20: abaixo do `MIN_EFFECTIVE_RADIUS` medido.
    let (_, (small_hits, _)) = deposit(&spec(Falloff::Smooth, 20.0), 20.0, 128, [64.0, 64.0]);
    assert_eq!(
        small_hits, 0,
        "abaixo do raio efetivo mínimo o depósito fica no caminho exato"
    );

    println!("[lut-wiring] r=100 hits {hits}, straddles {straddles}");
}

/// **Os bytes que o depósito ESCREVE batem com o oráculo das nove amostras reais.**
///
/// O oráculo recomputa a geometria do kernel (o mesmo `sweep_residual`, o mesmo `falloff_weight`) e
/// tira o filme por [`FilmAa::film_at_exact`] — as nove amostras REAIS. Ele não conhece a expansão da
/// LUT, e é isso que o torna referência em vez de espelho.
///
/// A barra é o template do épsilon do passe de luz: **magnitude E contagem**, nunca uma só.
#[test]
fn the_deposited_film_is_the_nine_sample_oracle_to_the_byte() {
    const SIDE: u32 = 256;
    const RADIUS: f32 = 100.0;
    const CENTRE: [f32; 2] = [128.0, 128.0];
    // O raio efetivo MÍNIMO que a admissibilidade aceita — e onde o resto de 3ª ordem é MAIOR.
    const SMALL: f32 = crate::height_film::FilmLut::MIN_EFFECTIVE_RADIUS;
    // Medido: produto correto 0..5 bytes, produto sem o `P` da banda 12..39. Ver o bloco abaixo.
    const MAX_DIFFERING: usize = 8;
    // ⚠️ **A mutação que apaga o `P` da banda sobreviveu a DUAS rodadas, e o buraco era a BARRA, não a
    // fixture.** Eu a tinha posto em 64 bytes por palpite (*"o aro tem ~1300 texels"*) — o limite que só
    // diz "por segurança" que o §0 proíbe. Medido: o produto correto diverge do oráculo em **0 a 5
    // bytes**; sem o `P`, em **12 a 39**. A barra é 8, e os dois números vivem aqui para ninguém a
    // afrouxar de novo. As fixtures extra ficam por cobertura: o bico ACHATADO é onde a versão
    // euclidiana desta LUT errava em silêncio, e **r=40** é onde o resto de 3ª ordem é maior (ele
    // escala com a curvatura, `1/raio²`: 0,58 nível a r=40 contra ~0,09 a r=100).
    let brushes = [
        ("redondo r100", spec(Falloff::Smooth, RADIUS), RADIUS),
        ("redondo r100", spec(Falloff::Smoother, RADIUS), RADIUS),
        ("redondo r100", spec(Falloff::Sphere, RADIUS), RADIUS),
        ("redondo r100", spec(Falloff::Sharp, RADIUS), RADIUS),
        ("achatado r100", flattened(Falloff::Smooth, RADIUS), RADIUS),
        ("achatado r100", flattened(Falloff::Sphere, RADIUS), RADIUS),
        ("redondo r40", spec(Falloff::Smooth, SMALL), SMALL),
        ("redondo r40", spec(Falloff::Sphere, SMALL), SMALL),
    ];
    for (shape_name, s, radius) in &brushes {
        let (falloff, radius) = (s.falloff, *radius);
        let (planes, (hits, _)) = deposit(s, radius, SIDE, CENTRE);
        assert!(hits > 0, "{falloff:?}: o fixture tem de conter o fenômeno");

        let dab = swept_dab(s, radius, CENTRE);
        let aa = FilmAa::for_dab(s, false, radius).expect("a banda existe");
        let sweep = crate::height::sweep_axis(&dab);
        let inv = 1.0 / radius;
        let coverage = s.flow * s.strength; // `dab.coverage` é 1.0
        let (mut worst, mut differing) = (0i32, 0usize);
        for py in 0..SIDE as i64 {
            let dy = (py as f32 + 0.5) - CENTRE[1];
            for px in 0..SIDE as i64 {
                let dx = (px as f32 + 0.5) - CENTRE[0];
                let (rx, ry) = crate::height::sweep_residual(dx, dy, sweep);
                let t = dab.footprint.falloff_t(rx * inv, ry * inv);
                let w = s.falloff_weight(t); // sem Shape => a silhueta E o falloff
                let film = aa.film_at_exact(t, w, |ox, oy| {
                    let (qx, qy) = crate::height::sweep_residual(dx + ox, dy + oy, sweep);
                    s.falloff_weight(dab.footprint.falloff_t(qx * inv, qy * inv))
                });
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a mesma quantização do kernel, sobre um valor clampado em [0,1]"
                )]
                let want = ((coverage * film).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                let got = planes.film[(py as usize) * (SIDE as usize) + px as usize];
                let d = i32::from(got) - i32::from(want);
                if d != 0 {
                    differing += 1;
                    worst = worst.max(d.abs());
                }
            }
        }
        // Magnitude: a expansão erra na 3ª ordem, e o gate do kernel a mediu em 0,06 nível — um byte
        // de folga cobre a quantização em cima disso.
        assert!(
            worst <= 1,
            "{shape_name} {falloff:?}: pior byte {worst} (limite 1) em {differing} texels"
        );
        // Contagem: `worst` sozinho não basta (tirar o `+0.5` de um `quantise` moveu 2375 bytes por UM
        // nível e passou sob um limite de magnitude) — e aqui ela é a METADE QUE MORDE: toda mutação
        // desta wave erra por exatamente um nível, então só a contagem as separa.
        assert!(
            differing <= MAX_DIFFERING,
            "{shape_name} {falloff:?}: {differing} bytes divergem (limite {MAX_DIFFERING}), pior {worst}"
        );
        println!("[lut-wiring] {shape_name} {falloff:?}: {differing} bytes divergem, pior {worst}");
    }
}

/// **Quão GRANDE é a banda do AA?** — a pergunta que o custo por-dab levantou.
///
/// O AA custa **4,18 ms de 9,92** num dab de raio 100 (`measure_impasto_cost::
/// where_the_relief_dab_spends_its_time`, pareado). Se a banda fosse o aro fino que eu vinha supondo
/// (~7 700 texels), isso daria **543 ns por texel** para nove leituras de tabela — absurdo. Ou a banda
/// é muito maior do que eu supunha, ou o custo não está onde eu acho. Esta sonda responde a metade que
/// é aritmética pura.
#[test]
#[ignore]
fn how_wide_is_the_aa_band() {
    println!("[banda] falloff | t_lo .. t_hi | fracao da AREA do disco | texels a r=100");
    for falloff in [
        Falloff::Sphere, // o default do impasto
        Falloff::Smooth,
        Falloff::Smoother,
        Falloff::Sharp,
    ] {
        let s = spec(falloff, 100.0);
        let aa = FilmAa::for_dab(&s, false, 100.0).expect("banda a r=100");
        let (lo, hi) = (aa.t_lo_for_test().max(0.0), aa.t_hi_for_test().min(1.0));
        let frac = hi * hi - lo * lo; // area do anel / area do disco
        let texels = frac * std::f32::consts::PI * 100.0 * 100.0;
        println!(
            "[banda] {:<10} | {lo:.3} .. {hi:.3} | {:>5.1}% | {texels:>8.0}",
            format!("{falloff:?}"),
            frac * 100.0
        );
    }
}
