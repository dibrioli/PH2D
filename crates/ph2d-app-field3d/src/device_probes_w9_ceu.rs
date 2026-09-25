//! ⏱️⭐⭐⭐⭐ **A OCLUSÃO DO CÉU MARCHADA NUMA GRADE ASSADA** — a sonda que decide se ela paga
//! (`docs/Render3d/03_o_plano.md` §W9, «a oclusão na grade»).
//!
//! Medido antes dela: no modo Render a mexer, a oclusão é `60`–`80 %` do quadro (`48` cones por
//! pixel, cada um uma marcha sobre a árvore inteira). A pergunta: marchar os cones numa grade
//! assada na placa — o *Distance Field Ambient Occlusion* dos motores de jogo — compra quanto, e
//! quanto move a imagem? O raio primário continua EXACTO (a silhueta e a normal são as da árvore);
//! só a luz do céu muda.

/// ⏱️ **Sonda: relógio e imagem, oclusão exacta contra grade de `n`**, pela porta do produto
/// (`paint_com`, o quadro de movimento). A imagem compara-se byte a byte com a exacta.
#[test]
#[ignore = "sonda de GPU"]
fn diag_a_oclusao_na_grade() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let cam = ph2d_field_render::Orbit::default();
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let olhar = ph2d_view_transform::Look::default();
    let pres = ph2d_field_render::Presentation::of(olhar);
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    println!(
        "\n  {}\n  cena · grade · ms [mín 5] · píxeis >2 B · >8 B · p99 B · máx B",
        super::super::super::super::contexto()
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let quadro = |ceu: Option<u32>| {
            let sonda = crate::gpu_frame::Sonda {
                ceu_na_grade: ceu,
                ..crate::gpu_frame::Sonda::default()
            };
            crate::gpu_frame::paint_com(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &pres,
                [0, 0, 0, 0],
                None,
                W,
                H,
                false,
                sonda,
            )
            .expect("o pintor")
            .rgba
        };
        let relogio = |ceu: Option<u32>| {
            let _ = quadro(ceu);
            let mut m = f64::INFINITY;
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let _ = quadro(ceu);
                m = m.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            m
        };
        let exacta = quadro(None);
        println!("  {cena:4} ·  exacta · {:>7.2}", relogio(None));
        for res in [128u32, 192, 256, 384] {
            let img = quadro(Some(res));
            let mut d: Vec<u8> = exacta
                .as_chunks::<4>()
                .0
                .iter()
                .zip(img.as_chunks::<4>().0)
                .map(|(a, b)| (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap_or(0))
                .collect();
            let acima = |b: u8| d.iter().filter(|x| **x > b).count();
            let (a2, a8) = (acima(2), acima(8));
            d.sort_unstable();
            let p99 = d[d.len() * 99 / 100];
            let max = d.last().copied().unwrap_or(0);
            println!(
                "  {cena:4} · {res:>6} · {:>7.2} · {a2:>8} · {a8:>6} · {p99:>5} · {max:>5}",
                relogio(Some(res))
            );
        }
    }
}

/// ⏱️ **Sonda: o CANAL do céu, exacto contra a grade** — o G-buffer volta pela `frame`, e compara-se
/// o valor da oclusão pixel a pixel (média com sinal, fracção mais escura, p99 do módulo).
#[test]
#[ignore = "sonda de GPU"]
#[allow(clippy::cast_precision_loss)]
fn diag_o_canal_do_ceu_na_grade() {
    const W: u32 = 480;
    const H: u32 = 270;
    let cam = ph2d_field_render::Orbit::default();
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let luz = [crate::gpu_frame::tests_lampada(&cam).world];
    println!(
        "\n  cena · grade · acertos · média(grade−exacta) · fracção mais escura · p99 |Δ| · máx |Δ|"
    );
    for cena in [26u32, 5, 28] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let ceu = |g: Option<u32>| {
            let sonda = crate::gpu_frame::Sonda {
                ceu_na_grade: g,
                ..crate::gpu_frame::Sonda::default()
            };
            let (c, f, setup) = crate::gpu_frame::pedido(
                &doc,
                &reg,
                &cam,
                &luz,
                None,
                ph2d_field_gpu::trace::MAX_LAMPS,
                sonda,
                W,
                H,
                None,
                true,
            )
            .expect("o pedido");
            let gb = t
                .lock()
                .expect("o traçador")
                .frame(&f, c.sculpts(), setup, W, H);
            (gb.t, gb.ambient)
        };
        let (tt, exacta) = ceu(None);
        for res in [32u32, 128, 512] {
            let (_, grade) = ceu(Some(res));
            let mut d: Vec<f32> = Vec::new();
            let mut soma = 0.0f64;
            let mut escura = 0usize;
            for i in 0..exacta.len() {
                if tt[i] < 0.0 {
                    continue;
                }
                let x = grade[i] - exacta[i];
                soma += f64::from(x);
                if x < -1e-3 {
                    escura += 1;
                }
                d.push(x.abs());
            }
            let n = d.len().max(1);
            d.sort_by(f32::total_cmp);
            println!(
                "  {cena:4} · {res:>5} · {n:>7} · {:>+9.4} · {:>6.3} · {:>7.4} · {:>7.4}",
                soma / n as f64,
                escura as f64 / n as f64,
                d[n * 99 / 100],
                d.last().copied().unwrap_or(0.0)
            );
        }
    }
}

