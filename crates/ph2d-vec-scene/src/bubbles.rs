//! **Balões** (fala · pensamento · nuvem · explosão · chave) — módulo irmão de
//! [`crate::shapes`].
//!
//! ATENÇÃO: este arquivo está em REESCRITA para o padrão-ouro. O conteúdo abaixo é a
//! implementação antiga (autorada em convenção de TELA, portanto espelhada na vertical:
//! os rabinhos apontam para cima) e está aqui apenas para o catálogo seguir compilando.

use crate::{Contour, VecPath, VecVertex};

/// Contorno fechado de quinas a partir de pontos crus.
fn corners(pts: Vec<[f64; 2]>) -> VecPath {
    VecPath {
        verts: pts.into_iter().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// Contorno fechado a partir de vértices já com handles.
fn smooth_path(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// Vértice com handles explícitos.
fn v(anchor: [f64; 2], i: [f64; 2], o: [f64; 2]) -> VecVertex {
    VecVertex::smooth(anchor, i, o)
}

/// Centro + semi-eixos da caixa do gesto.
fn box_of(a: [f64; 2], b: [f64; 2]) -> (f64, f64, f64, f64) {
    (
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (b[0] - a[0]).abs() * 0.5,
        (b[1] - a[1]).abs() * 0.5,
    )
}

/// **Balão de fala retangular** com rabicho. O corpo ocupa a caixa menos a altura do
/// rabicho; `tail_x` posiciona o rabicho (fração da largura, 0.5 = meio) e `tail_w` o
/// engrossa. É um round-rect + o triângulo, num contorno só.
#[must_use]
pub fn speech_rect(
    a: [f64; 2],
    b: [f64; 2],
    radius: f64,
    tail_h: f64,
    tail_x: f64,
    tail_w: f64,
) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let th = (2.0 * hh * tail_h.clamp(0.05, 0.6)).min(hh);
    let body_bot = cy + hh - th;
    let r = radius.clamp(0.0, (hw).min((body_bot - (cy - hh)) * 0.5));
    let tw = (2.0 * hw * tail_w.clamp(0.02, 0.5)).min(hw);
    let tx = cx - hw + 2.0 * hw * tail_x.clamp(0.05, 0.95);
    // Corpo arredondado (reusa o round-rect) e depois o rabicho é injetado na base.
    let body = crate::rounded_rect([cx - hw, cy - hh], [cx + hw, body_bot], r);
    let mut verts = body.verts;
    // Encontra o segmento da base (os dois vértices de menor y invertido) e insere o
    // rabicho entre eles: base-direita → ponta → base-esquerda.
    let tip = [tx, cy + hh];
    let insert_at = verts
        .iter()
        .position(|p| (p.anchor[1] - body_bot).abs() < 1e-9)
        .unwrap_or(0);
    let tail = [
        VecVertex::corner([(tx + tw * 0.5).min(cx + hw - r), body_bot]),
        VecVertex::corner(tip),
        VecVertex::corner([(tx - tw * 0.5).max(cx - hw + r), body_bot]),
    ];
    for (i, t) in tail.into_iter().enumerate() {
        verts.insert(insert_at + i, t);
    }
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// **Balão de fala oval**: a elipse + o rabicho.
#[must_use]
pub fn speech_oval(a: [f64; 2], b: [f64; 2], tail_h: f64, tail_x: f64, tail_w: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let th = (2.0 * hh * tail_h.clamp(0.05, 0.6)).min(hh);
    let body_hh = hh - th * 0.5;
    let body_cy = cy - th * 0.5;
    let mut p = crate::ellipse([cx, body_cy], hw, body_hh);
    let tx = cx - hw + 2.0 * hw * tail_x.clamp(0.1, 0.9);
    let tw = (2.0 * hw * tail_w.clamp(0.02, 0.4)).min(hw * 0.5);
    // O rabicho entra como contorno próprio (o balão oval é compound: corpo + rabicho —
    // com NonZero os dois se fundem num só preenchimento, que é o desenho certo).
    p.subpaths.push(Contour {
        verts: vec![
            VecVertex::corner([tx - tw * 0.5, body_cy + body_hh * 0.7]),
            VecVertex::corner([tx, cy + hh]),
            VecVertex::corner([tx + tw * 0.5, body_cy + body_hh * 0.7]),
        ],
        closed: true,
    });
    p
}

/// **Balão de pensamento**: a nuvem + as bolhas que descem até a boca.
#[must_use]
pub fn thought(a: [f64; 2], b: [f64; 2], bumps: f64, bubbles: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    // O corpo ocupa a parte de cima; as bolhas, a de baixo.
    let body_hh = hh * 0.72;
    let body_cy = cy - hh * 0.28;
    let mut p = cloud(
        [cx - hw, body_cy - body_hh],
        [cx + hw, body_cy + body_hh],
        bumps,
    );
    let n = (bubbles.clamp(1.0, 4.0)).round() as usize;
    for i in 0..n {
        let f = (i + 1) as f64 / (n + 1) as f64;
        let r = hw * 0.10 * (1.0 - f * 0.5);
        let bx = cx - hw * 0.35 - hw * 0.25 * f;
        let by = body_cy + body_hh + (hh * 2.0 - body_hh * 2.0) * f * 0.8;
        let bub = crate::ellipse([bx, by], r, r * (hh / hw).max(0.2));
        p.subpaths.push(Contour {
            verts: bub.verts,
            closed: true,
        });
    }
    p
}

/// **Nuvem**: círculos fundidos numa silhueta ondulada. `bumps` = quantas ondas.
#[must_use]
pub fn cloud(a: [f64; 2], b: [f64; 2], bumps: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let n = bumps.clamp(4.0, 12.0).round() as usize;
    // Uma elipse "dentada": o raio alterna entre cheio e recuado, e os vértices são
    // SUAVES — é o que dá o contorno de bolhas sem montar N círculos e uni-los.
    let mut verts = Vec::with_capacity(n * 2);
    let step = std::f64::consts::TAU / (n * 2) as f64;
    for i in 0..(n * 2) {
        let ang = step * i as f64 - std::f64::consts::FRAC_PI_2;
        let bump = i % 2 == 0;
        let r = if bump { 1.0 } else { 0.78 };
        let (s, c) = ang.sin_cos();
        let anchor = [cx + hw * r * c, cy + hh * r * s];
        // Handle tangente, generoso nas cristas (arredonda a bolha) e curto nos vales.
        let h = if bump { 0.55 } else { 0.30 } * step;
        let (tx, ty) = (-hw * r * s * h * 2.0, hh * r * c * h * 2.0);
        verts.push(VecVertex::smooth(
            anchor,
            [anchor[0] - tx, anchor[1] - ty],
            [anchor[0] + tx, anchor[1] + ty],
        ));
    }
    smooth_path(verts)
}

/// **Explosão / grito** (starburst): estrela de pontas irregulares — a de quadrinhos.
/// `points` = quantas pontas, `inner` = quanto o vale recua, `jag` = o desalinho que a
/// torna irregular (0 = estrela regular).
#[must_use]
pub fn burst(a: [f64; 2], b: [f64; 2], points: f64, inner: f64, jag: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let n = points.clamp(5.0, 24.0).round() as usize;
    let ir = inner.clamp(0.2, 0.9);
    let j = jag.clamp(0.0, 0.5);
    let step = std::f64::consts::PI / n as f64;
    let mut pts = Vec::with_capacity(n * 2);
    for i in 0..(n * 2) {
        let outer = i % 2 == 0;
        // O "desalinho" é DETERMINÍSTICO (função do índice): a mesma forma sempre sai
        // igual — uma explosão que muda de silhueta a cada re-cook seria inutilizável.
        let wobble = if j > 0.0 {
            let k = (i * 2_654_435_761) % 1000;
            1.0 - j * (k as f64 / 1000.0)
        } else {
            1.0
        };
        let r = if outer { 1.0 } else { ir } * wobble;
        let ang = step * i as f64 - std::f64::consts::FRAC_PI_2;
        let (s, c) = ang.sin_cos();
        pts.push([cx + hw * r * c, cy + hh * r * s]);
    }
    corners(pts)
}


/// **Chave** `{` (curly brace) — ABERTA (é um traço), a anotação de diagrama.
#[must_use]
pub fn brace(a: [f64; 2], b: [f64; 2], waist: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let w = waist.clamp(0.0, 1.0);
    let x0 = cx - hw;
    let x1 = cx + hw;
    let mid = cx + hw - 2.0 * hw * w;
    VecPath {
        verts: vec![
            v([x1, cy - hh], [x1, cy - hh], [x0, cy - hh]),
            v([mid, cy - hh * 0.35], [mid, cy - hh], [mid, cy - hh * 0.1]),
            v([x0, cy], [mid, cy - hh * 0.1], [mid, cy + hh * 0.1]),
            v([mid, cy + hh * 0.35], [mid, cy + hh * 0.1], [mid, cy + hh]),
            v([x1, cy + hh], [x0, cy + hh], [x1, cy + hh]),
        ],
        closed: false,
        ..VecPath::default()
    }
}

