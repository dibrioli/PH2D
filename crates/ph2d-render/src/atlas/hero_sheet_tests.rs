use super::*;

fn sheet() -> (Vec<SrgbRgba>, usize) {
    (
        hero_sheet_pixels(),
        (HERO_SHEET_CELL_PX * HERO_SHEET_COLS) as usize,
    )
}

fn cell_alpha(px: &[SrgbRgba], w: usize, row: u32, col: u32) -> Vec<u8> {
    let n = HERO_SHEET_CELL_PX as usize;
    (0..n * n)
        .map(|k| px[(row as usize * n + k / n) * w + col as usize * n + k % n].a())
        .collect()
}

/// A folha é a mesma em toda corrida — grava-se pela chave e reabre igual.
#[test]
fn a_folha_e_deterministica() {
    assert_eq!(hero_sheet_pixels(), hero_sheet_pixels());
    let (px, w) = sheet();
    assert_eq!(
        px.len(),
        w * (HERO_SHEET_CELL_PX * HERO_SHEET_ROWS) as usize
    );
}

/// Cada célula tem margem transparente de 2 texels: o filtro bilinear não traz a vizinha.
#[test]
fn cada_celula_tem_margem_transparente() {
    let (px, w) = sheet();
    let n = HERO_SHEET_CELL_PX as usize;
    for row in 0..HERO_SHEET_ROWS {
        for col in 0..HERO_SHEET_COLS {
            let a = cell_alpha(&px, w, row, col);
            for (k, alfa) in a.iter().enumerate() {
                let (x, y) = (k % n, k / n);
                if x < 2 || y < 2 || x >= n - 2 || y >= n - 2 {
                    assert_eq!(
                        *alfa, 0,
                        "linha {row} célula {col} pinta a margem em ({x},{y})"
                    );
                }
            }
            assert!(
                a.iter().filter(|v| **v > 128).count() > 400,
                "linha {row} célula {col} vazia"
            );
        }
    }
}

/// As tags cobrem as quatro linhas, uma por linha, sem buracos.
#[test]
fn as_tags_sao_as_linhas() {
    for (i, (_, from, to, ms)) in HERO_SHEET_TAGS.iter().enumerate() {
        assert_eq!(*from, i as u32 * HERO_SHEET_COLS);
        assert_eq!(*to, *from + HERO_SHEET_COLS - 1);
        assert!(*ms > 0);
    }
}

/// Correr MEXE: duas células da linha não são iguais (uma animação parada mentiria no smoke).
#[test]
fn cada_animacao_mexe() {
    let (px, w) = sheet();
    for row in 0..HERO_SHEET_ROWS {
        let a0 = cell_alpha(&px, w, row, 0);
        let differ = (1..HERO_SHEET_COLS).any(|c| cell_alpha(&px, w, row, c) != a0);
        assert!(differ, "a linha {row} tem seis células iguais");
    }
}

/// A sola declarada é a do desenho: a linha mais baixa com cobertura, em todas as células, fica a menos
/// de um texel dela (o personagem paira a esta altura — um número errado põe-no a flutuar ou a afundar).
#[test]
fn a_sola_declarada_e_a_do_desenho() {
    let (px, w) = sheet();
    let n = HERO_SHEET_CELL_PX as usize;
    let sola = 0.5 * n as f32 + HERO_SHEET_SOLE_BELOW_CENTRE * n as f32;
    for row in [0, 1] {
        for col in 0..HERO_SHEET_COLS {
            let a = cell_alpha(&px, w, row, col);
            let baixo = (0..n)
                .rev()
                .find(|y| (0..n).any(|x| a[y * n + x] > 64))
                .expect("desenho");
            assert!(
                (baixo as f32 + 1.0 - sola).abs() <= 1.5,
                "linha {row} célula {col}: a sola desenhada está no texel {baixo}, a declarada em {sola}"
            );
        }
    }
}

/// `PH2D_HERO_SHEET_RGBA=<ficheiro>` grava a folha em RGBA cru (para a olhar; não é um gate).
#[test]
fn grava_a_folha_para_olhar() {
    if let Ok(path) = std::env::var("PH2D_HERO_SHEET_RGBA") {
        let bytes: Vec<u8> = hero_sheet_pixels()
            .iter()
            .flat_map(|p| [p.r(), p.g(), p.b(), p.a()])
            .collect();
        std::fs::write(path, bytes).expect("grava");
    }
}
