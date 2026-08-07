//! **A varredura por FAIXAS entrega os MESMOS contornos que o flood pixel-a-pixel.**
//!
//! ⚠️ O oráculo é a [`trace_all_contours_flood`] — o corpo que shipava até 2026-08-06, congelado sob
//! `cfg(test)`. As duas rotas comparadas aqui são, literalmente, *a de antes* e *a de agora*, e a wave
//! inteira se apoia nisto: ela é uma reescrita de caminho quente, não uma mudança de desenho.

use super::selection_trace::{trace_all_contours, trace_all_contours_flood};

/// Uma máscara `w × h` desenhada por um predicado — a forma de escrever fixture que contém o
/// fenômeno em vez de uma que seja cômoda.
fn mask(w: usize, h: usize, f: impl Fn(i32, i32) -> bool) -> Vec<u8> {
    let mut m = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            if f(x as i32, y as i32) {
                m[y * w + x] = 255;
            }
        }
    }
    m
}

fn disc(cx: i32, cy: i32, r: i32) -> impl Fn(i32, i32) -> bool {
    move |x, y| (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r
}

/// As duas rotas devolvem os mesmos contornos, na mesma ordem, ponto a ponto.
fn same(m: &[u8], w: usize, h: usize, case: &str) {
    let a = trace_all_contours(m, w, h);
    let b = trace_all_contours_flood(m, w, h);
    assert_eq!(
        a.len(),
        b.len(),
        "contagem de contornos difere ({case}): {} contra {}",
        a.len(),
        b.len()
    );
    for (i, (ca, cb)) in a.iter().zip(&b).enumerate() {
        assert_eq!(
            ca.len(),
            cb.len(),
            "o contorno {i} tem {} pontos contra {} ({case})",
            ca.len(),
            cb.len()
        );
        for (j, (pa, pb)) in ca.iter().zip(cb).enumerate() {
            assert!(
                (pa[0] - pb[0]).abs() < 1e-6 && (pa[1] - pb[1]).abs() < 1e-6,
                "o ponto {j} do contorno {i} andou ({case}): {pa:?} contra {pb:?}"
            );
        }
    }
}

/// **A wave inteira se apoia nisto:** trocar o flood por faixas não move um ponto do contorno.
///
/// ⚠️ **A fixture TEM de conter os casos que separam as duas rotas**, e são três: o blob que ENCOSTA
/// na borda da janela (onde `sy == 0` faz o `wrapping_sub` do vizinho de cima estourar), os blobs que
/// se tocam **só na DIAGONAL** (4-conexo diz dois, 8-conexo diria um — é a conectividade que o flood
/// define e a varredura por faixas tem de reproduzir), e a forma **CÔNCAVA**, cuja linha tem mais de
/// uma corrida e por isso obriga a semeadura por-corrida a estar certa.
#[test]
fn the_span_scan_traces_what_the_pixel_flood_traced() {
    let cases: Vec<(&str, usize, usize, Vec<u8>)> = vec![
        ("vazia", 32, 32, mask(32, 32, |_, _| false)),
        ("um disco no meio", 64, 64, mask(64, 64, disc(32, 32, 20))),
        (
            "dois discos DISJUNTOS",
            96,
            48,
            mask(96, 48, |x, y| {
                disc(20, 24, 12)(x, y) || disc(70, 24, 12)(x, y)
            }),
        ),
        (
            "dois quadrados que se tocam so na DIAGONAL",
            32,
            32,
            mask(32, 32, |x, y| {
                (2..=10).contains(&x) && (2..=10).contains(&y)
                    || (11..=19).contains(&x) && (11..=19).contains(&y)
            }),
        ),
        (
            "blob ENCOSTADO na borda de cima",
            48,
            48,
            mask(48, 48, |x, y| {
                (5..=30).contains(&x) && (0..=20).contains(&y)
            }),
        ),
        (
            "blob ENCOSTADO nas quatro bordas",
            24,
            24,
            mask(24, 24, |_, _| true),
        ),
        (
            "anel (com BURACO)",
            64,
            64,
            mask(64, 64, |x, y| {
                disc(32, 32, 25)(x, y) && !disc(32, 32, 12)(x, y)
            }),
        ),
        (
            "CONCAVA em U — varias corridas por linha",
            48,
            48,
            mask(48, 48, |x, y| {
                (6..=41).contains(&y)
                    && ((6..=15).contains(&x) || (32..=41).contains(&x) || (32..=41).contains(&y))
            }),
        ),
        (
            "listras — muitos componentes",
            48,
            48,
            mask(48, 48, |x, y| (4..=43).contains(&x) && y % 6 < 3),
        ),
        (
            "um PIXEL solto",
            16,
            16,
            mask(16, 16, |x, y| x == 8 && y == 8),
        ),
    ];
    for (name, w, h, m) in cases {
        same(&m, w, h, name);
    }
}

/// **Nenhuma célula sobra marcada no buffer reusado.**
///
/// ⚠️ O `comp` deixou de ser alocado por componente e passou a ser limpo pelas FAIXAS do componente
/// que acabou de sair. Se essa limpeza esquecer uma célula, o componente SEGUINTE herda tinta do
/// anterior e o contorno dele engorda — e isso só aparece a partir do segundo blob, que é por que a
/// fixture tem três em fila.
#[test]
fn a_reused_buffer_does_not_leak_the_previous_component() {
    let (w, h) = (128usize, 40usize);
    let m = mask(w, h, |x, y| {
        disc(20, 20, 14)(x, y) || disc(60, 20, 14)(x, y) || disc(100, 20, 14)(x, y)
    });
    let a = trace_all_contours(&m, w, h);
    assert_eq!(a.len(), 3, "a fixture tem de produzir TRES contornos");
    same(&m, w, h, "tres discos em fila");
    // E os três têm de ter o mesmo tamanho: são o mesmo disco transladado.
    let n0 = a[0].len();
    for (i, c) in a.iter().enumerate() {
        assert!(
            c.len().abs_diff(n0) <= 2,
            "o contorno {i} tem {} pontos contra {n0} do primeiro — o buffer vazou",
            c.len()
        );
    }
}

/// **O A/B das duas rotas, na MESMA corrida** — a única forma honesta de medir numa máquina que é
/// compartilhada com outras linhas.
///
/// ⚠️ Comparar duas CORRIDAS aqui atribuiria a deriva da máquina ao ganho: nesta sessão a coluna
/// `rasteriza` do composite, que ninguém tocou, foi de 1,671 a 3,782 ms entre duas corridas só porque
/// o `load average` subiu para 30. Cronometrando as duas rotas dentro da mesma corrida, sobre a mesma
/// máscara, a carga vira fator comum e a RAZÃO sobrevive.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_span_scan_against_the_pixel_flood() {
    println!(
        "[trace] varredura por FAIXAS x flood pixel-a-pixel — a mesma mascara, a mesma corrida"
    );
    println!(
        "{:>22} {:>10}  {:>9} {:>9}  {:>8}",
        "cena", "celulas", "flood", "faixas", "razao"
    );
    // A janela do composite de uma elipse de 200 px a SS=3 (o caso do `measure_the_boolean_of_shapes`).
    let cases: Vec<(&str, usize, usize, Vec<u8>)> = vec![
        (
            "1 disco (SS=3, r=200)",
            1200,
            1200,
            mask(1200, 1200, disc(600, 600, 600)),
        ),
        (
            "3 discos",
            1800,
            1200,
            mask(1800, 1200, |x, y| {
                disc(300, 600, 290)(x, y) || disc(900, 600, 290)(x, y) || disc(1500, 600, 290)(x, y)
            }),
        ),
        (
            "anel (com buraco)",
            1200,
            1200,
            mask(1200, 1200, |x, y| {
                disc(600, 600, 590)(x, y) && !disc(600, 600, 300)(x, y)
            }),
        ),
    ];
    for (name, w, h, m) in cases {
        let (mut a, mut b) = (f64::MAX, f64::MAX);
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let ra = trace_all_contours_flood(&m, w, h);
            a = a.min(t0.elapsed().as_secs_f64() * 1e3);
            let t1 = std::time::Instant::now();
            let rb = trace_all_contours(&m, w, h);
            b = b.min(t1.elapsed().as_secs_f64() * 1e3);
            assert_eq!(ra.len(), rb.len(), "as duas rotas discordam em {name}");
        }
        println!(
            "{name:>22} {:>10}  {a:>9.3} {b:>9.3}  {:>7.2}x",
            w * h,
            a / b.max(1e-9)
        );
    }
}
