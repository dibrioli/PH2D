//! The brush **Shape** as a stack of z-ordered luminance layers, for the per-layer-colour mode (a
//! multi-colour, 3-D-looking tip). Captured from the Painter document's own raster layers (the active
//! sprite's layers, or an imported multi-layer doc). Held here because the heavy pixels can't live in
//! the `Copy` `BrushSpec`. Split from `paint.rs` for the workspace LOC cap.
//!
//! Two modes share the same captured stack:
//! - **Per-Layer Color OFF**: the layers are flattened (alpha-over, bottom→top, scaled by each layer's
//!   `opacity`) into the single [`super::PaintState::shape_image`], so the Shape behaves like a one-layer
//!   image and the Shape colour ramp works as before — the flatten is computed once at capture.
//! - **Per-Layer Color ON**: each layer paints with its OWN captured colours by default (or a custom tint
//!   when the artist checks its box), under its picked blend mode, scaled by its per-layer opacity; the
//!   higher layer paints above the lower one. The ramp section is hidden + nullified.

use super::brush_settings::BrushTextureImage;
use ph2d_painter_brush::texture::{ImageMask, ImageRgb};

/// Max Shape layers the per-layer-colour UI exposes (one row per layer) + the snapshot carries. A deep
/// stack past this is flattened-only (no per-layer colour) — well beyond any hand-authored brush tip.
pub const MAX_SHAPE_LAYERS: usize = 16;

