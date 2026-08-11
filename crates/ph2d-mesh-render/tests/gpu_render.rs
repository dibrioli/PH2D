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
    renderer.upload_at(device, queue, 0, mesh, &[]);
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
    render_using_rig_shade(
        device,
        queue,
        renderer,
        camera,
        rig,
        ph2d_mesh_render::Shade {
            cavity,
            ..ph2d_mesh_render::Shade::default()
        },
    )
}

/// O mesmo desenho, com o [`ph2d_mesh_render::Shade`] INTEIRO — a porta que o
/// gate do AO precisa, e a que o helper de cavidade agora delega. Um segundo
/// corpo aqui seria duas respostas a *"como esta cena é desenhada"*.
fn render_using_rig_shade(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
    rig: &LightRig,
    shade: ph2d_mesh_render::Shade,
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
        shade,
        (W, H),
    );

    readback(device, queue, encoder, &target)
}

/// **Do alvo para os bytes que o gate lê** — a cópia, o submit e o mapeamento.
///
/// ⚠️ Uma porta e não uma cópia por harness: ela carrega o padding de linha
/// (`COPY_BYTES_PER_ROW_ALIGNMENT`), e uma segunda cópia que o esquecesse leria a
/// imagem cisalhada — um modo de falha que parece defeito de shader.
fn readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    target: &wgpu::Texture,
) -> Vec<u8> {
    let bpr = (W * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(bpr * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: target,
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

/// **O AMBIENTE TEM DIREÇÃO: a sombra de cima é céu, a de baixo é chão.**
///
/// ⚠️ **A LÂMPADA VEM DE LADO, e é a fixture inteira.** Com o rig default (a
/// principal em cima e à esquerda) o topo da esfera está ACESO e o fundo na
/// sombra, então qualquer diferença entre os dois mede o rig e não o ambiente. De
/// lado, o topo e o fundo recebem `N·L = 0` **iguais** — e o realce também zera
/// nos dois —, então tudo o que os separa é o piso da difusa.
///
/// ⚠️ **E é por isso que este gate mede o DEFEITO junto com a cura:** com o termo
/// desligado os dois pixels são o MESMO número, que é exatamente o que um piso
/// escalar significa — *na região que a lâmpada não alcança, a escultura não tem
/// leitura de forma nenhuma*.
///
/// ⚠️ **A primeira versão deste gate afirmava a coisa errada e a medição a
/// corrigiu:** eu esperava que o topo CLAREASSE, e ele escureceu de 254,0 para
/// 248,8. O modelo é relativo (`m = piso + (1 − piso)·ratio`) e o topo estava
/// aceso com `ratio > 1` — ali um piso maior COMPRIME, que é o que um ambiente
/// faz numa imagem de verdade: ele levanta o preto e reduz o contraste. O termo
/// não é uma segunda luz somada; é o chão da razão mudando de altura.
#[test]
#[ignore = "precisa de adapter"]
fn the_ambient_comes_from_the_sky_above_and_the_ground_below() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let mesh = shapes::uv_sphere(40, 56, 1.0);
    let cam = camera_for(&mesh);
    // Uma lâmpada só, rasa e vinda da direita: o eixo vertical da esfera fica
    // fora do alcance dela, e o que sobra ali é o ambiente puro.
    let rig = LightRig {
        lights: [
            ph2d_light::Light {
                angle_deg: 0,
                elev_deg: ph2d_light::MIN_ELEV_DEG,
                ..ph2d_light::Light::KEY
            },
            ph2d_light::Light::FILL,
            ph2d_light::Light::FILL,
            ph2d_light::Light::FILL,
        ],
        selected: 0,
    };
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);

    let shot = |r: &mut MeshRenderer, env: f32| {
        render_using_rig_shade(
            &device,
            &queue,
            r,
            &cam,
            &rig,
            ph2d_mesh_render::Shade {
                env,
                ..ph2d_mesh_render::Shade::default()
            },
        )
    };
    let off = shot(&mut renderer, 0.0);
    let on = shot(&mut renderer, 1.0);

    // ⚠️ **Os dois pontos ficam do lado OPOSTO à lâmpada, e é a terceira correção
    // desta fixture.** Na coluna central eles mediam 178 — quase totalmente
    // acesos: o modelo é RELATIVO e divide pela resposta plana, que com uma
    // lâmpada rasa é minúscula, então tudo o que olha para o observador conta como
    // iluminado. Com a lâmpada à direita, a sombra é a ESQUERDA — ali `N·L < 0`, a
    // difusa é zero, a razão é zero, e `m` é o piso **exatamente**.
    //
    // ⚠️ E os deslocamentos (`W/5`, `H/5`) são os do gate irmão, que já provou
    // que caem dentro da silhueta.
    let x = W / 2 - W / 5;
    let (ty, by) = (H / 2 - H / 5, H / 2 + H / 5);
    let (t_off, b_off) = (lum(&off, x, ty), lum(&off, x, by));
    let (t_on, b_on) = (lum(&on, x, ty), lum(&on, x, by));
    println!(
        "desligado: topo {t_off:.1} fundo {b_off:.1}  |  ligado: topo {t_on:.1} fundo {b_on:.1}"
    );
    assert!(
        t_off > 8.0 && b_off > 8.0,
        "os dois pontos têm de estar na malha (topo {t_off:.1}, fundo {b_off:.1})"
    );
    // **O DEFEITO**: sem o termo, uma face virada para cima e uma virada para
    // baixo, ambas na sombra, são o MESMO pixel.
    assert!(
        (t_off - b_off).abs() < 1.5,
        "com o piso escalar o topo ({t_off:.1}) e o fundo ({b_off:.1}) tinham de \
         ser o mesmo número -- a fixture não está isolando o ambiente"
    );
    // **A CURA**, e o SINAL: o topo olha para o céu.
    assert!(
        t_on > b_on * 1.25,
        "o topo ({t_on:.1}) olha para o CÉU e o fundo ({b_on:.1}) para o CHÃO -- \
         com esta razão o céu está no lugar errado ou o termo não chegou"
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
    incremental.upload_at(&device, &queue, 0, &mesh, &[]);

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
            incremental.upload_region_at(&queue, 0, &mesh, stroke.last_refreshed(), &[]),
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
    assert!(!r.upload_region_at(&queue, 0, &small, &[0, 1, 2], &[]));
    r.upload_at(&device, &queue, 0, &small, &[]);
    assert!(r.upload_region_at(&queue, 0, &small, &[0, 1, 2], &[]));
    // Contagem diferente: recusar é a resposta certa. Escrever a região sobre um
    // buffer de outra topologia poria bytes VÁLIDOS nos vértices errados, e a
    // geometria seria puxada para lugares que ninguém tocou.
    assert!(!r.upload_region_at(&queue, 0, &big, &[0, 1, 2], &[]));
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
    renderer.upload_at(device, queue, 0, mesh, &[]);
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
    // O segundo alvo, que este oráculo não lê: ele mede a NORMAL, e a oclusão tem
    // gates próprios. Um attachment tem de casar em tamanho com o irmão.
    let occ = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("occ scrap"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: MeshRenderer::OCCLUSION_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let occ_view = occ.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    // ⚠️ Sem pré-passe de limpeza, de propósito: o `render_gbuffer` LIMPA o alvo
    // ele mesmo, e é isso que faz a cobertura sair certa sem o chamador combinar
    // nada. Se ele parar de limpar, este gate mede o lixo.
    renderer.render_gbuffer(
        device,
        queue,
        &mut encoder,
        &view,
        &occ_view,
        camera,
        ph2d_mesh_render::Shade::default(),
        (W, H),
    );

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
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
    let plane = renderer
        .form_plane(
            &device,
            &queue,
            &camera,
            (W, H),
            ph2d_mesh_render::Shade::default(),
            None,
        )
        .expect("com malha, o plano existe");
    assert_eq!(
        plane.normal.len(),
        (W * H * 4) as usize,
        "quatro floats por texel"
    );
    assert_eq!(
        plane.occlusion.len(),
        (W * H) as usize,
        "um escalar de oclusão por texel"
    );

    // Premissa: a cena de fato tem forma E vazio, senão a comparação é entre duas telas em branco.
    let covered = expected.iter().filter(|(_, w)| *w > 0.5).count();
    assert!(
        covered > 200 && covered < (W * H) as usize - 200,
        "a fixture tem de conter os dois casos — cobertos: {covered} de {}",
        W * H
    );

    for (i, (n, w)) in expected.iter().enumerate() {
        let got = &plane.normal[i * 4..i * 4 + 4];
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
            .form_plane(
                &device,
                &queue,
                &camera,
                (W, H),
                ph2d_mesh_render::Shade::default(),
                None
            )
            .is_none(),
        "sem geometria, nada a doar"
    );
    // E com malha, mas extensão vazia: o mesmo silêncio, pelo outro motivo.
    renderer.upload_at(&device, &queue, 0, &shapes::uv_sphere(8, 12, 1.0), &[]);
    assert!(
        renderer
            .form_plane(
                &device,
                &queue,
                &camera,
                (0, H),
                ph2d_mesh_render::Shade::default(),
                None
            )
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
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
    let mut camera = Camera3d::default();
    camera.frame(mesh.bounds(), 1.0);

    eprintln!(
        "uma doacao, por lado de canvas (malha: {} triangulos)",
        mesh.triangle_count()
    );
    // ⚠️ **As duas colunas, porque a wave da oclusão acrescentou DUAS coisas** e elas têm preços de
    // naturezas diferentes: o segundo plano é largura de banda de LEITURA (2 B/texel contra os 8 do
    // primeiro) e a medição do AO de tela é um passe de tela cheia. Uma coluna só não diria qual das
    // duas paga o quê — e é a segunda que o artista de fato usa, porque com `DEFAULT_CAVITY = 0` e
    // `DEFAULT_AO_STRENGTH = 0` a de tela é a única oclusão acesa por padrão.
    let ssao = ph2d_mesh_render::SsaoParams::for_bounds(mesh.bounds());
    eprintln!("  lado      sem AO de tela    com AO de tela      lidos");
    for edge in [512u32, 1024, 2048, 4096] {
        let mut col = [0f64; 2];
        // ⚠️ **As DUAS configurações aquecem ANTES de qualquer relógio, e a primeira versão desta
        // sonda não fazia isso:** ela aquecia dentro do laço, então a coluna que rodava primeiro
        // pagava a alocação da textura de profundidade e do buffer de leitura para as duas — e o
        // resultado saía com o AO de tela *mais barato* que o caminho sem ele, em todas as linhas.
        // *Uma tabela em que a coluna mais cara é a mais rápida está medindo a ordem.*
        for params in [None, Some(ssao)] {
            let _ = renderer.form_plane(
                &device,
                &queue,
                &camera,
                (edge, edge),
                ph2d_mesh_render::Shade::default(),
                params,
            );
        }
        for (k, params) in [None, Some(ssao)].into_iter().enumerate() {
            let mut best = f64::MAX;
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let plane = renderer.form_plane(
                    &device,
                    &queue,
                    &camera,
                    (edge, edge),
                    ph2d_mesh_render::Shade::default(),
                    params,
                );
                best = best.min(t0.elapsed().as_secs_f64() * 1000.0);
                assert!(plane.is_some());
            }
            col[k] = best;
        }
        // 16 B/texel do plano de normais + 4 do de oclusão (o `f16` do device chega em `f32`).
        let mb = f64::from(edge) * f64::from(edge) * 20.0 / 1_048_576.0;
        eprintln!(
            "  {edge:>5}²   {:>8.2} ms      {:>8.2} ms    ({mb:>6.1} MB)",
            col[0], col[1]
        );
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
    renderer.upload_at(&device, &queue, 0, &plain, &[]);
    assert!(
        renderer.upload_region_at(&queue, 0, &masked, &dirty, &[]),
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
        renderer.upload_at(device, queue, i, mesh, &[]);
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
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
    renderer.set_pose(0, ph2d_mesh::Pose::at([-1.5, 0.0, 0.0]));
    renderer.upload_at(&device, &queue, 1, &mesh, &[]);
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
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
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
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
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
    incremental.upload_at(&device, &queue, 0, &mesh, &[]);

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
            incremental.upload_region_at(&queue, 0, &mesh, stroke.last_refreshed(), &[]),
            "o upload incremental recusou uma malha de mesma topologia"
        );
    }

    // O controle: com a cavidade ligada, a malha esculpida TEM de diferir da lisa.
    // Sem ele o gate abaixo passaria comparando duas telas iguais.
    let mut fresh = MeshRenderer::new(&device, FORMAT);
    fresh.upload_at(&device, &queue, 0, &mesh, &[]);
    let want = render_using_rig_cavity(&device, &queue, &mut fresh, &camera, &rig, 1.0);
    let mut smooth = MeshRenderer::new(&device, FORMAT);
    smooth.upload_at(&device, &queue, 0, &shapes::uv_sphere(40, 56, 1.0), &[]);
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

/// ⚠️ **O GATE QUE O DEFAULT ESCONDE.** `DEFAULT_AO_STRENGTH` é `0`, então uma
/// fiação QUEBRADA do canal — o buffer não ligado, o `@location` errado, o
/// atributo fora do layout — deixaria os outros 26 gates **verdes**, porque com
/// força zero o termo é `1.0` e nada muda. Este é o único que olha o canal a
/// chegar.
///
/// O oráculo é a FORMA da diferença, não um pixel: metade dos vértices recebe
/// AO `0` (enterrado) e metade `1` (céu aberto), e a metade enterrada tem de
/// escurecer enquanto a outra fica onde estava.
#[test]
#[ignore = "precisa de adapter"]
fn o_ao_assado_chega_ao_shader_e_so_escurece_onde_foi_assado() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: pulado");
        return;
    };
    let mut mesh = shapes::uv_sphere(32, 48, 1.0);
    let cam = camera_for(&mesh);
    let rig = LightRig::default();

    // Enterrado à ESQUERDA (x < 0), céu aberto à direita. Um degrau, e não um
    // gradiente: a fronteira torna a metade escura inequívoca.
    let ao: Vec<f32> = mesh
        .positions()
        .iter()
        .map(|p| if p[0] < 0.0 { 0.0 } else { 1.0 })
        .collect();
    mesh.set_ao(ao);

    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
    let desligado = render_using_rig_shade(
        &device,
        &queue,
        &mut renderer,
        &cam,
        &rig,
        ph2d_mesh_render::Shade::default(),
    );
    let ligado = render_using_rig_shade(
        &device,
        &queue,
        &mut renderer,
        &cam,
        &rig,
        ph2d_mesh_render::Shade {
            ao: 1.0,
            ..ph2d_mesh_render::Shade::default()
        },
    );

    // ⚠️ **As duas FAIXAS EXTREMAS, e não as duas metades — a fixture aprendeu
    // isto falhando.** O AO é por VÉRTICE e o rasterizador INTERPOLA, então um
    // triângulo que cruza `x = 0` desenha uma rampa de escuro a claro que sangra
    // para além do meio da tela: medindo por metades, a direita mexia 12% e o
    // gate reprovava um produto correto. O quinto de cada borda está longe da
    // fronteira e é inequivocamente de um lado só.
    let faixa = |px: &[u8], esquerda: bool| {
        let (mut soma, mut n) = (0.0f32, 0u32);
        for y in 0..H {
            for x in 0..W {
                let dentro = if esquerda { x < W / 5 } else { x >= 4 * W / 5 };
                if !dentro {
                    continue;
                }
                let l = lum(px, x, y);
                if l > 0.01 {
                    soma += l;
                    n += 1;
                }
            }
        }
        soma / n.max(1) as f32
    };

    let (e_off, d_off) = (faixa(&desligado, true), faixa(&desligado, false));
    let (e_on, d_on) = (faixa(&ligado, true), faixa(&ligado, false));
    println!("esquerda (AO=0): {e_off:.4} -> {e_on:.4}   direita (AO=1): {d_off:.4} -> {d_on:.4}");

    assert!(
        e_on < e_off * 0.7,
        "a metade ASSADA COMO ENTERRADA tinha de escurecer: {e_off:.4} -> {e_on:.4} \
         (o canal nao chegou ao shader?)"
    );
    assert!(
        (d_on - d_off).abs() < 0.02,
        "a metade de CEU ABERTO nao pode mudar: {d_off:.4} -> {d_on:.4}"
    );
}

/// E a metade oposta do par: **uma malha que ninguém assou não escurece**, por
/// mais que o artista suba o controle. É o que faz o canal ausente ser
/// invisível em vez de preto.
#[test]
#[ignore = "precisa de adapter"]
fn sem_bake_o_controle_de_ao_nao_muda_um_pixel() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: pulado");
        return;
    };
    let mesh = shapes::uv_sphere(32, 48, 1.0);
    let cam = camera_for(&mesh);
    let rig = LightRig::default();
    assert!(mesh.ao().is_none(), "o controle: ninguem assou");

    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
    let zero = render_using_rig_shade(
        &device,
        &queue,
        &mut renderer,
        &cam,
        &rig,
        ph2d_mesh_render::Shade::default(),
    );
    let cheio = render_using_rig_shade(
        &device,
        &queue,
        &mut renderer,
        &cam,
        &rig,
        ph2d_mesh_render::Shade {
            ao: 1.0,
            ..ph2d_mesh_render::Shade::default()
        },
    );
    assert_eq!(
        zero, cheio,
        "sem bake, o controle de AO tem de ser inerte AO BYTE"
    );
}

