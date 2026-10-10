//! ⭐ **A apresentação** (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 P1–P6): os frames de topo como
//! diapositivos, na ordem do painel, um de cada vez. SESSÃO — o ficheiro só guarda «não apresentar»
//! (`Team::hidden_slides`). Puro: o relógio chega em segundos.

use ph2d_board_model::{Area, Board, Camera, ElementId};

/// P3 — a margem à volta do diapositivo, em fracção do lado limitante da área: medida no GIF oficial
/// do Miro (`21019593022866`: 7 px de 853 em cima e em baixo).
pub const SLIDE_MARGIN: f64 = 0.008;
/// P5 — a viagem da câmara `1 − (1 − t/T)^k`, ajustada aos dois pontos medidos no mesmo GIF (83 % aos
/// 0,50 s, 99 % aos 0,94 s).
pub const TRAVEL_SECS: f64 = 1.33;
/// P5 — o expoente da desaceleração.
pub const TRAVEL_POWER: f64 = 3.76;

/// P1 — os diapositivos: os frames de TOPO na ordem do painel de Frames, sem os escondidos.
#[must_use]
pub fn slides(board: &Board) -> Vec<ElementId> {
    board
        .doc
        .frames_in_panel_order()
        .into_iter()
        .filter(|f| f.frame.is_none() && !board.team.hidden_slides.contains(&f.id))
        .map(|f| f.id)
        .collect()
}

/// P2 — por onde começa: o diapositivo seleccionado, ou o que contém o 1.º seleccionado, ou o 1.º.
#[must_use]
pub fn first_slide(board: &Board, slides: &[ElementId], selection: &[ElementId]) -> usize {
    selection
        .iter()
        .find_map(|&s| {
            let chain = board.doc.frame_chain(s);
            slides.iter().position(|f| *f == s || chain.contains(f))
        })
        .unwrap_or(0)
}

/// P3 — a câmara que enquadra o diapositivo `frame` em `area`.
#[must_use]
pub fn slide_camera(board: &Board, frame: ElementId, area: Area) -> Option<Camera> {
    let f = board.doc.get(frame)?;
    let mut c = board.camera;
    let margin = area[2].min(area[3]) * SLIDE_MARGIN;
    c.fit(area, [f.x, f.y, f.x + f.w, f.y + f.h], margin);
    Some(c)
}

/// P5 — quanto da viagem já se fez ao fim de `t` segundos (`0..=1`).
#[must_use]
pub fn travel(t: f64) -> f64 {
    if !(t > 0.0) {
        return 0.0;
    }
    1.0 - (1.0 - (t / TRAVEL_SECS).min(1.0)).powf(TRAVEL_POWER)
}

/// P5 — a câmara a `t` segundos de `from` para `to`: o centro em linha recta, o zoom em escala
/// geométrica (um salto de 10× lê-se como uma subida contínua, não como um mergulho no fim).
#[must_use]
pub fn camera_at(from: Camera, to: Camera, t: f64) -> Camera {
    let k = travel(t);
    Camera {
        center_x: from.center_x + (to.center_x - from.center_x) * k,
        center_y: from.center_y + (to.center_y - from.center_y) * k,
        zoom: from.zoom * (to.zoom / from.zoom).powf(k),
    }
}

#[cfg(test)]
#[path = "present_tests.rs"]
mod tests;
