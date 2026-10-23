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
        // As Grain Colors: o mapa de granulação indexa a rampa ([`GraoDaAguada`]).
        let grao = self.grao_da_aguada(rampa.as_ref());
        let grao_img = self.paint.texture_image.as_ref().map(|i| i.as_mask());
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
        // **O buffer é o ÚNICO dono da cor** quando o `Pigment` mistura a tinta nova com a da sessão ou
        // quando a Shape Color Ramp troca a cor do dab pela das camadas: o composto, com o buffer vazio,
        // cai na cor do PINCEL — o que só é inofensivo quando a cor do dab É a do pincel. Aí a
        // prioridade do mixer (`pickup × carga`, `0` sobre o papel em branco) não pode pesar o depósito,
        // senão a `Charge < 1` apaga a tinta (doc 44 §2) ou as cores da rampa (lista 26 A8). A `depl`
        // (a reserva da brocha) pesa sempre.
        let buffer_dono_da_cor = pigment > 0.0 || rampa.is_some();
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
        for (di, (d, (dcol, prio, depl, apanha))) in dabs.iter().zip(&mixed).enumerate() {
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
            // `prio == 1` when the mixer is off ⇒ byte-identical source-over. With the buffer as the
            // colour's sole owner (`buffer_dono_da_cor`) the priority is `1` in BOTH branches below.
            let prio = if buffer_dono_da_cor {
                1.0
            } else {
                prio.clamp(0.0, 1.0)
            };
            let depl = depl.clamp(0.0, 1.0);
            let col = [
                (dcol[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ];
            let inv_r = 1.0 / r;
            // The dab-CAPSULE (Continuous): its sub-dabs back to the previous centre, oldest first — the
            // dense stroke's own passes over each pixel (the wash is a MAX union, the colour source-over).
            // Only the centre moves here: the wash is a frozen medium (lista 26 C3), not the A11 law.
            let capsula = d.capsula(&self.paint.brush);
            for k in (0..capsula.map_or(1, |c| c.n())).rev() {
                let o = capsula.map_or([0.0, 0.0], |c| c.off(k));
                let (cx, cy) = (d.center[0] + o[0], d.center[1] + o[1]);
                let centro = [cx, cy];
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
                        // `forma` = o valor da textura do Shape (a densidade), quando há um Shape.
                        let (wgt, forma) = match (&stamp, &frame) {
                            (Some(st), Some((fp, basis))) => {
                                let (w, d) = st.sample(
                                    WetShapeStamp::env_t(*fp, dn, dx, dy, inv_r),
                                    basis.as_ref(),
                                    shape_img_owned.as_ref(),
                                    [x, y],
                                    centro,
                                    r,
                                );
                                (w, st.forma_ativa().then_some(d))
                            }
                            _ => (feather(dn), None),
                        };
                        // A Shape Color Ramp ([`RampaDaAguada`]): a cor do PINCEL troca-se pela das
                        // camadas NESTE texel — a Texture Color pelo valor do Shape, a Falloff Color pelo
                        // peso — e é ESSA que se mistura com a tinta que a brocha apanhou (a `Charge`),
                        // como a do pincel no mixer: uma lei ([`super::watercolor_mixer::Apanha`]).
                        let (col, wgt) = match &rampa {
                            Some(r) if wgt > 0.0 => {
                                let g = grao.as_ref().map(|g| g.em(grao_img.as_ref(), x, y));
                                let c = r.cor_do_dab(forma, g, wgt, d.color_shift);
                                let rgb = [c[0], c[1], c[2]];
                                let rgb = apanha.map_or(rgb, |a| a.mistura(rgb));
                                let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                                ([byte(rgb[0]), byte(rgb[1]), byte(rgb[2])], wgt * c[3])
                            }
                            _ => (col, wgt),
                        };
                        let idx = (base + x) * 4;
                        if mistura > 0.0 {
                            // ⚠️ Com o `Pigment` o depósito é a tinta que a brocha LARGA (`depl`), SEM a
                            // prioridade do mixer — a lei `buffer_dono_da_cor` acima (report do dono,
                            // 2026-09-29; doc 44 §2). O Smudge SEM `Pigment` nem rampa guarda a prioridade:
                            // ele não pediu mistura de tinta, e a poça do Wet Mix (dono, 2026-07-07) vale
                            // para ele como para o traço comum (`watercolor_color_change_junction_is_soft`
                            // reprovava sem isto). ⚠️ E o arrasto não fica escondido por ela: sobre a tinta
                            // molhada o mixer APANHA (lê a base do traço, que já traz a união da sessão),
                            // logo a prioridade ali não é zero — medido, recompor também os texels sem
                            // depósito não mudava um byte a Charge `1`, `0,5`, `0,326`, `0,2` nem `0`.
                            let a = peak * wgt * prio * depl * keep;
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
        }
        // Manual stamp: advance the stroke's texture-rng stream (once per batch — the coverage pass
        // replayed the same seed without writing back). Automatic ⇒ `rng` untouched, write-back is a
        // no-op (the historical stream stays byte-identical).
        self.paint.tex_rng = rng.finish();
        self.paint.wet_mistura.arrasto = arrasto_de;
    }
}

/// **A rampa de cor na aguada e no Wet Paint** — a MESMA lei do traço digital
/// ([`ph2d_painter_brush::RampPaint::cor`]), com a dona que o motor escolhe ([`RampLutOwner`]):
/// - a **Shape Color** (B&W desligado, prompt 38): a Texture Color pelo valor da textura do Shape (a
///   densidade do carimbo), a Falloff Color pelo peso do carimbo (o `wgt` do splat). O padrão procedural
///   recolhido pela Texture Color deixa de ser a densidade ([`WetShapeStamp::densidade`]);
/// - as **Grain Colors**: a cor pelo VALOR do grão no texel — na aguada o mapa de granulação (o Grain é
///   esse mapa, [`GraoDaAguada`], e continua a assentar a granulação), no Wet Paint o grão do dab — que,
///   como no digital, deixa de morder a cobertura quando é o dono da cor.
///
/// O alfa da tinta pesa o depósito, sempre — a lei única das rampas do pincel. O modo TOM (B&W) da
/// Shape não mora aqui: remapeia a silhueta dentro do `WetShapeStamp`, como no digital.
pub(super) struct RampaDaAguada {
    lut: Vec<[f32; 4]>,
    camadas: super::shape_camadas::CamadasDoShape,
    dona: super::ramp_lut::RampLutOwner,
}

impl RampaDaAguada {
    /// A rampa como os núcleos do digital a leem, com o desvio `shift` do carimbo.
    #[inline]
    fn pintura(&self, forma: bool, shift: [f32; 3]) -> ph2d_painter_brush::RampPaint<'_> {
        ph2d_painter_brush::RampPaint {
            lut: &self.lut,
            alpha_mode: super::ramp_lut::BRUSH_RAMP_ALPHA,
            index: self.dona.index(),
            shape: if self.dona == super::ramp_lut::RampLutOwner::Shape {
                self.camadas.motor(forma)
            } else {
                ph2d_painter_brush::ShapeRampLayers::default()
            },
            shift,
        }
    }

    /// A cor num texel (sRGB reto + alfa da tinta): `forma` = o valor do Shape, `grao` = o do Grain,
    /// `wgt` = o peso do carimbo — a MESMA conta nas duas passadas (cor e densidade).
    #[inline]
    pub(super) fn cor(&self, forma: Option<f32>, grao: Option<f32>, wgt: f32) -> [f32; 4] {
        self.pintura(forma.is_some(), [0.0; 3])
            .cor(wgt, forma, grao)
    }

    /// [`Self::cor`] com o Randomize Color do carimbo (`shift` = [`ph2d_painter_brush::Dab::color_shift`])
    /// — a lei do digital ([`ph2d_painter_brush::RampPaint::shift`]): o RGB desloca-se, o alfa fica.
    #[inline]
    pub(super) fn cor_do_dab(
        &self,
        forma: Option<f32>,
        grao: Option<f32>,
        wgt: f32,
        shift: [f32; 3],
    ) -> [f32; 4] {
        self.pintura(forma.is_some(), shift).cor(wgt, forma, grao)
    }

    /// As Grain Colors são as donas da cor: o valor do grão indexa a rampa (e o grão deixa de morder).
    #[inline]
    pub(super) fn do_grao(&self) -> bool {
        self.dona == super::ramp_lut::RampLutOwner::Grain
    }
}

/// **O mapa de granulação da aguada como o índice das Grain Colors** — o MESMO texel que a lavagem
/// assenta (`watercolor_rewet_px::SubstrateSession::at`, o `gran_h` do mapa próprio): a escala do pincel
/// ([`super::watercolor_noise::grain_view_to_canvas_size`]), o encaixe do Tiling e o ângulo.
pub(super) struct GraoDaAguada {
    tex: ph2d_painter_brush::TextureSettings,
    rot: [f32; 2],
    periodo: [f32; 2],
}

impl GraoDaAguada {
    /// O valor do grão no texel `(x, y)` do canvas.
    #[inline]
    pub(super) fn em(
        &self,
        img: Option<&ph2d_painter_brush::texture::ImageMask<'_>>,
        x: usize,
        y: usize,
    ) -> f32 {
        ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
            &self.tex,
            x as i64,
            y as i64,
            img,
            self.rot,
            self.periodo,
        )
    }
}

