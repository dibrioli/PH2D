//! BlendMode compositing regression gate (Sprite Inspector v2 §10).
//!
//! Renders an opaque mid-gray BACKGROUND sprite (z=0, Mix) then an opaque
//! mid-gray FOREGROUND sprite (z=1) carrying a given [`BlendMode`] tag in
//! `flip_uv` bits 5-7, and samples the overlapped centre pixel (32,32).
//!
//! **Compositing is LINEAR-space** (textures sample sRGB→linear, the
//! `Rgba8Unorm` target stores linear), so the byte 128 sprite is linear
//! `0.216` (`srgb_to_linear(0.502)`), and the readback byte is
//! `round(linear · 255)`. With both colors linear 0.216 each mode lands
//! on a distinct value:
//!
//! | mode       | formula (linear, F=B=0.216) | byte |
//! |------------|-----------------------------|------|
//! | Mix        | F                           | 55   |
//! | Add        | min(B+F, 1) = 0.432         | 110  |
//! | Subtract   | max(B−F, 0) = 0             | 0    |
//! | Multiply   | F·B = 0.047                 | 12   |
//! | Screen     | 1−(1−B)(1−F) = 0.386        | 98   |
//! | PremultAlpha | F (premult over)          | 55   |
//!
//! Ordering is the real proof: Multiply < Mix < Screen < Add, and
//! Subtract bottoms out — i.e. each pipeline composites as advertised.
//! Skips gracefully on adapter-less CI.

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, RenderInstance, SpriteRenderer, TextureAtlas};

const W: u32 = 64;
const H: u32 = 64;

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
        label: Some("blend regression color target"),
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
        label: Some("blend regression staging"),
        size: (padded * H) as u64,
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
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    rx.recv().expect("map channel").expect("map ok");
    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * H) as usize);
    for row in 0..H as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

fn channel(pixels: &[u8], x: u32, y: u32) -> u8 {
    let i = ((y * W + x) * 4) as usize;
    pixels[i] // gray, so R == G == B
}

const BLACK: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// Full-cover sprite at `z` with `blend_tag` packed into `flip_uv` 5-7.
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
    }
}

#[test]
fn blend_modes_composite_as_advertised() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping blend_modes_composite_as_advertised: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    // Two opaque mid-gray (0.5) textures — same value so each mode lands
    // on a distinct composite (see module table).
    let gray = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("bg tex");
    let fg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("fg tex");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    let render_mode = |renderer: &mut SpriteRenderer, fg_blend: u8| -> u8 {
        let mut present = PresentWorld::new();
        present.world_mut().spawn(instance(gray, 0, 0)); // background, Mix
        present.world_mut().spawn(instance(fg, 1, fg_blend)); // foreground, mode
        renderer.render(&view, &mut present, &camera, window, BLACK);
        let px = readback(&gpu, &target);
        channel(&px, 32, 32)
    };

    // Linear-space expectations (see module table).
    let mix = render_mode(&mut renderer, 0);
    assert!(
        (mix as i32 - 55).abs() <= 14,
        "Mix centre {mix}, expected ~55"
    );
    let add = render_mode(&mut renderer, 1);
    assert!(
        (add as i32 - 110).abs() <= 16,
        "Add centre {add}, expected ~110"
    );
    assert!(add > mix, "Add must lighten vs Mix");
    let sub = render_mode(&mut renderer, 2);
    assert!(sub <= 14, "Subtract centre {sub}, expected ~0 (darkest)");
    let mul = render_mode(&mut renderer, 3);
    assert!(
        (mul as i32 - 12).abs() <= 14,
        "Multiply centre {mul}, expected ~12"
    );
    assert!(mul < mix, "Multiply must darken vs Mix");
    let screen = render_mode(&mut renderer, 4);
    assert!(
        (screen as i32 - 98).abs() <= 16,
        "Screen centre {screen}, expected ~98"
    );
    assert!(
        screen > mix && screen < add,
        "Screen lightens but softer than Add"
    );
    let premult = render_mode(&mut renderer, 5);
    assert!(
        (premult as i32 - 55).abs() <= 14,
        "PremultAlpha centre {premult}, expected ~55"
    );
}

