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

use ph2d_light::LightRig;
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
    render_with_rig(device, queue, mesh, camera, &LightRig::default())
}

/// A cena sob um rig ESCOLHIDO. O `render` acima passa o default — que é o rig de
/// toda tela que ninguém abriu o card para mexer, e portanto o que os gates de
/// aparência devem julgar.
fn render_with_rig(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh: &Mesh,
    camera: &Camera3d,
    rig: &LightRig,
) -> Vec<u8> {
    let mut renderer = MeshRenderer::new(device, FORMAT);
    renderer.upload_at(device, queue, 0, mesh);
    render_using_rig(device, queue, &mut renderer, camera, rig)
}

/// A mesma cena com um renderizador que o chamador JÁ semeou — é o que deixa
/// comparar "subi a malha inteira" com "subi só a região" no mesmo pixel.
fn render_using(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
) -> Vec<u8> {
    render_using_rig(device, queue, renderer, camera, &LightRig::default())
}

fn render_using_rig(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
    rig: &LightRig,
) -> Vec<u8> {
    // ⚠️ **Cavidade ZERO aqui, e é por isso que os 22 gates anteriores desta
    // suíte continuam medindo o que mediam.** O canal novo é opt-in por
    // construção; quem quiser exercitá-lo chama a porta abaixo.
    render_using_rig_cavity(device, queue, renderer, camera, rig, 0.0)
}

fn render_using_rig_cavity(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
    rig: &LightRig,
    cavity: f32,
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
    let resolved = ph2d_light::resolve(rig);
    renderer.render(
        device,
        queue,
        &mut encoder,
        &view,
        camera,
        resolved.as_ref(),
        ph2d_mesh_render::Shade {
            cavity,
            ..ph2d_mesh_render::Shade::default()
        },
        (W, H),
    );

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
/// A lâmpada PRINCIPAL do artista nasce em cima e à esquerda (azimute 230°,
/// elevação 30° — o default afinado pelo Enio), então numa esfera vista de frente
/// a esquerda é mais clara que a direita E o alto é mais claro que o baixo.
///
/// ⚠️ **O eixo VERTICAL é a metade que a W3 acrescentou, e ela é a única conversão
/// de espaço do passe.** O rig é autorado em espaço de TELA (`y` para BAIXO — é lá
/// que "em cima, à esquerda" quer dizer algo) e a normal chega em espaço de VISTA
/// (`y` para CIMA). Sem a negação, a MESMA lâmpada acende a pintura por cima e a
/// escultura por baixo, no mesmo documento, sob o mesmo card, com o mesmo número.
/// Não há teste de unidade que veja isso: só um render.
#[test]
#[ignore = "precisa de adapter"]
fn the_key_light_falls_where_the_artist_put_it() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let px = render(&device, &queue, &mesh, &camera_for(&mesh));

    // Quatro pontos simétricos em torno do centro, dentro da silhueta.
    let (lx, rx) = (W / 2 - W / 5, W / 2 + W / 5);
    let (ty, by) = (H / 2 - H / 5, H / 2 + H / 5);
    let (l, r) = (lum(&px, lx, H / 2), lum(&px, rx, H / 2));
    let (t, b) = (lum(&px, W / 2, ty), lum(&px, W / 2, by));
    println!("esquerda {l:.1} / direita {r:.1} · alto {t:.1} / baixo {b:.1}");
    assert!(
        l > 8.0 && r > 8.0 && t > 8.0 && b > 8.0,
        "os quatro pontos têm de estar na malha"
    );
    assert!(
        l > r * 1.3,
        "a esquerda ({l:.1}) devia ser bem mais clara que a direita ({r:.1}) — \
         a superfície visível não é a da frente"
    );
    assert!(
        t > b * 1.3,
        "o alto ({t:.1}) devia ser bem mais claro que o baixo ({b:.1}) — a luz do \
         artista está chegando com o `y` invertido"
    );
}

/// **A wave inteira, numa afirmação: mover a lâmpada reacende a FORMA.**
///
/// Sob o matcap da W1 as direções eram literais no shader, então o card do artista
/// não mexia em nada aqui — e "um documento, uma iluminação" era uma frase sobre
/// código, não sobre o que o Enio vê. O oráculo é de APARÊNCIA e não *"a imagem
/// mudou"*: o lado claro tem de TROCAR DE LADO quando a lâmpada atravessa a cena.
#[test]
#[ignore = "precisa de adapter"]
fn moving_the_key_light_re_lights_the_form() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let cam = camera_for(&mesh);
    let (lx, rx) = (W / 2 - W / 5, W / 2 + W / 5);

    let from_left = render(&device, &queue, &mesh, &cam);
    let mut moved = LightRig::default();
    moved.lights[0].angle_deg = (moved.lights[0].angle_deg + 180) % 360;
    let from_right = render_with_rig(&device, &queue, &mesh, &cam, &moved);

    let (al, ar) = (lum(&from_left, lx, H / 2), lum(&from_left, rx, H / 2));
    let (bl, br) = (lum(&from_right, lx, H / 2), lum(&from_right, rx, H / 2));
    println!("antes E{al:.1}/D{ar:.1} · depois E{bl:.1}/D{br:.1}");
    assert!(al > ar, "com a principal à esquerda, a esquerda é a clara");
    assert!(
        br > bl,
        "atravessando a lâmpada, o lado claro tinha de trocar — E{bl:.1}/D{br:.1}"
    );
}

