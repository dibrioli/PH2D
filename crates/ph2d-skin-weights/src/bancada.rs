//! ⏱️ **A BANCADA (`--ignored`) — o padrão-ouro contra a lei que estava em produção.**
//!
//! Corre com:
//! `cargo test -p ph2d-skin-weights --lib -- --ignored --nocapture bancada`
//!
//! ⚠️ **A fixtura é a CENA do dono**, não um caso simpático: `512 × 320` px, três ossos deitados ao
//! meio, `25°` por junta — a mesma geometria que ele fotografou em 2026-09-15.

use super::{Handle, Options, bounded_biharmonic};
use ph2d_affine::Xform;
use ph2d_poly2d::Mesh2d;
use ph2d_skeleton::{Skin, SkinBone};

const LARG: f64 = 512.0;
const ALT: f64 = 320.0;
const OSSOS: usize = 3;
const GRAUS: f64 = 25.0;
/// A escala da câmera do smoke, medida no ecrã.
const PX_POR_METRO: f64 = 152.0;

fn grelha(cols: usize, rows: usize) -> Mesh2d {
    let mut m = Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: [LARG as u32, ALT as u32],
    };
    for j in 0..=rows {
        for i in 0..=cols {
            m.rest
                .push([LARG * i as f64 / cols as f64, ALT * j as f64 / rows as f64]);
        }
    }
    let id = |i: usize, j: usize| (j * (cols + 1) + i) as u32;
    for j in 0..rows {
        for i in 0..cols {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}

fn tr(x: f64, y: f64) -> Xform {
    Xform([1.0, 0.0, 0.0, 1.0, x, y])
}
fn rot(a: f64) -> Xform {
    let (s, c) = a.sin_cos();
    Xform([c, s, -s, c, 0.0, 0.0])
}

/// Os ossos em px de IMAGEM (o eixo ao meio), e as poses depois da dobra.
///
/// `so_o_ultimo` roda **apenas** o último osso — é a fixtura da régua da LOCALIDADE.
fn corrente_com(forca: f64, so_o_ultimo: bool) -> (Vec<Handle>, Vec<Xform>, Skin) {
    corrente_dobrada(forca, so_o_ultimo, GRAUS)
}

/// ⚠️ **A MESMA porta, com o ângulo escolhido** — e ela existe porque a 1.ª redacção da varredura da
/// folga escreveu uma SEGUNDA corrente e compôs as poses ao contrário (empurrou o `mundo` onde vai a
/// matriz de pele `rest⁻¹ ∘ mundo`). Ela leu `21 %` de dobra a `25°` onde o produto lê `0,00 %`.
/// *Uma segunda cadeia de transformações no mesmo ficheiro é uma segunda resposta à mesma pergunta.*
fn corrente_dobrada(forca: f64, so_o_ultimo: bool, graus: f64) -> (Vec<Handle>, Vec<Xform>, Skin) {
    let passo = LARG / OSSOS as f64;
    let eixo = ALT / 2.0;
    let mut mundo = Xform::IDENTITY;
    let (mut handles, mut poses, mut ossos) = (Vec::new(), Vec::new(), Vec::new());
    for k in 0..OSSOS {
        let x0 = passo * k as f64;
        handles.push(Handle {
            a: [x0, eixo],
            b: [x0 + passo, eixo],
        });
        let rest = tr(x0, eixo);
        let dobra = if so_o_ultimo {
            if k + 1 == OSSOS { graus } else { 0.0 }
        } else if k == 0 {
            0.0
        } else {
            graus
        };
        mundo = if k == 0 {
            tr(0.0, eixo)
        } else {
            rot(dobra.to_radians()).then(&tr(passo, 0.0)).then(&mundo)
        };
        poses.push(rest.inverse().expect("rest invertivel").then(&mundo));
        ossos.push(SkinBone::new(rest, passo, forca, mundo, Xform::IDENTITY).expect("osso"));
    }
    (handles, poses, Skin::new(ossos).expect("pele"))
}

fn corrente(forca: f64) -> (Vec<Handle>, Vec<Xform>, Skin) {
    corrente_com(forca, false)
}

/// Onde a lei BBW põe um ponto: `Σ w_j · M_j · p`, com `w` interpolado na malha.
fn pos_bbw(p: [f64; 2], pesos: &[f64], poses: &[Xform]) -> [f64; 2] {
    let mut out = [0.0, 0.0];
    for (j, &w) in pesos.iter().enumerate() {
        if w != 0.0 {
            let q = poses[j].apply(p);
            out[0] += w * q[0];
            out[1] += w * q[1];
        }
    }
    out
}

/// As quatro réguas de 2026-09-15, sobre uma lei qualquer.
struct Reguas {
    orfa: f64,
    faceta_px: f64,
    esticao_max: f64,
    circulo: f64,
    /// ⭐⭐⭐ **A LOCALIDADE, e é a régua que faltava.** Quanto a arte que pertence ao PRIMEIRO osso
    /// se mexe quando só o ÚLTIMO roda — em pixels de imagem.
    ///
    /// ⛔⛔ As outras três medem SUAVIDADE, e uma mistura larga demais ganha nelas **por
    /// construção**: espalhar a influência de cada osso pela arte inteira alisa tudo e destrói a
    /// separação. *Sem esta coluna, a lei mais errada do campo lê-se como a melhor.*
    vazamento_px: f64,
}

/// A arte do PRIMEIRO osso: a faixa de `x` dele, na altura do eixo.
fn zona_do_primeiro() -> Vec<[f64; 2]> {
    let passo = LARG / OSSOS as f64;
    let mut v = Vec::new();
    for i in 0..=10 {
        for j in 0..=6 {
            v.push([
                passo * 0.1 + passo * 0.8 * f64::from(i) / 10.0,
                ALT * f64::from(j) / 6.0,
            ]);
        }
    }
    v
}

fn mede(m: &Mesh2d, campo: &dyn Fn([f64; 2]) -> Option<[f64; 2]>) -> Reguas {
    let posadas: Vec<Option<[f64; 2]>> = m.rest.iter().map(|&p| campo(p)).collect();
    let orfa = posadas.iter().filter(|p| p.is_none()).count() as f64 / posadas.len() as f64;
    // Faceta: o campo no meio da aresta contra a recta entre as duas pontas posadas.
    let mut faceta = 0.0_f64;
    for t in &m.tris {
        for e in 0..3 {
            let (i, j) = (t[e] as usize, t[(e + 1) % 3] as usize);
            let (Some(pa), Some(pb)) = (posadas[i], posadas[j]) else {
                continue;
            };
            let meio = [
                (m.rest[i][0] + m.rest[j][0]) / 2.0,
                (m.rest[i][1] + m.rest[j][1]) / 2.0,
            ];
            let Some(v) = campo(meio) else { continue };
            let r = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
            faceta = faceta.max((v[0] - r[0]).hypot(v[1] - r[1]));
        }
    }
    // Esticão local: a razão dos valores singulares do jacobiano, por diferenças centrais.
    let h = 0.5;
    let mut esticao: f64 = 1.0;
    for i in 0..=20 {
        for j in 0..=12 {
            let (x, y) = (
                (LARG * i as f64 / 20.0).clamp(1.0, LARG - 1.0),
                (ALT * j as f64 / 12.0).clamp(1.0, ALT - 1.0),
            );
            let (Some(px), Some(mx), Some(py), Some(my)) = (
                campo([x + h, y]),
                campo([x - h, y]),
                campo([x, y + h]),
                campo([x, y - h]),
            ) else {
                continue;
            };
            let (a, c) = ((px[0] - mx[0]) / (2.0 * h), (px[1] - mx[1]) / (2.0 * h));
            let (b, d) = ((py[0] - my[0]) / (2.0 * h), (py[1] - my[1]) / (2.0 * h));
            let e1 = (a * a + b * b + c * c + d * d) / 2.0;
            let det = a * d - b * c;
            let disc = (e1 * e1 - det * det).max(0.0).sqrt();
            let s2 = (e1 - disc).max(0.0).sqrt();
            if s2 > 1e-12 {
                esticao = esticao.max((e1 + disc).max(0.0).sqrt() / s2);
            }
        }
    }
    // Uma circunferência de r = 40% da altura, ao centro.
    let (n, r) = (720, ALT * 0.4);
    let pts: Vec<[f64; 2]> = (0..n)
        .filter_map(|i| {
            let t = std::f64::consts::TAU * f64::from(i) / f64::from(n);
            campo([LARG / 2.0 + r * t.cos(), ALT / 2.0 + r * t.sin()])
        })
        .collect();
    let c = pts
        .iter()
        .fold([0.0, 0.0], |a, p| [a[0] + p[0], a[1] + p[1]]);
    let c = [c[0] / pts.len() as f64, c[1] / pts.len() as f64];
    let (mut rmin, mut rmax) = (f64::INFINITY, 0.0_f64);
    for p in &pts {
        let d = (p[0] - c[0]).hypot(p[1] - c[1]);
        rmin = rmin.min(d);
        rmax = rmax.max(d);
    }
    Reguas {
        vazamento_px: 0.0,
        orfa: orfa * 100.0,
        faceta_px: faceta * PX_POR_METRO / 100.0, // px de imagem → metros → px de ecrã
        esticao_max: esticao,
        circulo: rmax / rmin,
    }
}

/// ⭐⭐⭐ **O VAZAMENTO: quanto a arte do PRIMEIRO osso se mexe quando só o ÚLTIMO roda.**
///
/// ⛔⛔ Numa corrente de três, rodar a ponta **não pode** mover a raiz. É a separação que um rig
/// precisa de ter, e é o que a literatura nomeia como o defeito das leis por distância: *cross-
/// influence artifacts*. ⚠️ **Nenhuma das outras três réguas o vê** — elas medem suavidade, e
/// espalhar a influência pela arte inteira ALISA tudo enquanto destrói a separação.
fn vazamento(campo: &dyn Fn([f64; 2]) -> Option<[f64; 2]>) -> f64 {
    let mut pior = 0.0_f64;
    for p in zona_do_primeiro() {
        if let Some(q) = campo(p) {
            pior = pior.max((q[0] - p[0]).hypot(q[1] - p[1]));
        }
    }
    pior
}

/// O campo do *bump* euclidiano, para uma corrente dada.
fn campo_bump(pele: &Skin) -> impl Fn([f64; 2]) -> Option<[f64; 2]> + '_ {
    move |p: [f64; 2]| {
        let mut w = pele.scratch();
        pele.weights_at(p, &mut w).then(|| {
            let mut out = [0.0, 0.0];
            for (b, &peso) in pele.bones().iter().zip(w.iter()) {
                if peso != 0.0 {
                    let q = b.pose.apply(p);
                    out[0] += peso * q[0];
                    out[1] += peso * q[1];
                }
            }
            out
        })
    }
}

