//! **Cada forma, na placa** (doc 129 §3.4 / §5.5 A1): a forma CERCADA recebe o MESMO efeito que
//! SOZINHA. A fixtura é a da sonda do dono — discos que se tocam, a sombra interna deslocada — e o
//! controlo é a camada INTEIRA, onde a união das formas É a lei e o miolo muda dezenas de níveis.
//!
//! `PH2D_GPU=1 cargo test -p ph2d-fx-cells --test it -- --ignored --nocapture`

use ph2d_fx_cells::{Celulas, Entrada, FORMATO, Formas, Sprites, Variante};
use ph2d_fx_op::FxOp;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{
    BandBlit, BandSource, Camera2d, FxOpGpu, FxStackPass, Origem, RenderInstance, SpriteRenderer,
    TextureAtlas, make_output_texture,
};
use ph2d_shape_gpu::{
    Copias, FillRule, ShapeGeometry, ShapeInput, ShapeInstance, ShapePass, ShapeView,
};
use ph2d_vector::{Circle, Shape};

const W: u32 = 400;
const H: u32 = 300;
/// Pixels por unidade do mundo: um disco de `0,7` tem 14 px de diâmetro (o raio 7 da sonda).
const K: f32 = 20.0;
const DIAMETRO: f32 = 0.7;

fn camera() -> Camera2d {
    Camera2d {
        height_world: H as f32 / K,
        ..Camera2d::default()
    }
}

/// As sprites de uma corrida: a fatia da CPU ou um buffer «da placa» (a rota do cozimento).
#[derive(Clone, Copy)]
enum Rota<'a> {
    Cpu(&'a [RenderInstance]),
    Placa(&'a [RenderInstance]),
}

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn vista() -> ShapeView {
    #[expect(clippy::cast_precision_loss, reason = "um alvo pequeno")]
    ShapeView {
        lin: [K, 0.0, 0.0, -K],
        t: [W as f32 * 0.5, H as f32 * 0.5],
        alvo: [W as f32, H as f32],
    }
}

fn disco(x: f32, y: f32) -> ShapeInstance {
    ShapeInstance {
        pos: [x, y],
        size: [DIAMETRO, DIAMETRO],
        basis: [1.0, 0.0, 0.0, 1.0],
        anchor: [0.0, 0.0],
        geometry: 7,
        _pad: 0,
        tint: [0.95, 0.85, 0.3, 1.0],
    }
}

fn passe(gpu: &GpuContext) -> ShapePass {
    let bp = Circle::new((0.0, 0.0), 0.5).to_path(0.01);
    let g = ShapeGeometry::prepare(&ShapeInput {
        fill: Some((&bp, FillRule::NonZero)),
        strokes: Vec::new(),
        stroke_fills: Vec::new(),
    })
    .expect("o disco prepara");
    let mut p = ShapePass::new(gpu, FORMATO);
    p.set_geometries(gpu, [(7u32, &g)]);
    p
}

fn textura(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMATO,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// As quatro variantes da rodada C.10 (doc 131 §7): a vala (dupla · justa — por lado desde o doc 132
/// §1) × saltar o alvo vazio.
const VARIANTES: [Variante; 4] = [
    Variante {
        vala_justa: false,
        salta_vazias: false,
        calada_vazia: true,
    },
    Variante {
        vala_justa: true,
        salta_vazias: false,
        calada_vazia: true,
    },
    Variante {
        vala_justa: false,
        salta_vazias: true,
        calada_vazia: true,
    },
    Variante {
        vala_justa: true,
        salta_vazias: true,
        calada_vazia: true,
    },
];

/// Lê uma textura `Rgba8Unorm` (a saída da pilha).
fn le(gpu: &GpuContext, tex: &wgpu::Texture) -> Vec<u8> {
    le_de(gpu, tex, W, H)
}

/// Lê uma textura `Rgba8Unorm` de `w×h`.
fn le_de(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let linha = (w * 4).next_multiple_of(256);
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(linha * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        tex.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(linha),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let fatia = buf.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let dados = fatia.get_mapped_range();
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let o = (y * linha) as usize;
        out.extend_from_slice(&dados[o..o + (w * 4) as usize]);
    }
    out
}

fn sombra_interna(contorno: bool) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::INNER_SHADOW,
        sigma_px: 5.3,
        offset_px: [6.0, 6.0],
        tint: [0.0, 0.0, 0.0, 1.0],
        tint_b: [1.0; 4],
        opacity: 1.0,
        mode: if contorno {
            FxOp::MODE_CONTOUR
        } else {
            FxOp::MODE_PROXIMITY
        },
        blend: 0,
        stops: [[0.0; 4]; 8],
        stop_pos: [[0.0; 4]; 2],
        stop_count: 0,
        noise_scale_px: 10.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
        ramp_lut: None,
    }
}

struct Banca {
    gpu: GpuContext,
    branco: [f32; 4],
    renderer: SpriteRenderer,
    celulas: Celulas,
    placa: ShapePass,
    pilha: FxStackPass,
    colagem: BandBlit,
    /// A rota da placa com UMA faixa explícita de amostragem (`Some(tag)`) em vez da partição
    /// vazia (P6, doc 132 §6).
    faixa: Option<u32>,
}

impl Banca {
    fn new(gpu: GpuContext) -> Self {
        let mut atlas = TextureAtlas::new(&gpu, 512);
        let branco = atlas.insert_reserved_tiles(&gpu).expect("o azulejo branco");
        let renderer = SpriteRenderer::new(gpu.clone(), FORMATO, atlas, 16);
        Self {
            branco,
            celulas: Celulas::new(&gpu, passe(&gpu)),
            placa: passe(&gpu),
            pilha: FxStackPass::new(&gpu),
            colagem: BandBlit::new(&gpu, FORMATO),
            faixa: None,
            renderer,
            gpu,
        }
    }

    /// As sprites num buffer do dispositivo (a rota do cozimento).
    fn sprites_na_placa(&self, v: &[RenderInstance]) -> wgpu::Buffer {
        let b = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of_val(v) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.gpu.queue.write_buffer(&b, 0, bytemuck::cast_slice(v));
        b
    }

    /// As faixas da rota da placa: vazias (o átlas no sampler do projecto) ou uma explícita.
    fn faixas(&self, n: usize) -> Vec<ph2d_render::GpuTexRun> {
        #[expect(clippy::cast_possible_truncation, reason = "poucas sprites")]
        self.faixa
            .map(|sampling| ph2d_render::GpuTexRun {
                texture_id: 0,
                start: 0,
                end: n as u32,
                blend: 0,
                sampling,
            })
            .into_iter()
            .collect()
    }