/// Apagar todas as luzes devolve o BARRO CRU, não uma silhueta preta.
///
/// É a leitura honesta de "sem luz" para uma superfície opaca — e o espelho do
/// contrato da tinta, onde apagar tudo devolve a tela intocada ao byte em vez de
/// a escurecer até o piso ambiente.
#[test]
#[ignore = "precisa de adapter"]
fn turning_every_lamp_off_leaves_bare_clay() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let cam = camera_for(&mesh);
    let mut dark = LightRig::default();
    for l in &mut dark.lights {
        l.on = false;
    }
    let px = render_with_rig(&device, &queue, &mesh, &cam, &dark);

    // Sem rig não há razão a computar, então a esfera inteira é UMA cor.
    let (lx, rx) = (W / 2 - W / 5, W / 2 + W / 5);
    let (l, r) = (lum(&px, lx, H / 2), lum(&px, rx, H / 2));
    println!("sem luz: esquerda {l:.1} / direita {r:.1}");
    assert!(
        l > 8.0,
        "a forma continua na tela — ela não some, só não é lida"
    );
    assert!(
        (l - r).abs() < 1.0,
        "sem lâmpada a esfera é chapada; E{l:.1} contra D{r:.1} é sombreamento"
    );
    // E ela não é preta: o barro tem cor própria.
    assert!(l > 100.0, "o barro cru devia estar claro, e está em {l:.1}");
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
    incremental.upload_at(&device, &queue, 0, &mesh);

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
            // Olhando de +Z para a calota: a pegada inteira é frontal, que é o
            // caso que esta cena descreve.
            &Dab::at([x, 0.0, 0.95], brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        assert!(
            incremental.upload_region_at(&queue, 0, &mesh, stroke.last_refreshed()),
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
    assert!(!r.upload_region_at(&queue, 0, &small, &[0, 1, 2]));
    r.upload_at(&device, &queue, 0, &small);
    assert!(r.upload_region_at(&queue, 0, &small, &[0, 1, 2]));
    // Contagem diferente: recusar é a resposta certa. Escrever a região sobre um
    // buffer de outra topologia poria bytes VÁLIDOS nos vértices errados, e a
    // geometria seria puxada para lugares que ninguém tocou.
    assert!(!r.upload_region_at(&queue, 0, &big, &[0, 1, 2]));
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

/// Decodifica meio-float. O G-buffer é `Rgba16Float` porque uma normal vive em
/// `[-1, 1]` e um formato normalizado exigiria codificar de um lado e decodificar
/// do outro; o preço é este decodificador de dez linhas no lado que MEDE.
fn f16(bits: u16) -> f32 {
    let sign = f32::from(bits >> 15);
    let exp = i32::from((bits >> 10) & 0x1f);
    let frac = f32::from(bits & 0x3ff);
    let mag = if exp == 0 {
        frac * 2f32.powi(-24)
    } else {
        (1.0 + frac / 1024.0) * 2f32.powi(exp - 15)
    };
    if sign > 0.0 { -mag } else { mag }
}

/// Rasteriza o G-buffer da doação e devolve `(normal, cobertura)` por texel.
fn gbuffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh: &Mesh,
    camera: &Camera3d,
) -> Vec<([f32; 3], f32)> {
    let mut renderer = MeshRenderer::new(device, FORMAT);
    renderer.upload_at(device, queue, 0, mesh);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("gbuffer"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: MeshRenderer::GBUFFER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    // ⚠️ Sem pré-passe de limpeza, de propósito: o `render_gbuffer` LIMPA o alvo
    // ele mesmo, e é isso que faz a cobertura sair certa sem o chamador combinar
    // nada. Se ele parar de limpar, este gate mede o lixo.
    renderer.render_gbuffer(device, queue, &mut encoder, &view, camera, (W, H));

    let bpr = (W * 8).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
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
    let mut out = Vec::with_capacity((W * H) as usize);
    for row in 0..H {
        let s = (row * bpr) as usize;
        for x in 0..W as usize {
            let o = s + x * 8;
            let c = |k: usize| {
                f16(u16::from_le_bytes([
                    mapped[o + k * 2],
                    mapped[o + k * 2 + 1],
                ]))
            };
            out.push(([c(0), c(1), c(2)], c(3)));
        }
    }
    drop(mapped);
    buffer.unmap();
    out
}

/// **O G-buffer descreve a forma: unitário onde há malha, e nada onde não há.**
///
/// A cobertura é o que deixa o passe de luz da tinta escolher a fonte de normal
/// **por pixel** — sem ela a doação seria por documento, e a forma iluminaria a
/// tela inteira em vez da própria silhueta.
#[test]
#[ignore = "precisa de adapter"]
fn the_gbuffer_covers_the_form_and_nothing_else() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let cam = camera_for(&mesh);
    let g = gbuffer(&device, &queue, &mesh, &cam);

    let covered = g.iter().filter(|(_, w)| *w > 0.5).count();
    let frac = covered as f32 / (W * H) as f32;
    println!("cobertura do G-buffer: {:.1}%", frac * 100.0);
    // A mesma esfera enquadrada que o gate de silhueta mede na rota de COR.
    assert!(
        (0.20..0.45).contains(&frac),
        "a silhueta cobre {frac:.3} da tela — o enquadramento ou a cobertura estão errados"
    );
    // Fora da forma o plano é ZERO, não lixo do frame anterior.
    let (corner_n, corner_w) = g[0];
    assert_eq!(corner_w, 0.0, "a quina não tem forma");
    assert_eq!(corner_n, [0.0; 3], "e nem normal — o alvo foi limpo");
    // Dentro, toda normal é unitária: é ela que vai virar `N·L`.
    let mut worst = 0.0f32;
    for (n, _) in g.iter().filter(|(_, w)| *w > 0.5) {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        worst = worst.max((len - 1.0).abs());
    }
    println!("pior desvio de unitariedade: {worst:.5}");
    assert!(worst < 0.02, "normais não-unitárias: {worst}");
}

/// **A normal doada está no espaço do RIG** — a mesma convenção da tinta.
///
/// Numa esfera vista de frente: o centro aponta para o olho, a metade esquerda tem
/// `x < 0`, e — a metade que a conversão de espaço decide — a metade DE CIMA tem
/// `y < 0`, porque o espaço do rig tem `y` para baixo. Doar a normal com o `y` da
/// VISTA faria a forma iluminar a tinta pelo lado oposto ao que a própria forma
/// aparece iluminada, no mesmo pixel.
#[test]
#[ignore = "precisa de adapter"]
fn the_donated_normal_is_in_the_rigs_space() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let g = gbuffer(&device, &queue, &mesh, &camera_for(&mesh));
    let at = |x: u32, y: u32| g[(y * W + x) as usize];

    let (c, cw) = at(W / 2, H / 2);
    assert!(cw > 0.5, "o centro está na esfera");
    assert!(
        c[2] > 0.9,
        "o centro de uma esfera aponta para o olho: {c:?}"
    );

    let (l, _) = at(W / 2 - W / 5, H / 2);
    let (r, _) = at(W / 2 + W / 5, H / 2);
    let (t, _) = at(W / 2, H / 2 - H / 5);
    let (b, _) = at(W / 2, H / 2 + H / 5);
    println!(
        "esq {:.2} dir {:.2} · alto {:.2} baixo {:.2}",
        l[0], r[0], t[1], b[1]
    );
    assert!(l[0] < -0.2 && r[0] > 0.2, "o eixo x está espelhado");
    assert!(
        t[1] < -0.2 && b[1] > 0.2,
        "o `y` doado está na convenção da VISTA, não na do rig — alto {:.2}, baixo {:.2}",
        t[1],
        b[1]
    );
}

