//! **O RECORTE DE UMA SPRITE NÃO LÊ O VIZINHO** ([doc](../../../../docs/Sprite_projeto/22_o_recorte_nao_le_o_vizinho.md)).
//!
//! A folha 3×3 tem a célula do MEIO transparente e as outras opacas. Desenhar SÓ a do meio não pode pintar
//! um único pixel: o alfa do alvo é a régua, em todo zoom, posição fraccionária, rotação, espelho e filtro.

use ph2d_ecs::{SliceDrawMode, SliceNine};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::nine_slice::nine_slice_patches;
use ph2d_render::sprite::{region_subrect, sprite_sheet_subrect};
use ph2d_render::{
    Camera2d, GameRt, LiftedInstances, RenderInstance, SpriteRenderer, TextureAtlas,
};

const C: u32 = 64;
const FOLHA: u32 = C * 3;
const ALVO: u32 = 400;
/// Meio degrau de 8 bits: o que o ecrã do dono não consegue mostrar.
const LIMIAR: f32 = 1.0 / 510.0;
/// Pixels do ecrã por texel: ampliado, 1:1, fraccionário e os níveis de mip.
const ZOOMS: [f32; 7] = [4.0, 1.37, 1.0, 0.75, 0.5, 0.25, 0.1];
/// 0 Inherit · 1 Nearest · 2 Linear · 3 NearestMipmap · 4 LinearMipmap · 6 LinearAniso.
const FILTROS: [u8; 6] = [0, 1, 2, 3, 4, 6];

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn alfas(gpu: &GpuContext, t: &wgpu::Texture) -> Vec<[f32; 4]> {
    let (w, h) = (t.width(), t.height());
    let passo =
        (w * 8).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("risco leitura"),
        size: u64::from(passo * h),
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
                bytes_per_row: Some(passo),
                rows_per_image: Some(h),
            },
        },
        t.size(),
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let d = buf.slice(..).get_mapped_range().expect("mapped range");
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            let o = (y * passo + x * 8) as usize;
            let c = |k: usize| half::f16::from_le_bytes([d[o + 2 * k], d[o + 2 * k + 1]]).to_f32();
            [c(0), c(1), c(2), c(3)]
        })
        .collect()
}

fn folha() -> Vec<u8> {
    let mut px = Vec::with_capacity((FOLHA * FOLHA * 4) as usize);
    for y in 0..FOLHA {
        for x in 0..FOLHA {
            let meio = x / C == 1 && y / C == 1;
            px.extend_from_slice(&[255, 0, 0, if meio { 0 } else { 255 }]);
        }
    }
    px
}

fn folha_16() -> Vec<u16> {
    folha()
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| {
            [p[0], p[1], p[2], p[3]].map(|c| half::f16::from_f32(f32::from(c) / 255.0).to_bits())
        })
        .collect()
}

fn instancia(texture_id: u32, rect: [f32; 4], size: [f32; 2], filtro: u8) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size,
        atlas_uv: rect,
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id,
        z_order: 0,
        sampling: RenderInstance::pack_sampling(filtro, 0),
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

#[derive(Clone, Copy)]
struct Pose {
    zoom: f32,
    deslocado: bool,
    rodado: bool,
    espelho: bool,
    esmagado: bool,
    filtro: u8,
}

fn poses() -> Vec<Pose> {
    let mut v = Vec::new();
    for zoom in ZOOMS {
        for filtro in FILTROS {
            for (deslocado, rodado, espelho, esmagado) in [
                (false, false, false, false),
                (true, false, false, false),
                (true, true, false, false),
                (true, false, true, false),
                (true, false, false, true),
                (true, true, false, true),
            ] {
                v.push(Pose {
                    zoom,
                    deslocado,
                    rodado,
                    espelho,
                    esmagado,
                    filtro,
                });
            }
        }
    }
    v
}

fn posar(mut i: RenderInstance, p: Pose) -> RenderInstance {
    if p.deslocado {
        i.world_pos = [0.37, 0.61];
    }
    if p.rodado {
        let (s, c) = 30.0_f32.to_radians().sin_cos();
        i.basis = [c, s, -s, c];
    }
    if p.esmagado {
        // A anisotropia lê ao longo do eixo comprimido.
        i.basis = [i.basis[0] * 0.3, i.basis[1] * 0.3, i.basis[2], i.basis[3]];
    }
    if p.espelho {
        i.flip_uv |= RenderInstance::pack_flip_flags(true, true, false);
    }
    i.sampling = RenderInstance::pack_sampling(p.filtro, 0);
    i
}

