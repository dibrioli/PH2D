//! ⭐⭐⭐ **P4 (b) — O MUNDO POR BAIXO, pela placa** (doc 132 §4): o `Everything`, o `Scene` e a linha
//! sem grupo FORA da camada isolada, desenhados pela placa de formas e POUSADOS no acumulador do mundo
//! com a mistura de hardware ([`ph2d_render::BandBlit::blit_pousa`], a porta do presente) — contra a
//! tabela fechada do doc 118 §1 e contra o Vello, que mistura com o cenário na cena dele.
//!
//! A fixtura é a da tabela ([`crate::motion_shape_gen::mistura::gpu_tests`]): o cenário `B` (aqui o
//! MUNDO, o acumulador limpo a `B`, como no produto) e duas cópias opacas que se sobrepõem, lidas em
//! quatro colunas. ⭐ O controlo vem primeiro: a mesma camada pousada em `Normal` (o que o presente
//! fazia) tem de DIFERIR da tabela — senão a fixtura não distingue a pousa.
//!
//! `#[ignore]`: precisa de adaptador real.

use ph2d_eval_motion::{BlendWith, MisturaDoSink, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_render::{BandBlit, BandSource, Pousa, VelloPass, WorldRt};
use ph2d_vector::Affine;

use super::{GeometriasDaPlaca, Misturas, PlacaDeFormas, corridas_das_faixas};
use crate::motion_shape_gen::VecPathStore;
use crate::motion_shape_gen::mistura::gpu_tests::{
    A1, A2, B, BARRA, COLUNAS, H, W, byte, copia, esperado, mede,
};

fn srgb_para_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// O canal R das quatro colunas da linha do meio do mundo.
fn le_colunas(gpu: &GpuContext, mundo: &WorldRt) -> [i32; 4] {
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
    // Cinzentos: a ordem `BGRA` do mundo não importa.
    COLUNAS.map(|x| i32::from(px[linha + x as usize * 4]))
}

/// As cópias como o cozimento as escreve: `[ShapeInstance]` num buffer `STORAGE`.
fn buffer_do_cozimento(gpu: &GpuContext, insts: &[VectorInstance]) -> wgpu::Buffer {
    let copias: Vec<ph2d_shape_gpu::ShapeInstance> = insts
        .iter()
        .map(|i| ph2d_shape_gpu::ShapeInstance {
            pos: i.world_pos,
            size: i.size,
            basis: i.basis,
            anchor: i.anchor,
            geometry: i.geometry_id,
            _pad: 0,
            tint: i.tint,
        })
        .collect();
    let bytes: &[u8] = bytemuck::cast_slice(&copias);
    let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test cópias do cozimento"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue.write_buffer(&b, 0, bytes);
    b
}

/// Por onde as cópias chegam à placa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rota {
    Cpu,
    Dispositivo,
}

/// A placa desenha as cópias e a camada pousa no mundo `B` pela porta do presente
/// ([`super::cola_no_mundo`], segmento a segmento) — ou toda com `forca` (o controlo). `None` se a
/// placa recusa. Na rota do dispositivo cada cópia é a faixa de um sink com a mistura DELA.
fn pela_placa(
    gpu: &GpuContext,
    blit: &BandBlit,
    insts: &[VectorInstance],
    rota: Rota,
    forca: Option<Pousa>,
) -> Option<([i32; 4], Pousa)> {
    let mut store = VecPathStore::default();
    let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    let misturas = Misturas::do_quadro(gpu, false);
    let mut placa = PlacaDeFormas::default();
    let mut geos = GeometriasDaPlaca::default();
    let mut buffer = None;
    let vai = match rota {
        Rota::Cpu => {
            placa.decide_com_misturas(true, insts, &store, &mut geos, Affine::IDENTITY, misturas)
        }
        Rota::Dispositivo => {
            let mut g = ph2d_nodegraph::graph::Graph::new();
            let sinks: Vec<_> = insts.iter().map(|_| g.add_node("motion.output")).collect();
            let faixas: Vec<_> = (0u32..)
                .zip(&sinks)
                .map(|(k, &s)| (s, k..k + 1, 0..0))
                .collect();
            let de = |s| insts[sinks.iter().position(|&x| x == s).unwrap_or(0)].mistura;
            let r = corridas_das_faixas(&faixas, de, misturas).ok()?;
            buffer = Some(buffer_do_cozimento(gpu, insts));
            let vivas: Vec<u32> = insts.iter().map(|i| i.geometry_id).collect();
            let n = u32::try_from(insts.len()).expect("poucas");
            placa.decide_do_dispositivo_com_pousa(
                true,
                n,
                &vivas,
                &store,
                &mut geos,
                Affine::IDENTITY,
                r,
            )
        }
    };
    if !vai {
        return None;
    }
    let pousa = placa.pousa();
    let camada = placa.desenha(gpu, (W, H), &geos, buffer.as_ref())?.clone();
    let mundo = mundo_b(gpu);
    match forca {
        Some(f) => blit.blit_pousa(
            gpu,
            mundo.blend_view(),
            &camada,
            BandSource::Formas,
            None,
            f,
        ),
        None => super::cola_no_mundo(&mut placa, gpu, blit, mundo.blend_view()),
    }
    Some((le_colunas(gpu, &mundo), pousa))
}

