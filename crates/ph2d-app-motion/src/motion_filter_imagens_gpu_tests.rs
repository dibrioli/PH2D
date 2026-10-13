//! ⭐⭐ **P4 — as IMAGENS de um grupo na camada filtrada** (doc 132 §4.1): com um `Each Shape` no fio,
//! CADA elemento do grupo — forma OU imagem — é uma célula: leva o filtro sozinho e volta na mistura do
//! grupo. As duas rotas da placa (a da CPU e a do dispositivo) desenham pelo MESMO passe das imagens e
//! pelas MESMAS células, e dão a mesma camada. Irmão do [`super::mistura_gpu_tests`] (as fixturas são
//! as dele), cortado pelo tecto de LOC.

use ph2d_eval_motion::{BlendWith, MisturaDoSink, VectorInstance};
use ph2d_render::{BandBlit, Camera2d, RenderInstance, WorldRt};
use ph2d_vector::Affine;

use super::Engrenagem;
use super::mistura_gpu_tests::{
    buffer_do_cozimento, le_linha, motion_com_filtro, srgb_para_linear,
};
use crate::motion_shape_gen::VecPathStore;
use crate::motion_shape_gen::mistura::gpu_tests::{
    A1, A2, B, BARRA, H, W, byte, copia, try_headless_gpu,
};
use crate::motion_shape_placa::{Misturas, PlacaDeFormas};
use crate::motion_state::MotionState;

/// O cinzento LINEAR do `Color Overlay` em cada forma.
const EFEITO: f32 = 0.6;

/// O `Output` com o modo em `Copies` e um `Color Overlay` cinzento em CADA forma (as células correm).
fn com_o_efeito(tag: u8) -> MotionState {
    let mut m = motion_com_filtro(tag, BlendWith::Copies, 0.0);
    let no = m.filtro.pilha()[0].no;
    let g = &mut m.doc.graph;
    ph2d_node_fx_filter::switch_kind(g, no, ph2d_ecs::FxOp::COLOR_OVERLAY);
    for (k, v) in ph2d_node_fx_filter::param::COLOR
        .into_iter()
        .zip([EFEITO, EFEITO, EFEITO, 1.0])
    {
        g.set_param(no, k, v);
    }
    g.set_param(no, ph2d_node_fx_filter::param::OPACITY, 1.0);
    m.filtro.prepara(&m.doc.graph, &m.sinks);
    m
}

/// As cópias da fixtura: a A (`cx = 16`) e a B (`cx = 32`), cada uma FORMA ou IMAGEM sólida do átlas.
fn copias(mistura: MisturaDoSink, imagens: [bool; 2], uvs: [[f32; 4]; 2]) -> Vec<VectorInstance> {
    [(16.0, A1, 0), (32.0, A2, 1)]
        .iter()
        .map(|&(cx, cinza, k)| {
            let mut c = copia(cx, 24.0, cinza, mistura, imagens[k].then_some(0));
            if imagens[k] {
                c.atlas_uv = uvs[k];
            }
            c
        })
        .collect()
}

