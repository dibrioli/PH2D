//! As SONDAS da cena `=49` — os instrumentos, não a lei.
//!
//! ⛔ **Saíram do [`super`] por TECTO DE LOC** (`830` contra `700`), e o corte
//! é por RESPONSABILIDADE: ali ficam os dois gates (a cena tem o que mostrar ·
//! a cena está fiada) e aqui ficam as réguas exploratórias e os desenhadores
//! de PPM, que nenhuma tabela cita e que existem para a próxima janela medir.
//!
//! ⚠️ **O nome acaba em `_tests.rs` de propósito:** a classificação da família
//! é DERIVADA, e um ficheiro compilado só sob `cfg(test)` com outro nome
//! passaria a ser lido como PRODUTO pelas réguas de arquitectura — a
//! armadilha que o §41 desta linha já pagou.
//!
//! ⛔⛔ **E uma premissa minha CAIU no corte, desmentida pelo compilador:** eu
//! escrevi que mover o condutor `traco_com` para cá tornava o gate de fiação
//! do irmão mais forte (a agulha montada num ficheiro, varrida noutro). É
//! FALSO — o `traco_com` **é** a agulha, porque o `traco` que os dois gates
//! usam delega nele: ele é o condutor do traço, não um instrumento. Ficou lá.
//! *O que impede aquela agulha de se satisfazer a si própria continua a ser
//! ela ser montada por `format!`, e não o sítio onde mora.*

