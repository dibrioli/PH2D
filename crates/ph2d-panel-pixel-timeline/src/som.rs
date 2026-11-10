//! **As linhas de som** (P13 T) sob os quadros: uma por faixa do sprite, com a onda (os picos da
//! `ph2d-pixel-audio`) no tempo dos quadros, Mute e Remove; **Add Audio…** na barra. O painel lê
//! as linhas que a família publicou ([`ph2d_pixel_audio::canal`]) e pede; nunca vê o documento.

use crate::PixelTimelinePanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx};
use ph2d_editor_core::pixel_panel::PixelPanelInfo;
use ph2d_editor_core::widget::{ButtonState, IconButtonStyle, IconGlyph, paint_icon_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_pixel_audio::canal::{self, Linha, Pedido};
use ph2d_pixel_audio::faixas::Edicao;
use ph2d_tokens::{ColorToken, Radius, Spacing, TypeToken};
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// **Add Audio…**
pub const ADD_AUDIO: NodeId = hash_node_id("pixel_timeline.add_audio");

/// O Mute da faixa `i`.
#[must_use]
pub fn mute_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_timeline.audio.{i}.mute"))
}

/// O Remove da faixa `i`.
#[must_use]
pub fn remove_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_timeline.audio.{i}.remove"))
}

/// A banda das linhas de som tirada do fundo de `grade`: `(grade que sobra, banda)`. Uma linha por
/// faixa, até metade da grade; sem som, nenhuma (o Add Audio… mora na barra).
#[must_use]
pub(crate) fn parte(grade: Rect, n: usize) -> (Rect, Rect) {
    #[expect(clippy::cast_precision_loss, reason = "poucas faixas")]
    let h = (n as f32 * ph2d_tokens::row_pitch_px()).min(grade.h * 0.5);
    let resto = Rect::new(grade.x, grade.y, grade.w, (grade.h - h).max(0.0));
    (resto, Rect::new(grade.x, grade.y + resto.h, grade.w, h))
}

/// O `x` do instante `ms` na grelha (as colunas têm a largura igual, os quadros durações próprias;
/// depois do último quadro, a duração dele continua).
fn x_de(info: &PixelPanelInfo, geo: &crate::gestos::Geo, ms: f64) -> f32 {
    let mut t = 0.0_f64;
    let mut c = 0_usize;
    let mut dur = f64::from(info.frames.first().map_or(1, |f| f.duration_ms.max(1)));
    for f in &info.frames {
        dur = f64::from(f.duration_ms.max(1));
        if ms < t + dur {
            break;
        }
        t += dur;
        c += 1;
    }
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "px à vista"
    )]
    let x = geo.gx + (c as f32 - geo.first as f32 + ((ms - t) / dur) as f32) * geo.col_w;
    x
}

pub(crate) fn paint(
    st: &PixelTimelinePanelState,
    ctx: &mut PaintCtx,
    info: &PixelPanelInfo,
    banda: Rect,
) {
    let Some(geo) = st.geo.as_ref() else {
        return;
    };
    if banda.h <= 0.0 {
        return;
    }
    let linhas = canal::linhas();
    let theme = ctx.host.theme();
    let pitch = ph2d_tokens::row_pitch_px();
    let h = ph2d_tokens::ROW_H_PX;
    let esquerda = (geo.gx - banda.x - Spacing::Sm.px()).max(h);
    for (i, l) in linhas.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "poucas faixas")]
        let y = banda.y + i as f32 * pitch;
        if y + h > banda.y + banda.h {
            break;
        }
        for (id, icon, r) in [
            (
                mute_id(i),
                if l.mudo {
                    IconId::VolumeX
                } else {
                    IconId::Volume2
                },
                Rect::new(banda.x, y, h, h),
            ),
            (
                remove_id(i),
                IconId::Trash,
                Rect::new(banda.x + esquerda - h, y, h, h),
            ),
        ] {
            ctx.host.hit_index_mut().register(id, r);
            let visual = ctx.host.store().button_visual(id);
            let glyph = IconGlyph::Builtin(icon);
            paint_icon_button(r, glyph, IconButtonStyle::Compact, visual, ctx.scene, theme);
        }
        let nome = if l.dur_ms.is_some() {
            l.nome.clone()
        } else {
            tr_with("panel.pixel.timeline.audio_missing", &[("asset", &l.nome)])
        };
        let tinta = if l.mudo {
            ColorToken::Text3
        } else {
            ColorToken::Text2
        };
        paint_text(
            ctx.text_system,
            ctx.scene,
            &nome,
            banda.x + h + Spacing::Xs.px(),
            y + (h - TypeToken::Xs.px()) * 0.5,
            TypeToken::Xs.px(),
            (esquerda - 2.0 * h - Spacing::Xs.px()).max(1.0),
            resolve(tinta, theme),
        );
        onda(
            ctx,
            info,
            geo,
            l,
            Rect::new(geo.gx, y, banda.x + banda.w - geo.gx, h),
        );
    }
}