#[test]
#[ignore = "bancada: imprime a tabela, sem barra"]
fn bancada() {
    let m = grelha(24, 15);
    println!(
        "\nfixtura: {LARG}x{ALT} px, {OSSOS} ossos, {GRAUS}° por junta, {} triangulos",
        m.tris.len()
    );
    println!(
        "\n  {:<34} {:>8} {:>10} {:>11} {:>9} {:>11}",
        "lei", "orfa %", "faceta px", "esticao max", "CIRCULO", "VAZAMENTO"
    );
    let linha = |rot: &str, mut r: Reguas, vaz: f64| {
        r.vazamento_px = vaz;
        println!(
            "  {:<34} {:>8.2} {:>10.2} {:>11.3} {:>9.3} {:>11.1}",
            rot, r.orfa, r.faceta_px, r.esticao_max, r.circulo, r.vazamento_px
        );
    };

    // ── A lei que estava em produção ─────────────────────────────────────────────────────────
    for forca in [1.0_f64, 1.5, 2.0] {
        let (_, _, pele) = corrente(forca);
        let (_, _, so_ponta) = corrente_com(forca, true);
        linha(
            &format!("bump euclidiano (forca {forca:.1})"),
            mede(&m, &campo_bump(&pele)),
            vazamento(&campo_bump(&so_ponta)),
        );
    }

    // ── O padrão-ouro ────────────────────────────────────────────────────────────────────────
    let (handles, poses, _) = corrente(1.0);
    let (_, poses_ponta, _) = corrente_com(1.0, true);
    let t0 = std::time::Instant::now();
    let pesos = bounded_biharmonic(&m, &handles, Options::default()).expect("resolve");
    let custo = t0.elapsed();
    let campo = |p: [f64; 2]| interpola(&m, &pesos.por_vertice, p).map(|w| pos_bbw(p, &w, &poses));
    let campo_ponta =
        |p: [f64; 2]| interpola(&m, &pesos.por_vertice, p).map(|w| pos_bbw(p, &w, &poses_ponta));
    linha(
        "BOUNDED BIHARMONIC (padrao-ouro)",
        mede(&m, &campo),
        vazamento(&campo_ponta),
    );
    println!(
        "\n  o solver: {} vertices, {} ossos, {} presos, {} rondas, residuo {:.2e}, {:.1} ms",
        pesos.report.vertices,
        pesos.report.ossos,
        pesos.report.presos,
        pesos.report.rondas,
        pesos.report.residuo,
        custo.as_secs_f64() * 1e3
    );
    println!(
        "\n  VAZAMENTO = quanto a arte do 1.o osso se mexe (px de imagem) quando SO' a ponta roda.\n           O ideal e' ZERO: rodar a ponta nao pode mexer a raiz."
    );
}

