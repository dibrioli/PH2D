//! Os números e o desenho da cena 18.

/// O tamanho da cena 18.
pub const SIZE: (u32, u32) = (96, 64);
/// Onde o desenho está (o canto de cima à esquerda).
pub const AT: (i32, i32) = (6, 6);
/// O desenho: 6 × 4, uma cor por coluna.
pub const DRAWING: [[Option<[u8; 4]>; 6]; 4] = {
    const R: Option<[u8; 4]> = Some([230, 40, 40, 255]);
    const O: Option<[u8; 4]> = Some([245, 150, 30, 255]);
    const Y: Option<[u8; 4]> = Some([240, 220, 40, 255]);
    const G: Option<[u8; 4]> = Some([40, 170, 70, 255]);
    const B: Option<[u8; 4]> = Some([40, 90, 220, 255]);
    const P: Option<[u8; 4]> = Some([150, 50, 190, 255]);
    [
        [R, O, Y, G, B, P],
        [R, O, Y, G, B, P],
        [R, O, Y, G, B, P],
        [R, O, Y, G, B, P],
    ]
};
/// O Size da barra (o máximo da dinâmica).
pub const BRUSH_SIZE: u32 = 16;
