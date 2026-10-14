//! ⭐ P4 (doc 132 §4.1 do Motion) — **as IMAGENS de grupo como células do `Each Shape`**: o que a placa
//! entrega (as linhas, a vista, as corridas e o passe que as pinta) e a ordem das linhas no plano.
//! Irmão do `celulas.rs`, cortado pelo tecto de LOC.

use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::{Corrida, ShapeView};

/// Quem pinta as imagens relocadas numa página: o alvo, e `(vista da página, linhas relocadas, a
/// linha original de cada uma)`.
pub type PintaImagens<'a> = &'a mut dyn FnMut(
    &GpuContext,
    &mut wgpu::CommandEncoder,
    &wgpu::TextureView,
    (ShapeView, &wgpu::Buffer, &[u32]),
);

/// ⭐ P4 — as linhas de imagem de grupo da placa (`[RenderInstance]`, as que o passe dela lê), quantas,
/// a vista da camada, as corridas da placa e o passe que as pinta.
pub struct Imagens<'a> {
    pub buffer: &'a wgpu::Buffer,
    pub n: u32,
    pub vista: ShapeView,
    pub corridas: &'a [Corrida],
    pub pinta: PintaImagens<'a>,
}

/// ⭐ P4 — **a ordem das linhas no plano**: as sprites, e depois, corrida a corrida da placa, as
/// formas e as IMAGENS dela — as células de uma corrida voltam juntas e o grupo dela não se reabre
/// (com as imagens no fim, um 2.º grupo entre as formas e as imagens do 1.º partia-o em dois). As
/// linhas que nenhuma corrida nomeia seguem pela ordem delas.
#[must_use]
pub fn ordem_das_linhas(n_s: u32, n_f: u32, n_i: u32, corridas: &[Corrida]) -> Vec<u32> {
    let mut v: Vec<u32> = (0..n_s).collect();
    let (mut f, mut i) = (vec![false; n_f as usize], vec![false; n_i as usize]);
    for c in corridas {
        // doc 132 §4.1 (3): a corrida ORDENADA é a mesma faixa nos dois buffers (a linha `j` em
        // `copias.start + j` e `imagens.start + j`, a do outro meio vazia) — linha a linha.
        if c.ordenada() {
            for (kf, ki) in c.copias.clone().zip(c.imagens.clone()) {
                if kf < n_f && !std::mem::replace(&mut f[kf as usize], true) {
                    v.push(n_s + kf);
                }
                if ki < n_i && !std::mem::replace(&mut i[ki as usize], true) {
                    v.push(n_s + n_f + ki);
                }
            }
            continue;
        }
        for k in c.copias.clone().filter(|&k| k < n_f) {
            if !std::mem::replace(&mut f[k as usize], true) {
                v.push(n_s + k);
            }
        }
        for k in c.imagens.clone().filter(|&k| k < n_i) {
            if !std::mem::replace(&mut i[k as usize], true) {
                v.push(n_s + n_f + k);
            }
        }
    }
    v.extend((0..n_f).filter(|&k| !f[k as usize]).map(|k| n_s + k));
    v.extend((0..n_i).filter(|&k| !i[k as usize]).map(|k| n_s + n_f + k));
    v
}

/// `L⁻¹ · t`: a translação no MUNDO que move a projecção `t` pixels — a de toda célula (sprite, forma
/// ou imagem), pela vista dela.
pub(super) fn no_mundo(lin: [f64; 4], t: [i32; 2]) -> [f32; 2] {
    let [a, b, c, d] = lin;
    let det = a * d - b * c;
    let (x, y) = (f64::from(t[0]), f64::from(t[1]));
    #[expect(clippy::cast_possible_truncation, reason = "o mundo lê f32")]
    [
        ((d * x - c * y) / det) as f32,
        ((a * y - b * x) / det) as f32,
    ]
}
