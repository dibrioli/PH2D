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
pub(crate) fn desenha(m: &ph2d_mesh::Mesh, percurso: &[[f32; 3]], raio: f32, caminho: &str) {
    desenha_com(m, percurso, raio, caminho, false);
}

/// ⭐⭐⭐⭐ **O mesmo arame, com as FILEIRAS realçadas.**
///
/// ⛔⛔⛔ **Ele existe porque eu OLHEI e disse o contrário do que a régua diz.**
/// Em 20/09 olhei um recorte do arame e escrevi *«são RETALHOS, não fileiras»*;
/// a régua da [`ph2d_sculpt3d::medida_da_fileira`], escrita a seguir e pregada
/// entre uma grade verdadeira e uma malha sacudida, lê fileiras de **`12`–`23`**
/// arestas contra `2`–`4` do controlo.
///
/// *Duas testemunhas em desacordo sobre a MESMA malha é o achado, e quem
/// desempata é a imagem da própria grandeza:* aqui as arestas **ALINHADAS** (a
/// população que a régua encadeia) saem a **PRETO** e todas as outras a
/// cinzento claro.
///
/// ⭐⭐⭐⭐ **E a imagem decidiu contra mim:** à esquerda tracinhos partidos, à
/// direita **linhas contínuas de ponta a ponta do traço**. *Eu tinha olhado um
/// recorte do arame inteiro e chamado retalhos ao que era uma grade escondida no
/// meio das outras duas famílias de arestas.*
///
/// ⚠️ **O que sai a preto é a população ALINHADA e não as cadeias** — as cadeias
/// são uma estrutura sobre ela, e desenhar só as cadeias esconderia a aresta que
/// duas delas disputam.
pub(crate) fn desenha_fileiras(
    m: &ph2d_mesh::Mesh,
    percurso: &[[f32; 3]],
    raio: f32,
    caminho: &str,
) {
    desenha_com(m, percurso, raio, caminho, true);
}

fn desenha_com(
    m: &ph2d_mesh::Mesh,
    percurso: &[[f32; 3]],
    raio: f32,
    caminho: &str,
    realca: bool,
) {
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
    // As arestas que a régua da fileira encadeia, para o realce.
    let fileiras: std::collections::BTreeSet<(u32, u32)> = if realca {
        ph2d_sculpt3d::medida_da_fileira::arestas_das_fileiras(m, percurso, raio)
            .into_iter()
            .collect()
    } else {
        std::collections::BTreeSet::new()
    };
    let linha = |a: (i32, i32), b: (i32, i32), tom: [u8; 3], buf: &mut Vec<u8>| {
        let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
        let (sx, sy) = (
            if a.0 < b.0 { 1 } else { -1 },
            if a.1 < b.1 { 1 } else { -1 },
        );
        let (mut x, mut y, mut err) = (a.0, a.1, dx + dy);
        loop {
            if x >= 0 && y >= 0 && (x as usize) < N && (y as usize) < N {
                let i = ((y as usize) * N + x as usize) * 3;
                buf[i] = tom[0];
                buf[i + 1] = tom[1];
                buf[i + 2] = tom[2];
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
    // ⚠️ **As claras primeiro e as escuras por cima** — na ordem das faces, uma
    // aresta comum apagaria a fileira que passa por ela.
    for passagem in 0..2 {
        for f in m.faces() {
            let vs = f.verts();
            for k in 0..vs.len() {
                let (ia, ib) = (vs[k], vs[(k + 1) % vs.len()]);
                let (a, b) = (pos[ia as usize], pos[ib as usize]);
                // Só a calota virada a nós, senão o arame de trás polui a leitura.
                if a[2] < 0.3 || b[2] < 0.3 {
                    continue;
                }
                let na_fileira = fileiras.contains(&(ia.min(ib), ia.max(ib)));
                if realca && na_fileira != (passagem == 1) {
                    continue;
                }
                if !realca && passagem == 1 {
                    continue;
                }
                let tom = if realca && !na_fileira {
                    [205, 205, 215]
                } else {
                    [20, 20, 40]
                };
                linha(para_px(a), para_px(b), tom, &mut buf);
            }
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
