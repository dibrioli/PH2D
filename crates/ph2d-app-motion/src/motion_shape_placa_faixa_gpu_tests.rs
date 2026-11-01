//! ⭐ GEF 19 — **as faixas Motion de BAIXO, pela placa real**: a faixa de baixo cola as formas dos
//! objectos dela depois do `desenha` do presente, e as cópias de SPRITE dela (`cola_os_sprites`); o
//! topo leva o resto. Lido em pixéis do mundo.
//!
//! ```text
//! PH2D_GPU=1 cargo test -p ph2d-app-motion --profile smoke --lib -- --ignored --nocapture faixa::gpu_tests
//! ```

use ph2d_eval_motion::{FaixaDoSink, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::NodeId;
use ph2d_render::draw_bands::{Band, Family, FrameOrder};
use ph2d_render::{BandBlit, Camera2d, RenderInstance, WorldRt};
use ph2d_vector::Affine;

use super::{Colagem, PasseDeSprites, sprites_por_faixa};
use crate::motion_filter::Engrenagem;
use crate::motion_objeto::{Dono, IDENTIDADE};
use crate::motion_shape_gen::VecPathStore;
use crate::motion_shape_placa::{GeometriasDaPlaca, PlacaDeFormas};
use crate::motion_state::MotionState;

const LADO: u32 = 256;
const BAIXO: u64 = 7;
const CIMA: u64 = 8;

fn gpu() -> GpuContext {
    GpuContext::new(GpuContext::default_instance(), None)
        .expect("sem adapter — este gate mede a placa")
}

/// Os objectos `BAIXO` (rank 0) e `CIMA` (rank 2), com uma faixa de VETOR entre eles.
fn ordem() -> (FrameOrder, [Band; 3]) {
    let ordem = FrameOrder {
        motion_ranks: vec![(BAIXO, 0), (CIMA, 2)],
        ..FrameOrder::default()
    };
    let b = |family, lo, hi| Band { family, lo, hi };
    let bands = [
        b(Family::Motion, 0, 1),
        b(Family::Sprite, 1, 2),
        b(Family::Motion, 2, 3),
    ];
    (ordem, bands)
}

/// Um Motion com três sinks: o de `BAIXO`, o de `CIMA` e um sem dono vivo.
fn motion() -> (MotionState, [NodeId; 3]) {
    let mut m = MotionState::new();
    let s = [(); 3].map(|()| m.doc.graph.add_node("motion.output"));
    for (no, e) in [(s[0], BAIXO), (s[1], CIMA)] {
        m.doc.set_owner(no, e);
        m.objeto.donos.insert(
            e,
            Dono {
                entity: e,
                visible: true,
                afim: IDENTIDADE,
            },
        );
    }
    (m, s)
}

/// O mundo limpo a preto, e a leitura da linha do meio: o maior canal de cada coluna.
fn mundo(gpu: &GpuContext) -> WorldRt {
    let w = WorldRt::new(gpu, (LADO, LADO));
    w.clear_linear(gpu, wgpu::Color::BLACK);
    w
}

fn linha_do_meio(gpu: &GpuContext, mundo: &WorldRt) -> Vec<u8> {
    let alinhado = (LADO * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test leitura do mundo"),
        size: u64::from(alinhado * LADO),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        mundo.texture().as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(alinhado),
                rows_per_image: Some(LADO),
            },
        },
        wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let px = buf.slice(..).get_mapped_range().expect("mapa lido");
    let linha = (LADO / 2 * alinhado) as usize;
    (0..LADO as usize)
        .map(|x| {
            px[linha + x * 4..linha + x * 4 + 3]
                .iter()
                .copied()
                .max()
                .unwrap_or(0)
        })
        .collect()
}

/// Pintado em `x` (pixel)?
fn pinta(linha: &[u8], x: usize) -> bool {
    linha[x] > 128
}

fn forma(h: u32, x: f32) -> VectorInstance {
    VectorInstance {
        geometry_id: h,
        texture_id: 0,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        premultiplied: 0.0,
        world_pos: [x, 0.0],
        size: [2.0, 2.0],
        basis: [1.0, 0.0, 0.0, 1.0],
        tint: [1.0; 4],
        anchor: [0.0, 0.0],
        sampling: 0,
        blend_linha: 0,
        mistura: Default::default(),
    }
}

