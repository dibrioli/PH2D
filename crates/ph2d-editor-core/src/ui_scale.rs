//! ⭐⭐ **A escala da interface inteira** (ordem do dono, 2026-10-02) — spec
//! `docs/UI_New_and_Simple/spec/05_a_escala_da_interface.md`.
//!
//! A interface vive em pixels LÓGICOS; a janela, a câmera e os passes de GPU em FÍSICOS. Este
//! módulo é a ÚNICA porta entre os dois: o quadro desenha o chrome num viewport lógico e entrega a
//! cena sob `Affine::scale(s)`; o ponteiro que pergunta ao chrome é dividido por `s`; um rect do
//! chrome que vai a um pass físico é multiplicado. ⚠️ A `100 %` toda conversão é a identidade AO BIT
//! (`x / 1,0 == x`).
//!
//! ⛔ **Recusado (medido 02/10): escalar os TOKENS** (o `EDSCALE` da Godot) — `1 647` literais
//! `LITERAL-PX-OK` e `~80` constantes de compilação (`ROW_H_PX`, `MENU_BAR_H`, `DOCK_W_*`) ficariam
//! de fora: a interface escalaria pela metade.

use crate::zones::Rect;
use ph2d_tokens::UiScale;
use ph2d_vector::VectorScene;

thread_local! {
    static ACTIVE: std::cell::Cell<UiScale> = const { std::cell::Cell::new(UiScale::P100) };
    /// A cena lógica, reutilizada entre quadros (o Vello guarda as alocações) — um rascunho de
    /// pintura, não estado: nasce vazia em cada [`pintar_no_chrome`].
    static CHROME_LOGICO: std::cell::RefCell<VectorScene> = std::cell::RefCell::new(VectorScene::new());
}

/// ⭐⭐ **Pinta `f` no espaço LÓGICO do chrome e cola o resultado em `scene` sob a escala** — a porta
/// de saída. `f` recebe o viewport lógico e a cena onde pintar. ⚠️ A `100 %` é o caminho de sempre,
/// byte a byte: `f` pinta directamente em `scene`, sem cena intermédia nem `append`.
pub fn pintar_no_chrome<R>(
    mapa: UiScaleMap,
    viewport: Rect,
    scene: &mut VectorScene,
    f: impl FnOnce(Rect, &mut VectorScene) -> R,
) -> R {
    if mapa.is_identity() {
        return f(viewport, scene);
    }
    CHROME_LOGICO.with(|c| {
        let mut chrome = c.borrow_mut();
        chrome.reset();
        let r = f(mapa.rect_to_logical(viewport), &mut chrome);
        let s = f64::from(mapa.factor());
        scene.append_transformed(&chrome, ph2d_vector::Affine::scale(s));
        r
    })
}

/// Publica a escala do quadro — o `paint_hero_screen` chama-a, como faz com o estilo do texto.
pub fn publish(scale: UiScale) {
    ACTIVE.with(|c| c.set(scale));
}

/// A escala publicada neste quadro (a marca do menu lê-a).
#[must_use]
pub fn active() -> UiScale {
    ACTIVE.with(std::cell::Cell::get)
}

/// ⭐ **O mapa entre o pixel FÍSICO da janela e o LÓGICO da interface.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UiScaleMap {
    s: f32,
}

impl Default for UiScaleMap {
    /// A identidade: ecrã `1,0` × `100 %`.
    fn default() -> Self {
        Self::new(UiScale::P100)
    }
}

impl UiScaleMap {
    /// A preferência num ecrã de factor `1,0`.
    #[must_use]
    pub fn new(scale: UiScale) -> Self {
        Self::no_ecra(scale, 1.0)
    }

    /// ⭐ **HiDPI: `s` = factor do ECRÃ × a preferência.** `ecra` é o `scale_factor` do winit. O
    /// Blender multiplica os dois; o `Auto` da Godot é o factor do ecrã, que é o nosso `100 %`
    /// (`docs/UI_New_and_Simple/spec/oraculos/hidpi_2026-10-02.md`). Um factor que não é finito e
    /// positivo não vem do winit — vale `1,0`.
    #[must_use]
    pub fn no_ecra(scale: UiScale, ecra: f32) -> Self {
        debug_assert!(ecra.is_finite() && ecra > 0.0, "factor do ecrã {ecra}");
        let ecra = if ecra.is_finite() && ecra > 0.0 {
            ecra
        } else {
            1.0
        };
        Self {
            s: ecra * scale.factor(),
        }
    }

    /// O factor `s` (físico por lógico).
    #[must_use]
    pub fn factor(self) -> f32 {
        self.s
    }

    /// `true` com `s == 1,0` (ecrã `1,0` × `100 %`) — quem pode saltar trabalho (o `append` da
    /// cena) pergunta aqui.
    #[must_use]
    pub fn is_identity(self) -> bool {
        self.s.to_bits() == 1.0_f32.to_bits()
    }

    /// Um ponto da janela (físico) no espaço do chrome (lógico).
    #[must_use]
    pub fn to_logical(self, (x, y): (f32, f32)) -> (f32, f32) {
        (x / self.s, y / self.s)
    }

    /// Um rect da janela (físico) no espaço do chrome — o viewport que o chrome recebe.
    #[must_use]
    pub fn rect_to_logical(self, r: Rect) -> Rect {
        Rect::new(r.x / self.s, r.y / self.s, r.w / self.s, r.h / self.s)
    }

    /// Um rect do chrome (lógico) na janela (físico) — o que um pass de GPU ou a câmera recebem.
    #[must_use]
    pub fn rect_to_physical(self, r: Rect) -> Rect {
        Rect::new(r.x * self.s, r.y * self.s, r.w * self.s, r.h * self.s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fábrica é a identidade AO BIT, e ida-e-volta devolve o rect.
    #[test]
    fn the_factory_is_the_identity_and_the_round_trip_returns() {
        let r = Rect::new(13.7, 0.3, 1366.0, 1024.0);
        let id = UiScaleMap::new(UiScale::P100);
        assert!(id.is_identity());
        assert_eq!(id.rect_to_logical(r), r);
        assert_eq!(id.rect_to_physical(r), r);
        for z in UiScale::ALL {
            let m = UiScaleMap::new(z);
            let back = m.rect_to_physical(m.rect_to_logical(r));
            assert!(
                (back.w - r.w).abs() < 1e-3 && (back.x - r.x).abs() < 1e-3,
                "{z:?}"
            );
            let (x, y) = m.to_logical((r.x, r.y));
            assert!((x * m.factor() - r.x).abs() < 1e-4 && (y * m.factor() - r.y).abs() < 1e-4);
        }
    }
}
