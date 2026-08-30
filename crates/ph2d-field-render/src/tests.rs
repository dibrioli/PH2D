//! Os gates do traçador.

use super::*;
use ph2d_field::{NodeId, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;

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
    let par = trace_with_threads(&doc, &Registry::new(), &cam, 96, 72, true);
    let ser = trace_with_threads(&doc, &Registry::new(), &cam, 96, 72, false);
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
    let g = trace(&sphere(radius), &Registry::new(), &cam, w, h);

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
    let g = trace(&sphere(0.6), &Registry::new(), &Orbit::default(), 128, 128);
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
        &Registry::new(),
        &Orbit::from_yaw_pitch(0.0, 0.0),
        160,
        160,
    );
    let b = trace(
        &doc,
        &Registry::new(),
        &Orbit::from_yaw_pitch(1.1, -0.7),
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
    let g = trace(&sphere(0.6), &Registry::new(), &Orbit::default(), 101, 101);
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
    let g = trace_with(
        &sphere(0.5),
        &Registry::new(),
        &Orbit::default(),
        64,
        64,
        true,
        false,
    );
    let bg = [7u8, 8, 9, 255];
    let rgba = shade(&g, &toy(2, &TOY_MATCAP), bg);
    assert!(g.edges.is_empty(), "com `antialias = false` não há bordas");
    assert_eq!(rgba.len(), 64 * 64 * 4);
    let painted = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(i, px)| g.hit[*i] && **px != bg)
        .count();
    assert_eq!(
        painted,
        g.hits(),
        "todo pixel com superfície tem de ser pintado"
    );
    let background = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(i, px)| !g.hit[*i] && **px == bg)
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
    let g = trace(&sphere(0.5), &Registry::new(), &cam, 64, 64);
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
        .as_chunks::<4>()
        .0
        .iter()
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
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("cubo");
    let g = trace(&cube, &Registry::new(), &Orbit::default(), 96, 96);

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
                chamfer: 0.0,
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
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(3),
    )
    .expect("junção");

    for (w, h) in [(640u32, 480u32), (1280, 720), (1920, 1080)] {
        for parallel in [false, true] {
            let t0 = std::time::Instant::now();
            let g = trace_with_threads(&doc, &Registry::new(), &Orbit::default(), w, h, parallel);
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
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");

        let mut row = format!("{n:7} |");
        for (parallel, aa) in [(false, false), (true, false), (true, true)] {
            let t0 = std::time::Instant::now();
            let g = trace_with(
                &doc,
                &Registry::new(),
                &Orbit::default(),
                640,
                480,
                parallel,
                aa,
            );
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
                chamfer: 0.0,
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
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(3),
    )
    .expect("junção");

    println!("     quadro |  sem AA |  com AA |  bordas | % da imagem");
    for (w, h) in [(640u32, 480u32), (1024, 1024), (1600, 1200)] {
        let t0 = std::time::Instant::now();
        let plain = trace_with(&doc, &Registry::new(), &Orbit::default(), w, h, true, false);
        let raw = t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = std::time::Instant::now();
        let aa = trace(&doc, &Registry::new(), &Orbit::default(), w, h);
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
                chamfer: 0.0,
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
                mods: Vec::new(),
                verb: None,
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
        let g = trace_with(&doc, &Registry::new(), &Orbit::default(), w, h, true, aa);
        let rgba = shade(&g, &toy(SIDE as u32, &texels), [0, 0, 0, 0]);
        // Composição sobre cinza médio: `dst = src + (1-a)*bg`, com `src` já pré-multiplicado.
        let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
        for px in rgba.as_chunks::<4>().0.iter() {
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

/// ⭐ **Aproximar mostra MAIS forma, e não uma forma inchada.**
///
/// Um campo implícito tem detalhe infinito, e a tolerância de acerto é o que decidia se ele
/// aparecia. Com um `HIT_EPS` fixo em 2·10⁻⁴, uma peça de raio 10⁻³ vista de perto sairia com raio
/// **1,2·10⁻³** — 20 % maior, e a área na tela **44 %** maior. Não é franja: é a peça errada.
///
/// A cura não foi um limite de zoom, foi tirar a causa: as tolerâncias descem com o pixel
/// ([`Sharpness`]). Este gate é o que impede alguém de as voltar a fixar "para simplificar".
#[test]
fn zooming_in_does_not_inflate_the_part() {
    let radius = 1.0e-3_f32;
    let cam = Orbit {
        half_extent: 2.0e-3,
        ..Orbit::default()
    };
    let (w, h) = (200u32, 200u32);
    let g = trace(&sphere(radius), &Registry::new(), &cam, w, h);

    let frac = f64::from(radius) / f64::from(cam.half_extent);
    let expected = std::f64::consts::PI * frac * frac / 4.0;
    let got = g.hits() as f64 / f64::from(w * h);
    assert!(
        (got - expected).abs() < 0.01,
        "de muito perto a esfera devia cobrir {expected:.4} do quadro e cobriu {got:.4} — com \
         tolerância FIXA isto dá 0,283 contra 0,196"
    );
}

/// ⚠️ **A câmera nova reproduz a antiga, exatamente** — `from_yaw_pitch` é a mesma base que os dois
/// ângulos davam.
///
/// A troca para quaternion foi feita para **remover os polos**, não para mudar o enquadramento. Sem
/// este gate, um erro de sinal na conversão mudaria toda cena de smoke e toda imagem de referência
/// de uma vez, e o sintoma seria "as imagens da W0 já não batem" — longe da causa.
#[test]
fn the_quaternion_camera_reproduces_the_two_angle_basis() {
    for (yaw, pitch) in [
        (0.0_f32, 0.0_f32),
        (0.72, 0.52),
        (-1.3, 0.9),
        (2.5, -1.1),
        (0.4, 1.5),
    ] {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        // A fórmula ANTIGA, escrita à mão aqui: um gate que pedisse a base ao código sob teste
        // estaria a compará-lo consigo próprio.
        let want_fwd = [cp * sy, sp, cp * cy];
        let want_right = [cy, 0.0, -sy];
        let want_up = [-sp * sy, cp, -sp * cy];

        let (right, up, fwd) = Orbit::from_yaw_pitch(yaw, pitch).basis();
        for k in 0..3 {
            assert!(
                (right[k] - want_right[k]).abs() < 1e-5
                    && (up[k] - want_up[k]).abs() < 1e-5
                    && (fwd[k] - want_fwd[k]).abs() < 1e-5,
                "yaw={yaw} pitch={pitch}: base divergiu no eixo {k}\n  right {right:?} != {want_right:?}\
                 \n  up    {up:?} != {want_up:?}\n  fwd   {fwd:?} != {want_fwd:?}"
            );
        }
    }
}

/// ⭐ **A rotação livre NÃO TEM POLO** — e este é o gate que existe por causa do relato
/// *"só rotaciona em uma direção"* (Enio, 2026-08-19).
///
/// Numa câmera de dois ângulos, girar na vertical satura em ±90°: a peça para de se mexer e o gesto
/// morre contra uma parede. Aqui a afirmação é **exata**, não estatística: cada passo é uma rotação
/// de `STEP` em torno de um eixo local fixo, logo a direção da vista tem de virar **exatamente**
/// `STEP` — a cada um dos mil passos, que são quase três voltas completas.
#[test]
fn free_rotation_never_hits_a_pole() {
    const STEP: f32 = 0.02;
    let mut cam = Orbit::default();
    let (_, _, mut prev) = cam.basis();
    let mut worst = f32::INFINITY;
    for _ in 0..1000 {
        // Um arrasto puramente VERTICAL — é onde a câmera de dois ângulos morre.
        cam.turn_local([-1.0, 0.0, 0.0], STEP);
        let (_, _, fwd) = cam.basis();
        let dot = (0..3)
            .map(|k| prev[k] * fwd[k])
            .sum::<f32>()
            .clamp(-1.0, 1.0);
        worst = worst.min(dot.acos());
        prev = fwd;
    }
    assert!(
        worst > STEP * 0.99,
        "algum passo virou só {worst} rad em vez de {STEP} — isso é um POLO, e é exatamente o que \
         a câmera de dois ângulos fazia"
    );

    // E a orientação continua a ser uma ROTAÇÃO: um quaternion que perdesse a norma passaria a
    // escalar a peça, e o sintoma seria a forma a encolher devagar enquanto se gira.
    let q = cam.rotation;
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    assert!(
        (n - 1.0).abs() < 1e-4,
        "depois de mil giros a norma do quaternion é {n}: a peça está a ser escalada, não girada"
    );
    let doc = sphere(0.55);
    let before = trace(&doc, &Registry::new(), &Orbit::default(), 120, 120).hits() as f64;
    let after = trace(&doc, &Registry::new(), &cam, 120, 120).hits() as f64;
    assert!(
        (before - after).abs() / before < 0.02,
        "girar não pode mudar o TAMANHO da peça: {before} -> {after} pixels"
    );
}

/// ⚠️ **A inversa do mapeamento pixel↔plano é escrita à mão, e é onde um sinal trocado sobrevive
/// anos** — ela só é exercida pelo gizmo, cujo sintoma (a alça meio pixel ao lado) ninguém chama de
/// bug de projeção.
#[test]
fn a_pixel_survives_the_round_trip() {
    let screen = Screen::new(800, 480, 0.8);
    for (x, y) in [(0.0, 0.0), (400.0, 240.0), (799.0, 479.0), (123.0, 45.0)] {
        let (u, v) = screen.plane_at(x, y);
        let (bx, by) = screen.pixel_of(u, v);
        assert!(
            (bx - x).abs() < 1e-3 && (by - y).abs() < 1e-3,
            "({x}, {y}) voltou como ({bx}, {by})"
        );
    }
}

/// ⭐ **O gizmo e o traçador concordam sobre onde um ponto do mundo cai.**
///
/// ⚠️ É o gate que junta as duas metades que **têm** de ser a mesma conta. O gizmo projeta as alças
/// com [`Orbit::project`]; a marcha constrói os raios com [`Screen::plane_at`]. Duas cópias
/// divergiriam sem nada ficar vermelho, e o sintoma seria uma seta que agarra ao lado da superfície
/// que ela diz mover.
///
/// A afirmação é forte de propósito: **traça de verdade** e mede o centroide dos pixels de peça.
/// Sob projeção ortográfica o centroide da silhueta de uma esfera é a projeção exacta do centro
/// dela, então o alvo não é uma aproximação — é o número.
#[test]
fn a_point_projects_where_the_march_actually_hits_it() {
    let center = [0.3f32, -0.2, 0.15];
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.12 },
            Xform::at(center[0], center[1], center[2]),
        )],
        NodeId(0),
    )
    .expect("esfera deslocada");

    let (w, h) = (256u32, 200u32);
    let cam = Orbit::default();
    let g = trace(&doc, &Registry::new(), &cam, w, h);
    assert!(g.hits() > 100, "a esfera não apareceu no quadro");

    let (mut sx, mut sy, mut n) = (0.0f64, 0.0f64, 0.0f64);
    for i in 0..(w as usize * h as usize) {
        if g.hit[i] {
            sx += (i % w as usize) as f64 + 0.5;
            sy += (i / w as usize) as f64 + 0.5;
            n += 1.0;
        }
    }
    let (cx, cy) = (sx / n, sy / n);
    let ([px, py], depth) = cam
        .project(center, Screen::new(w, h, cam.half_extent))
        .expect("o centro da peça está à frente do olho");

    assert!(
        (f64::from(px) - cx).abs() < 1.0 && (f64::from(py) - cy).abs() < 1.0,
        "a projeção diz ({px}, {py}) e a marcha pôs a peça em ({cx}, {cy})"
    );
    // E a profundidade cresce na direção do observador: o mesmo ponto empurrado para trás fica
    // com profundidade menor.
    let (_, back) = cam
        .project(
            {
                let (_, _, fwd) = cam.basis();
                [center[0] - fwd[0], center[1] - fwd[1], center[2] - fwd[2]]
            },
            Screen::new(w, h, cam.half_extent),
        )
        .expect("um passo para trás continua à frente do olho");
    assert!(
        back < depth,
        "a profundidade tem de crescer para o observador"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// A LENTE — o que o olho faz com o que está longe.
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐ **As duas lentes COINCIDEM no plano do alvo** — é o que faz a lente ser só uma lente.
///
/// ⚠️ É a propriedade que mantém o resto do módulo intacto: zoom, enquadramento, o passo da grelha e
/// a lei do pan continuam a falar do mesmo `half_extent`, porque ele quer dizer a mesma coisa nas
/// duas. Uma perspectiva que mudasse o significado dele obrigaria a reconferir cada número que dele
/// deriva — e nenhum deles ficaria vermelho ao mudar.
#[test]
fn the_two_lenses_agree_exactly_on_the_target_plane() {
    let ortho = Orbit {
        lens: Lens::Ortho,
        ..Orbit::default()
    };
    let persp = Orbit::default();
    let s = Screen::new(400, 300, ortho.half_extent);
    let (right, up, _) = ortho.basis();

    for (a, b) in [(0.0f32, 0.0f32), (0.4, -0.2), (-0.7, 0.7)] {
        let p = [
            ortho.target[0] + right[0] * a + up[0] * b,
            ortho.target[1] + right[1] * a + up[1] * b,
            ortho.target[2] + right[2] * a + up[2] * b,
        ];
        let (o, _) = ortho.project(p, s).expect("paralela nunca recusa");
        let (q, _) = persp.project(p, s).expect("está no plano do alvo");
        assert!(
            (o[0] - q[0]).abs() < 1e-3 && (o[1] - q[1]).abs() < 1e-3,
            "no plano do alvo as duas lentes têm de dar o mesmo pixel: {o:?} vs {q:?}"
        );
    }
}

/// ⭐ **O que está mais longe do olho aparece MENOR** — e na paralela não aparece.
///
/// ⚠️ O gate mede as **duas** lentes com a mesma medida, porque é a diferença entre elas que se está
/// a afirmar: um gate só sobre a convergente passaria com a paralela também convergente.
#[test]
fn distance_shrinks_under_the_converging_lens_and_not_under_the_parallel_one() {
    let persp = Orbit::default();
    let mut ortho = persp;
    ortho.lens = Lens::Ortho;
    let s = Screen::new(400, 300, persp.half_extent);
    let (right, _, fwd) = persp.basis();
    // Duas hastes do mesmo comprimento, uma perto e outra longe.
    let bar = |depth: f32| -> [[f32; 3]; 2] {
        let c = [
            persp.target[0] + fwd[0] * depth,
            persp.target[1] + fwd[1] * depth,
            persp.target[2] + fwd[2] * depth,
        ];
        [
            [
                c[0] - right[0] * 0.2,
                c[1] - right[1] * 0.2,
                c[2] - right[2] * 0.2,
            ],
            [
                c[0] + right[0] * 0.2,
                c[1] + right[1] * 0.2,
                c[2] + right[2] * 0.2,
            ],
        ]
    };
    let width = |cam: &Orbit, depth: f32| -> f32 {
        let [a, b] = bar(depth);
        let (pa, _) = cam.project(a, s).expect("à frente");
        let (pb, _) = cam.project(b, s).expect("à frente");
        (pb[0] - pa[0]).abs()
    };
    let (near, far) = (width(&persp, 0.6), width(&persp, -0.6));
    assert!(
        near > far * 1.2,
        "a haste perto tinha de medir bem mais na tela: {near:.1} contra {far:.1}"
    );
    let (o_near, o_far) = (width(&ortho, 0.6), width(&ortho, -0.6));
    assert!(
        (o_near - o_far).abs() < 1e-3,
        "na paralela a distância não pode mudar o tamanho: {o_near:.1} contra {o_far:.1}"
    );
}

/// ⭐ **Um ponto ao lado do olho, ou atrás dele, NÃO tem projeção.**
///
/// ⚠️ E a paralela nunca recusa — não há divisão nenhuma nela. O gate mede as duas metades: sem a
/// segunda, um `project` que devolvesse `None` sempre passaria na primeira.
#[test]
fn a_point_at_or_behind_the_eye_has_no_projection() {
    let persp = Orbit::default();
    let mut ortho = persp;
    ortho.lens = Lens::Ortho;
    let s = Screen::new(400, 300, persp.half_extent);
    let dist = persp.eye_distance().expect("a convergente tem olho");
    let (_, _, fwd) = persp.basis();
    for beyond in [1.0f32, 1.5, 4.0] {
        let p = [
            persp.target[0] + fwd[0] * dist * beyond,
            persp.target[1] + fwd[1] * dist * beyond,
            persp.target[2] + fwd[2] * dist * beyond,
        ];
        assert!(
            persp.project(p, s).is_none(),
            "um ponto a {beyond}× a distância do olho não tem pixel"
        );
        assert!(
            ortho.project(p, s).is_some(),
            "a paralela não tem por que recusar: não há divisão"
        );
    }
}

/// ⭐ **O raio da MARCHA é o mesmo raio que a projeção promete** — e agora nas duas lentes.
///
/// ⚠️ Este é o gate que a consolidação exigiu: a marcha reconstruía a aritmética do `Orbit::ray` com
/// um afastamento próprio, e a segunda cópia teria ficado **paralela** quando a convergente entrou —
/// a peça traçada de uma forma e as alças noutra, sem nada vermelho. Aqui manda-se um ponto pela
/// projeção e pergunta-se ao raio daquele pixel se ele passa por lá.
#[test]
fn the_ray_of_a_pixel_passes_through_what_projects_onto_it() {
    for lens in [
        Lens::Ortho,
        Lens::Perspective {
            half_fov: DEFAULT_HALF_FOV,
        },
    ] {
        let cam = Orbit {
            lens,
            ..Orbit::default()
        };
        let s = Screen::new(400, 300, cam.half_extent);
        let (right, up, fwd) = cam.basis();
        for (a, b, c) in [
            (0.0f32, 0.0f32, 0.0f32),
            (0.3, -0.2, 0.25),
            (-0.5, 0.4, -0.3),
        ] {
            let p = [
                cam.target[0] + right[0] * a + up[0] * b + fwd[0] * c,
                cam.target[1] + right[1] * a + up[1] * b + fwd[1] * c,
                cam.target[2] + right[2] * a + up[2] * b + fwd[2] * c,
            ];
            let (px, _) = cam.project(p, s).expect("à frente do olho");
            let (o, d) = cam.ray(px[0], px[1], s);
            // A distância do ponto à reta do raio: `‖(p−o) − ((p−o)·d)d‖`.
            let v = [p[0] - o[0], p[1] - o[1], p[2] - o[2]];
            let t = v[0] * d[0] + v[1] * d[1] + v[2] * d[2];
            let perp = [v[0] - d[0] * t, v[1] - d[1] * t, v[2] - d[2] * t];
            let miss = (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt();
            assert!(
                miss < 1.0e-4,
                "{lens:?}: o raio do pixel {px:?} passa a {miss:e} do ponto que lá projeta"
            );
        }
    }
}

/// ⭐ **Uma alça de gizmo mede o mesmo na TELA a qualquer distância** — nas duas lentes.
///
/// ⚠️ É a razão de [`Orbit::px_per_world_at`] existir: com a lente convergente, um braço dimensionado
/// pela constante do quadro encolheria conforme a peça se afasta, e `MIN_ARM_PX` — que decide se a
/// alça é sequer oferecida — passaria a morder por distância em vez de por ângulo.
#[test]
fn a_screen_sized_arm_measures_the_same_at_any_distance() {
    let cam = Orbit::default();
    let s = Screen::new(400, 300, cam.half_extent);
    let (right, _, fwd) = cam.basis();
    const ARM_PX: f32 = 90.0;
    for depth in [0.6f32, 0.0, -0.6, -1.2] {
        let origin = [
            cam.target[0] + fwd[0] * depth,
            cam.target[1] + fwd[1] * depth,
            cam.target[2] + fwd[2] * depth,
        ];
        let arm = ARM_PX / cam.px_per_world_at(origin, s);
        let tip = [
            origin[0] + right[0] * arm,
            origin[1] + right[1] * arm,
            origin[2] + right[2] * arm,
        ];
        let (a, _) = cam.project(origin, s).expect("à frente");
        let (b, _) = cam.project(tip, s).expect("à frente");
        let on_screen = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        assert!(
            (on_screen - ARM_PX).abs() < 1.0,
            "a {depth} de profundidade o braço mediu {on_screen:.1} px e devia medir {ARM_PX}"
        );
    }
}

/// ⚠️ **O custo REAL da inclinação** — o relógio de um quadro, e não um proxy.
///
/// A sonda irmã (`ph2d_field_eval::measure_taper_cost`) mede `min ‖∇f‖`: quão curto é o **pior**
/// passo. Ela não diz quantos pixels pagam esse pior passo. Esta mede a coisa que o artista sente,
/// e é ela que escolhe o [`ph2d_field::mods::MAX_TAPER_SLOPE`].
///
/// ⚠️ Vive **aqui** e não junto da lei porque o avaliador não pode depender do traçador — o
/// traçador é que depende dele. A medição mora onde a marcha mora.
#[test]
#[ignore = "medição, não gate — corre com --ignored --nocapture"]
fn measure_taper_frame_cost() {
    use ph2d_field::{Node, NodeId, NodeKind, Primitive, Unary, Xform};
    println!("declive | ms/quadro (320x240) | razão");
    let mut base = 0.0f64;
    for slope in [0.0f32, 0.25, 0.5, 0.75, 1.0, 1.5] {
        let doc = ph2d_field::FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.4 }),
                mods: vec![Unary::Taper { slope }],
                verb: None,
            }],
            NodeId(0),
        )
        .expect("esfera inclinada");
        let cam = Orbit::default();
        // Aquece: a primeira corrida paga a compilação da árvore.
        let _ = trace(&doc, &Registry::new(), &cam, 320, 240);
        let t0 = std::time::Instant::now();
        const N: u32 = 5;
        for _ in 0..N {
            let _ = trace(&doc, &Registry::new(), &cam, 320, 240);
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
        if slope == 0.0 {
            base = ms;
        }
        println!("{slope:7.2} | {ms:19.2} | {:5.2}x", ms / base);
    }
}

/// ⭐ **Uma marcha ABANDONADA devolve nada — e devolve depressa** (W32).
///
/// ⚠️ **As duas metades são o gate.** *Devolver nada* é o contrato (quem pediu já mudou de ideias, e
/// um G-buffer meio traçado na tela seria pior do que a espera); *depressa* é a razão de existir —
/// se a bandeira fosse lida uma vez no fim, a função cumpriria o contrato e não pouparia **um único
/// milissegundo** dos 121 que a wave veio cortar.
#[test]
fn an_abandoned_march_returns_nothing_and_returns_fast() {
    use std::sync::atomic::AtomicBool;
    let doc = sphere(0.6);
    let cam = Orbit::default();
    let (w, h) = (320, 240);

    // Aquece — a primeira corrida paga a compilação da fita.
    let _ = trace(&doc, &Registry::new(), &cam, w, h);
    let t0 = std::time::Instant::now();
    let whole = trace(&doc, &Registry::new(), &cam, w, h);
    let full_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(whole.hits() > 0, "a fixture tem de desenhar alguma coisa");

    let cancel = AtomicBool::new(true);
    let t1 = std::time::Instant::now();
    let out = trace_cancellable(&doc, &Registry::new(), &cam, w, h, &cancel, true, None);
    let cut_ms = t1.elapsed().as_secs_f64() * 1000.0;

    assert!(out.is_none(), "uma marcha abandonada não devolve imagem");
    assert!(
        cut_ms < full_ms * 0.5,
        "abandonar tem de POUPAR o trabalho: {cut_ms:.2} ms contra {full_ms:.2} ms inteiros — a \
         bandeira não está a ser lida por linha"
    );

    // ⚠️ E o CONTROLE: sem a bandeira, a mesma função devolve a mesma imagem de sempre.
    let ok = trace_cancellable(
        &doc,
        &Registry::new(),
        &cam,
        w,
        h,
        &AtomicBool::new(false),
        true,
        None,
    )
    .expect("sem cancelamento, ela traça");
    assert_eq!(
        ok.hits(),
        whole.hits(),
        "a marcha cancelável e a de sempre são a MESMA marcha"
    );
}

/// Polígono regular de `n` lados inscrito no círculo de raio `r` — a fixture das sondas de perfil.
/// A região de um ladrilho de 64 px pelo **caminho do produto** — `tile_t_range` + `region_between`.
///
/// ⚠️ Ela existe porque o `tile_region` de uma peça só **morreu** quando a marcha passou a fatiar:
/// o produto já não pergunta a caixa do tubo inteiro. Uma sonda que chamasse a função morta mediria
/// código que ninguém corre.
fn region_of_tile(
    cam: &crate::Orbit,
    plane: crate::Screen,
    tile: (usize, usize),
    bbox: ([f32; 3], [f32; 3]),
    margin: f32,
) -> Option<crate::tiles::Region> {
    let lo_px = (tile.0 * 64, tile.1 * 64);
    let hi_px = (lo_px.0 + 64, lo_px.1 + 64);
    let (t_lo, t_hi) = crate::tiles::tile_t_range(cam, plane, lo_px, hi_px, bbox)?;
    crate::tiles::region_between(cam, plane, lo_px, hi_px, bbox, margin, t_lo, t_hi)
}

fn ngon_probe(n: usize, r: f64) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(r * a.cos()) as f32, (r * a.sin()) as f32]
        })
        .collect()
}