    fn copias(&self, ds: &[ShapeInstance]) -> wgpu::Buffer {
        let b = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of_val(ds) as u64).max(64),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.gpu.queue.write_buffer(&b, 0, bytemuck::cast_slice(ds));
        b
    }

    /// `Each Shape`: as células, e a camada que elas devolvem levada ao sRGB reto (pilha vazia).
    fn cada_forma(&mut self, ds: &[ShapeInstance], ops: &[FxOpGpu]) -> Vec<u8> {
        self.cada_um(ds, Rota::Cpu(&[]), ops)
    }

    fn cada_um(&mut self, ds: &[ShapeInstance], sprites: Rota<'_>, ops: &[FxOpGpu]) -> Vec<u8> {
        let (placa_s, faixas);
        let sprites = match sprites {
            Rota::Cpu(v) => Sprites::Cpu(v),
            Rota::Placa(v) => {
                placa_s = self.sprites_na_placa(v);
                faixas = self.faixas(v.len());
                #[expect(clippy::cast_possible_truncation, reason = "poucas sprites")]
                Sprites::Placa {
                    buffer: &placa_s,
                    n: v.len() as u32,
                    runs: &faixas,
                }
            }
        };
        let buf = self.copias(ds);
        let camada = textura(&self.gpu);
        let vista_camada = camada.create_view(&wgpu::TextureViewDescriptor::default());
        let cam = camera();
        #[expect(clippy::cast_possible_truncation, reason = "poucas cópias")]
        self.celulas.corre(Entrada {
            gpu: &self.gpu,
            renderer: &mut self.renderer,
            camera: &cam,
            window: WindowSize {
                width: W,
                height: H,
            },
            scene_viewport: None,
            sprites,
            formas: Some(Formas {
                buffer: &buf,
                n: ds.len() as u32,
                vista: vista(),
                corridas: &[],
            }),
            ops,
            pilha: &mut self.pilha,
            camada: &vista_camada,
            tamanho: (W, H),
            imagens: None,
        });
        let out = make_output_texture(&self.gpu, W, H);
        self.pilha
            .run_layer(&self.gpu, &camada, &out, W, H, &[], Origem::Margem);
        le(&self.gpu, &out)
    }

    /// `Whole Layer`: a placa desenha todas na camada e a pilha corre sobre a união.
    fn camada_inteira(&mut self, ds: &[ShapeInstance], ops: &[FxOpGpu]) -> Vec<u8> {
        self.inteira(ds, &[], ops)
    }

    fn inteira(&mut self, ds: &[ShapeInstance], ss: &[RenderInstance], ops: &[FxOpGpu]) -> Vec<u8> {
        self.inteira_por(ds, Rota::Cpu(ss), ops)
    }

    fn inteira_por(&mut self, ds: &[ShapeInstance], ss: Rota<'_>, ops: &[FxOpGpu]) -> Vec<u8> {
        let linear = textura(&self.gpu);
        let vl = linear.create_view(&wgpu::TextureViewDescriptor::default());
        let (cpu, placa): (&[RenderInstance], Option<wgpu::Buffer>) = match ss {
            Rota::Cpu(v) => (v, None),
            Rota::Placa(v) => (&[], Some(self.sprites_na_placa(v))),
        };
        #[expect(clippy::cast_possible_truncation, reason = "poucas sprites")]
        let n = match ss {
            Rota::Placa(v) => v.len() as u32,
            Rota::Cpu(_) => 0,
        };
        let faixas = self.faixas(n as usize);
        self.renderer.render_instances_only(
            &vl,
            &camera(),
            WindowSize {
                width: W,
                height: H,
            },
            wgpu::Color::TRANSPARENT,
            cpu,
            placa.as_ref().map(|b| (b, n, faixas.as_slice())),
            None,
        );
        let buf = self.copias(ds);
        let formas = textura(&self.gpu);
        let camada = textura(&self.gpu);
        let (vf, vc) = (
            formas.create_view(&wgpu::TextureViewDescriptor::default()),
            camada.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        let mut enc = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        #[expect(clippy::cast_possible_truncation, reason = "poucas cópias")]
        self.placa.draw(
            &self.gpu,
            &mut enc,
            &vf,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            vista(),
            Copias {
                buffer: &buf,
                count: ds.len() as u32,
            },
        );
        drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &vc,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        }));
        self.gpu.queue.submit([enc.finish()]);
        self.colagem.blit(&self.gpu, &vc, &vl, BandSource::Sprites);
        self.colagem.blit(&self.gpu, &vc, &vf, BandSource::Formas);
        let out = make_output_texture(&self.gpu, W, H);
        self.pilha
            .run_layer(&self.gpu, &camada, &out, W, H, ops, Origem::Margem);
        le(&self.gpu, &out)
    }
}

/// A grelha 9×9 de discos que se TOCAM (ou com `vao` px entre eles), centrada na origem.
fn grelha(vao_px: f32) -> Vec<ShapeInstance> {
    let passo = DIAMETRO + vao_px / K;
    (-4..=4)
        .flat_map(|j| (-4..=4).map(move |i| disco(i as f32 * passo, j as f32 * passo)))
        .collect()
}

/// O pior desvio de cor no MIOLO do disco central (os texels opacos da forma sozinha a mais de
/// 1 px da borda), e quantos texels.
fn desvio_no_miolo(sozinho: &[u8], outro: &[u8]) -> (u8, usize) {
    let (cx, cy) = (W as f32 * 0.5, H as f32 * 0.5);
    let raio = DIAMETRO * K * 0.5 - 1.5;
    let (mut pior, mut n) = (0u8, 0usize);
    for y in 0..H {
        for x in 0..W {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            if dx * dx + dy * dy > raio * raio {
                continue;
            }
            let o = ((y * W + x) * 4) as usize;
            for c in 0..4 {
                pior = pior.max(sozinho[o + c].abs_diff(outro[o + c]));
            }
            n += 1;
        }
    }
    (pior, n)
}

