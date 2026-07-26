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

/// A base `B·o` no espaço deformado, para uma geometria dada — a MESMA conta que os dois kernels
/// farão: `B = A·M/radius`, com `M = I` no disco/calota e `M = I − uuᵀ` na banda da cápsula.
fn deformed_basis(
    fp: crate::FootprintDeform,
    radius: f32,
    axis: Option<[f32; 2]>,
) -> ([f32; 2], [f32; 2]) {
    let inv = 1.0 / radius;
    let m = |o: [f32; 2]| match axis {
        None => o,
        Some(u) => {
            let s = o[0] * u[0] + o[1] * u[1];
            [o[0] - s * u[0], o[1] - s * u[1]]
        }
    };
    let bx = fp.apply(m([inv, 0.0]));
    let by = fp.apply(m([0.0, inv]));
    (bx, by)
}

/// As QUATRO geometrias que os dois kernels de fato usam, e a quinta que é a fronteira.
#[derive(Clone, Copy, Debug)]
enum Geom {
    /// Disco redondo — a rota do pigmento no caso comum.
    Disc,
    /// Disco sob **Flatten & Rotate** — onde a versão euclidiana desta LUT errava EM SILÊNCIO.
    Ellipse,
    /// Uma elipse **fortemente** achatada (`minor = 0,2`). Ela existe para separar `raio` de
    /// `raio × minor`: a r=40 o efetivo é **8**, que a regra correta recusa e uma regra escrita só no
    /// `raio` aceitaria. Sem ela a mutação que apaga o `minor` SOBREVIVE — aconteceu.
    Sliver,
    /// A **BANDA** da cápsula varrida — a rota da altura no meio de um traço.
    CapsuleBand,
    /// A **CALOTA** da cápsula — geometricamente um disco no extremo.
    CapsuleCap,
    /// O **STRADDLE**: os texels cuja grade 3×3 cruza a fronteira calota↔banda, onde o `B` correto muda
    /// no meio da grade. É o único lugar onde a expansão não tem uma base única válida, e por isso ele é
    /// uma REGIÃO nomeada em vez de ruído dentro das outras duas — o produto vai devolvê-lo ao caminho
    /// exato, e é este gate que diz quanto custa não fazê-lo.
    CapsuleStraddle,
}

