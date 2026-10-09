//! **As palavras da LUZ 2D** (W4) — a secção LUZ do Inspector e os avisos da luz do quadro.
//!
//! ⚠️ Um ficheiro próprio pela razão do [`super::inspector_nav`]: uma família nova, e o
//! `inspector_game` perto do tecto. Rótulos de linha CURTOS; a explicação mora no aviso.

/// A tradução de uma chave `panel.inspector.luz.*` / `light2d.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.luz.title" => "Light",
        "panel.inspector.luz.type" => "Type",
        "panel.inspector.luz.point" => "Point",
        "panel.inspector.luz.ambient" => "Ambient",
        "panel.inspector.luz.color" => "Color",
        "panel.inspector.luz.intensity" => "Intensity",
        "panel.inspector.luz.size" => "Size",
        "panel.inspector.luz.limit_range" => "Limit Range",
        "panel.inspector.luz.range" => "Range",
        "panel.inspector.luz.white_at_d_m" => "A white sprite is fully lit at {d} m.",
        "panel.inspector.luz.ambient_n_percent" => {
            "Lights every open spot at {n} % \u{2014} 100 % is the scene as it was without light."
        }
        "panel.inspector.luz.editing_primary_of_n" => {
            "{n} objects selected \u{2014} editing the active one."
        }
        "panel.inspector.luz.grid_too_big" => {
            "All light is off \u{2014} the light grid ({lado} px) is larger than this GPU allows \
             ({limite} px). Make the window smaller."
        }
        "panel.inspector.luz.solver_unavailable" => {
            "All light is off \u{2014} this GPU cannot run the light."
        }
        "panel.inspector.luz.point_without_size" => {
            "A size of 0 is not drawn yet \u{2014} give the lamp a size."
        }
        "panel.inspector.luz.too_small_for_zoom" => {
            "The lamp is smaller than the light grid at this zoom, so it is dark. Zoom in or make \
             it bigger."
        }
        "panel.inspector.luz.outside_view" => {
            "The lamp is too far outside the view to light anything."
        }
        "panel.inspector.luz.sprites_by_box_n_of_total" => {
            "{n} of {total} glowing or shadow-casting sprites use their box, not their pixels \
             (mesh, tiling or baked texture)."
        }
        "light2d.toast.grid_too_big" => {
            "2D light is off: its grid ({lado} px) is larger than this GPU allows ({limite} px)."
        }
        "light2d.toast.solver_unavailable" => "2D light is off: this GPU cannot run it.",
        _ => return None,
    })
}