fn faixa(
    sink: NodeId,
    instancias: std::ops::Range<usize>,
    vetores: std::ops::Range<usize>,
) -> FaixaDoSink {
    FaixaDoSink {
        sink,
        instancias,
        vetores,
        ladrilhos: 0..0,
    }
}

/// ⭐ **A faixa Motion de BAIXO cola depois do `desenha`** (curado em `9a88830e50`): o `desenha` do
/// presente consome a decisão (`ativa`); a faixa tem de perguntar `desenhou`. Mundo: `BAIXO` em
/// `x = −4`, sem dono em `0`, `CIMA` em `+4` (16 px por unidade). A faixa de baixo pinta só `BAIXO`;
/// o topo, o resto. (Mutação: `placa.desenhou()` → `placa.ativa()` na `cola_a_faixa` ⇒ RED.)
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_faixa_motion_de_baixo_cola_depois_do_desenha() {
    let gpu = gpu();
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let (mut m, s) = motion();
    let mut store = VecPathStore::default();
    let h = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    m.pump.vector_instances = vec![forma(h, -4.0), forma(h, 0.0), forma(h, 4.0)];
    m.pump.faixas = vec![
        faixa(s[0], 0..0, 0..1),
        faixa(s[2], 0..0, 1..2),
        faixa(s[1], 0..0, 2..3),
    ];
    let k = f64::from(LADO) / 16.0;
    let cam = Affine::translate((f64::from(LADO) * 0.5, f64::from(LADO) * 0.5))
        * Affine::scale_non_uniform(k, -k);
    let mut placa = PlacaDeFormas::default();
    let mut geos = GeometriasDaPlaca::default();
    let insts = m.pump.vector_instances.clone();
    assert!(
        placa.decide(true, &insts, &store, &mut geos, cam),
        "CONTROLO: o quadro vai à placa"
    );
    // O presente desenha a placa ANTES das faixas — e isso consome a decisão.
    assert!(placa.desenha(&gpu, (LADO, LADO), &geos, None).is_some());
    assert!(!placa.ativa(), "CONTROLO: o `desenha` consumiu a decisão");
    let (ordem, bands) = ordem();
    let c = Colagem {
        gpu: &gpu,
        ordem: &ordem,
        bands: &bands,
        sprites: false,
    };
    let (x_baixo, x_sem, x_cima) = (128 - 64, 128, 128 + 64);
    let baixo = mundo(&gpu);
    c.cola_a_faixa(&mut placa, &m, &blit, baixo.blend_view(), bands[0]);
    let l = linha_do_meio(&gpu, &baixo);
    eprintln!(
        "  faixa de baixo: {} {} {}",
        l[x_baixo], l[x_sem], l[x_cima]
    );
    assert!(
        pinta(&l, x_baixo),
        "a faixa de baixo cola as formas de BAIXO"
    );
    assert!(!pinta(&l, x_sem) && !pinta(&l, x_cima), "e só as dela");
    let topo = mundo(&gpu);
    c.cola_a_faixa(&mut placa, &m, &blit, topo.blend_view(), bands[2]);
    let l = linha_do_meio(&gpu, &topo);
    eprintln!("  topo: {} {} {}", l[x_baixo], l[x_sem], l[x_cima]);
    assert!(!pinta(&l, x_baixo), "o topo não repete as de BAIXO");
    assert!(pinta(&l, x_sem) && pinta(&l, x_cima), "o topo leva o resto");
}

fn sprite(x: f32) -> RenderInstance {
    let mut r: RenderInstance = bytemuck::Zeroable::zeroed();
    r.world_pos = [x, 0.0];
    r.size = [1.5, 1.5];
    r.atlas_uv = ph2d_render::WHITE_TILE_UV;
    r.tint = [1.0; 4];
    r.basis = RenderInstance::IDENTITY_BASIS;
    r.opacity = 1.0;
    r.uv_xform = RenderInstance::IDENTITY_UV_XFORM;
    r.per_corner_tint = [[1.0; 4]; 4];
    r.clip_group = RenderInstance::CLIP_GROUP_NONE;
    r
}

