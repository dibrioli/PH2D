//! Os números e a arte das cenas 1 (o rosto) e 3 (a bola).

/// Os quatro traços da cara (células de um sprite 32 × 32) e a cor de cada um (índice da DB32):
/// olho esquerdo e direito (preto), o sorriso (vermelho), a moldura (azul).
pub const FACE: [(&[(i32, i32)], usize); 4] = [
    (&[(11, 11), (11, 13)], 0),
    (&[(20, 11), (20, 13)], 0),
    (&[(9, 19), (12, 22), (19, 22), (22, 19)], 27),
    (&[(6, 6), (25, 6), (25, 25), (6, 25), (6, 6)], 17),
];

/// A rampa da bola da cena 3 (índices da DB32, do escuro ao claro e contíguos: o Shading anda em
/// ordem de PALETA).
pub const BALL_RAMP: [usize; 4] = [4, 5, 6, 7];
/// Os azuis do céu, de cima para baixo (índices contíguos da DB32).
pub const SKY: [usize; 6] = [15, 16, 17, 18, 19, 20];
/// O chão.
pub const GROUND: usize = 12;
/// O centro e o raio da bola.
pub const BALL: ((i32, i32), i32) = ((32, 26), 9);

/// O degrau da rampa de cada célula da bola: a luz vem de cima à esquerda.
pub fn ball_step(x: i32, y: i32) -> Option<usize> {
    let ((cx, cy), r) = BALL;
    let (dx, dy) = (x - cx, y - cy);
    if dx * dx + dy * dy > r * r {
        return None;
    }
    let (lx, ly) = (x - (cx - 3), y - (cy - 3));
    let d = lx * lx + ly * ly;
    Some(match d {
        0..=6 => 3,
        7..=30 => 2,
        31..=110 => 1,
        _ => 0,
    })
}
