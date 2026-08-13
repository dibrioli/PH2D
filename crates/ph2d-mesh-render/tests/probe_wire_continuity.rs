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
fn continuity(
    mesh: &Mesh,
    plain: &[u8],
    wired: &[u8],
    cam: &Camera3d,
    strict: bool,
) -> (f64, usize, usize) {
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
        // ⚠️ **O modo ESTRITO é o único que responde *"o descarte cortou uma
        // aresta da FRENTE?"*.** Uma aresta que CRUZA a silhueta tem uma ponta de
        // cada lado, então ela DEVE perder metade — contá-la mede a lei, não o
        // defeito.
        if strict
            && (facing_at(mesh, e[0], cam) <= STRICT_MARGIN
                || facing_at(mesh, e[1], cam) <= STRICT_MARGIN)
        {
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

/// `n · (olho − p)`, normalizado — o cosseno do ângulo entre a normal do vértice
/// e o raio que vai dele até o olho.
fn facing_at(mesh: &Mesh, v: u32, cam: &Camera3d) -> f32 {
    let n = mesh.normals()[v as usize];
    let p = mesh.positions()[v as usize];
    let eye = cam.eye();
    let d = [eye[0] - p[0], eye[1] - p[1], eye[2] - p[2]];
    let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9);
    let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
    (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / (dl * nl)
}

/// **AS ARESTAS INEQUÍVOCAS** — as que têm as DUAS pontas voltadas para o olho
/// com folga (`facing > MARGIN`), rasterizadas numa máscara, e o comprimento
/// delas.
///
/// ⚠️ **Elas existem porque a silhueta torna a pergunta mal-posta.** Uma aresta
/// que CRUZA a silhueta tem metade visível e metade do outro lado, então
/// *"quanto dela deveria chegar?"* não tem resposta sem re-implementar a lei que
/// o produto usa — e um oráculo que copia a regra sob teste é um espelho, não um
/// oráculo. Com a folga, o conjunto é o miolo da peça: ali *toda* a aresta
/// deveria chegar, sob qualquer lei, e a percentagem volta a significar o que o
/// nome dela diz.
const STRICT_MARGIN: f32 = 0.2;

fn strict_front(mesh: &Mesh, cam: &Camera3d, mask: &mut [bool]) -> f64 {
    let mut wire = Vec::new();
    ph2d_mesh_render::wire_indices(mesh, &mut wire);
    let pos = mesh.positions();
    let mut sum = 0.0;
    for e in wire.chunks_exact(2) {
        if facing_at(mesh, e[0], cam) <= STRICT_MARGIN
            || facing_at(mesh, e[1], cam) <= STRICT_MARGIN
        {
            continue;
        }
        let (a, b) = (pos[e[0] as usize], pos[e[1] as usize]);
        let (Some(pa), Some(pb)) = (cam.project(a, (W, H)), cam.project(b, (W, H))) else {
            continue;
        };
        sum += f64::from(((pb.0 - pa.0).powi(2) + (pb.1 - pa.1).powi(2)).sqrt());
        let steps = ((pb.0 - pa.0).abs().max((pb.1 - pa.1).abs()).ceil() as i32).max(1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let (x, y) = (
                (pa.0 + (pb.0 - pa.0) * t).round() as i32,
                (pa.1 + (pb.1 - pa.1) * t).round() as i32,
            );
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (x, y) = (x + dx, y + dy);
                    if x >= 0 && y >= 0 && x < W as i32 && y < H as i32 {
                        mask[(y as u32 * W + x as u32) as usize] = true;
                    }
                }
            }
        }
    }
    sum
}

