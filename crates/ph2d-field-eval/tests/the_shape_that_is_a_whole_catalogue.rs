//! ⭐⭐⭐ **A SUPERFÓRMULA DE GIELIS (W128)** — a maior razão forma/linha-de-código do levantamento.
//!
//! | gate | o defeito que ele apanha |
//! |---|---|
//! | `the_neutral_point_is_the_sphere` | a fórmula ter deixado de degenerar |
//! | `the_symmetry_makes_exactly_that_many_lobes` | a simetria ter virado enfeite |
//! | `the_piece_never_leaves_the_box_it_was_given` | a **normalização** cair — e aí um expoente muda o TAMANHO |
//! | `the_divisor_holds_across_the_family` | o **divisor** ficar curto (a peça rasga) — **e a costura do `atan2`**, medido por mutação |
//! | `the_two_curves_are_not_the_same_axis` | as duas curvas descreverem a mesma coisa |

use ph2d_field::{
    FieldDoc, MAX_SUPERFORMULA_N, MAX_SUPERFORMULA_N1, MAX_SUPERFORMULA_SYMMETRY,
    MIN_SUPERFORMULA_N, MIN_SUPERFORMULA_N1, Node, NodeId, NodeKind, Primitive, Xform,
};
use ph2d_field_eval::Field;

#[allow(clippy::too_many_arguments)]
fn peca(
    half: [f32; 3],
    tm: f32,
    t1: f32,
    t2: f32,
    t3: f32,
    sm: f32,
    s1: f32,
    s2: f32,
    s3: f32,
) -> Primitive {
    Primitive::Superformula {
        half,
        top_symmetry: tm,
        top_n1: t1,
        top_n2: t2,
        top_n3: t3,
        side_symmetry: sm,
        side_n1: s1,
        side_n2: s2,
        side_n3: s3,
    }
}

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

/// O raio da superfície na direcção dada, por bissecção.
fn raio(f: &Field, dir: [f64; 3]) -> f64 {
    let n = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let u = [dir[0] / n, dir[1] / n, dir[2] / n];
    let (mut lo, mut hi) = (0.0_f64, 6.0_f64);
    for _ in 0..70 {
        let m = 0.5 * (lo + hi);
        if f.at(u[0] * m, u[1] * m, u[2] * m) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    0.5 * (lo + hi)
}

/// ⭐⭐⭐ **Quantos lobos a silhueta tem — contados com PROEMINÊNCIA.**
///
/// ⛔⛔ **A 1.ª redacção contava todo máximo local e leu `237` lobos numa silhueta LISA**: um raio
/// quase constante oscila no último dígito, e cada oscilação vira um «pico». *Uma régua de contagem
/// sem limiar mede o ruído do instrumento*, e aqui ela mediria `n/2` para qualquer forma redonda.
///
/// A cura são duas metades: uma silhueta cuja excursão é `< 2 %` do raio médio **não tem lobos**, e
/// entre as que têm, só conta o máximo que passa do meio da excursão.
fn lobos(f: &Field, no_plano: bool) -> usize {
    let n = 2048;
    let r: Vec<f64> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            if no_plano {
                raio(f, [a.cos(), 0.0, a.sin()])
            } else {
                raio(f, [a.cos(), a.sin(), 0.0])
            }
        })
        .collect();
    let lo = r.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = r.iter().copied().fold(0.0_f64, f64::max);
    let media = r.iter().sum::<f64>() / r.len() as f64;
    if (hi - lo) / media < 0.02 {
        return 0;
    }
    let meio = 0.5 * (lo + hi);
    (0..n)
        .filter(|&i| r[i] > meio && r[i] > r[(i + n - 1) % n] && r[i] >= r[(i + 1) % n])
        .count()
}

/// ⭐⭐ **No ponto NEUTRO ela é a esfera, ao bit da regularização.**
///
/// Com `m = 4` e os três expoentes a `2`, `A = cos²+sin² = 1` **identicamente** ⇒ `r = 1` ⇒ as duas
/// medidas viram a norma euclidiana. ⚠️ *Uma fórmula geral que não colapsa na forma simples tem um
/// erro que nenhuma varredura de gradiente encontra* — foi assim que o divisor da superquadrática
/// se conferiu, e é assim aqui.
#[test]
fn the_neutral_point_is_the_sphere() {
    let r = f64::from(0.35_f32);
    let f = campo(peca([0.35; 3], 4.0, 2.0, 2.0, 2.0, 4.0, 2.0, 2.0, 2.0));
    let mut pior = 0.0_f64;
    for i in 0..22 {
        for j in 0..22 {
            for k in 0..22 {
                let at = |t: usize| -0.9 + 1.8 * (t as f64 + 0.5) / 22.0;
                let (x, y, z) = (at(i), at(j), at(k));
                let verdade = (x * x + y * y + z * z).sqrt() - r;
                pior = pior.max((f.at(x, y, z) - verdade).abs());
            }
        }
    }
    assert!(
        pior < 1.0e-9,
        "no ponto neutro ela tinha de ser a esfera EXACTA — pior erro {pior:.3e}"
    );
}

