//! Os gates da COLOCAÇÃO (plano 33 §5) — onde o ladrilho fica, e quanto ele mede no mundo.
//!
//! ⚠️ A colocação é devolvida como `[f64; 6]` na disposição do `kurbo::Affine` (`x' = a*x + c*y + e`,
//! `y' = b*x + d*y + f`) — a mesma convenção que o `xform_of` da `ph2d-vec-scene` já usa, e a razão
//! de esta folha não depender de `kurbo`.

use super::{HEX_ROW_RATIO, TileKind, TileLaw, gap_px_from_world, hex_row_period, placement};

fn apply(m: [f64; 6], p: [f64; 2]) -> [f64; 2] {
    [
        m[0] * p[0] + m[2] * p[1] + m[4],
        m[1] * p[0] + m[3] * p[1] + m[5],
    ]
}

fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

/// ⭐ **O ladrilho ocupa exactamente `período x células`** — é isto que faz o padrão repetir-se com
/// o passo que o artista pediu, e não com o tamanho dos pixels que por acaso a arte tem.
#[test]
fn the_tile_corners_land_on_the_period() {
    let m = placement([30.0, 20.0], [1, 3], [7.0, -4.0], 0.0, [64, 192]);
    assert!(
        close(apply(m, [0.0, 0.0]), [7.0, -4.0]),
        "o canto e' a origem"
    );
    assert!(
        close(apply(m, [64.0, 192.0]), [7.0 + 30.0, -4.0 + 60.0]),
        "o ladrilho mede um periodo na horizontal e TRES na vertical"
    );
}

/// ⚠️ **A resolução do assado não pode mudar onde o padrão fica.** Dois ladrilhos com o mesmo
/// período e contagens de pixels diferentes têm de mapear para o mesmo rectângulo de mundo — senão
/// re-assar em maior qualidade deslocaria o desenho do artista.
#[test]
fn the_bake_resolution_does_not_move_the_pattern() {
    let a = placement([12.0, 12.0], [1, 1], [0.0, 0.0], 0.0, [16, 16]);
    let b = placement([12.0, 12.0], [1, 1], [0.0, 0.0], 0.0, [512, 512]);
    assert!(close(apply(a, [16.0, 16.0]), apply(b, [512.0, 512.0])));
    assert!(close(apply(a, [8.0, 0.0]), apply(b, [256.0, 0.0])));
}

/// **Uma rotação roda e nada mais**: o canto oposto conserva a distância à origem, e o eixo do
/// ladrilho sai do ângulo pedido.
#[test]
fn a_rotation_turns_the_tile_and_nothing_else() {
    let ang = std::f64::consts::FRAC_PI_2;
    let m = placement([10.0, 10.0], [1, 1], [3.0, 3.0], ang, [10, 10]);
    let far = apply(m, [10.0, 0.0]);
    assert!(
        close(far, [3.0, 13.0]),
        "um quarto de volta poe o eixo x em +y, deu {far:?}"
    );
    let d = ((far[0] - 3.0).powi(2) + (far[1] - 3.0).powi(2)).sqrt();
    assert!((d - 10.0).abs() < 1e-9, "a rotacao nao pode mudar a medida");
}

/// ⭐⭐ **A colmeia é a LEI DO ESPAÇAMENTO, não um assado diferente.**
///
/// O `Hex` assa byte-a-byte como um `BrickRow` de meio passo; o que o torna colmeia é o período
/// vertical ser `√3/2` do horizontal, que é o único valor que põe os **seis** vizinhos à mesma
/// distância. Este gate mede isso na geometria, e não na constante.
#[test]
fn the_hex_rows_put_the_six_neighbours_at_equal_distance() {
    let p = 40.0_f64;
    let rp = hex_row_period(p);
    // Vizinho lateral: uma coluna. Vizinho diagonal: meia coluna e uma linha.
    let lateral = p;
    let diagonal = ((p / 2.0).powi(2) + rp.powi(2)).sqrt();
    assert!(
        (lateral - diagonal).abs() < 1e-9,
        "colmeia: lateral {lateral} != diagonal {diagonal}"
    );
    assert!((rp / p - HEX_ROW_RATIO).abs() < 1e-12);
}

/// ⚠️ **O `Hex` fecha em DUAS linhas**, como o tijolo de meio passo — se fechasse noutro número, o
/// período vertical acima seria o de outro reticulado.
#[test]
fn the_hex_law_closes_in_two_rows() {
    let law = TileLaw {
        kind: TileKind::Hex,
        offset_denom: 7, // ⚠️ ignorado de propósito: a colmeia É meio passo.
        gap_px: [0, 0],
    };
    assert_eq!(law.period(), 2);
    assert_eq!(law.cells(), [1, 2]);
}

/// ⚠️ **O vão do artista é MUNDO; o assador só sabe pixels.** A conversão é a escala da própria
/// arte (`pixels / unidades`), e ela vive numa função só — escrita duas vezes, um dos lados
/// envelheceria no dia em que a arte mudasse de resolução.
#[test]
fn the_world_gap_becomes_pixels_through_the_arts_own_scale() {
    // Uma arte de 256 px que mede 32 unidades: 8 px por unidade.
    assert_eq!(
        gap_px_from_world([2.0, -1.0], [32.0, 32.0], [256, 256]),
        [16, -8]
    );
    // Sem vão, sem pixels — e um tamanho de mundo degenerado não pode dividir por zero.
    assert_eq!(
        gap_px_from_world([0.0, 0.0], [32.0, 32.0], [256, 256]),
        [0, 0]
    );
    assert_eq!(
        gap_px_from_world([5.0, 5.0], [0.0, 0.0], [256, 256]),
        [0, 0]
    );
}