/// O maior alfa que `quads` pintam em `pose`, sobre fundo transparente.
fn maior_alfa(
    gpu: &GpuContext,
    r: &mut SpriteRenderer,
    alvo: &GameRt,
    quads: &[RenderInstance],
    p: Pose,
) -> f32 {
    let mut lifted = LiftedInstances::default();
    for q in quads {
        let mut q = posar(*q, p);
        q.size = [q.size[0] * p.zoom, q.size[1] * p.zoom];
        q.anchor = [q.anchor[0] * p.zoom, q.anchor[1] * p.zoom];
        lifted.push(q, None);
    }
    r.render_lifted_instances(
        alvo.view(),
        &Camera2d::new([0.0, 0.0], ALVO as f32),
        WindowSize::new(ALVO, ALVO),
        wgpu::Color::TRANSPARENT,
        &lifted,
        None,
        false,
    );
    alfas(gpu, alvo.texture())
        .iter()
        .fold(0.0_f32, |m, c| m.max(c[3]))
}

struct Fonte {
    nome: &'static str,
    id: u32,
    quads: Vec<RenderInstance>,
}

/// As fontes da matriz: a mesma célula transparente por cada caminho que o app tem até ela.
fn fontes(gpu: &GpuContext) -> (SpriteRenderer, Vec<Fonte>) {
    let mut atlas = TextureAtlas::new(gpu, 2048);
    atlas.insert(gpu, 1, FOLHA, FOLHA, &folha()).expect("cabe");
    // Nove regiões coladas pelo empacotador; a do meio, transparente, é a imagem inteira de um «Still».
    for k in 0..9_u32 {
        let px = if k == 4 {
            [255, 0, 0, 0]
        } else {
            [255, 0, 0, 255]
        };
        atlas
            .insert(gpu, 10 + k, C, C, &px.repeat((C * C) as usize))
            .expect("cabe");
    }
    let mut r = SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, atlas, 8);
    let ind = r
        .acquire_individual(FOLHA, FOLHA, &folha())
        .expect("8 bits");
    let ind16 = r
        .acquire_individual_16(FOLHA, FOLHA, &folha_16())
        .expect("16 bits");
    let inteiro = [0.0, 0.0, 1.0, 1.0];
    let cel = |uv| sprite_sheet_subrect(uv, 3, 3, 4);
    let c = C as f32;
    let f = FOLHA as f32;
    let reg = |uv, ht| region_subrect(uv, [c, c, c, c], f, f, ht);
    let atlas_id = RenderInstance::ATLAS_TEXTURE_ID;
    let um = |id, rect| vec![instancia(id, rect, [c, c], 0)];
    let ruv = r.atlas().region_uv_exact(1); // the EXACT rect, as the extract cuts it (09/11)
    let meia = 0.5 / r.atlas().size_px as f32;
    let nove = SliceNine {
        draw_mode: SliceDrawMode::Sliced,
        borders: [16.0; 4],
        fill_center: true,
        ..SliceNine::INERT
    };
    let patches = nine_slice_patches(
        cel(inteiro),
        [c, c],
        &nove,
        [c * 2.0, c * 2.0],
        1.0,
        [1.0, 1.0],
    );
    let nove_quads: Vec<_> = patches
        .iter()
        .flatten()
        .map(|p| {
            let mut i = instancia(ind, p.uv, p.size, 0);
            i.anchor = p.center_offset;
            i
        })
        .collect();
    assert!(!nove_quads.is_empty(), "o 9-slice tem de dar quads");
    let npot: Vec<u8> = (0..183_u32 * 183)
        .flat_map(|i| {
            let (x, y) = (i % 183, i / 183);
            [255, 0, 0, if x / 61 == 1 && y / 61 == 1 { 0 } else { 255 }]
        })
        .collect();
    let ind_npot = r.acquire_individual(183, 183, &npot).expect("NPOT");
    let mini: Vec<u8> = (0..24_u32 * 24)
        .flat_map(|i| {
            let (x, y) = (i % 24, i / 24);
            [255, 0, 0, if x / 8 == 1 && y / 8 == 1 { 0 } else { 255 }]
        })
        .collect();
    let ind_mini = r.acquire_individual(24, 24, &mini).expect("mini");
    let v = vec![
        Fonte {
            nome: "Individual · célula de 8 px (minificada além dos mips limpos)",
            id: ind_mini,
            quads: um(ind_mini, cel(inteiro)),
        },
        Fonte {
            nome: "Individual · célula de folha NPOT (183 px)",
            id: ind_npot,
            quads: um(ind_npot, cel(inteiro)),
        },
        Fonte {
            nome: "Individual · Animated/Still de folha",
            id: ind,
            quads: um(ind, cel(inteiro)),
        },
        Fonte {
            nome: "Individual 16 bits · célula",
            id: ind16,
            quads: um(ind16, cel(inteiro)),
        },
        Fonte {
            nome: "Individual · Region/Hand-packed (sem filter_clip)",
            id: ind,
            quads: um(ind, reg(inteiro, None)),
        },
        Fonte {
            nome: "Individual · 9-slice da célula",
            id: ind,
            quads: nove_quads,
        },
        Fonte {
            nome: "Atlas · célula de folha",
            id: atlas_id,
            quads: um(atlas_id, cel(ruv)),
        },
        Fonte {
            nome: "Atlas · Region com filter_clip",
            id: atlas_id,
            quads: um(atlas_id, reg(ruv, Some((meia, meia)))),
        },
        Fonte {
            nome: "Atlas · imagem inteira colada às vizinhas",
            id: atlas_id,
            quads: um(atlas_id, r.atlas().region_uv_exact(14)),
        },
    ];
    (r, v)
}

