//! T1.2/T1.3 — validação GPU end-to-end (headless), clean-room.
//!
//! Renderiza um traço numa textura offscreen, lê os pixels de volta e afirma o
//! comportamento observável: o traço pinta uma banda; o fundo fica vazio; a
//! hardness controla a queda de borda. `#[ignore]` — precisa de adapter (roda com
//! `--ignored`; skip gracioso sem GPU), como os testes do `ph2d-gpu`.

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba, StrokeTip};
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

/// 🔴 **Self Overlap ON: o cruzamento ACUMULA** (03 §8). O MESMO X do teste acima (1 traço,
/// opacity 0.5), agora com `self_overlap = true`: a profundidade vira por-SEGMENTO, então no
/// cruzamento a 2ª passagem BLENDA sobre a 1ª em vez de ser descartada — `α = 0.5 + 0.5·(1−0.5)
/// = 0.75 → ~191`, enquanto um braço (1 passagem) fica em `0.5 → ~128`. É o `GP_STROKE_OVERLAP`
/// do Grease Pencil (marcador/nanquim expressivo).
///
/// **Red-first:** com o código de hoje sem a flag (ou com a depth de volta a por-TRAÇO) o
/// cruzamento pinta UMA vez (`cross == arm`), e a asserção `cross > arm` fica VERMELHA. A
/// mutação que sangra é a depth por-segmento → por-traço (`z = base`, ignorando `li/count`).
///
/// ⚠️ **A quina afiada TAMBÉM acumula, por design** (as fitas estendidas do `miter_break` se
/// sobrepõem) — é o *bleed* de marcador, o preço do modelo; a asserção `corner > arm` o PINA
/// (para ninguém "consertá-lo" de volta em silêncio; o smoke o mostra e o Enio decide se quer
/// quina limpa depois). Curva SUAVE não acumula (as fitas mitram/abutam).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_stroke_with_self_overlap_accumulates_at_the_crossing() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
    let blue = Rgba::new(0.0, 0.0, 1.0, 1.0);
    // O MESMO X do teste OFF: cruzamento em (32,32), quinas afiadas em (48,48)/(48,16).
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
    s.self_overlap = true; // <-- a única diferença para o teste OFF acima
    d.strokes.push(s);

    let px = render(&device, &queue, &d);
    let arm = i32::from(alpha_at(&px, 20, 20));
    let cross = i32::from(alpha_at(&px, 32, 32));
    // Um braço é UMA passagem: 0.5 → ~128 (byte-idêntico ao caso OFF; a flag não muda o braço).
    assert!((arm - 128).abs() <= 6, "braço a opacity 0.5: {arm}");
    // O cruzamento é DUAS passagens compostas (over): 0.5 + 0.5·0.5 = 0.75 → ~191, mais ESCURO
    // que o braço. (No OFF era `cross == arm`; a flag é exatamente esta diferença.)
    assert!(
        (cross - 191).abs() <= 8,
        "cruzamento acumula 2 camadas (0.75): cross={cross} (braço={arm})"
    );
    assert!(
        cross > arm + 40,
        "Self Overlap: o cruzamento tem de ficar mais ESCURO que o braço: cross={cross} arm={arm}"
    );
    // A passagem MAIS NOVA (azul, seg. de índice maior) fica por cima no blend — o oposto do
    // first-wins do OFF (onde a vermelha, mais antiga, vencia).
    let rgb = rgb_at(&px, 32, 32);
    assert!(
        rgb[2] > rgb[0],
        "Self Overlap: a passagem mais nova (azul) compõe por cima: {rgb:?}"
    );
    // ⚠️ A quina afiada (miter_break em (48,48)) também acumula, por design (marcador bleed).
    let corner = i32::from(alpha_at(&px, 48, 48));
    assert!(
        corner > arm + 30,
        "a quina afiada tambem acumula (bleed de marcador, por design): corner={corner} arm={arm}"
    );
}

/// Um traço horizontal (o mesmo de `horizontal_stroke`) com o pincel **AIRBRUSH** ligado.
fn airbrush_horizontal(width: f32, hard: f32) -> FlipDrawing {
    let mut d = horizontal_stroke(width, hard);
    d.strokes[0].airbrush = true;
    d
}