/// ⭐⭐ **A forma cercada recebe o MESMO efeito que sozinha** (doc 129 §5.5 A1) — nos dois modos
/// da sombra interna, a tocar e com vão de 2 px. Controlo: a camada inteira, onde a união das
/// formas muda o miolo do disco central.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_surrounded_shape_gets_the_same_effect_as_alone() {
    let Some(gpu) = gpu() else {
        eprintln!("[fx_cells] sem adapter — skip");
        return;
    };
    let mut b = Banca::new(gpu);
    // Os degraus de DENTRO (só eles deixam o miolo da forma cercada intacto): a sombra interna nos
    // dois modos, o Bevel e o Inner Glow no campo de distância (o JFA da semente).
    let campo = |kind| FxOpGpu {
        kind,
        sigma_px: 4.0,
        offset_px: [-4.0, 4.0],
        mode: FxOp::MODE_CONTOUR,
        ..sombra_interna(true)
    };
    let degraus = [
        ("Inner Shadow · Proximity", sombra_interna(false)),
        ("Inner Shadow · Contour", sombra_interna(true)),
        ("Bevel", campo(FxOp::BEVEL)),
        ("Inner Glow · Contour", campo(FxOp::INNER_GLOW)),
    ];
    for ((nome, op), v) in degraus
        .into_iter()
        .flat_map(|d| VARIANTES.map(|v| (d.clone(), v)))
    {
        b.celulas.escolhe(v);
        let ops = [op];
        let sozinho = b.cada_forma(&[disco(0.0, 0.0)], &ops);
        for vao in [0.0, 2.0] {
            let cercado = b.cada_forma(&grelha(vao), &ops);
            let uniao = b.camada_inteira(&grelha(vao), &ops);
            let (d, n) = desvio_no_miolo(&sozinho, &cercado);
            let (dc, _) = desvio_no_miolo(&sozinho, &uniao);
            eprintln!(
                "[cada forma] {v:?} {nome} vão {vao} px: cercado × sozinho {d} níveis em {n} \
                 texels · a camada inteira {dc}"
            );
            assert!(n > 50, "o miolo tem texels");
            assert!(
                d <= 1,
                "{nome}: a forma cercada desvia {d} níveis da sozinha"
            );
            // O controlo: a união muda o miolo — a sombra interna sempre; o campo (banda de 4 px)
            // só quando as formas se TOCAM (com 2 px de vão a união já não o alcança).
            let barra = match (nome.starts_with("Inner Shadow"), vao == 0.0) {
                (true, _) => Some(20),
                (false, true) => Some(4),
                (false, false) => None,
            };
            if let Some(barra) = barra {
                assert!(
                    dc > barra,
                    "controlo: na camada inteira a união muda o miolo ({nome}: {dc})"
                );
            }
        }
    }
}

/// ⭐ **Uma forma SOZINHA é a mesma nas duas opções** — sem vizinhas, «cada forma» e «a camada
/// inteira» são a mesma pergunta; o que difere é só a ida e volta pelo átlas (≤ 1 nível).
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_lone_shape_is_the_same_under_both_scopes() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut b = Banca::new(gpu);
    for (ops, v) in [
        vec![sombra_interna(true)],
        vec![{
            let mut s = sombra_interna(false);
            s.kind = FxOp::DROP_SHADOW;
            s.offset_px = [3.25, 2.5];
            s
        }],
    ]
    .into_iter()
    .flat_map(|o| VARIANTES.map(|v| (o.clone(), v)))
    {
        b.celulas.escolhe(v);
        // Uma posição SUB-pixel: a fração tem de sobreviver à ida e volta.
        let um = [disco(0.33, -0.21)];
        let a = b.cada_forma(&um, &ops);
        let c = b.camada_inteira(&um, &ops);
        let pior = a
            .iter()
            .zip(&c)
            .map(|(p, q)| p.abs_diff(*q))
            .max()
            .unwrap_or(0);
        eprintln!(
            "[sozinha] {v:?} tipo {}: cada forma × camada inteira {pior} níveis",
            ops[0].kind
        );
        assert!(
            pior <= 1,
            "a mesma forma sozinha desvia {pior} níveis entre os dois scopes"
        );
    }
}

