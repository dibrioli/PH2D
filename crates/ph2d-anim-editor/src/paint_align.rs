//! **A pintura da prévia Align** ([`super::align`]): o botão, a barra, os três quadros (referência, anterior, aberto),
//! a linha do chão, a cruz do Pivot, a pega do canto e o menu do clique direito. Lê a disposição; não decide nada.

use super::align::{AlignPaint, View};
use super::doc::AnimDoc;
use super::paint::tom;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, label_column_width, paint_icon, paint_text,
    rect_for_label, rect_to_vello, stroke_polyline, stroke_rect, stroke_rounded_rect,
};
use ph2d_editor_core::widget::sheet_cell::paint_sheet_cell_with;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimFrameRef, IconId};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Color, Compose, ImageQuality, Mix, VectorScene, VelloBlend};

/// A opacidade do quadro anterior (ténue: o idioma do onion skin).
const PREVIOUS_ALPHA: f32 = 0.3; // LITERAL-PX-OK: opacidade
/// A opacidade da referência e o quanto a tinta dela cobre o desenho.
const REFERENCE_ALPHA: f32 = 0.5; // LITERAL-PX-OK: opacidade
const REFERENCE_TINT: f32 = 0.6; // LITERAL-PX-OK: opacidade da tinta
/// O braço da cruz do Pivot e o lado da pega do canto, em alturas de linha.
const CROSS_ROWS: f32 = 0.6; // LITERAL-PX-OK: múltiplo do token de linha
const HANDLE_ROWS: f32 = 0.4; // LITERAL-PX-OK: múltiplo do token de linha
const FONT: TypeToken = TypeToken::Sm;

/// Um quadro da prévia: a célula da folha dele esticada no quad, numa camada com `alpha` e, se dada, a tinta por cima
/// só do desenho (SrcAtop: o vazio fica vazio).
fn layer(
    doc: &AnimDoc,
    scene: &mut VectorScene,
    v: View,
    f: AnimFrameRef,
    alpha: f32,
    tint: Option<Color>,
) -> Option<Rect> {
    let r = v.quad(doc.frame_quad(f)?);
    let sheet = doc.snap.sheets.get(usize::from(f.sheet))?;
    let img = sheet.image.as_ref()?;
    let vr = rect_to_vello(r);
    scene.push_object_layer(&vr, VelloBlend::default(), alpha);
    let cut = (sheet.hframes, sheet.vframes, f.cell);
    paint_sheet_cell_with(scene, img, cut, r, ph2d_editor_core::ui_image_quality());
    if let Some(c) = tint {
        scene.push_object_layer(
            &vr,
            VelloBlend::new(Mix::Normal, Compose::SrcAtop),
            REFERENCE_TINT,
        );
        scene.fill_rect(vr, c);
        scene.pop_layer();
    }
    scene.pop_layer();
    Some(r)
}

