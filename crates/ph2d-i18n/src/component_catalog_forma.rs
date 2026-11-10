//! **O Look da forma de colisão no catálogo** (GEF02 onda 8): o nome do `ShapeLook` e os rótulos dos três canais.

/// A tradução de uma chave do Look da forma, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "component.shape_look.name" => "Collision Shape Look",
        "component.field.shape_look.0" => "Pivot",
        "component.field.shape_look.1" => "Color",
        "component.field.shape_look.2" => "Opacity",
        _ => return None,
    })
}
