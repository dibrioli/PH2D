//! **A amostra VÊ-SE a mover** (smoke do dono 01/11) — a lei do cartão do Kanban
//! ([`ph2d_editor_core::paint_flight`]), a mesma do bloco da linha do tempo do Pixel: a grelha pinta
//! a ordem prevista com o vão onde o bloco cai ([`palette::drag_permutation`]; com Ctrl, o vão da
//! cópia inserida, [`palette::copy_insert`]) e o bloco voa com o rato, inclinado, com a sombra de
//! voo, por cima do painel. Os alvos do clique ficam nas células de AGORA: a queda resolve-se como
//! antes (`seam::larga`).

use super::*;
use ph2d_editor_core::paint::{fill_rounded_rect, frame_radius};
use ph2d_editor_core::paint_flight::{FLIGHT_TILT_DEG, flight_shadow, fly};
use ph2d_pixel_core::palette;
use ph2d_tokens::Radius;

/// O bloco que o arrasto da amostra `from` leva: a selecção, se ela a contém; senão, só ela (a
/// regra da família ao largar).
fn bloco(info: &PixelPanelInfo, from: usize) -> Vec<usize> {
    if info.selected.contains(&from) {
        let mut v = info.selected.clone();
        v.sort_unstable();
        v.dedup();
        v
    } else {
        vec![from]
    }
}

/// Célula `j` da grelha → a amostra que lá se pinta (`None` = o vão). Sem alvo (o rato fora das
/// células), a queda cancela: a ordem é a de agora, e um bloco mudado deixa o vão no sítio dele.
fn ordem(
    info: &PixelPanelInfo,
    sel: &[usize],
    alvo: Option<usize>,
    copy: bool,
) -> Vec<Option<usize>> {
    let n = info.palette.len();
    let agora = |vao: bool| -> Vec<Option<usize>> {
        (0..n)
            .map(|i| (!(vao && sel.contains(&i))).then_some(i))
            .collect()
    };
    match (alvo, copy) {
        (None, c) => agora(!c),
        (Some(to), true) => {
            let Some((cores, onde)) = palette::copy_insert(&info.palette, sel, to) else {
                return agora(false);
            };
            let mut v = vec![None; cores.len()];
            for (antigo, &novo) in onde.iter().enumerate() {
                v[novo] = Some(antigo);
            }
            v
        }
        (Some(to), false) => palette::drag_permutation(n, sel, to)
            .into_iter()
            .map(|i| (!sel.contains(&i)).then_some(i))
            .collect(),
    }
}

impl Corpo<'_> {
    /// O arrasto activo da paleta, como a grelha o pinta: a ordem prevista e o voo. `rect(i)` é a
    /// célula `i` de agora (onde a queda se resolve).
    pub(super) fn paleta_prevista(
        &mut self,
        info: &PixelPanelInfo,
        rect: impl Fn(usize) -> Rect,
        cols: usize,
    ) -> Option<Vec<Option<usize>>> {
        let (_, d) = self
            .store
            .panel_row_drag()
            .filter(|(f, d)| *f == PanelRowFamily::PixelPalette && d.active)?;
        let n = info.palette.len();
        let from = (0..n).find(|&i| ids::palette_id(i) == d.dragged)?;
        let dentro = |r: Rect| {
            d.cursor_x >= r.x
                && d.cursor_x < r.x + r.w
                && d.cursor_y >= r.y
                && d.cursor_y < r.y + r.h
        };
        let alvo = (0..n).find(|&i| i != from && dentro(rect(i)));
        let sel = bloco(info, from);
        let copy = self.store.cmd_held();
        let slots = ordem(info, &sel, alvo, copy);

        // O bloco voa em linhas de até `cols` amostras, com a agarrada sob o ponto onde o rato a
        // apanhou.
        let cell = Spacing::Xl.px();
        let gap = Spacing::Xxs.px();
        let pad = Spacing::Xxs.px();
        let por_linha = cols.min(sel.len()).max(1);
        let linhas = sel.len().div_ceil(por_linha);
        let j = sel.iter().position(|&i| i == from).unwrap_or(0);
        #[expect(clippy::cast_precision_loss, reason = "uma paleta, ≤ 256")]
        let (w, h, ox, oy) = (
            por_linha as f32 * (cell + gap) - gap + pad * 2.0,
            linhas as f32 * (cell + gap) - gap + pad * 2.0,
            (j % por_linha) as f32 * (cell + gap) + pad,
            (j / por_linha) as f32 * (cell + gap) + pad,
        );
        let de = rect(from);
        let caixa = Rect::new(
            de.x + d.cursor_x - d.down_x - ox,
            de.y + d.cursor_y - d.down_y - oy,
            w,
            h,
        );
        let cores = sel
            .iter()
            .filter_map(|&i| info.palette.get(i).copied())
            .collect();
        self.paleta_vista.voo = Some(crate::state::VooPaleta {
            caixa,
            cores,
            por_linha,
        });
        Some(slots)
    }

    /// O vão: a célula onde o bloco vai cair, vazia.
    pub(super) fn vao(&mut self, r: Rect) {
        let radius = frame_radius(self.theme, Radius::Sm.px());
        fill_rounded_rect(self.scene, r, radius, resolve(ColorToken::Bg2, self.theme));
    }
}

/// Pinta o bloco em voo por cima do painel (fora do recorte da rolagem).
pub(super) fn voa(scene: &mut VectorScene, theme: Theme, v: &crate::state::VooPaleta) {
    let cell = Spacing::Xl.px();
    let gap = Spacing::Xxs.px();
    let pad = Spacing::Xxs.px();
    let mut local = VectorScene::new();
    let radius = frame_radius(theme, Radius::Sm.px());
    let caixa = Rect::new(0.0, 0.0, v.caixa.w, v.caixa.h);
    flight_shadow(&mut local, theme, caixa, radius);
    fill_rounded_rect(
        &mut local,
        caixa,
        radius,
        resolve(ColorToken::BgElev, theme),
    );
    for (k, c) in v.cores.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma paleta, ≤ 256")]
        let r = Rect::new(
            pad + (k % v.por_linha) as f32 * (cell + gap),
            pad + (k / v.por_linha) as f32 * (cell + gap),
            cell,
            cell,
        );
        paint_color_swatch(
            &ColorSwatch::new(ids::palette_id(k), "", *c),
            r,
            &mut local,
            theme,
        );
    }
    let accent = resolve(ColorToken::Accent, theme);
    stroke_rounded_rect(&mut local, caixa, radius, StrokeToken::Thick.px(), accent); // FRAME-RAW-OK: o bloco agarrado
    fly(scene, &local, v.caixa, FLIGHT_TILT_DEG);
}
