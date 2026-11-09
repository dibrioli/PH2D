//! **P14 «L» — o receptor com normal** (`docs/PixelArt/P14_plano.md` §4/§7): a colagem do vetor com a
//! BANDA NORMAL lê `(F + n_xy·M)/2π`. Aqui `F` e `M` são texturas SINTÉTICAS (o receptor isolado do
//! solver): `M = (0,4·2π, 0)` é o momento de uma luz que vem da DIREITA.
//!
//! - sem normal (banda de alfa 0, ou a colagem sem banda) o quadro é o da colagem de hoje, byte a byte;
//! - `n = (1, 0)` mais claro que de frente, de frente mais claro que `n = (−1, 0)`.

use std::collections::BTreeSet;
use std::f64::consts::TAU;

use ph2d_gpu::GpuContext;
use ph2d_render::tonemap::Tonemap;
use ph2d_render::{BandBlit, BandSource, GameRt, LuzRecebida, WorldRt};

const W: u32 = 16;
const H: u32 = 16;
const TAM: [u32; 2] = [4, 4];
/// `luz_em` de frente (por canal) e `|M|/2π` (a luz da direita).
const DE_FRENTE: f64 = 0.5;
const MOMENTO_X: f64 = 0.4;

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn textura(
    gpu: &GpuContext,
    formato: wgpu::TextureFormat,
    tam: [u32; 2],
    dados: &[u8],
) -> wgpu::Texture {
    let bpp = formato.block_copy_size(None).expect("formato simples");
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("p14 receptor"),
        size: wgpu::Extent3d {
            width: tam[0],
            height: tam[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: formato,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        t.as_image_copy(),
        dados,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(tam[0] * bpp),
            rows_per_image: Some(tam[1]),
        },
        t.size(),
    );
    t
}

fn campo(gpu: &GpuContext, valor: f64) -> wgpu::Texture {
    let mut d = Vec::new();
    for _ in 0..TAM[0] * TAM[1] {
        for c in [valor, valor, valor, 1.0] {
            d.extend_from_slice(&half::f16::from_f64(c).to_le_bytes());
        }
    }
    textura(gpu, wgpu::TextureFormat::Rgba16Float, TAM, &d)
}

fn uniforme(gpu: &GpuContext, rgba: [u8; 4]) -> wgpu::Texture {
    let d: Vec<u8> = (0..W * H).flat_map(|_| rgba).collect();
    textura(gpu, wgpu::TextureFormat::Rgba8Unorm, [W, H], &d)
}

fn ler(gpu: &GpuContext, t: &wgpu::Texture) -> Vec<u8> {
    let padded =
        (W * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("p14 leitura"),
        size: u64::from(padded * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(H),
            },
        },
        t.size(),
    );
    gpu.queue.submit([enc.finish()]);
    let fatia = buf.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let m = fatia.get_mapped_range().expect("mapped range");
    (0..H as usize)
        .flat_map(|y| m[y * padded as usize..][..(W * 4) as usize].to_vec())
        .collect()
}

/// Como a colagem é chamada.
#[derive(Clone, Copy)]
enum Colagem {
    /// A de hoje (`blit_rect`), luz sem momento.
    DeHoje,
    /// Luz COM momento, colagem sem banda.
    SemBanda,
    /// Luz com momento e a banda normal destes bytes.
    Banda([u8; 4]),
}

fn quadro(gpu: &GpuContext, como: Colagem) -> Vec<u8> {
    let game = GameRt::new(gpu, (W, H));
    let tonemap = Tonemap::new(gpu, game.texture().create_view(&Default::default()), (W, H));
    let alvo = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("p14 alvo"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: WorldRt::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let alvo_v = alvo.create_view(&Default::default());
    let src = uniforme(gpu, [150, 150, 150, 255]);
    let src_v = src.create_view(&Default::default());
    let f = campo(gpu, DE_FRENTE * TAU);
    let mx = campo(gpu, MOMENTO_X * TAU);
    let my = campo(gpu, 0.0);
    let (fv, mxv, myv) = (
        f.create_view(&Default::default()),
        mx.create_view(&Default::default()),
        my.create_view(&Default::default()),
    );
    let ninguem = BTreeSet::new();
    let luz = LuzRecebida {
        fluencia: &fv,
        origem_m: [0.0, 0.0],
        texel_m: 1.0,
        tamanho: TAM,
        sem_luz: &ninguem,
        momento: (!matches!(como, Colagem::DeHoje)).then_some([&mxv, &myv]),
    };
    let mut colagem = BandBlit::new(gpu, WorldRt::FORMAT);
    let tela = [[0.25, 0.0, 0.0], [0.0, 0.25, 0.0]];
    colagem.receber_luz(gpu, Some((&luz, tela, &tonemap)));
    match como {
        Colagem::DeHoje | Colagem::SemBanda => {
            colagem.blit_rect(gpu, &alvo_v, &src_v, BandSource::Vector, None);
        }
        Colagem::Banda(n) => {
            let banda = uniforme(gpu, n);
            let bv = banda.create_view(&Default::default());
            colagem.blit_com_normal(gpu, &alvo_v, &src_v, &bv, BandSource::Vector, None);
        }
    }
    ler(gpu, &alvo)
}

fn verde_medio(bytes: &[u8]) -> f64 {
    // BGRA: o verde é o byte 1.
    let n = bytes.len() / 4;
    bytes.chunks(4).map(|p| f64::from(p[1])).sum::<f64>() / n as f64
}

#[test]
fn a_normal_de_frente_e_o_receptor_de_hoje_byte_a_byte() {
    let Some(gpu) = gpu() else {
        eprintln!("p14: sem placa — saltado");
        return;
    };
    let hoje = quadro(&gpu, Colagem::DeHoje);
    assert_eq!(
        quadro(&gpu, Colagem::SemBanda),
        hoje,
        "o momento ligado sem banda mudou o quadro"
    );
    assert_eq!(
        quadro(&gpu, Colagem::Banda([255, 128, 255, 0])),
        hoje,
        "a banda de alfa 0 (sem normal) mudou o quadro"
    );
}

#[test]
fn a_lampada_a_direita_acende_a_normal_a_direita() {
    let Some(gpu) = gpu() else {
        eprintln!("p14: sem placa — saltado");
        return;
    };
    let plano = verde_medio(&quadro(&gpu, Colagem::DeHoje));
    let direita = verde_medio(&quadro(&gpu, Colagem::Banda([255, 128, 255, 255])));
    let esquerda = verde_medio(&quadro(&gpu, Colagem::Banda([0, 128, 255, 255])));
    eprintln!("p14 receptor: n=(1,0) {direita:.1} · plano {plano:.1} · n=(−1,0) {esquerda:.1}");
    assert!(
        direita > plano + 10.0 && plano > esquerda + 10.0,
        "a ordem da luz pela normal falhou: {direita:.1} / {plano:.1} / {esquerda:.1}"
    );
}
