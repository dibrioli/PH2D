//! Gates da porta de saída.
//!
//! ⚠️ **O oráculo é o ROUND-TRIP**, e ele só existe porque a wave trouxe os
//! leitores junto: escrever um arquivo e afirmar que os bytes "parecem certos" é
//! um gate que casa com a própria escrita. Aqui o arquivo é lido de volta pelo
//! caminho que o artista usa, e o que se compara é GEOMETRIA.

use super::*;
use crate::face::Face;
use crate::mesh::Mesh;
use crate::ply::PlyError;
use crate::{import_obj, import_ply, import_stl, shapes};

/// Uma peça com pose deslocada e escalada — a fixture TEM de conter a pose,
/// senão o gate do mundo é verde por vácuo.
fn piece(mesh: &Mesh, at: [f32; 3], scale: f32) -> ExportPiece<'_> {
    ExportPiece {
        name: Some("P"),
        mesh,
        pose: Pose::new(at, scale),
    }
}

/// A caixa de um conjunto de pontos.
fn bounds(p: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for v in p {
        for k in 0..3 {
            lo[k] = lo[k].min(v[k]);
            hi[k] = hi[k].max(v[k]);
        }
    }
    (lo, hi)
}

/// **A geometria sai em MUNDO** — a decisão central da wave, nos três formatos.
///
/// ⚠️ Sem isto, duas peças com poses diferentes saem EMPILHADAS na origem: o
/// defeito espelho exato do que o import curou ao centrar cada peça. A fixture
/// tem duas peças bem separadas, porque com uma só *local* e *mundo* diferem
/// apenas por uma translação que ninguém nota num arquivo.
#[test]
fn every_format_writes_the_world_the_artist_sees() {
    let a = shapes::cube(1.0);
    let b = shapes::cube(1.0);
    let pieces = [
        piece(&a, [-5.0, 0.0, 0.0], 1.0),
        piece(&b, [5.0, 0.0, 0.0], 2.0),
    ];

    for fmt in MeshFormat::ALL {
        let bytes = fmt.write(&pieces);
        let mesh = match fmt {
            MeshFormat::Obj => {
                let ps = import_obj(&String::from_utf8(bytes).expect("utf8")).expect("obj");
                // O OBJ preserva peças: junta as duas caixas para comparar.
                let mut all = Vec::new();
                for p in &ps {
                    all.extend_from_slice(p.mesh.positions());
                }
                let (lo, hi) = bounds(&all);
                assert!(
                    lo[0] < -5.0 && hi[0] > 5.0,
                    "{fmt:?}: as peças saíram empilhadas — {lo:?}..{hi:?}"
                );
                continue;
            }
            MeshFormat::Ply => import_ply(&bytes).expect("ply"),
            MeshFormat::Stl => import_stl(&bytes).expect("stl"),
        };
        let (lo, hi) = bounds(mesh.positions());
        assert!(
            lo[0] < -5.0 && hi[0] > 5.0,
            "{fmt:?}: as peças saíram empilhadas — {lo:?}..{hi:?}"
        );
        // E a ESCALA da pose viajou: a peça da direita mede o dobro.
        assert!(
            hi[0] - lo[0] > 11.0,
            "{fmt:?}: a escala da pose não chegou ao arquivo ({})",
            hi[0] - lo[0]
        );
    }
}

/// **A COR sobrevive onde o formato a tem** — e o gate cobre as duas metades,
/// porque a de AUSÊNCIA é a que o aviso ao artista promete.
#[test]
fn colour_survives_exactly_where_the_format_keeps_it() {
    let mut m = shapes::cube(1.0);
    for (i, c) in m.colors_mut().iter_mut().enumerate() {
        *c = if i % 2 == 0 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 0.0, 1.0]
        };
    }
    let pieces = [piece(&m, [0.0; 3], 1.0)];

    let obj = import_obj(&write_obj(&pieces)).expect("obj");
    assert!(
        obj[0].mesh.colors().is_some(),
        "o OBJ declara que preserva cor e a perdeu"
    );
    let ply = import_ply(&write_ply(&pieces)).expect("ply");
    let c = ply
        .colors()
        .expect("o PLY declara que preserva cor e a perdeu");
    assert!(
        c.iter().any(|v| v[0] > 0.9 && v[2] < 0.1) && c.iter().any(|v| v[2] > 0.9 && v[0] < 0.1),
        "as DUAS cores têm de atravessar, e vieram {c:?}"
    );

    // A ausência: o STL não tem onde pôr cor, e o `keeps_colour` diz isso.
    let stl = import_stl(&write_stl(&pieces)).expect("stl");
    assert!(
        stl.colors().is_none(),
        "um STL não pode trazer cor de volta"
    );
    assert!(
        !MeshFormat::Stl.keeps_colour(),
        "e a tabela tem de concordar"
    );
    assert!(MeshFormat::Obj.keeps_colour() && MeshFormat::Ply.keeps_colour());
}

