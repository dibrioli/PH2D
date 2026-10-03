//! Os gates da triplanar: o ORÁCULO do Blender por passo (pesos e vistas, depois a cor) e as
//! propriedades da normal.

use super::*;

struct Linha {
    objecto: String,
    caso: String,
    blend: f32,
    p: V3,
    n: V3,
    rgb: V3,
}

fn oraculo() -> Vec<Linha> {
    let txt = include_str!("../fixtures/oraculo_triplanar.csv");
    txt.lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            let f = |i: usize| c[i].parse::<f32>().expect("número");
            Linha {
                objecto: c[0].to_owned(),
                caso: c[1].to_owned(),
                blend: f(2),
                p: [f(3), f(4), f(5)],
                n: [f(6), f(7), f(8)],
                rgb: [f(9), f(10), f(11)],
            }
        })
        .collect()
}

/// A coordenada mapeada do Blender (`p_b · escala + desloca`, nos eixos DELE), de volta aos nossos.
fn mapeada(p: V3, escala: f32, desloca: V3) -> V3 {
    let pb = [p[0], -p[2], p[1]];
    let c: V3 = std::array::from_fn(|i| pb[i] * escala + desloca[i]);
    [c[0], c[2], -c[1]]
}

fn teste_colorida() -> Mipmaps {
    use ph2d_imageio::ImageImporter;
    let b = include_bytes!("../fixtures/teste_colorida.png");
    let ph2d_imageio::DecodedImage::Flat(img) = ph2d_imageio_png::PngImporter
        .import(b, &ph2d_imageio::ImportOpts::default())
        .expect("png")
    else {
        panic!("png plano");
    };
    let px: Vec<[u8; 4]> = img.pixels.iter().map(|p| p.0).collect();
    Mipmaps::de_rgba8(img.width, img.height, &px, img.width, true)
}

/// A soma das três vistas com uma leitura dada.
fn soma(n: V3, blend: f32, c: V3, ler: &dyn Fn(f32, f32) -> V3) -> V3 {
    let w = pesos(n, blend);
    let mut s = [0.0f32; 3];
    for (e, &we) in w.iter().enumerate() {
        if we <= 0.0 {
            continue;
        }
        let vi = vista(e, n[e]);
        let (u, v) = (dot(c, vi.t) + vi.k, dot(c, vi.b));
        let x = ler(u, v);
        for q in 0..3 {
            s[q] += x[q] * we;
        }
    }
    s
}

fn erro(a: V3, b: V3) -> f32 {
    (0..3).map(|q| (a[q] - b[q]).abs()).fold(0.0, f32::max)
}

/// ⭐⭐⭐ **Passo 1 — os PESOS e as VISTAS são os do Blender.** A rampa (`r = u`, `g = v`,
/// `b = u·v`) diz, por ponto, que `(u, v)` cada vista leu e com que peso.
///
/// Medido: `≤ 7e-7` em esfera e caixa, blends `0,2 · 0,5 · 1`. **Mutações que sangram:** trocar o
/// sinal de um `t`, o `k` de uma vista, a ordem dos eixos do Blender no [`pesos`], o regime de três.
#[test]
fn os_pesos_e_as_vistas_sao_os_do_blender() {
    let linhas: Vec<Linha> = oraculo()
        .into_iter()
        .filter(|l| l.caso == "pesos")
        .collect();
    assert!(linhas.len() > 2000, "a fixtura tem as duas peças");
    let mut pior = 0.0f32;
    for l in &linhas {
        let c = mapeada(l.p, 0.25, [0.5; 3]);
        let s = soma(l.n, l.blend, c, &|u, v| [u, v, u * v]);
        pior = pior.max(erro(s, l.rgb));
    }
    eprintln!("pesos: {} pontos, pior {pior:e}", linhas.len());
    assert!(pior <= 1.0e-5, "a triplanar diverge do Blender: {pior}");
}