#[test]
#[ignore = "bancada: a densidade e o relogio"]
fn bancada_densidade() {
    println!("\nO PADRAO-OURO contra a DENSIDADE da malha (a mesma cena):");
    println!(
        "  {:>12} {:>8} {:>10} {:>11} {:>9} {:>11} {:>9}",
        "malha", "tris", "faceta px", "esticao max", "CIRCULO", "VAZAMENTO", "ms"
    );
    for (cols, rows) in [(24usize, 15usize), (36, 22), (48, 30), (64, 40)] {
        let m = grelha(cols, rows);
        let (handles, poses, _) = corrente(1.0);
        let (_, poses_ponta, _) = corrente_com(1.0, true);
        let t0 = std::time::Instant::now();
        let Some(pesos) = bounded_biharmonic(&m, &handles, Options::default()) else {
            println!("  {cols}x{rows}: sem resposta");
            continue;
        };
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let campo =
            |p: [f64; 2]| interpola(&m, &pesos.por_vertice, p).map(|w| pos_bbw(p, &w, &poses));
        let campo_ponta = |p: [f64; 2]| {
            interpola(&m, &pesos.por_vertice, p).map(|w| pos_bbw(p, &w, &poses_ponta))
        };
        let r = mede(&m, &campo);
        println!(
            "  {:>12} {:>8} {:>10.2} {:>11.3} {:>9.3} {:>11.2} {:>9.0}",
            format!("{cols}x{rows}"),
            m.tris.len(),
            r.faceta_px,
            r.esticao_max,
            r.circulo,
            vazamento(&campo_ponta),
            ms
        );
    }
    println!(
        "\n  ⚠️ o relogio e' de uma build DEBUG, e este custo e' do BIND (uma vez), nao do quadro."
    );
}

