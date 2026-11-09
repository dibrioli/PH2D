//! As teclas das ferramentas como o artista as lê, derivadas do mapa VIVO (U5 do fecho P12): o
//! tooltip, a coluna, o título do cartão e o aviso da troca leem ISTO, nunca uma tabela à parte.

use crate::{Context, Keymap, PixelAction};
use ph2d_editor_core::pixel_panel::{PixelTool as T, PixelToolKeys};

/// A primeira tecla de cada ferramenta no mapa (a do contexto Normal primeiro); `R` vale para o
/// Blur e para o Jumble (a mesma acção alterna-os).
#[must_use]
pub fn tool_keys(map: &Keymap) -> PixelToolKeys {
    let mut out = Vec::new();
    for &a in PixelAction::ALL {
        let tools = match (a, a.tool()) {
            (PixelAction::ToolBlurJumble, _) => vec![T::Blur, T::Jumble],
            (_, Some(t)) => vec![t],
            _ => continue,
        };
        let keys = map.keys_of(a);
        let Some((chord, _)) = keys
            .iter()
            .find(|(_, c)| *c == Context::Normal)
            .or_else(|| keys.first())
        else {
            continue;
        };
        out.extend(tools.iter().map(|&t| (t, chord.to_string())));
    }
    PixelToolKeys(out)
}