/// ⭐⭐ **O TETO DE QUALQUER CURA DO PERFIL** — quanto do quadro é, de facto, o perfil.
///
/// ⚠️ Sem este número, uma aceleração de `k×` no perfil vira uma promessa sobre o quadro que ninguém
/// mediu. Ele é o **limite de Amdahl** desta wave.
#[test]
#[ignore]
fn the_ceiling_of_any_profile_cure() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let trace = |p: Primitive| -> f64 {
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(p),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let mut ms = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let _ = crate::trace(&doc, &reg, &cam, 640, 480);
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        ms.sort_by(f64::total_cmp);
        ms[3]
    };
    let base = trace(Primitive::Cylinder {
        radius: 0.5,
        half_height: 0.2,
        round: 0.0,
        chamfer: 0.0,
    });
    println!("um cilindro analítico: {base:.1} ms  (o piso: marcha, normais, anti-serrilhado)");
    println!("arestas | traçado | fração que é o PERFIL | teto de uma cura de {{k}}x no perfil");
    for n in [56usize, 168, 664] {
        let p =
            Profile::new(vec![ngon_probe(n, 0.5)], FillRule::NonZero, 1e-3).expect("perfil válido");
        let ms = trace(Primitive::Extrude {
            profile: p,
            half_height: 0.2,
            round: 0.0,
            chamfer: 0.0,
        });
        let frac = (ms - base) / ms;
        println!(
            "{n:>7} | {ms:>5.1} ms | {:>19.1}% | 2x -> {:.1}x · 5x -> {:.1}x · ∞ -> {:.1}x",
            frac * 100.0,
            1.0 / (1.0 - frac / 2.0),
            1.0 / (1.0 - frac * 0.8),
            1.0 / (1.0 - frac)
        );
    }
}

/// ⭐⭐⭐ **A MARCHA POR LADRILHO DESENHA A MESMA IMAGEM QUE A DE LINHA** (W56).
///
/// ⚠️ **É o gate que autoriza a troca.** O caminho novo especializa a árvore por região, prende o
/// raio à caixa da peça e parte o quadro em ladrilhos — três mudanças cuja falha comum é a **mesma**:
/// uma distância sobre-estimada faz a esfera-marcha **atravessar** a superfície, e o sintoma é um
/// buraco na imagem, não um número errado.
///
/// ⚠️ **A régua é a MÁSCARA e a NORMAL, e não uma média.** Um buraco de um pixel some numa média; e
/// a normal é o que a quina viva escreve — uma imagem com a máscara certa e a normal alisada é
/// exactamente o defeito que este módulo existe para não ter.
#[test]
fn the_tiled_march_draws_the_same_image_as_the_row_march() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let prof =
        |pts: Vec<[f32; 2]>| Profile::new(vec![pts], FillRule::NonZero, 1e-3).expect("perfil");
    let cases: Vec<(&str, Primitive, Xform)> = vec![
        (
            "extrusão de 168 lados",
            Primitive::Extrude {
                profile: prof(ngon_probe(168, 0.5)),
                half_height: 0.2,
                round: 0.0,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        ),
        (
            "extrusão POSADA e com filete",
            Primitive::Extrude {
                profile: prof(ngon_probe(96, 0.45)),
                half_height: 0.3,
                round: 0.04,
                chamfer: 0.0,
            },
            Xform {
                translation: [0.1, -0.05, 0.08],
                rotation: [0.25, 0.12, 0.05, 0.95],
                scale: 1.2,
            },
        ),
        (
            "torno",
            Primitive::Revolve {
                profile: prof(vec![[0.15, -0.3], [0.5, -0.3], [0.5, 0.3], [0.15, 0.3]]),
            },
            Xform::IDENTITY,
        ),
    ];
    for (name, p, xform) in cases {
        let doc = FieldDoc::new(
            vec![Node {
                xform,
                kind: NodeKind::Leaf(p),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        for cam in [
            crate::Orbit::from_yaw_pitch(0.72, 0.52),
            crate::Orbit::from_yaw_pitch(0.0, 0.0),
            crate::Orbit::from_yaw_pitch(1.9, -0.8),
        ] {
            let tiled = crate::trace(&doc, &reg, &cam, 240, 180);
            let rows = crate::trace_by_rows_for_test(&doc, &reg, &cam, 240, 180);
            // ⚠️ **A barra não é ZERO, e a razão é aritmética — não folga.** As duas árvores são
            // algebricamente a mesma, mas o `min` corre sobre subconjuntos diferentes: a soma e a
            // raiz caem em ordens diferentes e o resultado difere no **último bit**. Num raio
            // **rasante** (tangente à silhueta) `1e-7` decide entre acertar e passar ao lado, e numa
            // **quina viva** decide de que face a normal é. ⇒ o que se exige é que a discordância
            // seja **rara e de fronteira**, e a metade que a prende é a segunda asserção.
            let (w, h) = (tiled.width as usize, tiled.height as usize);
            let on_silhouette = |k: usize| {
                let (x, y) = (k % w, k / w);
                let me = rows.hit[k];
                [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dy)| {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            return true;
                        }
                        rows.hit[ny as usize * w + nx as usize] != me
                    })
            };
            let miss: Vec<usize> = (0..w * h)
                .filter(|k| tiled.hit[*k] != rows.hit[*k])
                .collect();
            let interior: Vec<usize> = miss
                .iter()
                .copied()
                .filter(|k| !on_silhouette(*k))
                .collect();
            for k in &interior {
                let (x, y) = (k % w, k / w);
                println!(
                    "  DIVERGE em ({x}, {y})  ladrilho={} linha={}  borda de ladrilho: x%64={} y%64={}",
                    tiled.hit[*k],
                    rows.hit[*k],
                    x % 64,
                    y % 64
                );
            }
            assert!(
                interior.is_empty(),
                "{name}: {} pixels de máscara diferem LONGE da silhueta — isso não é o último bit, \
                 é a marcha a atravessar a peça nalgum ladrilho",
                interior.len()
            );
            assert!(
                miss.len() * 500 < w * h,
                "{name}: {} pixels de máscara diferem ({:.2}% do quadro) — de fronteira ou não, \
                 isto deixou de ser o último bit",
                miss.len(),
                miss.len() as f32 * 100.0 / (w * h) as f32
            );
            let mut worst_interior = 0.0f32;
            let mut off = 0usize;
            for k in 0..w * h {
                if !tiled.hit[k] || !rows.hit[k] {
                    continue;
                }
                let d = (0..3)
                    .map(|i| (tiled.normal[k][i] - rows.normal[k][i]).abs())
                    .fold(0.0f32, f32::max);
                if d > 1.0e-3 {
                    off += 1;
                    if !on_silhouette(k) {
                        worst_interior = worst_interior.max(d);
                    }
                }
            }
            assert!(
                off * 200 < w * h,
                "{name}: {off} pixels com normal diferente ({:.2}% do quadro)",
                off as f32 * 100.0 / (w * h) as f32
            );
            let _ = worst_interior;
            // …e o controle: a peça de facto aparece, senão duas imagens vazias seriam iguais.
            assert!(
                tiled.hit.iter().filter(|h| **h).count() > 500,
                "{name}: a peça não apareceu — o gate compararia dois vazios"
            );
        }
    }
}

/// ⭐⭐⭐ **O QUE A MARCHA POR LADRILHO COMPRA NO QUADRO** (W56) — o número que o artista sente.
///
/// ⚠️ `#[ignore]` porque mede relógio — máquina calma:
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_what_the_tiled_march_buys --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_what_the_tiled_march_buys() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let time = |f: &dyn Fn() -> usize| {
        let mut ms = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let n = f();
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(n > 0);
        }
        ms.sort_by(f64::total_cmp);
        ms[3]
    };
    println!("arestas | linha | ladrilho | ganho");
    for n in [56usize, 168, 664] {
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile: Profile::new(vec![ngon_probe(n, 0.5)], FillRule::NonZero, 1e-3)
                        .expect("perfil"),
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let rows = time(&|| {
            crate::trace_by_rows_for_test(&doc, &reg, &cam, 640, 480)
                .hit
                .len()
        });
        let tiled = time(&|| crate::trace(&doc, &reg, &cam, 640, 480).hit.len());
        let by_tile: Vec<String> = [32usize, 48, 64, 96, 128, 192]
            .into_iter()
            .map(|t| {
                let ms = time(&|| {
                    crate::trace_tiled_for_test(
                        &doc,
                        &reg,
                        &cam,
                        640,
                        480,
                        t,
                        crate::tiles::SLABS,
                        true,
                        true,
                    )
                    .expect("ladrilho")
                    .hit
                    .len()
                });
                format!("{t}:{ms:.0}")
            })
            .collect();
        println!(
            "{n:>7} | {rows:>5.1} | {tiled:>8.1} | {:>4.1}x   por lado: {}",
            rows / tiled,
            by_tile.join("  ")
        );
    }
}

/// ⭐⭐ **A REGIÃO DE UM LADRILHO TEM DE SER MENOR QUE A PEÇA** — senão especializar não especializa.
///
/// ⛔ **Defeito medido (W56):** a primeira versão tomava o tubo do raio até `T_MAX` e intersectava
/// com a caixa da peça — e o tubo é tão comprido que a caixa dele **engolia a peça inteira**. Toda
/// região saía sendo a peça, a especialização guardava todas as arestas, e o ganho no quadro caiu de
/// `5×` para `1,3×`. ⚠️ **Nada ficou errado na imagem** — foi só lento, que é a forma de defeito que
/// um gate de paridade não vê. *Uma região que não é menor que a peça não é uma região.*
#[test]
fn a_tile_region_is_much_smaller_than_the_piece() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![ngon_probe(168, 0.5)], FillRule::NonZero, 1e-3)
                    .expect("perfil"),
                half_height: 0.2,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("a peça");
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("a caixa");
    let piece = bbox.1[0] - bbox.0[0];
    for cam in [
        crate::Orbit::from_yaw_pitch(0.72, 0.52),
        crate::Orbit::from_yaw_pitch(0.0, 0.0),
    ] {
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
        // Os ladrilhos que de facto tocam a peça — os de fundo devolvem a caixa inteira e não
        // interessam a esta medida.
        // ⚠️ Os ladrilhos **de fundo** recebem a caixa inteira de propósito (nenhum raio de canto
        // alcança a peça, e desistir da especialização é a resposta segura) — o que este gate mede é
        // que os ladrilhos **sobre a peça** recebem uma região pequena.
        let mut small = 0usize;
        let mut n = 0usize;
        for ty in 0..480usize / 64 {
            for tx in 0..640usize / 64 {
                let Some(r) = region_of_tile(&cam, plane, (tx, ty), bbox, sharp.normal) else {
                    continue;
                };
                let side = (0..3).map(|k| r.hi[k] - r.lo[k]).fold(0.0f32, f32::max);
                n += 1;
                if side / piece < 0.5 {
                    small += 1;
                }
            }
        }
        assert!(n > 20, "poucos ladrilhos medidos ({n})");
        assert!(
            small >= 8,
            "só {small} de {n} ladrilhos receberam uma região com menos de metade da peça — o tubo \
             do ladrilho está a engolir tudo, e a especialização não especializa"
        );
    }
}

/// ⭐⭐⭐ **O QUE UMA FATIA DE PROFUNDIDADE COMPRA, ANTES DE A CONSTRUIR** (W56e) — a régua.
///
/// ⚠️ **A W56d parou em `1,8×` com o tecto em `12,5×`, e o mecanismo foi medido:** um raio de viés
/// varre em `(u, v)` muito mais do que a largura do ladrilho, então a pegada efectiva é `≈ 0,4` da
/// peça e não os `0,125` do lado. ⭐ **E a varredura é `largura + profundidade · |direcção|`** — o
/// segundo termo **não depende do lado do ladrilho**, e é por isso que a varredura de `TILE` viu um
/// vale e não uma descida: encolher o ladrilho não encolhe a pegada. *A única forma de encolher a
/// pegada é encolher a PROFUNDIDADE.*
///
/// Esta sonda mede, na moeda que decide — **arestas guardadas** —, o que repartir a profundidade em
/// `N` fatias faz às duas contas que puxam para lados opostos:
///
/// - **Σ das fatias** — o que a montagem custaria se TODAS fossem construídas (o pessimista).
/// - **média das fatias** — o que cada avaliação passa a custar.
/// - **1ª fatia com peça** — quantas fatias um raio que ACERTA de facto atravessa, que é o que a
///   montagem preguiçosa paga.
///
/// ⚠️ `#[ignore]` porque imprime uma tabela:
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_what_a_depth_slab_would_buy --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_what_a_depth_slab_would_buy() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    for n in [56usize, 168, 664] {
        let profile =
            Profile::new(vec![ngon_probe(n, 0.5)], FillRule::NonZero, 1e-3).expect("perfil");
        let idx = ph2d_field_eval::profile_index::ProfileIndex::build(&profile);
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile,
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .map(ph2d_field_eval::bounds::Ball::aabb)
            .expect("a caixa");
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
        // ⚠️ A pegada é do EXTRUDE, cujo `(u, v)` é `(x, y)` local — e a pose é a identidade, então
        // a caixa de mundo é a caixa local. Uma peça com pose pediria o `Affine::box_of`.
        let kept = |r: crate::tiles::Region| idx.probe_cull([r.lo[0], r.lo[1]], [r.hi[0], r.hi[1]]);
        println!("--- {n} arestas ---");
        println!("  N | Σ fatias | média | máx | 1ª com peça | ladrilhos");
        for slabs in [1usize, 2, 3, 4, 6, 8] {
            let (mut sum, mut mean, mut mx, mut first, mut tiles) =
                (0.0f64, 0.0f64, 0usize, 0.0f64, 0usize);
            for ty in 0..480usize / 64 {
                for tx in 0..640usize / 64 {
                    let (lo_px, hi_px) = ((tx * 64, ty * 64), (tx * 64 + 64, ty * 64 + 64));
                    // Só os ladrilhos que de facto tocam a peça — os de fundo desistem, e
                    // acrescentá-los diluiria a medida com regiões que ninguém especializa.
                    let Some((t_lo, t_hi)) =
                        crate::tiles::tile_t_range(&cam, plane, lo_px, hi_px, bbox)
                    else {
                        continue;
                    };
                    let mut each = Vec::with_capacity(slabs);
                    for k in 0..slabs {
                        let a = t_lo + (t_hi - t_lo) * (k as f32) / (slabs as f32);
                        let b = t_lo + (t_hi - t_lo) * ((k + 1) as f32) / (slabs as f32);
                        let e = crate::tiles::region_between(
                            &cam,
                            plane,
                            lo_px,
                            hi_px,
                            bbox,
                            sharp.normal,
                            a,
                            b,
                        )
                        .map_or(0, kept);
                        each.push(e);
                    }
                    if each.iter().all(|e| *e == 0) {
                        continue;
                    }
                    tiles += 1;
                    sum += each.iter().sum::<usize>() as f64;
                    mean += each.iter().sum::<usize>() as f64 / slabs as f64;
                    mx = mx.max(each.iter().copied().max().unwrap_or(0));
                    // Quantas fatias um raio atravessa até à primeira que contém alguma aresta —
                    // ⚠️ é um LIMITE INFERIOR do que a montagem preguiçosa paga (um raio que falha a
                    // peça atravessa todas), e é de propósito: ele diz se a preguiça tem o que colher.
                    first += each.iter().position(|e| *e > 0).map_or(slabs, |i| i + 1) as f64;
                }
            }
            let t = tiles.max(1) as f64;
            println!(
                "  {slabs} | {:>8.1} | {:>5.1} | {mx:>3} | {:>11.2} | {tiles}",
                sum / t,
                mean / t,
                first / t,
            );
        }
    }
}

/// ⭐⭐⭐ **AS DUAS METADES DO QUADRO POR LADRILHO: montar a fita, e marchar** (W56e).
///
/// ⚠️ **Sem esta separação, qualquer decisão sobre fatiar a profundidade é um modelo.** Repartir a
/// profundidade em `N` fatias **divide** o custo de avaliar e **multiplica** o de montar (medido em
/// `the_table_of_what_a_depth_slab_would_buy`: a `N = 4`, avaliar cai a `0,52×` e montar sobe a
/// `2,1×`). Qual das duas manda decide se a wave vale — e o número tem de ser medido, não estimado.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_which_half_the_tiled_frame_pays --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_which_half_the_tiled_frame_pays() {
    use rayon::prelude::*;

    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("arestas | quadro | montar | marchar | montar%");
    for n in [56usize, 168, 664] {
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile: Profile::new(vec![ngon_probe(n, 0.5)], FillRule::NonZero, 1e-3)
                        .expect("perfil"),
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .map(ph2d_field_eval::bounds::Ball::aabb)
            .expect("a caixa");
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
        let rc = ph2d_field_eval::RegionCompiler::new(&doc);
        let mut frame = Vec::new();
        let mut build = Vec::new();
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let g = crate::trace(&doc, &reg, &cam, 640, 480);
            frame.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(g.hit.iter().any(|h| *h));
            // ⚠️ **A montagem SOZINHA, pelo mesmo caminho** — a mesma `region_between`, a mesma
            // `compile`, o mesmo `from_tree`. Uma segunda conta aqui mediria outra coisa.
            // ⚠️ **`par_iter`, como o quadro faz.** A 1ª versão desta sonda mediu a montagem em
            // SÉRIE e imprimiu `197%` do quadro — um número impossível que só dizia que o
            // denominador corria em 32 núcleos e o numerador em um. *Uma régua tem de correr no
            // mesmo regime do que ela mede.*
            let tiles: Vec<(usize, usize)> = (0..480usize / 64)
                .flat_map(|ty| (0..640usize / 64).map(move |tx| (tx, ty)))
                .collect();
            let t0 = std::time::Instant::now();
            let acc: usize = tiles
                .par_iter()
                .map(|&(tx, ty)| {
                    let Some(r) = region_of_tile(&cam, plane, (tx, ty), bbox, sharp.normal) else {
                        return 0;
                    };
                    let tree = rc.compile_at(&doc, r.lo, r.hi, &r.pts);
                    ph2d_field_eval::hybrid::Hybrid::from_tree(tree).sampled_count() + 1
                })
                .sum();
            build.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(acc > 0);
        }
        let (f, b) = (med(frame), med(build));
        println!(
            "{n:>7} | {f:>6.1} | {b:>6.1} | {:>7.1} | {:>6.0}%",
            f - b,
            b * 100.0 / f
        );
    }
}

/// ⭐⭐⭐ **O PASSO DA MARCHA É UMA PROPRIEDADE DO DOCUMENTO, NÃO UMA CONSTANTE?** (W56e) — a sonda.
///
/// A marcha anda `d · SAFE_STEP` com `SAFE_STEP = 1/√2`, e o número é o recíproco de uma constante
/// **medida**: a W0 viu `‖∇f‖` chegar a `√2` no operador de arredondamento exacto. ⚠️ Mas um extrude
/// sem `round`, sobre uma distância de polígono exacta, é uma distância **verdadeira** — `‖∇f‖ = 1`
/// quase em todo o lado —, e nele andar `d` inteiro é seguro. *O caminho mais lento a definir o
/// passo do mais rápido é exactamente o que o `CLAUDE.md` §0 proíbe.*
///
/// A sonda mede as duas respostas **no mesmo processo** e compara também a IMAGEM: um passo maior
/// que atravesse a superfície aparece como pixel de fundo no meio da peça.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_what_a_full_march_step_would_buy --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_what_a_full_march_step_would_buy() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("arestas | 1/√2 | 1,0 | ganho | pixels diferentes");
    for n in [56usize, 168, 664] {
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile: Profile::new(vec![ngon_probe(n, 0.5)], FillRule::NonZero, 1e-3)
                        .expect("perfil"),
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        // ⚠️ **Alternadas**, e não sete de uma e sete da outra: uma deriva de máquina a meio da
        // corrida ficaria toda dentro de um dos dois lados.
        let (mut safe, mut full) = (Vec::new(), Vec::new());
        let (mut a, mut b) = (None, None);
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let g = crate::trace_stepped_for_test(
                &doc,
                &reg,
                &cam,
                640,
                480,
                std::f32::consts::FRAC_1_SQRT_2,
            );
            safe.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = std::time::Instant::now();
            let h = crate::trace_stepped_for_test(&doc, &reg, &cam, 640, 480, 1.0);
            full.push(t0.elapsed().as_secs_f64() * 1e3);
            a = Some(g);
            b = Some(h);
        }
        let (g, h) = (a.expect("g"), b.expect("h"));
        let diff = (0..g.hit.len()).filter(|k| g.hit[*k] != h.hit[*k]).count();
        let (s, f) = (med(safe), med(full));
        println!("{n:>7} | {s:>4.1} | {f:>3.1} | {:>4.2}x | {diff}", s / f);
    }
}

/// ⭐⭐⭐ **ONDE A MONTAGEM DE UM LADRILHO GASTA** (W56e) — porque é ela que limita as fatias.
///
/// ⚠️ **É o número que decide a wave.** Repartir a profundidade em `N` fatias divide o custo de
/// avaliar e MULTIPLICA o de montar; a `N = 4` a conta modelada dá `1,25×` só porque montar custa
/// `18%` do quadro. Se a montagem tiver gordura, `N` pode subir e o ganho com ela.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_where_the_tile_assembly_goes --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_where_the_tile_assembly_goes() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("arestas | árvore | fita | total (µs por ladrilho, SÉRIE)");
    for n in [56usize, 168, 664] {
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile: Profile::new(vec![ngon_probe(n, 0.5)], FillRule::NonZero, 1e-3)
                        .expect("perfil"),
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .map(ph2d_field_eval::bounds::Ball::aabb)
            .expect("a caixa");
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
        let rc = ph2d_field_eval::RegionCompiler::new(&doc);
        let regions: Vec<crate::tiles::Region> = (0..480usize / 64)
            .flat_map(|ty| (0..640usize / 64).map(move |tx| (tx, ty)))
            .filter_map(|(tx, ty)| region_of_tile(&cam, plane, (tx, ty), bbox, sharp.normal))
            .collect();
        let (mut tree_ms, mut tape_ms) = (Vec::new(), Vec::new());
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let trees: Vec<_> = regions
                .iter()
                .map(|r| rc.compile_at(&doc, r.lo, r.hi, &r.pts))
                .collect();
            tree_ms.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = std::time::Instant::now();
            let mut acc = 0usize;
            for t in trees {
                acc += ph2d_field_eval::hybrid::Hybrid::from_tree(t).sampled_count() + 1;
            }
            tape_ms.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(acc > 0);
        }
        let k = regions.len() as f64;
        let (a, b) = (med(tree_ms) * 1e3 / k, med(tape_ms) * 1e3 / k);
        println!(
            "{n:>7} | {a:>6.0} | {b:>4.0} | {:>5.0}  ({} ladrilhos)",
            a + b,
            regions.len()
        );
    }
}

/// Um contorno em **estrela** com `n` vértices — o oposto de equidistante.
fn star_probe(n: usize, r_in: f64, r_out: f64) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            let r = if i % 2 == 0 { r_out } else { r_in };
            [(r * a.cos()) as f32, (r * a.sin()) as f32]
        })
        .collect()
}

/// Uma **barra dentada** — comprida num eixo, com dentes de um lado só.
fn comb_probe(teeth: usize, half_w: f64, half_h: f64, tooth: f64) -> Vec<[f32; 2]> {
    let mut v = Vec::with_capacity(teeth * 4 + 4);
    v.push([-half_w as f32, -half_h as f32]);
    v.push([half_w as f32, -half_h as f32]);
    v.push([half_w as f32, half_h as f32]);
    for i in (0..teeth).rev() {
        let x0 = -half_w + 2.0 * half_w * (i as f64 + 0.15) / teeth as f64;
        let x1 = -half_w + 2.0 * half_w * (i as f64 + 0.85) / teeth as f64;
        v.push([x1 as f32, half_h as f32]);
        v.push([x1 as f32, (half_h + tooth) as f32]);
        v.push([x0 as f32, (half_h + tooth) as f32]);
        v.push([x0 as f32, half_h as f32]);
    }
    v.push([-half_w as f32, half_h as f32]);
    v
}

/// ⭐⭐⭐ **O CORTE NUNCA PODIA CORTAR: a fixtura desta wave é um CÍRCULO** (W56e).
///
/// ⛔ **Medido, e reabre o número de manchete.** O corte guarda toda aresta a menos de
/// `dmax = min_e (máx distância de um CANTO da caixa a `e`)`. Numa peça **redonda** todas as
/// arestas estão à mesma distância do centro ⇒ uma região no interior, por mais pequena que seja,
/// guarda **as 168**. A coluna `máx` da sonda das fatias dizia-o de frente — `168` em todo `N` — e
/// eu li-a como "há um ladrilho mau", quando ela é a lei da peça.
///
/// ⚠️ **Um `n`-gono regular é o PIOR caso de um corte por distância, e foi a fixtura de toda a
/// W56.** O `1,8×` de manchete é, portanto, um **piso**, não a medida do que o artista sente.
/// *A terceira vez nesta wave que a fixtura mede outra coisa que não o fenómeno.*
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_what_the_shape_of_the_outline_does --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_what_the_shape_of_the_outline_does() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let shapes: Vec<(&str, Vec<[f32; 2]>)> = vec![
        ("círculo 168", ngon_probe(168, 0.5)),
        ("estrela 168", star_probe(168, 0.22, 0.5)),
        ("pente 172", comb_probe(42, 0.55, 0.12, 0.10)),
    ];
    println!("contorno | arestas | guardadas | % | linha | ladrilho | ganho");
    for (name, ring) in shapes {
        let n = ring.len();
        let profile = Profile::new(vec![ring], FillRule::NonZero, 1e-3).expect("perfil");
        let idx = ph2d_field_eval::profile_index::ProfileIndex::build(&profile);
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile,
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .map(ph2d_field_eval::bounds::Ball::aabb)
            .expect("a caixa");
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
        let (mut kept, mut tiles) = (0usize, 0usize);
        for ty in 0..480usize / 64 {
            for tx in 0..640usize / 64 {
                let Some(r) = region_of_tile(&cam, plane, (tx, ty), bbox, sharp.normal) else {
                    continue;
                };
                kept += idx.probe_cull([r.lo[0], r.lo[1]], [r.hi[0], r.hi[1]]);
                tiles += 1;
            }
        }
        let (mut rows, mut tiled) = (Vec::new(), Vec::new());
        for _ in 0..7 {
            let t0 = std::time::Instant::now();
            let a = crate::trace_by_rows_for_test(&doc, &reg, &cam, 640, 480);
            rows.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = std::time::Instant::now();
            let b = crate::trace(&doc, &reg, &cam, 640, 480);
            tiled.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(a.hit.iter().any(|h| *h) && b.hit.iter().any(|h| *h));
        }
        let (r, t) = (med(rows), med(tiled));
        let k = kept as f64 / tiles.max(1) as f64;
        println!(
            "{name:>11} | {n:>7} | {k:>9.1} | {:>3.0}% | {r:>5.1} | {t:>8.1} | {:>4.1}x",
            k * 100.0 / n as f64,
            r / t
        );
    }
}

