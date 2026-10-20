//! ⭐ **A rota da ponta de PÍXEL** (L18, `docs/Painter/Pinceis/03_MOTORES.md` §4; a lei em
//! [`ph2d_painter_brush::pixel`]). Duas metades, nas duas portas por onde todo dab passa:
//!
//! - **O trilho** ([`PainterTool::pixel_trilho`], em `stamp_dabs_routed`, ANTES do Tiling): cada dab vira a
//!   âncora dele na grelha, o caminho liga-se de âncora em âncora por Bresenham e, no «pixel-perfect» de 1 px, a
//!   quina de cada L sai. A quina só se conhece com o píxel SEGUINTE, então o último píxel de cada fluxo fica
//!   PENDENTE até ao próximo lote ou ao levantar da caneta ([`PainterTool::pixel_fim_do_traco`]). Um fluxo por
//!   cópia da simetria (o lote vem intercalado, `SymmetrySettings::copies`); com spray cada marca é solta.
//! - **O carimbo** ([`PainterTool::stamp_dabs_pixel`], em `stamp_dabs_inner`): binário, a cobertura toda do dab
//!   em cada píxel da pegada, o tecto do traço com o Accumulate desligado.
//!
//! Só o Digital lê a ponta de píxel ([`PainterTool::pixel_vivo`]); a placa não a carimba (a rota sai antes do
//! predicado do device).

use ph2d_painter_brush::pixel::{self, PixelTip};
use ph2d_painter_brush::{BrushSpec, Dab, StrokeMethod};

use super::{PainterTool, Region, union_region};

/// O raio mais pequeno da ponta de píxel: meio píxel = a ponta de 1 px (a com AA pára em
/// [`super::BRUSH_SIZE_MIN_PX`], 2 px, porque não cumpre o 1).
pub(super) const PIXEL_SIZE_MIN_PX: f32 = 0.5;

/// O fluxo de UMA cópia da simetria: a última âncora andada, o último píxel carimbado e o pendente.
#[derive(Clone, Debug, Default)]
pub(super) struct Fluxo {
    ultima: Option<([i32; 2], i32)>,
    emitido: Option<[i32; 2]>,
    pendente: Option<([i32; 2], Dab)>,
}

/// Os fluxos do traço (um por cópia da simetria) — zerados no pen-down.
#[derive(Clone, Debug, Default)]
pub(super) struct Trilhos {
    fluxos: Vec<Fluxo>,
    /// O lote que chega já ancorado (os pendentes soltos no pen-up): o trilho não o anda de novo.
    pub(super) ja_ancorado: bool,
}

impl Trilhos {
    pub(super) fn zera(&mut self) {
        self.fluxos.clear();
    }
}

/// O dab `d` posto no centro da âncora `a` (diâmetro `dm`).
fn na_ancora(d: &Dab, a: [i32; 2], dm: i32) -> Dab {
    Dab {
        center: pixel::centro(a, dm),
        ..*d
    }
}

impl Fluxo {
    /// Anda até ao dab `d`; os dabs a carimbar vão para `out`.
    fn anda(&mut self, d: &Dab, perfeito: bool, out: &mut Vec<Dab>) {
        let dm = pixel::diametro(d.radius_px);
        let a = pixel::ancora(d.center, dm);
        let mut pontos = Vec::new();
        match self.ultima {
            Some((u, du)) if du == dm => pixel::bresenham(u, a, (dm / 8).max(1), &mut pontos),
            _ => pontos.push(a),
        }
        self.ultima = Some((a, dm));
        let perfeito = perfeito && dm == 1;
        for p in pontos {
            if !perfeito {
                self.solta(out);
                out.push(na_ancora(d, p, dm));
                continue;
            }
            if let Some((p1, d1)) = self.pendente.take() {
                let quina = self.emitido.is_some_and(|p0| pixel::e_quina(p0, p1, p));
                if !quina {
                    out.push(d1);
                    self.emitido = Some(p1);
                }
            }
            self.pendente = Some((p, na_ancora(d, p, dm)));
        }
    }

    /// Carimba o pendente (o fim do traço, ou a troca para fora do pixel-perfect).
    fn solta(&mut self, out: &mut Vec<Dab>) {
        if let Some((p, d)) = self.pendente.take() {
            out.push(d);
            self.emitido = Some(p);
        }
    }
}

impl PainterTool {
    /// A ponta de píxel pinta neste meio (só o Digital a lê).
    pub(super) fn pixel_vivo(&self, brush: &BrushSpec) -> bool {
        brush.pixel.on() && self.paint_media() == super::media::PaintMedia::Digital
    }

