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
/// tracejada), as da maquete B1. O DESENHO dela é a curva: [`pinta_curva`].
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

/// ⭐ **A barra entre duas chaves É a curva** (dono 13/11: «as linhas coloridas entre frames devem
/// também representar a curva de easing escolhida»): pintada por baixo dos losangos, na cor da
/// interpolação da chave de saída, vai da ponta de baixo dela à ponta de cima da seguinte (ou o
/// contrário, se o valor desce) pela conta do documento; a segura continua tracejada e reta. Segue o
/// arrasto como os losangos.
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
    let faixa = Faixa {
        cy: y + ROW_H_PX * 0.5,
        amp: crate::tracks::chave_h(),
        lim: (y + BARRA_TH * 0.5, y + ROW_H_PX - BARRA_TH * 0.5),
    };
    for par in track.keys.windows(2) {
        let (a, b) = (x_de(&par[0]), x_de(&par[1]));
        let visivel = (a.min(b).max(view.time_x), a.max(b).min(view.right));
        if visivel.1 <= visivel.0 {
            continue;
        }
        let cor = crate::folha_tinta::tinta(cor_da_barra(par[0].interp), theme);
        let valores = (par[0].value, par[1].value);
        pinta_curva(ctx, cor, par[0].interp, valores, (a, b), visivel, faixa);
    }
}

/// Onde a curva de uma barra mora: o centro da linha, a meia-altura (0 → `cy + amp`, 1 → `cy − amp`)
/// e os limites `(cima, baixo)` a que o que passa (Back, Elastic) se prende.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Faixa {
    pub cy: f32,
    pub amp: f32,
    pub lim: (f32, f32),
}

/// O passo, em px, entre duas amostras da curva de uma barra, e o tecto de amostras por barra.
const PASSO_PX: f32 = 2.0; // LITERAL-PX-OK: densidade da curva da barra (2 px por amostra)
const MAX_AMOSTRAS: f32 = 256.0; // LITERAL-PX-OK: tecto de amostras por barra (uma barra de 512 px)

/// Pinta a barra de `interp` entre as chaves em `x = a` e `x = b` (valores `v0 → v1`), só no trecho
/// `visivel`: a segura tracejada e reta no centro; o resto, a curva de [`pontos_da_curva`].
pub(crate) fn pinta_curva(
    ctx: &mut PaintCtx,
    cor: ph2d_vector::Color,
    interp: ph2d_anim::Interp,
    valores: (f32, f32),
    ab: (f32, f32),
    visivel: (f32, f32),
    faixa: Faixa,
) {
    if matches!(interp, ph2d_anim::Interp::Hold) {
        pinta_barra(ctx, cor, true, visivel, faixa.cy - BARRA_TH * 0.5);
        return;
    }
    let pts = pontos_da_curva(interp, valores, ab, visivel, faixa);
    ph2d_editor_core::paint::stroke_polyline(ctx.scene, &pts, BARRA_TH, cor);
}

/// Os pontos de tela da curva de uma barra. A fração é a do documento (`Interp::value`); sobe quando
/// o valor sobe e desce quando desce (parada sobe: mostra a forma escolhida). A Bézier de tangentes
/// absolutas (`BezierW`) só tem forma com o valor a mudar — parada, é a linha do centro.
pub(crate) fn pontos_da_curva(
    interp: ph2d_anim::Interp,
    (v0, v1): (f32, f32),
    (a, b): (f32, f32),
    (x0, x1): (f32, f32),
    f: Faixa,
) -> Vec<(f32, f32)> {
    let span = b - a;
    if span.abs() <= f32::EPSILON {
        return vec![(x0, f.cy), (x1, f.cy)];
    }
    let (v0, v1) = (f64::from(v0), f64::from(v1));
    let dv = v1 - v0;
    let fracao = |u: f64| match interp {
        ph2d_anim::Interp::BezierW { .. } if dv.abs() <= f64::EPSILON => 0.5,
        ph2d_anim::Interp::BezierW { .. } => (interp.value(v0, v1, u) - v0) / dv,
        _ => interp.value(0.0, 1.0, u),
    };
    let desce = dv < 0.0;
    let n = ph2d_editor_core::math::safe_clamp(((x1 - x0) / PASSO_PX).ceil(), 1.0, MAX_AMOSTRAS)
        as usize;
    (0..=n)
        .map(|i| {
            let x = x0 + (x1 - x0) * i as f32 / n as f32;
            let u = f64::from(((x - a) / span).clamp(0.0, 1.0));
            let fr = fracao(u) as f32;
            let fr = if desce { 1.0 - fr } else { fr };
            (
                x,
                ph2d_editor_core::math::safe_clamp(
                    f.cy + f.amp - 2.0 * f.amp * fr,
                    f.lim.0,
                    f.lim.1,
                ),
            )
        })
        .collect()
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

#[cfg(test)]
mod curva_tests {
    use super::*;
    use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M, Interp};

    const FX: Faixa = Faixa {
        cy: 50.0,
        amp: 6.0,
        lim: (40.0, 60.0),
    };

    fn y_em(pts: &[(f32, f32)], x: f32) -> f32 {
        pts.iter()
            .min_by(|a, b| (a.0 - x).abs().total_cmp(&(b.0 - x).abs()))
            .unwrap()
            .1
    }

    /// A barra é a curva da chave de saída: sai da ponta de baixo e chega à de cima (ou o contrário
    /// se o valor desce), com o miolo da forma escolhida; o que passa prende-se à linha.
    #[test]
    fn the_bar_between_two_keys_draws_the_chosen_easing() {
        let ab = (0.0, 100.0);
        let reta = pontos_da_curva(Interp::Linear, (0.0, 5.0), ab, ab, FX);
        assert_eq!((reta[0].1, reta.last().unwrap().1), (56.0, 44.0));
        assert!((y_em(&reta, 50.0) - 50.0).abs() < 0.1);
        // Quad In: no meio fez só um quarto do caminho (mais perto da ponta de baixo).
        let quad = Interp::Eased(Easing::new(F::Quad, M::In));
        let q = pontos_da_curva(quad, (0.0, 5.0), ab, ab, FX);
        assert!((y_em(&q, 50.0) - (56.0 - 12.0 * 0.25)).abs() < 0.1);
        // Valor a descer: a mesma forma, de cima para baixo.
        let d = pontos_da_curva(quad, (5.0, 0.0), ab, ab, FX);
        assert_eq!((d[0].1, d.last().unwrap().1), (44.0, 56.0));
        // O Elastic passa do 1 e fica preso ao limite da linha, nunca fora dela.
        let elastic = Interp::Eased(Easing::new(F::Elastic, M::Out));
        let b = pontos_da_curva(elastic, (0.0, 5.0), ab, ab, FX);
        assert!(b.iter().all(|p| p.1 >= FX.lim.0 && p.1 <= FX.lim.1));
        assert!(
            b.iter().any(|p| p.1 < 44.0),
            "o Elastic passa da chave de chegada"
        );
        // Só o trecho visível é pintado, com a fração do segmento inteiro.
        let meio = pontos_da_curva(Interp::Linear, (0.0, 5.0), ab, (50.0, 100.0), FX);
        assert!((meio[0].1 - 50.0).abs() < 1e-3);
    }
}