/// ⛔⛔ **TODA AMOSTRA CAI DENTRO DA REGIÃO QUE CONSTRUIU A FITA QUE A AVALIA** (W56e).
///
/// É **o** invariante da marcha por fatia: a árvore especializada só concorda com o documento
/// dentro da região para que foi construída, então uma amostra fora dela é resposta inventada — e
/// o sintoma não é ruído, é a marcha a atravessar a peça ou a inventar superfície.
///
/// # As duas coisas que este gate apanhou
///
/// ⛔ **Os quatro raios de canto NÃO bastam na lente convergente.** O doc do `tile_region` dizia
/// *"e não é aproximação"*. É, e só na **paralela** é exacta: lá a direcção é constante e um raio
/// interior é combinação convexa dos cantos. Na convergente a direcção é **normalizada**, `d̂(s)`
/// percorre um quadrilátero **esférico**, e ele abaúla para fora da corda. Medido, câmera de
/// frente: a fuga vai de `2,80e-4` com uma fatia a `4,03e-4` com oito, e **passa a folga** de
/// `4e-4` exactamente quando a fatia aperta. *A premissa não mordia porque o tubo era grande;
/// fatiar é o que a acorda.* A cura é a **flecha** (ver [`crate::tiles::region_between`]).
///
/// ⛔ **E o `t` de entrada é CONVEXO na posição de ecrã** (`max` de funções afins), então o mínimo
/// dele pode ser **interior** ao ladrilho: medido, um raio interior entra até **`7,4e-2` antes** do
/// `t_lo` que os quatro cantos dão, e sai até `1,2e-1` depois do `t_hi` — sobre uma peça que mede
/// `1,0`. É por isso que a 1.ª fatia começa em `0` e a última acaba em `T_MAX`.
#[test]
fn every_sample_lies_inside_the_region_that_built_its_tape() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![ngon_probe(24, 0.5)], FillRule::NonZero, 1e-3)
                    .expect("perfil"),
                half_height: 0.2,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("a peça");
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("a caixa");
    for (name, cam) in [
        ("de viés", crate::Orbit::from_yaw_pitch(0.72, 0.52)),
        ("de frente", crate::Orbit::from_yaw_pitch(0.0, 0.0)),
        ("quase de canto", crate::Orbit::from_yaw_pitch(0.78, 0.62)),
    ] {
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
        // ⚠️ **Mais fatias do que o produto usa, de propósito.** A fuga da flecha CRESCE quando a
        // fatia aperta, então medir só no `SLABS` que shipa é medir o caso fácil.
        for slabs in [crate::tiles::SLABS, 4, 8, 16] {
            let (mut worst, mut where_, mut measured) = (0.0f32, (0usize, 0usize, 0usize), 0usize);
            for ty in 0..480usize / 64 {
                for tx in 0..640usize / 64 {
                    let (lo_px, hi_px) = ((tx * 64, ty * 64), (tx * 64 + 64, ty * 64 + 64));
                    // As MESMAS fronteiras que a marcha constrói — ver `tiles::tiled_trace`.
                    let Some((t_lo, t_hi)) =
                        crate::tiles::tile_t_range(&cam, plane, lo_px, hi_px, bbox)
                    else {
                        continue;
                    };
                    let bounds = crate::tiles::slab_bounds(t_lo, t_hi, slabs);
                    measured += 1;
                    // ⭐⭐ **As fronteiras têm de COBRIR o que cada raio marcha.** ⛔ Sem esta
                    // metade, duas mutações que apagavam a 1.ª e a última fronteira SOBREVIVIAM:
                    // o gate cortava as amostras pelas próprias fronteiras, então o pedaço de raio
                    // que ficava de fora nunca era medido. *Um invariante que se avalia dentro do
                    // domínio que ele define não diz nada sobre a fronteira dele.*
                    for ia in 0..17 {
                        for ib in 0..17 {
                            let px = lo_px.0 as f32 + 64.0 * ia as f32 / 16.0;
                            let py = lo_px.1 as f32 + 64.0 * ib as f32 / 16.0;
                            let (sx, sy) = plane.plane_at(px, py);
                            let (o, d) = cam.ray_at_plane(sx, sy);
                            let Some((ea, eb)) = crate::slab(o, d, bbox.0, bbox.1) else {
                                continue;
                            };
                            let (e0, e1) = (ea.max(0.0), eb.min(crate::T_MAX));
                            if e1 <= e0 {
                                continue;
                            }
                            assert!(
                                bounds[0] <= e0 && e1 <= bounds[bounds.len() - 1],
                                "{name}, {slabs} fatias: o ladrilho ({tx}, {ty}) marcha \
                                 [{e0:.4}, {e1:.4}] e as fatias só cobrem [{:.4}, {:.4}] — um \
                                 pedaço do raio é avaliado por fita nenhuma",
                                bounds[0],
                                bounds[bounds.len() - 1]
                            );
                        }
                    }
                    for k in 0..bounds.len() - 1 {
                        let (a0, a1) = (bounds[k], bounds[k + 1]);
                        if a1 <= a0 {
                            continue;
                        }
                        // `None` = a fatia não cruza a peça ⇒ a marcha usa o documento INTEIRO,
                        // que vale em todo o lado. Nada a exigir.
                        let Some(r) = crate::tiles::slab_region(
                            &cam,
                            plane,
                            lo_px,
                            hi_px,
                            bbox,
                            sharp.normal,
                            &bounds,
                            k,
                        ) else {
                            continue;
                        };
                        for ia in 0..9 {
                            for ib in 0..9 {
                                let px = lo_px.0 as f32 + 64.0 * ia as f32 / 8.0;
                                let py = lo_px.1 as f32 + 64.0 * ib as f32 / 8.0;
                                let (sx, sy) = plane.plane_at(px, py);
                                let (o, d) = cam.ray_at_plane(sx, sy);
                                // Só a parte do raio que ESTA fatia marcha: da entrada na caixa
                                // até à saída, cortada pela fatia.
                                let Some((ea, eb)) = crate::slab(o, d, bbox.0, bbox.1) else {
                                    continue;
                                };
                                let (s0, s1) = (ea.max(0.0).max(a0), eb.min(crate::T_MAX).min(a1));
                                if s1 <= s0 {
                                    continue;
                                }
                                for j in 0..9 {
                                    let t = s0 + (s1 - s0) * j as f32 / 8.0;
                                    for c in 0..3 {
                                        let v = d[c].mul_add(t, o[c]);
                                        let out = (r.lo[c] - v).max(v - r.hi[c]);
                                        if out > worst {
                                            worst = out;
                                            where_ = (tx, ty, k);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            assert!(
                measured > 20,
                "{name}: poucos ladrilhos medidos ({measured})"
            );
            assert!(
                worst <= 0.0,
                "{name}, {slabs} fatias: uma amostra sai {worst:.3e} da região da fatia \
                 {where_:?} — a fita dela responde ali onde não vale"
            );
        }
    }
}

/// ⭐⭐⭐ **QUANTAS FATIAS DE PROFUNDIDADE** (W56e) — a varredura que escolhe o [`crate::tiles::SLABS`].
///
/// ⚠️ **Ela mede também a IMAGEM**, e não só o relógio: uma fatia a mais é uma árvore a mais a ser
/// perguntada, e a coluna `≠` é o que separa "ficou rápido" de "ficou rápido e errado".
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_how_many_depth_slabs --ignored --nocapture
/// ```
/// ⚠️ **RECONFERIDA na W71 e o veredito mudou** — esta varredura escolheu `SLABS = 2` quando uma
/// região custava o **dobro** (a W70 tirou-lhe a fita de gradiente e o `fork`). A tabela nova, e
/// intercalada, está em `measure_where_the_frame_goes_and_how_many_slabs_it_wants`; o produto ship
/// `4`. *Uma varredura envelhece com o custo que ela pesava.*
#[test]
#[ignore]
fn the_table_of_how_many_depth_slabs() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let shapes: Vec<(&str, Vec<[f32; 2]>)> = vec![
        ("círculo 56", ngon_probe(56, 0.5)),
        ("círculo 168", ngon_probe(168, 0.5)),
        ("estrela 168", star_probe(168, 0.22, 0.5)),
        ("círculo 664", ngon_probe(664, 0.5)),
    ];
    println!("contorno | linha | N=1 | 2 | 3 | 4 | 6 | 8 | melhor");
    for (name, ring) in shapes {
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile: Profile::new(vec![ring], FillRule::NonZero, 1e-3).expect("perfil"),
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let rows = med((0..5)
            .map(|_| {
                let t0 = std::time::Instant::now();
                let g = crate::trace_by_rows_for_test(&doc, &reg, &cam, 640, 480);
                assert!(g.hit.iter().any(|h| *h));
                t0.elapsed().as_secs_f64() * 1e3
            })
            .collect());
        let base = crate::trace_by_rows_for_test(&doc, &reg, &cam, 640, 480);
        let mut cells = Vec::new();
        let mut best = (f64::INFINITY, 0usize);
        for n in [1usize, 2, 3, 4, 6, 8] {
            let ms = med((0..5)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let g =
                        crate::trace_tiled_for_test(&doc, &reg, &cam, 640, 480, 64, n, true, true)
                            .expect("ladrilho");
                    assert!(g.hit.iter().any(|h| *h));
                    t0.elapsed().as_secs_f64() * 1e3
                })
                .collect());
            let g = crate::trace_tiled_for_test(&doc, &reg, &cam, 640, 480, 64, n, true, true)
                .expect("ladrilho");
            let diff = (0..g.hit.len())
                .filter(|k| g.hit[*k] != base.hit[*k])
                .count();
            cells.push(format!("{ms:.0}/{diff}"));
            if ms < best.0 {
                best = (ms, n);
            }
        }
        println!(
            "{name:>11} | {rows:>5.0} | {} | N={} a {:.2}x",
            cells.join(" | "),
            best.1,
            rows / best.0
        );
    }
}

/// ⭐⭐ **UM RAIO INTERIOR ENTRA ANTES DO QUE OS QUATRO CANTOS DIZEM?** (W56e) — a cerca medida.
///
/// A faixa `[t_lo, t_hi]` de um ladrilho sai dos quatro raios de CANTO. O `t` de entrada na caixa é
/// `max` de funções afins da posição de ecrã ⇒ **convexo** ⇒ o **mínimo** dele pode ser interior ao
/// ladrilho, e não num canto. Se isso acontecer, um raio interior entra **antes** de `t_lo`, e a
/// fatia que começa aí seria perguntada onde não vale. É por causa disto que a 1.ª fatia começa em
/// `0` e a última acaba em `T_MAX`.
///
/// ⚠️ **Duas mutações que apagavam essas duas fronteiras SOBREVIVERAM ao gate de paridade** — ou
/// seja, a cerca pode estar a defender algo que a fixtura não contém. Esta sonda mede o défice.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_whether_an_inner_ray_enters_first --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_whether_an_inner_ray_enters_first() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![ngon_probe(24, 0.5)], FillRule::NonZero, 1e-3)
                    .expect("perfil"),
                half_height: 0.2,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("a peça");
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("a caixa");
    println!("câmera | ladrilhos | pior défice de entrada | pior excesso de saída");
    for (name, cam) in [
        ("de viés", crate::Orbit::from_yaw_pitch(0.72, 0.52)),
        ("de frente", crate::Orbit::from_yaw_pitch(0.0, 0.0)),
        ("quase de canto", crate::Orbit::from_yaw_pitch(0.78, 0.62)),
    ] {
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let (mut early, mut late, mut n) = (0.0f32, 0.0f32, 0usize);
        for ty in 0..480usize / 64 {
            for tx in 0..640usize / 64 {
                let (lo_px, hi_px) = ((tx * 64, ty * 64), (tx * 64 + 64, ty * 64 + 64));
                let Some((t_lo, t_hi)) =
                    crate::tiles::tile_t_range(&cam, plane, lo_px, hi_px, bbox)
                else {
                    continue;
                };
                n += 1;
                for a in 0..17 {
                    for b in 0..17 {
                        let px = lo_px.0 as f32 + 64.0 * a as f32 / 16.0;
                        let py = lo_px.1 as f32 + 64.0 * b as f32 / 16.0;
                        let (sx, sy) = plane.plane_at(px, py);
                        let (o, d) = cam.ray_at_plane(sx, sy);
                        let Some((ea, eb)) = crate::slab(o, d, bbox.0, bbox.1) else {
                            continue;
                        };
                        early = early.max(t_lo - ea.max(0.0));
                        late = late.max(eb.min(crate::T_MAX) - t_hi);
                    }
                }
            }
        }
        println!("{name:>15} | {n:>9} | {early:>21.3e} | {late:>20.3e}");
    }
}

/// ⭐⭐ **UMA FATIA TEM DE GUARDAR MENOS ARESTAS QUE O TUBO INTEIRO** (W56e) — senão fatiar não
/// fatia, e o que sobra é o preço da montagem a mais.
///
/// ⚠️ É o irmão de [`a_tile_region_is_much_smaller_than_the_piece`], um nível abaixo: aquele mede a
/// região do LADRILHO contra a peça, este mede a região da FATIA contra a do ladrilho. ⛔ Uma
/// mutação que colapsava as fronteiras de volta a uma fatia só **sobreviveu** ao gate de cima —
/// ele não tem como ver a decomposição em profundidade. *Um gate que mede o pai não mede o filho.*
#[test]
fn a_depth_slab_keeps_fewer_edges_than_the_whole_tube() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let profile =
        Profile::new(vec![ngon_probe(168, 0.5)], FillRule::NonZero, 1e-3).expect("perfil");
    let idx = ph2d_field_eval::profile_index::ProfileIndex::build(&profile);
    let doc = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile,
                half_height: 0.2,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("a peça");
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("a caixa");
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let plane = crate::Screen::new(640, 480, cam.half_extent);
    let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
    let kept = |r: crate::tiles::Region| idx.probe_cull([r.lo[0], r.lo[1]], [r.hi[0], r.hi[1]]);
    let (mut tube, mut sliced, mut n) = (0usize, 0usize, 0usize);
    for ty in 0..480usize / 64 {
        for tx in 0..640usize / 64 {
            let (lo_px, hi_px) = ((tx * 64, ty * 64), (tx * 64 + 64, ty * 64 + 64));
            let Some((t_lo, t_hi)) = crate::tiles::tile_t_range(&cam, plane, lo_px, hi_px, bbox)
            else {
                continue;
            };
            let bounds = crate::tiles::slab_bounds(t_lo, t_hi, crate::tiles::SLABS);
            let region = |k: usize| {
                crate::tiles::slab_region(&cam, plane, lo_px, hi_px, bbox, sharp.normal, &bounds, k)
            };
            let Some(whole) = crate::tiles::region_between(
                &cam,
                plane,
                lo_px,
                hi_px,
                bbox,
                sharp.normal,
                t_lo,
                t_hi,
            ) else {
                continue;
            };
            n += 1;
            tube += kept(whole);
            // O PIOR caso da decomposição — não a média: é o que uma avaliação chega a pagar.
            // ⚠️ Sem as duas fatias de FORA, que não são profundidade da peça e sim a cerca.
            let mut worst = 0usize;
            for k in 1..bounds.len() - 2 {
                worst = worst.max(region(k).map_or(0, kept));
            }
            sliced += worst;
        }
    }
    assert!(n > 20, "poucos ladrilhos medidos ({n})");
    let (a, b) = (tube as f32 / n as f32, sliced as f32 / n as f32);
    assert!(
        b * 100.0 <= a * 92.0,
        "a fatia guarda {b:.1} arestas contra {a:.1} do tubo inteiro ({:.0}%) — repartir a \
         profundidade deixou de repartir, e o que sobra é a montagem a mais",
        b * 100.0 / a
    );
}

/// ⭐⭐⭐ **O PASSO INTEIRO DESENHA A MESMA IMAGEM QUE O CURTO** (W56f) — a outra metade da prova.
///
/// ⚠️ **O gate do gradiente vive na `ph2d-field-eval` e prova a ARITMÉTICA** (`passo × ‖∇f‖ ≤ 1`).
/// Este prova o **produto**: se a classificação estiver errada num construtor, o passo maior
/// atravessa a superfície e o sintoma é **pixel de fundo no meio da peça** — que é a coisa que o
/// artista vê, e que nenhuma norma de gradiente exprime.
///
/// ⛔ Ele compara com o `1/√2` de sempre, que é o comportamento que shipou até a W56e — *a régua é
/// o que a peça era ontem*.
#[test]
fn the_full_march_step_draws_the_same_piece_as_the_short_one() {
    use ph2d_field::{
        Blend, FieldDoc, FillRule, Node, NodeId, NodeKind, Op, Primitive, Profile, Unary, Xform,
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let short = std::f32::consts::FRAC_1_SQRT_2;
    let bx = Primitive::Box {
        half: [0.4, 0.3, 0.25],
        round: 0.0,
        chamfer: 0.0,
    };
    let one = |p: Primitive, mods: Vec<Unary>| {
        let mut n = Node::new(Xform::IDENTITY, NodeKind::Leaf(p));
        n.mods = mods;
        FieldDoc::new(vec![n], NodeId(0)).expect("a peça")
    };
    let two = |op: Op| {
        FieldDoc::new(
            vec![
                Node::new(
                    Xform::at(-0.2, 0.0, 0.0),
                    NodeKind::Leaf(Primitive::Box {
                        half: [0.6, 0.3, 0.3],
                        round: 0.0,
                        chamfer: 0.0,
                    }),
                ),
                Node::new(
                    Xform::at(0.2, 0.0, 0.0),
                    NodeKind::Leaf(Primitive::Box {
                        half: [0.3, 0.6, 0.3],
                        round: 0.0,
                        chamfer: 0.0,
                    }),
                ),
                Node::new(
                    Xform::IDENTITY,
                    NodeKind::Combine {
                        op,
                        children: vec![NodeId(0), NodeId(1)],
                    },
                ),
            ],
            NodeId(2),
        )
        .expect("a peça")
    };
    let cases: Vec<(&str, FieldDoc)> = vec![
        ("caixa", one(bx.clone(), vec![])),
        (
            "caixa arredondada",
            one(
                Primitive::Box {
                    half: [0.4, 0.3, 0.25],
                    round: 0.12,
                    chamfer: 0.0,
                },
                vec![],
            ),
        ),
        (
            "toro",
            one(
                Primitive::Torus {
                    major: 0.4,
                    minor: 0.12,
                },
                vec![],
            ),
        ),
        (
            "desenho puxado",
            one(
                Primitive::Extrude {
                    profile: Profile::new(vec![star_probe(64, 0.2, 0.5)], FillRule::NonZero, 1e-3)
                        .expect("perfil"),
                    half_height: 0.25,
                    round: 0.0,
                    chamfer: 0.0,
                },
                vec![],
            ),
        ),
        (
            "casca",
            one(bx.clone(), vec![Unary::Shell { thickness: 0.04 }]),
        ),
        (
            "matriz radial",
            one(
                bx.clone(),
                vec![Unary::Radial {
                    count: 7,
                    joint: ph2d_field::Joint::SHARP,
                }],
            ),
        ),
        (
            "inclinação",
            one(bx.clone(), vec![Unary::Taper { slope: 1.5 }]),
        ),
        ("união viva", two(Op::Union(Blend::Sharp))),
        ("subtracção viva", two(Op::Difference(Blend::Sharp))),
        (
            "união orgânica",
            two(Op::Union(Blend::Organic { radius: 0.4 })),
        ),
    ];
    for (name, doc) in cases {
        let step = ph2d_field_eval::safe_march_step(&doc);
        assert!(
            step > 0.99,
            "{name}: este documento nem sequer ganhou o passo inteiro — a fixtura não contém o que \
             o gate mede"
        );
        for cam in [
            crate::Orbit::from_yaw_pitch(0.72, 0.52),
            crate::Orbit::from_yaw_pitch(0.0, 0.0),
        ] {
            let a = crate::trace_stepped_for_test(&doc, &reg, &cam, 240, 180, short);
            let b = crate::trace_stepped_for_test(&doc, &reg, &cam, 240, 180, step);
            let (w, h) = (a.width as usize, a.height as usize);
            let on_silhouette = |k: usize| {
                let (x, y) = (k % w, k / w);
                [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dy)| {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        nx < 0
                            || ny < 0
                            || nx >= w as i32
                            || ny >= h as i32
                            || a.hit[ny as usize * w + nx as usize] != a.hit[k]
                    })
            };
            // ⛔ **O que se caça é o buraco INTERIOR**: um pixel que o passo curto acerta e o longo
            // falha, longe da silhueta, é a marcha a atravessar a peça.
            let holes: Vec<usize> = (0..w * h)
                .filter(|k| a.hit[*k] && !b.hit[*k] && !on_silhouette(*k))
                .collect();
            assert!(
                holes.is_empty(),
                "{name}: {} pixels que o passo curto acerta e o inteiro FURA, longe da silhueta — \
                 a classificação de `safe_march_step` está errada para este documento",
                holes.len()
            );
            let diff = (0..w * h).filter(|k| a.hit[*k] != b.hit[*k]).count();
            assert!(
                diff * 200 < w * h,
                "{name}: {diff} pixels de máscara diferem ({:.2}% do quadro) — deixou de ser \
                 fronteira",
                diff as f32 * 100.0 / (w * h) as f32
            );
            assert!(
                a.hit.iter().filter(|x| **x).count() > 300,
                "{name}: a peça não apareceu — o gate compararia dois vazios"
            );
            // ⭐⭐ **E o PRODUTO tem de usar o passo que a lei deu.** Sem esta metade, a lei podia
            // estar certa e o traçador continuar a ler uma constante — que é precisamente o estado
            // de antes desta wave, e ele passa em todo gate que compare `trace_stepped_for_test`
            // consigo próprio. *Uma lei que o caminho do produto não chama não é uma lei.*
            let prod = crate::trace(&doc, &reg, &cam, 240, 180);
            assert_eq!(
                prod.hit, b.hit,
                "{name}: a marcha do produto não desenhou o mesmo que o passo `{step}` que a \
                 `safe_march_step` deu para este documento"
            );
        }
    }
    // …e o outro lado: um documento que a lei classifica como INFLADOR tem de continuar no curto.
    let rounded = two(Op::Union(Blend::Exact { radius: 0.15 }));
    assert!(
        (ph2d_field_eval::safe_march_step(&rounded) - short).abs() < 1e-6,
        "um documento com arredondamento exacto deixou de receber o passo curto"
    );
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    assert_eq!(
        crate::trace(&rounded, &reg, &cam, 240, 180).hit,
        crate::trace_stepped_for_test(&rounded, &reg, &cam, 240, 180, short).hit,
        "a peça arredondada deixou de ser marchada com o passo curto"
    );
}

/// O casco convexo de uns quantos pontos 2D (Andrew monotone chain) — sentido anti-horário.
fn hull_of(mut pts: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    pts.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: [f32; 2], a: [f32; 2], b: [f32; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut out: Vec<[f32; 2]> = Vec::with_capacity(pts.len() * 2);
    for pass in 0..2 {
        let start = out.len();
        let it: Box<dyn Iterator<Item = &[f32; 2]>> = if pass == 0 {
            Box::new(pts.iter())
        } else {
            Box::new(pts.iter().rev())
        };
        for &p in it {
            while out.len() >= start + 2 && cross(out[out.len() - 2], out[out.len() - 1], p) <= 0.0
            {
                out.pop();
            }
            out.push(p);
        }
        out.pop();
    }
    out
}

/// Corta o polígono convexo contra um rectângulo (Sutherland–Hodgman) — continua convexo.
fn clip_to_rect(poly: &[[f32; 2]], lo: [f32; 2], hi: [f32; 2]) -> Vec<[f32; 2]> {
    let mut cur = poly.to_vec();
    // Cada lado do rectângulo é um semi-plano; `keep` diz de que lado se fica.
    for (axis, bound, keep_ge) in [
        (0usize, lo[0], true),
        (0, hi[0], false),
        (1, lo[1], true),
        (1, hi[1], false),
    ] {
        if cur.is_empty() {
            break;
        }
        let inside = |p: &[f32; 2]| {
            if keep_ge {
                p[axis] >= bound
            } else {
                p[axis] <= bound
            }
        };
        let mut out: Vec<[f32; 2]> = Vec::with_capacity(cur.len() + 1);
        for i in 0..cur.len() {
            let (a, b) = (cur[i], cur[(i + 1) % cur.len()]);
            let (ia, ib) = (inside(&a), inside(&b));
            if ia {
                out.push(a);
            }
            if ia != ib {
                let t = (bound - a[axis]) / (b[axis] - a[axis]);
                let mut q = [0.0f32; 2];
                for k in 0..2 {
                    q[k] = a[k] + t * (b[k] - a[k]);
                }
                out.push(q);
            }
        }
        cur = out;
    }
    cur
}

/// Infla um polígono convexo por `pad`, empurrando cada LADO para fora e re-intersectando.
///
/// ⚠️ **Conservador de propósito:** o offset verdadeiro de um polígono é arredondado nas quinas, e
/// empurrar os lados dá o polígono que o **contém** (as quinas ficam bicudas, para fora). Uma
/// região *maior* que a necessária corta menos — nunca fura.
fn inflate(poly: &[[f32; 2]], pad: f32) -> Vec<[f32; 2]> {
    if poly.len() < 3 || pad <= 0.0 {
        return poly.to_vec();
    }
    // Semi-planos deslocados: a intersecção deles é o inflado. Feita por cortes sucessivos sobre um
    // quadrado bem maior que a peça.
    let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for p in poly {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let big = (hi[0] - lo[0]).max(hi[1] - lo[1]) + 4.0 * pad + 1.0;
    let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let mut cur = vec![
        [c[0] - big, c[1] - big],
        [c[0] + big, c[1] - big],
        [c[0] + big, c[1] + big],
        [c[0] - big, c[1] + big],
    ];
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let e = [b[0] - a[0], b[1] - a[1]];
        let len = e[0].hypot(e[1]);
        if len <= f32::EPSILON {
            continue;
        }
        // A normal EXTERIOR de um polígono anti-horário é `(e.y, -e.x)`.
        let nrm = [e[1] / len, -e[0] / len];
        let off = [a[0] + nrm[0] * pad, a[1] + nrm[1] * pad];
        let mut out: Vec<[f32; 2]> = Vec::with_capacity(cur.len() + 1);
        let side = |p: &[f32; 2]| (p[0] - off[0]) * nrm[0] + (p[1] - off[1]) * nrm[1];
        for j in 0..cur.len() {
            let (u, v) = (cur[j], cur[(j + 1) % cur.len()]);
            let (su, sv) = (side(&u), side(&v));
            if su <= 0.0 {
                out.push(u);
            }
            if (su <= 0.0) != (sv <= 0.0) {
                let t = su / (su - sv);
                out.push([u[0] + t * (v[0] - u[0]), u[1] + t * (v[1] - u[1])]);
            }
        }
        cur = out;
        if cur.is_empty() {
            break;
        }
    }
    cur
}

/// ⭐⭐⭐ **O CASCO CORTA MAIS QUE A CAIXA?** (W59) — a régua, antes da obra.
///
/// A nota da W56e deixou ⏸️: *"ladrilhar em `(u, v)` contra o **paralelogramo** em vez da AABB — o
/// único eixo que não multiplica a montagem de JIT"*. ⚠️ Isso é uma afirmação sobre o **preço**, e
/// não sobre o **ganho**: se o casco não cortar mais arestas do que a caixa, não há obra nenhuma a
/// fazer. *Esta linha já pagou quatro vezes por construir o que a nota prescrevia sem medir.*
///
/// ⭐ **O mecanismo em disputa:** o `dmax` do corte cresce com o **diâmetro** da região, e o
/// diâmetro de uma caixa é a diagonal dela. Um tubo de viés tem uma caixa muito maior que ele — mas
/// a **diagonal** de uma e o **comprimento** do outro podem ser parecidos. É isso que a tabela
/// resolve.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::the_table_of_whether_a_hull_culls_better_than_its_box --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_whether_a_hull_culls_better_than_its_box() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    println!("contorno | câmera | fatias | caixa | casco | ganho | área caixa/casco");
    for (name, ring) in [
        ("círculo 168", ngon_probe(168, 0.5)),
        ("estrela 168", star_probe(168, 0.22, 0.5)),
    ] {
        let profile = Profile::new(vec![ring], FillRule::NonZero, 1e-3).expect("perfil");
        let idx = ph2d_field_eval::profile_index::ProfileIndex::build(&profile);
        let doc = FieldDoc::new(
            vec![Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile,
                    half_height: 0.2,
                    round: 0.0,
                    chamfer: 0.0,
                }),
                mods: Vec::new(),
                verb: None,
            }],
            NodeId(0),
        )
        .expect("a peça");
        let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .map(ph2d_field_eval::bounds::Ball::aabb)
            .expect("a caixa");
        for (cn, cam) in [
            ("de viés", crate::Orbit::from_yaw_pitch(0.72, 0.52)),
            ("de frente", crate::Orbit::from_yaw_pitch(0.0, 0.0)),
            ("rasante", crate::Orbit::from_yaw_pitch(0.9, 0.15)),
        ] {
            let plane = crate::Screen::new(640, 480, cam.half_extent);
            let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
            for slabs in [crate::tiles::SLABS, 4] {
                let (mut box_k, mut hull_k, mut n) = (0usize, 0usize, 0usize);
                let (mut area_box, mut area_hull) = (0.0f64, 0.0f64);
                for ty in 0..480usize / 64 {
                    for tx in 0..640usize / 64 {
                        let (lo_px, hi_px) = ((tx * 64, ty * 64), (tx * 64 + 64, ty * 64 + 64));
                        let Some((t_lo, t_hi)) =
                            crate::tiles::tile_t_range(&cam, plane, lo_px, hi_px, bbox)
                        else {
                            continue;
                        };
                        let bounds = crate::tiles::slab_bounds(t_lo, t_hi, slabs);
                        // ⚠️ Só as fatias INTERIORES: as duas de fora são a cerca, e ninguém as
                        // monta a não ser num caso raro (ver `tiles::slab_bounds`).
                        for k in 1..bounds.len() - 2 {
                            let Some(r) = crate::tiles::slab_region(
                                &cam,
                                plane,
                                lo_px,
                                hi_px,
                                bbox,
                                sharp.normal,
                                &bounds,
                                k,
                            ) else {
                                continue;
                            };
                            // O casco em `(u, v)` = `(x, y)` dos 8 cantos do tubo desta fatia.
                            let (a0, a1) = (bounds[k], bounds[k + 1]);
                            let mut pts = Vec::with_capacity(8);
                            for (px, py) in [
                                (lo_px.0 as f32, lo_px.1 as f32),
                                (hi_px.0 as f32, lo_px.1 as f32),
                                (lo_px.0 as f32, hi_px.1 as f32),
                                (hi_px.0 as f32, hi_px.1 as f32),
                            ] {
                                let (sx, sy) = plane.plane_at(px, py);
                                let (o, d) = cam.ray_at_plane(sx, sy);
                                for t in [a0, a1] {
                                    pts.push([d[0].mul_add(t, o[0]), d[1].mul_add(t, o[1])]);
                                }
                            }
                            // ⚠️ **A MESMA região que a caixa descreve**: ∩ com a caixa da peça em
                            // `(u, v)`, e inflada pela mesma folga. ⛔ A 1.ª versão desta sonda
                            // comparava o casco CRU com a caixa recortada-e-inflada, e imprimiu
                            // «área do casco MAIOR que a da caixa» — impossível para um casco dentro
                            // da própria AABB, e o sinal de que ela media duas coisas diferentes.
                            let pad = sharp.normal * 4.0 + a1.abs() * 1.0e-3;
                            let hull = inflate(
                                &clip_to_rect(
                                    &hull_of(pts),
                                    [bbox.0[0], bbox.0[1]],
                                    [bbox.1[0], bbox.1[1]],
                                ),
                                pad,
                            );
                            if hull.len() < 3 {
                                continue;
                            }
                            n += 1;
                            box_k += idx.probe_cull([r.lo[0], r.lo[1]], [r.hi[0], r.hi[1]]);
                            hull_k += idx.probe_cull_hull(&hull);
                            area_box += f64::from((r.hi[0] - r.lo[0]) * (r.hi[1] - r.lo[1]));
                            let mut a = 0.0f64;
                            for i in 0..hull.len() {
                                let (p, q) = (hull[i], hull[(i + 1) % hull.len()]);
                                a += f64::from(p[0] * q[1] - q[0] * p[1]);
                            }
                            area_hull += a.abs() * 0.5;
                        }
                    }
                }
                let t = n.max(1) as f64;
                println!(
                    "{name:>11} | {cn:>9} | {slabs:>6} | {:>5.1} | {:>5.1} | {:>4.2}x | {:>4.2}x  ({n} regiões)",
                    box_k as f64 / t,
                    hull_k as f64 / t,
                    box_k as f64 / hull_k.max(1) as f64,
                    area_box / area_hull.max(1e-9),
                );
            }
        }
    }
}

