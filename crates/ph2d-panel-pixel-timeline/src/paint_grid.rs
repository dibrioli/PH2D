//! ⭐ **A grelha do Aseprite**: a coluna das camadas à esquerda (a linha partilhada com o painel
//! PIXEL), e à direita as tags por cima dos números dos frames (com a duração de cada um), e um
//! cel por camada × frame — cheio, oco (imagem vazia) ou nada; os ligados (o mesmo dono na mesma
//! camada) unidos numa pílula; o «z» quando o z ≠ 0; mais apagado com opacidade < 255.
//!
//! As colunas que não cabem não transbordam: a grelha é recortada e segue o frame activo.

use crate::{PixelTimelinePanelState, event, ids};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::pixel_layer_row::LayerRowPainter;
use ph2d_editor_core::pixel_panel::{PixelCel, PixelPanelInfo, PixelTagInfo};
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use std::collections::BTreeMap;

/// A geometria da grelha (pintar e seguir o activo leem a MESMA).
struct Grelha {
    /// O início da coluna dos frames e a largura dela.
    gx: f32,
    col_w: f32,
    first: usize,
    end: usize,
    tags_y: f32,
    head_y: f32,
    rows_y: f32,
    bottom: f32,
}

impl Grelha {
    #[expect(clippy::cast_precision_loss, reason = "uma posição de coluna à vista")]
    fn x(&self, c: usize) -> f32 {
        self.gx + (c - self.first) as f32 * self.col_w
    }
}

pub(crate) fn paint(
    st: &mut PixelTimelinePanelState,
    ctx: &mut PaintCtx,
    info: &PixelPanelInfo,
    area: Rect,
) {
    if area.w <= 0.0 || area.h <= 0.0 {
        return;
    }
    let layers_w = ph2d_tokens::chrome::PANEL_MIN_W_PX.min(area.w * 0.5);
    let gx = area.x + layers_w + Spacing::Sm.px();
    let gw = (area.x + area.w - gx).max(0.0);
    let duracoes: Vec<String> = info
        .frames
        .iter()
        .map(|f| {
            tr_with(
                "panel.pixel.timeline.duration_ms",
                &[("ms", &f.duration_ms)],
            )
        })
        .collect();
    let col_w = duracoes
        .iter()
        .map(|d| ctx.text_system.prefix_width(d, TypeToken::Xs.px()) + Spacing::Sm.px())
        .fold(ROW_H_PX, f32::max);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "quantas colunas cabem, ≥ 0"
    )]
    let cabem = ((gw / col_w).floor() as usize).max(1);
    let n = info.frames.len();
    if let Some(a) = event::activo(info) {
        if a < st.first_col {
            st.first_col = a;
        } else if a >= st.first_col + cabem {
            st.first_col = a + 1 - cabem;
        }
    }
    st.first_col = st.first_col.min(n.saturating_sub(cabem));
    let tags_y = area.y;
    let head_y = tags_y + ROW_H_PX;
    let rows_y = head_y + ROW_H_PX + TypeToken::Xs.px() + Spacing::Xs.px();
    let g = Grelha {
        gx,
        col_w,
        first: st.first_col,
        end: (st.first_col + cabem).min(n),
        tags_y,
        head_y,
        rows_y,
        bottom: area.y + area.h,
    };

    let colunas = Rect::new(gx, area.y, gw, area.h);
    ctx.scene.push_clip(&rect_to_vello(colunas));
    ctx.host.hit_index_mut().push_clip(colunas);
    cabecalhos(ctx, info, &g, &duracoes);
    let rename_tag = st.rename_tag.map(|(t, _)| t);
    for (i, t) in info.tags.iter().enumerate() {
        tag(ctx, &g, i, t, rename_tag == Some(i));
    }
    celulas(ctx, info, &g);
    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();

    // A coluna das camadas, alinhada às filas da grelha.
    let theme = ctx.host.theme();
    let renaming = st.rename_layer.map(|(id, _)| id);
    let col = Rect::new(area.x, g.rows_y, layers_w, (g.bottom - g.rows_y).max(0.0));
    ctx.scene.push_clip(&rect_to_vello(col));
    ctx.host.hit_index_mut().push_clip(col);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let mut p = LayerRowPainter {
        scene: &mut *ctx.scene,
        text_system: &mut *ctx.text_system,
        store,
        hit_index,
        theme,
    };
    let mut y = g.rows_y;
    for row in &info.layers {
        let rename = (renaming == Some(row.id)).then_some(ids::RENAME_LAYER);
        let active = info.active_layer == Some(row.id);
        y = p.paint(row, (area.x, y, layers_w), active, rename, &|w| {
            ids::layer_widget_id(row.id, w)
        });
    }
    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();
}