/// O botão «Align» e, ligado, a barra e a prévia.
pub(super) fn paint_align(
    doc: &mut AnimDoc,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let l = doc.layout.align;
    let menu_item = doc.align.painted.menu_item;
    doc.align.painted = AlignPaint {
        menu_item,
        ..AlignPaint::default()
    };
    paint_frame(doc, scene, text, theme);
    if !doc.align.on {
        return;
    }
    let p = l.preview;
    fill_rounded_rect(
        scene,
        p,
        frame_radius(theme, Radius::Sm.px()),
        tom(ColorToken::Bg0, theme),
    );
    let font = FONT.px();
    let reference = doc.align_reference();
    let Some(v) = doc.align_view(p) else {
        return;
    };
    scene.push_clip(&rect_to_vello(p));
    let mut painted = doc.align.painted;
    if doc.align.show_ref
        && let Some(f) = reference
    {
        painted.reference = layer(
            doc,
            scene,
            v,
            f,
            REFERENCE_ALPHA,
            Some(tom(ColorToken::Info, theme)),
        );
    }
    if doc.align.show_prev
        && let Some(f) = doc.align_previous()
    {
        painted.previous = layer(doc, scene, v, f, PREVIOUS_ALPHA, None);
    }
    let current = doc.align_current();
    if let Some((_, f)) = current {
        painted.current = layer(doc, scene, v, f, 1.0, None);
    }
    let thin = StrokeToken::Thin.px();
    // A silhueta da referência, POR CIMA do quadro aberto (dono 09/11): a imagem dela fica por baixo e é tapada.
    let caixa = doc.align.show_ref.then(|| doc.reference_box()).flatten();
    if let Some((lo, hi)) = caixa {
        painted.reference_box = Some(v.quad((
            [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0],
            [hi[0] - lo[0], hi[1] - lo[1]],
        )));
    }
    if doc.align.show_ref
        && let Some((f, sil)) = doc.reference_silhouette()
        && let Some(q) = doc.frame_quad(f)
    {
        let r = v.quad(q);
        let at = |fx: f32, fy: f32| (r.x + fx * r.w, r.y + fy * r.h);
        let c = tom(ColorToken::Info, theme);
        for e in &sil.arestas {
            stroke_polyline(scene, &[at(e[0], e[1]), at(e[2], e[3])], thin, c);
        }
        painted.wireframe = sil.arestas.len();
    }
    if doc.align.show_ref
        && let Some(floor) = doc.align.floor.or(caixa.map(|(lo, _)| lo[1]))
    {
        let (_, fy) = v.to_screen([0.0, floor]);
        let c = tom(ColorToken::Warn, theme);
        stroke_polyline(scene, &[(p.x, fy), (p.x + p.w, fy)], thin, c);
        painted.floor = Some(fy);
    }
    let accent = tom(ColorToken::Accent, theme);
    if let Some(r) = painted.current {
        stroke_rect(scene, r, thin, accent);
        let side = ROW_H_PX * HANDLE_ROWS;
        let quinas = [
            (r.x, r.y),
            (r.x + r.w, r.y),
            (r.x, r.y + r.h),
            (r.x + r.w, r.y + r.h),
        ];
        for (slot, (qx, qy)) in painted.handles.iter_mut().zip(quinas) {
            let h = Rect::new(qx - side / 2.0, qy - side / 2.0, side, side);
            fill_rounded_rect(scene, h, 0.0, accent);
            *slot = Some(h);
        }
    }
    let (ox, oy) = v.to_screen([0.0, 0.0]);
    let arm = ROW_H_PX * CROSS_ROWS;
    let thick = StrokeToken::Thick.px();
    stroke_polyline(scene, &[(ox - arm, oy), (ox + arm, oy)], thick, accent);
    stroke_polyline(scene, &[(ox, oy - arm), (ox, oy + arm)], thick, accent);
    fill_circle(scene, ox, oy, thick, tom(ColorToken::Bg0, theme));
    painted.pivot = Some((ox, oy));
    let hint = tr("anim_editor.align_hint");
    let hy = p.y + p.h - ROW_H_PX - Spacing::Xs.px();
    let hx = p.x + Spacing::Sm.px();
    paint_text(
        text,
        scene,
        hint,
        hx,
        hy,
        font,
        p.w,
        tom(ColorToken::Text3, theme),
    );
    scene.pop_layer();
    doc.align.painted = painted;
}

/// ⭐ A moldura própria do Align, embaixo à direita (dono 09/11): o título com a referência, «Align» e o conta-gotas,
/// «Previous» e «Reference». A moldura é a das outras com um fio de acento — levemente diferente.
fn paint_frame(doc: &AnimDoc, scene: &mut VectorScene, text: &mut TextSystem, theme: Theme) {
    let l = doc.layout.align;
    if l.frame.w <= 0.0 {
        return;
    }
    let radius = frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, l.frame, radius, tom(ColorToken::Bg2, theme));
    let fio = StrokeToken::Thin.px();
    stroke_rounded_rect(
        scene,
        l.frame,
        radius,
        fio,
        tom(ColorToken::AccentSoft, theme),
    );
    let font = TypeToken::Base.px();
    let t = l.title;
    let y = t.y + (t.h - ROW_H_PX).max(0.0) / 2.0;
    // A fila do título diz o que o Align está a fazer: a referência, o conta-gotas armado, ou só o nome dele.
    let reference = doc.align_reference().filter(|_| doc.align.on);
    let (s, c) = match reference {
        _ if doc.align.picking => (tr("anim_editor.align_picking").to_owned(), ColorToken::Info),
        Some(f) => {
            let name = doc
                .snap
                .sheets
                .get(usize::from(f.sheet))
                .map_or("", |s| s.name.as_str());
            let s = tr_with(
                "anim_editor.align_ref_is",
                &[("sheet", &name), ("n", &(f.cell + 1))],
            );
            (s, ColorToken::Info)
        }
        None => (tr("anim_editor.align").to_owned(), ColorToken::Text2),
    };
    paint_text(text, scene, &s, t.x, y, font, t.w, tom(c, theme));
    let a = &doc.align;
    let fio = StrokeToken::Thin.px();
    let toggle = (IconId::AlignBottom, tr("anim_editor.align"));
    chip(
        scene,
        text,
        theme,
        l.toggle,
        toggle,
        Look::of(true, a.on),
        fio,
    );
    let pick = (IconId::Eyedropper, tr("anim_editor.align_pick"));
    chip(
        scene,
        text,
        theme,
        l.pick,
        pick,
        Look::of(true, a.picking),
        fio,
    );
    // Previous e Reference só valem com o Align ligado: até lá, apagados.
    let prev = (IconId::SkipBack, tr("anim_editor.align_previous"));
    chip(
        scene,
        text,
        theme,
        l.show_prev,
        prev,
        Look::of(a.on, a.show_prev),
        fio,
    );
    let refe = (IconId::Pin, tr("anim_editor.align_reference"));
    chip(
        scene,
        text,
        theme,
        l.show_ref,
        refe,
        Look::of(a.on, a.show_ref),
        fio,
    );
    // Ações de um clique (onda C): nunca acesas; apagadas com o Align desligado.
    let feet = (IconId::AlignBottom, tr("anim_editor.align_by_feet"));
    chip(
        scene,
        text,
        theme,
        l.by_feet,
        feet,
        Look::of(a.on, false),
        fio,
    );
    let centre = (IconId::AlignMiddle, tr("anim_editor.align_by_centre"));
    chip(
        scene,
        text,
        theme,
        l.by_centre,
        centre,
        Look::of(a.on, false),
        fio,
    );
}

