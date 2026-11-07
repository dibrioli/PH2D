//! **O Follow Value sobre uma CAIXA no arquivo** (F41, `209 → 210`): num `<= 209` a pilha gravada passa
//! pela lei `Valor::desvira_caixas` ao abrir; num `210` fica como está. Os bytes da pilha são os que o
//! código de ANTES gravou (`ph2d-app-constraints/tests/fixtures/valor_caixa_v209_pilha.bin`, gate da
//! lei: `tests/it/valor_caixa_v209.rs` de lá).

use super::*;
use ph2d_constraints_ecs::{CONSTRAINTS_TYPE_NAME, Constraints};

const PILHA_V209: &[u8] = include_bytes!(
    "../../../crates/ph2d-app-constraints/tests/fixtures/valor_caixa_v209_pilha.bin"
);

fn abre(ver: u32) -> Constraints {
    let blob = ph2d_asset::ComponentBlob {
        type_id: ph2d_ecs::scene::registry::stable_type_id(CONSTRAINTS_TYPE_NAME),
        data: PILHA_V209.to_vec(),
    };
    let mut f = bare_file(empty_state());
    f.state
        .world
        .entities
        .push(std::sync::Arc::new(ph2d_ecs::scene::EntitySnapshotRow {
            id: ph2d_ecs::StableId(5),
            components: vec![blob],
            parent: None,
        }));
    let path = tmp_path(&format!("caixas_v{ver}"));
    // Um degrau antigo grava a cena sem os estilos de cor (v28); o vivo grava a de hoje.
    let bytes = if ver <= crate::project_schema::PROJECT_SCHEMA_SEM_ESTILOS {
        ph2d_vec_scene::with_wire_layout(28, || postcard::to_allocvec(&(ver, &f)))
    } else {
        postcard::to_allocvec(&(ver, &f))
    };
    std::fs::write(&path, bytes.expect("serializa")).expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("aceite");
    postcard::from_bytes(&back.state.world.entities[0].components[0].data).expect("a pilha")
}

/// ⭐ **Um v209 abre com as faixas sobre caixas viradas; um v210 abre igual.**
#[test]
fn um_v209_vira_o_follow_value_sobre_caixas_e_um_v210_nao() {
    let _fio = ph2d_physics_ecs::LayerRowsV218::enter();
    let antes: Constraints = postcard::from_bytes(PILHA_V209).expect("a fixture");
    assert_eq!(abre(PROJECT_SCHEMA), antes, "v210: nada muda");
    let mut esperada = antes.clone();
    assert_eq!(ph2d_constraints_ecs::desvira_caixas(&mut esperada), 6);
    let lida = abre(crate::project_schema::PROJECT_SCHEMA_CAIXA_ANTIGA);
    assert_eq!(lida, esperada, "v209: passa pela lei");
    assert_ne!(lida, antes, "v209: a lei mudou alguma coisa");
}
