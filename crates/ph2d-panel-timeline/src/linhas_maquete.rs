//! A pintura da maquete B1 na fita (doc 15 onda 2): o cabeçalho do objecto, as barras entre
//! chaves e a folha. Irmão de `linhas.rs` (tecto de LOC); re-exportado por `linhas`.

use super::*;

/// A espessura da barra entre duas chaves (maquete B1: 3 px numa linha de 22).
pub(crate) const BARRA_TH: f32 = 3.0; // LITERAL-PX-OK: interpolation bar thickness
/// O traço e o vão da barra de uma chave segura (Hold), tracejada.
pub(crate) const BARRA_DASH: f32 = 3.0; // LITERAL-PX-OK: hold bar dash length
const BARRA_GAP: f32 = 3.0; // LITERAL-PX-OK: hold bar dash gap

/// ⭐ **O cabeçalho do objecto nos cartões** (doc 15 onda 2, maquete B1): o nome em negrito e um
/// ponto por instante em que o objecto tem chave. Só pinta: o clique continua nas faixas.
pub(crate) fn paint_objeto(
    ctx: &mut PaintCtx,
    theme: Theme,
    geo: &geom::Geom,
    view: TimeView,
    preview_dx: f32,
    snap: &TimelineViewSnapshot,
    linha: (u64, f32),
) {
    let (entity, y) = linha;
    let x = geo.rows.x;
    // ▾ só marca o grupo: não há gesto de recolher um objecto (nenhum id/evento), então é pintura.
    let tenue = crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::TextoValor, theme);
    paint_icone(ctx, IconId::ChevronDown, (x, y), tenue);
    let ic = crate::hier::icone_do_objeto(entity);
    let cor_ic = crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::TextoPropriedade, theme);
    paint_icone(ctx, ic, (x + TWIRL_W * 0.5 + ICONE_PX * 0.5, y), cor_ic);
    let tx = x + TWIRL_W * 0.5 + icone_w(Some(ic)) + ICONE_PX * 0.5;
    let font = TypeToken::Sm.px();
    ph2d_editor_core::paint::paint_text_title_elided(
        ctx.text_system,
        ctx.scene,
        &crate::tracks::objeto_snap(snap, entity),
        tx,
        y_do_texto(y, font),
        font,
        (x + geo.label_w - tx - Spacing::Xs.px()).max(0.0),
        crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::TextoCabecalho, theme),
    );
    let cy = y + ROW_H_PX * 0.5;
    for c in columns_of(snap.tracks.iter().filter(|t| t.entity == entity)) {
        let base = view.time_x + ((c.t_seconds - view.view_start) * view.px_per_s) as f32;
        let kx = if c.all_selected {
            base + preview_dx
        } else {
            base
        };
        if kx < view.time_x - DIAMOND_H || kx > view.right + DIAMOND_H {
            continue;
        }
        paint_diamond(
            ctx,
            kx,
            cy,
            ph2d_tokens::TIMELINE_OBJECT_DOT_HALF_PX,
            crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::Ponto, theme),
        );
    }
}

/// A tinta da barra que SAI de uma chave: a da interpolação dela (curva ciano · reta violeta · segura
/// tracejada), as da maquete B1.
pub(crate) fn cor_da_barra(i: ph2d_anim::Interp) -> ph2d_tokens::TimelineTinta {
    match i {
        ph2d_anim::Interp::Hold => ph2d_tokens::TimelineTinta::Ponto,
        ph2d_anim::Interp::Linear => ph2d_tokens::TimelineTinta::BarraReta,
        _ => ph2d_tokens::TimelineTinta::BarraCurva,
    }
}

/// ⭐ **O fundo da folha nos cartões** (maquete B1, onda 2g): a coluna de nomes e a fita com os
/// tons das propriedades, também onde não há linha.
pub(crate) fn pinta_folha_maquete(ctx: &mut PaintCtx, theme: Theme, geo: &geom::Geom) {
    let r = geo.rows;
    let (nomes, fita) = (
        crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::NomesPropriedade, theme),
        crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::FitaPropriedade, theme),
    );
    let fill = |ctx: &mut PaintCtx, rr, c| {
        ph2d_editor_core::paint::fill_rounded_rect(ctx.scene, rr, 0.0, c)
    };
    fill(ctx, Rect::new(r.x, r.y, geo.label_w, r.h), nomes);
    fill(
        ctx,
        Rect::new(r.x + geo.label_w, r.y, (r.w - geo.label_w).max(0.0), r.h),
        fita,
    );
}

