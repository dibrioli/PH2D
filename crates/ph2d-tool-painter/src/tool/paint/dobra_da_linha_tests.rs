//! ⭐ B15.6 — a dobra de UMA linha ([`ReliefFields::dobra_da_linha`]) é a das duas pistas: a luz da CPU em faixas e os
//! planos da placa dão AO BIT o que os laços de antes (B10, guardados aqui verbatim) davam com os três amostradores.

use super::*;
use crate::tool::paint::impasto_gpu::ImpastoPlanes;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

impl PainterTool {
    /// As faixas da luz de antes do B15.6 (B10), verbatim: altura por `height_at`, corpo e material por píxel.
    fn apply_impasto_light_faixas_b10(&self, rgba: &mut [u8], region: Region) {
        if !self.impasto_visible() {
            return;
        }
        let Some(fields) = self.impasto_fields() else {
            return;
        };
        let (w, h) = (region.w as usize, region.h as usize);
        if w == 0 || h == 0 || rgba.len() < w * h * 4 {
            return;
        }
        let Some(light) = Rig::new(&self.paint.impasto_rig) else {
            return;
        };
        let paper_body = fields.paper_body();
        let (x0, y0) = (i64::from(region.x), i64::from(region.y));
        rgba[..w * h * 4]
            .par_chunks_mut(w * 4 * FAIXA_LINHAS)
            .enumerate()
            .for_each(|(b, faixa)| {
                let n = faixa.len() / (w * 4);
                let gy0 = y0 + (b * FAIXA_LINHAS) as i64;
                let fw = w + 2;
                let mut alt = vec![0.0f32; fw * (n + 2)];
                for (j, linha) in alt.chunks_exact_mut(fw).enumerate() {
                    let gy = gy0 - 1 + j as i64;
                    for (i, a) in linha.iter_mut().enumerate() {
                        *a = fields.height_at(x0 - 1 + i as i64, gy);
                    }
                }
                let mut mat =
                    light.resolve(ph2d_painter_brush::material::Material::NEUTRAL.to_bytes());
                for ry in 0..n {
                    let gy = gy0 + ry as i64;
                    let c = (ry + 1) * fw + 1;
                    for rx in 0..w {
                        let gx = x0 + rx as i64;
                        let k = c + rx;
                        let mut dhx = (alt[k + 1] - alt[k - 1]) * 0.5;
                        let mut dhy = (alt[k + fw] - alt[k - fw]) * 0.5;
                        let cover = fields.cover_at(gx, gy);
                        // B31.1: a lei nova da inclinação com papel (o oráculo guarda as FAIXAS, não a lei).
                        if fields.substrate.is_some() {
                            let p = |x: i64, y: i64| fields.altura_e_papel_at(x, y).1;
                            let px = (p(gx + 1, gy) - p(gx - 1, gy)) * 0.5;
                            let py = (p(gx, gy + 1) - p(gx, gy - 1)) * 0.5;
                            (dhx, dhy) = super::super::impasto_luz_dobra::inclinacao_com_papel(
                                (dhx, dhy),
                                (px, py),
                                cover,
                            );
                        }
                        let i = (ry * w + rx) * 4;
                        let albedo = [
                            f32::from(faixa[i]) / 255.0,
                            f32::from(faixa[i + 1]) / 255.0,
                            f32::from(faixa[i + 2]) / 255.0,
                        ];
                        let key = fields.material_at(gx, gy);
                        if key != mat.key {
                            mat = light.resolve(key);
                        }
                        let body = paint_body(cover).max(paper_body);
                        let (mul, add) =
                            light.shade(&mat, body, gloss_body(cover), dhx, dhy, albedo);
                        if mul == [1.0; 3] && add == [0.0; 3] {
                            continue;
                        }
                        for c in 0..3 {
                            let lit = light_pixel(albedo[c], mul[c], add[c]);
                            faixa[i + c] = (lit * 255.0 + 0.5) as u8;
                        }
                    }
                }
            });
    }