/// ⭐⭐⭐ **A simetria faz EXACTAMENTE aquele número de lobos.**
///
/// ⚠️ **E NÃO é este gate que apanha a costura** — medido por mutação: tirar o deslocamento de meia
/// volta do argumento deixa este verde e reprova o `the_divisor_holds_across_the_family`. *O vinco
/// da costura é uma descontinuidade da DERIVADA, e a contagem de máximos do raio não a vê.* Vale
/// registá-lo: uma leitura rápida esperaria o contrário.
#[test]
fn the_symmetry_makes_exactly_that_many_lobes() {
    for m in [1_u32, 2, 3, 5, 7, 8, 12, MAX_SUPERFORMULA_SYMMETRY] {
        #[allow(clippy::cast_precision_loss)]
        let f = campo(peca([0.35; 3], m as f32, 1.0, 1.7, 1.7, 4.0, 2.0, 2.0, 2.0));
        let n = lobos(&f, true);
        assert_eq!(
            n, m as usize,
            "simetria {m} tinha de dar {m} lobos e deu {n}"
        );
    }
}

/// ⭐⭐⭐ **A PEÇA NUNCA SAI DA CAIXA QUE LHE DERAM — e encosta nela.**
///
/// ⛔⛔ **Sem a normalização das curvas o controlo mente:** `r = A^(−1/n1)` chega a `8`, e uma peça de
/// meia-medida `0,35` media `2,8`. *Mexer num EXPOENTE mudava o TAMANHO*, e a caixa envolvente do
/// documento passava a descrever outra coisa — o que estraga o recorte, o gizmo e a exportação.
#[test]
fn the_piece_never_leaves_the_box_it_was_given() {
    let h = [0.40_f32, 0.22, 0.34];
    let mut maior_sobra = 0.0_f64;
    let mut menor_encosto = 1.0_f64;
    for (tm, t1, t2, t3) in [
        (5.0_f32, 0.6_f32, 1.7_f32, 1.3_f32),
        (3.0, 4.0, MAX_SUPERFORMULA_N, MAX_SUPERFORMULA_N),
        (7.0, MIN_SUPERFORMULA_N1, 1.0, 3.0),
        (1.0, MAX_SUPERFORMULA_N1, 1.0, 1.0),
        (4.0, 2.0, 2.0, 2.0),
    ] {
        let f = campo(peca(h, tm, t1, t2, t3, 4.0, 2.0, 2.0, 2.0));
        // O maior raio no plano de cima, em fracção da meia-medida daquele eixo.
        let mut maior = 0.0_f64;
        for i in 0..720 {
            let a = std::f64::consts::TAU * f64::from(i) / 720.0;
            let (c, s) = (a.cos(), a.sin());
            // A meia-medida NA DIRECÇÃO `a` da elipse da caixa.
            let caixa =
                1.0 / ((c / f64::from(h[0])).powi(2) + (s / f64::from(h[2])).powi(2)).sqrt();
            maior = maior.max(raio(&f, [c, 0.0, s]) / caixa);
        }
        maior_sobra = maior_sobra.max(maior);
        menor_encosto = menor_encosto.min(maior);
    }
    assert!(
        maior_sobra <= 1.001,
        "a peça saiu da caixa: chegou a {maior_sobra:.4} da meia-medida — a normalização caiu"
    );
    assert!(
        menor_encosto > 0.98,
        "a peça encolheu dentro da caixa (só {menor_encosto:.4} dela) — o tamanho deixou de ser o \
         que o painel diz"
    );
}

