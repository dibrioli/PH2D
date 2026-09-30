//! ⭐⭐⭐ **A PARIDADE DO PRODUTO** (doc 121, W2) — a MESMA lista de cópias do Motion desenhada
//! pela porta de hoje ([`crate::motion_shape_gen::encode`] → cena Vello → o `VelloPass` do produto)
//! e pela placa ([`PlacaDeFormas`]), comparadas pixel a pixel.
//!
//! ⚠️ **O gate da crate `ph2d-shape-gpu` mede o PASSE** sobre geometrias montadas à mão; este mede a
//! ROTA — as formas de fábrica do `source.shape`, o traço pela porta única do traço, o `basis`, o
//! `size`, a tinta e o afim da câmara com o espelho do Y, tudo como o produto os entrega. *Uma
//! paridade medida a montante do adaptador não afirma nada sobre o adaptador.*
//!
//! A régua é a da W1 (o alfa em todo pixel; a cor SEPARADA onde `α ≥ 64`), e as barras são as
//! dela: o passe aplana as curvas no espaço local com o erro no ecrã `≤ 0,25 px` e o Vello no ecrã
//! com os mesmos `0,25 px`, logo uma borda curva pode diferir até esse quarto de pixel.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib motion_shape_placa::gpu_tests -- --ignored --nocapture
//! ```

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec};

pub(crate) const LADO: u32 = 512;

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// A câmara do produto: `height_world = 16`, centro `(0, 0)`, Y para cima no mundo.
pub(crate) fn camara() -> Affine {
    let k = f64::from(LADO) / 16.0;
    Affine::translate((f64::from(LADO) * 0.5, f64::from(LADO) * 0.5))
        * Affine::scale_non_uniform(k, -k)
}

/// Um gerador pequeno e determinístico (o gate não pode depender de um sorteio do sistema).
struct Semente(u64);
impl Semente {
    fn f(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        #[expect(clippy::cast_precision_loss, reason = "24 bits de uma fracção")]
        let v = ((self.0 >> 40) as f32) / ((1u64 << 24) as f32);
        v
    }
}

/// As formas de fábrica do `source.shape`, com e sem traço — as mesmas construtoras que o
/// `motion_shape_gen` chama.
fn store() -> (VecPathStore, Vec<u32>) {
    let mut tracada = ph2d_vec_scene::regular_polygon_rounded([0.0, 0.0], 0.5, 0.5, 7, 0.05);
    tracada.stroke = Some(StrokeSpec::new(Rgba8::new(20, 30, 200, 230), 0.06));
    let mut circ = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5);
    circ.stroke = Some(StrokeSpec::new(Rgba8::new(250, 250, 40, 255), 0.1));
    let formas = [
        ph2d_vec_scene::star_rounded([0.0, 0.0], 0.5, 0.5, 5, 0.45, 0.0, 0.0),
        ph2d_vec_scene::star_rounded([0.0, 0.0], 0.5, 0.5, 6, 0.5, 0.08, 0.04),
        ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5),
        // ⚠️ Arestas HORIZONTAIS exactas — a forma que apanhou o defeito deste dia.
        ph2d_vec_scene::rounded_rect([-0.5, -0.35], [0.5, 0.35], 0.12),
        tracada,
        circ,
    ];
    let mut s = VecPathStore::default();
    let hs = formas.into_iter().map(|f| s.push(f)).collect();
    (s, hs)
}

/// `n` cópias espalhadas pelo campo, rodadas, de tamanhos diferentes, algumas translúcidas.
fn copias(hs: &[u32], n: usize) -> Vec<VectorInstance> {
    let mut r = Semente(0x5eed_f00d);
    (0..n)
        .map(|i| {
            let ang = r.f() * std::f32::consts::TAU;
            let lado = 0.3 + r.f() * 2.2;
            let (s, c) = ang.sin_cos();
            VectorInstance {
                geometry_id: hs[i % hs.len()],
                texture_id: 0,
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                premultiplied: 0.0,
                world_pos: [r.f() * 15.0 - 7.5, r.f() * 15.0 - 7.5],
                size: [lado, lado],
                basis: [c, s, -s, c],
                tint: [r.f(), r.f(), r.f(), 0.35 + r.f() * 0.65],
                anchor: [0.0, 0.0],
                sampling: 0,
                blend_linha: 0,
                mistura: Default::default(),
            }
        })
        .collect()
}

