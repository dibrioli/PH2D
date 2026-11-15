//! Os dados dos níveis 1 e 2 do `PH2D_GAME_SMOKE` (blocos, moedas, céu, a câmara do início): dado de cena,
//! fora da `ph2d-app-game` (a catraca de LOC, GEF 07). A `ph2d_app_game::smoke` reexporta-os.

/// Os blocos do nível: nome, centro, meia-extensão (metros).
pub const BLOCOS: [(&str, [f32; 2], [f32; 2]); 6] = [
    ("Ground Left", [-5.0, -0.5], [7.0, 0.5]),
    ("Ground Right", [11.0, -0.5], [6.0, 0.5]),
    ("Platform Low", [3.5, 1.6], [1.2, 0.25]),
    ("Platform High", [9.0, 3.0], [1.5, 0.25]),
    ("Wall Left", [-12.5, 3.0], [0.5, 4.0]),
    ("Wall Right", [17.5, 3.0], [0.5, 4.0]),
];

pub const MOEDAS: [[f32; 2]; 5] = [
    [-4.0, 0.7],
    [-1.5, 0.7],
    [3.5, 2.4],
    [9.0, 3.8],
    [14.0, 0.7],
];

/// Os blocos do nível 2 (*Static Body*): o desenho do nível 1 com as plataformas à altura do salto do
/// boneco de fábrica (sobe ~1,4 m; a baixa tem o topo a 1,1 m, a alta alcança-se da baixa).
pub const BLOCOS_2: [(&str, [f32; 2], [f32; 2]); 6] = [
    ("Ground Left", [-5.0, -0.5], [7.0, 0.5]),
    ("Ground Right", [11.0, -0.5], [6.0, 0.5]),
    ("Platform Low", [3.5, 0.85], [1.2, 0.25]),
    ("Platform High", [8.5, 2.0], [1.5, 0.25]),
    ("Wall Left", [-12.5, 3.0], [0.5, 4.0]),
    // Fundo a 0,60 m: de pé (topo 0,80) não cabe, agachado (0,50) passa — plano 09.
    ("Low Ceiling", [-10.5, 0.85], [1.5, 0.25]),
];

/// A parede invisível do fim do nível: centro e meia-extensão no mundo (m).
pub const PAREDE_INVISIVEL: ([f32; 2], [f32; 2]) = ([17.5, 3.0], [0.5, 4.0]);

/// As moedas do nível 2: sobre o chão, sobre cada plataforma e antes da meta.
pub const MOEDAS_2: [[f32; 2]; 5] = [
    [-4.0, 0.6],
    [-1.5, 0.6],
    [3.5, 1.65],
    [8.5, 2.8],
    [13.5, 0.6],
];

/// O fundo do nível 2 (plano 11): o céu parado no ecrã, maior que a vista mais larga, na cor do céu do fundo
/// azul do Kenney (`#DFF6F5`).
pub const CEU_TAMANHO: [f32; 2] = [40.0, 16.0];
/// ⚠️ O tingimento é LINEAR (`#DFF6F5` pela curva sRGB); `223/255` saía `#F0FBFA` (foto do plano 11).
pub const CEU_COR: [f32; 4] = [0.7379, 0.9216, 0.9131, 1.0];
/// O centro da câmara quando o nível começa (o jogador em `[-8, 1,5]` encostado ao limite da esquerda, 16:9).
pub const CENTRO_DO_INICIO: [f32; 2] = [-5.9, 3.0];

/// A pose AUTORADA que põe uma camada com fracção `k` em `no_mundo` quando a câmara está em
/// [`CENTRO_DO_INICIO`] (a ponte desloca-a de `centro·(1 − k)`).
#[must_use]
pub fn autorada(no_mundo: [f32; 2], k: f32) -> [f32; 2] {
    [
        no_mundo[0] - CENTRO_DO_INICIO[0] * (1.0 - k),
        no_mundo[1] - CENTRO_DO_INICIO[1] * (1.0 - k),
    ]
}
