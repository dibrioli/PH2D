//! ⏱️⏱️ **A W0-b DA F9 — O TECTO DA PLACA: quantos triângulos de pele cabem num quadro?**
//!
//! Rodar: `cargo test -p ph2d-render --test it -- --ignored --nocapture skin_mesh_gpu_ceiling`
//!
//! # ⭐⭐⭐ A pergunta da fila estava certa e a PREMISSA dela não
//!
//! `docs/Skeleton/01_a_fila.md` (F9 W0) manda medir *«o custo GPU real de `10⁴`–`10⁶` triângulos
//! deformados no *vertex shader* nesta máquina»*. Ao levantar as costuras (a outra metade da W0)
//! apareceu o facto que reescreve a pergunta:
//!
//! > **A malha JÁ vai para a placa em todos os quadros.** Desde que a pele entrou no passe de
//! > sprites, o [`ph2d_render::SpriteMesh`] é copiado para um buffer e desenhado por
//! > `renderer_draw` — a CPU posa **e faz upload** de `N` vértices por quadro.
//!
//! ⇒ a F9 **não acrescenta** um desenho de `N` triângulos: ele já acontece. O que ela TIRA da CPU é
//! (a) a deformação por vértice e (b) o upload por quadro, trocando-o por `N_ossos × 6` números.
//!
//! ⇒ logo o número decisivo desta sonda não é *«a placa aguenta?»* e sim **onde o passe que já
//! existe deixa de caber num quadro** — porque é esse o tecto que a densidade assada no bind (W1)
//! não pode ultrapassar, com ou sem *vertex shader*.
//!
//! ⚠️ **Ela mede o passe REAL** (`SpriteRenderer::render` sobre um alvo offscreen), não um
//! sucedâneo: *uma sonda que mede um sucedâneo mede outro programa para sempre* — a lei que esta
//! casa pagou quando a porta de cópia dizia `17 %` e a do produto dizia `32 %`.
//!
//! ⚠️ **O relógio desta workstation não vale nada acima de `load ~5`** (`CLAUDE.md` §5.0): a sonda
//! imprime o `loadavg` ao lado de cada linha e toma o **MÍNIMO** de várias corridas.

use std::time::Instant;

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, RenderInstance, SpriteMesh, SpriteRenderer, TextureAtlas};

const W: u32 = 512;
const H: u32 = 512;
const BLACK: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

fn try_headless_gpu() -> Option<GpuContext> {
    let instance = GpuContext::default_instance();
    GpuContext::new(instance, None).ok()
}

fn carga() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}

fn alvo(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("alvo da sonda"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Uma instância que cobre o alvo — o sujeito cuja malha substitui o quad.
fn instancia(texture_id: u32) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: [8.0, 8.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: RenderInstance::pack_blend_bits(0),
        texture_id,
        z_order: 0,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// Uma grelha `n × n` de quads sobre o quad de repouso ⇒ `2·n²` triângulos.
///
/// ⚠️ **Ela cobre o alvo inteiro de propósito:** uma malha fora do ecrã seria descartada cedo e a
/// sonda mediria o recorte, não o desenho — *uma medição cujo sujeito não é rasterizado mede zero
/// com um número plausível ao lado*.
fn grelha(n: usize) -> SpriteMesh {
    let anchor = [0.0_f32, 0.0];
    let size = [8.0_f32, 8.0];
    let lado = n + 1;
    let mut local = Vec::with_capacity(lado * lado);
    let mut uv = Vec::with_capacity(lado * lado);
    for j in 0..lado {
        for i in 0..lado {
            let fx = i as f32 / n as f32;
            let fy = j as f32 / n as f32;
            let p = [
                anchor[0] + (fx - 0.5) * size[0],
                anchor[1] + (fy - 0.5) * size[1],
            ];
            local.push(p);
            uv.push(SpriteMesh::uv_at(p, anchor, size).expect("size nao nulo"));
        }
    }
    let mut tris = Vec::with_capacity(n * n * 2);
    for j in 0..n {
        for i in 0..n {
            let a = (j * lado + i) as u32;
            let (b, c, d) = (a + 1, a + lado as u32, a + lado as u32 + 1);
            tris.push([a, b, c]);
            tris.push([b, d, c]);
        }
    }
    SpriteMesh { local, uv, tris }
}

/// ⏱️ **O TECTO, medido no passe real.**
#[test]
#[ignore = "GPU: pede adaptador; e' uma MEDICAO, nao um gate"]
fn skin_mesh_gpu_ceiling() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem GPU headless — sonda saltada");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let px: Vec<u8> = (0..16 * 16 * 4).map(|i| (i % 251) as u8).collect();
    let tex = renderer.acquire_individual(16, 16, &px).expect("textura");
    let t = alvo(&gpu);
    let view = t.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 64.0);
    let janela = WindowSize::new(W, H);

    println!("\n  TRIANGULOS |   VERTICES |  upload/quadro |   quadro | vs 16,67 ms | load");
    println!("  -----------+------------+----------------+----------+-------------+------");
    // `n` tal que `2n²` cubra a faixa que a fila pede: 10⁴ .. ~4·10⁶.
    for n in [71_usize, 224, 707, 1414] {
        let malha = grelha(n);
        let tris = malha.tris.len();
        let verts = malha.local.len();
        // O que a CPU manda para a placa HOJE, por quadro: posicoes + uv + indices.
        let bytes = verts * 2 * 4 * 2 + tris * 3 * 4;
        let mut present = PresentWorld::new();
        present.world_mut().spawn((instancia(tex), malha));
        // Aquece (a 1.ª corrida paga a alocacao dos buffers), depois o MINIMO de cinco.
        for _ in 0..2 {
            renderer.render(&view, &mut present, &camera, janela, BLACK);
        }
        let mut melhor = f64::INFINITY;
        for _ in 0..5 {
            let t0 = Instant::now();
            renderer.render(&view, &mut present, &camera, janela, BLACK);
            gpu.device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("poll");
            melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0);
        }
        println!(
            "  {tris:>10} | {verts:>10} | {:>10.2} KiB | {melhor:>6.2} ms | {:>10.1}% | {:.2}",
            bytes as f64 / 1024.0,
            melhor / 16.67 * 100.0,
            carga()
        );
    }
    println!(
        "\n  ⇒ o upload por quadro e' o que a F9 TROCA por `N_ossos x 6` numeros.\n\
           ⇒ o quadro e' o tecto que a densidade assada no bind (W1) nao pode passar.\n"
    );
}