/// O mundo limpo a `B`.
fn mundo_b(gpu: &GpuContext) -> WorldRt {
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
    mundo
}

/// ⭐⭐ **P4 (b) — pousas diferentes no mesmo quadro, por SEGMENTOS** (doc 132 §4.1): a camada
/// desenha cada segmento e pousa-o por sua vez, na ordem das linhas. Fixtura: a A (`A1`) e a B (`A2`)
/// em sinks DIFERENTES — um `Everything` em `f` e um `Normal` (as duas ordens), e dois `Scene` em `f`.
/// Tabela fechada: `[f(B, A1), A2, A2, B]` · `[A1, f(A1, A2), f(B, A2), B]` · `[f(B, A1), f(f(B, A1),
/// A2), f(B, A2), B]`. ⚠️ Controlo: a camada inteira pousada no modo da 1.ª sai da tabela.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn as_pousas_diferentes_pousam_por_segmentos() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a placa");
    };
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let mut casos = 0;
    for tag in [1u8, 3, 4] {
        let ms = |blend, com, sink| MisturaDoSink { blend, com, sink };
        let f = |x, y| crate::motion_shape_gen::mistura::gpu_tests::f(tag, x, y);
        let arranjos = [
            (
                "Everything + Normal",
                [
                    ms(tag, BlendWith::Everything, 0),
                    ms(0, BlendWith::Copies, 1),
                ],
                [f(B, A1), A2, A2, B],
            ),
            (
                "Normal + Everything",
                [
                    ms(0, BlendWith::Copies, 0),
                    ms(tag, BlendWith::Everything, 1),
                ],
                [A1, f(A1, A2), f(B, A2), B],
            ),
            (
                "Scene + Scene",
                [ms(tag, BlendWith::Scene, 0), ms(tag, BlendWith::Scene, 1)],
                [f(B, A1), f(f(B, A1), A2), f(B, A2), B],
            ),
        ];
        for (nome, [ma, mb], quer) in arranjos {
            let insts = [
                copia(16.0, 24.0, A1, ma, None),
                copia(32.0, 24.0, A2, mb, None),
            ];
            let quer = quer.map(byte);
            for rota in [Rota::Cpu, Rota::Dispositivo] {
                let caso = format!("tag {tag} {nome} {rota:?}");
                let (lido, pousa) = pela_placa(&gpu, &blit, &insts, rota, None)
                    .unwrap_or_else(|| panic!("{caso}: a placa compõe-no"));
                let (inteira, _) = pela_placa(&gpu, &blit, &insts, rota, Some(pousa)).expect("ctl");
                eprintln!("{caso}: placa {lido:?} · tabela {quer:?} · camada inteira {inteira:?}");
                assert!(
                    lido.iter().zip(&quer).all(|(l, q)| (l - q).abs() <= BARRA),
                    "{caso}: {lido:?} contra a tabela {quer:?}"
                );
                assert!(
                    inteira
                        .iter()
                        .zip(&quer)
                        .any(|(l, q)| (l - q).abs() > 4 * BARRA),
                    "{caso}: CONTROLO — uma pousa só tem de sair da tabela ({inteira:?})"
                );
                casos += 1;
            }
        }
    }
    assert_eq!(casos, 18);
}

