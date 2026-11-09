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
