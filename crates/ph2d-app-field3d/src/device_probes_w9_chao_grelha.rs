//! 📏 **A GRELHA DA LUZ DEVOLVIDA AO CHÃO, contra uma referência FINA** — o report *«ainda com
//! áreas retangulares ruins»* (2026-09-24, foto da cena `=28` com a lâmpada encostada ao nó).
//!
//! A foto da sonda irmã ([`super::diag_a_luz_encostada_com_chao`]) separa as duas metades: com a cor
//! devolvida ao chão ligada o chão tinha LOSANGOS do tamanho de uma célula; sem ela, não. A pergunta
//! destas sondas foi *de que é feito o erro* — e a resposta (`docs/Render3d/09` §10) é que NÃO era
//! resolução: a grelha lia um PONTO onde a verdade tem riscas mais finas que a célula.

use ph2d_field_render::ground_bounce::{GROUND_BOUNCE_DIRS, bake_ground_bounce};

/// 📏 **Sonda: o erro da grelha por resolução** — tudo na CPU, sem placa. A tabela da lei ANTIGA
/// (nó pontual + bilinear) está no `docs/Render3d/09` §10: ela não caía com a resolução.
#[test]
#[ignore = "sonda: assa campos de ate 192^2"]
fn diag_a_grelha_da_luz_do_chao() {
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height })
        .expect("o chão");
    let (c, r) = (bola.center, bola.radius);
    let luz = [ph2d_field_render::PointLamp {
        world: super::luz_no_vazio_do_no(&doc, c, r),
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let assa = |n: usize| {
        let t0 = std::time::Instant::now();
        let g = bake_ground_bounce(
            &doc,
            &reg,
            &cam,
            chao,
            &surfaces,
            &luz,
            n,
            GROUND_BOUNCE_DIRS,
            720,
        );
        (g, t0.elapsed().as_secs_f64() * 1e3)
    };
    let (referencia, t_ref) = assa(192);
    // O miolo que a câmera vê: `±3 r` à volta do centro, onde a orla vale 1.
    const M: usize = 241;
    let pontos: Vec<[f32; 3]> = (0..M * M)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let (i, j) = ((k % M) as f32, (k / M) as f32);
            #[allow(clippy::cast_precision_loss)]
            let s = 6.0 * r / (M - 1) as f32;
            [c[0] - 3.0 * r + i * s, chao.height, c[2] - 3.0 * r + j * s]
        })
        .collect();
    let lum = |v: [f32; 3]| (v[0] + v[1] + v[2]) / 3.0;
    let verdade: Vec<f32> = pontos.iter().map(|p| lum(referencia.sample(*p))).collect();
    let pico = verdade.iter().copied().fold(0.0f32, f32::max);
    println!(
        "\n  referência 192² ({t_ref:.1} ms) · pico {pico:.4}\n  grelha · pior/pico · p99/pico · média/pico · assa ms"
    );
    for n in [32usize, 48, 64, 96, 128] {
        let (g, ms) = assa(n);
        let mut d: Vec<f32> = pontos
            .iter()
            .zip(&verdade)
            .map(|(p, v)| (lum(g.sample(*p)) - v).abs())
            .collect();
        d.sort_by(f32::total_cmp);
        #[allow(clippy::cast_precision_loss)]
        let media = d.iter().sum::<f32>() / d.len() as f32;
        let p99 = d[d.len() * 99 / 100];
        println!(
            "  {n:>4}² · {:>7.3} % · {:>7.3} % · {:>7.3} % · {ms:>7.1}",
            100.0 * d[d.len() - 1] / pico,
            100.0 * p99 / pico,
            100.0 * media / pico
        );
    }
}

