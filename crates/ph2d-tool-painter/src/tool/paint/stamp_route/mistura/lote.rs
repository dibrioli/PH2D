//! O LOTE da mistura opaca (`stamp_dabs_mistura_opaca`): a pegada de cada dab e a película em ordem — a lei do
//! módulo [`super`].

use super::{Region, pegada_com_forma, pegada_do_dab, pelicula};
use crate::tool::PainterTool;
use ph2d_painter_brush::mistura_opaca::{Carimbo, Pegada, ReservatorioOpaco};
use ph2d_painter_brush::{BrushSpec, Dab};
use rayon::prelude::*;

impl PainterTool {
    pub(in crate::tool::paint::stamp_route) fn stamp_dabs_mistura_opaca(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let n4 = (w as usize) * (h as usize) * 4;
        let caixa = super::super::super::region::dabs_bounds(dabs, w, h, brush.tip_reach());
        self.prepara_o_po(brush, w, h, caixa);
        if self.mistura_meia_arma(brush, alpha_locked, n4) {
            self.stamp_dabs_mistura_meia(dabs, brush, w, h);
            return;
        }
        let copias = brush.symmetry.copies().max(1);
        let pressao = self.paint.mistura_opaca.pressao;
        let groups = self.paint.dab_groups.clone();
        let mut res = std::mem::take(&mut self.paint.mistura_opaca.res);
        res.resize(copias, ReservatorioOpaco::default());
        // No plano do traço a tela de antes é a base dele; noutro plano (uma camada do Composite) a do undo.
        let base = if self.paint.acumulando_no_plano {
            self.paint
                .mistura_opaca
                .base_do_plano
                .clone()
                .or_else(|| self.tela_de_antes_do_traco(false))
                .filter(|b| b.len() == n4)
        } else {
            None
        };
        // «Sample: All visible layers» (#79): as camadas visíveis compostas no 1.º lote do traço, com a ativa como
        // estava antes dele. Uma camada só = a própria (nada a compor).
        let visivel = self.mistura_amostra_visivel(brush, base.as_ref(), n4);
        let mut ladrilhos = std::mem::take(&mut self.paint.mistura_opaca.antes);
        let mut sombra = std::mem::take(&mut self.paint.mistura_opaca.sombra);
        // A pegada real (Shape, Grain); o stream de RNG anda por dab como na rota normal.
        self.ensure_shape_ramp_lut();
        let caixa = super::super::super::region::dabs_bounds(dabs, w, h, brush.tip_reach());
        self.prepara_o_po(brush, w, h, caixa);
        let papel = self.papel_como_grain();
        let shape_tom = self.shape_tom_ativo();
        let shape_active = brush.shape_silhouette_active(self.paint.shape_image.is_some());
        let textured = brush.texture.is_active();
        // A COR da pegada (Bug #77): as rotas que dão cor por texel — a ponta de píxel, o Shape de várias camadas,
        // a rampa do Grain ou do Shape — pintam-na na máscara, e a mistura lê o RGB como a cor do pincel. A
        // porta do despacho (B25.6, [`super::super::rota`]): a rota que a pegada tomaria fora da mistura.
        use super::super::rota::RotaDeCor;
        let rota = self.rota_da_pegada(dabs, brush);
        let pixel = rota == RotaDeCor::Pixel;
        let por_camada = matches!(
            rota,
            RotaDeCor::PorCamadaDinamica | RotaDeCor::PorCamadaRgba | RotaDeCor::PorCamadaCor
        );
        let rampa_px = pixel
            && self.paint.shape_color_ramp_enabled
            && !self.shape_tom_ativo()
            && self.paint.shape_camadas.alguma(false);
        let dono = if matches!(
            rota,
            RotaDeCor::RampaEmCache | RotaDeCor::RampaCorEmCache | RotaDeCor::RampaPorPixel
        ) {
            self.color_ramp_owner(textured)
        } else {
            super::super::super::ramp_lut::RampLutOwner::None
        };
        if rampa_px {
            self.ensure_ramp_lut(super::super::super::ramp_lut::RampLutOwner::Shape);
        } else if dono != super::super::super::ramp_lut::RampLutOwner::None {
            self.ensure_ramp_lut(dono);
        }
        let grain_ramp_camadas = por_camada && self.ensure_per_layer_grain_ramp(brush);
        let com_cor =
            rampa_px || por_camada || dono != super::super::super::ramp_lut::RampLutOwner::None;
        // Re-empresta o que o `ensure_*` (`&mut self`) acabou de preparar.
        let image = if papel {
            &self.paint.paper_image
        } else {
            &self.paint.texture_image
        }
        .as_ref()
        .map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = shape_tom.then_some(self.paint.shape_ramp_lut.as_slice());
        let po = super::super::super::po_do_papel::po(self.paint.po_do_papel.as_ref(), w);
        let rampa = if rampa_px {
            Some(ph2d_painter_brush::RampPaint {
                lut: &self.paint.texture_ramp_lut,
                alpha_mode: super::super::super::ramp_lut::BRUSH_RAMP_ALPHA,
                index: super::super::super::ramp_lut::RampLutOwner::Shape.index(),
                shape: self.paint.shape_camadas.motor(false),
                shift: [0.0; 3],
            })
        } else if dono != super::super::super::ramp_lut::RampLutOwner::None {
            Some(ph2d_painter_brush::RampPaint {
                lut: &self.paint.texture_ramp_lut,
                alpha_mode: super::super::super::ramp_lut::BRUSH_RAMP_ALPHA,
                index: dono.index(),
                shape: if dono == super::super::super::ramp_lut::RampLutOwner::Shape {
                    self.paint.shape_camadas.motor(shape_active)
                } else {
                    ph2d_painter_brush::ShapeRampLayers::default()
                },
                shift: [0.0; 3],
            })
        } else {
            None
        };
        let camadas_do_shape = por_camada.then(|| {
            let sl = &self.paint.shape_layers;
            let n = sl.len();
            let blends: Vec<u8> = (0..n).map(|i| sl.effective_blend(i)).collect();
            let any_blend = blends.iter().any(|&b| b != 0);
            (
                sl.masks(),
                sl.resolved_colors(brush.color),
                (0..n).map(|i| sl.rgb_image(i)).collect::<Vec<_>>(),
                blends,
                sl.opacities(),
                any_blend,
            )
        });
        let grain_ramp = grain_ramp_camadas.then_some(self.paint.texture_ramp_lut.as_slice());
        let mut mapas = std::mem::take(&mut self.paint.mistura_opaca.camadas);
        if let Some((masks, ..)) = &camadas_do_shape {
            mapas.resize_with(masks.len(), Vec::new);
            for m in &mut mapas {
                if m.len() != n4 {
                    m.clear();
                    m.resize(n4, 0);
                }
            }
        }
        let mut dab_rng = super::super::super::tiling::DabRng::new(self.paint.tex_rng);
        let buf = super::super::super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            w,
            super::super::super::region::dabs_bounds(dabs, w, h, brush.tip_reach()),
        );
        // O véu sai em todo ponto do caminho a 4,5× o raio, e a cópia do spray no mesmo centro: ela salta-se. Com o
        // spray parado (Space) o pincel parado acumula: aí não.
        let afina = self.paint.mistura_opaca.salta_a_copia_do_veu
            && brush.parado_s <= 0.0
            && brush.dab_jitter.veu > 0.0
            && brush.spray_count > 1
            && brush.stroke_method.allows_jitter();
        // ⭐ Só o Grain (o Wet Wash Mix, Bug #84): a pegada de um dab não lê a tela, só a película lê — as pegadas do
        // lote calculam-se de uma vez, em paralelo ENTRE os dabs, cada uma num buffer da caixa dela; a película corre
        // depois, em ordem. As bases do Grain sorteiam-se antes, na ordem do lote (o mesmo stream de RNG). A cópia que
        // cai no centro da anterior do grupo salta-se quase sempre: a pegada dela só se calcula se não saltar.
        // Sem o Grain (o Hard Round, missão 4K B28.3) o mesmo: a pegada sem base não consome o stream.
        let juntas = textured
            || ph2d_painter_brush::mistura_opaca::PEGADAS_LOCAIS_SEM_GRAO
                .load(std::sync::atomic::Ordering::Relaxed);
        // Com o Shape (o Oil Paint, o Gouache: as cerdas) também, missão 4K onda 3.
        let com_forma = ph2d_painter_brush::mistura_opaca::PEGADAS_LOCAIS_COM_FORMA
            .load(std::sync::atomic::Ordering::Relaxed);
        let locais = !pixel
            && camadas_do_shape.is_none()
            && rampa.is_none()
            && (!shape_active || com_forma)
            && (juntas || shape_active);
        // As máscaras da TELA inteira (33 MB cada a 4K) só servem à rota em série; a das pegadas locais solta-as.
        let mut mascara = std::mem::take(&mut self.paint.mistura_opaca.pegada);
        let mut liso = std::mem::take(&mut self.paint.mistura_opaca.pegada_lisa);
        if locais {
            mascara = Vec::new();
            liso = Vec::new();
        } else {
            if mascara.len() != n4 {
                mascara.clear();
                mascara.resize(n4, 0);
            }
            if (textured || shape_active) && liso.len() != n4 {
                liso.clear();
                liso.resize(n4, 0);
            }
        }
        let pega = |(spec, basis, forma): &(
            BrushSpec,
            Option<ph2d_painter_brush::texture::TexDabBasis>,
            Option<ph2d_painter_brush::texture::TexDabBasis>,
        ),
                    d: &Dab| match forma {
            Some(sb) => pegada_com_forma(
                spec,
                basis.as_ref(),
                ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                },
                d,
                w,
                h,
                image.as_ref(),
                po,
            ),
            None => pegada_do_dab(spec, basis.as_ref(), d, w, h, image.as_ref(), po),
        };
        let mut pegadas: Option<(Vec<_>, Vec<_>)> = locais.then(|| {
            let bases: Vec<_> = dabs
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let spec = brush.at_dab(d);
                    let rng = dab_rng.enter(&groups, i);
                    let fp = spec.dab_footprint_of(d);
                    // A ordem do stream é a da rota em série: o Shape, depois o Grain.
                    let forma = shape_active.then(|| {
                        ph2d_painter_brush::texture::shape_basis(
                            &spec.shape,
                            &mut *rng,
                            [w as f32, h as f32],
                            fp,
                            ph2d_painter_brush::texture::ShapeFrame::Stroke {
                                arc_len: d.arc_len,
                                unit_px: d.stroke_radius_px,
                            },
                        )
                    });
                    let basis = textured.then(|| {
                        ph2d_painter_brush::texture::dab_basis(
                            &spec.texture,
                            rng,
                            [w as f32, h as f32],
                            fp,
                        )
                    });
                    (spec, basis, forma)
                })
                .collect();
            let pegadas = (0..dabs.len())
                .into_par_iter()
                .map(|i| {
                    let copia = afina
                        && i > 0
                        && dabs[i - 1].center == dabs[i].center
                        && groups.get(i - 1) == groups.get(i);
                    (!copia).then(|| pega(&bases[i], &dabs[i]))
                })
                .collect();
            (bases, pegadas)
        });
        let mut touched: Option<Region> = None;
        let mut grupo_ant: Option<(usize, ReservatorioOpaco)> = None;
        let mut salta_o_grupo = None;
        for (i, d) in dabs.iter().enumerate() {
            let g = groups.get(i).map_or(i, |&g| g as usize);
            let k = g % copias;
            // Cópia do Tiling: parte do estado do grupo e não o avança.
            let mut copia_do_grupo;
            let alvo: &mut ReservatorioOpaco = match grupo_ant {
                Some((ga, r)) if ga == g => {
                    copia_do_grupo = r;
                    &mut copia_do_grupo
                }
                _ => {
                    grupo_ant = Some((g, res[k]));
                    salta_o_grupo = None;
                    &mut res[k]
                }
            };
            let spec = brush.at_dab(d);
            // As cópias do Tiling seguem a decisão da primeira (a costura casa).
            let salta =
                afina && *salta_o_grupo.get_or_insert_with(|| alvo.repete_o_centro(d.center));
            if let Some((bases, pre)) = pegadas.as_mut() {
                if salta {
                    continue;
                }
                let pre = pre[i].take().unwrap_or_else(|| pega(&bases[i], d));
                let Some((local, Some(r), r2)) = pre else {
                    continue;
                };
                let caixa = (r.x, r.y, r.x + r.w, r.y + r.h);
                debug_assert!(
                    r2.is_none_or(|r2| r2 == r),
                    "a pegada lisa tem a caixa do kernel"
                );
                let carimbo = Carimbo {
                    center: d.center,
                    radius_px: d.radius_px,
                    cor: d.color,
                    coverage: d.coverage,
                    pressao,
                };
                let pegada = Pegada::Mascara {
                    mascara: &local.mascara,
                    caixa,
                    org: [local.caixa.x, local.caixa.y],
                    largura: local.caixa.w,
                    sem_grao: r2.map(|_| local.liso.as_slice()),
                    com_cor,
                };
                let r = pelicula(
                    &mut *buf,
                    &mut sombra,
                    &mut ladrilhos,
                    w,
                    h,
                    alpha_locked,
                    base.as_deref().map(Vec::as_slice),
                    visivel.as_deref().map(Vec::as_slice),
                    &spec,
                    carimbo,
                    pegada,
                    alvo,
                    caixa,
                );
                if let Some(r) = r {
                    let r = Region {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                    };
                    touched = Some(touched.map_or(r, |t| super::super::super::union_region(t, r)));
                }
                continue;
            }
            let tex_rng = dab_rng.enter(&groups, i);
            // A pegada: o kernel da rota normal (`stamp_dabs_per_pixel`), a branco, força 1, Flow 1, sobre o vazio.
            let fp = spec.dab_footprint_of(d);
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::shape_basis(
                    &spec.shape,
                    &mut *tex_rng,
                    [w as f32, h as f32],
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                )
            });
            let shape_in = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    &mut *tex_rng,
                    [w as f32, h as f32],
                    fp,
                )
            });
            let branco = BrushSpec {
                color: [1.0; 3],
                flow: 1.0,
                blend: ph2d_painter_brush::BrushBlend::default(),
                ..spec
            };
            // O stream de RNG já andou (as bases acima): saltar não muda a pegada dos seguintes.
            if salta {
                continue;
            }
            // Só o Grain (sem Shape, sem rampa, sem camadas): a pegada lisa sai na MESMA visita do kernel.
            let mut liso_junto = None;
            let r = if pixel {
                // A ponta de píxel: a pegada binária na grelha (a textura não a carimba; a rampa pela queda sim).
                let d1 = Dab {
                    coverage: 1.0,
                    ..*d
                };
                let rampa = rampa.map(|r| ph2d_painter_brush::RampPaint {
                    shift: d.color_shift,
                    ..r
                });
                ph2d_painter_brush::pixel::carimba(
                    &mut mascara,
                    w,
                    h,
                    &d1,
                    &BrushSpec {
                        color: d.color,
                        ..branco
                    },
                    false,
                    None,
                    rampa.as_ref(),
                )
            } else if let (Some((masks, cores, rgb, blends, opac, any_blend)), Some(sb)) =
                (&camadas_do_shape, &shape_basis)
            {
                let colors = cores
                    .iter()
                    .zip(rgb)
                    .map(|(&c, rgb)| {
                        if rgb.is_some() {
                            c
                        } else {
                            ph2d_painter_brush::apply_color_shift(c, d.color_shift)
                        }
                    })
                    .collect();
                let um = [ph2d_painter_brush::DynDab {
                    center: d.center,
                    radius: d.radius_px,
                    spec: BrushSpec {
                        color: d.color,
                        ..branco
                    },
                    shape_basis: *sb,
                    grain_basis: basis,
                    colors,
                    color_shift: d.color_shift,
                    coverage: 1.0,
                }];
                let r = ph2d_painter_brush::accumulate_shape_layers_rgba_batch(
                    &mut mapas,
                    w,
                    h,
                    &um,
                    masks,
                    rgb,
                    image.as_ref(),
                    grain_ramp,
                );
                if let Some(r) = r {
                    // As camadas compostas num RGBA recto na máscara; os mapas voltam a ser vazios.
                    let n = masks.len();
                    for y in r.y..r.y + r.h {
                        for x in r.x..r.x + r.w {
                            let i = ((y * w + x) * 4) as usize;
                            let s = super::super::super::stamp_color_dynamic::camadas_num_texel(
                                |k| {
                                    let m = &mapas[k];
                                    [m[i], m[i + 1], m[i + 2], m[i + 3]]
                                },
                                n,
                                blends,
                                opac,
                                *any_blend,
                            );
                            for (dst, v) in mascara[i..i + 4].iter_mut().zip(s) {
                                *dst = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                            }
                            for m in &mut mapas {
                                m[i..i + 4].fill(0);
                            }
                        }
                    }
                }
                r
            } else if let Some(rampa) = rampa {
                // A rampa lê a cor do carimbo (`stamp_dabs_ramped`): a pegada leva-a, não o branco.
                ph2d_painter_brush::stamp_dab_ramped(
                    &mut mascara,
                    w,
                    h,
                    d.center,
                    &BrushSpec {
                        color: d.color,
                        ..branco
                    },
                    1.0,
                    false,
                    basis.as_ref(),
                    image.as_ref(),
                    shape_in,
                    ph2d_painter_brush::RampPaint {
                        shift: d.color_shift,
                        ..rampa
                    },
                    None,
                    spec.dab_rotor(d),
                    d.capsula(&branco),
                    po,
                )
            } else if shape_in.is_none() && basis.is_some() {
                let (r, r2) = ph2d_painter_brush::stamp_dab_textured_masked_po_com_liso(
                    &mut mascara,
                    &mut liso,
                    w,
                    h,
                    d.center,
                    &branco,
                    1.0,
                    false,
                    basis.as_ref(),
                    image.as_ref(),
                    spec.dab_rotor(d),
                    d.capsula(&branco),
                    po,
                );
                liso_junto = Some(r2);
                r
            } else {
                ph2d_painter_brush::stamp_dab_textured_masked_po(
                    &mut mascara,
                    w,
                    h,
                    d.center,
                    &branco,
                    1.0,
                    false,
                    basis.as_ref(),
                    image.as_ref(),
                    shape_in,
                    None,
                    None,
                    spec.dab_rotor(d),
                    d.capsula(&branco),
                    po,
                )
            };
            let Some(r) = r else {
                // O grão comeu o dab inteiro e a pegada lisa já saiu: ela volta a ser vazia.
                if let Some(Some(r2)) = liso_junto {
                    for y in r2.y..r2.y + r2.h {
                        let a = ((y * w + r2.x) * 4) as usize;
                        liso[a..a + (r2.w * 4) as usize].fill(0);
                    }
                }
                continue;
            };
            let mut caixa = (r.x, r.y, r.x + r.w, r.y + r.h);
            // Com o Grain ou a Shape, a mesma pegada sem eles (o grão e a silhueta vão ao tecto, como no Off —
            // `Pegada::Mascara::sem_grao`; na taxa as cerdas saíam ralas e translúcidas, lote A 08/11).
            let com_grao = !pixel && (basis.is_some() || shape_in.is_some());
            let r2 = match liso_junto {
                Some(r2) => r2,
                None if com_grao => ph2d_painter_brush::stamp_dab_textured_masked(
                    &mut liso,
                    w,
                    h,
                    d.center,
                    &BrushSpec {
                        texture: Default::default(),
                        dente: None, // o dente do papel morde como o grão: fora da pegada lisa
                        ..branco
                    },
                    1.0,
                    false,
                    None,
                    None,
                    None,
                    None,
                    None,
                    spec.dab_rotor(d),
                    d.capsula(&branco),
                ),
                None => None,
            };
            if let Some(r2) = r2 {
                caixa = (
                    caixa.0.min(r2.x),
                    caixa.1.min(r2.y),
                    caixa.2.max(r2.x + r2.w),
                    caixa.3.max(r2.y + r2.h),
                );
            }
            let carimbo = Carimbo {
                center: d.center,
                radius_px: d.radius_px,
                cor: d.color,
                coverage: d.coverage,
                pressao,
            };
            let pegada = Pegada::Mascara {
                mascara: &mascara,
                caixa,
                org: [0, 0],
                largura: w,
                sem_grao: com_grao.then_some(liso.as_slice()),
                com_cor,
            };
            let r = pelicula(
                &mut *buf,
                &mut sombra,
                &mut ladrilhos,
                w,
                h,
                alpha_locked,
                base.as_deref().map(Vec::as_slice),
                visivel.as_deref().map(Vec::as_slice),
                &spec,
                carimbo,
                pegada,
                alvo,
                caixa,
            );
            // A máscara volta a ser vazia (só a caixa que o kernel pintou).
            let (x0, y0, x1, y1) = caixa;
            for y in y0..y1 {
                let a = ((y * w + x0) * 4) as usize;
                mascara[a..a + ((x1 - x0) * 4) as usize].fill(0);
                if com_grao {
                    liso[a..a + ((x1 - x0) * 4) as usize].fill(0);
                }
            }
            let Some(r) = r else {
                continue;
            };
            let r = Region {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            };
            touched = Some(touched.map_or(r, |t| super::super::super::union_region(t, r)));
        }
        self.paint.mistura_opaca.res = res;
        self.paint.mistura_opaca.antes = ladrilhos;
        self.paint.mistura_opaca.sombra = sombra;
        self.paint.mistura_opaca.pegada = mascara;
        self.paint.mistura_opaca.pegada_lisa = liso;
        self.paint.mistura_opaca.camadas = mapas;
        self.paint.tex_rng = dab_rng.finish();
        self.declare_wrote(touched);
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }
}