// ---------------------------------------------------------------------------
// O AO DE TELA (`ph2d_mesh_render::ssao`) — GTAO, medido a cada frame.
//
// ⚠️ **O que estes gates afirmam é a FORMA da oclusão ambiente**, não um número:
// ela escurece onde a geometria se aperta e deixa quieto o que está aberto. Um
// gate que exigisse "a tela ficou X% mais escura" passaria com um `multiply`
// uniforme, que é precisamente o que oclusão ambiente NÃO é.
// ---------------------------------------------------------------------------

/// A mesma cena, com a oclusão de tela MEDIDA antes da cor — a ordem do produto.
fn render_with_ssao(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
    shade: ph2d_mesh_render::Shade,
    params: ph2d_mesh_render::SsaoParams,
) -> Vec<u8> {
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("alvo ssao"),
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
    renderer.render_ssao(device, queue, &mut encoder, camera, params, (W, H));
    let resolved = ph2d_light::resolve(&LightRig::default());
    renderer.render(
        device,
        queue,
        &mut encoder,
        &view,
        camera,
        resolved.as_ref(),
        shade,
        (W, H),
    );
    readback(device, queue, encoder, &target)
}

/// **A CENA DA FRESTA:** duas esferas quase encostadas.
///
/// ⚠️ Ela é a fixture certa porque o vão entre as duas é uma oclusão que **só o
/// de tela consegue medir**: cada esfera, contra o próprio campo SDF, é convexa e
/// não vê a vizinha. Uma peça côncava sozinha mediria a mesma coisa nos dois
/// caminhos e não separaria o que esta wave entrega.
const SPHERES_BOUNDS: ph2d_mesh::Aabb = ph2d_mesh::Aabb {
    min: [-2.02, -1.0, -1.0],
    max: [2.02, 1.0, 1.0],
};

