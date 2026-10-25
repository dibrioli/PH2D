//! **Os itens das listas no arquivo** (F30 §A1, v195): um v194 abre com os blobs dos itens na forma
//! nova (os ids nascem no primeiro passe, na ordem da lista).

use super::*;

/// ⭐ **Um projecto v194 abre com os timers legíveis** — os bytes do `Timers` escritos na forma de
/// então, campo a campo (não pelo tipo vivo, senão o teste provaria o braço com a hipótese dele).
#[test]
fn um_projecto_v194_abre_com_os_itens_na_forma_nova() {
    let v194: Vec<(String, u64, bool, bool, String)> = vec![
        ("a".into(), 1_000_000, false, true, String::new()),
        ("b".into(), 3_000_000, true, false, "s".into()),
    ];
    let mut f = bare_file(empty_state());
    f.state
        .world
        .entities
        .push(std::sync::Arc::new(ph2d_ecs::scene::EntitySnapshotRow {
            id: ph2d_ecs::StableId(5),
            components: vec![ph2d_asset::ComponentBlob {
                type_id: ph2d_ecs::scene::registry::stable_type_id("ph2d::ecs::Timers"),
                data: postcard::to_allocvec(&v194).expect("grava"),
            }],
            parent: None,
        }));
    let path = tmp_path("itens_v194");
    std::fs::write(
        &path,
        postcard::to_allocvec(&(194u32, &f)).expect("serializa"),
    )
    .expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("um v194 é aceite");
    let t: ph2d_ecs::Timers =
        postcard::from_bytes(&back.state.world.entities[0].components[0].data).expect("forma nova");
    assert_eq!(t.0[1].duration_us, 3_000_000);
    assert!(
        t.0.iter().all(|x| x.item.is_none()),
        "o passe é que dá os ids"
    );
}
