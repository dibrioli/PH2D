//! O que os gates partilham: o dispositivo, a leitura de volta, o gabarito na grade, o erro.

use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_gpu::{Grade, Solver};
use ph2d_light2d_ref::fluencia;

/// O dispositivo da casa; `None` (com aviso) numa máquina sem placa.
pub fn gpu() -> Option<GpuContext> {
    match GpuContext::new(GpuContext::default_instance(), None) {
        Ok(g) => Some(g),
        Err(e) => {
            eprintln!("sem GPU ({e}): gate de placa saltado");
            None
        }
    }
}

pub fn tem_f16(gpu: &GpuContext) -> bool {
    gpu.device.features().contains(wgpu::Features::SHADER_F16)
}

/// Os bytes crus de `tex` (linhas sem o enchimento de 256).
pub fn ler_bytes(gpu: &GpuContext, tex: &wgpu::Texture) -> Vec<u8> {
    let (w, h) = (tex.width(), tex.height());
    let bpp = tex.format().block_copy_size(None).expect("formato de cor");
    let linha = (w * bpp).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("teste leitura"),
        size: u64::from(linha * h),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        tex.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(linha),
                rows_per_image: Some(h),
            },
        },
        tex.size(),
    );
    gpu.queue.submit([enc.finish()]);
    let fatia = buf.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |r| r.expect("map da leitura"));
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let dados = fatia.get_mapped_range();
    let util = (w * bpp) as usize;
    let mut out = Vec::with_capacity(util * h as usize);
    for j in 0..h as usize {
        let o = j * linha as usize;
        out.extend_from_slice(&dados[o..o + util]);
    }
    drop(dados);
    buf.unmap();
    out
}

/// A fluência rgb do último quadro, texel a texel (linha a linha).
pub fn ler_fluencia(gpu: &GpuContext, solver: &Solver) -> Vec<[f32; 3]> {
    let tex = solver.fluencia();
    let bytes = ler_bytes(gpu, tex);
    match tex.format() {
        wgpu::TextureFormat::Rgba16Float => bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|t| {
                let c = |k: usize| half::f16::from_le_bytes([t[2 * k], t[2 * k + 1]]).to_f32();
                [c(0), c(1), c(2)]
            })
            .collect(),
        wgpu::TextureFormat::Rgba32Float => bytes
            .as_chunks::<16>()
            .0
            .iter()
            .map(|t| {
                let c = |k: usize| {
                    f32::from_le_bytes([t[4 * k], t[4 * k + 1], t[4 * k + 2], t[4 * k + 3]])
                };
                [c(0), c(1), c(2)]
            })
            .collect(),
        f => panic!("formato de fluência inesperado: {f:?}"),
    }
}

/// O gabarito no centro de cada texel de `g`, em paralelo (a `fluencia` é pura).
pub fn gabarito(cena: &CenaDeLuz, g: &Grade) -> Vec<[f64; 3]> {
    gabarito_por(g, |p| fluencia(cena, p))
}

/// ⭐ W5n — o gabarito com a lei `lei` no centro de cada texel de `g`, em paralelo.
pub fn gabarito_por(g: &Grade, lei: impl Fn([f64; 2]) -> [f64; 3] + Sync) -> Vec<[f64; 3]> {
    let lei = &lei;
    let w = g.largura as usize;
    let n = w * g.altura as usize;
    let fios = std::thread::available_parallelism()
        .map_or(4, |k| k.get())
        .min(16);
    let pedaco = n.div_ceil(fios);
    let mut out = vec![[0.0; 3]; n];
    std::thread::scope(|s| {
        for (k, fatia) in out.chunks_mut(pedaco).enumerate() {
            s.spawn(move || {
                for (o, v) in fatia.iter_mut().enumerate() {
                    let idx = k * pedaco + o;
                    *v = lei(g.centro((idx % w) as u32, (idx / w) as u32));
                }
            });
        }
    });
    out
}

/// ⭐ W5a (F3): o centro do texel `k` de `g` cai DENTRO de um corpo opaco (oclusor de opacidade 1 ou
/// polígono que brilha)? Lá a resposta é a do exterior mais próximo (o preenchimento).
pub fn dentro_de_opaco(cena: &CenaDeLuz, g: &Grade) -> impl Fn(usize) -> bool + use<> {
    dentro_de_opaco_a_menos_de(cena, g, f64::INFINITY)
}