/// Varre uma geometria e devolve `(pior erro, texels >= 1/255, texels na banda, flips do toe)`.
fn sweep_lut(falloff: Falloff, radius: f32, geom: Geom) -> (f32, usize, usize, usize) {
    let mut s = spec(falloff, radius);
    if matches!(geom, Geom::Ellipse | Geom::Sliver) {
        // Um bico achatado e girado: exatamente o caso que a fórmula euclidiana não cobria.
        s.dab_flatten = if matches!(geom, Geom::Sliver) {
            0.8
        } else {
            0.45
        };
        s.dab_angle_deg = 31;
    }
    let Some(aa) = FilmAa::for_dab(&s, false, radius) else {
        return (0.0, 0, 0, 0);
    };
    let lut = FilmLut::new(&s);
    let fp = s.dab_footprint([1.0, 0.0]);
    let inv = 1.0 / radius;
    // A cápsula: eixo a 27° e uma corda de ~1/5 do raio (a ordem do produto a spacing 0,1).
    let (ux, uy) = (0.891_f32, 0.454_f32);
    let back = radius * 0.2;
    let reach = radius.ceil() as i64 + 3;
    let (mut worst, mut differing, mut band, mut toe_flips) = (0.0f32, 0usize, 0usize, 0usize);
    for py in -reach..=reach {
        for px in -reach..=reach {
            let (dx, dy) = (px as f32 + 0.5, py as f32 + 0.5);
            // O resíduo, e a região — exatamente como o `sweep_residual` decide.
            let (rx, ry, in_band) = match geom {
                Geom::Disc | Geom::Ellipse | Geom::Sliver => (dx, dy, false),
                Geom::CapsuleBand | Geom::CapsuleCap | Geom::CapsuleStraddle => {
                    let proj = dx * ux + dy * uy;
                    let sc = proj.clamp(0.0, back);
                    // A grade alcança `AA_REACH` texels em cada direção, e `u` é unitário, então uma
                    // sub-amostra pode mover a projeção por até isso. Um texel cuja projeção está a menos
                    // disso de 0 ou de `back` tem sub-amostras nas DUAS regiões.
                    const AA_REACH: f32 = 0.667;
                    let straddles = (proj - 0.0).abs() < AA_REACH || (proj - back).abs() < AA_REACH;
                    let inside = proj > 0.0 && proj < back;
                    let want = match geom {
                        Geom::CapsuleStraddle => straddles,
                        Geom::CapsuleBand => inside && !straddles,
                        _ => !inside && !straddles,
                    };
                    if !want {
                        continue;
                    }
                    (dx - sc * ux, dy - sc * uy, inside)
                }
            };
            let wv = fp.apply([rx * inv, ry * inv]);
            let t = (wv[0] * wv[0] + wv[1] * wv[1]).sqrt();
            if t <= aa.t_lo_for_test() || t >= aa.t_hi_for_test() {
                continue;
            }
            band += 1;
            let axis = in_band.then_some([ux, uy]);
            let (bx, by) = deformed_basis(fp, radius, axis);
            let a = aa.film_at_lut(&lut, t, wv, bx, by);
            // O oráculo: as nove amostras REAIS, pela MESMA cadeia que o produto percorre.
            let b = aa.film_at_exact(t, s.falloff_weight(t), |ox, oy| {
                let (qx, qy) = (dx + ox, dy + oy);
                let (r2x, r2y) = match geom {
                    Geom::Disc | Geom::Ellipse | Geom::Sliver => (qx, qy),
                    Geom::CapsuleBand | Geom::CapsuleCap | Geom::CapsuleStraddle => {
                        let sc = (qx * ux + qy * uy).clamp(0.0, back);
                        (qx - sc * ux, qy - sc * uy)
                    }
                };
                s.falloff_weight(fp.falloff_t(r2x * inv, r2y * inv))
            });
            // O corte do TOE e a cliff do PRODUTO, contada a parte (ver o gate).
            if (a == 0.0) != (b == 0.0) && a.max(b) < 3.0 / 255.0 {
                toe_flips += 1;
                continue;
            }
            let d = (a - b).abs();
            if d > worst {
                worst = d;
            }
            if d >= 1.0 / 255.0 {
                differing += 1;
            }
        }
    }
    (worst, differing, band, toe_flips)
}

/// **O ÉPSILON da LUT nas QUATRO geometrias** — a medição que decide o regime admissível.
///
/// Rodar: `cargo test -p ph2d-painter-brush --release measure_the_lut_epsilon -- --ignored --nocapture`
#[test]
#[ignore = "medicao: o epsilon da LUT por geometria x falloff x raio"]
fn measure_the_lut_epsilon() {
    println!("[lut-eps] geom          falloff       raio   niveis u8  >=1/255  banda  toe");
    for geom in [
        Geom::Disc,
        Geom::Ellipse,
        Geom::Sliver,
        Geom::CapsuleBand,
        Geom::CapsuleCap,
        Geom::CapsuleStraddle,
    ] {
        for falloff in FALLOFFS {
            for radius in [8.0f32, 12.0, 20.0, 40.0, 100.0] {
                let (worst, differing, band, toe) = sweep_lut(falloff, radius, geom);
                if band == 0 {
                    continue;
                }
                println!(
                    "[lut-eps] {geom:<13?} {falloff:<12?} {radius:>5}  {:>9.2}  {differing:>7}  {band:>5}  {toe:>3}",
                    worst * 255.0
                );
            }
        }
    }
}

