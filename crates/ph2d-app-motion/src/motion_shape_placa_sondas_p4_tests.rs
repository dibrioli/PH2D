//! ⭐ **As sondas que decidem o P4 (b) e o `Scene` no dispositivo** (doc 132 §4.1, §7 P4) — manuais,
//! `--release --nocapture`, com placa: variantes no MESMO processo, blocos intercalados com a ordem
//! rodada, o MÍNIMO, a carga ao lado (`MEDIR_VELOCIDADE.md`). Cada quadro espera a placa.

use ph2d_eval_motion::{BlendWith, MisturaDoSink, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_vector::Affine;

use super::{GeometriasDaPlaca, Misturas, PlacaDeFormas};
use crate::motion_shape_gen::VecPathStore;

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn espera(gpu: &GpuContext) {
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}

/// `n` cópias `24×24` px numa grelha sobre `(w, h)`, forma ou imagem do átlas, com a mistura dada.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "uma grelha pequena"
)]
fn copias(
    n: usize,
    (w, h): (u32, u32),
    de: impl Fn(usize) -> (bool, MisturaDoSink),
) -> Vec<VectorInstance> {
    let colunas = ((n as f32).sqrt().ceil() as usize).max(1);
    (0..n)
        .map(|k| {
            let (imagem, mistura) = de(k);
            let (x, y) = (
                (k % colunas) as f32 / colunas as f32 * w as f32 * 0.9 + 20.0,
                (k / colunas) as f32 / colunas as f32 * h as f32 * 0.9 + 20.0,
            );
            VectorInstance {
                geometry_id: u32::from(!imagem),
                texture_id: 0,
                atlas_uv: [0.0, 0.0, 0.25, 0.25],
                premultiplied: 0.0,
                world_pos: [x, y],
                size: [24.0, 24.0],
                basis: [1.0, 0.0, 0.0, 1.0],
                tint: [0.6, 0.5, 0.4, 0.8],
                anchor: [0.0, 0.0],
                sampling: 0,
                blend_linha: 0,
                mistura,
            }
        })
        .collect()
}

/// Um quadro da rota da CPU da placa: a decisão, as fontes, o desenho e a colagem segmento a segmento.
fn quadro(
    gpu: &GpuContext,
    renderer: &ph2d_render::SpriteRenderer,
    blit: &ph2d_render::BandBlit,
    mundo: &ph2d_render::WorldRt,
    insts: &[VectorInstance],
    placa: &mut PlacaDeFormas,
    tamanho: (u32, u32),
) {
    let mut store = VecPathStore::default();
    let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    let mut geos = GeometriasDaPlaca::default();
    assert!(placa.decide_com_misturas(
        true,
        insts,
        &store,
        &mut geos,
        Affine::IDENTITY,
        Misturas::do_quadro(gpu, false),
    ));
    let m = crate::motion_state::MotionState::new();
    let fontes = super::imagens::fontes_do_quadro(
        placa,
        gpu,
        &m,
        renderer,
        ph2d_render::ImageFilterMode::Smooth,
    );
    placa.com_imagens(fontes);
    let _ = placa.desenha(gpu, tamanho, &geos, None);
    super::cola_no_mundo(placa, gpu, blit, mundo.blend_view());
    espera(gpu);
}

/// Mede `variantes` (blocos de 10 quadros, 12 blocos intercalados com a ordem rodada) — o mínimo.
fn mede(variantes: &mut [&mut dyn FnMut()]) -> Vec<f64> {
    for v in variantes.iter_mut() {
        for _ in 0..5 {
            v();
        }
    }
    let n = variantes.len();
    let mut min = vec![f64::MAX; n];
    for b in 0..12 {
        for k in 0..n {
            let i = (b + k) % n;
            let t = std::time::Instant::now();
            for _ in 0..10 {
                variantes[i]();
            }
            min[i] = min[i].min(t.elapsed().as_secs_f64() * 1e3 / 10.0);
        }
    }
    min
}

