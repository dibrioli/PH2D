//! **QUANTO DE UMA ARESTA CHEGA À TELA** — a sonda do report *"os wireframes
//! saem todos cortados"* (Enio, 2026-08-12, com foto).
//!
//! ```text
//! cargo test -p ph2d-mesh-render --release --test probe_wire_continuity \
//!   -- --ignored --nocapture
//! ```
//!
//! ⚠️ **O oráculo é a CONTINUIDADE, e não a presença.** Um gate que perguntasse
//! *"há tinta de wireframe na tela?"* fica verde sobre a foto do report: as
//! linhas ESTÃO lá, em pedaços. O que a foto mostra é que cada aresta chega
//! **partida**, então a grandeza é *que fração dos pixels ao longo de uma aresta
//! carrega a tinta dela*.
//!
//! ⚠️ **E ela é medida pela porta do PRODUTO** (`MeshRenderer::render` com
//! `Shade::wireframe`), nunca por um laço próprio sobre a lista de arestas: o
//! que se quer saber é o que o DEVICE desenha depois do teste de profundidade,
//! e um laço de CPU sobre `wire_indices` responderia sobre a lista, que não é a
//! pergunta.

use ph2d_light::LightRig;
use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_render::{Camera3d, MeshRenderer, Shade};

const W: u32 = 512;
const H: u32 = 512;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("sonda do wireframe"),
        ..Default::default()
    }))
    .ok()
}

fn camera_for(mesh: &Mesh) -> Camera3d {
    let mut cam = Camera3d {
        yaw: 0.0,
        pitch: 0.0,
        fov_y: core::f32::consts::FRAC_PI_4,
        ..Camera3d::default()
    };
    cam.frame(mesh.bounds(), 1.0);
    cam
}

fn render(device: &wgpu::Device, queue: &wgpu::Queue, mesh: &Mesh, wire: bool) -> Vec<u8> {
    let mut renderer = MeshRenderer::new(device, FORMAT);
    renderer.upload_at(device, queue, 0, mesh, &[]);
    renderer.upload_wire_at(device, 0, mesh);
    let camera = camera_for(mesh);
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
    let shade = Shade {
        wireframe: wire,
        ..Shade::default()
    };
    let rig = LightRig::default();
    let resolved = ph2d_light::resolve(&rig);
    renderer.render(
        device,
        queue,
        &mut encoder,
        &view,
        &camera,
        resolved.as_ref(),
        shade,
        (W, H),
    );

    let bpr = (W * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
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
            buffer: &buf,
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
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let data = buf.slice(..).get_mapped_range();
    let mut out = vec![0u8; (W * H * 4) as usize];
    for y in 0..H as usize {
        let src = y * bpr as usize;
        let dst = y * W as usize * 4;
        out[dst..dst + W as usize * 4].copy_from_slice(&data[src..src + W as usize * 4]);
    }
    drop(data);
    buf.unmap();
    out
}

fn lum(px: &[u8], x: i32, y: i32) -> Option<f32> {
    if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
        return None;
    }
    let i = ((y as u32 * W + x as u32) * 4) as usize;
    Some(0.2126 * f32::from(px[i]) + 0.7152 * f32::from(px[i + 1]) + 0.0722 * f32::from(px[i + 2]))
}

