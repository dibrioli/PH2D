//! Os DESENHADORES da cena `=49` — as sondas que escrevem `.ppm` para se OLHAR.
//!
//! ⛔⛔⛔ **Eles existem porque o produto desta wave é uma IMAGEM, e as réguas
//! todas são números.** Os dois reports do dono sobre esta cena vieram com FOTO
//! (*«não sei o que é para esperar»* · *«pouca ou nenhuma diferença»*), e nas
//! duas vezes o que decidiu foi pôr os dois lados do controlo lado a lado e
//! olhar. *Uma linha que só sabe medir aprova o que o dono reprova.*
//!
//! ⛔ **Saíram do irmão [`super`] por TECTO DE LOC** (`735` contra `700`), e o
//! corte é por RESPONSABILIDADE — ali ficam as réguas exploratórias, aqui os
//! desenhadores. O cabeçalho daquele ficheiro já nomeava as duas.
//!
//! ⚠️ **O nome acaba em `_tests.rs` de propósito:** a classificação da família é
//! DERIVADA, e um ficheiro compilado só sob `cfg(test)` com outro nome passaria
//! a ser lido como PRODUTO pelas réguas de arquitectura.

use super::*;

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
