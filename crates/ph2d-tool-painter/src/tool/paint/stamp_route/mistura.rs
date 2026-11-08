//! A rota do **Color mixing «Wet blend»** do Digital ([`ph2d_painter_brush::mistura_opaca`], a lei medida do CSP):
//! só com o modo ligado e sem apagar; `Off` nunca chega aqui (a rota de sempre, ao bit).
//!
//! - **Reservatório:** um por cópia da Symmetry (as cópias saem em blocos de `copies()` dabs,
//!   `symmetry::push_symmetric`), para o lado espelhado não levar a cor do outro. As cópias do Tiling
//!   (`dab_groups`) partem todas do MESMO estado e só a primeira o avança — a costura casa.
//! - **Renasce** no pen-down e em cada re-carimbo de figura (`stamp_drag_preview`).
//! - **Amostra a tela do pen-DESTE traço**, sem o undo (que agrupa traços sobrepostos, W16, e vai
//!   deixar de guardar a tela inteira): na tela directa, os ladrilhos [`Ladrilhos`] que o traço vai
//!   escrever, guardados antes da 1.ª escrita (só com a mistura ligada; `Off` não paga nada); no plano
//!   do traço (o dual, L8), a `base` dele, que já é o pré-traço. O re-carimbo de figura repõe a tela e
//!   recomeça os dois.
//! - **Pressão:** a do último evento da caneta (o `Dab` não a leva), para todos os dabs do lote.
//! - **Alpha lock:** a cor nova entra na proporção do alfa de antes e o alfa fica (o `preserve_alpha`).
//!   A proteção e a seleção vêm do invólucro (`stamp_dabs_gated`), como em toda rota.
//! - **A pegada:** o kernel de sempre pinta o dab numa máscara (`pegada`: falloff, Shape, Grain, a cápsula da cauda),
//!   com o MESMO stream `tex_rng` que a rota normal consome por dab; o alfa dela pesa a amostra e a cobertura.
//! - **A tela escrita** vive numa `Sombra` `f32` do traço (a lei acumula centenas de carimbos de α pequeno);
//!   quem repõe a tela por baixo dela (re-carimbo de figura, a cauda) recomeça-a.
//! - A 2.ª ponta (dual) carimba a máscara SEM a mistura (`DualTip::pincel`): partilhava o reservatório.

use super::super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::mistura_opaca::{
    Carimbo, Pegada, ReservatorioOpaco, Sombra, Tela, mistura_opaca_carimbo,
};
use ph2d_painter_brush::{BrushSpec, Dab};
use std::sync::Arc;

/// O lado de um ladrilho da tela de antes (texels): 64² × 4 = 16 KiB por ladrilho tocado.
const LADO: u32 = 64;

/// A tela do pen-down, só nos ladrilhos que o traço tocou (o resto ainda É o pen-down na tela viva).
#[derive(Clone, Debug, Default)]
pub(in crate::tool::paint) struct Ladrilhos {
    w: u32,
    h: u32,
    colunas: u32,
    guardados: Vec<Option<Box<[u8]>>>,
}

impl Ladrilhos {
    fn recomeca(&mut self) {
        self.guardados.clear();
    }

    /// Guarda (uma vez por traço) os ladrilhos da caixa `x0..x1 × y0..y1` de `tela`, antes de ela ser escrita.
    fn guarda(&mut self, tela: &[u8], w: u32, h: u32, (x0, y0, x1, y1): (u32, u32, u32, u32)) {
        if self.w != w || self.h != h || self.guardados.is_empty() {
            self.w = w;
            self.h = h;
            self.colunas = w.div_ceil(LADO);
            self.guardados.clear();
            self.guardados
                .resize((self.colunas * h.div_ceil(LADO)) as usize, None);
        }
        for ty in y0 / LADO..=(y1 - 1) / LADO {
            for tx in x0 / LADO..=(x1 - 1) / LADO {
                let slot = &mut self.guardados[(ty * self.colunas + tx) as usize];
                if slot.is_some() {
                    continue;
                }
                let (lx, ly) = (tx * LADO, ty * LADO);
                let (lw, lh) = (LADO.min(w - lx), LADO.min(h - ly));
                let mut b = Vec::with_capacity((LADO * LADO * 4) as usize);
                for y in ly..ly + lh {
                    let i = ((y * w + lx) * 4) as usize;
                    b.extend_from_slice(&tela[i..i + (lw * 4) as usize]);
                }
                *slot = Some(b.into_boxed_slice());
            }
        }
    }

