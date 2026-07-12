//! T1.2/T1.3 — validação GPU end-to-end (headless), clean-room.
//!
//! Renderiza um traço numa textura offscreen, lê os pixels de volta e afirma o
//! comportamento observável: o traço pinta uma banda; o fundo fica vazio; a
//! hardness controla a queda de borda. `#[ignore]` — precisa de adapter (roda com
//! `--ignored`; skip gracioso sem GPU), como os testes do `ph2d-gpu`.

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_render::{CameraRaw, FlipRenderer, pack_drawing};

const W: u32 = 64;
const H: u32 = 64;

/// Device headless, ou `None` se a máquina não tem adapter (CI).
fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-flip test device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

/// Câmera que mapeia mundo `[0,W]×[0,H]` (y para baixo) para o alvo px 1:1.
/// `mat4x4` do WGSL é COLUNA-major: cada `[f32;4]` aqui é uma COLUNA.
fn pixel_camera() -> CameraRaw {
    let sx = 2.0 / W as f32;
    let sy = -2.0 / H as f32;
    let world_to_clip = [
        [sx, 0.0, 0.0, 0.0],   // coluna do x
        [0.0, sy, 0.0, 0.0],   // coluna do y (flip: mundo-y-baixo = linha-baixo)
        [0.0, 0.0, 1.0, 0.0],  // z
        [-1.0, 1.0, 0.0, 1.0], // translação
    ];
    CameraRaw::new(world_to_clip, [W as f32, H as f32], 1.0)
}

/// Rasteriza `drawing` num alvo `Rgba8Unorm` `W×H`, limpo transparente, e devolve
/// os pixels RGBA (sem padding de linha).
fn render(device: &wgpu::Device, queue: &wgpu::Queue, drawing: &FlipDrawing) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-flip test target"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut fr = FlipRenderer::new(device, format);
    fr.upload(device, queue, &pixel_camera(), &pack_drawing(drawing));
    fr.ensure_depth(device, (W, H));

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-flip test pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: fr.depth_view().map(|v| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: v,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        fr.draw(&mut pass);
    }
    queue.submit([encoder.finish()]);
    readback(device, queue, &texture)
}

fn readback(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let unpadded = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-flip readback"),
        size: (padded as u64) * (H as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
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
    queue.submit([encoder.finish()]);

    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();

    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded as usize) * (H as usize));
    for row in 0..H as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

/// Alpha (0..255) do pixel (col x, row y).
fn alpha_at(px: &[u8], x: u32, y: u32) -> u8 {
    px[((y * W + x) * 4 + 3) as usize]
}
fn rgb_at(px: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * W + x) * 4) as usize;
    [px[i], px[i + 1], px[i + 2]]
}