/// The captured Shape layer stack + the per-layer-colour mode state. `layers` are **bottom-to-top**
/// (last = topmost), so the engine composites them in order. `color_on[i]` / `color[i]` align with
/// `layers[i]`. Empty `layers` ⇒ no multi-layer Shape (the single `shape_image` path is used).
///
/// Per-layer colour source: **Texture Color is the DEFAULT** — `color_on[i]` off ⇒ the layer paints its
/// OWN captured per-pixel RGB `rgb[i]` (or, with no captured RGB, the brush base colour). The "Layer N
/// Color" checkbox sets `color_on[i]` ⇒ a **custom tint** `color[i]` instead. The per-layer **blend** is
/// the panel-picked `manual_blend[i]` (the "B" dropdown). **Opacity** `opacity[i]` scales that layer's tip
/// contribution at recomposite AND mirrors the SOURCE document layer's opacity (`doc_ids[i]`): the box is
/// two-way with that layer's Layers-panel slider (guarded to the captured source, never the wrong layer).
#[derive(Default)]
pub(super) struct ShapeLayers {
    /// The ACTIVE silhouette per layer — **derived**, never authored: [`Self::rebuild_derived`] builds
    /// it from [`Self::src`] (o alpha autorado) or from the luminance of [`Self::lit`].
    layers: Vec<BrushTextureImage>,
    /// The **source plane** each layer was captured with: its ALPHA. Kept pristine (un-gained,
    /// un-chosen) because the silhouette law is a TOGGLE — deriving into `layers` and throwing the
    /// source away would make flipping the checkbox a re-capture, which the file / sprite paths
    /// cannot do (the document they came from may not even be open).
    ///
    /// ⚠️ **Custa um plano de `w·h` por camada** — ao lado dos `w·h·3` que a cor já guarda, +25 % do
    /// que o stack pesava. É o preço de o interruptor ser instantâneo em vez de destrutivo.
    src: Vec<BrushTextureImage>,
    /// O **ganho do relevo** do documento de origem (um byte por texel, [`super::impasto_gain`]),
    /// vazio quando não há relevo. Guardado em vez de pré-multiplicado na captura porque o derivado é
    /// re-construído a cada troca de fonte, e um ganho já assado nele seria aplicado duas vezes.
    ///
    /// ⚠️ **Ele pertence à COR, nunca à cobertura** (report do Enio, 2026-08-09, com foto: *"mesmo
    /// sem alpha na imagem … aparece um alpha, o branco no lugar da sombra, no carimbo pintado"*). A
    /// 1ª versão o multiplicava na MÁSCARA, e a consequência é aritmética: a sombra de um relevo é
    /// ganho `< 1`, então ela virava **cobertura parcial** e a tela aparecia através dela — um
    /// documento opaco passava a carimbar furado exatamente onde o artista esculpiu. O relevo
    /// SOMBREIA a tinta; ele não a perfura.
    gain: Vec<u8>,
    /// A cor capturada **com o relevo já sombreado** — `rgb × gain`, derivada como as máscaras e pela
    /// MESMA porta. É por aqui que o *"Use as Brush Shape não transfere os relevos"* (Enio, mesmo
    /// dia) chega ao carimbo: o slot pinta a APARÊNCIA do documento, e a aparência de tinta esculpida
    /// inclui a sombra dela.
    ///
    /// ⚠️ **A luminância lê DAQUI, não do `rgb` cru** — e é isso que mantém a metade que já shipava:
    /// no modo de silhueta por TOM a sombra continua a valer, porque o tom de tinta esculpida É mais
    /// escuro. O modo por ALPHA é que deixa de a sentir, que é precisamente o report.
    lit: Vec<Vec<u8>>,
    /// **A silhueta vem do ALPHA da imagem** (em vez das diferenças de claro e escuro dela).
    ///
    /// O default é o comportamento que cada rota JÁ tinha — a captura de documento silhueta pelo
    /// alpha autorado das camadas, uma imagem importada silhueta pelo tom —, e o checkbox é a
    /// escolha que faltava. Ele sobrevive a uma re-captura da MESMA fonte, como as cores por camada.
    alpha_from_image: bool,
    /// The mode toggle (the "Per-Layer Color" checkbox). Only meaningful with ≥ 1 captured layer.
    per_layer_color: bool,
    /// Per-layer "use a custom colour" toggle (the "Layer N Color" checkbox). `false` (default) ⇒ the
    /// layer paints its own captured texture colour (`rgb[i]`), or the brush base colour if none captured.
    color_on: Vec<bool>,
    /// Per-layer custom colour (straight RGB), used when `color_on[i]`.
    color: Vec<[f32; 3]>,
    /// Per-layer captured per-pixel **straight RGB** (`w·h·3`) — the default paint colour when `color_on[i]`
    /// is off. Empty ⇒ no real colour captured for that layer (falls back to the brush base colour).
    rgb: Vec<Vec<u8>>,
    /// Per-layer blend mode ([`ph2d_painter_effects::BlendMode`] discriminant; the "Blend" dropdown).
    /// Mirrors the SOURCE document layer's blend mode (`doc_ids[i]`) — a remote control, two-way with its
    /// Layers-panel blend chip. The bottom layer (index 0, the base) has none (Photoshop Background). `0` = Normal.
    manual_blend: Vec<u8>,
    /// Per-layer **opacity** `0..1` (the numeric box), applied as a scale on that layer's tip contribution
    /// at recomposite. Mirrors the SOURCE document layer's opacity (`doc_ids[i]`): the box edits that
    /// layer's slider (and vice versa, via the auto-recapture which re-reads it here).
    opacity: Vec<f32>,
    /// Per-layer source **document layer id** (`LayerId.0`) — so the opacity box edits that layer's opacity
    /// (two-way with its Layers-panel slider). `0` ⇒ no back-reference (a non-document capture).
    doc_ids: Vec<u64>,
    /// **O RELEVO capturado**, normalizado a `0..255` contra o próprio pico do documento — a FORMA da
    /// escultura, não a aparência dela. Vazio quando a fonte não tem relevo.
    ///
    /// ⚠️ **Ele é uma coisa DIFERENTE do [`Self::gain`], e a diferença é a wave inteira:** o ganho é
    /// *como a luz daquele documento sombreou aquele relevo* — um número já assado, que não reage a
    /// nada. Este é a forma que a luz do documento de DESTINO vai sombrear sozinha, e é por isso que
    /// o carimbo passa a ter brilho, especular e reação à luz que se move (Enio, 2026-08-09: *"não
    /// temos o relevo e o brilho e a reação à luz de uma imagem criada com Impasto"*).
    relief: Vec<u8>,
    /// O relevo capturado VIAJA com o carimbo (o default quando existe relevo).
    relief_from_image: bool,
    /// Bumped on any change that re-bakes the coloured stamp (layers / mode / a colour / a toggle).
    version: u64,
}

