//! **O tendão da pele no arquivo** (F41, `210 → 211`): num `<= 210` a `SkinBind` gravada sem o
//! `comprimento` abre pelo degrau de forma, com `comprimento = 0` e o resto igual. Os bytes são
//! escritos campo a campo aqui — nem do tipo vivo nem do congelado.

use super::*;
use ph2d_skeleton_ecs::{CorreccaoDePeso, SKIN_TYPE_NAME, SkinBind, SkinLaw};

#[derive(serde::Serialize)]
struct TendaoGravado {
    bone: u64,
    rest: [f64; 6],
}

#[derive(serde::Serialize)]
struct PeleGravada {
    source: Vec<u8>,
    tendons: Vec<TendaoGravado>,
    law: SkinLaw,
    correcoes: Vec<CorreccaoDePeso>,
}

const RESTS: [[f64; 6]; 2] = [
    [1.0, 0.0, 0.0, 1.0, -3.5, 2.5],
    [0.0, 1.0, -1.0, 0.0, 7.25, -1.0],
];

fn pele_v210() -> Vec<u8> {
    postcard::to_allocvec(&PeleGravada {
        source: vec![9, 8, 7, 6],
        tendons: vec![
            TendaoGravado {
                bone: 41,
                rest: RESTS[0],
            },
            TendaoGravado {
                bone: 42,
                rest: RESTS[1],
            },
        ],
        law: SkinLaw::default(),
        correcoes: Vec::new(),
    })
    .expect("codifica")
}

/// ⭐ **Um v210 com a pele antiga abre na forma viva, com `comprimento = 0`.**
#[test]
fn um_v210_abre_a_pele_sem_comprimento() {
    let _fio = ph2d_physics_ecs::LayerRowsV218::enter();
    let antiga = pele_v210();
    // ⛔ CONTROLO: o tipo vivo não lê a forma antiga — sem o degrau, o blob ficava ilegível.
    assert!(postcard::from_bytes::<SkinBind>(&antiga).is_err());
    // Sem montar o registo à mão: o `headless_app` não abre a janela, e o load tem de garantir o
    // degrau sozinho (era o defeito: um load sem janela deixava a pele por migrar).
    let blob = ph2d_asset::ComponentBlob {
        type_id: ph2d_ecs::scene::registry::stable_type_id(SKIN_TYPE_NAME),
        data: antiga,
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
    let ver = crate::project_schema::PROJECT_SCHEMA_TENDAO_SEM_COMPRIMENTO;
    let path = tmp_path(&format!("pele_v{ver}"));
    let bytes = ph2d_vec_scene::with_wire_layout(28, || postcard::to_allocvec(&(ver, &f)));
    std::fs::write(&path, bytes.expect("serializa")).expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("aceite");
    let dados = &back.state.world.entities[0].components[0].data;
    let pele: SkinBind = postcard::from_bytes(dados)
        .unwrap_or_else(|e| panic!("a pele não abriu na forma viva ({e}): {dados:?}"));
    assert_eq!(pele.source, vec![9, 8, 7, 6]);
    assert_eq!(pele.law, SkinLaw::default());
    assert!(pele.correcoes.is_empty());
    let lidos: Vec<_> = (pele.tendons.iter())
        .map(|t| (t.bone.0, t.rest, t.comprimento))
        .collect();
    assert_eq!(lidos, vec![(41, RESTS[0], 0.0), (42, RESTS[1], 0.0)]);
}
