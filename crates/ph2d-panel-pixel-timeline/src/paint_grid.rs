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
use ph2d_editor_core::pixel_panel::{PixelCel, PixelPanelInfo, PixelRangeKind as K, PixelTagInfo};
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use std::collections::BTreeMap;

/// A geometria da grelha (pintar e seguir o activo leem a MESMA).
pub(crate) struct Grelha {
    /// O início da coluna dos frames e a largura dela.
    pub(crate) gx: f32,
    pub(crate) col_w: f32,
    pub(crate) first: usize,
    pub(crate) end: usize,
    tags_y: f32,
    pub(crate) head_y: f32,
    /// O topo do corpo que rola, e onde a primeira fila cai depois da rolagem.
    pub(crate) top: f32,
    pub(crate) rows_y: f32,
    pub(crate) bottom: f32,
}

impl Grelha {
    #[expect(clippy::cast_precision_loss, reason = "uma posição de coluna à vista")]
    pub(crate) fn x(&self, c: usize) -> f32 {
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
    // ⭐ D2: com o bloco agarrado a grelha pinta a ordem PREVISTA (o vão onde ele cai).
    let voo = crate::gestos::bloco_previsto(st, info);
    let original = info;
    let info = voo.as_ref().map_or(info, |v| &v.info);
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
    // A coluna das camadas guarda a largura de um painel enquanto sobrar UMA coluna de quadro (a
    // grelha rola até ao activo); a 1280 a metade da banda cortava «Background» (portão
    // `nenhum_corte_novo_entra_sem_ser_nomeado`).
    let folga = (area.w - Spacing::Sm.px() - col_w).max(area.w * 0.5);
    let layers_w = ph2d_tokens::chrome::PANEL_MIN_W_PX.min(folga);
    let gx = area.x + layers_w + Spacing::Sm.px();
    let gw = (area.x + area.w - gx).max(0.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "quantas colunas cabem, ≥ 0"
    )]
    let cabem = ((gw / col_w).floor() as usize).max(1);
    let n = info.frames.len();
    if let Some(a) = event::activo(original) {
        if a < st.first_col {
            st.first_col = a;
        } else if a >= st.first_col + cabem {
            st.first_col = a + 1 - cabem;
        }
    }
    st.first_col = st.first_col.min(n.saturating_sub(cabem));
    // ⭐ Cada nível de tags a sua fila (lei do Trial): a de dentro por baixo, as sobrepostas
    // desfasadas; o cabeçalho e as camadas descem com elas.
    let spans: Vec<(u32, u32)> = info.tags.iter().map(|t| (t.from, t.to)).collect();
    let filas_tag = ph2d_pixel_core::play::tag_rows(&spans);
    let n_filas = filas_tag.iter().max().map_or(1, |m| m + 1);
    let tags_y = area.y;
    #[expect(clippy::cast_precision_loss, reason = "poucas filas de tags")]
    let head_y = tags_y + n_filas as f32 * ROW_H_PX;
    let rows_y = head_y + ROW_H_PX + TypeToken::Xs.px() + Spacing::Xs.px();
    let mut g = Grelha {
        gx,
        col_w,
        first: st.first_col,
        end: (st.first_col + cabem).min(n),
        tags_y,
        head_y,
        top: rows_y,
        rows_y,
        bottom: area.y + area.h,
    };

    let colunas = Rect::new(gx, area.y, gw, area.h);
    ctx.scene.push_clip(&rect_to_vello(colunas));
    ctx.host.hit_index_mut().push_clip(colunas);
    cabecalhos(ctx, info, &g, &duracoes);
    let rename_tag = st.rename_tag.map(|(t, _)| t);
    for (i, t) in info.tags.iter().enumerate() {
        let (from, to) = crate::gestos::tag_previsto(st, i, t.from, t.to);
        let t = PixelTagInfo {
            from,
            to,
            ..t.clone()
        };
        let puxada = match st.arrasto {
            Some(crate::gestos::Arrasto::Ponta { index, right, .. }) if index == i => Some(right),
            _ => None,
        };
        #[expect(clippy::cast_precision_loss, reason = "poucas filas de tags")]
        let y = g.tags_y + filas_tag[i] as f32 * ROW_H_PX;
        tag(ctx, &g, i, &t, y, rename_tag == Some(i), puxada);
    }
    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();

