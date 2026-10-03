//! ⭐⭐⭐ **O MATCAP INTACTO** — a impressão digital do Matcap em todas as cenas, na placa e na CPU.
//!
//! Instrumento da retirada do Render traçado (03/10): corre-se ANTES e DEPOIS de mexer na marcha
//! partilhada e as duas listas têm de ser iguais ao byte. Imprime uma linha por cena e câmara.
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib matcap_intacto -- --ignored --nocapture`

use std::sync::atomic::AtomicBool;

fn impressao(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[test]
#[ignore = "instrumento: imprime as impressões digitais do Matcap"]
fn matcap_intacto_impressoes() {
    let matcap = super::super::load_matcap();
    let look = ph2d_view_transform::Look::default();
    let reg = ph2d_field_eval::hybrid::Registry::default();
    let tracer = crate::gpu_frame::shared();
    let (w, h) = (320u32, 240u32);
    let camaras = [(0.0_f32, 0.0_f32), (0.7, 0.35)];
    for n in 0..crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let doc = crate::smoke::scenes::scene(n);
        for (k, (yaw, pitch)) in camaras.iter().enumerate() {
            let cam = ph2d_field_render::Orbit::from_yaw_pitch(*yaw, *pitch);
            let cpu = ph2d_field_render::trace_cancellable(
                &doc,
                &reg,
                &cam,
                w,
                h,
                &AtomicBool::new(false),
                crate::preview::re_amostra_a_silhueta(),
                None,
            )
            .map(|g| {
                impressao(&ph2d_field_render::shade_with(
                    &g,
                    &ph2d_field_render::Matcap {
                        side: matcap.side,
                        rgb_linear: &matcap.rgb,
                    },
                    look,
                    super::super::BACKGROUND,
                ))
            });
            let placa = |entrega: Option<(u32, u32)>| {
                let t = tracer?;
                if !crate::gpu_frame::takes_the_frame(Some(t), &doc, &reg) {
                    return None;
                }
                crate::gpu_frame::pinta_matcap_com(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &ph2d_field_gpu::matcap::MatcapSetup {
                        rgb_linear: &matcap.rgb,
                        side: matcap.side,
                        chave: matcap.chave,
                        stops: look.exposure_stops,
                        view: ph2d_view_transform::wgsl::view_code(look.view),
                        background: super::super::BACKGROUND,
                        entrega,
                    },
                    w,
                    h,
                    crate::gpu_frame::Sonda {
                        entrega,
                        ..crate::gpu_frame::Sonda::default()
                    },
                )
                .map(|p| (impressao(&p.rgba), p.edges))
            };
            println!(
                "MATCAP cena {n:2} cam {k} cpu {:?} placa {:?} ampliada {:?}",
                cpu,
                placa(None),
                placa(Some((2 * w, 2 * h)))
            );
        }
    }
}
