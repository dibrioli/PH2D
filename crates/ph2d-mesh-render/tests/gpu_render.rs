//! W1/M2 — validação GPU end-to-end (headless): **a malha aparece, e aparece
//! sombreada pela própria forma.**
//!
//! `#[ignore]` — precisa de adapter (roda com `--ignored`; skip gracioso sem
//! GPU), como os gates do `ph2d-gpu` e do `ph2d-flip-render`.
//!
//! ```text
//! cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored --nocapture
//! ```
//!
//! Alvo `Rgba8Unorm` e não `Rgba16Float` como no produto: o que estes gates
//! afirmam é APARÊNCIA (que lado está aceso, quanto da tela a silhueta cobre), e
//! ler `f16` de volta acrescentaria uma conversão entre a medição e o olho.

use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

const W: u32 = 128;
const H: u32 = 128;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-mesh test device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

/// Rasteriza `mesh` com `camera` e devolve os pixels RGBA (sem padding).
fn render(device: &wgpu::Device, queue: &wgpu::Queue, mesh: &Mesh, camera: &Camera3d) -> Vec<u8> {
    let mut renderer = MeshRenderer::new(device, FORMAT);
    renderer.upload(device, queue, mesh);
    render_using(device, queue, &mut renderer, camera)
}

/// A mesma cena com um renderizador que o chamador JÁ semeou — é o que deixa
/// comparar "subi a malha inteira" com "subi só a região" no mesmo pixel.
fn render_using(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
) -> Vec<u8> {
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("alvo"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    // O passe da malha usa `LoadOp::Load` (a cena 2D fica por baixo), então quem
    // limpa é este pré-passe — exatamente como no shell.
    {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("limpa"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    renderer.render(device, queue, &mut encoder, &view, camera, (W, H));

    let bpr = (W * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(bpr * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for row in 0..H {
        let s = (row * bpr) as usize;
        out.extend_from_slice(&mapped[s..s + (W * 4) as usize]);
    }
    drop(mapped);
    buffer.unmap();
    out
}

/// Luminância aproximada do pixel `(x, y)`.
fn lum(px: &[u8], x: u32, y: u32) -> f32 {
    let i = ((y * W + x) * 4) as usize;
    0.2126 * f32::from(px[i]) + 0.7152 * f32::from(px[i + 1]) + 0.0722 * f32::from(px[i + 2])
}

/// Fração de pixels que não são o fundo.
fn coverage(px: &[u8]) -> f32 {
    let lit = px
        .chunks_exact(4)
        .filter(|p| p[0] + p[1] + p[2] > 8)
        .count();
    lit as f32 / (W * H) as f32
}

fn camera_for(mesh: &Mesh) -> Camera3d {
    // De frente, para que "esquerda" e "direita" na tela sejam esquerda e
    // direita do modelo — o gate da luz depende disso.
    //
    // ⚠️ **Os ângulos ANTES do `frame`, e a ordem é load-bearing:** o
    // enquadramento é do ângulo atual (ver o doc do `Camera3d::frame`), então
    // enquadrar num ângulo e renderizar noutro dá uma distância errada. A
    // primeira versão deste helper fazia exatamente isso e a cobertura saía
    // 18,2% em vez de 31% — eu li o número como defeito do produto até refazer
    // a conta.
    let mut cam = Camera3d {
        yaw: 0.0,
        pitch: 0.0,
        fov_y: core::f32::consts::FRAC_PI_4,
        ..Camera3d::default()
    };
    cam.frame(mesh.bounds(), 1.0);
    cam
}

/// **A malha aparece**, e ocupa a fração de tela que o enquadramento promete.
///
/// A banda é larga de propósito: o que este gate refuta é *tela preta* e
/// *geometria explodida ocupando tudo*, não uma diferença de 5% de silhueta.
#[test]
#[ignore = "precisa de adapter"]
fn the_mesh_appears_on_screen_at_the_size_the_framing_promised() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let px = render(&device, &queue, &mesh, &camera_for(&mesh));
    let cov = coverage(&px);
    println!("cobertura da esfera enquadrada: {:.1}%", cov * 100.0);
    assert!(
        (0.25..0.45).contains(&cov),
        "cobertura {cov:.3} — tela preta ou geometria fora de escala"
    );
    // O centro está aceso e as quinas não: a silhueta é um disco no meio.
    assert!(lum(&px, W / 2, H / 2) > 40.0, "o centro ficou escuro");
    assert!(lum(&px, 2, 2) < 8.0, "a quina devia ser fundo");
}

/// ⚠️ **O gate que prova o DEPTH, e o oráculo é a luz.** Sem teste de
/// profundidade, o que sobra num pixel é o último triângulo que passou por ele —
/// metade das vezes o do lado de TRÁS da esfera. E o hemisfério de trás tem a
/// normal espelhada em `z`, que o shader vira para o olho: o lado esquerdo passa
/// a responder como se fosse o direito.
///
/// A luz-chave vem de cima-e-da-ESQUERDA (`key_dir.x < 0`), então numa esfera
/// vista de frente a esquerda é mais clara que a direita. Isso é uma afirmação
/// sobre a APARÊNCIA — não sobre `depth_compare` —, e é por isso que ela pega
/// tanto um depth desligado quanto um invertido.
#[test]
#[ignore = "precisa de adapter"]
fn the_lit_side_is_the_side_the_key_light_comes_from() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let px = render(&device, &queue, &mesh, &camera_for(&mesh));

    // Dois pontos simétricos em torno do centro, dentro da silhueta.
    let (lx, rx) = (W / 2 - W / 5, W / 2 + W / 5);
    let y = H / 2;
    let (l, r) = (lum(&px, lx, y), lum(&px, rx, y));
    println!("luminância esquerda {l:.1} / direita {r:.1}");
    assert!(l > 8.0 && r > 8.0, "os dois pontos têm de estar na malha");
    assert!(
        l > r * 1.3,
        "a esquerda ({l:.1}) devia ser bem mais clara que a direita ({r:.1}) — \
         a superfície visível não é a da frente"
    );
}

/// A câmera CHEGA ao device: girar muda a imagem. O fixture é um cubo porque
/// uma esfera é simétrica demais — ela renderiza quase igual de qualquer
/// ângulo, e o gate ficaria verde com o uniform congelado.
#[test]
#[ignore = "precisa de adapter"]
fn orbiting_changes_what_the_device_draws() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::cube(1.0);
    let mut cam = camera_for(&mesh);
    let face_on = render(&device, &queue, &mesh, &cam);
    cam.orbit(0.9, 0.5);
    let angled = render(&device, &queue, &mesh, &cam);

    let diff = face_on
        .chunks_exact(4)
        .zip(angled.chunks_exact(4))
        .filter(|(a, b)| a[0].abs_diff(b[0]) > 8)
        .count();
    let frac = diff as f32 / (W * H) as f32;
    println!("pixels que mudaram ao girar: {:.1}%", frac * 100.0);
    assert!(frac > 0.05, "girar não mudou nada — o uniform não chegou");
}

/// Uma malha vazia não desenha e não derruba nada — o passe é no-op antes de
/// haver o que mostrar, que é o estado do app no primeiro frame.
#[test]
#[ignore = "precisa de adapter"]
fn an_empty_mesh_draws_nothing_and_does_not_panic() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = Mesh::default();
    let px = render(&device, &queue, &mesh, &Camera3d::default());
    assert_eq!(coverage(&px), 0.0);
}

#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_region_upload_shows_exactly_what_a_full_upload_shows() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — pulando");
        return;
    };
    let mut mesh = shapes::uv_sphere(40, 56, 1.0);
    let camera = camera_for(&mesh);

    // Um renderizador semeado com a malha ORIGINAL, que daqui para a frente só
    // recebe regiões — é ele o caminho sob teste.
    let mut incremental = MeshRenderer::new(&device, FORMAT);
    incremental.upload(&device, &queue, &mesh);

    // Esculpe de verdade, pela porta do produto.
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.45,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let mut uploaded = 0u32;
    for k in 0..5 {
        let x = -0.3 + 0.15 * k as f32;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([x, 0.0, 0.95], brush.radius),
            Symmetry::default(),
        );
        assert!(
            incremental.upload_region(&queue, &mesh, stroke.last_refreshed()),
            "o upload incremental recusou uma malha de mesma topologia"
        );
        uploaded += incremental.last_region_verts();
    }

    let want = render(&device, &queue, &mesh, &camera);
    let got = render_using(&device, &queue, &mut incremental, &camera);
    assert_eq!(
        got, want,
        "a região subiu bytes diferentes do que a malha inteira subiria"
    );

    // E o ponto do exercício: viajou MUITO menos que a malha. Sem esta metade o
    // gate ficaria verde com um `upload_region` que delega ao upload cheio — o
    // caminho rápido virando código morto com todos os gates verdes, que é a
    // armadilha que o ADR-0120 do áudio documentou.
    let whole = mesh.vert_count() as u32 * 5;
    let share = f64::from(uploaded) / f64::from(whole);
    println!("upload incremental: {uploaded} de {whole} vértices ({share:.3})");
    assert!(
        share < 0.5,
        "a região viajou como {share:.3} da malha — não é incremental"
    );
}

