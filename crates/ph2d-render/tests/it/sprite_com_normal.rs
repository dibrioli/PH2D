//! **P14 — o Sprite do motor com normal map** (`docs/PixelArt/P14_plano.md` §4 «L», §5): uma segunda
//! textura com o MESMO recorte da cor (`SpriteRenderer::definir_normal`), lida pelo `fs_main_n` do
//! `sprite.wgsl`, levada aos eixos do mundo pela base da instância e entregue a `luz_receber_n`.
//! `F` e `M` são SINTÉTICOS (o receptor isolado do solver, o molde de `luz_com_normal_no_receptor`):
//! `M = (0,4·2π, 0)` é a luz de uma lâmpada à DIREITA; `(0, 0,4·2π)` a de uma lâmpada em CIMA.

use std::collections::BTreeSet;
use std::f64::consts::TAU;

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{
    Camera2d, LiftedInstances, LuzRecebida, RenderInstance, SpriteRenderer, TextureAtlas,
};

const W: u32 = 64;
const H: u32 = 32;
const ORIGEM: [f64; 2] = [-2.0, -1.0];
const TEXEL: f64 = 0.5;
const TAM: [u32; 2] = [8, 4];
const DE_FRENTE: f64 = 0.5;
const MOMENTO: f64 = 0.4;
/// A folha: duas células 4×4 lado a lado. Cor: as duas cinzentas (a da direita mais clara, para o
/// recorte se ver também na cor); normal: a da esquerda aponta `+x`, a da direita `−x`.
const CEL: u32 = 4;
const COR_A: [u8; 4] = [200, 200, 200, 255];
const COR_B: [u8; 4] = [220, 220, 220, 255];
const N_MAIS_X: [u8; 4] = [255, 128, 128, 255];
const N_MENOS_X: [u8; 4] = [0, 128, 128, 255];

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn folha(esq: [u8; 4], dir: [u8; 4]) -> Vec<u8> {
    let mut d = Vec::new();
    for _ in 0..CEL {
        for x in 0..2 * CEL {
            d.extend_from_slice(if x < CEL { &esq } else { &dir });
        }
    }
    d
}

fn campo(gpu: &GpuContext, valor: f64) -> wgpu::Texture {
    let mut d = Vec::new();
    for _ in 0..TAM[0] * TAM[1] {
        for c in [valor, valor, valor, 1.0] {
            d.extend_from_slice(&half::f16::from_f64(c).to_le_bytes());
        }
    }
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sprite normal campo"),
        size: wgpu::Extent3d {
            width: TAM[0],
            height: TAM[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        t.as_image_copy(),
        &d,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(TAM[0] * 8),
            rows_per_image: Some(TAM[1]),
        },
        t.size(),
    );
    t
}

fn ler(gpu: &GpuContext, t: &wgpu::Texture) -> Vec<u8> {
    let padded =
        (W * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sprite normal leitura"),
        size: u64::from(padded * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(H),
            },
        },
        t.size(),
    );
    gpu.queue.submit([enc.finish()]);
    let fatia = buf.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let m = fatia.get_mapped_range().expect("mapped range");
    (0..H as usize)
        .flat_map(|y| m[y * padded as usize..][..(W * 4) as usize].to_vec())
        .collect()
}

/// Um quadro.
#[derive(Clone, Copy)]
struct Caso {
    /// A normal da folha (`None` = o sprite de hoje, sem normal).
    normal: Option<([u8; 4], [u8; 4])>,
    /// A célula desenhada (0 = esquerda, 1 = direita).
    celula: u32,
    /// A base 2×2 da instância (coluna-maior) e os bits de espelho.
    basis: [f32; 4],
    flip: u32,
    /// O momento `[Mx, My]/2π` da luz (`None` = luz sem momento).
    momento: Option<[f64; 2]>,
    /// Regista a normal de OUTRA textura (a do sprite fica sem): as pipelines nascem, este não as usa.
    normal_alheia: bool,
}

const HOJE: Caso = Caso {
    normal: None,
    celula: 0,
    basis: RenderInstance::IDENTITY_BASIS,
    flip: 0,
    momento: Some([MOMENTO, 0.0]),
    normal_alheia: false,
};