/// Uma sprite branca de `4,4 × 2,4` (88 × 48 px), rodada 30°, numa posição sub-pixel.
///
/// ⚠️ A textura é AMPLIADA (o azulejo tem 64 px): reduzida, a amostragem escolhe o nível de mipmap
/// pela derivada, e perto da fronteira entre dois níveis (medido: o disco de 256 px a ~22 px, ×11,6 ≈
/// 2^3,5) uma diferença ínfima troca o nível inteiro — a sensibilidade é da amostragem, não da célula.
fn sprite(b: &Banca, x: f32, y: f32) -> RenderInstance {
    let (s, c) = 30f32.to_radians().sin_cos();
    RenderInstance {
        world_pos: [x, y],
        size: [4.4, 2.4],
        atlas_uv: b.branco,
        tint: [0.3, 0.6, 0.9, 1.0],
        basis: [c, s, -s, c],
        premultiplied: 0.0,
        anchor: [0.1, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id: 0,
        z_order: 0,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// ⭐ **Uma SPRITE sozinha é a mesma nas duas opções, pelas DUAS rotas** — a da CPU (a soma na CPU)
/// e a da placa (a relocação por compute sobre o buffer do cozimento). E a caixa medida contém todo
/// pixel que ela desenha, com no máximo 2 px de folga (`the_sprite_box_holds_every_pixel_it_draws`).
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_lone_sprite_is_the_same_under_both_scopes_on_both_routes() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut b = Banca::new(gpu);
    let um = [sprite(&b, 0.37, -0.45)];
    // A sozinha e mais quatro, rodadas, em posições sub-pixel (as translações das células saem
    // pares e ímpares antes da lei).
    let cena = silhueta::cena(&b);
    let mut sombra = sombra_interna(false);
    sombra.kind = FxOp::DROP_SHADOW;
    sombra.offset_px = [2.5, 3.25];
    let ops = [sombra];
    // A silhueta: os texels da sprite crua com um vizinho vazio, ou vazios com um vizinho pintado.
    let crua = b.inteira(&[], &um, &[]);
    let crua_cena = b.inteira(&[], &cena, &[]);
    let a = |x: u32, y: u32| crua_cena[((y * W + x) * 4 + 3) as usize] > 0;
    let na_silhueta = |x: u32, y: u32| {
        let v = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
        v.iter().any(|&(i, j)| a(i, j) != a(x, y))
    };
    // ⭐ As duas ROTAS dão o mesmo texel (doc 132 §6, P6): a partição vazia do buffer da placa e o
    // `SAMPLING_DEFAULT` da fatia da CPU ligam o MESMO sampler, o do projecto. Antes: 61 texels da
    // silhueta a até 29 níveis.
    let muda = |p: &[u8], q: &[u8]| {
        p.chunks(4)
            .zip(q.chunks(4))
            .filter(|(x, y)| x.iter().zip(*y).any(|(i, j)| i.abs_diff(*j) > 1))
            .count()
    };
    let placa_crua = b.inteira_por(&[], Rota::Placa(&cena), &[]);
    let placa = b.inteira_por(&[], Rota::Placa(&cena), &ops);
    let cpu = b.inteira_por(&[], Rota::Cpu(&cena), &ops);
    let (rc, rf) = (muda(&placa_crua, &crua_cena), muda(&placa, &cpu));
    eprintln!("[sprite] rotas placa × cpu: crua {rc} texels · filtrada {rf} texels");
    assert_eq!((rc, rf), (0, 0), "as duas rotas desenham a mesma sprite");
    for ((rota, r, r2), v) in [
        ("placa", Rota::Placa(&cena), Rota::Placa(&cena)),
        ("cpu", Rota::Cpu(&cena), Rota::Cpu(&cena)),
    ]
    .into_iter()
    .flat_map(|x| VARIANTES.map(|v| (x, v)))
    {
        b.celulas.escolhe(v);
        let inteira = b.inteira_por(&[], r2, &ops);
        let cada = b.cada_um(&[], r, &ops);
        let (mut pior_dentro, mut na_borda, mut tinta) = (0u8, 0usize, 0usize);
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                let o = ((y * W + x) * 4) as usize;
                tinta += usize::from(cada[o + 3] > 0);
                let d = (0..4)
                    .map(|c| cada[o + c].abs_diff(inteira[o + c]))
                    .max()
                    .unwrap_or(0);
                if na_silhueta(x, y) {
                    na_borda += usize::from(d > 1);
                } else {
                    pior_dentro = pior_dentro.max(d);
                }
            }
        }
        eprintln!(
            "[sprite] {v:?} rota {rota}: fora da silhueta {pior_dentro} níveis · na silhueta {na_borda} \
             texels mudam ({tinta} pintados)"
        );
        assert!(tinta > 100, "a sprite desenhou");
        assert!(
            pior_dentro <= 1,
            "rota {rota}: a sprite desvia {pior_dentro} níveis"
        );
        // ⭐ A barra é a MEDIDA: zero texels da silhueta (doc 132 §6, P6). O sampler do projecto
        // (trilinear, anisotropia 16) lê as derivadas do quadrado 2×2 de pixels, e a célula com
        // translação ÍMPAR trocava o quadrado da aresta: 75 texels a até 21 níveis nesta cena. A
        // translação das sprites sai PAR ([`ph2d_fx_cells::planeia_com_par`]).
        assert_eq!(
            na_borda, 0,
            "rota {rota} · {v:?}: {na_borda} texels da silhueta mudam"
        );
    }
    // A caixa contra os pixels que a sprite REALMENTE desenha (sem filtro).
    let caixa = b.celulas.ultimas_caixas()[0];
    let (mut lo, mut hi) = ([u32::MAX; 2], [0u32; 2]);
    for y in 0..H {
        for x in 0..W {
            if crua[((y * W + x) * 4 + 3) as usize] > 0 {
                lo = [lo[0].min(x), lo[1].min(y)];
                hi = [hi[0].max(x + 1), hi[1].max(y + 1)];
            }
        }
    }
    eprintln!("[sprite] caixa medida {caixa:?} · pixels {lo:?}..{hi:?}");
    #[expect(clippy::cast_precision_loss, reason = "pixels")]
    let (plo, phi) = ([lo[0] as f32, lo[1] as f32], [hi[0] as f32, hi[1] as f32]);
    for k in 0..2 {
        assert!(
            caixa.lo[k].floor() <= plo[k] && caixa.hi[k].ceil() >= phi[k],
            "a caixa contém a tinta"
        );
        assert!(
            plo[k] - caixa.lo[k] <= 2.0 && caixa.hi[k] - phi[k] <= 2.0,
            "e é apertada"
        );
    }
}

/// A lente com as QUATRO secções ligadas (o pior caso do passe: a aberração amostra 16 nós).
fn lente_cheia() -> ph2d_lens::Lente {
    use ph2d_lens::{Aberracao, Grade, Grao, Lente, Vinheta};
    Lente {
        aberracao: Aberracao {
            amount: 0.03,
            linear: false,
            angle: 0.0,
        },
        vinheta: Vinheta {
            amount: 0.5,
            midpoint: 0.9,
            roundness: 0.0,
            feather: 0.5,
            center: [0.0; 2],
            cor: [0.0; 3],
        },
        grade: Grade {
            exposure: 0.2,
            temperatura: 0.3,
            matiz: 0.1,
            contraste: 0.2,
            saturacao: 0.25,
            lut_mix: 0.0,
        },
        grao: Grao {
            amount: 0.4,
            size: 2.0,
            response: 0.5,
            colorido: true,
            animado: true,
            semente: 77,
        },
    }
}

