//! As do cartão USER DATA (P12 U): cor, texto e a tabela de propriedades de cada dono do Pixel.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "card.user_data.toggle" => "User Data",
        "card.user_data.color" => "Color",
        "card.user_data.text" => "Text",
        "card.user_data.name" => "Name",
        "card.user_data.value" => "Value",
        "card.user_data.add" => "+ Property",
        "card.user_data.remove" => "×",
        "pixel.history.user_data" => "User Data",
        "panel.pixel.sprite.properties" => "Properties…",
        "pixel.keys.sprite.properties" => "Sprite Properties…",
        "panel.pixel.sprite_props.title" => "Sprite Properties",
        "panel.pixel.sprite_props.size" => "Size",
        "panel.pixel.sprite_props.size_value" => "{w} × {h}",
        "panel.pixel.sprite_props.line" => "{k}: {v}",
        "panel.pixel.sprite_props.mode" => "Color Mode",
        "panel.pixel.sprite_props.frames" => "Frames",
        "panel.pixel.sprite_props.layers" => "Layers",
        "panel.pixel.sprite_props.rgba" => "RGB Color",
        "panel.pixel.sprite_props.gray" => "Grayscale",
        "panel.pixel.sprite_props.indexed" => "Indexed",
        _ => return None,
    })
}
