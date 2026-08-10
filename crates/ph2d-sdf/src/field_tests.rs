//! Gates do campo.
//!
//! O oráculo do sinal é **geométrico** — onde a esfera está, e onde ela não
//! está — nunca uma segunda escrita do flood fill: um espelho do algoritmo
//! concordaria com ele exatamente onde ele erra.

use super::*;
use ph2d_mesh::{shapes, shapes_open};

/// A célula mais próxima de um ponto do mundo.
fn cell_at(f: &VoxelField, p: [f32; 3]) -> usize {
    let d = f.dims();
    let o = f.origin();
    let idx: Vec<usize> = (0..3)
        .map(|a| (((p[a] - o[a]) / f.step()).round() as isize).clamp(0, d[a] as isize - 1) as usize)
        .collect();
    idx[0] + idx[1] * d[0] + idx[2] * d[0] * d[1]
}

fn sphere_field(res: u32) -> VoxelField {
    let m = shapes::uv_sphere(24, 32, 1.0);
    let mut f = VoxelField::for_bounds(m.bounds(), res);
    f.voxelize(&m);
    f.flood_fill();
    f
}

#[test]
fn the_field_is_negative_inside_the_sphere_and_positive_outside() {
    let f = sphere_field(32);
    let inside = f.distances()[cell_at(&f, [0.0, 0.0, 0.0])];
    assert!(inside < 0.0, "o centro da esfera saiu {inside}");

    // Um canto da caixa: fora por construção, porque a grade nasce com folga.
    let outside = f.distances()[0];
    assert!(outside > 0.0, "o canto da grade saiu {outside}");

    // E um ponto claramente fora do raio, mas ainda dentro da caixa.
    let corner = f.distances()[cell_at(&f, [0.95, 0.95, 0.95])];
    assert!(corner > 0.0, "fora da esfera saiu {corner}");
}

/// ⚠️ O gate que justifica o passo de tapar buracos do [`crate::remesh`]: uma
/// superfície com beira **não tem dentro**, e o campo inteiro sai positivo.
#[test]
fn an_open_surface_has_no_inside_at_all() {
    let m = shapes_open::open_tube3();
    let mut f = VoxelField::for_bounds(m.bounds(), 24);
    f.voxelize(&m);
    f.flood_fill();
    let negatives = f.distances().iter().filter(|d| **d < 0.0).count();
    assert_eq!(
        negatives, 0,
        "a onda tinha de entrar pelas bocas do tubo e rotular tudo como fora"
    );
}

/// ⚠️ **O campo é uma BANDA ESTREITA, não um SDF completo** — e quem consome
/// precisa saber disso. A distância só é escrita onde a caixa de algum triângulo
/// alcança, o que dá ~1,5 passos de cada lado da superfície; medido numa esfera
/// a 48 células, **36.308 de 148.877 células (24%) são finitas**. O resto é
/// `±INFINITY`, com o SINAL certo — que é tudo de que a extração precisa, porque
/// ela só olha onde o zero é cruzado.
///
/// ⚠️ E o oráculo é a distância à **MALHA**, por força bruta sobre os
/// triângulos — não à esfera ideal. A malha é um poliedro de 24×32 anéis, a
/// corda dele afunda, e a primeira versão deste gate reprovou o produto CERTO
/// por 1,57 passos comparando com a forma errada.
#[test]
fn the_distance_inside_the_band_is_the_distance_to_the_mesh() {
    let m = shapes::uv_sphere(24, 32, 1.0);
    let f = sphere_field(48);
    let (step, dims, o) = (f.step(), f.dims(), f.origin());

    let pos = m.positions();
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    let prepared: Vec<ph2d_mesh::TriEdges> = tris
        .iter()
        .map(|t| {
            ph2d_mesh::TriEdges::new(pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize])
        })
        .collect();

    // Uma amostra: a força bruta é O(células × triângulos), e 200 células já
    // atravessam a banda inteira.
    let mut checked = 0usize;
    let mut worst = 0.0f32;
    for n in (0..f.cell_count()).step_by(f.cell_count() / 200 + 1) {
        let d = f.distances()[n];
        if !d.is_finite() {
            continue;
        }
        let z = n / (dims[0] * dims[1]);
        let rem = n - z * dims[0] * dims[1];
        let y = rem / dims[0];
        let x = rem - y * dims[0];
        let p = [
            o[0] + x as f32 * step,
            o[1] + y as f32 * step,
            o[2] + z as f32 * step,
        ];
        let truth = prepared
            .iter()
            .map(|t| t.closest_to(p).0)
            .fold(f32::INFINITY, f32::min)
            .sqrt();
        worst = worst.max((d.abs() - truth).abs());
        checked += 1;
    }
    assert!(checked > 30, "só {checked} células na banda");
    // ⚠️ **Não é exato, e o desvio tem mecanismo:** o campo guarda o mínimo sobre
    // os triângulos cuja CAIXA alcança a célula, e o mais próximo de verdade
    // pode ser um triângulo diagonal que não a cobre — então o número
    // superestima um pouco. Medido: **0,0029 num passo de 0,0417 = 7% de um
    // passo**. Onde importa isso desaparece por construção: no cruzamento de
    // zero, o triângulo que cobre a célula É o mais próximo.
    assert!(
        worst < 0.1 * step,
        "pior erro {worst} sobre {checked} células (passo {step})"
    );
}

