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
    let path = {
        let _fio = ph2d_physics_ecs::LayerRowsV218::enter();
        written("camadas_v217", &(ver, v217))
    };
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (f, _) = lido.expect("um v217 é recusado");
    assert_eq!(f.sfx_lab, [7], "o último campo do v217 lê-se no sítio dele");
    assert_eq!(
        ph2d_physics_ecs::LayerNames::from_bytes(&f.physics_layers),
        Ok(ph2d_physics_ecs::LayerNames(Default::default(), 0xFF)),
        "um projeto antigo mostra os números, das oito de sempre"
    );
}

/// ⭐ Um v218: a matriz de 8 bytes alarga-se no meio do fio e os campos seguintes leem-se no sítio.
#[test]
fn um_v218_abre_com_os_mesmos_pares_as_oito_a_vista_e_o_resto_no_sitio() {
    let mut m = ph2d_physics_ecs::LayerMatrix::all();
    m.set(1, 5, false);
    let velha = ph2d_physics_ecs::PhysicsSettings {
        layer_matrix: m.rows(),
        air_drag: 0.25,
        ..Default::default()
    };
    let f = ProjectFile {
        physics: velha,
        sfx_lab: vec![7],
        ..bare_file(empty_state())
    };
    let ver = crate::project_schema::PROJECT_SCHEMA_MATRIZ_DE_OITO;
    let path = {
        let _fio = ph2d_physics_ecs::LayerRowsV218::enter();
        written("camadas_v218_oito", &(ver, &f))
    };
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (f, _) = lido.expect("um v218 é recusado");
    assert_eq!(f.physics, velha, "os pares de sempre e o campo seguinte");
    assert_eq!(f.sfx_lab, [7], "o fim do ficheiro lê-se no sítio");
    let nomes = ph2d_physics_ecs::LayerNames::from_bytes(&f.physics_layers).expect("nomes");
    assert_eq!(
        nomes.visible(0, &ph2d_physics_ecs::LayerMatrix::all()),
        0xFF,
        "um projeto antigo sem nomes mostra as oito de sempre"
    );
}

/// O de hoje: a célula alta e o «+ Layer» atravessam o arquivo; sem nada, a 0 só.
#[test]
fn um_v219_guarda_a_celula_alta_e_o_que_se_mostra() {
    let mut m = ph2d_physics_ecs::LayerMatrix::all();
    m.set(20, 25, false);
    let mut nomes = ph2d_physics_ecs::LayerNames::default();
    nomes.1 |= 1 << 30;
    let f = ProjectFile {
        physics: ph2d_physics_ecs::PhysicsSettings {
            layer_matrix: m.rows(),
            ..Default::default()
        },
        physics_layers: nomes.to_bytes(),
        ..bare_file(empty_state())
    };
    let path = written("camadas_v219", &(PROJECT_SCHEMA, &f));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (f, _) = lido.expect("aceite");
    assert_eq!(f.physics.layer_matrix, m.rows());
    assert_eq!(
        ph2d_physics_ecs::LayerNames::from_bytes(&f.physics_layers),
        Ok(nomes)
    );
}