impl PainterTool {
    /// O Grain que o meio da aguada LÊ (e que por isso pode ser o dono da cor): na aquarela o mapa de
    /// granulação próprio (`Same as Paper` desligado — a lavagem não lê o slot com ele ligado); no Wet
    /// Paint o grão que amostra alguma coisa (`grain_samples`, a porta do depósito).
    pub(super) fn grao_que_a_aguada_le(&self) -> bool {
        let b = &self.paint.brush;
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            b.grain_samples(self.paint.texture_image.is_some())
        } else {
            !b.granulation_use_paper && b.texture.is_active()
        }
    }

    /// A aguada tem uma rampa em COR — a Shape Color ou as Grain Colors são as donas da cor do depósito
    /// (o predicado do [`Self::rampa_da_aguada`] e da marca `WetStrokeStyle::deposito_dono`).
    pub(super) fn aguada_tem_rampa_de_cor(&self) -> bool {
        self.color_ramp_owner(self.grao_que_a_aguada_le()) != super::ramp_lut::RampLutOwner::None
    }

    /// As Grain Colors da AQUARELA são as donas da cor: o mapa de granulação indexa a rampa
    /// ([`GraoDaAguada`]) — o predicado de [`Self::grao_da_aguada`] e da marca
    /// `WetStrokeStyle::cor_no_papel` (no Wet Paint o grão é o do dab, que não passa pelo composite).
    pub(super) fn aguada_tem_cor_do_grao(&self) -> bool {
        !matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && self.color_ramp_owner(self.grao_que_a_aguada_le())
                == super::ramp_lut::RampLutOwner::Grain
    }

    /// ⭐ As [`super::watercolor_field::MarcasDaAguada`] do estado EFECTIVO — a porta única do pen-down
    /// (o estilo do dono) e do composite (os texels sem dono). A ponta é texturada quando a silhueta
    /// da Shape manual está activa ou uma rampa em cor pesa a densidade — o mesmo predicado que arma
    /// o plano `stroke_density` no splat.
    pub(super) fn marcas_da_aguada(&self) -> super::watercolor_field::MarcasDaAguada {
        let deposito_dono = self.aguada_tem_rampa_de_cor();
        super::watercolor_field::MarcasDaAguada {
            deposito_dono,
            cor_no_papel: self.aguada_tem_cor_do_grao(),
            ponta_texturada: deposito_dono
                || self.wet_shape_stamp().is_some_and(|s| s.forma_ativa()),
        }
    }

    /// A rampa em cor que este lote lê — `None` com as duas desligadas ou a da Shape no modo B&W (o
    /// caminho de sempre, ao byte).
    pub(super) fn rampa_da_aguada(&mut self) -> Option<RampaDaAguada> {
        if self.paint.shape_color_ramp_enabled && self.paint.shape_color_ramp_bw {
            self.ensure_shape_ramp_lut(); // o tom do `wet_shape_stamp` lê-a
        }
        let dona = self.color_ramp_owner(self.grao_que_a_aguada_le());
        if dona == super::ramp_lut::RampLutOwner::None {
            return None;
        }
        self.ensure_ramp_lut(dona);
        Some(RampaDaAguada {
            lut: self.paint.texture_ramp_lut.clone(),
            camadas: self.paint.shape_camadas.clone(),
            dona,
        })
    }

    /// O mapa de granulação que indexa as Grain Colors da aquarela — `None` quando elas não são as
    /// donas da cor (ou no Wet Paint, onde o grão é o do dab).
    pub(super) fn grao_da_aguada(&self, rampa: Option<&RampaDaAguada>) -> Option<GraoDaAguada> {
        if !(rampa.is_some_and(RampaDaAguada::do_grao) && self.aguada_tem_cor_do_grao()) {
            return None;
        }
        let (fw, fh) = self.source_size;
        let tile = if self.paint.tiling[0] || self.paint.tiling[1] {
            super::watercolor_noise::NoiseTile::new((fw as usize, fh as usize), self.paint.tiling)
        } else {
            super::watercolor_noise::NoiseTile::NONE
        };
        let b = &self.paint.brush;
        let tex = super::watercolor_noise::snap_slot_size(
            super::watercolor_noise::grain_view_to_canvas_size(b.texture, b.radius_px),
            tile,
        );
        Some(GraoDaAguada {
            rot: ph2d_painter_brush::texture::angle_basis(tex.angle_deg),
            tex,
            periodo: tile.slot_period(),
        })
    }
}
