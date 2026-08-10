//! **AS SONDAS do campo** — instrumentos, não gates.
//!
//! Irmão do [`super::tests`], e a linha do corte é o que cada arquivo AFIRMA:
//! lá moram asserções sobre o produto, aqui medições que respondem *"por onde a
//! onda entra?"*. Todas são `#[ignore]`: elas imprimem, não julgam.
//!
//! # ⚠️ Por que o oráculo aqui é o ENROLAMENTO, e não algo mais simples
//!
//! A caçada ao vazamento do flood fill queimou quatro oráculos, e os quatro
//! erraram **no mesmo lugar** — na casca, que é justamente onde a pergunta
//! importa:
//!
//! | tentativa | como degenera |
//! |---|---|
//! | a esfera ideal (`r = 1`) | a malha é um poliedro **inscrito** e mergulha abaixo disso no meio de cada face |
//! | força bruta com o `ray_hit` | é o mesmo algoritmo que **constrói** a parede, dos dois lados: razão entre dois doentes |
//! | paridade em `+x` da origem | `(1,0,0)` é um **VÉRTICE** desta esfera: o raio acerta várias faces e a paridade sai par, dizendo *"fora"* sobre o CENTRO |
//! | paridade oblíqua | sã longe da casca, **indefinida** numa célula com `dist = 0` |
//!
//! O **número de enrolamento generalizado** (Jacobson, Kavan & Sorkine 2013) não
//! tem esse modo de falha: ele é uma função **contínua**, vale ~1 dentro, ~0
//! fora, e **~0,5 apenas EXATAMENTE sobre a superfície** — um conjunto de medida
//! zero. ⚠️ O controle mostrou que a transição é **NÍTIDA**: a 0,002 da casca
//! (um quinze avos de um triângulo) ele já satura. Não há faixa ambígua larga a
//! temer; o que ele não faz é escolher uma direção que possa roçar um vértice,
//! e é isso que o separa dos quatro anteriores.
//!
//! ⚠️ **E ele não vale nada sem o CONTROLE.** [`the_winding_oracle_is_sane`]
//! roda primeiro; sem ele esta seria a quinta resposta confiante e errada.

use super::*;
use ph2d_mesh::shapes;

/// A esfera de teste, fechada, e os triângulos dela como vértices crus.
fn sphere_tris() -> (Mesh, Vec<[[f32; 3]; 3]>) {
    let m = shapes::uv_sphere(96, 144, 1.0);
    let mut closed = Mesh::from_parts(m.positions().to_vec(), m.faces().to_vec()).unwrap();
    let _ = ph2d_mesh::fill_holes(&mut closed);
    let pos = closed.positions().to_vec();
    let mut idx = Vec::new();
    closed.triangle_indices(&mut idx);
    let tris = idx
        .iter()
        .map(|t| {
            [
                pos[t[0] as usize],
                pos[t[1] as usize],
                pos[t[2] as usize],
            ]
        })
        .collect();
    (closed, tris)
}

/// **O número de enrolamento generalizado em `p`.**
///
/// Soma o ângulo sólido com sinal que cada triângulo subtende, dividido por 4π.
/// O ângulo sólido sai da fórmula de **Van Oosterom & Strackee (1983)**, que é
/// a canônica e não precisa de raio nenhum — nenhuma direção a escolher, logo
/// nenhuma direção a degenerar.
///
/// ⚠️ Em `f64`, e não por gosto: são 27 mil `atan2` somados, e o sinal do
/// resultado é a resposta inteira.
fn winding_number(tris: &[[[f32; 3]; 3]], p: [f32; 3]) -> f64 {
    let mut total = 0.0f64;
    for t in tris {
        let v = |i: usize| {
            [
                f64::from(t[i][0]) - f64::from(p[0]),
                f64::from(t[i][1]) - f64::from(p[1]),
                f64::from(t[i][2]) - f64::from(p[2]),
            ]
        };
        let (a, b, c) = (v(0), v(1), v(2));
        let len = |u: [f64; 3]| (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let dot = |u: [f64; 3], w: [f64; 3]| u[0] * w[0] + u[1] * w[1] + u[2] * w[2];
        let (la, lb, lc) = (len(a), len(b), len(c));
        // O produto misto: o volume com sinal, que carrega a orientação.
        let num = a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);
        let den = la * lb * lc + dot(a, b) * lc + dot(a, c) * lb + dot(b, c) * la;
        total += 2.0 * num.atan2(den);
    }
    total / (4.0 * core::f64::consts::PI)
}