use super::*;
/// **SONDA — que peça tem GRÃO, e quanto?** (a régua que escolheu a da cena)
#[test]
#[ignore = "sonda"]
fn diag_o_grao_das_pecas_candidatas() {
    type Candidata = (&'static str, fn() -> ph2d_mesh::Mesh);
    let candidatas: Vec<Candidata> = vec![
        (
            "a peca da cena (uv 12k)",
            crate::scenes::pente::peca as fn() -> _,
        ),
        ("uv + ruido", || {
            ph2d_mesh::shapes::uv_sphere_noisy(55, 82, 1.0, 0.01)
        }),
        ("uv + remesh isotropico", || {
            let mut m = ph2d_mesh::shapes::sphere_with_triangles(12_000, 1.0);
            m.triangulate();
            let _ = ph2d_remesh_iso::remesh_isotropic(&mut m, 0.0142);
            m
        }),
        ("sculpt_sphere (a de fabrica)", || {
            ph2d_mesh::shapes::sculpt_sphere(1.0)
        }),
    ];
    for (nome, faz) in candidatas {
        let mut m = faz();
        m.triangulate();
        let v = m.positions().len();
        for (rotulo, eixo) in [("equador x", 0usize), ("meridiano y", 1)] {
            let mut centros = Vec::new();
            for k in 0..24 {
                let u = -0.55 + 0.05 * k as f32;
                let mut c = [0.0, 0.0, u.cos()];
                c[eixo] = u.sin();
                centros.push(c);
            }
            let (q, n) = q_da_faixa(&m, &centros, raio_do_app());
            eprintln!("{nome:<30} {rotulo:<12} v={v:<7} Q={q:+.4} (n={n})");
        }
    }
}

/// **SONDA — a escada do pente por RUMO contra a grade**, de onde saem a tabela
/// do cabeçalho da cena e a linha de água dos 45°.
#[test]
#[ignore = "sonda"]
fn diag_a_escada_por_rumo() {
    for (nome, e) in [
        ("ao longo (x)", [1.0f32, 0.0]),
        ("30 graus", [0.866_025_4, 0.5]),
        ("45 graus", RUMOS[2].1),
        ("60 graus", [0.5, 0.866_025_4]),
        ("atravessado (y)", [0.0, 1.0]),
    ] {
        for pente in [0.0f32, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0] {
            let (m, c) = traco(pente, e);
            let (q, _) = q_da_faixa(&m, &c, raio_do_app());
            let (ang, _) = pior_angulo(&m, &c, raio_do_app());
            let (finas, total) = lascas(&m, &c, raio_do_app(), LIMIAR_DA_LASCA);
            eprintln!(
                "{nome:<18} pente {pente:.3}  Q={q:+.4}  pior={ang:6.2}°  \
                 <{LIMIAR_DA_LASCA}°: {finas:3} de {total}"
            );
        }
    }
}

/// ⛔⛔⛔ **SONDA — O REGIME QUE O APP DE FACTO DÁ**, contra o que o gate da cena
/// escolheu. O report do dono foi *«não percebi diferença»*, e o gate estava
/// VERDE: a primeira coisa a medir é se ele corre no regime do artista.
#[test]
#[ignore = "sonda"]
fn diag_o_regime_do_app_contra_o_do_gate() {
    use crate::Camera3d;

    let peca = crate::scenes::pente::peca();
    let bounds = peca.bounds();
    let (vw, vh) = (1920.0f32, 1080.0f32);
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        ..Camera3d::default()
    };
    cam.frame(bounds, vw / vh);

    // O raio de fábrica do pincel, em PIXELS, convertido como o `armed_brush_on`
    // converte: através da câmera, no ponto onde o cursor aterra.
    let radius_px = ph2d_panel_sculpt3d::state::Sculpt3dUi::default().radius_px;
    let no_mundo =
        cam.world_radius_for_screen_px([0.0, 0.0, 1.0], radius_px, (vw as u32, vh as u32));

    // O alvo do refino que o passe de topologia usa com o `Detail` de fábrica.
    let detalhe = ph2d_panel_sculpt3d::state::Sculpt3dUi::default().dyn_detail;
    let area = peca.surface_area();
    let tris = ph2d_mesh::tris_for_detail(detalhe);
    let alvo = ph2d_mesh::edge_for_tri_count(area, tris);

    // A aresta média da peça, para se ler quantas cabem num raio.
    let pos = peca.positions();
    let (mut soma, mut n) = (0.0f64, 0usize);
    for f in peca.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (pos[vs[k] as usize], pos[vs[(k + 1) % vs.len()] as usize]);
            soma += f64::from((a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]));
            n += 1;
        }
    }
    let aresta = soma / n.max(1) as f64;

    eprintln!("--- o que o APP da' ---");
    eprintln!("  raio do pincel: {radius_px} px  ->  {no_mundo:.4} no mundo");
    eprintln!("  refino: Detail {detalhe:.2} -> {tris} triangulos -> aresta alvo {alvo:.4}");
    eprintln!("  aresta media da peca: {aresta:.4}");
    eprintln!(
        "  arestas por raio (antes do refino): {:.1}",
        f64::from(no_mundo) / aresta
    );
    eprintln!(
        "  arestas por raio (depois):          {:.1}",
        f64::from(no_mundo / alvo)
    );
    eprintln!("--- o que o GATE mede ---");
    eprintln!(
        "  raio {:.4} (derivado) · refino {:.4} (derivado do Detail da cena) \
         · arestas por raio {:.1}",
        raio_do_app(),
        alvo_do_refino(),
        f64::from(raio_do_app() / alvo_do_refino())
    );
}

