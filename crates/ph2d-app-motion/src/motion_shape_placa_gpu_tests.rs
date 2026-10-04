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

/// Com `PH2D_PARIDADE_DIR`, grava as duas imagens sobre branco (`<prefixo>_vello.ppm` ·
/// `<prefixo>_placa.ppm`) — para OLHAR, não só medir.
pub(crate) fn fotografa(prefixo: &str, vello: &[u8], placa: &[u8]) {
    let Ok(dir) = std::env::var("PH2D_PARIDADE_DIR") else {
        return;
    };
    for (rota, img) in [("vello", vello), ("placa", placa)] {
        let mut ppm = format!("P6\n{LADO} {LADO}\n255\n").into_bytes();
        for px in img.as_chunks::<4>().0 {
            ppm.extend(px[..3].iter().map(|c| {
                #[expect(clippy::cast_possible_truncation, reason = "um byte")]
                let b = (u32::from(*c) * u32::from(px[3]) / 255 + (255 - u32::from(px[3]))) as u8;
                b
            }));
        }
        let _ = std::fs::write(
            std::path::Path::new(&dir).join(format!("{prefixo}_{rota}.ppm")),
            ppm,
        );
    }
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
    fotografa("produto", &v, &p);
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

/// ⭐⭐⭐ **O TRAÇO SOB ESCALA NÃO UNIFORME, pela rota do PRODUTO** (doc 121 W4) — as MESMAS seis
/// formas de fábrica, ESTICADAS (aspecto entre `0,35` e `2,8`), pelo `encode` do produto (que aplica a
/// lei da casa: a geometria transformada, a caneta redonda `w·√|det|` — `stroke_uniform`) e pela
/// placa (o traço construído no ecrã a partir do eixo). Até à W4 este quadro nem ia à placa.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_rota_da_placa_desenha_o_traco_esticado_como_a_casa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let (store, hs) = store();
    let mut insts = copias(&hs, 240);
    for (i, c) in insts.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma fixtura pequena")]
        let k = (i as f32 * 0.618_034).fract();
        c.size[1] *= 0.35 + 2.45 * k;
    }
    assert!(
        insts.iter().filter(|c| !super::conforme(c)).count() > 200,
        "controlo: a fixtura tem de ser feita de copias NAO conformes"
    );
    let v = pelo_vello(&gpu, &insts, &store);
    let p = pela_placa(&gpu, &insts, &store);
    let (alfa, cor, nv, np, fora) = compara(&v, &p);
    eprintln!(
        "  produto esticado: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} px · placa {np} px"
    );
    assert!(nv > 20_000, "controlo: a cena pinta pouco ({nv} px)");
    let diff = nv.abs_diff(np);
    assert!(diff * 100 <= nv, "área pintada diverge: {nv} contra {np}");
    assert!(alfa <= 100, "alfa {alfa} acima da barra das curvas (100)");
    assert!(cor <= 100, "cor {cor} acima da barra das curvas (100)");
    assert!(
        fora * 20 <= nv,
        "{fora} pixels desviam > 16 — acima de 5 % dos {nv} pintados"
    );
}

