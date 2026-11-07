//! Os nomes das camadas no arquivo (v218): atravessam igual e um v217 abre sem eles. A tabela: `ph2d_physics_ecs::LayerNames`.

use super::sounds::written;
use super::*;

#[test]
fn os_nomes_atravessam_o_arquivo_byte_a_byte() {
    let file = ProjectFile {
        physics_layers: vec![1, 2, 3],
        ..bare_file(empty_state())
    };
    let path = written("camadas_v218", &(PROJECT_SCHEMA, &file));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert_eq!(lido.expect("aceite").0.physics_layers, [1, 2, 3]);
}

/// ⚠️ Os bytes do v217 são escritos campo a campo: a partir do tipo vivo, o teste provaria o braço com a hipótese dele.
#[test]
fn um_v217_abre_sem_nomes_e_o_ultimo_campo_dele_le_se_no_sitio() {
    let f = bare_file(empty_state());
    let v217 = (
        (&f.state, &f.assets, &f.painted, &f.motion),
        (&f.timeline, &f.physics, &f.tokens, &f.settings),
        (&f.player_tape, &f.sprite_pixels, &f.stable_id_counter),
        (&f.input_map, &f.pattern_art, &f.boards, &f.scripts),
        (&f.marcas_do_artista, &f.jogo, &f.mesa, &f.kanbans),
        (&f.pixel_sprites, &f.sounds, vec![7u8]),
    );
    let ver = crate::project_schema::PROJECT_SCHEMA_SEM_NOMES_DE_CAMADA;
    let path = written("camadas_v217", &(ver, v217));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (f, _) = lido.expect("um v217 é recusado");
    assert_eq!(f.sfx_lab, [7], "o último campo do v217 lê-se no sítio dele");
    assert!(f.physics_layers.is_empty());
    assert_eq!(
        ph2d_physics_ecs::LayerNames::from_bytes(&f.physics_layers),
        Ok(ph2d_physics_ecs::LayerNames::default()),
        "um projeto antigo mostra os números"
    );
}