/// 🔴 **O pincel AIRBRUSH carrega tinta ATÉ O ARO; o padrão já parou lá** (03 §8).
/// O falloff do airbrush é a transmitância física de um dab esférico `A = 1 − exp(−k·√(1−dn²))`,
/// `k` da hardness — um domo que cobre quase todo o raio antes de cair.
///
/// ⚠️ **O DISCRIMINANTE MUDOU DE LUGAR em 2026-07-28, e o gate antigo teria ficado ao CONTRÁRIO.**
/// Ele comparava o **eixo** e o **meio-raio**, porque o padrão era o `pow`+smoothstep do GP — um
/// PICO que já caía a um pixel do centro (medido então: `std_axis=222`, `std_mid=0`). Com a lei do
/// Painter o padrão tem PLATÔ: agora `std_axis=255` (MAIOR que os 252 do airbrush, invertendo a
/// asserção `air_axis > std_axis + 15`) e `std_mid=248` (o meio-raio é exatamente o fim do platô
/// em hardness 0.5, então "o padrão desaba no meio-raio" virou FALSO por projeto).
/// A diferença entre os dois pincéis não sumiu — ela MUDOU DE LUGAR, do centro para o ARO
/// (delta máximo medido 0,76 sobre `dn`, `tests/hardness_law.rs`).
///
/// **Red-first:** com o ramo do airbrush revertido (a flag ignorada) os dois perfis viram o MESMO,
/// e as asserções de aro (`air` alto onde `std` já é resíduo) ficam VERMELHAS. A mutação que sangra
/// é `hardness_mask` ignorar o parâmetro `airbrush`.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn an_airbrush_has_a_flatter_core_than_the_standard_brush() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Banda de 20px (raio 10) no meio, opacity 1, hardness 0.5. Eixo em y=32; o ARO em y=40
    // (dn≈0.8) e y=41 (dn≈0.9) — é lá que os dois perfis discordam sob a lei do Painter.
    let std = render(&device, &queue, &horizontal_stroke(20.0, 0.5));
    let air = render(&device, &queue, &airbrush_horizontal(20.0, 0.5));

    let std_axis = i32::from(alpha_at(&std, 32, 32));
    let air_axis = i32::from(alpha_at(&air, 32, 32));
    let std_rim = i32::from(alpha_at(&std, 32, 40));
    let air_rim = i32::from(alpha_at(&air, 32, 40));
    let std_outer = i32::from(alpha_at(&std, 32, 41));
    let air_outer = i32::from(alpha_at(&air, 32, 41));

    // Medido (RTX, hardness 0.5): std 255/255/255/255/255/248/200/127/**55**/**7**/0 de y=32 a 42;
    // air 252/252/252/251/250/249/247/242/**231**/**192**/0. Os dois têm núcleo cheio — o que os
    // separa é o ARO.
    //
    // O airbrush NÃO é totalmente opaco nem no eixo (`1−exp(−4.5) = 0.989` → 252): a névoa mais
    // densa ainda deixa passar. É o que o torna uma névoa e não um disco.
    assert!(
        (240..255).contains(&air_axis),
        "airbrush denso mas nao opaco no eixo: {air_axis}"
    );
    // O padrão a hardness 0.5 é CHEIO no eixo (o platô) — a metade que prova que a lei do Painter
    // chegou, e que o gate NÃO está medindo dois pincéis macios quaisquer.
    assert!(std_axis > 250, "o padrao tem PLATO no eixo: {std_axis}");
    // ⚠️ O DISCRIMINANTE: no aro o padrão já é resíduo e o airbrush ainda pinta quase cheio.
    // Fosso de ~4× em dn≈0.8 e ~27× em dn≈0.9 — nenhum ajuste de hardness fecha isso, porque a
    // forma do domo (`√(1−dn²)`) só chega a zero EXATAMENTE no aro.
    assert!(
        air_rim > 200 && std_rim < 100,
        "no aro (dn≈0.8) o domo segura e o padrao ja caiu: air={air_rim} std={std_rim}"
    );
    assert!(
        air_outer > 150 && std_outer < 40,
        "e um pixel adiante o fosso ABRE: air={air_outer} std={std_outer}"
    );
    // E a borda do airbrush é SEMPRE macia: rola suave a zero em dn=1, nunca um corte.
    assert!(
        air_outer < air_rim && air_rim < air_axis,
        "a nevoa do airbrush cai MONOTONICA ate zero: {air_axis} > {air_rim} > {air_outer}"
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
/// pixels da faixa de AA são pulados por `expected_union_alpha`.
///
/// ⚠️ Esta é a **SEGUNDA cópia** da lei do perfil (a 1ª é o `hardness_mask` do `flip.wgsl`), e ela
/// é o preço de o oráculo ser independente: um oráculo que chamasse o shader não provaria nada.
/// Ela ficou para trás quando a lei virou a do Painter em 2026-07-28 — **nove** gates de união
/// caíram de uma vez, e é assim que essa dívida se cobra. Quem mexer no `hardness_mask` mexe aqui,
/// e a paridade termo-a-termo com `ph2d_painter_brush` vive em `tests/hardness_law.rs`.
///
/// A LEI: platô cheio até `hardness`, e a curva `Falloff::Smooth` (`3p²−2p³`) na faixa restante.
/// Sem transcendental, e sem a restrição de expoente inteiro que a lei do GP exigia.
fn cpu_mask(dn: f32, hardness: f32) -> f32 {
    let h = hardness.clamp(0.0, 1.0);
    if h >= 1.0 {
        return f32::from(dn < 1.0);
    }
    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
    let p = 1.0 - remapped;
    3.0 * p * p - 2.0 * p * p * p
}

/// 🔴 **O oráculo de união FALA a lei do Painter** — o elo que fecha a cadeia de prova.
///
/// Os 9 gates de união provam `shader ≡ cpu_mask` (numericamente, em milhares de pixels); este
/// prova `cpu_mask ≡ ph2d_painter_brush`, a função que o Painter de fato roda. Sem ele os dois
/// lados podiam derivar JUNTOS e todo gate de união ficaria verde sobre a lei errada — foi
/// exatamente o que aconteceu até 2026-07-28, com o shader e o oráculo concordando na lei do GP.
///
/// Não precisa de adapter: é aritmética.
#[test]
fn the_union_oracle_is_the_painters_law() {
    for hi in 0..=20 {
        let h = hi as f32 / 20.0;
        for di in 0..=200 {
            let dn = di as f32 / 200.0;
            // `BrushSpec::falloff_weight`: o platô remapeia, o preset avalia.
            let expected = if h >= 1.0 {
                f32::from(dn < 1.0)
            } else {
                let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
                ph2d_painter_brush::Falloff::Smooth.weight(remapped)
            };
            let got = cpu_mask(dn, h);
            assert!(
                (got - expected).abs() < 1e-6,
                "hardness {h} dn {dn}: oraculo {got} != Painter {expected}"
            );
        }
    }
}

/// **SONDA** — um traço MACIO que cruza a SI MESMO (o report do Enio, 2026-07-28, 2ª foto:
/// *"se o mesmo traço cruza a si mesmo então temos o mesmo aspecto indesejado"*).
///
/// ⚠️ A rota é OUTRA, e é por isso que dois traços cruzados passam: com traços distintos o depth
/// difere e o mais NOVO pinta por cima (nunca há união). No auto-cruzamento o depth é o MESMO
/// (por-traço) e o `GREATER` estrito DESCARTA o 2º quad ⇒ o pixel fica com o que o 1º computou,
/// e só a lista de extras (`seg_extras`) pode fazer os dois concordarem.
///
/// ⚠️ O gate `a_stroke_crossing_itself_is_a_clean_union_without_accumulation` usa
/// `hardness = 1.0` — **disco duro, onde a união é exata por construção fora da franja de AA**
/// (o §3 do doc 03 diz isso com todas as letras). Ele não pode ver este defeito.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_a_soft_stroke_crossing_itself() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Cada caso: rótulo + polilinha. Todos com pincel GROSSO e MACIO — o regime da foto.
    let x_detour: Vec<(f32, f32)> = vec![(16.0, 16.0), (48.0, 48.0), (48.0, 16.0), (16.0, 48.0)];
    let zigzag: Vec<(f32, f32)> = vec![(12.0, 12.0), (46.0, 40.0), (14.0, 40.0), (50.0, 14.0)];
    let loop_back: Vec<(f32, f32)> = vec![
        (14.0, 32.0),
        (34.0, 14.0),
        (50.0, 32.0),
        (34.0, 50.0),
        (24.0, 24.0),
    ];
    let cases: [(&str, &Vec<(f32, f32)>); 3] = [
        ("X com desvio", &x_detour),
        ("zigzag (a foto)", &zigzag),
        ("laco que volta", &loop_back),
    ];

    // ⚠️ A DENSIDADE é a variável: um traço de MÃO passa pelo RDP e depois pela reamostragem
    // (`0.4 × largura` de arco), então ele chega ao pack com DEZENAS de segmentos por perna, não
    // com um. Uma fixture de 3 segmentos cabe folgada em qualquer orçamento — ela não contém o
    // fenômeno. `step` em px de arco: `None` = a polilinha crua.
    fn densify(pts: &[(f32, f32)], step: f32) -> Vec<(f32, f32)> {
        let mut out = vec![pts[0]];
        for w in pts.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (len / step).ceil().max(1.0) as usize;
            for k in 1..=n {
                let t = k as f32 / n as f32;
                out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        out
    }

    println!("\n=== TRACO MACIO CRUZANDO A SI MESMO vs a UNIAO analitica ===");
    // raio 4 ⇒ largura 8 ⇒ o passo do produto é 0.4·8 = 3.2 px de arco.
    // ⚠️ O RAIO decide o que a sonda consegue VER: o oráculo pula a faixa `aa = 2/raio` em `dn`,
    // então com raio 4 ele descarta metade do traço e vira um teste BINÁRIO (núcleo sólido vs
    // fora vazio), cego ao ombro macio — que é exatamente onde o defeito da foto mora. Raio 20
    // ⇒ `aa = 0.1` ⇒ ele compara até `dn = 0.9`.
    for radius in [4.0_f32, 12.0, 20.0] {
        for step in [f32::INFINITY, radius * 0.8] {
            println!(
                "\n-- raio {radius} px | passo de reamostragem: {} --",
                if step.is_finite() {
                    format!("{step} px")
                } else {
                    "cru (3 segmentos)".to_string()
                }
            );
            for hardness in [0.7_f32, 0.4] {
                for (label, raw) in &cases {
                    let dense = if step.is_finite() {
                        densify(raw, step)
                    } else {
                        (*raw).clone()
                    };
                    let pts = &dense;
                    let radii: Vec<f32> = vec![radius; pts.len()];
                    let poly = Polyline {
                        pts,
                        radii: &radii,
                        closed: false,
                        hardness,
                    };
                    let mut d = FlipDrawing::new();
                    let mut st = FlipStroke::new();
                    for (&(x, y), &r) in pts.iter().zip(&radii) {
                        st.push_point(Point {
                            pos: Vec2::new(x, y),
                            width: r * 2.0,
                            opacity: 1.0,
                            color: Rgba::new(1.0, 0.2, 0.1, 1.0),
                        });
                    }
                    st.hardness = hardness;
                    d.strokes.push(st);
                    let px = render(&device, &queue, &d);

                    let (mut worst, mut at, mut pair, mut checked, mut bad) =
                        (0i32, (0u32, 0u32), (0, 0), 0u32, 0u32);
                    for y in 0..H {
                        for x in 0..W {
                            let Some(exp) =
                                expected_union_alpha(&poly, (x as f32 + 0.5, y as f32 + 0.5))
                            else {
                                continue;
                            };
                            let got = i32::from(alpha_at(&px, x, y));
                            let want = (exp * 255.0).round() as i32;
                            let diff = (got - want).abs();
                            checked += 1;
                            if diff > 8 {
                                bad += 1;
                            }
                            if diff > worst {
                                worst = diff;
                                at = (x, y);
                                pair = (got, want);
                            }
                        }
                    }
                    println!(
                        "  h={hardness:.1} {label:>18} ({:3} pts): pior {worst:3} em {at:?} (viu {}, pede {}) | \
                 {bad} px fora de 8 de {checked}",
                        pts.len(),
                        pair.0,
                        pair.1
                    );
                }
            }
        }
    }
}

/// 🔴 **UM traço que cruza a SI MESMO pinta o que DOIS traços cruzados pintam** — o oráculo é o
/// desenho que o Enio APROVOU (2026-07-28, 2ª foto: dois traços cruzados certos, o mesmo traço
/// cruzando a si mesmo errado).
///
/// ⚠️ **As duas rotas são código diferente e é por isso que o defeito existia:** traços distintos
/// têm depth diferente ⇒ o mais novo pinta por cima ⇒ **composição `over`**, lisa. Um traço só tem
/// o MESMO depth ⇒ o `GREATER` estrito descarta o 2º quad ⇒ **união** (`min` de distâncias), e
/// `min` de duas funções lisas tem **VINCO** na bissetriz.
///
/// **Red-first, medido:** com a composição revertida para a união o pior desvio no disco do
/// cruzamento é **48/255 em hardness 0.4** e **35/255 em 0.7**; com ela, **1/255** (arredondamento)
/// e **zero** pixel fora de 8. ⚠️ Em `hardness = 1.0` as duas leis já concordavam (a máscara é
/// binária) — é por isso que nenhum gate via isto, e é por isso que a fixture é MACIA.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_stroke_crossing_itself_paints_what_two_crossing_strokes_paint() {
    let Some((device, queue)) = device() else {
        return;
    };
    for hardness in [0.7_f32, 0.4] {
        let (one, two) = crossing_pair(hardness);
        let px_one = render(&device, &queue, &one);
        let px_two = render(&device, &queue, &two);
        let (worst, at, bad, checked) = crossing_disc_diff(&px_one, &px_two);
        assert!(checked > 500, "o disco cobriu pixels de menos ({checked})");
        assert!(
            worst <= 4 && bad == 0,
            "h={hardness}: um traco cruzando a si mesmo difere de dois traços cruzados -- \
             pior {worst} em {at:?}, {bad} px fora de 8 (de {checked})"
        );
    }
}

