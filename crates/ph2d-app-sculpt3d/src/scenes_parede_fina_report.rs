//! ⛔⛔⛔ **A REPRODUÇÃO DO REPORT DE 2026-09-19** — *«ainda não ficou bom»*, com
//! duas fotos: a frente limpa e as costas **RASGADAS**.
//!
//! ⚠️⚠️ **A fixtura é a da CENA e o gesto é o do ROTEIRO** — esta linha já pagou
//! cinco vezes por medir noutra peça, e a 2.ª foto mostra o pincel **muito maior**
//! que o da 1.ª (o anel amarelo cobre ~⅓ da barbatana ⇒ `R ≈ 0,65`), que é
//! exactamente o que o passo (1) do roteiro manda fazer.
//!
//! O que estas sondas respondem, por ordem:
//!
//! 1. **quanto** as costas ainda se movem, por raio de pincel e por distância à
//!    beira — a coluna que o gate da cena mede num ponto só;
//! 2. **porquê** — a razão `superfície/ar` dos vértices de trás que PASSAM, e o
//!    deslocamento LATERAL deles (a hipótese: com pincel grande a razão tende
//!    para `1` e a lei deixa de discriminar);
//! 3. **o rasgo** — faces viradas do avesso e a cinta da beira, que é o que
//!    produz as bandas brancas da foto e que nenhuma régua desta wave contava.

use super::parede_fina::{ESPESSURA, MEIO, barbatana};
use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// O pincel da 2.ª foto: o anel amarelo cobre ~⅓ do lado de `2,0`.
const RAIO_DA_FOTO: f32 = 0.65;