/// **O RELÓGIO** (doc 129 §3.4 e §4 W4, `MEDIR_VELOCIDADE.md`): «cada forma» contra «a camada
/// inteira», a 256 e a 4 096 discos que se tocam, a sombra interna (σ 5,3 px, deslocada 6 px) — a
/// 1600×972 e a 4K. Ao lado, o que custaria a cura do sub-rect (a camada inteira num retângulo de
/// 55 % × 55 %, a fração da cena na foto de 1600×972) e a lente com as quatro secções na janela
/// inteira. As variantes no MESMO processo, intercaladas em blocos, o MÍNIMO de cada. Inclui a
/// leitura das caixas (a espera pela placa). Imprime; não reprova.
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-fx-cells --test it relogio -- --ignored --nocapture`
#[test]
#[ignore = "medição de GPU"]
#[expect(
    clippy::too_many_lines,
    reason = "uma bancada: as variantes lado a lado"
)]
fn relogio_cada_forma_contra_camada_inteira() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut b = Banca::new(gpu);
    let mut lente = ph2d_render::LensPass::new(&b.gpu, ph2d_render::WorldRt::FORMAT);
    let cheia = lente_cheia();
    let ops = [sombra_interna(false)];
    for (lw, lh) in [(1600u32, 972u32), (3840, 2160)] {
        let (sw, sh) = (lw * 55 / 100, lh * 55 / 100);
        let mundo = b.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: lw,
                height: lh,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ph2d_render::WorldRt::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let vm = mundo.create_view(&wgpu::TextureViewDescriptor::default());
        let alvo = |gpu: &GpuContext| {
            gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: lw,
                    height: lh,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FORMATO,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let k = 12.0f32;
        let vista = ShapeView {
            lin: [k, 0.0, 0.0, -k],
            t: [lw as f32 * 0.5, lh as f32 * 0.5],
            alvo: [lw as f32, lh as f32],
        };
        let cam = Camera2d {
            height_world: lh as f32 / k,
            ..Camera2d::default()
        };
        for lado in [16i32, 64] {
            let ds: Vec<ShapeInstance> = (0..lado)
                .flat_map(|j| {
                    (0..lado).map(move |i| ShapeInstance {
                        size: [1.0, 1.0],
                        ..disco((i - lado / 2) as f32, (j - lado / 2) as f32)
                    })
                })
                .collect();
            let buf = b.copias(&ds);
            let (camada, saida, formas) = (
                alvo(&b.gpu),
                make_output_texture(&b.gpu, lw, lh),
                alvo(&b.gpu),
            );
            let (vc, vf) = (
                camada.create_view(&wgpu::TextureViewDescriptor::default()),
                formas.create_view(&wgpu::TextureViewDescriptor::default()),
            );
            let n = ds.len() as u32;
            let mut cada = f64::MAX;
            let mut inteira = f64::MAX;
            let mut sub = f64::MAX;
            let mut janela = f64::MAX;
            let mut pilha_sub = FxStackPass::new(&b.gpu);
            let mut paginas = 0;
            for _bloco in 0..12 {
                let t0 = std::time::Instant::now();
                b.celulas.corre(Entrada {
                    gpu: &b.gpu,
                    renderer: &mut b.renderer,
                    camera: &cam,
                    window: WindowSize {
                        width: lw,
                        height: lh,
                    },
                    scene_viewport: None,
                    sprites: Sprites::Cpu(&[]),
                    formas: Some(Formas {
                        buffer: &buf,
                        n,
                        vista,
                        corridas: &[],
                    }),
                    ops: &ops,
                    pilha: &mut b.pilha,
                    camada: &vc,
                    tamanho: (lw, lh),
                    imagens: None,
                });
                b.pilha
                    .run_layer(&b.gpu, &camada, &saida, lw, lh, &[], Origem::Margem);
                let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
                cada = cada.min(t0.elapsed().as_secs_f64() * 1e3);
                paginas = b.celulas.ultimas_paginas().len();

                let t0 = std::time::Instant::now();
                let mut enc = b
                    .gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                b.placa.draw(
                    &b.gpu,
                    &mut enc,
                    &vf,
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    vista,
                    Copias {
                        buffer: &buf,
                        count: n,
                    },
                );
                b.gpu.queue.submit([enc.finish()]);
                b.colagem.blit(&b.gpu, &vc, &vf, BandSource::Formas);
                b.pilha
                    .run_layer(&b.gpu, &camada, &saida, lw, lh, &ops, Origem::Margem);
                let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
                inteira = inteira.min(t0.elapsed().as_secs_f64() * 1e3);

                // A cura do sub-rect: a MESMA camada, a pilha só sobre o retângulo da cena (uma
                // pilha à parte — alternar o tamanho na mesma realocaria o trabalho a cada bloco).
                let t0 = std::time::Instant::now();
                let mut enc = b
                    .gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                b.placa.draw(
                    &b.gpu,
                    &mut enc,
                    &vf,
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    vista,
                    Copias {
                        buffer: &buf,
                        count: n,
                    },
                );
                b.gpu.queue.submit([enc.finish()]);
                b.colagem.blit(&b.gpu, &vc, &vf, BandSource::Formas);
                pilha_sub.run_layer(&b.gpu, &camada, &saida, sw, sh, &ops, Origem::Margem);
                let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
                sub = sub.min(t0.elapsed().as_secs_f64() * 1e3);

                let t0 = std::time::Instant::now();
                assert!(lente.aplica(&b.gpu, &mundo, &vm, [0, 0, lw, lh], &cheia, 3));
                let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
                janela = janela.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            eprintln!(
                "[relógio] {lw}×{lh} · {n} discos: cada forma {cada:.2} ms ({paginas} página(s)) · camada inteira \
             {inteira:.2} ms · camada no sub-rect {sw}×{sh} {sub:.2} ms · lente (4 secções) {janela:.2} ms · load {}",
                std::fs::read_to_string("/proc/loadavg")
                    .unwrap_or_default()
                    .split(' ')
                    .next()
                    .unwrap_or("?")
            );
        }
    }
}

/// A grelha `cols × rows` de discos de 1 unidade que se tocam (12 px a `k = 12`), centrada.
fn grelha_de(cols: i32, rows: i32) -> Vec<ShapeInstance> {
    (0..rows)
        .flat_map(|j| {
            (0..cols).map(move |i| ShapeInstance {
                size: [1.0, 1.0],
                ..disco((i - cols / 2) as f32, (j - rows / 2) as f32)
            })
        })
        .collect()
}

/// As cinco populações da rodada C.10 (doc 131 §4): 256 · 1 024 · 2 048 · 4 096 · 8 192 discos,
/// todos dentro de 1600×972.
const POPULACOES: [(i32, i32); 5] = [(16, 16), (32, 32), (64, 32), (64, 64), (128, 64)];

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split(' ')
        .next()
        .unwrap_or("?")
        .to_owned()
}

/// A bancada da rodada C.10: a camada de 1600×972, a vista e a câmera a `k = 12`.
struct Mesa {
    lw: u32,
    lh: u32,
    vista: ShapeView,
    cam: Camera2d,
    camada: wgpu::Texture,
    vc: wgpu::TextureView,
}

impl Mesa {
    fn new(gpu: &GpuContext) -> Self {
        let (lw, lh) = (1600u32, 972u32);
        let k = 12.0f32;
        let camada = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: lw,
                height: lh,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMATO,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        Self {
            lw,
            lh,
            vista: ShapeView {
                lin: [k, 0.0, 0.0, -k],
                t: [lw as f32 * 0.5, lh as f32 * 0.5],
                alvo: [lw as f32, lh as f32],
            },
            cam: Camera2d {
                height_world: lh as f32 / k,
                ..Camera2d::default()
            },
            vc: camada.create_view(&wgpu::TextureViewDescriptor::default()),
            camada,
        }
    }

    /// Uma corrida «cada forma» das `n` cópias em `buf` (e das `sprites`, pela CPU) sobre a camada.
    fn corre(
        &self,
        b: &mut Banca,
        buf: &wgpu::Buffer,
        n: u32,
        sprites: &[RenderInstance],
        ops: &[FxOpGpu],
    ) {
        b.celulas.corre(Entrada {
            gpu: &b.gpu,
            renderer: &mut b.renderer,
            camera: &self.cam,
            window: WindowSize {
                width: self.lw,
                height: self.lh,
            },
            scene_viewport: None,
            sprites: Sprites::Cpu(sprites),
            formas: Some(Formas {
                buffer: buf,
                n,
                vista: self.vista,
                corridas: &[],
            }),
            ops,
            pilha: &mut b.pilha,
            camada: &self.vc,
            tamanho: (self.lw, self.lh),
            imagens: None,
        });
    }
}

/// **C.10 — onde vão os milissegundos de «cada forma»** (doc 131 §4 / §7): por população, o
/// relógio da PLACA por fase (`pass_profiler`, uma submissão por fase) e o da CPU (`Instant`: a
/// corrida até à última submissão, e até a placa acabar). Imprime; não reprova.
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-fx-cells --test it relogio_c10_decomposicao -- --ignored --nocapture`
#[test]
#[ignore = "medição de GPU"]
fn relogio_c10_decomposicao() {
    let Some(gpu) = gpu() else {
        return;
    };
    ph2d_gpu::pass_profiler::init_forced(&gpu.device, &gpu.queue);
    let mut b = Banca::new(gpu);
    let mesa = Mesa::new(&b.gpu);
    eprintln!("[c10] load no início {}", carga());
    for (pilha, ops) in pilhas_de_fora() {
        let ops = ops.as_slice();
        let (al, at, ar, ab) = ph2d_render::stack_reach(ops);
        let [ll, lt, lr, lb] = ph2d_render::stack_read_por_lado(ops);
        // O disco de 12 px é um conteúdo de 14 (a folga de anti-aliasing); a vala por lado.
        let vala_x = (ar + lr).max(al + ll).max(al + ar) + 1;
        let vala_y = (ab + lb).max(at + lt).max(at + ab) + 1;
        eprintln!(
            "[c10] ══ {pilha}: A {:?} · L {:?} px · célula {}×{} px · saída {}×{} px",
            (al, at, ar, ab),
            [ll, lt, lr, lb],
            14 + vala_x,
            14 + vala_y,
            14 + al + ar,
            14 + at + ab
        );
        camada_inteira_decomposta(&mut b, &mesa, ops);
        decompoe(&mut b, &mesa, ops, &POPULACOES);
    }
    ph2d_gpu::pass_profiler::shutdown();
}