/// ⭐ A marca na tira de um quadro com acerto próprio (onda C): o ícone de mover, no canto de cima à direita (o de baixo
/// é o nome da folha), na tinta do Align. Sob o rato o canto é do ×, e a marca cede-lhe o lugar.
pub(super) fn paint_nudge_mark(scene: &mut VectorScene, theme: Theme, thumb: Rect) {
    // Um selo cheio na cor de acento (smoke do dono 09/11: o ícone sozinho, fino e em Info, não se via).
    let side = ROW_H_PX * 0.8; // LITERAL-PX-OK: fração da linha
    let pad = Spacing::Xs.px();
    let r = Rect::new(thumb.x + thumb.w - side - pad, thumb.y + pad, side, side);
    let (cx, cy) = (r.x + side / 2.0, r.y + side / 2.0);
    fill_circle(scene, cx, cy, side / 2.0, tom(ColorToken::Accent, theme));
    let inset = side * 0.2; // LITERAL-PX-OK: a margem do ícone no selo
    let icon = Rect::new(
        r.x + inset,
        r.y + inset,
        side - 2.0 * inset,
        side - 2.0 * inset,
    ); // LITERAL-PX-OK: as duas margens
    paint_icon(
        scene,
        IconId::Move,
        icon,
        tom(ColorToken::Bg0, theme),
        StrokeToken::Thick.px(),
    );
}

/// Como um botão da moldura se mostra.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Look {
    /// Sem efeito agora (o Align desligado): letra e ícone apagados.
    Off,
    Idle,
    On,
}

impl Look {
    const fn of(enabled: bool, on: bool) -> Self {
        match (enabled, on) {
            (false, _) => Self::Off,
            (true, false) => Self::Idle,
            (true, true) => Self::On,
        }
    }
}

/// Um botão da moldura do Align no idioma dos campos ao lado («F 10», «C 6»): a caixa um degrau acima da moldura, o
/// ícone e a palavra; ligado, o fio e a tinta de acento; sem efeito, apagado (smoke do dono 09/11).
fn chip(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    (icon, label): (IconId, &str),
    look: Look,
    fio: f32,
) {
    let radius = frame_radius(theme, m::FIELD_RADIUS);
    fill_rounded_rect(scene, r, radius, tom(ColorToken::Bg3, theme));
    let fg = match look {
        Look::Off => ColorToken::TextDisabled,
        Look::Idle => ColorToken::Text1,
        Look::On => ColorToken::Accent,
    };
    if look == Look::On {
        stroke_rounded_rect(scene, r, radius, fio, tom(fg, theme));
    }
    let font = TypeToken::Base.px();
    let pad = Spacing::Xs.px();
    let side = r.h - 2.0 * pad; // LITERAL-PX-OK: as duas margens
    // A largura que se PINTA (a do `paint_text_centered`), não a da coluna de rótulos: noutro peso, ícone e palavra
    // saíam encostados à esquerda na tela do dono (09/11).
    let medida = text.layout(label, font, f32::INFINITY);
    let (word, word_h) = (medida.width(), medida.height());
    let gap = Spacing::Xs.px();
    let x = r.x + ((r.w - side - gap - word) / 2.0).max(pad);
    paint_icon(
        scene,
        icon,
        Rect::new(x, r.y + pad, side, side),
        tom(fg, theme),
        fio,
    );
    let y = r.y + (r.h - word_h) / 2.0;
    let tx = x + side + gap;
    paint_text(
        text,
        scene,
        label,
        tx,
        y,
        font,
        (r.x + r.w - tx).max(0.0),
        tom(fg, theme),
    );
}

/// O menu do clique direito sobre um quadro da tira, por cima de tudo.
pub(super) fn paint_align_menu(
    doc: &mut AnimDoc,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    doc.align.painted.menu_item = None;
    let Some((_, (x, y))) = doc.align.menu else {
        return;
    };
    let font = FONT.px();
    let label = tr("anim_editor.use_as_reference");
    let w = rect_for_label(label_column_width(text, font, [label]));
    let r = Rect::new(x, y, w, ROW_H_PX);
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, Radius::Sm.px()),
        tom(ColorToken::Bg3, theme),
    );
    let c = tom(ColorToken::Text1, theme);
    paint_text(
        text,
        scene,
        label,
        r.x + Spacing::Sm.px(),
        r.y,
        font,
        r.w,
        c,
    );
    doc.align.painted.menu_item = Some(r);
}