/// ⭐ A matriz inteira. A contraprova é desfazer o `recorte_uv` do `sprite.wgsl`: esta mesma tabela enche-se.
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_celula_transparente_nao_pinta_um_pixel() {
    let Some(gpu) = gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    println!("placa: {}", gpu.adapter.get_info().name);
    let (mut r, fontes) = fontes(&gpu);
    let alvo = GameRt::new(&gpu, (ALVO, ALVO));
    let mut falhas = Vec::new();
    for f in &fontes {
        println!("── {} (textura {})", f.nome, f.id);
        for zoom in ZOOMS {
            let mut por_filtro = [0.0_f32; 7];
            for p in poses().into_iter().filter(|p| p.zoom == zoom) {
                let a = maior_alfa(&gpu, &mut r, &alvo, &f.quads, p);
                por_filtro[usize::from(p.filtro)] = por_filtro[usize::from(p.filtro)].max(a);
                if a > LIMIAR {
                    falhas.push((f.nome, zoom, p.filtro, p.deslocado, p.rodado, p.esmagado, a));
                }
            }
            let colunas = FILTROS.map(|t| format!("{t}:{:.3}", por_filtro[usize::from(t)]));
            println!("   zoom {zoom:>5}: {}", colunas.join(" "));
        }
    }
    println!("{} poses com risco", falhas.len());
    assert!(
        falhas.is_empty(),
        "{} poses com risco, ex.: {:?}",
        falhas.len(),
        &falhas[..falhas.len().min(6)]
    );
}

/// Uma folha 3×3 de células `lado`, com `celula(x, y)` a pintar a do canto (frame 0) e o resto transparente.
fn folha_com_canto(lado: u32, celula: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut px = Vec::with_capacity((lado * lado * 36) as usize);
    for y in 0..lado * 3 {
        for x in 0..lado * 3 {
            px.extend_from_slice(&if x < lado && y < lado {
                celula(x, y)
            } else {
                [0, 0, 0, 0]
            });
        }
    }
    px
}

/// ⭐ O avesso do risco: uma célula OPACA ao lado de vizinhas transparentes tem de chegar opaca até à última
/// fileira do quad — nem furo (a vizinha a entrar) nem recorte (a cura a comer a arte).
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_celula_opaca_chega_opaca_ate_a_borda() {
    let Some(gpu) = gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 1024);
    let mut r = SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, atlas, 8);
    let px = folha_com_canto(C, |_, _| [0, 255, 0, 255]);
    let id = r.acquire_individual(FOLHA, FOLHA, &px).expect("textura");
    let rect = sprite_sheet_subrect([0.0, 0.0, 1.0, 1.0], 3, 3, 0);
    let alvo = GameRt::new(&gpu, (ALVO, ALVO));
    for zoom in [4.0_f32, 1.0, 0.5, 0.25] {
        for filtro in FILTROS {
            let lado = C as f32 * zoom;
            let i = instancia(id, rect, [lado, lado], filtro);
            let mut lifted = LiftedInstances::default();
            lifted.push(i, None);
            r.render_lifted_instances(
                alvo.view(),
                &Camera2d::new([0.0, 0.0], ALVO as f32),
                WindowSize::new(ALVO, ALVO),
                wgpu::Color::TRANSPARENT,
                &lifted,
                None,
                false,
            );
            let px = alfas(&gpu, alvo.texture());
            let meio = ALVO as f32 / 2.0;
            let (lo, hi) = ((meio - lado / 2.0) as u32, (meio + lado / 2.0) as u32);
            let furos = (lo..hi)
                .flat_map(|y| (lo..hi).map(move |x| (x, y)))
                .filter(|&(x, y)| px[(y * ALVO + x) as usize][3] < 0.99)
                .count();
            assert_eq!(
                furos, 0,
                "zoom {zoom}, filtro {filtro}: {furos} pixels furados no quad"
            );
        }
    }
}