/// ⭐⭐⭐ **O `Everything`, o `Scene` e a linha sem grupo fora da camada isolada: a placa pousa no mundo
/// o que o Vello pinta** — cada modo, pelas duas rotas da placa (a da CPU e a do dispositivo, que
/// recebe as corridas e a pousa por faixa), contra a tabela fechada (`esperado`) e contra o Vello
/// (`mede`), à barra de ARREDONDAMENTO da tabela (`BARRA`, a do doc 118 W1).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn o_mundo_por_baixo_pousa_pela_placa_como_o_vello_o_pinta() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a placa");
    };
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let mut vp = VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (W, H)).expect("vello");
    let mut casos = 0;
    for tag in [1u8, 3, 4] {
        for (com, linha) in [
            (BlendWith::Everything, false),
            (BlendWith::Scene, false),
            (BlendWith::Everything, true),
        ] {
            let caso = format!("tag {tag} {com:?} linha={linha}");
            // A linha sem grupo: o modo na LINHA (`blend_linha = tag + 1`) e o sink em `Normal`.
            let m = MisturaDoSink {
                blend: if linha { 0 } else { tag },
                com,
                sink: 1,
            };
            let mut insts = [
                copia(16.0, 24.0, A1, m, None),
                copia(32.0, 24.0, A2, m, None),
            ];
            if linha {
                for i in &mut insts {
                    i.blend_linha = tag + 1;
                }
            }
            let quer = esperado(tag, Some(com)).map(byte);
            let vello = mede(&mut vp, &gpu, tag, com, false, (1, 1));
            if !linha {
                assert!(
                    quer.iter().zip(&vello).all(|(q, v)| (q - v).abs() <= BARRA),
                    "{caso}: o Vello saiu da tabela ({vello:?} contra {quer:?})"
                );
            }
            let rotas: &[Rota] = if linha {
                // A linha com modo próprio e formas recusa o cozimento: só a rota da CPU a tem.
                &[Rota::Cpu]
            } else {
                &[Rota::Cpu, Rota::Dispositivo]
            };
            for &rota in rotas {
                let (lido, pousa) = pela_placa(&gpu, &blit, &insts, rota, None)
                    .unwrap_or_else(|| panic!("{caso} {rota:?}: a placa compõe-no"));
                let (normal, _) =
                    pela_placa(&gpu, &blit, &insts, rota, Some(Pousa::Normal)).expect("o controlo");
                eprintln!(
                    "{caso} {rota:?}: placa {lido:?} · pousa {pousa:?} · tabela {quer:?} · vello {vello:?} · pousada em Normal {normal:?}"
                );
                assert_ne!(pousa, Pousa::Normal, "{caso} {rota:?}");
                assert!(
                    lido.iter().zip(&quer).all(|(l, q)| (l - q).abs() <= BARRA),
                    "{caso} {rota:?}: {lido:?} contra a tabela {quer:?}"
                );
                assert!(
                    normal
                        .iter()
                        .zip(&quer)
                        .any(|(l, q)| (l - q).abs() > 4 * BARRA),
                    "{caso} {rota:?}: CONTROLO — pousada em Normal a fixtura tem de sair da tabela ({normal:?})"
                );
                casos += 1;
            }
        }
    }
    assert_eq!(casos, 15);
}