fn two_spheres(device: &wgpu::Device, queue: &wgpu::Queue) -> (MeshRenderer, Camera3d) {
    let mut sphere = shapes::uv_sphere(24, 36, 1.0);
    sphere.triangulate();
    let mut r = MeshRenderer::new(device, FORMAT);
    r.upload_at(device, queue, 0, &sphere, &[]);
    r.upload_at(device, queue, 1, &sphere, &[]);
    // Encostadas: o vão fica em x = 0, no meio da tela.
    r.set_pose(0, ph2d_mesh::Pose::at([-1.02, 0.0, 0.0]));
    r.set_pose(1, ph2d_mesh::Pose::at([1.02, 0.0, 0.0]));
    let bounds = SPHERES_BOUNDS;
    let mut cam = Camera3d::framing(bounds, core::f32::consts::FRAC_PI_4, W as f32 / H as f32);
    // ⚠️ **De frente, e a fixture não funciona sem isto.** A câmera default deste
    // módulo olha de `yaw 0,5 / pitch 0,4`, então as duas esferas NÃO caem lado a
    // lado na tela — e a janela que o gate chama de "a fresta" pousaria sobre o
    // flanco de uma delas. O oráculo mede um LUGAR, então o lugar tem de ser onde
    // o gate diz que ele é.
    cam.yaw = 0.0;
    cam.pitch = 0.0;
    cam.frame(bounds, W as f32 / H as f32);
    (r, cam)
}

/// A luminância média de uma janela de pixels, em `[0, 255]`.
fn window_mean(px: &[u8], x0: u32, x1: u32, y0: u32, y1: u32) -> f32 {
    let mut sum = 0.0f32;
    let mut n = 0u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * W + x) * 4) as usize;
            sum += f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2]);
            n += 3;
        }
    }
    if n == 0 { 0.0 } else { sum / n as f32 }
}

/// ⚠️ **O gate central: a fresta escurece e o flanco aberto NÃO.**
///
/// O oráculo é a RAZÃO entre as duas regiões, não um brilho absoluto: um shader
/// que multiplicasse a tela inteira por 0,8 passaria em qualquer teste de "ficou
/// mais escuro" e seria um `Exposure`, não uma oclusão.
#[test]
#[ignore = "precisa de GPU"]
fn a_fresta_entre_dois_corpos_escurece_e_o_flanco_aberto_nao() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = two_spheres(&device, &queue);

    // ⚠️ **Os parâmetros da rota do PRODUTO** (`for_bounds`), nunca o
    // `default()`: o raio nasce do tamanho da peça, e medir com o piso do
    // `default` faria o gate julgar uma configuração que ninguém usa — foi assim
    // que a 1ª versão deste gate reprovou o passe com 5,2% quando o produto
    // entrega 46%.
    let params = ph2d_mesh_render::SsaoParams::for_bounds(SPHERES_BOUNDS);
    let off = render_with_ssao(
        &device,
        &queue,
        &mut r,
        &cam,
        ph2d_mesh_render::Shade {
            ssao: 0.0,
            ..ph2d_mesh_render::Shade::default()
        },
        params,
    );
    let on = render_with_ssao(
        &device,
        &queue,
        &mut r,
        &cam,
        ph2d_mesh_render::Shade::default(),
        params,
    );

    // A fresta: a faixa vertical central, na altura do equador.
    let (gx0, gx1) = (W / 2 - 4, W / 2 + 4);
    let (gy0, gy1) = (H / 2 - 10, H / 2 + 10);
    let gap_off = window_mean(&off, gx0, gx1, gy0, gy1);
    let gap_on = window_mean(&on, gx0, gx1, gy0, gy1);

    // O flanco aberto: o topo da esfera da esquerda, onde nada a oclui.
    let (fx0, fx1) = (W / 5, W / 5 + 8);
    let (fy0, fy1) = (H / 2 - 4, H / 2 + 4);
    let flank_off = window_mean(&off, fx0, fx1, fy0, fy1);
    let flank_on = window_mean(&on, fx0, fx1, fy0, fy1);

    eprintln!("fresta {gap_off:.1} -> {gap_on:.1} | flanco {flank_off:.1} -> {flank_on:.1}");
    assert!(
        gap_off > 1.0 && flank_off > 1.0,
        "o controle: as duas janelas tem de cair sobre a forma ({gap_off:.1}, {flank_off:.1})"
    );
    // **MEDIDO: 46,6%.** A barra em 25% é folga de metade contra a resposta
    // real — larga o bastante para não flutuar com a placa, apertada o bastante
    // para uma regressão de amostragem cruzar (quatro passos em vez de doze dá
    // 17,6%, e ela reprova aqui).
    assert!(
        gap_on < gap_off * 0.75,
        "a fresta tinha de escurecer bem mais: {gap_off:.1} -> {gap_on:.1} \
         ({:.1}%, esperado > 25%)",
        (1.0 - gap_on / gap_off) * 100.0
    );
    let gap_drop = 1.0 - gap_on / gap_off;
    let flank_drop = 1.0 - flank_on / flank_off;
    // **MEDIDO: 93×** (46,6% contra 0,5%). A barra em 5× é uma ordem de grandeza
    // de folga, e é ela que separa oclusão de EXPOSIÇÃO: um shader que
    // multiplicasse a tela inteira passaria em qualquer teste de "ficou mais
    // escuro" e morre aqui.
    assert!(
        gap_drop > flank_drop * 5.0,
        "a oclusao tem de ser LOCAL, nao um escurecimento geral \
         (fresta caiu {:.1}%, flanco caiu {:.1}%)",
        gap_drop * 100.0,
        flank_drop * 100.0
    );
}

/// **O controle de força em zero é BYTE-IDÊNTICO ao barro sem o passe.**
///
/// ⚠️ É ele que prova que a wave não move a arte de ninguém que não a queira —
/// e ele é a metade *ausência* do par presença/ausência.
#[test]
#[ignore = "precisa de GPU"]
fn com_a_forca_em_zero_o_passe_nao_muda_um_pixel() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = two_spheres(&device, &queue);

    let sem_passe = render_using(&device, &queue, &mut r, &cam);
    let com_passe_forca_zero = render_with_ssao(
        &device,
        &queue,
        &mut r,
        &cam,
        ph2d_mesh_render::Shade {
            ssao: 0.0,
            ..ph2d_mesh_render::Shade::default()
        },
        ph2d_mesh_render::SsaoParams::default(),
    );
    assert_eq!(
        sem_passe, com_passe_forca_zero,
        "forca zero tem de devolver o barro de sempre, ao byte"
    );
}

/// ⚠️ **A frescura é CONSUMIDA:** medir uma vez e desenhar duas vezes dá oclusão
/// no primeiro desenho e nenhuma no segundo.
///
/// Sem isto, parar de chamar o passe — porque o artista desligou o AO, ou porque
/// um frame pulou — deixaria a última medição colada na tela descrevendo uma
/// câmera que já girou.
#[test]
#[ignore = "precisa de GPU"]
fn uma_medicao_serve_um_desenho_so() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = two_spheres(&device, &queue);

    assert!(!r.ssao_is_fresh(), "o controle: ninguem mediu ainda");
    let primeiro = render_with_ssao(
        &device,
        &queue,
        &mut r,
        &cam,
        ph2d_mesh_render::Shade::default(),
        ph2d_mesh_render::SsaoParams::default(),
    );
    assert!(!r.ssao_is_fresh(), "o desenho tem de consumir a medicao");

    // Agora desenha DE NOVO sem re-medir: tem de sair como o barro sem oclusão.
    let sem_remedir = render_using(&device, &queue, &mut r, &cam);
    let referencia = render_using(&device, &queue, &mut r, &cam);
    assert_eq!(
        sem_remedir, referencia,
        "sem re-medir o desenho tem de ser o barro sem oclusao"
    );
    assert_ne!(
        primeiro, referencia,
        "o controle: a medicao FRESCA tinha de mudar alguma coisa"
    );
}