/// ⭐ Pixel-art: com `Nearest`, ampliada ou não, a célula continua a ter só as duas cores do xadrez.
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_celula_em_nearest_nao_borra() {
    let Some(gpu) = gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    const LADO: u32 = 8;
    let atlas = TextureAtlas::new(&gpu, 1024);
    let mut r = SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, atlas, 8);
    let xadrez = folha_com_canto(LADO, |x, y| {
        if (x + y) % 2 == 0 {
            [255, 255, 255, 255]
        } else {
            [0, 0, 0, 255]
        }
    });
    let id = r
        .acquire_individual(LADO * 3, LADO * 3, &xadrez)
        .expect("textura");
    let rect = sprite_sheet_subrect([0.0, 0.0, 1.0, 1.0], 3, 3, 0);
    let alvo = GameRt::new(&gpu, (ALVO, ALVO));
    for zoom in [8.0_f32, 4.0, 1.0] {
        let lado = LADO as f32 * zoom;
        let mut lifted = LiftedInstances::default();
        lifted.push(instancia(id, rect, [lado, lado], 1), None);
        r.render_lifted_instances(
            alvo.view(),
            &Camera2d::new([0.0, 0.0], ALVO as f32),
            WindowSize::new(ALVO, ALVO),
            wgpu::Color::TRANSPARENT,
            &lifted,
            None,
            false,
        );
        let px = alfas(&gpu, alvo.texture());
        let meio = ALVO as f32 / 2.0;
        let lo = (meio - lado / 2.0) as u32;
        let mut pretos = 0;
        let mut brancos = 0;
        for y in lo..lo + lado as u32 {
            for x in lo..lo + lado as u32 {
                let c = px[(y * ALVO + x) as usize];
                assert!(c[3] > 0.999, "zoom {zoom}: ({x},{y}) não é opaco: {c:?}");
                assert!(
                    c[0] < 0.001 || c[0] > 0.999,
                    "zoom {zoom}: ({x},{y}) borrou: {c:?}"
                );
                if c[0] < 0.5 {
                    pretos += 1;
                } else {
                    brancos += 1;
                }
                // Em todo texel, a cor é a do xadrez: a geometria da célula não mudou.
                let (tx, ty) = (
                    ((x - lo) as f32 / zoom) as u32,
                    ((y - lo) as f32 / zoom) as u32,
                );
                assert_eq!(
                    c[0] > 0.5,
                    (tx + ty) % 2 == 0,
                    "zoom {zoom}: ({x},{y}) é de outro texel"
                );
            }
        }
        assert_eq!(pretos, brancos, "zoom {zoom}: o xadrez perdeu a proporção");
    }
}

/// ⭐ A repetida NÃO é recortada: a costura da imagem inteira continua a misturar a última linha com a primeira
/// (o sampler dá a volta). Com o recorte a passar por aqui, a costura seria um degrau duro de uma cor à outra.
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_repetida_continua_a_costurar() {
    let Some(gpu) = gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    const LADO: u32 = 8;
    let atlas = TextureAtlas::new(&gpu, 1024);
    let mut r = SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, atlas, 8);
    let mut px = Vec::new();
    for y in 0..LADO {
        for _ in 0..LADO {
            px.extend_from_slice(&if y < LADO / 2 {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            });
        }
    }
    let id = r.acquire_individual(LADO, LADO, &px).expect("textura");
    let alvo = GameRt::new(&gpu, (ALVO, ALVO));
    let zoom = 8.0_f32;
    let azulejos = 3.0_f32;
    let lado = LADO as f32 * zoom * azulejos;
    let mut i = instancia(id, [0.0, 0.0, 1.0, 1.0], [lado, lado], 2);
    i.flip_uv = RenderInstance::pack_repeat_bits(2);
    i.uv_xform = [azulejos, azulejos, 0.0, 0.0];
    i.sampling = RenderInstance::pack_sampling(2, 2);
    let mut lifted = LiftedInstances::default();
    lifted.push(i, None);
    r.render_lifted_instances(
        alvo.view(),
        &Camera2d::new([0.0, 0.0], ALVO as f32),
        WindowSize::new(ALVO, ALVO),
        wgpu::Color::TRANSPARENT,
        &lifted,
        None,
        false,
    );
    let px = alfas(&gpu, alvo.texture());
    // A volta do 1.º para o 2.º azulejo: o azul da última linha mistura-se com o vermelho da primeira.
    let topo = (ALVO as f32 - lado) / 2.0;
    let volta = (topo + lado / azulejos) as u32;
    let c = px[(volta * ALVO + ALVO / 2) as usize];
    let pintados = px.iter().filter(|c| c[3] > 0.0).count();
    assert!(
        c[0] > 0.1 && c[2] > 0.1,
        "a costura não mistura: {c:?} ({pintados} pixels pintados no alvo)"
    );
}