/// Devolve os bytes RGBA do alvo e quantas normais o renderer tem alocadas.
fn quadro(gpu: &GpuContext, c: Caso) -> (Vec<u8>, usize) {
    let janela = WindowSize::new(W, H);
    let camera = Camera2d::new([0.0, 0.0], 2.0);
    let f = campo(gpu, DE_FRENTE * TAU);
    let m = c.momento.unwrap_or([0.0, 0.0]);
    let mx = campo(gpu, m[0] * TAU);
    let my = campo(gpu, m[1] * TAU);
    let (fv, mxv, myv) = (
        f.create_view(&Default::default()),
        mx.create_view(&Default::default()),
        my.create_view(&Default::default()),
    );
    let ninguem = BTreeSet::new();
    let luz = LuzRecebida {
        fluencia: &fv,
        origem_m: ORIGEM,
        texel_m: TEXEL,
        tamanho: TAM,
        sem_luz: &ninguem,
        momento: c.momento.map(|_| [&mxv, &myv]),
    };
    let alvo = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sprite normal alvo"),
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
    });
    let mut renderer = SpriteRenderer::new(
        gpu.clone(),
        wgpu::TextureFormat::Rgba8Unorm,
        TextureAtlas::new(gpu, 256),
        8,
    );
    let tex = renderer
        .acquire_individual(2 * CEL, CEL, &folha(COR_A, COR_B))
        .expect("textura do sprite");
    if let Some((a, b)) = c.normal {
        renderer
            .definir_normal(tex, Some((2 * CEL, CEL, &folha(a, b))))
            .expect("normal da folha");
    }
    if c.normal_alheia {
        let outra = renderer
            .acquire_individual(2 * CEL, CEL, &folha(COR_A, COR_B))
            .expect("outra textura");
        renderer
            .definir_normal(outra, Some((2 * CEL, CEL, &folha(N_MAIS_X, N_MAIS_X))))
            .expect("normal alheia");
    }
    let u0 = 0.5 * c.celula as f32;
    let inst = RenderInstance {
        world_pos: [0.0, 0.0],
        size: [1.0, 1.0],
        atlas_uv: [u0, 0.0, u0 + 0.5, 1.0],
        tint: [1.0; 4],
        basis: c.basis,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: c.flip,
        texture_id: tex,
        z_order: 0,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    let mut extra = LiftedInstances::default();
    extra.push(inst, None);
    renderer.receber_luz(Some(&luz));
    renderer.render_with_streams(
        &alvo.create_view(&Default::default()),
        &mut PresentWorld::new(),
        &camera,
        janela,
        wgpu::Color::TRANSPARENT,
        &extra,
        None,
        None,
        None,
        None,
    );
    let n = renderer.normais_alocadas();
    (ler(gpu, &alvo), n)
}

/// O verde médio do miolo do sprite (o quad cobre x 24..40, y 8..24; a borda de 2 px fica fora).
fn miolo(bytes: &[u8]) -> f64 {
    let mut s = 0.0;
    let mut n = 0.0;
    for y in 10..22 {
        for x in 26..38 {
            s += f64::from(bytes[(y * W as usize + x) * 4 + 1]);
            n += 1.0;
        }
    }
    s / n
}

fn com(normal: ([u8; 4], [u8; 4]), celula: u32) -> Caso {
    Caso {
        normal: Some(normal),
        celula,
        ..HOJE
    }
}

#[test]
fn o_sprite_com_normal_acende_do_lado_da_lampada() {
    let Some(gpu) = gpu() else {
        eprintln!("sprite normal: sem placa — saltado");
        return;
    };
    let plano = miolo(&quadro(&gpu, HOJE).0);
    let claro = miolo(&quadro(&gpu, com((N_MAIS_X, N_MAIS_X), 0)).0);
    let escuro = miolo(&quadro(&gpu, com((N_MENOS_X, N_MENOS_X), 0)).0);
    let razao = claro / escuro.max(1.0);
    eprintln!(
        "sprite normal: virado à lâmpada {claro:.1} · de frente {plano:.1} · de costas {escuro:.1} · razão {razao:.2}"
    );
    assert!(
        claro > plano + 10.0 && plano > escuro + 10.0 && razao > 2.0,
        "a luz não seguiu a normal do sprite: {claro:.1} / {plano:.1} / {escuro:.1}"
    );
}

#[test]
fn a_normal_do_sprite_roda_com_a_instancia() {
    let Some(gpu) = gpu() else {
        eprintln!("sprite normal: sem placa — saltado");
        return;
    };
    let luz_de_cima = Some([0.0, MOMENTO]);
    let mais_x = com((N_MAIS_X, N_MAIS_X), 0);
    let de_frente_cima = miolo(
        &quadro(
            &gpu,
            Caso {
                momento: luz_de_cima,
                ..HOJE
            },
        )
        .0,
    );
    // Girado 90° (x → y): a normal +x pintada passa a apontar para CIMA, onde está a lâmpada.
    let girado = miolo(
        &quadro(
            &gpu,
            Caso {
                basis: [0.0, 1.0, -1.0, 0.0],
                momento: luz_de_cima,
                ..mais_x
            },
        )
        .0,
    );
    // Girado −90°: aponta para BAIXO, de costas à lâmpada.
    let girado_ao_contrario = miolo(
        &quadro(
            &gpu,
            Caso {
                basis: [0.0, -1.0, 1.0, 0.0],
                momento: luz_de_cima,
                ..mais_x
            },
        )
        .0,
    );
    // Espelhado em x (o bit do flip): +x vira −x, de costas à lâmpada da direita.
    let espelhado = miolo(&quadro(&gpu, Caso { flip: 1, ..mais_x }).0);
    // Escala −1 em x na BASE (o espelho do Transform): o mesmo que o bit.
    let escala_neg = miolo(
        &quadro(
            &gpu,
            Caso {
                basis: [-1.0, 0.0, 0.0, 1.0],
                ..mais_x
            },
        )
        .0,
    );
    let de_frente = miolo(&quadro(&gpu, HOJE).0);
    eprintln!(
        "sprite normal roda: luz de cima — de frente {de_frente_cima:.1} · girado 90° {girado:.1} · −90° {girado_ao_contrario:.1}; \
         luz da direita — de frente {de_frente:.1} · flip x {espelhado:.1} · escala −1 {escala_neg:.1}"
    );
    assert!(
        girado > de_frente_cima + 10.0 && de_frente_cima > girado_ao_contrario + 10.0,
        "a normal não rodou com a instância: {girado:.1} / {de_frente_cima:.1} / {girado_ao_contrario:.1}"
    );
    assert!(
        espelhado + 10.0 < de_frente && escala_neg + 10.0 < de_frente,
        "o espelho não virou a normal: flip {espelhado:.1} · escala {escala_neg:.1} · de frente {de_frente:.1}"
    );
}

