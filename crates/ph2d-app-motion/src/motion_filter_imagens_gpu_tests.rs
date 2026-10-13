//! ⭐⭐ **P4 — as IMAGENS de um grupo na camada filtrada** (doc 132 §4.1): com um `Each Shape` no fio
//! a camada não cola a placa, e a volta das células pinta as imagens do grupo no grupo delas. Irmão do
//! [`super::mistura_gpu_tests`] (as fixturas são as dele), cortado pelo tecto de LOC.

use ph2d_eval_motion::{BlendWith, MisturaDoSink};
use ph2d_render::{BandBlit, Camera2d, WorldRt};
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

/// ⭐⭐ **P4 — as IMAGENS de um grupo voltam nas células do `Each Shape`** (doc 132 §4.1): o
/// dispositivo entrega as linhas de imagem do grupo à placa, e com um `Each Shape` no fio a camada não
/// cola a placa — a volta das células pinta-as no grupo delas ([`ph2d_fx_cells::Imagens`]). Fixtura:
/// as duas cópias da tabela, um `Color Overlay` cinzento `X` em CADA forma (as células correm),
/// `Copies` em cada modo, em dois arranjos:
/// - as DUAS cópias como imagens sólidas do átlas (`A1`, `A2`) — sem célula nenhuma, as imagens voltam
///   no fim da sessão; a linha é a da tabela, porque a imagem não leva o efeito das formas;
/// - a cópia A como FORMA (`A1`, que o efeito pinta `x`) e a B como imagem (`A2`) na MESMA corrida —
///   a imagem volta ANTES da célula da forma, no mesmo grupo: a sobreposição é `f(x, A2)`.
#[test]
#[ignore = "placa: precisa de GPU"]
fn as_imagens_de_um_grupo_voltam_nas_celulas_do_each_shape() {
    use ph2d_render::RenderInstance;
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    // O átlas com as duas artes sólidas (4×4) nas chaves 1 e 2.
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
    let atlas_view = renderer
        .texture_view_and_dims(0)
        .expect("o átlas")
        .0
        .clone();
    let blit = BandBlit::new(&gpu, WorldRt::FORMAT);
    let cinza_do_efeito = 0.6f32;
    let x = ph2d_color::srgb::linear_to_srgb_unit(cinza_do_efeito);
    for forma_em_a in [false, true] {
        for tag in [1u8, 3, 4] {
            let mut m = motion_com_filtro(tag, BlendWith::Copies, 0.0);
            let no = m.filtro.pilha()[0].no;
            let g = &mut m.doc.graph;
            ph2d_node_fx_filter::switch_kind(g, no, ph2d_ecs::FxOp::COLOR_OVERLAY);
            for (k, v) in ph2d_node_fx_filter::param::COLOR.into_iter().zip([
                cinza_do_efeito,
                cinza_do_efeito,
                cinza_do_efeito,
                1.0,
            ]) {
                g.set_param(no, k, v);
            }
            g.set_param(no, ph2d_node_fx_filter::param::OPACITY, 1.0);
            m.filtro.prepara(&m.doc.graph, &m.sinks);
            let mistura = MisturaDoSink {
                blend: tag,
                com: BlendWith::Copies,
                sink: 0,
            };
            // As linhas de imagem como o cozimento as entrega à placa: `RenderInstance`.
            let imagens: Vec<(f32, [f32; 4])> = if forma_em_a {
                vec![(32.0, uvs[1])]
            } else {
                vec![(16.0, uvs[0]), (32.0, uvs[1])]
            };
            let linhas: Vec<RenderInstance> = imagens
                .iter()
                .map(|&(cx, uv)| {
                    let mut r: RenderInstance = bytemuck::Zeroable::zeroed();
                    r.world_pos = [cx, H as f32 / 2.0];
                    r.size = [24.0, H as f32];
                    r.atlas_uv = uv;
                    r.tint = [1.0; 4];
                    r.basis = [1.0, 0.0, 0.0, 1.0];
                    r.opacity = 1.0;
                    r.uv_xform = RenderInstance::IDENTITY_UV_XFORM;
                    r
                })
                .collect();
            let bytes: &[u8] = bytemuck::cast_slice(&linhas);
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("test imagens do grupo"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.queue.write_buffer(&buffer, 0, bytes);
            let n_img = u32::try_from(linhas.len()).expect("poucas");
            // A forma A (o retângulo da fixtura), quando o arranjo a tem.
            let mut store = VecPathStore::default();
            let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
            let formas = forma_em_a.then(|| vec![copia(16.0, 24.0, A1, mistura, None)]);
            let n_formas = formas
                .as_ref()
                .map_or(0, |f| u32::try_from(f.len()).expect("uma"));
            let buf_formas = formas.as_ref().map(|f| buffer_do_cozimento(&gpu, f));
            let misturas = Misturas::do_quadro(&gpu, m.pump.camada_isolada());
            let corridas = crate::motion_shape_placa::corridas_das_faixas(
                &[(m.sinks[0], 0..n_formas, 0..n_img)],
                |_| mistura,
                misturas,
            )
            .expect("a placa compõe-na");
            let mut placa = PlacaDeFormas::default();
            placa.com_imagens(Some(crate::motion_shape_placa::imagens::FontesDasImagens {
                buffer,
                texturas: vec![(0..n_img, atlas_view.clone())],
                projeto: ph2d_render::ImageFilterMode::Smooth,
            }));
            let vivas: Vec<u32> = formas.iter().flatten().map(|i| i.geometry_id).collect();
            let geos = &mut m.placa_geometrias;
            assert!(placa.decide_do_dispositivo_com_pousa(
                true,
                n_formas,
                &vivas,
                &store,
                geos,
                Affine::IDENTITY,
                corridas,
            ));
            let _ = placa
                .desenha(&gpu, (W, H), geos, buf_formas.as_ref())
                .expect("a placa desenha");
            assert!(placa.tem_imagens(), "tag {tag}: as imagens foram à placa");
            let mundo = WorldRt::new(&gpu, (W, H));
            let b = srgb_para_linear(f64::from(B));
            mundo.clear_linear(
                &gpu,
                wgpu::Color {
                    r: b,
                    g: b,
                    b,
                    a: 1.0,
                },
            );
            assert!(m.filtro.liga(true));
            assert!(m.filtra_a_camada(Engrenagem {
                gpu: &gpu,
                renderer: &mut renderer,
                camera: &Camera2d::default(),
                window: ph2d_host::WindowSize {
                    width: W,
                    height: H,
                },
                scene_viewport: None,
                formas: Some(&placa),
                mundo: &mundo,
                band_blit: &blit,
            }));
            let lido = le_linha(&gpu, &mundo);
            let a = if forma_em_a { x } else { A1 };
            let f = crate::motion_shape_gen::mistura::gpu_tests::f;
            let quer = [a, f(tag, a, A2), A2, B].map(byte);
            println!("  forma em A={forma_em_a} tag {tag}: lido {lido:?} · esperado {quer:?}");
            assert!(
                lido.iter().zip(&quer).all(|(l, q)| (l - q).abs() <= BARRA),
                "forma em A={forma_em_a} tag {tag}: {lido:?} contra {quer:?}"
            );
        }
    }
}
