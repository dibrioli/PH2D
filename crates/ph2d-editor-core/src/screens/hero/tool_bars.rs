//! ⭐ **As barras do MÓDULO na área** — ① a coluna de ferramentas, o flyout de um botão dela e ④ as
//! ações junto da seleção: a CARA das [`ToolBars`] que o módulo publica (hoje o Vetor), pela
//! geometria do [`super`] (o `float_bar`).
//!
//! ⚠️ Os ids dos botões são do PAINEL dono — o clique segue pela porta dele. Os únicos ids daqui são
//! os dois do chrome da ④: **fixar** e **desligar** (D10 decisão 4: ligada por omissão no desktop e
//! no iPad, com fixar e desligar na MESMA entrega — a armadilha da *Contextual Task Bar*, estudo §4).

use super::{
    column_rects, grid_rects, inner, paint_button, paint_panel, paint_swatch, pinned_strip_rects,
    strip_rects_at,
};
use crate::icons::IconId;
use crate::ids;
use crate::interaction::{BarButton, BarFace, BarFamily, ToolBars};
use crate::paint::paint_icon;
use crate::screens::hero::HeroScreen;
use crate::zones::Rect;
use ph2d_i18n::tr;
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
///
/// ⭐ `left_handed` (F5, *Look ▸ Left-handed*): a coluna e o flyout passam para a borda DIREITA da
/// área — o lado da mão livre de quem desenha com a esquerda (Procreate, Nomad). A ④ segue a seleção.
#[must_use]
pub fn placed(
    bars: &ToolBars,
    area: Rect,
    actions_off: bool,
    pinned: bool,
    left_handed: bool,
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
        out.extend(
            items
                .iter()
                .cloned()
                .zip(grid_rects(area, a, items.len(), 3)),
        );
    }
    if left_handed {
        for (_, r) in &mut out {
            *r = super::mirrored(area, *r);
        }
    }
    let has_actions = bars.actions.iter().any(|g| !g.is_empty());
    if has_actions && !actions_off && (pinned || bars.selection.is_some()) {
        let mut groups = bars.actions.clone();
        groups.push(chrome_group(pinned));
        let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();
        // Os ícones da ④ lêem-se ao lado da COLUNA, não ao das caras do Quadro (foto da cena 9).
        let s = super::button_px();
        let rects = match bars.selection {
            Some(sel) if !pinned => strip_rects_at(area, sel, &sizes, s, super::selection_gap()),
            _ => pinned_strip_rects(area, &sizes, s),
        };
        out.extend(groups.into_iter().flatten().zip(rects));
    }
    out
}

/// Os blocos de [`placed`] (a coluna, o flyout, a ④), como intervalos na lista: cada um é um
/// painel que o pintor lê daqui. (O cartão lateral dos campos sem alça saiu na F12: toda forma
/// tem gesto no canvas.)
#[must_use]
pub fn blocks(bars: &ToolBars, n_placed: usize) -> [std::ops::Range<usize>; 3] {
    let n_col: usize = bars.column.iter().map(Vec::len).sum();
    let n_fly = bars.flyout.as_ref().map_or(0, |(_, f)| f.len());
    [0..n_col, n_col..n_col + n_fly, n_col + n_fly..n_placed]
}

/// Pinta as barras do módulo dentro de `area` (a área de desenho sem as réguas).
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene) {
    let Some(bars) = hero.store.tool_bars().cloned() else {
        return;
    };
    let theme = hero.theme;
    let off = hero.store.selection_actions_off();
    let pinned = hero.store.selection_actions_pinned();
    let all = placed(&bars, area, off, pinned, hero.view.ui_mirrored);
    // Os fundos primeiro, por bloco (a coluna, o flyout, a ④): cada um é um painel.
    for b in blocks(&bars, all.len()) {
        if b.start < b.end && b.end <= all.len() {
            paint_panel(scene, all[b].iter().map(|(_, r)| *r), theme);
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
        super::Btn {
            id: b.id,
            rect: r,
            on: b.on,
            tint: b.family.tint(),
        },
    );
    let steady = hero.motion.reduced_motion();
    super::flash::ring_if_home(scene, &hero.store, steady, theme, b.id, r);
    match &b.face {
        BarFace::Icon(icon) => paint_icon(
            scene,
            *icon,
            super::module_face_rect(r),
            fg,
            StrokeToken::Default.px(),
        ),
        BarFace::Swatch { rgba, ring } => paint_swatch(scene, theme, inner(r), *rgba, *ring, fg),
        BarFace::Outline(path) => {
            // O contorno da forma cresce como os ícones (1,5× o miolo, preso ao botão).
            let m = inner(r);
            let side = (m.w.min(m.h) * super::MODULE_FACE_GROWTH).min(r.w.min(r.h));
            let i = Rect::new(
                r.x + (r.w - side) / 2.0,
                r.y + (r.h - side) / 2.0,
                side,
                side,
            );
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
