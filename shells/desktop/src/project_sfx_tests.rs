//! As RECEITAS DE SOM no arquivo (v217): o blob atravessa igual e um v216 abre sem elas. O blob: `ph2d_sfx_recipe`.

use super::sounds::written;
use super::*;

#[test]
fn as_receitas_atravessam_o_arquivo_byte_a_byte() {
    let file = ProjectFile {
        sfx_lab: vec![1, 2, 3],
        ..bare_file(empty_state())
    };
    let path = written("sfx_v217", &(PROJECT_SCHEMA, &file));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert_eq!(lido.expect("aceite").0.sfx_lab, [1, 2, 3]);
}

/// ⚠️ Os bytes do v216 são escritos campo a campo: a partir do tipo vivo, o teste provaria o braço com a hipótese dele.
#[test]
fn um_v216_abre_sem_receitas() {
    let _fio = ph2d_physics_ecs::LayerRowsV218::enter();
    let f = bare_file(empty_state());
    let v216 = (
        (&f.state, &f.assets, &f.painted, &f.motion),
        (&f.timeline, &f.physics, &f.tokens, &f.settings),
        (&f.player_tape, &f.sprite_pixels, &f.stable_id_counter),
        (&f.input_map, &f.pattern_art, &f.boards, &f.scripts),
        (
            &f.marcas_do_artista,
            &f.jogo,
            &f.mesa,
            &f.kanbans,
            &f.pixel_sprites,
            vec![7u8],
        ),
    );
    let ver = crate::project_schema::PROJECT_SCHEMA_SEM_RECEITAS_DE_SOM;
    let path = written("sfx_v216", &(ver, v216));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (f, _) = lido.expect("um v216 é recusado");
    assert_eq!(f.sounds, [7], "o último campo do v216 lê-se no sítio dele");
    assert!(f.sfx_lab.is_empty());
}