#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_region_upload_refuses_a_mesh_whose_topology_changed() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — pulando");
        return;
    };
    let small = shapes::uv_sphere(12, 16, 1.0);
    let big = shapes::uv_sphere(20, 28, 1.0);
    let mut r = MeshRenderer::new(&device, FORMAT);
    // Sem nada subido ainda: não há com que reconciliar.
    assert!(!r.upload_region(&queue, &small, &[0, 1, 2]));
    r.upload(&device, &queue, &small);
    assert!(r.upload_region(&queue, &small, &[0, 1, 2]));
    // Contagem diferente: recusar é a resposta certa. Escrever a região sobre um
    // buffer de outra topologia poria bytes VÁLIDOS nos vértices errados, e a
    // geometria seria puxada para lugares que ninguém tocou.
    assert!(!r.upload_region(&queue, &big, &[0, 1, 2]));
}

#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_pixels_the_ray_hits_are_the_pixels_the_mesh_painted() {
    // ⚠️ **O oráculo que faltava.** O gate de round-trip da câmera prova
    // raio↔MATRIZ; este prova raio↔IMAGEM. Entre os dois mora tudo que pode
    // deslocar o pincel do cursor — viewport de outro tamanho, um flip de Y no
    // alvo, uma aspect diferente entre quem projeta e quem dispara — e nenhum
    // teste de nenhuma das duas metades enxerga isso.
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — pulando");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    // Câmera ASSIMÉTRICA de propósito: com o modelo centrado, um espelho em X ou
    // em Y é indistinguível do certo, e o gate ficaria verde sobre a inversão.
    let mut camera = camera_for(&mesh);
    camera.yaw = 0.8;
    camera.pitch = 0.5;
    camera.pan(0.18, -0.12);

    let px = render(&device, &queue, &mesh, &camera);

    let mut painted = 0usize;
    let mut agree = 0usize;
    let mut disagree = Vec::new();
    for y in 0..H {
        for x in 0..W {
            let lit = lum(&px, x, y) > 0.02;
            // Amostra do CENTRO do pixel: é para onde o rasterizador olha.
            let ray = camera.ray_through(x as f32 + 0.5, y as f32 + 0.5, (W, H));
            let hit = mesh.raycast(&ray).is_some();
            if lit {
                painted += 1;
            }
            if lit == hit {
                agree += 1;
            } else if disagree.len() < 8 {
                disagree.push((x, y, lit, hit));
            }
        }
    }
    let total = (W * H) as usize;
    let share = agree as f64 / total as f64;
    println!(
        "pintados {painted} de {total}; raio e imagem concordam em {:.4} dos pixels",
        share
    );
    assert!(painted > total / 20, "a malha mal apareceu ({painted} px)");
    // A discordância honesta é a BORDA: um pixel meio coberto acende e o raio
    // pelo centro dele erra (e vice-versa). Numa silhueta de ~300 px de diâmetro
    // a borda é ~1000 px de 65 mil = 1,5%.
    assert!(
        share > 0.975,
        "raio e imagem discordam em {:.1}% dos pixels — o pincel não cai onde o \
         cursor aponta. Amostras (x, y, pintado, acertou): {disagree:?}",
        (1.0 - share) * 100.0
    );
}
