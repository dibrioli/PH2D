//! ⭐⭐⭐ **A MISTURA DE UM SINK COZIDO NA PLACA CHEGA AO PIXEL?** — a metade que
//! faltava à família do blend, e ela nasceu VERMELHA.
//!
//! O irmão [`super::blend_mode_regression`] prova a tabela pelo caminho da CPU: duas
//! sprites cinzentas, a de cima com um tag de mistura, e o pixel do meio aterra num
//! byte distinto por modo. Esta faz a MESMA cena com a sprite de cima a vir pelo
//! buffer do DISPOSITIVO (`gpu_extra`), que é por onde o cozimento do Motion desenha.
//!
//! ⛔⛔ **O `motion.output` declara um param `blend` com rótulos de artista**, a
//! `sink_style` lê-o, o lowering embala-o em `flip_uv` bits 5-7 e um gate da
//! `ph2d-gpu-cook` afirma que ele chega ao BUFFER. Esta régua pergunta a linha
//! seguinte — *ele chega à PIPELINE?* — e é a pergunta que o `CLAUDE.md` §5.0 nomeia
//! como a que nenhum instrumento deste repo fazia: *«o consumidor que PROJECTA o
//! valor fora — o fio está completo, o valor chega, e a matemática descarta-o».*
//!
//! ⚠️ **A tabela é a do irmão**, de propósito: as duas rotas desenham a mesma cena,
//! logo têm de aterrar no mesmo byte. Uma barra própria aqui seria uma segunda
//! resposta à mesma pergunta.
//!
//! ⚠️ **O `texture_id` viaja num RUN e não no buffer**: a partição vazia é o desenho
//! do átlas (o caminho de sempre), e sem o run a sprite de cima seria pintada com o
//! átlas partilhado em vez da textura dela.

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{
    Camera2d, GpuTexRun, LiftedInstances, RenderInstance, SpriteRenderer, TextureAtlas,
};

/// Os tags da tabela, pelo nome — ver o irmao `blend_mode_regression`.
const ADD: u8 = 1;
const MULTIPLY: u8 = 3;

const W: u32 = 64;
const H: u32 = 64;

const BLACK: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn solid_rgba(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        out.extend_from_slice(&rgba);
    }
    out
}

fn make_target(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("blend no device: alvo"),
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

fn readback(gpu: &GpuContext, texture: &wgpu::Texture) -> Vec<u8> {
    let unpadded = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("blend no device: staging"),
        size: u64::from(padded * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        let row = (y * padded) as usize;
        out.extend_from_slice(&data[row..row + unpadded as usize]);
    }
    drop(data);
    staging.unmap();
    out
}

fn channel(pixels: &[u8], x: u32, y: u32) -> u8 {
    let i = ((y * W + x) * 4) as usize;
    pixels[i] // cinzento, logo R == G == B
}

/// Sprite de cobertura total em `z` com `blend_tag` nos bits 5-7 de `flip_uv`.
fn instance(texture_id: u32, z: u32, blend_tag: u8) -> RenderInstance {
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
        flip_uv: RenderInstance::pack_blend_bits(blend_tag),
        texture_id,
        z_order: z,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

#[test]
fn a_mistura_de_um_sink_cozido_na_placa_chega_ao_pixel() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador headless — saltando");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let gray = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("textura do fundo");
    let fg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("textura da frente");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);
    let vazio = LiftedInstances::default();

    let por_modo = |renderer: &mut SpriteRenderer, tag: u8| -> u8 {
        // O FUNDO vem da cena, como no irmão da CPU.
        let mut present = PresentWorld::new();
        present.world_mut().spawn(instance(gray, 0, 0));
        // A FRENTE vem do buffer do dispositivo — a rota do cozimento do Motion.
        let frente = [instance(fg, 1, tag)];
        let bytes: &[u8] = bytemuck::cast_slice(&frente);
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blend no device: instancias"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&buffer, 0, bytes);
        let runs = [GpuTexRun {
            texture_id: fg,
            start: 0,
            end: 1,
            // ⭐ O SUJEITO deste gate: e' por aqui que o tag chega a' pipeline.
            blend: tag,
            sampling: 0,
        }];
        renderer.render_with_streams(
            &view,
            &mut present,
            &camera,
            window,
            BLACK,
            &vazio,
            Some((&buffer, 1, &runs)),
            None,
            None,
            None,
        );
        let px = readback(&gpu, &target);
        channel(&px, 32, 32)
    };

    // A MESMA tabela do irmão da CPU — as duas rotas desenham a mesma cena.
    let mix = por_modo(&mut renderer, 0);
    assert!(
        (i32::from(mix) - 55).abs() <= 14,
        "Mix no centro {mix}, esperado ~55"
    );
    let add = por_modo(&mut renderer, 1);
    let sub = por_modo(&mut renderer, 2);
    let mul = por_modo(&mut renderer, 3);
    let screen = por_modo(&mut renderer, 4);
    // ⚠️ A ORDEM é a prova, e ela é o que um `set_pipeline(0)` cravado apaga: com a
    // pipeline fixa os cinco leem o MESMO byte e nada aqui discrimina.
    assert!(
        add > mix,
        "Add tem de clarear contra Mix — leu {add} contra {mix} \
         (iguais = a rota do dispositivo cravou a pipeline e o tag nao chega)"
    );
    assert!(
        sub < mix,
        "Subtract tem de escurecer contra Mix — leu {sub} contra {mix}"
    );
    assert!(
        mul < mix,
        "Multiply tem de escurecer contra Mix — leu {mul} contra {mix}"
    );
    assert!(
        screen > mix && screen < add,
        "Screen fica entre Mix e Add — leu {screen}, com mix {mix} e add {add}"
    );
    assert!(
        (i32::from(add) - 110).abs() <= 16,
        "Add no centro {add}, esperado ~110"
    );
    assert!(sub <= 14, "Subtract no centro {sub}, esperado ~0");
}

