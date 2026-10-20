//! **O LOTE** — o que um lote de dabs do Wet Paint acerta antes de pousar (filho de [`super`], LOC cap, a partir de
//! `dab_route.rs`): o papel da sessão e o sub-passo da cápsula.

use super::*;

impl PainterTool {
    /// O papel do artista dá o dente ao motor (W2.7), reconciliado por lote pela [`PaperKey`].
    pub(super) fn reconcilia_o_papel(&self, sess: &mut WetSession, brush: &BrushSpec, ratio: u8) {
        let (w, h) = self.source_size;
        // RECONCILED per batch, not seeded once: paper is substrate under
        // live water, but the SLOT is authored state and the session spans
        // strokes — a key of everything the seed reads decides. Key moved =
        // re-seed; slot disarmed = the engine re-bakes its own preset
        // (`rebake_paper`); unchanged = one small struct compare. The law is
        // the painter's own canvas-fixed paper sampling (the NEUTRAL door
        // `sample_tiled_rot_wrapped`, the same law the watercolor substrate
        // reads — one paper, never a second system). Known v1 gap: the
        // bitmap Size seam-snap under sprite Tiling (`snap_slot_size`) is
        // not applied — procedurals wrap exactly; a bitmap paper may seam.
        let period = [
            if self.paint.tiling[0] { w as f32 } else { 0.0 },
            if self.paint.tiling[1] { h as f32 } else { 0.0 },
        ];
        // O Tooth (BUGS #32) mora no MOTOR, que o aplica nas duas portas que escrevem o papel: o assado
        // dele re-assa no `reconcile_facts` abaixo (papel sujo); um papel do artista re-semeia pela chave.
        let tooth = brush
            .paper_depth
            .clamp(0.0, ph2d_painter_brush::PAPER_TOOTH_MAX);
        sess.engine.set_paper_tooth(tooth);
        // ⚠️ **UM papel, o do PAINEL (W1e, F2; dono 10/10).** Com um Kind armado, o do artista; SEM Kind,
        // um papel SEM DESENHO (escolha do dono, 10/10 — `prancha_w1g_sem_papel.png`): as alturas do papel
        // de fábrica, ao acaso célula a célula. A água comporta-se como num papel (o limiar do depósito
        // por célula é o que a faz sangrar no molhado); o desenho do papel — as manchas — some. Nunca o
        // papel procedural do MOTOR: a chave existe nos dois casos, então o motor não re-assa o dele.
        let want = Some(PaperKey {
            tex: brush.paper,
            image_version: self.paint.paper_image_version,
            period,
            tooth,
        });
        if sess.paper_key != want && !brush.paper.is_active() {
            let q = super::super::substrate_relief::alturas_do_papel_de_fabrica();
            // O Tooth esculpe um papel; sem papel não age (as alturas entram inteiras, `dente(h, 1) = h`).
            sess.engine.set_paper_tooth(1.0);
            sess.engine.seed_paper_with(&|cx, cy| {
                f64::from(q[super::super::substrate_relief::celula_ao_acaso(cx, cy, q.len())])
            });
            sess.engine.set_paper_tooth(tooth);
            sess.paper_key = want;
        }
        if sess.paper_key != want {
            let paper_tex = brush.paper;
            let paper_img = self.paint.paper_image.as_ref().map(|i| i.as_mask());
            let rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
            // ⚠️ O `seed_paper_with` fala em CÉLULAS (ele passa `cell − 1`),
            // e o papel é uma lei de CANVAS — sob razão > 1 amostrá-lo na
            // coordenada de célula encolheria o dente do papel pelo mesmo
            // fator, e o papel deixaria de casar com o do resto do app.
            sess.engine.seed_paper_with(&|cx, cy| {
                let px = grid_map::cell_center_texel(cx as i32 + 1, ratio);
                let py = grid_map::cell_center_texel(cy as i32 + 1, ratio);
                f64::from(ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                    &paper_tex,
                    px,
                    py,
                    paper_img.as_ref(),
                    rot,
                    period,
                ))
            });
            sess.paper_key = want;
        }
    }
}

/// ⭐ The dab-CAPSULE (Continuous, [`ph2d_painter_brush::capsula`]) reaches the engine as its
/// sub-dabs, oldest first, at least one engine cell apart: the deposit gates each against the
/// paper (`trail/deposit.rs::cell`), so only the sub-dabs themselves lay the dense stroke —
/// a capsule composed host-side either dilutes under the gate or widens past it (measured).
/// O sub-passo é o do raio do PINCEL — o do dab traz a pressão e a caneta deitada (×3,3), e o traçador sempre
/// espaçou pelo do pincel —, um inteiro de células ([`ph2d_painter_brush::capsula::sub_passo_sem_batimento`]).
pub(super) fn sub_passo_do_lote(
    tuning: &ph2d_wet_paint::tuning::Tuning,
    brush: &BrushSpec,
    ratio: u8,
) -> f32 {
    let sub_da_capsula = tuning.get(ph2d_wet_paint::tuning::Knob::CapsuleSpacing) as f32;
    let tecto_da_capsula = tuning.get(ph2d_wet_paint::tuning::Knob::CapsuleMaxPx) as f32;
    let mais_perto = tuning.get(ph2d_wet_paint::tuning::Knob::CapsuleRoundNearest) > 0.5;
    let n = ph2d_painter_brush::capsula::sub_passo_px_a(brush.clamped_radius(), sub_da_capsula);
    let n = if tecto_da_capsula > 0.0 {
        n.min(tecto_da_capsula)
    } else {
        n
    };
    let cel = f32::from(ratio).max(1.0);
    if mais_perto {
        ph2d_painter_brush::capsula::sub_passo_mais_perto(n, cel)
    } else {
        ph2d_painter_brush::capsula::sub_passo_sem_batimento(n, cel)
    }
}

/// ⭐ **O traço FINO pesa por área o que o largo pesa** (J2, `docs/Painter/Pinceis/03_MOTORES.md` §7 b): cada célula
/// recebe `2r ÷ passo` dabs; o passo é uma fracção do diâmetro, mas nunca menos de UMA célula (o piso do caminhador
/// e da cápsula). Abaixo dele a contagem por célula cai com o diâmetro, e com ela o pigmento e a água por área: o
/// recto de 5 px do 8B cobria 0,38 contra 0,81 a 40 px (o Rebelle 0,80 nos dois). Cada dab pesa então a célula ÷ o
/// passo natural a [`FRACAO_DO_FINO`] do diâmetro — a contagem por área de um traço de ~29 px —; `1` daí para cima, ao
/// bit.
pub(super) fn peso_do_passo_fino(brush: &BrushSpec, ratio: u8) -> f32 {
    let natural = FRACAO_DO_FINO * 2.0 * brush.clamped_radius();
    let cel = f32::from(ratio).max(1.0);
    if natural > 0.0 && natural < cel {
        cel / natural
    } else {
        1.0
    }
}

/// A fracção do diâmetro de [`peso_do_passo_fino`] (o diâmetro de referência `1 ÷ (2 · 0,035)` ≈ 29 px). MEDIDA
/// (03_MOTORES §7 b): 0,025 (a cápsula, 40 px) repunha tudo mas tirava do vale as mídias afinadas a 17–18 px (Soft
/// Pastel, Marker Liner, Dabber) e a tabela da largura colapsava a 1–2 px; 0,05 (20 px) deixava 8 pincéis abaixo
/// da barra (o 8B 0,82 do recto de 40); 0,035 passa os 26 com origem sem tirar nenhuma cena de fábrica do lugar.
pub(super) const FRACAO_DO_FINO: f32 = 0.035;
