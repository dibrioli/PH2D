//! ⭐⭐⭐ **UMA SPRITE DESENHADA COMO MALHA É O QUAD, AO PIXEL, NA POSE DE REPOUSO — e sem costuras.**
//!
//! O primitivo `ph2d_render::SpriteMesh` põe a imagem presa ao esqueleto DENTRO do passe de sprites
//! (plano `docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md`). Em repouso a malha cobre o quad
//! e amostra os mesmos texels ⇒ o oráculo é o **quad da mesma instância**, na mesma GPU, no mesmo
//! alvo — nenhuma constante escrita à mão.
//!
//! ⚠️ **As duas perguntas são diferentes:** a malha de 2 triângulos prova a CONVERSÃO (âncora
//! deslocada, tinta, opacidade, espelhamento chegam iguais); a malha FINA em arte TRANSLÚCIDA prova a
//! PARTIÇÃO — o caminho do Vello (um recorte por triângulo) reprovava aqui com `10 580` px.

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, RenderInstance, SpriteMesh, SpriteRenderer, TextureAtlas};

const W: u32 = 64;
const H: u32 = 64;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance = GpuContext::default_instance();
            GpuContext::new(instance, None).ok()
        })
        .clone()
}

fn make_target(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sprite mesh gate target"),
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
    let padded =
        unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sprite mesh gate staging"),
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
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    rx.recv().expect("canal").expect("map");
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

/// Arte com estrutura e alfa uniforme `alfa`.
fn textura(lado: u32, alfa: u8) -> Vec<u8> {
    let mut px = Vec::with_capacity((lado * lado * 4) as usize);
    for y in 0..lado {
        for x in 0..lado {
            let r = u8::try_from(x * 255 / lado).unwrap_or(255);
            let g = u8::try_from(y * 255 / lado).unwrap_or(255);
            px.extend_from_slice(&[r, g, 180, alfa]);
        }
    }
    px
}

/// Uma instância com tudo o que a malha tem de herdar: âncora DESLOCADA, tinta, opacidade, espelho.
fn instancia(texture_id: u32) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: [3.0, 3.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [0.9, 0.8, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.25, -0.25],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 0.75,
        flip_uv: RenderInstance::FLIP_X_BIT,
        texture_id,
        z_order: 0,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: 0,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// A malha de REPOUSO de `inst` em `cols × rows` células de dois triângulos. `quad_pos = (u − ½,
/// ½ − v)` é a convenção do `QUAD_STRIP` (o vértice de `y = +½` amostra `v = 0`).
fn grelha(inst: &RenderInstance, cols: u32, rows: u32) -> SpriteMesh {
    let mut m = SpriteMesh::default();
    for j in 0..=rows {
        for i in 0..=cols {
            #[expect(clippy::cast_precision_loss, reason = "grelhas pequenas de fixtura")]
            let (u, v) = (i as f32 / cols as f32, j as f32 / rows as f32);
            m.local.push([
                inst.anchor[0] + (u - 0.5) * inst.size[0],
                inst.anchor[1] + (0.5 - v) * inst.size[1],
            ]);
            m.uv.push([u, v]);
        }
    }
    let id = |i: u32, j: u32| j * (cols + 1) + i;
    for j in 0..rows {
        for i in 0..cols {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}

fn desenha(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    inst: RenderInstance,
    malha: Option<SpriteMesh>,
) -> Vec<u8> {
    let target = make_target(gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut present = PresentWorld::new();
    match malha {
        Some(m) => present.world_mut().spawn((inst, m)),
        None => present.world_mut().spawn(inst),
    };
    renderer.render(
        &view,
        &mut present,
        &Camera2d::new([0.0, 0.0], 4.0),
        WindowSize::new(W, H),
        wgpu::Color::BLACK,
    );
    readback(gpu, &target)
}

/// `(pixels com algum canal a diferir mais de 1, pior diferença)`.
fn diferenca(a: &[u8], b: &[u8]) -> (usize, u8) {
    let (mut n, mut pior) = (0, 0_u8);
    for (pa, pb) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        let d = (0..4).map(|c| pa[c].abs_diff(pb[c])).max().unwrap_or(0);
        if d > 1 {
            n += 1;
        }
        pior = pior.max(d);
    }
    (n, pior)
}

/// Controlo de toda a comparação: a sprite tem de ter pintado alguma coisa, senão «igual» é vácuo.
fn pintou(px: &[u8]) -> usize {
    px.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 8 || p[1] > 8 || p[2] > 8)
        .count()
}

/// `(mínimo, mediana)` do quadro em ms, com o trabalho da GPU **esperado** — um `submit` volta antes
/// de a placa ter feito coisa alguma. Inclui o que o quadro paga por uma malha: a recolha, a costura
/// da tira, o envio dos vértices e o desenho.
///
/// ⚠️⚠️ **O MÍNIMO é a leitura que sobrevive a esta workstation:** a carga de FUNDO (o editor, o
/// rust-analyzer, o sccache, as outras sessões) fica em `~7` sem ninguém compilar, e o `CLAUDE.md`
/// §5.0 diz que acima de `~5` nenhum relógio daqui vale nada. O mínimo é o custo quando o
/// escalonador deu o núcleo; a mediana ao lado diz quanto a máquina estava a roubar.
fn quadro_ms(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    inst: RenderInstance,
    malha: Option<&SpriteMesh>,
    alvo: &wgpu::TextureView,
    rondas: u32,
) -> (f64, f64) {
    let mut present = PresentWorld::new();
    match malha {
        Some(m) => present.world_mut().spawn((inst, m.clone())),
        None => present.world_mut().spawn(inst),
    };
    let cam = Camera2d::new([0.0, 0.0], 4.0);
    let janela = WindowSize::new(W, H);
    let mut desenhar = |renderer: &mut SpriteRenderer| {
        renderer.render(alvo, &mut present, &cam, janela, wgpu::Color::BLACK);
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
    };
    desenhar(renderer);
    let mut ms = Vec::with_capacity(rondas as usize);
    for _ in 0..rondas {
        let t = std::time::Instant::now();
        desenhar(renderer);
        ms.push(t.elapsed().as_secs_f64() * 1e3);
    }
    ms.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN no relogio"));
    (ms[0], ms[ms.len() / 2])
}

/// ⏱️ **SONDA (`--ignored`) — O QUE UMA MALHA CUSTA POR QUADRO** (plano `docs/Skeleton/03`, W4).
///
/// O orçamento de peças da pele (`ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`) foi derivado
/// do buffer do Vello, que este caminho já não gasta. Do lado do desenho o recurso que sobra é o
/// TEMPO do quadro, e é o que esta sonda mede: o mesmo instante desenhado como quad e como malha de
/// `N` triângulos, com a GPU esperada.
///
/// ⚠️ **O alvo é `64×64`**: o que se mede é o custo por VÉRTICE mais o do quadro (recolher, costurar
/// `5N − 2` vértices, enviar, desenhar) — o preenchimento é o mesmo nas duas colunas, porque a área
/// coberta é a mesma.
///
/// ⚠️ **Acima de `load ~5` uma leitura de relógio desta workstation não vale nada** (`CLAUDE.md`
/// §5.0) — a carga é impressa ao lado. Corre com:
/// `cargo test -p ph2d-render --test it -- --ignored --nocapture measure_the_frame_cost`
#[test]
#[ignore = "sonda de GPU: imprime a tabela do custo, sem barra"]
fn measure_the_frame_cost_of_a_mesh_sprite() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem GPU headless — nada medido");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let tex = renderer
        .acquire_individual(16, 16, &textura(16, 255))
        .expect("textura");
    let inst = instancia(tex);
    let alvo = make_target(&gpu);
    let view = alvo.create_view(&wgpu::TextureViewDescriptor::default());
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "carga: {}",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );
    const RONDAS: u32 = 60;
    let quad = quadro_ms(&gpu, &mut renderer, inst, None, &view, RONDAS);
    println!("ms: MINIMO/mediana de {RONDAS} corridas (a media sob contencao mede o vizinho)");
    println!("{:>8} | {:>15} | {:>9}", "pecas", "quadro", "vs quad");
    println!(
        "{:>8} | {:>6.3}/{:<8.3} | {:>9}",
        "quad", quad.0, quad.1, "—"
    );
    for (cols, rows) in [
        (1_u32, 1_u32),
        (8, 8),
        (24, 24),
        (48, 48),
        (72, 72),
        (96, 96),
    ] {
        let m = grelha(&inst, cols, rows);
        let pecas = m.tris.len();
        let ms = quadro_ms(&gpu, &mut renderer, inst, Some(&m), &view, RONDAS);
        println!(
            "{pecas:>8} | {:>6.3}/{:<8.3} | {:>8.2}x",
            ms.0,
            ms.1,
            ms.0 / quad.0.max(f64::MIN_POSITIVE)
        );
    }
}

/// ⭐⭐ **Uma malha de 2 triângulos em repouso É o quad**, com âncora deslocada, tinta, opacidade e
/// espelhamento — a conversão `local → quad_pos` e a herança das propriedades.
#[test]
#[ignore = "gate de GPU: precisa de adaptador"]
fn a_two_triangle_mesh_at_rest_is_the_quad_with_tint_opacity_and_flip() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem GPU headless — nada medido");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let tex = renderer
        .acquire_individual(16, 16, &textura(16, 255))
        .expect("textura");
    let inst = instancia(tex);
    let quad = desenha(&gpu, &mut renderer, inst, None);
    let malha = desenha(&gpu, &mut renderer, inst, Some(grelha(&inst, 1, 1)));
    assert!(
        pintou(&quad) > 1_000,
        "o quad nao pintou nada — a comparacao seria vacua"
    );
    let (n, pior) = diferenca(&quad, &malha);
    assert_eq!(
        (n, pior > 1),
        (0, false),
        "a malha de 2 triangulos em repouso nao e' o quad: {n} px diferem, pior {pior}"
    );
}

