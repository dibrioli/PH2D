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

/// Célula `j` da grelha → a amostra que lá se pinta (`None` = o vão). Sem alvo (o rato longe da
/// grelha ou sobre a própria amostra), a queda cancela: a ordem é a de agora, e um bloco mudado
/// deixa o vão no sítio dele.
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

/// ⭐ **A queda é função SÓ do lugar do rato** (smoke do dono 01/11: «ainda não ficou estável»): a
/// célula `c` sob o rato na grelha FIXA (os intervalos contam para a célula mais próxima, a borda
/// prende) põe a agarrada nela — o vão começa em `c − j` (`j` = a posição da agarrada no bloco).
/// Nada depende da ordem prevista pintada, por isso o vão não salta nem pisca. Devolve o `to` do
/// mover ([`palette::drag_permutation`]) e o do copiar ([`palette::copy_insert`]).
pub(crate) fn queda(
    (rx, ry): (f32, f32),
    cols: usize,
    n: usize,
    sel: &[usize],
    j: usize,
) -> Option<Queda> {
    let pitch = Spacing::Xl.px() + Spacing::Xxs.px();
    let (k, first) = (sel.len(), *sel.first()?);
    let (rx, ry) = (rx / pitch, ry / pitch);
    #[expect(clippy::cast_precision_loss, reason = "uma paleta, ≤ 256")]
    let (largura, altura) = (cols as f32, (n + k).div_ceil(cols) as f32);
    // Meia célula de folga à volta da grelha; além dela, a queda cancela.
    if !(-0.5..largura + 0.5).contains(&rx) || !(-0.5..altura + 0.5).contains(&ry) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "já preso à grelha"
    )]
    let celula = |v: f32, max: usize| (v.max(0.0) as usize).min(max.saturating_sub(1));
    let c = celula(ry, altura as usize) * cols + celula(rx, cols);
    let inicio = |total: usize| c.min(total - 1).saturating_sub(j).min(total - k);
    let s = inicio(n);
    Some(Queda {
        mover: if s > first { s + k - 1 } else { s },
        copiar: inicio(n + k),
    })
}

/// Os dois `to` de uma queda: o do mover e o do copiar (o Ctrl lê-se ao largar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Queda {
    pub(crate) mover: usize,
    pub(crate) copiar: usize,
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
        let sel = bloco(info, from);
        let j = sel.iter().position(|&i| i == from).unwrap_or(0);
        // Sobre a própria amostra o despacho não larga nada: a pré-visualização também cancela.
        let de = rect(from);
        let sobre_si = de.contains(d.cursor_x, d.cursor_y);
        let o = rect(0);
        let q = queda(
            (
                d.cursor_x - o.x + Spacing::Xxs.px() * 0.5,
                d.cursor_y - o.y + Spacing::Xxs.px() * 0.5,
            ),
            cols,
            n,
            &sel,
            j,
        )
        .filter(|_| !sobre_si);
        self.paleta_vista.queda = q;
        let copy = self.store.cmd_held();
        let alvo = q.map(|q| if copy { q.copiar } else { q.mover });
        let slots = ordem(info, &sel, alvo, copy);

        // O bloco voa em linhas de até `cols` amostras, com a agarrada sob o ponto onde o rato a
        // apanhou.
        let cell = Spacing::Xl.px();
        let gap = Spacing::Xxs.px();
        let pad = Spacing::Xxs.px();
        let por_linha = cols.min(sel.len()).max(1);
        let linhas = sel.len().div_ceil(por_linha);
        #[expect(clippy::cast_precision_loss, reason = "uma paleta, ≤ 256")]
        let (w, h, ox, oy) = (
            por_linha as f32 * (cell + gap) - gap + pad * 2.0,
            linhas as f32 * (cell + gap) - gap + pad * 2.0,
            (j % por_linha) as f32 * (cell + gap) + pad,
            (j / por_linha) as f32 * (cell + gap) + pad,
        );
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

    /// A marca da queda: uma barra vertical do acento no intervalo à esquerda da célula `r` (a
    /// primeira do vão), um pouco mais alta que ela.
    pub(super) fn marca_da_queda(&mut self, r: Rect) -> Rect {
        let gap = Spacing::Xxs.px();
        let w = StrokeToken::Heavy.px();
        let m = Rect::new(r.x - (gap + w) * 0.5, r.y - gap, w, r.h + gap * 2.0);
        fill_rounded_rect(
            self.scene,
            m,
            w * 0.5,
            resolve(ColorToken::Accent, self.theme),
        );
        m
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
