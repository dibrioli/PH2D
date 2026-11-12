//! ⭐⭐⭐ **A JANELA em que a CENA vive — a porta única de todo mapeamento mundo↔tela do chrome.**
//!
//! # A lei, e as quatro vezes que ela foi paga

use ph2d_app_motion::field_gizmo::scene_camera_window;
use ph2d_editor_core::screens::layout::CenterSplit;
use ph2d_host::WindowSize;

/// **A mesma janela, quando o `hero` JÁ está emprestado** — a forma que o empréstimo obriga.
///
/// ⛔⛔ **Ela não é uma segunda resposta: é a MESMA conta com menos pedido.** O
/// [`crate::AppGfx::scene_window`] pede o `AppGfx` inteiro, e metade dos gestos do canvas corre
/// dentro de um `if let Some(hero) = gfx.hero_screen.as_mut()` — ali o `gfx` inteiro já não se
/// empresta, e **o compilador recusa**. Foi isso, e não desleixo, que fez a lei ser violada em 50
/// sítios: *a porta pedia mais do que o sítio tinha para dar*, e o caminho que compilava era o
/// errado (`gfx.surface.size()`, que toca UM campo).
///
/// ⇒ quem tem o `hero` na mão passa o split dele e o tamanho da superfície, que são campos
/// **disjuntos**. As duas formas chamam a mesma função da crate do Motion.
pub(crate) fn janela(split: CenterSplit, superficie: WindowSize) -> WindowSize {
    scene_camera_window(split, superficie)
}

impl crate::AppGfx {
    /// **A janela da CENA** — a que toda conta mundo↔tela do chrome tem de usar.
    ///
    /// ⚠️ Fora do split é a janela inteira, bit a bit (ver o cabeçalho).
    pub(crate) fn scene_window(&self) -> WindowSize {
        let split = self
            .hero_screen
            .as_ref()
            .map_or(CenterSplit::None, |h| h.view.center_split);
        scene_camera_window(split, self.surface.size())
    }
}