/// A outra metade da banda: fora dela o valor é infinito, e o SINAL continua
/// certo. É o que a extração consome, e é o que separa um campo útil de um campo
/// que só parece certo perto da casca.
#[test]
fn beyond_the_band_the_field_is_infinite_with_the_right_sign() {
    let f = sphere_field(48);
    let (step, dims, o) = (f.step(), f.dims(), f.origin());
    let (mut finite, mut core, mut shell) = (0usize, 0usize, 0usize);

    for (n, d) in f.distances().iter().enumerate() {
        let z = n / (dims[0] * dims[1]);
        let rem = n - z * dims[0] * dims[1];
        let y = rem / dims[0];
        let x = rem - y * dims[0];
        let p = [
            o[0] + x as f32 * step,
            o[1] + y as f32 * step,
            o[2] + z as f32 * step,
        ];
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();

        if d.is_finite() {
            finite += 1;
        } else if *d < 0.0 {
            core += 1;
            assert!(r < 1.0, "−infinito em r={r:.3}, que está FORA");
        } else {
            shell += 1;
            assert!(r > 0.9, "+infinito em r={r:.3}, que está DENTRO");
        }
    }
    // Os três têm de existir: se a banda ou o miolo colapsassem, o gate acima
    // ficaria verde sobre um campo que não descreve nada.
    assert!(finite > 1000, "só {finite} células com distância");
    assert!(core > 1000, "só {core} células de miolo");
    assert!(shell > 1000, "só {shell} células de casca");
}

/// A semente do flood fill é a célula 0, e ela **tem** de estar fora — é a folga
/// de 1,51 passos que garante isso. Sem essa garantia o campo sai do avesso.
#[test]
fn the_padding_keeps_the_seed_cell_clear_of_the_mesh() {
    let m = shapes::cube(2.0);
    let f = VoxelField::for_bounds(m.bounds(), 16);
    let o = f.origin();
    let b = m.bounds();
    for (a, (lo, org)) in b.min.iter().zip(o.iter()).enumerate() {
        let gap = lo - org;
        assert!(
            gap > f.step(),
            "eixo {a}: a folga é {gap}, menor que um passo ({})",
            f.step()
        );
    }
}

/// A união sai de graça: voxelizar duas malhas no MESMO campo dá o corpo das
/// duas, porque a distância guarda o menor. É isso que deixa um remesh fundir
/// uma cena inteira sem operação booleana nenhuma.
#[test]
fn voxelizing_two_meshes_gives_the_union_of_both() {
    let a = shapes::cube(1.0);
    let mut moved = shapes::cube(1.0);
    for p in moved.positions_mut() {
        p[0] += 1.4;
    }
    moved.rebuild();

    let bounds = ph2d_mesh::Aabb {
        min: [-0.6, -0.6, -0.6],
        max: [2.1, 0.6, 0.6],
    };
    let mut f = VoxelField::for_bounds(bounds, 32);
    f.voxelize(&a);
    f.voxelize(&moved);
    f.flood_fill();

    assert!(f.distances()[cell_at(&f, [0.0, 0.0, 0.0])] < 0.0, "cubo A");
    assert!(f.distances()[cell_at(&f, [1.4, 0.0, 0.0])] < 0.0, "cubo B");
    // E o vão entre eles continua sendo vão.
    assert!(f.distances()[cell_at(&f, [0.7, 0.0, 0.0])] > 0.0, "o vão");
}