/// ⭐⭐⭐ **UMA FORMA ALINHADA AOS EIXOS NÃO RISCA UMA LINHA** (report do dono, 2026-09-30, a foto da
/// cena `=127`: *«artefatos de imagem: veja linha nas estrelas»*).
///
/// ⛔⛔ **Os gates de paridade nunca tinham uma cópia ALINHADA**: todos sorteiam o ângulo, e com
/// um ângulo qualquer nenhuma aresta cai exactamente na vertical. Na cena as estrelas não rodam, e
/// a junta redonda da ponta de uma estrela esticada tem uma aresta VERTICAL (os dois lados da ponta
/// são espelho um do outro, ao bit). A cobertura portada do Vello divide por `xmax − xmin` e conta
/// com um `−1e-6` para nunca dar zero — verdade num ladrilho de `16 px`, falso aqui, onde as
/// coordenadas são relativas ao PIXEL e chegam a centenas: longe à esquerda o `−1e-6` perde-se no
/// `f32`, a conta vira `0/0 = NaN`, e o `min(abs(NaN), 1)` pintava a fileira inteira à direita da
/// ponta até ao fim do quad — a linha escura da foto.
///
/// **Medido** (RTX, três estrelas de 8 pontas com cantos arredondados, esticadas `1,8 × 0,6`, sem
/// rotação): antes da cura **alfa `255` · cor `235` · `1 934` px fora**; depois **alfa `65` · cor
/// `68` · `685` px** sobre `23 144` (o quarto de pixel das curvas da W1). O CONTROLO é a estrela de
/// 5 pontas sem cantos, que não tem aresta vertical e lia limpa antes e depois.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_forma_alinhada_aos_eixos_nao_risca_uma_linha() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let mut store = VecPathStore::default();
    let mut f = ph2d_vec_scene::star_rounded([0.0, 0.0], 0.5, 0.5, 8, 0.6, 0.08, 0.08);
    f.stroke = Some(StrokeSpec::new(Rgba8::new(20, 30, 90, 255), 0.06));
    let h = store.push(f);
    let insts: Vec<VectorInstance> = [2.0f32, 3.3, 4.7]
        .iter()
        .enumerate()
        .map(|(i, lado)| {
            #[expect(clippy::cast_precision_loss, reason = "tres copias")]
            let y = 5.0 - 5.0 * i as f32;
            VectorInstance {
                geometry_id: h,
                texture_id: 0,
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                premultiplied: 0.0,
                world_pos: [0.3, y + 0.013],
                size: [lado * 1.8, lado * 0.6],
                // ⚠️ SEM rotação: é o que põe a aresta da ponta exactamente na vertical.
                basis: [1.0, 0.0, 0.0, 1.0],
                tint: [1.0, 0.8, 0.2, 1.0],
                anchor: [0.0, 0.0],
                sampling: 0,
                blend_linha: 0,
                mistura: Default::default(),
            }
        })
        .collect();
    assert!(
        insts.iter().all(|c| !super::conforme(c)),
        "controlo: as copias tem de ser NAO conformes (o traço sai do eixo)"
    );
    let v = pelo_vello(&gpu, &insts, &store);
    let p = pela_placa(&gpu, &insts, &store);
    let (alfa, cor, nv, np, fora) = compara(&v, &p);
    eprintln!(
        "  alinhada: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} · placa {np}"
    );
    assert!(nv > 15_000, "controlo: a cena pinta pouco ({nv} px)");
    assert!(
        nv.abs_diff(np) * 100 <= nv,
        "área pintada diverge: {nv} contra {np}"
    );
    assert!(alfa <= 100, "alfa {alfa} acima da barra das curvas (100)");
    assert!(
        cor <= 100,
        "cor {cor} acima da barra das curvas (100) — a linha da ponta voltou"
    );
    assert!(
        fora * 20 <= nv,
        "{fora} pixels desviam > 16 — acima de 5 % dos {nv} pintados"
    );
}