/// Os pesos num ponto qualquer, por coordenadas baricêntricas. `None` fora da malha.
pub(super) fn interpola(m: &Mesh2d, por_vertice: &[Vec<f64>], p: [f64; 2]) -> Option<Vec<f64>> {
    for t in &m.tris {
        let (i, j, k) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let (a, b, c) = (m.rest[i], m.rest[j], m.rest[k]);
        let den = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        if den.abs() < f64::EPSILON {
            continue;
        }
        let l1 = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / den;
        let l2 = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / den;
        let l3 = 1.0 - l1 - l2;
        if l1 >= -1e-9 && l2 >= -1e-9 && l3 >= -1e-9 {
            let n = por_vertice[i].len();
            return Some(
                (0..n)
                    .map(|o| {
                        l1 * por_vertice[i][o] + l2 * por_vertice[j][o] + l3 * por_vertice[k][o]
                    })
                    .collect(),
            );
        }
    }
    None
}

/// A fracção da ÁREA virada do avesso, e a fracção da arte que é RÍGIDA.
fn dobra_e_rigidez(m: &Mesh2d, pesos: &[Vec<f64>], poses: &[Xform]) -> (f64, f64) {
    let p: Vec<[f64; 2]> = m
        .rest
        .iter()
        .enumerate()
        .map(|(v, &q)| pos_bbw(q, &pesos[v], poses))
        .collect();
    let (mut mau, mut tot) = (0.0, 0.0);
    for t in &m.tris {
        let a = |q: &[[f64; 2]]| {
            let (x, y, z) = (q[t[0] as usize], q[t[1] as usize], q[t[2] as usize]);
            (y[0] - x[0]) * (z[1] - x[1]) - (y[1] - x[1]) * (z[0] - x[0])
        };
        let (s0, s1) = (a(&m.rest), a(&p));
        tot += s0.abs();
        if s0 * s1 <= 0.0 {
            mau += s0.abs();
        }
    }
    // ⛔⛔ **RÍGIDO é «UM osso leva tudo», nunca «algum peso é zero».** A 1.ª redacção usou a
    // segunda, e com TRÊS ossos ela acusa `63 %` de uma malha perfeitamente misturada: um vértice
    // com `(0,5 · 0,5 · 0,0)` tem um peso zero e mistura dois ossos. *Um predicado que é quase
    // sempre verdadeiro não é um censo, é ruído.*
    let duros = pesos
        .iter()
        .filter(|w| w.iter().any(|v| *v >= 1.0 - 1e-9))
        .count();
    (100.0 * mau / tot, 100.0 * duros as f64 / pesos.len() as f64)
}