    /// O RGBA do byte `i` da tela do pen-down (o ladrilho tem de estar guardado).
    fn le(&self, i: usize) -> [u8; 4] {
        let p = (i / 4) as u32;
        let (x, y) = (p % self.w, p / self.w);
        let (tx, ty) = (x / LADO, y / LADO);
        let lw = LADO.min(self.w - tx * LADO);
        let t = self.guardados[(ty * self.colunas + tx) as usize]
            .as_deref()
            .expect("ladrilho guardado antes da leitura");
        let j = (((y - ty * LADO) * lw + (x - tx * LADO)) * 4) as usize;
        [t[j], t[j + 1], t[j + 2], t[j + 3]]
    }

    /// Bytes guardados agora (a memória que a mistura paga).
    #[cfg(test)]
    pub(in crate::tool::paint) fn bytes(&self) -> usize {
        self.guardados.iter().flatten().map(|b| b.len()).sum()
    }
}

/// O estado da mistura opaca num traço.
#[derive(Clone, Debug)]
pub(in crate::tool::paint) struct MisturaDoTraco {
    /// Um reservatório por cópia da Symmetry.
    pub res: Vec<ReservatorioOpaco>,
    /// A pressão da caneta do último evento (`1` no rato).
    pub pressao: f32,
    /// A tela do pen-down nos ladrilhos tocados (a tela directa).
    pub antes: Ladrilhos,
    /// A `base` do plano do traço durante um lote no plano (`traco_plano::carimba_no_plano`).
    pub base_do_plano: Option<Arc<Vec<u8>>>,
    /// A máscara RGBA (tamanho da tela, transparente fora do dab) onde o kernel pinta a pegada de cada dab.
    pub pegada: Vec<u8>,
    /// A mesma pegada sem o Grain (só com o Grain ativo).
    pub pegada_lisa: Vec<u8>,
    /// A tela escrita em `f32` durante o traço.
    pub sombra: Sombra,
}

impl Default for MisturaDoTraco {
    fn default() -> Self {
        Self {
            res: Vec::new(),
            pressao: 1.0,
            antes: Ladrilhos::default(),
            base_do_plano: None,
            pegada: Vec::new(),
            pegada_lisa: Vec::new(),
            sombra: Sombra::default(),
        }
    }
}

impl PainterTool {
    /// O pincel começa limpo e a tela de antes é a de agora (pen-down, re-carimbo de figura — que já repôs a tela).
    pub(in crate::tool::paint) fn mistura_opaca_recomeca(&mut self) {
        self.paint.mistura_opaca.res.clear();
        self.paint.mistura_opaca.antes.recomeca();
        self.paint.mistura_opaca.sombra.recomeca();
    }

    /// A pressão do evento que vai gerar o próximo lote.
    pub(in crate::tool::paint) fn mistura_opaca_pressao(&mut self, p: f32) {
        self.paint.mistura_opaca.pressao = if p.is_finite() {
            p.clamp(0.0, 1.0)
        } else {
            1.0
        };
    }