/// A onda: um traço por pico, do mínimo ao máximo, no tempo da faixa (`inicio_ms` desloca-a).
fn onda(ctx: &mut PaintCtx, info: &PixelPanelInfo, geo: &crate::gestos::Geo, l: &Linha, r: Rect) {
    if r.w <= 0.0 {
        return;
    }
    let theme = ctx.host.theme();
    ctx.scene.push_clip(&rect_to_vello(r));
    let fundo = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(ctx.scene, r, fundo, resolve(ColorToken::AccentSoft, theme));
    let cor = resolve(
        if l.mudo {
            ColorToken::Text3
        } else {
            ColorToken::Accent
        },
        theme,
    );
    let passo = f64::from(ph2d_pixel_audio::PICO_MS);
    let meio = r.y + r.h * 0.5;
    for (k, p) in l.picos.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "ms de um som")]
        let ms = f64::from(l.inicio_ms) + k as f64 * passo;
        let (x0, x1) = (x_de(info, geo, ms), x_de(info, geo, ms + passo));
        if x1 < r.x {
            continue;
        }
        if x0 > r.x + r.w {
            break;
        }
        let topo = meio - p.max.clamp(0.0, 1.0) * r.h * 0.5;
        let base = meio - p.min.clamp(-1.0, 0.0) * r.h * 0.5;
        let barra = Rect::new(x0, topo, (x1 - x0).max(0.0), (base - topo).max(0.0));
        fill_rounded_rect(ctx.scene, barra, 0.0, cor);
    }
    ctx.scene.pop_layer();
}

/// O registo por quadro: o Add Audio… e o Mute/Remove de cada linha publicada.
pub(crate) fn registra(store: &mut WidgetStore) {
    let n = canal::linhas().len();
    let ids = (0..n).flat_map(|i| {
        [
            (mute_id(i), "panel.pixel.timeline.audio_mute_tip"),
            (remove_id(i), "panel.pixel.timeline.audio_remove_tip"),
        ]
    });
    for (id, dica) in std::iter::once((ADD_AUDIO, "panel.pixel.timeline.add_audio_tip")).chain(ids)
    {
        let state = ButtonState::Normal;
        store.register_if_absent(id, InteractiveState::Button { state });
        store.set_tooltip(id, tr(dica));
    }
}

/// Os cliques das linhas de som; `None` = não é daqui.
pub(crate) fn evento(ev: &WidgetEvent) -> Option<EventOutcome> {
    let WidgetEvent::Click(id) = *ev else {
        return None;
    };
    if id == ADD_AUDIO {
        canal::pede(Pedido::Escolher);
        return Some(EventOutcome::Consumed);
    }
    let linhas = canal::linhas();
    let i = (0..linhas.len()).find(|&i| mute_id(i) == id || remove_id(i) == id)?;
    let e = if mute_id(i) == id {
        Edicao::Mudo(i, !linhas[i].mudo)
    } else {
        Edicao::Remove(i)
    };
    canal::pede(Pedido::Edita(e));
    Some(EventOutcome::Consumed)
}
