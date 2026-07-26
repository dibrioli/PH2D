//! ⛔ **O AA do filme com MENOS amostras: construído, medido, REJEITADO** (plano 26 §9.5).
//!
//! O AA é **54% de um traço de impasto** (68,7 de 127,1 ms a raio 100), então trocar as nove amostras
//! da grade 3×3 pela **cruz de cinco + as quinas por extensão separável** valia ~30% do traço. Este
//! arquivo é o que mediu, e o que a medição disse foi **não**:
//!
//! 1. **`Constant` erra `2/9` EXATO (56,67 níveis de u8) em TODO raio e em TODOS os texels da banda** —
//!    com borda dura o `film_of` é um DEGRAU, e a extensão separável erra dois dos nove termos por
//!    construção. É o caso pelo qual o AA existe.
//! 2. **O erro dos falloffs suaves NÃO é monotônico no raio** — picos isolados de ~2 níveis em raios
//!    arbitrários (`Sharp` r=50 e r=70; `Sphere` r=50 e r=90) ⇒ **nenhum limiar de raio limita o
//!    erro**, e um limiar tirado da tabela seria o *"limite que só diz por segurança"* do §0.
//!
//! O que **FICA** é a maquinaria: `film_at_exact` (o oráculo de nove amostras), a varredura por
//! falloff × raio × texel, e a `measure_the_epsilon_by_radius` que produz a tabela. Sem elas a próxima
//! tentativa é opinião; com elas é uma rodada de `cargo test`.
//!
//! ⚠️ O gate ativo virou **regressão-guard**: hoje o `film_at` DELEGA ao exato, então a paridade é
//! byte-exata, e quem reinstalar uma estimativa vê o gate acender com o número dela. O template do
//! épsilon (magnitude **E** contagem, nunca uma só — tirar o `+0.5` de um `quantise` moveu 2375 bytes
//! por UM nível e passava sob um limite de 2) fica escrito para quando houver o que medir.

use crate::height_film::{FilmAa, W_TAIL};
use crate::{BrushSpec, Falloff};

/// Os falloffs que o produto pinta. `Custom` fica fora: a `for_dab` toma o dab INTEIRO como banda para
/// ele (uma curva não-monótona não tem cruzamento único), então ele já é o caso mais largo e entra pelo
/// gate do pior caso abaixo, não por nome.
const FALLOFFS: [Falloff; 6] = [
    Falloff::Smooth,
    Falloff::Sphere,
    Falloff::Sharp,
    Falloff::Pow4,
    Falloff::Constant,
    Falloff::Root,
];

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

/// A silhueta do **DISCO** — a cadeia que o `dab.rs` supersampleia.
fn disc(s: &BrushSpec, radius: f32, dx: f32, dy: f32) -> impl Fn(f32, f32) -> f32 + use<'_> {
    let inv = 1.0 / radius;
    move |ox, oy| {
        let (x, y) = ((dx + ox) * inv, (dy + oy) * inv);
        s.falloff_weight(s.dab_footprint([1.0, 0.0]).falloff_t(x, y))
    }
}

/// O pior erro e quantos texels o carregam, varrendo a banda inteira de um dab.
/// Devolve `(pior |Δ| em fração, texels com Δ ≥ 1/255, texels na banda)`.
fn sweep(falloff: Falloff, radius: f32) -> (f32, usize, usize) {
    let s = spec(falloff, radius);
    let Some(aa) = FilmAa::for_dab(&s, false, radius) else {
        return (0.0, 0, 0);
    };
    let inv = 1.0 / radius;
    let fp = s.dab_footprint([1.0, 0.0]);
    let reach = radius.ceil() as i64 + 2;
    let (mut worst, mut differing, mut band) = (0.0f32, 0usize, 0usize);
    for py in -reach..=reach {
        for px in -reach..=reach {
            // Centro de texel, como os dois kernels amostram.
            let (dx, dy) = (px as f32 + 0.5, py as f32 + 0.5);
            let t = fp.falloff_t(dx * inv, dy * inv);
            let sil = s.falloff_weight(t);
            let f = disc(&s, radius, dx, dy);
            // ⚠️ A banda é onde a GRADE de fato é amostrada, não onde as duas rotas divergem: com o
            // `film_at` delegando ao exato a divergência é sempre zero, e um controle definido pela
            // divergência se autodestruiria (aconteceu — este gate acendeu no próprio controle).
            if t > aa.t_lo_for_test() && t < aa.t_hi_for_test() {
                band += 1;
            }
            let a = aa.film_at(t, sil, &f);
            let b = aa.film_at_exact(t, sil, &f);
            let d = (a - b).abs();
            if d > worst {
                worst = d;
            }
            if d >= 1.0 / 255.0 {
                differing += 1;
            }
        }
    }
    (worst, differing, band)
}