/// **AS ARESTAS DO MIOLO, SEPARADAS POR QUÃO DE FRENTE ELAS ESTÃO.**
///
/// A `strict_front` responde *"quanto do miolo chegou?"* com um número só, e é
/// esse número que satura em 86 %. Esta responde **ONDE os 14 % que faltam
/// moram** — e a pergunta não é curiosidade: o empurrão LATERAL do Blender pesa
/// `1 − facing²`, que é **zero** numa face de frente para o olho. Se a tinta que
/// falta estiver no bin de `facing` alto, aquele empurrão não a alcança, e
/// trocar o descarte por ele seria trocar uma cura por uma que não morde ali.
fn strict_front_binned(mesh: &Mesh, cam: &Camera3d, bins: &mut [(Vec<bool>, f64)]) {
    let mut wire = Vec::new();
    ph2d_mesh_render::wire_indices(mesh, &mut wire);
    let pos = mesh.positions();
    let n = bins.len();
    for e in wire.chunks_exact(2) {
        let (fa, fb) = (facing_at(mesh, e[0], cam), facing_at(mesh, e[1], cam));
        let f = fa.min(fb);
        if f <= STRICT_MARGIN {
            continue;
        }
        // `STRICT_MARGIN..=1` fatiado em `n`.
        let t = (f - STRICT_MARGIN) / (1.0 - STRICT_MARGIN);
        let b = ((t * n as f32) as usize).min(n - 1);
        let (a, c) = (pos[e[0] as usize], pos[e[1] as usize]);
        let (Some(pa), Some(pb)) = (cam.project(a, (W, H)), cam.project(c, (W, H))) else {
            continue;
        };
        bins[b].1 += f64::from(((pb.0 - pa.0).powi(2) + (pb.1 - pa.1).powi(2)).sqrt());
        let steps = ((pb.0 - pa.0).abs().max((pb.1 - pa.1).abs()).ceil() as i32).max(1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let (x, y) = (
                (pa.0 + (pb.0 - pa.0) * t).round() as i32,
                (pa.1 + (pb.1 - pa.1) * t).round() as i32,
            );
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (x, y) = (x + dx, y + dy);
                    if x >= 0 && y >= 0 && x < W as i32 && y < H as i32 {
                        bins[b].0[(y as u32 * W + x as u32) as usize] = true;
                    }
                }
            }
        }
    }
}

