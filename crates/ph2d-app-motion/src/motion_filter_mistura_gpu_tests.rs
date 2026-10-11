//! ⭐⭐⭐ **A MISTURA do `Output` atravessa a camada filtrada** (ciclo 20 W4b, doc 129 §3.5): a tabela
//! do doc 118 (a fixtura irmã, [`crate::motion_shape_gen::mistura::gpu_tests`]) pintada pela porta do
//! PRODUTO — a cena vectorial do Motion na camada ([`super::MotionFiltro::cena_da_camada`]),
//! [`MotionState::filtra_a_camada`] com uma pilha NEUTRA e a pousa com o modo — tem de sair igual ao
//! quadro de hoje sem filtro (o Vello com o cenário por baixo), em cada modo × cada alcance.
//!
//! ⚠️ O cenário aqui é o MUNDO (o acumulador limpo a `B`), e não uma cópia na cena: é por baixo da
//! camada que ele está no produto.

use ph2d_eval_motion::{BlendWith, MisturaDoSink};
use ph2d_render::{BandBlit, Camera2d, VelloPass, WorldRt};
use ph2d_vector::Affine;

use super::Engrenagem;
use crate::motion_shape_gen::mistura::gpu_tests::{
    A1, A2, B, BARRA, COLUNAS, H, W, byte, copia, esperado, mede, try_headless_gpu,
};
use crate::motion_shape_gen::{VecPathStore, encode};
use crate::motion_state::MotionState;

fn srgb_para_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn le_linha(gpu: &ph2d_gpu::GpuContext, mundo: &WorldRt) -> [i32; 4] {
    let alinhado =
        (W * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test leitura do mundo"),
        size: u64::from(alinhado * H),
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
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let px = buf.slice(..).get_mapped_range();
    let linha = (H / 2 * alinhado) as usize;
    // Cinzento: a ordem `BGRA` do mundo não importa.
    COLUNAS.map(|x| i32::from(px[linha + x as usize * 4]))
}

/// O `Output` com o modo e o alcance, um `fx.filter` `Each Shape` (um borrão de raio `raio`; zero =
/// NEUTRO) no fio.
fn motion_com_filtro(tag: u8, com: BlendWith, raio: f32) -> MotionState {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let filtro = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    g.set_param(filtro, ph2d_node_fx_filter::param::RADIUS, raio);
    let saida = g.add_node("motion.output".to_string());
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (filtro, 0),
        to: (saida, 0),
        delayed: false,
    })
    .expect("liga");
    g.set_param(saida, ph2d_eval_motion::SINK_BLEND_PARAM, f32::from(tag));
    let alcance = BlendWith::ALL.iter().position(|c| *c == com).unwrap_or(0);
    #[allow(clippy::cast_precision_loss)]
    g.set_param(
        saida,
        ph2d_eval_motion::SINK_BLEND_WITH_PARAM,
        alcance as f32,
    );
    m.sinks = vec![saida];
    m.filtro.prepara(&m.doc.graph, &m.sinks);
    m.pump.isola_na_camada(true);
    m
}

/// Pinta as duas cópias pela camada filtrada sobre o mundo `B` e lê as quatro colunas.
fn pela_camada(
    gpu: &ph2d_gpu::GpuContext,
    renderer: &mut ph2d_render::SpriteRenderer,
    blit: &BandBlit,
    tag: u8,
    com: BlendWith,
    raio: f32,
    inteira: bool,
) -> ([i32; 4], MotionState) {
    let mut m = motion_com_filtro(tag, com, raio);
    if inteira {
        // O degrau em `Whole Layer`: o Vello cola-se na camada sem passar pelos `Each Shape`.
        let no = m.filtro.pilha()[0].no;
        m.doc
            .graph
            .set_param(no, ph2d_node_fx_filter::param::SCOPE, 1.0);
        m.filtro.prepara(&m.doc.graph, &m.sinks);
    }
    let mut mistura = MisturaDoSink {
        blend: tag,
        com,
        sink: 1,
    };
    // A regra da bomba (`MotionCookPump::pump`): o `Scene` pousa e não agrupa.
    if mistura.pousa_na_camada(m.pump.camada_isolada()) {
        mistura.blend = 0;
    }
    let mut store = VecPathStore::default();
    let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    let insts = vec![
        copia(16.0, 24.0, A1, mistura, None),
        copia(32.0, 24.0, A2, mistura, None),
    ];
    let mut art = |_: u32, _: [f32; 4]| None;
    let cena = m
        .filtro
        .cena_da_camada(true)
        .expect("com filtro, a cena vai à camada");
    encode(
        &insts,
        &store,
        &mut art,
        Affine::IDENTITY,
        None,
        ph2d_render::ImageFilterMode::Smooth,
        cena,
    );
    m.pump.vector_instances = insts;
    let mundo = WorldRt::new(gpu, (W, H));
    let b = srgb_para_linear(f64::from(B));
    mundo.clear_linear(
        gpu,
        wgpu::Color {
            r: b,
            g: b,
            b,
            a: 1.0,
        },
    );
    assert!(m.filtro.liga(true));
    assert!(m.filtra_a_camada(Engrenagem {
        gpu,
        renderer,
        camera: &Camera2d::default(),
        window: ph2d_host::WindowSize {
            width: W,
            height: H,
        },
        scene_viewport: None,
        formas: None,
        mundo: &mundo,
        band_blit: blit,
    }));
    (le_linha(gpu, &mundo), m)
}