/// As tintas `(coluna de nomes, fita)` de uma linha nos cartões: o cabeçalho do objecto um degrau
/// acima das propriedades, como a maquete.
pub(crate) fn tinta_da_linha(l: Linha) -> (ph2d_tokens::TimelineTinta, ph2d_tokens::TimelineTinta) {
    use ph2d_tokens::TimelineTinta as T;
    match l {
        Linha::Objeto(_) => (T::NomesCabecalho, T::FitaCabecalho),
        _ => (T::NomesPropriedade, T::FitaPropriedade),
    }
}

/// Uma linha da maquete: a célula da coluna, a fita e o fio de baixo (coluna e fita, cada um o seu).
pub(crate) fn pinta_linha_maquete(
    ctx: &mut PaintCtx,
    theme: Theme,
    geo: &geom::Geom,
    l: Linha,
    (y, h): (f32, f32),
) {
    use ph2d_tokens::TimelineTinta as T;
    let r = geo.rows;
    let t = |x| crate::folha_tinta::tinta(x, theme);
    let (nomes, fita) = tinta_da_linha(l);
    let lane_w = (r.w - geo.label_w).max(0.0);
    let fill = |ctx: &mut PaintCtx, rr, c| {
        ph2d_editor_core::paint::fill_rounded_rect(ctx.scene, rr, 0.0, c)
    };
    fill(ctx, Rect::new(r.x, y, geo.label_w, h), t(nomes));
    fill(ctx, Rect::new(r.x + geo.label_w, y, lane_w, h), t(fita));
    let fy = y + h - DIVISORIA_H;
    fill(
        ctx,
        Rect::new(r.x, fy, geo.label_w, DIVISORIA_H),
        t(T::NomesFio),
    );
    fill(
        ctx,
        Rect::new(r.x + geo.label_w, fy, lane_w, DIVISORIA_H),
        t(T::FitaFio),
    );
}

/// ⭐ **A barra entre duas chaves** (doc 15 onda 2, maquete B1): pintada por baixo dos losangos, na
/// cor da interpolação da chave de saída; a segura é tracejada. Segue o arrasto como os losangos.
pub(crate) fn paint_barras(
    ctx: &mut PaintCtx,
    theme: Theme,
    track: &TrackView,
    view: TimeView,
    preview_dx: f32,
    y: f32,
) {
    let x_de = |k: &ph2d_timeline::KeyView| {
        let base = view.time_x + ((k.t_seconds - view.view_start) * view.px_per_s) as f32;
        if k.selected { base + preview_dx } else { base }
    };
    let top = y + (ROW_H_PX - BARRA_TH) * 0.5;
    for par in track.keys.windows(2) {
        let (a, b) = (x_de(&par[0]), x_de(&par[1]));
        let (x0, x1) = (a.min(b).max(view.time_x), a.max(b).min(view.right));
        if x1 <= x0 {
            continue;
        }
        let t = cor_da_barra(par[0].interp);
        let tracejada = matches!(par[0].interp, ph2d_anim::Interp::Hold);
        pinta_barra(
            ctx,
            crate::folha_tinta::tinta(t, theme),
            tracejada,
            (x0, x1),
            top,
        );
    }
}

/// Uma barra de `x0` a `x1` com o topo em `top`: cheia, ou tracejada quando é a segura (`tok`).
pub(crate) fn pinta_barra(
    ctx: &mut PaintCtx,
    cor: ph2d_vector::Color,
    tracejada: bool,
    (x0, x1): (f32, f32),
    top: f32,
) {
    if tracejada {
        let mut x = x0;
        while x < x1 {
            let w = BARRA_DASH.min(x1 - x);
            ph2d_editor_core::paint::fill_rounded_rect(
                ctx.scene,
                Rect::new(x, top, w, BARRA_TH),
                0.0,
                cor,
            );
            x += BARRA_DASH + BARRA_GAP;
        }
    } else {
        ph2d_editor_core::paint::fill_rounded_rect(
            ctx.scene,
            Rect::new(x0, top, x1 - x0, BARRA_TH),
            BARRA_TH * 0.5,
            cor,
        );
    }
}