/// ⛔⛔ **O CASCO CONTÉM TUDO O QUE A FITA DELE É PERGUNTADA** (W59) — o invariante da wave.
///
/// ⚠️ **É a mesma lei do [`every_sample_lies_inside_the_region_that_built_its_tape`], um nível mais
/// apertado.** Aquele mede a **caixa**; este mede o **polígono** que a substituiu no corte da
/// distância. Um casco apertado demais não fica lento — ele deita fora a aresta mais próxima, a
/// distância sai **grande demais**, e a marcha **atravessa a peça**.
///
/// ⭐ **A régua vai ao caminho do produto**: `region_between` dá os cantos, `ph2d_field_eval` deriva
/// o casco com a mesma função que a especialização usa, e o gate amostra uma grelha densa de raios
/// **interiores** — que são exactamente os que os quatro cantos não descrevem.
#[test]
fn the_hull_contains_every_ray_of_its_own_tile() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![ngon_probe(24, 0.5)], FillRule::NonZero, 1e-3)
                    .expect("perfil"),
                half_height: 0.2,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("a peça");
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("a caixa");
    for (name, cam) in [
        ("de viés", crate::Orbit::from_yaw_pitch(0.72, 0.52)),
        ("de frente", crate::Orbit::from_yaw_pitch(0.0, 0.0)),
        ("rasante", crate::Orbit::from_yaw_pitch(0.9, 0.15)),
    ] {
        let plane = crate::Screen::new(640, 480, cam.half_extent);
        let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
        // ⚠️ **Mais fatias do que o produto usa**, como no gate irmão: o casco aperta com a fatia, e
        // medir só no `SLABS` que shipa é medir o caso fácil.
        for slabs in [crate::tiles::SLABS, 4, 8] {
            let (mut worst, mut where_, mut measured) = (0i32, (0usize, 0usize, 0usize), 0usize);
            for ty in 0..480usize / 64 {
                for tx in 0..640usize / 64 {
                    let (lo_px, hi_px) = ((tx * 64, ty * 64), (tx * 64 + 64, ty * 64 + 64));
                    let Some((t_lo, t_hi)) =
                        crate::tiles::tile_t_range(&cam, plane, lo_px, hi_px, bbox)
                    else {
                        continue;
                    };
                    let bounds = crate::tiles::slab_bounds(t_lo, t_hi, slabs);
                    for k in 0..bounds.len() - 1 {
                        let Some(r) = crate::tiles::slab_region(
                            &cam,
                            plane,
                            lo_px,
                            hi_px,
                            bbox,
                            sharp.normal,
                            &bounds,
                            k,
                        ) else {
                            continue;
                        };
                        // ⚠️ **A pose é a identidade**, então a caixa de mundo é a local e os pontos
                        // passam directamente — é a mesma conta que `compile_at` faz com `Affine`.
                        let hull = ph2d_field_eval::probe_hull_uv(
                            &r.pts,
                            [r.lo[0], r.lo[1]],
                            [r.hi[0], r.hi[1]],
                        );
                        if hull.len() < 3 {
                            continue;
                        }
                        measured += 1;
                        let (a0, a1) = (bounds[k], bounds[k + 1]);
                        for ia in 0..9 {
                            for ib in 0..9 {
                                let px = lo_px.0 as f32 + 64.0 * ia as f32 / 8.0;
                                let py = lo_px.1 as f32 + 64.0 * ib as f32 / 8.0;
                                let (sx, sy) = plane.plane_at(px, py);
                                let (o, d) = cam.ray_at_plane(sx, sy);
                                let Some((ea, eb)) = crate::slab(o, d, bbox.0, bbox.1) else {
                                    continue;
                                };
                                let (s0, s1) = (ea.max(0.0).max(a0), eb.min(crate::T_MAX).min(a1));
                                if s1 <= s0 {
                                    continue;
                                }
                                for j in 0..9 {
                                    let t = s0 + (s1 - s0) * j as f32 / 8.0;
                                    let uv = [d[0].mul_add(t, o[0]), d[1].mul_add(t, o[1])];
                                    if !ph2d_field_eval::probe_in_hull(uv, &hull) {
                                        worst += 1;
                                        where_ = (tx, ty, k);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            assert!(measured > 20, "{name}: poucas regiões medidas ({measured})");
            assert_eq!(
                worst, 0,
                "{name}, {slabs} fatias: {worst} amostras caíram FORA do casco da fatia \
                 {where_:?} — o corte da distância deita fora a aresta mais próxima, a distância sai \
                 grande demais e a marcha atravessa a peça"
            );
        }
    }
}

/// ⭐⭐ **O CASCO CORTA MAIS QUE A CAIXA, E NUNCA MENOS** — a promessa de perf, gateada.
///
/// ⚠️ A **monotonia** dá a segunda metade de graça: o casco está **dentro** da própria caixa, e o
/// corte é monótono (região menor ⇒ `dmax` menor ⇒ menos arestas). Se alguma região guardasse MAIS
/// arestas com o casco, ou o casco não está dentro da caixa, ou a regra deixou de ser a mesma.
#[test]
fn the_hull_culls_strictly_better_than_its_box() {
    use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let profile =
        Profile::new(vec![ngon_probe(168, 0.5)], FillRule::NonZero, 1e-3).expect("perfil");
    let idx = ph2d_field_eval::profile_index::ProfileIndex::build(&profile);
    let doc = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile,
                half_height: 0.2,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("a peça");
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("a caixa");
    // ⚠️ **A câmera de VIÉS**, que é onde o tubo é de facto oblíquo — de frente o casco quase É a
    // caixa, e um gate ali mediria o caso em que a wave não faz nada.
    let cam = crate::Orbit::from_yaw_pitch(0.72, 0.52);
    let plane = crate::Screen::new(640, 480, cam.half_extent);
    let sharp = crate::Sharpness::for_frame(cam.half_extent, 480);
    let (mut boxed, mut hulled, mut n) = (0usize, 0usize, 0usize);
    for ty in 0..480usize / 64 {
        for tx in 0..640usize / 64 {
            let (lo_px, hi_px) = ((tx * 64, ty * 64), (tx * 64 + 64, ty * 64 + 64));
            let Some((t_lo, t_hi)) = crate::tiles::tile_t_range(&cam, plane, lo_px, hi_px, bbox)
            else {
                continue;
            };
            let bounds = crate::tiles::slab_bounds(t_lo, t_hi, crate::tiles::SLABS);
            for k in 1..bounds.len() - 2 {
                let Some(r) = crate::tiles::slab_region(
                    &cam,
                    plane,
                    lo_px,
                    hi_px,
                    bbox,
                    sharp.normal,
                    &bounds,
                    k,
                ) else {
                    continue;
                };
                let hull =
                    ph2d_field_eval::probe_hull_uv(&r.pts, [r.lo[0], r.lo[1]], [r.hi[0], r.hi[1]]);
                if hull.len() < 3 {
                    continue;
                }
                let b = idx.probe_cull([r.lo[0], r.lo[1]], [r.hi[0], r.hi[1]]);
                let h = idx.probe_cull_hull(&hull);
                assert!(
                    h <= b,
                    "a região ({tx}, {ty}, fatia {k}) guarda {h} arestas com o casco e {b} com a \
                     caixa — o casco tem de estar DENTRO dela, e o corte é monótono"
                );
                n += 1;
                boxed += b;
                hulled += h;
            }
        }
    }
    assert!(n > 20, "poucas regiões medidas ({n})");
    assert!(
        hulled * 100 <= boxed * 92,
        "o casco guardou {:.1} arestas contra {:.1} da caixa ({:.0}%) — ele deixou de apertar, e o \
         que sobra é o custo de o construir",
        hulled as f32 / n as f32,
        boxed as f32 / n as f32,
        hulled as f32 * 100.0 / boxed as f32
    );
}

/// ⭐⭐⭐ **QUANTO O ANTI-SERRILHADO CUSTA DE FACTO** — e ela existe porque a nota que o acusava
/// estava errada por **4×**.
///
/// # ⛔ A nota que esta sonda mata
///
/// O §55.3 do doc do módulo dizia *"o traçado ficou ~2,4× mais caro desde a W3 e ninguém o
/// reconferiu; o suspeito nomeado é o anti-serrilhado adaptativo"*. Medido aqui:
///
/// | arestas | s/AA | c/AA | a quota do AA |
/// |---|---|---|---|
/// | 64 | 29,0 ms | 36,4 ms | **26 %** |
/// | 128 | 46,3 ms | 60,9 ms | 32 % |
/// | 256 | 86,8 ms | 105,9 ms | 22 % |
/// | 512 | 171,3 ms | 229,6 ms | 34 % |
///
/// ⇒ o AA custa **22–34 %**, não os 140 % de que era acusado. E contra o número da W3 (`24,1 ms` a
/// 64 arestas, antes de o AA existir) o traçado de hoje **sem AA** está em `29,0 ms`: **1,2×**, não
/// `2,4×`. *A nota envelheceu porque as waves de perf seguintes (W56e, W56f, W59) moveram o número
/// e ninguém reconferiu a nota que elas desmentiam.*
///
/// # ⚠️ A RÉGUA era o defeito, e ela foi corrigida antes da resposta
///
/// A primeira leitura desta pergunta subtraiu **dois relógios de ~30 ms**, medidos em **corridas
/// separadas**, para ler um delta de ~10 ms — e devolveu `+34 %` numa e `+22 %` noutra sobre o
/// **mesmo** código. *Subtrair dois números ruidosos não dá um número menos ruidoso: dá a soma dos
/// dois ruídos.* A lição já estava escrita na porta irmã do passo (`trace_stepped_for_test`) e foi
/// paga outra vez. ⇒ as duas configurações correm no **mesmo processo**, `RUNS` vezes cada, e o que
/// se reporta é a **mediana**.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::measure_the_edge_pass_share --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_edge_pass_share() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    const RUNS: usize = 7;
    let reg = Registry::new();
    let cam = Orbit::default();
    let median = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("arestas | s/AA | c/AA | quota do AA | bordas");
    for n in [64_usize, 128, 256, 512] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-3).expect("perfil");
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");

        // ⚠️ **Uma corrida de aquecimento antes de medir**: a primeira paga a montagem a frio, e
        // incluí-la na mediana mede o cache e não o algoritmo.
        let _ = crate::trace_with(&doc, &reg, &cam, 640, 480, true, false);

        let mut off = Vec::new();
        let mut on = Vec::new();
        let mut edges = 0;
        for _ in 0..RUNS {
            let t = std::time::Instant::now();
            let _ = crate::trace_with(&doc, &reg, &cam, 640, 480, true, false);
            off.push(t.elapsed().as_secs_f64() * 1000.0);

            let t = std::time::Instant::now();
            let g = crate::trace_with(&doc, &reg, &cam, 640, 480, true, true);
            on.push(t.elapsed().as_secs_f64() * 1000.0);
            edges = g.edges.len();
        }
        let (o, a) = (median(off), median(on));
        println!(
            "{n:7} | {o:6.1} ms | {a:6.1} ms | {:5.0} % | {edges}",
            100.0 * (a - o) / o
        );
    }
}

/// ⭐⭐⭐ **QUANTAS ÁRVORES UM QUADRO ESPECIALIZA, e quanto isso pesa** — a sonda que decide a cura
/// do report do Enio (*"queda de fps e lentidão com resoluções altas"*).
///
/// # ⛔ A contagem tinha sido ADIVINHADA
///
/// Uma sonda anterior forçou `60` especializações (a contagem de **ladrilhos** a `D=6`) e leu
/// `245 ms`. O produto compila **preguiçosamente** — só as fatias que algum raio alcança — então
/// aquele número é um **tecto**, não o custo. *Uma sonda que assume a contagem mede a sua própria
/// suposição.* Aqui a contagem vem do [`crate::SPECIALISED`], que o produto incrementa.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::measure_how_many_trees_a_frame_specialises --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_how_many_trees_a_frame_specialises() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    println!("arestas | divisor | tamanho | árvores | ms | ms/árvore");
    for n in [168usize, 672] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");
        for d in [1u32, 3, 6] {
            let (w, h) = (1920 / d, 1080 / d);
            let _ = crate::trace(&doc, &reg, &cam, w, h);
            crate::SPECIALISED.store(0, Ordering::Relaxed);
            let t = std::time::Instant::now();
            let _ = crate::trace(&doc, &reg, &cam, w, h);
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            let trees = crate::SPECIALISED.load(Ordering::Relaxed);
            println!(
                "{n:7} | {d:7} | {w:4}x{h:4} | {trees:7} | {ms:8.1} | {:9.2}",
                ms / (trees.max(1)) as f64
            );
        }
    }
}

/// ⭐⭐⭐ **O LADRILHO CERTO PARA UMA IMAGEM PEQUENA** — a sonda que a cura do preview precisa.
///
/// # ⛔ O `TILE = 64` foi escolhido a RESOLUÇÃO CHEIA
///
/// Medido aqui: um traçado a `1920×1080` especializa **917** árvores e um a `640×360` especializa
/// **132**, a `0,33–0,54 ms` cada — ou seja **o traçado é quase só montagem**, e marchar os raios é
/// barato ao lado dela.
///
/// ⚠️ **A especialização paga-se por AMORTIZAÇÃO** (a mesma lei que a cadeia de quads mediu no mesmo
/// dia, noutro subsistema): o custo é por **região**, e o que o dilui são os **raios** que caem
/// nela. Numa imagem 9× menor há 9× menos raios por ladrilho a pagar a mesma montagem — e o `64`
/// nunca foi medido nesse regime.
///
/// ⚠️ **Ela usa a porta que já existe** ([`crate::trace_tiled_for_test`]), então mede o **produto**
/// com outro parâmetro, e não uma reconstrução dele.
///
/// ```text
/// cargo test -p ph2d-field-render --release -- --exact \
///     tests::measure_the_tile_that_fits_a_small_image --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_tile_that_fits_a_small_image() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    let contour: Vec<[f32; 2]> = (0..168)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 168.0;
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile,
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão");
    println!("tamanho | tile | slabs | árvores | ms");
    for d in [3u32] {
        let (w, h) = (1920 / d, 1080 / d);
        for tile in [16usize, 24, 32, 48, 64, 96, 128] {
            for slabs in [1usize, 2, 3, 4] {
                let _ =
                    crate::trace_tiled_for_test(&doc, &reg, &cam, w, h, tile, slabs, true, true);
                crate::SPECIALISED.store(0, Ordering::Relaxed);
                let mut runs: Vec<f64> = (0..5)
                    .map(|_| {
                        let t = std::time::Instant::now();
                        let _ = crate::trace_tiled_for_test(
                            &doc, &reg, &cam, w, h, tile, slabs, true, true,
                        );
                        t.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect();
                runs.sort_by(f64::total_cmp);
                println!(
                    "{w:4}x{h:4} | {tile:4} | {slabs:5} | {:7} | {:8.1}",
                    crate::SPECIALISED.load(Ordering::Relaxed) / 5,
                    runs[2]
                );
            }
        }
    }
}

/// ⭐⭐⭐ **O QUE O ORÇAMENTO DE FITAS COMPRA** — a sonda A/B da W70.
///
/// Ela mede o **quadro do produto** (`trace`, com anti-serrilhado, a `640×360` — o tamanho do
/// preview em movimento) e imprime, ao lado do relógio, as duas contagens que explicam o número:
/// quantas regiões foram especializadas e quantas **fitas** isso custou.
///
/// ⚠️ **O A/B faz-se trocando o código**, não um interruptor: as duas leis da W70 são ausências (a
/// fita de gradiente que não se monta, o `fork` que não se faz), e um interruptor para as ligar de
/// volta seria produto a carregar a versão lenta para sempre. Quem alterna é o arnês de mutação, e
/// a comparação é **intercalada** (A,B,A,B) para a deriva da máquina não se colar a um dos lados.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_what_the_tape_budget_buys --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_what_the_tape_budget_buys() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use ph2d_field_eval::hybrid::{FLOAT_TAPES, GRAD_TAPES};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    println!("tamanho | arestas | ms (mediana de 7) | regiões | fitas float | fitas grad");
    for (w, h) in [(640u32, 360u32), (1920, 1080)] {
        for n in [168usize, 672] {
            let contour: Vec<[f32; 2]> = (0..n)
                .map(|i| {
                    let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                    [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
                })
                .collect();
            let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
            let doc = FieldDoc::new(
                vec![ph2d_field_eval::leaf(
                    Primitive::Extrude {
                        profile,
                        half_height: 0.4,
                        round: 0.06,
                        chamfer: 0.0,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
            .expect("extrusão");
            let _ = crate::trace(&doc, &reg, &cam, w, h);
            SPECIALISED.store(0, Ordering::Relaxed);
            FLOAT_TAPES.store(0, Ordering::Relaxed);
            GRAD_TAPES.store(0, Ordering::Relaxed);
            let mut ms: Vec<f64> = (0..7)
                .map(|_| {
                    let t = std::time::Instant::now();
                    let _ = crate::trace(&doc, &reg, &cam, w, h);
                    t.elapsed().as_secs_f64() * 1000.0
                })
                .collect();
            ms.sort_by(f64::total_cmp);
            println!(
                "{w:4}x{h:4} | {n:7} | {:17.1} | {:7} | {:11} | {:10}",
                ms[3],
                SPECIALISED.load(Ordering::Relaxed) / 7,
                FLOAT_TAPES.load(Ordering::Relaxed) / 7,
                GRAD_TAPES.load(Ordering::Relaxed) / 7,
            );
        }
    }
}

/// ⭐⭐⭐ **PARA ONDE VAI O QUADRO, e quantas fatias ele quer** (W71) — as duas perguntas que a W70
/// deixou por responder, medidas juntas porque partilham a fixtura.
///
/// # 1. A fracção de MONTAGEM
///
/// ⛔ **A tabela do A/B da W70 admitia duas leituras que diferem por `3×`:** ela removeu `132` fitas
/// float **e** `293` de gradiente e ganhou `27,2 ms`. Dividir por `132` diz que a montagem que
/// sobra é `79 %` do quadro; dividir por `425` diz `25 %`. *Duas divisões da mesma medição não são
/// uma medição* — e elas mandam em waves opostas (cache entre quadros contra atacar a marcha).
///
/// ⚠️ **O traçado é SERIAL aqui**: o [`crate::SPECIALISE_NS`] soma tempo de **CPU**, e só contra um
/// relógio de parede serial é que essa soma é uma fracção.
///
/// # 2. Quantas FATIAS
///
/// ⚠️ **O `SLABS = 2` foi escolhido quando uma região custava o DOBRO** (a W70 tirou-lhe a fita de
/// gradiente e o `fork`). Repartir **divide** o custo de avaliar e **multiplica** o de montar — se
/// montar ficou metade do preço, o vale move-se para mais fatias. *Quem move o número que sustenta
/// uma nota tem de reconferir a nota.*
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_where_the_frame_goes_and_how_many_slabs_it_wants --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_where_the_frame_goes_and_how_many_slabs_it_wants() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    let piece = |n: usize| -> FieldDoc {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão")
    };
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };

    println!("== 1. onde o quadro serial se gasta (640x360, com AA) ==");
    println!("arestas | quadro | montagem | fracção | regiões");
    for n in [168usize, 672] {
        let doc = piece(n);
        let _ = crate::trace_with(&doc, &reg, &cam, 640, 360, false, true);
        SPECIALISED.store(0, Ordering::Relaxed);
        crate::SPECIALISE_NS.store(0, Ordering::Relaxed);
        let t = std::time::Instant::now();
        let _ = crate::trace_with(&doc, &reg, &cam, 640, 360, false, true);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let asm = crate::SPECIALISE_NS.load(Ordering::Relaxed) as f64 / 1.0e6;
        println!(
            "{n:7} | {ms:6.1} | {asm:8.1} | {:6.1}% | {:7}",
            100.0 * asm / ms,
            SPECIALISED.load(Ordering::Relaxed)
        );
    }

    println!("\n== 2. quantas fatias, intercalado (tile 64, mediana de 3 rondas x 5) ==");
    let slabs_set = [2usize, 3, 4, 6];
    for (w, h) in [(640u32, 360u32), (1920, 1080)] {
        for n in [168usize, 672] {
            let doc = piece(n);
            let mut best: Vec<Vec<f64>> = vec![Vec::new(); slabs_set.len()];
            for _ in 0..3 {
                for (k, &s) in slabs_set.iter().enumerate() {
                    let _ = crate::trace_tiled_for_test(&doc, &reg, &cam, w, h, 64, s, true, true);
                    let runs: Vec<f64> = (0..5)
                        .map(|_| {
                            let t = std::time::Instant::now();
                            let _ = crate::trace_tiled_for_test(
                                &doc, &reg, &cam, w, h, 64, s, true, true,
                            );
                            t.elapsed().as_secs_f64() * 1000.0
                        })
                        .collect();
                    best[k].push(med(runs));
                }
            }
            let cols: Vec<f64> = best.into_iter().map(med).collect();
            let win = cols
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| slabs_set[i])
                .unwrap_or(0);
            println!(
                "{w:4}x{h:4} arestas {n:4} | N=2 {:6.1} | N=3 {:6.1} | N=4 {:6.1} | N=6 {:6.1} | melhor {win}",
                cols[0], cols[1], cols[2], cols[3]
            );
        }
    }
}

/// ⭐⭐⭐ **QUE FORMA TEM A MARCHA** (W71) — a sonda que escolhe a wave seguinte.
///
/// A §72.1 mediu que a marcha é `80 %` do quadro. **Um raio que dá 8 passos e um que dá 40 pedem
/// curas opostas:** o primeiro é caro *por amostra* (a fita avalia devagar) e o segundo é caro *em
/// passos* (a lei da marcha aproxima-se devagar da superfície — e aí a saída publicada é a
/// **sobre-relaxação** da *Enhanced Sphere Tracing*).
///
/// ⚠️ Ela imprime também **quanto custa uma amostra**, que é a divisão que separa as duas leituras.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_the_shape_of_the_march --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_shape_of_the_march() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    println!("arestas | quadro | montagem | amostras | por pixel | ns/amostra | pixels de peça");
    for n in [168usize, 672] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");
        let (w, h) = (640u32, 360u32);
        let _ = crate::trace_with(&doc, &reg, &cam, w, h, false, true);
        crate::SPECIALISE_NS.store(0, Ordering::Relaxed);
        crate::STEP_SAMPLES.store(0, Ordering::Relaxed);
        let t = std::time::Instant::now();
        let g = crate::trace_with(&doc, &reg, &cam, w, h, false, true);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let asm = crate::SPECIALISE_NS.load(Ordering::Relaxed) as f64 / 1.0e6;
        let samples = crate::STEP_SAMPLES.load(Ordering::Relaxed) as f64;
        let pixels = f64::from(w) * f64::from(h);
        println!(
            "{n:7} | {ms:6.1} | {asm:8.1} | {samples:8.0} | {:9.1} | {:10.1} | {:14}",
            samples / pixels,
            (ms - asm) * 1.0e6 / samples,
            g.hits()
        );
    }
}

/// A distância com sinal de uma caixa afiada, escrita à mão — o oráculo dos dois gates do estêncil.
///
/// ⚠️ **De propósito não passa pelo avaliador**: o que se mede aqui é a lei do **estêncil**, e um
/// campo de JIT no meio poria a fita a responder por ela.
fn sharp_box_sd(half: [f32; 3], p: [f32; 3]) -> f32 {
    let q = [
        p[0].abs() - half[0],
        p[1].abs() - half[1],
        p[2].abs() - half[2],
    ];
    let out = [q[0].max(0.0), q[1].max(0.0), q[2].max(0.0)];
    out[0]
        .mul_add(out[0], out[1].mul_add(out[1], out[2] * out[2]))
        .sqrt()
        + q[0].max(q[1]).max(q[2]).min(0.0)
}

/// O gradiente que um estêncil lê de `sharp_box_sd` no ponto `p`, já unitário.
fn stencil_normal(s: crate::Stencil, half: [f32; 3], p: [f32; 3], e: f32) -> [f32; 3] {
    let mut g = [0.0f32; 3];
    for d in s.offsets() {
        let v = sharp_box_sd(
            half,
            [
                d[0].mul_add(e, p[0]),
                d[1].mul_add(e, p[1]),
                d[2].mul_add(e, p[2]),
            ],
        );
        for k in 0..3 {
            g[k] = d[k].mul_add(v, g[k]);
        }
    }
    let len = g[0]
        .mul_add(g[0], g[1].mul_add(g[1], g[2] * g[2]))
        .sqrt()
        .max(f32::MIN_POSITIVE);
    [g[0] / len, g[1] / len, g[2] / len]
}

/// ⭐⭐⭐ **A LEI QUE RECUSOU O ESTÊNCIL DE QUATRO** (W81) — e ela mede a **propriedade**, não a
/// constante.
///
/// Numa quina viva a normal verdadeira **não existe** (o gradiente salta de uma face para a outra),
/// e o que a imagem precisa ali é da **bissectriz** — a média das duas faces, que é o que faz a
/// aresta ler-se como uma linha e não como um degrau. A diferença central de seis amostras
/// devolve-a por **simetria**: cada eixo é sondado nos dois sentidos, e sobre a aresta os dois
/// sentidos pertencem a faces opostas.
///
/// ⛔ **O estêncil do tetraedro não tem essa simetria** — os quatro sentidos dele caem
/// desigualmente nas duas faces, e a normal inclina-se para a que apanhou mais. Medido aqui:
/// `24,9°` fora da bissectriz, e no traçado a sério **até `35,1°`** num cilindro afiado
/// (`measure_what_the_four_sample_normal_changes`).
///
/// ⚠️ **É por isso que este gate fica vermelho se alguém trocar o [`crate::NORMAL_STENCIL`]** — e o
/// nome dele diz porquê. *Uma constante guardada por um gate que mede a razão dela não é uma
/// tautologia: é a razão, executável.*
#[test]
fn the_shipping_stencil_reads_a_crease_as_the_bisector_of_its_two_faces() {
    let half = [0.5f32, 0.4, 0.45];
    // Exactamente sobre a aresta entre a face `+x` e a face `+y`.
    let p = [half[0], half[1], 0.0];
    let e = 1.0e-4f32;
    let bis = [0.5f32.sqrt(), 0.5f32.sqrt(), 0.0];
    let off = |s| -> f32 {
        let n = stencil_normal(s, half, p, e);
        n[0].mul_add(bis[0], n[1].mul_add(bis[1], n[2] * bis[2]))
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    };
    let shipped = off(crate::NORMAL_STENCIL);
    assert!(
        shipped < 0.5,
        "o estêncil que ship lê a quina a {shipped:.2}° da bissectriz — ver a recusa da W81 no doc \
         de `crate::Stencil` antes de mexer no `NORMAL_STENCIL`"
    );
    let four = off(crate::Stencil::Tetra4);
    assert!(
        four > 10.0,
        "o estêncil de quatro passou a ler a bissectriz ({four:.2}°) — se isto for verdade a recusa \
         da W81 dissolveu e a tabela dela tem de ser re-medida"
    );
}

/// ⭐⭐ **Numa superfície LISA os dois estênceis são o mesmo número** (W81) — a outra metade da
/// recusa, e sem ela o gate acima aceitaria um estêncil que erra em todo o lado.
///
/// ⚠️ *Uma afirmação de segurança precisa da metade justa*: «o de quatro erra na quina» só decide
/// alguma coisa ao lado de «e acerta fora dela». No meio de uma face a normal é uma função, e as
/// duas aproximações concordam a menos de `0,01°`.
#[test]
fn on_a_smooth_face_the_two_stencils_agree() {
    let half = [0.5f32, 0.4, 0.45];
    // No meio da face `+x`, longe de toda a aresta.
    let p = [half[0], 0.05, -0.07];
    let e = 1.0e-4f32;
    let a = stencil_normal(crate::Stencil::Central6, half, p, e);
    let b = stencil_normal(crate::Stencil::Tetra4, half, p, e);
    let ang = a[0]
        .mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees();
    assert!(
        ang < 0.01,
        "os dois estênceis divergem {ang:.4}° numa face lisa"
    );
}

/// ⭐⭐⭐ **O ESTÊNCIL NÃO MOVE A SILHUETA** (W81) — a cerca que separa as duas metades da marcha.
///
/// ⚠️ **A normal é lida DEPOIS de o raio parar**, e a única coisa que ela pode fazer é anular um
/// acerto cujo gradiente saiu nulo. Um estêncil que mudasse **onde** o raio pára seria um estêncil
/// dentro da marcha — e o sintoma seria a peça a mudar de tamanho ao trocar de estêncil, que
/// nenhuma tabela de ângulos veria.
#[test]
fn the_stencil_never_moves_the_silhouette() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let reg = Registry::new();
    let cam = Orbit::default();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.5, 0.4, 0.45],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("caixa");
    let a = crate::trace_stencil_for_test(&doc, &reg, &cam, 320, 180, crate::Stencil::Central6);
    let b = crate::trace_stencil_for_test(&doc, &reg, &cam, 320, 180, crate::Stencil::Tetra4);
    let diff = a.hit.iter().zip(&b.hit).filter(|(x, y)| **x != **y).count();
    assert_eq!(
        diff, 0,
        "{diff} pixels mudaram de acerto ao trocar o estêncil"
    );
    assert!(
        a.hits() > 1000,
        "a peça saiu vazia — a fixtura não mede nada"
    );
}