#[test]
#[ignore = "placa: precisa de GPU"]
fn the_output_blend_crosses_the_filtered_layer_as_it_paints_without_it() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let Ok(mut vello) = VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (W, H)) else {
        println!("sem VelloPass — saltado");
        return;
    };
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let mut falhas = Vec::new();
    // Os dois ramos da colagem do Vello na camada: com `Each Shape` (a pilha sobre o conjunto) e
    // sem ele (a colagem directa).
    for inteira in [false, true] {
        // ⚠️ O CONTROLO primeiro: sem modo a camada pinta as cópias por cima do cenário.
        let (normal, _) = pela_camada(
            &gpu,
            &mut renderer,
            &blit,
            0,
            BlendWith::Everything,
            0.0,
            inteira,
        );
        let alvo = esperado(0, None).map(byte);
        assert!(
            normal.iter().zip(alvo).all(|(m, e)| (m - e).abs() <= BARRA),
            "sem mistura a camada leu {normal:?} contra {alvo:?} — a fixtura não pinta o que diz"
        );
        for tag in [1u8, 3, 4] {
            for com in BlendWith::ALL {
                let (lido, _) = pela_camada(&gpu, &mut renderer, &blit, tag, com, 0.0, inteira);
                let hoje = mede(&mut vello, &gpu, tag, com, false, (1, 1));
                let tabela = esperado(tag, Some(com)).map(byte);
                println!(
                    "  tag {tag} {com:?}: camada {lido:?} · sem filtro {hoje:?} · tabela {tabela:?}"
                );
                if lido.iter().zip(tabela).any(|(m, e)| (m - e).abs() > BARRA)
                    || lido.iter().zip(hoje).any(|(m, e)| (m - e).abs() > 1)
                {
                    falhas.push(format!(
                    "inteira {inteira} tag {tag} {com:?}: camada {lido:?} · sem filtro {hoje:?} · \
                     tabela {tabela:?}"
                ));
                }
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "a camada filtrada não pinta a mistura de sempre (barra {BARRA} contra a tabela, 1 contra o \
         quadro sem filtro):\n{}",
        falhas.join("\n")
    );
}

/// ⭐ **O `Each Shape` sobre as formas do Vello corre — sobre o CONJUNTO** (doc 129 §3.5): o Vello não
/// tem células, e a cena `=135` abre com um relevo em `Each Shape` — sem este ramo, uma saída em
/// `Add` com `Everything` (ou `Copies`) perdia-o em silêncio. ⚠️ O controlo: o mesmo borrão com raio zero não muda
/// nada (a fixtura mede o filtro, não outra coisa), e o cartão do `Each Shape` di-lo.
#[test]
#[ignore = "placa: precisa de GPU"]
fn an_each_shape_step_runs_on_the_shapes_the_vector_path_draws() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    // `Screen` em `Copies` (vai ao Vello, e não satura como o `Add`), um borrão largo face às cópias.
    let (neutro, _) = pela_camada(&gpu, &mut renderer, &blit, 4, BlendWith::Copies, 0.0, false);
    let (borrado, m) = pela_camada(
        &gpu,
        &mut renderer,
        &blit,
        4,
        BlendWith::Copies,
        20.0,
        false,
    );
    println!("  neutro {neutro:?} · borrado {borrado:?}");
    assert!(
        neutro
            .iter()
            .zip(borrado)
            .any(|(a, b)| (a - b).abs() > 4 * BARRA),
        "o borrão `Each Shape` não mudou as formas do Vello: {neutro:?} · {borrado:?}"
    );
    let no = m.filtro.pilha()[0].no;
    let q = m.filtro.queixa(no).expect("o cartão do `Each Shape` diz");
    assert!(q.contains("one piece"), "{q}");
}

