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
    let a = trace(&doc, &Orbit::from_yaw_pitch(0.0, 0.0), 160, 160);
    let b = trace(&doc, &Orbit::from_yaw_pitch(1.1, -0.7), 160, 160);
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
    let g = trace(&sphere(radius), &cam, w, h);

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
    let before = trace(&doc, &Orbit::default(), 120, 120).hits() as f64;
    let after = trace(&doc, &cam, 120, 120).hits() as f64;
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
    let g = trace(&doc, &cam, w, h);
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