/// 🔴 **Um traço DENSO que NÃO cruza a si mesmo é EXATAMENTE a união** — a garantia de segurança
/// da composição por passagem, e o gate que faltava.
///
/// ⚠️ **Ele existe porque uma mutação sobreviveu.** Neutralizar a partição (`ribbon = 0`, que faz
/// a própria fita contar como passagem ESTRANHA) passou nos 30 gates: as pernas do X do gate
/// irmão são UM segmento cada — não há fita local para compor — e os 9 gates de união usam
/// **raio 4**, onde a faixa de AA do oráculo (`2/raio = 0.5` em `dn`) engole o OMBRO inteiro e
/// eles viram um teste binário (núcleo sólido × fora vazio).
///
/// Aqui a curva é **DENSA** (a densidade que o `resample_smooth` produz) e o raio é **14**, então
/// o oráculo compara até `dn ≈ 0.86` — o ombro entra. Sob a mutação a fita se compõe consigo
/// mesma e o traço engorda.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_dense_soft_ribbon_that_never_crosses_itself_is_exactly_the_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // Um arco denso: 24 pontos, passo ~2 px de arco num raio de pincel 14 (mais denso que o
    // produto, de propósito — é onde a fita local enche a lista).
    let mut pts = Vec::new();
    for k in 0..24u32 {
        let a = std::f32::consts::PI * 0.9 * (k as f32 / 23.0);
        pts.push((32.0 + 22.0 * a.cos(), 34.0 - 22.0 * a.sin()));
    }
    let radii = vec![14.0_f32; pts.len()];
    for hardness in [0.7_f32, 0.4] {
        assert_matches_union(
            &device,
            &queue,
            &Polyline {
                pts: &pts,
                radii: &radii,
                closed: false,
                hardness,
            },
            &format!("fita densa h={hardness}"),
        );
    }
}