/// ⭐⭐⭐ **Arte TRANSLÚCIDA numa malha FINA em repouso É o quad — nenhuma costura.**
///
/// ⛔ O caminho do Vello (um recorte por triângulo) compunha `1 − a·b` em cada aresta partilhada:
/// `10 580` px com o alfa errado a zoom 4 com `216` peças. Aqui cada centro de pixel é de UM
/// triângulo.
#[test]
#[ignore = "gate de GPU: precisa de adaptador"]
fn a_translucent_image_in_a_fine_mesh_at_rest_has_no_seams() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem GPU headless — nada medido");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let tex = renderer
        .acquire_individual(16, 16, &textura(16, 128))
        .expect("textura");
    let inst = instancia(tex);
    let quad = desenha(&gpu, &mut renderer, inst, None);
    let malha = desenha(&gpu, &mut renderer, inst, Some(grelha(&inst, 8, 8)));
    assert!(
        pintou(&quad) > 1_000,
        "o quad nao pintou nada — a comparacao seria vacua"
    );
    let (n, pior) = diferenca(&quad, &malha);
    assert_eq!(
        n, 0,
        "a malha fina em repouso tem {n} px diferentes do quad (pior {pior}) — costura ou \
         composicao dupla"
    );
}

/// A pele de DUAS peles sobre `m`: o osso `0` parado e o `1` rodado de `rad` em torno de `junta`,
/// com o peso a subir linearmente em `x`. ⇒ o `θ̄` varia por vértice e a arte TORCE.
///
/// ⚠️ **O `junta` entra na tabela `n × n` em ordem de linha** — com `2` ossos ela tem `4` entradas e
/// só a `(0,1)` é lida; escrevê-la inteira é o que impede um índice trocado de passar por acaso.
fn com_pele(m: &mut SpriteMesh, rad: f32, junta: [f32; 2]) {
    let (co, si) = (rad.cos(), rad.sin());
    let (jx, jy) = (junta[0], junta[1]);
    let rodado = [
        co,
        si,
        -si,
        co,
        jx - (co * jx - si * jy),
        jy - (si * jx + co * jy),
    ];
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for p in &m.local {
        lo = lo.min(p[0]);
        hi = hi.max(p[0]);
    }
    let (pesos, ossos) = m
        .local
        .iter()
        .map(|p| {
            let u = ((p[0] - lo) / (hi - lo)).clamp(0.0, 1.0);
            ([1.0 - u, u, 0.0, 0.0], [0_u32, 1, 0, 0])
        })
        .collect::<(Vec<_>, Vec<_>)>();
    m.skin = Some(ph2d_render::SpriteMeshSkin {
        pesos,
        ossos,
        afins: vec![[1.0, 0.0, 0.0, 1.0, 0.0, 0.0], rodado],
        juntas: vec![junta; 4],
        angulos: vec![[1.0, 0.0], [co, si]],
    });
}