/// **SONDA — por qual aresta a onda entra no corpo.**
///
/// ⚠️ A réplica do flood fill aqui é INSTRUMENTO, não oráculo: o oráculo é
/// geométrico (`|p| < 0,9` numa esfera de raio 1 está dentro, e ponto). Para o
/// espelho não me enganar, a sonda **confere que ele concorda com o original**
/// na contagem antes de acreditar no caminho que ele registrou.
#[test]
#[ignore = "sonda"]
fn by_which_edge_the_flood_enters_the_body() {
    let m = shapes::uv_sphere(96, 144, 1.0);
    let mut closed = Mesh::from_parts(m.positions().to_vec(), m.faces().to_vec()).unwrap();
    let _ = ph2d_mesh::fill_holes(&mut closed);

    let build = |res: u32| {
        let mut f = VoxelField::for_bounds(closed.bounds(), res);
        f.voxelize(&closed);
        f
    };

    let truth = {
        let mut f = build(151);
        f.flood_fill()
    };
    assert_eq!(truth, 0, "151 deveria vazar");

    let f = build(151);
    let (rx, ry, rz) = (f.dims[0], f.dims[1], f.dims[2]);
    let rxy = rx * ry;
    let cells = f.dist.len();

    let mut outside = vec![false; cells];
    let mut parent = vec![usize::MAX; cells];
    let mut stack: Vec<usize> = vec![0];
    outside[0] = true;
    while let Some(cell) = stack.pop() {
        let z = cell / rxy;
        let rem = cell - z * rxy;
        let y = rem / rx;
        let x = rem - y * rx;
        let guarded = f.dist[cell] < f.step;
        for ax in 0..3 {
            for step in [-1isize, 1] {
                let (mut nx, mut ny, mut nz) = (x as isize, y as isize, z as isize);
                match ax {
                    0 => nx += step,
                    1 => ny += step,
                    _ => nz += step,
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
                    let owner = if step > 0 { cell } else { next };
                    if f.crossed[owner * 3 + ax] == 1 {
                        continue;
                    }
                }
                outside[next] = true;
                parent[next] = cell;
                stack.push(next);
            }
        }
    }

    let reached = outside.iter().filter(|o| **o).count();
    assert_eq!(
        cells - reached,
        truth,
        "a réplica DIVERGIU do original — o caminho abaixo não descreveria o produto"
    );

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
    let radius = |c: usize| {
        let p = pos(c);
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
    };

    let deep = (0..cells)
        .find(|c| outside[*c] && radius(*c) < 0.9)
        .expect("nenhuma célula funda foi alcançada");

    let mut chain = vec![deep];
    let mut c = deep;
    while parent[c] != usize::MAX {
        c = parent[c];
        chain.push(c);
    }
    chain.reverse();

    eprintln!(
        "\nstep = {:.6}, cadeia de {} passos ate' uma celula em r={:.4}",
        f.step,
        chain.len(),
        radius(deep)
    );
    for w in chain.windows(2) {
        let (from, to) = (w[0], w[1]);
        if radius(from) >= 1.0 && radius(to) < 1.0 {
            let ax = if to.abs_diff(from) == 1 {
                0
            } else if to.abs_diff(from) == rx {
                1
            } else {
                2
            };
            let owner = from.min(to);
            eprintln!("  A TRAVESSIA, no eixo {ax}:");
            eprintln!(
                "    de   r={:.5}  dist={:9.6}  guarded={}",
                radius(from),
                f.dist[from],
                f.dist[from] < f.step
            );
            eprintln!("    para r={:.5}  dist={:9.6}", radius(to), f.dist[to]);
            eprintln!(
                "    bit de travessia do dono = {}",
                f.crossed[owner * 3 + ax]
            );
            return;
        }
    }
    eprintln!("  a cadeia nunca cruza r=1 -- a onda entrou por outro caminho");
}