/// ⭐ **P4 (b) — o preço de um SEGMENTO de pousa** a 1080p e a 4K: as mesmas 1 024 cópias numa pousa só
/// (todas `Everything · Soma`) e em dois segmentos (metade `Everything · Soma`, metade `Normal`).
#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn sonda_o_preco_de_um_segmento_de_pousa() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        return;
    };
    println!("\ncarga: {}", carga());
    let renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let blit = ph2d_render::BandBlit::new(&gpu, ph2d_render::WorldRt::FORMAT);
    let soma = MisturaDoSink {
        blend: 1,
        com: BlendWith::Everything,
        sink: 0,
    };
    let normal = MisturaDoSink {
        blend: 0,
        com: BlendWith::Copies,
        sink: 1,
    };
    for (nome, tamanho) in [("1080p", (1920u32, 1080u32)), ("4K", (3840, 2160))] {
        let mundo = ph2d_render::WorldRt::new(&gpu, tamanho);
        let uma = copias(1024, tamanho, |_| (false, soma));
        let duas = copias(1024, tamanho, |k| {
            (false, if k < 512 { soma } else { normal })
        });
        let (mut p1, mut p2) = (PlacaDeFormas::default(), PlacaDeFormas::default());
        let mut a = || quadro(&gpu, &renderer, &blit, &mundo, &uma, &mut p1, tamanho);
        let mut b = || quadro(&gpu, &renderer, &blit, &mundo, &duas, &mut p2, tamanho);
        let min = mede(&mut [&mut a, &mut b]);
        println!(
            "  {nome}: 1 segmento {:.3} ms · 2 segmentos {:.3} ms · o segmento a mais {:+.3} ms · carga {}",
            min[0],
            min[1],
            min[1] - min[0],
            carga()
        );
    }
}

/// ⭐ **O `Scene` no dispositivo — o preço da ORDEM das linhas** (doc 132 §4.1): a rota da CPU sabe a
/// média de cada linha e desenha uma corrida por troço de média; o dispositivo não a sabe no host, e a
/// única composição exacta sem a ler seria um desenho por linha. Mede-se esse pior caso pela rota da
/// CPU (as médias a alternar linha a linha: `N` corridas) contra as mesmas linhas agrupadas por média
/// (2 corridas), a 1k/4k/16k, num `Scene · Soma` a 1080p.
#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn sonda_o_preco_de_um_desenho_por_linha() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        return;
    };
    println!("\ncarga: {}", carga());
    let renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let blit = ph2d_render::BandBlit::new(&gpu, ph2d_render::WorldRt::FORMAT);
    let tamanho = (1920u32, 1080u32);
    let mundo = ph2d_render::WorldRt::new(&gpu, tamanho);
    let cena = MisturaDoSink {
        blend: 1,
        com: BlendWith::Scene,
        sink: 0,
    };
    for n in [1024usize, 4096, 16384] {
        let alterna = copias(n, tamanho, |k| (k % 2 == 1, cena));
        let agrupa = copias(n, tamanho, |k| (k >= n / 2, cena));
        let (mut p1, mut p2) = (PlacaDeFormas::default(), PlacaDeFormas::default());
        let mut a = || quadro(&gpu, &renderer, &blit, &mundo, &alterna, &mut p1, tamanho);
        let mut b = || quadro(&gpu, &renderer, &blit, &mundo, &agrupa, &mut p2, tamanho);
        let min = mede(&mut [&mut a, &mut b]);
        #[expect(clippy::cast_precision_loss, reason = "uma contagem")]
        let por_linha = (min[0] - min[1]) / n as f64;
        println!(
            "  {n:>6} linhas: um desenho por linha {:.3} ms · por média {:.3} ms · {:+.4} ms por linha · carga {}",
            min[0],
            min[1],
            por_linha,
            carga()
        );
    }
}

/// Os dois buffers do COZIMENTO para `insts` (a faixa inteira em cada: a linha de imagem sem
/// geometria nas formas, a de forma com tamanho zero nas imagens).
fn cozimento_alinhado(gpu: &GpuContext, insts: &[VectorInstance]) -> (wgpu::Buffer, wgpu::Buffer) {
    let formas: Vec<ph2d_shape_gpu::ShapeInstance> = insts
        .iter()
        .map(|i| ph2d_shape_gpu::ShapeInstance {
            pos: i.world_pos,
            size: i.size,
            basis: i.basis,
            anchor: i.anchor,
            geometry: if i.geometry_id > 0 {
                i.geometry_id
            } else {
                ph2d_gpu_cook::lower_forma::SEM_GEOMETRIA
            },
            _pad: 0,
            tint: i.tint,
        })
        .collect();
    let linhas: Vec<ph2d_render::RenderInstance> = insts
        .iter()
        .map(|i| {
            let mut l = super::linha_da_imagem(i);
            if i.geometry_id > 0 {
                l.size = [0.0, 0.0];
            }
            l
        })
        .collect();
    let buffer = |bytes: &[u8]| {
        let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sonda cozimento alinhado"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&b, 0, bytes);
        b
    };
    (
        buffer(bytemuck::cast_slice(&formas)),
        buffer(bytemuck::cast_slice(&linhas)),
    )
}