/// **O produto usa as NOVE amostras** — e este gate é o guard do negativo.
///
/// Hoje o `film_at` delega ao `film_at_exact`, então a paridade é **byte-exata** sobre todo falloff ×
/// raio × texel da banda. Quem reinstalar uma estimativa acende este gate com o número dela, em vez de
/// descobrir na tela.
///
/// ⚠️ O gate afirma **as duas** perguntas (quão longe · quantos), porque um limite de magnitude sozinho
/// não basta — a lição do `quantise` do passe de luz: 2375 bytes por UM nível passavam sob um limite
/// de 2. Com a delegação os dois são zero; a barra de meio nível fica escrita para a próxima tentativa.
///
/// Mutação: apontar o `film_at` de volta para a estimativa separável ⇒ `Constant` sangra 56,67 níveis
/// em 788 de 788 texels, RED imediato.
#[test]
#[ignore = "medicao: acha o joelho do raio"]
fn measure_the_epsilon_by_radius() {
    println!("[eps] falloff        raio  pior erro  niveis u8  texels>=1/255");
    for falloff in FALLOFFS {
        for radius in [30.0f32, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 120.0] {
            let (worst, differing, band) = sweep(falloff, radius);
            if band == 0 {
                continue;
            }
            println!(
                "[eps] {falloff:<12?} {radius:>5}  {worst:>9.6}  {:>9.2}  {differing:>4} de {band}",
                worst * 255.0
            );
        }
    }
}

#[test]
fn the_product_film_is_the_nine_sample_reference_to_the_byte() {
    // Um nível de u8 = 3,92e-3. A barra é **meio nível**: abaixo dela nenhum texel pode nem arredondar
    // para um byte diferente por causa da estimativa sozinha.
    const WORST_BAR: f32 = 0.5 / 255.0;
    let mut worst_overall = 0.0f32;
    let mut worst_where = (Falloff::Smooth, 0.0f32);
    for falloff in FALLOFFS {
        for radius in [3.0f32, 6.0, 12.0, 25.0, 50.0, 100.0, 200.0] {
            let (worst, differing, band) = sweep(falloff, radius);
            assert!(
                band > 0 || radius < 4.0,
                "controle: a varredura de {falloff:?} r={radius} nao encontrou banda de AA nenhuma — \
                 um gate sobre zero texels e verde por construcao"
            );
            if worst > worst_overall {
                worst_overall = worst;
                worst_where = (falloff, radius);
            }
            assert!(
                worst <= WORST_BAR,
                "{falloff:?} r={radius}: pior erro {worst:.6} > barra {WORST_BAR:.6} \
                 ({differing} de {band} texels da banda a >= 1/255)"
            );
            assert_eq!(
                differing, 0,
                "{falloff:?} r={radius}: o produto delega ao exato, entao NENHUM texel pode divergir \
                 — {differing} de {band} divergiram, logo alguem reinstalou uma estimativa"
            );
        }
    }
    println!(
        "[aa-parity] pior erro {worst_overall:.8} ({:.3} nivel de u8) em {:?} r={}",
        worst_overall * 255.0,
        worst_where.0,
        worst_where.1
    );
}

/// **Na CRUZ e no CENTRO nada é aproximado** — as cinco amostras reais são as mesmas cinco que as nove
/// usam, então um texel cujo gradiente é puramente axial sai **byte-idêntico**, não meramente perto.
///
/// É a metade que separa esta estimativa de uma fórmula nova: ela é exata para qualquer silhueta
/// **separável** sobre o texel, e a única fonte de erro é o termo cruzado.
#[test]
fn a_separable_silhouette_is_reproduced_exactly() {
    let s = spec(Falloff::Smooth, 40.0);
    let aa = FilmAa::for_dab(&s, false, 40.0).expect("a banda existe a r=40");
    // Uma silhueta separável POR CONSTRUÇÃO: soma de um termo em x e um em y, dentro da banda.
    let sil_sep = |ox: f32, oy: f32| (W_TAIL + 0.2) + 0.03 * ox + 0.05 * oy;
    let t = 0.5 * (aa.t_lo_for_test() + aa.t_hi_for_test()); // no meio da banda
    let a = aa.film_at(t, sil_sep(0.0, 0.0), sil_sep);
    let b = aa.film_at_exact(t, sil_sep(0.0, 0.0), sil_sep);
    assert!(
        (a - b).abs() == 0.0,
        "uma silhueta separavel tem de sair EXATA: {a} vs {b}"
    );
}

/// **Fora da banda as duas são a mesma função, ao bit** — o interior e o exterior do dab (a maior parte
/// da bbox) nunca entraram na grade, e o early-out é literalmente compartilhado.
#[test]
fn outside_the_band_both_are_the_single_sample() {
    let s = spec(Falloff::Sphere, 60.0);
    let aa = FilmAa::for_dab(&s, false, 60.0).expect("a banda existe a r=60");
    let never = |_: f32, _: f32| panic!("fora da banda a grade nao pode ser amostrada");
    for (t, sil) in [(0.0f32, 1.0f32), (1.0, 0.0)] {
        assert!(
            (aa.film_at(t, sil, never) - aa.film_at_exact(t, sil, never)).abs() == 0.0,
            "t={t} esta fora da banda: as duas devolvem o mesmo single-sample"
        );
    }
}
