//! ⭐⭐⭐ **O vocabulário do CÉU fotográfico** do Render por malha (`docs/3DModeling`, a onda do céu).
//!
//! Irmão do [`super::model3d_bloom`] por responsabilidade: o brilho é um passe sobre o quadro, isto é
//! a LUZ que chega à peça.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⚠️ **"Sky" e não "HDRI" nem "Environment"**: é a palavra do artista (o Blender diz
        // *World*, o Unreal *Sky Light*), e o que ele escolhe é de onde vem a luz.
        "panel.model3d.section.sky" => "Sky",
        "panel.model3d.sky.choice" => "Sky",
        // ⚠️ **O 1.º é o estúdio de SEMPRE** (a rampa e a caixa de luz) e não uma foto; a foto de
        // um estúdio chama-se «Photo Studio» para os dois não se confundirem.
        "panel.model3d.sky.studio" => "Studio",
        "panel.model3d.sky.photo_studio" => "Photo Studio",
        "panel.model3d.sky.interior" => "Interior",
        "panel.model3d.sky.city" => "City",
        "panel.model3d.sky.courtyard" => "Courtyard",
        "panel.model3d.sky.forest" => "Forest",
        "panel.model3d.sky.sunrise" => "Sunrise",
        "panel.model3d.sky.sunset" => "Sunset",
        "panel.model3d.sky.night" => "Night",
        "panel.model3d.sky.rotation" => "Rotation",
        "panel.model3d.sky.strength" => "Strength",
        // ⚠️ **"Key Light"**: é o nome de fotógrafo para a luz principal — sob um céu fotografado, o
        // SOL dele (ou a lâmpada mais forte), a que faz a sombra.
        "panel.model3d.sky.key_light" => "Key Light",
        "panel.model3d.sky.background" => "Background",
        "panel.model3d.sky.off" => "Off",
        "panel.model3d.sky.on" => "On",
        "panel.model3d.sky.blur" => "Background Blur",

        "panel.model3d.sky.choice.tip" => {
            "Where the light comes from. Studio is the usual light box; the others are real \
             photographed skies that light and reflect in the model."
        }
        "panel.model3d.sky.rotation.tip" => {
            "Turns the sky around the model, in degrees \u{2014} moves where the reflections and \
             the brightest light come from."
        }
        "panel.model3d.sky.strength.tip" => {
            "Brightness of the sky in stops. 0 gives it the same average light as the Studio, so \
             switching skies does not blow out or darken the scene."
        }
        "panel.model3d.sky.key_light.tip" => {
            "The sky's sun (or its brightest lamp), the light that casts the shadow. 1 is as \
             photographed; 0 leaves only the rest of the sky."
        }
        "panel.model3d.sky.background.tip" => "Shows the sky behind the model.",
        "panel.model3d.sky.blur.tip" => {
            "Softens the sky behind the model so it does not compete with it. 0 is sharp."
        }

        "field.inert.sky_is_studio" => {
            "Inactive: the Studio sky is a light box, with no photo to turn or dim. Pick another \
             Sky to use it."
        }
        "field.inert.sky_background_is_off" => {
            "Inactive: the sky is not shown behind the model. Switch Background to On to use it."
        }
        _ => return None,
    })
}
