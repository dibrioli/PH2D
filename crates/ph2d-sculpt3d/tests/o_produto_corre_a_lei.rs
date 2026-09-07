//! ⭐⭐⭐ **O PRODUTO CORRE A LEI QUE A BANCADA PROVA** — a costura que faltava.
//!
//! ⛔⛔ **A `ph2d-cloth` reproduz o oráculo a `10⁻⁶` e nada provava que o PRODUTO
//! corre o mesmo programa.** Entre a bancada e a mão do artista há um adaptador —
//! a tradução `Brush → Pincel`, a ordem de visita derivada da malha, o anel-1 da
//! adjacência, o `δ` projectado, a máscara e o alpha — e cada uma dessas peças é
//! um sítio onde o produto pode entregar outra coisa com a suíte inteira verde.
//!
//! ⚠️ **E não é hipotético:** a sonda que comparou as duas geometrias mediu o
//! produto a deformar `1,96 · R` onde o oráculo deforma `0,94 · R`, no mesmo
//! caminho e com o mesmo raio. Este ficheiro é o instrumento que diz **se** e
//! **onde**: ele constrói a malha do oráculo, corre o caminho do oráculo pela
//! porta do produto (`SculptStroke::dab`), e compara com o que o alvo gravou.
//!
//! ⚠️ **As fixtures são DADOS, não expressão** (GPLv2 §0) — o mesmo estatuto que
//! a bancada da `ph2d-cloth` declara no cabeçalho dela.

