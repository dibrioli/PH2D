//! **Cada forma, na placa** (doc 129 §3.4 / §5.5 A1): a forma CERCADA recebe o MESMO efeito que
//! SOZINHA. A fixtura é a da sonda do dono — discos que se tocam, a sombra interna deslocada — e o
//! controlo é a camada INTEIRA, onde a união das formas É a lei e o miolo muda dezenas de níveis.
//!
//! `PH2D_GPU=1 cargo test -p ph2d-fx-cells --test it -- --ignored --nocapture`

use ph2d_fx_cells::{Celulas, Entrada, FORMATO, Formas, Sprites};
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

/// Lê uma textura `Rgba8Unorm` (a saída da pilha).
fn le(gpu: &GpuContext, tex: &wgpu::Texture) -> Vec<u8> {
    let linha = (W * 4).next_multiple_of(256);
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(linha * H),
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
    let fatia = buf.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let dados = fatia.get_mapped_range();
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        let o = (y * linha) as usize;
        out.extend_from_slice(&dados[o..o + (W * 4) as usize]);
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
        let placa_s;
        let sprites = match sprites {
            Rota::Cpu(v) => Sprites::Cpu(v),
            Rota::Placa(v) => {
                placa_s = self.sprites_na_placa(v);
                #[expect(clippy::cast_possible_truncation, reason = "poucas sprites")]
                Sprites::Placa {
                    buffer: &placa_s,
                    n: v.len() as u32,
                    runs: &[],
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
            }),
            ops,
            pilha: &mut self.pilha,
            camada: &vista_camada,
            tamanho: (W, H),
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
        self.renderer.render_instances_only(
            &vl,
            &camera(),
            WindowSize {
                width: W,
                height: H,
            },
            wgpu::Color::TRANSPARENT,
            cpu,
            placa.as_ref().map(|b| (b, n, &[][..])),
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
    for contorno in [false, true] {
        let ops = [sombra_interna(contorno)];
        let sozinho = b.cada_forma(&[disco(0.0, 0.0)], &ops);
        for vao in [0.0, 2.0] {
            let cercado = b.cada_forma(&grelha(vao), &ops);
            let uniao = b.camada_inteira(&grelha(vao), &ops);
            let (d, n) = desvio_no_miolo(&sozinho, &cercado);
            let (dc, _) = desvio_no_miolo(&sozinho, &uniao);
            eprintln!(
                "[cada forma] modo {} vão {vao} px: cercado × sozinho {d} níveis em {n} texels · \
                 a camada inteira {dc}",
                if contorno { "Contour" } else { "Proximity" }
            );
            assert!(n > 50, "o miolo tem texels");
            assert!(d <= 1, "a forma cercada desvia {d} níveis da sozinha");
            assert!(
                dc > 20,
                "controlo: na camada inteira a união muda o miolo ({dc})"
            );
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
    for ops in [
        vec![sombra_interna(true)],
        vec![{
            let mut s = sombra_interna(false);
            s.kind = FxOp::DROP_SHADOW;
            s.offset_px = [3.25, 2.5];
            s
        }],
    ] {
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
            "[sozinha] tipo {}: cada forma × camada inteira {pior} níveis",
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
    let mut sombra = sombra_interna(false);
    sombra.kind = FxOp::DROP_SHADOW;
    sombra.offset_px = [2.5, 3.25];
    let ops = [sombra];
    // A silhueta: os texels da sprite crua com um vizinho vazio, ou vazios com um vizinho pintado.
    let crua = b.inteira(&[], &um, &[]);
    let a = |x: u32, y: u32| crua[((y * W + x) * 4 + 3) as usize] > 0;
    let na_silhueta = |x: u32, y: u32| {
        let v = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
        v.iter().any(|&(i, j)| a(i, j) != a(x, y))
    };
    for (rota, r, r2) in [
        ("placa", Rota::Placa(&um), Rota::Placa(&um)),
        ("cpu", Rota::Cpu(&um), Rota::Cpu(&um)),
    ] {
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
            "[sprite] rota {rota}: fora da silhueta {pior_dentro} níveis · na silhueta {na_borda} \
             texels mudam ({tinta} pintados)"
        );
        assert!(tinta > 100, "a sprite desenhou");
        assert!(
            pior_dentro <= 1,
            "rota {rota}: a sprite sozinha desvia {pior_dentro} níveis"
        );
        // ⚠️ Um quad SEM anti-aliasing inclui o pixel cujo CENTRO cai dentro: a relocação em `f32`
        // move a aresta ~10⁻⁵ px, e um centro que está sobre ela muda de lado — a mesma natureza de
        // qualquer movimento sub-pixel da sprite.
        // ⚠️ E a rota da PLACA do desenhador tem uma sensibilidade própria na silhueta: as duas
        // rotas da camada inteira já diferem em 13 texels desta sprite (doc 129 §7). Medido: CPU 0,
        // placa 17 de 8 716 (0,2 %).
        assert!(
            na_borda * 200 <= tinta,
            "rota {rota}: {na_borda} texels da silhueta mudam"
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

/// **O RELÓGIO** (doc 129 §3.4, `MEDIR_VELOCIDADE.md`): «cada forma» contra «a camada inteira»,
/// a 256 e a 4 096 discos que se tocam, a sombra interna (σ 5,3 px, deslocada 6 px) — numa camada
/// de 1600×972. As duas variantes no MESMO processo, intercaladas em blocos, o MÍNIMO de cada.
/// Inclui a leitura das caixas (a espera pela placa). Imprime; não reprova.
#[test]
#[ignore = "medição de GPU"]
fn relogio_cada_forma_contra_camada_inteira() {
    const LW: u32 = 1600;
    const LH: u32 = 972;
    let Some(gpu) = gpu() else {
        return;
    };
    let mut b = Banca::new(gpu);
    let ops = [sombra_interna(false)];
    let alvo = |gpu: &GpuContext| {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: LW,
                height: LH,
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
        t: [LW as f32 * 0.5, LH as f32 * 0.5],
        alvo: [LW as f32, LH as f32],
    };
    let cam = Camera2d {
        height_world: LH as f32 / k,
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
            make_output_texture(&b.gpu, LW, LH),
            alvo(&b.gpu),
        );
        let (vc, vf) = (
            camada.create_view(&wgpu::TextureViewDescriptor::default()),
            formas.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        let n = ds.len() as u32;
        let mut cada = f64::MAX;
        let mut inteira = f64::MAX;
        let mut paginas = 0;
        for _bloco in 0..12 {
            let t0 = std::time::Instant::now();
            b.celulas.corre(Entrada {
                gpu: &b.gpu,
                renderer: &mut b.renderer,
                camera: &cam,
                window: WindowSize {
                    width: LW,
                    height: LH,
                },
                scene_viewport: None,
                sprites: Sprites::Cpu(&[]),
                formas: Some(Formas {
                    buffer: &buf,
                    n,
                    vista,
                }),
                ops: &ops,
                pilha: &mut b.pilha,
                camada: &vc,
                tamanho: (LW, LH),
            });
            b.pilha
                .run_layer(&b.gpu, &camada, &saida, LW, LH, &[], Origem::Margem);
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
                .run_layer(&b.gpu, &camada, &saida, LW, LH, &ops, Origem::Margem);
            let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
            inteira = inteira.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        eprintln!(
            "[relógio] {n} discos: cada forma {cada:.2} ms ({paginas} página(s)) · camada inteira {inteira:.2} ms · \
             load {}",
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .split(' ')
                .next()
                .unwrap_or("?")
        );
    }
}
