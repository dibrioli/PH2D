//! ⏱📷 **AS SONDAS DO RENDER POR MALHA** (ignoradas — correm à mão, em release, com a placa):
//! o tempo de um quadro a GIRAR (cada quadro uma câmara nova, logo cada um desenhado de raiz) e a
//! FOTO do quadro, composta sobre cinzento como o ecrã a compõe.
//!
//! ```text
//! PH2D_GPU=1 PH2D_SONDA_DIR=/tmp/fotos bash scripts/ph2d-run.sh cargo test --release \
//!     -p ph2d-app-field3d --lib sonda_do_render_por_malha -- --ignored --nocapture
//! ```

use crate::scene::lasso_tests::armed_with;

/// O fundo da foto (o cinzento do viewport), em sRGB.
const FUNDO: [f32; 3] = [0.16, 0.16, 0.17];

fn grava_ppm(caminho: &std::path::Path, rgba: &[u8], (w, h): (u32, u32)) {
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for px in rgba.as_chunks::<4>().0 {
        let a = f32::from(px[3]) / 255.0;
        for k in 0..3 {
            // Premultiplicado em sRGB, como o vello compõe: c + fundo·(1 − a).
            let v = f32::from(px[k]) / 255.0 + FUNDO[k] * (1.0 - a);
            out.push((v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }
    }
    std::fs::write(caminho, out).expect("grava a foto");
}

#[test]
#[ignore = "sonda: placa + release, à mão"]
fn sonda_do_render_por_malha() {
    let dir = std::env::var("PH2D_SONDA_DIR")
        .unwrap_or_else(|_| std::env::temp_dir().display().to_string());
    let cenas: Vec<u32> = std::env::var("PH2D_SONDA_CENAS")
        .ok()
        .map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![28, 36]);
    let tamanho = (1920u32, 1080u32);
    for n in cenas {
        let doc = crate::smoke::scenes::scene(n);
        armed_with(&doc, |sim| {
            // ⛔ O mundo novo de cada cena dá à raiz o MESMO `Entity` da anterior, e o estado do
            // Render (por thread) não recomeçava: a 2.ª cena era medida com as malhas da 1.ª.
            crate::malha_render_estado::sync(sim, false, false);
            let slot = crate::shading::Shading::ALL
                .iter()
                .position(|s| *s == crate::shading::Shading::Render)
                .expect("Render");
            ph2d_panel_model3d::state::push_intent_for_test(
                ph2d_panel_model3d::ModelIntent::SetShading { slot },
            );
            crate::scene::apply_intents_for_test(sim.world_mut(), &[]);
            // As cores da cena — o smoke real semeia-as ao abrir (`seed_materials`).
            if let Some(mats) = crate::smoke::scenes::materiais_da_cena(n) {
                let world = sim.world_mut();
                let mut q =
                    world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
                let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
                let folhas = crate::materials::folhas(world, root);
                for ((e, _, _), m) in folhas.iter().zip(mats) {
                    world.entity_mut(*e).insert(m);
                }
            }
            // `PH2D_SONDA_BRILHO=1` liga o brilho de fábrica — o que o artista tem ao clicar «On».
            let brilho = std::env::var("PH2D_SONDA_BRILHO").is_ok_and(|v| v == "1");
            crate::smoke::with_smoke(|s| {
                s.vp_mut().cam = ph2d_field_render::Orbit::from_yaw_pitch(0.72, 0.52);
                crate::input::frame_the_part(s);
                if brilho {
                    s.set_bloom(ph2d_field_render::Bloom {
                        enabled: true,
                        ..s.bloom
                    });
                }
            });
            let t0 = std::time::Instant::now();
            loop {
                crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
                if crate::malha_render_estado::com(|e| !e.esperando()).unwrap_or(false)
                    || t0.elapsed().as_secs() > 30
                {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let entrar = t0.elapsed().as_secs_f64() * 1e3;
            let doc = crate::smoke::with_smoke(|s| s.doc.clone())
                .flatten()
                .expect("doc");
            let mut tempos = Vec::new();
            let mut primeira = None;
            for k in 0..61 {
                let t = std::time::Instant::now();
                let feito = crate::smoke::with_smoke(|s| {
                    if k > 0 {
                        s.vp_mut()
                            .cam
                            .turn_world([0.0, 1.0, 0.0], 3.0_f32.to_radians());
                    }
                    crate::malha_render_quadro::desenha(s, s.active, tamanho, &doc, false)
                })
                .expect("armado");
                let crate::malha_render_quadro::Feito::Novo(rgba) = feito else {
                    println!("SONDA cena {n}: sem quadro ({feito:?})");
                    return;
                };
                if k > 0 {
                    tempos.push(t.elapsed().as_secs_f64() * 1e3);
                }
                if primeira.is_none() {
                    primeira = Some(rgba);
                }
            }
            tempos.sort_by(f64::total_cmp);
            let (objs, tris) = crate::malha_render_estado::com(|e| {
                (
                    e.objetos.len(),
                    e.objetos
                        .iter()
                        .map(|o| o.malha.triangulos())
                        .sum::<usize>(),
                )
            })
            .unwrap_or_default();
            let sufixo = if brilho { "_brilho" } else { "" };
            let caminho =
                std::path::Path::new(&dir).join(format!("render_malha_cena_{n}{sufixo}.ppm"));
            grava_ppm(&caminho, &primeira.expect("quadro"), tamanho);
            println!(
                "SONDA cena {n}: {objs} objetos · {tris} triângulos · entrar {entrar:.0} ms · quadro a girar \
                 {}x{}: mediana {:.2} ms · p90 {:.2} ms · máx {:.2} ms · foto {}",
                tamanho.0,
                tamanho.1,
                tempos[tempos.len() / 2],
                tempos[tempos.len() * 9 / 10],
                tempos[tempos.len() - 1],
                caminho.display()
            );
        });
    }
}
