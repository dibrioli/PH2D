//! ⭐⭐ **O CONTACTO NA PLACA** — as grelhas de [`ph2d_contacto::Grade`] num atlas de três texturas
//! `3D` `Rgba16Float` (os `9` coeficientes), uma TELHA de `LADO³` por malha, e a tabela do quadro: por
//! instância com grelha, a afim mundo → grelha e a telha. O `forward.wgsl` lê-as no `contacto(..)`.
//!
//! ⚠️ A grelha vive no referencial da MALHA: mover uma instância só muda a afim, nada se refaz.

use std::collections::BTreeMap;

use ph2d_contacto::{COEFS, Grade, LADO};

/// As instâncias com grelha que um quadro lê (o resto não tapa ninguém).
pub const MAX_CONTACTO: usize = 32;
/// Os floats da tabela: `[quantas, 0, 0, 0]` e `4 vec4` por entrada.
pub(crate) const TABELA: usize = 4 + MAX_CONTACTO * 16;
const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(crate) struct Contacto {
    vistas: [wgpu::TextureView; 3],
    texturas: [wgpu::Texture; 3],
    /// Telhas por eixo (`x` e `y`; em `z` é uma).
    telhas: u32,
    /// Por malha: a telha e a grelha (em `f16`, já como a placa a lê).
    grades: BTreeMap<u64, (u32, Grade)>,
    pub tabela: wgpu::Buffer,
}

fn atlas(device: &wgpu::Device, telhas: u32) -> ([wgpu::Texture; 3], [wgpu::TextureView; 3]) {
    let t = std::array::from_fn(|k| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(["contacto 0", "contacto 1", "contacto 2"][k]),
            size: wgpu::Extent3d {
                width: LADO as u32 * telhas,
                height: LADO as u32 * telhas,
                depth_or_array_layers: LADO as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: FORMATO,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    });
    let v =
        std::array::from_fn(|k: usize| t[k].create_view(&wgpu::TextureViewDescriptor::default()));
    (t, v)
}

/// As três entradas das texturas (filtráveis, `3D`) a partir de `primeira`.
pub(crate) fn entradas(primeira: u32) -> [wgpu::BindGroupLayoutEntry; 3] {
    std::array::from_fn(|k| wgpu::BindGroupLayoutEntry {
        binding: primeira + k as u32,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D3,
            multisampled: false,
        },
        count: None,
    })
}

impl Contacto {
    pub(crate) fn novo(device: &wgpu::Device) -> Self {
        let telhas = 2;
        let (texturas, vistas) = atlas(device, telhas);
        let tabela = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward contacto"),
            size: (TABELA * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            vistas,
            texturas,
            telhas,
            grades: BTreeMap::new(),
            tabela,
        }
    }

    pub(crate) fn vista(&self, k: usize) -> &wgpu::TextureView {
        &self.vistas[k]
    }

    /// Sobe (ou substitui) a grelha da malha `id`; o atlas dobra quando enche (e volta a subir tudo).
    pub(crate) fn sobe(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, id: u64, g: &Grade) {
        let telha = match self.grades.get(&id) {
            Some((t, _)) => *t,
            None => (0..)
                .find(|t| self.grades.values().all(|(u, _)| u != t))
                .unwrap_or(0),
        };
        self.grades.insert(id, (telha, g.em_f16()));
        if telha >= self.telhas * self.telhas {
            while telha >= self.telhas * self.telhas {
                self.telhas *= 2;
            }
            (self.texturas, self.vistas) = atlas(device, self.telhas);
            for (t, g) in self.grades.values() {
                self.escreve(queue, *t, g);
            }
        } else {
            self.escreve(queue, telha, &self.grades[&id].1);
        }
    }

    pub(crate) fn esquece(&mut self, id: u64) {
        self.grades.remove(&id);
    }