/// Os números dos frames (1-based) e a duração de cada um por baixo; o activo aceso.
fn cabecalhos(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha, duracoes: &[String]) {
    let theme = ctx.host.theme();
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    let alto = g.rows_y - g.head_y;
    for c in g.first..g.end {
        let f = info.frames[c];
        let r = Rect::new(g.x(c), g.head_y, g.col_w, alto);
        if info.active_frame == Some(f.id) {
            fill_rounded_rect(ctx.scene, r, radius, resolve(ColorToken::AccentSoft, theme));
            // A coluna do activo, por baixo dos cels.
            let coluna = Rect::new(r.x, g.rows_y, r.w, (g.bottom - g.rows_y).max(0.0));
            fill_rounded_rect(ctx.scene, coluna, radius, resolve(ColorToken::Bg2, theme));
        }
        let numero = (c + 1).to_string();
        centrado(
            ctx,
            &numero,
            r.x,
            g.head_y,
            g.col_w,
            ROW_H_PX,
            TypeToken::Sm,
            ColorToken::Text1,
        );
        let y = g.head_y + ROW_H_PX;
        let fonte = TypeToken::Xs;
        centrado(
            ctx,
            &duracoes[c],
            r.x,
            y,
            g.col_w,
            fonte.px(),
            fonte,
            ColorToken::Text3,
        );
        ctx.host.hit_index_mut().register(ids::header_id(f.id), r);
    }
}

#[expect(clippy::too_many_arguments, reason = "um texto centrado numa caixa")]
fn centrado(
    ctx: &mut PaintCtx,
    texto: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fonte: TypeToken,
    cor: ColorToken,
) {
    let tw = ctx.text_system.prefix_width(texto, fonte.px());
    let theme = ctx.host.theme();
    paint_text(
        ctx.text_system,
        ctx.scene,
        texto,
        x + ((w - tw) * 0.5).max(0.0),
        y + (h - fonte.px()) * 0.5,
        fonte.px(),
        w.max(1.0),
        resolve(cor, theme),
    );
}