/// **CONTROLE — o oráculo responde o que se sabe de antemão?**
///
/// ⚠️ Isto roda ANTES de qualquer conclusão tirada dele. Um oráculo sem controle
/// é como as quatro tentativas anteriores viraram respostas confiantes e
/// erradas; aqui o ponto sobre a casca é o caso que importa, porque é onde os
/// outros quatro quebraram — e a resposta certa dele é **0,5**, não um lado.
#[test]
#[ignore = "sonda"]
fn the_winding_oracle_is_sane() {
    let (_, tris) = sphere_tris();
    eprintln!("\nenrolamento (1 = dentro, 0 = fora):");
    for (label, p) in [
        ("centro", [0.0f32, 0.0, 0.0]),
        ("fundo, no eixo x", [0.9, 0.0, 0.0]),
        ("fundo, num vertice", [0.0, 0.0, 0.9]),
        ("logo dentro", [0.0, 0.0, 0.99]),
        ("fora da casca", [0.0, 0.0, 1.01]),
        ("longe", [2.0, 0.0, 0.0]),
    ] {
        eprintln!("  {label:<18} w = {:+.4}", winding_number(&tris, p));
    }

    // ⚠️ **O caso que o desenho de fato usa, e que eu quase deixei sem
    // controle:** um ponto EM CIMA da malha. `[0,0,1]` não serve — o poliedro é
    // inscrito e mergulha abaixo de `r = 1`, então aquele ponto está FORA e
    // responde 0 corretamente. Quem está sobre a superfície é o centroide de um
    // triângulo de verdade, e a resposta dele tem de ser ~0,5.
    let mid = |t: &[[f32; 3]; 3]| {
        [
            (t[0][0] + t[1][0] + t[2][0]) / 3.0,
            (t[0][1] + t[1][1] + t[2][1]) / 3.0,
            (t[0][2] + t[1][2] + t[2][2]) / 3.0,
        ]
    };
    eprintln!("  -- centroides de triangulos REAIS (esperado ~0,5) --");
    for t in tris.iter().step_by(tris.len() / 4).take(4) {
        eprintln!("     w = {:+.4}", winding_number(&tris, mid(t)));
    }
    // E logo dentro / logo fora do MESMO triângulo, ao longo da normal dele.
    let t = &tris[tris.len() / 3];
    let c = mid(t);
    let n = {
        let e1 = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
        let e2 = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
        let k = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let l = (k[0] * k[0] + k[1] * k[1] + k[2] * k[2]).sqrt();
        [k[0] / l, k[1] / l, k[2] / l]
    };
    for (label, s) in [("um pouco DENTRO", -0.002f32), ("um pouco FORA", 0.002)] {
        let p = [c[0] + n[0] * s, c[1] + n[1] * s, c[2] + n[2] * s];
        eprintln!("     {label:<16} w = {:+.4}", winding_number(&tris, p));
    }
}

