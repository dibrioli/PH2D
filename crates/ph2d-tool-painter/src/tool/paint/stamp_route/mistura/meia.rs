//! ⭐ **A mistura a MEIA definição** (Bug #84, dono 10/11 «vamos tentar»; [`MisturaOpaca::meia_definicao`], só a receita
//! do Wet Wash Mix a liga): no plano do traço a pegada e a película correm numa tela de metade do lado (¼ dos texels) —
//! o carimbo com centro, raio, passo e arco ÷ 2, o grão Tiled e o dente do papel com o `size` × 2 (o `size` é a
//! FREQUÊNCIA do ladrilho) —
//! e a ampliação bilinear pré-multiplicada escreve o plano inteiro na caixa tocada, uma vez por lote. A amostra lê a
//! base (e as camadas visíveis) em médias 2×2 pré-multiplicadas, na hora. O plano do traço nasce transparente no
//! 1.º lote; a meia tela também (recomeça com a mistura, no pen-down).
//!
//! Arma só na rota do Grain Tiled (e dente Tiled) sem Shape, rampa, píxel, pó, papel como Grain, trinco de alfa, cauda
//! nem dual: o resto vai à rota de sempre, à definição inteira. Medição e aparência: `docs/Painter/BUGS_painter.md` #84.
//!
//! [`MisturaOpaca::meia_definicao`]: ph2d_painter_brush::mistura_opaca::MisturaOpaca::meia_definicao

use super::{Region, pegada_do_dab};
use crate::tool::PainterTool;
use ph2d_painter_brush::dente::Dente;
use ph2d_painter_brush::mistura_opaca::{
    Carimbo, Pegada, ReservatorioOpaco, Tela, mistura_opaca_carimbo,
};
use ph2d_painter_brush::texture::{TEX_SIZE_MAX, TextureMapping, TextureSettings};
use ph2d_painter_brush::{BrushSpec, Dab};
use rayon::prelude::*;

/// Um ladrilho preso à TELA (Tiled): só ele escala com a meia tela pelo `size` (a frequência). View Plane, Random e
/// Stencil seguem o dab ou o rect, que a meia tela já divide — o `size` × 2 dobrar-lhes-ia a frequência.
fn preso_a_tela(t: &TextureSettings) -> bool {
    t.mapping == TextureMapping::Tiled
}

/// O ladrilho Tiled `t` na meia tela: o `size` × 2 (o tecto do `size` é o píxel, `TEX_SIZE_MAX`).
fn na_meia_tela(t: TextureSettings) -> TextureSettings {
    TextureSettings {
        size: t.size.map(|s| (s * 2.0).min(TEX_SIZE_MAX)),
        ..t
    }
}

/// O dab `d` na meia tela.
fn meio(d: &Dab) -> Dab {
    let m = |p: [f32; 2]| [p[0] * 0.5, p[1] * 0.5];
    Dab {
        center: m(d.center),
        radius_px: d.radius_px * 0.5,
        stroke_radius_px: d.stroke_radius_px * 0.5,
        step: m(d.step),
        arc_len: d.arc_len * 0.5,
        ante: d.ante.map(|a| ph2d_painter_brush::Ante {
            radius_px: a.radius_px * 0.5,
            arc_len: a.arc_len * 0.5,
            salto: a.salto.map(|s| ph2d_painter_brush::capsula::Salto {
                desvio: m(s.desvio),
                alcance: s.alcance * 0.5,
                atraves: s.atraves * 0.5,
                ..s
            }),
            ..a
        }),
        ..*d
    }
}

/// O RGBA recto da média 2×2 pré-multiplicada de `src` (`w`×`h`) no texel do byte `i` da meia tela de largura `w2`.
fn media(src: &[u8], w: u32, h: u32, w2: u32, i: usize) -> [u8; 4] {
    let p = (i / 4) as u32;
    let (x2, y2) = (p % w2, p / w2);
    let (mut acc, mut n) = ([0u32; 4], 0u32);
    for y in 2 * y2..(2 * y2 + 2).min(h) {
        for x in 2 * x2..(2 * x2 + 2).min(w) {
            let j = ((y * w + x) * 4) as usize;
            let a = u32::from(src[j + 3]);
            for k in 0..3 {
                acc[k] += u32::from(src[j + k]) * a;
            }
            acc[3] += a;
            n += 1;
        }
    }
    if acc[3] == 0 || n == 0 {
        return [0; 4];
    }
    let c = |v: u32| ((v + acc[3] / 2) / acc[3]) as u8;
    [
        c(acc[0]),
        c(acc[1]),
        c(acc[2]),
        ((acc[3] + n / 2) / n) as u8,
    ]
}