/// Um quadro da rota do DISPOSITIVO sobre os buffers do cozimento: as corridas pela porta do produto
/// (`corridas_das_faixas`), a decisão, o desenho e a colagem.
#[expect(clippy::too_many_arguments, reason = "um quadro inteiro")]
fn quadro_do_dispositivo(
    gpu: &GpuContext,
    renderer: &ph2d_render::SpriteRenderer,
    blit: &ph2d_render::BandBlit,
    mundo: &ph2d_render::WorldRt,
    (formas, linhas): &(wgpu::Buffer, wgpu::Buffer),
    n: u32,
    mistura: MisturaDoSink,
    placa: &mut PlacaDeFormas,
    tamanho: (u32, u32),
) {
    let mut store = VecPathStore::default();
    let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    let mut geos = GeometriasDaPlaca::default();
    let sink = ph2d_nodegraph::graph::NodeId(0);
    let corridas = super::corridas_das_faixas(
        &[(sink, 0..n, 0..n)],
        |_| mistura,
        Misturas::do_quadro(gpu, false),
    )
    .expect("a placa compõe-na");
    assert!(placa.decide_do_dispositivo_com_pousa(
        true,
        n,
        &[1],
        &store,
        &mut geos,
        Affine::IDENTITY,
        corridas,
    ));
    let atlas = renderer
        .texture_view_and_dims(0)
        .expect("o átlas")
        .0
        .clone();
    placa.com_imagens(Some(super::imagens::FontesDasImagens {
        buffer: linhas.clone(),
        linhas: n,
        texturas: vec![(0..n, atlas)],
        projeto: ph2d_render::ImageFilterMode::Smooth,
    }));
    let _ = placa.desenha(gpu, tamanho, &geos, Some(formas));
    super::cola_no_mundo(placa, gpu, blit, mundo.blend_view());
    espera(gpu);
}

/// ⭐⭐ doc 132 §4.1 (3) — **o `Scene` com imagens NO DISPOSITIVO: o preço da corrida ORDENADA** (um
/// desenho na ordem das linhas) contra a lei de antes (a rota da CPU, uma corrida por troca de média:
/// o pior caso, as médias a alternar linha a linha) e contra a corrida COMUTATIVA do dispositivo
/// (`Copies · Soma`: formas e depois imagens, dois desenhos — o preço da placa), a 1k/4k/16k a 1080p.
#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn sonda_o_scene_ordenado_no_dispositivo() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        return;
    };
    println!("\ncarga: {}", carga());
    let renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let blit = ph2d_render::BandBlit::new(&gpu, ph2d_render::WorldRt::FORMAT);
    let tamanho = (1920u32, 1080u32);
    let mundo = ph2d_render::WorldRt::new(&gpu, tamanho);
    let cena = MisturaDoSink {
        blend: 1,
        com: BlendWith::Scene,
        sink: 0,
    };
    let soma = MisturaDoSink {
        blend: 1,
        com: BlendWith::Copies,
        sink: 0,
    };
    for n in [1024usize, 4096, 16384] {
        let alterna = copias(n, tamanho, |k| (k % 2 == 1, cena));
        let cozido = cozimento_alinhado(&gpu, &alterna);
        let k = u32::try_from(n).expect("poucas");
        let (mut p1, mut p2, mut p3) = (
            PlacaDeFormas::default(),
            PlacaDeFormas::default(),
            PlacaDeFormas::default(),
        );
        let mut a = || quadro(&gpu, &renderer, &blit, &mundo, &alterna, &mut p1, tamanho);
        let mut b = || {
            quadro_do_dispositivo(
                &gpu, &renderer, &blit, &mundo, &cozido, k, cena, &mut p2, tamanho,
            );
        };
        let mut c = || {
            quadro_do_dispositivo(
                &gpu, &renderer, &blit, &mundo, &cozido, k, soma, &mut p3, tamanho,
            );
        };
        let min = mede(&mut [&mut a, &mut b, &mut c]);
        println!(
            "  {n:>6} linhas: por linha (CPU) {:.3} ms · ORDENADO {:.3} ms · comutativa {:.3} ms · \
             ordenado/comutativa {:.2}× · carga {}",
            min[0],
            min[1],
            min[2],
            min[1] / min[2],
            carga()
        );
    }
}
