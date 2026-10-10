//! **Os oráculos CORRIDOS** (doc 129 §5.1) — as fixtures de
//! `docs/Motion Nodes/ferramentas/lens_oracle/corre.sh` contra as leis desta crate. Cada gate tem
//! um CONTROLO: uma lei errada que ele TEM de reprovar, senão a tolerância não distingue nada.

use ph2d_lens::aberracao::{self, Aberracao};
use ph2d_lens::cubo::Cubo;
use ph2d_lens::vinheta;

const W: i32 = 192;
const H: i32 = 128;

/// A entrada de `entrada.py cena`, a mesma fórmula (o gate `a_entrada_e_a_do_oraculo` confere).
fn entrada_cena(x: i32, y: i32) -> [u8; 3] {
    let mut c = [
        (x * 255 / (W - 1)) as u8,
        (y * 255 / (H - 1)) as u8,
        ((x + y) * 255 / (W + H - 2)) as u8,
    ];
    for (cx, cy, r, cor) in [
        (48, 40, 22, [230, 40, 30]),
        (120, 70, 28, [20, 180, 60]),
        (160, 30, 16, [40, 60, 230]),
    ] {
        if (x - cx).pow(2) + (y - cy).pow(2) <= r * r {
            c = cor;
        }
    }
    if (20..172).contains(&x) && (100..108).contains(&y) {
        c = [255, 255, 255];
    }
    c
}

fn canal(c: usize) -> impl Fn(i32, i32) -> f32 {
    move |x, y| f32::from(entrada_cena(x, y)[c]) / 255.0
}

/// As linhas de dados de uma fixture: `(x, y, valores)`.
fn fixture(nome: &str) -> Vec<(i32, i32, Vec<f32>)> {
    let caminho = format!("{}/tests/fixtures/{nome}", env!("CARGO_MANIFEST_DIR"));
    let texto = std::fs::read_to_string(&caminho).unwrap_or_else(|e| panic!("{caminho}: {e}"));
    let linhas: Vec<_> = texto
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let mut it = l.split_whitespace();
            let x = it.next().unwrap().parse().unwrap();
            let y = it.next().unwrap().parse().unwrap();
            (x, y, it.map(|v| v.parse().unwrap()).collect())
        })
        .collect();
    assert_eq!(linhas.len(), 48 * 32, "{nome}: a grelha de passo 4");
    linhas
}

#[test]
fn a_entrada_e_a_do_oraculo() {
    for (x, y, v) in fixture("entrada_cena.txt") {
        let c = entrada_cena(x, y);
        for k in 0..3 {
            assert!(
                (f32::from(c[k]) / 255.0 - v[k]).abs() < 1e-5,
                "({x},{y}) canal {k}"
            );
        }
    }
}

/// `(max, rms)` de uma lei de máscara contra uma fixture da vinheta.
fn erro_vinheta(
    nome: &str,
    sigma: f32,
    inset: [f32; 2],
    lei: impl Fn([f32; 2], [f32; 2], f32) -> f32,
) -> (f32, f32) {
    let semi = [W as f32 / 2.0 - inset[0], H as f32 / 2.0 - inset[1]];
    let (mut max, mut sq) = (0.0_f32, 0.0_f32);
    let linhas = fixture(nome);
    for (x, y, v) in &linhas {
        // O centro do `-vignette` é o pixel de ÍNDICE (96, 64): o centro do pixel (x, y) fica a
        // `(x - 96, y - 64)` dele.
        let e = (lei([(x - 96) as f32, (y - 64) as f32], semi, sigma) - v[0]).abs();
        max = max.max(e);
        sq += e * e;
    }
    (max, (sq / linhas.len() as f32).sqrt())
}

/// A vinheta é a elipse CONVOLVIDA com a gaussiana. Medido (10/10, a grelha inteira, em Python):
/// rms `0,009`–`0,020`, máximo `0,015`–`0,064` — o resto é a elipse que o ImageMagick RASTERIZA
/// antes de borrar (pior com `σ` pequeno), não a lei.
#[test]
fn a_vinheta_e_a_elipse_borrada_do_imagemagick() {
    for (nome, s, inset) in [
        ("vinheta_s8_24_16.txt", 8.0, [24.0, 16.0]),
        ("vinheta_s16_24_16.txt", 16.0, [24.0, 16.0]),
        ("vinheta_s4_40_10.txt", 4.0, [40.0, 10.0]),
        ("vinheta_s20_8_30.txt", 20.0, [8.0, 30.0]),
    ] {
        let (max, rms) = erro_vinheta(nome, s, inset, |q, semi, sigma| {
            vinheta::mascara(q, semi, 2.0, sigma)
        });
        eprintln!("{nome}: max {max:.4} rms {rms:.5}");
        assert!(rms < 0.025 && max < 0.08, "{nome}: max {max} rms {rms}");
    }
}

