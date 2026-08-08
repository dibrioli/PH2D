//! Os gates da espessura por vértice. Ver [`super`].

use super::{DEFAULT_THICKNESS, bake};
use crate::VoxelField;
use ph2d_mesh::{Face, Mesh, shapes};

/// Constrói o campo do jeito que o gesto de bake constrói — a MESMA sequência
/// do `sculpt3d_history::bake_ao`, para os gates medirem o que o produto mede.
fn field_for(mesh: &Mesh) -> VoxelField {
    let mut f = VoxelField::for_bounds(mesh.bounds(), crate::DEFAULT_RESOLUTION);
    f.voxelize(mesh);
    f.flood_fill();
    f
}

/// **O ORÁCULO É EXTERNO:** numa esfera de raio `r` o raio que entra por
/// qualquer vértice sai no antípoda, e a espessura é `2r` — um número que a
/// fixture conhece **sem chamar a função sob teste**.
#[test]
fn a_spheres_thickness_is_twice_its_radius() {
    for &r in &[1.0f32, 0.45, 0.2] {
        let mut m = shapes::uv_sphere(48, 72, r);
        m.triangulate();
        let t = bake(&field_for(&m), &m);
        assert_eq!(t.len(), m.positions().len());
        assert!(
            t.iter().all(|v| v.is_finite()),
            "uma esfera FECHADA nao pode deixar vertice sem medicao"
        );
        let worst = t
            .iter()
            .map(|&v| (v - 2.0 * r).abs() / (2.0 * r))
            .fold(0.0f32, f32::max);
        assert!(
            worst < 0.05,
            "esfera r={r}: pior erro {:.2}% contra o oraculo 2r",
            100.0 * worst
        );
    }
}

/// **A CHAPA é o caso de uso, e ela é o gate que o proxy `2/|κ|` reprova.**
///
/// ⚠️ Este teste é o que impede alguém de "simplificar" o bake para a curvatura:
/// medido, o proxy diz **1,100** onde a verdade é **0,180** (511%). Uma fixture
/// de esferas não distingue os dois — a chapa distingue.
#[test]
fn a_slab_is_thin_where_the_curvature_proxy_would_call_it_infinite() {
    let mut slab = shapes::cylinder(64, 1.0, 0.1);
    slab.triangulate();
    let t = bake(&field_for(&slab), &slab);

    // As TAMPAS: os vértices que olham para cima ou para baixo, onde a espessura
    // é a altura do disco.
    let flat: Vec<f32> = slab
        .normals()
        .iter()
        .zip(&t)
        .filter(|(n, _)| n[1].abs() > 0.9)
        .map(|(_, &v)| v)
        .collect();
    assert!(!flat.is_empty(), "a fixture nao tem tampa");
    let worst = flat.iter().fold(0.0f32, |a, &v| a.max(v));
    assert!(
        worst < 0.3,
        "a tampa da chapa mediu {worst:.3}; a altura e' 0,1 — o proxy pela \
         curvatura diria ~1,1 aqui, e e' esse o erro que este gate recusa"
    );

    // E o proxy, computado AQUI, para o número da recusa ser visível.
    let proxy_at_cap = slab
        .normals()
        .iter()
        .enumerate()
        .filter(|(_, n)| n[1].abs() > 0.9)
        .map(|(i, _)| slab.curv_world()[i].abs())
        .fold(0.0f32, f32::max);
    assert!(
        proxy_at_cap < 2.0,
        "a curvatura na tampa e' ~0 (|k| max {proxy_at_cap:.4}) => 2/|k| explode; \
         e' por isso que o bake MEDE"
    );
}

/// **Uma casca aberta não é fina: ela não foi MEDIDA.**
///
/// ⚠️ É o gate da guarda *nasce dentro* do [`super::at`], e ele nasceu vermelho:
/// sem ela o primeiro passo de uma folha sem volume já cai fora do campo, a
/// marcha devolve meio voxel, e a superfície inteira acende como vidro.
/// Colapsar *não medi* em *é fino* é o modo de falha deste canal.
#[test]
fn an_open_shell_reports_unmeasured_not_thin() {
    // ⚠️ **Uma CHAPA de duas faces, e não um tubo aberto:** um tubo tem a parede
    // do outro lado, então os raios dele acertam e a fixture não conteria o
    // fenômeno. Aqui não há nada atrás de vértice nenhum.
    let sheet = Mesh::from_parts(
        vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
        ],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("a chapa e' valida");
    let t = bake(&field_for(&sheet), &sheet);
    assert!(
        t.iter().all(|&v| v > 0.0),
        "nenhum vertice pode medir espessura zero"
    );
    assert!(
        t.iter().all(|&v| v == DEFAULT_THICKNESS),
        "uma folha sem volume tem de continuar NAO-MEDIDA: a marcha nao tem \
         interior por onde andar"
    );
    assert!(
        DEFAULT_THICKNESS.is_infinite(),
        "o default e' opaco: exp(-inf) = 0"
    );
}

/// **A escada da cena `=19` é MONOTÔNICA** — é ela que separa um canal de
/// translucidez de um slider de brilho.
///
/// ⚠️ O oráculo é a ORDEM entre três peças, não um valor: uma peça só não
/// distingue *"a luz atravessa formas finas"* de *"a peça clareou"*.
#[test]
fn a_smaller_piece_is_thinner_and_the_order_is_the_oracle() {
    let mut measured = Vec::new();
    for &r in &[1.0f32, 0.45, 0.2] {
        let mut m = shapes::uv_sphere(48, 72, r);
        m.triangulate();
        let t = bake(&field_for(&m), &m);
        measured.push(t.iter().sum::<f32>() / t.len() as f32);
    }
    assert!(
        measured[0] > measured[1] && measured[1] > measured[2],
        "a escada tem de descer: {measured:?}"
    );
    // E a razão entre elas é a razão dos raios, não um número qualquer.
    let ratio = measured[0] / measured[2];
    assert!(
        (ratio - 5.0).abs() < 0.5,
        "r=1 contra r=0,2 tem de dar ~5x; deu {ratio:.2}"
    );
}