use ph2d_mesh::{Face, Mesh};
use ph2d_sculpt3d::{Brush, ClothArea, ClothMode, Dab, SculptStroke, Symmetry, Verb};
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/cloth")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — a gémea da da `ph2d-cloth`.
fn inflar(nome: &str) -> String {
    let raw = std::fs::read(fixture_dir().join(nome)).unwrap_or_else(|e| panic!("{nome}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{nome}: nao e' gzip"
    );
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0x04 != 0 {
        let xlen = usize::from(raw[off]) | (usize::from(raw[off + 1]) << 8);
        off += 2 + xlen;
    }
    for bit in [0x08u8, 0x10] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
    }
    if flg & 0x02 != 0 {
        off += 2;
    }
    let bytes = miniz_oxide::inflate::decompress_to_vec(&raw[off..raw.len() - 8])
        .unwrap_or_else(|e| panic!("{nome}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

fn v3(campos: &[&str]) -> [f32; 3] {
    let n = |i: usize| campos[i].parse::<f32>().expect("numero");
    [n(0), n(1), n(2)]
}

/// A malha de repouso e as faces de uma superfície das fixtures.
fn malha(superficie: &str) -> Mesh {
    let mut pos = Vec::new();
    for l in inflar(&format!("{superficie}.repouso.txt.gz")).lines() {
        let c: Vec<&str> = l.split_whitespace().collect();
        if c.first().copied() == Some("v") {
            pos.push(v3(&c[1..]));
        }
    }
    let mut faces = Vec::new();
    for l in inflar(&format!("{superficie}.faces.txt.gz")).lines() {
        let c: Vec<&str> = l.split_whitespace().collect();
        if c.first().copied() != Some("f") {
            continue;
        }
        let idx: Vec<u32> = c[1..]
            .iter()
            .map(|t| t.parse::<u32>().expect("indice"))
            .collect();
        faces.push(match idx.len() {
            3 => Face::tri(idx[0], idx[1], idx[2]),
            4 => Face::quad(idx[0], idx[1], idx[2], idx[3]),
            n => panic!("face de {n} vertices"),
        });
    }
    Mesh::from_parts(pos, faces).expect("malha do oraculo")
}

/// O cabeçalho de um traço: o caminho do cursor e a malha DEPOIS.
struct Traco {
    caminho: Vec<[f32; 3]>,
    depois: Vec<[f32; 3]>,
    raio: f32,
    forca: f32,
}

fn traco(nome: &str) -> Traco {
    let texto = inflar(&format!("{nome}.deformado.txt.gz"));
    let (mut caminho, mut depois) = (Vec::new(), Vec::new());
    let (mut raio, mut forca) = (0.35f32, 1.0f32);
    for l in texto.lines() {
        let c: Vec<&str> = l.split_whitespace().collect();
        match c.first().copied() {
            Some("c") => caminho.push(v3(&c[1..])),
            Some("d") => depois.push(v3(&c[1..])),
            Some("raio") => raio = c[1].parse().expect("raio"),
            Some("forca") => forca = c[1].parse().expect("forca"),
            _ => {}
        }
    }
    Traco {
        caminho,
        depois,
        raio,
        forca,
    }
}

/// ⭐⭐⭐ **GATE — o produto reproduz o traço do oráculo, pela porta do artista.**
///
/// ⚠️ **A barra é a mesma do gate 15 da espec** (o pior erro por vértice sobre o
/// maior deslocamento do alvo), e a razão de não ser a resolução do ficheiro é
/// que o caminho do produto passa por um `f32` a cada dab — a malha vive em
/// `f32`, e a bancada corre tudo em `f64`.
///
/// ⛔ **O anti-vácuo é metade do gate:** um produto que não movesse nada daria
/// erro igual ao deslocamento do alvo, e um que movesse ao acaso também — por
/// isso as duas metades, o alcance e o erro.
/// **OS TRAÇOS que o produto sabe reproduzir pela porta do artista.**
///
/// ⚠️ **A lista é curta de propósito e diz porquê:** o produto só alcança o que a
/// tradução `Brush → Pincel` exprime, e o `Dab` do gesto tem UM olho — os traços
/// de esfera do corpus são de área *Dynamic* com o olho a mudar, e o Grab/Snake
/// Hook levam o `δ` TOTAL, que o `Dab::hooking` não exprime (ele é o incremento).
/// *Um gate que varresse as 78 fixtures mediria o arnês, não o adaptador.*
const TRACOS: [(&str, ph2d_sculpt3d::ClothMode, ClothArea); 5] = [
    (
        "plano_arrastar_radial_local_origem",
        ClothMode::Drag,
        ClothArea::Local,
    ),
    (
        "plano_arrastar_radial_global_origem",
        ClothMode::Drag,
        ClothArea::Global,
    ),
    (
        "plano_empurrar_radial_local_origem",
        ClothMode::Push,
        ClothArea::Local,
    ),
    (
        "plano_inflar_radial_local_origem",
        ClothMode::Inflate,
        ClothArea::Local,
    ),
    (
        "plano_apertar_linha_radial_local_origem",
        ClothMode::PinchPerpendicular,
        ClothArea::Local,
    ),
];

#[test]
fn o_produto_corre_a_lei_do_oraculo() {
    for (nome, modo, area) in TRACOS {
        um_traco(nome, modo, area);
    }
}

fn um_traco(nome: &str, modo: ClothMode, area: ClothArea) {
    /// ⭐ **A barra é a RESOLUÇÃO, não a do gate 15.**
    ///
    /// Medido nos cinco: `0,000003` a `0,000021` de erro absoluto, `≤ 10⁻⁴`
    /// relativo. ⛔ Deixá-la nos `0,13` do gate de paridade seria dar ao
    /// adaptador **quatro ordens de grandeza** de deriva antes de alguém reparar
    /// — e o que este ficheiro existe para medir é exactamente o adaptador, cuja
    /// lei já está provada do outro lado.
    ///
    /// ⚠️ Ela não desce mais porque o caminho do produto passa por um `f32` a
    /// cada dab (a malha vive em `f32`) enquanto a bancada corre tudo em `f64`.
    const BARRA: f32 = 1e-3;
    let t = traco(nome);
    let antes = malha("plano");
    let mut mesh = malha("plano");
    assert_eq!(mesh.vert_count(), t.depois.len(), "malha e alvo nao batem");
    let b = Brush {
        verb: Verb::Cloth,
        cloth_mode: modo,
        cloth_area: area,
        radius: t.raio,
        strength: t.forca,
        hardness: 0.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for (k, c) in t.caminho.iter().enumerate() {
        let passo = if k == 0 {
            [0.0; 3]
        } else {
            let p = t.caminho[k - 1];
            [c[0] - p[0], c[1] - p[1], c[2] - p[2]]
        };
        s.dab(
            &mut mesh,
            &b,
            &Dab::hooking(*c, b.radius, [0.0, 0.0, -1.0], passo),
            Symmetry::default(),
        );
    }
    let d = |a: [f32; 3], b: [f32; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    let max_alvo = (0..mesh.vert_count())
        .map(|v| d(antes.positions()[v], t.depois[v]))
        .fold(0.0f32, f32::max);
    let max_nosso = (0..mesh.vert_count())
        .map(|v| d(antes.positions()[v], mesh.positions()[v]))
        .fold(0.0f32, f32::max);
    let erro = (0..mesh.vert_count())
        .map(|v| d(mesh.positions()[v], t.depois[v]))
        .fold(0.0f32, f32::max);
    assert!(max_alvo > 0.1, "o alvo mal deformou ({max_alvo:.4})");
    assert!(
        max_nosso > 0.5 * max_alvo,
        "o produto deformou {max_nosso:.4} contra {max_alvo:.4} do alvo -- \
         alcance a menos de metade e' um pincel que nao corre a lei"
    );
    assert!(
        erro / max_alvo <= BARRA,
        "{nome} PELA PORTA DO PRODUTO: erro {erro:.6} = {:.5} relativo, contra a \
         barra {BARRA} (nosso {max_nosso:.4}, alvo {max_alvo:.4}) -- a bancada \
         reproduz este traco a 10^-6, logo o defeito esta' no ADAPTADOR",
        erro / max_alvo
    );
}
