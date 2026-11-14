//! B26.4 (missão 4K, N10): a [`FilmLut::new`] em paralelo contra o laço em série de antes, VERBATIM.

use super::FilmLut;
use crate::height_film::film_of;
use crate::{BrushSpec, Falloff};

/// O ORÁCULO: o corpo da `FilmLut::new` antes de B26.4, copiado sem mudar uma vírgula.
fn tabela_em_serie(spec: &BrushSpec) -> Vec<f32> {
    let n = FilmLut::N;
    #[expect(clippy::cast_precision_loss, reason = "N = 16384 é exato em f32")]
    let inv = 1.0 / (n as f32);
    let mut table = Vec::with_capacity(n + 1);
    for i in 0..=n {
        #[expect(clippy::cast_precision_loss, reason = "i <= 16384, exato em f32")]
        let t = (i as f32) * inv;
        table.push(film_of(spec.falloff_weight(t)));
    }
    table
}

fn spec(falloff: Falloff, hardness: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 100.0,
        falloff,
        hardness,
        impasto: true,
        impasto_depth: 0.5,
        impasto_smooth_edges: true,
        ..Default::default()
    }
}

/// Toda queda × durezas da borda (0, a do meio, as da mão, 1): os `N + 1` bits iguais, e na ORDEM do índice.
#[test]
fn a_lut_em_paralelo_e_a_serie_ao_bit() {
    let durezas = [0.0, 0.05, 0.3, 0.5, 0.731_25, 0.9, 0.999, 1.0];
    let mut comparadas = 0usize;
    for &f in &Falloff::ALL {
        for &h in &durezas {
            let s = spec(f, h);
            let lut = FilmLut::new(&s);
            let oraculo = tabela_em_serie(&s);
            assert_eq!(lut.table.len(), oraculo.len(), "{f:?} h={h}: tamanho");
            for (i, (a, b)) in lut.table.iter().zip(&oraculo).enumerate() {
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "{f:?} h={h}: a amostra {i} da tabela em paralelo não é a da série"
                );
            }
            comparadas += 1;
        }
    }
    // O controlo: a régua viu tabelas que DIFEREM entre si (senão uma tabela constante passaria).
    let a = FilmLut::new(&spec(Falloff::Smooth, 0.0));
    let b = FilmLut::new(&spec(Falloff::Smooth, 0.9));
    assert!(a.table.iter().zip(&b.table).any(|(x, y)| x != y));
    assert_eq!(comparadas, Falloff::ALL.len() * durezas.len());
}

/// A·B (perfil smoke): o custo de UMA reconstrução da tabela — o que a dureza da mão paga por dab no Impasto com
/// Smooth Edges. Intercalada, mínimo de 41. `cargo test -p ph2d-painter-brush --profile smoke --lib
/// mede_b26_4_a_lut_por_dab -- --ignored --nocapture`.
#[test]
#[ignore = "medição B26.4: corre à mão, perfil smoke"]
fn mede_b26_4_a_lut_por_dab() {
    use std::time::Instant;
    let s = spec(Falloff::Smooth, 0.5);
    let (mut serie, mut par) = (f64::MAX, f64::MAX);
    for _ in 0..41 {
        let t = Instant::now();
        std::hint::black_box(tabela_em_serie(std::hint::black_box(&s)));
        serie = serie.min(t.elapsed().as_secs_f64() * 1e3);
        let t = Instant::now();
        std::hint::black_box(FilmLut::new(std::hint::black_box(&s)));
        par = par.min(t.elapsed().as_secs_f64() * 1e3);
    }
    println!("B26.4 FilmLut::new — série {serie:.4} ms · paralelo {par:.4} ms (mínimo de 41)");
}
