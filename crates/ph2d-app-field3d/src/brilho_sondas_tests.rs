//! ⏱️ **AS SONDAS DO BRILHO** — as que IMPRIMEM uma tabela e não afirmam nada.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super::cena_tests`], e o corte foi forçado pelo tecto de
//! LOC:** ali estão os gates que reprovam; aqui está o instrumento com que se olha. ⛔ As duas
//! coisas no mesmo ficheiro leem-se iguais numa listagem de testes, e *ninguém lê uma tabela que
//! passa* — foi essa confusão que deixou três «gates» de paridade a serem impressoras com o
//! veredito escrito no nome (`docs/Render3d/11` §11).

/// ⭐⭐⭐ **A SONDA DO REPORT DE 19/09** — *«bem diferente do que vc descreveu»*, com foto.
///
/// Ela mede o halo **no tamanho da janela do dono** e não num quadro de teste, porque a suspeita é
/// exactamente essa: a sonda dos tectos correu a `96×96` e o report vem de `~1920`.
///
/// ```text
/// cargo test -p ph2d-app-field3d --lib o_halo_no_tamanho_do_dono -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a tabela do report, não afirma"]
fn o_halo_no_tamanho_do_dono() {
    use crate::render_light::{StudioSky, lamps};
    let doc = crate::smoke::scenes::scene(36);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let folhas: Vec<bevy_ecs::entity::Entity> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("filhos")
        .iter()
        .copied()
        .collect();
    for (e, m) in folhas
        .iter()
        .zip(crate::smoke::scenes::materiais_da_cena(36).expect("material"))
    {
        sim.world_mut().entity_mut(*e).insert(m);
    }
    let rig = ph2d_light::LightRig::default();
    let lam = lamps(&rig);
    let light = ph2d_field_render::Lighting {
        lamps: &lam,
        points: &[],
        sky: &StudioSky,
        shadows: None,
    };
    // ⭐ PRIMEIRO: as três bolas distinguem-se? O roteiro promete «fraca, média, forte».
    {
        let (w, h) = (900u32, 700u32);
        let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let t = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
        let px = ph2d_field_render::shade_render(
            &g,
            &cam,
            &t.surfaces_for(),
            &light,
            &ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK),
            [0, 0, 0, 255],
        );
        let donos = t.owners.as_ref().expect("quatro folhas");
        let mut soma = vec![(0u64, 0u64); folhas.len()];
        let mut saturados = vec![0u64; folhas.len()];
        for i in 0..(w * h) as usize {
            if !g.hit[i] {
                continue;
            }
            if let Some(k) = donos.at(g.point[i]) {
                let m = px[i * 4].max(px[i * 4 + 1]).max(px[i * 4 + 2]);
                soma[k].0 += u64::from(m);
                soma[k].1 += 1;
                if m >= 250 {
                    saturados[k] += 1;
                }
            }
        }
        println!("\n=== AS TRÊS LUZES SE DISTINGUEM? (o roteiro promete fraca/média/forte) ===");
        println!(
            "{:>8} {:>10} {:>12} {:>14}",
            "folha", "emissão", "byte médio", "% saturado"
        );
        for (k, (s, n)) in soma.iter().enumerate() {
            let emissao = crate::smoke::scenes::edge::BRILHOS_DA_CENA.get(k).copied();
            #[allow(clippy::cast_precision_loss)]
            let media = *s as f64 / (*n).max(1) as f64;
            #[allow(clippy::cast_precision_loss)]
            let sat = saturados[k] as f64 / (*n).max(1) as f64 * 100.0;
            println!(
                "{k:>8} {:>10} {media:>12.1} {sat:>13.1}%",
                emissao.map_or("(barra)".to_string(), |e| format!("{e:.0}"))
            );
        }
    }

    // ⭐ SEGUNDO: o HALO distingue-as? (é a única coisa que pode, acima do ponto branco)
    {
        let (w, h) = (900u32, 700u32);
        let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let t = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
        let quadro = |b: ph2d_field_render::Bloom| {
            ph2d_field_render::shade_render(
                &g,
                &cam,
                &t.surfaces_for(),
                &light,
                &ph2d_field_render::Presentation {
                    bloom: b,
                    ..ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK)
                },
                [0, 0, 0, 255],
            )
        };
        let sem = quadro(ph2d_field_render::Bloom::default());
        let donos = t.owners.as_ref().expect("quatro folhas");
        // O centro de cada bola no ECRÃ.
        let mut cen = vec![(0f64, 0f64, 0f64); folhas.len()];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                if let Some(k) = g.hit[i].then(|| donos.at(g.point[i])).flatten() {
                    cen[k].0 += x as f64;
                    cen[k].1 += y as f64;
                    cen[k].2 += 1.0;
                }
            }
        }
        println!("\n=== O HALO DISTINGUE-AS? (alcance a partir do centro de cada bola) ===");
        println!(
            "{:>8} {:>10} {:>14} {:>16}",
            "folha", "emissão", "alcance px", "limiar que a mata"
        );
        for (k, c) in cen.iter().enumerate() {
            if c.2 < 1.0 {
                continue;
            }
            let (cx, cy) = (c.0 / c.2, c.1 / c.2);
            let com = quadro(ph2d_field_render::Bloom {
                enabled: true,
                ..ph2d_field_render::Bloom::default()
            });
            // O pixel de fundo aceso MAIS LONGE do centro desta bola e mais perto dela que de
            // qualquer outra — senão o halo do vizinho conta para esta.
            let mut alcance = 0f64;
            for y in 0..h as usize {
                for x in 0..w as usize {
                    let i = y * w as usize + x;
                    if g.hit[i] || com[i * 4..i * 4 + 3] == sem[i * 4..i * 4 + 3] {
                        continue;
                    }
                    let d = ((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt();
                    let minha = cen.iter().enumerate().all(|(j, o)| {
                        j == k
                            || o.2 < 1.0
                            || d <= ((x as f64 - o.0 / o.2).powi(2)
                                + (y as f64 - o.1 / o.2).powi(2))
                            .sqrt()
                    });
                    if minha {
                        alcance = alcance.max(d);
                    }
                }
            }
            // E qual limiar a apaga: sobe-se até o halo dela sumir.
            let mut mata = f32::INFINITY;
            for lim in [1.0f32, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0] {
                let q = quadro(ph2d_field_render::Bloom {
                    enabled: true,
                    params: ph2d_field_render::BloomParams {
                        threshold: lim,
                        ..ph2d_field_render::BloomParams::default()
                    },
                });
                let vivo = (0..(w * h) as usize).any(|i| {
                    !g.hit[i]
                        && q[i * 4..i * 4 + 3] != sem[i * 4..i * 4 + 3]
                        && ((i % w as usize) as f64 - cx).powi(2)
                            + ((i / w as usize) as f64 - cy).powi(2)
                            < 40000.0
                });
                if !vivo {
                    mata = lim;
                    break;
                }
            }
            println!(
                "{k:>8} {:>10} {alcance:>14.0} {mata:>16.0}",
                crate::smoke::scenes::edge::BRILHOS_DA_CENA
                    .get(k)
                    .map_or("(barra)".to_string(), |e| format!("{e:.0}"))
            );
        }
    }

    println!(
        "\n{:>12} {:>8} {:>10} {:>10} {:>10} {:>12}",
        "janela", "níveis", "acesos", "% fundo", "Δ máx", "alcance px"
    );
    for (w, h) in [(96u32, 96u32), (320, 240), (960, 540), (1900, 1000)] {
        let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let t = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
        let pinta = |b: ph2d_field_render::Bloom| {
            ph2d_field_render::shade_render(
                &g,
                &cam,
                &t.surfaces_for(),
                &light,
                &ph2d_field_render::Presentation {
                    bloom: b,
                    ..ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK)
                },
                [0, 0, 0, 255],
            )
        };
        let sem = pinta(ph2d_field_render::Bloom::default());
        let com = pinta(ph2d_field_render::Bloom {
            enabled: true,
            ..ph2d_field_render::Bloom::default()
        });
        let (mut acesos, mut fundo, mut dmax, mut alcance) = (0usize, 0usize, 0u8, 0f64);
        // O centro da peça no ecrã, para medir o ALCANCE em píxeis.
        let (mut cx, mut cy, mut n) = (0f64, 0f64, 0f64);
        for y in 0..h as usize {
            for x in 0..w as usize {
                if g.hit[y * w as usize + x] {
                    cx += x as f64;
                    cy += y as f64;
                    n += 1.0;
                }
            }
        }
        let (cx, cy) = (cx / n.max(1.0), cy / n.max(1.0));
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                if g.hit[i] {
                    continue;
                }
                fundo += 1;
                let d = (0..3)
                    .map(|c| com[i * 4 + c].saturating_sub(sem[i * 4 + c]))
                    .max()
                    .unwrap_or(0);
                if d > 0 {
                    acesos += 1;
                    dmax = dmax.max(d);
                    alcance =
                        alcance.max(((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt());
                }
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let pct = acesos as f64 / fundo.max(1) as f64 * 100.0;
        println!(
            "{:>12} {:>8} {acesos:>10} {pct:>9.1}% {dmax:>10} {alcance:>12.0}",
            format!("{w}×{h}"),
            ph2d_bloom::levels_that_fit(w as usize, h as usize)
        );
    }
}
/// Despeja o quadro do RENDER com e sem brilho, para se VER o halo (`#[ignore]`).
///
/// ⚠️ **A foto da janela não serve para isto:** a cena abre em Matcap, e o matcap não corre o
/// brilho. *O único sítio onde o halo se vê é o caminho do Render, e é este o despejo dele.*
///
/// ⛔⛔⛔ **E ELA NÃO AUDITA O QUADRO — só a COR dele.** O PPM não tem canal alfa, e foi com esta
/// sonda que eu dei o halo por bom antes do report do dono de 19/09: a imagem estava perfeita e o
/// quadro saía com **cobertura zero** em todo o halo, logo o canvas apagava-o inteiro. *Um despejo
/// que deita fora um canal não pode auditar esse canal* — quem quiser o veredito usa o gate
/// [`o_halo_da_cena_chega_com_cobertura_sobre_o_fundo_do_modulo`], que mede o alfa.
///
/// ```text
/// PH2D_BLOOM_DUMP=/tmp/x cargo test -p ph2d-app-field3d --lib despeja_o_halo -- --ignored
/// ```
#[test]
#[ignore = "sonda: despeja PPMs do halo"]
fn despeja_o_halo() {
    use crate::render_light::{StudioSky, lamps};
    let Ok(dir) = std::env::var("PH2D_BLOOM_DUMP") else {
        println!("sem PH2D_BLOOM_DUMP — nada a fazer");
        return;
    };
    std::fs::create_dir_all(&dir).expect("a pasta");
    let (w, h) = (960u32, 720u32);
    let doc = crate::smoke::scenes::scene(36);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let folhas: Vec<bevy_ecs::entity::Entity> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("filhos")
        .iter()
        .copied()
        .collect();
    for (e, m) in folhas
        .iter()
        .zip(crate::smoke::scenes::materiais_da_cena(36).expect("material"))
    {
        sim.world_mut().entity_mut(*e).insert(m);
    }
    let t = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
    let rig = ph2d_light::LightRig::default();
    let lam = lamps(&rig);
    let light = ph2d_field_render::Lighting {
        lamps: &lam,
        points: &[],
        sky: &StudioSky,
        shadows: None,
    };
    for (nome, b) in [
        ("sem", ph2d_field_render::Bloom::default()),
        (
            "com",
            ph2d_field_render::Bloom {
                enabled: true,
                ..ph2d_field_render::Bloom::default()
            },
        ),
        (
            "forte",
            ph2d_field_render::Bloom {
                enabled: true,
                params: ph2d_field_render::BloomParams {
                    intensity: 2.0,
                    radius: 8.0,
                    ..ph2d_field_render::BloomParams::default()
                },
            },
        ),
    ] {
        let px = ph2d_field_render::shade_render(
            &g,
            &cam,
            &t.surfaces_for(),
            &light,
            &ph2d_field_render::Presentation {
                bloom: b,
                ..ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK)
            },
            [90, 90, 92, 255],
        );
        let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
        for c in px.as_chunks::<4>().0 {
            ppm.extend_from_slice(&c[..3]);
        }
        let p = format!("{dir}/halo_{nome}.ppm");
        std::fs::write(&p, ppm).expect("escrever");
        println!("{p}");
    }
}
/// Sonda: que ESCADA de luzes cabe nos NOSSOS valores de fábrica? (`#[ignore]`)
///
/// ⚠️ Os nossos são muito mais fortes que os do oráculo (`intensity 0,8` contra `0,3`) — eles estão
/// afinados para as faíscas do Motion. *Uma cena herdada de outro modelo tem de ser re-afinada.*
#[test]
#[ignore = "sonda"]
fn que_escada_cabe_na_fabrica() {
    use crate::render_light::{StudioSky, lamps};
    let (w, h) = (480u32, 360u32);
    let doc = crate::smoke::scenes::scene(36);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let folhas: Vec<bevy_ecs::entity::Entity> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("filhos")
        .iter()
        .copied()
        .collect();
    let rig = ph2d_light::LightRig::default();
    let lam = lamps(&rig);
    let light = ph2d_field_render::Lighting {
        lamps: &lam,
        points: &[],
        sky: &StudioSky,
        shadows: None,
    };
    println!(
        "\n{:>18} {:>12} {:>12} {:>12}",
        "escada", "fundo lavado", "fundo aceso", "halo médio"
    );
    for escada in [
        [0.5f32, 1.5, 4.5],
        [1.2, 3.0, 8.0],
        [2.0, 6.0, 18.0],
        [1.5, 4.0, 12.0],
        [1.1, 2.2, 4.4],
    ] {
        for (e, f) in folhas.iter().zip(escada.iter().copied()) {
            sim.world_mut()
                .entity_mut(*e)
                .insert(ph2d_field_ecs::FieldMaterial {
                    emission: f,
                    emission_color: [1.0, 0.92, 0.80],
                    specular_weight: 0.0,
                    ..ph2d_field_ecs::FieldMaterial::default()
                });
        }
        let t = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
        let quadro = |b: ph2d_field_render::Bloom| {
            ph2d_field_render::shade_render(
                &g,
                &cam,
                &t.surfaces_for(),
                &light,
                &ph2d_field_render::Presentation {
                    bloom: b,
                    ..ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK)
                },
                [90, 90, 92, 255],
            )
        };
        let sem = quadro(ph2d_field_render::Bloom::default());
        let com = quadro(ph2d_field_render::Bloom {
            enabled: true,
            params: ph2d_field_render::BloomParams::default(),
        });
        let (mut lavado, mut aceso, mut soma, mut n) = (0usize, 0usize, 0i64, 0i64);
        for i in 0..(w * h) as usize {
            if g.hit[i] {
                continue;
            }
            n += 1;
            let d = i64::from(com[i * 4]) - i64::from(sem[i * 4]);
            soma += d;
            if d > 2 {
                aceso += 1;
            }
            if com[i * 4] >= 250 {
                lavado += 1;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let (pl, pa) = (
            lavado as f64 / n as f64 * 100.0,
            aceso as f64 / n as f64 * 100.0,
        );
        #[allow(clippy::cast_precision_loss)]
        let media = soma as f64 / n as f64;
        println!("{escada:>18?} {pl:>11.1}% {pa:>11.1}% {media:>12.1}");
    }
}