/// Controlo: a imagem INTEIRA parada é presa pelo sampler. A última linha, opaca, não aparece no topo do quad.
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_imagem_inteira_nao_le_a_borda_oposta() {
    let Some(gpu) = gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 1024);
    let mut r = SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, atlas, 8);
    let px: Vec<u8> = (0..C * C)
        .flat_map(|i| [255, 0, 0, if i / C == C - 1 { 255 } else { 0 }])
        .collect();
    let id = r.acquire_individual(C, C, &px).expect("textura");
    let alvo = GameRt::new(&gpu, (ALVO, ALVO));
    for filtro in FILTROS {
        let lado = C as f32 * 4.0;
        let mut lifted = LiftedInstances::default();
        lifted.push(
            instancia(id, [0.0, 0.0, 1.0, 1.0], [lado, lado], filtro),
            None,
        );
        r.render_lifted_instances(
            alvo.view(),
            &Camera2d::new([0.0, 0.0], ALVO as f32),
            WindowSize::new(ALVO, ALVO),
            wgpu::Color::TRANSPARENT,
            &lifted,
            None,
            false,
        );
        let px = alfas(&gpu, alvo.texture());
        let metade = (ALVO * ALVO / 2) as usize;
        let topo = px[..metade].iter().fold(0.0_f32, |m, c| m.max(c[3]));
        assert!(
            topo <= LIMIAR,
            "filtro {filtro}: alfa {topo} na metade de cima"
        );
        assert!(
            px[metade..].iter().any(|c| c[3] > 0.5),
            "filtro {filtro}: a linha opaca sumiu"
        );
    }
}

/// ⭐ A cura não come arte: com `Linear`, a meia fileira de fora do texel da borda é a cor DELE, sem misturar a
/// fileira de dentro (nem a de fora).
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_fileira_da_borda_mantem_a_cor_dela() {
    let Some(gpu) = gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    const LADO: u32 = 8;
    let atlas = TextureAtlas::new(&gpu, 1024);
    let mut r = SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, atlas, 8);
    let px = folha_com_canto(LADO, |_, y| {
        if y == 0 {
            [255, 0, 0, 255]
        } else {
            [0, 0, 255, 255]
        }
    });
    let id = r
        .acquire_individual(LADO * 3, LADO * 3, &px)
        .expect("textura");
    let rect = sprite_sheet_subrect([0.0, 0.0, 1.0, 1.0], 3, 3, 0);
    let alvo = GameRt::new(&gpu, (ALVO, ALVO));
    let zoom = 8.0_f32;
    let lado = LADO as f32 * zoom;
    let mut lifted = LiftedInstances::default();
    lifted.push(instancia(id, rect, [lado, lado], 2), None);
    r.render_lifted_instances(
        alvo.view(),
        &Camera2d::new([0.0, 0.0], ALVO as f32),
        WindowSize::new(ALVO, ALVO),
        wgpu::Color::TRANSPARENT,
        &lifted,
        None,
        false,
    );
    let px = alfas(&gpu, alvo.texture());
    let topo = ((ALVO as f32 - lado) / 2.0) as u32;
    for y in topo..topo + (zoom / 2.0) as u32 {
        let c = px[(y * ALVO + ALVO / 2) as usize];
        assert!(
            c[0] > 0.999 && c[2] < 0.001 && c[3] > 0.999,
            "linha {}: {c:?}",
            y - topo
        );
    }
}
