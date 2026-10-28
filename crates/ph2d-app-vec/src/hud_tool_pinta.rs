//! O desenho das setas do arranjo do HUD (W5.5): o contorno de cada elemento, a seta de cada par com o selo dela
//! (cheio = «ao lado», vazado = «não invade»; um traço cruzado = centrados), a seta a ser puxada, e o aceso do
//! hover e da escolhida. Os pincéis são os da tool Regras.

use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

use crate::cad_tool_pinta::{disco, linha, seta};
use crate::hud_tool::{HudTool, Plano};
use ph2d_panel_vector::state_hud::TipoSeta;

/// O meio-lado do selo de uma seta, em px de ecrã (o rect que se acerta).
pub const SELO_PX: f64 = 9.0; // LITERAL-PX-OK: figura do overlay (o alcance da mão)
const TRACO_PX: f64 = 1.5; // LITERAL-PX-OK: figura do overlay (a dos ícones)
const ACESO_PX: f64 = 3.0; // LITERAL-PX-OK: figura do overlay

fn contorno(
    target: &mut VectorScene,
    r: &ph2d_editor_core::zones::Rect,
    w: f64,
    cor: ColorToken,
    theme: Theme,
) {
    let (x0, y0) = (f64::from(r.x), f64::from(r.y));
    let (x1, y1) = (x0 + f64::from(r.w), y0 + f64::from(r.h));
    linha(
        target,
        &[[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]],
        w,
        cor,
        theme,
    );
}

fn unit(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let d = [b[0] - a[0], b[1] - a[1]];
    let n = d[0].hypot(d[1]);
    if n > 0.0 {
        [d[0] / n, d[1] / n]
    } else {
        [1.0, 0.0]
    }
}

/// ⭐ Pinta o plano. `hover` = o cursor no ecrã.
pub(crate) fn pinta(
    p: &Plano,
    t: &HudTool,
    hover: Option<[f64; 2]>,
    theme: Theme,
    target: &mut VectorScene,
) {
    let sob = |r: &ph2d_editor_core::zones::Rect| {
        #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
        hover.is_some_and(|q| r.contains(q[0] as f32, q[1] as f32))
    };
    for (_, id, r) in &p.elementos {
        let origem = t.puxada.is_some_and(|x| x.de == *id);
        let (w, cor) = if origem || sob(r) {
            (ACESO_PX, ColorToken::Selection)
        } else {
            (TRACO_PX, ColorToken::Info)
        };
        contorno(target, r, w, cor, theme);
    }
    for s in &p.setas {
        let escolhida = t.escolhida == Some(s.par);
        let cor = if escolhida {
            ColorToken::Accent
        } else if sob(&s.selo) {
            ColorToken::Selection
        } else {
            ColorToken::Info
        };
        let w = if escolhida { ACESO_PX } else { TRACO_PX };
        linha(target, &[s.de, s.para], w, cor, theme);
        seta(target, s.para, unit(s.de, s.para), cor, theme);
        let c = [f64::from(s.selo.x) + SELO_PX, f64::from(s.selo.y) + SELO_PX];
        match s.tipo {
            Some(TipoSeta::AoLado) => disco(target, c, SELO_PX * 0.6, cor, theme),
            Some(TipoSeta::NaoInvade) | None => contorno(target, &s.selo, w, cor, theme),
        }
        if s.centrado {
            let d = unit(s.de, s.para);
            let n = [-d[1] * SELO_PX * 1.5, d[0] * SELO_PX * 1.5];
            linha(
                target,
                &[[c[0] - n[0], c[1] - n[1]], [c[0] + n[0], c[1] + n[1]]],
                w,
                cor,
                theme,
            );
        }
    }
    if let Some(x) = t.puxada
        && let Some((_, _, r)) = p
            .elementos
            .iter()
            .find(|(c, id, _)| *c == x.canvas && *id == x.de)
    {
        let de = [f64::from(r.x + r.w * 0.5), f64::from(r.y + r.h * 0.5)];
        let cor = if t.recusada {
            ColorToken::Danger
        } else {
            ColorToken::Accent
        };
        linha(target, &[de, x.ponta], ACESO_PX, cor, theme);
        seta(target, x.ponta, unit(de, x.ponta), cor, theme);
        disco(target, de, f64::from(Spacing::Xs.px()), cor, theme);
    }
}
