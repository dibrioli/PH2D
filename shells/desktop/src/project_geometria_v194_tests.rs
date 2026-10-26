//! ⭐⭐ **Um projecto gravado ANTES do id por vértice abre igual, já numerado** (W4.2, `PROJECT_SCHEMA`
//! `194 → 195`).
//!
//! `tests/fixtures/projecto_v194_geometria.ph2dproj` foi gravado pelo código v194 (commit `db09f4bd45`,
//! numa worktree temporária) — ⛔ **nunca o regenere**. Dentro: a cena de `ph2d-vec-scene/tests/it/
//! fixtures/vec_scene_v23_geometria.bin` e um estado Hover na «caixa» cuja pose leva a geometria dos
//! «dentes».

use super::*;

const V194: &[u8] = include_bytes!("../tests/fixtures/projecto_v194_geometria.ph2dproj");

fn le(bytes: &[u8], nome: &str) -> Option<ProjectFile> {
    let path = tmp_path(nome);
    std::fs::write(&path, bytes).expect("grava o fixture");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    lido.map(|(f, _)| f)
}

fn ids(verts: &[ph2d_vec_scene::VecVertex]) -> Vec<u32> {
    verts.iter().map(|v| v.id.0).collect()
}

#[test]
fn a_v194_project_opens_with_vertex_ids_in_order() {
    let file = le(V194, "geometria_v194").expect("um v194 abre");
    let [dentes, caixa] = file.state.vec.paths() else {
        panic!("duas formas, lidas {}", file.state.vec.paths().len());
    };
    assert_eq!(ids(&dentes.verts), (1..=10).collect::<Vec<_>>());
    assert_eq!(dentes.verts[3].corner_radius, 2.5);
    assert_eq!(dentes.verts[9].anchor, [90.0, 7.5]);
    assert_eq!(caixa.subpaths[0].verts[2].anchor, [20.0, 25.0]);
    assert_eq!(caixa.subpaths[0].verts[2].id.0, 7);

    // A geometria de uma POSE de estado viaja no mesmo ficheiro e lê-se pelo mesmo âmbito.
    let [estado] = file.state.ui_states.get(caixa.id) else {
        panic!("um estado no hospedeiro");
    };
    let g = estado.objects[0].geometry.as_ref().expect("a pose tem geometria");
    assert_eq!(ids(&g.verts), (1..=10).collect::<Vec<_>>());
    assert_eq!(g.verts[6].in_handle, [57.0, -2.0]);
}

/// ⛔ CONTROLO: os MESMOS bytes com o cabeçalho de hoje são lidos pela forma viva, e não abrem igual.
#[test]
fn the_same_bytes_under_todays_header_do_not_open_the_same() {
    let mut hoje = V194.to_vec();
    // `194` e `195` são varints de dois bytes (`0xC2 0x01` / `0xC3 0x01`).
    assert_eq!(&hoje[..2], &[0xC2, 0x01], "o fixture é um v194");
    const { assert!(PROJECT_SCHEMA == 195) };
    hoje[0] = 0xC3;
    let certo = le(V194, "geometria_v194_certo").expect("um v194 abre");
    let torto = le(&hoje, "geometria_v194_torto");
    assert!(
        torto.is_none_or(|f| f.state.vec != certo.state.vec),
        "lido como v195, o v194 deu a mesma cena — o gate de cima é cego"
    );
}

/// A fronteira do fio por versão de projecto.
#[test]
fn each_project_version_reads_the_vector_wire_it_wrote() {
    use crate::project_schema::fio_do_projeto;
    assert_eq!(fio_do_projeto(189), Some(22));
    assert_eq!(fio_do_projeto(190), Some(23));
    assert_eq!(fio_do_projeto(194), Some(23));
    assert_eq!(fio_do_projeto(PROJECT_SCHEMA), None);
}