/// As pilhas da decomposição do P1 (doc 132 §1): o controlo de DENTRO (a sombra interna da C.10) e
/// os dois degraus de FORA — o Outer Glow (alcance ≈ leitura, `16`/`17`) e a Drop Shadow deslocada.
fn pilhas_de_fora() -> Vec<(&'static str, Vec<FxOpGpu>)> {
    let fora = |kind, offset_px| FxOpGpu {
        kind,
        sigma_px: 5.3,
        offset_px,
        tint: [1.0, 0.8, 0.2, 1.0],
        mode: FxOp::MODE_PROXIMITY,
        ..sombra_interna(false)
    };
    vec![
        ("Inner Shadow (dentro)", vec![sombra_interna(false)]),
        ("Outer Glow", vec![fora(FxOp::GLOW, [0.0; 2])]),
        (
            "Drop Shadow (fora)",
            vec![fora(FxOp::DROP_SHADOW, [6.0, 6.0])],
        ),
        // Longa para UM lado (doc 132 §1, a vala por lado): o lado oposto quase não lê.
        (
            "Drop Shadow longa (→)",
            vec![fora(FxOp::DROP_SHADOW, [14.0, 0.0])],
        ),
    ]
}

/// A pilha da CAMADA INTEIRA na mesma janela (1600×972), por passe — o termo de comparação.
fn camada_inteira_decomposta(b: &mut Banca, mesa: &Mesa, ops: &[FxOpGpu]) {
    let out = make_output_texture(&b.gpu, mesa.lw, mesa.lh);
    for _ in 0..3 {
        b.pilha.run_layer(
            &b.gpu,
            &mesa.camada,
            &out,
            mesa.lw,
            mesa.lh,
            ops,
            Origem::Margem,
        );
        ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
    }
    let _ = ph2d_gpu::pass_profiler::drain(&b.gpu.device);
    for _ in 0..20 {
        b.pilha.run_layer(
            &b.gpu,
            &mesa.camada,
            &out,
            mesa.lw,
            mesa.lh,
            ops,
            Origem::Margem,
        );
        ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
    }
    let t = ph2d_gpu::pass_profiler::drain(&b.gpu.device).unwrap_or_default();
    let f = f64::from(t.frames.max(1));
    let mut linha = format!(
        "[c10] camada inteira {:.2} Mpx · placa {:.3} ms\n[c10]   ",
        f64::from(mesa.lw * mesa.lh) / 1e6,
        t.span_ns as f64 / f / 1e6
    );
    let mut fases = t.per_label.clone();
    fases.sort_by_key(|f| std::cmp::Reverse(f.1));
    for (rotulo, ns, k) in fases {
        linha.push_str(&format!(
            "{rotulo}={:.3} ms(×{:.0}) ",
            ns as f64 / f / 1e6,
            f64::from(k) / f
        ));
    }
    eprintln!("{linha}");
}

/// «Cada forma» por população, por fase e por passe da pilha.
fn decompoe(b: &mut Banca, mesa: &Mesa, ops: &[FxOpGpu], populacoes: &[(i32, i32)]) {
    for &(cols, rows) in populacoes {
        let ds = grelha_de(cols, rows);
        let buf = b.copias(&ds);
        let n = ds.len() as u32;
        let (mut submete, mut total) = (f64::MAX, f64::MAX);
        for _ in 0..3 {
            mesa.corre(b, &buf, n, &[], ops);
            ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
        }
        let _ = ph2d_gpu::pass_profiler::drain(&b.gpu.device);
        for _ in 0..20 {
            let t0 = std::time::Instant::now();
            mesa.corre(b, &buf, n, &[], ops);
            submete = submete.min(t0.elapsed().as_secs_f64() * 1e3);
            let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
            total = total.min(t0.elapsed().as_secs_f64() * 1e3);
            ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
        }
        let t = ph2d_gpu::pass_profiler::drain(&b.gpu.device).unwrap_or_default();
        let f = f64::from(t.frames.max(1));
        let paginas = b.celulas.ultimas_paginas();
        let area: u64 = paginas
            .iter()
            .map(|p| u64::from(p.tamanho[0]) * u64::from(p.tamanho[1]))
            .sum();
        let tamanhos: Vec<[u32; 2]> = paginas.iter().map(|p| p.tamanho).collect();
        let mut linha = format!(
            "[c10] {n} discos · {} página(s) {tamanhos:?} · {:.1} Mpx · CPU até submeter \
             {submete:.2} ms · até a placa acabar {total:.2} ms · placa ocupada {:.2} ms ({} \
             quadros) · load {}\n[c10]   ",
            paginas.len(),
            area as f64 / 1e6,
            t.span_ns as f64 / f / 1e6,
            t.frames,
            carga()
        );
        let mut fases = t.per_label.clone();
        fases.sort_by_key(|f| std::cmp::Reverse(f.1));
        for (rotulo, ns, k) in fases {
            linha.push_str(&format!(
                "{rotulo}={:.3} ms(×{:.0}) ",
                ns as f64 / f / 1e6,
                f64::from(k) / f
            ));
        }
        eprintln!("{linha}");
    }
}

