//! **O GRÃO** — ruído de valor gaussiano, por célula de `size` px, somado aos valores CODIFICADOS
//! (o grão é do filme, que é o que se vê), pesado pela resposta aos meios-tons.
//!
//! O hash é o `pcg3d` de Jarzynski & Olano (*Hash Functions for GPU Rendering*, JCGT 2020): só
//! operações de inteiro de 32 bits, logo o MESMO bit nas duas línguas. A semente anda com o TIQUE
//! do relógio do Motion, nunca com o relógio da parede: o scrub devolve o MESMO grão (ciclo 19).

/// Os params do artista.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grao {
    /// `0..1`: o desvio-padrão é `amount × ¼` (em valores codificados). `0` desliga.
    pub amount: f32,
    /// O lado da célula, em px do ecrã (`1` = um grão por pixel).
    pub size: f32,
    /// `0..1`: `0` é uniforme, `1` concentra o grão nos meios-tons (`4L(1-L)`).
    pub response: f32,
    /// Um ruído por canal (cor) ou o mesmo nos três (mono).
    pub colorido: bool,
    /// A semente soma o tique (o grão mexe) ou fica parada.
    pub animado: bool,
    pub semente: u32,
}

impl Grao {
    #[must_use]
    pub fn neutra(&self) -> bool {
        self.amount == 0.0
    }
}

/// O desvio-padrão por unidade de `amount`.
pub const SIGMA_POR_AMOUNT: f32 = 0.25;

#[must_use]
pub fn pcg3d(v: [u32; 3]) -> [u32; 3] {
    let mut v = v.map(|x| x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223));
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[2]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    v = v.map(|x| x ^ (x >> 16));
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[2]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    v
}

/// A média da soma de quatro uniformes de 23 bits (`4 × (2²³ - 1) / 2`).
const CENTRO: i32 = (1 << 24) - 2;
/// `√3 / 2²³`: os bits do `√3` com o expoente descido 23.
const ESCALA: f32 = f32::from_bits(0x345D_B3D7);

/// Uma amostra ~N(0,1) por célula: Irwin–Hall de 4 uniformes de **23 bits** somadas em INTEIRO
/// (a soma centrada cabe exacta num `f32`), escalada para variância 1 — um arredondamento só, o
/// da multiplicação, e o MESMO bit nas duas línguas. Medido: com a soma em `f32` a placa errava
/// 1 ulp (a célula `[-6, -6]` do `lens_gpu`).
#[must_use]
pub fn gauss_celula(cel: [i32; 2], semente: u32, canal: u32) -> f32 {
    let a = pcg3d([
        cel[0] as u32,
        cel[1] as u32,
        semente ^ canal.wrapping_mul(0x9E37_79B9),
    ]);
    let b = pcg3d([a[0], a[1], a[2] ^ 0x85EB_CA6B]);
    let soma = (a[0] >> 9) + (a[1] >> 9) + (a[2] >> 9) + (b[0] >> 9);
    (soma as i32 - CENTRO) as f32 * ESCALA
}

/// O ruído no pixel de centro `p`: as quatro células vizinhas, interpoladas com `smoothstep`.
#[must_use]
pub fn ruido(p: [f32; 2], size: f32, semente: u32, canal: u32) -> f32 {
    let s = size.max(1.0);
    let q = [p[0] / s - 0.5, p[1] / s - 0.5];
    let i = [q[0].floor(), q[1].floor()];
    let f = [q[0] - i[0], q[1] - i[1]].map(|t| t * t * (3.0 - 2.0 * t));
    let (ix, iy) = (i[0] as i32, i[1] as i32);
    let g = |dx: i32, dy: i32| gauss_celula([ix + dx, iy + dy], semente, canal);
    let topo = g(0, 0) + (g(1, 0) - g(0, 0)) * f[0];
    let base = g(0, 1) + (g(1, 1) - g(0, 1)) * f[0];
    topo + (base - topo) * f[1]
}

/// A semente do quadro.
#[must_use]
pub fn semente(g: &Grao, tique: u32) -> u32 {
    if g.animado {
        g.semente.wrapping_add(tique)
    } else {
        g.semente
    }
}

/// O grão sobre uma cor CODIFICADA no pixel de centro `p`.
#[must_use]
pub fn aplica(cod: [f32; 3], p: [f32; 2], g: &Grao, semente: u32) -> [f32; 3] {
    let l = (0.2126 * cod[0] + 0.7152 * cod[1] + 0.0722 * cod[2]).clamp(0.0, 1.0);
    let peso = 1.0 + (4.0 * l * (1.0 - l) - 1.0) * g.response.clamp(0.0, 1.0);
    let k = g.amount * SIGMA_POR_AMOUNT * peso;
    if g.colorido {
        [0, 1, 2].map(|c| cod[c] + k * ruido(p, g.size, semente, c as u32))
    } else {
        let n = ruido(p, g.size, semente, 0);
        cod.map(|v| v + k * n)
    }
}