/// A mesma instância, com a alfa autorada (`tint.a`) que o shader dobra em
/// `extra_alpha` — a via por onde o `fx.drop_shadow` faz a sua sombra ser translúcida.
fn instance_alpha(texture_id: u32, z: u32, blend_tag: u8, alpha: f32) -> RenderInstance {
    let mut i = instance(texture_id, z, blend_tag);
    i.tint[3] = alpha;
    i
}

/// Um banco de ensaio reutilizável: fundo opaco em `Mix`, frente opcional com modo e alfa.
struct Rig {
    gpu: GpuContext,
    renderer: SpriteRenderer,
    view: wgpu::TextureView,
    _target: wgpu::Texture,
    camera: Camera2d,
    window: WindowSize,
    bg: u32,
    fg: u32,
}

impl Rig {
    fn new(gpu: GpuContext, bg_rgba: [u8; 4], fg_rgba: [u8; 4]) -> Self {
        let atlas = TextureAtlas::new(&gpu, 256);
        let mut renderer =
            SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
        let bg = renderer
            .acquire_individual(8, 8, &solid_rgba(8, 8, bg_rgba))
            .expect("bg tex");
        let fg = renderer
            .acquire_individual(8, 8, &solid_rgba(8, 8, fg_rgba))
            .expect("fg tex");
        let target = make_target(&gpu);
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            gpu,
            renderer,
            view,
            _target: target,
            camera: Camera2d::new([0.0, 0.0], 4.0),
            window: WindowSize::new(W, H),
            bg,
            fg,
        }
    }

    /// O byte do centro depois de compor. `fg = None` é o **controle**: o fundo sozinho.
    fn centre(&mut self, fg: Option<(u8, f32)>) -> u8 {
        let mut present = PresentWorld::new();
        present.world_mut().spawn(instance(self.bg, 0, 0));
        if let Some((mode, alpha)) = fg {
            present
                .world_mut()
                .spawn(instance_alpha(self.fg, 1, mode, alpha));
        }
        self.renderer
            .render(&self.view, &mut present, &self.camera, self.window, BLACK);
        let px = readback(&self.gpu, &self._target);
        channel(&px, 32, 32)
    }
}

/// Os seis modos, pelo nome, na ordem das tags.
const MODES: [(&str, u8); 6] = [
    ("Mix", 0),
    ("Add", 1),
    ("Subtract", 2),
    ("Multiply", 3),
    ("Screen", 4),
    ("Premult", 5),
];

/// **A SONDA DA ALFA** — a segunda dimensão que esta suíte não tinha.
///
/// ⚠️ Tudo acima mede a `alpha = 1`, que é **o único valor em que os seis modos concordam
/// sobre o que a alfa quer dizer**. Ela imprime a resposta de cada modo ao longo do curso e
/// não afirma nada: o veredito é do gate seguinte.
#[test]
fn measure_alpha_response_of_every_mode() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping measure_alpha_response_of_every_mode: no headless GPU");
        return;
    };
    let mut rig = Rig::new(gpu, [128, 128, 128, 255], [128, 128, 128, 255]);
    let backdrop = rig.centre(None);
    println!("fundo sozinho = {backdrop}");
    println!("modo      a=0,00  a=0,25  a=0,50  a=0,75  a=1,00");
    for (name, tag) in MODES {
        let row: Vec<String> = [0.0, 0.25, 0.5, 0.75, 1.0]
            .iter()
            .map(|a| format!("{:>6}", rig.centre(Some((tag, *a)))))
            .collect();
        println!("{name:<9} {}", row.join("  "));
    }
}