    // ⭐ As FILAS rolam (a roda sobre o painel, ou a barra): camadas a mais do que a banda nunca
    // ficam recortadas em silêncio. Cabeçalhos e tags ficam fixos por cima.
    let corpo = Rect::new(area.x, g.top, area.w, (g.bottom - g.top).max(0.0));
    let rolagem = ph2d_editor_core::panel::scroll_area::open(
        ctx,
        ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL,
        ph2d_editor_core::widget::PIXEL_TIMELINE_SCROLLBAR_ID,
        corpo,
    );
    g.rows_y = rolagem.top();
    st.geo = Some(crate::gestos::Geo {
        gx,
        col_w,
        first: g.first,
        rows_y: g.rows_y,
        pitch: ph2d_tokens::row_pitch_px(),
    });
    ctx.scene.push_clip(&rect_to_vello(colunas));
    ctx.host.hit_index_mut().push_clip(colunas);
    celulas(ctx, info, &g);
    st.marca = voo
        .as_ref()
        .and_then(|v| crate::paint_voo::vao(ctx, &g, &v.vao));
    moldura(ctx, info, &g);
    if let Some(m) = st.marca {
        crate::paint_voo::marca(ctx, m);
    }
    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();

    // A coluna das camadas, alinhada às filas da grelha.
    let theme = ctx.host.theme();
    let renaming = st.rename_layer.map(|(id, _)| id);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let mut p = LayerRowPainter {
        scene: &mut *ctx.scene,
        text_system: &mut *ctx.text_system,
        store,
        hit_index,
        theme,
        continuous: true,
    };
    let mut y = g.rows_y;
    for row in &info.layers {
        let rename = (renaming == Some(row.id)).then_some(ids::RENAME_LAYER);
        let active = info.active_layer == Some(row.id);
        y = p.paint(row, (area.x, y, layers_w), active, rename, &|w| {
            ids::layer_widget_id(row.id, w)
        });
    }
    #[expect(clippy::cast_precision_loss, reason = "o número de camadas")]
    let alto = info.layers.len() as f32 * ph2d_tokens::row_pitch_px();
    ph2d_editor_core::panel::scroll_area::close(rolagem, ctx, alto);
    if let Some(v) = voo {
        crate::paint_voo::voa(ctx, original, &g, v.dx, v.dy);
    }
}