/// O X em duas montagens: `(um traço, dois traços)`. **Porta única** — o gate e a sonda encenam
/// pela MESMA função, senão eles mediriam figuras diferentes e a sonda deixaria de diagnosticar
/// o gate.
fn crossing_pair(hardness: f32) -> (FlipDrawing, FlipDrawing) {
    let leg_a = [(10.0_f32, 10.0_f32), (54.0, 54.0)];
    let leg_b = [(54.0_f32, 10.0_f32), (10.0, 54.0)];
    let r = 14.0_f32;
    let mk = |p: (f32, f32)| Point {
        pos: Vec2::new(p.0, p.1),
        width: r * 2.0,
        opacity: 1.0,
        color: Rgba::new(1.0, 1.0, 1.0, 1.0),
    };

    // ⚠️ **A polilinha é DENSA, e não é capricho.** O produto reamostra todo traço
    // (`resample_smooth`, passo `0.4 × largura`), então segmentos GIGANTES não existem nele — e
    // com eles a fixture MENTE: a fronteira de passagem é "a caminhada saiu da vizinhança", e a
    // distância segmento-a-segmento de dois segmentos enormes vê só as PONTAS, então a caminhada
    // atravessava o desvio inteiro sem nunca sair, e o braço que cruza virava a mesma passagem.
    // Com segmentos curtos o desvio é uma fila de segmentos LONGE, a caminhada quebra no primeiro
    // deles, e o braço de volta é a passagem estranha que ele de fato é.
    fn densify(a: (f32, f32), b: (f32, f32), out: &mut Vec<(f32, f32)>) {
        const STEP: f32 = 5.0;
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let n = (len / STEP).ceil().max(1.0) as usize;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }

    let mut one = FlipDrawing::new();
    let mut s1 = FlipStroke::new();
    // O desvio vai para FORA do quadro (x=120): ele existe só porque um traço SÓ não pode
    // desenhar um X sem voltar, e dentro do quadro mediria diferença de FORMA em vez de LEI.
    let mut path = vec![leg_a[0]];
    for (a, b) in [
        (leg_a[0], leg_a[1]),
        (leg_a[1], (120.0, 32.0)),
        ((120.0, 32.0), leg_b[0]),
        (leg_b[0], leg_b[1]),
    ] {
        densify(a, b, &mut path);
    }
    for p in path {
        s1.push_point(mk(p));
    }
    s1.hardness = hardness;
    one.strokes.push(s1);

    let mut two = FlipDrawing::new();
    for leg in [leg_a, leg_b] {
        let mut s = FlipStroke::new();
        let mut pts = vec![leg[0]];
        densify(leg[0], leg[1], &mut pts);
        for p in pts {
            s.push_point(mk(p));
        }
        s.hardness = hardness;
        two.strokes.push(s);
    }
    (one, two)
}

/// A discordância entre dois renders num DISCO de raio 18 em torno do cruzamento (32,32) —
/// `(pior, onde, quantos fora de 8, quantos conferidos)`.
fn crossing_disc_diff(a: &[u8], b: &[u8]) -> (i32, (u32, u32), u32, u32) {
    let (mut worst, mut at, mut bad, mut checked) = (0i32, (0u32, 0u32), 0u32, 0u32);
    for y in 14..50u32 {
        for x in 14..50u32 {
            if (x as f32 - 32.0).powi(2) + (y as f32 - 32.0).powi(2) > 18.0 * 18.0 {
                continue;
            }
            checked += 1;
            let d = (i32::from(alpha_at(a, x, y)) - i32::from(alpha_at(b, x, y))).abs();
            if d > 8 {
                bad += 1;
            }
            if d > worst {
                worst = d;
                at = (x, y);
            }
        }
    }
    (worst, at, bad, checked)
}

/// **SONDA** — o MESMO X, desenhado como UM traço e como DOIS: a comparação literal da foto do
/// Enio (2026-07-28, 2ª foto). Ele aprovou o de dois e reprovou o de um.
///
/// ⚠️ As duas rotas são código diferente: com traços distintos o depth difere e o mais NOVO pinta
/// por cima ⇒ **composição `over`**, que é lisa. Com um traço só o depth é o mesmo, o `GREATER`
/// estrito descarta o 2º quad ⇒ **união** (`profile(min dn)`), e `min` de duas funções lisas tem
/// **VINCO** (gradiente descontínuo) na bissetriz. Com hardness 1 o vinco é invisível (a máscara
/// é binária); com pincel macio ele é uma costura na tela.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_one_stroke_crossing_itself_against_two_strokes() {
    let Some((device, queue)) = device() else {
        return;
    };
    // O X: duas diagonais que se cruzam em (32,32). Como UM traço, a 2ª perna vem depois de um
    // desvio (duas quinas afiadas) — a única forma de um traço só desenhar um X.
    let leg_a = [(10.0_f32, 10.0_f32), (54.0, 54.0)];
    let leg_b = [(54.0_f32, 10.0_f32), (10.0, 54.0)];

    let mk_point = |p: (f32, f32), r: f32| Point {
        pos: Vec2::new(p.0, p.1),
        width: r * 2.0,
        opacity: 1.0,
        color: Rgba::new(1.0, 1.0, 1.0, 1.0),
    };

    println!("\n=== UM TRACO CRUZANDO A SI MESMO vs DOIS TRACOS CRUZADOS ===");
    println!("(alpha ao longo da BISSETRIZ, saindo do cruzamento em (32,32) para o nordeste)");
    for hardness in [1.0_f32, 0.7, 0.4] {
        let r = 14.0;

        // (a) UM traço: perna A, desvio, perna B.
        let mut one = FlipDrawing::new();
        let mut s1 = FlipStroke::new();
        // ⚠️ O desvio vai para FORA do quadro (x=120): ele existe só porque um traço SÓ não
        // pode desenhar um X sem voltar, e dentro do quadro ele mediria diferença de FORMA
        // (o par de traços não o tem) em vez da diferença de LEI, que é o que se mede aqui.
        for p in [leg_a[0], leg_a[1], (120.0, 32.0), leg_b[0], leg_b[1]] {
            s1.push_point(mk_point(p, r));
        }
        s1.hardness = hardness;
        one.strokes.push(s1);

        // (b) DOIS traços independentes — o que o Enio aprovou.
        let mut two = FlipDrawing::new();
        for leg in [leg_a, leg_b] {
            let mut s = FlipStroke::new();
            for p in leg {
                s.push_point(mk_point(p, r));
            }
            s.hardness = hardness;
            two.strokes.push(s);
        }

        let px_one = render(&device, &queue, &one);
        let px_two = render(&device, &queue, &two);

        // A BISSETRIZ do cruzamento é o eixo +y a partir de (32,32): ali as duas pernas estão
        // EQUIDISTANTES, que é onde `min` faz o vinco e `over` não faz nada.
        let mut linha_one = String::new();
        let mut linha_two = String::new();
        for d in (0..=28u32).step_by(2) {
            linha_one.push_str(&format!("{:4}", alpha_at(&px_one, 32, 32 - d)));
            linha_two.push_str(&format!("{:4}", alpha_at(&px_two, 32, 32 - d)));
        }
        println!("  h={hardness:.1}  UM  :{linha_one}");
        println!("  h={hardness:.1}  DOIS:{linha_two}");

        // O tamanho da discordância — ⚠️ SÓ na metade esquerda (`x < 44`). O traço único tem um
        // DESVIO até (58,32) que o par de traços não tem; incluí-lo mediria a diferença de FORMA
        // (255, trivial) em vez da diferença de LEI, que é o que a foto mostra.
        // ⚠️ Só um DISCO em torno do cruzamento (raio 18): o traço único tem um DESVIO até
        // (58,32) que o par não tem, e incluí-lo mede diferença de FORMA em vez de LEI.
        let (mut worst, mut at, mut bad) = (0i32, (0u32, 0u32), 0u32);
        for y in 14..50u32 {
            for x in 14..50u32 {
                if (x as f32 - 32.0).powi(2) + (y as f32 - 32.0).powi(2) > 18.0 * 18.0 {
                    continue;
                }
                let a = i32::from(alpha_at(&px_one, x, y));
                let b = i32::from(alpha_at(&px_two, x, y));
                let _ = (a, b);
                if (a - b).abs() > 8 {
                    bad += 1;
                }
                if (a - b).abs() > worst {
                    worst = (a - b).abs();
                    at = (x, y);
                }
            }
        }
        println!("           pior divergencia {worst} em {at:?} | {bad} px fora de 8\n");
    }
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

