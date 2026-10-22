//! **O PÓ SOLTO no Wet Paint** (J4 `pastel_carvao`, `docs/Painter/Pinceis/exames/pastel_carvao.md` 2.ª ronda) — filho
//! de [`super::dab_route`] por LOC.
//!
//! A lei dos grãos é a do Digital ([`ph2d_painter_brush::stroke::spray::graos_do_po`], o mesmo
//! [`ph2d_painter_brush::marca::DabJitter::po`]); o traço do Wet não os emite (`dab_spec`: o `po` sai neutro fora do
//! Digital) porque dentro do lote eles caíam nas lanes e nos sub-passos da cápsula e pintavam NÓDOAS (medido em
//! `exames/texturas.md` «Motor J4»). Aqui cada dab do lote larga os seus grãos DEPOIS do lote, e cada grão pousa À
//! PARTE ([`ph2d_wet_paint::painter::Engine::pousa_grao_solto`]): um rasto próprio, sem corda, âncora nem carga da lane.
//! `po = 0` (todo preset de hoje): nada se sorteia nem se aloca — o traço ao byte.

use super::*;

/// O estado do pó de UM traço (zerado quando o traço abre).
#[derive(Clone, Default)]
pub(in crate::tool::paint) struct PoDoTraco {
    /// O rasto dos grãos (nasce no 1.º grão).
    rasto: Option<Box<ph2d_wet_paint::trail::Trail>>,
    /// O resto da contagem de cada cópia (simetria, spray), pela ordem da cópia.
    restos: Vec<f32>,
    /// O fluxo dos grãos, semeado pelo centro do 1.º dab do traço (o mesmo gesto, os mesmos grãos).
    rng: Option<u64>,
}

impl PainterTool {
    /// Os grãos de pó do lote `dabs` (as cópias `copia`), pousados à parte na sessão `sess`. `opacidade_da_pressao`
    /// e `strength`: as da rota (o grão lê a sua cobertura como o dab lê a dele).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn pousa_o_po_do_lote(
        &self,
        sess: &mut WetSession,
        dabs: &[Dab],
        copia: &[u32],
        brush: &BrushSpec,
        opacidade_da_pressao: Option<f32>,
        strength: f32,
    ) {
        let j = self.paint.brush.dab_jitter;
        if j.po <= 0.0
            || !brush.stroke_method.allows_jitter()
            || self.paint.eraser
            || self.paint.wetpaint.tool != WetTool::Paint
        {
            return;
        }
        let ratio = sess.ratio;
        let passo = brush.passo_de(brush.spacing);
        // O dab do Wet tem o raio do MOTOR (a largura prometida, `largura.rs`: o portão do papel come a orla); o pó cai
        // à volta da borda que se VÊ — o raio do artista. Sem a lei, o mesmo.
        let visto = if brush.radius_px > 0.0 {
            (self.paint.brush.radius_px / brush.radius_px).min(1.0)
        } else {
            1.0
        };
        let po = &mut sess.po;
        let mut graos = Vec::new();
        for (d, &k) in dabs.iter().zip(copia) {
            let d = &Dab {
                radius_px: d.radius_px * visto,
                ..*d
            };
            let rng = po.rng.get_or_insert_with(|| semente(d.center));
            let k = k as usize;
            if po.restos.len() <= k {
                po.restos.resize(k + 1, 0.0);
            }
            ph2d_painter_brush::stroke::spray::graos_do_po(
                &j,
                d,
                passo,
                &mut po.restos[k],
                rng,
                |g| {
                    graos.push(g);
                },
            );
        }
        if graos.is_empty() {
            return;
        }
        let rasto = po.rasto.get_or_insert_with(Box::default);
        let (aa_n, aa_step) = grid_map::cell_subsamples(ratio);
        let aa_inv = 1.0 / f32::from(u16::from(aa_n) * u16::from(aa_n));
        // ⭐ Um grão pousa UMA vez; a célula do traço recebe `1 ÷ FRACAO_DO_FINO` dabs (o passo da cápsula). O grão pesa
        // isso — `po_opacidade` 1 = o tom do miolo do traço (medido: com o peso de um dab só, nem a 1 passavam de névoa).
        let peso = 1.0 / f64::from(super::lote::FRACAO_DO_FINO);
        for g in &graos {
            let (b, opacidade) = match opacidade_da_pressao {
                Some(ov) => (
                    f64::from(ov) * 10.0,
                    (f64::from(g.coverage) / (f64::from(strength) * f64::from(ov)).max(1e-9))
                        .clamp(0.0, 1.0),
                ),
                None => (
                    f64::from((g.coverage / strength).clamp(0.0, 1.0)) * 10.0,
                    1.0,
                ),
            };
            let opacidade = opacidade * peso;
            let (c, r) = (g.center, g.radius_px);
            // O grão é um disco duro com a orla de AA de um píxel, pela cobertura dos sub-pontos da célula.
            let sil = |cx: i32, cy: i32| -> f64 {
                let mut acc = 0.0f32;
                for jj in 0..aa_n {
                    let sy = grid_map::cell_subsample_px(cy, ratio, jj, aa_step);
                    for ii in 0..aa_n {
                        let sx = grid_map::cell_subsample_px(cx, ratio, ii, aa_step);
                        acc += (r + 0.5 - (sx - c[0]).hypot(sy - c[1])).clamp(0.0, 1.0);
                    }
                }
                f64::from(acc * aa_inv)
            };
            let ink = g.color.map(|v| f64::from(v) * 255.0);
            sess.engine.pousa_grao_solto(
                rasto,
                ink,
                grid_map::px_to_cell(f64::from(c[0]), ratio),
                grid_map::px_to_cell(f64::from(c[1]), ratio),
                b,
                grid_map::px_len_to_cell(f64::from(r), ratio),
                opacidade,
                &sil,
            );
        }
    }
}

/// A semente do fluxo dos grãos: os bits do centro do 1.º dab, dobrados (splitmix64).
fn semente(c: [f32; 2]) -> u64 {
    let mut z =
        (u64::from(c[0].to_bits()) << 32 | u64::from(c[1].to_bits())) ^ 0x706F_5F77_6574_0000;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
