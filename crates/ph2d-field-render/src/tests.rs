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
///
/// ⚠️ **E compara as BORDAS também.** A segunda passagem é a que divide o trabalho por lotes, e é
/// nela que uma reordenação passaria despercebida — a máscara sairia igual e só as bordas mudariam
/// de sítio, que é o tipo de diferença que não se vê num quadro parado e cintila num que gira.
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
    assert!(!par.edges.is_empty(), "a esfera tem borda — senão não mede");
    let key = |g: &Gbuffer| -> Vec<(u32, [bool; 4], [[f32; 3]; 4])> {
        g.edges.iter().map(|e| (e.pixel, e.hit, e.normal)).collect()
    };
    assert_eq!(
        key(&par),
        key(&ser),
        "as bordas divergiram entre paralelo e serial — a segunda passagem perdeu a ordem"
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

/// Um matcap sintético 2×2: o suficiente para provar a costura, sem carregar asset.
const TOY_MATCAP: [f32; 12] = [0.1, 0.1, 0.1, 0.9, 0.2, 0.2, 0.2, 0.9, 0.9, 0.5, 0.5, 0.5];

fn toy(side: u32, texels: &[f32]) -> Matcap<'_> {
    Matcap {
        side,
        rgb_linear: texels,
    }
}

/// **Sem anti-serrilhado, a máscara é a lei**: fundo onde não há peça, cor onde há, e nada no meio.
///
/// ⚠️ Este gate ficou explicitamente **sem AA** quando o AA entrou, e não foi afrouxado por isso: o
/// que ele mede é a costura entre a máscara e o pintor, e essa relação continua a ser exata. Quem
/// mede o AA é o gate irmão.
#[test]
fn without_antialiasing_shading_is_exactly_the_mask() {
    let g = trace_with(&sphere(0.5), &Orbit::default(), 64, 64, true, false);
    let bg = [7u8, 8, 9, 255];
    let rgba = shade(&g, &toy(2, &TOY_MATCAP), bg);
    assert!(g.edges.is_empty(), "com `antialias = false` não há bordas");
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

/// ⭐ **O AA existe, e produz cobertura PARCIAL de verdade.**
///
/// Um gate que só verificasse "a imagem mudou" passaria com o AA a escrever qualquer coisa. O que
/// se afirma é o que define anti-serrilhado: existem pixels cuja cobertura **não é 0 nem 1**, eles
/// vivem na silhueta, e o alfa que sai deles é intermédio.
#[test]
fn antialiasing_produces_real_partial_coverage_on_the_silhouette() {
    let cam = Orbit::default();
    let g = trace(&sphere(0.5), &cam, 64, 64);
    assert!(
        !g.edges.is_empty(),
        "uma esfera contra o fundo TEM silhueta — sem bordas o detector não está a olhar"
    );

    let partial = g
        .edges
        .iter()
        .filter(|e| {
            let n = e.hit.iter().filter(|h| **h).count();
            n > 0 && n < 4
        })
        .count();
    assert!(
        partial > 20,
        "só {partial} pixels de cobertura parcial numa esfera de 64² — o padrão de amostragem não \
         está a cair dentro do pixel"
    );

    // E a cobertura vira ALFA intermédio na imagem.
    let rgba = shade(&g, &toy(2, &TOY_MATCAP), [0, 0, 0, 0]);
    let soft = rgba
        .chunks_exact(4)
        .filter(|px| px[3] > 0 && px[3] < 255)
        .count();
    assert!(
        soft > 20,
        "a cobertura parcial tem de chegar ao alfa: só {soft} pixels com alfa intermédio"
    );

    // ⚠️ E o interior continua OPACO — um AA que amolecesse o miolo estaria a medir ruído da
    // marcha, não geometria.
    let centre = (32 * 64 + 32) * 4;
    assert_eq!(
        rgba[centre + 3],
        255,
        "o centro da esfera é interior, e interior é opaco"
    );
}

/// ⭐ **O detector de borda olha a NORMAL, e não só a máscara.**
///
/// Uma aresta viva no meio da peça não muda a máscara: os dois lados acertam. Se o detector só
/// olhasse `hit`, a quina do cubo — a coisa que este módulo existe para entregar afiada — ficaria
/// serrilhada, e nenhum gate de silhueta acusaria.
#[test]
fn the_edge_detector_sees_a_sharp_crease_inside_the_mask() {
    let cube = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.45; 3],
                round: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("cubo");
    let g = trace(&cube, &Orbit::default(), 96, 96);

    // As bordas que estão INTEIRAMENTE dentro da máscara: as quatro amostras acertam, e os quatro
    // vizinhos do pixel também. Só uma quina produz isso.
    let (w, h) = (96usize, 96usize);
    let interior = g
        .edges
        .iter()
        .filter(|e| {
            let i = e.pixel as usize;
            let (x, y) = (i % w, i / w);
            if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
                return false;
            }
            e.hit.iter().all(|h| *h) && [i - 1, i + 1, i - w, i + w].iter().all(|&j| g.hit[j])
        })
        .count();
    assert!(
        interior > 30,
        "só {interior} pixels de borda no MIOLO do cubo — o detector está a olhar apenas a \
         máscara, e a quina viva fica serrilhada"
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
    println!("arestas | serial s/AA | paralelo s/AA | paralelo c/AA | bordas");
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
        for (parallel, aa) in [(false, false), (true, false), (true, true)] {
            let t0 = std::time::Instant::now();
            let g = trace_with(&doc, &Orbit::default(), 640, 480, parallel, aa);
            row.push_str(&format!(
                " {:9.1} ms |",
                t0.elapsed().as_secs_f64() * 1000.0
            ));
            if aa {
                row.push_str(&format!(" {}", g.edges.len()));
            }
        }
        println!("{row}");
    }
}