/// ⭐ GEF 19 A — **as cópias de SPRITE colam na faixa do dono**: com a ordem intercalada, a faixa de
/// baixo desenha só as imagens de `BAIXO` e o topo o resto (as sem dono incluídas). Mundo: `BAIXO`
/// em `x = −3`, sem dono em `0`, `CIMA` em `+3` (`Camera2d` por omissão: 25,6 px por unidade).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn as_copias_de_sprite_colam_na_faixa_do_dono() {
    let gpu = gpu();
    let (mut m, s) = motion();
    m.pump.instances = vec![sprite(-3.0), sprite(0.0), sprite(3.0)];
    m.pump.faixas = vec![
        faixa(s[0], 0..1, 0..0),
        faixa(s[2], 1..2, 0..0),
        faixa(s[1], 2..3, 0..0),
    ];
    let (baixo, topo) = colhe_as_duas(&gpu, &m);
    let (x_baixo, x_sem, x_cima) = (128 - 77, 128, 128 + 77);
    let l = baixo;
    eprintln!(
        "  faixa de baixo: {} {} {}",
        l[x_baixo], l[x_sem], l[x_cima]
    );
    assert!(
        pinta(&l, x_baixo),
        "a faixa de baixo desenha as imagens de BAIXO"
    );
    assert!(!pinta(&l, x_sem) && !pinta(&l, x_cima), "e só as dela");
    let l = topo;
    eprintln!("  topo: {} {} {}", l[x_baixo], l[x_sem], l[x_cima]);
    assert!(!pinta(&l, x_baixo), "o topo não repete as de BAIXO");
    assert!(pinta(&l, x_sem) && pinta(&l, x_cima), "o topo leva o resto");
}

/// O `cola_os_sprites` da faixa de baixo e o do topo, cada um num mundo limpo: as linhas do meio.
fn colhe_as_duas(gpu: &GpuContext, m: &MotionState) -> (Vec<u8>, Vec<u8>) {
    let blit = BandBlit::new(gpu, WorldRt::FORMAT);
    let atlas = ph2d_render::TextureAtlas::new(gpu, 128);
    let mut renderer =
        ph2d_render::SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let game_rt = ph2d_render::GameRt::new(gpu, (LADO, LADO));
    let mut tonemap = ph2d_render::Tonemap::new(gpu, game_rt.view().clone(), (LADO, LADO));
    let camera = Camera2d::default();
    let (ordem, bands) = ordem();
    assert!(
        sprites_por_faixa(true, &bands),
        "CONTROLO: a ordem é intercalada"
    );
    let c = Colagem {
        gpu,
        ordem: &ordem,
        bands: &bands,
        sprites: true,
    };
    let mut colhe = |band| {
        let mundo = mundo(gpu);
        let mut e = Engrenagem {
            gpu,
            renderer: &mut renderer,
            camera: &camera,
            window: ph2d_host::WindowSize {
                width: LADO,
                height: LADO,
            },
            scene_viewport: None,
            formas: None,
            mundo: &mundo,
            band_blit: &blit,
        };
        let passe = PasseDeSprites {
            game_rt: &game_rt,
            tonemap: &mut tonemap,
        };
        c.cola_os_sprites(m, &mut e, passe, band);
        linha_do_meio(gpu, &mundo)
    };
    (colhe(bands[0]), colhe(bands[2]))
}