/// ⏱️ **Sonda: o quadro ASSENTE partido** — com e sem o ricochete (o passageiro caro da bandeira
/// `assente`). É ele que fica no caminho quando a mão HESITA um quadro a meio de uma órbita: o
/// trabalho da placa não se cancela, e o quadro de movimento seguinte espera por ele.
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_quadro_assente_partido() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let cam = ph2d_field_render::Orbit::default();
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    println!(
        "\n  {}\n  cena · a mexer ms · assente ms · assente SEM ricochete ms [mín 3]",
        super::super::super::super::contexto()
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let mede = |assente: bool, ricochete: bool| {
            let sonda = crate::gpu_frame::Sonda {
                ricochete,
                ..crate::gpu_frame::Sonda::default()
            };
            let mut m = f64::INFINITY;
            for i in 0..4 {
                let t0 = std::time::Instant::now();
                let _ = crate::gpu_frame::paint_com(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &luz,
                    &surfaces,
                    &pres,
                    [0, 0, 0, 0],
                    None,
                    W,
                    H,
                    assente,
                    sonda,
                );
                if i > 0 {
                    m = m.min(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
            m
        };
        println!(
            "  {cena:4} · {:>7.2} · {:>7.2} · {:>7.2}",
            mede(false, true),
            mede(true, true),
            mede(true, false)
        );
    }
}

/// ⏱️📷 **Sonda: a LUZ ENCOSTADA à peça, com CHÃO** — o report *«ao aproximar a luz do objeto
/// resultados muito ruins de render»* (2026-09-24, foto da cena `=28` com a lâmpada dentro de um nó).
/// Grava o quadro assente do dispositivo em `PH2D_SONDA_DIR` (PPM), para várias posições da luz.
#[test]
#[ignore = "sonda de GPU que grava imagens"]
fn diag_a_luz_encostada_com_chao() {
    const W: u32 = 1280;
    const H: u32 = 720;
    let Ok(dir) = std::env::var("PH2D_SONDA_DIR") else {
        println!("sem PH2D_SONDA_DIR");
        return;
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let cam = ph2d_field_render::Orbit::default();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    println!("bola {:?} r {} chão {:?}", bola.center, bola.radius, chao);
    let c = bola.center;
    let r = bola.radius;
    for (nome, fx, fy) in [("dentro", 0.75, 0.0)] {
        let _ = (fx, fy);
        let luz = [ph2d_field_render::PointLamp {
            world: luz_no_vazio_do_no(&doc, c, r),
            radiance_at_one: [7.0, 7.0, 7.0],
        }];
        for (variante, assente, ricochete, chao_cor) in [
            ("mexer", false, true, true),
            ("assente", true, true, true),
            ("assente_sem_ricochete", true, false, true),
            ("assente_sem_cor_no_chao", true, true, false),
        ] {
            let sonda = crate::gpu_frame::Sonda {
                ricochete,
                chao_recebe_cor: chao_cor,
                ..crate::gpu_frame::Sonda::default()
            };
            let p = crate::gpu_frame::paint_com(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &pres,
                [40, 40, 40, 255],
                chao,
                W,
                H,
                assente,
                sonda,
            )
            .expect("o pintor");
            let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
            for px in p.rgba.as_chunks::<4>().0 {
                ppm.extend_from_slice(&px[..3]);
            }
            let caminho = format!("{dir}/luz_{nome}_{variante}.ppm");
            std::fs::write(&caminho, ppm).expect("grava");
            println!("gravado {caminho}");
        }
    }
}

/// 📷 **Sonda: os CANAIS do chão com a luz encostada** — o céu e a sombra da lâmpada, do G-buffer
/// (`frame`), gravados como imagens cinzentas em `PH2D_SONDA_DIR`. Separa qual das duas leis desenha
/// as riscas em leque do report.
#[test]
#[ignore = "sonda de GPU que grava imagens"]
fn diag_os_canais_do_chao_com_a_luz_encostada() {
    const W: u32 = 1280;
    const H: u32 = 720;
    let Ok(dir) = std::env::var("PH2D_SONDA_DIR") else {
        return;
    };
    let Some(t) = crate::gpu_frame::shared() else {
        return;
    };
    let cam = ph2d_field_render::Orbit::default();
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let c = bola.center;
    let r = bola.radius;
    let luz = [luz_no_vazio_do_no(&doc, c, r)];
    let (campo, fita, setup) = crate::gpu_frame::pedido(
        &doc,
        &reg,
        &cam,
        &luz,
        chao,
        ph2d_field_gpu::trace::MAX_LAMPS,
        crate::gpu_frame::Sonda::default(),
        W,
        H,
        None,
        true,
    )
    .expect("o pedido");
    let gb = t
        .lock()
        .expect("o traçador")
        .frame(&fita, campo.sculpts(), setup, W, H);
    let grava = |nome: &str, v: &dyn Fn(usize) -> f32| {
        let mut ppm = format!("P5\n{W} {H}\n255\n").into_bytes();
        for i in 0..(W * H) as usize {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            ppm.push((v(i).clamp(0.0, 1.0) * 255.0) as u8);
        }
        std::fs::write(format!("{dir}/canal_{nome}.pgm"), ppm).expect("grava");
    };
    grava("ceu", &|i| gb.ambient[i]);
    grava("sombra", &|i| gb.shadow[i * gb.lamps]);
}

/// A luz no VAZIO do nó da direita: o ponto de maior distância à peça numa grelha em volta de
/// `centro + 0,75·r·x̂` — o sítio onde o dono a pôs na foto (dentro do anel, fora do tubo).
fn luz_no_vazio_do_no(doc: &ph2d_field::FieldDoc, c: [f32; 3], r: f32) -> [f32; 3] {
    let campo = ph2d_field_eval::Field::new(doc);
    let mut melhor = ([c[0] + 0.75 * r, c[1], c[2]], f64::NEG_INFINITY);
    for i in -8..=8 {
        for j in -8..=8 {
            for k in -8..=8 {
                #[allow(clippy::cast_precision_loss)]
                let p = [
                    c[0] + 0.75 * r + i as f32 * 0.02 * r,
                    c[1] + j as f32 * 0.02 * r,
                    c[2] + k as f32 * 0.02 * r,
                ];
                let d = campo.at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
                if d > melhor.1 {
                    melhor = (p, d);
                }
            }
        }
    }
    println!("luz no vazio: {:?} a {:.4} da peça", melhor.0, melhor.1);
    melhor.0
}

/// 📏 **A grelha da luz devolvida ao chão contra uma referência fina** — ver o cabeçalho do [`grelha`].
#[path = "device_probes_w9_chao_grelha.rs"]
mod grelha;

/// ⏱️ **Sonda: o quadro ASSENTE com as sondas FRIAS contra GUARDADAS** — a cura do travão ao girar
/// (`ph2d_field_gpu::sondas_na_placa`). A luz é FIXA em mundo, senão orbitar trocava a chave.
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_assente_com_as_sondas_guardadas() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let luz = [ph2d_field_render::PointLamp {
        world: [1.6, 2.4, 1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    println!(
        "\n  {}\n  cena · a mexer ms · assente FRIO ms · assente GUARDADO ms [mín 3]",
        super::super::super::super::contexto()
    );
    for cena in [5u32, 28, 1, 11, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let mede = |assente: bool, frio: bool| {
            let mut m = f64::INFINITY;
            for i in 0..4 {
                let cam = ph2d_field_render::Orbit {
                    rotation: ph2d_field_render::Orbit::from_yaw_pitch(0.3 * i as f32, 0.5)
                        .rotation,
                    ..ph2d_field_render::Orbit::default()
                };
                if frio {
                    t.lock().expect("o traçador").esquece_as_sondas();
                }
                let t0 = std::time::Instant::now();
                let _ = crate::gpu_frame::paint(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &luz,
                    &surfaces,
                    &pres,
                    [0, 0, 0, 0],
                    None,
                    W,
                    H,
                    assente,
                );
                if i > 0 {
                    m = m.min(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
            m
        };
        println!(
            "  {cena:4} · {:>7.2} · {:>7.2} · {:>7.2}",
            mede(false, false),
            mede(true, true),
            mede(true, false)
        );
    }
}