    fn escreve(&self, queue: &wgpu::Queue, telha: u32, g: &Grade) {
        let l = LADO as u32;
        let origem = wgpu::Origin3d {
            x: (telha % self.telhas) * l,
            y: (telha / self.telhas) * l,
            z: 0,
        };
        for (k, tex) in self.texturas.iter().enumerate() {
            let dados: Vec<half::f16> = g
                .coef
                .iter()
                .flat_map(|c| {
                    std::array::from_fn::<f32, 4, _>(|j| {
                        let i = 4 * k + j;
                        if i < COEFS { c[i] } else { 0.0 }
                    })
                })
                .map(half::f16::from_f32)
                .collect();
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: tex,
                    mip_level: 0,
                    origin: origem,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&dados),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(l * 8),
                    rows_per_image: Some(l),
                },
                wgpu::Extent3d {
                    width: l,
                    height: l,
                    depth_or_array_layers: l,
                },
            );
        }
    }

    /// ⭐ A tabela do quadro: as instâncias (índice na lista desenhada) cuja malha tem grelha.
    pub(crate) fn prepara(&self, queue: &wgpu::Queue, objs: &[&crate::Instancia]) {
        let mut u = vec![0.0f32; TABELA];
        let mut n = 0;
        for (i, o) in objs.iter().enumerate() {
            let Some((telha, g)) = self.grades.get(&o.malha) else {
                continue;
            };
            if n == MAX_CONTACTO {
                break;
            }
            let a = mundo_para_grade(&o.modelo, g);
            let e = 4 + 16 * n;
            for (r, linha) in a.iter().enumerate() {
                u[e + 4 * r..e + 4 * r + 4].copy_from_slice(linha);
            }
            u[e + 12] = ((telha % self.telhas) * LADO as u32) as f32;
            u[e + 13] = ((telha / self.telhas) * LADO as u32) as f32;
            u[e + 14] = i as f32;
            n += 1;
        }
        u[0] = n as f32;
        u[1] = (LADO as u32 * self.telhas) as f32;
        queue.write_buffer(&self.tabela, 0, bytemuck::cast_slice(&u));
    }
}

/// As três linhas da afim mundo → `u ∈ [0, 1]³` da grelha: `u = (modelo⁻¹·p − lo)/(hi − lo)`.
/// ⚠️ A parte linear serve também a NORMAL (normalizada): a pose é rígida com escala uniforme e o
/// cubo da grelha tem as três arestas iguais.
pub(crate) fn mundo_para_grade(modelo: &[[f32; 4]; 4], g: &Grade) -> [[f32; 4]; 3] {
    let inv = inversa_afim(modelo);
    let mut a = [[0.0f32; 4]; 3];
    for (r, linha) in a.iter_mut().enumerate() {
        let esc = 1.0 / (g.hi[r] - g.lo[r]);
        for c in 0..3 {
            linha[c] = inv[c][r] * esc;
        }
        linha[3] = (inv[3][r] - g.lo[r]) * esc;
    }
    a
}

/// O inverso de uma afim por colunas (`m[coluna][linha]`, a última linha `0 0 0 1`).
fn inversa_afim(m: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let l = |r: usize, c: usize| f64::from(m[c][r]);
    let det = l(0, 0) * (l(1, 1) * l(2, 2) - l(1, 2) * l(2, 1))
        - l(0, 1) * (l(1, 0) * l(2, 2) - l(1, 2) * l(2, 0))
        + l(0, 2) * (l(1, 0) * l(2, 1) - l(1, 1) * l(2, 0));
    let d = if det.abs() > 1.0e-30 { 1.0 / det } else { 0.0 };
    // A adjunta transposta: inv[r][c] da parte linear.
    let mut li = [[0.0f64; 3]; 3];
    for (r, linha) in li.iter_mut().enumerate() {
        for (c, v) in linha.iter_mut().enumerate() {
            let (r1, r2) = ((c + 1) % 3, (c + 2) % 3);
            let (c1, c2) = ((r + 1) % 3, (r + 2) % 3);
            *v = (l(r1, c1) * l(r2, c2) - l(r1, c2) * l(r2, c1)) * d;
        }
    }
    let t = [m[3][0], m[3][1], m[3][2]].map(f64::from);
    let mut out = [[0.0f32; 4]; 4];
    for r in 0..3 {
        for c in 0..3 {
            out[c][r] = li[r][c] as f32;
        }
        out[3][r] = -(0..3).map(|c| li[r][c] * t[c]).sum::<f64>() as f32;
    }
    out[3][3] = 1.0;
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_inversa_desfaz_a_pose() {
        let (c, s) = (0.6f32.cos(), 0.6f32.sin());
        let m = [
            [c * 1.5, 0.0, -s * 1.5, 0.0],
            [0.0, 1.5, 0.0, 0.0],
            [s * 1.5, 0.0, c * 1.5, 0.0],
            [0.3, -0.2, 0.7, 1.0],
        ];
        let inv = super::inversa_afim(&m);
        let p = [0.1f32, 0.4, -0.3];
        let w: [f32; 3] =
            std::array::from_fn(|r| (0..3).map(|k| m[k][r] * p[k]).sum::<f32>() + m[3][r]);
        let q: [f32; 3] =
            std::array::from_fn(|r| (0..3).map(|k| inv[k][r] * w[k]).sum::<f32>() + inv[3][r]);
        for e in 0..3 {
            assert!((q[e] - p[e]).abs() < 1.0e-5, "{q:?} contra {p:?}");
        }
    }
}
