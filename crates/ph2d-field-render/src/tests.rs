//! Os gates do traçador.

use super::*;
use ph2d_field::{NodeId, Primitive, Xform};

fn sphere(radius: f32) -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("esfera")
}

/// ⚠️ **A condição que autoriza o `rayon` aqui, MEDIDA e não afirmada** (ADR-0109).
///
/// Cada pixel é um gather puro contra uma árvore imutável e escreve só o próprio slot — então a
/// saída paralela tem de ser **byte-idêntica** à serial. Se algum dia alguém somar entre pixels, ou
/// partilhar um avaliador, este gate cai antes de o artefato aparecer numa imagem.
#[test]
fn the_threaded_trace_is_byte_identical_to_the_serial_one() {
    let doc = sphere(0.6);
    let cam = Orbit::default();
    let par = trace_with_threads(&doc, &cam, 96, 72, true);
    let ser = trace_with_threads(&doc, &cam, 96, 72, false);
    assert_eq!(
        par.hit, ser.hit,
        "a máscara divergiu entre paralelo e serial"
    );
    assert_eq!(
        par.normal, ser.normal,
        "as normais divergiram entre paralelo e serial"
    );
}

/// A esfera aparece com o **tamanho certo**: uma esfera de raio `r` vista de qualquer ângulo é um
/// disco de raio `r`, e a fração da tela que ela cobre é conta fechada.
///
/// É o gate que pega câmera errada, escala trocada e projeção invertida de uma vez.
#[test]
fn a_sphere_covers_exactly_the_area_geometry_predicts() {
    let radius = 0.5_f32;
    let cam = Orbit {
        half_extent: 1.0,
        ..Orbit::default()
    };
    let (w, h) = (200u32, 200u32);
    let g = trace(&sphere(radius), &cam, w, h);

    // Meia altura de tela = `half_extent` unidades ⇒ o disco tem raio `radius/half_extent` em
    // frações de meia-tela, e ocupa `π r² / 4` do quadro (o quadro tem lado 2 em meia-telas).
    let frac = f64::from(radius) / f64::from(cam.half_extent);
    let expected = std::f64::consts::PI * frac * frac / 4.0;
    let got = g.hits() as f64 / f64::from(w * h);
    assert!(
        (got - expected).abs() < 0.01,
        "a esfera devia cobrir {expected:.4} do quadro e cobriu {got:.4}"
    );
}

/// ⚠️ **O gate do passo seguro.** A W0 mediu ‖∇f‖ = √2 no operador exato; um passo de `d` faria o
/// raio atravessar a superfície e o furo apareceria como fundo **no meio da peça**.
///
/// Aqui isso vira teste: o disco da esfera não pode ter um único pixel de fundo dentro dele.
#[test]
fn the_march_never_steps_through_the_surface() {
    let g = trace(&sphere(0.6), &Orbit::default(), 128, 128);
    let (w, h) = (g.width as usize, g.height as usize);
    let mut holes = 0usize;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            if g.hit[i] {
                continue;
            }
            // Um vazio cercado de superfície nos quatro lados é um furo, não silhueta.
            if g.hit[i - 1] && g.hit[i + 1] && g.hit[i - w] && g.hit[i + w] {
                holes += 1;
            }
        }
    }
    assert_eq!(holes, 0, "{holes} pixel(s) atravessaram a superfície");
}

/// Orbitar **não muda a forma**: a mesma esfera, de dois ângulos, cobre a mesma área.
/// É o gate que pega uma base de câmera não-ortonormal — que deformaria a peça ao girar, e é o tipo
/// de erro que se vê como *"a peça respira quando eu orbito"* e não como erro de compilação.
#[test]
fn orbiting_does_not_change_the_shape() {
    let doc = sphere(0.5);
    let a = trace(
        &doc,
        &Orbit {
            yaw: 0.0,
            pitch: 0.0,
            ..Orbit::default()
        },
        160,
        160,
    );
    let b = trace(
        &doc,
        &Orbit {
            yaw: 1.1,
            pitch: -0.7,
            ..Orbit::default()
        },
        160,
        160,
    );
    let (ha, hb) = (a.hits() as f64, b.hits() as f64);
    assert!(
        (ha - hb).abs() / ha < 0.01,
        "a esfera mudou de tamanho ao orbitar: {ha} contra {hb}"
    );
}