/// **Um desenho SÓ DE PREENCHIMENTO renderiza — não derruba o app.**
///
/// Um traço `hide_stroke` não empurra ponto nenhum (só o `GpuStroke` com
/// `point_count: 0`, para manter o alinhamento do `sid`). Se TODOS os traços são
/// assim — e isso acontece de verdade: preencha uma região e apague o line-art com
/// a borracha, ou apague o "C" depois de fechá-lo com o Gap Closure — o buffer de
/// pontos fica VAZIO. Um storage buffer de tamanho zero é inválido na wgpu, e o
/// bind group inteiro explodia: "binding size is zero".
///
/// O fill tem de continuar aparecendo (ele é o desenho todo, agora).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_fill_only_drawing_renders_instead_of_crashing() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    // Um quadrado, sem contorno visível — só a cor.
    for p in [
        Vec2::new(16.0, 16.0),
        Vec2::new(48.0, 16.0),
        Vec2::new(48.0, 48.0),
        Vec2::new(16.0, 48.0),
    ] {
        s.push_point(Point {
            pos: p,
            width: 0.0,
            opacity: 1.0,
            color: Rgba::new(0.0, 1.0, 0.0, 1.0),
        });
    }
    s.closed = true;
    s.hide_stroke = true;
    s.fill = Some(Fill {
        color: Rgba::new(0.0, 1.0, 0.0, 1.0),
        opacity: 1.0,
    });
    d.strokes.push(s);

    // Se o buffer vazio voltar a ser criado com tamanho 0, isto entra em pânico.
    let px = render(&device, &queue, &d);

    assert!(
        alpha_at(&px, 32, 32) > 200,
        "o miolo do preenchimento tem de estar pintado"
    );
    let c = rgb_at(&px, 32, 32);
    assert!(c[1] > 200 && c[0] < 60, "o miolo e verde: {c:?}");
    assert_eq!(alpha_at(&px, 4, 4), 0, "fora do quadrado, vazio");
}

// ── O *tip* pontilhado (Dots/Squares, 03 §8) ──────────────────────────────────

/// O fixture do *tip*: um traço horizontal reto no meio do alvo — px (6,32)→(58,32), com a
/// `width` dada. O `spacing` é a pitch como MÚLTIPLO do diâmetro (relativo à espessura): `1.5`
/// ⇒ pitch = `1.5 × width` (mundo=px na câmera 1:1).
fn dotted_line(tip: StrokeTip, spacing: f32, width: f32) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for x in [6.0, 32.0, 58.0] {
        s.push_point(Point {
            pos: Vec2::new(x, 32.0),
            width,
            opacity: 1.0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
        });
    }
    s.tip = tip;
    s.dot_spacing = spacing;
    d.strokes.push(s);
    d
}

/// Um traço GROSSO em curva suave (senoide, 13 pontos), pontilhado. Largura 12; é o mais perto
/// de um traço desenhado à mão que dá para montar, e onde o *lens* de arco (o bug) estica as
/// contas numa banana. A área coberta das contas mede se elas são DISCOS cheios ou lentes
/// encolhidas.
fn curved_thick_dotted(tip: StrokeTip, spacing: f32) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for i in 0..=12 {
        let x = 6.0 + i as f32 * 4.3;
        let y = 32.0 + 14.0 * (i as f32 * 0.55).sin();
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 12.0,
            opacity: 1.0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
        });
    }
    s.tip = tip;
    s.dot_spacing = spacing;
    d.strokes.push(s);
    d
}

/// 🔴 **As contas de um traço GROSSO numa CURVA são DISCOS EUCLIDIANOS cheios, não lentes de
/// arco encolhidas.** O 2º report do Enio (2026-07-25, com screenshot): *"pontos deformados em
/// linhas grossas"*. A métrica antiga `√(dn² + da_arco²)` mede a distância AO-LONGO do arco, que
/// curva — então a conta esticava numa banana e ENCOLHIA a área coberta. Medindo o PONTO-centro
/// 2D e a distância EUCLIDIANA `|frag − C|`, a conta fica redonda e cheia em qualquer curva.
///
/// O oráculo é a ÁREA coberta pelas contas: o disco Euclidiano cobre **710** px; o *lens* de
/// arco encolhe para **621** (medido). O `bbox`/aspecto de uma conta isolada foi tentado e
/// REPROVADO como oráculo — numa curva as contas vizinhas invadem a janela e poluem a medida
/// (o disco cheio, sendo maior, polui MAIS, invertendo o sinal). A área é robusta e direta:
/// *contas cheias vs encolhidas*. Mutação que sangra: trocar a distância Euclidiana pela
/// métrica de arco (a área cai para ~621, < 665).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn dots_on_a_curved_thick_stroke_are_full_disks_not_shrunken_lenses() {
    let Some((device, queue)) = device() else {
        return;
    };
    let px = render(&device, &queue, &curved_thick_dotted(StrokeTip::Dots, 1.5));
    let mut area = 0u32;
    for y in 0..H {
        for x in 0..W {
            if alpha_at(&px, x, y) > 50 {
                area += 1;
            }
        }
    }
    assert!(
        area > 665,
        "as contas da curva grossa encolheram numa banana de arco (area {area}; disco cheio ~710, lens ~621)"
    );
}