/// Amplia a caixa `r` da meia tela `meia` (`w2`×`h2`) para o plano `buf` (`w`×`h`), bilinear pré-multiplicada; devolve
/// a caixa escrita no plano.
fn amplia(buf: &mut [u8], w: u32, h: u32, meia: &[u8], w2: u32, h2: u32, r: Region) -> Region {
    let x0 = (2 * r.x).saturating_sub(1);
    let y0 = (2 * r.y).saturating_sub(1);
    let x1 = (2 * (r.x + r.w) + 1).min(w);
    let y1 = (2 * (r.y + r.h) + 1).min(h);
    // O texel da meia tela à esquerda de `x` e o peso do da direita (centros em `X + 0,5` = `2X + 1` no plano).
    let eixo = |x: u32, n: u32| -> (u32, u32, f32) {
        let u = (x as f32 + 0.5) * 0.5 - 0.5;
        let f = u.floor();
        let t = u - f;
        let a = (f.max(0.0) as u32).min(n - 1);
        let b = ((f + 1.0).max(0.0) as u32).min(n - 1);
        (a, b, t)
    };
    let colunas: Vec<_> = (x0..x1).map(|x| eixo(x, w2)).collect();
    let linha = (w * 4) as usize;
    buf.par_chunks_mut(linha)
        .enumerate()
        .skip(y0 as usize)
        .take((y1 - y0) as usize)
        .for_each(|(y, out)| {
            let (ya, yb, ty) = eixo(y as u32, h2);
            for (k, &(xa, xb, tx)) in colunas.iter().enumerate() {
                let mut s = [0.0f32; 4];
                for (yy, wy) in [(ya, 1.0 - ty), (yb, ty)] {
                    for (xx, wx) in [(xa, 1.0 - tx), (xb, tx)] {
                        let wt = wx * wy;
                        if wt <= 0.0 {
                            continue;
                        }
                        let j = ((yy * w2 + xx) * 4) as usize;
                        let a = f32::from(meia[j + 3]) / 255.0 * wt;
                        for c in 0..3 {
                            s[c] += f32::from(meia[j + c]) * a;
                        }
                        s[3] += a;
                    }
                }
                let i = ((x0 as usize) + k) * 4;
                let px = &mut out[i..i + 4];
                if s[3] <= 0.0 {
                    px.fill(0);
                    continue;
                }
                for c in 0..3 {
                    px[c] = (s[c] / s[3] + 0.5).clamp(0.0, 255.0) as u8;
                }
                px[3] = (s[3] * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            }
        });
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

impl PainterTool {
    /// O lote vai à meia definição? Função do pincel e do traço (estável do pen-down ao pen-up).
    pub(super) fn mistura_meia_arma(
        &self,
        brush: &BrushSpec,
        alpha_locked: bool,
        n4: usize,
    ) -> bool {
        use super::super::super::ramp_lut::RampLutOwner;
        brush.mistura_opaca.meia_definicao
            && self.paint.acumulando_no_plano
            && self
                .paint
                .mistura_opaca
                .base_do_plano
                .as_ref()
                .is_some_and(|b| b.len() == n4)
            && !alpha_locked
            && brush.texture.is_active()
            && preso_a_tela(&brush.texture)
            && brush.dente.as_ref().is_none_or(|d| preso_a_tela(&d.papel))
            && !self.pixel_vivo(brush)
            && !brush.shape_silhouette_active(self.paint.shape_image.is_some())
            && self.color_ramp_owner(true) == RampLutOwner::None
            && !self.papel_como_grain()
            && self.paint.po_do_papel.as_ref().is_none_or(|p| p.vazio())
            && !brush.taper.end_active()
            && brush.dual.is_none()
    }

    /// O lote `dabs` (tela `w`×`h`) pela meia tela: [`Self::mistura_meia_arma`] já disse que sim.
    pub(super) fn stamp_dabs_mistura_meia(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        w: u32,
        h: u32,
    ) {
        let (w2, h2) = (w.div_ceil(2), h.div_ceil(2));
        let n4 = (w as usize) * (h as usize) * 4;
        let copias = brush.symmetry.copies().max(1);
        let pressao = self.paint.mistura_opaca.pressao;
        let groups = self.paint.dab_groups.clone();
        let Some(base) = self.paint.mistura_opaca.base_do_plano.clone() else {
            return;
        };
        let visivel = self.mistura_amostra_visivel(brush, Some(&base), n4);
        let mut res = std::mem::take(&mut self.paint.mistura_opaca.res);
        res.resize(copias, ReservatorioOpaco::default());
        let mut sombra = std::mem::take(&mut self.paint.mistura_opaca.sombra);
        let mut meia = std::mem::take(&mut self.paint.mistura_opaca.meia);
        let m4 = (w2 as usize) * (h2 as usize) * 4;
        if meia.len() != m4 {
            meia.clear();
            meia.resize(m4, 0);
        }
        let pincel = BrushSpec {
            texture: na_meia_tela(brush.texture),
            // O dente do papel é preso à tela como o grão Tiled: a mesma escala, senão sai 2× maior.
            dente: brush.dente.map(|d| Dente {
                papel: na_meia_tela(d.papel),
                ..d
            }),
            borda_px: brush.borda_px * 0.5,
            ..*brush
        };
        let metades: Vec<Dab> = dabs.iter().map(meio).collect();
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let mut dab_rng = super::super::super::tiling::DabRng::new(self.paint.tex_rng);
        let afina = self.paint.mistura_opaca.salta_a_copia_do_veu
            && brush.parado_s <= 0.0
            && brush.dab_jitter.veu > 0.0
            && brush.spray_count > 1
            && brush.stroke_method.allows_jitter();
        let bases: Vec<_> = metades
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let spec = pincel.at_dab(d);
                let basis = ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    dab_rng.enter(&groups, i),
                    [w2 as f32, h2 as f32],
                    spec.dab_footprint_of(d),
                );
                (spec, Some(basis))
            })
            .collect();
        let mut pegadas: Vec<_> = (0..metades.len())
            .into_par_iter()
            .map(|i| {
                let copia = afina
                    && i > 0
                    && metades[i - 1].center == metades[i].center
                    && groups.get(i - 1) == groups.get(i);
                (!copia).then(|| {
                    pegada_do_dab(
                        &bases[i].0,
                        bases[i].1.as_ref(),
                        &metades[i],
                        w2,
                        h2,
                        image.as_ref(),
                        None,
                    )
                })
            })
            .collect();
        let antes = |i: usize| media(&base, w, h, w2, i);
        let mut tocada: Option<Region> = None;
        let mut grupo_ant: Option<(usize, ReservatorioOpaco)> = None;
        let mut salta_o_grupo = None;
        for (i, d) in metades.iter().enumerate() {
            let g = groups.get(i).map_or(i, |&g| g as usize);
            let k = g % copias;
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
            if afina && *salta_o_grupo.get_or_insert_with(|| alvo.repete_o_centro(d.center)) {
                continue;
            }
            let pre = pegadas[i].take().unwrap_or_else(|| {
                pegada_do_dab(
                    &bases[i].0,
                    bases[i].1.as_ref(),
                    d,
                    w2,
                    h2,
                    image.as_ref(),
                    None,
                )
            });
            let Some((local, Some(r), r2)) = pre else {
                continue;
            };
            let carimbo = Carimbo {
                center: d.center,
                radius_px: d.radius_px,
                cor: d.color,
                coverage: d.coverage,
                pressao,
            };
            let pegada = Pegada::Mascara {
                mascara: &local.mascara,
                caixa: (r.x, r.y, r.x + r.w, r.y + r.h),
                org: [local.caixa.x, local.caixa.y],
                largura: local.caixa.w,
                sem_grao: r2.map(|_| local.liso.as_slice()),
                com_cor: false,
            };
            let mut tela = Tela {
                buf: &mut meia,
                width: w2,
                height: h2,
                sombra: &mut sombra,
                trava_alfa: false,
                plano: true,
            };
            let feito = match visivel.as_deref() {
                Some(v) => {
                    let amostra = |i: usize| media(v, w, h, w2, i);
                    mistura_opaca_carimbo(
                        &mut tela,
                        antes,
                        amostra,
                        &bases[i].0,
                        carimbo,
                        pegada,
                        alvo,
                    )
                }
                None => mistura_opaca_carimbo(
                    &mut tela,
                    antes,
                    antes,
                    &bases[i].0,
                    carimbo,
                    pegada,
                    alvo,
                ),
            };
            if let Some(r) = feito {
                let r = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                tocada = Some(tocada.map_or(r, |t| super::super::super::union_region(t, r)));
            }
        }
        let touched = tocada.map(|r| {
            // Superconjunto da caixa ampliada: a dos dabs com a orla da ampliação e o pad de AA da meia tela.
            let caixa = super::super::super::region::dabs_bounds(dabs, w, h, brush.tip_reach())
                .map(|b| {
                    let (x0, y0) = (b.x.saturating_sub(4), b.y.saturating_sub(4));
                    Region {
                        x: x0,
                        y: y0,
                        w: (b.x + b.w + 4).min(w) - x0,
                        h: (b.y + b.h + 4).min(h) - y0,
                    }
                });
            let buf = super::super::super::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                w,
                caixa,
            );
            amplia(buf, w, h, &meia, w2, h2, r)
        });
        self.paint.mistura_opaca.res = res;
        self.paint.mistura_opaca.sombra = sombra;
        self.paint.mistura_opaca.meia = meia;
        if let Some(r) = touched {
            self.paint.mistura_opaca.texels_ampliados += u64::from(r.w) * u64::from(r.h);
        }
        self.paint.tex_rng = dab_rng.finish();
        self.declare_wrote(touched);
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }
}
