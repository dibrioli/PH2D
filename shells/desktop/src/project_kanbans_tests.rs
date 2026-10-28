//! **Os KANBANS no arquivo** (TrelloClone W1, v194, ADR-0195): o blob atravessa o `.ph2dproj` igual,
//! e um v193 (sem o campo) abre sem Kanbans.

use super::*;
use ph2d_editor_core::documents::KanbanSet;

/// ⭐ **Os Kanbans atravessam o arquivo** — o conjunto que se gravou é o que se lê.
#[test]
fn os_kanbans_atravessam_o_arquivo() {
    let mut set = KanbanSet::default();
    set.create("Roadmap");
    set.create("Sprint");
    let file = ProjectFile {
        kanbans: set.to_bytes().expect("bytes"),
        ..bare_file(empty_state())
    };
    let path = tmp_path("kanbans_v194");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let (back, _) = headless_app()
        .read_project_file(&path.to_string_lossy())
        .expect("aceite");
    let _ = std::fs::remove_file(&path);
    assert_eq!(KanbanSet::from_bytes(&back.kanbans), Ok(set));
}

/// ⭐ **Um v193, um v192 e um v190 abrem, sem Kanbans** — e com as marcas do artista intactas. ⚠️ Os
/// bytes são escritos campo a campo (a forma de então), não pelo tipo vivo.
#[test]
fn um_v193_um_v192_e_um_v190_abrem_sem_kanbans() {
    let f = bare_file(empty_state());
    let marcas = vec![3u8, 1, 4];
    let ate_scripts = (
        &f.state,
        &f.assets,
        &f.painted,
        &f.motion,
        &f.timeline,
        &f.physics,
        &f.tokens,
        &f.settings,
        &f.player_tape,
        &f.sprite_pixels,
        &f.stable_id_counter,
        &f.input_map,
        &f.pattern_art,
        &f.boards,
    );
    let casos: [(u32, Vec<u8>); 3] = [
        (
            193,
            postcard::to_allocvec(&(193u32, (&ate_scripts, &f.scripts, &marcas))).unwrap(),
        ),
        (
            192,
            postcard::to_allocvec(&(192u32, (&ate_scripts, &f.scripts))).unwrap(),
        ),
        (190, postcard::to_allocvec(&(190u32, &ate_scripts)).unwrap()),
    ];
    for (ver, bytes) in casos {
        let path = tmp_path(&format!("kanbans_v{ver}"));
        std::fs::write(&path, bytes).expect("grava");
        let lido = headless_app().read_project_file(&path.to_string_lossy());
        let _ = std::fs::remove_file(&path);
        let (back, _) = lido.unwrap_or_else(|| panic!("o v{ver} foi recusado"));
        assert!(back.kanbans.is_empty(), "v{ver}");
        let esperadas: &[u8] = if ver == 193 { &marcas } else { &[] };
        assert_eq!(back.marcas_do_artista, esperadas, "v{ver}");
    }
}
