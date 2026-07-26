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

use crate::height_film::{FilmAa, FilmLut, W_TAIL};
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

/// **A LUT PRÉ-CONVOLUÍDA contra as nove amostras reais** (plano 26 §9.6) — o épsilon, medido.
///
/// O gradiente vem de onde os chamadores o têm: para o disco `t = |d|/radius`, logo
/// `∇t = d̂/radius = d/(t·radius²)`. Nenhuma raiz nova — o `t` já traz a norma.
///
/// Rodar: `cargo test -p ph2d-painter-brush --release measure_the_lut_epsilon -- --ignored --nocapture`
#[test]
#[ignore = "medicao: o epsilon da LUT por falloff x raio"]
fn measure_the_lut_epsilon() {
    println!("[lut-eps] falloff        raio  pior erro  niveis u8  texels>=1/255  de");
    for falloff in FALLOFFS {
        for radius in [3.0f32, 5.0, 8.0, 12.0, 20.0, 40.0, 100.0] {
            let s = spec(falloff, radius);
            let Some(aa) = FilmAa::for_dab(&s, false, radius) else {
                continue;
            };
            let lut = FilmLut::new(&s);
            let fp = s.dab_footprint([1.0, 0.0]);
            let inv = 1.0 / radius;
            let reach = radius.ceil() as i64 + 2;
            let (mut worst, mut differing, mut band) = (0.0f32, 0usize, 0usize);
            for py in -reach..=reach {
                for px in -reach..=reach {
                    let (dx, dy) = (px as f32 + 0.5, py as f32 + 0.5);
                    let t = fp.falloff_t(dx * inv, dy * inv);
                    if t <= aa.t_lo_for_test() || t >= aa.t_hi_for_test() {
                        continue;
                    }
                    band += 1;
                    let sil = s.falloff_weight(t);
                    // ∇t em unidades de texel: d̂/radius, e d̂ = d/(t·radius).
                    let scale = 1.0 / (t * radius * radius).max(1e-9);
                    let (gx, gy) = (dx * scale, dy * scale);
                    let a = aa.film_at_lut(&lut, t, gx, gy);
                    let b = aa.film_at_exact(t, sil, disc(&s, radius, dx, dy));
                    let d = (a - b).abs();
                    if d > worst {
                        worst = d;
                    }
                    if d >= 1.0 / 255.0 {
                        differing += 1;
                    }
                }
            }
            if band == 0 {
                continue;
            }
            println!(
                "[lut-eps] {falloff:<12?} {radius:>5}  {worst:>9.6}  {:>9.2}  {differing:>10}  {band}",
                worst * 255.0
            );
        }
    }
}

/// **A LUT é mais RÁPIDA?** — a razão contra as nove amostras reais, no mesmo instante.
///
/// Rodar: `cargo test -p ph2d-painter-brush --release measure_the_lut_speed -- --ignored --nocapture`
#[test]
#[ignore = "medicao: a razao de velocidade"]
fn measure_the_lut_speed() {
    use std::time::Instant;
    const N: usize = 3_000_000;
    println!("[lut-perf] falloff      nove reais      LUT      razao");
    for falloff in [Falloff::Smooth, Falloff::Sphere, Falloff::Constant] {
        let radius = 100.0f32;
        let s = spec(falloff, radius);
        let aa = FilmAa::for_dab(&s, false, radius).expect("banda");
        let lut = FilmLut::new(&s);
        let fp = s.dab_footprint([1.0, 0.0]);
        let inv = 1.0 / radius;
        // Um texel na banda.
        let mut t0 = 0.0f32;
        for k in 1..4000u32 {
            let t = f32::from(u16::try_from(k).unwrap_or(u16::MAX)) / 4000.0;
            if t > aa.t_lo_for_test() && t < aa.t_hi_for_test() {
                t0 = t;
                break;
            }
        }
        let d = t0 * radius;
        let sil = s.falloff_weight(t0);
        let scale = 1.0 / (t0 * radius * radius);
        let (gx, gy) = (d * scale, 0.0);
        let mut acc = 0.0f32;
        let t_ref = Instant::now();
        for k in 0..N {
            let j = (k % 7) as f32 * 1e-6;
            acc += aa.film_at_exact(t0 + j, sil, |ox, oy| {
                s.falloff_weight(fp.falloff_t((d + ox) * inv, oy * inv))
            });
        }
        let nine = t_ref.elapsed().as_secs_f64() * 1e3;
        let t_lut = Instant::now();
        for k in 0..N {
            let j = (k % 7) as f32 * 1e-6;
            acc += aa.film_at_lut(&lut, t0 + j, gx, gy);
        }
        let l = t_lut.elapsed().as_secs_f64() * 1e3;
        println!(
            "[lut-perf] {falloff:<10?} {nine:>10.1} ms {l:>8.1} ms  {:>5.2}x   (acc {acc:.0})",
            nine / l.max(1e-9)
        );
    }
}