/// ⭐⭐⭐ **Passo 2 — a COR é a do Blender** com a textura de teste colorida, repetida: os texels, o
/// centro deles, a origem em baixo e a curva sRGB.
///
/// ⚠️ O Cycles filtra nos BYTES e decodifica depois; o produto (a placa) decodifica e filtra em
/// linear. ⇒ a geometria prova-se com a leitura do Cycles (`≤ 2e-4`, medido `6e-5`), e a
/// divergência do produto é MEDIDA e declarada (handoff §2): p50 `~0,002`, máx `~0,26`.
#[test]
fn a_cor_e_a_do_blender() {
    let m = teste_colorida();
    let linhas: Vec<Linha> = oraculo().into_iter().filter(|l| l.caso == "cor").collect();
    assert!(linhas.len() > 2000, "a fixtura tem as duas peças");
    let crus = |k: u32, i: u32, j: u32| -> [f32; 4] {
        m.nivel(k)[(j * (m.lado() >> k) + i) as usize].map(|c| f32::from(c) / 255.0)
    };
    let cycles = |u: f32, v: f32| -> V3 {
        let b = m.bilinear_com(0, u, v, &crus);
        let d = |x: f32| {
            let x = f64::from(x);
            (if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }) as f32
        };
        [d(b[0]), d(b[1]), d(b[2])]
    };
    let produto = |u: f32, v: f32| -> V3 {
        let c = m.amostra(true, u, v, 0.0);
        [c[0], c[1], c[2]]
    };
    let (mut pior, mut div) = (0.0f32, Vec::new());
    for l in &linhas {
        let c = mapeada(l.p, 1.3, [0.17, 0.31, 0.05]);
        pior = pior.max(erro(soma(l.n, l.blend, c, &cycles), l.rgb));
        div.push(erro(soma(l.n, l.blend, c, &produto), l.rgb));
    }
    div.sort_by(f32::total_cmp);
    let (p50, max) = (div[div.len() / 2], div[div.len() - 1]);
    eprintln!(
        "cor: {} pontos, Cycles pior {pior:e} · produto p50 {p50} máx {max}",
        linhas.len()
    );
    assert!(
        pior <= 2.0e-4,
        "a geometria da cor diverge do Blender: {pior}"
    );
    assert!(max > 0.01, "o controlo: filtrar em linear É outra conta");
    assert!(
        p50 <= 0.01 && max <= 0.35,
        "a divergência declarada cresceu: {p50} / {max}"
    );
    let _ = linhas.iter().filter(|l| l.objecto == "caixa").count();
}

/// ⭐ **Um mapa PLANO não mexe na normal** (Whiteout reconstrói `n`), e relevo `0` também não.
#[test]
fn o_mapa_plano_devolve_a_normal_da_forma() {
    let plano = [128u8, 128, 255, 128];
    let cor = Mipmaps::de_rgba8(4, 4, &[[200, 100, 50, 255]; 16], 4, true);
    let nrh = Mipmaps::de_rgba8(4, 4, &[plano; 16], 4, false);
    let m = Mapas {
        cor,
        nrh,
        tem_normal: true,
        tem_rugosidade: true,
    };
    let mut pior = 0.0f32;
    for i in 0..400 {
        let a = i as f32 * 0.37;
        let b = i as f32 * 0.11;
        let n = normaliza([a.cos() * b.sin(), b.cos(), a.sin() * b.sin()]);
        for blend in [0.0, 0.3, 1.0] {
            let t = Triplanar {
                tamanho: 1.0,
                aspecto: 1.0,
                blend,
                relevo: 1.0,
            };
            let r = avalia(
                &m,
                &t,
                [0.3, -0.2, 0.7],
                n,
                [1e-3, 0.0, 0.0],
                [0.0, 1e-3, 0.0],
            );
            // 128/255 não é 0,5 exacto: a inclinação do texel plano é 1/255.
            pior = pior.max(erro(r.normal, n));
            let s: f32 = pesos(n, blend).iter().sum();
            assert!((s - 1.0).abs() < 1e-5, "os pesos somam 1: {s}");
        }
    }
    assert!(pior < 0.01, "o mapa plano moveu a normal: {pior}");
}