/// ⭐ **O PREÇO que a pousa poupa** (P4 (b), sonda manual, `--release --nocapture`): ler o mundo
/// por baixo exige uma CÓPIA dele — o acumulador é o alvo da colagem (o wgpu não lê e escreve a mesma
/// textura num passe) e nem tem `COPY_SRC`; a rota da CPU paga-a no Vello (`vello_fundo`: a cópia por
/// render para um `Rgba8Unorm` registado, marcado sujo todo quadro). Mede-se essa porta do produto
/// (`render_to_intermediate_over_world` contra `render_to_intermediate`, a MESMA cena vazia: a
/// diferença é o preço de ler o mundo) e a colagem com pousa (`blit_pousa` em `Normal`, `Soma` e
/// `Multiplica`, que são dois passes) a 1080p e a 4K — variantes no MESMO processo, blocos
/// intercalados com a ordem rodada, o MÍNIMO, a carga ao lado. Cada medida espera a placa.
#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn sonda_o_preco_de_ler_o_mundo_contra_a_pousa() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        return;
    };
    let carga = || {
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let espera = |gpu: &GpuContext| {
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    };
    let vazia = ph2d_vector::VectorScene::new();
    for (nome, tamanho) in [("1080p", (1920u32, 1080u32)), ("4K", (3840, 2160))] {
        let mundo = WorldRt::new(&gpu, tamanho);
        let camada = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sonda camada"),
            size: wgpu::Extent3d {
                width: tamanho.0,
                height: tamanho.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::FORMATO_DA_CAMADA,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let vista = camada.create_view(&wgpu::TextureViewDescriptor::default());
        let vp = std::cell::RefCell::new(
            VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, tamanho).expect("vello"),
        );
        let pousa = |p: Pousa| {
            blit.blit_pousa(
                &gpu,
                mundo.blend_view(),
                &vista,
                BandSource::Formas,
                None,
                p,
            )
        };
        type Variante<'a> = (&'static str, Box<dyn Fn() + 'a>);
        let variantes: [Variante<'_>; 5] = [
            (
                "Vello SEM o mundo (cena vazia)",
                Box::new(|| {
                    vp.borrow_mut()
                        .render_to_intermediate(
                            &gpu,
                            vazia.inner(),
                            tamanho,
                            ph2d_vector::Color::TRANSPARENT,
                        )
                        .expect("vello");
                }),
            ),
            (
                "Vello COM o mundo por baixo (cena vazia)",
                Box::new(|| {
                    vp.borrow_mut()
                        .render_to_intermediate_over_world(
                            &gpu,
                            vazia.inner(),
                            mundo.sample_view(),
                            tamanho,
                        )
                        .expect("vello");
                }),
            ),
            ("pousa Normal", Box::new(|| pousa(Pousa::Normal))),
            ("pousa Soma", Box::new(|| pousa(Pousa::Soma))),
            ("pousa Multiplica", Box::new(|| pousa(Pousa::Multiplica))),
        ];
        for (_, v) in &variantes {
            for _ in 0..5 {
                v();
                espera(&gpu);
            }
        }
        const BLOCOS: usize = 12;
        const VEZES: u32 = 10;
        let n = variantes.len();
        let mut min = vec![f64::MAX; n];
        for b in 0..BLOCOS {
            for k in 0..n {
                let i = (b + k) % n;
                let t = std::time::Instant::now();
                for _ in 0..VEZES {
                    variantes[i].1();
                    espera(&gpu);
                }
                min[i] = min[i].min(t.elapsed().as_secs_f64() * 1e3 / f64::from(VEZES));
            }
        }
        for (i, (v, _)) in variantes.iter().enumerate() {
            println!(
                "  {nome}: {v} {:.3} ms (min, com a espera da placa)",
                min[i]
            );
        }
        println!("  carga: {}", carga());
    }
}