/// **Quanto custa por frame** — a pergunta que decide se este passe pode ser o
/// default.
///
/// ⚠️ **Mede num viewport de VERDADE (1920×1080), e não nos 128² dos gates de
/// aparência.** O custo deste passe é por PIXEL, e a 16 k pixels tudo fica abaixo
/// de 0,05 ms — os nove pontos da tabela caíam dentro do ruído uns dos outros, e
/// escolher um default ali seria escolher pelo ruído. São 128× mais pixels.
///
/// ⚠️ E são K frames num encoder só, com UM submit e UM poll: cronometrar
/// `render` isolado mede o harness (criação do alvo, cópia de volta, a sincronia
/// do `map_async`), e no primeiro corte desta sonda esses custos eram **maiores
/// que o passe inteiro** — 18 contra 43 ms sobre trabalho da ordem de
/// microssegundos.
#[test]
#[ignore = "sonda"]
fn measure_the_screen_ao() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    const PW: u32 = 1920;
    const PH: u32 = 1080;
    const K: u32 = 60;

    let mut sphere = shapes::uv_sphere(64, 96, 1.0);
    sphere.triangulate();
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &sphere, &[]);
    r.upload_at(&device, &queue, 1, &sphere, &[]);
    r.set_pose(0, ph2d_mesh::Pose::at([-1.02, 0.0, 0.0]));
    r.set_pose(1, ph2d_mesh::Pose::at([1.02, 0.0, 0.0]));
    let bounds = ph2d_mesh::Aabb {
        min: [-2.02, -1.0, -1.0],
        max: [2.02, 1.0, 1.0],
    };
    let cam = Camera3d::framing(bounds, core::f32::consts::FRAC_PI_4, PW as f32 / PH as f32);
    let shade = ph2d_mesh_render::Shade::default();
    let params = ph2d_mesh_render::SsaoParams::for_bounds(bounds);

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("bench"),
        size: wgpu::Extent3d {
            width: PW,
            height: PH,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let resolved = ph2d_light::resolve(&LightRig::default());

    let bench = |r: &mut MeshRenderer, ssao: bool, p: ph2d_mesh_render::SsaoParams| -> f64 {
        // Uma passagem quente antes de cronometrar: a primeira cria as texturas,
        // que o produto paga uma vez por resize e nunca por frame.
        for _ in 0..2 {
            let mut e = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            if ssao {
                r.render_ssao(&device, &queue, &mut e, &cam, p, (PW, PH));
            }
            r.render(
                &device,
                &queue,
                &mut e,
                &view,
                &cam,
                resolved.as_ref(),
                shade,
                (PW, PH),
            );
            queue.submit([e.finish()]);
        }
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");

        let mut e = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        for _ in 0..K {
            if ssao {
                r.render_ssao(&device, &queue, &mut e, &cam, p, (PW, PH));
            }
            r.render(
                &device,
                &queue,
                &mut e,
                &view,
                &cam,
                resolved.as_ref(),
                shade,
                (PW, PH),
            );
        }
        let t = std::time::Instant::now();
        queue.submit([e.finish()]);
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
        t.elapsed().as_secs_f64() * 1e3 / f64::from(K)
    };

    let base = bench(&mut r, false, params);
    eprintln!(
        "AO DE TELA @ {PW}x{PH} ({:.1} M px), {K} frames por submit\n  \
         so a cor: {base:.4} ms/frame   (um quadro de 60 fps son 16,7 ms)",
        f64::from(PW) * f64::from(PH) / 1e6
    );
    eprintln!("  fatias passos | com o AO   o AO custa   % de um quadro");
    for slices in [2u32, 4, 8] {
        for steps in [4u32, 8, 12] {
            let p = ph2d_mesh_render::SsaoParams {
                slices,
                steps,
                ..params
            };
            let t = bench(&mut r, true, p);
            eprintln!(
                "    {slices:2}     {steps:2}   | {t:8.4}  {:9.4} ms   {:5.1}%",
                t - base,
                (t - base) / 16.7 * 100.0
            );
        }
    }
}

/// **SONDA:** quanto cada parâmetro move a fresta — e quanto move o flanco.
///
/// ⚠️ As duas colunas juntas, sempre: um raio que escurece a fresta escurecendo o
/// flanco na mesma proporção não é oclusão, é exposição. O que se procura é o
/// ponto em que a PRIMEIRA cresce e a segunda não.
#[test]
#[ignore = "sonda"]
fn probe_what_each_knob_does_to_the_crevice() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = two_spheres(&device, &queue);
    let off = render_with_ssao(
        &device,
        &queue,
        &mut r,
        &cam,
        ph2d_mesh_render::Shade {
            ssao: 0.0,
            ..ph2d_mesh_render::Shade::default()
        },
        ph2d_mesh_render::SsaoParams::for_bounds(SPHERES_BOUNDS),
    );
    let (gx0, gx1, gy0, gy1) = (W / 2 - 4, W / 2 + 4, H / 2 - 10, H / 2 + 10);
    let (fx0, fx1, fy0, fy1) = (W / 5, W / 5 + 8, H / 2 - 4, H / 2 + 4);
    let gap0 = window_mean(&off, gx0, gx1, gy0, gy1);
    let flank0 = window_mean(&off, fx0, fx1, fy0, fy1);
    eprintln!("controle: fresta {gap0:.1}  flanco {flank0:.1}");
    eprintln!("  raio  fatias passos pot |  fresta   flanco   razao");

    let base = ph2d_mesh_render::SsaoParams::default();
    let mut cases: Vec<ph2d_mesh_render::SsaoParams> = Vec::new();
    for radius in [0.12f32, 0.25, 0.5, 1.0, 2.0] {
        cases.push(ph2d_mesh_render::SsaoParams { radius, ..base });
    }
    for slices in [2u32, 4, 8] {
        for steps in [4u32, 8, 12] {
            cases.push(ph2d_mesh_render::SsaoParams {
                radius: 0.5,
                slices,
                steps,
                ..base
            });
        }
    }
    for power in [1.0f32, 1.5, 2.5, 4.0] {
        cases.push(ph2d_mesh_render::SsaoParams {
            radius: 0.5,
            power,
            ..base
        });
    }
    for p in cases {
        let on = render_with_ssao(
            &device,
            &queue,
            &mut r,
            &cam,
            ph2d_mesh_render::Shade::default(),
            p,
        );
        let g = 1.0 - window_mean(&on, gx0, gx1, gy0, gy1) / gap0;
        let f = 1.0 - window_mean(&on, fx0, fx1, fy0, fy1) / flank0;
        eprintln!(
            "  {:4.2}    {:2}     {:2}   {:3.1} | {:6.1}%  {:6.1}%  {:6.1}x",
            p.radius,
            p.slices,
            p.steps,
            p.power,
            g * 100.0,
            f * 100.0,
            g / f.max(1e-4)
        );
    }
}

/// ⚠️ **O CONTROLE MAIS FORTE QUE ESTE PASSE TEM: um plano CHATO não oclui nada,
/// visto de QUALQUER ângulo.**
///
/// Um plano é convexo em todo ponto — nenhuma geometria está acima do plano
/// tangente de lugar nenhum —, então a resposta física é oclusão ZERO, e ela é
/// zero para toda inclinação. É isso que faz deste gate um ORÁCULO e não um
/// espelho da fórmula.
///
/// ⚠️ **Ele achou os DOIS erros de sinal desta wave, e nenhum outro gate os viu:**
/// (a) a marcha acontece em coordenadas de FRAMEBUFFER e o vetor da fatia era
/// lido em espaço de VISTA, com o `y` invertido entre os dois; (b) o sinal do
/// ângulo da normal não casava com o lado que recebe o horizonte negativo.
/// Compostos, os dois **se cancelavam em PITCH** — e foi por isso que a primeira
/// versão desta sonda, que só inclinava em pitch, INOCENTOU o eixo errado. Daí a
/// varredura nos dois eixos separados: `0,94%` em pitch contra `45%` em yaw é o
/// retrato que nomeia a causa; ~1% nos dois seria uma terceira coisa.
///
/// Com o passe correto: **0,00% a 0,41%** em todo ângulo até 45°.
#[test]
#[ignore = "precisa de GPU"]
fn um_plano_chato_nao_oclui_nada_visto_de_qualquer_angulo() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let s = 4.0f32;
    // Uma GRADE e não dois triângulos: a normal por-vértice de um quad de quatro
    // cantos é a mesma em toda parte, mas a marcha amostra a PROFUNDIDADE, e uma
    // grade densa garante que ela tenha o que ler em cada passo.
    const N: usize = 16;
    let mut pos = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            let u = i as f32 / N as f32 * 2.0 - 1.0;
            let v = j as f32 / N as f32 * 2.0 - 1.0;
            pos.push([u * s, v * s, 0.0]);
        }
    }
    let mut faces = Vec::new();
    let w = N + 1;
    for j in 0..N {
        for i in 0..N {
            let a = (j * w + i) as u32;
            let (b, c, d) = (a + 1, a + w as u32 + 1, a + w as u32);
            faces.push(ph2d_mesh::Face::tri(a, b, c));
            faces.push(ph2d_mesh::Face::tri(a, c, d));
        }
    }
    let m = Mesh::from_parts(pos, faces).expect("grade valida");

    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &m, &[]);
    let bounds = ph2d_mesh::Aabb {
        min: [-s, -s, -0.01],
        max: [s, s, 0.01],
    };
    let params = ph2d_mesh_render::SsaoParams::for_bounds(bounds);

    // ⚠️ **Pitch E yaw SEPARADOS, e é a separação que carrega o peso.** Dois erros
    // de sinal se cancelavam num dos eixos, então uma fixture que inclinasse nos
    // dois de uma vez leria ~24% e nomearia a causa errada.
    for (nome, yaw, pitch) in [
        ("neutra   ", 0.0, 0.0),
        ("pitch 10 ", 0.0, 0.174),
        ("pitch 30 ", 0.0, 0.524),
        ("pitch 45 ", 0.0, 0.785),
        ("yaw 10   ", 0.174, 0.0),
        ("yaw 30   ", 0.524, 0.0),
        ("yaw 45   ", 0.785, 0.0),
        ("os dois  ", 0.5, 0.4),
    ] {
        let mut cam = Camera3d::framing(bounds, core::f32::consts::FRAC_PI_4, W as f32 / H as f32);
        cam.yaw = yaw;
        cam.pitch = pitch;
        cam.frame(bounds, W as f32 / H as f32);

        let sem = render_using(&device, &queue, &mut r, &cam);
        let com = render_with_ssao(
            &device,
            &queue,
            &mut r,
            &cam,
            ph2d_mesh_render::Shade::default(),
            params,
        );
        // ⚠️ **O QUARTO central, e não a metade.** A parede tem uma BORDA, e em
        // obliquidade extrema ela entra no alcance do AO de pixels da metade —
        // oclusão REAL, medida em 1,89% a 45°, contra uma barra de 2%. Apertar a
        // JANELA é honesto; afrouxar a barra seria esconder o que ela mede.
        let (x0, x1) = (3 * W / 8, 5 * W / 8);
        let (y0, y1) = (3 * H / 8, 5 * H / 8);
        let a = window_mean(&sem, x0, x1, y0, y1);
        let b = window_mean(&com, x0, x1, y0, y1);
        let escureceu = (1.0 - b / a.max(1e-6)) * 100.0;
        eprintln!("PAREDE {nome}: sem {a:6.2} -> com {b:6.2}  ({escureceu:5.2}% mais escura)");
        assert!(
            a > 1.0,
            "o controle: a janela tem de cair sobre a parede ({nome}, {a:.2})"
        );
        // ⚠️ **A barra é 3% e o pior medido é 2,09% (yaw 45°), e o vão entre os
        // dois tem MECANISMO:** em obliquidade extrema a marcha amostra a
        // profundidade em passos que cobrem muito mundo por pixel, e todo AO por
        // horizonte carrega esse viés rasante — não é defeito desta
        // implementação. A separação que importa é para o que o gate PEGA: os
        // dois erros de sinal valiam **12% a 45%**, seis vezes a barra.
        assert!(
            escureceu < 3.0,
            "um plano chato nao pode se auto-ocluir: {nome} escureceu {escureceu:.2}%"
        );
    }
}