/// Tinta de wireframe DENTRO de uma máscara.
fn ink_in(plain: &[u8], wired: &[u8], mask: &[bool]) -> usize {
    (0..(W * H) as usize)
        .filter(|&i| {
            let (x, y) = ((i as u32 % W) as i32, (i as u32 / W) as i32);
            mask[i]
                && match (lum(plain, x, y), lum(wired, x, y)) {
                    (Some(p), Some(w)) => p - w > 6.0,
                    _ => false,
                }
        })
        .count()
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
    front_edge_mask(mesh, cam, &mut near);
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

/// Rasteriza as arestas de FRENTE numa máscara com um pixel de folga.
///
/// ⚠️ **UMA porta, dois consumidores** (o total vazado e a tabela por anel): duas
/// cópias desta rasterização divergiriam na folga, e aí os dois números falariam
/// de conjuntos diferentes com o mesmo nome.
fn front_edge_mask(mesh: &Mesh, cam: &Camera3d, near: &mut [bool]) {
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

/// **ONDE a tinta cai, por ANEL** — a sonda do 2º report (*"ainda ruim, veja as
/// bordas"*, Enio, 2026-08-12, com foto de uma esfera cuja borda sai numa faixa
/// escura).
///
/// ⚠️ **Ela existe porque um número GLOBAL não distingue as duas leituras da
/// foto.** *Vazamento de 2 % do quadro* e *densidade honesta da silhueta* são
/// compatíveis com a mesma percentagem total, e pedem curas OPOSTAS: a primeira
/// é um defeito de profundidade, a segunda é a projeção de uma esfera UV — os
/// anéis de latitude comprimem-se na borda e escurecem sozinhos, sem nada de
/// errado. O que separa as duas é ONDE a tinta está, e se ela é vazada.
///
/// A régua é `u = raio / raio_da_silhueta`, com a silhueta MEDIDA do quadro sem
/// wireframe (o pixel mais distante do centro que não é fundo) em vez de
/// derivada da câmera — é a silhueta que o device desenhou.
fn ink_by_annulus(mesh: &Mesh, plain: &[u8], wired: &[u8], cam: &Camera3d) -> [(usize, usize); 10] {
    let c = cam
        .project([0.0, 0.0, 0.0], (W, H))
        .unwrap_or((W as f32 * 0.5, H as f32 * 0.5));
    let mut r_sil = 1.0f32;
    for i in 0..(W * H) as usize {
        let (x, y) = ((i as u32 % W) as i32, (i as u32 / W) as i32);
        // O fundo é preto sólido: qualquer luminância é peça.
        if lum(plain, x, y).is_some_and(|l| l > 1.0) {
            let d = ((x as f32 - c.0).powi(2) + (y as f32 - c.1).powi(2)).sqrt();
            r_sil = r_sil.max(d);
        }
    }
    let mut near = vec![false; (W * H) as usize];
    front_edge_mask(mesh, cam, &mut near);
    let mut bands = [(0usize, 0usize); 10];
    for (i, near) in near.iter().enumerate() {
        let (x, y) = ((i as u32 % W) as i32, (i as u32 / W) as i32);
        let dark = match (lum(plain, x, y), lum(wired, x, y)) {
            (Some(p), Some(w)) => p - w > 6.0,
            _ => false,
        };
        if !dark {
            continue;
        }
        let d = ((x as f32 - c.0).powi(2) + (y as f32 - c.1).powi(2)).sqrt() / r_sil;
        let b = ((d * 10.0).floor() as usize).min(9);
        bands[b].0 += 1;
        if !near {
            bands[b].1 += 1;
        }
    }
    bands
}

#[test]
#[ignore = "sonda: precisa de adapter, roda com --ignored --nocapture"]
fn where_the_wire_ink_falls() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    for (name, mesh) in [
        ("esfera 32x64", shapes::uv_sphere(32, 64, 1.0)),
        ("esfera 64x128", shapes::uv_sphere(64, 128, 1.0)),
        ("toro 48x24", shapes::torus(48, 24, 1.0, 0.35)),
    ] {
        let cam = camera_for(&mesh);
        let plain = render(&device, &queue, &mesh, false);
        let wired = render(&device, &queue, &mesh, true);
        let bands = ink_by_annulus(&mesh, &plain, &wired, &cam);
        let total: usize = bands.iter().map(|b| b.0).sum();
        println!("\n  {name}   (tinta total {total})");
        println!("    u          tinta    % do quadro   vazada");
        for (i, (ink, leak)) in bands.iter().enumerate() {
            println!(
                "    {:.1}-{:.1}   {ink:>7}   {:>9.1}%   {:>5.1}%",
                i as f32 / 10.0,
                (i + 1) as f32 / 10.0,
                *ink as f64 / total.max(1) as f64 * 100.0,
                *leak as f64 / (*ink).max(1) as f64 * 100.0
            );
        }
    }
}

#[test]
#[ignore = "sonda: precisa de adapter, roda com --ignored --nocapture"]
fn how_much_of_each_edge_reaches_the_screen() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    println!(
        "\n  malha              continuidade   inteiras / arestas   tinta   vazada   miolo    teto"
    );
    println!(
        "  ----------------   ------------   ------------------   -----   ------   -----   -----"
    );
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
        let (c, whole, edges) = continuity(&mesh, &plain, &wired, &cam, false);
        let (cs, _, es) = continuity(&mesh, &plain, &wired, &cam, true);
        let ink = total_ink(&plain, &wired);
        let leak = leaked_ink(&mesh, &plain, &wired, &cam);
        let want = front_edge_pixels(&mesh, &cam);
        let mut smask = vec![false; (W * H) as usize];
        let swant = strict_front(&mesh, &cam, &mut smask);
        let sink = ink_in(&plain, &wired, &smask);
        // ⚠️ **O CONTROLE do miolo:** a máscara é dilatada e arestas vizinhas
        // partilham pixels, então o comprimento SOMADO conta duas vezes o que a
        // tela gasta uma. Este número é o teto que o oráculo consegue reportar
        // com a malha 100 % desenhada — sem ele, um deficit de sobreposição lê-se
        // como aresta cortada.
        let smask_px = smask.iter().filter(|b| **b).count();
        println!(
            "  {name:<16}   {:>10.1}%   {whole:>6} / {edges}   {:>5.0}%   {:>5.1}%   {:>5.0}%   {:>5.0}%",
            c * 100.0,
            ink as f64 / want * 100.0,
            leak as f64 / ink.max(1) as f64 * 100.0,
            sink as f64 / swant * 100.0,
            smask_px as f64 / swant * 100.0
        );
        println!("      miolo estrito: {:.1}% sobre {es} arestas", cs * 100.0);
    }
}