/// 🔴 A linha CHEIA (`Continuous`) cobre a fileira do meio SEM buraco; a pontilhada (`Dots`)
/// RECORTA a MESMA fileira em contas — vãos onde a cheia é sólida.
///
/// Red-first (o *tip* é novo) e provado dos DOIS lados: a mutação "ignore o tip" (sempre
/// Continuous) tira os vãos da Dots e derruba a 2ª metade; "sempre Dots" (tirar o guard
/// `tip != 0`) dá vãos à Continuous e derruba a 1ª. O guard E o recorte sangram.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn dots_carve_gaps_that_a_continuous_line_does_not() {
    let Some((device, queue)) = device() else {
        return;
    };
    let row = 32;
    let span = || 10u32..54u32; // dentro do traço, longe das pontas

    let cont = render(
        &device,
        &queue,
        &dotted_line(StrokeTip::Continuous, 1.5, 8.0),
    );
    let min_cont = span().map(|x| alpha_at(&cont, x, row)).min().unwrap();
    assert!(
        min_cont > 40,
        "a linha CHEIA nao pode ter buraco na fileira do meio (min alpha {min_cont})"
    );

    let dots = render(&device, &queue, &dotted_line(StrokeTip::Dots, 1.5, 8.0));
    let min_dots = span().map(|x| alpha_at(&dots, x, row)).min().unwrap();
    let max_dots = span().map(|x| alpha_at(&dots, x, row)).max().unwrap();
    assert!(
        min_dots < 10,
        "a pontilhada tem de ter VAO (min alpha {min_dots}) — as contas sao espacadas"
    );
    assert!(
        max_dots > 100,
        "a pontilhada tem de ter CONTAS (max alpha {max_dots})"
    );
}

/// A conta QUADRADA cobre mais área que a REDONDA (quadrado de lado 2r vs disco de raio r:
/// 4r² vs πr² ≈ 1,27×) — é o que distingue `Squares` de `Dots` na tela. Mesmo fixture,
/// mesmo espaçamento; só o `tip` muda.
///
/// ⚠️ **Largura 20 (raio 10), não 8** — a raio 4 o disco e o quadrado ambos preenchem a
/// largura do traço (a conta é a meia-espessura), então a diferença mora só nas QUINAS, que
/// caem na borda do traço, e o AA de ~1 px domina a contagem (a razão desaba para ~6 %). Com
/// raio 10 o AA é uma fração pequena e a razão 4/π ≈ 1,27 aparece limpa.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn squares_cover_more_area_than_round_dots() {
    let Some((device, queue)) = device() else {
        return;
    };
    let covered = |px: &[u8]| -> u32 {
        let mut n = 0;
        for y in 0..H {
            for x in 0..W {
                if alpha_at(px, x, y) > 50 {
                    n += 1;
                }
            }
        }
        n
    };
    let dots = covered(&render(
        &device,
        &queue,
        &dotted_line(StrokeTip::Dots, 1.5, 20.0),
    ));
    let squares = covered(&render(
        &device,
        &queue,
        &dotted_line(StrokeTip::Squares, 1.5, 20.0),
    ));
    assert!(
        squares > dots + dots / 10,
        "o quadrado cobre >10% mais que o disco: quadrado {squares}, disco {dots}"
    );
}

// ---------- O laço e a hachura DENSIFICADOS: a classe que a suíte não tinha ----------
//
// **A premissa que faltava.** Todo gate de união acima usa polilinhas de 4-6 pontos
// escritas à mão, e os dois gates de auto-cruzamento cravam `hardness = 1.0` — o
// ÚNICO regime onde o defeito não pode existir (máscara binária ⇒ o raio de escrita
// É o raio visível ⇒ o first-wins vira união exata). Mas desde a reamostragem suave
// (T2.8, 2026-07-27) **todo traço do produto é DENSIFICADO**: o passo é
// `0.4 × largura = 0.8·r`, então um traço de 4 pontos vira dezenas.
//
// **Por que a densidade muda a resposta.** O broadphase de vizinhos alcança
// `2·r_i + r_j = 3·r` (`neighbors.rs`), e a razão
//
//     alcance / passo  =  3·r / 0.8·r  =  3,75
//
// é **constante** — as duas grandezas são proporcionais ao raio, então o pincel some
// da conta. Logo os vizinhos `i±1 … i±4` da PRÓPRIA fita entram na lista em qualquer
// espessura, antes de existir cruzamento nenhum, e comem os slots de
// `MAX_EXTRAS_PER_SEGMENT`. Onde o traço de fato volta sobre si mesmo, o segmento que
// cruza cai FORA da lista e aquele pixel volta ao first-wins: a GPU pinta a cauda
// macia de uma passagem por cima do NÚCLEO de outra.
//
// O par abaixo é o **discriminante**, não mais um verde: o LAÇO (um cruzamento, folga
// larga) passa; a HACHURA (voltas a meio raio) é a que sangra.

/// O passo de densificação do PRODUTO: `RESAMPLE_STEP_FRACTION = 0.4` × a largura
/// (`shells/desktop/src/flip_draw.rs::resample_step`), ou seja `0.8 · r`.
///
/// Escrito aqui como número e **não importado** de propósito: este gate mede o que o
/// produto faz *hoje*. Se aquele número mudar, o teste tem de ser **re-lido** — um
/// gate que acompanha a constante que ele vigia fica verde por construção.
const PRODUCT_RESAMPLE_STEP_OVER_RADIUS: f32 = 0.8;

/// Reamostra a polilinha por comprimento de arco no passo dado, preservando os
/// vértices originais. Só a DENSIDADE interessa aqui (a forma da curva é assunto do
/// `flip_smooth`), então a interpolação é linear.
fn densify(pts: &[(f32, f32)], step: f32) -> Vec<(f32, f32)> {
    let mut out = vec![pts[0]];
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let d = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if d > step {
            let n = (d / step).floor() as usize;
            for k in 1..=n {
                let t = k as f32 * step / d;
                if t < 1.0 {
                    out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
                }
            }
        }
        out.push(b);
    }
    out
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_densified_soft_loop_matches_the_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // O CONTROLE. Um laço que cruza a si mesmo UMA vez, com folga larga entre as duas
    // passagens: os segmentos que se cruzam estão longe no índice, mas sobram slots
    // para eles. Se este ficar vermelho, o problema não é o cap — é a união.
    const R: f32 = 5.0;
    let spine = [
        (12.0, 20.0),
        (44.0, 20.0),
        (44.0, 44.0),
        (20.0, 44.0),
        (20.0, 12.0),
    ];
    let pts = densify(&spine, PRODUCT_RESAMPLE_STEP_OVER_RADIUS * R);
    let radii = uniform_radii(R, pts.len());
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &radii,
            closed: false,
            hardness: 0.7,
        },
        "laço macio DENSIFICADO (cruza uma vez, folga larga)",
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_densified_soft_hatching_matches_the_union() {
    let Some((device, queue)) = device() else {
        return;
    };
    // O DISCRIMINANTE. Uma serpentina que volta sobre si mesma a **meio raio** de
    // distância — hachura, o gesto mais comum de quem sombreia. Densificada, cada
    // segmento tem os 4 vizinhos da própria fita dentro do alcance ANTES de qualquer
    // vizinho de outra passagem, e a lista satura: o segmento da passagem vizinha não
    // entra, e o núcleo dela é pintado pela cauda macia desta.
    const R: f32 = 5.0;
    const GAP: f32 = 0.5 * R;
    let (y0, y1) = (14.0, 50.0);
    let mut spine = Vec::new();
    for i in 0..5 {
        let x = 20.0 + i as f32 * GAP;
        let (from, to) = if i % 2 == 0 { (y0, y1) } else { (y1, y0) };
        spine.push((x, from));
        spine.push((x, to));
    }
    let pts = densify(&spine, PRODUCT_RESAMPLE_STEP_OVER_RADIUS * R);
    let radii = uniform_radii(R, pts.len());
    assert_matches_union(
        &device,
        &queue,
        &Polyline {
            pts: &pts,
            radii: &radii,
            closed: false,
            hardness: 0.7,
        },
        "hachura macia DENSIFICADA (voltas a meio raio)",
    );
}

