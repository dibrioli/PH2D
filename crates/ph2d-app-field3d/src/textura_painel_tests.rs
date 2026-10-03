//! As fileiras da TEXTURA e a costura delas: o pedido do painel pelo dreno REAL até ao mundo.

use super::*;
use crate::materials::colour_row_tests::three_balls;
use ph2d_panel_model3d::{ModelIntent, state::push_intent_for_test};

fn sincroniza(sim: &mut ph2d_ecs::SimWorld, folhas: &[bevy_ecs::entity::Entity]) {
    crate::scene::sync_scene_and_birth(sim, None, folhas, 0.0, &crate::scene::no_drawing());
}

fn fonte(sim: &ph2d_ecs::SimWorld, e: bevy_ecs::entity::Entity) -> Option<u8> {
    sim.world().get::<FieldTexture>(e).map(|t| t.source)
}

/// ⭐ **As nove chaves seguem o pacote**, e todas se traduzem.
#[test]
fn os_nomes_seguem_o_pacote() {
    let esperado = [
        "red_brick",
        "wood_table_001",
        "rock_boulder_dry",
        "metal_plate",
        "rusty_metal_02",
        "brown_leather",
        "concrete_floor_worn_001",
    ];
    let chaves: Vec<&str> = ph2d_triplanar::Embarcada::TODAS
        .iter()
        .map(|e| e.chave())
        .collect();
    assert_eq!(
        chaves, esperado,
        "a ordem do pacote mudou: as chaves do painel apontam para outra"
    );
    assert_eq!(FONTES.len(), usize::from(ph2d_field::TEXTURE_SOURCES));
    assert_eq!(FONTES[0], "panel.model3d.texture.none");
    assert_eq!(FONTES[8], "panel.model3d.texture.from_file");
    for k in FONTES.iter().chain(MAPA.iter()) {
        assert_ne!(ph2d_i18n::tr(k), *k, "{k} não tem texto");
    }
}

fn linhas_do_material(entity: u64) -> Vec<ParamRow> {
    [9u8, 10, 11]
        .iter()
        .map(|&k| ParamRow {
            entity,
            param: Param::Material(k),
            key: "field.dim.roughness",
            value: 0.3,
            lo: 0.0,
            bound: Bound::Soft(1.0),
            inert: None,
            integral: false,
            choices: &[],
            section: None,
            swatch: None,
            subject: None,
        })
        .collect()
}

/// ⭐⭐ **Só no Render por malha; logo depois do material; a rugosidade apaga-se com a do pacote.**
#[test]
fn as_fileiras_seguem_o_material_e_apagam_a_rugosidade() {
    let (mut sim, _g, folhas) = three_balls();
    let e = folhas[0];
    let mut fora = linhas_do_material(e.to_bits());
    junta(sim.world(), false, &mut fora);
    assert_eq!(fora.len(), 3, "fora do Render por malha não há fileiras");

    let mut rows = linhas_do_material(e.to_bits());
    junta(sim.world(), true, &mut rows);
    assert_eq!(rows.len(), 9, "seis fileiras depois das três do material");
    assert!(
        rows[3..]
            .iter()
            .enumerate()
            .all(|(i, r)| r.param == Param::Texture(i as u8))
    );
    assert!(
        rows[4..]
            .iter()
            .all(|r| r.inert == Some("field.inert.no_texture"))
    );
    assert_eq!(
        rows[1].inert, None,
        "sem textura a rugosidade é do material"
    );

    sim.world_mut().entity_mut(e).insert(FieldTexture {
        source: 1,
        ..FieldTexture::default()
    });
    let mut rows = linhas_do_material(e.to_bits());
    junta(sim.world(), true, &mut rows);
    assert_eq!(rows[1].inert, Some("field.inert.roughness_from_texture"));
    assert_eq!(rows[6].inert, None, "o relevo do pacote é vivo");
    assert_eq!(
        rows[7].inert,
        Some("field.inert.pack_texture_brings_its_maps")
    );
    assert!(
        (rows[4].value - 1.4).abs() < 1e-6,
        "o ladrilho 0 mostra o tamanho real do tijolo"
    );
}

/// ⭐⭐⭐ **A escolha espalha-se pela selecção, pelo dreno real**, como o material.
#[test]
fn a_textura_espalha_pela_seleccao() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, _g, folhas) = three_balls();
    push_intent_for_test(ModelIntent::SetParam {
        entity: folhas[0].to_bits(),
        param: Param::Texture(0),
        value: 2.0,
    });
    sincroniza(&mut sim, &folhas);
    for &f in &folhas {
        assert_eq!(
            fonte(&sim, f),
            Some(2),
            "a madeira não chegou a todas as formas escolhidas"
        );
    }
}

/// ⭐⭐⭐ **«De ficheiro» abre o diálogo e não escreve; o ficheiro escolhido escreve.**
#[test]
fn de_ficheiro_pede_o_dialogo_e_o_escolhido_chega_ao_mundo() {
    let _ = ph2d_panel_model3d::drain_intents();
    let _ = crate::texturas::take_pedido();
    let (mut sim, _g, folhas) = three_balls();
    push_intent_for_test(ModelIntent::SetParam {
        entity: folhas[0].to_bits(),
        param: Param::Texture(0),
        value: f32::from(ph2d_field::TEXTURE_FROM_FILE),
    });
    sincroniza(&mut sim, &folhas);
    assert_eq!(
        fonte(&sim, folhas[0]),
        None,
        "a escolha sozinha não escreve nada"
    );
    assert_eq!(
        crate::texturas::take_pedido(),
        Some((folhas[0].to_bits(), crate::texturas::Canal::Cor)),
        "o pedido do diálogo não saiu"
    );
    crate::texturas::escolhe(
        folhas[0].to_bits(),
        crate::texturas::Canal::Cor,
        "x.png".into(),
    );
    sincroniza(&mut sim, &folhas);
    for &f in &folhas {
        let t = sim
            .world()
            .get::<FieldTexture>(f)
            .expect("a textura chegou");
        assert_eq!(
            (t.source, t.color_file.as_str()),
            (ph2d_field::TEXTURE_FROM_FILE, "x.png")
        );
    }
    assert_eq!(pede_ficheiro(4, 1.0), Some(crate::texturas::Canal::Normal));
    assert_eq!(pede_ficheiro(5, 0.0), None, "«None» apaga e não abre nada");
}