    /// O laço dos planos da placa de antes do B15.6, verbatim: os três amostradores por texel.
    fn impasto_gpu_planes_in_por_texel(
        &self,
        region: (u32, u32, u32, u32),
    ) -> Option<ImpastoPlanes> {
        if !self.impasto_visible() {
            return None;
        }
        let fields = self.impasto_fields()?;
        let lamps = Rig::new(&self.paint.impasto_rig)?.export_lamps();
        let (w, h) = self.source_size;
        let (rx, ry, rw, rh) = region;
        let (rx, ry) = (rx.min(w), ry.min(h));
        let (rw, rh) = (rw.min(w - rx), rh.min(h - ry));
        let n = (rw as usize) * (rh as usize);
        let mut relief = vec![0f32; n];
        let mut cover = vec![0u8; n];
        let mut mat0 = vec![0u8; n * 4];
        let mut mat1 = vec![0u8; n * 4];
        let rww = rw as usize;
        relief
            .par_chunks_mut(rww)
            .zip(cover.par_chunks_mut(rww))
            .zip(mat0.par_chunks_mut(rww * 4))
            .zip(mat1.par_chunks_mut(rww * 4))
            .enumerate()
            .for_each(|(row, (((rrow, crow), m0row), m1row))| {
                let y = i64::from(ry) + row as i64;
                for i in 0..rww {
                    let x = i64::from(rx) + i as i64;
                    rrow[i] = fields.height_at(x, y);
                    crow[i] = (fields.cover_at(x, y) * 255.0 + 0.5) as u8;
                    let m = fields.material_at(x, y);
                    m0row[i * 4..i * 4 + 4].copy_from_slice(&[m[0], m[1], m[2], m[3]]);
                    m1row[i * 4..i * 4 + 4].copy_from_slice(&[m[4], m[5], m[6], 255]);
                }
            });
        // B31.1: o plano do papel, por texel.
        let papel: Vec<f32> = if fields.substrate.is_some() {
            (0..n)
                .map(|i| {
                    let (x, y) = (
                        i64::from(rx) + (i % rww) as i64,
                        i64::from(ry) + (i / rww) as i64,
                    );
                    fields.altura_e_papel_at(x, y).1
                })
                .collect()
        } else {
            Vec::new()
        };
        let lut = ph2d_painter_brush::material::SpecLut::get();
        Some(ImpastoPlanes {
            width: w,
            height: h,
            region: (rx, ry, rw, rh),
            relief,
            cover,
            mat0,
            mat1,
            paper_body: fields.paper_body(),
            realce_linear: false,
            papel,
            lamps,
            spec_lut: lut.table(),
            lut_width: ph2d_painter_brush::material::SPEC_LUT as u32,
            rough_levels: ph2d_painter_brush::material::ROUGH_LEVELS as u32,
        })
    }
}

/// Papel com relevo, duas camadas (a de baixo com crista e cera; a de cima com uma crista translúcida que a cruza) e,
/// com `aberto`, um traço por soltar (o `live_h`/`live_c`) na de cima.
fn papel_e_cristas(w: u32, h: u32, aberto: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_substrate_depth(1.0);
    t.set_substrate_roughness(0.5);
    t.toggle_brush_impasto();
    t.set_brush_size_px(h as f32 * 0.3);
    t.paint.brush.color = [0.8, 0.2, 0.2];
    t.paint.brush.impasto_wax = 0.9;
    t.paint.brush.impasto_wax_color = [1.0, 0.6, 0.2];
    let at = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x * w as f32, y * h as f32],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(0.2, 0.5, PointerPhase::Down));
    t.on_canvas_pointer(at(0.5, 0.5, PointerPhase::Move));
    t.on_canvas_pointer(at(0.8, 0.5, PointerPhase::Up));
    t.add_raster_layer("cima").expect("camada de cima");
    t.set_brush_size_px(h as f32 * 0.2);
    t.paint.brush.strength = 0.5;
    t.paint.brush.color = [0.1, 0.3, 0.9];
    t.paint.brush.impasto_wax = 0.0;
    t.on_canvas_pointer(at(0.5, 0.1, PointerPhase::Down));
    t.on_canvas_pointer(at(0.5, 0.6, PointerPhase::Move));
    if !aberto {
        t.on_canvas_pointer(at(0.5, 0.9, PointerPhase::Up));
    }
    t.reconcile_substrate();
    t
}