    /// ⭐ **O trilho de píxel** do lote (ver o módulo): devolve os dabs ancorados e ligados. `fim` solta os
    /// pendentes. Os métodos que re-carimbam o traço inteiro a cada evento (Line, Curve…) começam e acabam o
    /// trilho no próprio lote.
    pub(super) fn pixel_trilho(&mut self, dabs: &[Dab], fim: bool) -> Vec<Dab> {
        let b = &self.paint.brush;
        let perfeito = b.pixel == PixelTip::Perfect;
        let incremental = matches!(
            b.stroke_method,
            StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
        );
        // Com spray (ou a contagem sorteada da marca) cada ponto deixa várias marcas: soltas, sem trilho.
        let soltas = b.spray_count > 1 || b.dab_jitter.count > 0.0;
        let copias = b.symmetry.copies().max(1);
        let tr = &mut self.paint.pixel_trilhos;
        if !incremental {
            tr.zera();
        }
        let mut out = Vec::with_capacity(dabs.len());
        if soltas {
            out.extend(dabs.iter().map(|d| {
                let dm = pixel::diametro(d.radius_px);
                na_ancora(d, pixel::ancora(d.center, dm), dm)
            }));
            return out;
        }
        if tr.fluxos.len() != copias {
            tr.fluxos = vec![Fluxo::default(); copias];
        }
        for (i, d) in dabs.iter().enumerate() {
            tr.fluxos[i % copias].anda(d, perfeito, &mut out);
        }
        if fim || !incremental {
            for f in &mut tr.fluxos {
                f.solta(&mut out);
            }
        }
        out
    }

    /// O levantar da caneta: carimba os píxeis pendentes do pixel-perfect.
    pub(super) fn pixel_fim_do_traco(&mut self) {
        if !self.pixel_vivo(&self.paint.brush) {
            return;
        }
        let resto = self.pixel_trilho(&[], true);
        if !resto.is_empty() {
            self.paint.pixel_trilhos.ja_ancorado = true;
            self.stamp_dabs(&resto);
            self.paint.pixel_trilhos.ja_ancorado = false;
        }
        self.paint.pixel_trilhos.zera();
    }

    /// ⭐ **O carimbo de píxel** de um lote já ancorado: binário, a cobertura toda do dab por píxel.
    pub(super) fn stamp_dabs_pixel(&mut self, dabs: &[Dab], brush: &BrushSpec, alpha_locked: bool) {
        let (w, h) = self.source_size;
        let cap = !brush.accumulate;
        if cap {
            super::stamp_cache::prepare_stroke_mask(
                &mut self.paint.stroke_mask,
                (w as usize) * (h as usize),
                brush.stroke_method,
            );
        }
        let mut regiao: Option<Region> = None;
        for d in dabs {
            let r = pixel::diametro(d.radius_px) as f32 * 0.5 + 2.0;
            let x0 = (d.center[0] - r).floor().max(0.0) as u32;
            let y0 = (d.center[1] - r).floor().max(0.0) as u32;
            let x1 = ((d.center[0] + r).ceil().max(0.0) as u32).min(w);
            let y1 = ((d.center[1] + r).ceil().max(0.0) as u32).min(h);
            if x1 > x0 && y1 > y0 {
                let q = Region {
                    x: x0,
                    y: y0,
                    w: x1 - x0,
                    h: y1 - y0,
                };
                regiao = Some(regiao.map_or(q, |a| union_region(a, q)));
            }
        }
        if regiao.is_none() {
            return;
        }
        // A rampa de cor da Shape pela QUEDA («Falloff Color»): a cor de cada píxel no perfil dele, o alfa binário.
        // A Texture Color e a rampa do Grain leem a textura, que a ponta de píxel não carimba (esmaecem).
        let p = &self.paint;
        let rampa_viva = p.shape_color_ramp_enabled && !p.shape_color_ramp_bw && p.shape_camadas.alguma(false);
        if rampa_viva {
            self.ensure_ramp_lut(super::ramp_lut::RampLutOwner::Shape);
        }
        let buf = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            regiao,
        );
        let rampa = rampa_viva.then(|| ph2d_painter_brush::RampPaint {
            lut: &self.paint.texture_ramp_lut,
            alpha_mode: super::ramp_lut::BRUSH_RAMP_ALPHA,
            index: super::ramp_lut::RampLutOwner::Shape.index(),
            shape: self.paint.shape_camadas.motor(false),
            shift: [0.0; 3],
        });
        let mut mask = cap.then_some(self.paint.stroke_mask.as_mut_slice());
        let mut tocado: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                color: d.color,
                ..brush.at_dab(d)
            };
            let rampa = rampa.map(|r| ph2d_painter_brush::RampPaint {
                shift: d.color_shift,
                ..r
            });
            if let Some(r) =
                pixel::carimba(buf, w, h, d, &spec, alpha_locked, mask.as_deref_mut(), rampa.as_ref())
            {
                let q = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                tocado = Some(tocado.map_or(q, |a| union_region(a, q)));
            }
        }
        self.declare_wrote(tocado);
        if let Some(r) = tocado {
            self.mark_dirty(r);
        }
    }
}