/// **A CONTINUIDADE de cada aresta** — a fração dos pontos amostrados ao longo
/// dela em que a tinta de wireframe de fato chegou.
///
/// ⚠️ **O oráculo é a DIFERENÇA contra o mesmo quadro SEM wireframe**, e não um
/// limiar de escuridão absoluto: a tinta é `rgba(0.05, 0.06, 0.08, 0.55)` sobre
/// barro cuja luminância varia de ponta a ponta da peça, então um limiar fixo
/// mediria a ILUMINAÇÃO junto e chamaria de *"aresta que sumiu"* toda aresta que
/// cai numa sombra.
fn continuity(mesh: &Mesh, plain: &[u8], wired: &[u8], cam: &Camera3d) -> (f64, usize, usize) {
    let mut wire = Vec::new();
    ph2d_mesh_render::wire_indices(mesh, &mut wire);
    let pos = mesh.positions();
    let mut hit = 0usize;
    let mut total = 0usize;
    let mut whole = 0usize;
    let mut edges = 0usize;
    for e in wire.chunks_exact(2) {
        let (a, b) = (pos[e[0] as usize], pos[e[1] as usize]);
        let (Some(pa), Some(pb)) = (cam.project(a, (W, H)), cam.project(b, (W, H))) else {
            continue;
        };
        let len = ((pb.0 - pa.0).powi(2) + (pb.1 - pa.1).powi(2)).sqrt();
        // Aresta curta demais não tem interior a amostrar: ela é UM pixel, e a
        // pergunta *"ela chegou inteira?"* não significa nada ali.
        if len < 6.0 {
            continue;
        }
        // Só as arestas VOLTADAS PARA O OLHO: uma do outro lado da peça é
        // ocultada de propósito, e contá-la mediria o depth-write, não o corte.
        let mid = [
            (a[0] + b[0]) * 0.5,
            (a[1] + b[1]) * 0.5,
            (a[2] + b[2]) * 0.5,
        ];
        if !front_facing(mesh, e[0], mid, cam) {
            continue;
        }
        edges += 1;
        let steps = len.floor() as i32;
        let mut on = 0usize;
        let mut seen = 0usize;
        // As pontas ficam de fora: um pixel de vértice pertence a várias arestas
        // e ele chegaria mesmo com o interior inteiro cortado.
        for s in 2..steps - 1 {
            let t = s as f32 / steps as f32;
            let x = (pa.0 + (pb.0 - pa.0) * t).round() as i32;
            let y = (pa.1 + (pb.1 - pa.1) * t).round() as i32;
            let (Some(p), Some(w)) = (lum(plain, x, y), lum(wired, x, y)) else {
                continue;
            };
            seen += 1;
            // A tinta é escura e semitransparente: ela só pode ESCURECER.
            if p - w > 6.0 {
                on += 1;
            }
        }
        if seen == 0 {
            continue;
        }
        hit += on;
        total += seen;
        if on == seen {
            whole += 1;
        }
    }
    (hit as f64 / total.max(1) as f64, whole, edges)
}

/// A normal do vértice aponta para o olho?
fn front_facing(mesh: &Mesh, v: u32, at: [f32; 3], cam: &Camera3d) -> bool {
    let n = mesh.normals()[v as usize];
    let eye = cam.eye();
    let d = [eye[0] - at[0], eye[1] - at[1], eye[2] - at[2]];
    n[0] * d[0] + n[1] * d[1] + n[2] * d[2] > 0.0
}

/// **O CONTROLE da sonda** — a tinta TOTAL do quadro contra o comprimento total
/// das arestas de frente.
///
/// ⚠️ **Ele existe porque a continuidade por-aresta amostra a reta que EU
/// projeto, e o device rasteriza a linha DELE.** Num segmento de ângulo raso as
/// duas escolhas de pixel divergem em metade dos passos, e a sonda reportaria
/// como *"aresta cortada"* um desencontro do amostrador. Este número é global e
/// imune a isso: se a tinta total bate com o comprimento total, as linhas estão
/// inteiras e quem erra sou eu.
fn total_ink(plain: &[u8], wired: &[u8]) -> usize {
    (0..(W * H) as usize)
        .filter(|&i| {
            let (x, y) = ((i as u32 % W) as i32, (i as u32 / W) as i32);
            match (lum(plain, x, y), lum(wired, x, y)) {
                (Some(p), Some(w)) => p - w > 6.0,
                _ => false,
            }
        })
        .count()
}

