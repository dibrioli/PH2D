use super::*;

const L: usize = 120;

fn campo() -> Vec<u8> {
    let mut p = vec![0u8; L * L * 4];
    for y in 0..L {
        for x in 0..L {
            let c: [u8; 4] = if (40..42).contains(&x) {
                [90, 30, 120, 255] // a linha roxa
            } else if (70..73).contains(&x) {
                [235, 170, 170, 255] // a faixa levantada
            } else if x >= 90 && y % 6 < 2 {
                [175, 25, 25, 255] // as listras das cerdas
            } else {
                [200, 30, 30, 255]
            };
            p[(y * L + x) * 4..(y * L + x) * 4 + 4].copy_from_slice(&c);
        }
    }
    p
}

/// O premultiplicado de cada pixel da linha `y = 60` depois do Smooth Edges no máximo.
fn depois(linhas: Option<bool>) -> Vec<[f32; 4]> {
    let p = campo();
    let sampler = SampleU::new(1, L, L);
    let s = Suave::nova(
        sampler,
        (&p, L * 4),
        (L, L),
        (0, 0, L, L),
        RAIO_MAX,
        linhas,
        true,
    );
    (0..L)
        .map(|x| {
            let dele = premul_da_celula(&p, (60 * L + x) * 4);
            let (c, a) = s.aplica(x, 60, dele);
            [c[0], c[1], c[2], a]
        })
        .collect()
}

fn contraste(v: &[[f32; 4]], x: usize, fundo: usize) -> f32 {
    (0..3)
        .map(|c| (v[x][c] - v[fundo][c]).abs())
        .fold(0.0, f32::max)
}

#[test]
fn a_linha_e_a_faixa_da_gravidade_amaciam_e_as_cerdas_nao() {
    let antes: Vec<[f32; 4]> = {
        let p = campo();
        (0..L)
            .map(|x| {
                let (c, a) = premul_da_celula(&p, (60 * L + x) * 4);
                [c[0], c[1], c[2], a]
            })
            .collect()
    };
    let baixo = depois(Some(true));
    let direita = depois(None);
    let (l0, l1) = (contraste(&antes, 40, 20), contraste(&baixo, 40, 20));
    let (f0, f1) = (contraste(&antes, 71, 20), contraste(&baixo, 71, 20));
    eprintln!("[linhas] a linha roxa {l0:.1} → {l1:.1} · a faixa clara {f0:.1} → {f1:.1}");
    assert!(
        l1 <= 0.6 * l0,
        "a linha roxa não amaciou ({l0:.1} → {l1:.1})"
    );
    assert!(
        f1 <= 0.6 * f0,
        "a faixa levantada não amaciou ({f0:.1} → {f1:.1})"
    );
    // As listras horizontais (x ≥ 90, longe da faixa): nenhum pixel muda mais de 1 %.
    for x in 100..L - 8 {
        for c in 0..4 {
            let (a, b) = (antes[x][c], baixo[x][c]);
            assert!(
                (a - b).abs() <= 0.01 * a.max(1.0),
                "as cerdas mudaram em x {x}: {a} → {b}"
            );
        }
    }
    // Com a tela direita não se procuram linhas: a linha e a faixa ficam.
    assert!(
        contraste(&direita, 40, 20) >= 0.95 * l0,
        "com a tela direita a linha mudou"
    );
}
