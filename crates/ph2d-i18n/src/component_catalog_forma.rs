//! **O Look da forma de colisão no catálogo** (GEF02 onda 8): o nome do `ShapeLook` e os rótulos dos canais;
//! e os canais do filtro do Ray Sensor (GEF 07, versão C) — irmão do `component_catalog`, que está no tecto.

/// A tradução de uma chave do Look da forma, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "component.shape_look.name" => "Collision Shape Look",
        "component.field.shape_look.0" => "Pivot",
        "component.field.shape_look.1" => "Color",
        "component.field.shape_look.2" => "Opacity",
        "component.field.shape_look.3" => "Own Color",
        "component.field.shape_look.4" => "Stroke Width",
        "component.field.shape_look.5" => "Dashed",
        "component.field.shape_look.6" => "Dash and Gap",
        "component.field.shape_look.7" => "Own Stroke",
        // O filtro do Ray Sensor (GEF 07, versão C): os canais 4..11, pelo mesmo tecto.
        "component.field.ray_sensor.4" => "Ignore my body",
        "component.field.ray_sensor.5" => "Ignore my other parts",
        "component.field.ray_sensor.6" => "Only see",
        "component.field.ray_sensor.7" => "Tag",
        "component.field.ray_sensor.8" => "Walls block",
        "component.field.ray_sensor.9" => "Hit triggers",
        "component.field.ray_sensor.10" => "Starts inside",
        "component.field.ray_sensor.11" => "See all hits",
        _ => return None,
    })
}