/// A barra de uma tag sobre as colunas `from..=to` à vista: o nome, o glifo da direcção e as
/// repetições (> 0), e a faixa da cor dela por baixo.
fn tag(ctx: &mut PaintCtx, g: &Grelha, i: usize, t: &PixelTagInfo, renaming: bool) {
    let (Ok(from), Ok(to)) = (usize::try_from(t.from), usize::try_from(t.to)) else {
        return;
    };
    let (a, b) = (from.max(g.first), (to + 1).min(g.end));
    if a >= b {
        return;
    }
    let theme = ctx.host.theme();
    let x0 = g.x(a);
    let r = Rect::new(x0, g.tags_y, g.x(b) - x0, ROW_H_PX);
    let faixa_h = Spacing::Xs.px();
    let faixa = Rect::new(r.x, r.y + r.h - faixa_h, r.w, faixa_h);
    let [cr, cg, cb, ca] = t.color;
    let cor = ph2d_vector::Color::from_rgba8(cr, cg, cb, ca); // LITERAL-COLOR-OK: a cor da tag é DADO do sprite (o .ase grava-a)
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(ctx.scene, faixa, radius, cor);
    let texto_r = Rect::new(r.x, r.y, r.w, r.h - faixa_h);
    if renaming {
        let store = ctx.host.store();
        let (st, text, caret, anchor) = match store.get(ids::RENAME_TAG) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            _ => (TextInputState::Focused, String::new(), 0, None),
        };
        let input =
            TextInput::new(ids::RENAME_TAG, "").visual((st, store.hover_live(ids::RENAME_TAG)));
        paint_text_input_with_buffer(
            &input,
            Some(text.as_str()),
            Some(caret),
            anchor,
            texto_r,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        ctx.host.hit_index_mut().register(ids::RENAME_TAG, texto_r);
        return;
    }
    let glifo = tr(ids::direction_keys(t.direction).1);
    let texto = if t.repeat > 0 {
        tr_with(
            "panel.pixel.timeline.tag_label_repeat",
            &[("name", &t.name), ("dir", &glifo), ("n", &t.repeat)],
        )
    } else {
        tr_with(
            "panel.pixel.timeline.tag_label",
            &[("name", &t.name), ("dir", &glifo)],
        )
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        &texto,
        texto_r.x + Spacing::Xxs.px(),
        texto_r.y + (texto_r.h - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        texto_r.w.max(1.0),
        resolve(ColorToken::Text1, theme),
    );
    ctx.host.hit_index_mut().register(ids::tag_id(i), r);
}

/// Os cels de cada camada (os grupos não têm): o cel activo aceso, os ligados numa pílula.
fn celulas(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha) {
    let theme = ctx.host.theme();
    let cels: BTreeMap<(u32, u32), &PixelCel> =
        info.cels.iter().map(|c| ((c.layer, c.frame), c)).collect();
    let h = ROW_H_PX;
    let d = h * 0.5;
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    let mut y = g.rows_y;
    for row in &info.layers {
        if y >= g.bottom {
            break;
        }
        if row.group {
            y += ph2d_tokens::row_pitch_px();
            continue;
        }
        let cel = |c: usize| cels.get(&(row.id, info.frames[c].id)).copied();
        // As pílulas dos ligados, por baixo dos pontos.
        let mut c = g.first;
        while c < g.end {
            let mut fim = c;
            if let Some(a) = cel(c) {
                while fim + 1 < g.end && cel(fim + 1).is_some_and(|b| b.owner == a.owner) {
                    fim += 1;
                }
            }
            if fim > c {
                let x0 = g.x(c) + (g.col_w - d) * 0.5;
                let x1 = g.x(fim) + (g.col_w + d) * 0.5;
                let r = Rect::new(x0, y + (h - d) * 0.5, x1 - x0, d);
                fill_rounded_rect(ctx.scene, r, d * 0.5, resolve(ColorToken::Text3, theme));
            }
            c = fim + 1;
        }
        for c in g.first..g.end {
            let f = info.frames[c].id;
            let r = Rect::new(g.x(c), y, g.col_w, h);
            if info.active_layer == Some(row.id) && info.active_frame == Some(f) {
                fill_rounded_rect(ctx.scene, r, radius, resolve(ColorToken::AccentSoft, theme));
            }
            ponto(ctx, r, cel(c));
            ctx.host
                .hit_index_mut()
                .register(ids::cell_id(row.id, f), r);
        }
        y += ph2d_tokens::row_pitch_px();
    }
}

/// O ponto de um cel: cheio (mais apagado com opacidade < 255), oco se a imagem é vazia, um ponto
/// pequeno e oco sem cel; e o «z» no canto quando o z ≠ 0.
fn ponto(ctx: &mut PaintCtx, r: Rect, cel: Option<&PixelCel>) {
    let theme = ctx.host.theme();
    let d = r.h * 0.5;
    let dot = |d: f32| Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d);
    let traco = StrokeToken::Default.px();
    let Some(c) = cel else {
        let d = d * 0.5;
        stroke_rounded_rect(
            ctx.scene,
            dot(d),
            d * 0.5,
            traco,
            resolve(ColorToken::Border, theme),
        ); // FRAME-RAW-OK: o ponto oco É a marca do frame sem cel (Aseprite)
        return;
    };
    if c.empty {
        stroke_rounded_rect(
            ctx.scene,
            dot(d),
            d * 0.5,
            traco,
            resolve(ColorToken::Text2, theme),
        ); // FRAME-RAW-OK: o ponto oco É o cel de imagem vazia (Aseprite)
    } else {
        let cor = if c.opacity == u8::MAX {
            ColorToken::Text1
        } else {
            ColorToken::Text3
        };
        fill_rounded_rect(ctx.scene, dot(d), d * 0.5, resolve(cor, theme));
    }
    if c.z != 0 {
        let fonte = TypeToken::Xxs.px();
        paint_text(
            ctx.text_system,
            ctx.scene,
            tr("panel.pixel.timeline.z_mark"),
            r.x + r.w - fonte,
            r.y,
            fonte,
            fonte,
            resolve(ColorToken::Accent, theme),
        );
    }
}
