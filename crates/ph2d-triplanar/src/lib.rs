//! ⭐⭐⭐ **A TEXTURA SEM UV** — a projecção triplanar para cor, rugosidade e normal.
//!
//! As malhas do campo não têm UV: cada ponto lê a imagem pelas três vistas dos eixos, pesadas pela
//! normal. A geometria (que `(u, v)` cada face lê e com que peso as três se misturam) é a do nó
//! Image Texture «Box» + Blend do **Blender, medida por oráculo** sobre entradas nossas
//! (`docs/3DModeling/ferramentas/oraculo_triplanar_blender.py`). A filtragem é a da PLACA: decodifica
//! o sRGB e filtra em linear, trilinear com o nível dado pelas derivadas de cada vista.
//!
//! O gémeo em WGSL é [`wgsl::fonte`]; os dois avaliam as mesmas contas na mesma ordem.
//!
//! Eixos do produto (Y para cima): `(x, y, z) = (x, z, −y)` do Blender. As coordenadas e a normal que
//! entram aqui são as da FOLHA (o espaço do objecto), para a textura andar com ela.

mod embarcadas;
mod mipmaps;
pub mod wgsl;

pub use embarcadas::Embarcada;
pub use mipmaps::{Mipmaps, srgb_para_linear};

/// O lado das imagens que a placa guarda: medido (handoff AS_TEXTURAS §2) — `1024` custa o mesmo
/// que `512` por pixel, e `2048` paga cache e 4× a memória.
pub const LADO: u32 = 1024;

type V3 = [f32; 3];

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// ⭐ **Os pesos das três vistas** (X, Y, Z) — a regra do «Blend» do Blender, nos três regimes:
/// uma vista onde um eixo domina os outros dois aos pares, duas onde o terceiro é pequeno, três no
/// canto. `blend = 0` sem eixo dominante cai na vista X (a escolha do Blender).
#[must_use]
pub fn pesos(n: V3, blend: f32) -> V3 {
    // A ordem dos testes é a do Blender, nos eixos DELE: (x, y, z)_b = (x, −z, y)_nosso.
    let s = n[0].abs() + n[2].abs() + n[1].abs();
    if s <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let (x, y, z) = (n[0].abs() / s, n[2].abs() / s, n[1].abs() / s);
    let l = 0.5 * (1.0 + blend);
    let mut w = [0.0f32; 3];
    if x > l * (x + y) && x > l * (x + z) {
        w[0] = 1.0;
    } else if y > l * (x + y) && y > l * (y + z) {
        w[1] = 1.0;
    } else if z > l * (x + z) && z > l * (y + z) {
        w[2] = 1.0;
    } else if blend > 0.0 {
        let par = |a: f32, b: f32| {
            let q = ((a / (a + b) - 0.5 * (1.0 - blend)) / blend).clamp(0.0, 1.0);
            (q, 1.0 - q)
        };
        if z < (1.0 - l) * (y + x) {
            (w[0], w[1]) = par(x, y);
        } else if x < (1.0 - l) * (y + z) {
            (w[1], w[2]) = par(y, z);
        } else if y < (1.0 - l) * (x + z) {
            (w[0], w[2]) = par(x, z);
        } else {
            let k = 2.0 * l - 1.0;
            w = [x, y, z].map(|c| ((2.0 - l) * c + (l - 1.0)) / k);
        }
    } else {
        w[0] = 1.0;
    }
    // De volta aos nossos eixos.
    [w[0], w[2], w[1]]
}

/// ⭐ **Uma das seis vistas**: a direcção em que `u` cresce (`t`), a de `v` (`b`), a normal de fora
/// (`a = t × b`) e a constante de `u` (`k`). Medido: cada face lê a imagem de FRENTE.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vista {
    pub t: V3,
    pub b: V3,
    pub a: V3,
    pub k: f32,
}

/// A vista do `eixo` (0 X, 1 Y, 2 Z) do lado do sinal de `n_eixo` (zero conta como positivo).
#[must_use]
pub fn vista(eixo: usize, n_eixo: f32) -> Vista {
    let p = n_eixo >= 0.0;
    let s = if p { 1.0 } else { -1.0 };
    match eixo {
        0 => Vista {
            t: [0.0, 0.0, -s],
            b: [0.0, 1.0, 0.0],
            a: [s, 0.0, 0.0],
            k: if p { 0.0 } else { 1.0 },
        },
        1 => Vista {
            t: [0.0, 0.0, s],
            b: [1.0, 0.0, 0.0],
            a: [0.0, s, 0.0],
            k: if p { 1.0 } else { 0.0 },
        },
        _ => Vista {
            t: [s, 0.0, 0.0],
            b: [0.0, 1.0, 0.0],
            a: [0.0, 0.0, s],
            k: if p { 0.0 } else { 1.0 },
        },
    }
}

