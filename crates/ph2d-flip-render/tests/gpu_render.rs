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
    render_with(device, queue, drawing, pixel_camera())
}

/// O mesmo, com a câmera dada — o que deixa o teste de Ghost Frames rasterizar o
/// MESMO desenho com e sem o tint de fantasma.
fn render_with(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    drawing: &FlipDrawing,
    camera: CameraRaw,
) -> Vec<u8> {
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
    fr.upload(device, queue, &camera, &pack_drawing(drawing));
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

// ---------- Oráculo de APARÊNCIA: a UNIÃO da polilinha ----------
//
// **O expected NÃO modela o shader — modela o OBJETO.** Um traço macio É a união
// dos discos varridos ao longo da polilinha: a cobertura num pixel é o perfil de
// hardness aplicado à MENOR distância normalizada às cápsulas de TODOS os
// segmentos (o perfil é monótono decrescente ⇒ min-distância ⇔ max-cobertura).
// Nada aqui sabe de quads, depth, ordem de desenho ou discard.
//
// Isto é deliberado e caro de esquecer: a versão anterior deste oráculo espelhava
// a IMPLEMENTAÇÃO (first-wins por depth) e ficou **verde com a mordida na tela**
// (memória `feedback_oracle_must_model_appearance_not_implementation`). Toda
// classe de artefato — mordida na quina, bead, escama, spike, acúmulo, buraco de
// junção — é uma divergência da união, e por isso vermelha aqui.
//
// **Semântica pinada:** auto-cruzamento NÃO-adjacente (laço) segue first-wins (o
// default do GP, "the stroke cannot overlap itself") — logo os traços deste
// oráculo não se auto-cruzam; o laço tem teste próprio
// (`a_soft_self_crossing_keeps_first_wins_semantics`).

/// `smoothstep(0, 1, x)` do WGSL.
fn smoothstep01(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// O perfil de hardness (a queda do pincel), sem o termo de AA da borda — os
/// pixels da faixa de AA são pulados por `expected_union_alpha`. Os hardness
/// usados aqui dão expoente INTEIRO (`10·(1-h)`: 0.8 → 2, 0.7 → 3) — `powi`,
/// nada transcendental.
fn cpu_mask(dn: f32, hardness: f32) -> f32 {
    let inv = (1.0 - dn).clamp(0.0, 1.0);
    let exp = 10.0 * (1.0 - hardness);
    assert!(
        (exp - exp.round()).abs() < 1e-4,
        "use hardness com expoente inteiro no oráculo"
    );
    smoothstep01(inv.powi(exp.round() as i32))
}

/// Distância NORMALIZADA (0 = centro, 1 = borda) do ponto `p` à cápsula `a`→`b`
/// de raios `ra`/`rb` — o raio efetivo é interpolado pelo `t` CLAMPADO (a cápsula
/// de raio variável). É a definição do objeto; o shader tem de convergir a ela.
fn capsule_dn(p: (f32, f32), a: (f32, f32), b: (f32, f32), ra: f32, rb: f32) -> f32 {
    let ab = (b.0 - a.0, b.1 - a.1);
    let ap = (p.0 - a.0, p.1 - a.1);
    let len_sq = ab.0 * ab.0 + ab.1 * ab.1;
    if len_sq < 1e-6 {
        return 1e9; // cápsula degenerada: não existe
    }
    let t = ((ap.0 * ab.0 + ap.1 * ab.1) / len_sq).clamp(0.0, 1.0);
    let d = (ap.0 - t * ab.0, ap.1 - t * ab.1);
    let radius = (ra + (rb - ra) * t).max(1e-4);
    (d.0 * d.0 + d.1 * d.1).sqrt() / radius
}

/// A polilinha de um traço: pontos + raios por-ponto (px) + fechamento.
struct Polyline<'a> {
    pts: &'a [(f32, f32)],
    radii: &'a [f32],
    closed: bool,
    hardness: f32,
}

/// Largura MÍNIMA rasterizada (px) — abaixo dela o traço não afina, desbota. É
/// parte da APARÊNCIA do objeto (o par clamp+fade do Grease Pencil), não um detalhe
/// do shader: uma linha de 0,3 px que "afinasse" piscaria ao mover.
const MIN_WIDTH_PX: f32 = 1.3;

impl Polyline<'_> {
    /// Índices dos segmentos (com o de fechamento, se cíclico).
    fn segments(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let n = self.pts.len();
        let last = if self.closed { n } else { n - 1 };
        (0..last).map(move |i| (i, (i + 1) % n))
    }

    /// O raio EFETIVO de um ponto: nunca abaixo do mínimo rasterizável.
    fn radius(&self, i: usize) -> f32 {
        self.radii[i].max(MIN_WIDTH_PX * 0.5)
    }

    /// A distância normalizada à POLILINHA = o mínimo sobre todos os segmentos.
    fn union_dn(&self, p: (f32, f32)) -> f32 {
        self.segments()
            .map(|(i, j)| capsule_dn(p, self.pts[i], self.pts[j], self.radius(i), self.radius(j)))
            .fold(f32::MAX, f32::min)
    }

    /// O fade sub-pixel: um traço mais fino que 1 px desbota em vez de afinar. Como
    /// os traços deste oráculo são grossos, isto é 1.0 em todos eles — está aqui
    /// para o modelo ser a APARÊNCIA completa, não um subconjunto conveniente.
    fn subpixel_fade(&self) -> f32 {
        let w = self.radii.iter().copied().fold(f32::MAX, f32::min) * 2.0;
        smoothstep01(w)
    }

    /// O menor raio EFETIVO da polilinha — a escala do gradiente de `dn` (≈ largura
    /// da faixa de AA por pixel).
    fn min_radius(&self) -> f32 {
        (0..self.pts.len())
            .map(|i| self.radius(i))
            .fold(f32::MAX, f32::min)
    }
}