/// **O GATE: no regime admissível a LUT é o mesmo filme, nas CINCO regiões.**
///
/// # A regra de admissibilidade, e de que RECURSO ela é
///
/// **família de falloff SUAVE  ∧  `raio × minor ≥ 40`.** O segundo fator não é um raio escolhido: o erro
/// é o resto de 3ª ordem da expansão, logo escala com a **CURVATURA** da silhueta, e a curvatura é
/// governada pelo **menor raio local** — que sob Flatten & Rotate é `raio × minor`, não `raio`. A
/// medição confirma a lei: a elipse de `minor = 0,45` erra **6×** a versão redonda no mesmo raio, e
/// `1/0,45² = 4,9`.
///
/// Medido (pior nível de u8, por região):
///
/// | região | r=20 | r=40 | r=100 |
/// |---|---|---|---|
/// | **CapsuleBand** | **0,02** | **0,00** | **0,00** |
/// | Disc | 0,44 | 0,06 | 0,00 |
/// | CapsuleCap | 0,66 | 0,06 | 0,00 |
/// | CapsuleStraddle | 0,74 | 0,21 | 0,04 |
/// | Ellipse (minor 0,45) | 2,68 | 0,30 | 0,01 |
///
/// ⚠️ **E a coincidência que não é coincidência:** a LUT é admissível a partir de `raio × minor ≥ 40`, e
/// é **a partir daí que o AA custa caro** (68,7 ms a r=100 contra ~9 a r=20). Os dois escalam com o
/// tamanho da pegada, então *ela rende exactamente onde o custo está e é recusada exactamente onde
/// erraria*. O pincel default do impasto (40 px de diâmetro = raio 20) fica no caminho EXATO — e é
/// barato lá.
///
/// `Constant` se exclui por DOIS motivos ao mesmo tempo: é errático (o degrau interage com a grade de
/// texels) **e é mais LENTO** (0,46× — a curva dele é a constante 1, não há raiz a economizar).
/// `Custom` fica fora porque a `for_dab` toma o dab inteiro como banda para ele e a tabela seria
/// indexada por uma curva do documento.
///
/// ⚠️ **A `Ellipse` pegou um bug MEU:** a primeira versão desta LUT expandia `t = |d|/r` como se o
/// espaço fosse euclidiano, e sob Flatten & Rotate isso está errado — **silenciosamente**. A derivação
/// certa é no espaço **deformado**, e é ela que também torna a cápsula trivial.
///
/// Mutações: `FilmLut::N = 256` sangra · tirar o termo de 2ª ordem sangra · usar a base **sem** o afim do
/// footprint (o bug euclidiano) sangra na `Ellipse`.
#[test]
fn the_lut_film_is_inside_its_epsilon_in_every_geometry() {
    const WORST_BAR: f32 = 0.5 / 255.0;
    const SMOOTH_FAMILY: [Falloff; 5] = [
        Falloff::Smooth,
        Falloff::Sphere,
        Falloff::Sharp,
        Falloff::Pow4,
        Falloff::Root,
    ];
    let mut worst_overall = 0.0f32;
    let mut checked = 0usize;
    // ⚠️ O **STRADDLE fica FORA**, e não por conveniência: ele é a cláusula 3 da
    // [`FilmLut::admissible`] — a região onde nenhuma base única serve, que o chamador devolve ao
    // caminho exato. O gate irmão `the_straddle_is_excluded_because_it_would_miss` mede o que
    // aconteceria se alguém a ignorasse, para a exclusão ter um número em vez de uma frase.
    for geom in [
        Geom::Disc,
        Geom::Ellipse,
        Geom::CapsuleBand,
        Geom::CapsuleCap,
    ] {
        // O `minor` NÃO é derivado aqui: quem o sabe é o footprint do `probe`, e é de lá que a mensagem
        // de falha o lê. Uma cópia local seria a segunda resposta a *"quão achatado é este bico?"*.
        for falloff in SMOOTH_FAMILY {
            // ⚠️ O **20 está aqui de propósito e a regra correta o REJEITA** (efetivo 20 < 40, e 9 na
            // elipse). Sem ele, uma mutação que apaga o `minor` da regra admite a elipse a r=40
            // (efetivo 18, que erra só 0,30 nível — sob a barra) e **sobrevive ao gate**: aconteceu. O
            // raio que separa as duas regras é o que a regra correta recusa e a mutante aceita.
            for radius in [20.0f32, 40.0, 100.0, 200.0] {
                // A PORTA do produto decide, não o gate: se as duas discordarem, é a porta que manda.
                let mut probe = spec(falloff, radius);
                if matches!(geom, Geom::Ellipse | Geom::Sliver) {
                    probe.dab_flatten = if matches!(geom, Geom::Sliver) {
                        0.8
                    } else {
                        0.45
                    };
                    probe.dab_angle_deg = 31;
                }
                if !FilmLut::admissible(&probe, radius) {
                    continue;
                }
                let (worst, differing, band, toe) = sweep_lut(falloff, radius, geom);
                assert!(
                    band > 20,
                    "controle: {geom:?}/{falloff:?} r={radius} varreu {band} texels — poucos para o \
                     gate significar algo"
                );
                checked += 1;
                worst_overall = worst_overall.max(worst);
                assert!(
                    worst <= WORST_BAR,
                    "{geom:?}/{falloff:?} r={radius} (efetivo {}): pior erro {:.2} nivel > barra 0,50",
                    radius * probe.dab_footprint([1.0, 0.0]).minor_fraction(),
                    worst * 255.0
                );
                assert_eq!(
                    differing, 0,
                    "{geom:?}/{falloff:?} r={radius}: {differing} de {band} texels movem um nivel — a \
                     barra de MAGNITUDE sozinha nao basta, e esta e a metade da CONTAGEM"
                );
                assert!(
                    toe * 100 <= band.max(100),
                    "{geom:?}/{falloff:?} r={radius}: {toe} de {band} cruzaram o corte do toe"
                );
            }
        }
    }
    assert!(
        checked >= 25,
        "controle: o gate cobriu {checked} combinacoes — poucas para a regra significar algo"
    );
    println!(
        "[lut-gate] {checked} combinacoes, pior erro {:.3} nivel de u8",
        worst_overall * 255.0
    );
}