/// A camada pela rota `dispositivo` (as formas num buffer do cozimento, as imagens como as separa) ou
/// pela da CPU (o `decide_com_misturas` e as fontes da placa), pousada no mundo `B`.
fn pela_rota(
    gpu: &ph2d_gpu::GpuContext,
    renderer: &mut ph2d_render::SpriteRenderer,
    blit: &BandBlit,
    m: &mut MotionState,
    insts: &[VectorInstance],
    dispositivo: bool,
) -> [i32; 4] {
    let mut store = VecPathStore::default();
    let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    let misturas = Misturas::do_quadro(gpu, m.pump.camada_isolada());
    let mut placa = PlacaDeFormas::default();
    let (formas, imagens): (Vec<_>, Vec<_>) =
        insts.iter().copied().partition(|i| i.geometry_id > 0);
    let mut buf_formas = None;
    if dispositivo {
        let linhas: Vec<RenderInstance> = imagens
            .iter()
            .map(crate::motion_shape_placa::linha_da_imagem)
            .collect();
        let n_f = u32::try_from(formas.len()).expect("poucas");
        let n_i = u32::try_from(linhas.len()).expect("poucas");
        let mistura = insts[0].mistura;
        let corridas = crate::motion_shape_placa::corridas_das_faixas(
            &[(m.sinks[0], 0..n_f, 0..n_i)],
            |_| mistura,
            misturas,
        )
        .expect("a placa compõe-na");
        buf_formas = (!formas.is_empty()).then(|| buffer_do_cozimento(gpu, &formas));
        let vivas: Vec<u32> = formas.iter().map(|i| i.geometry_id).collect();
        let bytes: &[u8] = bytemuck::cast_slice(&linhas);
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("test imagens do grupo"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&buffer, 0, bytes);
        let atlas = renderer
            .texture_view_and_dims(0)
            .expect("o átlas")
            .0
            .clone();
        assert!(placa.decide_do_dispositivo_com_pousa(
            true,
            n_f,
            &vivas,
            &store,
            &mut m.placa_geometrias,
            Affine::IDENTITY,
            corridas,
        ));
        placa.com_imagens(Some(crate::motion_shape_placa::imagens::FontesDasImagens {
            buffer,
            linhas: n_i,
            texturas: vec![(0..n_i, atlas)],
            projeto: ph2d_render::ImageFilterMode::Smooth,
        }));
    } else {
        assert!(
            placa.decide_com_misturas(
                true,
                insts,
                &store,
                &mut m.placa_geometrias,
                Affine::IDENTITY,
                misturas,
            ),
            "a rota da CPU leva a imagem de grupo à placa ({:?})",
            placa.motivo_do_vello()
        );
        let fontes = crate::motion_shape_placa::imagens::fontes_do_quadro(
            &placa,
            gpu,
            m,
            renderer,
            ph2d_render::ImageFilterMode::Smooth,
        );
        placa.com_imagens(fontes);
    }
    let _ = placa
        .desenha(gpu, (W, H), &m.placa_geometrias, buf_formas.as_ref())
        .expect("a placa desenha");
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
        formas: Some(&placa),
        mundo: &mundo,
        band_blit: blit,
    }));
    le_linha(gpu, &mundo)
}

/// ⭐⭐⭐ **Cada elemento do grupo — forma OU imagem — leva o filtro «cada forma», nas DUAS rotas**
/// (doc 132 §4.1). Fixtura: as duas cópias da tabela, um `Color Overlay` cinzento `x` em CADA forma,
/// `Copies` em cada modo, em três arranjos — forma + imagem na MESMA corrida, só imagens, e imagem +
/// forma (a ordem trocada). A linha tem de ser `[x, f(x, x), x, B]` (o efeito pinta cada elemento
/// antes da mistura), e as duas rotas iguais à barra da tabela. ⚠️ Controlo: a arte das imagens é
/// `A1`/`A2`, longe de `x` — uma imagem que escapasse ao filtro mostrava a arte dela.
#[test]
#[ignore = "placa: precisa de GPU"]
fn cada_elemento_do_grupo_leva_o_filtro_nas_duas_rotas() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let mut atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    for (k, cinza) in [(1u32, A1), (2, A2)] {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let v = byte(cinza) as u8;
        atlas
            .insert(&gpu, k, 4, 4, &[v, v, v, 255].repeat(16))
            .expect("cabe");
    }
    let mut renderer =
        ph2d_render::SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let uvs = [renderer.atlas().region_uv(1), renderer.atlas().region_uv(2)];
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let x = ph2d_color::srgb::linear_to_srgb_unit(EFEITO);
    let f = crate::motion_shape_gen::mistura::gpu_tests::f;
    let mut casos = 0;
    for imagens in [[false, true], [true, true], [true, false]] {
        for tag in [1u8, 3, 4] {
            let mistura = MisturaDoSink {
                blend: tag,
                com: BlendWith::Copies,
                sink: 0,
            };
            let insts = copias(mistura, imagens, uvs);
            let quer = [x, f(tag, x, x), x, B].map(byte);
            let mut lidos = Vec::new();
            for dispositivo in [false, true] {
                let mut m = com_o_efeito(tag);
                let lido = pela_rota(&gpu, &mut renderer, &blit, &mut m, &insts, dispositivo);
                println!(
                    "  imagens {imagens:?} tag {tag} dispositivo={dispositivo}: lido {lido:?} · esperado {quer:?}"
                );
                assert!(
                    lido.iter().zip(&quer).all(|(l, q)| (l - q).abs() <= BARRA),
                    "imagens {imagens:?} tag {tag} dispositivo={dispositivo}: {lido:?} contra {quer:?}"
                );
                lidos.push(lido);
            }
            assert!(
                lidos[0]
                    .iter()
                    .zip(&lidos[1])
                    .all(|(a, b)| (a - b).abs() <= 1),
                "as duas rotas: {lidos:?}"
            );
            casos += 1;
        }
    }
    assert_eq!(casos, 9);
    let _ = (A1, A2);
}