/// **SONDA — o FURO, com o oráculo que não degenera.**
///
/// Replica o flood fill e para no primeiro passo que vai de um `from`
/// inequivocamente FORA (`w < 0,1`) para um `to` inequivocamente DENTRO
/// (`w > 0,9`). ⚠️ **A faixa do meio é DESCARTADA de propósito**: uma célula com
/// `w ≈ 0,5` está sobre a casca, e é exatamente ali que os oráculos anteriores
/// inventaram um lado. Descartar é a diferença entre não saber e mentir.
///
/// ⚠️ A réplica é INSTRUMENTO, não oráculo, e ela confere que concorda com o
/// original na contagem antes que alguém acredite no caminho.
#[test]
#[ignore = "sonda"]
fn the_puncture_by_winding_number() {
    let (closed, tris) = sphere_tris();

    let truth = {
        let mut f = VoxelField::for_bounds(closed.bounds(), 151);
        f.voxelize(&closed);
        f.flood_fill()
    };
    assert_eq!(truth, 0, "151 deveria vazar");

    let mut f = VoxelField::for_bounds(closed.bounds(), 151);
    f.voxelize(&closed);

    let (rx, ry, rz) = (f.dims[0], f.dims[1], f.dims[2]);
    let rxy = rx * ry;
    let cells = f.dist.len();
    let pos = |c: usize| {
        let z = c / rxy;
        let rem = c - z * rxy;
        let y = rem / rx;
        let x = rem - y * rx;
        [
            f.min[0] + x as f32 * f.step,
            f.min[1] + y as f32 * f.step,
            f.min[2] + z as f32 * f.step,
        ]
    };
    // O enrolamento é caro (27 mil `atan2`), então ele é preguiçoso e cacheado.
    let mut w_cache: Vec<f32> = vec![f32::NAN; cells];
    let mut asked = 0usize;

    let mut outside = vec![false; cells];
    let mut stack: Vec<usize> = vec![0];
    outside[0] = true;
    let mut found = None;
    'flood: while let Some(cell) = stack.pop() {
        let z = cell / rxy;
        let rem = cell - z * rxy;
        let y = rem / rx;
        let x = rem - y * rx;
        let guarded = f.dist[cell] < f.step;
        for ax in 0..3 {
            for st in [-1isize, 1] {
                let (mut nx, mut ny, mut nz) = (x as isize, y as isize, z as isize);
                match ax {
                    0 => nx += st,
                    1 => ny += st,
                    _ => nz += st,
                }
                if nx < 0
                    || ny < 0
                    || nz < 0
                    || nx >= rx as isize
                    || ny >= ry as isize
                    || nz >= rz as isize
                {
                    continue;
                }
                let next = nx as usize + ny as usize * rx + nz as usize * rxy;
                if outside[next] {
                    continue;
                }
                if guarded {
                    if f.dist[next] == f32::INFINITY {
                        continue;
                    }
                    let owner = if st > 0 { cell } else { next };
                    if f.crossed[owner * 3 + ax] == 1 {
                        continue;
                    }
                }
                // Só perto da casca vale pagar o enrolamento.
                if f.dist[next] < 2.0 * f.step {
                    let w_of = |c: usize, cache: &mut Vec<f32>, n: &mut usize| -> f32 {
                        if cache[c].is_nan() {
                            cache[c] = winding_number(&tris, pos(c)) as f32;
                            *n += 1;
                        }
                        cache[c]
                    };
                    let w_to = w_of(next, &mut w_cache, &mut asked);
                    if w_to > 0.9 {
                        let w_from = w_of(cell, &mut w_cache, &mut asked);
                        if w_from < 0.1 {
                            found = Some((cell, next, ax, st, guarded, w_from, w_to));
                            break 'flood;
                        }
                    }
                }
                outside[next] = true;
                stack.push(next);
            }
        }
    }

    eprintln!("\n{asked} enrolamentos calculados");
    match found {
        None => eprintln!("  NENHUM furo inequivoco -- a onda entra pela faixa da casca"),
        Some((from, to, ax, st, guarded, wf, wt)) => {
            let owner = if st > 0 { from } else { to };
            eprintln!("  O FURO, eixo {ax} (passo {st}):");
            eprintln!(
                "    de   {:?}  w={wf:+.4}  dist={:.6}  guarded={guarded}",
                pos(from),
                f.dist[from]
            );
            eprintln!("    para {:?}  w={wt:+.4}  dist={:.6}", pos(to), f.dist[to]);
            eprintln!("    bit de travessia = {}", f.crossed[owner * 3 + ax]);
        }
    }
}

