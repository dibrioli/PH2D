//! O splat da COR da lavagem (`accumulate_wet_color`) — irmão do splat da cobertura, que vive em
//! [`super::watercolor_accum`]: os dois carimbam a MESMA pegada com o MESMO peso, e a cor e a
//! silhueta concordam por isso. Partido daquele ficheiro pelo tecto de LOC quando o Smudge sobre a
//! tinta molhada ([`super::watercolor_mistura_arrasto`]) entrou aqui (2026-09-29).

use super::watercolor_accum::{WASH_DEPOSIT_PEAK, WetShapeStamp, feather, splat_keep};
use super::*;

impl PainterTool {
    /// Splat each dab's colour into the per-stroke colour buffer, **source-over** (recent dab wins on
    /// overlap, carrying its colour — wet_edges `stampColor`), straight-alpha RGBA. Same soft-disc
    /// feather as the coverage, so the deposited colour and the silhouette agree.
    pub(super) fn accumulate_wet_color(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_color.len() != fw * fh * 4 {
            self.paint.stroke_color = vec![0u8; fw * fh * 4];
        }
        // Wet Mix (`docs/Painter/07` §4): the deposited colour is the mixer's per-dab blend of the
        // brush colour with the surface it picked up (Charge/Pull). Off (default `wet_charge = 1`) ⇒
        // each dab's own colour (byte-identical). Computed BEFORE borrowing `stroke_color`.
        let mixed = self.wet_mix_dab_colors(dabs);
        // Selection + protection + alpha-lock gates — twin of the coverage splat (see [`splat_keep`]).
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
        // Shape "Automatic" OFF: REPLAY the coverage pass's rng stream from the same seed (identical
        // per-dab Random bases ⇒ colour and coverage agree pixel-wise), then ADVANCE the stroke
        // stream here — net one advance per batch, like every other stamp route.
        let rampa = self.rampa_da_aguada();
        let stamp = self.wet_shape_stamp();
        let groups = self.paint.dab_groups.clone(); // same group map ⇒ the two passes stay in lock-step
        let mut rng = super::tiling::DabRng::new(self.paint.tex_rng);
        let shape_img_owned = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let canvas = [fw as f32, fh as f32];
        // O `Pigment` mistura a cor nova com a tinta que a SESSÃO já tinha ([`super::watercolor_mistura`]).
        // Sobre a tinta molhada da sessão, o Smudge arrasta-a e mistura-a ([`Self::arrasto_da_sessao`])
        // e o Rewet redissolve-a na água deste traço ([`Self::agua_da_sessao`]).
        let arrasto = self.arrasto_da_sessao();
        let pigment = self.paint.brush.effective_pigment_mix();
        let pesos = (
            pigment.max(arrasto.unwrap_or(0.0)),
            self.agua_da_sessao().unwrap_or(0.0),
        );
        let mistura = pesos.0.max(pesos.1);
        let pigment = pigment > 0.0;
        let (spec, tiling) = (self.paint.brush, self.paint.tiling);
        let planos = &mut self.paint.wet_mistura;
        let (mut arrasto_de, mut passo) = (planos.arrasto, None);
        if mistura > 0.0 {
            planos.garante(fw * fh);
        }
        let buf = &mut self.paint.stroke_color;
        // O Rewet dissolve a tinta de antes do traço na vizinhança do LOTE, uma vez por lote
        // ([`super::watercolor_mistura_agua`]).
        let dissolucao = (pesos.1 > 0.0)
            .then(|| {
                let caixa = dabs
                    .iter()
                    .filter(|d| d.radius_px > 0.0)
                    .map(|d| {
                        let (cx, cy, r) = (d.center[0], d.center[1], d.radius_px);
                        let lo = |v: f32, n: usize| ((v - r).floor().max(0.0) as usize).min(n);
                        let hi = |v: f32, n: usize| ((v + r).ceil().max(0.0) as usize).min(n);
                        [lo(cx, fw), lo(cy, fh), hi(cx, fw), hi(cy, fh)]
                    })
                    .reduce(|a, b| {
                        [
                            a[0].min(b[0]),
                            a[1].min(b[1]),
                            a[2].max(b[2]),
                            a[3].max(b[3]),
                        ]
                    })?;
                let raio = super::watercolor_mistura_agua::raio_da_agua(&spec);
                Some(planos.dissolve(buf, (fw, fh), caixa, raio, pesos.1))
            })
            .flatten();
        for (di, (d, (dcol, prio, depl))) in dabs.iter().zip(&mixed).enumerate() {
            // Frame draw BEFORE any skip — mirror of the coverage pass (stream sync; see there).
            let rng = rng.enter(&groups, di);
            let frame = stamp.as_ref().map(|s| s.dab_frame(d, &mut *rng, canvas));
            let r = d.radius_px;
            let peak = WASH_DEPOSIT_PEAK; // NUNCA `d.coverage` — ver a const
            if r <= 0.0 || peak <= 0.0 {
                continue;
            }
            // O Smudge sobre a tinta molhada ([`super::watercolor_mistura_arrasto`]), ANTES do depósito
            // deste dab. O passo sai do dab ORIGINAL; as cópias de Tiling (no mesmo grupo) repetem-no.
            if let Some(forca) = arrasto {
                if groups.len() != dabs.len() || di == 0 || groups[di] != groups[di - 1] {
                    passo = arrasto_de.map(|p: [f32; 2]| [d.center[0] - p[0], d.center[1] - p[1]]);
                    arrasto_de = Some(d.center);
                }
                let keep = |i: usize| {
                    splat_keep(
                        sel.as_deref().map(Vec::as_slice),
                        prot.as_deref().map(Vec::as_slice),
                        alock.as_deref().map(Vec::as_slice),
                        i,
                    )
                };
                let guarda: Option<&dyn Fn(usize) -> f32> = if gated { Some(&keep) } else { None };
                planos.arrasta(
                    buf,
                    (fw, fh),
                    d.center,
                    passo,
                    &BrushSpec {
                        radius_px: r,
                        ..spec
                    },
                    forca,
                    tiling,
                    guarda,
                );
            }
            // Deposit PRIORITY (Wet Mix): a high-pickup dab writes at full alpha; a low-pickup one
            // (leaving a pool over bare ground) barely writes, so it can't overwrite the stronger
            // picked-up colour — the pool's exit edge stays as coloured as its entry (Enio 2026-07-07).
            // `prio == 1` when the mixer is off ⇒ byte-identical source-over.
            let prio = prio.clamp(0.0, 1.0);
            let depl = depl.clamp(0.0, 1.0);
            let col = [
                (dcol[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ];
            let inv_r = 1.0 / r;
            let (cx, cy) = (d.center[0], d.center[1]);
            let x0 = (cx - r).floor().max(0.0) as usize;
            let y0 = (cy - r).floor().max(0.0) as usize;
            let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
            let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
            for y in y0..y1 {
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let keep = if gated {
                        splat_keep(
                            sel.as_deref().map(Vec::as_slice),
                            prot.as_deref().map(Vec::as_slice),
                            alock.as_deref().map(Vec::as_slice),
                            base + x,
                        )
                    } else {
                        1.0
                    };
                    // source alpha = silhouette weight × deposit priority × the paint gates —
                    // the SAME weight as the coverage splat (manual Shape stamp or the feather),
                    // so the deposited colour and the silhouette always agree.
                    let wgt = match (&stamp, &frame) {
                        (Some(st), Some((fp, basis))) => {
                            st.sample(
                                WetShapeStamp::env_t(*fp, dn, dx, dy, inv_r),
                                basis.as_ref(),
                                shape_img_owned.as_ref(),
                                [x, y],
                                d.center,
                                r,
                            )
                            .0
                        }
                        _ => feather(dn),
                    };
                    // A Shape Color Ramp ([`RampaDaAguada`]): a cor do dab troca-se pela da rampa
                    // NESTE texel.
                    let (col, wgt) = match &rampa {
                        Some(RampaDaAguada { lut, alfa_pesa }) if wgt > 0.0 => {
                            let c = lut[(wgt.clamp(0.0, 1.0) * 255.0 + 0.5) as usize];
                            let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                            (
                                [byte(c[0]), byte(c[1]), byte(c[2])],
                                if *alfa_pesa { wgt * c[3] } else { wgt },
                            )
                        }
                        _ => (col, wgt),
                    };
                    let idx = (base + x) * 4;
                    if mistura > 0.0 {
                        // ⚠️ O `Pigment` deposita a tinta que a brocha LARGA (`depl`), SEM a prioridade
                        // do mixer (`prio = pickup × carga`): sobre tinta molhada da própria sessão o
                        // mixer não apanha nada (ele lê a base CONGELADA), a prioridade é zero, e com
                        // ela no peso o `Charge < 1` não gravava tinta nenhuma — a mistura ficava sem
                        // parceiro e o botão virava inerte AO BIT (report do dono, 2026-09-29; doc 44 §2).
                        // O Smudge SEM `Pigment` guarda a prioridade: ele não pediu mistura de tinta, e
                        // a poça do Wet Mix (dono, 2026-07-07) vale para ele como para o traço comum
                        // (`watercolor_color_change_junction_is_soft` reprovava sem isto). ⚠️ E o arrasto
                        // não fica escondido por ela: sobre a tinta molhada o mixer APANHA (lê a base do
                        // traço, que já traz a união da sessão), logo a prioridade ali não é zero —
                        // medido, recompor também os texels sem depósito não mudava um byte a Charge
                        // `1`, `0,5`, `0,326`, `0,2` nem `0`.
                        let prio_do_deposito = if pigment { 1.0 } else { prio };
                        let a = peak * wgt * prio_do_deposito * depl * keep;
                        if a > 0.0 {
                            let parceiro = dissolucao.as_ref().map(|dis| {
                                let de = if planos.capturado[base + x] {
                                    &planos.antes[idx..idx + 4]
                                } else {
                                    &buf[idx..idx + 4]
                                };
                                dis.parceiro([de[0], de[1], de[2], de[3]], x, y)
                            });
                            super::watercolor_mistura::deposita(
                                buf, planos, idx, col, a, pesos, parceiro,
                            );
                        }
                        continue;
                    }
                    let a = peak * wgt * prio * depl * keep;
                    if a <= 0.0 {
                        continue;
                    }
                    let da = f32::from(buf[idx + 3]) / 255.0;
                    let na = a + da * (1.0 - a); // straight-alpha over
                    if na <= 0.0 {
                        continue;
                    }
                    for c in 0..3 {
                        let dc = f32::from(buf[idx + c]) / 255.0;
                        let sc = f32::from(col[c]) / 255.0;
                        let out = (sc * a + dc * da * (1.0 - a)) / na;
                        buf[idx + c] = (out * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                    }
                    buf[idx + 3] = (na * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                }
            }
        }
        // Manual stamp: advance the stroke's texture-rng stream (once per batch — the coverage pass
        // replayed the same seed without writing back). Automatic ⇒ `rng` untouched, write-back is a
        // no-op (the historical stream stays byte-identical).
        self.paint.tex_rng = rng.finish();
        self.paint.wet_mistura.arrasto = arrasto_de;
    }
}

/// **A Shape Color Ramp na aguada, no modo COR** (B&W desligado) — a mesma lei do traço digital
/// (`stamp_ramped`): o peso do carimbo (a silhueta com o falloff, o `wgt` do splat) indexa a rampa, e
/// a rampa é a DONA da cor — cada texel deposita `lut[wgt]`. No `Alpha Mode` Strength (e no Texture
/// Alpha, que numa aguada não tem alfa de camada a baixar) o alfa da rampa pesa o depósito. O modo
/// TOM (B&W) não mora aqui: remapeia a silhueta da Shape dentro do `WetShapeStamp`, como no digital.
pub(super) struct RampaDaAguada {
    lut: Vec<[f32; 4]>,
    alfa_pesa: bool,
}

impl PainterTool {
    /// A rampa da Shape no modo cor que este lote lê — `None` com ela desligada ou no modo B&W (o
    /// caminho de sempre, ao byte).
    pub(super) fn rampa_da_aguada(&mut self) -> Option<RampaDaAguada> {
        if !self.paint.shape_color_ramp_enabled {
            return None;
        }
        if self.paint.shape_color_ramp_bw {
            self.ensure_shape_ramp_lut(); // o tom do `wet_shape_stamp` lê-a
            return None;
        }
        let dona = super::ramp_lut::RampLutOwner::Shape;
        self.ensure_ramp_lut(dona);
        Some(RampaDaAguada {
            lut: self.paint.texture_ramp_lut.clone(),
            alfa_pesa: !matches!(
                self.active_ramp_alpha_mode(dona),
                ph2d_painter_brush::RampAlphaMode::None
            ),
        })
    }
}