impl ShapeLayers {
    /// Replace the captured stack with `layers` (`(luminance, w, h)`, bottom-to-top), capped at
    /// [`MAX_SHAPE_LAYERS`]. Resets the per-layer colours to "off" (each layer paints its texture colour
    /// until the artist assigns a custom one). Does NOT change the mode toggle. Bumps the version.
    pub(super) fn set_layers(&mut self, layers: Vec<(Vec<u8>, u32, u32)>) {
        let n = layers.len().min(MAX_SHAPE_LAYERS);
        self.src = layers
            .into_iter()
            .take(n)
            .map(|(lum, w, h)| BrushTextureImage::new(lum, w, h))
            .collect();
        // O plano instalado É a silhueta, e esse é o default de TODA rota — o que muda é quem o
        // instala: a captura entrega o alpha autorado de cada camada, o arquivo entrega o que
        // ele tem. Quem quiser a outra lei chama [`Self::set_alpha_from_image`] logo em seguida.
        //
        // ⚠️ **Um default MEDIDO foi construído aqui e descartado** (*alpha quando ele recorta algo,
        // luminância quando é opaco*): ele é defensável e mudava o comportamento em DUAS direções
        // que ninguém pediu — uma captura toda opaca deixava de ser o quadrado que shipava, e um
        // `.png` recortado deixava de ser silhuetado pelo tom. O pedido era um CHECKBOX; o default
        // fica onde estava e a escolha passa a existir.
        self.alpha_from_image = true;
        self.gain.clear();
        self.relief.clear();
        self.color_on = vec![false; n];
        self.color = vec![[0.0, 0.0, 0.0]; n];
        // The captured `rgb` / `opacity` / `doc_ids` are re-filled by the capture path via
        // `set_layers_meta`; `restore_assignments` re-applies the artist's `color_on` / `color` /
        // `manual_blend` across a same-source re-capture. Opacity is NOT preserved — it mirrors the live
        // source layer, re-read each capture. Reset to neutral defaults here.
        self.rgb = vec![Vec::new(); n];
        self.manual_blend = vec![0; n];
        self.opacity = vec![1.0; n];
        self.doc_ids = vec![0; n];
        self.rebuild_derived();
    }

    /// Re-derive the ACTIVE masks from the pristine sources. Bumps the version.
    ///
    /// ⚠️ **UMA porta decide de onde a silhueta vem**, e ela é perguntada aqui — não no carimbo, não
    /// no flatten, não na captura. O carimbo por-camada (**Per-Layer Color**) lê `masks()` cruas e o
    /// caminho de imagem única lê o `flatten()`; os dois derivam DESTE plano, então não há um segundo
    /// sítio onde alguém possa esquecer a escolha do artista (é exatamente o defeito que o ganho do
    /// relevo teve em 2026-08-09, aplicado no flatten e ausente do modo colorido).
    fn rebuild_derived(&mut self) {
        // (1) A COR sombreada pelo relevo. Ela vem primeiro porque a silhueta por TOM lê dela.
        //
        // ⚠️ **E ela NÃO é sombreada quando o relevo VIAJA — esta é a espinha da wave, e o erro que
        // ela evita é aritmético e mudo.** Se o carimbo deposita relevo de verdade, quem sombreia é a
        // luz do documento de DESTINO; assar a sombra da luz de ORIGEM na cor faria a sombra pousar
        // DUAS vezes — a mesma sombra escura multiplicada por si mesma, num carimbo que reage à luz e
        // já vinha reagindo. Uma das duas tem de existir, nunca as duas:
        //
        //   - relevo VIAJA  ⇒ cor PRISTINA (`rgb`), a luz do destino faz a sombra, e ela se move
        //     quando o artista move a lâmpada — que é o pedido;
        //   - relevo NÃO viaja ⇒ cor SOMBREADA (`rgb × gain`), a aparência assada do documento de
        //     origem, que é o que shipou de manhã e continua certo para quem só quer a APARÊNCIA.
        let gain = if self.relief_travels() {
            &[][..]
        } else {
            &self.gain[..]
        };
        self.lit = self
            .rgb
            .iter()
            .map(|rgb| {
                if gain.len() * 3 < rgb.len() {
                    // Sem relevo (ou de outra medida) o ganho não se aplica: a cor é a capturada, ao
                    // byte. É esta linha que faz de todo documento sem escultura um no-op exato.
                    return rgb.clone();
                }
                let mut out = rgb.clone();
                for (k, px) in out.chunks_exact_mut(3).enumerate() {
                    let g = u32::from(gain[k]);
                    for c in px {
                        let v = u32::from(*c) * g / u32::from(super::impasto_gain::FLAT_PROBE);
                        *c = v.min(255) as u8;
                    }
                }
                out
            })
            .collect();
        // (2) A SILHUETA: o alpha autorado, ou o tom — e o tom sai da cor JÁ sombreada.
        self.layers = self
            .src
            .iter()
            .enumerate()
            .map(|(i, plane)| {
                let (alpha, w, h) = plane.parts();
                let n = (w as usize) * (h as usize);
                // A luminância sai do RGB já capturado — guardar um terceiro plano para ela seria
                // guardar o que já está ali (Rec.601, os mesmos pesos 77/150/29 do resto do tool).
                let lum = (!self.alpha_from_image)
                    .then(|| self.lit.get(i))
                    .flatten()
                    .filter(|rgb| rgb.len() >= n * 3)
                    .map(|rgb| {
                        (0..n)
                            .map(|k| {
                                ((u32::from(rgb[k * 3]) * 77
                                    + u32::from(rgb[k * 3 + 1]) * 150
                                    + u32::from(rgb[k * 3 + 2]) * 29)
                                    >> 8) as u8
                            })
                            .collect::<Vec<u8>>()
                    });
                BrushTextureImage::new(lum.unwrap_or_else(|| alpha.to_vec()), w, h)
            })
            .collect();
        self.version = self.version.wrapping_add(1);
    }