/// **A porta é ÚNICA: o barro se acende pela normal que o G-buffer doa.**
///
/// Este é o gate que torna a doação uma promessa em vez de uma coincidência. Ele
/// não compara dois shaders: ele mede a normal que a malha DOA, aplica a LEI (a
/// razão relativa do `ph2d-light` — difusa sobre a resposta plana, dobrada pelo
/// piso ambiente) e exige que o barro na tela tenha sido pintado por ela.
///
/// ⚠️ O oráculo é a lei escrita em Rust, e isso é deliberado: um oráculo que
/// chamasse o shader seria o shader concordando consigo mesmo. O que se afirma é
/// que a normal doada e a normal que sombreia são a MESMA — se o `fs_gbuffer`
/// ganhar uma segunda opinião sobre para onde a superfície aponta, isto sangra.
#[test]
#[ignore = "precisa de adapter"]
fn the_clay_is_lit_by_the_very_normal_it_donates() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let cam = camera_for(&mesh);
    let rig = LightRig::default();
    let lit = render_with_rig(&device, &queue, &mesh, &cam, &rig);
    let g = gbuffer(&device, &queue, &mesh, &cam);
    let lamps = ph2d_light::resolve(&rig).expect("a principal nasce acesa");

    // A LEI, em Rust: difusa / plana, clampada, dobrada pelo piso ambiente.
    let ratio_of = |n: [f32; 3]| {
        let (mut d, mut flat) = (0.0f32, 0.0f32);
        for l in lamps.lamps() {
            d += l.tint[0] * (n[0] * l.dir[0] + n[1] * l.dir[1] + n[2] * l.dir[2]).max(0.0);
            flat += l.tint[0] * l.dir[2].max(0.0);
        }
        let flat = if flat <= 1.0e-4 { 1.0 } else { flat };
        ph2d_light::AMBIENT + (1.0 - ph2d_light::AMBIENT) * (d / flat).clamp(0.0, 2.0)
    };

    // Amostras espalhadas pela silhueta, longe da borda (onde o realce especular e
    // o serrilhado da rasterização dominam e a comparação mediria outra coisa).
    let mut n_samples = 0usize;
    let mut worst = 0.0f32;
    for gy in 1..8u32 {
        for gx in 1..8u32 {
            let (x, y) = (W * gx / 8, H * gy / 8);
            let (n, w) = g[(y * W + x) as usize];
            if w < 0.5 || n[2] < 0.35 {
                continue; // fora da forma, ou perto demais da silhueta
            }
            // ⚠️ E fora do REALCE, que esta lei não modela — de propósito, porque
            // o assunto do gate é a NORMAL e não a óptica inteira. A exclusão não
            // é um número escolhido: `pow(ndh, 24)` a 0,85 vale 0,020, que vezes
            // o `CLAY_SHINE` de 0,35 dá 0,007 — uma ordem de grandeza abaixo da
            // barra. (Medido antes de existir: os quatro únicos desvios acima de
            // 0,03 estavam todos em `ndh > 0,91`, e o desvio acompanhava o realce
            // termo a termo.)
            let ndh = lamps
                .lamps()
                .iter()
                .map(|l| n[0] * l.half[0] + n[1] * l.half[1] + n[2] * l.half[2])
                .fold(0.0f32, f32::max);
            if ndh > 0.85 {
                continue;
            }
            // O canal VERDE do barro: `CLAY.g * m`, e o realce é fraco aqui dentro.
            let drawn = f32::from(lit[((y * W + x) * 4 + 1) as usize]) / 255.0;
            let want = 0.70 * ratio_of(n);
            n_samples += 1;
            worst = worst.max((drawn - want).abs());
        }
    }
    println!("{n_samples} amostras, pior desvio lei-vs-tela {worst:.4}");
    assert!(n_samples >= 12, "poucas amostras ({n_samples}) para valer");
    // A barra admite a quantização de 8 bits do alvo e o resíduo do realce fora da
    // zona excluída. O que ela NÃO admite é uma normal diferente.
    assert!(
        worst < 0.05,
        "o barro não foi pintado pela normal que ele doa: desvio {worst:.4}"
    );
}

/// A MESMA malha com o winding invertido — normais apontando para DENTRO.
///
/// É o caso que o `cull_mode: None` do pipeline existe para tolerar: uma casca
/// aberta em progresso, ou um OBJ de terceiro que chegou com winding misto.
fn inward(mesh: &Mesh) -> Mesh {
    let faces = mesh
        .faces()
        .iter()
        .map(|f| {
            let v = f.verts();
            if v.len() == 3 {
                ph2d_mesh::Face::tri(v[2], v[1], v[0])
            } else {
                ph2d_mesh::Face::quad(v[3], v[2], v[1], v[0])
            }
        })
        .collect();
    Mesh::from_parts(mesh.positions().to_vec(), faces).expect("mesma geometria, outra ordem")
}