/// SONDA de relógio (doc 121 W4, report de 2026-09-30: *«com `PH2D_FORMAS_NA_PLACA=0` o `raw` está
/// quase sempre maior»*, com as estrelas GRANDES no ecrã). Estrelas esticadas que enchem o alvo,
/// desenhadas `N` vezes por cada rota, com o dispositivo drenado entre elas.
#[test]
#[ignore = "sonda de relógio"]
fn sonda_relogio_das_estrelas_grandes() {
    let Some(gpu) = gpu() else { return };
    let mut store = VecPathStore::default();
    let mut f = ph2d_vec_scene::star_rounded([0.0, 0.0], 0.5, 0.5, 8, 0.6, 0.08, 0.08);
    let modo = std::env::var("PH2D_SONDA_MODO").unwrap_or_default();
    // `PH2D_SONDA_DENSO=1`: o arranjo DENSO da `=127` (estrela de `8 px`, contorno de `1 px`, o vão
    // da cena) a encher o alvo — o regime em que o proxy de telemóvel ficou preso na placa (§9.3).
    //
    // `PH2D_SONDA_DENSO=2`: o MESMO arranjo com o vão da CENA (`2 · 0,125 · 1,8 = 0,45` unidades,
    // `14,4 px`) — a `=127` densa põe as estrelas a metade do vão do `=1`, e o custo por estrela do
    // app só se reproduz com a densidade dele.
    let nivel_denso = std::env::var("PH2D_SONDA_DENSO").unwrap_or_default();
    let denso = nivel_denso == "1" || nivel_denso == "2";
    if modo != "fill" {
        let w = if denso { 0.125 } else { 0.06 };
        f.stroke = Some(StrokeSpec::new(Rgba8::new(20, 30, 90, 255), w));
    }
    // `=2`: a ESTRELA DA CENA (quinas vivas), cozida pela porta da shell — e a escala das cópias
    // põe-na nos MESMOS píxeis (a cena tem `55,5 px` por unidade, a sonda `32`).
    let k_cena = if nivel_denso == "2" {
        use crate::motion_state::traco_esticado_demo as cena;
        f = cena::forma_da_cena(cena::DENSO);
        crate::motion_state::carimbo_demo::PX_POR_UNIDADE / 32.0
    } else {
        1.0
    };
    // `PH2D_SONDA_TRACEJADO=1`: o contorno com o `Dash` e o `Dash Gap` da `=127` tracejada — a
    // variante COMPLETA do shader (doc 121 §9.13).
    if std::env::var("PH2D_SONDA_TRACEJADO").is_ok_and(|v| v == "1") {
        use crate::motion_state::traco_esticado_demo::TRACEJADO;
        if let Some(s) = f.stroke.as_mut() {
            s.dash = Some((f64::from(TRACEJADO.0), f64::from(TRACEJADO.1)));
        }
    }
    let (ex, ey) = if modo == "conforme" {
        (1.04, 1.04)
    } else {
        (1.8, 0.6)
    };
    let h = store.push(f);
    let mut insts = Vec::new();
    // denso: estrela de `0,25` unidades (`8 px` a `32 px`/unidade) e o vão da cena (`2 · 0,25 · 1,8`)
    let (tam, vao, lado) = if nivel_denso == "2" {
        (0.25_f32, 0.45_f32, 35)
    } else if denso {
        (0.25_f32, 0.9_f32, 18)
    } else {
        (2.0, 0.0, 0)
    };
    let inicio = if nivel_denso == "2" { -7.65_f32 } else { -7.8 };
    let celulas: Vec<(f32, f32)> = if denso {
        #[expect(clippy::cast_precision_loss, reason = "uma grelha pequena")]
        (0..lado * lado)
            .map(|k| {
                (
                    inicio + vao * (k % lado) as f32,
                    inicio + vao * (k / lado) as f32,
                )
            })
            .collect()
    } else {
        #[expect(clippy::cast_precision_loss, reason = "uma grelha pequena")]
        (0..72)
            .map(|k| (-7.0 + 2.8 * (k / 12) as f32, -7.5 + 1.3 * (k % 12) as f32))
            .collect()
    };
    for (x, y) in celulas {
        {
            insts.push(VectorInstance {
                geometry_id: h,
                texture_id: 0,
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                premultiplied: 0.0,
                world_pos: [x, y],
                size: if nivel_denso == "2" {
                    [ex * k_cena, ey * k_cena]
                } else {
                    [tam * ex, tam * ey]
                },
                basis: [1.0, 0.0, 0.0, 1.0],
                tint: [1.0, 0.8, 0.2, 1.0],
                anchor: [0.0, 0.0],
                sampling: 0,
                blend_linha: 0,
                mistura: Default::default(),
            });
        }
    }
    // `PH2D_FLUID_PROFILE=1`: o relógio da placa POR PASSE (o cálculo `render.contorno` e o desenho
    // `render.formas`), impresso a cada `120` quadros — a decomposição que o relógio de parede não dá.
    let perfil = std::env::var("PH2D_FLUID_PROFILE").is_ok_and(|v| v != "0");
    ph2d_gpu::pass_profiler::init(&gpu.device, &gpu.queue);
    let n = if perfil { 250u32 } else { 40 };
    let mut p = PlacaDeFormas::default();
    // `PH2D_SONDA_SEM_CONTORNO=1`: o traço do eixo pixel a pixel (o caminho antes do doc 121 §9.5).
    if std::env::var("PH2D_SONDA_SEM_CONTORNO").is_ok_and(|v| v == "1") {
        p.sem_contorno();
    }
    let mut geo = GeometriasDaPlaca::default();
    // doc 121 §9.15 (c) — o relógio de CPU de cada fase do quadro, `[decide, desenha, espera]`: a
    // parede menos a soma dos passes nunca foi explicada, e o `gpu-busy(span)` já iguala a soma.
    let mut cpu = [0.0_f64; 4];
    let quadro = |p: &mut PlacaDeFormas, geo: &mut GeometriasDaPlaca, cpu: &mut [f64; 4]| {
        let t0 = std::time::Instant::now();
        assert!(p.decide(true, &insts, &store, geo, camara()));
        let t1 = std::time::Instant::now();
        let _ = p.desenha(&gpu, (LADO, LADO), geo, None);
        let t2 = std::time::Instant::now();
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let t3 = std::time::Instant::now();
        ph2d_gpu::pass_profiler::end_frame(&gpu.device, &gpu.queue);
        cpu[0] += (t1 - t0).as_secs_f64();
        cpu[1] += (t2 - t1).as_secs_f64();
        cpu[2] += (t3 - t2).as_secs_f64();
        cpu[3] += t3.elapsed().as_secs_f64();
    };
    quadro(&mut p, &mut geo, &mut cpu);
    cpu = [0.0; 4];
    // A parede de CADA quadro: a média esconde os primeiros, que vão pixel a pixel até a capacidade
    // medida chegar (dois quadros depois — `Contorno::colhe`).
    let mut paredes = Vec::with_capacity(n as usize);
    let t = std::time::Instant::now();
    for _ in 0..n {
        let tq = std::time::Instant::now();
        quadro(&mut p, &mut geo, &mut cpu);
        paredes.push(tq.elapsed().as_secs_f64() * 1e3);
    }
    let placa = t.elapsed().as_secs_f64() * 1e3 / f64::from(n);
    let [decide, desenha, espera, perfil_ms] = cpu.map(|s| s * 1e3 / f64::from(n));
    eprintln!(
        "  cpu por quadro: decide {decide:.3} ms · desenha {desenha:.3} ms · espera {espera:.3} ms · perfilador {perfil_ms:.3} ms"
    );
    let primeiros: Vec<String> = paredes
        .iter()
        .take(4)
        .map(|ms| format!("{ms:.2}"))
        .collect();
    let mut ordenadas = paredes.clone();
    ordenadas.sort_by(f64::total_cmp);
    eprintln!(
        "  parede por quadro: mediana {:.3} ms · primeiros {} ms",
        ordenadas[ordenadas.len() / 2],
        primeiros.join(" ")
    );
    let (reservadas, escritas, do_contorno) = p.arestas_do_ultimo_quadro(&gpu);
    eprintln!(
        "  arestas: reservadas {reservadas} · escritas {escritas} (do contorno {do_contorno})"
    );
    let (com_contorno, cap) = p.copias_com_contorno(&gpu, u32::try_from(insts.len()).unwrap_or(0));
    let (pediram, cap_celulas) = p.celulas_do_ultimo_quadro(&gpu);
    let (tocadas, usadas) = p.celulas_tocadas_do_ultimo_quadro(&gpu);
    eprintln!(
        "  contorno calculado em {com_contorno} de {} copias (capacidade {cap} arestas) · celulas {pediram} de {cap_celulas} · tocadas {tocadas} de {usadas}",
        insts.len()
    );
    // ⛔ doc 121 §9.3 — **um relógio sobre um passe que não desenhou é um número de NADA.** Com o
    // shader partido (um erro de compilação só sai no registo do `wgpu`) esta sonda leu `0,005 ms`
    // e saiu verde. ⇒ a camada cronometrada tem de ter tinta.
    let tinta = le_a_camada(&gpu, &p)
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[3] > 0)
        .count();
    assert!(
        tinta > 1000,
        "o passe cronometrado nao desenhou ({tinta} px com tinta) -- o relogio mede nada"
    );
    let mut cena = ph2d_vector::VectorScene::new();
    let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
    crate::motion_shape_gen::encode(
        &insts,
        &store,
        &mut |_, _| None,
        camara(),
        Some(janela),
        ph2d_render::ImageFilterMode::Smooth,
        &mut cena,
    );
    let mut vp =
        ph2d_render::VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (LADO, LADO))
            .expect("vello");
    let _ = vp.render_and_readback(&gpu, cena.inner(), (LADO, LADO));
    let t = std::time::Instant::now();
    for _ in 0..n {
        let _ = vp.render_and_readback(&gpu, cena.inner(), (LADO, LADO));
    }
    let vello = t.elapsed().as_secs_f64() * 1e3 / f64::from(n);
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "  [{modo}] {} copias: placa {placa:.3} ms · vello {vello:.3} ms (com leitura) · load {carga}",
        insts.len()
    );
}

#[path = "motion_shape_placa_gpu_letras_tests.rs"]
mod letras;

#[path = "motion_shape_placa_gpu_tracejado_tests.rs"]
mod tracejado;
