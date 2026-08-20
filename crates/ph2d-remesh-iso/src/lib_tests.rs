//! **OS GATES DO REMESH ISOTRÓPICO** — a propriedade que o passe existe para dar.

use ph2d_mesh::shapes;

use super::{ALPHA, Report, remesh_isotropic, target_edge};

fn mean_edge(mesh: &ph2d_mesh::Mesh) -> f32 {
    let p = mesh.positions();
    let (mut sum, mut n) = (0.0f64, 0usize);
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            sum += f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
            n += 1;
        }
    }
    (sum / n.max(1) as f64) as f32
}

fn volume(mesh: &ph2d_mesh::Mesh) -> f64 {
    let p = mesh.positions();
    let mut vol = 0.0f64;
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
            vol += f64::from(a[0].mul_add(
                b[1].mul_add(c[2], -(b[2] * c[1])),
                a[1].mul_add(
                    b[2].mul_add(c[0], -(b[0] * c[2])),
                    a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                ),
            )) / 6.0;
        }
    }
    vol
}

/// ⭐ **A PROPRIEDADE INTEIRA: a saída não herda a densidade da entrada.**
///
/// ⚠️ **É o gate que a medição do oráculo pediu.** Oito vértices e treze mil têm
/// de sair no mesmo lugar — é isso que faz o pipeline a jusante deixar de
/// depender de como o artista deixou a malha. Sem este passe, o cubo devolvia
/// **malha vazia** no remesher.
///
/// ⚠️ **A RÉGUA é a razão para o alvo DA PRÓPRIA MALHA, e a primeira versão
/// deste gate errava nisso** — ela comparava arestas médias ABSOLUTAS entre
/// fixturas de caixas diferentes (o cubo tem diagonal `1,732` e a esfera
/// `3,464`), e acusava `2,01×` de dispersão sobre um passe que estava correto:
/// os dois estavam a **9 %** do respectivo alvo. *Uma régua que compara dois
/// números de unidades diferentes acusa o algoritmo pelo erro de quem mede.*
#[test]
fn the_output_density_does_not_depend_on_the_input_density() {
    let mut ratios: Vec<(String, f32, usize)> = Vec::new();
    for (name, mut mesh) in [
        ("cubo (8 v)", shapes::cube(1.0)),
        ("esfera 24x36 (830 v)", shapes::uv_sphere(24, 36, 1.0)),
        ("esfera 96x144 (13 682 v)", shapes::uv_sphere(96, 144, 1.0)),
    ] {
        let before = mesh.vert_count();
        let want = target_edge(&mesh, ALPHA);
        let r: Report = remesh_isotropic(&mut mesh, ALPHA);
        let got = mean_edge(&mesh);
        eprintln!(
            "[iso] {name}: {before} -> {} v em {} rodadas, aresta {got:.4} / alvo {want:.4} = {:.3}x",
            r.verts_after,
            r.rounds,
            got / want
        );
        assert!(
            r.verts_after > 0 && !mesh.faces().is_empty(),
            "{name}: o passe devolveu malha vazia"
        );
        ratios.push((name.to_string(), got / want, r.verts_after));
    }

    // (1) cada malha chega ao SEU alvo.
    for (name, ratio, _) in &ratios {
        assert!(
            (0.75..=1.35).contains(ratio),
            "{name}: a aresta media saiu {ratio:.3}x o alvo -- o passe nao converge para o que \
             lhe foi pedido"
        );
    }
    // (2) e as três chegam ao MESMO múltiplo do alvo — é isso que quer dizer
    //     "a densidade da saída não depende da entrada".
    let (lo, hi) = ratios
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), e| (l.min(e.1), h.max(e.1)));
    assert!(
        hi / lo < 1.15,
        "as razoes ao alvo saem entre {lo:.3}x e {hi:.3}x ({:.2}x de dispersao) -- a saida ainda \
         HERDA a densidade da entrada: {ratios:?}",
        hi / lo
    );
    // (3) ⭐ E o caso DIRETO: duas esferas da MESMA caixa, com 16x de diferenca
    //     na entrada, têm de sair com a mesma contagem.
    let (small, big) = (ratios[1].2 as f32, ratios[2].2 as f32);
    let spread = small.max(big) / small.min(big);
    assert!(
        spread < 1.15,
        "830 vertices deram {small} e 13 682 deram {big} ({spread:.2}x) -- a entrada ainda manda"
    );
}

/// ⭐ **O ALVO É DA CAIXA, e a escala prova-o.**
///
/// ⚠️ Duas esferas de raios diferentes têm de sair com arestas na mesma razão dos
/// raios. Um alvo derivado da entrada não teria esta propriedade.
#[test]
fn the_target_scales_with_the_bounding_box() {
    let small = target_edge(&shapes::uv_sphere(24, 36, 1.0), ALPHA);
    let big = target_edge(&shapes::uv_sphere(24, 36, 4.0), ALPHA);
    assert!(
        (big / small - 4.0).abs() < 1.0e-3,
        "o alvo nao escalou com a caixa: {small:.5} e {big:.5} ({:.3}x, esperado 4)",
        big / small
    );
}

/// ⭐ **A FORMA SOBREVIVE** — a reprojeção é o que separa remalhar de alisar.
///
/// ⚠️ **Sem a projeção de volta à superfície ORIGINAL o Laplaciano encolhe**, e o
/// mecanismo já está medido e registrado (recusa 13 do ADR-0160). A barra é o
/// volume, que é a grandeza que um encolhimento move primeiro.
#[test]
fn the_shape_survives_the_remesh() {
    for (name, mut mesh) in [
        ("esfera", shapes::uv_sphere(48, 64, 1.0)),
        ("toro", shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let before = volume(&mesh);
        remesh_isotropic(&mut mesh, ALPHA);
        let after = volume(&mesh);
        eprintln!("[iso] {name}: volume {before:.4} -> {after:.4}");
        assert!(
            (after - before).abs() < 0.05 * before.abs(),
            "{name}: o volume andou {:.1}% ({before:.4} -> {after:.4}) -- a reprojecao nao esta' \
             a segurar o Laplaciano",
            100.0 * (after - before).abs() / before.abs()
        );
    }
}

/// ⭐ **DETERMINÍSTICO** (HR-5) — duas corridas, a mesma malha ao bit.
#[test]
fn the_remesh_is_bit_reproducible() {
    let (mut a, mut b) = (
        shapes::uv_sphere(24, 36, 1.0),
        shapes::uv_sphere(24, 36, 1.0),
    );
    remesh_isotropic(&mut a, ALPHA);
    remesh_isotropic(&mut b, ALPHA);
    assert_eq!(a.positions(), b.positions(), "duas corridas divergiram");
}

/// ⭐ **A TOPOLOGIA ATRAVESSA** — o gênero da entrada é o da saída.
#[test]
fn the_genus_survives() {
    use std::collections::BTreeSet;
    for (name, mut mesh, want) in [
        ("esfera", shapes::uv_sphere(24, 36, 1.0), 2i64),
        ("toro", shapes::torus(64, 32, 1.0, 0.35), 0),
    ] {
        remesh_isotropic(&mut mesh, ALPHA);
        let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
        for f in mesh.faces() {
            let v = f.verts();
            for i in 0..v.len() {
                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                edges.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
        let chi = mesh.vert_count() as i64 - edges.len() as i64 + mesh.faces().len() as i64;
        eprintln!("[iso] {name}: chi={chi} (esperado {want})");
        assert_eq!(chi, want, "{name}: o remesh mudou o GENERO da superficie");
    }
}