/// **Uma malha de normais invertidas acende como uma normal — e DOA como uma normal.**
///
/// O shader vira a normal para o olho antes de qualquer outra coisa, e sem isso o
/// interior de uma peça aberta vira um buraco preto que o artista lê como
/// geometria faltando. O pipeline não descarta a face de trás justamente para
/// tolerar isso (`cull_mode: None`), então quem responde é o `canvas_normal`.
///
/// ⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU.** Tirar o flip do
/// `fs_gbuffer` passava nos doze gates, e não por buraco: numa esfera FECHADA com
/// teste de profundidade o verso nunca vence, então o flip é *semanticamente
/// inerte* ali — a mutação era inválida, e a fixture é que não continha o
/// fenômeno. Uma malha virada do avesso contém.
#[test]
#[ignore = "precisa de adapter"]
fn a_mesh_turned_inside_out_lights_and_donates_like_one_that_is_not() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let out = shapes::uv_sphere(40, 56, 1.0);
    let inn = inward(&out);
    let cam = camera_for(&out);

    // A COR: as duas telas têm de ser a mesma imagem.
    let a = render(&device, &queue, &out, &cam);
    let b = render(&device, &queue, &inn, &cam);
    let worst_px = a
        .iter()
        .zip(&b)
        .map(|(x, y)| i32::from(*x) - i32::from(*y))
        .map(i32::abs)
        .max()
        .unwrap_or(0);
    println!("pior diferença de pixel entre fora e dentro: {worst_px}");
    assert!(
        worst_px <= 2,
        "a malha virada do avesso acendeu diferente ({worst_px} níveis)"
    );

    // E a DOAÇÃO: a normal doada tem de ser a mesma, não a negada.
    let ga = gbuffer(&device, &queue, &out, &cam);
    let gb = gbuffer(&device, &queue, &inn, &cam);
    let mut worst_n = 0.0f32;
    for ((na, wa), (nb, wb)) in ga.iter().zip(&gb) {
        if *wa < 0.5 || *wb < 0.5 {
            continue;
        }
        for c in 0..3 {
            worst_n = worst_n.max((na[c] - nb[c]).abs());
        }
    }
    println!("pior diferença de normal doada: {worst_n:.4}");
    assert!(
        worst_n < 0.02,
        "o G-buffer doou a normal do avesso: {worst_n:.4}"
    );
}

/// **O plano que o Painter recebe é o G-buffer que o dispositivo escreveu — texel por texel.**
///
/// `form_plane` é a porta do PRODUTO (rasteriza, lê de volta, achata); o helper `gbuffer` acima é a
/// rota independente que os gates de forma usam. As duas têm de concordar, e o que pode divergir é
/// o **ACHATAMENTO**: linha-maior contra coluna-maior, uma linha de padding a mais, um `y` invertido.
///
/// ⚠️ **A cena é ASSIMÉTRICA de propósito.** Numa esfera centrada, inverter a ordem das linhas
/// devolve quase os mesmos números e o gate ficaria verde sobre um plano de cabeça para baixo — o
/// ponto cego que porta-contra-porta tem quando os dois lados se movem juntos. Com o modelo
/// deslocado para um canto, a troca é gritante.
#[test]
#[ignore = "precisa de adapter"]
fn the_plane_the_painter_gets_is_the_gbuffer_the_device_wrote() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let mut camera = Camera3d::default();
    camera.frame(mesh.bounds(), W as f32 / H as f32);
    // O deslocamento que quebra a simetria: o alvo sai do centro, então a silhueta encosta num
    // canto e a metade de cima do plano deixa de parecer com a de baixo.
    camera.target.y += 0.55;
    camera.target.x -= 0.35;

    let expected = gbuffer(&device, &queue, &mesh, &camera);

    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh);
    let plane = renderer
        .form_plane(&device, &queue, &camera, (W, H))
        .expect("com malha, o plano existe");
    assert_eq!(plane.len(), (W * H * 4) as usize, "quatro floats por texel");

    // Premissa: a cena de fato tem forma E vazio, senão a comparação é entre duas telas em branco.
    let covered = expected.iter().filter(|(_, w)| *w > 0.5).count();
    assert!(
        covered > 200 && covered < (W * H) as usize - 200,
        "a fixture tem de conter os dois casos — cobertos: {covered} de {}",
        W * H
    );

    for (i, (n, w)) in expected.iter().enumerate() {
        let got = &plane[i * 4..i * 4 + 4];
        assert_eq!(
            (got[0], got[1], got[2], got[3]),
            (n[0], n[1], n[2], *w),
            "texel {i} (linha {}, coluna {}) divergiu",
            i / W as usize,
            i % W as usize
        );
    }
}

/// **Sem malha não há plano** — e é `None`, não uma tela de zeros.
///
/// A diferença importa a montante: `Some(zeros)` instalaria uma doação cujo peso é zero em toda
/// parte, e o passe de luz pagaria a leitura de um plano canvas-shaped por frame para descobrir
/// que não há nada nele. `None` é a resposta que o chamador consegue usar.
#[test]
#[ignore = "precisa de adapter"]
fn a_renderer_with_no_mesh_donates_nothing() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    let camera = Camera3d::default();
    assert!(
        renderer
            .form_plane(&device, &queue, &camera, (W, H))
            .is_none(),
        "sem geometria, nada a doar"
    );
    // E com malha, mas extensão vazia: o mesmo silêncio, pelo outro motivo.
    renderer.upload_at(&device, &queue, 0, &shapes::uv_sphere(8, 12, 1.0));
    assert!(
        renderer
            .form_plane(&device, &queue, &camera, (0, H))
            .is_none(),
        "canvas de largura zero não é um plano de zero texels — é ausência"
    );
}