/// ⭐ **O mip de uma cor é a média em LINEAR** (preto + branco → 50 % de luz = sRGB `188`).
#[test]
fn o_mip_da_cor_faz_a_media_em_linear() {
    let px = [
        [0, 0, 0, 255],
        [255, 255, 255, 255],
        [255, 255, 255, 255],
        [0, 0, 0, 255],
    ];
    let m = Mipmaps::de_rgba8(2, 2, &px, 2, true);
    assert_eq!(m.contagem(), 2);
    assert_eq!(m.nivel(1)[0], [188, 188, 188, 255]);
    // ⛔ Preto e branco são pontos FIXOS da curva (a média crua também dá 188 — a prova de mutação
    // apanhou-o): um tom médio separa as duas contas (linear 146, crua 128).
    let meio = [
        [64, 64, 64, 255],
        [192, 192, 192, 255],
        [192, 192, 192, 255],
        [64, 64, 64, 255],
    ];
    assert_eq!(
        Mipmaps::de_rgba8(2, 2, &meio, 2, true).nivel(1)[0],
        [146, 146, 146, 255]
    );
    let n = Mipmaps::de_rgba8(2, 2, &px, 2, false);
    assert_eq!(
        n.nivel(1)[0],
        [128, 128, 128, 255],
        "um mapa de números faz a média crua"
    );
}

/// ⭐ **O pacote decodifica** — sete texturas `1024²` com os onze níveis.
#[test]
fn o_pacote_embutido_decodifica() {
    for e in Embarcada::TODAS {
        let m = e.mapas().unwrap_or_else(|x| panic!("{e:?}: {x}"));
        assert_eq!(m.cor.lado(), LADO, "{e:?}");
        assert_eq!(m.cor.contagem(), 11, "{e:?}");
        assert_eq!(m.nrh.contagem(), 11, "{e:?}");
        // A normal média de um mapa de tangente aponta para fora (z > 0 ⇒ r, g perto de 0,5).
        let topo = m.nrh.nivel(10)[0];
        assert!(
            (100..=156).contains(&topo[0]) && (100..=156).contains(&topo[1]),
            "{e:?}: a normal média {topo:?}"
        );
    }
}

fn png(b: &[u8], srgb: bool) -> Mipmaps {
    use ph2d_imageio::ImageImporter;
    let ph2d_imageio::DecodedImage::Flat(img) = ph2d_imageio_png::PngImporter
        .import(b, &ph2d_imageio::ImportOpts::default())
        .expect("png")
    else {
        panic!("png plano");
    };
    let px: Vec<[u8; 4]> = img.pixels.iter().map(|p| p.0).collect();
    Mipmaps::de_rgba8(img.width, img.height, &px, img.width, srgb)
}

/// ⭐⭐⭐ **Passo 3 — o MAPA DE NORMAL é lido como o Blender o lê** (OpenGL, verde = `+v`): nas faces
/// planas da caixa a Whiteout é a normal de tangente com a base `(t, b, a)` da vista, e o Blender dá
/// a verdade pelo Normal Map (MikkTSpace sobre uma UV igual à projecção do passo 1).
///
/// Medido `4,4e-5`. ⛔ Reconstruir o `z` de `xy` filtrados dava `0,078`: o `z` viaja no azul.
/// **Mutações que sangram:** o verde invertido (DirectX), trocar `t` e `b` na soma.
#[test]
fn o_mapa_de_normal_e_lido_como_o_blender() {
    let nrh = png(include_bytes!("../fixtures/teste_normal.png"), false);
    let m = Mapas {
        cor: Mipmaps::de_rgba8(1, 1, &[[255; 4]], 1, true),
        nrh,
        tem_normal: true,
        tem_rugosidade: false,
    };
    let t = Triplanar {
        tamanho: 1.0,
        aspecto: 1.0,
        blend: 0.0,
        relevo: 1.0,
    };
    let linhas: Vec<Linha> = oraculo()
        .into_iter()
        .filter(|l| l.caso == "normal")
        .collect();
    assert!(
        linhas.len() > 300,
        "a fixtura tem a caixa: {}",
        linhas.len()
    );
    let (mut pior, mut longe) = (0.0f32, 0.0f32);
    for l in &linhas {
        let c = mapeada(l.p, 1.3, [0.17, 0.31, 0.05]);
        let r = avalia(&m, &t, c, l.n, [1e-6, 0.0, 0.0], [0.0, 1e-6, 0.0]);
        pior = pior.max(erro(r.normal, l.rgb));
        longe = longe.max(erro(l.n, l.rgb));
    }
    eprintln!(
        "normal: {} pontos, pior {pior:e} (o mapa afasta até {longe})",
        linhas.len()
    );
    assert!(longe > 0.2, "o controlo: o mapa inclina a normal");
    assert!(pior <= 2.0e-4, "a normal diverge do Blender: {pior}");
}
