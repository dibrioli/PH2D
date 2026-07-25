//! **A cena pronta para o smoke das COLOR HARMONIES** — `PH2D_HARMONY_SMOKE=1`.
//!
//! O picker de cor baseado no Blender é a superfície ÚNICA de cor do app inteiro
//! (Painter/Vector/Inspector o abrem via `register_picker_swatch`). Esta cena abre
//! esse picker flutuante já semeado com uma base laranja SATURADA e o esquema
//! **Triad** selecionado — então a seção **Color Harmonies** aparece de imediato,
//! com o seletor de 7 esquemas + a tira das parceiras derivadas + o botão "+".
//!
//! O que conferir:
//! - trocar de esquema (Off/Comp/Anlg/Triad/Split/Tetra/Mono) muda a tira de parceiras;
//! - clicar numa parceira a adota como cor ativa (a base gira para ela — o modelo "linked");
//! - "+" acrescenta todas as parceiras à paleta;
//! - mover a base (roda / hex / chips) gira TODAS as parceiras pelo mesmo Δ.
//!
//! Hues medidos com a base laranja (matiz 29,9°):
//! Comp `[29,9, 209,9]` · Triad `[29,9, 149,9, 269,9]` · Tetrad `[29,9, 119,7, 209,9, 299,7]`.

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn harmony_smoke(&mut self) {
        if self.harmony_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_HARMONY_SMOKE").is_none() {
            return;
        }
        // Precisa do `hero_screen` (a store dos widgets vive nele); tenta no próximo frame.
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        self.harmony_smoke_done = true;

        let id = ph2d_editor::ids::INSP_BLENDER_PICKER;
        // Abre o picker flutuante (ele pinta enquanto `picker_target()` é Some).
        hero.store.set_picker_target(Some(id));
        // Base laranja saturada (matiz ~30°) — um cinza não teria matiz para girar.
        hero.store
            .set_blender_value(id, ph2d_tokens::ColorValue::from_rgba8(230, 126, 23, 255));
        // Já entra na Triad para a tira de parceiras aparecer de cara.
        hero.store
            .set_blender_harmony(id, ph2d_editor::widget::Harmony::Triad);
    }
}
