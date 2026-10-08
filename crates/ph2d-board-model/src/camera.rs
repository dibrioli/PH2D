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

    /// ⭐ **Enquadra** `bbox` (`[x0, y0, x1, y1]`, mundo) em `area`, com `margin` px de ecrã de cada
    /// lado: o «zoom to fit» (`Alt+1`) e o «zoom to selected» (`Alt+2`) do Miro, e ir a um frame.
    /// Uma caixa vazia ou não finita não faz nada.
    pub fn fit(&mut self, area: Area, bbox: [f64; 4], margin: f64) {
        let [x0, y0, x1, y1] = bbox;
        if !(x0.is_finite() && y0.is_finite() && x1.is_finite() && y1.is_finite()) {
            return;
        }
        let (bw, bh) = ((x1 - x0).max(f64::EPSILON), (y1 - y0).max(f64::EPSILON));
        let (w, h) = (
            (area[2] - 2.0 * margin).max(1.0),
            (area[3] - 2.0 * margin).max(1.0),
        );
        self.zoom = (w / bw).min(h / bh).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        self.center_x = (x0 + x1) / 2.0;
        self.center_y = (y0 + y1) / 2.0;
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