/// 📏 **Sonda: a mesma grelha com mais DIRECÇÕES** — separa o ruído do estimador (direcções) do
/// erro de resolução (células). Compara nó a nó contra a mesma grelha assada com `4 096`.
#[test]
#[ignore = "sonda: assa com ate 4096 direccoes"]
fn diag_as_direccoes_da_luz_do_chao() {
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height })
        .expect("o chão");
    let (c, r) = (bola.center, bola.radius);
    for (nome, luz_em) in [
        ("encostada", super::luz_no_vazio_do_no(&doc, c, r)),
        (
            "longe    ",
            [c[0] + 2.0 * r, c[1] + 2.0 * r, c[2] + 2.0 * r],
        ),
    ] {
        let luz = [ph2d_field_render::PointLamp {
            world: luz_em,
            radiance_at_one: [7.0, 7.0, 7.0],
        }];
        let assa = |dirs: u32| {
            let t0 = std::time::Instant::now();
            let g = bake_ground_bounce(&doc, &reg, &cam, chao, &surfaces, &luz, 32, dirs, 720);
            (g, t0.elapsed().as_secs_f64() * 1e3)
        };
        let (verdade, _) = assa(4096);
        let lum = |v: [f32; 3]| (v[0] + v[1] + v[2]) / 3.0;
        let pico = verdade.value.iter().map(|v| lum(*v)).fold(0.0f32, f32::max);
        println!("\n  luz {nome} · pico {pico:.4}\n  direcções · pior/pico · p99/pico · assa ms");
        for dirs in [128u32, 512, 1024] {
            let (g, ms) = assa(dirs);
            let mut d: Vec<f32> = g
                .value
                .iter()
                .zip(&verdade.value)
                .map(|(a, b)| (lum(*a) - lum(*b)).abs())
                .collect();
            d.sort_by(f32::total_cmp);
            println!(
                "  {dirs:>9} · {:>7.3} % · {:>7.3} % · {ms:>7.1}",
                100.0 * d[d.len() - 1] / pico,
                100.0 * d[d.len() * 99 / 100] / pico
            );
        }
    }
}

/// 📷 **Sonda: o campo do chão VISTO DE CIMA** — a referência fina e a `32²` do produto, na mesma
/// escala (`√(v/pico)`), gravadas em `PH2D_SONDA_DIR`.
#[test]
#[ignore = "sonda que grava imagens"]
fn diag_o_campo_do_chao_visto_de_cima() {
    let Ok(dir) = std::env::var("PH2D_SONDA_DIR") else {
        return;
    };
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height })
        .expect("o chão");
    let (c, r) = (bola.center, bola.radius);
    let luz = [ph2d_field_render::PointLamp {
        world: super::luz_no_vazio_do_no(&doc, c, r),
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let n_ref: usize = std::env::var("PH2D_CHAO_REF")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(192);
    let assa = |n: usize| {
        bake_ground_bounce(
            &doc,
            &reg,
            &cam,
            chao,
            &surfaces,
            &luz,
            n,
            GROUND_BOUNCE_DIRS,
            720,
        )
    };
    let (fina, grossa) = (assa(n_ref), assa(32));
    const M: usize = 480;
    let ponto = |k: usize| {
        #[allow(clippy::cast_precision_loss)]
        let (i, j) = ((k % M) as f32, (k / M) as f32);
        #[allow(clippy::cast_precision_loss)]
        let s = 6.0 * r / (M - 1) as f32;
        [c[0] - 3.0 * r + i * s, chao.height, c[2] - 3.0 * r + j * s]
    };
    let lum = |v: [f32; 3]| (v[0] + v[1] + v[2]) / 3.0;
    let pico = (0..M * M)
        .map(|k| lum(fina.sample(ponto(k))))
        .fold(0.0f32, f32::max);
    let grava = |nome: &str, f: &dyn Fn([f32; 3]) -> f32| {
        let mut ppm = format!("P5\n{M} {M}\n255\n").into_bytes();
        for k in 0..M * M {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            ppm.push(((f(ponto(k)) / pico).clamp(0.0, 1.0).sqrt() * 255.0) as u8);
        }
        std::fs::write(format!("{dir}/chao_{nome}.pgm"), ppm).expect("grava");
    };
    grava("fina", &|p| lum(fina.sample(p)));
    grava("produto32", &|p| lum(grossa.sample(p)));
    println!("pico {pico:.4} · luz {:?}", luz[0].world);
}