/// Alpha esperado no pixel, ou `None` se o pixel é AMBÍGUO — nas duas faixas onde
/// CPU e GPU podem legitimamente discordar: o limiar do discard (`alpha ≈ 0.001`)
/// e a faixa de AA da borda (o shader fecha a borda com `fwidth`, que este modelo
/// não reproduz de propósito — modelar o AA seria modelar a implementação).
fn expected_union_alpha(poly: &Polyline<'_>, p: (f32, f32)) -> Option<f32> {
    let dn = poly.union_dn(p);
    // Faixa de AA: `fwidth(dn) ≈ 1/raio` por pixel; o shader atenua a borda em
    // `dn ∈ [1-aa, 1]`. Margem generosa (2 px) — o interesse do teste é o miolo.
    let aa = 2.0 / poly.min_radius().max(1e-4);
    if dn > 1.0 - aa && dn < 1.0 + aa {
        return None;
    }
    let m = cpu_mask(dn, poly.hardness) * poly.subpixel_fade();
    if m > 0.0002 && m < 0.005 {
        return None; // limiar do discard — o GPU pode pintar ou descartar
    }
    Some(if m < 0.001 { 0.0 } else { m })
}

/// Rasteriza a polilinha (um traço, opacity 1, cor sólida) e compara TODOS os
/// pixels não-ambíguos do alvo com a UNIÃO analítica.
fn assert_matches_union(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    poly: &Polyline<'_>,
    label: &str,
) {
    assert!(
        poly.hardness < 0.95,
        "{label}: o oráculo não modela o AA da borda dura — use hardness macio"
    );
    assert_eq!(poly.pts.len(), poly.radii.len(), "{label}: pts/radii");

    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for (&(x, y), &r) in poly.pts.iter().zip(poly.radii) {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: r * 2.0,
            opacity: 1.0,
            color: Rgba::new(1.0, 0.2, 0.1, 1.0),
        });
    }
    s.hardness = poly.hardness;
    s.closed = poly.closed;
    d.strokes.push(s);
    let px = render(device, queue, &d);

    let mut checked = 0u32;
    let mut worst = 0i32;
    let mut worst_at = (0u32, 0u32);
    let mut worst_pair = (0i32, 0i32);
    for y in 0..H {
        for x in 0..W {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let Some(exp) = expected_union_alpha(poly, p) else {
                continue;
            };
            let got = i32::from(alpha_at(&px, x, y));
            let want = (exp * 255.0).round() as i32;
            let diff = (got - want).abs();
            checked += 1;
            if diff > worst {
                worst = diff;
                worst_at = (x, y);
                worst_pair = (got, want);
            }
        }
    }
    assert!(
        checked > 500,
        "{label}: o oráculo cobriu pixels de menos ({checked})"
    );
    assert!(
        worst <= 8,
        "{label}: a GPU diverge da UNIÃO da polilinha - pior desvio {worst} em {worst_at:?} \
         (pintou {}, esperado {})",
        worst_pair.0,
        worst_pair.1
    );
}