/// ⭐⭐⭐ **O QUE O ESTÊNCIL DE QUATRO MUDA NA IMAGEM** (W81) — a medição que decide a normal.
///
/// A normal é **`21 %`** de todas as amostras de campo de um quadro
/// (`measure_who_the_march_samples_belong_to`) e custa **seis** avaliações por pixel acertado. O
/// estêncil do tetraedro custa **quatro** — `1,5×` menos —, e a única pergunta que importa é *quanto
/// a imagem muda*.
///
/// ⚠️ **A régua é o ÂNGULO entre as duas normais, e ela é lida em dois grupos.** Numa quina viva a
/// normal verdadeira **não existe** (o gradiente salta), então uma diferença grande ali não é erro
/// de nenhum dos dois; o que decide é a superfície **lisa**, onde a normal é uma função e as duas
/// aproximações têm de concordar. ⇒ um pixel entra em «liso» quando os quatro vizinhos dele
/// concordam a menos de `25°` ([`crate::EDGE_COS`], a mesma cerca do anti-serrilhado).
///
/// ⚠️ Ângulos, não relógio: vale com a máquina sob carga.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_what_the_four_sample_normal_changes --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_what_the_four_sample_normal_changes() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    let disc = |n: usize| -> Primitive {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        Primitive::Extrude {
            profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
            half_height: 0.4,
            round: 0.06,
            chamfer: 0.0,
        }
    };
    let pieces: Vec<(&str, Primitive)> = vec![
        (
            "caixa afiada",
            Primitive::Box {
                half: [0.5, 0.4, 0.45],
                round: 0.0,
                chamfer: 0.0,
            },
        ),
        (
            "caixa com filete",
            Primitive::Box {
                half: [0.5, 0.4, 0.45],
                round: 0.08,
                chamfer: 0.0,
            },
        ),
        ("esfera", Primitive::Sphere { radius: 0.6 }),
        (
            "toro",
            Primitive::Torus {
                major: 0.5,
                minor: 0.18,
            },
        ),
        ("extrusão 168", disc(168)),
        (
            // ⚠️ A fronteira da recusa: um contorno de **quatro** arestas tem quinas verticais a
            // sério, e elas não vêm de `round` nenhum — *«as arestas verticais são o que o perfil
            // desenhou»* ([`ph2d_field::Primitive::Extrude`]).
            "extrusão quadrada",
            Primitive::Extrude {
                profile: Profile::new(
                    vec![vec![[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]],
                    FillRule::NonZero,
                    1e-4,
                )
                .expect("perfil"),
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
        ),
        (
            "cilindro afiado",
            Primitive::Cylinder {
                radius: 0.5,
                half_height: 0.4,
                round: 0.0,
                chamfer: 0.0,
            },
        ),
    ];
    let (w, h) = (640usize, 360usize);
    println!(
        "peça                | acertos | amostras 6 | amostras 4 | LISO: médio  p99    máx | SILHUETA:   n     p99     máx | VINCO:    n     p99     máx | >1° liso"
    );
    for (name, prim) in pieces {
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(prim, Xform::IDENTITY)],
            NodeId(0),
        )
        .expect("peça");
        let mut samples = [0u64; 2];
        let mut g = Vec::new();
        for (k, s) in [crate::Stencil::Central6, crate::Stencil::Tetra4]
            .into_iter()
            .enumerate()
        {
            crate::NORMAL_SAMPLES.store(0, Ordering::Relaxed);
            g.push(crate::trace_stencil_for_test(
                &doc, &reg, &cam, w as u32, h as u32, s,
            ));
            samples[k] = crate::NORMAL_SAMPLES.load(Ordering::Relaxed);
            // ⚠️ **A conta do contador contra a conta da imagem** — uma normal por acerto, `n`
            // amostras por normal. Sem isto o `21 %` seria uma divisão sem juiz.
            assert_eq!(
                samples[k],
                g[k].hits() as u64 * s.offsets().len() as u64,
                "{name}: o contador da normal e os acertos discordam"
            );
        }
        // ⚠️ **Só onde os dois acertaram** — a máscara é a mesma por construção (o estêncil só
        // toca a normal), e o gate `the_stencil_does_not_move_the_silhouette` prova-o.
        let ang = |a: [f32; 3], b: [f32; 3]| -> f64 {
            let d = f64::from(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]);
            d.clamp(-1.0, 1.0).acos().to_degrees()
        };
        // ⚠️ **TRÊS grupos, e não dois.** Um pixel de silhueta e um de vinco interior falham os dois
        // a mesma pergunta («os vizinhos concordam?») por razões opostas — ali o vizinho **não
        // existe**, aqui ele existe e discorda —, e a normal deles é consumida de maneiras
        // diferentes. Somá-los esconde qual dos dois carrega o número grande.
        let group = |i: usize| -> usize {
            let (x, y) = (i % w, i / w);
            if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
                return 1;
            }
            let viz = [i - 1, i + 1, i - w, i + w];
            if viz.iter().any(|&j| !g[0].hit[j]) {
                return 1; // silhueta
            }
            if viz.iter().all(|&j| {
                let (a, b) = (g[0].normal[i], g[0].normal[j]);
                a[0] * b[0] + a[1] * b[1] + a[2] * b[2] >= crate::EDGE_COS
            }) {
                0 // liso
            } else {
                2 // vinco interior
            }
        };
        let (mut lisos, mut quinas) = (Vec::new(), Vec::new());
        let mut silh: Vec<f64> = Vec::new();
        let mut hits = 0usize;
        for i in 0..w * h {
            if !(g[0].hit[i] && g[1].hit[i]) {
                continue;
            }
            hits += 1;
            let a = ang(g[0].normal[i], g[1].normal[i]);
            match group(i) {
                0 => lisos.push(a),
                1 => silh.push(a),
                _ => quinas.push(a),
            }
        }
        let pct = |v: &mut Vec<f64>, q: f64| -> f64 {
            if v.is_empty() {
                return 0.0;
            }
            v.sort_by(f64::total_cmp);
            v[((v.len() - 1) as f64 * q) as usize]
        };
        let mean = if lisos.is_empty() {
            0.0
        } else {
            lisos.iter().sum::<f64>() / lisos.len() as f64
        };
        let acima = lisos.iter().filter(|a| **a > 1.0).count();
        println!(
            "{name:19} | {hits:7} | {:10} | {:10} | {mean:6.3} {:6.3} {:6.3} | {:8} {:7.3} {:7.3} | {:6} {:7.3} {:7.3} | {acima:8}",
            samples[0],
            samples[1],
            pct(&mut lisos, 0.99),
            pct(&mut lisos, 1.0),
            silh.len(),
            pct(&mut silh, 0.99),
            pct(&mut silh, 1.0),
            quinas.len(),
            pct(&mut quinas, 0.99),
            pct(&mut quinas, 1.0),
        );
    }
}

/// Uma peça de perfil com `n` arestas — a família em que há o que especializar.
fn cache_piece(n: usize) -> ph2d_field::FieldDoc {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let contour: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão")
}

/// ⭐⭐⭐ **A CACHE NÃO MUDA A IMAGEM** (W82) — o gate que decide se ela pode shipar.
///
/// ⚠️ **E ela pode mudá-la de duas maneiras diferentes, e as duas são reais:** a fita guardada foi
/// construída para uma caixa **inflada** (guarda mais arestas) e para uma **caixa** em vez do casco
/// (guarda mais ainda). ⇒ a árvore especializada não é a mesma árvore.
///
/// ⭐ **O que salva a distância é a aritmética:** um `min` sobre um superconjunto de segmentos
/// escolhe **o mesmo** segmento, e a escolha de um mínimo entre `f32` é exacta. Já o **sinal** sai
/// de um enrolamento ancorado num canto da região — com outra caixa, outra âncora e outro caminho.
///
/// ⇒ *este gate não é uma formalidade: ele é a pergunta em aberto do desenho.* Ele mede um
/// **arrasto**, e não um quadro, porque é ao longo dele que as fitas envelhecem dentro da cache.
#[test]
fn the_cache_never_changes_the_image() {
    let reg = Registry::new();
    let doc = cache_piece(168);
    let cache = crate::TapeCache::new();
    let (w, h) = (200u32, 120u32);
    let pior = |a: &crate::Gbuffer, b: &crate::Gbuffer| -> (usize, f64) {
        let mut pix = 0usize;
        let mut ang = 0.0f64;
        for k in 0..(w * h) as usize {
            if a.hit[k] != b.hit[k] {
                pix += 1;
                continue;
            }
            if !a.hit[k] {
                continue;
            }
            let (x, y) = (a.normal[k], b.normal[k]);
            let d = f64::from(x[0] * y[0] + x[1] * y[1] + x[2] * y[2]);
            ang = ang.max(d.clamp(-1.0, 1.0).acos().to_degrees());
        }
        (pix, ang)
    };
    let (mut hits, mut cache_pix, mut cache_ang) = (0usize, 0usize, 0.0f64);
    let (mut ctrl_pix, mut ctrl_ang) = (0usize, 0.0f64);
    for i in 0..8 {
        let cam = Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + (i as f32) * 2.0f32.to_radians(), 0.52).rotation,
            ..Orbit::default()
        };
        let base = crate::trace_cached_for_test(&doc, &reg, &cam, w, h, true, None);
        let com = crate::trace_cached_for_test(&doc, &reg, &cam, w, h, true, Some(&cache));
        // ⭐⭐⭐ **O CONTROLO, e sem ele esta medição não decide nada.** A marcha por LINHA não
        // especializa nada; a por ladrilho especializa por região. As duas árvores são
        // algebricamente a mesma e diferem **no último bit** — *a soma e a raiz caem em ordens
        // diferentes* (a cerca já escrita em
        // `the_tiled_march_draws_the_same_image_as_the_row_march`). ⇒ o traçado **já tem** um
        // desacordo, e a pergunta desta wave não é *«a cache muda alguma coisa?»* mas *«a cache
        // muda MAIS do que a especialização já mudava?»*.
        //
        // ⚠️ E um ULP no campo é `~0,05°` na normal, porque ela sai de uma diferença central com
        // passo `1e-4` sobre um valor que ali é quase zero. *Uma barra absoluta aqui seria uma
        // afirmação sobre o escalonador do JIT.*
        let linha = crate::trace_by_rows_for_test(&doc, &reg, &cam, w, h);
        hits += base.hits();
        let (p, a) = pior(&base, &com);
        cache_pix += p;
        cache_ang = cache_ang.max(a);
        let (p, a) = pior(&base, &linha);
        ctrl_pix += p;
        ctrl_ang = ctrl_ang.max(a);
    }
    assert!(
        hits > 5_000,
        "o arrasto não desenhou a peça ({hits} acertos)"
    );
    // ⛔ **O controlo tem de estar CHEIO.** Se a especialização não discordasse da marcha por linha
    // em nada, a comparação abaixo aceitaria qualquer coisa — *um zero de «não mediu» e um de
    // «perfeito» são o mesmo byte*.
    assert!(
        ctrl_ang > 0.0,
        "o controlo não mediu desacordo nenhum — a barra abaixo não estaria a prender nada"
    );
    // ⭐⭐⭐ **A máscara é EXACTAMENTE a mesma** — mais duro que o controlo, que tolera falhas de
    // silhueta. *A cache não move a superfície um pixel.*
    assert_eq!(
        cache_pix, 0,
        "{cache_pix} pixels mudaram de acerto por causa da cache (o controlo mudou {ctrl_pix})"
    );
    // ⭐⭐⭐ **E a normal não se mexe mais do que a especialização já a mexia.**
    assert!(
        cache_ang <= ctrl_ang,
        "a cache mexeu a normal {cache_ang:.4}° e a especialização sozinha mexe {ctrl_ang:.4}° — a \
         cache passou a mudar mais que o último bit, e isso é uma fita servida onde ela não vale"
    );
}

/// ⭐⭐⭐ **UMA FITA NUNCA É SERVIDA A OUTRO DOCUMENTO** (W82).
///
/// ⚠️ **É o pior modo de falha que uma cache destas tem**, e não é «uma imagem errada»: é uma imagem
/// **quase certa**. Uma fita da peça de ontem devolve uma distância plausível para a peça de hoje, e
/// o artista vê uma forma que quase responde ao controle que ele acabou de mexer.
///
/// ⚠️ **O nome deste gate era `the_cache_dies_with_the_document`, e o desenho desmentiu-o:** a cache
/// guarda [`crate::TapeCache`] **dois** documentos ao mesmo tempo (o contorno grosso e o cheio, que
/// o preview alterna) e nenhum deles morre — o que a lei diz é que uma fita só é servida a **quem a
/// construiu**. *Um nome que descreve o mecanismo envelhece com ele; um que descreve a lei não.*
///
/// ⚠️ O gate mede-o **na imagem**, e não no contador: um contador a zero prova que a cache esvaziou,
/// não que o que ela serviu estava certo.
#[test]
fn a_cached_tape_is_never_served_to_another_document() {
    let reg = Registry::new();
    let cache = crate::TapeCache::new();
    let cam = Orbit::default();
    let (w, h) = (200u32, 120u32);
    let a = cache_piece(168);
    let b = cache_piece(24);
    // Enche a cache com a peça `a`…
    let _ = crate::trace_cached_for_test(&a, &reg, &cam, w, h, true, Some(&cache));
    assert!(!cache.is_empty(), "a cache não guardou nada com a peça A");
    // …e pede a peça `b` pela MESMA cache.
    let com = crate::trace_cached_for_test(&b, &reg, &cam, w, h, true, Some(&cache));
    let sem = crate::trace_cached_for_test(&b, &reg, &cam, w, h, true, None);
    assert!(sem.hits() > 2_000, "a peça B não desenhou nada");
    let dif = (0..(w * h) as usize)
        .filter(|&k| com.hit[k] != sem.hit[k])
        .count();
    assert_eq!(
        dif, 0,
        "{dif} pixels da peça B saíram de fitas da peça A — a cache não morreu com o documento"
    );
}