/// A dobra da linha == os três amostradores, ao bit, em cada linha e uma coluna para lá de cada borda.
fn dobra_igual_aos_amostradores(t: &PainterTool, caso: &str) -> usize {
    let f = t.impasto_fields().expect("campos");
    let (w, h) = (i64::from(t.source_size.0), i64::from(t.source_size.1));
    let n = (w + 2) as usize;
    let (mut alt, mut cob, mut mat) = (vec![0f32; n], vec![0f32; n], vec![[0u8; 7]; n]);
    let mut acesos = 0;
    for y in -1..=h {
        f.dobra_da_linha(y, -1, &mut alt, &mut cob, &mut mat);
        for (k, x) in (-1..=w).enumerate() {
            let (a, c) = (f.height_at(x, y), f.cover_at(x, y));
            assert_eq!(alt[k].to_bits(), a.to_bits(), "{caso}: altura em ({x},{y})");
            assert_eq!(cob[k].to_bits(), c.to_bits(), "{caso}: corpo em ({x},{y})");
            assert_eq!(mat[k], f.material_at(x, y), "{caso}: material em ({x},{y})");
            acesos += usize::from(c > 0.0 && c < 1.0);
        }
    }
    acesos
}

/// A luz da CPU e os planos da placa == os laços de antes (e o passe por píxel), inteira e numa janela fora da origem.
fn pistas_iguais_aos_lacos_de_antes(mut t: PainterTool, caso: &str) -> PainterTool {
    let (w, h) = t.source_size;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("o pool do teste");
    for r in [
        Region { x: 0, y: 0, w, h },
        Region {
            x: 7,
            y: 33,
            w: w - 20,
            h: h - 40,
        },
    ] {
        let base = (r.y..r.y + r.h)
            .flat_map(|y| {
                let i = (y * w + r.x) as usize * 4;
                t.canvas_rgba[i..i + r.w as usize * 4].to_vec()
            })
            .collect::<Vec<_>>();
        let (de_volta, novo, antes) = pool.install({
            let (mut novo, mut antes) = (base.clone(), base.clone());
            move || {
                t.apply_impasto_light(&mut novo, r);
                t.apply_impasto_light_faixas_b10(&mut antes, r);
                (t, novo, antes)
            }
        });
        t = de_volta;
        let mut por_pixel = base.clone();
        t.apply_impasto_light_por_pixel(&mut por_pixel, r);
        assert!(novo != base, "{caso}: a fixture não acendeu nada em {r:?}");
        assert!(
            novo == antes,
            "{caso}: a luz difere das faixas B10 em {r:?}"
        );
        assert!(
            novo == por_pixel,
            "{caso}: a luz difere do passe por píxel em {r:?}"
        );
        let jan = (r.x, r.y, r.w, r.h);
        let (p, q) = (
            t.impasto_gpu_planes_in(jan).expect("planos"),
            t.impasto_gpu_planes_in_por_texel(jan).expect("planos"),
        );
        let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<_>>();
        assert_eq!(
            bits(&p.relief),
            bits(&q.relief),
            "{caso}: relevo dos planos {r:?}"
        );
        assert!(p.cover == q.cover, "{caso}: corpo dos planos {r:?}");
        assert_eq!(
            bits(&p.papel),
            bits(&q.papel),
            "{caso}: papel dos planos {r:?}"
        );
        assert!(
            p.mat0 == q.mat0 && p.mat1 == q.mat1,
            "{caso}: material dos planos {r:?}"
        );
        assert_eq!(p.region, q.region);
    }
    t
}

