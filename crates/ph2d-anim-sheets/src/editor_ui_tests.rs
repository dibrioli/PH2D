use super::*;

#[test]
fn a_mistura_conta_so_quadros_de_verdade() {
    let a = AnimFrameRef { sheet: 0, cell: 1 };
    let b = AnimFrameRef { sheet: 1, cell: 0 };
    assert!(!mixes_sheets(&[a, AnimFrameRef::EMPTY, a]));
    assert!(mixes_sheets(&[a, b]));
}

/// ⭐ **Uma fonte só para o nome da folha** (plano GEF 26): o editor mostra o nome da biblioteca (`ph2d_ecs::SheetNames`,
/// o mesmo do cartão Image), nunca o do ficheiro. *Mutação: voltar ao nome do ficheiro ⇒ «Sheet 2».*
#[test]
fn o_editor_mostra_o_nome_da_biblioteca_o_mesmo_do_cartao() {
    use ph2d_asset::AssetDb;
    use ph2d_ecs::{ExtraSheet, SheetNames, SpritePixels};
    let db = AssetDb::default();
    let idle = db.insert_image_rgba8(8, 2, vec![255; 8 * 2 * 4]);
    let run = db.insert_image_rgba8(12, 4, vec![200; 12 * 4 * 4]);
    let mut w = World::new();
    let e = w
        .spawn((
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpritePixels(idle),
            SpriteGrid::new(4, 1),
            SpriteSheets(
                [ExtraSheet {
                    asset: run,
                    size: [1.5, 1.5],
                    anchor: [0.0, 0.0],
                    hframes: 3,
                    vframes: 2,
                }]
                .into_iter()
                .collect(),
            ),
        ))
        .id();
    w.insert_resource(SheetNames {
        ids: vec![idle, run],
        names: [(idle, "idle".to_owned()), (run, "run (2)".to_owned())].into(),
    });
    crate::sheet_cache::refresh(
        &mut w,
        &[e.to_bits()],
        &db,
        &std::collections::BTreeMap::new(),
    );
    let (sheets, _) = editor_sheets(&w, e.to_bits());
    let nomes: Vec<&str> = sheets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(nomes, ["idle", "run (2)"]);
}
