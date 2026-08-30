//! Os gates do salto do ladrilho (plano 33, W10).
//!
//! ⚠️ A fixtura é **assimétrica de propósito**: `5x3`, largura ímpar e não quadrada. Um ladrilho
//! quadrado deixaria passar uma troca de eixos, que é precisamente uma das mutações que estes gates
//! têm de matar.

use super::{SEAM_VISIBLE, Tile, tiles_cleanly, wrap_seam};

/// Constrói um ladrilho a partir de uma função `(x, y) -> RGBA`.
fn tile(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Tile {
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            rgba.extend_from_slice(&f(x, y));
        }
    }
    Tile {
        rgba,
        width: w,
        height: h,
        cells: [1, 1],
    }
}

/// ⭐ Um ladrilho **chapado** fecha por construção.
#[test]
fn a_flat_tile_closes_exactly() {
    assert_eq!(wrap_seam(&tile(5, 3, |_, _| [90, 120, 200, 255])), 0);
}

/// ⭐⭐⭐ **O caso do produto: um motivo assado de uma FORMA fecha, e é por isso que ele não tem
/// costura nenhuma na GPU.**
///
/// A caixa de uma forma é justa, então a cobertura vai a **zero** nos quatro lados. O RGB debaixo
/// dessa alfa é o do motivo (é o que o `bake` preserva de propósito) — em alfa reto as bordas
/// pareceriam contrastadas; em pré-multiplicado elas são o que são: **invisíveis**.
#[test]
fn a_motif_baked_from_a_shape_closes_because_its_edges_are_transparent() {
    let t = tile(5, 3, |x, y| {
        let dentro = x > 0 && x < 4 && y == 1;
        [220, 60, 40, u8::from(dentro) * 255]
    });
    assert_eq!(
        wrap_seam(&t),
        0,
        "as bordas sao TRANSPARENTES - um salto aqui significa que a medicao le RGB reto, e entao \
         todo motivo com bbox justa seria acusado de nao encaixar"
    );
}

/// ⚠️ **A prova directa da pré-multiplicação**, sem depender da forma do motivo: dois lados
/// **totalmente transparentes** com RGB oposto.
#[test]
fn two_transparent_edges_with_opposite_rgb_report_no_seam() {
    let t = tile(5, 3, |x, _| {
        if x == 0 {
            [0, 0, 0, 0]
        } else if x == 4 {
            [255, 255, 255, 0]
        } else {
            [128, 128, 128, 255]
        }
    });
    assert_eq!(
        wrap_seam(&t),
        0,
        "preto e branco INVISIVEIS nao sao um salto - o artista nao ve nenhum dos dois"
    );
}

/// ⭐ O salto é o degrau, e o número é o degrau.
#[test]
fn the_seam_is_the_jump_itself() {
    let t = tile(5, 3, |x, _| {
        let v = if x == 4 { 100 } else { 40 };
        [v, v, v, 255]
    });
    assert_eq!(wrap_seam(&t), 60, "100 encosta em 40");
}

/// ⛔⛔ **OS DOIS EIXOS.** Um ladrilho que fecha em `x` e **não** em `y` tem de ser apanhado — sem
/// esta folha, apagar um dos dois laços passa despercebido.
#[test]
fn a_tile_that_closes_in_x_but_not_in_y_is_still_caught() {
    let t = tile(5, 3, |_, y| {
        let v = if y == 2 { 200 } else { 50 };
        [v, v, v, 255]
    });
    assert_eq!(wrap_seam(&t), 150, "a ultima LINHA encosta na primeira");
}

/// O gémeo do anterior: fecha em `y` e não em `x`. ⚠️ Os dois juntos é que prendem os dois laços —
/// um só deixaria a mutação que troca `w` por `h` viva.
#[test]
fn a_tile_that_closes_in_y_but_not_in_x_is_still_caught() {
    let t = tile(5, 3, |x, _| {
        let v = if x == 0 { 210 } else { 30 };
        [v, v, v, 255]
    });
    assert_eq!(wrap_seam(&t), 180, "a primeira COLUNA encosta na ultima");
}

/// ⚠️ O veredito do painel lê o joelho MEDIDO, e a fronteira é fechada de um lado só.
#[test]
fn the_verdict_reads_the_measured_knee() {
    let com = |salto: u8| {
        tiles_cleanly(&tile(5, 3, |x, _| {
            let v = if x == 4 {
                40u8.saturating_add(salto)
            } else {
                40
            };
            [v, v, v, 255]
        }))
    };
    assert!(com(SEAM_VISIBLE), "exactamente no joelho ainda encaixa");
    assert!(
        !com(SEAM_VISIBLE + 1),
        "um nivel acima do joelho ja' e' banda, e o artista tem de saber"
    );
}

/// Um ladrilho de uma coluna encaixa em `x` por identidade — não é caso de aviso.
#[test]
fn a_single_column_tile_closes_in_x_by_identity() {
    assert_eq!(wrap_seam(&tile(1, 3, |_, _| [10, 20, 30, 255])), 0);
}