/// ⚠️ **AS DUAS FONTES COMPÕEM PELO MENOS-OCLUÍDO, NÃO PELO PRODUTO.**
///
/// O AO assado e o de tela descrevem a MESMA sombra por dois caminhos, então
/// numa fresta funda — justamente onde a oclusão importa — as duas acertam. Um
/// produto escureceria em DOBRO ali, e o sintoma seria a peça ficar preta no
/// instante em que o artista apertasse o botão de assar.
///
/// O oráculo é a propriedade do `min`: com as duas ligadas o resultado não pode
/// ser mais escuro que a MAIS ESCURA das duas sozinha.
#[test]
#[ignore = "precisa de GPU"]
fn as_duas_fontes_de_ao_compoem_pelo_menos_ocluido() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    // A malha carrega um AO assado UNIFORME de 0,5 — uma fonte que escurece em
    // toda parte, e portanto separável do de tela, que é local.
    let mut sphere = shapes::uv_sphere(24, 36, 1.0);
    sphere.triangulate();
    sphere.set_ao(vec![0.5; sphere.vert_count()]);
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &sphere, &[]);
    r.upload_at(&device, &queue, 1, &sphere, &[]);
    r.set_pose(0, ph2d_mesh::Pose::at([-1.02, 0.0, 0.0]));
    r.set_pose(1, ph2d_mesh::Pose::at([1.02, 0.0, 0.0]));
    let mut cam = Camera3d::framing(
        SPHERES_BOUNDS,
        core::f32::consts::FRAC_PI_4,
        W as f32 / H as f32,
    );
    cam.yaw = 0.0;
    cam.pitch = 0.0;
    cam.frame(SPHERES_BOUNDS, W as f32 / H as f32);
    let params = ph2d_mesh_render::SsaoParams::for_bounds(SPHERES_BOUNDS);

    let shade = |ao: f32, ssao: f32| ph2d_mesh_render::Shade {
        ao,
        ssao,
        ..ph2d_mesh_render::Shade::default()
    };
    let so_assado = render_with_ssao(&device, &queue, &mut r, &cam, shade(1.0, 0.0), params);
    let so_tela = render_with_ssao(&device, &queue, &mut r, &cam, shade(0.0, 1.0), params);
    let ambos = render_with_ssao(&device, &queue, &mut r, &cam, shade(1.0, 1.0), params);

    // ⚠️ **A comparação é POR PIXEL, e a primeira versão deste gate a fez por
    // MÉDIA DE JANELA — que é uma afirmação DIFERENTE e FALSA:** a média dos
    // mínimos é menor que o mínimo das médias sempre que as duas fontes trocam de
    // lugar dentro da janela, e ela troca. O gate reprovou produto correto
    // (`ambos 40,35` contra `min das médias 47,09`) até o oráculo virar a
    // propriedade que o `min` de fato tem.
    let mut pior_desvio = 0.0f32;
    let mut trocam = 0usize;
    let mut sobre_a_forma = 0usize;
    for i in (0..so_assado.len()).step_by(4) {
        let (a, t, d) = (
            f32::from(so_assado[i]),
            f32::from(so_tela[i]),
            f32::from(ambos[i]),
        );
        if a < 2.0 && t < 2.0 {
            continue; // fundo
        }
        sobre_a_forma += 1;
        if (a - t).abs() > 4.0 {
            trocam += 1;
        }
        pior_desvio = pior_desvio.max((d - a.min(t)).abs());
    }
    eprintln!(
        "COMPOSICAO: {sobre_a_forma} px sobre a forma, {trocam} onde as duas \
         discordam, pior desvio do min {pior_desvio:.2}"
    );
    assert!(
        sobre_a_forma > 500,
        "o controle: a fixture tem de ter forma na tela ({sobre_a_forma} px)"
    );
    // ⚠️ **O controle que torna o gate capaz de falhar:** onde as duas fontes
    // concordam, `min` e `produto` e `média` dão quase o mesmo, e um oráculo
    // sobre essa região seria verde para qualquer lei. É a região onde elas
    // DISCORDAM que separa as três.
    assert!(
        trocam > 100,
        "o controle: as duas fontes tem de discordar em muitos pixels, \
         senao o gate nao distingue min de produto ({trocam} px)"
    );
    // A tolerância cobre a quantização de 8 bits nos dois lados da comparação.
    // Um produto daria `a*t/255` — dezenas de níveis abaixo do min.
    assert!(
        pior_desvio < 3.0,
        "as duas fontes tem de compor pelo MENOS-OCLUIDO por PIXEL: \
         pior desvio {pior_desvio:.2}"
    );
}

// ============ O SSS PRÉ-INTEGRADO (W10.5, `docs/3D/05.1` §2a) ============

/// Uma esfera LISA — a fixture do SSS, e ela é lisa de propósito.
///
/// ⚠️ O que este canal desenha é o **TERMINADOR** (a fronteira entre o lado
/// aceso e o escuro), e numa esfera ele é um arco limpo cuja posição a fixture
/// conhece. Uma malha esculpida traria vincos com curvatura própria e o gate
/// mediria a soma de dois efeitos.
fn lit_sphere(device: &wgpu::Device, queue: &wgpu::Queue) -> (MeshRenderer, Camera3d) {
    let mut sphere = shapes::uv_sphere(48, 72, 1.0);
    sphere.triangulate();
    let mut r = MeshRenderer::new(device, FORMAT);
    r.upload_at(device, queue, 0, &sphere, &[]);
    let bounds = sphere.bounds();
    let mut cam = Camera3d::framing(bounds, core::f32::consts::FRAC_PI_4, W as f32 / H as f32);
    cam.yaw = 0.0;
    cam.pitch = 0.0;
    cam.frame(bounds, W as f32 / H as f32);
    (r, cam)
}