/// CONTROLO: `Φ(distância/σ)` — a aproximação que a convolução substituiu — reprova a borda larga
/// (medido: máximo `0,23`).
#[test]
fn o_controlo_da_vinheta_reprova_a_aproximacao_pela_distancia() {
    let pela_distancia = |q: [f32; 2], semi: [f32; 2], sigma: f32| {
        let rho = (q[0] / semi[0]).hypot(q[1] / semi[1]);
        if rho < 1e-6 {
            return 1.0;
        }
        vinheta::phi((1.0 - rho) * q[0].hypot(q[1]) / rho / sigma)
    };
    let (max, _) = erro_vinheta("vinheta_s20_8_30.txt", 20.0, [8.0, 30.0], pela_distancia);
    assert!(
        max > 0.08,
        "o controlo passou ({max}): a tolerância não distingue as leis"
    );
}

fn erro_aberracao(nome: &str, a: &Aberracao) -> f32 {
    let meia_diag = 0.5 * (W as f32).hypot(H as f32);
    let mut max = 0.0_f32;
    for (x, y, v) in fixture(nome) {
        let p = [x as f32 + 0.5, y as f32 + 0.5];
        let pos = aberracao::posicoes(p, [96.0, 64.0], meia_diag, a);
        for k in 0..3 {
            max = max.max((aberracao::bilinear(canal(k), W, H, pos[k]) - v[k]).abs());
        }
    }
    max
}

/// Ao bit de 16: as posições e a interpolação são as do `-distort SRT` por canal.
#[test]
fn a_aberracao_e_o_distort_por_canal_do_imagemagick() {
    let radial = Aberracao {
        amount: 0.04,
        linear: false,
        angle: 0.0,
    };
    let meia_diag = 0.5 * (W as f32).hypot(H as f32);
    let linear = aberracao::de_deslocamento([2.5, -1.5], meia_diag);
    for (nome, a) in [
        ("aberracao_radial.txt", radial),
        ("aberracao_linear.txt", linear),
    ] {
        let max = erro_aberracao(nome, &a);
        eprintln!("{nome}: max {max:.7}");
        assert!(max < 3e-5, "{nome}: max {max}");
    }
    // CONTROLO: o sentido trocado (o vermelho a encolher) reprova.
    let trocado = Aberracao {
        amount: -0.04,
        ..radial
    };
    assert!(erro_aberracao("aberracao_radial.txt", &trocado) > 0.05);
}

fn erro_lut(nome: &str, lei: impl Fn(&Cubo, [f32; 3]) -> [f32; 3]) -> f32 {
    let caminho = format!("{}/tests/fixtures/look.cube", env!("CARGO_MANIFEST_DIR"));
    let cubo = Cubo::ler(&std::fs::read_to_string(caminho).unwrap()).unwrap();
    let mut max = 0.0_f32;
    for (x, y, v) in fixture(nome) {
        let c = entrada_cena(x, y).map(|b| f32::from(b) / 255.0);
        let o = lei(&cubo, c);
        for k in 0..3 {
            max = max.max((o[k] - v[k]).abs());
        }
    }
    max
}

/// As duas interpolações batem com o `lut3d` do ffmpeg ao bit de 16 — e o CONTROLO: trocadas,
/// reprovam (a LUT mistura canais de propósito, medido `0,0036`).
#[test]
fn a_lut_e_o_lut3d_do_ffmpeg_nas_duas_interpolacoes() {
    let tet = erro_lut("lut_tetrahedral.txt", Cubo::tetraedrica);
    let tri = erro_lut("lut_trilinear.txt", Cubo::trilinear);
    eprintln!("tetraedrica {tet:.7} · trilinear {tri:.7}");
    assert!(tet < 3e-5 && tri < 3e-5, "tet {tet} tri {tri}");
    assert!(erro_lut("lut_tetrahedral.txt", Cubo::trilinear) > 1e-3);
    assert!(erro_lut("lut_trilinear.txt", Cubo::tetraedrica) > 1e-3);
}