/// SONDA (não é gate) — **quanto** o piso de largura mínima que FALTA no laço de
/// `seg_extras` muda a tela.
///
/// `MIN_WIDTH_PX` é aplicado ao raio das cápsulas PRÓPRIA/anterior/seguinte
/// (`flip.wgsl`, `min_r`) e **não** ao das cápsulas de `seg_extras` — os vizinhos
/// GEOMÉTRICOS, que é por onde um traço que volta sobre si mesmo se enxerga. A mesma
/// pergunta ("qual é o raio desta cápsula?") com duas portas, e uma delas esqueceu a
/// regra. O regime em que isso morde é o SUB-PIXEL, que não é exótico: é todo traço
/// depois de um zoom out.
///
/// ⚠️ **Onde o fenômeno VIVE, medido:** uma cápsula de extras só muda a união onde
/// ela GANHA o `min`, e todo fragment está a no máximo um raio do PRÓPRIO eixo. Num
/// **X** as duas passagens se cruzam em ângulo e a região em que a outra ganha é um
/// estilhaço onde as duas já saturam ⇒ o piso que falta é INVISÍVEL (medido:
/// byte-idêntico). Ele morde onde as duas passagens correm **quase paralelas e a menos
/// de um raio** — hachura vista de longe, o gesto que a wave anterior curou.
///
/// Rode ANTES e DEPOIS do fix e compare as linhas:
///   cargo test -p ph2d-flip-render --release --test gpu_render \
///     measure_the_missing_floor -- --ignored --nocapture
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_missing_floor_in_the_extras_loop() {
    let Some((device, queue)) = device() else {
        return;
    };
    // (a) O X: as duas passagens se cruzam em ângulo reto. Os segmentos que se cruzam
    //     estão a 44 unidades de arco um do outro, muito além do alcance da fita local
    //     ⇒ o cruzamento chega pela GRADE, que é exatamente o laço sem piso.
    let x_spine = [(10.0, 10.0), (54.0, 54.0), (54.0, 10.0), (10.0, 54.0)];
    // (b) O GRAMPO: ida e volta quase paralelas, a `GAP` px de distância. Com o traço
    //     sub-pixel a cápsula da OUTRA perna ganha o `min` numa faixa larga — é ali que
    //     o raio dela decide a tela.
    const GAP: f32 = 0.4;
    let hairpin = [
        (12.0, 32.0),
        (52.0, 32.0),
        (52.0, 32.0 + GAP),
        (12.0, 32.0 + GAP),
    ];
    for (name, spine, hardness) in [
        ("X          ", &x_spine[..], 0.7f32),
        ("grampo 0.4px", &hairpin[..], 0.9),
    ] {
        let at_floor = total_ink(&render_uniform(&device, &queue, spine, 0.65, hardness));
        println!("\n  {name} (hardness {hardness}) — tinta no piso {at_floor}");
        println!("  raio | piso? |  npx | soma alfa |  max");
        for &r in &[0.10f32, 0.20, 0.30, 0.45, 0.65, 1.00] {
            let mut d = FlipDrawing::new();
            let mut s = FlipStroke::new();
            for &(x, y) in spine {
                s.push_point(Point {
                    pos: Vec2::new(x, y),
                    width: r * 2.0,
                    opacity: 1.0,
                    color: Rgba::new(1.0, 0.2, 0.1, 1.0),
                });
            }
            s.hardness = hardness;
            d.strokes.push(s);
            let px = render(&device, &queue, &d);

            let mut n = 0u32;
            let mut sum = 0u64;
            let mut max = 0u8;
            for y in 0..H {
                for x in 0..W {
                    let a = alpha_at(&px, x, y);
                    if a > 0 {
                        n += 1;
                        sum += u64::from(a);
                        max = max.max(a);
                    }
                }
            }
            println!(
                "  {r:.2} | {:5} | {n:4} | {sum:9} | {max:4} | razao {:.4} | fade {:.4}",
                if r * 2.0 < MIN_WIDTH_PX { "SIM" } else { "nao" },
                sum as f64 / at_floor as f64,
                smoothstep01(r * 2.0),
            );
        }
    }
}

/// A tinta TOTAL de um traço (a soma dos alfas do alvo).
fn total_ink(px: &[u8]) -> u64 {
    let mut sum = 0u64;
    for y in 0..H {
        for x in 0..W {
            sum += u64::from(alpha_at(px, x, y));
        }
    }
    sum
}

/// Rasteriza `spine` como UM traço de largura uniforme `2r`.
fn render_uniform(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    spine: &[(f32, f32)],
    r: f32,
    hardness: f32,
) -> Vec<u8> {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for &(x, y) in spine {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: r * 2.0,
            opacity: 1.0,
            color: Rgba::new(1.0, 0.2, 0.1, 1.0),
        });
    }
    s.hardness = hardness;
    d.strokes.push(s);
    render(device, queue, &d)
}