/// **Quanto custa o anti-serrilhado adaptativo, e sobre que fração de pixels ele corre.**
///
/// ⚠️ É este número que justifica a escolha de re-amostrar só as bordas em vez de supersamplear a
/// imagem inteira. Se a fração de borda se aproximasse de 1, a adaptação deixaria de valer a pena e
/// o certo passaria a ser 4× uniforme — e este é o instrumento que diria isso.
///
/// `#[ignore]`: medição.
#[test]
#[ignore]
fn measure_antialias_cost() {
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

    println!("     quadro |  sem AA |  com AA |  bordas | % da imagem");
    for (w, h) in [(640u32, 480u32), (1024, 1024), (1600, 1200)] {
        let t0 = std::time::Instant::now();
        let plain = trace_with(&doc, &Orbit::default(), w, h, true, false);
        let raw = t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = std::time::Instant::now();
        let aa = trace(&doc, &Orbit::default(), w, h);
        let full = t1.elapsed().as_secs_f64() * 1000.0;
        let px = (w as usize) * (h as usize);
        println!(
            "{w:5}x{h:<5} | {raw:6.1} ms | {full:6.1} ms | {:7} | {:5.2} %",
            aa.edges.len(),
            100.0 * aa.edges.len() as f64 / px as f64
        );
        assert_eq!(plain.hits(), aa.hits(), "o AA não muda a máscara base");
    }
}

/// **Despeja um quadro para OLHAR** — o diagnóstico deste módulo, porque aqui a imagem *é* o
/// produto e nenhum número substitui um par de olhos.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --ignored --nocapture dump_frame
/// ```
///
/// Escreve um PPM (P6) por cena em `PH2D_FIELD_DUMP` (por omissão, o diretório atual), composto
/// sobre cinza médio — é preciso um fundo para se ver o que a cobertura parcial faz na borda.
#[test]
#[ignore]
fn dump_frame() {
    let dir = std::env::var("PH2D_FIELD_DUMP").unwrap_or_else(|_| ".".into());
    let s = std::f32::consts::FRAC_1_SQRT_2;
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
    let doc = FieldDoc::new(
        vec![
            cyl([0.0, 0.0, 0.0, 1.0]),
            cyl([s, 0.0, 0.0, s]),
            cyl([0.0, s, 0.0, s]),
            ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Exact { radius: 0.12 }),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
            },
        ],
        NodeId(3),
    )
    .expect("junção");

    // Um matcap sintético contínuo: um degradê suave, que é onde a banda do vizinho-mais-próximo
    // aparece. Um asset da casa não pode ser dependência desta crate (ver o `Cargo.toml`).
    const SIDE: usize = 64;
    let mut texels = vec![0.0f32; SIDE * SIDE * 3];
    for y in 0..SIDE {
        for x in 0..SIDE {
            let (u, v) = (x as f32 / SIDE as f32, y as f32 / SIDE as f32);
            let t = (1.0 - (u - 0.35).hypot(v - 0.3) * 1.4).clamp(0.05, 1.0);
            let i = (y * SIDE + x) * 3;
            texels[i] = t * 0.95;
            texels[i + 1] = t * 0.72;
            texels[i + 2] = t * 0.62;
        }
    }

    for (name, aa) in [("sem-aa", false), ("com-aa", true)] {
        let (w, h) = (400u32, 400u32);
        let g = trace_with(&doc, &Orbit::default(), w, h, true, aa);
        let rgba = shade(&g, &toy(SIDE as u32, &texels), [0, 0, 0, 0]);
        // Composição sobre cinza médio: `dst = src + (1-a)*bg`, com `src` já pré-multiplicado.
        let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
        for px in rgba.chunks_exact(4) {
            let a = f32::from(px[3]) / 255.0;
            for c in &px[..3] {
                let v = f32::from(*c) + (1.0 - a) * 90.0;
                ppm.push(v.clamp(0.0, 255.0) as u8);
            }
        }
        let path = format!("{dir}/field-{name}.ppm");
        std::fs::write(&path, ppm).expect("escreve o despejo");
        println!("[dump] {path} — {} pixels de borda", g.edges.len());
    }
}