/// Dois objectos Motion (`BAIXO` deslocado para `x = −3`, `CIMA` para `+3`) e uma cadeia sem dono
/// (em `0`), cada um um ladrilho branco do átlas numa grelha 1×1, cozidos NA PLACA pelo quadro do produto.
fn na_placa(gpu: &GpuContext) -> MotionState {
    use ph2d_nodegraph::attr::{Column, Stream};
    let mut m = MotionState::new();
    m.pump.cook.set_external(
        "Particle",
        Stream::new(1)
            .with("texture_id", Column::Scalar(vec![0.0]))
            .with("uv_rect", Column::Vec4(vec![ph2d_render::WHITE_TILE_UV]))
            .with("size", Column::Vec2(vec![[1.5, 1.5]])),
    );
    let cadeias = [(Some(BAIXO), -3.0f32), (None, 0.0), (Some(CIMA), 3.0)];
    for (k, (dono, x)) in cadeias.into_iter().enumerate() {
        let g = &mut m.doc.graph;
        let obj = g.add_node("source.object");
        g.set_text_param(obj, "object", "Particle");
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", 1.0);
        let dup = g.add_node("motion.duplicator");
        let out = g.add_node("motion.output");
        for (a, porta, b) in [(obj, 0, dup), (grid, 1, dup), (dup, 0, out)] {
            g.connect(ph2d_nodegraph::graph::Edge {
                from: (a, 0),
                to: (b, porta),
                delayed: false,
            })
            .expect("ligar");
        }
        for (i, n) in [obj, grid, dup, out].into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "oito nos")]
            let p = ph2d_nodegraph::graph::Pos {
                x: i as f32 * 200.0,
                y: k as f32 * 300.0,
            };
            m.doc.graph.set_pos(n, p);
            if let Some(d) = dono {
                m.doc.set_owner(n, d);
            }
        }
        let Some(dono) = dono else { continue };
        m.objeto.donos.insert(
            dono,
            Dono {
                entity: dono,
                visible: true,
                afim: [1.0, 0.0, 0.0, 1.0, x, 0.0],
            },
        );
    }
    m.doc.graph.validate(&m.registry).expect("bem tipado");
    m.gpu_enabled = true;
    let dt = 1.0 / 60.0;
    for t in 0..2u32 {
        let mut p = ph2d_core::Playhead::new(dt);
        p.seek(f64::from(t) * dt);
        let mao = crate::motion_bridge::quadro::MaoDoEditor::default();
        crate::motion_bridge::quadro::o_quadro(&mut m, Some(gpu), &p, dt, mao);
    }
    assert!(
        m.gpu_live,
        "CONTROLO: a cena tem de cozer na PLACA, senão o gate mede a CPU — {:?}",
        m.route_said
    );
    m
}

/// ⭐ GEF 19 A — **o RAMO DO DISPOSITIVO** de `cola_os_sprites`: as cópias de sprite cozidas na placa
/// (o buffer do cozimento, os `runs` cortados por `corta_runs`/`leva`) colam na faixa do dono.
/// (Mutações: `leva` com o topo e o de baixo trocados ⇒ RED; `corta_runs` sem cortar ⇒ RED. Sem a
/// cadeia sem dono a troca do `leva` sobrevivia: com dois donos as duas ramas dão o mesmo.)
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn as_copias_de_sprite_cozidas_na_placa_colam_na_faixa_do_dono() {
    let gpu = gpu();
    let m = na_placa(&gpu);
    let (_, n, _) = (m.sprites_da_camada(true).0).expect("CONTROLO: as sprites vêm da placa");
    assert_eq!(
        n, 3,
        "CONTROLO: uma cópia por cadeia no buffer do cozimento"
    );
    let linhas = ph2d_gpu_cook::read_instances(&gpu, m.gpu_cook.instances().expect("instâncias"));
    let (donos, _) = m.gpu_cook.faixas_dos_sinks();
    let de = |s| crate::motion_objeto::origem::dono_do_no(&m, s).map(|d| d.entity);
    // `Camera2d` por omissão: 25,6 px por unidade, a origem no meio.
    let px = |dono| {
        let (_, r) = (donos.iter().find(|(s, _)| de(*s) == Some(dono))).expect("o sink do dono");
        let x = 128.0 + linhas[r.start as usize].world_pos[0] * 25.6;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "um pixel"
        )]
        let x = x as usize;
        x
    };
    let (x_baixo, x_sem, x_cima) = (px(BAIXO), 128, px(CIMA));
    let (baixo, topo) = colhe_as_duas(&gpu, &m);
    eprintln!(
        "  x {x_baixo}/{x_sem}/{x_cima} · faixa de baixo: {} {} {} · topo: {} {} {}",
        baixo[x_baixo], baixo[x_sem], baixo[x_cima], topo[x_baixo], topo[x_sem], topo[x_cima]
    );
    assert!(
        x_cima > x_baixo + 40,
        "CONTROLO: os dois objectos estão separados"
    );
    assert!(
        pinta(&baixo, x_baixo),
        "a faixa de baixo desenha a cópia de BAIXO"
    );
    assert!(
        !pinta(&baixo, x_sem) && !pinta(&baixo, x_cima),
        "e só a dela"
    );
    assert!(!pinta(&topo, x_baixo), "o topo não repete a de BAIXO");
    assert!(
        pinta(&topo, x_sem) && pinta(&topo, x_cima),
        "o topo leva o resto (a sem dono incluída)"
    );
}
