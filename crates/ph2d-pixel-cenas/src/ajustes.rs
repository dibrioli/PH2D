//! Os números da cena 14.

/// O tamanho da cena 14.
pub const SIZE: (u32, u32) = (128, 80);
/// O retângulo assentado: a caixa (cantos inclusivos) e a cor.
pub const SETTLED: ((i32, i32), (i32, i32), [u8; 4]) = ((6, 10), (39, 69), [70, 110, 220, 255]);
/// A estrela viva: a caixa, a cor e o giro (graus, horário na tela: a barra mostra o oposto).
pub const STAR: ((i32, i32), (i32, i32), [u8; 4], f64) =
    ((58, 8), (117, 67), [240, 200, 60, 255], -15.0);
/// As pontas da estrela ao nascer e depois do ajuste guardado.
pub const POINTS: (f64, f64) = (5.0, 7.0);
/// Só para a foto: a alça das pontas agarrada, como a meio de um arrasto.
pub const ENV_AGARRA: &str = "PH2D_PIXEL_SMOKE_AGARRA";