/// ⛔⛔ **O DECIMADOR DO PREVIEW APAGA QUINAS?** (W84) — a sonda que o doc dele pede.
///
/// O [`ph2d_field::coarsen`] tira **um em cada `k`** vértices, e o doc justifica-o: *«um contorno
/// achatado por tolerância tem os pontos densos onde a curvatura é alta — então tirar um em cada `k`
/// preserva o carácter da forma»*. ⭐ Isso é **verdade para curvatura**, que é distribuída.
///
/// ⚠️ **Uma QUINA não é curvatura distribuída: é um vértice só, com todo o ângulo dentro.** Tirar um
/// em cada `k` apaga-a com probabilidade `(k−1)/k`, e o que fica no lugar é um bisel. A guarda
/// `c.len() <= 8` salva um quadrado simples; ela **não** salva uma forma que tenha quinas **e**
/// curvas — que é a forma que um artista desenha.
///
/// A sonda mede a **imagem** de uma estrela: quinas a sério, e amostras densas nas arestas.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_whether_the_preview_decimation_eats_corners --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_whether_the_preview_decimation_eats_corners() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let reg = Registry::new();
    let cam = Orbit::default();
    // Uma estrela de 5 pontas, com cada aresta amostrada em `POR_ARESTA` pontos: as quinas caem em
    // múltiplos exactos, que é o que uma polilinha achatada de um desenho faz.
    const PONTAS: usize = 5;
    const POR_ARESTA: usize = 40;
    let mut estrela: Vec<[f32; 2]> = Vec::new();
    for i in 0..PONTAS * 2 {
        let a0 = std::f64::consts::TAU * (i as f64) / (PONTAS * 2) as f64;
        let a1 = std::f64::consts::TAU * ((i + 1) as f64) / (PONTAS * 2) as f64;
        let (r0, r1) = if i % 2 == 0 {
            (0.65, 0.28)
        } else {
            (0.28, 0.65)
        };
        for s in 0..POR_ARESTA {
            let t = s as f64 / POR_ARESTA as f64;
            let (x0, y0) = (r0 * a0.cos(), r0 * a0.sin());
            let (x1, y1) = (r1 * a1.cos(), r1 * a1.sin());
            estrela.push([(x0 + (x1 - x0) * t) as f32, (y0 + (y1 - y0) * t) as f32]);
        }
    }
    let cheio = Profile::new(vec![estrela], FillRule::NonZero, 1e-4).expect("estrela");
    let build = |p: Profile| -> FieldDoc {
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile: p,
                    half_height: 0.4,
                    round: 0.02,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão")
    };
    let (w, h) = (640u32, 360u32);
    let base = crate::trace_with(&build(cheio.clone()), &reg, &cam, w, h, true, false);
    println!(
        "a estrela tem {} pontos e {} quinas",
        cheio.segment_count(),
        PONTAS * 2
    );
    println!("teto | arestas depois | pixels que mudam | % da peça | normal p99 | normal máx");
    for n in [336usize, 168, 84] {
        let fina = ph2d_field::coarsen(&cheio, n);
        let g = crate::trace_with(&build(fina.clone()), &reg, &cam, w, h, true, false);
        let mut mudou = 0usize;
        let mut angs: Vec<f64> = Vec::new();
        for k in 0..(w * h) as usize {
            if base.hit[k] != g.hit[k] {
                mudou += 1;
                continue;
            }
            if !base.hit[k] {
                continue;
            }
            let (a, b) = (base.normal[k], g.normal[k]);
            let d = f64::from(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]);
            angs.push(d.clamp(-1.0, 1.0).acos().to_degrees());
        }
        angs.sort_by(f64::total_cmp);
        println!(
            "{n:4} | {:14} | {mudou:16} | {:8.3}% | {:10.3} | {:10.3}",
            fina.segment_count(),
            100.0 * mudou as f64 / base.hits() as f64,
            angs[(angs.len() - 1) * 99 / 100],
            angs[angs.len() - 1],
        );
    }
}

/// ⭐⭐⭐ **QUANTAS ARESTAS DO CONTORNO SE VÊEM?** (W84) — o que o `Resolution` alto de facto compra.
///
/// O Enio subiu o `Resolution` (a pedido meu) e a peça ficou muito mais lenta. **Está certo que
/// fique**: o custo de uma amostra é *linear nas arestas do contorno*, e a §84.4 mediu `3,39×` entre
/// `168` e `672` no assentar. ⚠️ **A pergunta que ninguém fez é se aquelas arestas se VÊEM.**
///
/// ⭐ **A lei que este módulo já usa é a mesma**: a [`crate::Sharpness`] deriva as tolerâncias do
/// **tamanho do pixel em mundo**, e não de uma constante. A resolução do contorno é a única grandeza
/// do traçado que ainda não o faz — ela vem do knob do artista, que é sobre a **peça exportada**.
///
/// A sonda mede a **imagem**: quantos pixels mudam de acerto, e quanto a normal se mexe, entre o
/// contorno cheio e um decimado.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_how_many_contour_edges_are_visible --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_how_many_contour_edges_are_visible() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let reg = Registry::new();
    let cam = Orbit::default();
    let build = |p: Profile| -> FieldDoc {
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile: p,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão")
    };
    // ⚠️ O contorno de REFERÊNCIA é bem mais fino do que qualquer candidato: comparar dois
    // decimados mediria a diferença entre eles, e não o erro contra a curva.
    let fino: Vec<[f32; 2]> = (0..2048)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / 2048.0;
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    let referencia = Profile::new(vec![fino], FillRule::NonZero, 1e-6).expect("perfil");
    // ⭐⭐⭐ **A régua final é o PIXEL, e não a normal.** Um erro de normal só importa se ele mover
    // a cor: o matcap é uma bola suave, e um grau de desvio anda um cento e oitenta avos dela.
    // *Uma diferença geométrica que nenhum pixel mostra é trabalho que ninguém vê.*
    let matcap: Vec<f32> = {
        let side = 64usize;
        let mut v = vec![0.0f32; side * side * 3];
        for y in 0..side {
            for x in 0..side {
                // Uma bola difusa com um realce — o molde de um matcap de modelagem.
                let (u, w) = (
                    (x as f32 + 0.5) / side as f32 * 2.0 - 1.0,
                    (y as f32 + 0.5) / side as f32 * 2.0 - 1.0,
                );
                let r2 = u * u + w * w;
                let z = (1.0 - r2).max(0.0).sqrt();
                let l = (0.35 * u - 0.5 * w + 0.8 * z).max(0.0);
                let c = 0.12 + 0.75 * l + 0.6 * l.powi(24);
                for k in 0..3 {
                    v[(y * side + x) * 3 + k] = c;
                }
            }
        }
        v
    };
    let mc = crate::Matcap {
        side: 64,
        rgb_linear: &matcap,
    };
    const FUNDO: [u8; 4] = [30, 30, 34, 255];
    println!(
        "tamanho | arestas | pixels que mudam | % da peça | normal p99 | normal máx | PIXEL p99 | PIXEL máx"
    );
    for (w, h) in [(640u32, 360u32), (1600, 900)] {
        let base = crate::trace_with(&build(referencia.clone()), &reg, &cam, w, h, true, false);
        let base_px = crate::shade(&base, &mc, FUNDO);
        for n in [672usize, 336, 168, 84, 42, 24] {
            let doc = build(ph2d_field::coarsen(&referencia, n));
            let g = crate::trace_with(&doc, &reg, &cam, w, h, true, false);
            let mut mudou = 0usize;
            let mut angs: Vec<f64> = Vec::new();
            for k in 0..(w * h) as usize {
                if base.hit[k] != g.hit[k] {
                    mudou += 1;
                    continue;
                }
                if !base.hit[k] {
                    continue;
                }
                let (a, b) = (base.normal[k], g.normal[k]);
                let d = f64::from(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]);
                angs.push(d.clamp(-1.0, 1.0).acos().to_degrees());
            }
            angs.sort_by(f64::total_cmp);
            let p99 = angs[(angs.len() - 1) * 99 / 100];
            // O maior desvio de canal, em níveis de 8 bits, sobre os pixels da peça.
            let px = crate::shade(&g, &mc, FUNDO);
            let mut dif: Vec<u32> = Vec::new();
            for k in 0..(w * h) as usize {
                if !(base.hit[k] && g.hit[k]) {
                    continue;
                }
                let d = (0..3)
                    .map(|c| u32::from(base_px[k * 4 + c].abs_diff(px[k * 4 + c])))
                    .max()
                    .unwrap_or(0);
                dif.push(d);
            }
            dif.sort_unstable();
            println!(
                "{w:4}x{h:4} | {n:7} | {mudou:16} | {:8.3}% | {p99:10.3} | {:10.3} | {:9} | {:9}",
                100.0 * mudou as f64 / base.hits() as f64,
                angs[angs.len() - 1],
                dif[(dif.len() - 1) * 99 / 100],
                dif[dif.len() - 1],
            );
        }
    }
}

/// ⛔⛔⛔ **A CACHE FICA MAIS LENTA À MEDIDA QUE ENCHE?** (W84) — o report *«piorou muito»*.
///
/// A consulta da [`crate::TapeCache`] é uma **varredura LINEAR** sobre tudo o que ela guarda, e o
/// número de consultas de um quadro é uma por ladrilho por fatia. ⇒ o custo dela é
/// `consultas × entradas`, e as **duas** crescem: as consultas com o tamanho da imagem, as entradas
/// com cada quadro que passa.
///
/// ⚠️ **A bancada não podia ver isto**: ela mede 12 quadros e a cache pára nas `~1 200` entradas. Uma
/// sessão a sério enche-a até ao tecto (`CAPACITY`), e o artista vê a coisa **piorar com o tempo**.
///
/// A sonda mede o quadro **em função do que a cache já guarda**.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_whether_the_cache_gets_slower_as_it_fills --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_whether_the_cache_gets_slower_as_it_fills() {
    let reg = Registry::new();
    let doc = cache_piece(168);
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    // ⚠️ **Dois tamanhos**: o do preview (poucas consultas) e um cheio (muitas). O custo da
    // varredura é `consultas × entradas`, então ele tem de aparecer **muito mais** no segundo.
    for (w, h) in [(640u32, 360u32), (1600, 900)] {
        let cache = crate::TapeCache::new();
        println!("--- {w}x{h} ---");
        println!("quadros já dados | entradas na cache | ms do quadro");
        let mut passo = 0usize;
        for bloco in 0..8 {
            // Enche mais um bocado…
            for _ in 0..12 {
                let cam = Orbit {
                    rotation: Orbit::from_yaw_pitch(
                        0.72 + (passo as f32) * 2.0f32.to_radians(),
                        0.52,
                    )
                    .rotation,
                    ..Orbit::default()
                };
                passo += 1;
                let _ = crate::trace_cached_for_test(&doc, &reg, &cam, w, h, false, Some(&cache));
            }
            // …e mede um quadro no MESMO sítio, cinco vezes.
            let cam = Orbit {
                rotation: Orbit::from_yaw_pitch(0.72 + (passo as f32) * 2.0f32.to_radians(), 0.52)
                    .rotation,
                ..Orbit::default()
            };
            let _ = crate::trace_cached_for_test(&doc, &reg, &cam, w, h, false, Some(&cache));
            let ms = med((0..5)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let _ =
                        crate::trace_cached_for_test(&doc, &reg, &cam, w, h, false, Some(&cache));
                    t0.elapsed().as_secs_f64() * 1000.0
                })
                .collect());
            println!("{:16} | {:17} | {ms:12.2}", (bloco + 1) * 12, cache.len());
        }
        // E o controlo: o MESMO quadro sem cache nenhuma.
        let cam = Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + (passo as f32) * 2.0f32.to_radians(), 0.52)
                .rotation,
            ..Orbit::default()
        };
        let _ = crate::trace_cached_for_test(&doc, &reg, &cam, w, h, false, None);
        let sem = med((0..5)
            .map(|_| {
                let t0 = std::time::Instant::now();
                let _ = crate::trace_cached_for_test(&doc, &reg, &cam, w, h, false, None);
                t0.elapsed().as_secs_f64() * 1000.0
            })
            .collect());
        println!("{:16} | {:17} | {sem:12.2}   <- SEM CACHE", "—", "—");
    }
}

/// ⭐⭐⭐ **O ASSENTAR DE UMA PEÇA NA RESOLUÇÃO NORMAL** (W83) — e ele é outro problema.
///
/// ⚠️ **A `measure_the_stop_and_go_cycle_the_app_really_does` mediu a peça de resolução ALTA**, onde
/// o `coarse_doc` de facto engrossa o contorno e o app alterna dois documentos. ⛔ **Numa peça na
/// resolução de omissão isso não acontece:** o `PREVIEW_MAX_EDGES` é `168`, que é *exactamente* o
/// que o contorno já tem, então `coarse_doc` devolve `None` e **o documento é o mesmo o tempo todo**.
///
/// ⇒ para essa peça — a que o artista tem por omissão — o assentar difere do movimento por **duas**
/// coisas, e só duas: o **anti-serrilhado** (1.º degrau) e o **tamanho cheio** (2.º).
///
/// ⭐⭐⭐ **E a pergunta que decide a wave é de CONTAGEM:** o 2.º degrau corre numa grelha de
/// ladrilhos **mais fina**, e o tubo de um ladrilho pequeno está **dentro** do tubo do ladrilho
/// grande que o cobria. ⇒ *as fitas do movimento deviam servir o assentar sem uma única
/// recompilação*, e se não servem há uma razão que vale a wave.
///
/// ⚠️ Contagens — vale com a máquina sob carga.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_the_settle_of_a_default_resolution_piece --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_settle_of_a_default_resolution_piece() {
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    // ⚠️ **Um documento SÓ** — é o que o app tem quando o contorno já cabe no `PREVIEW_MAX_EDGES`.
    let doc = cache_piece(168);
    let cache = crate::TapeCache::new();
    let mut passo = 0usize;
    let mut quadro = |rot: bool, w: u32, h: u32, aa: bool, nome: &str| {
        ph2d_field_eval::hybrid::FLOAT_TAPES.store(0, Ordering::Relaxed);
        crate::TAPE_HITS.store(0, Ordering::Relaxed);
        // ⚠️ **O assentar corre na câmera do ÚLTIMO quadro de movimento**, e não na seguinte — ele
        // acontece precisamente porque a câmera parou. A 1.ª versão desta sonda avançava-a, e por
        // isso media um assentar que o app nunca faz. *Uma sonda que muda uma variável a mais mede
        // outra coisa.*
        if rot {
            passo += 1;
        }
        let cam = Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + (passo as f32) * 2.0f32.to_radians(), 0.52)
                .rotation,
            ..Orbit::default()
        };
        let g = crate::trace_cached_for_test(&doc, &reg, &cam, w, h, aa, Some(&cache));
        println!(
            "{nome} | {w:4}x{h:4} | AA {} | fitas {:4} | acertos {:5} | bordas {:6}",
            u8::from(aa),
            ph2d_field_eval::hybrid::FLOAT_TAPES.load(Ordering::Relaxed),
            crate::TAPE_HITS.load(Ordering::Relaxed),
            g.edges.len(),
        );
    };
    println!("o que o app faz         |    tamanho | AA | fitas | acertos | bordas");
    for i in 0..4 {
        quadro(true, 640, 360, false, &format!("gira {i}                 "));
    }
    quadro(false, 640, 360, true, "DEGRAU 1 (mesmo tamanho)");
    quadro(false, 1280, 720, true, "DEGRAU 2 (tamanho cheio)");
    println!("--- e uma segunda volta, com a cache já cheia ---");
    for i in 0..2 {
        quadro(true, 640, 360, false, &format!("gira {i}                 "));
    }
    quadro(false, 640, 360, true, "DEGRAU 1 (mesmo tamanho)");
    quadro(false, 1280, 720, true, "DEGRAU 2 (tamanho cheio)");
}

/// ⭐⭐⭐ **ONDE O ASSENTAR SE GASTA** (W83) — a sonda que escolhe a wave, depois do smoke do Enio.
///
/// A §83.10.2 mediu que o ciclo do artista **não** é dominado pelos quadros a girar (`13`–`23 ms`)
/// mas pelos dois degraus do **assentar** (`52`–`102 ms` cada). O assentar difere de um quadro de
/// movimento por **três** coisas multiplicadas, e uma sonda que não as separa não escolhe nada:
///
/// | factor | quadro de movimento | assentar |
/// |---|---|---|
/// | contorno | grosso (`PREVIEW_MAX_EDGES = 168`) | **cheio** |
/// | anti-serrilhado | desligado | **ligado** |
/// | tamanho | o do preview | o do preview (1.º degrau) · **cheio** (2.º) |
///
/// ⚠️ E há uma quarta pergunta que só esta sonda responde: **a cache serve o assentar?** O 1.º
/// degrau corre no mesmo tamanho (regiões parecidas, documento outro); o 2.º corre no tamanho
/// **cheio**, e um tamanho diferente é uma grelha de ladrilhos diferente ⇒ **outras regiões**.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_where_the_settle_goes --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_where_the_settle_goes() {
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let grosso = cache_piece(168);
    let cheio = cache_piece(672);
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    // ⚠️ Os dois tamanhos que o app usa: o do preview (um divisor do cheio) e o cheio.
    let casos: Vec<(&str, &ph2d_field::FieldDoc, u32, u32, bool)> = vec![
        ("movimento          ", &grosso, 640, 360, false),
        ("+ contorno cheio   ", &cheio, 640, 360, false),
        ("+ anti-serrilhado  ", &grosso, 640, 360, true),
        ("DEGRAU 1 (os dois) ", &cheio, 640, 360, true),
        ("DEGRAU 2 (+tamanho)", &cheio, 1280, 720, true),
    ];
    println!("o que corre         | sem cache | com cache | ganho | fitas | acertos");
    for (nome, doc, w, h, aa) in casos {
        let mut ms: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        let mut conta = (0usize, 0usize);
        let caches = [crate::TapeCache::new(), crate::TapeCache::new()];
        // ⚠️ **A cache é AQUECIDA com o ciclo inteiro**, e não com o caso a medir: é assim que ela
        // chega ao assentar no app — com as fitas do movimento lá dentro e o documento a alternar.
        for (k, c) in caches.iter().enumerate() {
            let cc = (k == 1).then_some(c);
            for i in 0..6 {
                let cam = Orbit {
                    rotation: Orbit::from_yaw_pitch(0.72 + (i as f32) * 2.0f32.to_radians(), 0.52)
                        .rotation,
                    ..Orbit::default()
                };
                let _ = crate::trace_cached_for_test(&grosso, &reg, &cam, 640, 360, false, cc);
            }
        }
        for _ in 0..3 {
            for (k, c) in caches.iter().enumerate() {
                let cc = (k == 1).then_some(c);
                let cam = Orbit::default();
                let _ = crate::trace_cached_for_test(doc, &reg, &cam, w, h, aa, cc);
                ph2d_field_eval::hybrid::FLOAT_TAPES.store(0, Ordering::Relaxed);
                crate::TAPE_HITS.store(0, Ordering::Relaxed);
                let t0 = std::time::Instant::now();
                let _ = crate::trace_cached_for_test(doc, &reg, &cam, w, h, aa, cc);
                ms[k].push(t0.elapsed().as_secs_f64() * 1000.0);
                if k == 1 {
                    conta = (
                        ph2d_field_eval::hybrid::FLOAT_TAPES.load(Ordering::Relaxed),
                        crate::TAPE_HITS.load(Ordering::Relaxed),
                    );
                }
            }
        }
        let (sem, com) = (med(ms[0].clone()), med(ms[1].clone()));
        println!(
            "{nome} | {sem:9.1} | {com:9.1} | {:5.2}x | {:5} | {:7}",
            sem / com,
            conta.0,
            conta.1
        );
    }
}

/// ⭐⭐⭐ **O CICLO A SÉRIO DO APP: gira, pára, gira** (W82) — e a bancada não o media.
///
/// ⚠️ **A `measure_what_the_tape_cache_buys` mede um arrasto CONTÍNUO com um documento só.** O app
/// não faz isso: enquanto a mão mexe ele traça o contorno **grosso**
/// (`field3d_preview::coarse_doc`), e ao parar ele traça o **cheio** — dois documentos que se
/// alternam. E a cache morre com o documento, de propósito (a fita da peça de ontem responde um
/// número plausível e errado).
///
/// ⇒ *cada paragem deita a cache fora inteira*, e o quadro seguinte à retoma volta a compilar tudo.
/// **Uma bancada que mede o caso contínuo não pode ver isto.**
///
/// ⚠️ Contagens — vale com a máquina sob carga.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_the_stop_and_go_cycle_the_app_really_does --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_stop_and_go_cycle_the_app_really_does() {
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let grosso = cache_piece(168);
    // O documento CHEIO é outro objecto — é o que o `coarse_doc` devolve ao contrário.
    let cheio = cache_piece(672);
    let (w, h) = (320u32, 180u32);
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    // ⭐ **Um CICLO** = quatro quadros a girar (contorno grosso) + os dois degraus do assentar
    // (contorno cheio), que é a escada da W73. É o que a mão do artista faz.
    let ciclo =
        |cache: Option<&crate::TapeCache>, de: usize, detalhe: bool| -> (f64, usize, usize) {
            ph2d_field_eval::hybrid::FLOAT_TAPES.store(0, Ordering::Relaxed);
            crate::TAPE_HITS.store(0, Ordering::Relaxed);
            let mut ms = 0.0f64;
            let mut passo = de;
            for volta in 0..3 {
                for fase in 0..6 {
                    let doc = if fase < 4 { &grosso } else { &cheio };
                    let cam = Orbit {
                        rotation: Orbit::from_yaw_pitch(
                            0.72 + (passo as f32) * 2.0f32.to_radians(),
                            0.52,
                        )
                        .rotation,
                        ..Orbit::default()
                    };
                    if fase < 4 {
                        passo += 1;
                    }
                    let t0 = std::time::Instant::now();
                    let _ =
                        crate::trace_cached_for_test(&doc.clone(), &reg, &cam, w, h, false, cache);
                    let el = t0.elapsed().as_secs_f64() * 1000.0;
                    ms += el;
                    if detalhe {
                        println!(
                            "  volta {volta} · {} {fase} | {el:7.2} ms | fitas {:4} | acertos {:4}",
                            if fase < 4 { "gira " } else { "pára " },
                            ph2d_field_eval::hybrid::FLOAT_TAPES.swap(0, Ordering::Relaxed),
                            crate::TAPE_HITS.swap(0, Ordering::Relaxed),
                        );
                    }
                }
            }
            (
                ms,
                ph2d_field_eval::hybrid::FLOAT_TAPES.load(Ordering::Relaxed),
                crate::TAPE_HITS.load(Ordering::Relaxed),
            )
        };
    println!("=== o ciclo em detalhe, COM cache ===");
    let cache = crate::TapeCache::new();
    let _ = ciclo(Some(&cache), 0, false);
    let _ = ciclo(Some(&cache), 100, true);

    println!("\n=== A/B intercalado: 18 quadros por ciclo (4 a girar + 2 a parar) x 3 ===");
    let caches = [crate::TapeCache::new(), crate::TapeCache::new()];
    for (k, c) in caches.iter().enumerate() {
        let _ = ciclo((k == 1).then_some(c), 0, false);
    }
    let mut tot: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    for ronda in 0..5usize {
        for (k, c) in caches.iter().enumerate() {
            let (ms, _, _) = ciclo((k == 1).then_some(c), 1000 * (ronda + 1), false);
            tot[k].push(ms);
        }
    }
    let (sem, com) = (med(tot[0].clone()), med(tot[1].clone()));
    println!(
        "sem cache: {sem:7.2} ms o ciclo  ·  com cache: {com:7.2} ms  ·  ganho {:.2}x",
        sem / com
    );
}