/// ⛔⛔⛔ **SONDA — quantas ARESTAS POR RAIO o pente precisa para se ver.**
///
/// O report do dono (*«não percebi diferença»*) com o gate VERDE: ele corre a
/// `10` arestas por raio e o app dá `2,0`.
#[test]
#[ignore = "sonda"]
fn diag_o_pente_contra_as_arestas_por_raio() {
    for (nome, raio, alvo) in [
        ("o APP de fabrica", 0.1634f32, 0.0805f32),
        ("Detail 0,75", 0.1634, 0.0402),
        ("Detail 1,00", 0.1634, 0.0175),
        ("raio 2x, Detail 1", 0.3268, 0.0175),
        ("o GATE de hoje", 0.35, 0.035),
    ] {
        let mut q = [0.0f64; 2];
        let mut lasca = [0usize; 2];
        let (mut verts, mut n1) = (0usize, 0usize);
        let mut saidas: Vec<Vec<[f32; 3]>> = Vec::new();
        for (i, pente) in [0.0f32, 1.0].into_iter().enumerate() {
            let (m, c) = traco_com(pente, RUMOS[2].1, raio, alvo);
            let (qq, nn) = q_da_faixa(&m, &c, raio);
            q[i] = qq;
            n1 = nn;
            lasca[i] = lascas(&m, &c, raio, LIMIAR_DA_LASCA).0;
            verts = m.positions().len();
            saidas.push(m.positions().to_vec());
        }
        // ⭐ O que o OLHO lê não é a média: é quantos vértices o pente de facto
        // deslocou, e quanto.
        let movidos = saidas[0]
            .iter()
            .zip(&saidas[1])
            .filter(|(a, b)| a != b)
            .count();
        eprintln!(
            "{nome:<20} arestas/raio {:>4.1}  Q {:+.4} -> {:+.4} (D {:+.4})  \
             arestas na faixa {n1:>5}  mexidos {movidos:>5}  lascas {}  v={verts}",
            f64::from(raio / alvo),
            q[0],
            q[1],
            q[1] - q[0],
            lasca[1]
        );
    }
}

/// ⛔⛔⛔ **SONDA — DESENHA o arame, porque o produto desta wave é uma IMAGEM.**
///
/// O report do dono veio com FOTO e a frase *«não sei o que é para esperar»*.
/// ⚠️ **Toda régua desta cena é um NÚMERO** (o `Q`, a contagem, as lascas), e
/// nenhuma responde *«o que é que isto parece»*. Esta escreve dois `.ppm` — o
/// mesmo traço com o pente desligado e no tecto — para se OLHAR.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_desenha_o_arame -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_desenha_o_arame() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    for (nome, raio, alvo) in [
        ("app_fabrica", 0.1634f32, 0.0805f32),
        ("cena_detail1", raio_do_app(), alvo_do_refino()),
        ("pincel_3x", raio_do_app() * 3.0, alvo_do_refino()),
    ] {
        for pente in [0.0f32, 1.0] {
            let (m, c) = traco_com(pente, RUMOS[2].1, raio, alvo);
            let alvo_png = format!(
                "{dir}/pente_{nome}_{}.ppm",
                if pente > 0.0 { "on" } else { "off" }
            );
            desenha(&m, &c, raio, &alvo_png);
        }
        eprintln!("{nome}: raio {raio:.4} alvo {alvo:.4} -> {dir}/pente_{nome}_*.ppm");
    }
}

/// Um arame ortográfico, olhando de `+z`, centrado no percurso — em `.ppm`,
/// que se converte com `magick`.
fn desenha(m: &ph2d_mesh::Mesh, percurso: &[[f32; 3]], raio: f32, caminho: &str) {
    const N: usize = 900;
    // A janela é o percurso mais dois raios de cada lado — o enquadramento que
    // o artista teria se olhasse para o traço dele.
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for p in percurso {
        x0 = x0.min(p[0]);
        x1 = x1.max(p[0]);
        y0 = y0.min(p[1]);
        y1 = y1.max(p[1]);
    }
    let pad = raio * 2.0;
    let (cx, cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let meia = ((x1 - x0).max(y1 - y0) * 0.5 + pad).max(1e-4);
    let para_px = |p: [f32; 3]| -> (i32, i32) {
        let u = (p[0] - cx) / (2.0 * meia) + 0.5;
        let v = 0.5 - (p[1] - cy) / (2.0 * meia);
        ((u * N as f32) as i32, (v * N as f32) as i32)
    };

    let mut buf = vec![255u8; N * N * 3];
    let pos = m.positions();
    let linha = |a: (i32, i32), b: (i32, i32), buf: &mut Vec<u8>| {
        let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
        let (sx, sy) = (
            if a.0 < b.0 { 1 } else { -1 },
            if a.1 < b.1 { 1 } else { -1 },
        );
        let (mut x, mut y, mut err) = (a.0, a.1, dx + dy);
        loop {
            if x >= 0 && y >= 0 && (x as usize) < N && (y as usize) < N {
                let i = ((y as usize) * N + x as usize) * 3;
                buf[i] = 40;
                buf[i + 1] = 40;
                buf[i + 2] = 60;
            }
            if x == b.0 && y == b.1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    };
    for f in m.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let a = pos[vs[k] as usize];
            let b = pos[vs[(k + 1) % vs.len()] as usize];
            // Só a calota virada a nós, senão o arame de trás polui a leitura.
            if a[2] < 0.3 || b[2] < 0.3 {
                continue;
            }
            linha(para_px(a), para_px(b), &mut buf);
        }
    }
    let mut ficheiro = format!("P6\n{N} {N}\n255\n").into_bytes();
    ficheiro.extend_from_slice(&buf);
    std::fs::write(caminho, ficheiro).expect("escreve o ppm");
}