/// ⭐ **A dobra da linha é a dos três amostradores AO BIT, e as duas pistas que a usam dão o que davam.** Papel com
/// relevo (com e sem o plano do dente), duas camadas com cera e translúcida, o traço aberto e a dobra através de um
/// Gaussiano (`folded`). *Mutações (vistas vermelhas): a camada activa fora do máximo do corpo ⇒ «fechado» reprova; o corpo da tela
/// pela dobra das camadas em vez do `folded` ⇒ o caso «gaussiano» reprova.*
#[test]
fn a_dobra_da_linha_e_a_dos_tres_amostradores_ao_bit() {
    let fechado = papel_e_cristas(150, 101, false);
    assert!(fechado.plano_do_dente.is_some());
    assert!(
        dobra_igual_aos_amostradores(&fechado, "fechado") > 100,
        "sem corpo translúcido"
    );
    let mut fechado = pistas_iguais_aos_lacos_de_antes(fechado, "fechado");
    fechado.plano_do_dente = None;
    dobra_igual_aos_amostradores(&fechado, "sem o plano do dente");
    let aberto = papel_e_cristas(150, 101, true);
    assert!(
        aberto.impasto_fields().expect("campos").live_c.is_some(),
        "o traço não ficou aberto"
    );
    dobra_igual_aos_amostradores(&aberto, "aberto");
    pistas_iguais_aos_lacos_de_antes(aberto, "aberto");
    let mut g = papel_e_cristas(150, 101, false);
    let id = g
        .layers
        .add_adjustment(ph2d_painter_effects::adjustments::AdjustmentKind::GaussianBlur)
        .expect("ajuste");
    g.layers.adjustment_mut(id).expect("ajuste").params =
        ph2d_painter_effects::adjustments::AdjustmentParams::GaussianBlur(
            ph2d_painter_effects::adjustments::GaussianBlurParams { radius: 3.0 },
        );
    assert!(
        g.impasto_fields().expect("campos").folded.is_some(),
        "o Gaussiano não dobrou"
    );
    dobra_igual_aos_amostradores(&g, "gaussiano");
    pistas_iguais_aos_lacos_de_antes(g, "gaussiano");
}

/// Sonda: as duas pistas a 4K com papel (plano do dente em dia) e duas cristas, laço de antes × dobra da linha,
/// intercalados, o MÍNIMO de 3. `bash scripts/cargo-test-narrow.sh ph2d-tool-painter --profile smoke -- --ignored
/// sonda_dobra_da_linha_4k --nocapture`
#[test]
#[ignore = "sonda de custo"]
fn sonda_dobra_da_linha_4k() {
    let (w, h) = (3840, 2160);
    let t = papel_e_cristas(w, h, false);
    let r = Region { x: 0, y: 0, w, h };
    let mut m = [[f64::MAX; 2]; 2];
    let ms = |t0: std::time::Instant| t0.elapsed().as_secs_f64() * 1e3;
    for _ in 0..3 {
        let mut px = t.canvas_rgba.to_vec();
        let t0 = std::time::Instant::now();
        t.apply_impasto_light_faixas_b10(&mut px, r);
        m[0][0] = m[0][0].min(ms(t0));
        let t0 = std::time::Instant::now();
        let _ = t.impasto_gpu_planes_in_por_texel((0, 0, w, h));
        m[1][0] = m[1][0].min(ms(t0));
        let mut px = t.canvas_rgba.to_vec();
        let t0 = std::time::Instant::now();
        t.apply_impasto_light(&mut px, r);
        m[0][1] = m[0][1].min(ms(t0));
        let t0 = std::time::Instant::now();
        let _ = t.impasto_gpu_planes_in((0, 0, w, h));
        m[1][1] = m[1][1].min(ms(t0));
    }
    eprintln!(
        "[dobra da linha 4K] luz CPU {:.1} → {:.1} ms · planos da placa {:.1} → {:.1} ms",
        m[0][0], m[0][1], m[1][0], m[1][1]
    );
}
