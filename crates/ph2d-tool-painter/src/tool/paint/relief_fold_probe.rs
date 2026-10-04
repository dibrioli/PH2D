//! ⭐ **A sonda da dobra do relevo para o gate cruzado** (`docs/3D/30` §5: *«a dobra do relevo é
//! UMA»*) — a peça 3D compara a dobra dela com a do 2D sobre os MESMOS planos, e o 2D responde pelo
//! amostrador da própria luz (`ReliefFields::height_at`), nunca por uma redacção para o gate.

use super::*;
use crate::layers::ReliefComposite;

/// Uma camada da fixtura: o relevo e a cobertura por píxel, e o metadado do relevo.
#[derive(Clone, Debug)]
pub struct ReliefPlaneProbe {
    pub height: Vec<f32>,
    pub cover: Vec<u8>,
    pub depth: f32,
    pub composite: ReliefComposite,
    pub visible: bool,
}

impl PainterTool {
    /// ⭐ **O relevo composto que a luz do 2D lê** numa tela `w × h` nova cujas camadas (de baixo para
    /// cima) levam exactamente estes planos — píxel a píxel, depois do tecto de vidro (identidade
    /// abaixo do joelho). `None` se um plano não tem `w · h` amostras.
    #[doc(hidden)]
    #[must_use]
    pub fn composed_relief_of_planes(
        w: u32,
        h: u32,
        layers: &[ReliefPlaneProbe],
    ) -> Option<Vec<f32>> {
        let n = (w as usize) * (h as usize);
        if layers
            .iter()
            .any(|l| l.height.len() != n || l.cover.len() != n)
        {
            return None;
        }
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; n * 4], w, h);
        for (k, l) in layers.iter().enumerate() {
            let id = match (k, t.layers.active()) {
                (0, Some(base)) => base,
                _ => t.add_raster_layer(format!("probe {k}"))?,
            };
            t.heights.insert(id, std::sync::Arc::new(l.height.clone()));
            t.covers.insert(id, std::sync::Arc::new(l.cover.clone()));
            t.layers.set_impasto_depth(id, l.depth);
            t.layers.set_impasto_composite(id, l.composite);
            t.layers.set_visible(id, l.visible);
        }
        t.sync_relief_flags();
        let f = t.impasto_fields()?;
        Some(
            (0..i64::from(h))
                .flat_map(|y| (0..i64::from(w)).map(move |x| (x, y)))
                .map(|(x, y)| f.height_at(x, y))
                .collect(),
        )
    }
}