/// ⭐⭐⭐ **O DIVISOR SEGURA A FAMÍLIA INTEIRA** — o gate que impede a peça de rasgar.
///
/// ⚠️ **Ele mede o campo JÁ DIVIDIDO**, e não recalcula a conta fechada: um oráculo feito da função
/// sob teste não prova nada. ⚠️ E varre as **esquinas da caixa de cercas**, não só o meio — foi uma
/// varredura de esquinas que apanhou o divisor `69 %` curto na construção.
#[test]
fn the_divisor_holds_across_the_family() {
    let mut pior = 0.0_f64;
    let mut onde = String::new();
    #[allow(clippy::cast_precision_loss)]
    let teto = MAX_SUPERFORMULA_SYMMETRY as f32;
    for half in [[0.35_f32; 3], [0.42, 0.20, 0.30]] {
        for (tm, t1, t2, t3) in [
            (5.0_f32, 0.6_f32, 1.7_f32, 1.3_f32),
            (
                teto,
                MIN_SUPERFORMULA_N1,
                MAX_SUPERFORMULA_N,
                MAX_SUPERFORMULA_N,
            ),
            (
                teto,
                MAX_SUPERFORMULA_N1,
                MIN_SUPERFORMULA_N,
                MIN_SUPERFORMULA_N,
            ),
            (
                1.0,
                MIN_SUPERFORMULA_N1,
                MIN_SUPERFORMULA_N,
                MAX_SUPERFORMULA_N,
            ),
            (3.0, 1.0, MAX_SUPERFORMULA_N, MIN_SUPERFORMULA_N),
        ] {
            for (sm, s1, s2, s3) in [
                (4.0_f32, 2.0_f32, 2.0_f32, 2.0_f32),
                (
                    2.0,
                    MIN_SUPERFORMULA_N1,
                    MAX_SUPERFORMULA_N,
                    MIN_SUPERFORMULA_N,
                ),
                (teto, 1.0, MIN_SUPERFORMULA_N, MAX_SUPERFORMULA_N),
            ] {
                let f = campo(peca(half, tm, t1, t2, t3, sm, s1, s2, s3));
                let e = f64::from(half[0].max(half[1]).max(half[2])) * 2.2;
                let n = 30;
                let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n as f64;
                let mut aqui = 0.0_f64;
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            let g = f.gradient_norm(at(i), at(j), at(k), 1.0e-5);
                            if g.is_finite() && g > aqui {
                                aqui = g;
                            }
                        }
                    }
                }
                if aqui > pior {
                    pior = aqui;
                    onde = format!(
                        "half {half:?} cima ({tm},{t1},{t2},{t3}) lado ({sm},{s1},{s2},{s3})"
                    );
                }
            }
        }
    }
    assert!(
        pior <= 1.02,
        "o divisor não segura: ‖∇f‖ = {pior:.4} em {onde} — a marcha atravessa a superfície"
    );
}

/// ⭐⭐ **AS DUAS CURVAS SÃO EIXOS DIFERENTES** — a de cima desenha a planta, a de lado o perfil.
#[test]
fn the_two_curves_are_not_the_same_axis() {
    // ⛔⛔ **DUAS réguas erradas antes desta, e as duas pela mesma razão: no produto esférico a
    // DIRECÇÃO que se sonda não é o PARÂMETRO da curva.**
    //
    // 1. o corte `X–Y` atravessa os lobos do PLANO, logo acusava lobos num perfil liso;
    // 2. o raio contra a elevação num azimute fixo é a curva do perfil **esticada** por `r₁(θ)` —
    //    com um perfil neutro ele desenha uma ELIPSE, e a excursão dela lia `0,104`.
    //
    // ⭐ A régua que separa é o **desvio da elipse**: ajusta-se a elipse pelos dois semi-eixos
    // medidos e pergunta-se quanto o resto se afasta dela. Um perfil neutro é a elipse; um perfil
    // com lobos não é.
    let desvio_da_elipse = |f: &Field| {
        let a = raio(f, [1.0, 0.0, 0.0]);
        let b = raio(f, [0.0, 1.0, 0.0]);
        let n = 512;
        (0..n)
            .map(|i| {
                let psi = std::f64::consts::PI * (i as f64 + 0.5) / n as f64
                    - std::f64::consts::FRAC_PI_2;
                let (sp, cp) = psi.sin_cos();
                let elipse = 1.0 / ((cp / a).powi(2) + (sp / b).powi(2)).sqrt();
                (raio(f, [cp, sp, 0.0]) - elipse).abs() / elipse
            })
            .fold(0.0_f64, f64::max)
    };
    // Cinco lobos em cima, perfil no ponto neutro.
    let f = campo(peca([0.35; 3], 5.0, 1.0, 1.7, 1.7, 4.0, 2.0, 2.0, 2.0));
    assert_eq!(lobos(&f, true), 5, "de CIMA tinham de ser 5 lobos");
    let d = desvio_da_elipse(&f);
    assert!(
        d < 0.01,
        "o PERFIL tinha de ser a elipse e desviou {d:.4} — as curvas trocaram de eixo"
    );
    // E com as duas trocadas, o contrário.
    let g = campo(peca([0.35; 3], 4.0, 2.0, 2.0, 2.0, 5.0, 1.0, 1.7, 1.7));
    assert_eq!(
        lobos(&g, true),
        0,
        "trocando as curvas, a PLANTA tinha de ficar um círculo"
    );
    // ⚠️ **A barra sai da SEPARAÇÃO medida, e não de um número redondo**: o perfil neutro desvia
    // `< 0,01` e o de cinco lobos desvia `0,0959` — uma década entre eles, e a barra ao meio.
    let d = desvio_da_elipse(&g);
    assert!(
        d > 0.05,
        "e o PERFIL tinha de deixar de ser a elipse — desviou só {d:.4}"
    );
}
