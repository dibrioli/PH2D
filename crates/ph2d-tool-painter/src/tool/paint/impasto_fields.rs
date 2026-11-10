//! Os CAMPOS que a luz do Impasto lê — a dobra das camadas, emprestada, e a dobra através dos ajustes de
//! vizinhança (Gaussiano/Nitidez). [Corte de LOC de `impasto_light`, B52.R14: o construtor mora aqui; a luz, lá.]

use super::impasto_light::ReliefLayer;
use super::relief_fields::ReliefFields;
use crate::tool::PainterTool;

impl PainterTool {
    /// The relief and the paint coverage the light must read, as BORROWED slices — never materialised.
    ///
    /// Building the composed fields into two canvas-sized buffers is the obvious thing, and it is what
    /// this did first: it cost an `O(canvas)` allocate-and-sum on **every frame**, while the pass itself
    /// only ever lights the dirty rect. At 2048² that is the difference between 3.9 ms and 2.1 ms per
    /// move — most of the impasto budget spent composing pixels nobody was going to look at.
    ///
    /// Heights **add** across layers (more paint IS thicker); coverage takes the **max** (it is a
    /// presence, not a quantity — two layers of paint over one pixel do not make it 200% paint). Per-layer
    /// Subtract / Replace / Ignore is named and deferred. A hidden layer contributes neither: hide it and
    /// its paint stops catching the light.
    pub(super) fn impasto_fields(&self) -> Option<ReliefFields<'_>> {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if n == 0 {
            return None;
        }
        let active = self.layers.active();
        // The open stroke rides on the active layer — which may not have a committed entry yet.
        let live_visible = active.is_some_and(|a| self.layers.effectively_visible(a));
        let live_h =
            (live_visible && self.paint.relief.stroke_height.len() == n && !self.paint.eraser)
                .then_some(self.paint.relief.stroke_height.as_slice());
        let live_c =
            (live_visible && self.paint.relief.stroke_film.len() == n && !self.paint.eraser)
                .then_some(self.paint.relief.stroke_film.as_slice());

        // Bottom-up, because `Level` is not commutative. Only layers that were actually sculpted have
        // an entry — the map is lazy, so this is empty for every document nobody has used Impasto on
        // (plus the active layer, when a live stroke is in flight on a layer that has none yet).
        let mut layers: Vec<ReliefLayer<'_>> = Vec::new();
        for id in self.layers.relief_layers_bottom_up() {
            let is_active = active == Some(id);
            let height = self
                .heights
                .get(&id)
                .map(|f| f.as_slice())
                .filter(|f| f.len() == n);
            let carries_live = is_active && live_h.is_some();
            if height.is_none() && !carries_live {
                continue;
            }
            let Some(layer) = self.layers.get(id) else {
                continue;
            };
            layers.push(ReliefLayer {
                id,
                height,
                cover: self
                    .covers
                    .get(&id)
                    .map(|c| c.as_slice())
                    .filter(|c| c.len() == n),
                mat: self
                    .mats
                    .get(&id)
                    .map(|m| m.as_slice())
                    .filter(|m| m.len() == n),
                depth: layer.impasto_depth,
                composite: layer.impasto_composite,
                active: is_active,
            });
        }
        // ⚠️ Sem camada de relevo E sem SUBSTRATO não há nada a iluminar. **Este guarda é irmão do
        // [`Self::impasto_visible`] e os dois têm de concordar.** Corrigir só um deixa o passe correr e
        // desistir aqui: a luz nunca acende, todos os outros gates passam, e o modo de falha é
        // exatamente *"a feature não faz nada"*. É a lei que esta casa já pagou com o default de mídia
        // do Painter — *a regra de ENTRADA mascara a de SAÍDA* —, e é por isso que há um gate por
        // camada (`the_paper_lights_on_a_canvas_with_no_paint_at_all` nasceu VERMELHO aqui).
        if layers.is_empty() && self.substrate().is_none() {
            return None;
        }
        let substrate = self.substrate();
        // §BUG-papel: a tinta de cada camada raster visível, para o dente se esconder sob ela. Sem o papel do
        // documento (`papel` `None`, a sprite crua É a camada de baixo) a camada de BAIXO é o papel: não o tapa.
        let tinta = if substrate.is_some() {
            let base = self.papel.is_none().then(|| {
                self.layers.z_order_bottom_up().into_iter().find(|&id| {
                    self.layers
                        .get(id)
                        .is_some_and(|l| matches!(l.kind, crate::layers::LayerKind::Raster(_)))
                })
            });
            self.layers
                .z_order_bottom_up()
                .into_iter()
                .filter(|&id| self.layers.effectively_visible(id) && !self.layers.is_mask(id))
                .filter(|&id| base.flatten() != Some(id))
                .filter_map(|id| {
                    let l = self.layers.get(id)?;
                    if !matches!(l.kind, crate::layers::LayerKind::Raster(_)) {
                        return None;
                    }
                    let px: &[u8] = if active == Some(id) {
                        self.canvas_rgba.as_slice()
                    } else {
                        self.images.get(&id)?.rgba8.as_slice()
                    };
                    (px.len() == n * 4).then_some((
                        px,
                        l.opacity.clamp(0.0, 1.0),
                        self.tintas_molhadas.contains(&id),
                    ))
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut fields = ReliefFields {
            folded: None,
            substrate,
            tinta,
            layers,
            live_h,
            live_c,
            live_mat: self.paint.brush.material().to_bytes(),
            neutral: ph2d_painter_brush::material::Material::NEUTRAL.to_bytes(),
            width: w as usize,
            height: h as usize,
        };
        // ⭐ Um Gaussiano ou uma Nitidez por cima de uma camada com relevo borram-no como borram a cor:
        // a dobra materializa-se pela porta única; sem eles, a dobra por píxel.
        let plan = self.layers.relief_plan();
        if crate::layers::relief_plan_filters(&plan) {
            let mascaras = self.relief_filter_masks(&plan);
            let folded = crate::layers::fold_relief_through(
                &plan,
                &super::relief_fields::Amostras {
                    f: &fields,
                    mascaras,
                },
                &ph2d_painter_effects::adjustments::AdjustWindow::full(w, h),
            );
            fields.folded = folded;
        }
        Some(fields)
    }

    /// As máscaras dos ajustes do plano, `0..1` por píxel (a luminância, como o compositor as lê).
    fn relief_filter_masks(
        &self,
        plan: &[crate::layers::ReliefStep],
    ) -> std::collections::BTreeMap<crate::layers::LayerId, Vec<f32>> {
        let active = self.layers.active();
        plan.iter()
            .filter_map(|s| match *s {
                crate::layers::ReliefStep::Filter {
                    mask: Some((m, _)), ..
                } => Some(m),
                _ => None,
            })
            .filter_map(|m| {
                let rgba: &[u8] = if active == Some(m) {
                    self.canvas_rgba.as_slice()
                } else {
                    self.images.get(&m)?.rgba8.as_slice()
                };
                let n = rgba.len() / 4;
                Some((
                    m,
                    (0..n)
                        .map(|i| crate::compositor::mask_value(rgba, i))
                        .collect(),
                ))
            })
            .collect()
    }
}