/// **Um traço abaixo do piso é o traço NO piso, desbotado — e isso vale também onde
/// ele se enxerga.**
///
/// `MIN_WIDTH_PX` é um PAR: a GEOMETRIA é clampada (a fita não afina mais) e a
/// INTENSIDADE desbota (`smoothstep(0,1,espessura)`) — sem o clamp a linha fina não
/// cobre o centro de pixel nenhum e PISCA ao mover; sem o fade ela fica grossa demais.
/// O par É o contrato, e ele tem uma consequência verificável: com largura uniforme o
/// fade é um ESCALAR, então a tinta total a `r < piso` tem de ser a tinta a `r = piso`
/// vezes `smoothstep01(2r)`.
///
/// O laço de `seg_extras` — os vizinhos GEOMÉTRICOS, por onde um traço que volta sobre
/// si mesmo se enxerga — media o raio dessas cápsulas com piso `0.0`: a MESMA pergunta
/// que `min_r` responde no vertex, por uma segunda porta que esquecia a regra. A
/// cápsula do CRUZAMENTO ficava mais fina que a da própria fita, então a união media
/// menos cobertura exatamente onde as duas passagens se encontram.
///
/// ⚠️ **Onde o fenômeno VIVE, medido** (sonda `measure_the_missing_floor_in_the_extras_loop`):
/// num **X** ele é INVISÍVEL (byte-idêntico) — as duas passagens se cruzam em ângulo e a
/// região em que a outra ganha o `min` é um estilhaço onde as duas já saturam. Ele morde
/// onde elas correm quase **paralelas e a menos de um raio**: hachura vista de longe.
/// Medido no grampo a 0,4 px — tinta/tinta-no-piso, esperado vs sem o piso:
/// `r=0,20: 0,352 vs 0,170` · `r=0,30: 0,651 vs 0,454` · `r=0,45: 0,973 vs 0,813`
/// (até **52% da tinta** sumindo).
///
/// O oráculo é a PROPRIEDADE (clamp + fade), nunca a implementação do laço.
///
/// ⚠️ **O CONTROLE é o X, não um traço reto** — e a escolha é o achado. Um reto não
/// tem vizinho geométrico nenhum, então ele prova só que o resto do shader não regrediu;
/// o X percorre o MESMO laço de extras que o grampo e mesmo assim fica VERDE sob a
/// mutação. É o par que diz a coisa exata: *o laço é exercitado pelos dois, e só o
/// quase-paralelo consegue enxergar o piso que falta.*
///
/// **A BARRA saiu da medição:** com o piso, o pior desvio dos dois formatos é `0,004`;
/// sem ele, o grampo erra por `0,18-0,20`. `0,02` deixa 5× de folga sobre o ruído
/// medido e 9× de fosso até a mutação. `r=0,10` fica FORA de propósito — ali os alfas
/// caem para ~2/255 e o limiar de discard os come (medido: razão 0,018 contra 0,104
/// esperados, com o piso no lugar). O regime não é a wave.
/// Mutação que sangra: piso `0.0` de volta no laço de extras.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn a_sub_pixel_stroke_is_the_stroke_at_the_floor_faded_even_where_it_meets_itself() {
    let Some((device, queue)) = device() else {
        return;
    };
    const FLOOR_R: f32 = 1.3 * 0.5; // MIN_WIDTH_PX * 0.5
    const GAP: f32 = 0.4;
    // O CONTROLE: um X de um traço só — as duas passagens se cruzam em ÂNGULO, e a
    // região em que a cápsula da outra ganha o `min` é um estilhaço onde as duas já
    // saturam. Percorre o laço de extras e não enxerga o piso.
    let x_cross = [(10.0, 10.0), (54.0, 54.0), (54.0, 10.0), (10.0, 54.0)];
    // O DISCRIMINANTE: ida e volta quase PARALELAS a `GAP` px — as duas pernas são
    // NÃO-adjacentes (a volta as separa), então a outra chega pela GRADE, e ela ganha
    // o `min` numa faixa larga. É hachura vista de longe.
    let hairpin = [
        (12.0, 32.0),
        (52.0, 32.0),
        (52.0, 32.0 + GAP),
        (12.0, 32.0 + GAP),
    ];
    for (name, spine) in [
        ("X que cruza (CONTROLE)", &x_cross[..]),
        ("grampo 0.4px", &hairpin[..]),
    ] {
        let at_floor = total_ink(&render_uniform(&device, &queue, spine, FLOOR_R, 0.9));
        assert!(at_floor > 1000, "{name}: fixture vazia ({at_floor})");
        for &r in &[0.20f32, 0.30, 0.45] {
            let thin = total_ink(&render_uniform(&device, &queue, spine, r, 0.9));
            let got = thin as f64 / at_floor as f64;
            let want = f64::from(smoothstep01(r * 2.0));
            assert!(
                (got - want).abs() < 0.02,
                "{name} r={r:.2}: a tinta sub-pixel não é a tinta do piso desbotada \
                 — razão {got:.3}, esperada {want:.3} (o piso do laço de extras)"
            );
        }
    }
}

/// SONDA — **quanto** o `self_overlap` erra hoje. Com o bit, a profundidade vira
/// por-SEGMENTO e as faces sobrepostas BLENDam; mas cada face computa a **UNIÃO
/// GLOBAL** (`min` sobre TODAS as cápsulas alcançáveis), então as `N` faces que
/// cobrem o pixel compõem `1−(1−u)^N` com o MESMO `u` — a cobertura da passagem mais
/// próxima, contada `N` vezes. `N` é o número de QUADS sobrepostos, não o de
/// PASSAGENS.
///
/// A lei certa é a de TINTA: `α = 1 − Π_passagem (1 − mask(min das cápsulas DAQUELA
/// passagem))`. Num pixel a `dn=0,5` de uma passagem e `dn=0,9` da outra, o modelo de
/// hoje credita a segunda com `m(0,5)` em vez de `m(0,9)`.
///
///   cargo test -p ph2d-flip-render --release --test gpu_render \
///     measure_the_self_overlap -- --ignored --nocapture
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_self_overlap_double_count() {
    let Some((device, queue)) = device() else {
        return;
    };
    const OPACITY: f32 = 0.5;
    const HARDNESS: f32 = 0.7;
    const R: f32 = 6.0;
    // Um X de UM traço. A passagem A é o segmento 0, a B é o segmento 2; o segmento 1
    // (a vertical da direita) é o conector e fica longe do cruzamento.
    let spine = [(10.0, 10.0), (54.0, 54.0), (54.0, 10.0), (10.0, 54.0)];
    let render_flag = |on: bool| {
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        for &(x, y) in &spine {
            s.push_point(Point {
                pos: Vec2::new(x, y),
                width: R * 2.0,
                opacity: OPACITY,
                color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            });
        }
        s.hardness = HARDNESS;
        s.self_overlap = on;
        d.strokes.push(s);
        render(&device, &queue, &d)
    };
    let off = render_flag(false);
    let on = render_flag(true);

    // A lei de TINTA, na CPU: uma passagem = um segmento do X.
    let passage_alpha = |p: (f32, f32), seg: (usize, usize)| -> f32 {
        let dn = capsule_dn(p, spine[seg.0], spine[seg.1], R, R);
        cpu_mask(dn, HARDNESS) * OPACITY
    };
    println!("  pixel      | dnA   dnB   |  OFF |  ON  | lei de tinta | erro do ON");
    let mut worst = 0i32;
    for &(x, y) in &[
        (32u32, 32u32),
        (32, 34),
        (32, 36),
        (30, 36),
        (34, 36),
        (32, 38),
        (28, 32),
        (36, 32),
    ] {
        let p = (x as f32 + 0.5, y as f32 + 0.5);
        let (ma, mb) = (passage_alpha(p, (0, 1)), passage_alpha(p, (2, 3)));
        let ink = 1.0 - (1.0 - ma) * (1.0 - mb);
        let want = (ink * 255.0).round() as i32;
        let got_on = i32::from(alpha_at(&on, x, y));
        let err = (got_on - want).abs();
        worst = worst.max(err);
        println!(
            "  ({x:2},{y:2})    | {:.2}  {:.2}  | {:4} | {:4} | {want:12} | {:+}",
            capsule_dn(p, spine[0], spine[1], R, R),
            capsule_dn(p, spine[2], spine[3], R, R),
            alpha_at(&off, x, y),
            got_on,
            got_on - want,
        );
    }
    println!("  pior erro do modelo de hoje contra a lei de tinta: {worst}/255");
}
