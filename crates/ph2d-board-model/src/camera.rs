//! **A vista de um quadro** — a conta entre o ecrã e o mundo do quadro, o zoom à volta do cursor e
//! o arrastar. Pura: a área chega como `[x, y, w, h]` em px de ecrã.

use crate::Camera;

/// Faixa do zoom. ⚠️ Não é tecto de desempenho: é o limite em que um gesto deixa de ser
/// distinguível — abaixo de `1e-4` um quadro de 10⁸ unidades cabe num px, acima de `1e4` um px é
/// 10⁻⁴ unidade. Fora dela o `f64` continua exacto, quem se perde é a mão.
pub const ZOOM_RANGE: (f64, f64) = (1e-4, 1e4);

/// `[x, y, w, h]` da área do quadro no ecrã, em px.
pub type Area = [f64; 4];

impl Camera {
    /// Ponto do mundo → ponto do ecrã.
    #[must_use]
    pub fn to_screen(&self, area: Area, world: [f64; 2]) -> [f64; 2] {
        let [x, y, w, h] = area;
        [
            x + w / 2.0 + (world[0] - self.center_x) * self.zoom,
            y + h / 2.0 + (world[1] - self.center_y) * self.zoom,
        ]
    }

    /// Ponto do ecrã → ponto do mundo.
    #[must_use]
    pub fn to_world(&self, area: Area, screen: [f64; 2]) -> [f64; 2] {
        let [x, y, w, h] = area;
        [
            self.center_x + (screen[0] - x - w / 2.0) / self.zoom,
            self.center_y + (screen[1] - y - h / 2.0) / self.zoom,
        ]
    }

    /// Multiplica o zoom por `factor` mantendo FIXO o ponto do mundo sob `screen` (o cursor).
    /// Um `factor` não finito ou ≤ 0 não faz nada.
    pub fn zoom_about(&mut self, area: Area, screen: [f64; 2], factor: f64) {
        if !(factor.is_finite() && factor > 0.0) {
            return;
        }
        let anchor = self.to_world(area, screen);
        self.zoom = (self.zoom * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        let moved = self.to_world(area, screen);
        self.center_x += anchor[0] - moved[0];
        self.center_y += anchor[1] - moved[1];
    }

    /// Arrasta a vista: o mundo segue o cursor por `(dx, dy)` px de ecrã.
    pub fn pan_by_screen(&mut self, dx: f64, dy: f64) {
        self.center_x -= dx / self.zoom;
        self.center_y -= dy / self.zoom;
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