/// ⛔⛔⛔ **SONDA — o alinhamento ACUMULA com as passagens?**
///
/// A espec §3.1 diz que o pente **move vértices** e não troca arestas; quem
/// muda a ligação é o passe de topologia, que parte e funde **seguindo** as
/// posições. ⇒ a hipótese é que a grade se forma ao longo de VÁRIAS passagens.
/// Esta sonda mede e **desenha** um recorte legível.
#[test]
#[ignore = "sonda"]
fn diag_o_pente_acumula_com_as_passagens() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    // ⚠️ O regime do ORÁCULO: raio `0,35` e um alvo de refino da ORDEM da aresta
    // da peça — ali o passe **mantém** a densidade em vez de a afinar a cada
    // passagem, e é isso que deixa o alinhamento acumular.
    let (raio, alvo) = (0.35f32, 0.05f32);
    for passagens in [1usize, 4, 8, 16] {
        for pente in [0.0f32, 1.0] {
            let (m, c) = traco_repetido(pente, RUMOS[2].1, raio, alvo, passagens);
            let (q, n) = q_da_faixa(&m, &c, raio);
            let (ang, _) = pior_angulo(&m, &c, raio);
            eprintln!(
                "passagens {passagens:>2}  pente {pente:.1}  Q={q:+.4} (n={n:>5})  \
                 pior={ang:5.2}°  v={}",
                m.positions().len()
            );
            if pente > 0.0 || passagens == 1 {
                desenha_recorte(
                    &m,
                    &c,
                    raio,
                    &format!(
                        "{dir}/acum_{passagens:02}_{}.ppm",
                        if pente > 0.0 { "on" } else { "off" }
                    ),
                );
            }
        }
    }
}