/// ⭐ **Os números da textura de um material.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triplanar {
    /// Quanto mede um ladrilho na direcção `u`, em unidades da folha.
    pub tamanho: f32,
    /// Altura/largura da imagem original: o ladrilho mede `tamanho × aspecto` em `v`.
    pub aspecto: f32,
    /// O «Blend» do Blender, `0..1`.
    pub blend: f32,
    /// A força do relevo do mapa de normal (`0` = a normal da forma).
    pub relevo: f32,
}

impl Triplanar {
    /// `(1/tamanho, 1/(tamanho·aspecto))` — o que a coordenada multiplica.
    #[must_use]
    pub fn escalas(&self) -> (f32, f32) {
        (1.0 / self.tamanho, 1.0 / (self.tamanho * self.aspecto))
    }
}

/// O que a textura dá a um ponto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resultado {
    /// A cor em LINEAR (multiplica a cor-base do material).
    pub cor: V3,
    /// A rugosidade (só tem sentido com [`Mapas::tem_rugosidade`]).
    pub rugosidade: f32,
    /// A normal perturbada, unitária, no espaço da folha.
    pub normal: V3,
}

/// ⭐ **Os dois mapas de um material**: a cor (sRGB) e `nrh` (r, g = a normal xy no padrão OpenGL,
/// b = a rugosidade).
#[derive(Clone, Debug, PartialEq)]
pub struct Mapas {
    pub cor: Mipmaps,
    pub nrh: Mipmaps,
    pub tem_normal: bool,
    pub tem_rugosidade: bool,
}

/// `(u, v)` da vista `w` no ponto `p` — e a mesma conta faz as derivadas, sem o `k`.
fn uv(w: &Vista, p: V3, eu: f32, ev: f32, k: f32) -> (f32, f32) {
    (dot(p, w.t) * eu + k, dot(p, w.b) * ev)
}

/// ⭐ **O nível do mip** de uma vista: `log2` da maior pegada do pixel, em texels.
#[must_use]
pub fn nivel(w: &Vista, dx: V3, dy: V3, eu: f32, ev: f32, lado: f32) -> f32 {
    let (ax, bx) = uv(w, dx, eu * lado, ev * lado, 0.0);
    let (ay, by) = uv(w, dy, eu * lado, ev * lado, 0.0);
    0.5 * (ax * ax + bx * bx).max(ay * ay + by * by).max(1.0e-12).log2()
}

/// ⭐⭐⭐ **A TRIPLANAR num ponto**: `p` e `n` (unitária) no espaço da folha; `dx`/`dy` = a variação
/// de `p` entre pixels vizinhos (as derivadas de ecrã).
#[must_use]
pub fn avalia(m: &Mapas, t: &Triplanar, p: V3, n: V3, dx: V3, dy: V3) -> Resultado {
    let w = pesos(n, t.blend);
    let (eu, ev) = t.escalas();
    let lado = m.cor.lado() as f32;
    let mut cor = [0.0f32; 3];
    let mut rug = 0.0f32;
    let mut soma_n = [0.0f32; 3];
    let usa_nrh = (m.tem_normal && t.relevo != 0.0) || m.tem_rugosidade;
    for (e, &we) in w.iter().enumerate() {
        if we <= 0.0 {
            continue;
        }
        let vi = vista(e, n[e]);
        let (u, v) = uv(&vi, p, eu, ev, vi.k);
        let lod = nivel(&vi, dx, dy, eu, ev, lado);
        let c = m.cor.amostra(true, u, v, lod);
        for q in 0..3 {
            cor[q] += c[q] * we;
        }
        if !usa_nrh {
            continue;
        }
        let h = m.nrh.amostra(false, u, v, lod);
        rug += h[2] * we;
        let tn = normal_do_mapa(h, if m.tem_normal { t.relevo } else { 0.0 });
        // Whiteout (Golus 2017) na base (t, b, a) desta vista.
        let (nt, nb, na) = (dot(n, vi.t), dot(n, vi.b), dot(n, vi.a));
        let (x, y, z) = (tn[0] + nt, tn[1] + nb, tn[2] * na);
        for q in 0..3 {
            soma_n[q] += (vi.t[q] * x + vi.b[q] * y + vi.a[q] * z) * we;
        }
    }
    let normal = if m.tem_normal && t.relevo != 0.0 {
        normaliza(soma_n)
    } else {
        n
    };
    Resultado {
        cor,
        rugosidade: rug,
        normal,
    }
}

/// A normal de tangente de um texel `nrh` com a força `k`: `xy·k`, e o `z` que a fecha.
#[must_use]
pub fn normal_do_mapa(h: [f32; 4], k: f32) -> V3 {
    let x = (h[0] * 2.0 - 1.0) * k;
    let y = (h[1] * 2.0 - 1.0) * k;
    [x, y, (1.0 - (x * x + y * y).min(1.0)).sqrt()]
}

fn normaliza(v: V3) -> V3 {
    let l = dot(v, v).sqrt();
    if l > 0.0 { v.map(|c| c / l) } else { v }
}

#[cfg(test)]
mod tests;