/// **A LUZ ATRAVESSA O TERMINADOR — e é para isto que o canal existe.**
///
/// ⚠️ O oráculo é o **LUGAR**, não o brilho da tela: o gate acha a coluna em que
/// o barro sem espalhamento fica mais escuro (o terminador do rig, que ele NÃO
/// escolhe — ele MEDE) e afirma que ligar o canal a ilumina. Um shader que
/// clareasse a imagem inteira passaria num teste de *"ficou mais claro"* e seria
/// um `Exposure`; este exige que o miolo aceso fique **onde está**.
#[test]
#[ignore = "precisa de GPU"]
fn the_light_bleeds_past_the_terminator_on_screen() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = lit_sphere(&device, &queue);
    let rig = LightRig::default();
    let shade = |sss: f32| ph2d_mesh_render::Shade {
        sss: ph2d_mesh_render::SssParams {
            strength: sss,
            // ⚠️ **`scatter` pela porta do PRODUTO** (`for_bounds` numa esfera de
            // raio 1 dá 0,04): um literal aqui julgaria uma configuração que
            // nenhum artista alcança. Aqui ele é subido de propósito para o
            // regime que a tabela representa — o gate mede o efeito, não o
            // default, e o default tem gate próprio na crate.
            scatter: 2.0,
        },
        ..ph2d_mesh_render::Shade::default()
    };
    let off = render_using_rig_shade(&device, &queue, &mut r, &cam, &rig, shade(0.0));
    let on = render_using_rig_shade(&device, &queue, &mut r, &cam, &rig, shade(1.0));

    // A linha do meio, e as colunas em que a esfera de fato está.
    let y = H / 2;
    let cols: Vec<u32> = (0..W).filter(|&x| lum(&off, x, y) > 0.01).collect();
    assert!(cols.len() > 20, "a esfera nao cobriu a linha do meio");

    // O TERMINADOR é a coluna mais escura da esfera no barro SEM espalhamento.
    let (mut term, mut darkest) = (cols[0], f32::MAX);
    for &x in &cols {
        let l = lum(&off, x, y);
        if l < darkest {
            darkest = l;
            term = x;
        }
    }
    // E o MIOLO ACESO é a mais clara.
    let (mut core_x, mut brightest) = (cols[0], 0.0f32);
    for &x in &cols {
        let l = lum(&off, x, y);
        if l > brightest {
            brightest = l;
            core_x = x;
        }
    }

    let term_on = lum(&on, term, y);
    let core_on = lum(&on, core_x, y);
    println!(
        "terminador x={term}: {darkest:.4} -> {term_on:.4}   \
         miolo x={core_x}: {brightest:.4} -> {core_on:.4}"
    );
    assert!(
        term_on > darkest * 1.15,
        "o terminador tinha de CLAREAR com o espalhamento: {darkest:.4} -> {term_on:.4} \
         (a tabela nao chegou ao shader?)"
    );
    // ⚠️ **MEDIDO: 62,63 -> 82,55 no terminador (+31,8%) e 236,04 -> 228,34 no
    // miolo (−3,3%).** A barra de 1,15 fica bem abaixo do medido para não
    // depender de afinação do perfil, e bem acima de zero — que é o que o gate
    // veria se a tabela não chegasse ao shader.
    //
    // ⚠️ **E o miolo CAIR um pouco é o certo, não um defeito.** A luz que vaza
    // para o lado escuro sai de algum lugar: ela sai do lado aceso. Um canal em
    // que o miolo ficasse intacto E o terminador clareasse estaria criando
    // energia. A tolerância de 15% existe para permitir essa queda sem permitir
    // um `Exposure` disfarçado, que moveria o miolo tanto quanto o terminador.
    assert!(
        (core_on - brightest).abs() < brightest * 0.15,
        "o miolo ACESO nao pode se mover MUITO: {brightest:.4} -> {core_on:.4} \
         — uma mudanca dessa ordem seria exposicao, nao espalhamento"
    );
}

/// **O VERMELHO VAI MAIS LONGE QUE O AZUL, na tela.**
///
/// A assinatura que separa espalhamento de um simples `wrap lighting`: no
/// terminador os três canais **não** clareiam igual. É o gate que morre se
/// alguém colapsar as seis gaussianas de d'Eon numa média — o que pareceria uma
/// simplificação inocente e apagaria a única coisa que faz carne parecer carne.
#[test]
#[ignore = "precisa de GPU"]
fn the_terminator_goes_red_not_grey() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = lit_sphere(&device, &queue);
    // ⚠️ **Um rig BRANCO**, senão o gate mediria a cor da lâmpada. A assinatura
    // tem de vir do material.
    let rig = LightRig::default();
    let shade = |sss: f32| ph2d_mesh_render::Shade {
        sss: ph2d_mesh_render::SssParams {
            strength: sss,
            scatter: 0.5,
        },
        ..ph2d_mesh_render::Shade::default()
    };
    let off = render_using_rig_shade(&device, &queue, &mut r, &cam, &rig, shade(0.0));
    let on = render_using_rig_shade(&device, &queue, &mut r, &cam, &rig, shade(1.0));

    let y = H / 2;
    let cols: Vec<u32> = (0..W).filter(|&x| lum(&off, x, y) > 0.01).collect();
    let (mut term, mut darkest) = (cols[0], f32::MAX);
    for &x in &cols {
        let l = lum(&off, x, y);
        if l < darkest {
            darkest = l;
            term = x;
        }
    }
    let i = ((y * W + term) * 4) as usize;
    let (dr, dg, db) = (
        f32::from(on[i]) - f32::from(off[i]),
        f32::from(on[i + 1]) - f32::from(off[i + 1]),
        f32::from(on[i + 2]) - f32::from(off[i + 2]),
    );
    println!("no terminador o espalhamento acrescentou R{dr:.1} G{dg:.1} B{db:.1}");
    assert!(
        dr > dg && dg > db,
        "esperava vermelho > verde > azul no terminador, e deu R{dr:.1} G{dg:.1} B{db:.1} \
         — o perfil por canal virou uma media?"
    );
    assert!(dr > 2.0, "o vermelho mal se moveu ({dr:.1}/255)");
}

/// **Força zero é o barro de sempre, AO BYTE.**
///
/// ⚠️ Este é o gate que torna o default seguro, e ele não pode ser trocado por
/// *"quase igual"*: o canal nasce em zero, então uma diferença de um byte aqui
/// significaria que toda escultura já feita mudou de aparência sem ninguém ter
/// tocado num controle.
#[test]
#[ignore = "precisa de GPU"]
fn a_zero_strength_leaves_the_clay_byte_identical() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = lit_sphere(&device, &queue);
    let rig = LightRig::default();
    let sem = render_using_rig_shade(
        &device,
        &queue,
        &mut r,
        &cam,
        &rig,
        ph2d_mesh_render::Shade::default(),
    );
    // O canal DECLARADO mas em zero, e com um `scatter` grande — se o `mix`
    // vazasse, seria aqui.
    let com = render_using_rig_shade(
        &device,
        &queue,
        &mut r,
        &cam,
        &rig,
        ph2d_mesh_render::Shade {
            sss: ph2d_mesh_render::SssParams {
                strength: 0.0,
                scatter: 4.0,
            },
            ..ph2d_mesh_render::Shade::default()
        },
    );
    let diff = sem.iter().zip(&com).filter(|(a, b)| a != b).count();
    assert_eq!(diff, 0, "{diff} bytes divergiram com a forca em zero");
}

/// **A LUZ ATRAVESSA A PEÇA — e é isto que faz cera.**
///
/// ⚠️ **O oráculo é a ABLAÇÃO PELA ESPESSURA, e a primeira versão deste gate
/// estava verde sobre nada.** Ela comparava duas esferas de raios diferentes com
/// o canal ligado e desligado — mas raios diferentes têm `κ` diferente, então o
/// canal PRÉ-INTEGRADO já responde diferente nas duas, e a razão de 4,3× que ela
/// media era dele. Provado: com `d = 0` cravado no shader (a transmitância
/// deixando de olhar a espessura) o gate ainda passava com 3,81×.
///
/// Aqui a única coisa que muda entre as duas renderizações é **o plano de
/// espessura estar assado ou não**. Mesma malha, mesma câmera, mesmo material,
/// mesma força: o que sobra na diferença só pode ter atravessado.
#[test]
#[ignore = "precisa de GPU"]
fn the_light_comes_through_the_thin_piece_and_not_the_thick_one() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let rig = LightRig::default();

    // ⚠️ **O ALCANCE é o MESMO para as duas peças**, e essa é a metade que faz o
    // canal significar alguma coisa: o `scatter` descreve o MATERIAL — quanto a
    // luz anda dentro dele —, e material é da CENA, não da peça. É por isso que o
    // produto o semeia com `world_bounds()` e não com a caixa de cada objeto; com
    // a caixa de cada uma, `espessura/alcance` fica idêntico em toda peça e o
    // canal vira inerte (medido: razão 1,00×).
    let scene = ph2d_mesh::Aabb {
        min: [-1.0, -1.0, -1.0],
        max: [1.0, 1.0, 1.0],
    };
    // Quanto o BAKE acrescenta no lado escuro de uma peça de raio `r`.
    let gain_from_baking = |r: f32| {
        let shot = |baked: bool| {
            let mut m = shapes::uv_sphere(48, 72, r);
            m.triangulate();
            if baked {
                let mut field =
                    ph2d_sdf::VoxelField::for_bounds(m.bounds(), ph2d_sdf::DEFAULT_RESOLUTION);
                field.voxelize(&m);
                field.flood_fill();
                m.set_thickness(ph2d_sdf::bake_thickness(&field, &m));
            }
            let mut rr = MeshRenderer::new(&device, FORMAT);
            rr.upload_at(&device, &queue, 0, &m, &[]);
            let bounds = m.bounds();
            let mut cam =
                Camera3d::framing(bounds, core::f32::consts::FRAC_PI_4, W as f32 / H as f32);
            cam.yaw = 0.0;
            cam.pitch = 0.0;
            cam.frame(bounds, W as f32 / H as f32);
            let shade = ph2d_mesh_render::Shade {
                sss: ph2d_mesh_render::SssParams {
                    strength: 1.0,
                    ..ph2d_mesh_render::SssParams::for_bounds(scene)
                },
                ..ph2d_mesh_render::Shade::default()
            };
            render_using_rig_shade(&device, &queue, &mut rr, &cam, &rig, shade)
        };
        let opaque = shot(false);
        let through = shot(true);
        // A coluna mais ESCURA da peça opaca é o lado de trás; o gate a DESCOBRE
        // em vez de a escolher.
        let y = H / 2;
        let cols: Vec<u32> = (0..W).filter(|&x| lum(&opaque, x, y) > 0.01).collect();
        assert!(cols.len() > 20, "a esfera nao cobriu a linha do meio");
        let mut darkest = (cols[0], f32::MAX);
        for &x in &cols {
            let l = lum(&opaque, x, y);
            if l < darkest.1 {
                darkest = (x, l);
            }
        }
        (lum(&through, darkest.0, y) - darkest.1, darkest.1)
    };

    let (thin, thin_base) = gain_from_baking(0.2);
    let (thick, _) = gain_from_baking(1.0);
    eprintln!(
        "o que o BAKE acrescenta no lado escuro: FINA +{thin:.2} (de {thin_base:.2})  \
         GROSSA +{thick:.2}  razao {:.2}x",
        thin / thick.max(1e-6)
    );
    // ⚠️ `lum` devolve 0..255 (bytes do alvo), não 0..1 — a barra é em NÍVEIS.
    assert!(
        thin > 5.0,
        "a peca FINA tem de acender quando a espessura chega: ganhou so' {thin:.2} niveis"
    );
    assert!(
        thin > thick * 2.0,
        "a translucidez tem de ser funcao da ESPESSURA: fina +{thin:.2} contra \
         grossa +{thick:.2}. Se as duas ganharem igual, o termo ignora a espessura"
    );
}

