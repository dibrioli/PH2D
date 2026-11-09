use super::*;

fn folha(byte: u8, h: u32, v: u32) -> ExtraSheet {
    ExtraSheet {
        asset: AssetId::from_bytes(&[byte]),
        size: [1.0, 1.0],
        anchor: [0.0, 0.0],
        hframes: h,
        vframes: v,
        offsets: Default::default(),
    }
}

#[test]
fn a_folha_k_e_a_entrada_k_menos_um() {
    let s = SpriteSheets([folha(1, 6, 2), folha(2, 3, 1)].into_iter().collect());
    assert_eq!(s.sheet(0), None, "a folha 0 é a do Sprite");
    assert_eq!(s.sheet(1).map(|f| f.asset), Some(folha(1, 1, 1).asset));
    assert_eq!(s.sheet(2).map(|f| f.asset), Some(folha(2, 1, 1).asset));
    assert_eq!(s.sheet(3), None);
}

#[test]
fn as_celulas_de_cada_folha_comecam_pela_grelha_do_objeto() {
    let mut w = World::new();
    let e = w.spawn(SpriteGrid::new(4, 1)).id();
    assert_eq!(sheet_cells_of(&w, e).as_slice(), &[4]);
    w.entity_mut(e).insert(SpriteSheets(
        [folha(1, 6, 2), folha(2, 3, 1)].into_iter().collect(),
    ));
    assert_eq!(sheet_cells_of(&w, e).as_slice(), &[4, 12, 3]);
    let sem_grelha = w.spawn(()).id();
    assert_eq!(sheet_cells_of(&w, sem_grelha).as_slice(), &[1]);
}

/// ⭐ GEF 26 F5: duplicar (a cópia funda do Duplicate, da instância e do prefab) leva as folhas extra; a imagem é um
/// nome durável, nada a re-apontar. Mutação que sangra: tirar o `SpriteSheets` do registo.
#[test]
fn duplicar_leva_as_folhas() {
    let mut reg = crate::scene::ComponentRegistry::new();
    crate::scene::register_ecs_components(&mut reg);
    let mut w = World::new();
    let folhas = SpriteSheets([folha(2, 6, 1), folha(3, 4, 2)].into_iter().collect());
    let e = w
        .spawn((
            crate::Transform::IDENTITY,
            SpriteGrid::new(4, 1),
            folhas.clone(),
        ))
        .id();
    let copia = crate::deep_copy_subtree(&mut w, &reg, e, None)
        .expect("copia")
        .root;
    assert_ne!(copia, e);
    assert_eq!(w.get::<SpriteSheets>(copia), Some(&folhas));
}