/// Os degraus da paridade entre variantes: de DENTRO (alcance nulo — onde a vala justa encolhe), de
/// FORA com deslocamento sub-pixel, a turbulência (o mapa da origem por célula) e uma pilha de dois.
fn degraus_da_paridade() -> Vec<(&'static str, Vec<FxOpGpu>)> {
    let com = |kind, sigma_px, offset_px: [f32; 2], mode| FxOpGpu {
        kind,
        sigma_px,
        offset_px,
        mode,
        ..sombra_interna(false)
    };
    vec![
        ("Inner Shadow · Proximity", vec![sombra_interna(false)]),
        ("Inner Shadow · Contour", vec![sombra_interna(true)]),
        (
            "Drop Shadow",
            vec![com(
                FxOp::DROP_SHADOW,
                4.0,
                [3.25, 2.5],
                FxOp::MODE_PROXIMITY,
            )],
        ),
        (
            "Glow",
            vec![com(FxOp::GLOW, 4.0, [0.0; 2], FxOp::MODE_PROXIMITY)],
        ),
        (
            "Bevel",
            vec![com(FxOp::BEVEL, 4.0, [-4.0, 4.0], FxOp::MODE_PROXIMITY)],
        ),
        (
            "Turbulence",
            vec![FxOpGpu {
                detail: 3,
                ..com(FxOp::TURBULENCE, 4.0, [0.0; 2], FxOp::MODE_SMOOTH)
            }],
        ),
        (
            "Drop Shadow + Inner Shadow",
            vec![
                com(FxOp::DROP_SHADOW, 3.0, [2.0, 2.0], FxOp::MODE_PROXIMITY),
                sombra_interna(false),
            ],
        ),
    ]
    .into_iter()
    // As pilhas de FORA da decomposição do P1 (doc 132 §1): alcance ≈ leitura.
    .chain(pilhas_de_fora().into_iter().skip(1))
    .collect()
}

/// A faixa `[min, max]` por canal que as instâncias IGUAIS de uma cena dão a cada pixel relativo ao
/// centro delas (`centros` em pixels inteiros, janela `±r`) — a verdade de «cada forma» para formas
/// idênticas em posições inteiras, e a dispersão que a própria referência já tem por posição no
/// átlas (o arredondamento `f32` da relocação: medido, o Bevel dá dois valores a 2 níveis um do
/// outro no mesmo pixel relativo de discos iguais).
fn faixas(img: &[u8], lw: u32, centros: &[(i32, i32)], r: i32) -> Vec<[(u8, u8); 4]> {
    let lado = (2 * r + 1) as usize;
    let mut f = vec![[(u8::MAX, 0u8); 4]; lado * lado];
    for &(cx, cy) in centros {
        for dy in -r..=r {
            for dx in -r..=r {
                let o = (((cy + dy) as u32 * lw + (cx + dx) as u32) * 4) as usize;
                let k = ((dy + r) as usize) * lado + (dx + r) as usize;
                for c in 0..4 {
                    let (lo, hi) = &mut f[k][c];
                    *lo = (*lo).min(img[o + c]);
                    *hi = (*hi).max(img[o + c]);
                }
            }
        }
    }
    f
}

/// ⭐⭐ **As variantes da rodada C.10 dão o pixel da de sempre** (doc 131 §7): para formas IGUAIS em
/// posições inteiras, cada pixel de cada instância cai na faixa que a referência dá a esse pixel
/// relativo (± 1 nível, a barra das portas «sozinha × camada inteira» deste ficheiro). Duas cenas a
/// 1600×972: 4 096 discos que se tocam (várias páginas: a vala dupla faz até 4) e 24 sprites
/// rodadas, por cada degrau de [`degraus_da_paridade`]. Controlo: o filtro muda a cena.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_c10_variants_give_the_same_pixels() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut b = Banca::new(gpu);
    let mesa = Mesa::new(&b.gpu);
    let ds = grelha_de(64, 64);
    let buf = b.copias(&ds);
    let ss: Vec<RenderInstance> = (0..24)
        .map(|i| {
            // A MESMA fração sub-pixel em todas (0,396 px · −0,252 px), e o azulejo AMPLIADO
            // (79 px de 64): a sensibilidade do desenhador — a aresta sobre um centro de pixel, o
            // nível de mipmap de uma textura reduzida (o gate da sprite sozinha) — não é da célula.
            RenderInstance {
                size: [6.6, 3.6],
                ..sprite(
                    &b,
                    (i % 6) as f32 * 12.0 - 30.0 + 0.033,
                    (i / 6) as f32 * 12.0 - 18.0 - 0.021,
                )
            }
        })
        .collect();
    let px = |p: [f32; 2]| ((800.0 + p[0] * 12.0) as i32, (486.0 - p[1] * 12.0) as i32);
    // Os discos com três vizinhos de cada lado (a mesma vizinhança até ±20 px) e todas as sprites.
    let miolo: Vec<(i32, i32)> = ds
        .iter()
        .enumerate()
        .filter(|(k, _)| (3..61).contains(&(k % 64)) && (3..61).contains(&(k / 64)))
        .map(|(_, d)| px(d.pos))
        .collect();
    let sprites: Vec<(i32, i32)> = ss.iter().map(|s| px(s.world_pos)).collect();
    let out = make_output_texture(&b.gpu, mesa.lw, mesa.lh);
    let foto = |b: &mut Banca, v: Variante, discos: bool, ops: &[FxOpGpu]| {
        b.celulas.escolhe(v);
        if discos {
            mesa.corre(b, &buf, ds.len() as u32, &[], ops);
        } else {
            mesa.corre(b, &buf, 0, &ss, ops);
        }
        let paginas = b.celulas.ultimas_paginas().len();
        b.pilha.run_layer(
            &b.gpu,
            &mesa.camada,
            &out,
            mesa.lw,
            mesa.lh,
            &[],
            Origem::Margem,
        );
        (le_de(&b.gpu, &out, mesa.lw, mesa.lh), paginas)
    };
    for (cena, discos, centros, r) in [
        ("discos", true, &miolo, 20),
        ("sprites", false, &sprites, 64),
    ] {
        let (cru, _) = foto(&mut b, VARIANTES[0], discos, &[]);
        for (nome, ops) in degraus_da_paridade() {
            let (referencia, p0) = foto(&mut b, VARIANTES[0], discos, &ops);
            let controlo = cru.iter().zip(&referencia).filter(|(a, c)| a != c).count();
            assert!(controlo > 1_000, "{cena} · {nome}: o filtro muda a cena");
            let faixa = faixas(&referencia, mesa.lw, centros, r);
            let dispersao = faixa
                .iter()
                .flat_map(|f| f.iter().map(|(lo, hi)| hi.saturating_sub(*lo)))
                .max()
                .unwrap_or(0);
            // A silhueta da cena CRUA: um texel pintado com um vizinho vazio, ou o contrário (o
            // gate da sprite sozinha: aí um quad sem anti-aliasing muda de lado com o arredondamento).
            let a = |x: i32, y: i32| cru[((y as u32 * mesa.lw + x as u32) * 4 + 3) as usize] > 0;
            let na_silhueta = |x: i32, y: i32| {
                [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                    .iter()
                    .any(|&(i, j)| a(i, j) != a(x, y))
            };
            let lado = 2 * r + 1;
            for v in &VARIANTES[1..] {
                let (img, p) = foto(&mut b, *v, discos, &ops);
                let (mut pior, mut no_miolo, mut na_borda, mut pintados) = (0u8, 0usize, 0usize, 0);
                for &(cx, cy) in centros.iter() {
                    for (k, f) in faixa.iter().enumerate() {
                        let (x, y) = (cx + k as i32 % lado - r, cy + k as i32 / lado - r);
                        let o = ((y as u32 * mesa.lw + x as u32) * 4) as usize;
                        let d = (0..4)
                            .map(|c| {
                                f[c].0
                                    .saturating_sub(img[o + c])
                                    .max(img[o + c].saturating_sub(f[c].1))
                            })
                            .max()
                            .unwrap_or(0);
                        pintados += usize::from(a(x, y));
                        if d > 1 && na_silhueta(x, y) {
                            na_borda += 1;
                        } else {
                            pior = pior.max(d);
                            no_miolo += usize::from(d > 1);
                        }
                    }
                }
                eprintln!(
                    "[c10 paridade] {cena} · {nome} · {v:?}: {p0} → {p} página(s) · fora da faixa \
                     da referência {pior} nível(is) fora da silhueta · {na_borda} texels da \
                     silhueta mudam ({pintados} pintados) · dispersão da referência {dispersao} · \
                     igual ao byte: {}",
                    img == referencia
                );
                assert!(
                    no_miolo == 0,
                    "{cena} · {nome} · {v:?}: {no_miolo} texels a {pior} níveis fora da faixa"
                );
                assert!(
                    na_borda * 200 <= pintados,
                    "{cena} · {nome} · {v:?}: {na_borda} texels da silhueta mudam"
                );
            }
        }
    }
}