/// **As PEÇAS sobrevivem só no OBJ**, e a tabela é quem responde.
///
/// ⚠️ É este par que impede o toast de mentir: se `keeps_pieces` divergir do que
/// o escritor faz, o artista exporta três peças em PLY e o app diz que estão
/// separadas.
#[test]
fn pieces_survive_exactly_where_the_table_says() {
    let a = shapes::cube(1.0);
    let b = shapes::octahedron(1.0);
    let pieces = [
        piece(&a, [-3.0, 0.0, 0.0], 1.0),
        piece(&b, [3.0, 0.0, 0.0], 1.0),
    ];

    let obj = import_obj(&write_obj(&pieces)).expect("obj");
    assert_eq!(obj.len(), 2, "o OBJ preserva peças");
    assert!(MeshFormat::Obj.keeps_pieces());

    // PLY e STL fundem — e a tabela diz.
    assert!(!MeshFormat::Ply.keeps_pieces() && !MeshFormat::Stl.keeps_pieces());
    let ply = import_ply(&write_ply(&pieces)).expect("ply");
    assert_eq!(
        ply.positions().len(),
        a.positions().len() + b.positions().len(),
        "o PLY funde tudo num corpo só"
    );
}

/// **Os QUADS sobrevivem em OBJ e PLY**, e o STL os triangula porque o formato
/// não tem outra forma.
#[test]
fn quads_survive_in_the_indexed_formats_and_the_stl_triangulates() {
    let m = shapes::cube(1.0); // o cubo do módulo é feito de quads
    assert!(
        m.faces().iter().any(|f| !f.is_tri()),
        "a fixture precisa CONTER quads, senão o gate é vácuo"
    );
    let pieces = [piece(&m, [0.0; 3], 1.0)];

    let obj = import_obj(&write_obj(&pieces)).expect("obj");
    assert!(
        obj[0].mesh.faces().iter().any(|f| !f.is_tri()),
        "OBJ perdeu os quads"
    );
    let ply = import_ply(&write_ply(&pieces)).expect("ply");
    assert!(
        ply.faces().iter().any(|f| !f.is_tri()),
        "PLY perdeu os quads"
    );

    let stl = import_stl(&write_stl(&pieces)).expect("stl");
    assert!(
        stl.faces().iter().all(|f| f.is_tri()),
        "um STL só sabe falar em triângulos"
    );
    assert_eq!(
        stl.faces().len(),
        triangle_count(&pieces),
        "e a contagem tem de bater com a que o cabeçalho escreveu"
    );
}

/// **A extensão decide o formato** — e nada mais decide.
#[test]
fn the_extension_names_the_format_in_both_directions() {
    for f in MeshFormat::ALL {
        assert_eq!(
            MeshFormat::from_extension(f.extension()),
            Some(f),
            "{f:?} não sobrevive ao par extensão↔formato"
        );
        assert_eq!(
            MeshFormat::from_extension(&f.extension().to_uppercase()),
            Some(f),
            "a caixa da extensão não pode decidir nada"
        );
    }
    assert_eq!(MeshFormat::from_extension("png"), None);
    assert_eq!(MeshFormat::from_extension(""), None);
}

/// **Um triângulo degenerado escreve normal ZERO, nunca `NaN`.**
///
/// ⚠️ É o que a spec do STL prescreve (o leitor deriva a normal pela regra da
/// mão direita), e normalizar um vetor nulo daria `NaN` — que atravessa o
/// arquivo e reaparece como geometria ausente três programas adiante.
#[test]
fn a_degenerate_triangle_writes_a_zero_normal_not_a_nan() {
    let m = Mesh::from_parts(
        vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("colinear ainda é malha");
    let bytes = write_stl(&[piece(&m, [0.0; 3], 1.0)]);
    let n: Vec<f32> = (0..3)
        .map(|k| {
            let at = 84 + k * 4;
            f32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        })
        .collect();
    assert!(
        n.iter().all(|v| v.is_finite()),
        "normal não-finita no arquivo: {n:?}"
    );
    assert_eq!(n, vec![0.0, 0.0, 0.0]);
}

/// **Uma cena VAZIA escreve um arquivo VÁLIDO e vazio** — não um arquivo
/// corrompido nem um pânico.
///
/// ⚠️ O caso existe: exportar logo depois de apagar a última peça. Um cabeçalho
/// PLY com contagens que não batem com o corpo é recusado por todo leitor, longe
/// da causa.
#[test]
fn an_empty_scene_writes_a_valid_empty_file() {
    let stl = write_stl(&[]);
    assert_eq!(stl.len(), 84, "STL vazio é só cabeçalho + contagem");
    assert_eq!(u32::from_le_bytes([stl[80], stl[81], stl[82], stl[83]]), 0);

    let ply = write_ply(&[]);
    let head = String::from_utf8_lossy(&ply);
    assert!(head.contains("element vertex 0") && head.contains("element face 0"));
    // E ele volta como recusa NOMEADA, não como malha fantasma.
    assert!(matches!(import_ply(&ply), Err(PlyError::NoPositions)));
}