/// **A ARESTA CHEGA À TELA INTEIRA** — o gate do report de 2026-08-12.
///
/// ⚠️ **O oráculo é o MIOLO ESTRITO, e a régua anterior estava INFLADA.** Ela
/// media a tinta TOTAL contra o comprimento das arestas de frente, e numa esfera
/// densa o fio do outro lado da peça projeta-se **por cima** do da frente: parte
/// do que ela contava como *"a aresta chegou"* era a malha de trás tapando o
/// buraco de uma aresta cortada. Medido, ao instalar a remoção de linha
/// escondida a mesma cena caiu de `109 %` para `86 %` — e a queda é a ilusão a
/// sair, não cobertura a perder.
///
/// O miolo estrito não tem essa ambiguidade: são as arestas cujas DUAS pontas
/// encaram o olho com folga, num sólido CONVEXO (onde encarar o olho implica ser
/// visível). Ali toda a aresta deve chegar, sob qualquer lei.
///
/// | | miolo | vazada |
/// |---|---|---|
/// | sem a nudge (o defeito do 1º report) | **45 %** | 0,0 % |
/// | hoje | **86 %** | **0,0 %** |
///
/// ⚠️ **E as DUAS metades continuam necessárias:** só a cobertura passaria com o
/// teste de profundidade removido, e aí a malha do outro lado atravessa.
#[test]
#[ignore = "sonda: precisa de adapter, roda com --ignored --nocapture"]
fn where_the_interior_miss_lives() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    const N: usize = 4;
    for (name, mesh) in [
        ("esfera 32x64", shapes::uv_sphere(32, 64, 1.0)),
        ("esfera 64x128", shapes::uv_sphere(64, 128, 1.0)),
    ] {
        let cam = camera_for(&mesh);
        let plain = render(&device, &queue, &mesh, false);
        let wired = render(&device, &queue, &mesh, true);
        let mut bins: Vec<(Vec<bool>, f64)> = (0..N)
            .map(|_| (vec![false; (W * H) as usize], 0.0))
            .collect();
        strict_front_binned(&mesh, &cam, &mut bins);
        println!("\n  {name}");
        println!("    facing        quer      chegou   cobertura   peso do empurrao lateral");
        for (i, (mask, want)) in bins.iter().enumerate() {
            let lo = STRICT_MARGIN + (1.0 - STRICT_MARGIN) * i as f32 / N as f32;
            let hi = STRICT_MARGIN + (1.0 - STRICT_MARGIN) * (i + 1) as f32 / N as f32;
            let mid = (lo + hi) / 2.0;
            let got = ink_in(&plain, &wired, mask);
            println!(
                "    {lo:.2}-{hi:.2}   {want:>8.0}   {got:>9}   {:>8.1}%   {:>10.2}",
                got as f64 / want.max(1.0) * 100.0,
                1.0 - mid * mid,
            );
        }
    }
}

/// **A ARESTA RASANTE NÃO É COMIDA PELA PRÓPRIA SUPERFÍCIE.**
///
/// ⚠️ **É o gate do EMPURRÃO LATERAL, e ele existe porque o SINAL do empurrão
/// foi um defeito real desta wave.** Perto da silhueta o triângulo se projeta
/// quase de perfil e cobre a linha que nasce sobre a aresta dele; a nudge de
/// profundidade **satura** e não compra nada ali (3e-3 a 4,8e-2 dão o mesmo
/// número), então quem morde é o deslocamento lateral.
///
/// O oráculo é o bin de `facing` mais RASANTE do miolo estrito — o único lugar
/// onde `1 − facing²` vale quase 1, logo o único que o empurrão alcança. Os três
/// mundos que a barra separa, medidos na mesma esfera:
///
/// | empurrão | cobertura do bin rasante |
/// |---|---|
/// | para FORA (o sinal errado) | 72,9 % |
/// | nenhum | 75,1 % |
/// | **para DENTRO, meio pixel** | **79,2 %** |
///
/// ⚠️ A barra fica em **78 %**, entre o certo e os DOIS modos de falha — apagar
/// o empurrão e invertê-lo têm de sangrar, e o irmão de vazamento acima é o que
/// impede a cura de ser *"empurre mais"* (a 0,75 px o vazamento já cruza 0,1 %).
#[test]
#[ignore = "precisa de adapter"]
fn the_grazing_edges_are_not_eaten_by_their_own_surface() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    const N: usize = 4;
    let mesh = shapes::uv_sphere(64, 128, 1.0);
    let cam = camera_for(&mesh);
    let plain = render(&device, &queue, &mesh, false);
    let wired = render(&device, &queue, &mesh, true);
    let mut bins: Vec<(Vec<bool>, f64)> = (0..N)
        .map(|_| (vec![false; (W * H) as usize], 0.0))
        .collect();
    strict_front_binned(&mesh, &cam, &mut bins);
    let (mask, want) = &bins[0];
    let covered = ink_in(&plain, &wired, mask) as f64 / want.max(1.0) * 100.0;
    assert!(
        covered > 78.0,
        "as arestas rasantes chegam comidas: {covered:.1}% (sem empurrao: 75,1%, para fora: 72,9%)"
    );
}

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
    let mut smask = vec![false; (W * H) as usize];
    let swant = strict_front(&mesh, &cam, &mut smask);
    let covered = ink_in(&plain, &wired, &smask) as f64 / swant * 100.0;
    // A barra fica entre o defeito (45 %) e os 86 % de hoje, para não pinar
    // ruído de driver.
    assert!(
        covered > 70.0,
        "as arestas do miolo chegam CORTADAS: {covered:.0}%"
    );
    // ⚠️ **ZERO, e não *"pouco"* — e as DUAS densidades.** Numa peça fechada
    // nenhuma tinta pode cair onde aresta de frente nenhuma passa. A barra é
    // apertada porque o modo de falha mais próximo não é ruído: trocar o
    // `facing` perspectiva-correto pelo atalho ortográfico (`n_view.z`) devolve
    // **0,3 %** na esfera grossa e 0,0 % na fina — o erro dele cresce com o
    // ângulo do raio contra o eixo da câmera, logo com a DENSIDADE aparente na
    // borda do quadro, e uma malha só não o vê.
    for (name, mesh) in [
        ("esfera 32x64", shapes::uv_sphere(32, 64, 1.0)),
        ("esfera 64x128", mesh),
    ] {
        let cam = camera_for(&mesh);
        let plain = render(&device, &queue, &mesh, false);
        let wired = render(&device, &queue, &mesh, true);
        let ink = total_ink(&plain, &wired) as f64;
        let leak = leaked_ink(&mesh, &plain, &wired, &cam) as f64 / ink * 100.0;
        assert!(
            leak < 0.1,
            "{name}: a malha do outro lado atravessou: {leak:.1}% da tinta"
        );
    }
}