/// **O GATE da LUT: no regime admissível ela é o mesmo filme, dentro do épsilon declarado.**
///
/// "Admissível" não é gosto — é a família de falloffs **suaves** com **raio ≥ 20**, e o número saiu da
/// varredura (`measure_the_lut_epsilon`), não de preferência. O pior caso a r=20 é o `Pow4` com **0,44
/// nível de u8 e ZERO texels movendo um nível**; a r=40 já é 0,06 e a r=100, 0,00.
///
/// ⚠️ **O limiar era r ≥ 90 antes do termo de segunda ordem** — só a expansão de 1ª ordem deixava o
/// `Pow4` em 1,49 nível a r=50. Os ~4 flops do `perp2/(2t)` derrubaram o erro ~150× e custaram ~20% da
/// razão de velocidade (2,8× → 2,3×). É a troca certa: sem eles o regime admissível não cobria nem o
/// pincel default do impasto (40 px de diâmetro = raio 20).
///
/// **`Constant` se exclui por DOIS motivos ao mesmo tempo:** é errático (0,00 nível até r=40, mas
/// **13,25 em 16 texels** a r=100 — o degrau interage com a grade) **e é mais LENTO** (0,46×, porque a
/// curva dele é a constante 1 e não há raiz a economizar). Um recorte que serve à precisão E à velocidade
/// não é caso especial: é o domínio da otimização. `Custom` fica fora porque a `for_dab` toma o dab
/// inteiro como banda para ele e a tabela seria indexada por uma curva do documento.
///
/// ⚠️ **E o que este gate NÃO cobre, nomeado:** a geometria é o **DISCO** (a rota do pigmento). A rota
/// de ALTURA supersampleia a **CÁPSULA VARRIDA**, cujo campo de distância não é `|d|` em torno de um
/// ponto — a expansão de 2ª ordem acima **não está validada lá**, e é o primeiro passo de qualquer
/// fiação (plano 26 §9.6).
///
/// O gate afirma **as duas** perguntas (quão longe · quantos), a lição do `quantise` do passe de luz.
///
/// Mutação: baixar o `FilmLut::N` para 256 ⇒ a resolução em `t` (3,9e-3) passa a dominar o erro da
/// linearização (~4,4e-5) e o gate acende.
#[test]
fn the_lut_film_is_inside_its_epsilon_where_it_is_admissible() {
    // Meio nível de u8: abaixo dela a estimativa sozinha não pode nem arredondar para outro byte.
    const WORST_BAR: f32 = 0.5 / 255.0;
    // A família SUAVE — `Constant` fora por medição (ver o doc acima), `Custom` fora porque a `for_dab`
    // toma o dab inteiro como banda para ele e a tabela seria indexada por uma curva do documento.
    const SMOOTH_FAMILY: [Falloff; 5] = [
        Falloff::Smooth,
        Falloff::Sphere,
        Falloff::Sharp,
        Falloff::Pow4,
        Falloff::Root,
    ];
    let mut worst_overall = 0.0f32;
    for falloff in SMOOTH_FAMILY {
        for radius in [50.0f32, 100.0, 200.0] {
            let s = spec(falloff, radius);
            let aa = FilmAa::for_dab(&s, false, radius).expect("a banda existe");
            let lut = FilmLut::new(&s);
            let fp = s.dab_footprint([1.0, 0.0]);
            let inv = 1.0 / radius;
            let reach = radius.ceil() as i64 + 2;
            let (mut worst, mut differing, mut band) = (0.0f32, 0usize, 0usize);
            let mut toe_flips = 0usize;
            for py in -reach..=reach {
                for px in -reach..=reach {
                    let (dx, dy) = (px as f32 + 0.5, py as f32 + 0.5);
                    let t = fp.falloff_t(dx * inv, dy * inv);
                    if t <= aa.t_lo_for_test() || t >= aa.t_hi_for_test() {
                        continue;
                    }
                    band += 1;
                    let scale = 1.0 / (t * radius * radius).max(1e-9);
                    let a = aa.film_at_lut(&lut, t, dx * scale, dy * scale);
                    let b = aa.film_at_exact(t, s.falloff_weight(t), disc(&s, radius, dx, dy));
                    let d = (a - b).abs();
                    // ⚠️ O **corte do TOE** é uma descontinuidade do PRODUTO (`frac < 2/255 ⇒ 0`), não
                    // erro da estimativa: quando um lado cai um fio abaixo dele e o outro acima, a
                    // diferença de saída é ~2/255 por construção, vinda de uma diferença de entrada
                    // ínfima. Contado à parte — misturá-lo com a precisão fez a varredura anterior ler
                    // "picos em raios arbitrários" onde havia UM mecanismo, e foi por isso que a §9.5
                    // rejeitou a estimativa separável por uma razão que era metade falsa.
                    if (a == 0.0) != (b == 0.0) && a.max(b) < 3.0 / 255.0 {
                        toe_flips += 1;
                        continue;
                    }
                    if d > worst {
                        worst = d;
                    }
                    if d >= 1.0 / 255.0 {
                        differing += 1;
                    }
                }
            }
            assert!(
                band > 100,
                "controle: {falloff:?} r={radius} varreu {band} texels de banda — poucos para o gate \
                 significar algo"
            );
            worst_overall = worst_overall.max(worst);
            assert!(
                worst <= WORST_BAR,
                "{falloff:?} r={radius}: pior erro {worst:.6} ({:.2} nivel) > barra {WORST_BAR:.6}",
                worst * 255.0
            );
            assert_eq!(
                differing, 0,
                "{falloff:?} r={radius}: {differing} de {band} texels da banda movem um nivel de u8 — \
                 a barra de MAGNITUDE sozinha nao basta, e esta e a metade da CONTAGEM"
            );
            // O toe é a cliff do produto, mas ela não pode virar uma cerca larga: um punhado de texels
            // a ~1% de opacidade trocando por zero é invisível; centenas seriam outra coisa.
            assert!(
                toe_flips * 200 <= band,
                "{falloff:?} r={radius}: {toe_flips} de {band} texels cruzaram o corte do toe — acima \
                 de 0,5% da banda isso deixa de ser a cliff do produto e passa a ser a estimativa"
            );
        }
    }
    println!(
        "[lut-gate] pior erro no regime admissivel: {:.3} nivel de u8",
        worst_overall * 255.0
    );
}

/// **A LUT É a curva, amostrada** — não uma segunda resposta a *"qual é o filme neste `t`"*.
///
/// Nos nós ela devolve exactamente `film_of(falloff_weight(t))`, ao bit: é o que impede a tabela de
/// virar um modelo paralelo que deriva do produto (a doença que o doc 24 nomeou ao tabelar o sRGB).
#[test]
fn the_lut_is_the_curve_sampled_not_a_second_answer() {
    for falloff in FALLOFFS {
        let s = spec(falloff, 50.0);
        let lut = FilmLut::new(&s);
        for i in [
            0usize,
            1,
            7,
            100,
            FilmLut::N / 3,
            FilmLut::N - 1,
            FilmLut::N,
        ] {
            #[expect(clippy::cast_precision_loss, reason = "i <= N = 16384, exato em f32")]
            let t = (i as f32) / (FilmLut::N as f32);
            let exact = crate::height_film::film_of(s.falloff_weight(t));
            assert!(
                (lut.at(t) - exact).abs() == 0.0,
                "{falloff:?} no no t={t}: a tabela ({}) tem de SER a curva ({exact})",
                lut.at(t)
            );
        }
    }
}