/// A normal aponta para o observador no centro do disco, e é unitária em toda parte.
/// Sem isto, um matcap amostraria fora do disco e a peça sairia com a cor errada — sem erro nenhum.
#[test]
fn the_view_space_normal_faces_the_camera_and_is_unit_length() {
    let g = trace(&sphere(0.6), &Orbit::default(), 101, 101);
    let centre = (g.height as usize / 2) * g.width as usize + g.width as usize / 2;
    assert!(g.hit[centre], "o centro do quadro tem de estar na esfera");
    let n = g.normal[centre];
    assert!(
        n[2] > 0.99,
        "no centro a normal aponta para o observador: {n:?}"
    );
    for (i, n) in g.normal.iter().enumerate() {
        if !g.hit[i] {
            continue;
        }
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-3, "normal não unitária: {len}");
    }
}

/// O sombreamento respeita a máscara: fundo onde não há peça, e cor onde há.
#[test]
fn shading_paints_the_background_only_where_there_is_no_surface() {
    let g = trace(&sphere(0.5), &Orbit::default(), 64, 64);
    // Um matcap sintético 2×2: o suficiente para provar a costura, sem carregar asset.
    let texels = [0.1, 0.1, 0.1, 0.9, 0.2, 0.2, 0.2, 0.9, 0.9, 0.5, 0.5, 0.5];
    let bg = [7u8, 8, 9, 255];
    let rgba = shade(
        &g,
        &Matcap {
            side: 2,
            rgb_linear: &texels,
        },
        bg,
    );
    assert_eq!(rgba.len(), 64 * 64 * 4);
    let painted = rgba
        .chunks_exact(4)
        .enumerate()
        .filter(|(i, px)| g.hit[*i] && *px != bg)
        .count();
    assert_eq!(
        painted,
        g.hits(),
        "todo pixel com superfície tem de ser pintado"
    );
    let background = rgba
        .chunks_exact(4)
        .enumerate()
        .filter(|(i, px)| !g.hit[*i] && *px == bg)
        .count();
    assert_eq!(
        background,
        64 * 64 - g.hits(),
        "todo pixel sem superfície tem de ficar com o fundo"
    );
}

/// **A sonda de custo** — `cargo test -p ph2d-field-render -- --ignored --nocapture`.
///
/// ⚠️ `#[ignore]` porque mede relógio, e relógio sob carga não vale nada (a workstation já deu
/// 11,36 e 5,50 ms para o mesmo passe). Ela existe para responder *"cabe num quadro?"* com número,
/// e para que a resposta seja re-medida em vez de lembrada.
#[test]
#[ignore]
fn measure_trace_cost() {
    // A peça da W0: três cilindros com filete interno e aros externos.
    let r = 0.12_f32;
    let cyl = |axis: [f32; 4]| {
        ph2d_field_eval::leaf(
            Primitive::Cylinder {
                radius: 0.22,
                half_height: 0.78,
                round: 0.05,
            },
            Xform {
                rotation: axis,
                ..Xform::IDENTITY
            },
        )
    };
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let doc = FieldDoc::new(
        vec![
            cyl([0.0, 0.0, 0.0, 1.0]),
            cyl([s, 0.0, 0.0, s]),
            cyl([0.0, s, 0.0, s]),
            ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Exact { radius: r }),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
            },
        ],
        NodeId(3),
    )
    .expect("junção");

    for (w, h) in [(640u32, 480u32), (1280, 720), (1920, 1080)] {
        for parallel in [false, true] {
            let t0 = std::time::Instant::now();
            let g = trace_with_threads(&doc, &Orbit::default(), w, h, parallel);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            println!(
                "{w}x{h} {:9} {dt:7.1} ms  ({} pixels de peça)",
                if parallel { "paralelo" } else { "serial" },
                g.hits()
            );
        }
    }
}

/// **Quanto custa traçar um PERFIL** — em função do número de arestas (W3).
///
/// ⚠️ É este número que escolhe a tolerância de cozimento, e não o contrário: a tolerância decide
/// quantas arestas o perfil tem, e cada aresta é ~26 nós avaliados **por pixel**. Um perfil é a
/// única primitiva do módulo cujo custo o autor controla sem saber.
///
/// `#[ignore]`: medição.
#[test]
#[ignore]
fn measure_profile_trace_cost() {
    println!("arestas |   serial | paralelo | pixels");
    for n in [8_usize, 16, 32, 64, 128, 256, 512] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = ph2d_field::Profile::new(vec![contour], ph2d_field::FillRule::NonZero, 1e-3)
            .expect("perfil");
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");

        let mut row = format!("{n:7} |");
        let mut hits = 0;
        for parallel in [false, true] {
            let t0 = std::time::Instant::now();
            let g = trace_with_threads(&doc, &Orbit::default(), 640, 480, parallel);
            hits = g.hits();
            row.push_str(&format!(
                " {:6.1} ms |",
                t0.elapsed().as_secs_f64() * 1000.0
            ));
        }
        println!("{row} {hits}");
    }
}
