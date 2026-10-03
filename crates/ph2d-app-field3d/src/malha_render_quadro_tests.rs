//! A malha cai onde o traçado, o gizmo e o contorno a desenham.

use ph2d_field_render::{Lens, Orbit, Screen};

fn projeta(m: &[[f32; 4]; 4], p: [f32; 3], (w, h): (u32, u32)) -> ([f32; 2], f32) {
    let c = [0, 1, 2, 3].map(|i| m[0][i] * p[0] + m[1][i] * p[1] + m[2][i] * p[2] + m[3][i]);
    let (x, y, z) = (c[0] / c[3], c[1] / c[3], c[2] / c[3]);
    ([(x + 1.0) * 0.5 * w as f32, (1.0 - y) * 0.5 * h as f32], z)
}

/// ⭐ **A projeção é a do `Orbit`** — perspectiva e ortográfica, ecrã largo e alto, câmara girada
/// e afastada do centro: o pixel da matriz é o `Orbit::project`, e a profundidade cai em `0..1`.
#[test]
fn a_projecao_e_a_do_orbit() {
    let mut pior = 0.0f32;
    for lens in [Lens::Perspective { half_fov: 0.4 }, Lens::Ortho] {
        for (yaw, pitch) in [(0.72, 0.52), (-1.9, -0.3), (2.6, 1.1)] {
            for tamanho in [(640u32, 360u32), (300, 900)] {
                let mut cam = Orbit::from_yaw_pitch(yaw, pitch);
                cam.lens = lens;
                cam.target = [0.2, -0.1, 0.3];
                cam.half_extent = 1.3;
                let m = super::camera(&cam, tamanho).view_proj;
                let screen = Screen::new(tamanho.0, tamanho.1, cam.half_extent);
                for p in [
                    [0.0, 0.0, 0.0],
                    [0.5, 0.3, -0.4],
                    [-0.7, 0.1, 0.6],
                    [0.2, -0.6, 0.0],
                ] {
                    let Some((px, _)) = cam.project(p, screen) else {
                        continue;
                    };
                    let (q, z) = projeta(&m, p, tamanho);
                    pior = pior.max((q[0] - px[0]).abs()).max((q[1] - px[1]).abs());
                    assert!(
                        (0.0..=1.0).contains(&z),
                        "profundidade fora: {z} ({lens:?})"
                    );
                }
            }
        }
    }
    assert!(pior < 1.0e-2, "a malha cai {pior} px longe do traçado");
}

