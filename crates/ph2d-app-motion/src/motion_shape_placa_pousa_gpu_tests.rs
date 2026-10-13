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

/// A placa desenha as duas cópias e a camada pousa no mundo `B` — com a pousa do quadro, ou com
/// `forca` (o controlo). `None` se a placa recusa.
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
            // O sink do dispositivo: a mistura dele por faixa, sem modo por linha (a coluna `blend`
            // com formas recusa o cozimento).
            let mut g = ph2d_nodegraph::graph::Graph::new();
            let sink = g.add_node("motion.output");
            let m = insts[0].mistura;
            let n = u32::try_from(insts.len()).expect("poucas");
            let r = corridas_das_faixas(&[(sink, 0..n, 0..0)], |_| m, misturas).ok()?;
            buffer = Some(buffer_do_cozimento(gpu, insts));
            let vivas: Vec<u32> = insts.iter().map(|i| i.geometry_id).collect();
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
    Some((
        pousa_no_mundo(gpu, blit, &camada, forca.unwrap_or(pousa)),
        pousa,
    ))
}

/// O mundo limpo a `B` e a camada pousada nele pela porta do presente.
fn pousa_no_mundo(
    gpu: &GpuContext,
    blit: &BandBlit,
    camada: &wgpu::TextureView,
    pousa: Pousa,
) -> [i32; 4] {
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
    blit.blit_pousa(
        gpu,
        mundo.blend_view(),
        camada,
        BandSource::Formas,
        None,
        pousa,
    );
    le_colunas(gpu, &mundo)
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