#[test]
fn sem_normal_o_sprite_e_o_de_hoje_byte_a_byte() {
    let Some(gpu) = gpu() else {
        eprintln!("sprite normal: sem placa — saltado");
        return;
    };
    let (hoje, alocadas) = quadro(&gpu, HOJE);
    assert_eq!(alocadas, 0, "sem normal à vista o renderer alocou normais");
    let (com_alheia, alheias) = quadro(
        &gpu,
        Caso {
            normal_alheia: true,
            ..HOJE
        },
    );
    assert_eq!(alheias, 1);
    assert_eq!(
        com_alheia, hoje,
        "a normal de OUTRA textura mudou este sprite"
    );
    // A conta de hoje, por píxel: o sRGB 200 descodificado vezes `F/2π` (sem momento, o `luz_em`).
    let (sem_momento, _) = quadro(
        &gpu,
        Caso {
            momento: None,
            ..HOJE
        },
    );
    assert_eq!(sem_momento, hoje, "o momento mudou um sprite sem normal");
    let esperado = (((200.0 / 255.0 + 0.055) / 1.055f64).powf(2.4) * DE_FRENTE * 255.0).round();
    let lido = f64::from(hoje[(16 * W as usize + 32) * 4 + 1]);
    eprintln!("sprite sem normal: centro {lido} · conta {esperado}");
    assert!(
        (lido - esperado).abs() <= 1.0,
        "o sprite de hoje mudou: {lido} × {esperado}"
    );
}

#[test]
fn a_folha_normal_tem_o_recorte_da_cor() {
    let Some(gpu) = gpu() else {
        eprintln!("sprite normal: sem placa — saltado");
        return;
    };
    let normal = (N_MAIS_X, N_MENOS_X);
    let c0 = miolo(&quadro(&gpu, com(normal, 0)).0);
    let c1 = miolo(&quadro(&gpu, com(normal, 1)).0);
    let p0 = miolo(&quadro(&gpu, HOJE).0);
    let p1 = miolo(&quadro(&gpu, Caso { celula: 1, ..HOJE }).0);
    eprintln!(
        "sprite recorte: célula 0 {c0:.1} (plana {p0:.1}) · célula 1 {c1:.1} (plana {p1:.1})"
    );
    // A célula 0 da cor com a célula 0 da normal (+x, virada à lâmpada); a 1 com a 1 (−x).
    assert!(
        c0 > p0 + 10.0,
        "a célula 0 não leu a normal dela: {c0:.1} × {p0:.1}"
    );
    assert!(
        c1 + 10.0 < p1,
        "a célula 1 não leu a normal dela: {c1:.1} × {p1:.1}"
    );
}

#[test]
fn a_normal_de_outro_tamanho_e_recusada() {
    let Some(gpu) = gpu() else {
        eprintln!("sprite normal: sem placa — saltado");
        return;
    };
    let mut r = SpriteRenderer::new(
        gpu.clone(),
        wgpu::TextureFormat::Rgba8Unorm,
        TextureAtlas::new(&gpu, 256),
        8,
    );
    let tex = r
        .acquire_individual(8, 4, &folha(COR_A, COR_B))
        .expect("tex");
    assert!(r.definir_normal(tex, Some((4, 4, &[128; 64]))).is_err());
    assert_eq!(r.normais_alocadas(), 0);
    r.definir_normal(tex, Some((8, 4, &folha(N_MAIS_X, N_MAIS_X))))
        .expect("mesmo tamanho");
    assert_eq!(r.normais_alocadas(), 1);
    r.definir_normal(tex, None).expect("tirar");
    assert_eq!(r.normais_alocadas(), 0);
}