/// Um traço horizontal reto no meio (y=32), de x=10 a x=54, largura `width`,
/// hardness `hard`, cor vermelha opaca.
fn horizontal_stroke(width: f32, hard: f32) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
    s.push_point(Point {
        pos: Vec2::new(10.0, 32.0),
        width,
        opacity: 1.0,
        color: red,
    });
    s.push_point(Point {
        pos: Vec2::new(54.0, 32.0),
        width,
        opacity: 1.0,
        color: red,
    });
    s.hardness = hard;
    d.strokes.push(s);
    d
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn straight_stroke_paints_a_band_and_leaves_background_empty() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Largura 8 (world=px), hardness dura → banda nítida y∈[28,36].
    let px = render(&device, &queue, &horizontal_stroke(8.0, 1.0));

    // Centro do traço: pintado e avermelhado.
    assert!(alpha_at(&px, 32, 32) > 200, "centro do traço opaco");
    let c = rgb_at(&px, 32, 32);
    assert!(
        c[0] > 200 && c[1] < 60 && c[2] < 60,
        "centro vermelho: {c:?}"
    );

    // Fundo bem longe do traço: vazio.
    assert_eq!(alpha_at(&px, 5, 5), 0, "canto vazio");
    assert_eq!(alpha_at(&px, 32, 8), 0, "acima da banda vazio");
    // Antes do início (x<10) e depois do fim (x>54): vazio (cap flat v1).
    assert_eq!(alpha_at(&px, 2, 32), 0, "antes do início vazio");

    // A banda tem ~8px de altura no meio: y=32 opaco, y=20 e y=44 vazios.
    assert!(alpha_at(&px, 32, 32) > 200);
    assert_eq!(alpha_at(&px, 32, 20), 0, "fora da banda (acima)");
    assert_eq!(alpha_at(&px, 32, 44), 0, "fora da banda (abaixo)");
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn newer_stroke_draws_over_older_at_crossing() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Mesmo desenho: traço 0 = vermelho horizontal, traço 1 = azul vertical. No
    // cruzamento (32,32), o mais novo (sid 1, profundidade maior, GREATER ganha)
    // fica por cima → azul.
    let mut d = FlipDrawing::new();
    let mut red = FlipStroke::new();
    red.push_point(Point {
        pos: Vec2::new(6.0, 32.0),
        width: 10.0,
        opacity: 1.0,
        color: Rgba::new(1.0, 0.0, 0.0, 1.0),
    });
    red.push_point(Point {
        pos: Vec2::new(58.0, 32.0),
        width: 10.0,
        opacity: 1.0,
        color: Rgba::new(1.0, 0.0, 0.0, 1.0),
    });
    red.hardness = 1.0;
    let mut blue = FlipStroke::new();
    blue.push_point(Point {
        pos: Vec2::new(32.0, 6.0),
        width: 10.0,
        opacity: 1.0,
        color: Rgba::new(0.0, 0.0, 1.0, 1.0),
    });
    blue.push_point(Point {
        pos: Vec2::new(32.0, 58.0),
        width: 10.0,
        opacity: 1.0,
        color: Rgba::new(0.0, 0.0, 1.0, 1.0),
    });
    blue.hardness = 1.0;
    d.strokes.push(red);
    d.strokes.push(blue);

    let px = render(&device, &queue, &d);
    // No braço só-vermelho: vermelho. No braço só-azul: azul. No cruzamento: azul.
    let only_red = rgb_at(&px, 12, 32);
    let cross = rgb_at(&px, 32, 32);
    assert!(
        only_red[0] > 200 && only_red[2] < 60,
        "braço vermelho: {only_red:?}"
    );
    assert!(
        cross[2] > 200 && cross[0] < 60,
        "cruzamento é azul (mais novo por cima): {cross:?}"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_stroke_crossing_itself_is_a_clean_union_without_accumulation() {
    let Some((device, queue)) = device() else {
        return;
    };
    // UM traço (mesmo sid → mesmo depth) em X com opacity 0.5. No default do GP o
    // traço NÃO compõe sobre si mesmo ("the stroke cannot overlap itself",
    // `gpencil_vert.glsl`: depth por-STROKE + GREATER estrito): o cruzamento pinta
    // UMA vez — alpha do cruzamento == alpha de um braço — e a parte desenhada
    // PRIMEIRO fica por cima (com cor sólida a sobreposição é união invisível).
    // O modo que deixa o traço acumular sobre si (`GP_STROKE_OVERLAP`, depth
    // por-ponto) é opção de MATERIAL no GP, não o default — não portado.
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
    let blue = Rgba::new(0.0, 0.0, 1.0, 1.0);
    // p0→p1 = diagonal ↘ vermelha (passa por (32,32)); p1→p2 = desvio pela direita
    // (duas quinas AFIADAS de 135° — miter_break); p2→p3 = diagonal ↙ azul (passa
    // por (32,32), desenhada DEPOIS).
    for (p, c) in [
        (Vec2::new(16.0, 16.0), red),
        (Vec2::new(48.0, 48.0), red),
        (Vec2::new(48.0, 16.0), blue),
        (Vec2::new(16.0, 48.0), blue),
    ] {
        s.push_point(Point {
            pos: p,
            width: 8.0,
            opacity: 0.5,
            color: c,
        });
    }
    s.hardness = 1.0;
    d.strokes.push(s);

    let px = render(&device, &queue, &d);
    // Braço sem cruzamento (na linha-de-centro vermelha): 0.5 → ~128.
    let arm = i32::from(alpha_at(&px, 20, 20));
    let cross = i32::from(alpha_at(&px, 32, 32));
    assert!((arm - 128).abs() <= 6, "braço a opacity 0.5: {arm}");
    assert!(
        (cross - arm).abs() <= 6,
        "cruzamento pinta UMA vez (união, sem acúmulo premult): cross={cross} arm={arm}"
    );
    // A parte desenhada PRIMEIRO (vermelha) fica por cima no cruzamento.
    let rgb = rgb_at(&px, 32, 32);
    assert!(
        rgb[0] > rgb[2],
        "GP default: a 1ª parte do traço fica por cima no auto-cruzamento: {rgb:?}"
    );
    // A quina QUEBRADA (miter_break em (48,48), virada de 135°) está coberta (a
    // extensão da fita cobre o disco da junção) e também não acumula.
    let corner = i32::from(alpha_at(&px, 48, 48));
    assert!(
        (corner - arm).abs() <= 8,
        "quina quebrada coberta e sem acúmulo: corner={corner} arm={arm}"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_sharp_corner_is_a_round_join_without_an_outward_spike() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Um "L" de 90° (um traço, 3 pontos): horizontal (10,32)→(32,32) e vertical
    // (32,32)→(32,54). A junção deve ser REDONDA (disco de raio r=4 em torno de
    // (32,32)) — sem o SPIKE de miter que a cobertura por-quad antiga cuspia na
    // quina externa (o artefato do Enio 2026-07-11 com hardness baixo nas curvas).
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    let blue = Rgba::new(0.2, 0.3, 1.0, 1.0);
    for p in [
        Vec2::new(10.0, 32.0),
        Vec2::new(32.0, 32.0),
        Vec2::new(32.0, 54.0),
    ] {
        s.push_point(Point {
            pos: p,
            width: 8.0,
            opacity: 1.0,
            color: blue,
        });
    }
    // A junção é GEOMÉTRICA (independe de hardness); hardness baixo só tornava o
    // artefato mais visível. Testa a geometria com borda dura (o perfil macio é
    // coberto por `hardness_controls_edge_falloff`).
    s.hardness = 1.0;
    d.strokes.push(s);

    let px = render(&device, &queue, &d);
    // Dentro do disco da junção (dist ~2.8 < r=4): coberto (junção redonda).
    assert!(
        alpha_at(&px, 34, 34) > 120,
        "a junção redonda cobre a quina: {}",
        alpha_at(&px, 34, 34)
    );
    // Fora do disco, na quina EXTERNA (dist ~7 > 4): vazio — um miter cuspiria spike.
    assert_eq!(
        alpha_at(&px, 37, 27),
        0,
        "sem spike de miter na quina externa"
    );
    // Bem longe: vazio.
    assert_eq!(alpha_at(&px, 50, 10), 0, "canto oposto vazio");
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_soft_stroke_has_no_bead_at_the_joints() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Um traço reto SOFT com várias amostras (junções em x=25 e x=40). A fita
    // CONECTADA (miter compartilhado) não sobrepõe os segmentos nas junções, então a
    // opacidade FORA do eixo é UNIFORME ao longo do traço — sem o "bead"/mastigado que
    // o double-blend de quads sobrepostos criava (Enio 2026-07-11). Com o bug, o pixel
    // na junção era bem mais opaco que o do meio do segmento.
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    let c = Rgba::new(0.9, 0.9, 0.1, 1.0);
    for x in [10.0_f32, 25.0, 40.0, 55.0] {
        s.push_point(Point {
            pos: Vec2::new(x, 32.0),
            width: 12.0,
            opacity: 1.0,
            color: c,
        });
    }
    s.hardness = 0.7; // macio o bastante p/ ter alpha fora do eixo (o bead vivia aqui)
    d.strokes.push(s);

    let px = render(&device, &queue, &d);
    // Fora do eixo (y=34, ~2px do centro numa banda de raio 6): junções (x=25, x=40)
    // vs meios (x=17, x=32, x=48). Numa reta a distância à linha-de-centro é constante,
    // então SEM bead o alpha é uniforme; COM bead as junções ficam mais opacas.
    let joint_a = i32::from(alpha_at(&px, 25, 34));
    let joint_b = i32::from(alpha_at(&px, 40, 34));
    let mid_a = i32::from(alpha_at(&px, 17, 34));
    let mid_b = i32::from(alpha_at(&px, 32, 34));
    let mid_c = i32::from(alpha_at(&px, 48, 34));
    let vals = [joint_a, joint_b, mid_a, mid_b, mid_c];
    let hi = *vals.iter().max().unwrap();
    let lo = *vals.iter().min().unwrap();
    assert!(lo > 20, "a fita pinta fora do eixo (não some): {vals:?}");
    assert!(
        hi - lo <= 24,
        "opacidade UNIFORME (sem bead nas junções): {vals:?}"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn filled_closed_stroke_renders_fill_under_stroke() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Quadrado fechado: traço vermelho, fill azul. Centro = azul (fill dentro);
    // borda = vermelho (traço por cima do próprio fill).
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
    for corner in [
        Vec2::new(16.0, 16.0),
        Vec2::new(48.0, 16.0),
        Vec2::new(48.0, 48.0),
        Vec2::new(16.0, 48.0),
    ] {
        s.push_point(Point {
            pos: corner,
            width: 6.0,
            opacity: 1.0,
            color: red,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color: Rgba::new(0.0, 0.0, 1.0, 1.0),
        opacity: 1.0,
    });
    d.strokes.push(s);

    let px = render(&device, &queue, &d);
    // Centro bem dentro: azul (fill).
    let center = rgb_at(&px, 32, 32);
    assert!(
        center[2] > 200 && center[0] < 60,
        "centro é o fill azul: {center:?}"
    );
    // Na borda esquerda (x=16): vermelho (traço por cima do fill).
    let border = rgb_at(&px, 16, 32);
    assert!(
        border[0] > 180 && border[2] < 90,
        "borda é o traço vermelho: {border:?}"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn hardness_controls_edge_falloff() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Traço largo (20px) pra ter várias linhas de queda. Compara hardness alta
    // (borda dura) vs baixa (airbrush): a mesma linha perto da BORDA tem alpha
    // MAIOR na dura que na macia.
    let hard = render(&device, &queue, &horizontal_stroke(20.0, 1.0));
    let soft = render(&device, &queue, &horizontal_stroke(20.0, 0.05));

    // A DURA é sólida em toda a banda (topo chapado): o centro é opaco.
    assert!(alpha_at(&hard, 32, 32) > 200, "hard center opaco");
    // A MACIA (airbrush) tem tinta mas já cai fora do eixo — só afirmamos que
    // pinta ALGO no centro (o falloff é o ponto, não a opacidade máxima).
    assert!(alpha_at(&soft, 32, 32) > 60, "soft center pinta algo");

    // Perto da borda (y=39, a ~0.75 do eixo numa banda de meia-altura 10): a dura
    // mantém alpha alto (topo chapado); a macia praticamente sumiu (pow alto).
    let a_hard = alpha_at(&hard, 32, 39);
    let a_soft = alpha_at(&soft, 32, 39);
    assert!(
        a_hard > a_soft + 60,
        "borda dura ({a_hard}) deve superar em muito a macia ({a_soft}) perto da borda"
    );
    // E dentro da mesma banda, a macia é MAIS opaca no eixo que na borda (queda).
    assert!(
        alpha_at(&soft, 32, 32) > alpha_at(&soft, 32, 39) + 20,
        "airbrush: eixo mais opaco que a borda"
    );
}

// ---------- Oráculo analítico (paridade CPU↔GPU, pixel-a-pixel) ----------
//
// O modelo espelha o raster do GP: o pixel é ganho pelo PRIMEIRO segmento (ordem
// de desenho) cujo fragmento sobrevive ao discard (`alpha >= 0.001`) — depth
// GREATER estrito + write-depth = a primeira face fica, as demais são descartadas,
// nunca misturadas. O valor do pixel é a máscara de hardness DELE (distância
// clampada à linha-de-centro do segmento). Pixels com alguma máscara na zona
// ambígua do limiar de discard são pulados; TODO o resto do alvo é comparado,
// inclusive o fundo (0). Qualquer classe de artefato — bead (blend dobrado),
// escama (depth furado por fragmento transparente), spike (fita dobrada), buraco
// (junção descoberta) — diverge do oráculo.

/// `smoothstep(0, 1, x)` do WGSL.
fn smoothstep01(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A `hardness_mask` do `flip.wgsl` na CPU, sem o termo de AA (os pixels na faixa
/// de AA caem na zona ambígua e são pulados). Os hardness usados aqui dão expoente
/// INTEIRO (`10·(1-h)`: 0.8 → 2, 0.7 → 3) — `powi`, nada transcendental.
fn cpu_mask(dn: f32, hardness: f32) -> f32 {
    let inv = (1.0 - dn).clamp(0.0, 1.0);
    let exp = 10.0 * (1.0 - hardness);
    assert!(
        (exp - exp.round()).abs() < 1e-4,
        "use hardness com expoente inteiro no oráculo"
    );
    smoothstep01(inv.powi(exp.round() as i32))
}

/// Distância NORMALIZADA (0 = centro, 1 = borda) do ponto `p` ao segmento `a`→`b`,
/// clampada ao segmento (a mesma conta do fragment).
fn seg_dn(p: (f32, f32), a: (f32, f32), b: (f32, f32), radius: f32) -> f32 {
    let ab = (b.0 - a.0, b.1 - a.1);
    let ap = (p.0 - a.0, p.1 - a.1);
    let len_sq = (ab.0 * ab.0 + ab.1 * ab.1).max(1e-6);
    let t = ((ap.0 * ab.0 + ap.1 * ab.1) / len_sq).clamp(0.0, 1.0);
    let d = (ap.0 - t * ab.0, ap.1 - t * ab.1);
    (d.0 * d.0 + d.1 * d.1).sqrt() / radius.max(1e-4)
}

/// O quad de UM segmento, com os mesmos 4 cantos que o `vs_main` emite:
/// `[a_esq, a_dir, b_esq, b_dir]` (miter compartilhado nas junções mitradas;
/// perpendicular própria + extensão `r` nas quebradas/tampas).
struct SegQuad {
    corners: [(f32, f32); 4],
}

fn norm(v: (f32, f32)) -> (f32, f32) {
    let l = (v.0 * v.0 + v.1 * v.1).sqrt().max(1e-6);
    (v.0 / l, v.1 / l)
}

/// Replica a geometria do `vs_main` (fita conectada + miter_break do GP) na CPU,
/// para um traço ABERTO de largura uniforme com tampas redondas.
fn seg_quads(pts: &[(f32, f32)], width: f32) -> Vec<SegQuad> {
    const MITER_BREAK_COS: f32 = 0.5;
    let r = width * 0.5;
    let mut quads = Vec::new();
    for i in 0..pts.len() - 1 {
        let a = pts[i];
        let b = pts[i + 1];
        let dir = norm((b.0 - a.0, b.1 - a.1));
        let n_seg = (-dir.1, dir.0);

        let mut miter_a = n_seg;
        let mut scale_a = 1.0;
        let mut ext_a = if i == 0 { r } else { 0.0 };
        if i > 0 {
            let p_prev = pts[i - 1];
            let d_prev = norm((a.0 - p_prev.0, a.1 - p_prev.1));
            if -(dir.0 * d_prev.0 + dir.1 * d_prev.1) > MITER_BREAK_COS {
                ext_a = r;
            } else {
                let m_tan = norm((d_prev.0 + dir.0, d_prev.1 + dir.1));
                miter_a = (-m_tan.1, m_tan.0);
                scale_a = 1.0 / (m_tan.0 * d_prev.0 + m_tan.1 * d_prev.1).max(MITER_BREAK_COS);
            }
        }
        let mut miter_b = n_seg;
        let mut scale_b = 1.0;
        let mut ext_b = if i + 2 == pts.len() { r } else { 0.0 };
        if i + 2 < pts.len() {
            let p_next = pts[i + 2];
            let d_next = norm((p_next.0 - b.0, p_next.1 - b.1));
            if -(dir.0 * d_next.0 + dir.1 * d_next.1) > MITER_BREAK_COS {
                ext_b = r;
            } else {
                let m_tan = norm((dir.0 + d_next.0, dir.1 + d_next.1));
                miter_b = (-m_tan.1, m_tan.0);
                scale_b = 1.0 / (m_tan.0 * d_next.0 + m_tan.1 * d_next.1).max(MITER_BREAK_COS);
            }
        }

        let sa = (a.0 - dir.0 * ext_a, a.1 - dir.1 * ext_a);
        let sb = (b.0 + dir.0 * ext_b, b.1 + dir.1 * ext_b);
        let ra = r * scale_a;
        let rb = r * scale_b;
        quads.push(SegQuad {
            corners: [
                (sa.0 + miter_a.0 * ra, sa.1 + miter_a.1 * ra),
                (sa.0 - miter_a.0 * ra, sa.1 - miter_a.1 * ra),
                (sb.0 + miter_b.0 * rb, sb.1 + miter_b.1 * rb),
                (sb.0 - miter_b.0 * rb, sb.1 - miter_b.1 * rb),
            ],
        });
    }
    quads
}

enum Containment {
    In,
    Out,
    Borderline,
}

/// Ponto-no-triângulo com margem: `In`/`Out` definitivos exigem folga `eps` (px)
/// de TODA aresta — na faixa da aresta o raster e a CPU podem discordar por ulp.
fn tri_contains(p: (f32, f32), t: [(f32, f32); 3], eps: f32) -> Containment {
    let area = (t[1].0 - t[0].0) * (t[2].1 - t[0].1) - (t[1].1 - t[0].1) * (t[2].0 - t[0].0);
    if area.abs() < 1e-6 {
        return Containment::Out; // degenerado não emite fragmento útil
    }
    let s = area.signum();
    let mut min_inset = f32::MAX;
    for k in 0..3 {
        let v0 = t[k];
        let v1 = t[(k + 1) % 3];
        let e = (v1.0 - v0.0, v1.1 - v0.1);
        let len = (e.0 * e.0 + e.1 * e.1).sqrt().max(1e-6);
        let cross = e.0 * (p.1 - v0.1) - e.1 * (p.0 - v0.0);
        min_inset = min_inset.min(s * cross / len);
    }
    if min_inset > eps {
        Containment::In
    } else if min_inset < -eps {
        Containment::Out
    } else {
        Containment::Borderline
    }
}

/// O quad rasteriza como os 2 triângulos do shader: `[0,1,2]` + `[2,1,3]`.
fn quad_contains(q: &SegQuad, p: (f32, f32)) -> Containment {
    let c = &q.corners;
    let t1 = tri_contains(p, [c[0], c[1], c[2]], 0.08);
    let t2 = tri_contains(p, [c[2], c[1], c[3]], 0.08);
    match (t1, t2) {
        (Containment::In, _) | (_, Containment::In) => Containment::In,
        (Containment::Out, Containment::Out) => Containment::Out,
        _ => Containment::Borderline,
    }
}

/// Alpha esperado no pixel, ou `None` se o pixel é ambíguo: perto de uma aresta
/// de quad (o raster decide por ulp) ou com máscara na faixa do discard de 0.001
/// (CPU e GPU podem cair em lados diferentes do limiar).
fn expected_alpha(
    pts: &[(f32, f32)],
    quads: &[SegQuad],
    width: f32,
    hardness: f32,
    p: (f32, f32),
) -> Option<f32> {
    let r = width * 0.5;
    for (i, w) in pts.windows(2).enumerate() {
        let m = cpu_mask(seg_dn(p, w[0], w[1], r), hardness);
        match quad_contains(&quads[i], p) {
            Containment::Out => continue,
            cont => {
                if m > 0.0002 && m < 0.005 {
                    return None; // limiar do discard — ambíguo
                }
                if m < 0.001 {
                    continue; // descartado com folga: não escreve depth
                }
                match cont {
                    Containment::In => return Some(m),
                    _ => return None, // aresta de quad — ambíguo
                }
            }
        }
    }
    Some(0.0)
}

/// Renderiza a polilinha (um traço, opacity 1, cor sólida) e compara TODOS os
/// pixels comparáveis do alvo com o oráculo analítico.
fn assert_matches_analytic(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pts: &[(f32, f32)],
    width: f32,
    hardness: f32,
    label: &str,
) {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width,
            opacity: 1.0,
            color: Rgba::new(1.0, 0.2, 0.1, 1.0),
        });
    }
    s.hardness = hardness;
    d.strokes.push(s);
    let px = render(device, queue, &d);
    let quads = seg_quads(pts, width);

    let mut checked = 0u32;
    let mut worst = 0i32;
    let mut worst_at = (0u32, 0u32);
    for y in 0..H {
        for x in 0..W {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let Some(exp) = expected_alpha(pts, &quads, width, hardness, p) else {
                continue;
            };
            let got = i32::from(alpha_at(&px, x, y));
            let want = (exp * 255.0).round() as i32;
            let diff = (got - want).abs();
            checked += 1;
            if diff > worst {
                worst = diff;
                worst_at = (x, y);
            }
        }
    }
    assert!(
        checked > 500,
        "{label}: o oráculo cobriu pixels de menos ({checked})"
    );
    assert!(
        worst <= 8,
        "{label}: GPU diverge do oráculo analítico — pior desvio {worst} em {worst_at:?}"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_sharp_corner_does_not_accumulate_color() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Zigzag com DUAS viradas afiadas (~158°/~171° > 120° → miter_break) e
    // hardness baixo: a geometria estendida cobre o disco de cada junção DUAS
    // vezes, e o GREATER estrito + discard pintam UMA — todo pixel bate com o
    // oráculo single-coverage. Era o spike/estrela da 6ª rodada: a fita mitrada
    // DOBRAVA na quina afiada (bowtie) e o premult-over acumulava na bissetriz.
    // O último trecho ainda CRUZA DE VOLTA o canto estendido (mask=0) do 1º
    // segmento — sem o discard, aquele fragmento transparente escreve depth e
    // FURA o traço que passa depois (a classe "escamado").
    assert_matches_analytic(
        &device,
        &queue,
        &[(10.0, 32.0), (44.0, 32.0), (14.0, 44.0), (56.0, 35.0)],
        10.0,
        0.8,
        "hairpin",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_smooth_curve_matches_the_analytic_coverage() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Arco suave (viradas ~15° → fita mitrada, junções compartilhadas) com
    // hardness baixo e largura GORDA (o miter interno até dobra — segmentos mais
    // curtos que o inset — e ainda assim não pode acumular nem escamar): alpha
    // idêntico ao oráculo em todo o alvo — sem escama (corrente de ovais), sem
    // bead nas junções, sem costura entre segmentos.
    assert_matches_analytic(
        &device,
        &queue,
        &[
            (8.0, 40.0),
            (20.0, 34.0),
            (32.0, 31.0),
            (44.0, 31.0),
            (56.0, 34.0),
        ],
        16.0,
        0.7,
        "arco",
    );
}