/// ⭐⭐⭐ **O BRILHO DO DESENHISTA DE JOGO É A LEI DA CPU** — a cadeia em passes de desenho e a
/// composição depois do olhar, medidas em BYTES contra `ph2d_bloom::halo` + `soma_halo` (a lei que o
/// Render traçado já responde), sobre a MESMA cena-linear e as MESMAS baterias do gate do traçado
/// (`brilho_parity_tests::o_halo_e_o_mesmo_nos_dois_motores`), mais um olhar com exposição e vista.
///
/// ⚠️ A cena é feita de valores EXACTOS em `f16` (múltiplos de `1/64`), para a pergunta ser a da
/// cadeia e não a da quantização da entrada; os níveis do meio são `Rgba16Float` e é isso que a
/// barra de `2` bytes absorve. O controlo (os dois lados ACENDERAM) vem antes da barra.
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib \
///   o_brilho_do_desenhista_de_jogo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "precisa de aparelho"]
fn o_brilho_do_desenhista_de_jogo_e_a_lei_da_cpu() {
    use ph2d_bloom::{Bloom, BloomParams};
    use ph2d_view_transform::{Look, ViewTransform};
    let (w, h) = (192u32, 128u32);
    let constantes = crate::studio_wgsl::constants();
    let tabela = crate::studio_wgsl::tables();
    let Some(mut fw) = ph2d_mesh_forward::Forward::no_aparelho(&ph2d_mesh_forward::Ambiente {
        wgsl: crate::studio_wgsl::SOURCE,
        constantes: &constantes,
        tabela: &tabela,
        piso_luz: ph2d_field_render::POINT_LAMP_MIN_DISTANCE,
    }) else {
        println!("sem aparelho — saltado");
        return;
    };
    if !fw.tem_brilho() {
        println!("esta placa não tem o brilho — saltado");
        return;
    }

    // A cena: um disco quente, um ponto a ESTOURAR (o tecto), um degradê, e uma mancha de chão
    // (preto, meio transparente) que não entra no brilho mas recebe o halo.
    let mut cena = vec![[0.0f32; 4]; (w * h) as usize];
    let mut chao = vec![false; cena.len()];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let (dx, dy) = (x as f32 - 64.0, y as f32 - 64.0);
            if dx * dx + dy * dy <= 24.0 * 24.0 {
                cena[i] = [2.0, 1.25, 0.5, 1.0];
            } else if (129..132).contains(&x) && (39..42).contains(&y) {
                cena[i] = [40.0, 40.0, 40.0, 1.0];
            } else if (110..170).contains(&x) && (80..110).contains(&y) {
                cena[i] = [0.5, 0.25 + (x - 110) as f32 / 32.0, 1.0, 1.0];
            } else if (20..60).contains(&x) && (100..124).contains(&y) {
                chao[i] = true;
            }
        }
    }
    let rgb: Vec<[f32; 3]> = cena.iter().map(|c| [c[0], c[1], c[2]]).collect();

    let base = BloomParams::default();
    let liga = |params| Bloom {
        enabled: true,
        params,
    };
    let baterias = [
        ("fábrica", liga(base)),
        (
            "raio 8",
            liga(BloomParams {
                radius: 8.0,
                ..base
            }),
        ),
        (
            "tingido",
            liga(BloomParams {
                tint: [1.0, 0.25, 0.10, 1.0],
                saturation: 0.0,
                ..base
            }),
        ),
        ("tecto 2", liga(BloomParams { clamp: 2.0, ..base })),
        (
            "anamórfico",
            liga(BloomParams {
                stretch: 3.0,
                angle: 30.0,
                ..base
            }),
        ),
    ];
    let olhares = [
        Look::default(),
        Look {
            exposure_stops: 1.0,
            view: ViewTransform::Neutral,
        },
    ];

    println!("\n  OLHAR        BATERIA        acendeu(cpu)  acendeu(gpu)  fora(>1)  pior");
    let mut pior_global = 0u8;
    for look in olhares {
        let olhar: Vec<[f32; 4]> = cena
            .iter()
            .zip(&chao)
            .map(|(c, &no_chao)| {
                if no_chao {
                    [0.0, 0.0, 0.0, 0.375]
                } else if c[3] > 0.0 {
                    let d = look.apply([c[0], c[1], c[2]]);
                    [d[0], d[1], d[2], 1.0]
                } else {
                    [0.0; 4]
                }
            })
            .collect();
        let vista = ph2d_view_transform::wgsl::view_code(look.view);
        let sem = fw
            .brilho_sobre(
                &olhar,
                &cena,
                (w, h),
                &Bloom::default(),
                look.exposure_stops,
                vista,
            )
            .expect("o quadro sem brilho");
        for (rot, bloom) in baterias {
            let gpu = fw
                .brilho_sobre(&olhar, &cena, (w, h), &bloom, look.exposure_stops, vista)
                .expect("o quadro com brilho");
            let mut cpu = sem.clone();
            let halo = ph2d_bloom::halo(&rgb, w as usize, h as usize, &bloom);
            ph2d_field_render::soma_halo(
                &mut cpu,
                &halo,
                &ph2d_field_render::Presentation::of(look),
            );
            let acendeu = |a: &[u8]| a.iter().zip(&sem).filter(|(x, y)| x != y).count();
            let (mut fora, mut pior) = (0usize, 0u8);
            for (c, g) in cpu.iter().zip(&gpu) {
                let d = c.abs_diff(*g);
                if d > 1 {
                    fora += 1;
                }
                pior = pior.max(d);
            }
            pior_global = pior_global.max(pior);
            println!(
                "  {:<12} {rot:<14} {:>12} {:>13} {fora:>9} {pior:>5}",
                format!("{:?}{:+}", look.view, look.exposure_stops),
                acendeu(&cpu),
                acendeu(&gpu)
            );
            assert!(
                acendeu(&cpu) > 500,
                "{rot}: a REFERÊNCIA mal acendeu — a fixtura não tem o fenómeno"
            );
            assert!(
                acendeu(&gpu) > 500,
                "{rot}: o DESENHISTA não acendeu — a cadeia não correu"
            );
            assert!(
                pior <= 2,
                "{rot}: o desenhista e a CPU discordam em {pior} bytes ({fora} canais fora)"
            );
        }
    }
    println!("  pior de todas: {pior_global} bytes\n");
}