/// **C.10 — a régua intercalada das variantes** (doc 131 §4 / §7, doc 132 §1, `MEDIR_VELOCIDADE.md`):
/// as [`VARIANTES`] × as cinco [`POPULACOES`] × as três [`pilhas_de_fora`], blocos de 10 quadros com
/// a ordem RODADA, 7 rodadas; por variante, o MÍNIMO (e a mediana) da placa ocupada por quadro
/// (`pass_profiler`) e da parede até a placa acabar. Imprime; não reprova.
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-fx-cells --test it relogio_c10_intercalado -- --ignored --nocapture`
#[test]
#[ignore = "medição de GPU"]
fn relogio_c10_intercalado() {
    let Some(gpu) = gpu() else {
        return;
    };
    ph2d_gpu::pass_profiler::init_forced(&gpu.device, &gpu.queue);
    let mut b = Banca::new(gpu);
    let mesa = Mesa::new(&b.gpu);
    let carga0 = carga();
    for (pilha, ops) in pilhas_de_fora() {
        for (cols, rows) in POPULACOES {
            regua(&mut b, &mesa, pilha, &ops, grelha_de(cols, rows));
        }
    }
    eprintln!("[c10 régua] load {carga0} → {}", carga());
    ph2d_gpu::pass_profiler::shutdown();
}

/// Uma população de uma pilha: as [`VARIANTES`] intercaladas.
fn regua(b: &mut Banca, mesa: &Mesa, pilha: &str, ops: &[FxOpGpu], ds: Vec<ShapeInstance>) {
    const N: usize = VARIANTES.len();
    let buf = b.copias(&ds);
    let n = ds.len() as u32;
    // O aquecimento de cada variante (átlas e pilha no tamanho), fora da régua.
    for v in VARIANTES {
        b.celulas.escolhe(v);
        for _ in 0..3 {
            mesa.corre(b, &buf, n, &[], ops);
            ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
        }
    }
    let _ = ph2d_gpu::pass_profiler::drain(&b.gpu.device);
    let mut placa: [Vec<f64>; N] = Default::default();
    let mut parede: [Vec<f64>; N] = Default::default();
    let mut paginas = [0usize; N];
    for rodada in 0..7 {
        for k in 0..N {
            let i = (k + rodada) % N;
            b.celulas.escolhe(VARIANTES[i]);
            let mut minimo = f64::MAX;
            for _ in 0..10 {
                let t0 = std::time::Instant::now();
                mesa.corre(b, &buf, n, &[], ops);
                let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
                minimo = minimo.min(t0.elapsed().as_secs_f64() * 1e3);
                ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
            }
            let t = ph2d_gpu::pass_profiler::drain(&b.gpu.device).unwrap_or_default();
            placa[i].push(t.span_ns as f64 / f64::from(t.frames.max(1)) / 1e6);
            parede[i].push(minimo);
            paginas[i] = b.celulas.ultimas_paginas().len();
        }
    }
    let resumo = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v[0], v[v.len() / 2])
    };
    let mut linha = format!("[c10 régua] {pilha} · {n} discos (load {}):", carga());
    for i in 0..N {
        let (pm, pmed) = resumo(&mut placa[i]);
        let (wm, wmed) = resumo(&mut parede[i]);
        let v = VARIANTES[i];
        linha.push_str(&format!(
            "\n[c10 régua]   justa {} salta {} · {} pág · placa {pm:.2} (med {pmed:.2}) ms · parede \
             {wm:.2} (med {wmed:.2}) ms",
            u8::from(v.vala_justa),
            u8::from(v.salta_vazias),
            paginas[i]
        ));
    }
    eprintln!("{linha}");
}

#[path = "celulas_gpu_silhueta.rs"]
mod silhueta;

#[path = "celulas_gpu_caladas.rs"]
mod caladas;