/// ⭐⭐⭐ **O QUE A CACHE DE FITAS COMPRA, NUM ARRASTO A SÉRIO** (W82) — o A/B que a decide.
///
/// A §82.12 mediu a contenção **região a região**; esta mede o **arrasto**: `N` quadros seguidos com
/// a câmera a rodar `g` graus por quadro, com e sem cache, no mesmo processo e intercalados.
///
/// ⚠️ **O 1.º quadro de cada arrasto é descartado** — ele enche a cache do zero e não representa
/// nenhum quadro que o artista veja depois do primeiro. O que se compara é o **regime**.
///
/// ⚠️ Precisa da máquina a `load < 5` para o ms; as colunas de **contagem** (fitas compiladas,
/// acertos) valem sempre.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_what_the_tape_cache_buys --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_what_the_tape_cache_buys() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let piece = |n: usize| -> FieldDoc {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão")
    };
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let (w, h) = (640u32, 360u32);
    // Um arrasto: 12 quadros, e a 1.ª é deitada fora.
    let quadros = 12usize;
    println!(
        "arestas | graus/quadro |      f | cache | ms/quadro | fitas compiladas | acertos | guardadas"
    );
    for n in [168usize, 672] {
        let doc = piece(n);
        for (graus, fator) in [
            (1.0f32, 1.0f32),
            (1.0, 1.1),
            (1.0, 1.25),
            (1.0, 1.5),
            (2.0, 1.0),
            (2.0, 1.1),
            (2.0, 1.25),
            (2.0, 1.5),
            (4.0, 1.25),
            (4.0, 1.5),
            (4.0, 2.0),
        ] {
            // ⚠️ **INTERCALADO**: as duas metades do A/B alternam ronda a ronda, e não uma inteira
            // depois da outra. Entre duas corridas desta workstation o mesmo passe já deu `11,36` e
            // `5,50 ms`; medir `sem` durante três segundos e `COM` nos três seguintes mede a
            // máquina, não a cache.
            let mut ms: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
            let mut contagem: [(usize, usize, usize); 2] = [(0, 0, 0); 2];
            let arrasto = |doc: &FieldDoc, c: Option<&crate::TapeCache>, de: usize| -> Vec<f64> {
                let mut out = Vec::new();
                for i in 0..quadros {
                    let cam = Orbit {
                        rotation: Orbit::from_yaw_pitch(
                            0.72 + ((de + i) as f32) * graus.to_radians(),
                            0.52,
                        )
                        .rotation,
                        ..Orbit::default()
                    };
                    let t0 = std::time::Instant::now();
                    let _ = crate::trace_cached_for_test(doc, &reg, &cam, w, h, false, c);
                    // ⚠️ O 1.º quadro enche a cache do zero e não representa quadro nenhum que o
                    // artista veja depois do primeiro. O que se compara é o **regime**.
                    if i > 0 {
                        out.push(t0.elapsed().as_secs_f64() * 1000.0);
                    }
                }
                out
            };
            let caches = [
                crate::TapeCache::new(),
                crate::TapeCache::with_inflate(fator),
            ];
            // Aquecimento: um arrasto inteiro de cada lado, para o JIT do processo e as caches
            // chegarem ao regime.
            for (k, cache) in caches.iter().enumerate() {
                let _ = arrasto(&doc, (k == 1).then_some(cache), 0);
            }
            for ronda in 0..3usize {
                for (k, cache) in caches.iter().enumerate() {
                    let c = (k == 1).then_some(cache);
                    ph2d_field_eval::hybrid::FLOAT_TAPES.store(0, Ordering::Relaxed);
                    crate::TAPE_HITS.store(0, Ordering::Relaxed);
                    // ⚠️ O arrasto CONTINUA de onde parou: recomeçá-lo daria à cache um conjunto de
                    // regiões que ela acabou de ver, e o acerto seria o de um arrasto de
                    // ida-e-volta em vez do de um arrasto.
                    ms[k].push(med(arrasto(&doc, c, quadros * (ronda + 1))));
                    contagem[k] = (
                        ph2d_field_eval::hybrid::FLOAT_TAPES.load(Ordering::Relaxed) / quadros,
                        crate::TAPE_HITS.load(Ordering::Relaxed) / quadros,
                        cache.len(),
                    );
                }
            }
            for k in 0..2 {
                let (fitas, hits, guardadas) = contagem[k];
                println!(
                    "{n:7} | {graus:12.0} | {:6.2} | {:5} | {:9.2} | {fitas:16} | {hits:7} | {guardadas:9}",
                    if k == 1 { fator } else { 0.0 },
                    if k == 1 { "COM" } else { "sem" },
                    med(ms[k].clone()),
                );
            }
        }
    }
}

/// ⭐⭐⭐ **UMA FITA DE UM QUADRO SERVE O QUADRO SEGUINTE?** (W81) — a medição que desenha a W82.
///
/// A §82.9 mediu que compilar as fitas custa `~10`–`14 ms` de um quadro de `~24` e **não escala**.
/// A cura é não recompilar — e a pergunta que a desenha é **quanto uma região se mexe entre dois
/// quadros de um arrasto**.
///
/// ⭐ **Uma fita construída para a região `R` é válida em toda a sub-região de `R`** — é a cerca que
/// a W56 já escreveu. ⇒ se a fita for construída para `R` **inflada** por `f`, ela serve o quadro
/// seguinte sempre que a região nova ainda lá caiba, e a cache não precisa de chave nenhuma: ela
/// precisa de um **teste de contenção**.
///
/// A sonda mede as duas metades do compromisso:
///
/// | metade | o que se lê |
/// |---|---|
/// | **acerto** | que fracção das regiões do quadro `n+1` cabe na região do quadro `n` inflada por `f` |
/// | **preço** | quantas arestas a região inflada guarda a mais (o custo por amostra sobe com ela) |
///
/// ⚠️ Contagens e razões — vale com a máquina sob carga.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_whether_one_frames_tape_serves_the_next --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_whether_one_frames_tape_serves_the_next() {
    /// A caixa de mundo de uma região — o par que esta sonda passa de mão em mão.
    type Aabb = ([f32; 3], [f32; 3]);

    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let reg = Registry::new();
    let n = 168usize;
    let contour: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
    let index = ph2d_field_eval::profile_index::ProfileIndex::build(&profile);
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile,
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão");
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("caixa");
    let (w, h) = (640u32, 360u32);
    let plane = Screen::new(w, h, Orbit::default().half_extent);
    let tile = crate::tile_for_test();
    let slabs = crate::slabs_for_test();
    let sharp =
        crate::Sharpness::for_frame(Orbit::default().half_extent, (w as usize).min(h as usize));

    // As regiões de um quadro, na ordem (ladrilho, fatia) — `None` onde nenhuma se constrói.
    let regions = |cam: &Orbit| -> Vec<Option<Aabb>> {
        let mut out = Vec::new();
        for ty in 0..(h as usize).div_ceil(tile) {
            for tx in 0..(w as usize).div_ceil(tile) {
                let (x0, y0) = (tx * tile, ty * tile);
                let (x1, y1) = ((x0 + tile).min(w as usize), (y0 + tile).min(h as usize));
                let Some((t_lo, t_hi)) =
                    crate::tiles::tile_t_range(cam, plane, (x0, y0), (x1, y1), bbox)
                else {
                    for _ in 0..slabs + 2 {
                        out.push(None);
                    }
                    continue;
                };
                let bounds = crate::tiles::slab_bounds(t_lo, t_hi, slabs);
                for k in 0..bounds.len() - 1 {
                    out.push(
                        crate::tiles::slab_region(
                            cam,
                            plane,
                            (x0, y0),
                            (x1, y1),
                            bbox,
                            sharp.normal,
                            &bounds,
                            k,
                        )
                        .map(|r| (r.lo, r.hi)),
                    );
                }
            }
        }
        out
    };
    let inflate = |b: Aabb, f: f32| -> Aabb {
        let mut lo = b.0;
        let mut hi = b.1;
        for k in 0..3 {
            let c = 0.5 * (b.0[k] + b.1[k]);
            let half = 0.5 * (b.1[k] - b.0[k]) * f;
            lo[k] = c - half;
            hi[k] = c + half;
        }
        (lo, hi)
    };
    let inside =
        |a: Aabb, b: Aabb| -> bool { (0..3).all(|k| a.0[k] >= b.0[k] && a.1[k] <= b.1[k]) };
    // ⚠️ Para uma extrusão na pose identidade o `(u, v)` do perfil **é** o `(x, y)` do mundo, então
    // o corte da região lê-se directamente no índice do contorno.
    let kept = |b: Aabb| -> usize { index.probe_cull([b.0[0], b.0[1]], [b.1[0], b.1[1]]) };

    let base = Orbit::default();
    println!("arrasto | f    | acerto | arestas por região (média) | contra f=1");
    for graus in [1.0f32, 2.0, 4.0, 8.0] {
        let mut moved = base;
        // ⚠️ Um arrasto orbita: a rotação muda, o alvo e a lente não.
        moved.rotation = Orbit::from_yaw_pitch(0.72 + graus.to_radians(), 0.52).rotation;
        let (a, b) = (regions(&base), regions(&moved));
        let pares: Vec<(Aabb, Aabb)> = a
            .iter()
            .zip(&b)
            .filter_map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some((*x, *y)),
                _ => None,
            })
            .collect();
        let base_kept =
            pares.iter().map(|(x, _)| kept(*x)).sum::<usize>() as f64 / pares.len() as f64;
        for f in [1.0f32, 1.25, 1.5, 2.0, 3.0] {
            let hits = pares
                .iter()
                .filter(|(x, y)| inside(*y, inflate(*x, f)))
                .count();
            let k = pares
                .iter()
                .map(|(x, _)| kept(inflate(*x, f)))
                .sum::<usize>() as f64
                / pares.len() as f64;
            println!(
                "{graus:5.0}° | {f:4.2} | {:5.1}% | {k:26.1} | {:10.2}x",
                100.0 * hits as f64 / pares.len() as f64,
                k / base_kept,
            );
        }
    }
}

/// ⭐⭐⭐ **O QUADRO ESCALA MELHOR AGORA QUE A COMPILAÇÃO SAIU?** (W86) — a reconferência da §82.8.
///
/// A §82.8.1 mediu **`36 %`** de eficiência a 32 threads e a §82.9 nomeou a causa com um controlo: o
/// JIT **satura às 16**. ⭐ A W82 (cache entre quadros) e a W83 (o `fork` deixa de compilar) tiraram
/// quase toda a compilação de um quadro — de `226` fitas para `5`–`15`. ⇒ *a causa nomeada deixou de
/// estar lá, e a nota tem de ser reconferida* (`CLAUDE.md §0.0`).
///
/// ⚠️ **A cache é aquecida antes de medir**, e o mesmo quadro é medido em cada tamanho de pool: o que
/// se compara é o **mesmo trabalho** repartido por mais threads.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_whether_the_cached_frame_scales_better --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_whether_the_cached_frame_scales_better() {
    let reg = Registry::new();
    let doc = cache_piece(168);
    let cam = Orbit::default();
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    for com_cache in [false, true] {
        let cache = crate::TapeCache::new();
        let c = com_cache.then_some(&cache);
        // Aquecimento: um arrasto, e depois o quadro que se mede.
        for i in 0..8 {
            let quente = Orbit {
                rotation: Orbit::from_yaw_pitch(0.72 + (i as f32) * 2.0f32.to_radians(), 0.52)
                    .rotation,
                ..Orbit::default()
            };
            let _ = crate::trace_cached_for_test(&doc, &reg, &quente, 640, 360, false, c);
        }
        println!(
            "== {} cache ==\nthreads | ms      | ganho | eficiência",
            if com_cache { "COM" } else { "sem" }
        );
        let mut base = 0.0f64;
        for (k, t) in [1usize, 2, 4, 8, 16, 32].into_iter().enumerate() {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(t)
                .build()
                .expect("pool");
            let ms = pool.install(|| {
                let _ = crate::trace_cached_for_test(&doc, &reg, &cam, 640, 360, false, c);
                med((0..5)
                    .map(|_| {
                        let t0 = std::time::Instant::now();
                        let _ = crate::trace_cached_for_test(&doc, &reg, &cam, 640, 360, false, c);
                        t0.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect())
            });
            if k == 0 {
                base = ms;
            }
            println!(
                "{t:7} | {ms:7.2} | {:5.2}x | {:9.0}%",
                base / ms,
                100.0 * (base / ms) / t as f64
            );
        }
    }
}

/// ⭐⭐⭐ **O `TILE` COM A CACHE LIGADA** (W88) — a terceira reconferência, e a premissa mudou outra
/// vez.
///
/// A §82.10 varreu o `TILE` e fechou-o em `64`. ⚠️ **Aquela varredura é ANTERIOR à cache** (W82): ela
/// media um mundo em que um ladrilho mais pequeno pagava **uma compilação de JIT a mais**, e a
/// montagem era metade do quadro. Com a cache, a fita de um ladrilho pequeno **também** é reusada, e
/// o termo que castigava os ladrilhos pequenos quase desapareceu.
///
/// ⭐ E o oráculo do escalonamento (`measure_what_a_perfect_tile_schedule_would_buy`) diz porque isto
/// importa agora: a 32 threads, **nem a ordem perfeita passa de `1,52×`** — o pior ladrilho vale
/// `4,74 %` do quadro inteiro e a fatia ideal é `3,1 %`. *Uma ordem não parte um ladrilho.*
///
/// *Quem move o número que sustenta uma nota tem de reconferir a nota* (`CLAUDE.md §0.0`).
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_the_tile_size_now_that_the_cache_exists --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_tile_size_now_that_the_cache_exists() {
    let reg = Registry::new();
    let cam = Orbit::default();
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let tiles = [16usize, 24, 32, 48, 64, 96];
    for ((w, h), n) in [
        ((640u32, 360u32), 168usize),
        ((640, 360), 672),
        ((1600, 900), 168),
    ] {
        let doc = cache_piece(n);
        // ⚠️ **INTERCALADO** ronda a ronda: entre duas corridas desta workstation o mesmo passe já
        // deu `11,36` e `5,50 ms`, e varrer um tamanho de cada vez mede a máquina.
        let mut cols: Vec<Vec<f64>> = vec![Vec::new(); tiles.len()];
        let caches: Vec<crate::TapeCache> = tiles.iter().map(|_| crate::TapeCache::new()).collect();
        for ronda in 0..3usize {
            for (k, &tile) in tiles.iter().enumerate() {
                // Aquecimento: um arrasto, para a cache daquele tamanho chegar ao regime.
                for i in 0..6 {
                    let quente = Orbit {
                        rotation: Orbit::from_yaw_pitch(
                            0.72 + ((ronda * 6 + i) as f32) * 2.0f32.to_radians(),
                            0.52,
                        )
                        .rotation,
                        ..Orbit::default()
                    };
                    let _ = crate::trace_tiled_with_cache_for_test(
                        &doc,
                        &reg,
                        &quente,
                        w,
                        h,
                        tile,
                        crate::slabs_for_test(),
                        Some(&caches[k]),
                    );
                }
                let runs: Vec<f64> = (0..5)
                    .map(|_| {
                        let t0 = std::time::Instant::now();
                        let _ = crate::trace_tiled_with_cache_for_test(
                            &doc,
                            &reg,
                            &cam,
                            w,
                            h,
                            tile,
                            crate::slabs_for_test(),
                            Some(&caches[k]),
                        );
                        t0.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect();
                cols[k].push(med(runs));
            }
        }
        let ms: Vec<f64> = cols.into_iter().map(med).collect();
        let melhor = ms
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| tiles[i]);
        print!("{w}x{h} {n:4}ar |");
        for (k, t) in tiles.iter().enumerate() {
            print!(" {t}:{:7.2} ({:5}) |", ms[k], caches[k].len());
        }
        println!(" melhor {melhor}");
    }
}

/// ⭐⭐⭐ **O ORÁCULO DO ESCALONAMENTO** (W88) — *simule antes de construir*.
///
/// A §89 mediu que a decomposição custa `1,47×` e que ordenar por **profundidade** é neutro a pior.
/// ⚠️ **A pergunta que ficou não é «que estimador de custo usar»: é se a ORDEM é sequer o
/// mecanismo.** Construir um estimador para descobrir depois que não era seria pagar duas vezes.
///
/// ⭐ Esta sonda responde com o **custo VERDADEIRO** de cada ladrilho (medido, serial) e uma
/// simulação do escalonamento — aritmética pura, sem uma linha de produto:
///
/// | ordem | o que ela prevê |
/// |---|---|
/// | **natural** (varrimento) | o que o produto faz hoje |
/// | **LPT** (o mais caro primeiro) | o melhor que uma ordem consegue |
/// | **ideal** | `total / threads`, que nenhuma ordem alcança se um ladrilho for maior que a fatia |
///
/// ⇒ se o LPT previsto encostar no ideal, a ordem **é** o mecanismo e vale construir o estimador; se
/// não, o `1,47×` está noutro sítio e o estimador seria trabalho perdido.
///
/// ⚠️ Contagens e aritmética — vale com a máquina sob carga.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_what_a_perfect_tile_schedule_would_buy --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_what_a_perfect_tile_schedule_would_buy() {
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let doc = cache_piece(168);
    let cam = Orbit::default();
    let (w, h) = (640u32, 360u32);
    crate::TILE_COSTS.lock().expect("mutex").clear();
    crate::RECORD_TILE_COSTS.store(true, Ordering::Relaxed);
    // ⚠️ **SERIAL**: a diferença dos contadores globais em torno de um ladrilho só é a conta dele
    // quando ninguém mais escreve entretanto.
    let g = crate::trace_tiled_for_test(
        &doc,
        &reg,
        &cam,
        w,
        h,
        crate::tile_for_test(),
        crate::slabs_for_test(),
        false,
        false,
    )
    .expect("traçado");
    crate::RECORD_TILE_COSTS.store(false, Ordering::Relaxed);
    assert!(g.hits() > 1000, "a peça saiu vazia — a sonda não mede nada");
    let mut custos: Vec<u64> = crate::TILE_COSTS
        .lock()
        .expect("mutex")
        .iter()
        .map(|(_, c)| *c)
        .collect();
    let total: u64 = custos.iter().sum();
    let pior = custos.iter().copied().max().unwrap_or(0);
    println!(
        "{} ladrilhos · {total} amostras · o pior vale {pior} ({:.2}% do total)",
        custos.len(),
        100.0 * pior as f64 / total as f64
    );

    // ⭐ **A simulação: escalonamento de lista guloso** — cada ladrilho vai para a thread que está
    // livre mais cedo. É exactamente o que um `par_iter` com roubo de trabalho faz.
    let makespan = |ordem: &[u64], t: usize| -> u64 {
        let mut carga = vec![0u64; t];
        for &c in ordem {
            let i = carga
                .iter()
                .enumerate()
                .min_by_key(|(_, l)| **l)
                .map_or(0, |(i, _)| i);
            carga[i] += c;
        }
        carga.into_iter().max().unwrap_or(0)
    };
    let natural = custos.clone();
    custos.sort_unstable_by(|a, b| b.cmp(a));
    println!("threads | ideal | ordem natural | LPT (o pior 1º) | natural/ideal | LPT/ideal");
    for t in [8usize, 16, 32] {
        let ideal = total as f64 / t as f64;
        let (a, b) = (makespan(&natural, t) as f64, makespan(&custos, t) as f64);
        println!(
            "{t:7} | {ideal:9.0} | {a:13.0} | {b:15.0} | {:13.2}x | {:9.2}x",
            a / ideal,
            b / ideal
        );
    }
}

/// ⭐⭐⭐ **O QUE ESTRAGA A ESCALA: A DECOMPOSIÇÃO OU UM RECURSO PARTILHADO?** (W87) — o
/// discriminador.
///
/// A §88.2 mediu que o quadro usa `~30 %` da máquina e que **o JIT não é a causa** (tirá-lo não
/// mudou a forma da curva). Sobram dois candidatos nomeados, e eles mandam em waves opostas:
///
/// | candidato | o que ele diz | a wave que ele pede |
/// |---|---|---|
/// | **a decomposição** (desequilíbrio de ladrilhos, sincronização) | o trabalho existe e está mal repartido | repartir melhor |
/// | **um recurso partilhado** (largura de banda, SMT) | a máquina não tem mais para dar | fazer **menos** trabalho |
///
/// ⭐⭐⭐ **O experimento que os separa é clássico:** correr `T` quadros **independentes**, cada um
/// numa thread e cada um **serial**, contra **um** quadro repartido por `T` threads. O trabalho total
/// é o mesmo e a decomposição desaparece.
///
/// - se os independentes escalam e o repartido não ⇒ **a decomposição**;
/// - se nenhum dos dois escala ⇒ **o recurso partilhado**.
///
/// ⚠️ **Sem cache nos dois braços**, de propósito: uma cache partilhada entre `T` traçados seria uma
/// terceira variável, e o que se mede aqui é a **marcha**.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_whether_the_loss_is_the_decomposition_or_a_shared_resource --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_whether_the_loss_is_the_decomposition_or_a_shared_resource() {
    use rayon::prelude::*;
    let reg = Registry::new();
    let doc = cache_piece(168);
    let cam = Orbit::default();
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let (w, h) = (640u32, 360u32);
    let slabs = crate::slabs_for_test();
    let tile = crate::tile_for_test();
    // ⚠️ O quadro SERIAL de referência — a mesma porta, com o paralelismo desligado.
    let serial = || {
        crate::trace_tiled_for_test(&doc, &reg, &cam, w, h, tile, slabs, false, false)
            .expect("traçado")
            .hits()
    };
    let _ = serial();
    let um = med((0..5)
        .map(|_| {
            let t0 = std::time::Instant::now();
            let _ = serial();
            t0.elapsed().as_secs_f64() * 1000.0
        })
        .collect());
    println!("um quadro SERIAL: {um:.2} ms");
    println!(
        "threads |  1 quadro repartido | T quadros independentes | ganho repartido | ganho independentes"
    );
    for t in [2usize, 4, 8, 16, 32] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build()
            .expect("pool");
        // A: um quadro, repartido por `t` threads.
        let repartido = pool.install(|| {
            let _ = crate::trace_tiled_for_test(&doc, &reg, &cam, w, h, tile, slabs, false, true);
            med((0..5)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let _ = crate::trace_tiled_for_test(
                        &doc, &reg, &cam, w, h, tile, slabs, false, true,
                    );
                    t0.elapsed().as_secs_f64() * 1000.0
                })
                .collect())
        });
        // B: `t` quadros INDEPENDENTES, cada um serial. O relógio é o do lote inteiro dividido por
        // `t` — o tempo por quadro, comparável com o de cima.
        let independentes = pool.install(|| {
            let lote = || {
                (0..t).into_par_iter().for_each(|_| {
                    let _ = crate::trace_tiled_for_test(
                        &doc, &reg, &cam, w, h, tile, slabs, false, false,
                    );
                });
            };
            lote();
            med((0..5)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    lote();
                    t0.elapsed().as_secs_f64() * 1000.0 / t as f64
                })
                .collect())
        });
        println!(
            "{t:7} | {repartido:19.2} | {independentes:23.2} | {:15.2}x | {:19.2}x",
            um / repartido,
            um / independentes,
        );
    }
}

/// ⭐⭐⭐ **O JIT CONTENDE, OU FOI SÓ MEDIDO NO MEIO DA MARCHA?** (W81) — o controlo da §82.8.2.
///
/// A `measure_where_the_parallel_frame_stops_scaling` mediu que uma fita custa `1,93×` mais CPU a
/// 32 threads que a 1. ⚠️ **Ela mediu-o DENTRO de um quadro**, com as outras 31 threads a marchar —
/// e duas explicações diferentes dão o mesmo número:
///
/// 1. **o JIT contende** (ele mapeia memória **executável**, e `mmap`/`mprotect` são do kernel);
/// 2. **a marcha satura a memória** e a compilação, que corre ao lado dela, apanha a factura.
///
/// ⛔ **As duas mandam em waves diferentes** — a primeira diz *«compile menos fitas»*, a segunda diz
/// *«o problema é a marcha e a montagem é uma vítima»*. Esta sonda separa-as: ela **só compila**,
/// sem marchar uma única amostra.
///
/// ⚠️ Precisa da máquina a `load < 5`.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_whether_the_jit_contends_on_its_own --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_whether_the_jit_contends_on_its_own() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use rayon::prelude::*;
    let reg = Registry::new();
    let n = 168usize;
    let contour: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão");
    let rc = ph2d_field_eval::RegionCompiler::new(&doc);
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)
        .expect("caixa");
    // ⚠️ **Regiões parecidas com as de um quadro a sério**: uma grelha de caixas dentro da peça, com
    // a mesma ordem de grandeza de arestas guardadas. O que se mede é o **compilador**, e ele não
    // sabe de onde a caixa veio.
    let jobs: Vec<([f32; 3], [f32; 3])> = (0..242)
        .map(|k| {
            let (i, j) = ((k % 11) as f32, ((k / 11) % 11) as f32);
            let lo = [
                bbox.0[0] + (bbox.1[0] - bbox.0[0]) * i / 12.0,
                bbox.0[1] + (bbox.1[1] - bbox.0[1]) * j / 12.0,
                bbox.0[2],
            ];
            let hi = [
                lo[0] + (bbox.1[0] - bbox.0[0]) / 6.0,
                lo[1] + (bbox.1[1] - bbox.0[1]) / 6.0,
                bbox.1[2],
            ];
            (lo, hi)
        })
        .collect();
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!(
        "SÓ compilação, {} regiões, nenhuma amostra marchada",
        jobs.len()
    );
    println!("threads | ms      | ns por fita | contra 1 thread");
    let mut base = 0.0f64;
    for (k, &t) in [1usize, 2, 4, 8, 16, 32].iter().enumerate() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build()
            .expect("pool");
        let build = || {
            jobs.par_iter()
                .map(|(lo, hi)| {
                    let pts = [
                        [lo[0], lo[1], lo[2]],
                        [hi[0], hi[1], hi[2]],
                        [lo[0], hi[1], lo[2]],
                        [hi[0], lo[1], hi[2]],
                    ];
                    let tape = ph2d_field_eval::hybrid::Hybrid::from_tree(
                        rc.compile_at(&doc, *lo, *hi, &pts),
                    );
                    tape.sampled_count()
                })
                .sum::<usize>()
        };
        let ms = pool.install(|| {
            let _ = build();
            med((0..5)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let _ = build();
                    t0.elapsed().as_secs_f64() * 1000.0
                })
                .collect())
        });
        // ⭐ O custo de CPU de UMA fita: relógio de parede × threads ÷ fitas.
        let per = ms * t as f64 * 1.0e6 / jobs.len() as f64;
        if k == 0 {
            base = per;
        }
        println!("{t:7} | {ms:7.2} | {per:11.0} | {:14.2}x", per / base);
    }
}