/// ⭐ **O PREÇO do Vello dentro da camada** (doc 129 §5.8) — sonda manual, `--release --nocapture`:
/// o quadro da camada filtrada (pilha neutra) a `1600×972` e a 4K com `n` discos pelo Vello (`Add` com
/// tudo), contra o MESMO quadro sem formas. Blocos intercalados, o MÍNIMO (MEDIR_VELOCIDADE.md).
#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn o_preco_do_vello_na_camada() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    for (w, h) in [(1600u32, 972u32), (3840, 2160)] {
        let mut renderer = ph2d_render::SpriteRenderer::new(
            gpu.clone(),
            ph2d_render::GameRt::FORMAT,
            ph2d_render::TextureAtlas::dummy(&gpu),
            8,
        );
        let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
        let mundo = WorldRt::new(&gpu, (w, h));
        let mut store = VecPathStore::default();
        let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
        println!(
            "\ncarga: {}",
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
        for n in [256usize, 4096] {
            let lado = (n as f32).sqrt().ceil() as usize;
            let mistura = MisturaDoSink {
                blend: 1,
                com: BlendWith::Everything,
                sink: 1,
            };
            #[allow(clippy::cast_precision_loss)]
            let insts: Vec<_> = (0..n)
                .map(|i| {
                    let mut c = copia(0.0, 12.0, A1, mistura, None);
                    c.world_pos = [
                        200.0 + (i % lado) as f32 * 1200.0 / lado as f32,
                        100.0 + (i / lado) as f32 * 770.0 / lado as f32,
                    ];
                    c.size = [14.0, 14.0];
                    c
                })
                .collect();
            let mut minimo = [f64::MAX; 2];
            // Um Motion por variante, criado UMA vez (os passes e o Vello da camada nascem no 1.º
            // quadro, que é o aquecimento e não conta).
            let mut ms = [0, 1].map(|_| motion_com_filtro(1, BlendWith::Everything, 0.0));
            for bloco in 0..13 {
                for variante in [bloco % 2, 1 - bloco % 2] {
                    let m = &mut ms[variante];
                    if variante == 1 {
                        let mut art = |_: u32, _: [f32; 4]| None;
                        let cena = m.filtro.cena_da_camada(true).expect("a cena");
                        encode(
                            &insts,
                            &store,
                            &mut art,
                            Affine::IDENTITY,
                            None,
                            ph2d_render::ImageFilterMode::Smooth,
                            cena,
                        );
                        m.pump.vector_instances.clone_from(&insts);
                    }
                    assert!(m.filtro.liga(true));
                    let t = std::time::Instant::now();
                    m.filtra_a_camada(Engrenagem {
                        gpu: &gpu,
                        renderer: &mut renderer,
                        camera: &Camera2d::default(),
                        window: ph2d_host::WindowSize {
                            width: w,
                            height: h,
                        },
                        scene_viewport: None,
                        formas: None,
                        mundo: &mundo,
                        band_blit: &blit,
                    });
                    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                    if bloco > 0 {
                        minimo[variante] = minimo[variante].min(t.elapsed().as_secs_f64() * 1e3);
                    }
                }
            }
            println!(
                "  {w}×{h} {n:>5} discos: sem formas {:.2} ms · com o Vello na camada {:.2} ms",
                minimo[0], minimo[1]
            );
        }
    }
}

/// ⭐ **O passo do quadro liga a camada isolada na bomba** (`coze_o_quadro`): com `fx.filter` no fio
/// a bomba fica isolada, sem ele não — senão o `Scene` agrupava e ia ao Vello com a camada ligada.
#[test]
#[ignore = "placa: precisa de GPU"]
fn the_frame_step_isolates_the_pump_when_a_filter_is_on_the_wire() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let playhead = ph2d_core::Playhead::new(1.0 / 60.0);
    let mut com = motion_com_filtro(1, BlendWith::Scene, 0.0);
    com.pump.isola_na_camada(false);
    crate::motion_bridge::quadro::coze_o_quadro(&mut com, &gpu, &playhead, 1.0 / 60.0);
    assert!(
        com.pump.camada_isolada(),
        "com filtro no fio a bomba isola-se"
    );
    let mut sem = MotionState::new();
    let saida = sem.doc.graph.add_node("motion.output".to_string());
    sem.sinks = vec![saida];
    sem.pump.isola_na_camada(true);
    crate::motion_bridge::quadro::coze_o_quadro(&mut sem, &gpu, &playhead, 1.0 / 60.0);
    assert!(
        !sem.pump.camada_isolada(),
        "sem filtro a bomba não se isola"
    );
}