/// ⏱️⭐⭐⭐ **A FOLGA DA JUNTA, varrida na GEOMETRIA DA CENA** (`--ignored`).
///
/// ⛔⛔ **A mesa do oráculo é DOIS ossos com a arte `2,4×` mais alta que um osso é longo; a cena do
/// produto é TRÊS ossos com a arte mais LARGA que alta.** O joelho de um número que depende de
/// quantos ossos disputam um vértice tem de sair da segunda, não da primeira.
#[test]
#[ignore = "bancada: varre a folga da junta na cena, sem barra"]
fn bancada_folga_da_junta() {
    let m = grelha(48, 30);
    println!(
        "folga  presos  rigida   VAZAM  faceta      dobra por junta: 25°     45°     60°     90°"
    );
    for folga in [0.0_f64, 1.0, 2.0, 4.0, 8.0, 12.0, 16.0, 24.0] {
        let (handles, _, _) = corrente_dobrada(1.0, false, 25.0);
        let opts = Options {
            folga_da_junta: folga,
            ..Options::default()
        };
        let Some(w) = bounded_biharmonic(&m, &handles, opts) else {
            println!("{folga:<6} — nao resolve");
            continue;
        };
        print!("{folga:<6} {:>6}", w.report.presos);
        // ⭐⭐⭐ **O VAZAMENTO é a coluna que limita esta varredura.** Alargar a mistura é
        // exactamente o que faz um osso alcançar a arte do vizinho — o defeito que o padrão-ouro
        // entrou para curar. *Uma folga grande demais re-compra o borrão global pela porta do lado.*
        let (_, so_ultimo, _) = corrente_dobrada(1.0, true, GRAUS);
        let campo_bbw =
            |p: [f64; 2]| interpola(&m, &w.por_vertice, p).map(|ws| pos_bbw(p, &ws, &so_ultimo));
        let vaz = vazamento(&campo_bbw);
        let (_, poses_cena, _) = corrente_dobrada(1.0, false, GRAUS);
        let r = mede(&m, &|p| {
            interpola(&m, &w.por_vertice, p).map(|ws| pos_bbw(p, &ws, &poses_cena))
        });
        let mut rig = 0.0;
        let mut linha = String::new();
        for g in [25.0_f64, 45.0, 60.0, 90.0] {
            let (_, poses, _) = corrente_dobrada(1.0, false, g);
            let (d, rr) = dobra_e_rigidez(&m, &w.por_vertice, &poses);
            rig = rr;
            linha.push_str(&format!("  {d:>6.2}%"));
        }
        println!(
            "  {rig:>5.1}%  {vaz:>6.2}  {:>6.2}                      {linha}",
            r.faceta_px
        );
    }
}