/// O mesmo traço, `n` vezes por cima — como a mão faz.
fn traco_repetido(
    pente: f32,
    e: [f32; 2],
    raio: f32,
    alvo: f32,
    passagens: usize,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 0.05,
        pente,
        ..Brush::default()
    };
    let passo = raio * 0.15;
    let mut centros = Vec::new();
    for _ in 0..passagens {
        let mut stroke = SculptStroke::default();
        stroke.begin(&malha);
        let mut births = Vec::new();
        let mut region = ph2d_mesh::RegionScratch::default();
        centros.clear();
        for k in 0..24 {
            let u = -passo * 12.0 + passo * k as f32;
            let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
            centros.push(centro);
            let _ = ph2d_mesh::refine_in_sphere(
                &mut malha,
                centro,
                brush.radius,
                alvo,
                &mut births,
                &mut region,
            );
            stroke.grow_with(&malha, &births);
            stroke.dab(
                &mut malha,
                &brush,
                &Dab::at(centro, brush.radius, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
    }
    (malha, centros)
}

/// Um recorte de `3` raios à volta do meio do traço — grande o bastante para os
/// triângulos se lerem.
fn desenha_recorte(m: &ph2d_mesh::Mesh, percurso: &[[f32; 3]], raio: f32, caminho: &str) {
    const N: usize = 900;
    let meio = percurso[percurso.len() / 2];
    let meia = raio * 1.5;
    let para_px = |p: [f32; 3]| -> (i32, i32) {
        let u = (p[0] - meio[0]) / (2.0 * meia) + 0.5;
        let v = 0.5 - (p[1] - meio[1]) / (2.0 * meia);
        ((u * N as f32) as i32, (v * N as f32) as i32)
    };
    let mut buf = vec![255u8; N * N * 3];
    let pos = m.positions();
    let linha = |a: (i32, i32), b: (i32, i32), buf: &mut Vec<u8>| {
        let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
        let (sx, sy) = (
            if a.0 < b.0 { 1 } else { -1 },
            if a.1 < b.1 { 1 } else { -1 },
        );
        let (mut x, mut y, mut err) = (a.0, a.1, dx + dy);
        loop {
            if x >= 0 && y >= 0 && (x as usize) < N && (y as usize) < N {
                let i = ((y as usize) * N + x as usize) * 3;
                buf[i] = 30;
                buf[i + 1] = 30;
                buf[i + 2] = 50;
            }
            if x == b.0 && y == b.1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    };
    for f in m.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let a = pos[vs[k] as usize];
            let b = pos[vs[(k + 1) % vs.len()] as usize];
            if a[2] < 0.3 || b[2] < 0.3 {
                continue;
            }
            linha(para_px(a), para_px(b), &mut buf);
        }
    }
    let mut ficheiro = format!("P6\n{N} {N}\n255\n").into_bytes();
    ficheiro.extend_from_slice(&buf);
    std::fs::write(caminho, ficheiro).expect("escreve o ppm");
}

/// ⛔⛔⛔ **SONDA — e sobre uma peça que É uma GRADE, vê-se?**
///
/// A pergunta que sobra depois de o oráculo mostrar que a saída DELE é uma sopa
/// de triângulos: numa malha com grade (a esfera UV, que é o que o artista tem)
/// o pente **vira** linhas que existem — e isso pode ser visível.
#[test]
#[ignore = "sonda"]
fn diag_desenha_sobre_uma_grade() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    let e = RUMOS[2].1;
    for (nome, alvo) in [
        ("neutro", 0.0527f32),
        ("fino", alvo_do_refino()),
        ("grosso", 0.09f32),
    ] {
        for pente in [0.0f32, 1.0] {
            let mut malha = ph2d_mesh::shapes::sphere_with_triangles(12_000, 1.0);
            malha.triangulate();
            let raio = raio_do_app() * 2.0;
            let brush = Brush {
                verb: Verb::Draw,
                radius: raio,
                strength: 0.05,
                pente,
                ..Brush::default()
            };
            let mut stroke = SculptStroke::default();
            stroke.begin(&malha);
            let mut births = Vec::new();
            let mut region = ph2d_mesh::RegionScratch::default();
            let mut centros = Vec::new();
            let passo = raio * 0.15;
            for k in 0..24 {
                let u = -passo * 12.0 + passo * k as f32;
                let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
                centros.push(centro);
                let _ = ph2d_mesh::refine_in_sphere(
                    &mut malha,
                    centro,
                    raio,
                    alvo,
                    &mut births,
                    &mut region,
                );
                stroke.grow_with(&malha, &births);
                stroke.dab(
                    &mut malha,
                    &brush,
                    &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
                    Symmetry::default(),
                );
            }
            let (q, n) = q_da_faixa(&malha, &centros, raio);
            eprintln!("grade {nome} pente {pente:.1}  Q={q:+.4} (n={n})");
            desenha_recorte(
                &malha,
                &centros,
                raio,
                &format!(
                    "{dir}/grade_{nome}_{}.ppm",
                    if pente > 0.0 { "on" } else { "off" }
                ),
            );
        }
    }
}