/// [`dentro_de_opaco`] a menos de `texels` texels de `g` da borda dele — a faixa que a bilinear do
/// receptor alcança e a face que a parede mostra, longe do EIXO MEDIAL (onde «a face mais próxima»
/// troca de lado num salto).
pub fn dentro_de_opaco_a_menos_de(
    cena: &CenaDeLuz,
    g: &Grade,
    texels: f64,
) -> impl Fn(usize) -> bool + use<> {
    let mut opacos: Vec<Vec<[f32; 2]>> = cena
        .oclusores
        .iter()
        .filter(|o| o.opacidade >= 1.0)
        .map(|o| o.vertices.clone())
        .collect();
    opacos.extend(cena.emissores.iter().filter_map(|e| match e {
        Emissor::Poligono { vertices, .. } => Some(vertices.clone()),
        _ => None,
    }));
    let g = *g;
    move |k| {
        let w = g.largura as usize;
        let p = g.centro((k % w) as u32, (k / w) as u32);
        let p = [p[0] as f32, p[1] as f32];
        let limite = texels * g.texel_m;
        opacos.iter().any(|v| {
            let mut dentro = false;
            let mut d = f64::INFINITY;
            let mut j = v.len() - 1;
            for i in 0..v.len() {
                let (a, b) = (v[i], v[j]);
                if (a[1] > p[1]) != (b[1] > p[1])
                    && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
                {
                    dentro = !dentro;
                }
                let (ab, ap) = (
                    [f64::from(b[0] - a[0]), f64::from(b[1] - a[1])],
                    [f64::from(p[0] - a[0]), f64::from(p[1] - a[1])],
                );
                let t = ((ap[0] * ab[0] + ap[1] * ab[1])
                    / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-30))
                .clamp(0.0, 1.0);
                d = d.min((ap[0] - ab[0] * t).hypot(ap[1] - ab[1] * t));
                j = i;
            }
            dentro && d <= limite
        })
    }
}

/// `√(média (gpu − ref)²) / média(ref)` sobre os texels de `mascara`, nos três canais.
pub fn rmse_rel(gpu: &[[f32; 3]], r: &[[f64; 3]], mascara: impl Fn(usize) -> bool) -> f64 {
    let (mut se, mut soma, mut n) = (0.0_f64, 0.0_f64, 0_usize);
    for (k, (g, r)) in gpu.iter().zip(r).enumerate() {
        if !mascara(k) {
            continue;
        }
        for c in 0..3 {
            se += (f64::from(g[c]) - r[c]).powi(2);
            soma += r[c];
            n += 1;
        }
    }
    assert!(n > 0 && soma > 0.0, "máscara vazia ou referência escura");
    (se / n as f64).sqrt() / (soma / n as f64)
}

pub fn disco(centro: [f32; 2], raio: f32, radiancia: [f32; 3], alcance_m: f32) -> Emissor {
    Emissor::Disco {
        centro,
        raio,
        radiancia,
        alcance_m,
    }
}

/// Um polígono regular de `lados` lados, girado de `giro` radianos.
pub fn regular(centro: [f32; 2], raio: f32, lados: u32, giro: f32) -> Vec<[f32; 2]> {
    (0..lados)
        .map(|k| {
            let a = giro + std::f32::consts::TAU * k as f32 / lados as f32;
            [centro[0] + raio * a.cos(), centro[1] + raio * a.sin()]
        })
        .collect()
}

pub fn oclusor(vertices: Vec<[f32; 2]>, opacidade: f32) -> Oclusor {
    Oclusor {
        vertices,
        opacidade,
        albedo: [0.0; 3],
    }
}

/// Uma grade quadrada de `lado` texels de `texel_m`, com origem em `(0, 0)`.
pub fn grade(lado: u32, texel_m: f64) -> Grade {
    Grade {
        origem_m: [0.0, 0.0],
        texel_m,
        largura: lado,
        altura: lado,
    }
}