/// Os números dos frames (1-based) e a duração de cada um por baixo; o activo aceso.
fn cabecalhos(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha, duracoes: &[String]) {
    let theme = ctx.host.theme();
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    let alto = g.top - g.head_y;
    for (c, duracao) in duracoes.iter().enumerate().take(g.end).skip(g.first) {
        let f = info.frames[c];
        let r = Rect::new(g.x(c), g.head_y, g.col_w, alto);
        let no_intervalo =
            matches!(info.range.kind, K::Frames | K::Cels) && info.range.frames.contains(&f.id);
        if info.active_frame == Some(f.id) || no_intervalo {
            fill_rounded_rect(ctx.scene, r, radius, resolve(ColorToken::AccentSoft, theme));
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
            duracao,
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
pub(crate) fn centrado(
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
/// repetições (> 0), e a faixa da cor dela por baixo; `puxada` = a ponta que se arrasta.
fn tag(
    ctx: &mut PaintCtx,
    g: &Grelha,
    i: usize,
    t: &PixelTagInfo,
    y: f32,
    renaming: bool,
    puxada: Option<bool>,
) {
    let (Ok(from), Ok(to)) = (usize::try_from(t.from), usize::try_from(t.to)) else {
        return;
    };
    let (a, b) = (from.max(g.first), (to + 1).min(g.end));
    if a >= b {
        return;
    }
    let theme = ctx.host.theme();
    let x0 = g.x(a);
    let r = Rect::new(x0, y, g.x(b) - x0, ROW_H_PX);
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
    // O nome começa depois da pega da esquerda (smoke 01/11: «walk» caía em cima dela).
    let ponta = Spacing::Sm.px();
    let recuo = ponta + Spacing::Xxs.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        &texto,
        texto_r.x + recuo,
        texto_r.y + (texto_r.h - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        (texto_r.w - recuo).max(1.0),
        resolve(ColorToken::Text1, theme),
    );
    ctx.host.hit_index_mut().register(ids::tag_id(i), r);
    // ⭐ D3: a barra que estica leva a moldura enquanto se arrasta.
    if puxada.is_some() {
        let cor = resolve(ColorToken::Accent, theme);
        let traco = StrokeToken::Thick.px();
        stroke_rounded_rect(ctx.scene, r, radius, traco, cor); // FRAME-RAW-OK: a barra puxada
    }
    // As pontas, por cima do corpo (o último registo vence): só a que está à vista, com a pega
    // sempre pintada e acesa sob o rato ou a ser puxada.
    let quente = ctx.host.store().hot_id();
    for (right, visivel, x) in [
        (false, from >= g.first, r.x),
        (true, to < g.end, r.x + r.w - ponta),
    ] {
        if visivel {
            let id = ids::tag_edge_id(i, right);
            let viva = puxada == Some(right) || quente == Some(id);
            let (w, cor) = if viva {
                (Spacing::Xs.px(), ColorToken::Accent)
            } else {
                (StrokeToken::Thick.px(), ColorToken::Text3)
            };
            let folga = Spacing::Xxs.px();
            let pega = Rect::new(x + (ponta - w) * 0.5, r.y + folga, w, r.h - folga * 2.0);
            fill_rounded_rect(ctx.scene, pega, w * 0.5, resolve(cor, theme));
            let pr = Rect::new(x, r.y, ponta, r.h);
            ctx.host.hit_index_mut().register(id, pr);
        }
    }
}

/// O cel `(camada, quadro)` está no intervalo escolhido.
fn no_intervalo(info: &PixelPanelInfo, layer: u32, frame: u32) -> bool {
    let r = &info.range;
    r.frames.contains(&frame)
        && match r.kind {
            K::None => false,
            K::Frames => true,
            K::Cels | K::Layers => r.layers.contains(&layer),
        }
}

/// A moldura do intervalo (a caixa à volta dele, também com quadros soltos) e os seus quatro lados
/// como alças de mover — registados depois dos cels, para ganharem na borda.
fn moldura(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha) {
    let r = &info.range;
    let cols: Vec<usize> = (g.first..g.end)
        .filter(|&c| r.frames.contains(&info.frames[c].id))
        .collect();
    let filas: Vec<usize> = (0..info.layers.len())
        .filter(|&i| r.kind == K::Frames || r.layers.contains(&info.layers[i].id))
        .collect();
    let (Some(&c0), Some(&c1), Some(&f0), Some(&f1)) =
        (cols.first(), cols.last(), filas.first(), filas.last())
    else {
        return;
    };
    let pitch = ph2d_tokens::row_pitch_px();
    #[expect(clippy::cast_precision_loss, reason = "uma fila à vista")]
    let (y0, y1) = (
        g.rows_y + f0 as f32 * pitch,
        g.rows_y + f1 as f32 * pitch + ROW_H_PX,
    );
    let caixa = Rect::new(g.x(c0), y0, g.x(c1) + g.col_w - g.x(c0), y1 - y0);
    let theme = ctx.host.theme();
    let traco = StrokeToken::Thick.px();
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    // FRAME-RAW-OK: a moldura do intervalo é a marca dele
    stroke_rounded_rect(
        ctx.scene,
        caixa,
        radius,
        traco,
        resolve(ColorToken::Accent, theme),
    );
    let e = Spacing::Xs.px();
    let lados = [
        Rect::new(caixa.x, caixa.y - e * 0.5, caixa.w, e),
        Rect::new(caixa.x + caixa.w - e * 0.5, caixa.y, e, caixa.h),
        Rect::new(caixa.x, caixa.y + caixa.h - e * 0.5, caixa.w, e),
        Rect::new(caixa.x - e * 0.5, caixa.y, e, caixa.h),
    ];
    for (id, lado) in ids::RANGE_EDGES.into_iter().zip(lados) {
        ctx.host.hit_index_mut().register(id, lado);
    }
}

/// Os cels de cada camada (os grupos não têm): o cel activo aceso, os ligados numa pílula.
fn celulas(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha) {
    let theme = ctx.host.theme();
    let cels: BTreeMap<(u32, u32), &PixelCel> =
        info.cels.iter().map(|c| ((c.layer, c.frame), c)).collect();
    let h = ROW_H_PX;
    let d = h * 0.5;
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    // A coluna do activo, por baixo dos cels (fixa: não rola).
    if let Some(c) = event::activo(info).filter(|c| (g.first..g.end).contains(c)) {
        let coluna = Rect::new(g.x(c), g.top, g.col_w, (g.bottom - g.top).max(0.0));
        fill_rounded_rect(ctx.scene, coluna, radius, resolve(ColorToken::Bg2, theme));
    }
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
            let activo = info.active_layer == Some(row.id) && info.active_frame == Some(f);
            if activo || no_intervalo(info, row.id, f) {
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
pub(crate) fn ponto(ctx: &mut PaintCtx, r: Rect, cel: Option<&PixelCel>) {
    let theme = ctx.host.theme();
    let d = r.h * 0.5;
    let dot = |d: f32| Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d);
    let traco = StrokeToken::Default.px();
    // O ponto oco É a marca (o frame sem cel, o cel de imagem vazia), não uma moldura.
    let Some(c) = cel else {
        let (d, cor) = (d * 0.5, resolve(ColorToken::Border, theme));
        stroke_rounded_rect(ctx.scene, dot(d), d * 0.5, traco, cor); // FRAME-RAW-OK: marca
        return;
    };
    if c.empty {
        let cor = resolve(ColorToken::Text2, theme);
        stroke_rounded_rect(ctx.scene, dot(d), d * 0.5, traco, cor); // FRAME-RAW-OK: marca
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