/// **O VAZAMENTO** — tinta que caiu onde NENHUMA aresta de frente passa.
///
/// ⚠️ **É a régua honesta do segundo oráculo, e o orçamento não servia:** *"mais
/// tinta do que o comprimento das arestas"* confunde vazamento com sobreposição
/// (duas arestas no mesmo pixel gastam um pixel e contam dois). Aqui as arestas
/// de frente são RASTERIZADAS numa máscara com um pixel de folga, e o que sobra
/// fora dela só pode ter vindo do outro lado da peça — o emaranhado que o teste
/// de profundidade existe para impedir.
fn leaked_ink(mesh: &Mesh, plain: &[u8], wired: &[u8], cam: &Camera3d) -> usize {
    let mut near = vec![false; (W * H) as usize];
    let mut wire = Vec::new();
    ph2d_mesh_render::wire_indices(mesh, &mut wire);
    let pos = mesh.positions();
    let mut mark = |x: i32, y: i32| {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (x, y) = (x + dx, y + dy);
                if x >= 0 && y >= 0 && x < W as i32 && y < H as i32 {
                    near[(y as u32 * W + x as u32) as usize] = true;
                }
            }
        }
    };
    for e in wire.chunks_exact(2) {
        let (a, b) = (pos[e[0] as usize], pos[e[1] as usize]);
        let (Some(pa), Some(pb)) = (cam.project(a, (W, H)), cam.project(b, (W, H))) else {
            continue;
        };
        let mid = [
            (a[0] + b[0]) * 0.5,
            (a[1] + b[1]) * 0.5,
            (a[2] + b[2]) * 0.5,
        ];
        if !front_facing(mesh, e[0], mid, cam) {
            continue;
        }
        let steps = ((pb.0 - pa.0).abs().max((pb.1 - pa.1).abs()).ceil() as i32).max(1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            mark(
                (pa.0 + (pb.0 - pa.0) * t).round() as i32,
                (pa.1 + (pb.1 - pa.1) * t).round() as i32,
            );
        }
    }
    (0..(W * H) as usize)
        .filter(|&i| {
            let (x, y) = ((i as u32 % W) as i32, (i as u32 / W) as i32);
            !near[i]
                && match (lum(plain, x, y), lum(wired, x, y)) {
                    (Some(p), Some(w)) => p - w > 6.0,
                    _ => false,
                }
        })
        .count()
}

/// O comprimento, em pixels de tela, de todas as arestas voltadas para o olho.
fn front_edge_pixels(mesh: &Mesh, cam: &Camera3d) -> f64 {
    let mut wire = Vec::new();
    ph2d_mesh_render::wire_indices(mesh, &mut wire);
    let pos = mesh.positions();
    let mut sum = 0.0;
    for e in wire.chunks_exact(2) {
        let (a, b) = (pos[e[0] as usize], pos[e[1] as usize]);
        let (Some(pa), Some(pb)) = (cam.project(a, (W, H)), cam.project(b, (W, H))) else {
            continue;
        };
        let mid = [
            (a[0] + b[0]) * 0.5,
            (a[1] + b[1]) * 0.5,
            (a[2] + b[2]) * 0.5,
        ];
        if !front_facing(mesh, e[0], mid, cam) {
            continue;
        }
        sum += f64::from(((pb.0 - pa.0).powi(2) + (pb.1 - pa.1).powi(2)).sqrt());
    }
    sum
}