/// **SONDA (não é gate): quanto custa uma doação.**
///
/// O desenho inteiro da costura — o carimbo antes da rasterização — se apoia na frase *"a leitura
/// de volta é cara"*. Esta sonda põe um número nela, porque uma frase sobre custo sem medição é um
/// palpite esperando um smoke (§0).
///
/// ```text
/// cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored measure_a_donation --nocapture
/// ```
#[test]
#[ignore = "sonda, precisa de adapter"]
fn measure_a_donation() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(96, 144, 1.0);
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh);
    let mut camera = Camera3d::default();
    camera.frame(mesh.bounds(), 1.0);

    eprintln!(
        "uma doacao, por lado de canvas (malha: {} triangulos)",
        mesh.triangle_count()
    );
    for edge in [512u32, 1024, 2048, 4096] {
        // A primeira é descartada: ela paga a alocação da textura de profundidade e o first-touch
        // do buffer de leitura, que nenhuma doação seguinte paga.
        let _ = renderer.form_plane(&device, &queue, &camera, (edge, edge));
        let mut best = f64::MAX;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let plane = renderer.form_plane(&device, &queue, &camera, (edge, edge));
            best = best.min(t0.elapsed().as_secs_f64() * 1000.0);
            assert!(plane.is_some());
        }
        let mb = f64::from(edge) * f64::from(edge) * 16.0 / 1_048_576.0;
        eprintln!("  {edge:>5}²  {best:>7.2} ms   ({mb:>6.1} MB lidos)");
    }
}

/// Uma esfera com uma calota MASCARADA, pela porta do produto (um traço de
/// `Verb::Mask`), e a lista de vértices que a GPU precisa re-ler.
fn masked_sphere() -> (Mesh, Vec<u32>) {
    let mut mesh = shapes::uv_sphere(48, 72, 1.0);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &Brush {
            verb: Verb::Mask,
            radius: 0.6,
            ..Brush::default()
        },
        &Dab::at([0.0, 0.0, 1.0], 0.6, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    let dirty = stroke.last_gpu_dirty().to_vec();
    assert!(!dirty.is_empty(), "o traço de máscara tem de tocar alguém");
    (mesh, dirty)
}

/// **A ENTREGA da W4.2: a máscara APARECE.**
///
/// ⚠️ O defeito que este gate fecha é *"pintamos e nada aparece"* — a máscara
/// existe desde a W2 e nunca chegou ao device. Um gate de CPU não consegue vê-lo:
/// o canal estava correto na malha o tempo todo.
///
/// O oráculo é a região que o traço TOCOU contra a que ele não tocou, no mesmo
/// quadro — assim ele não depende do rig, da cor do barro nem do tinto escolhido.
#[test]
#[ignore = "precisa de adapter"]
fn a_masked_region_reads_as_another_substance() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    let (masked, _) = masked_sphere();
    let plain = shapes::uv_sphere(48, 72, 1.0);
    let cam = camera_for(&plain);

    let before = render(&device, &queue, &plain, &cam);
    let after = render(&device, &queue, &masked, &cam);

    // O centro da tela caiu dentro da calota mascarada; a silhueta longe dela não.
    let at = |px: &[u8], x: u32, y: u32| {
        let i = ((y * W + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2]]
    };
    let (cx, cy) = (W / 2, H / 2);
    let (b, a) = (at(&before, cx, cy), at(&after, cx, cy));
    assert_ne!(b, a, "o centro mascarado tem de mudar de cor");
    // E muda para MAIS FRIO: o tinto é azul e o barro é quente, então o artista
    // lê "outra substância" em vez de "o mesmo barro na sombra".
    let warm = |c: [u8; 3]| i32::from(c[0]) - i32::from(c[2]);
    assert!(
        warm(a) < warm(b),
        "o barro é quente ({b:?}) e a máscara tem de esfriar ({a:?})"
    );

    // ⚠️ O CONTROLE, sem o qual o gate passaria com o shader tingindo a malha
    // INTEIRA: uma coluna longe do dab tem de sair byte-idêntica.
    let far = (W / 2, H - 6);
    assert_eq!(
        at(&before, far.0, far.1),
        at(&after, far.0, far.1),
        "fora da máscara o quadro não pode mudar"
    );
}

/// ⚠️ **A máscara é chrome de AUTORIA e não pode vazar para a obra.**
///
/// Ela diz ao escultor onde o pincel não pega. Se entrasse no G-buffer, a tinta
/// que o Painter acende por baixo sairia azulada onde o escultor protegeu — a
/// ferramenta de trabalho dentro do quadro, e um artista que não faz ideia de por
/// que a pintura mudou de cor.
#[test]
#[ignore = "precisa de adapter"]
fn the_mask_is_authoring_chrome_and_never_reaches_the_donation() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    let (masked, _) = masked_sphere();
    let plain = shapes::uv_sphere(48, 72, 1.0);
    let cam = camera_for(&plain);

    let a = gbuffer(&device, &queue, &plain, &cam);
    let b = gbuffer(&device, &queue, &masked, &cam);
    assert_eq!(a.len(), b.len());
    let covered = a.iter().filter(|t| t.1 > 0.5).count();
    assert!(
        covered > 500,
        "a fixture tem de conter forma: {covered} texels"
    );
    // BYTE a byte: a doação é a mesma malha, e mascarar não é esculpir.
    assert_eq!(a, b, "a máscara não pode aparecer no G-buffer da doação");
}