fn d3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Um traço de `n` dabs, como o dono faz — ⛔ **não um carimbo**: ele fez *«duas
/// ou três bossas»*, e cada bossa é um arrasto.
fn traco(repouso: &Mesh, de: [f32; 3], para: [f32; 3], n: usize, raio: f32, mascara: bool) -> Mesh {
    let mut m = repouso.clone();
    let b = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 1.0,
        surface_only: mascara,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    for k in 0..n {
        let t = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        let c = [
            de[0] + (para[0] - de[0]) * t,
            de[1] + (para[1] - de[1]) * t,
            de[2] + (para[2] - de[2]) * t,
        ];
        st.dab(
            &mut m,
            &b,
            &Dab::at(c, raio, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    m
}

/// Os índices de trás (os que nasceram em `z = −t/2`) e os da frente.
fn lados(repouso: &Mesh) -> (Vec<u32>, Vec<u32>) {
    let meia = ESPESSURA * 0.5;
    let mut frente = Vec::new();
    let mut costas = Vec::new();
    for (i, p) in repouso.positions().iter().enumerate() {
        if (p[2] - meia).abs() < 1e-5 {
            frente.push(i as u32);
        } else if (p[2] + meia).abs() < 1e-5 {
            costas.push(i as u32);
        }
    }
    (frente, costas)
}

/// ⚠️ **A régua do RASGO** — quantas faces apontam ao contrário da vizinhança.
///
/// *Uma bossa que atravessa é lisa; o que a foto mostra são bandas BRANCAS
/// serrilhadas, que é geometria virada.* Nenhuma régua desta wave contava isto.
fn faces_viradas(repouso: &Mesh, depois: &Mesh) -> usize {
    let mut n = 0usize;
    for f in depois.faces() {
        let idx = f.verts();
        if idx.len() < 3 {
            continue;
        }
        let nd = normal_da_face(depois.positions(), idx);
        let nr = normal_da_face(repouso.positions(), idx);
        let dot = nd[0] * nr[0] + nd[1] * nr[1] + nd[2] * nr[2];
        if dot < 0.0 {
            n += 1;
        }
    }
    n
}

fn normal_da_face(pos: &[[f32; 3]], idx: &[u32]) -> [f32; 3] {
    let a = pos[idx[0] as usize];
    let b = pos[idx[1] as usize];
    let c = pos[idx[2] as usize];
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-20);
    [n[0] / l, n[1] / l, n[2] / l]
}

/// ⭐ **(1) QUANTO** — a coluna do report, varrida em raio e em distância à beira.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_report_quanto_as_costas_ainda_se_movem() {
    let repouso = barbatana();
    let (frente, costas) = lados(&repouso);
    println!("\n== (1) o que as COSTAS ainda fazem, no gesto do roteiro ==");
    println!("   barbatana 2,0 × 2,0 · espessura {ESPESSURA} · traco de 6 dabs, forca 1,00\n");
    println!(
        "{:>6} {:>7} | {:>9} {:>9} {:>7} | {:>8} {:>8} | {:>7}",
        "R", "d", "frente", "costas", "%", "n_costas", "n_frente", "viradas"
    );
    println!("{}", "-".repeat(78));

    for raio in [0.20f32, 0.40, 0.65, 0.90] {
        for dist in [0.10f32, 0.25, 0.50] {
            let x = MEIO - dist;
            let m = traco(
                &repouso,
                [x, -0.25, ESPESSURA * 0.5],
                [x, 0.25, ESPESSURA * 0.5],
                6,
                raio,
                true,
            );
            let mut mf = 0.0f32;
            let mut nf = 0usize;
            for &i in &frente {
                let dd = d3(repouso.positions()[i as usize], m.positions()[i as usize]);
                mf = mf.max(dd);
                if dd > 1e-6 {
                    nf += 1;
                }
            }
            let mut mc = 0.0f32;
            let mut nc = 0usize;
            for &i in &costas {
                let dd = d3(repouso.positions()[i as usize], m.positions()[i as usize]);
                mc = mc.max(dd);
                if dd > 1e-6 {
                    nc += 1;
                }
            }
            let pct = if mf > 0.0 { 100.0 * mc / mf } else { 0.0 };
            println!(
                "{raio:>6.2} {dist:>7.2} | {mf:>9.4} {mc:>9.4} {pct:>6.1}% | {nc:>8} {nf:>8} | {:>7}",
                faces_viradas(&repouso, &m)
            );
        }
    }
}

/// ⭐⭐ **(2) PORQUÊ** — a razão dos vértices de TRÁS que passam, contra o
/// deslocamento LATERAL deles.
///
/// ⚠️ **A hipótese a abater:** para um ponto de trás a `L` de lado, o ar mede
/// `√(L² + t²)` e a superfície mede `2d + t + L` ⇒ **com `L ≫ t` a razão tende
/// para `1`** e a lei deixa de discriminar. Se for isso, *um pincel grande
/// derrota a lei por construção*, e o `RAZAO_MAXIMA` não é afinável.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_report_porque_a_razao_deixa_de_discriminar() {
    let repouso = barbatana();
    let (_, costas) = lados(&repouso);
    let raio = RAIO_DA_FOTO;
    let dist = 0.25f32;
    let x = MEIO - dist;
    let centro = [x, 0.0, ESPESSURA * 0.5];
    let m = traco(&repouso, centro, centro, 1, raio, true);

    // A distância pela SUPERFÍCIE, pelo mesmo passeio por arestas que a lei usa.
    let sup = distancias_por_arestas(&repouso, centro);

    println!("\n== (2) os vertices de TRAS que PASSAM, por deslocamento lateral ==");
    println!("   R = {raio:.2} · d = {dist:.2} · UM dab\n");
    println!(
        "{:>10} {:>8} {:>9} {:>9} {:>8} {:>9}",
        "lateral", "n", "sup med", "ar med", "razao", "mov med"
    );
    println!("{}", "-".repeat(60));

    let baldes = [0.0f32, 0.10, 0.20, 0.30, 0.45, 0.60, 0.80];
    for w in baldes.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        let (mut n, mut ssup, mut sar, mut smov) = (0usize, 0.0f64, 0.0f64, 0.0f64);
        for &i in &costas {
            let p = repouso.positions()[i as usize];
            let lat = ((p[0] - centro[0]).powi(2) + (p[1] - centro[1]).powi(2)).sqrt();
            if lat < lo || lat >= hi {
                continue;
            }
            let mov = d3(p, m.positions()[i as usize]);
            if mov <= 1e-6 {
                continue;
            }
            let s = sup[i as usize];
            if !s.is_finite() {
                continue;
            }
            n += 1;
            ssup += f64::from(s);
            sar += f64::from(d3(p, centro));
            smov += f64::from(mov);
        }
        if n == 0 {
            println!(
                "{lo:>5.2}–{hi:<4.2} {n:>8} {:>9} {:>9} {:>8} {:>9}",
                "-", "-", "-", "-"
            );
            continue;
        }
        let (ms, ma, mm) = (ssup / n as f64, sar / n as f64, smov / n as f64);
        println!(
            "{lo:>5.2}–{hi:<4.2} {n:>8} {ms:>9.4} {ma:>9.4} {:>8.2} {mm:>9.4}",
            ms / ma.max(1e-9)
        );
    }
}