/// ⭐⭐⭐ **ONDE O QUADRO PARALELO DEIXA DE ESCALAR** (W81) — a medição que fecha o piso da
/// `measure_the_floor_that_the_tile_size_puts_under_the_frame`.
///
/// A contagem diz que o ladrilho mais caro sozinho vale `1,52×` a fatia perfeita de todo o trabalho
/// do quadro, e que o joelho está em `48`. ⚠️ **A contagem é um minorante** (dois ladrilhos com as
/// mesmas amostras custam diferente, porque as regiões guardam números diferentes de arestas), e a
/// varredura de relógio da §72 diz o contrário — `48` mais lento que `64`. Só o relógio decide, e
/// duas grandezas o decidem:
///
/// 1. **a curva de escalamento por threads** com o ladrilho que ship — se ela achata cedo, o piso
///    morde;
/// 2. **a varredura de `TILE` no quadro que HOJE ship** — paralelo, **sem anti-serrilhado** (a
///    varredura da §72 mediu com ele, e a W72 tirou-o do quadro de movimento).
///
/// ⛔ **Precisa da máquina a `load < 5`** (`CLAUDE.md §5.0`), e as duas metades correm
/// **intercaladas** — entre duas corridas desta workstation o mesmo passe já deu `11,36` e
/// `5,50 ms`.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_where_the_parallel_frame_stops_scaling --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_where_the_parallel_frame_stops_scaling() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let reg = Registry::new();
    let cam = Orbit::default();
    let piece = |n: usize| -> FieldDoc {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão")
    };
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let (w, h) = (640u32, 360u32);
    let doc = piece(168);
    let tile_now = crate::tile_for_test();
    let slabs = crate::slabs_for_test();

    println!(
        "== 1. escalamento por threads (640x360, 168 arestas, lado {tile_now}, SEM anti-serrilhado) =="
    );
    let threads_set = [1usize, 2, 4, 8, 16, 32];
    let mut cols: Vec<Vec<f64>> = vec![Vec::new(); threads_set.len()];
    for _ in 0..3 {
        for (k, &t) in threads_set.iter().enumerate() {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(t)
                .build()
                .expect("pool");
            let runs: Vec<f64> = pool.install(|| {
                let _ = crate::trace_tiled_for_test(
                    &doc, &reg, &cam, w, h, tile_now, slabs, false, true,
                );
                (0..5)
                    .map(|_| {
                        let t0 = std::time::Instant::now();
                        let _ = crate::trace_tiled_for_test(
                            &doc, &reg, &cam, w, h, tile_now, slabs, false, true,
                        );
                        t0.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect()
            });
            cols[k].push(med(runs));
        }
    }
    let ms: Vec<f64> = cols.into_iter().map(med).collect();
    println!("threads | ms      | ganho | eficiência");
    for (k, &t) in threads_set.iter().enumerate() {
        println!(
            "{t:7} | {:7.2} | {:5.2}x | {:9.0}%",
            ms[k],
            ms[0] / ms[k],
            100.0 * (ms[0] / ms[k]) / t as f64
        );
    }

    println!("\n== 3. a MONTAGEM escala? (ns de CPU por fita, por número de threads) ==");
    // ⭐⭐⭐ **A pergunta de Amdahl.** A montagem é 96% JIT, e um JIT mapeia memória executável —
    // `mmap`/`mprotect` são recursos do KERNEL, partilhados por todas as threads. Se o custo de CPU
    // de UMA fita subir com o número de threads, a montagem não é uma fracção que se divide: é uma
    // fracção que se **serializa**, e nenhuma quantidade de núcleos a atravessa.
    println!("threads | ms      | fitas | montagem ms (CPU) | ns por fita");
    for &t in &threads_set {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build()
            .expect("pool");
        let (ms_t, asm, tapes) = pool.install(|| {
            let _ =
                crate::trace_tiled_for_test(&doc, &reg, &cam, w, h, tile_now, slabs, false, true);
            let mut best = (f64::INFINITY, 0.0, 0usize);
            for _ in 0..5 {
                crate::SPECIALISE_NS.store(0, std::sync::atomic::Ordering::Relaxed);
                crate::SPECIALISED.store(0, std::sync::atomic::Ordering::Relaxed);
                let t0 = std::time::Instant::now();
                let _ = crate::trace_tiled_for_test(
                    &doc, &reg, &cam, w, h, tile_now, slabs, false, true,
                );
                let el = t0.elapsed().as_secs_f64() * 1000.0;
                if el < best.0 {
                    best = (
                        el,
                        crate::SPECIALISE_NS.load(std::sync::atomic::Ordering::Relaxed) as f64
                            / 1.0e6,
                        crate::SPECIALISED.load(std::sync::atomic::Ordering::Relaxed),
                    );
                }
            }
            best
        });
        println!(
            "{t:7} | {ms_t:7.2} | {tapes:5} | {asm:17.2} | {:11.0}",
            asm * 1.0e6 / tapes as f64
        );
    }

    println!(
        "\n== 4. varredura de SLABS no quadro que HOJE ship (paralelo, SEM anti-serrilhado) =="
    );
    // ⭐⭐⭐ **A constante que a §82.8.2 reabre.** O `SLABS` foi escolhido na W71 **com
    // anti-serrilhado** — e a 2.ª passagem acrescenta marcha e **nenhuma** montagem, logo ela
    // sub-pesa exactamente o termo que não escala. Repartir MULTIPLICA as fitas e DIVIDE as arestas
    // por amostra: com o JIT a saturar às 16 threads, o primeiro termo ficou mais caro do que a
    // tabela da W71 podia ver.
    let slabs_set = [2usize, 3, 4, 6];
    for n in [168usize, 672] {
        let doc = piece(n);
        let mut cols: Vec<Vec<f64>> = vec![Vec::new(); slabs_set.len()];
        for _ in 0..3 {
            for (k, &sl) in slabs_set.iter().enumerate() {
                let _ =
                    crate::trace_tiled_for_test(&doc, &reg, &cam, w, h, tile_now, sl, false, true);
                let runs: Vec<f64> = (0..5)
                    .map(|_| {
                        let t0 = std::time::Instant::now();
                        let _ = crate::trace_tiled_for_test(
                            &doc, &reg, &cam, w, h, tile_now, sl, false, true,
                        );
                        t0.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect();
                cols[k].push(med(runs));
            }
        }
        let ms: Vec<f64> = cols.into_iter().map(med).collect();
        let win = ms
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| slabs_set[i])
            .unwrap_or(0);
        println!(
            "arestas {n:4} | N=2 {:6.2} | N=3 {:6.2} | N=4 {:6.2} | N=6 {:6.2} | melhor {win}",
            ms[0], ms[1], ms[2], ms[3]
        );
    }

    println!(
        "\n== 2. varredura de TILE no quadro que HOJE ship (paralelo, SEM anti-serrilhado) =="
    );
    let tiles_set = [32usize, 48, 64, 96];
    for n in [168usize, 672] {
        let doc = piece(n);
        let mut cols: Vec<Vec<f64>> = vec![Vec::new(); tiles_set.len()];
        for _ in 0..3 {
            for (k, &tile) in tiles_set.iter().enumerate() {
                let _ =
                    crate::trace_tiled_for_test(&doc, &reg, &cam, w, h, tile, slabs, false, true);
                let runs: Vec<f64> = (0..5)
                    .map(|_| {
                        let t0 = std::time::Instant::now();
                        let _ = crate::trace_tiled_for_test(
                            &doc, &reg, &cam, w, h, tile, slabs, false, true,
                        );
                        t0.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect();
                cols[k].push(med(runs));
            }
        }
        let ms: Vec<f64> = cols.into_iter().map(med).collect();
        let win = ms
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| tiles_set[i])
            .unwrap_or(0);
        println!(
            "arestas {n:4} | 32 {:6.2} | 48 {:6.2} | 64 {:6.2} | 96 {:6.2} | melhor {win}",
            ms[0], ms[1], ms[2], ms[3]
        );
    }
}

/// ⭐⭐⭐ **O PISO QUE O TAMANHO DO LADRILHO PÕE DEBAIXO DO QUADRO** (W81).
///
/// ⚠️ **Um ladrilho é indivisível**: ele compila a própria fita e marcha os próprios raios, e nenhuma
/// thread o pode partir ao meio. ⇒ o quadro **não pode** acabar antes do ladrilho mais caro, por
/// mais núcleos que a máquina tenha:
///
/// ```text
/// relógio >= max(trabalho_total / threads, ladrilho_mais_caro)
/// ```
///
/// ⭐⭐ **É a grandeza que nenhuma varredura de relógio viu**, porque ela não é um tempo: é uma
/// razão entre contagens, e lê-se com a máquina sob carga. As varreduras do `TILE` e do
/// [`crate::tiles::SLABS`] mediram **trabalho total**, e trabalho total não sabe que uma peça dele
/// não se reparte.
///
/// ⚠️ **A régua é a AMOSTRA, que é um minorante do trabalho de um ladrilho** — dois ladrilhos com o
/// mesmo número de amostras podem custar diferente, porque a região de cada um guarda um número
/// diferente de arestas. O `piso` abaixo é portanto uma **estimativa por baixo** do desequilíbrio, e
/// a medição que o fecha é a curva de escalamento por número de threads, que precisa da máquina
/// calma.
///
/// ⚠️ **Serial de propósito**: a diferença dos contadores globais em torno de um ladrilho só é a
/// conta dele quando ninguém mais escreve entretanto. As contagens não dependem do escalonamento.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_the_floor_that_the_tile_size_puts_under_the_frame --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_floor_that_the_tile_size_puts_under_the_frame() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    let n = 168usize;
    let contour: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile: Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão");
    let threads = rayon::current_num_threads().max(1) as f64;
    let (w, h) = (640u32, 360u32);
    println!(
        "640x360, 168 arestas, {threads:.0} threads — o quadro de MOVIMENTO (sem anti-serrilhado)"
    );
    println!("lado | ladrilhos | fitas | amostras | ideal/thread | mais caro | PISO");
    for tile in [16usize, 32, 48, 64, 96, 128] {
        crate::STEP_SAMPLES.store(0, Ordering::Relaxed);
        crate::NORMAL_SAMPLES.store(0, Ordering::Relaxed);
        crate::SPECIALISED.store(0, Ordering::Relaxed);
        crate::TILE_MAX.store(0, Ordering::Relaxed);
        let g = crate::trace_tiled_for_test(
            &doc,
            &reg,
            &cam,
            w,
            h,
            tile,
            crate::tiles::SLABS,
            false,
            false,
        )
        .expect("ladrilho");
        assert!(g.hits() > 1000, "a peça saiu vazia a lado {tile}");
        let total = (crate::STEP_SAMPLES.load(Ordering::Relaxed)
            + crate::NORMAL_SAMPLES.load(Ordering::Relaxed)) as f64;
        let worst = crate::TILE_MAX.load(Ordering::Relaxed) as f64;
        let ideal = total / threads;
        println!(
            "{tile:4} | {:9} | {:5} | {total:8.0} | {ideal:12.0} | {worst:9.0} | {:5.2}x",
            (w as usize).div_ceil(tile) * (h as usize).div_ceil(tile),
            crate::SPECIALISED.load(Ordering::Relaxed),
            worst.max(ideal) / ideal,
        );
    }
}

/// ⭐⭐⭐ **DE QUEM SÃO AS AMOSTRAS DA MARCHA** (W81) — a sonda que reconfere a conclusão da §73.
///
/// A §73 dividiu as amostras pelos **pixels** e leu `8,7` por pixel, e daí escreveu que *«a
/// sobre-relaxação não tem de onde tirar»*. ⚠️ **Um quadro é sobretudo fundo**, e o fundo que não
/// entra na caixa da peça custa **zero** amostras: ele afunda a média sem participar dela.
///
/// Esta sonda traz o denominador que faltava ([`crate::MARCH_RAYS`]), a parcela que faltava ao
/// numerador ([`crate::NORMAL_SAMPLES`]) e — porque **uma média não escolhe entre duas curas
/// opostas** — a **curva de sobrevivência** ([`crate::STEP_HIST`]).
///
/// ⚠️ **Contagens, não relógio**: ela vale com a máquina sob carga, que é precisamente quando as
/// tabelas de ms desta seção não valem nada.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_who_the_march_samples_belong_to --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_who_the_march_samples_belong_to() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    use std::sync::atomic::Ordering;
    let reg = Registry::new();
    let cam = Orbit::default();
    println!(
        "arestas | pixels | acertos | raios | amostras | /pixel | /raio | normais | % normais"
    );
    for n in [168usize, 672] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        let doc = FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");
        let (w, h) = (640u32, 360u32);
        crate::STEP_SAMPLES.store(0, Ordering::Relaxed);
        crate::MARCH_RAYS.store(0, Ordering::Relaxed);
        crate::NORMAL_SAMPLES.store(0, Ordering::Relaxed);
        crate::FORKED.store(0, Ordering::Relaxed);
        crate::SPECIALISED.store(0, Ordering::Relaxed);
        crate::TILE_MAX.store(0, Ordering::Relaxed);
        for b in &crate::STEP_HIST {
            b.store(0, Ordering::Relaxed);
        }
        for b in crate::SLAB_SPEC.iter().chain(crate::SLAB_SAMPLES.iter()) {
            b.store(0, Ordering::Relaxed);
        }
        // ⚠️ **Sem anti-serrilhado**: é o quadro de MOVIMENTO que não alcança o orçamento, e é dele
        // que a wave fala. A 2.ª passagem re-marcha a silhueta e contaria por cima.
        let g = crate::trace_with(&doc, &reg, &cam, w, h, false, false);
        let samples = crate::STEP_SAMPLES.load(Ordering::Relaxed) as f64;
        let rays = crate::MARCH_RAYS.load(Ordering::Relaxed) as f64;
        let normals = crate::NORMAL_SAMPLES.load(Ordering::Relaxed) as f64;
        let pixels = f64::from(w) * f64::from(h);
        println!(
            "{n:7} | {pixels:6.0} | {:7} | {rays:5.0} | {samples:8.0} | {:6.1} | {:5.1} | {normals:7.0} | {:8.1}",
            g.hits(),
            samples / pixels,
            samples / rays.max(1.0),
            100.0 * normals / (samples + normals),
        );
        let hist: Vec<u64> = crate::STEP_HIST
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect();
        // ⚠️ **A curva e o total são o MESMO número contado de duas maneiras** — se divergirem, a
        // forma da marcha que esta sonda imprime é a forma de outra coisa.
        assert_eq!(
            hist.iter().sum::<u64>() as f64,
            samples,
            "a curva de sobrevivência não soma as amostras"
        );
        assert_eq!(
            crate::NORMAL_SAMPLES.load(Ordering::Relaxed),
            g.hits() as u64 * 6,
            "o contador da normal e os acertos discordam"
        );
        // A curva de sobrevivência, em décimos do total — onde ela cai é a forma da marcha.
        let acc = |from: usize, to: usize| -> f64 {
            100.0 * hist[from..to.min(crate::HIST)].iter().sum::<u64>() as f64 / samples
        };
        println!(
            "        passos 0-3 {:5.1}% · 4-7 {:5.1}% · 8-15 {:5.1}% · 16-31 {:5.1}% · 32-63 {:5.1}%",
            acc(0, 4),
            acc(4, 8),
            acc(8, 16),
            acc(16, 32),
            acc(32, 64),
        );
        println!("        sobreviventes por passo: {:?}", &hist[..24]);
        println!(
            "        ladrilhos {} · especializadas {} · RECUOS (fork da árvore inteira) {} · ladrilho mais caro {}",
            (640usize.div_ceil(64)) * (360usize.div_ceil(64)),
            crate::SPECIALISED.load(Ordering::Relaxed),
            crate::FORKED.load(Ordering::Relaxed),
            crate::TILE_MAX.load(Ordering::Relaxed),
        );
        let spec: Vec<u64> = crate::SLAB_SPEC
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect();
        let ssam: Vec<u64> = crate::SLAB_SAMPLES
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect();
        println!("        por fatia — fitas montadas: {spec:?}");
        println!("        por fatia — amostras:       {ssam:?}");
        println!(
            "        por fatia — amostras por fita: {:?}",
            spec.iter()
                .zip(&ssam)
                .map(|(s, a)| if *s == 0 { 0 } else { a / s })
                .collect::<Vec<u64>>()
        );
    }
}

/// ⭐⭐⭐ **OS DOIS BOTÕES DO QUADRO DE MOVIMENTO** (W71) — e os dois são a mesma lei que a W69 já
/// ship: *grosso a mexer, nítido ao assentar*.
///
/// A §72.1 mediu que a marcha é `80 %` do quadro, e a `measure_the_shape_of_the_march` mediu a
/// forma dela: **`8,7` amostras por pixel** (a marcha já está apertada — sobre-relaxação não tem de
/// onde tirar) e **`147,5 ns` por amostra a 168 arestas contra `558,0` a 672**. ⇒ *o custo é por
/// ARESTA TOCADA*, e os dois botões que existem são:
///
/// 1. **quantas arestas o contorno tem enquanto a mão mexe** (`PREVIEW_MAX_EDGES`, W69);
/// 2. **o anti-serrilhado**, que re-marcha a silhueta quatro vezes.
///
/// ⚠️ **Nenhum dos dois é gratuito, e é por isso que a tabela traz as duas colunas** — quem decide
/// o que se perde ao mexer é quem vê, não esta sonda.
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_the_two_knobs_of_the_moving_frame --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_two_knobs_of_the_moving_frame() {
    use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
    let reg = Registry::new();
    let cam = Orbit::default();
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let piece = |n: usize| -> FieldDoc {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão")
    };
    println!("arestas | com anti-serrilhado | sem | o que o AA custa");
    for n in [48usize, 64, 96, 128, 168] {
        let doc = piece(n);
        let mut on = Vec::new();
        let mut off = Vec::new();
        for _ in 0..3 {
            for (aa, out) in [(true, &mut on), (false, &mut off)] {
                let _ = crate::trace_with(&doc, &reg, &cam, 640, 360, true, aa);
                let runs: Vec<f64> = (0..5)
                    .map(|_| {
                        let t = std::time::Instant::now();
                        let _ = crate::trace_with(&doc, &reg, &cam, 640, 360, true, aa);
                        t.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect();
                out.push(med(runs));
            }
        }
        let (a, b) = (med(on), med(off));
        println!("{n:7} | {a:19.1} | {b:5.1} | {:16.2}x", a / b);
    }
}

/// ⭐⭐⭐ **O QUE A SEGURANÇA DO PASSO CUSTA** (W75) — o preço de não furar a peça.
///
/// A W75 mediu que arredondamentos exactos **encadeados** compõem o factor de inflação do gradiente
/// (`1,4142` a um nível, `1,69` a dois, `1,96` a três), e o passo da marcha passou a ser
/// `1/√2^k`. ⚠️ **Isto é mais lento**, e o quanto é o que esta sonda mede — no MESMO processo, com
/// a porta que já existe para forçar o passo (`trace_stepped_for_test`).
///
/// *Um teto de segurança não se negocia com o relógio; mas o preço dele diz-se.*
///
/// ```text
/// cargo test -p ph2d-field-render --profile ci-test -- --exact \
///     tests::measure_what_the_safe_step_costs --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_what_the_safe_step_costs() {
    use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
    let reg = Registry::new();
    let cam = Orbit::default();
    let bx = |h: [f32; 3], at: Xform| {
        ph2d_field_eval::leaf(
            Primitive::Box {
                half: h,
                round: 0.0,
                chamfer: 0.0,
            },
            at,
        )
    };
    let chain = |levels: usize, r: f32| -> FieldDoc {
        let mut nodes = vec![
            bx([0.5, 0.25, 0.25], Xform::at(-0.25, 0.0, 0.0)),
            bx([0.25, 0.5, 0.25], Xform::at(0.25, 0.0, 0.0)),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Exact { radius: r }),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ];
        let mut root = 2u32;
        for k in 1..levels {
            let leafi = u32::try_from(nodes.len()).expect("arena pequena");
            nodes.push(bx(
                [0.3, 0.3, 0.3],
                Xform::at(0.0, 0.2 * k as f32, 0.15 * k as f32),
            ));
            nodes.push(Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Exact { radius: r }),
                    children: vec![NodeId(root), NodeId(leafi)],
                },
            ));
            root = leafi + 1;
        }
        FieldDoc::new(nodes, NodeId(root)).expect("a peça")
    };
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!(
        "níveis | passo seguro | ms com ele | ms com 1/√2 (INSEGURO) | o que a segurança custa"
    );
    for levels in [1usize, 2, 3] {
        let doc = chain(levels, 0.2);
        let safe = ph2d_field_eval::safe_march_step(&doc);
        let mut a = Vec::new();
        let mut b = Vec::new();
        for _ in 0..3 {
            for (step, out) in [(safe, &mut a), (std::f32::consts::FRAC_1_SQRT_2, &mut b)] {
                let _ = crate::trace_stepped_for_test(&doc, &reg, &cam, 640, 360, step);
                let runs: Vec<f64> = (0..5)
                    .map(|_| {
                        let t = std::time::Instant::now();
                        let _ = crate::trace_stepped_for_test(&doc, &reg, &cam, 640, 360, step);
                        t.elapsed().as_secs_f64() * 1000.0
                    })
                    .collect();
                out.push(med(runs));
            }
        }
        let (sa, sb) = (med(a), med(b));
        println!(
            "{levels:6} | {safe:12.4} | {sa:10.1} | {sb:22.1} | {:22.2}x",
            sa / sb
        );
    }
}

/// ⭐⭐⭐ **A CAIXA DESLOCADA AINDA CONTÉM A REGIÃO** (W89) — a invariante da
/// [`crate::tape_cache::PHASE`].
///
/// # Porque ela é uma invariante e não uma tolerância
///
/// A cache serve uma fita a toda região que caiba na caixa dela. Uma caixa deslocada para FORA da
/// região serviria uma fita especializada num sítio onde a região não está — e o resultado não é um
/// erro, é uma **imagem plausível e errada**, que é o pior modo de falha que há.
///
/// A conta é fechada: a folga por lado que a inflação paga é `half·(f−1)`, e o deslocamento é
/// `half·(f−1)·u` com `|u| ≤ amp`. ⇒ para `amp ≤ 1` a região continua dentro, **por construção**.
/// Este gate afirma-o sobre a amplitude que ship, em muitas sementes e em regiões de formas
/// diferentes — porque a fórmula certa escrita uma vez pode ser reescrita errada.
#[test]
fn the_phased_box_still_contains_its_region() {
    for (i, (lo, hi)) in [
        ([-0.2f32, -0.2, -0.2], [0.2f32, 0.2, 0.2]),
        ([0.0, -1.0, 3.0], [0.01, 1.0, 3.5]),
        ([-5.0, -5.0, -5.0], [-4.9, 5.0, 0.0]),
    ]
    .into_iter()
    .enumerate()
    {
        for seed in 0..512u64 {
            let (blo, bhi) = crate::tape_cache::inflate_phased(
                lo,
                hi,
                crate::tape_cache::INFLATE,
                seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ (i as u64),
                crate::tape_cache::PHASE,
            );
            for k in 0..3 {
                assert!(
                    blo[k] <= lo[k] && bhi[k] >= hi[k],
                    "a caixa deslocada tem de conter a região (eixo {k}, semente {seed}): \
                     [{}, {}] não contém [{}, {}]",
                    blo[k],
                    bhi[k],
                    lo[k],
                    hi[k]
                );
            }
        }
    }
}

/// ⭐⭐⭐ **O DESPEJO DEITA FORA METADE, E A CACHE NUNCA PASSA O TECTO** (W89).
///
/// # A lei, e o modo de falha que ela fecha
///
/// A escolha do que despejar é por **índice** (ordenar por idade e tirar `k`), nunca por corte de
/// valor. ⚠️ Um corte por idade — *«deita fora todos os mais velhos que X»* — deita fora **nada**
/// quando metade das fitas foi tocada no mesmo tique, e uma cache que não consegue despejar
/// **cresce para sempre**: cada fita é um `mmap` de código executável, e o tecto de mapeamentos de
/// um processo Linux é `65 530`.
///
/// ⚠️ **É um gate de CONTAGEM, de propósito** — o defeito que ele apanha (a cache a crescer, o
/// despejo a não despejar) é um facto de população, e um gate de relógio sobre ele reprovaria sob
/// fan-out sem nada ter mudado.
#[test]
fn the_eviction_drops_half_and_the_cache_never_grows_past_its_ceiling() {
    use ph2d_field_eval::hybrid::RegionTape;
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node {
            xform: Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Leaf(Primitive::Box {
                half: [0.4, 0.3, 0.2],
                round: 0.05,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("caixa");
    let rc = ph2d_field_eval::RegionCompiler::new(&doc);
    let cache = crate::TapeCache::new();
    // O tecto é derivado do que o quadro pede: `64` regiões × `FRAMES_KEPT`, com o piso de `64`.
    cache.begin(&doc, 1);
    let tecto = 64usize;
    // ⚠️ **Regiões DISTINTAS** — 200 inserções da mesma caixa provariam outra coisa.
    let mut visto_acima = false;
    for i in 0..(tecto * 3) {
        let t = (i as f32) * 0.01 - 1.0;
        let tape = RegionTape::compile(rc.compile(&doc, [t, -0.1, -0.1], [t + 0.02, 0.1, 0.1]));
        cache.insert([t, -0.1, -0.1], [t + 0.02, 0.1, 0.1], tape);
        assert!(
            cache.len() <= tecto,
            "a cache passou o tecto ({} > {tecto}) na inserção {i} — um despejo que não despeja \
             deixa-a crescer para sempre",
            cache.len()
        );
        visto_acima |= cache.len() >= tecto / 2;
    }
    assert!(
        visto_acima,
        "a cache nunca chegou perto do tecto: esta fixtura não contém o fenómeno"
    );
    // Depois de despejar, ela fica na ordem de metade — não vazia (deitaria fora o quadro corrente)
    // nem cheia (não teria despejado).
    assert!(
        cache.len() > tecto / 4,
        "o despejo levou fitas a mais ({} de {tecto}): metade é a lei",
        cache.len()
    );
}