/// **UMA CASCA ABERTA NÃO PERDE O WIREFRAME** — o preço da remoção de linha
/// escondida, cobrado.
///
/// ⚠️ **Ela é a única coisa que o descarte por normal poderia quebrar**, e o modo
/// de falha seria mudo: numa folha vista pelo lado de TRÁS toda normal aponta
/// para longe do olho, então uma regra que lesse *"normal de costas ⇒ está
/// escondido"* apagaria exatamente o que o artista está olhando — com a
/// superfície ainda desenhada por baixo.
///
/// O oráculo é a MESMA geometria com as duas orientações: um plano enrolado para
/// o olho e o mesmo plano enrolado ao contrário têm de desenhar a mesma malha.
/// ⚠️ **Ele não menciona `wire_cull`** — afirma a propriedade, e é por isso que
/// ele morre quando alguém arma o descarte incondicionalmente.
#[test]
#[ignore = "precisa de adapter"]
fn an_open_shell_keeps_its_wireframe() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: skip");
        return;
    };
    let (front, back) = (open_grid(false), open_grid(true));
    assert!(!front.is_closed(), "a fixture TEM de ser uma casca");
    let cam = camera_for(&front);
    assert!(
        facing_at(&front, 0, &cam) > 0.0 && facing_at(&back, 0, &cam) < 0.0,
        "a fixture não contém o fenômeno: as duas orientações têm de cair em \
         lados opostos do olho"
    );
    let ink_front = total_ink(
        &render(&device, &queue, &front, false),
        &render(&device, &queue, &front, true),
    );
    let ink_back = total_ink(
        &render(&device, &queue, &back, false),
        &render(&device, &queue, &back, true),
    );
    assert!(ink_front > 500, "a fixture não desenhou nada: {ink_front}");
    let ratio = ink_back as f64 / ink_front as f64;
    assert!(
        ratio > 0.9,
        "a folha vista por trás perdeu o wireframe: {ink_back} contra \
         {ink_front} pixels ({ratio:.2}x)"
    );
}

/// Uma grade PLANA — uma casca aberta, com a orientação escolhida.
fn open_grid(flip: bool) -> Mesh {
    const N: usize = 8;
    let mut pos = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            let (u, v) = (i as f32 / N as f32 - 0.5, j as f32 / N as f32 - 0.5);
            pos.push([u * 2.0, v * 2.0, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * (N + 1) + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..N {
        for i in 0..N {
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1));
            faces.push(if flip {
                ph2d_mesh::Face::quad(a, d, c, b)
            } else {
                ph2d_mesh::Face::quad(a, b, c, d)
            });
        }
    }
    Mesh::from_parts(pos, faces).expect("grade plana")
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