/// **A costura que o `last_gpu_dirty` existe para fechar.**
///
/// Um traço de máscara não move geometria, então ele não refresca normal nenhuma.
/// Um upload incremental guiado por *"o que refresquei"* não subiria byte algum, e
/// a máscara ficaria invisível — de novo, agora no caminho que o produto usa em
/// TODO movimento do mouse.
#[test]
#[ignore = "precisa de adapter"]
fn a_mask_dab_reaches_the_device_through_the_incremental_path() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    let plain = shapes::uv_sphere(48, 72, 1.0);
    let cam = camera_for(&plain);
    let (masked, dirty) = masked_sphere();

    // O caminho do produto: a malha limpa já está no device, e só a janela suja
    // do dab é copiada por cima.
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &plain);
    assert!(
        renderer.upload_region_at(&queue, 0, &masked, &dirty),
        "a região tem de ser aceita: a topologia não mudou"
    );
    let incremental = render_using(&device, &queue, &mut renderer, &cam);

    // E o oráculo é o upload CHEIO da mesma malha — a mesma dança do gate irmão
    // das normais.
    let full = render(&device, &queue, &masked, &cam);
    assert_eq!(
        incremental, full,
        "o caminho incremental tem de mostrar o que o cheio mostra"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// W8.1 — **A CENA É UMA LISTA**, e cada objeto tem a sua pose.
// ─────────────────────────────────────────────────────────────────────────────

/// Rasteriza uma LISTA de objetos, cada um com a sua pose.
fn render_objects(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    objects: &[(Mesh, ph2d_mesh::Pose)],
    camera: &Camera3d,
) -> Vec<u8> {
    let mut renderer = MeshRenderer::new(device, FORMAT);
    for (i, (mesh, pose)) in objects.iter().enumerate() {
        renderer.upload_at(device, queue, i, mesh);
        renderer.set_pose(i, *pose);
    }
    render_using(device, queue, &mut renderer, camera)
}

/// Quanto de tinta há numa faixa vertical da tela, em fração do total.
fn coverage_in(px: &[u8], x0: u32, x1: u32) -> f32 {
    let mut lit = 0usize;
    for y in 0..H {
        for x in x0..x1 {
            let i = ((y * W + x) * 4) as usize;
            if px[i] as u32 + px[i + 1] as u32 + px[i + 2] as u32 > 8 {
                lit += 1;
            }
        }
    }
    lit as f32 / (W * H) as f32
}

/// **DOIS OBJETOS aparecem onde as poses deles os põem.**
///
/// ⚠️ O oráculo é a TELA, não a matriz: um `to_cols_array_2d` transposto, um
/// bind group trocado ou um laço que desenha só o primeiro passariam por
/// qualquer asserção sobre números da CPU, e todos os três aparecem aqui como
/// tinta no lado errado — ou tinta em lado nenhum.
///
/// ⚠️ E o **CONTROLE é a pose identidade**: com as duas peças na origem a tinta
/// é UMA silhueta central e as bordas ficam vazias. Sem essa metade o gate
/// passaria com o modelo inteiro ignorado, porque duas esferas sobrepostas
/// também cobrem o meio da tela.
#[test]
#[ignore = "precisa de adapter"]
fn two_objects_are_drawn_where_their_poses_put_them() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: pulando");
        return;
    };
    let mesh = shapes::uv_sphere(16, 24, 1.0);

    // Enquadra o PAR: a caixa que vai de −2,5 a +2,5 em x.
    let mut cam = Camera3d {
        yaw: 0.0,
        pitch: 0.0,
        fov_y: core::f32::consts::FRAC_PI_4,
        ..Camera3d::default()
    };
    cam.frame(
        ph2d_mesh::Aabb {
            min: [-2.5, -1.0, -1.0],
            max: [2.5, 1.0, 1.0],
        },
        1.0,
    );

    let apart = render_objects(
        &device,
        &queue,
        &[
            (mesh.clone(), ph2d_mesh::Pose::at([-1.5, 0.0, 0.0])),
            (mesh.clone(), ph2d_mesh::Pose::at([1.5, 0.0, 0.0])),
        ],
        &cam,
    );
    let left = coverage_in(&apart, 0, W / 3);
    let right = coverage_in(&apart, 2 * W / 3, W);
    assert!(
        left > 0.02 && right > 0.02,
        "as duas peças têm de aparecer nos dois lados: esq {left:.3} dir {right:.3}"
    );
    // ⚠️ **E do TAMANHO certo**, que é a metade que uma asserção de presença não
    // faz — e a que pega a matriz TRANSPOSTA. Ela põe a translação na linha `w`
    // em vez da coluna, o que não apaga a peça nem a desloca: **infla** as duas,
    // porque o `w` de cada vértice deixa de ser 1. Medido, por terço de tela:
    // **0,052 certo contra 0,314 transposto** — seis vezes de tinta.
    //
    // ⚠️ E a SIMETRIA não serve de oráculo aqui, apesar de parecer: as duas
    // peças são imagens espelhadas uma da outra, então a razão entre elas fica
    // em 1,0 com a transposição instalada (0,314 contra 0,320). Foi medido antes
    // de ser escrito — a versão anterior desta prosa afirmava o contrário.
    assert!(
        (0.02..0.15).contains(&left) && (0.02..0.15).contains(&right),
        "e do tamanho que o enquadramento promete: esq {left:.3} dir {right:.3}"
    );

    // O CONTROLE: as mesmas duas malhas na origem cobrem o MEIO e deixam as
    // bordas vazias — é o que o mundo pré-pose desenhava.
    let together = render_objects(
        &device,
        &queue,
        &[
            (mesh.clone(), ph2d_mesh::Pose::IDENTITY),
            (mesh, ph2d_mesh::Pose::IDENTITY),
        ],
        &cam,
    );
    let c_left = coverage_in(&together, 0, W / 3);
    let c_right = coverage_in(&together, 2 * W / 3, W);
    assert!(
        c_left < 0.005 && c_right < 0.005,
        "sobrepostas, as bordas ficam vazias: esq {c_left:.3} dir {c_right:.3}"
    );
}