/// **A BANDA da cápsula é EXATA, não aproximada** — a afirmação geométrica pinada como aritmética.
///
/// `P = I − uuᵀ` projeta num espaço 1-D (em 2D), então `e ∥ w` e `|e|² − (ŵ·e)² = 0`. Se alguém
/// "generalizar" a base para não projetar, este gate acende — e é o único que distingue *"a cápsula
/// funciona"* de *"a cápsula funciona porque o erro é pequeno"*.
#[test]
fn the_capsule_band_is_exact_not_merely_close() {
    for falloff in [Falloff::Smooth, Falloff::Sphere, Falloff::Pow4] {
        for radius in [40.0f32, 100.0] {
            let (worst, _, band, _) = sweep_lut(falloff, radius, Geom::CapsuleBand);
            assert!(
                band > 20,
                "controle: {falloff:?} r={radius} varreu {band} texels"
            );
            assert!(
                worst * 255.0 < 0.05,
                "a banda da capsula tem de ser EXATA: {falloff:?} r={radius} errou {:.4} nivel",
                worst * 255.0
            );
        }
    }
}

/// **A LUT É a curva, amostrada** — não uma segunda resposta a *"qual é o filme neste `t`"*.
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

/// **O STRADDLE é excluído porque ERRARIA — e este é o número.**
///
/// A cláusula 3 da [`FilmLut::admissible`] não é cautela: nos texels cuja grade 3×3 cruza a fronteira
/// calota↔banda da cápsula, o `B` correto muda no meio da grade e **nenhuma base única serve**. O gate
/// mede o que a LUT faria ali se o chamador ignorasse a cláusula — e exige que seja **pior que a barra**,
/// porque uma exclusão que não custa nada é uma exclusão que alguém vai remover.
///
/// ⚠️ Ele também mede o TAMANHO da faixa: ~2 linhas de texel por calota. É isso que torna o caminho
/// exato ali barato, e é o que faz da cláusula uma decisão e não um remendo.
#[test]
fn the_straddle_is_excluded_because_it_would_miss() {
    let (worst, _, band, _) = sweep_lut(Falloff::Pow4, 40.0, Geom::CapsuleStraddle);
    let (_, _, band_interior, _) = sweep_lut(Falloff::Pow4, 40.0, Geom::CapsuleBand);
    let (_, _, cap_interior, _) = sweep_lut(Falloff::Pow4, 40.0, Geom::CapsuleCap);
    assert!(band > 10, "controle: a faixa de straddle tem {band} texels");
    assert!(
        worst * 255.0 > 0.5,
        "a exclusao do straddle tem de CUSTAR: ele erra {:.2} nivel, e se isso estivesse sob a barra a \
         clausula 3 seria remendo em vez de decisao",
        worst * 255.0
    );
    // E a faixa e FINA — o caminho exato ali e barato.
    assert!(
        band * 8 < band_interior + cap_interior,
        "a faixa de straddle ({band}) tem de ser fina contra o interior ({}) — se nao for, devolver ela \
         ao caminho exato deixa de ser barato",
        band_interior + cap_interior
    );
    println!(
        "[straddle] {band} texels, pior erro {:.2} nivel (interior: {} texels)",
        worst * 255.0,
        band_interior + cap_interior
    );
}

