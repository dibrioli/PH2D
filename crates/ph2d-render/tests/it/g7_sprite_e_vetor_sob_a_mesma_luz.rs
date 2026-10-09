//! **G7 — um sprite e um vetor da MESMA cor, sob a MESMA luz, dão o MESMO pixel** (plano
//! `docs/Luz2D/01_plano.md` §2.4/§5).
//!
//! O sprite recebe a luz no passe dele (HDR, antes do tonemap) e o vetor — que o Vello só escreve em
//! `Rgba8Unorm` — na colagem da faixa (`BandBlit`), com a MESMA conta: `linear(albedo) × luz_em →
//! tonemap_mapear → sRGB`. Lado a lado sob uma luz colorida simétrica em `x` e em gradiente em `y`;
//! o pixel do sprite é comparado com o ESPELHADO do vetor, e o vetor com a conta da CPU (que pega uma
//! troca de eixo que a simetria esconderia). O vetor desenha-se pelo Vello de verdade (a porta do
//! documento), nunca por bytes sintéticos.
//!
//! ⛔ A colagem sem o caminho do vetor reprova: o quadrado vetorial sai na cor de sempre.

use std::collections::BTreeSet;
use std::f64::consts::TAU;

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::tonemap::Tonemap;
use ph2d_render::{
    BandBlit, BandSource, Camera2d, GameRt, LiftedInstances, LuzRecebida, RenderInstance,
    SpriteRenderer, TextureAtlas, VelloPass, WorldRt,
};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};

const W: u32 = 64;
const H: u32 = 32;
/// A cor dos dois quadrados (bytes sRGB, opaca) e o fundo do acumulador.
const COR: [u8; 4] = [200, 120, 60, 255];
const FUNDO: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.02,
    a: 1.0,
};
/// A grade de luz: 8×4 texels de 0,5 m a partir de `(−2, −1)` — cobre a vista inteira.
const ORIGEM: [f64; 2] = [-2.0, -1.0];
const TEXEL: f64 = 0.5;
const TAM: [u32; 2] = [8, 4];

/// **Tolerância MEDIDA** (RTX 5060 Ti, Vulkan, 2026-10-09): ver o handoff W3 e o número impresso
/// por este teste. O sprite passa por `f16` (o `game_rt`) e pela descodificação sRGB do HARDWARE;
/// o vetor pela curva ideal — `tonemap.wgsl` já mede a tabela da placa a afastar-se da ideal.
const TOL_BYTES: u8 = 1;

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// A luz RECEBIDA no centro do texel de linha `j` (a da CPU e a da placa partem do mesmo número):
/// uniforme em `x`, linear em `y`, colorida.
fn luz_na_linha(gradiente: bool, y_m: f64) -> [f64; 3] {
    let k = if gradiente { 1.0 + 0.4 * y_m } else { 1.0 };
    [0.9 * k, 0.55 * k, 0.3 * k]
}

fn textura_de_fluencia(gpu: &GpuContext, gradiente: bool) -> wgpu::Texture {
    let mut dados = Vec::new();
    for j in 0..TAM[1] {
        let y = ORIGEM[1] + (f64::from(j) + 0.5) * TEXEL;
        let e = luz_na_linha(gradiente, y);
        for _ in 0..TAM[0] {
            // A textura guarda a FLUÊNCIA: `F = 2π · luz_em`.
            for c in [e[0], e[1], e[2], 0.0] {
                dados.extend_from_slice(&half::f16::from_f64(c * TAU).to_le_bytes());
            }
        }
    }
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("g7 fluencia"),
        size: wgpu::Extent3d {
            width: TAM[0],
            height: TAM[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        t.as_image_copy(),
        &dados,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(TAM[0] * 8),
            rows_per_image: Some(TAM[1]),
        },
        t.size(),
    );
    t
}