/// **SONDA — o que o raio devolve NA aresta do furo?**
///
/// A célula de destino do furo tem `dist = 0`: ela está EM CIMA da casca. Então
/// a travessia acontece essencialmente no EXTREMO do segmento, e a pergunta é
/// se o `h` cai dentro de `[0, step]` ou escapa por um epsilon.
#[test]
#[ignore = "sonda"]
fn what_the_ray_returns_at_the_puncture() {
    let (closed, tris) = sphere_tris();
    let mut f = VoxelField::for_bounds(closed.bounds(), 151);
    f.voxelize(&closed);

    // A aresta que a sonda do enrolamento nomeou.
    let from = [-0.079_602_66f32, -1.006_755, -0.079_602_66];
    let dir = [0.0f32, 1.0, 0.0];
    let edges: Vec<ph2d_mesh::TriEdges> = tris
        .iter()
        .map(|t| ph2d_mesh::TriEdges::new(t[0], t[1], t[2]))
        .collect();

    eprintln!("\nstep = {:.9}", f.step);
    let mut near = Vec::new();
    for (i, t) in edges.iter().enumerate() {
        if let Some(h) = t.ray_hit(from, dir) {
            // Tudo que chega perto da janela, inclusive o que escapa dela.
            if h > -0.5 * f.step && h < 1.5 * f.step {
                near.push((i, h));
            }
        }
    }
    near.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    eprintln!("  {} triangulos com acerto perto da janela [0, step]:", near.len());
    for (i, h) in near.iter().take(8) {
        let inside_window = (0.0..=f.step).contains(h);
        eprintln!(
            "    tri {i:>6}  h = {h:.9}  h/step = {:.6}  dentro da janela? {inside_window}",
            h / f.step
        );
    }
    if near.is_empty() {
        eprintln!("    NENHUM -- o segmento nao fura nada, e o furo e' de outra natureza");
    }
    // E a distancia real desta celula a' casca.
    let d = edges
        .iter()
        .map(|t| t.closest_to(from).0)
        .fold(f32::INFINITY, f32::min)
        .sqrt();
    eprintln!("  distancia real da celula 'from' a' casca = {d:.9} (step = {:.9})", f.step);
    eprintln!("  d < step? {}  <- se falso, o early-out do voxelizador pula esta celula", d < f.step);
}

/// **SONDA — o resíduo do TUBO ABERTO (2 de 361).**
///
/// ⚠️ A pergunta certa não é *"que tolerância faz o número sumir?"* — é se este
/// resíduo é o MESMO mecanismo (excesso de arredondamento num extremo) ou
/// outro. Aumentar o epsilon até o verde aparecer é ajustar o gate ao bug.
#[test]
#[ignore = "sonda"]
fn what_is_left_in_the_open_tube() {
    let m = ph2d_mesh::shapes_open::open_tube3();
    let mut closed = Mesh::from_parts(m.positions().to_vec(), m.faces().to_vec()).unwrap();
    let fill = ph2d_mesh::fill_holes(&mut closed);
    let e = closed.edges();
    let borders = (0..e.len() as u32).filter(|x| e.valence(*x) == 1).count();
    eprintln!("\ntubo: {} buracos tapados, {borders} arestas de beira restantes", fill.filled());

    for res in [280u32, 377, 279, 281] {
        let mut f = VoxelField::for_bounds(closed.bounds(), res);
        f.voxelize(&closed);
        let inside = f.flood_fill();
        eprintln!("  resolucao {res:>4}: dentro = {inside}");
    }

    // Numa resolução que vaza, quão perto da casca está a amostra mais próxima?
    let mut f = VoxelField::for_bounds(closed.bounds(), 280);
    f.voxelize(&closed);
    let min_d = f
        .dist
        .iter()
        .filter(|d| d.is_finite())
        .fold(f32::INFINITY, |a, b| a.min(*b));
    eprintln!(
        "  a 280: amostra mais proxima da casca = {min_d:.9}, step = {:.9}, razao = {:.2e}",
        f.step,
        min_d / f.step
    );
}