#[test]
#[ignore = "sonda: precisa de adapter, roda com --ignored --nocapture"]
fn how_much_of_each_edge_reaches_the_screen() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    println!("\n  malha              continuidade   inteiras / arestas   tinta   vazada");
    println!("  ----------------   ------------   ------------------   -----   ------");
    for (name, mesh) in [
        // ⚠️ **Só malhas DENSAS, e a ausência das outras é MEDIDA.** Num cubo,
        // num octaedro ou na tampa de um cilindro vistos de frente as arestas
        // **projetam-se umas sobre as outras** na silhueta: o orçamento conta o
        // mesmo pixel várias vezes e a percentagem deixa de significar o que o
        // nome dela diz (o cubo media `0%` com o wireframe a funcionar, e o
        // cilindro `157%`). Uma escultura é densa e orgânica — é dela que a
        // pergunta trata.
        ("esfera 32x64", shapes::uv_sphere(32, 64, 1.0)),
        ("esfera 64x128", shapes::uv_sphere(64, 128, 1.0)),
        ("toro 48x24", shapes::torus(48, 24, 1.0, 0.35)),
    ] {
        let cam = camera_for(&mesh);
        let plain = render(&device, &queue, &mesh, false);
        let wired = render(&device, &queue, &mesh, true);
        let (c, whole, edges) = continuity(&mesh, &plain, &wired, &cam);
        let ink = total_ink(&plain, &wired);
        let leak = leaked_ink(&mesh, &plain, &wired, &cam);
        let want = front_edge_pixels(&mesh, &cam);
        println!(
            "  {name:<16}   {:>10.1}%   {whole:>6} / {edges}   {:>5.0}%   {:>5.1}%",
            c * 100.0,
            ink as f64 / want * 100.0,
            leak as f64 / ink.max(1) as f64 * 100.0
        );
    }
}

/// **A ARESTA CHEGA À TELA INTEIRA** — o gate do report de 2026-08-12.
///
/// ⚠️ **O oráculo é a TINTA e não a continuidade por-aresta.** Aquela amostra a
/// reta que EU projeto enquanto o device rasteriza a dele, e num segmento de
/// ângulo raso as duas escolhas de pixel divergem: o teto dela é ~65 % **com o
/// teste de profundidade desligado**, então uma barra ali falaria do meu
/// amostrador. A tinta total é global e imune a isso.
///
/// ⚠️ **E o gate tem DUAS metades porque o defeito tem dois lados.** Só a
/// cobertura passaria com o teste de profundidade removido — e aí a malha do
/// outro lado da peça atravessa, que é o emaranhado que o `depth_write` existe
/// para impedir. Medido: sem teste a tinta vai a **144 %** e a vazada a 19 %.
#[test]
#[ignore = "precisa de adapter"]
fn a_wireframe_edge_reaches_the_screen_whole() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    let mesh = shapes::uv_sphere(64, 128, 1.0);
    let cam = camera_for(&mesh);
    let plain = render(&device, &queue, &mesh, false);
    let wired = render(&device, &queue, &mesh, true);
    let ink = total_ink(&plain, &wired) as f64;
    let want = front_edge_pixels(&mesh, &cam);
    let leak = leaked_ink(&mesh, &plain, &wired, &cam) as f64;
    let covered = ink / want * 100.0;
    // O que shipava media 39 %: a barra fica no meio do caminho entre o defeito
    // e os 93 % de hoje, para não pinar ruído de driver.
    assert!(
        covered > 75.0,
        "as arestas chegam CORTADAS: {covered:.0}% da tinta esperada"
    );
    assert!(
        leak / ink * 100.0 < 6.0,
        "a malha do outro lado atravessou: {:.1}% da tinta caiu fora de toda \
         aresta de frente",
        leak / ink * 100.0
    );
}

// ⚠️ **E NÃO HÁ um gate afirmando *"o viés do pipeline não alcança uma linha"*,
// que é o achado mais caro desta investigação — porque eu não consigo fazê-lo
// falhar pelo motivo que ele alegaria.** O `DepthBiasState` das arestas é uma
// `const` privada do `pipeline_build`, então um teste de fora não tem como
// mexer nela; o que sobraria seria um gate que re-afirma a cobertura com outro
// nome — verde pelo motivo errado, exatamente o que este repo varre a cada
// wave. A varredura que estabelece o fato (`constant` de 0 a −4096, tinta
// idêntica ao pixel) está no doc do `WIRE_DEPTH_NUDGE`, e a rota para a
// reproduzir é a sonda acima.