/// Dijkstra por arestas a partir do vértice mais perto de `centro` — a MESMA lei
/// de caminho que a máscara corre (⚠️ com o viés `√2` dela, de propósito: a
/// sonda tem de medir o que o produto mede, não o ideal).
fn distancias_por_arestas(mesh: &Mesh, centro: [f32; 3]) -> Vec<f32> {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    #[derive(PartialEq)]
    struct No(f32, u32);
    impl Eq for No {}
    impl Ord for No {
        fn cmp(&self, o: &Self) -> Ordering {
            o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
        }
    }
    impl PartialOrd for No {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }

    let pos = mesh.positions();
    let semente = pos
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            d3(**a, centro)
                .partial_cmp(&d3(**b, centro))
                .unwrap_or(Ordering::Equal)
        })
        .map_or(0u32, |(i, _)| i as u32);

    let viz = &mesh.adjacency().vert_verts;
    let mut d = vec![f32::INFINITY; pos.len()];
    let mut fila = BinaryHeap::new();
    d[semente as usize] = 0.0;
    fila.push(No(0.0, semente));
    while let Some(No(du, u)) = fila.pop() {
        if du > d[u as usize] {
            continue;
        }
        for &v in viz.neighbours(u as usize) {
            let nd = du + d3(pos[u as usize], pos[v as usize]);
            if nd < d[v as usize] {
                d[v as usize] = nd;
                fila.push(No(nd, v));
            }
        }
    }
    d
}

/// ⭐⭐⭐ **(3) O RASGO** — onde vivem as faces viradas, e se é a CINTA da beira.
///
/// ⚠️ A cinta da barbatana é um anel de **um quad de altura**: `0,06` de alto por
/// `0,025` de largo, ou seja **aspecto `2,4`** e a coisa mais fina da peça. Um
/// pincel grande que a apanhe puxa aqueles quads para fora do plano deles.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_report_onde_esta_o_rasgo() {
    let repouso = barbatana();
    let raio = RAIO_DA_FOTO;
    println!("\n== (3) as faces VIRADAS: quantas, e onde ==");
    println!("   R = {raio:.2} · traco de 6 dabs, forca 1,00\n");
    println!(
        "{:>7} | {:>8} {:>9} {:>9} | {:>11}",
        "d", "viradas", "na cinta", "na chapa", "mov costas"
    );
    println!("{}", "-".repeat(56));

    let meia = ESPESSURA * 0.5;
    for dist in [0.05f32, 0.10, 0.15, 0.25, 0.40] {
        let x = MEIO - dist;
        let m = traco(&repouso, [x, -0.25, meia], [x, 0.25, meia], 6, raio, true);
        let (mut cinta, mut chapa) = (0usize, 0usize);
        for f in m.faces() {
            let idx = f.verts();
            if idx.len() < 3 {
                continue;
            }
            let nd = normal_da_face(m.positions(), idx);
            let nr = normal_da_face(repouso.positions(), idx);
            if nd[0] * nr[0] + nd[1] * nr[1] + nd[2] * nr[2] >= 0.0 {
                continue;
            }
            // Uma face da CINTA tem vértices dos DOIS lados no repouso.
            let tem_frente = idx
                .iter()
                .any(|&i| (repouso.positions()[i as usize][2] - meia).abs() < 1e-5);
            let tem_costas = idx
                .iter()
                .any(|&i| (repouso.positions()[i as usize][2] + meia).abs() < 1e-5);
            if tem_frente && tem_costas {
                cinta += 1;
            } else {
                chapa += 1;
            }
        }
        let (_, costas) = lados(&repouso);
        let mc = costas
            .iter()
            .map(|&i| d3(repouso.positions()[i as usize], m.positions()[i as usize]))
            .fold(0.0f32, f32::max);
        println!(
            "{dist:>7.2} | {:>8} {cinta:>9} {chapa:>9} | {mc:>11.4}",
            cinta + chapa
        );
    }
}