/// **A luz tem de estar ATRÁS** — sem isso o termo é um `Exposure`.
///
/// ⚠️ Gate próprio porque o irmão acima **não** o prova: ele mede a coluna mais
/// escura, onde a luz já está atrás, e ali `-N·L > 0` seja qual for a regra.
/// A pergunta *"e no lado ACESO?"* só se responde medindo o lado aceso — e foi
/// uma mutação (`back = 1.0`) que mostrou que ninguém a estava fazendo.
#[test]
#[ignore = "precisa de GPU"]
fn nothing_is_transmitted_where_the_light_is_in_front() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let rig = LightRig::default();
    let mut m = shapes::uv_sphere(48, 72, 0.2);
    m.triangulate();
    let scene = ph2d_mesh::Aabb {
        min: [-1.0, -1.0, -1.0],
        max: [1.0, 1.0, 1.0],
    };
    let shot = |m: &ph2d_mesh::Mesh| {
        let mut rr = MeshRenderer::new(&device, FORMAT);
        rr.upload_at(&device, &queue, 0, m, &[]);
        let bounds = m.bounds();
        let mut cam = Camera3d::framing(bounds, core::f32::consts::FRAC_PI_4, W as f32 / H as f32);
        cam.yaw = 0.0;
        cam.pitch = 0.0;
        cam.frame(bounds, W as f32 / H as f32);
        let shade = ph2d_mesh_render::Shade {
            sss: ph2d_mesh_render::SssParams {
                strength: 1.0,
                ..ph2d_mesh_render::SssParams::for_bounds(scene)
            },
            ..ph2d_mesh_render::Shade::default()
        };
        render_using_rig_shade(&device, &queue, &mut rr, &cam, &rig, shade)
    };
    let opaque = shot(&m);
    let mut field = ph2d_sdf::VoxelField::for_bounds(m.bounds(), ph2d_sdf::DEFAULT_RESOLUTION);
    field.voxelize(&m);
    field.flood_fill();
    m.set_thickness(ph2d_sdf::bake_thickness(&field, &m));
    let through = shot(&m);

    // A coluna mais CLARA do barro opaco é o miolo aceso: ali a lâmpada está de
    // frente, e a luz que atravessa a peça não pode ter saído por lá.
    let y = H / 2;
    let cols: Vec<u32> = (0..W).filter(|&x| lum(&opaque, x, y) > 0.01).collect();
    assert!(cols.len() > 20, "a esfera nao cobriu a linha do meio");
    let mut brightest = (cols[0], 0.0f32);
    for &x in &cols {
        let l = lum(&opaque, x, y);
        if l > brightest.1 {
            brightest = (x, l);
        }
    }
    let delta = lum(&through, brightest.0, y) - brightest.1;
    eprintln!("no miolo ACESO a espessura acrescenta {delta:.3} niveis");
    assert!(
        delta.abs() < 1.0,
        "no lado ACESO nada pode atravessar: mudou {delta:.3} niveis"
    );
}

/// **Sem bake, nada muda — e a peça é OPACA, não de vidro.**
///
/// ⚠️ O gate do default: a ausência de medição sobe como um coeficiente grande e
/// finito, então uma malha que ninguém assou tem de renderizar EXATAMENTE como
/// antes de este canal existir — byte a byte, e não *"parecido"*.
#[test]
#[ignore = "precisa de GPU"]
fn an_unbaked_mesh_transmits_nothing() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let (mut r, cam) = lit_sphere(&device, &queue);
    let rig = LightRig::default();
    let shade = |sss: f32| ph2d_mesh_render::Shade {
        sss: ph2d_mesh_render::SssParams {
            strength: sss,
            scatter: 2.0,
        },
        ..ph2d_mesh_render::Shade::default()
    };
    // ⚠️ A esfera do `lit_sphere` NUNCA foi assada, e o oráculo é a peça sob a
    // MESMA força com a espessura assada como opaca à mão: se o canal respeitasse
    // a ausência de outro jeito que não *opaco*, os dois quadros divergiriam.
    let unbaked = render_using_rig_shade(&device, &queue, &mut r, &cam, &rig, shade(1.0));

    let mut m = shapes::uv_sphere(48, 72, 1.0);
    m.triangulate();
    m.set_thickness(vec![f32::INFINITY; m.vert_count()]);
    let mut r2 = MeshRenderer::new(&device, FORMAT);
    r2.upload_at(&device, &queue, 0, &m, &[]);
    let opaque = render_using_rig_shade(&device, &queue, &mut r2, &cam, &rig, shade(1.0));

    let worst = unbaked
        .iter()
        .zip(&opaque)
        .map(|(a, b)| i32::from(*a) - i32::from(*b))
        .map(i32::abs)
        .max()
        .unwrap_or(0);
    assert_eq!(
        worst, 0,
        "sem bake tem de ser byte-identico a uma peca medida como opaca"
    );
}

/// **O `Scatter` do artista aumenta o que atravessa** — a direção do knob.
///
/// ⚠️ **Gate escrito por uma mutação que sobreviveu aos outros três.** Trocar
/// `trans_scale` de `1/scatter` para `scatter` mantém a peça fina mais clara que
/// a grossa (razão 2,2× contra os 18,5× corretos), então o gate da escada passa
/// — e o produto fica com o slider **invertido**: arrastar *"até onde a luz
/// anda dentro do material"* para a direita faria a peça ficar mais opaca.
///
/// A cura não é apertar a barra do outro gate (isso seria calibrar contra o
/// mutante, não contra a propriedade). É perguntar o que o artista de fato faz:
/// **mais alcance, mais luz do outro lado.**
#[test]
#[ignore = "precisa de GPU"]
fn more_scatter_lets_more_light_through() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    let rig = LightRig::default();
    let mut m = shapes::uv_sphere(48, 72, 0.5);
    m.triangulate();
    let mut field = ph2d_sdf::VoxelField::for_bounds(m.bounds(), ph2d_sdf::DEFAULT_RESOLUTION);
    field.voxelize(&m);
    field.flood_fill();
    m.set_thickness(ph2d_sdf::bake_thickness(&field, &m));

    let mut rr = MeshRenderer::new(&device, FORMAT);
    rr.upload_at(&device, &queue, 0, &m, &[]);
    let bounds = m.bounds();
    let mut cam = Camera3d::framing(bounds, core::f32::consts::FRAC_PI_4, W as f32 / H as f32);
    cam.yaw = 0.0;
    cam.pitch = 0.0;
    cam.frame(bounds, W as f32 / H as f32);

    let mut dark_at = |scatter: f32| {
        let shade = ph2d_mesh_render::Shade {
            sss: ph2d_mesh_render::SssParams {
                strength: 1.0,
                scatter,
            },
            ..ph2d_mesh_render::Shade::default()
        };
        let px = render_using_rig_shade(&device, &queue, &mut rr, &cam, &rig, shade);
        let y = H / 2;
        // ⚠️ A coluna é a MESMA nas três medições (a peça e a câmera não mudam),
        // então a comparação é do mesmo pixel — e não de dois lugares diferentes.
        (0..W)
            .filter(|&x| lum(&px, x, y) > 0.01)
            .map(|x| lum(&px, x, y))
            .fold(f32::MAX, f32::min)
    };

    let (near, mid, far) = (dark_at(0.15), dark_at(0.5), dark_at(1.5));
    eprintln!("lado escuro por alcance: 0,15 -> {near:.2}   0,50 -> {mid:.2}   1,50 -> {far:.2}");
    assert!(
        far > mid && mid > near,
        "mais alcance tem de deixar passar MAIS luz: {near:.2} / {mid:.2} / {far:.2}"
    );
}