    /// A silhueta vem do ALPHA da imagem?
    pub(super) fn alpha_from_image(&self) -> bool {
        self.alpha_from_image
    }

    /// Há uma fonte ALTERNATIVA para a silhueta — ou seja, o interruptor tem para onde virar. Sem RGB
    /// capturado a luminância não existe, e um checkbox que não muda nada é o controle morto que este
    /// painel recusa a pintar.
    pub(super) fn has_alpha_choice(&self) -> bool {
        !self.src.is_empty() && self.rgb.iter().any(|r| !r.is_empty())
    }

    /// Vira o interruptor da silhueta e re-deriva as máscaras.
    pub(super) fn toggle_alpha_from_image(&mut self) {
        self.set_alpha_from_image(!self.alpha_from_image);
    }

    /// Declara de onde a silhueta vem e re-deriva. Chamado por quem INSTALA os planos quando a lei
    /// da sua rota não é o default (uma imagem importada silhueta pelo TOM, não pelo alpha — o
    /// arquivo pode nem ter um).
    pub(super) fn set_alpha_from_image(&mut self, on: bool) {
        self.alpha_from_image = on;
        self.rebuild_derived();
    }

    /// Instala o **ganho do relevo** do documento de origem e re-deriva. Vazio = sem relevo.
    pub(super) fn set_gain(&mut self, gain: Vec<u8>) {
        self.gain = gain;
        self.rebuild_derived();
    }

    /// Instala a **FORMA do relevo** capturada (normalizada) e arma a viagem dele. Vazio = sem relevo.
    ///
    /// ⚠️ **Armar aqui é o que torna a capacidade automática**, que é o que o pedido diz: o relevo é
    /// propriedade da IMAGEM, não um ajuste do pincel — quem capturou uma escultura espera carimbá-la,
    /// sem procurar um interruptor num painel que o modo Digital nem mostra.
    pub(super) fn set_relief(&mut self, relief: Vec<u8>) {
        self.relief_from_image = !relief.is_empty();
        self.relief = relief;
        self.rebuild_derived();
    }

    /// O relevo VIAJA com o carimbo? Só então a cor sai pristina — ver [`Self::rebuild_derived`].
    pub(super) fn relief_travels(&self) -> bool {
        self.relief_from_image && !self.relief.is_empty()
    }

    /// A FORMA do relevo capturada, para o dab a carimbar. `None` quando ela não viaja.
    pub(super) fn relief_plane(&self) -> Option<&[u8]> {
        self.relief_travels().then_some(self.relief.as_slice())
    }