/// A cena de hoje: o `encode` do produto e o `VelloPass` do produto.
fn pelo_vello(gpu: &GpuContext, insts: &[VectorInstance], store: &VecPathStore) -> Vec<u8> {
    let mut cena = ph2d_vector::VectorScene::new();
    let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
    crate::motion_shape_gen::encode(
        insts,
        store,
        &mut |_, _| None,
        camara(),
        Some(janela),
        ph2d_render::ImageFilterMode::Smooth,
        &mut cena,
    );
    let mut vp =
        ph2d_render::VelloPass::new(gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (LADO, LADO))
            .expect("o VelloPass do produto nasce");
    vp.render_and_readback(gpu, cena.inner(), (LADO, LADO))
        .expect("o Vello desenha")
}

/// Meio-flutuante → `f32`.
fn f16(b: u16) -> f32 {
    let sinal = if b & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = i32::from((b >> 10) & 0x1f);
    let man = f32::from(b & 0x3ff);
    sinal
        * match exp {
            0 => man * 2f32.powi(-24),
            31 => f32::INFINITY,
            e => (1.0 + man / 1024.0) * 2f32.powi(e - 15),
        }
}

/// A placa: a camada do produto, lida de volta com a cor SEPARADA, como o Vello a grava.
fn pela_placa(gpu: &GpuContext, insts: &[VectorInstance], store: &VecPathStore) -> Vec<u8> {
    let mut p = PlacaDeFormas::default();
    let mut geometrias = GeometriasDaPlaca::default();
    assert!(
        p.decide(true, insts, store, &mut geometrias, camara()),
        "a cena do gate tem de ir à placa — senão ela mede o Vello contra si mesmo"
    );
    p.desenha(gpu, (LADO, LADO), &geometrias, None)
        .expect("a placa desenha");
    le_a_camada(gpu, &p)
}

/// A camada que a placa desenhou, lida de volta com a cor SEPARADA (`[r, g, b, a]` por pixel) —
/// partilhada com a paridade da rota do DISPOSITIVO (doc 121 W3).
pub(crate) fn le_a_camada(gpu: &GpuContext, p: &PlacaDeFormas) -> Vec<u8> {
    let tex = p.textura_da_camada().expect("a camada existe");
    let bpr = LADO * 8;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(bpr * LADO),
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
                bytes_per_row: Some(bpr),
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
    let v = buf.slice(..).get_mapped_range().to_vec();
    buf.unmap();
    let q = |x: f32| {
        #[expect(clippy::cast_possible_truncation, reason = "já limitado a um byte")]
        let b = (x.clamp(0.0, 1.0) * 255.0).round() as u8;
        b
    };
    v.as_chunks::<8>()
        .0
        .iter()
        .flat_map(|px| {
            let h = |i: usize| f16(u16::from_le_bytes([px[2 * i], px[2 * i + 1]]));
            let a = h(3);
            if a <= 0.0 {
                [0, 0, 0, 0]
            } else {
                [q(h(0) / a), q(h(1) / a), q(h(2) / a), q(a)]
            }
        })
        .collect()
}

