//! Gates da pose.
//!
//! O oráculo de toda ida-e-volta é a **identidade**, e o de toda conversão é a
//! GEOMETRIA (uma esfera de raio conhecido, uma caixa cujos cantos se sabe onde
//! estão) — nunca uma segunda escrita da mesma aritmética, que concordaria com
//! ela exatamente onde ela erra.

use super::*;
use crate::shapes;

fn close(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= tol)
}

/// A pose que toda malha tinha antes desta wave **não move um bit**. É o que
/// torna a lista de um objeto o mundo antigo, e não uma aproximação dele.
#[test]
fn the_identity_pose_is_the_world_it_replaces() {
    let p = Pose::IDENTITY;
    for q in [[0.0; 3], [1.5, -2.25, 7.0], [-1e3, 1e-3, 0.0]] {
        assert_eq!(p.point_to_world(q), q);
        assert_eq!(p.point_to_local(q), q);
        assert_eq!(p.vector_to_world(q), q);
        assert_eq!(p.vector_to_local(q), q);
    }
    assert_eq!(
        p.to_cols_array_2d(),
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    );
}

/// Ida e volta é a identidade, e é isso que impede o pincel de derivar: o pick
/// converte para local e o gesto converte de volta.
#[test]
fn the_round_trip_lands_where_it_started() {
    let p = Pose::new([3.0, -1.0, 0.5], 2.5);
    for q in [[0.0; 3], [1.0, 1.0, 1.0], [-4.0, 9.0, -0.25]] {
        assert!(close(p.point_to_local(p.point_to_world(q)), q, 1e-5));
        assert!(close(p.point_to_world(p.point_to_local(q)), q, 1e-5));
        assert!(close(p.vector_to_local(p.vector_to_world(q)), q, 1e-5));
    }
}

/// ⚠️ **A metade que separa um PONTO de um VETOR.** Trocar as duas portas é o
/// erro mais barato de cometer e o mais caro de ver: um deslocamento que herda a
/// translação puxa o barro para longe do dedo por um vetor constante.
#[test]
fn a_vector_ignores_the_translation_and_a_point_does_not() {
    let p = Pose::new([10.0, 0.0, 0.0], 1.0);
    assert_eq!(p.point_to_world([0.0; 3]), [10.0, 0.0, 0.0]);
    assert_eq!(p.vector_to_world([0.0; 3]), [0.0; 3]);
    assert_eq!(p.vector_to_world([1.0, 0.0, 0.0]), [1.0, 0.0, 0.0]);
}

/// **O raio pousa na malha onde o objeto ESTÁ.** Uma esfera unitária levada para
/// `x = 10` é atingida por um raio que aponta para `x = 10`, e o acerto volta em
/// coordenadas LOCAIS — ou seja, na casca unitária.
#[test]
fn a_ray_finds_the_moved_sphere_and_the_hit_comes_back_local() {
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let pose = Pose::new([10.0, 0.0, 0.0], 1.0);

    // Um raio de mundo que passa pelo centro do objeto deslocado.
    let world = Ray::new([10.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
    let local = pose.ray_to_local(&world);
    let hit = mesh.raycast(&local).expect("o raio tem de achar a esfera");

    // Local: na casca de raio 1 (a malha é um poliedro, daí a folga).
    let r =
        (hit.point[0] * hit.point[0] + hit.point[1] * hit.point[1] + hit.point[2] * hit.point[2])
            .sqrt();
    assert!((r - 1.0).abs() < 0.02, "acerto local em r={r}");

    // E de volta ao mundo ele está onde o artista aponta.
    let w = pose.point_to_world(hit.point);
    assert!(close(w, [10.0, 0.0, 1.0], 0.05), "acerto de mundo {w:?}");

    // O CONTROLE: o mesmo raio contra a pose identidade erra a esfera, que está
    // na origem. Sem esta metade o gate passaria com a conversão deletada.
    assert!(
        mesh.raycast(&world).is_none(),
        "sem a pose o raio não pode achar nada em x=10"
    );
}

/// A escala entra no RAIO da pegada tanto quanto na posição: um objeto ao dobro
/// é atingido ao dobro da distância do próprio centro.
#[test]
fn the_scale_reaches_the_hit_as_well_as_the_place() {
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let pose = Pose::new([0.0; 3], 3.0);
    let world = Ray::new([0.0, 0.0, 10.0], [0.0, 0.0, -1.0]);
    let hit = mesh
        .raycast(&pose.ray_to_local(&world))
        .expect("a esfera escalada continua no caminho");
    let w = pose.point_to_world(hit.point);
    assert!((w[2] - 3.0).abs() < 0.1, "a casca de mundo em z={}", w[2]);
}

/// A caixa em mundo é a caixa local movida e escalada — exata, porque sem
/// rotação nenhum canto sai do eixo.
#[test]
fn the_bounds_travel_with_the_pose() {
    let b = Aabb {
        min: [-1.0, -1.0, -1.0],
        max: [1.0, 1.0, 1.0],
    };
    let out = Pose::new([5.0, 0.0, 0.0], 2.0).bounds_to_world(b);
    assert_eq!(out.min, [3.0, -2.0, -2.0]);
    assert_eq!(out.max, [7.0, 2.0, 2.0]);
    assert!(Pose::at([1.0; 3]).bounds_to_world(Aabb::EMPTY).is_empty());
}

/// ⚠️ **O piso da escala não é gosto:** sem ele `point_to_local` divide por zero
/// e todo ponto do mundo vira infinito — o pick pararia de achar qualquer coisa
/// e nada na tela diria por quê.
#[test]
fn a_degenerate_scale_is_floored_instead_of_dividing_by_zero() {
    for s in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let p = Pose::new([0.0; 3], s);
        assert!(p.scale() >= MIN_SCALE, "escala {s} virou {}", p.scale());
        let q = p.point_to_local([1.0, 2.0, 3.0]);
        assert!(q.iter().all(|c| c.is_finite()), "local não-finito: {q:?}");
    }
}

/// A matriz é **coluna-major** — a mesma convenção do uniform da câmera. Uma
/// transposição aqui move o objeto por um vetor que ninguém autorou, e o gate
/// que a pega é este: a última COLUNA carrega a translação.
#[test]
fn the_matrix_is_column_major_with_the_translation_in_the_last_column() {
    let m = Pose::new([1.0, 2.0, 3.0], 4.0).to_cols_array_2d();
    assert_eq!(m[3], [1.0, 2.0, 3.0, 1.0], "a translação é a 4ª COLUNA");
    assert_eq!([m[0][0], m[1][1], m[2][2]], [4.0, 4.0, 4.0], "a diagonal");
    // E o resto é zero: qualquer termo fora da diagonal cisalharia a malha.
    assert_eq!(
        [m[0][1], m[0][2], m[1][0], m[1][2], m[2][0], m[2][1]],
        [0.0; 6]
    );
}
