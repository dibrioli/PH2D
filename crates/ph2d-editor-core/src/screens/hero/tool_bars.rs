//! ⭐ **As barras do MÓDULO na área** — ① a coluna de ferramentas, o flyout de um botão dela e ④ as
//! ações junto da seleção: a CARA das [`ToolBars`] que o módulo publica (hoje o Vetor), pela
//! geometria do [`super`] (o `float_bar`).
//!
//! ⚠️ Os ids dos botões são do PAINEL dono — o clique segue pela porta dele. Os únicos ids daqui são
//! os dois do chrome da ④: **fixar** e **desligar** (D10 decisão 4: ligada por omissão no desktop e
//! no iPad, com fixar e desligar na MESMA entrega — a armadilha da *Contextual Task Bar*, estudo §4).

use super::{
    column_rects, grid_rects, inner, paint_button, paint_panel, paint_swatch, pinned_strip_rects,
    strip_rects,
};
use crate::icons::IconId;
use crate::ids;
use crate::interaction::{BarButton, BarFace, BarFamily, ToolBars};
use crate::paint::paint_icon;
use crate::screens::hero::HeroScreen;
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// O grupo do chrome que fecha a ④: fixar (aceso quando fixada) e desligar.
fn chrome_group(pinned: bool) -> Vec<BarButton> {
    vec![
        BarButton {
            id: ids::SELECTION_ACTIONS_PIN,
            face: BarFace::Icon(IconId::Pin),
            label: tr("chrome.selection_actions.pin").to_owned(),
            on: pinned,
            family: BarFamily::Neutral,
        },
        BarButton {
            id: ids::SELECTION_ACTIONS_OFF,
            face: BarFace::Icon(IconId::Close),
            label: tr("chrome.selection_actions.off").to_owned(),
            on: false,
            family: BarFamily::Neutral,
        },
    ]
}

/// ⭐ **Onde cai cada botão neste quadro** — a porta que o pintor e os gates perguntam.
///
/// ⚠️ Uma função só: o pintor e o gate *todo botão pintado é despachado* leem a MESMA lista; uma
/// segunda conta para o gate mediria a fórmula e não o produto.
#[must_use]
pub fn placed(
    bars: &ToolBars,
    area: Rect,
    actions_off: bool,
    pinned: bool,
) -> Vec<(BarButton, Rect)> {
    let mut out = Vec::new();
    let sizes: Vec<usize> = bars.column.iter().map(Vec::len).collect();
    let col = column_rects(area, &sizes);
    let mut anchor = None;
    for (b, r) in bars.column.iter().flatten().zip(col) {
        if bars.flyout.as_ref().is_some_and(|(id, _)| *id == b.id) {
            anchor = Some(r);
        }
        out.push((b.clone(), r));
    }
    if let (Some((_, items)), Some(a)) = (&bars.flyout, anchor) {
        out.extend(items.iter().cloned().zip(grid_rects(area, a, items.len(), 3)));
    }
    let has_actions = bars.actions.iter().any(|g| !g.is_empty());
    if has_actions && !actions_off && (pinned || bars.selection.is_some()) {
        let mut groups = bars.actions.clone();
        groups.push(chrome_group(pinned));
        let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();
        let rects = match bars.selection {
            Some(sel) if !pinned => strip_rects(area, sel, &sizes),
            _ => pinned_strip_rects(area, &sizes),
        };
        out.extend(groups.into_iter().flatten().zip(rects));
    }
    out
}

/// Pinta as barras do módulo dentro de `area` (a área de desenho sem as réguas).
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, _text: &mut TextSystem) {
    let Some(bars) = hero.store.tool_bars().cloned() else {
        return;
    };
    let theme = hero.theme;
    let off = hero.store.selection_actions_off();
    let pinned = hero.store.selection_actions_pinned();
    let all = placed(&bars, area, off, pinned);
    // Os fundos primeiro, por bloco (a coluna, o flyout, a ④): cada um é um painel.
    let n_col: usize = bars.column.iter().map(Vec::len).sum();
    let n_fly = bars.flyout.as_ref().map_or(0, |(_, f)| f.len());
    let blocks = [(0, n_col), (n_col, n_col + n_fly), (n_col + n_fly, all.len())];
    for (a, b) in blocks {
        if a < b && b <= all.len() {
            paint_panel(scene, all[a..b].iter().map(|(_, r)| *r), theme);
        }
    }
    for (b, r) in all {
        face(hero, scene, theme, &b, r);
        hero.store.set_tooltip(b.id, b.label);
    }
}

fn face(hero: &mut HeroScreen, scene: &mut VectorScene, theme: Theme, b: &BarButton, r: Rect) {
    let fg = paint_button(
        scene,
        &mut hero.hit_index,
        &hero.store,
        theme,
        b.id,
        r,
        b.on,
        b.family.tint(),
    );
    match &b.face {
        BarFace::Icon(icon) => paint_icon(scene, *icon, inner(r), fg, StrokeToken::Default.px()),
        BarFace::Swatch { rgba, ring } => paint_swatch(scene, theme, inner(r), *rgba, *ring, fg),
        BarFace::Outline(path) => {
            let i = inner(r);
            let at = ph2d_vector::Affine::translate((f64::from(i.x), f64::from(i.y)))
                * ph2d_vector::Affine::scale_non_uniform(f64::from(i.w), f64::from(i.h));
            scene.inner_mut().stroke(
                &ph2d_vector::Stroke::new(f64::from(StrokeToken::Default.px())),
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(fg),
                None,
                &(at * path.clone()),
            );
        }
    }
}