/// ⭐⭐ **O `Scene` com imagens pela rota da CPU: a ordem das linhas** (doc 132 §4.1). As cópias de um
/// `Scene` juntam-se em `Normal` — a de cima TAPA a de baixo —, e a CPU, que sabe a média de cada linha,
/// parte a corrida a cada troca de média. Fixtura: a A IMAGEM (`A1`) e a B FORMA (`A2`), por esta ordem:
/// a sobreposição é a FORMA (`f(B, A2)`); com as imagens desenhadas depois das formas sairia a imagem.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn o_scene_com_imagens_pela_cpu_guarda_a_ordem_das_linhas() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a placa");
    };
    let mut atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let v = byte(A1) as u8;
    atlas
        .insert(&gpu, 1, 4, 4, &[v, v, v, 255].repeat(16))
        .expect("cabe");
    let renderer =
        ph2d_render::SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    for tag in [1u8, 3, 4] {
        let m = MisturaDoSink {
            blend: tag,
            com: BlendWith::Scene,
            sink: 0,
        };
        let mut a = copia(16.0, 24.0, A1, m, Some(0));
        a.atlas_uv = renderer.atlas().region_uv(1);
        let insts = [a, copia(32.0, 24.0, A2, m, None)];
        let mut store = VecPathStore::default();
        let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
        let mut placa = PlacaDeFormas::default();
        let mut geos = GeometriasDaPlaca::default();
        assert!(placa.decide_com_misturas(
            true,
            &insts,
            &store,
            &mut geos,
            Affine::IDENTITY,
            Misturas::do_quadro(&gpu, false),
        ));
        assert!(
            placa.corridas_do_quadro().len() >= 2,
            "uma corrida por média"
        );
        let mm = crate::motion_state::MotionState::new();
        let fontes = super::imagens::fontes_do_quadro(
            &placa,
            &gpu,
            &mm,
            &renderer,
            ph2d_render::ImageFilterMode::Smooth,
        );
        placa.com_imagens(fontes);
        let _ = placa.desenha(&gpu, (W, H), &geos, None).expect("desenha");
        let mundo = mundo_b(&gpu);
        super::cola_no_mundo(&mut placa, &gpu, &blit, mundo.blend_view());
        let lido = le_colunas(&gpu, &mundo);
        let f = |x, y| crate::motion_shape_gen::mistura::gpu_tests::f(tag, x, y);
        let quer = [f(B, A1), f(B, A2), f(B, A2), B].map(byte);
        eprintln!("Scene {tag}: lido {lido:?} · tabela {quer:?}");
        assert!(
            lido.iter().zip(&quer).all(|(l, q)| (l - q).abs() <= BARRA),
            "tag {tag}: {lido:?} contra {quer:?}"
        );
    }
}