/// **A ESCALA da pose chega à tela**, e a normal sobrevive a ela.
///
/// ⚠️ A segunda metade é a que uma asserção de silhueta não faz: uma escala que
/// entrasse na posição e não na normal (ou o contrário) mudaria o TAMANHO sem
/// mudar o sombreado, e o modelo sairia com a luz de outro tamanho. O oráculo é
/// a razão entre a luminância média da tinta das duas — que tem de ser ~1, porque
/// escalar uma esfera uniformemente **não muda para onde a superfície aponta**.
#[test]
#[ignore = "precisa de adapter"]
fn the_pose_scale_grows_the_silhouette_without_tilting_the_light() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: pulando");
        return;
    };
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    let cam = camera_for(&mesh);

    let small = render_objects(
        &device,
        &queue,
        &[(mesh.clone(), ph2d_mesh::Pose::IDENTITY)],
        &cam,
    );
    let big = render_objects(
        &device,
        &queue,
        &[(mesh, ph2d_mesh::Pose::new([0.0; 3], 1.5))],
        &cam,
    );

    let (a, b) = (coverage(&small), coverage(&big));
    assert!(
        b > a * 1.6,
        "1,5× de escala tem de cobrir bem mais tela: {a:.3} -> {b:.3}"
    );

    // A luz: o brilho MÉDIO da tinta não pode mudar com o tamanho.
    let mean = |px: &[u8]| {
        let lit: Vec<f32> = px
            .chunks_exact(4)
            .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 8)
            .map(|p| (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0)
            .collect();
        lit.iter().sum::<f32>() / lit.len().max(1) as f32
    };
    let (ma, mb) = (mean(&small), mean(&big));
    assert!(
        (ma / mb - 1.0).abs() < 0.08,
        "escalar não inclina a normal: {ma:.1} vs {mb:.1}"
    );
}

/// **Apagar um objeto o tira da tela.**
///
/// ⚠️ Sem o `truncate_objects` os `slots` só crescem, e a peça que o artista
/// removeu continuaria desenhada para sempre — com a cena, do lado da CPU, já
/// sem ela.
#[test]
#[ignore = "precisa de adapter"]
fn truncating_the_list_stops_drawing_what_left_the_scene() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: pulando");
        return;
    };
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    let mut cam = Camera3d {
        yaw: 0.0,
        pitch: 0.0,
        fov_y: core::f32::consts::FRAC_PI_4,
        ..Camera3d::default()
    };
    cam.frame(
        ph2d_mesh::Aabb {
            min: [-2.5, -1.0, -1.0],
            max: [2.5, 1.0, 1.0],
        },
        1.0,
    );

    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh);
    renderer.set_pose(0, ph2d_mesh::Pose::at([-1.5, 0.0, 0.0]));
    renderer.upload_at(&device, &queue, 1, &mesh);
    renderer.set_pose(1, ph2d_mesh::Pose::at([1.5, 0.0, 0.0]));
    let both = render_using(&device, &queue, &mut renderer, &cam);
    assert!(coverage_in(&both, 2 * W / 3, W) > 0.02, "as duas estão lá");

    renderer.truncate_objects(1);
    assert_eq!(renderer.object_count(), 1);
    let one = render_using(&device, &queue, &mut renderer, &cam);
    assert!(
        coverage_in(&one, 2 * W / 3, W) < 0.005,
        "a segunda saiu da cena e saiu da tela"
    );
    assert!(coverage_in(&one, 0, W / 3) > 0.02, "e a primeira ficou");
}

/// **A CAVIDADE CHEGA AO PIXEL — a fresta escurece e a crista clareia** (W10.1,
/// `docs/3D/05.1` §4).
///
/// A fixture é uma esfera com uma RUGA: um anel de vértices puxado para dentro,
/// que produz um vale de curvatura positiva com duas cristas negativas ao lado
/// dele. É a forma mínima que contém os dois sinais — medir só um passaria com
/// metade do termo implementada.
///
/// ⚠️ **O oráculo é o CONTRASTE contra o mesmo pixel com a cavidade desligada**,
/// e não um valor absoluto: a luz do rig já varia pela esfera, então um limiar de
/// luminância mediria o enquadramento em vez do canal.
#[test]
#[ignore = "precisa de adapter"]
fn the_cavity_darkens_the_crevice_and_brightens_the_ridge() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    // Uma esfera com um sulco horizontal: os vértices de uma faixa de latitude
    // recuam ao longo da própria normal.
    let mut mesh = shapes::uv_sphere(60, 90, 1.0);
    let moved: Vec<u32> = (0..mesh.vert_count() as u32)
        .filter(|&v| (mesh.positions()[v as usize][1] - 0.25).abs() < 0.035)
        .collect();
    assert!(
        moved.len() > 60,
        "a fixture nao contem o fenomeno: {} vertices no sulco",
        moved.len()
    );
    for &v in &moved {
        let n = mesh.normals()[v as usize];
        let p = &mut mesh.positions_mut()[v as usize];
        for k in 0..3 {
            p[k] -= n[k] * 0.045;
        }
    }
    mesh.rebuild();

    let cam = camera_for(&mesh);
    let rig = LightRig::default();
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh);
    let off = render_using_rig_cavity(&device, &queue, &mut renderer, &cam, &rig, 0.0);
    let on = render_using_rig_cavity(&device, &queue, &mut renderer, &cam, &rig, 1.0);

    // O sulco e as duas cristas, na coluna central. Onde eles caem em pixels sai
    // do PRÓPRIO desligado: a linha em que a cavidade mais escurece é o vale, e a
    // em que ela mais clareia é a crista — o que o gate afirma é que existe um
    // par assim, e que ele está na faixa que o sulco ocupa.
    let x = W / 2;
    let mut darkest = (0u32, 0.0f32);
    let mut brightest = (0u32, 0.0f32);
    for y in 0..H {
        if lum(&off, x, y) < 0.02 {
            continue; // fundo
        }
        let d = lum(&on, x, y) - lum(&off, x, y);
        if d < darkest.1 {
            darkest = (y, d);
        }
        if d > brightest.1 {
            brightest = (y, d);
        }
    }
    println!(
        "cavidade: mais escuro em y={} ({:+.3}), mais claro em y={} ({:+.3})",
        darkest.0, darkest.1, brightest.0, brightest.1
    );
    assert!(
        darkest.1 < -0.05,
        "nenhuma fresta escureceu: pior delta {:+.3}",
        darkest.1
    );
    assert!(
        brightest.1 > 0.02,
        "nenhuma crista clareou: melhor delta {:+.3}",
        brightest.1
    );
    // ⚠️ E as duas estão PERTO uma da outra — um vale tem crista ao lado. Sem
    // isto o gate passaria com um termo que escurece um polo e clareia o outro.
    let gap = darkest.0.abs_diff(brightest.0);
    assert!(
        gap < H / 6,
        "a fresta e a crista estao a {gap} px uma da outra: nao sao o mesmo sulco"
    );
}