    /// Liga/desliga a viagem do relevo e re-deriva — a cor muda com ele (a lei do duplo-sombreamento).
    ///
    /// ⚠️ **A capacidade é AUTOMÁTICA e este interruptor ainda não tem face no painel** — nomeado, não
    /// escondido. O pedido era *"mesmo pintando com o Digital"*, e uma captura com escultura arma a
    /// viagem sozinha; o que falta é a metade de DESLIGAR, que é uma row na seção Shape (pintada só
    /// quando a captura tem relevo — a lei do knob morto). Enquanto ela não existe, quem quiser a
    /// aparência assada de volta re-captura sem escultura visível.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "a porta que a row do painel vai chamar; hoje só os gates a usam"
        )
    )]
    pub(super) fn set_relief_from_image(&mut self, on: bool) {
        self.relief_from_image = on;
        self.rebuild_derived();
    }

    /// Fill the per-layer captured metadata in lock-step with [`Self::set_layers`] (same order / count):
    /// each layer's per-pixel straight `rgb` (`w·h·3`, empty if unavailable), its current document `opacity`
    /// and `blend` (mirrored into the box + dropdown), and its source document `doc_ids` (`LayerId.0`).
    /// Called by the capture path after `set_layers`. Bumps the version (the real-colour bake reads `rgb`).
    pub(super) fn set_layers_meta(
        &mut self,
        rgb: Vec<Vec<u8>>,
        opacity: Vec<f32>,
        blend: Vec<u8>,
        doc_ids: Vec<u64>,
    ) {
        for i in 0..self.layers.len() {
            if let Some(r) = rgb.get(i) {
                self.rgb[i] = r.clone();
            }
            if let Some(&o) = opacity.get(i) {
                self.opacity[i] = o.clamp(0.0, 1.0);
            }
            if let Some(&b) = blend.get(i) {
                self.manual_blend[i] = b;
            }
            if let Some(&d) = doc_ids.get(i) {
                self.doc_ids[i] = d;
            }
        }
        // O RGB é a fonte do ramo de LUMINÂNCIA — instalá-lo sem re-derivar deixaria a máscara
        // descrevendo a captura anterior.
        self.rebuild_derived();
    }

    /// Drop the stack (revert to the single-image / falloff Shape).
    pub(super) fn clear(&mut self) {
        self.layers.clear();
        self.src.clear();
        self.gain.clear();
        self.relief.clear();
        self.relief_from_image = false;
        self.lit.clear();
        self.alpha_from_image = false;
        self.color_on.clear();
        self.color.clear();
        self.rgb.clear();
        self.manual_blend.clear();
        self.opacity.clear();
        self.doc_ids.clear();
        self.per_layer_color = false;
        self.version = self.version.wrapping_add(1);
    }

    /// Number of captured layers.
    pub(super) fn len(&self) -> usize {
        self.layers.len()
    }

    /// No layers captured.
    pub(super) fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// The per-layer-colour mode is engaged (toggle on + at least one captured layer) — the route
    /// dispatcher uses this to pick the coloured-stamp path, and the panel to hide the ramp section.
    pub(super) fn is_color_mode(&self) -> bool {
        self.per_layer_color && !self.layers.is_empty()
    }

    /// The mode toggle (raw, regardless of whether layers are present).
    pub(super) fn per_layer_color(&self) -> bool {
        self.per_layer_color
    }

    /// Monotonic version (cache invalidation key for the coloured stamp).
    pub(super) fn version(&self) -> u64 {
        self.version
    }

    /// Borrow the layers as engine [`ImageMask`]s (bottom-to-top), for the coloured-stamp bake.
    pub(super) fn masks(&self) -> Vec<ImageMask<'_>> {
        self.layers.iter().map(BrushTextureImage::as_mask).collect()
    }

    /// Each layer's resolved colour: its custom pick when `color_on[i]`, else the brush base `brush_color`
    /// (the "Color" at the top of the brush panel) — exactly the substitution the spec calls for.
    pub(super) fn resolved_colors(&self, brush_color: [f32; 3]) -> Vec<[f32; 3]> {
        (0..self.layers.len())
            .map(|i| {
                if self.color_on.get(i).copied().unwrap_or(false) {
                    self.color.get(i).copied().unwrap_or(brush_color)
                } else {
                    brush_color
                }
            })
            .collect()
    }

    /// Snapshot the per-layer **artist assignments** `(color_on, color)` — the only state the artist owns —
    /// for re-applying across an automatic re-capture of the same Shape source (so editing the source
    /// sprite keeps the custom colours). The captured `rgb` / `opacity` / `blend` / `doc_ids` are NOT
    /// snapshotted: opacity + blend MIRROR the live source layer, so they are re-read fresh each capture.
    pub(super) fn assignments_snapshot(&self) -> (Vec<bool>, Vec<[f32; 3]>, bool) {
        (
            self.color_on.clone(),
            self.color.clone(),
            self.alpha_from_image,
        )
    }

    /// Re-apply an [`Self::assignments_snapshot`] by index (up to the current layer count) after a
    /// re-capture, bumping the version so the cached stamps + preview re-bake with the restored state.
    pub(super) fn restore_assignments(
        &mut self,
        on: &[bool],
        color: &[[f32; 3]],
        alpha_from_image: bool,
    ) {
        for i in 0..self.layers.len() {
            if let (Some(&o), Some(c)) = (on.get(i), color.get(i)) {
                self.color_on[i] = o;
                self.color[i] = *c;
            }
        }
        // ⚠️ A escolha da silhueta é do ARTISTA e sobrevive a uma re-captura da MESMA fonte, como as
        // cores; uma fonte NOVA volta ao default medido dela, que é o certo (é outra imagem).
        self.alpha_from_image = alpha_from_image;
        self.rebuild_derived();
    }

    /// Flatten the stack into one luminance silhouette (alpha-over, bottom→top) for the OFF-mode / ramp /
    /// preview single-image path — each layer's coverage scaled by its `opacity[i]` (so a half-opacity
    /// layer shows the ones below through it). `None` if there are no layers (or they disagree on size).
    pub(super) fn flatten(&self) -> Option<(Vec<u8>, u32, u32)> {
        let first = self.layers.first()?;
        let (w, h) = {
            let (_, w, h) = first.parts();
            (w, h)
        };
        let n = (w as usize) * (h as usize);
        let mut acc = vec![0.0f32; n];
        for (li, layer) in self.layers.iter().enumerate() {
            let (lum, lw, lh) = layer.parts();
            if lw != w || lh != h || lum.len() < n {
                return None; // require uniform size (document layers share the canvas size)
            }
            let op = self.opacity.get(li).copied().unwrap_or(1.0);
            for (a_dst, &px) in acc.iter_mut().zip(lum.iter()) {
                let a = f32::from(px) / 255.0 * op;
                *a_dst = a + *a_dst * (1.0 - a); // over-composite the coverage
            }
        }
        let lum = acc
            .iter()
            .map(|&a| (a * 255.0 + 0.5).clamp(0.0, 255.0) as u8)
            .collect();
        Some((lum, w, h))
    }

    /// Toggle the per-layer-colour mode (the "Per-Layer Color" checkbox). Bumps the version.
    pub(super) fn toggle_per_layer_color(&mut self) {
        self.per_layer_color = !self.per_layer_color;
        self.version = self.version.wrapping_add(1);
    }

    /// Toggle layer `i`'s custom-colour checkbox (off ⇒ it paints the brush base colour). Bumps version.
    pub(super) fn toggle_layer_color(&mut self, i: usize) {
        if let Some(on) = self.color_on.get_mut(i) {
            *on = !*on;
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Set layer `i`'s blend mode ([`ph2d_painter_effects::BlendMode`] discriminant). Bumps version. The
    /// "B" dropdown.
    pub(super) fn set_manual_blend(&mut self, i: usize, mode: u8) {
        if let Some(b) = self.manual_blend.get_mut(i) {
            *b = mode;
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Set layer `i`'s **opacity** (`0..1`, the numeric box) — the brush-side scale on its tip
    /// contribution. The caller (`set_brush_shape_layer_opacity`) also writes the source document layer's
    /// opacity (so the box is two-way with its Layers-panel slider). Bumps version (re-bakes the preview).
    pub(super) fn set_opacity(&mut self, i: usize, v: f32) {
        if let Some(o) = self.opacity.get_mut(i) {
            *o = v.clamp(0.0, 1.0);
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Layer `i`'s opacity (`0..1`), `1.0` by default.
    pub(super) fn opacity(&self, i: usize) -> f32 {
        self.opacity.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0)
    }

    /// Layer `i`'s source document layer id (`LayerId.0`), or `None` for a non-document capture.
    pub(super) fn doc_id(&self, i: usize) -> Option<u64> {
        self.doc_ids.get(i).copied().filter(|&d| d != 0)
    }

    /// Per-layer opacities (bottom-to-top), for the recomposite's per-layer scale.
    pub(super) fn opacities(&self) -> Vec<f32> {
        (0..self.layers.len()).map(|i| self.opacity(i)).collect()
    }

    /// Any layer paints its captured **texture colour** (`color_on[i]` off AND it has captured RGB) — this
    /// is the default mode and drives the route to the per-pixel dynamic path (which samples that RGB).
    pub(super) fn any_texture_color(&self) -> bool {
        (0..self.layers.len()).any(|i| self.rgb_image(i).is_some())
    }

    /// Layer `i`'s blend mode ([`ph2d_painter_effects::BlendMode`] discriminant; always the manual pick).
    pub(super) fn effective_blend(&self, i: usize) -> u8 {
        self.manual_blend.get(i).copied().unwrap_or(0)
    }

    /// Borrow layer `i`'s captured RGB as an engine [`ImageRgb`] for the default **texture-colour** path —
    /// `Some` only when the layer is NOT custom-coloured (`color_on` off) AND has captured RGB; `None`
    /// otherwise (custom tint, or no RGB captured ⇒ the flat brush-base fallback).
    pub(super) fn rgb_image(&self, i: usize) -> Option<ImageRgb<'_>> {
        if self.color_on.get(i).copied().unwrap_or(false) {
            return None;
        }
        let rgb = self.lit.get(i)?;
        if rgb.is_empty() {
            return None;
        }
        let (_, w, h) = self.layers.get(i)?.parts();
        (rgb.len() >= (w as usize) * (h as usize) * 3).then_some(ImageRgb {
            rgb,
            width: w,
            height: h,
        })
    }

    /// Set layer `i`'s custom colour (straight RGB, clamped); turns its checkbox on. Bumps version.
    pub(super) fn set_layer_color(&mut self, i: usize, rgb: [f32; 3]) {
        if let (Some(c), Some(on)) = (self.color.get_mut(i), self.color_on.get_mut(i)) {
            *c = [
                rgb[0].clamp(0.0, 1.0),
                rgb[1].clamp(0.0, 1.0),
                rgb[2].clamp(0.0, 1.0),
            ];
            *on = true;
            self.version = self.version.wrapping_add(1);
        }
    }

    /// `(count, color_on, color)` for the panel snapshot — fixed-size arrays so the snapshot stays `Copy`.
    pub(super) fn snapshot(&self) -> (u8, [bool; MAX_SHAPE_LAYERS], [[f32; 3]; MAX_SHAPE_LAYERS]) {
        let mut on = [false; MAX_SHAPE_LAYERS];
        let mut col = [[0.0; 3]; MAX_SHAPE_LAYERS];
        for i in 0..self.layers.len().min(MAX_SHAPE_LAYERS) {
            on[i] = self.color_on.get(i).copied().unwrap_or(false);
            col[i] = self.color.get(i).copied().unwrap_or([0.0; 3]);
        }
        (self.layers.len().min(MAX_SHAPE_LAYERS) as u8, on, col)
    }

    /// `(manual_blend, opacity)` fixed-size arrays for the panel snapshot (the "B" blend dropdown state +
    /// the opacity box per row). Entries `0..count` are valid.
    pub(super) fn panel_blend_arrays(&self) -> ([u8; MAX_SHAPE_LAYERS], [f32; MAX_SHAPE_LAYERS]) {
        let mut blend = [0u8; MAX_SHAPE_LAYERS];
        let mut opacity = [1.0f32; MAX_SHAPE_LAYERS];
        for i in 0..self.layers.len().min(MAX_SHAPE_LAYERS) {
            blend[i] = self.manual_blend.get(i).copied().unwrap_or(0);
            opacity[i] = self.opacity(i);
        }
        (blend, opacity)
    }
}
