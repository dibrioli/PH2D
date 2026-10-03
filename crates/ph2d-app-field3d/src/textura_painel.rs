//! ⭐⭐⭐ **AS FILEIRAS DA TEXTURA, no painel** — só no Render por MALHA (o traçado não a desenha: ali
//! seriam controlos mortos), logo a seguir ao material da forma, e para a MESMA folha: a do primeiro
//! alvo das linhas do material (o alcance da escrita é o do material — `scene_intents`).

use ph2d_field::{Bound, Param};
use ph2d_field_ecs::FieldTexture;
use ph2d_panel_model3d::ParamRow;

const SECCAO: &str = "panel.model3d.section.texture";

/// ⭐ **As nove fontes, como CHAVES** — `0` nenhuma, depois o [`ph2d_triplanar::Embarcada::TODAS`]
/// (há gate: `os_nomes_seguem_o_pacote`), e por último «de ficheiro».
pub(crate) const FONTES: [&str; ph2d_field::TEXTURE_SOURCES as usize] = [
    "panel.model3d.texture.none",
    "panel.model3d.texture.bricks",
    "panel.model3d.texture.wood",
    "panel.model3d.texture.stone",
    "panel.model3d.texture.metal_plate",
    "panel.model3d.texture.rusty_metal",
    "panel.model3d.texture.leather",
    "panel.model3d.texture.concrete",
    "panel.model3d.texture.from_file",
];

const MAPA: [&str; 2] = [
    "panel.model3d.texture.map_none",
    "panel.model3d.texture.map_from_file",
];

/// A posição da rugosidade na tabela do material (`field.dim.roughness`).
const RUGOSIDADE: u8 = 10;

/// Porque é que a fileira `slot` não faz nada agora.
fn apagada(slot: u8, t: &FieldTexture) -> Option<&'static str> {
    let ficheiro = t.source == ph2d_field::TEXTURE_FROM_FILE;
    match slot {
        0 => None,
        _ if t.source == 0 => Some("field.inert.no_texture"),
        3 if ficheiro && t.normal_file.is_empty() => Some("field.inert.texture_has_no_normal_map"),
        4 | 5 if !ficheiro => Some("field.inert.pack_texture_brings_its_maps"),
        _ => None,
    }
}

/// A textura dá a rugosidade? (o pacote sempre; um ficheiro só com o mapa dela)
fn da_rugosidade(t: &FieldTexture) -> bool {
    match t.source {
        0 => false,
        s if s == ph2d_field::TEXTURE_FROM_FILE => !t.roughness_file.is_empty(),
        _ => true,
    }
}

/// ⭐⭐⭐ **Junta as fileiras da textura** às do painel (`ligado` = Render por malha) e apaga a
/// rugosidade do material quando é a textura que a dá.
pub(crate) fn junta(world: &bevy_ecs::world::World, ligado: bool, rows: &mut Vec<ParamRow>) {
    if !ligado {
        return;
    }
    let Some(ultima) = rows
        .iter()
        .rposition(|r| matches!(r.param, Param::Material(_)))
    else {
        return;
    };
    let entity = rows[ultima].entity;
    let t = world
        .get::<FieldTexture>(bevy_ecs::entity::Entity::from_bits(entity))
        .cloned()
        .unwrap_or_default();
    if da_rugosidade(&t) {
        for r in rows.iter_mut() {
            if r.entity == entity && r.param == Param::Material(RUGOSIDADE) && r.inert.is_none() {
                r.inert = Some("field.inert.roughness_from_texture");
            }
        }
    }
    let tile = if t.tile > 0.0 {
        t.tile
    } else {
        usize::from(t.source)
            .checked_sub(1)
            .and_then(|i| ph2d_triplanar::Embarcada::TODAS.get(i))
            .map_or(1.0, |e| e.tamanho_real())
    };
    let linha = |slot: u8, key, value, lo, bound, escolhas: &'static [&'static str]| ParamRow {
        entity,
        param: Param::Texture(slot),
        key,
        value,
        lo,
        bound,
        inert: apagada(slot, &t),
        integral: !escolhas.is_empty(),
        choices: escolhas,
        section: (slot == 0).then_some(SECCAO),
        swatch: None,
        subject: None,
    };
    let mut novas = vec![
        linha(0, "panel.model3d.texture.choice", f32::from(t.source), 0.0, Bound::Hard(f32::from(ph2d_field::TEXTURE_SOURCES - 1)), &FONTES),
        linha(1, "panel.model3d.texture.tile", tile, 0.0, Bound::Soft(4.0), &[]),
        linha(2, "panel.model3d.texture.blend", t.blend, 0.0, Bound::Hard(1.0), &[]),
        linha(3, "panel.model3d.texture.bump", t.bump, 0.0, Bound::Soft(2.0), &[]),
        linha(4, "panel.model3d.texture.normal_map", f32::from(u8::from(!t.normal_file.is_empty())), 0.0, Bound::Hard(1.0), &MAPA),
        linha(5, "panel.model3d.texture.roughness_map", f32::from(u8::from(!t.roughness_file.is_empty())), 0.0, Bound::Hard(1.0), &MAPA),
    ];
    if t.source == ph2d_field::TEXTURE_FROM_FILE {
        let nome = std::path::Path::new(&t.color_file)
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        novas[0].subject = Some(format!("{}: {nome}", ph2d_i18n::tr("panel.model3d.texture.file")));
    }
    let fim = rows[ultima + 1..]
        .iter()
        .position(|r| !matches!(r.param, Param::Material(_)))
        .map_or(rows.len(), |p| ultima + 1 + p);
    let _ = rows.splice(fim..fim, novas);
}

/// ⭐ **A ESCRITA de uma fileira** — devolve o pedido de diálogo quando a escolha é «de ficheiro»
/// (o número não se escreve: só o ficheiro escolhido o faz), senão `None` e o dreno escreve.
#[must_use]
pub(crate) fn pede_ficheiro(slot: u8, value: f32) -> Option<crate::texturas::Canal> {
    match slot {
        0 if value.round() as u8 == ph2d_field::TEXTURE_FROM_FILE => Some(crate::texturas::Canal::Cor),
        4 if value >= 0.5 => Some(crate::texturas::Canal::Normal),
        5 if value >= 0.5 => Some(crate::texturas::Canal::Rugosidade),
        _ => None,
    }
}

#[cfg(test)]
#[path = "textura_painel_tests.rs"]
mod tests;