    pub(super) fn stamp_dabs_mistura_opaca(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let copias = brush.symmetry.copies().max(1);
        let pressao = self.paint.mistura_opaca.pressao;
        let groups = self.paint.dab_groups.clone();
        let mut res = std::mem::take(&mut self.paint.mistura_opaca.res);
        res.resize(copias, ReservatorioOpaco::default());
        // No plano do traço a tela de antes é a base dele; noutro plano (uma camada do Composite) a do undo.
        let n4 = (w as usize) * (h as usize) * 4;
        let base = if self.paint.acumulando_no_plano {
            self.paint
                .mistura_opaca
                .base_do_plano
                .clone()
                .or_else(|| {
                    self.paint
                        .stroke_undo
                        .as_ref()
                        .map(|b| Arc::clone(&b.canvas_rgba))
                })
                .filter(|b| b.len() == n4)
        } else {
            None
        };
        let mut ladrilhos = std::mem::take(&mut self.paint.mistura_opaca.antes);
        let mut sombra = std::mem::take(&mut self.paint.mistura_opaca.sombra);
        // A pegada real (Shape, Grain); o stream de RNG anda por dab como na rota normal.
        self.ensure_shape_ramp_lut();
        let papel = self.papel_como_grain();
        let shape_tom = self.shape_tom_ativo();
        let image = if papel {
            &self.paint.paper_image
        } else {
            &self.paint.texture_image
        }
        .as_ref()
        .map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = shape_tom.then_some(self.paint.shape_ramp_lut.as_slice());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        let textured = brush.texture.is_active();
        let mut mascara = std::mem::take(&mut self.paint.mistura_opaca.pegada);
        if mascara.len() != n4 {
            mascara.clear();
            mascara.resize(n4, 0);
        }
        let mut liso = std::mem::take(&mut self.paint.mistura_opaca.pegada_lisa);
        if textured && liso.len() != n4 {
            liso.clear();
            liso.resize(n4, 0);
        }
        let mut dab_rng = super::super::tiling::DabRng::new(self.paint.tex_rng);
        let buf = super::super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            w,
            super::super::region::dabs_bounds(dabs, w, h, brush.tip_reach()),
        );
        let mut touched: Option<Region> = None;
        let mut grupo_ant: Option<(usize, ReservatorioOpaco)> = None;
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
                    &mut res[k]
                }
            };
            let spec = brush.at_dab(d);
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
            let Some(r) = ph2d_painter_brush::stamp_dab_textured_masked(
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
            ) else {
                continue;
            };
            let mut caixa = (r.x, r.y, r.x + r.w, r.y + r.h);
            // Com o Grain, a mesma pegada sem ele (o grão vai ao tecto, `Pegada::Mascara::sem_grao`).
            let com_grao = basis.is_some();
            if com_grao
                && let Some(r2) = ph2d_painter_brush::stamp_dab_textured_masked(
                    &mut liso,
                    w,
                    h,
                    d.center,
                    &BrushSpec { texture: Default::default(), ..branco },
                    1.0,
                    false,
                    None,
                    None,
                    shape_in,
                    None,
                    None,
                    spec.dab_rotor(d),
                    d.capsula(&branco),
                )
            {
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
                sem_grao: com_grao.then_some(liso.as_slice()),
            };
            let mut tela = Tela { buf: &mut *buf, width: w, height: h, sombra: &mut sombra, trava_alfa: alpha_locked, plano: base.is_some() };
            let r = if let Some(b) = base.as_deref() {
                let antes = |i: usize| [b[i], b[i + 1], b[i + 2], b[i + 3]];
                mistura_opaca_carimbo(&mut tela, antes, &spec, carimbo, pegada, alvo)
            } else {
                // Os ladrilhos do dab guardam-se ANTES de ele escrever: nenhum é lido depois de escrito.
                ladrilhos.guarda(tela.buf, w, h, caixa);
                let antes = |i: usize| ladrilhos.le(i);
                mistura_opaca_carimbo(&mut tela, antes, &spec, carimbo, pegada, alvo)
            };
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
            touched = Some(touched.map_or(r, |t| super::super::union_region(t, r)));
        }
        self.paint.mistura_opaca.res = res;
        self.paint.mistura_opaca.antes = ladrilhos;
        self.paint.mistura_opaca.sombra = sombra;
        self.paint.mistura_opaca.pegada = mascara;
        self.paint.mistura_opaca.pegada_lisa = liso;
        self.paint.tex_rng = dab_rng.finish();
        self.declare_wrote(touched);
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }
}