/// **ALFA ZERO É AUSÊNCIA, EM TODO MODO** — a lei que o `Multiply` não cumpria.
///
/// Uma fonte pré-multiplicada codifica *"não contribuo"* como **zero**, e todo modo cujo
/// elemento neutro é `0` (`Add`, `Subtract`, `Screen`, o `over`) obedece à alfa de graça. O
/// neutro do `Multiply` é **`1`**: com `dst_factor: Zero` a pré-multiplicação levava-o para
/// PRETO em vez de para nada, e o cursor da alfa deixava de dizer *"quão presente"* para
/// dizer *"quão escuro"* — ao contrário.
///
/// ⚠️ **A barra é o FUNDO medido no mesmo passe** (`fg = None`), nunca um número escrito à
/// mão: o alvo é `Rgba8Unorm` linear e o byte exacto depende do sRGB do atlas.
#[test]
fn zero_alpha_is_absence_in_every_mode() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping zero_alpha_is_absence_in_every_mode: no headless GPU");
        return;
    };
    let mut rig = Rig::new(gpu, [128, 128, 128, 255], [128, 128, 128, 255]);
    let backdrop = rig.centre(None);
    // Controle positivo: a alfa cheia MOVE o pixel em pelo menos um modo — senão o
    // banco estaria a medir um fundo que a frente nunca alcança, e o zero passaria
    // por vacuidade.
    assert!(
        MODES
            .iter()
            .any(|&(_, t)| rig.centre(Some((t, 1.0))).abs_diff(backdrop) > 8),
        "controle: a alfa cheia tem de mudar o pixel"
    );
    for (name, tag) in MODES {
        let got = rig.centre(Some((tag, 0.0)));
        assert!(
            got.abs_diff(backdrop) <= 2,
            "{name} a alfa 0 tem de deixar o fundo intacto ({backdrop}), deu {got}"
        );
    }
}

/// **E O CURSO INTEIRO É MONÓTONO, DO FUNDO ATÉ AO MODO** (só o `Multiply`).
///
/// A alfa de uma sombra é *quanta sombra*, então subi-la só pode escurecer. ⚠️ Com o
/// `dst_factor: Zero` a resposta era **invertida** — `a = 0,25` dava um pixel mais escuro
/// que `a = 1,00`, e é exactamente isso que se vê no smoke como *"a alfa não faz nada"*
/// (ou faz o contrário).
#[test]
fn the_multiply_alpha_slider_runs_from_the_backdrop_to_the_full_product() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping the_multiply_alpha_slider_runs_...: no headless GPU");
        return;
    };
    let mut rig = Rig::new(gpu, [200, 200, 200, 255], [90, 90, 90, 255]);
    let backdrop = rig.centre(None);
    let steps: Vec<u8> = [0.0, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|a| rig.centre(Some((3, *a))))
        .collect();
    assert!(
        steps[0].abs_diff(backdrop) <= 2,
        "a 0 é o fundo ({backdrop}), deu {}",
        steps[0]
    );
    for w in steps.windows(2) {
        assert!(
            w[1] <= w[0] + 1,
            "subir a alfa do Multiply só pode escurecer: {steps:?}"
        );
    }
    assert!(
        steps[0] > steps[4] + 20,
        "e o curso tem excursão de facto: {steps:?}"
    );
}

#[test]
fn absent_blend_is_zero_regression_over() {
    // tag 0 (Mix, the absent-component default) composites an opaque
    // foreground as a plain "over" — identical to the pre-§10 renderer.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping absent_blend_is_zero_regression_over: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let bg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [40, 40, 40, 255]))
        .expect("bg");
    let fg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [200, 200, 200, 255]))
        .expect("fg");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);
    let mut present = PresentWorld::new();
    present.world_mut().spawn(instance(bg, 0, 0));
    present.world_mut().spawn(instance(fg, 1, 0));
    renderer.render(&view, &mut present, &camera, window, BLACK);
    let px = readback(&gpu, &target);
    let c = channel(&px, 32, 32);
    // fg byte 200 → linear 0.573 → readback byte ~146 (linear target).
    assert!(
        (c as i32 - 146).abs() <= 16,
        "opaque fg over bg = fg (linear ~146), got {c}"
    );
}
