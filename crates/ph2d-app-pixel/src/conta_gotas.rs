//! **O conta-gotas do seletor de cor partilhado, em Draw**: com ele armado, o clique sobre o sprite
//! lê a cor autorada (a da ferramenta conta-gotas, sem o `±1` da leitura do ecrã) e nunca pinta.
//! Fora do sprite o clique segue para o caminho da casa, que lê o ecrã.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_pixel_core::{Sample, pick};
use ph2d_tokens::ColorValue;

use crate::PixelApp;
use crate::input::{Canvas, cell, key_of};

/// A cor visível na célula sob `p`; `None` = fora do sprite.
pub(crate) fn amostra(app: &PixelApp, c: &Canvas<'_>, p: (f32, f32)) -> Option<[u8; 4]> {
    let bits = c.editing?;
    let key = key_of(c.world, bits)?;
    let (x, y) = cell(app, c, bits, key, p)?;
    let doc = app.docs.get(&key)?;
    let v = pick(&doc.sprite, doc.frame, x, y, Sample::AllLayers)?;
    Some(doc.sprite.rgba_of_value(v))
}

/// Entrega a amostra ao seletor armado e desarma-o. Célula vazia desarma sem trocar a cor: uma cor
/// de alfa zero pintaria nada.
pub fn entrega(store: &mut WidgetStore, [r, g, b, a]: [u8; 4]) {
    let Some(seletor) = store.eyedropper_pending() else {
        return;
    };
    if a > 0 {
        store.set_blender_value(seletor, ColorValue::from_rgba8(r, g, b, a));
    }
    store.set_eyedropper_pending(None);
}