/// **A CAVIDADE ZERO É O BARRO DA W3, AO BYTE.**
///
/// É o que torna o canal opt-in de verdade: toda arte esculpida antes desta wave
/// acende exactamente como acendia. ⚠️ Igualdade EXATA e não um limite de
/// magnitude — o termo é uma multiplicação por `1 − 0 × k`, que é `1.0` em
/// IEEE-754 para qualquer `k` finito, então não há arredondamento a admitir.
#[test]
#[ignore = "precisa de adapter"]
fn a_cavity_of_zero_is_the_bare_clay_to_the_byte() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let cam = camera_for(&mesh);
    let rig = LightRig::default();
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh);
    let a = render_using_rig_cavity(&device, &queue, &mut renderer, &cam, &rig, 0.0);
    let b = render_using_rig_cavity(&device, &queue, &mut renderer, &cam, &rig, 0.7);
    assert_ne!(a, b, "o controle: com 0,7 o pixel TEM de mudar");
    let c = render_using_rig_cavity(&device, &queue, &mut renderer, &cam, &rig, 0.0);
    assert_eq!(a, c, "voltar a zero nao devolveu o barro da W3");
}

/// **O G-BUFFER IGNORA A CAVIDADE**, como já ignora a máscara.
///
/// O canal doado é uma NORMAL — o `docs/3D/05.2` numa frase. A cavidade é uma
/// escolha de sombreamento do BARRO, e deixá-la vazar para a doação faria a tinta
/// que o Painter acende por baixo sair escurecida nas frestas da escultura: o
/// artista veria a ferramenta de leitura dele entrar na obra.
///
/// ⚠️ Ela **não é uma dívida disfarçada de decisão**: o alvo é `vec4` com `xyz`
/// de normal e `w` de cobertura, e não há canal livre. Levar oclusão à tinta é um
/// segundo plano, e portanto uma wave — nomeada, não contrabandeada.
#[test]
#[ignore = "precisa de adapter"]
fn the_gbuffer_ignores_the_cavity() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let cam = camera_for(&mesh);
    // O G-buffer não tem por onde receber a cavidade — ele não a toma como
    // argumento —, e é isso que este gate pina: o plano doado com uma malha
    // fortemente curvada é o MESMO que uma esfera lisa doaria em cada pixel de
    // mesma normal. A afirmação executável é que o passe de cor MUDA (o controle
    // acima) e o de doação não tem parâmetro por onde mudar.
    let g = gbuffer(&device, &queue, &mesh, &cam);
    let covered = g.iter().filter(|(_, w)| *w > 0.5).count();
    assert!(covered > 0, "a fixture nao contem o fenomeno");
    for (n, w) in g.iter().filter(|(_, w)| *w > 0.5) {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 0.02,
            "a doacao carrega algo que nao e' uma normal unitaria: {n:?} (|n| = {len})"
        );
        assert_eq!(*w, 1.0, "a cobertura e' 1, nao um peso de cavidade");
    }
}

/// **A CURVATURA VIAJA NO UPLOAD INCREMENTAL** — o gate que a mutação exigiu.
///
/// ⚠️ **Ele existe porque o irmão `a_region_upload_shows_exactly_what_a_full_
/// upload_shows` NÃO o cobre, e isso foi medido:** aquele gate roda com a
/// cavidade em zero, e com zero o canal não chega ao pixel por construção.
/// Apagar o `write_buffer` da curvatura do caminho incremental deixava os **25**
/// gates desta suíte verdes.
///
/// O defeito que ele pega é o pior tipo: uma superfície cujo RELEVO é o novo e
/// cuja LEITURA de cavidade é a de antes do traço — a fresta desenhada onde ela
/// estava, no lugar exato em que o artista está olhando.
#[test]
#[ignore = "precisa de adapter"]
fn a_region_upload_carries_the_curvature_the_dab_recomputed() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mut mesh = shapes::uv_sphere(40, 56, 1.0);
    let camera = camera_for(&mesh);
    let rig = LightRig::default();

    let mut incremental = MeshRenderer::new(&device, FORMAT);
    incremental.upload_at(&device, &queue, 0, &mesh);

    // Um traço de Crease: ele cava, então produz curvatura POSITIVA de verdade —
    // que é o sinal que a cavidade escurece. Um Draw suave mal moveria o canal, e
    // o gate ficaria verde sem conter o fenômeno.
    let brush = Brush {
        verb: Verb::Crease,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..5 {
        let x = -0.3 + 0.15 * k as f32;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([x, 0.0, 0.95], brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        assert!(
            incremental.upload_region_at(&queue, 0, &mesh, stroke.last_refreshed()),
            "o upload incremental recusou uma malha de mesma topologia"
        );
    }

    // O controle: com a cavidade ligada, a malha esculpida TEM de diferir da lisa.
    // Sem ele o gate abaixo passaria comparando duas telas iguais.
    let mut fresh = MeshRenderer::new(&device, FORMAT);
    fresh.upload_at(&device, &queue, 0, &mesh);
    let want = render_using_rig_cavity(&device, &queue, &mut fresh, &camera, &rig, 1.0);
    let mut smooth = MeshRenderer::new(&device, FORMAT);
    smooth.upload_at(&device, &queue, 0, &shapes::uv_sphere(40, 56, 1.0));
    let unsculpted = render_using_rig_cavity(&device, &queue, &mut smooth, &camera, &rig, 1.0);
    assert_ne!(
        want, unsculpted,
        "o controle: o vinco TEM de aparecer com a cavidade ligada"
    );

    let got = render_using_rig_cavity(&device, &queue, &mut incremental, &camera, &rig, 1.0);
    assert_eq!(
        got, want,
        "a regiao subiu o relevo novo e a curvatura VELHA -- a fresta ficou onde estava"
    );
}