/// ⭐⭐⭐ **A PLACA DESENHA O QUE A CPU POSA** — o gate que É a metade do dispositivo da F9 W2.
///
/// Duas imagens do MESMO sujeito: a malha de REPOUSO com a pele (quem posa é o `vs_main`) e a malha
/// **já posada** pela [`ph2d_render::SpriteMeshSkin::posa`] sem pele nenhuma. *Dois motores, uma
/// lei* — e aqui a régua é o PIXEL, não a aritmética.
///
/// ⛔⛔ **A lei NÃO é uma mistura linear de afins** e a primeira redacção deste caminho implementou
/// a antiga: o portão de paridade do payload leu `2,315e-3 m`. A lei do produto roda em torno da
/// JUNTA desde 2026-09-19 (`ph2d_skeleton::centro`), e este gate é o que a prende ao dispositivo.
///
/// ⚠️⚠️ **O `size` é NÃO-QUADRADO de propósito, e há asserção a exigi-lo:** conjugar uma rotação
/// por um `size` com `sx = sy` devolve a própria rotação, logo os dois factores `sx/sy` e `sy/sx`
/// do shader ficariam **inertes** e um erro neles passaria — *um corpus no ponto neutro de um knob
/// não testa esse knob*. A `anchor` também não é a origem, pela mesma razão: ela é o que a
/// conjugação do PONTO (a junta) usa.
///
/// ⚠️ **A barra é a BORDA e não zero:** as duas geometrias diferem por ULPs, e um pixel de borda
/// muda de cobertura com isso. O que se afirma é que a discordância vive num fio de pixels —
/// medida contra o CONTROLO, que é a mesma malha **sem** posar.
#[test]
#[ignore = "gate de GPU: precisa de adaptador"]
fn a_placa_desenha_o_que_a_cpu_posa() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem GPU headless — gate saltado");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut r = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 16);
    let tex = r
        .acquire_individual(8, 8, &textura(8, 255))
        .expect("textura");
    let mut inst = instancia(tex);
    // ⚠️ Ver o doc: quadrado tornaria as duas razões do shader INERTES.
    inst.size = [3.0, 1.5];
    assert!(
        (inst.size[0] - inst.size[1]).abs() > 0.1 && inst.anchor != [0.0, 0.0],
        "a fixtura deixou de exercitar a conjugacao (size {:?}, anchor {:?})",
        inst.size,
        inst.anchor
    );

    let repouso = grelha(&inst, 16, 16);
    let mut com = repouso.clone();
    com_pele(&mut com, 0.6, inst.anchor);
    let posada = SpriteMesh {
        local: com.posado().local.clone(),
        uv: com.uv.clone(),
        tris: com.tris.clone(),
        skin: None,
    };

    let placa = desenha(&gpu, &mut r, inst, Some(com));
    let cpu = desenha(&gpu, &mut r, inst, Some(posada));
    let parado = desenha(&gpu, &mut r, inst, Some(repouso));

    let pintados = pintou(&cpu);
    assert!(pintados > 400, "a sprite mal pintou ({pintados} px)");
    let (dif, pior) = diferenca(&placa, &cpu);
    let (controlo, _) = diferenca(&parado, &cpu);
    println!(
        "  pintados {pintados} · placa×cpu {dif} px (pior canal {pior}) · controlo {controlo} px"
    );
    // ⛔ O CONTROLO primeiro: sem dobra a fixtura não afirma nada.
    assert!(
        controlo > pintados / 4,
        "a fixtura nao DOBRA: a malha parada difere da posada em so' {controlo} px de {pintados}"
    );
    assert!(
        dif * 8 < controlo,
        "a placa e a CPU discordam em {dif} px contra {controlo} do controlo — a lei do shader nao \
         e' a lei da CPU, e a arte desenha num sitio enquanto o ponteiro aponta noutro"
    );
}
