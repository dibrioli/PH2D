//! **A pintura do plano das regras** (W4.6, §3.7): a cor dos lados, os alvos da tool, a escolha e o que a regra sob
//! o ponteiro (ou escolhida) liga, as cotas (traço, setas, a ficha do número) e os selos (a ficha com o glifo).

use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::readout::{CHIP_FONT_PX, CHIP_RADIUS_PX, paint_chip};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color as VelloColor, Point, Stroke, VectorScene};

use crate::cad_tool::{Alvo, CadTool};
use crate::cad_tool_plano::{Acerto, Plano};

/// O traço da cor da forma e das cotas, em px de ecrã.
const TRACO_PX: f64 = 1.5; // LITERAL-PX-OK: figura do overlay (a dos ícones)
/// O traço do que está aceso (escolha, hover, a regra escolhida).
const ACESO_PX: f64 = 3.0; // LITERAL-PX-OK: figura do overlay
/// O traço do glifo dentro do selo.
const GLIFO_PX: f32 = 1.5; // LITERAL-PX-OK: o traço dos ícones da barra

fn pincel(t: ColorToken, theme: Theme) -> Brush {
    let c = t.resolve(theme);
    Brush::Solid(VelloColor::from_rgba8(c.r, c.g, c.b, c.a))
}

fn linha(target: &mut VectorScene, pts: &[[f64; 2]], w: f64, cor: ColorToken, theme: Theme) {
    let mut l = BezPath::new();
    for (i, q) in pts.iter().enumerate() {
        if i == 0 {
            l.move_to(Point::new(q[0], q[1]));
        } else {
            l.line_to(Point::new(q[0], q[1]));
        }
    }
    target.inner_mut().stroke(
        &Stroke::new(w),
        Affine::IDENTITY,
        &pincel(cor, theme),
        None,
        &l,
    );
}

fn disco(target: &mut VectorScene, c: [f64; 2], r: f64, cor: ColorToken, theme: Theme) {
    let circ = ph2d_vector::Circle::new(Point::new(c[0], c[1]), r);
    target.inner_mut().fill(
        ph2d_vector::Fill::NonZero,
        Affine::IDENTITY,
        &pincel(cor, theme),
        None,
        &circ,
    );
}

/// A ponta de uma seta em `p`, a apontar para `d` (unitário).
fn seta(target: &mut VectorScene, p: [f64; 2], d: [f64; 2], cor: ColorToken, theme: Theme) {
    let l = f64::from(Spacing::Sm.px());
    let n = [-d[1], d[0]];
    let base = [p[0] - d[0] * l, p[1] - d[1] * l];
    let mut s = BezPath::new();
    s.move_to(Point::new(p[0], p[1]));
    s.line_to(Point::new(
        base[0] + n[0] * l * 0.5,
        base[1] + n[1] * l * 0.5,
    ));
    s.line_to(Point::new(
        base[0] - n[0] * l * 0.5,
        base[1] - n[1] * l * 0.5,
    ));
    s.close_path();
    target.inner_mut().fill(
        ph2d_vector::Fill::NonZero,
        Affine::IDENTITY,
        &pincel(cor, theme),
        None,
        &s,
    );
}

/// Acende os pontos de ecrã `pts` (um par = um lado; um só = um ponto).
fn acende(target: &mut VectorScene, pts: &[[f64; 2]], cor: ColorToken, theme: Theme) {
    for par in pts.chunks(2) {
        match par {
            [a, b] => linha(target, &[*a, *b], ACESO_PX, cor, theme),
            [a] => disco(target, *a, ACESO_PX * 2.0, cor, theme),
            _ => {}
        }
    }
}

/// ⭐ Pinta o plano. `tool` diz a escolha, o hover e a regra escolhida.
pub(crate) fn pinta(
    p: &Plano,
    tool: &CadTool,
    theme: Theme,
    text: &mut TextSystem,
    target: &mut VectorScene,
) {
    for &(a, b, cor) in &p.lados {
        linha(target, &[a, b], TRACO_PX, cor, theme);
    }
    for &(_, q) in &p.pontos {
        disco(target, q, TRACO_PX * 2.0, ColorToken::Text3, theme);
    }
    // O hover sobre um alvo, a escolha (pontos como disco; um lado inteiro).
    let alvo = |a: &Alvo| -> Vec<[f64; 2]> {
        match a {
            Alvo::Ponto(x) => p
                .pontos
                .iter()
                .filter(|(y, _)| y == x)
                .map(|(_, q)| *q)
                .collect(),
            Alvo::Lado(s) => p
                .segmentos
                .iter()
                .filter(|(t, ..)| t == s)
                .flat_map(|(_, a, b)| [*a, *b])
                .collect(),
            Alvo::Circulo(_) => Vec::new(),
        }
    };
    if let Some(Acerto::Alvo(a)) = tool.hover {
        acende(target, &alvo(&a), ColorToken::Selection, theme);
    }
    for a in &tool.escolha {
        acende(target, &alvo(a), ColorToken::Accent, theme);
        if let Alvo::Circulo(id) = a
            && let Some((_, c, r)) = p.circulos.iter().find(|(q, ..)| q == id)
        {
            let circ = ph2d_vector::Circle::new(Point::new(c[0], c[1]), *r);
            target.inner_mut().stroke(
                &Stroke::new(ACESO_PX),
                Affine::IDENTITY,
                &pincel(ColorToken::Accent, theme),
                None,
                &circ,
            );
        }
    }
    // O que a regra sob o ponteiro (ou escolhida) liga.
    let quente = match tool.hover {
        Some(Acerto::Numero(id) | Acerto::Selo(id)) => Some(id),
        _ => tool.regra,
    };
    if let Some(pts) = quente.and_then(|id| p.ligados.get(&id)) {
        acende(target, pts, ColorToken::Selection, theme);
    }
    for c in &p.cotas {
        for t in &c.tracos {
            linha(target, t, TRACO_PX, c.cor, theme);
        }
        for &(q, d) in &c.setas {
            seta(target, q, d, c.cor, theme);
        }
        paint_chip(
            text,
            target,
            &c.texto,
            [c.rect.x + c.rect.w * 0.5, c.rect.y + c.rect.h * 0.5],
            c.rect.w,
            theme,
        );
        if Some(c.regra) == quente {
            stroke_rounded_rect(
                target,
                c.rect,
                CHIP_RADIUS_PX,
                GLIFO_PX,
                resolve(ColorToken::Selection, theme),
            );
        }
    }
    for s in &p.selos {
        fill_rounded_rect(
            target,
            s.rect,
            CHIP_RADIUS_PX,
            resolve(ColorToken::Bg1, theme),
        );
        let borda = if Some(s.regra) == quente {
            ColorToken::Selection
        } else {
            s.cor
        };
        stroke_rounded_rect(
            target,
            s.rect,
            CHIP_RADIUS_PX,
            GLIFO_PX,
            resolve(borda, theme),
        );
        let glifo = ph2d_editor_core::zones::Rect::new(s.rect.x, s.rect.y, s.rect.h, s.rect.h);
        paint_icon(target, s.icon, glifo, resolve(s.cor, theme), GLIFO_PX);
        if let Some(n) = s.par {
            let resto = ph2d_editor_core::zones::Rect::new(
                s.rect.x + s.rect.h,
                s.rect.y,
                s.rect.w - s.rect.h,
                s.rect.h,
            );
            paint_text_centered(
                text,
                target,
                &n.to_string(),
                resto,
                CHIP_FONT_PX,
                resolve(s.cor, theme),
            );
        }
    }
}
