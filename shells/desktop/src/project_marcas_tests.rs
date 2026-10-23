//! **As MARCAS DO ARTISTA no arquivo** (Macros W9, v193, `docs/Macros/07_plano_W9.md`): o blob atravessa
//! o `.ph2dproj` igual, e um v191/v192 (sem o campo) abre sem marcas.

use super::*;

/// ⭐ **As marcas atravessam o arquivo** — os bytes que o palco deu voltam iguais ao abrir.
#[test]
fn as_marcas_do_artista_atravessam_o_arquivo() {
    let marcas = vec![1u8, 1, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7];
    let file = ProjectFile {
        marcas_do_artista: marcas.clone(),
        ..bare_file(empty_state())
    };
    let path = tmp_path("marcas_v193");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let (back, _) = headless_app()
        .read_project_file(&path.to_string_lossy())
        .expect("aceite");
    let _ = std::fs::remove_file(&path);
    assert_eq!(back.marcas_do_artista, marcas);
}

/// ⭐ **Um v191 e um v192 abrem, sem marcas**. ⚠️ Os bytes são escritos campo a campo (a forma de então, sem o
/// campo) — não a partir do tipo vivo, senão o teste provaria o braço com a hipótese dele.
#[test]
fn um_v191_e_um_v192_abrem_sem_marcas() {
    for ver in [191u32, 192] {
        let f = bare_file(empty_state());
        let v191 = (
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
            &f.scripts,
        );
        let path = tmp_path(&format!("marcas_v{ver}"));
        let bytes = postcard::to_allocvec(&(ver, v191)).expect("serializa");
        std::fs::write(&path, bytes).expect("grava");
        let lido = headless_app().read_project_file(&path.to_string_lossy());
        let _ = std::fs::remove_file(&path);
        let (back, _) = lido.expect("um v191/v192 é recusado");
        assert!(back.marcas_do_artista.is_empty());
    }
}