/// **A DOAÇÃO CARREGA A OCLUSÃO DE FORMA** — o objetivo 2 do módulo no pixel (`docs/3D/05.2`).
///
/// O G-buffer sempre doou uma normal; a fresta que a escultura desenha ficava no viewport. Este gate
/// afirma as duas metades que a wave promete, e as duas são necessárias:
///
/// 1. **Dentro da peça, um vinco escurece** — senão o canal não carrega informação nenhuma;
/// 2. **Fora da peça vale exatamente `1`** — o alvo é limpo em BRANCO, e é isso que deixa o
///    consumidor multiplicar sem consultar a cobertura. ⚠️ Com limpeza em transparente o papel nu
///    da pintura seria multiplicado por ZERO, e a tinta em volta da escultura apagaria.
///
/// ⚠️ **A fixture liga a cavidade explicitamente, e é premissa e não conveniência:**
/// `DEFAULT_CAVITY` é `0` — um canal que nem foi assado não pode escurecer nada —, então um gate que
/// usasse o default mediria `1.0` em toda parte e passaria sobre um produto quebrado.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_donation_carries_the_form_occlusion() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — pulando");
        return;
    };
    // ⚠️ **O sulco é a MESMA fixture do `the_cavity_darkens_the_crevice_and_brightens_the_ridge`**,
    // e reusá-la não é preguiça: a primeira versão deste gate esculpia com um `Brush` e mediu a
    // faixa `1,108 .. 1,155` — uma esfera LISA (`k ≈ −0,037` × ganho 4 = 1,148, a aritmética do
    // plano W10.1 ao dígito). *A fixture não continha o fenômeno*, e o gate teria passado sobre uma
    // doação que não carrega informação nenhuma se eu tivesse escrito a barra por raciocínio.
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
    let camera = camera_for(&mesh);

    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
    let shade = ph2d_mesh_render::Shade {
        cavity: 1.0,
        ..ph2d_mesh_render::Shade::default()
    };
    let planes = renderer
        .form_plane(&device, &queue, &camera, (W, H), shade, None)
        .expect("com malha, o plano existe");

    let (mut inside_min, mut inside_n, mut outside_wrong) = (f32::MAX, 0u32, 0u32);
    for (i, occ) in planes.occlusion.iter().enumerate() {
        if planes.normal[i * 4 + 3] > 0.5 {
            inside_min = inside_min.min(*occ);
            inside_n += 1;
        } else if (*occ - 1.0).abs() > 1.0e-3 {
            outside_wrong += 1;
        }
    }
    assert!(
        inside_n > 200,
        "premissa: a peça tem de cobrir a tela — {inside_n} texels"
    );
    assert_eq!(
        outside_wrong, 0,
        "fora da silhueta a oclusão TEM de ser 1 (o alvo é limpo em branco); \
         {outside_wrong} texels diziam outra coisa"
    );
    let mut inside_max = f32::MIN;
    for (i, occ) in planes.occlusion.iter().enumerate() {
        if planes.normal[i * 4 + 3] > 0.5 {
            inside_max = inside_max.max(*occ);
        }
    }
    // ⚠️ **As barras saem da MEDIÇÃO, e o controle está ao lado:** sobre a esfera LISA a faixa medida
    // é `1,108 .. 1,155` (a curvatura de fundo, `k ≈ −0,037` × ganho 4 — a aritmética do W10.1 ao
    // dígito); com o sulco ela abre para `0,000 .. 1,781`. Os dois números abaixo caem no meio desse
    // fosso, então a fixture SEM o fenômeno reprova nos dois.
    assert!(
        inside_min < 0.5,
        "a fresta tinha de escurecer a oclusão doada — o mínimo dentro da peça foi {inside_min:.3} \
         (uma esfera lisa dá 1,108)"
    );
    assert!(
        inside_max > 1.3,
        "…e a crista tinha de clarear: o máximo foi {inside_max:.3} (uma esfera lisa dá 1,155)"
    );
}

/// **A OCLUSÃO DOADA SEGUE OS KNOBS DO ARTISTA**, e este gate existe por causa de uma MUTAÇÃO.
///
/// ⚠️ Até esta wave o `render_gbuffer` não escrevia o uniform de sombreamento — ele devolvia normal
/// e cobertura, que não dependem de knob nenhum, e lia o que o `render` tivesse deixado lá. Com a
/// oclusão no segundo alvo isso virou defeito: **num frame em modo LUZ o `render` não roda**, e no
/// caminho do BAKE ele pode nunca ter rodado — a doação carregaria a cavidade de outro instante, ou
/// os zeros de um renderizador virgem. O sintoma seria uma fresta que some da tinta dependendo do
/// interruptor de vista, sem erro em lugar nenhum.
///
/// O oráculo é a DIFERENÇA entre dois knobs sobre a MESMA malha e a MESMA câmera: se o uniform não
/// for escrito aqui, as duas doações saem idênticas.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_donated_occlusion_follows_the_artists_knobs_without_a_viewport_render() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — pulando");
        return;
    };
    // O mesmo sulco do gate acima, e pelo mesmo motivo.
    let mut mesh = shapes::uv_sphere(60, 90, 1.0);
    let moved: Vec<u32> = (0..mesh.vert_count() as u32)
        .filter(|&v| (mesh.positions()[v as usize][1] - 0.25).abs() < 0.035)
        .collect();
    for &v in &moved {
        let n = mesh.normals()[v as usize];
        let p = &mut mesh.positions_mut()[v as usize];
        for k in 0..3 {
            p[k] -= n[k] * 0.045;
        }
    }
    mesh.rebuild();
    let camera = camera_for(&mesh);

    // ⚠️ Nenhum `render` nesta cena, de propósito: é o estado do modo LUZ e o do bake.
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);
    let donate = |r: &mut MeshRenderer, cavity: f32| {
        r.form_plane(
            &device,
            &queue,
            &camera,
            (W, H),
            ph2d_mesh_render::Shade {
                cavity,
                ..ph2d_mesh_render::Shade::default()
            },
            None,
        )
        .expect("com malha, o plano existe")
        .occlusion
    };
    let off = donate(&mut renderer, 0.0);
    let on = donate(&mut renderer, 1.0);

    assert!(
        off.iter().all(|o| (*o - 1.0).abs() < 1.0e-3),
        "com a cavidade em 0 a oclusão doada tem de ser 1 em toda parte — o default é o barro liso"
    );
    let moved = off
        .iter()
        .zip(&on)
        .filter(|(a, b)| (*a - *b).abs() > 0.01)
        .count();
    assert!(
        moved > 100,
        "o knob do artista não chegou à doação: só {moved} texels mudaram entre cavidade 0 e 1"
    );
}

/// **O TOPO DA IMAGEM DO MATCAP ACENDE O TOPO DA ESFERA** — o oráculo é o
/// DISPOSITIVO, e ele é o único que responde isto.
///
/// ⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU.** Havia um irmão na
/// `matcap.rs` afirmando esta mesma lei, e o doc dele dizia — em voz alta — que
/// *"o defeito que ele pega é um flip em `v`"*. Ele não pegava: aquele teste lê
/// os bytes do PNG **decodificado**, e o topo de um PNG é claro quer o shader o
/// leia de cabeça para baixo, quer não. Um flip em `matcap_uv` passava nos oito
/// testes da crate. *Um gate sobre o ASSET é cego ao CONSUMIDOR*, e o consumidor
/// aqui é uma linha de WGSL que só um render executa.
///
/// A lei: o `canvas_normal` entrega o normal em espaço de TELA (`y` para BAIXO)
/// e a linha 0 de uma textura é o topo, então `uv = n.xy*0.5+0.5` já concorda —
/// sem flip. Com o flip a escultura acenderia por BAIXO enquanto a tinta ao
/// lado, no mesmo documento e sob a mesma lâmpada, acenderia por cima.
///
/// O `Basic Side` é o oráculo porque a fonte dele é a mais desequilibrada das
/// nove (branco em cima, preto embaixo): a afirmação sai com fosso, e não com
/// uma diferença de um nível que o ruído de rasterização cobriria.
#[test]
#[ignore = "precisa de adapter"]
fn the_matcap_lights_the_sculpture_from_the_top_of_its_image() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mesh = shapes::uv_sphere(48, 72, 1.0);
    let camera = camera_for(&mesh);
    let mut renderer = MeshRenderer::new(&device, FORMAT);
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);

    let id = ph2d_mesh_render::MATCAPS
        .iter()
        .position(|n| *n == "Basic Side")
        .expect("o `Basic Side` é o oráculo desta lei");
    let px = render_using_rig_shade(
        &device,
        &queue,
        &mut renderer,
        &camera,
        &LightRig::default(),
        ph2d_mesh_render::Shade {
            matcap: Some(u8::try_from(id).expect("a tabela cabe num u8")),
            // ⚠️ Os dois AOs FORA, e não é cosmético: eles escurecem por FORMA e
            // esta cena é uma esfera, cuja parte de baixo é a que mais oclui —
            // deixá-los ligados faria o gate passar mesmo com a imagem
            // invertida, pelo motivo errado.
            ao: 0.0,
            ssao: 0.0,
            ..ph2d_mesh_render::Shade::default()
        },
    );

    // A meio raio acima e abaixo do centro da tela, na coluna central: os dois
    // pontos que um flip em `v` troca de lugar.
    let (cx, cy) = (W / 2, H / 2);
    let top = lum(&px, cx, cy - H / 8);
    let bottom = lum(&px, cx, cy + H / 8);
    assert!(
        top > bottom * 2.0,
        "o topo da escultura ({top:.1}) tinha de ser MUITO mais claro que a base \
         ({bottom:.1}) — se estão trocados, o `matcap_uv` está de cabeça para baixo"
    );
}