/// O pior desvio de alfa, o pior de cor separada (`α ≥ 64`), e quantos pixels cada rota pinta.
/// E, por último, quantos pixels desviam mais de `16` no alfa ou na cor — o que se VÊ.
pub(crate) fn compara(v: &[u8], p: &[u8]) -> (u8, u8, usize, usize, usize) {
    let (mut alfa, mut cor, mut nv, mut np, mut fora) = (0u8, 0u8, 0usize, 0usize, 0usize);
    for (a, b) in v.as_chunks::<4>().0.iter().zip(p.as_chunks::<4>().0.iter()) {
        let da = a[3].abs_diff(b[3]);
        alfa = alfa.max(da);
        nv += usize::from(a[3] > 0);
        np += usize::from(b[3] > 0);
        let mut dc = 0;
        if a[3] >= 64 && b[3] >= 64 {
            dc = (0..3).map(|k| a[k].abs_diff(b[k])).max().unwrap_or(0);
            cor = cor.max(dc);
        }
        fora += usize::from(da > 16 || dc > 16);
    }
    (alfa, cor, nv, np, fora)
}

/// ⭐⭐⭐ **A rota da placa desenha o que a cena Vello desenha** — formas de fábrica, traço, rotação,
/// escala, translucidez e a câmara com o espelho do Y.
///
/// **As barras, do vale MEDIDO** (2026-09-29, RTX, `240` cópias das seis formas de fábrica):
///
/// | estado | alfa máx. | cor máx. | pixels `> 16` | área (vello / placa) |
/// |---|---:|---:|---:|---|
/// | **a rota que shipa** | `66` | `63` | `3 790` (`2,0 %`) | `191 599` / `191 859` |
/// | a aresta horizontal saltada (o defeito deste dia) | `254` | `237` | `38 929` (`20,3 %`) | `191 599` / `187 861` |
///
/// ⭐ O alfa e a cor medem o MESMO quarto de pixel de borda curva que a W1 mediu (`65` nos círculos
/// isolados): a cor de um pixel de borda desvia `Δcobertura × contraste`, e com `Δcobertura ≤ 0,26`
/// isso cabe nos mesmos `100` do alfa (o vale da W1 entre o aplanamento nítido `77` e o grosso
/// `163`). ⇒ as duas barras são `100`, e o que separa o defeito é a CONTAGEM: `5 %` fica entre os
/// `2,0 %` da rota e os `20,3 %` do defeito. ⭐ E a MESMA área pintada a `1 %`.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_rota_da_placa_desenha_o_que_a_cena_vello_desenha() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let (store, hs) = store();
    let insts = copias(&hs, 240);
    let v = pelo_vello(&gpu, &insts, &store);
    let p = pela_placa(&gpu, &insts, &store);
    let (alfa, cor, nv, np, fora) = compara(&v, &p);
    if let Ok(dir) = std::env::var("PH2D_PARIDADE_DIR") {
        for (nome, img) in [("produto_vello", &v), ("produto_placa", &p)] {
            let mut ppm = format!("P6\n{LADO} {LADO}\n255\n").into_bytes();
            for px in img.as_chunks::<4>().0 {
                ppm.extend(px[..3].iter().map(|c| {
                    #[expect(clippy::cast_possible_truncation, reason = "um byte")]
                    let b =
                        (u32::from(*c) * u32::from(px[3]) / 255 + (255 - u32::from(px[3]))) as u8;
                    b
                }));
            }
            let _ = std::fs::write(std::path::Path::new(&dir).join(format!("{nome}.ppm")), ppm);
        }
    }
    eprintln!(
        "  produto: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} px · placa {np} px"
    );
    assert!(
        nv > 20_000,
        "controlo: a cena pinta pouco ({nv} px) e não contém o fenómeno"
    );
    let diff = nv.abs_diff(np);
    assert!(diff * 100 <= nv, "área pintada diverge: {nv} contra {np}");
    assert!(alfa <= 100, "alfa {alfa} acima da barra das curvas (100)");
    assert!(cor <= 100, "cor {cor} acima da barra das curvas (100)");
    assert!(
        fora * 20 <= nv,
        "{fora} pixels desviam > 16 — acima de 5 % dos {nv} pintados"
    );
}
