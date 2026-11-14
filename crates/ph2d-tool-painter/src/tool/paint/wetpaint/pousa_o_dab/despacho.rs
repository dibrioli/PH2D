//! **A porta do motor que pousa o dab** — o fim de [`super::pousa_o_dab`] (filho dele, LOC cap, B52.R11): a
//! borracha, o Paint (com o pente do relevo e o rebordo), o Blend e as ferramentas directas; e a lane que anda.

use super::super::*;
use super::LoteDoDab;
use ph2d_wet_paint::brush::{CellFn, ColorFn};

/// O que o dab apurou e a porta lê (os nomes são os de [`super::pousa_o_dab`]); `cell` em células.
pub(super) struct Despacho<'a, G, C> {
    pub(super) li: usize,
    pub(super) cell: [f64; 2],
    pub(super) b: f64,
    pub(super) cell_r: f64,
    pub(super) opacidade: f64,
    pub(super) peso: f32,
    pub(super) sil: CellFn<'a>,
    pub(super) grain_arg: Option<CellFn<'a>>,
    pub(super) padrao_arg: Option<CellFn<'a>>,
    pub(super) cor_arg: Option<ColorFn<'a>>,
    pub(super) grain: Option<&'a G>,
    pub(super) cerdas_fn: Option<&'a C>,
    /// O do dab, já sem o filme liso.
    pub(super) ponta_sob_o_grao: bool,
    pub(super) relevo_dab: Option<ph2d_wet_paint::bristles_fios::FiosNoDab>,
    pub(super) meia: [f64; 2],
    pub(super) px_cel: f64,
    pub(super) passos_do_pente: f64,
    pub(super) rumo: [f64; 2],
    pub(super) lane_prev: Option<[f32; 2]>,
    pub(super) passo: Option<f32>,
}

/// A lane fica no centro deste dab (pousado ou não) e larga a gota.
pub(super) fn anda_a_lane(l: &LoteDoDab, sess: &mut WetSession, li: usize, d: &Dab) {
    sess.lanes[li].pos = d.center;
    sess.lanes[li].arco = d.arc_len;
    if l.gota {
        super::super::rota_da_lane::solta_a_gota(sess, li);
    }
}

/// Pousa o dab pela porta da ferramenta.
pub(super) fn despacha<G, C>(
    l: &LoteDoDab,
    sess: &mut WetSession,
    d: &Dab,
    x: Despacho<'_, G, C>,
) where
    G: Fn(i32, i32) -> f64 + Sync,
    C: Fn(i32, i32) -> f64 + Sync,
{
    let &LoteDoDab {
        ref brush,
        erasing,
        wet_tool,
        ratio,
        filme_liso,
        fino,
        esmaece_fino,
        pigmento_da_marca,
        ref relevo,
        rebordo,
        lamina,
        media_depois_do_portao,
        sem_grao,
        ..
    } = l;
    let Despacho {
        li,
        cell: [cell_x, cell_y],
        b,
        cell_r,
        opacidade,
        peso,
        sil,
        grain_arg,
        padrao_arg,
        cor_arg,
        grain,
        cerdas_fn,
        ponta_sob_o_grao,
        relevo_dab,
        meia,
        px_cel,
        passos_do_pente,
        rumo,
        lane_prev,
        passo,
    } = x;
    if erasing {
        // W2.6: the same §9 mapping, routed to the engine's ERASER —
        // silhouette and grain shape the erase exactly as they shape
        // the deposit. O Flow (a taxa por dab, à vista na Borracha do Wet) escala a força
        // do Erase Strength; a Strength escondida já saiu do `b` (`coverage / strength`).
        // A Borracha não tem o canal do padrão: o que desenha depois do portão, de média 1, morde aqui (Bug #65 A1).
        let grao_mao = grain.filter(|_| ponta_sob_o_grao);
        let grao_mao = grao_mao.zip(media_depois_do_portao);
        let cerdas_mao = cerdas_fn.zip(sem_grao);
        let morde = (grao_mao.is_some() || cerdas_mao.is_some()).then_some(|cx: i32, cy: i32| {
            grain_arg.map_or(1.0, |g| g(cx, cy))
                * grao_mao.map_or(1.0, |(g, m)| g(cx, cy) / m)
                * cerdas_mao.map_or(1.0, |(c, [_, m])| c(cx, cy) / m)
        });
        sess.engine.dispatch_pressure_dab_erase(
            cell_x,
            cell_y,
            b,
            f64::from(brush.flow.clamp(0.0, 1.0)),
            cell_r,
            Some(sil),
            morde.as_ref().map_or(grain_arg, |f| Some(f as CellFn<'_>)),
        );
        // A borracha tira também o relevo que houver (sem o campo, nada a tirar).
        if !sess.engine.active_grid().altura.is_empty() {
            let passos = passo.map_or(1.0, |s| f64::from(s) / px_cel);
            sess.engine
                .pentear_relevo(cell_x, cell_y, meia, sil, &|_, _| 0.0, passos);
        }
        return;
    }
    match wet_tool {
        WetTool::Paint => {
            sess.engine.dispatch_pressure_dab_lane_cor(
                li,
                cell_x,
                cell_y,
                b,
                cell_r,
                // O FILME não pesa por comprimento (o máximo de um dab já é a camada): com o peso, o 1.º dab
                // da lane (peso 1) ficava acima dos seguintes (4 ÷ 4,775 a 190 px) — o disco do 1.º toque.
                opacidade
                    * f64::from(if filme_liso { fino } else { peso * fino })
                    * esmaece_fino
                    * f64::from(pigmento_da_marca),
                Some(sil),
                grain_arg,
                padrao_arg,
                cor_arg,
            );
            if let Some(f) = relevo_dab {
                let contacto = f64::from(d.radius_px / d.stroke_radius_px.max(1e-3));
                super::super::lote::penteia_o_relevo_do_dab(
                    &mut sess.engine,
                    &f,
                    relevo,
                    contacto,
                    [cell_x, cell_y],
                    meia,
                    px_cel,
                    sil,
                    passos_do_pente,
                );
            }
            if lamina {
                sess.engine.pentear_rebordo(
                    cell_x,
                    cell_y,
                    meia,
                    sil,
                    rumo,
                    rebordo / px_cel,
                    passos_do_pente,
                );
            }
        }
        // Blend keeps the engine's own fixed-hardness stamp (the
        // model's tools ignore the brush silhouette on purpose) — o Grain armado modula a máscara dele
        // (J4 mistura, L-2; sem Grain, a de sempre ao bit).
        WetTool::Blend => sess.engine.dispatch_pressure_dab_lane_blend_grao(
            li,
            cell_x,
            cell_y,
            b,
            cell_r,
            grain.map(|g| g as CellFn<'_>),
        ),
        // Direct tools: per-dab grid ops; the displacement source is
        // THIS lane's previous centre (host-tracked per copy).
        WetTool::Wet | WetTool::Dry | WetTool::Blow | WetTool::Smear => {
            sess.engine.dispatch_pressure_dab_tool(
                wet_tool.engine(),
                cell_x,
                cell_y,
                b,
                cell_r,
                lane_prev.map(|p| {
                    [
                        grid_map::px_to_cell(f64::from(p[0]), ratio),
                        grid_map::px_to_cell(f64::from(p[1]), ratio),
                    ]
                }),
            );
        }
    }
    anda_a_lane(l, sess, li, d);
}
