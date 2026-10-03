//! ⭐⭐⭐ **O vocabulário da TEXTURA** das formas no Render por malha (`docs/3DModeling`, AS_TEXTURAS).
//!
//! Irmão do [`super::model3d_sky`] por responsabilidade: o céu é a luz que chega, isto é a pele.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.model3d.section.texture" => "Texture",
        "panel.model3d.texture.choice" => "Texture",
        "panel.model3d.texture.none" => "None",
        "panel.model3d.texture.bricks" => "Bricks",
        "panel.model3d.texture.wood" => "Wood",
        "panel.model3d.texture.stone" => "Stone",
        "panel.model3d.texture.metal_plate" => "Metal Plate",
        "panel.model3d.texture.rusty_metal" => "Rusty Metal",
        "panel.model3d.texture.leather" => "Leather",
        "panel.model3d.texture.concrete" => "Concrete",
        "panel.model3d.texture.from_file" => "From File\u{2026}",
        "panel.model3d.texture.tile" => "Tile Size",
        "panel.model3d.texture.blend" => "Blend",
        "panel.model3d.texture.bump" => "Bumps",
        "panel.model3d.texture.normal_map" => "Normal Map",
        "panel.model3d.texture.roughness_map" => "Roughness Map",
        "panel.model3d.texture.map_none" => "None",
        "panel.model3d.texture.map_from_file" => "From File\u{2026}",
        "panel.model3d.texture.file" => "File",

        "panel.model3d.texture.choice.tip" => {
            "Paints the shape with a picture that wraps it from the three sides. The color of the \
             material tints it: white shows the texture as it is."
        }
        "panel.model3d.texture.tile.tip" => {
            "How big one repeat of the picture is on the shape. 0 uses the real size of the pack \
             texture."
        }
        "panel.model3d.texture.blend.tip" => {
            "How softly the three sides meet on round shapes: 0 is a sharp seam, 1 the softest."
        }
        "panel.model3d.texture.bump.tip" => {
            "How deep the bumps of the texture look. 0 is flat, 1 as made, 2 twice as deep."
        }
        "panel.model3d.texture.normal_map.tip" => {
            "A picture of the bumps (a normal map, the usual blue-purple kind) for a texture from \
             a file."
        }
        "panel.model3d.texture.roughness_map.tip" => {
            "A grey picture of where the surface is rough (white) or shiny (black) for a texture \
             from a file."
        }

        "field.inert.no_texture" => "No texture is chosen",
        "field.inert.texture_has_no_normal_map" => "This texture has no normal map",
        "field.inert.pack_texture_brings_its_maps" => "Pack textures bring their own maps",
        "field.inert.roughness_from_texture" => "The texture gives the roughness",

        "app.field3d.texture.could_not_read" => "Could not read the texture: {why}",
        "app.field3d.texture.images" => "Images",
        "app.field3d.texture.unknown_format" => "{path}: not an image format we read",
        "app.field3d.texture.not_flat" => "{path}: not a flat image",
        _ => return None,
    })
}