/// Raios uniformes para uma polilinha de `n` pontos.
fn uniform_radii(r: f32, n: usize) -> Vec<f32> {
    vec![r; n]
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_soft_broken_corner_matches_the_polyline_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // **O TESTE DA MORDIDA** (o zigzag do smoke do Enio). Duas viradas afiadas
    // (~158°/~171° > 120° → `miter_break`) com hardness macio: os quads dos dois
    // segmentos se sobrepõem no disco da junção e, com depth first-wins, o
    // ANTERIOR vencia todos os pixels compartilhados pintando a sua queda RADIAL
    // por cima do NÚCLEO do seguinte — o "bocado arrancado". A aparência correta
    // é a UNIÃO da polilinha (docs/Flip/03 §3-§4).
    // O último trecho ainda cruza de volta o canto estendido do 1º segmento, onde
    // a máscara é ~0 — sem o `discard`, aquele fragmento transparente escreveria
    // depth e furaria o traço que passa depois (a classe "escamado").
    let pts = [(10.0, 32.0), (44.0, 32.0), (14.0, 44.0), (56.0, 35.0)];
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &uniform_radii(5.0, pts.len()),
            closed: false,
            hardness: 0.8,
        },
        "zigzag quebrado",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_soft_mitered_corner_matches_the_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Arco suave (viradas ~15-30° → fita MITRADA, junções compartilhadas) com
    // pincel gordo e hardness macio. Com largura uniforme a cobertura mitrada já
    // É a união hoje (a aresta de miter é exatamente a bissetriz dos raios — o
    // Voronoi das duas cápsulas), então este teste é **regressão, não
    // discriminante**: fica verde ANTES e DEPOIS do fix. Ele guarda o outro lado
    // do tripé — sem bead nas junções, sem escama, sem costura entre segmentos.
    let pts = [
        (8.0, 40.0),
        (20.0, 34.0),
        (32.0, 31.0),
        (44.0, 31.0),
        (56.0, 34.0),
    ];
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &uniform_radii(8.0, pts.len()),
            closed: false,
            hardness: 0.7,
        },
        "arco mitrado",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_soft_hairpin_matches_the_polyline_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Hairpin de 3 pontos (virada ~175°): o EXTREMO da sobreposição A∩B — os dois
    // quads estendidos cobrem quase a mesma região em torno da dobra. A janela de
    // vizinhos {A,B} é idêntica dos dois lados, então a união tem de sair exata.
    let pts = [(12.0, 20.0), (52.0, 26.0), (12.0, 30.0)];
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &uniform_radii(5.0, pts.len()),
            closed: false,
            hardness: 0.8,
        },
        "hairpin",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_closed_soft_star_seam_matches_the_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Traço FECHADO com quinas afiadas (estrela de 4 pontas — quinas de 90° seriam
    // MITRADAS e não discriminariam nada). Prova que a COSTURA do wrap (o último
    // segmento fecha no primeiro) ganha a mesma janela de vizinhos que o miolo:
    // o vertex já faz wrap de prev/next em traço cíclico.
    let pts = [
        (32.0, 8.0),
        (37.0, 27.0),
        (56.0, 32.0),
        (37.0, 37.0),
        (32.0, 56.0),
        (27.0, 37.0),
        (8.0, 32.0),
        (27.0, 27.0),
    ];
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &uniform_radii(4.0, pts.len()),
            closed: true,
            hardness: 0.8,
        },
        "estrela fechada",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn soft_round_caps_are_unchanged_by_the_neighbor_window() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Traço de 2 pontos: a janela de vizinhos é TODA sentinela (não há prev nem
    // next). Regressão das tampas redondas — a cápsula única com os discos das
    // pontas — e prova que a sentinela (cápsula degenerada) é ignorada de verdade.
    let pts = [(14.0, 24.0), (50.0, 40.0)];
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &uniform_radii(6.0, pts.len()),
            closed: false,
            hardness: 0.7,
        },
        "cap redondo",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_tapered_broken_corner_matches_the_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // **O teste do TAPER** (largura por-ponto = pressão de tablet, o caso NORMAL).
    // A MESMA geometria do zigzag da mordida, agora com o raio variando 3→8→5→3 px.
    // O fragment precisa normalizar a distância pelo raio interpolado no `t`
    // CLAMPADO da cápsula — a MESMA função para o próprio segmento e para os
    // vizinhos. Se o segmento próprio usar o raio interpolado sobre o QUAD (que
    // inclui as extensões), os dois vencedores possíveis de um pixel compartilhado
    // medem raios diferentes e a mordida sobrevive em 2ª ordem — invisível para
    // todos os outros testes, que usam largura uniforme (docs/Flip/03 §4.1, D1).
    let pts = [(10.0, 32.0), (44.0, 32.0), (14.0, 44.0), (56.0, 35.0)];
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &[3.0, 8.0, 5.0, 3.0],
            closed: false,
            hardness: 0.8,
        },
        "quina com taper",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_duplicated_point_does_not_tear_the_stroke() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Um ponto DUPLICADO no meio do traço (o tablet repete uma amostra; o smooth ou
    // o simplify fundem dois). O miter faria `normalize(0)` = NaN e o quad inteiro
    // sumiria — um rasgo no traço. O `safe_dir` do vertex trata o vizinho degenerado
    // como "sem vizinho", e o `capsule_dn` ignora a cápsula de comprimento zero: a
    // aparência tem de ser exatamente a da polilinha sem o ponto repetido.
    let pts = [
        (10.0, 32.0),
        (28.0, 26.0),
        (28.0, 26.0), // <-- repetido
        (54.0, 38.0),
    ];
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &uniform_radii(5.0, pts.len()),
            closed: false,
            hardness: 0.8,
        },
        "ponto duplicado",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_subpixel_thin_stroke_fades_instead_of_flickering() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Um traço mais FINO que um pixel não pode "afinar" — ele perde OPACIDADE
    // (`gpencil_frag.glsl:534`). Sem esse fade, a linha fina pisca e serrilha ao
    // mover/zoomar, porque o rasterizador acerta ou erra o centro do pixel.
    let thin = render(&device, &queue, &horizontal_stroke(0.35, 1.0));
    let one_px = render(&device, &queue, &horizontal_stroke(1.0, 1.0));
    let thick = render(&device, &queue, &horizontal_stroke(4.0, 1.0));

    // A linha de 0.35 px AINDA PINTA (não some) — só que fraca.
    let a_thin = i32::from(alpha_at(&thin, 32, 32));
    let a_one = i32::from(alpha_at(&one_px, 32, 32));
    let a_thick = i32::from(alpha_at(&thick, 32, 32));
    assert!(a_thin > 10, "a linha sub-pixel não pode sumir: {a_thin}");
    assert!(
        a_thin < a_one,
        "a sub-pixel tem de ser MAIS FRACA que a de 1 px: {a_thin} vs {a_one}"
    );
    assert!(
        a_one < a_thick,
        "e a de 1 px, mais fraca que a grossa: {a_one} vs {a_thick}"
    );
    // A grossa é opaca (o fade não pode tocar em traço normal).
    assert!(a_thick > 240, "traço normal fica intacto: {a_thick}");
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_ghost_is_the_same_silhouette_recoloured_and_faded() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Ghost Frames (W3.T3.3): o fantasma NÃO é a arte com opacidade baixa — é a
    // SILHUETA dela, 100% recolorida na cor do lado (verde = passado) e esmaecida
    // pelo fade. Duas coisas têm de valer ao mesmo tempo:
    //   (a) a COBERTURA é idêntica à do passe normal (é o mesmo desenho);
    //   (b) a COR é a do tint, não a do traço (que aqui é vermelho puro).
    let art = horizontal_stroke(6.0, 0.4); // borda macia: pega o perfil todo
    let normal = render(&device, &queue, &art);

    let green = [0.145, 0.420, 0.137];
    let fade = 0.5;
    let cam = pixel_camera().with_ghost_tint(green, fade);
    let ghost = render_with(&device, &queue, &art, cam);

    // (a) A silhueta: em cada coluna do traço, o alpha do fantasma é o alpha normal
    // × fade — o perfil inteiro (núcleo E queda macia), não só o miolo.
    for y in 28..37 {
        let a_n = f32::from(alpha_at(&normal, 32, y));
        let a_g = f32::from(alpha_at(&ghost, 32, y));
        assert!(
            (a_g - a_n * fade).abs() <= 3.0,
            "y={y}: o fantasma tem de ser a MESMA cobertura × {fade} (normal {a_n}, ghost {a_g})"
        );
    }
    // (b) A cor: o traço é VERMELHO, o fantasma sai VERDE. Num traço DURO o núcleo é
    // chapado (alpha = fade), então a des-multiplicação do alvo premultiplicado é
    // exata — sem ruído de quantização.
    let hard_cam = pixel_camera().with_ghost_tint(green, fade);
    let hard = horizontal_stroke(6.0, 1.0);
    let normal = render(&device, &queue, &hard);
    let ghost = render_with(&device, &queue, &hard, hard_cam);
    let a = f32::from(alpha_at(&ghost, 32, 32)) / 255.0;
    assert!(
        (a - fade).abs() < 0.02,
        "o núcleo do fantasma é o fade puro: {a} vs {fade}"
    );
    let rgb = rgb_at(&ghost, 32, 32);
    let straight = [
        f32::from(rgb[0]) / 255.0 / a,
        f32::from(rgb[1]) / 255.0 / a,
        f32::from(rgb[2]) / 255.0 / a,
    ];
    for (i, want) in green.iter().enumerate() {
        assert!(
            (straight[i] - want).abs() < 0.05,
            "canal {i}: fantasma sai na cor do TINT ({straight:?} vs {green:?})"
        );
    }
    // E o passe normal segue vermelho (o tint não vaza para quem não é fantasma).
    let n = rgb_at(&normal, 32, 32);
    assert!(
        n[0] > 200 && n[1] < 40,
        "o traço normal continua vermelho: {n:?}"
    );
}