/// **A porta RECUSA o que tem de recusar** — as três cláusulas afirmadas na direção da negativa.
///
/// Um gate que só varre o regime admissível nunca pergunta se a porta **fecha**, e uma mutação que
/// apaga a cláusula do falloff suave sobrevive a ele inteiro. Aconteceu; este é o gate que faltava.
#[test]
fn the_admissibility_door_refuses_what_it_must() {
    // `Constant`: errático E mais lento — os dois motivos medidos.
    assert!(
        !FilmLut::admissible(&spec(Falloff::Constant, 100.0), 100.0),
        "Constant tem de ser recusado: o degrau interage com a grade (13,25 nivel a r=100) e a LUT ainda \
         e mais LENTA que a curva dele (0,46x)"
    );
    // `Custom`: a banda é o dab inteiro e a tabela seria indexada por uma curva do documento.
    assert!(
        !FilmLut::admissible(&spec(Falloff::Custom, 100.0), 100.0),
        "Custom tem de ser recusado: a for_dab toma o dab inteiro como banda para ele"
    );
    // `hardness >= 1` torna QUALQUER falloff um degrau — recai no caso Constant.
    let mut hard = spec(Falloff::Smooth, 100.0);
    hard.hardness = 1.0;
    assert!(
        !FilmLut::admissible(&hard, 100.0),
        "hardness >= 1 faz o falloff_weight devolver 1 ou 0 — e um degrau, e sai pela porta do Constant"
    );
    // O raio EFETIVO, não o raio: um bico muito achatado é recusado mesmo grande.
    let mut sliver = spec(Falloff::Smooth, 100.0);
    sliver.dab_flatten = 0.8; // minor 0,2 ⇒ efetivo 20 < 40
    assert!(
        !FilmLut::admissible(&sliver, 100.0),
        "raio 100 com minor 0,2 da efetivo 20 — a curvatura e do MENOR raio local, nao do raio"
    );
    // E o caso admissível de controle, senão o gate poderia recusar tudo e passar.
    assert!(
        FilmLut::admissible(&spec(Falloff::Smooth, 100.0), 100.0),
        "controle: um Smooth redondo de raio 100 TEM de ser admissivel, senao este gate nao diz nada"
    );
}