/// ⭐⭐⭐ doc 132 §4.1 (3) — **o `Scene` com imagens PELO DISPOSITIVO guarda a ordem das linhas, com a
/// faixa do grupo DESLOCADA**: a disposição do cozimento (a faixa inteira nos dois buffers, a linha do
/// outro meio muda) DEPOIS de linhas de outros sinks em número diferente nas formas (3) e nas imagens
/// (1) — o deslocamento que a corrida ordenada leva no `first_vertex`. Fixtura: a A (`A1`) e a B
/// (`A2`), imagem + forma e forma + imagem; a B tapa a A nas duas. Tabela fechada:
/// `[f(B, A1), f(B, A2), f(B, A2), B]`.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn o_scene_com_imagens_pelo_dispositivo_segue_a_faixa_deslocada() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a placa");
    };
    let mut atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    for (k, cinza) in [(1u32, A1), (2, A2)] {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let v = byte(cinza) as u8;
        atlas
            .insert(&gpu, k, 4, 4, &[v, v, v, 255].repeat(16))
            .expect("cabe");
    }
    let renderer =
        ph2d_render::SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let uvs = [renderer.atlas().region_uv(1), renderer.atlas().region_uv(2)];
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let mut casos = 0;
    let mut falhas: Vec<String> = Vec::new();
    for (folga_f, folga_i) in [(0u32, 0u32), (3, 3), (3, 1), (1, 3)] {
        for imagem_em_cima in [false, true] {
            for tag in [1u8, 3, 4] {
                let m = MisturaDoSink {
                    blend: tag,
                    com: BlendWith::Scene,
                    sink: 0,
                };
                let insts: Vec<VectorInstance> = [(16.0, A1, 0usize), (32.0, A2, 1)]
                    .iter()
                    .map(|&(cx, cinza, k)| {
                        let imagem = (k == 1) == imagem_em_cima;
                        let mut c = copia(cx, 24.0, cinza, m, imagem.then_some(0));
                        if imagem {
                            c.atlas_uv = uvs[k];
                        }
                        c
                    })
                    .collect();
                let muda = VectorInstance {
                    geometry_id: ph2d_gpu_cook::lower_forma::SEM_GEOMETRIA,
                    ..insts[0]
                };
                let formas: Vec<ph2d_shape_gpu::ShapeInstance> =
                    std::iter::repeat_n(muda, folga_f as usize)
                        .chain(insts.iter().map(|i| VectorInstance {
                            geometry_id: if i.geometry_id > 0 {
                                i.geometry_id
                            } else {
                                ph2d_gpu_cook::lower_forma::SEM_GEOMETRIA
                            },
                            ..*i
                        }))
                        .map(|i| ph2d_shape_gpu::ShapeInstance {
                            pos: i.world_pos,
                            size: i.size,
                            basis: i.basis,
                            anchor: i.anchor,
                            geometry: i.geometry_id,
                            _pad: 0,
                            tint: i.tint,
                        })
                        .collect();
                let mut calada = super::linha_da_imagem(&insts[0]);
                calada.size = [0.0, 0.0];
                let linhas: Vec<ph2d_render::RenderInstance> =
                    std::iter::repeat_n(calada, folga_i as usize)
                        .chain(insts.iter().map(|i| {
                            let mut l = super::linha_da_imagem(i);
                            if i.geometry_id > 0 {
                                l.size = [0.0, 0.0];
                            }
                            l
                        }))
                        .collect();
                let buffer = |bytes: &[u8]| {
                    let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("test cozimento deslocado"),
                        size: bytes.len() as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    gpu.queue.write_buffer(&b, 0, bytes);
                    b
                };
                let (buf_f, buf_i) = (
                    buffer(bytemuck::cast_slice(&formas)),
                    buffer(bytemuck::cast_slice(&linhas)),
                );
                let corridas = corridas_das_faixas(
                    &[(
                        ph2d_nodegraph::graph::NodeId(0),
                        folga_f..folga_f + 2,
                        folga_i..folga_i + 2,
                    )],
                    |_| m,
                    Misturas::do_quadro(&gpu, false),
                )
                .expect("a placa compõe-na");
                assert!(corridas.0.iter().any(ph2d_shape_gpu::Corrida::ordenada));
                let mut store = VecPathStore::default();
                let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
                let mut placa = PlacaDeFormas::default();
                let mut geos = GeometriasDaPlaca::default();
                assert!(placa.decide_do_dispositivo_com_pousa(
                    true,
                    folga_f + 2,
                    &[1],
                    &store,
                    &mut geos,
                    Affine::IDENTITY,
                    corridas,
                ));
                let atlas = renderer
                    .texture_view_and_dims(0)
                    .expect("o átlas")
                    .0
                    .clone();
                placa.com_imagens(Some(super::imagens::FontesDasImagens {
                    buffer: buf_i,
                    linhas: folga_i + 2,
                    texturas: vec![(0..folga_i + 2, atlas)],
                    projeto: ph2d_render::ImageFilterMode::Smooth,
                }));
                let _ = placa
                    .desenha(&gpu, (W, H), &geos, Some(&buf_f))
                    .expect("desenha");
                let mundo = mundo_b(&gpu);
                super::cola_no_mundo(&mut placa, &gpu, &blit, mundo.blend_view());
                let lido = le_colunas(&gpu, &mundo);
                let f = |x, y| crate::motion_shape_gen::mistura::gpu_tests::f(tag, x, y);
                let quer = [f(B, A1), f(B, A2), f(B, A2), B].map(byte);
                eprintln!(
                    "Scene {tag} folgas ({folga_f}, {folga_i}) imagem_em_cima={imagem_em_cima}: lido {lido:?} · tabela {quer:?}"
                );
                if !lido.iter().zip(&quer).all(|(l, q)| (l - q).abs() <= BARRA) {
                    falhas.push(format!(
                        "tag {tag} folgas ({folga_f}, {folga_i}) imagem_em_cima={imagem_em_cima}: \
                     {lido:?} contra {quer:?}"
                    ));
                }
                casos += 1;
            }
        }
    }
    assert_eq!(casos, 24);
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