#[test]
fn dois_runs_com_misturas_diferentes_desenham_cada_um_na_sua() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador headless — saltando");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let gray = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("textura do fundo");
    let fg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("textura da frente");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);
    let vazio = LiftedInstances::default();

    // A geometria é MEDIDA e não suposta (`sonda_da_geometria`, a que a escreveu):
    // uma unidade de mundo são 16 px e o centro é `px = 32 + x · 16`. Logo estas duas
    // peças de lado 2 pousam em `[0, 32)` e `[32, 64)`, sem se tocarem.
    let mut esquerda = instance(fg, 1, ADD);
    esquerda.world_pos = [-1.0, 0.0];
    esquerda.size = [2.0, 2.0];
    let mut direita = instance(fg, 1, MULTIPLY);
    direita.world_pos = [1.0, 0.0];
    direita.size = [2.0, 2.0];
    let frente = [esquerda, direita];

    let mut present = PresentWorld::new();
    present.world_mut().spawn(instance(gray, 0, 0));
    let bytes: &[u8] = bytemuck::cast_slice(&frente);
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dois runs: instancias"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue.write_buffer(&buffer, 0, bytes);
    let runs = [
        GpuTexRun {
            texture_id: fg,
            start: 0,
            end: 1,
            blend: ADD,
            sampling: 0,
        },
        GpuTexRun {
            texture_id: fg,
            start: 1,
            end: 2,
            blend: MULTIPLY,
            sampling: 0,
        },
    ];
    renderer.render_with_streams(
        &view,
        &mut present,
        &camera,
        window,
        BLACK,
        &vazio,
        Some((&buffer, 2, &runs)),
        None,
        None,
        None,
    );
    let px = readback(&gpu, &target);
    let clara = channel(&px, 16, 32);
    let escura = channel(&px, 48, 32);

    // ⭐⭐⭐ **É ISTO que obriga o `set_pipeline` a viver DENTRO do laço dos runs.**
    // Içado para fora ele fica com UMA mistura para os dois, e as duas metades leem o
    // mesmo byte — o gate irmão (um run só) não vê essa diferença, porque com um run
    // «dentro» e «fora» são a mesma coisa.
    assert!(
        clara > escura,
        "o run em Add tem de clarear e o em Multiply escurecer — leu {clara} e {escura} \
         (iguais = o set_pipeline saiu do laco e os dois runs partilham uma mistura)"
    );
    // E cada um aterra na SUA linha da tabela, não só «diferente do vizinho».
    assert!(
        (i32::from(clara) - 110).abs() <= 16,
        "a metade em Add leu {clara}, esperado ~110"
    );
    assert!(
        (i32::from(escura) - 12).abs() <= 14,
        "a metade em Multiply leu {escura}, esperado ~12"
    );
}