fn ler(gpu: &GpuContext, t: &wgpu::Texture) -> Vec<u8> {
    let padded =
        (W * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("g7 leitura"),
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
    let m = fatia.get_mapped_range();
    let mut out = Vec::new();
    for y in 0..H as usize {
        out.extend_from_slice(&m[y * padded as usize..][..(W * 4) as usize]);
    }
    out
}

/// O afim píxel → mundo: o INVERSO do que o Vello usou.
fn tela_para_mundo(a: Affine) -> [[f64; 3]; 2] {
    let [a, b, c, d, e, f] = a.inverse().as_coeffs();
    [[a, c, e], [b, d, f]]
}

fn srgb_dec(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_enc(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// O quadro: o sprite em `(−1, 0)`, o vetor em `(+1, 0)`, 1 m de lado; `vetor_recebe = false` é o
/// controlo (a colagem SEM o caminho do vetor). Devolve os bytes BGRA crus do acumulador.
fn quadro(gpu: &GpuContext, gradiente: bool, vetor_recebe: bool) -> Vec<u8> {
    let janela = WindowSize::new(W, H);
    let camera = Camera2d::new([0.0, 0.0], 2.0);
    let fluencia = textura_de_fluencia(gpu, gradiente);
    let vista = fluencia.create_view(&Default::default());
    let ninguem = BTreeSet::new();
    let luz = LuzRecebida {
        fluencia: &vista,
        origem_m: ORIGEM,
        texel_m: TEXEL,
        tamanho: TAM,
        sem_luz: &ninguem,
    };

    // O sprite, pelo passe da CENA (`render_with_streams`), no HDR.
    let game = GameRt::new(gpu, (W, H));
    let mut renderer = SpriteRenderer::new(
        gpu.clone(),
        wgpu::TextureFormat::Rgba16Float,
        TextureAtlas::new(gpu, 256),
        8,
    );
    let tex = renderer
        .acquire_individual(1, 1, &COR)
        .expect("textura do sprite");
    let mut inst = RenderInstance {
        world_pos: [-1.0, 0.0],
        size: [1.0, 1.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id: tex,
        z_order: 0,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    inst.flip_uv = 0;
    let mut extra = LiftedInstances::default();
    extra.push(inst, None);
    renderer.receber_luz(Some(&luz));
    renderer.render_with_streams(
        game.view(),
        &mut PresentWorld::new(),
        &camera,
        janela,
        wgpu::Color::TRANSPARENT,
        &extra,
        None,
        None,
        None,
        None,
    );
    let tonemap = Tonemap::new(gpu, game.texture().create_view(&Default::default()), (W, H));
    tonemap.run(gpu);

    // O acumulador do mundo: a faixa de sprites, depois a do vetor.
    let mundo = WorldRt::new(gpu, (W, H));
    mundo.clear_linear(gpu, FUNDO);
    let mut colagem = BandBlit::new(gpu, WorldRt::FORMAT);
    let afim = camera.world_to_screen_affine(janela);
    colagem.receber_luz(
        gpu,
        vetor_recebe.then_some((&luz, tela_para_mundo(afim), &tonemap)),
    );
    colagem.blit(
        gpu,
        mundo.blend_view(),
        tonemap.output_view(),
        BandSource::Sprites,
    );
    let mut vello =
        VelloPass::new(gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (W, H)).expect("vello");
    let mut cena = vello::Scene::new();
    cena.fill(
        Fill::NonZero,
        afim,
        Color::from_rgba8(COR[0], COR[1], COR[2], COR[3]),
        None,
        &Rect::new(0.5, -0.5, 1.5, 0.5),
    );
    vello
        .render_to_intermediate(gpu, &cena, (W, H), Color::TRANSPARENT)
        .expect("vello render");
    colagem.blit(
        gpu,
        mundo.blend_view(),
        vello.intermediate_view(),
        BandSource::Vector,
    );
    ler(gpu, mundo.texture())
}

/// `(rgb do sprite em (x, y), rgb do vetor no espelho, rgb esperado do vetor pela CPU)` no
/// interior dos dois quadrados (a borda de 1 px fica fora: AA do Vello × aresta do sprite).
fn pares(bgra: &[u8], gradiente: bool) -> Vec<([u8; 3], [u8; 3], [f64; 3])> {
    let rgb = |x: u32, y: u32| {
        let i = ((y * W + x) * 4) as usize;
        [bgra[i + 2], bgra[i + 1], bgra[i]]
    };
    let mut out = Vec::new();
    for y in 9..23 {
        for x in 9..23 {
            let xv = W - 1 - x;
            // O centro do píxel do vetor no mundo (16 px/m, y para cima).
            let ym = (f64::from(H) / 2.0 - (f64::from(y) + 0.5)) / 16.0;
            let e = luz_na_linha(gradiente, ym);
            let esperado: [f64; 3] = std::array::from_fn(|k| {
                255.0 * srgb_enc(srgb_dec(f64::from(COR[k]) / 255.0) * e[k])
            });
            out.push((rgb(x, y), rgb(xv, y), esperado));
        }
    }
    out
}

#[test]
fn g7_sprite_e_vetor_da_mesma_cor_dao_o_mesmo_pixel_sob_a_mesma_luz() {
    let Some(gpu) = gpu() else {
        eprintln!("g7: sem placa — saltado");
        return;
    };
    for gradiente in [false, true] {
        let bgra = quadro(&gpu, gradiente, true);
        let (mut pior_par, mut pior_cpu) = (0u8, 0.0_f64);
        for (s, v, cpu) in pares(&bgra, gradiente) {
            for k in 0..3 {
                pior_par = pior_par.max(s[k].abs_diff(v[k]));
                pior_cpu = pior_cpu.max((f64::from(v[k]) - cpu[k]).abs());
            }
        }
        eprintln!(
            "g7 (gradiente = {gradiente}): sprite × vetor espelhado max {pior_par} B; vetor × CPU \
             max {pior_cpu:.3} B"
        );
        assert!(
            pior_par <= TOL_BYTES,
            "sprite e vetor da MESMA cor sob a MESMA luz diferem {pior_par} B (> {TOL_BYTES})"
        );
        assert!(
            pior_cpu <= f64::from(TOL_BYTES) + 0.5,
            "o vetor aceso afasta-se da conta da CPU {pior_cpu:.3} B — a transformação píxel → \
             mundo ou a conta divergiu"
        );
    }
}

/// O CONTROLO: a colagem sem o caminho do vetor (o vetor na cor de sempre) TEM de reprovar a
/// igualdade — senão o gate acima não mede nada.
#[test]
fn g7_sem_o_caminho_do_vetor_os_dois_diferem() {
    let Some(gpu) = gpu() else {
        eprintln!("g7: sem placa — saltado");
        return;
    };
    let bgra = quadro(&gpu, true, false);
    let pior = pares(&bgra, true)
        .iter()
        .flat_map(|(s, v, _)| (0..3).map(move |k| s[k].abs_diff(v[k])))
        .max()
        .unwrap_or(0);
    assert!(
        pior > 20,
        "sem o caminho do vetor o quadrado vetorial devia sair na cor de sempre (diferença {pior} B)"
    );
}

/// ⭐ **O passe ISOLADO não recebe a luz** (o brilho do `present_fx`: a emissão É luz). Com a luz
/// acesa no renderer, `render_lifted_instances(.., com_luz = false)` tem de dar os bytes de um
/// renderer que nunca a viu; o controlo (`com_luz = true`) tem de diferir. Apanha a guarda
/// `com_luz` do grupo (mutação M2 da W3, que nenhum outro gate via).
#[test]
fn o_passe_isolado_do_brilho_nao_recebe_a_luz() {
    let Some(gpu) = gpu() else {
        eprintln!("g7: sem placa — saltado");
        return;
    };
    let fluencia = textura_de_fluencia(&gpu, true);
    let vista = fluencia.create_view(&Default::default());
    let ninguem = BTreeSet::new();
    let luz = LuzRecebida {
        fluencia: &vista,
        origem_m: ORIGEM,
        texel_m: TEXEL,
        tamanho: TAM,
        sem_luz: &ninguem,
    };
    let desenhar = |acesa: bool, com_luz: bool| {
        let game = GameRt::new(&gpu, (W, H));
        let mut r = SpriteRenderer::new(
            gpu.clone(),
            wgpu::TextureFormat::Rgba16Float,
            TextureAtlas::new(&gpu, 256),
            8,
        );
        let tex = r.acquire_individual(1, 1, &COR).expect("textura");
        let mut inst = RenderInstance {
            world_pos: [0.0, 0.0],
            size: [2.0, 1.5],
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0; 4],
            basis: RenderInstance::IDENTITY_BASIS,
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            texture_id: tex,
            z_order: 0,
            sampling: RenderInstance::SAMPLING_DEFAULT,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
            sub_order: 0,
        };
        inst.flip_uv = 0;
        let mut levantadas = LiftedInstances::default();
        levantadas.push(inst, None);
        if acesa {
            r.receber_luz(Some(&luz));
        }
        r.render_lifted_instances(
            game.view(),
            &Camera2d::new([0.0, 0.0], 2.0),
            WindowSize::new(W, H),
            wgpu::Color::TRANSPARENT,
            &levantadas,
            None,
            com_luz,
        );
        ler_bytes(&gpu, game.texture(), 8)
    };
    let nunca = desenhar(false, false);
    assert_eq!(
        desenhar(true, false),
        nunca,
        "o passe isolado (com_luz = false) recebeu a luz acesa no renderer"
    );
    assert_ne!(
        desenhar(true, true),
        nunca,
        "controlo: com_luz = true sob a luz acesa tem de mudar os bytes"
    );
}

/// Lê uma textura de `bpp` bytes por texel (linhas contíguas).
fn ler_bytes(gpu: &GpuContext, t: &wgpu::Texture, bpp: u32) -> Vec<u8> {
    let (w, h) = (t.width(), t.height());
    let padded =
        (w * bpp).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("g7 leitura isolada"),
        size: u64::from(padded * h),
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
                rows_per_image: Some(h),
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
    let m = fatia.get_mapped_range();
    (0..h as usize)
        .flat_map(|y| m[y * padded as usize..][..(w * bpp) as usize].to_vec())
        .collect()
}
