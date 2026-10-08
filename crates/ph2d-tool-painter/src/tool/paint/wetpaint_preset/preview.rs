//! **O preview VIVO do pincel** — o traço do pincel de AGORA pintado sobre a OBRA de agora (o papel de
//! cor, o dente, o relevo, a inclinação, a grade), numa tela própria de [`LADO_MINIATURA`]², quadro a
//! quadro: a animação é a própria simulação gravada, e a ponte da app toca-a ao ritmo do relógio.
//!
//! Três peças: [`CenaDoPreview`] (o que se copia do tool, barato: `Arc` nas imagens), [`ChaveDoPreview`]
//! (a pergunta «mudou alguma coisa que o traço mostra?», comparada a cada quadro) e [`pintar_preview`]
//! (corre em qualquer thread). A saída de cada quadro é a MESMA do ecrã (`take_preview_arc`: o papel de
//! cor, o vidro e a luz do relevo entram ali) — o papel do painel aparece porque é a mesma porta.
//!
//! A miniatura dos ficheiros ([`super::miniatura_do_preset`]) é o último quadro deste pintor, com a
//! obra neutra (tela branca, sem papel).

use super::super::brush_settings::BrushTextureImage;
use crate::{PaintMedia, PainterTool, WetKnobs};
use ph2d_brush_presets::LADO_MINIATURA;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::BrushSpec;

/// O maior diâmetro que cabe no S sem tocar as margens (o S oscila ±[`AMPLITUDE`] em torno do meio).
pub(super) const DIAMETRO_MAX: f32 = 96.0;
pub(super) const DIAMETRO_MIN: f32 = 2.0;
const MARGEM: f32 = 28.0;
const AMPLITUDE: f32 = 40.0;
const PASSO_PX: f32 = 3.0;
const TICK_MS: f32 = 8.0;
/// 2 s de água depois do pen-up, a 60 Hz.
const TICKS_DEPOIS: u32 = 120;
/// Quantos quadros o traço dá (o gesto tem ~67 eventos: um quadro a cada ~3).
const QUADROS_DO_TRACO: u32 = 24;
/// Um quadro a cada 10 ticks da água (12 quadros nos 2 s).
const TICKS_POR_QUADRO: u32 = 10;

/// Tudo o que decide como o traço FICA na tela: o pincel (o `BrushSpec`, os knobs, as máscaras) e a obra
/// (o papel de cor e as gotas dele, o papel do pincel e a imagem dele, o relevo do substrato, e o estado
/// autorado do fluido: inclinação, grade, mistura, o papel visível). Opaco e `Send`: a ponte só o leva à thread.
#[derive(Clone)]
pub struct CenaDoPreview {
    brush: BrushSpec,
    knobs: WetKnobs,
    shape_image: Option<BrushTextureImage>,
    texture_image: Option<BrushTextureImage>,
    paper_image: Option<BrushTextureImage>,
    papel: Option<[u8; 3]>,
    substrate_depth: f32,
    substrate_rough: f32,
    paper_visual: bool,
    km_mixing: bool,
    km_glaze: bool,
    show_wet: bool,
    tilt: (bool, u8, u8),
    gotas: ph2d_wet_paint::drips::DripPaper,
    grids: (u8, u8),
}

/// «Mudou alguma coisa que o preview mostra?» — os mesmos ingredientes da [`CenaDoPreview`], com as
/// imagens pela VERSÃO (nunca pelo endereço: um `Arc` novo pode nascer no endereço de um morto).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChaveDoPreview {
    brush: BrushSpec,
    knobs: WetKnobs,
    versoes: (u64, u64, u64),
    papel: Option<[u8; 3]>,
    substrato: (f32, f32),
    flags: (bool, bool, bool, bool),
    tilt: (bool, u8, u8),
    gotas: ph2d_wet_paint::drips::DripPaper,
    grids: (u8, u8),
}

impl PainterTool {
    /// A cena do preview, copiada do pincel e da obra de agora.
    #[must_use]
    pub fn wet_preview_cena(&self) -> CenaDoPreview {
        let w = &self.paint.wetpaint;
        CenaDoPreview {
            brush: self.paint.brush,
            knobs: w.knobs,
            shape_image: self.paint.shape_image.clone(),
            texture_image: self.paint.texture_image.clone(),
            paper_image: self.paint.paper_image.clone(),
            papel: self.papel,
            substrate_depth: self.paint.substrate_depth,
            substrate_rough: self.paint.substrate_rough,
            paper_visual: w.paper_visual,
            km_mixing: w.km_mixing,
            km_glaze: w.km_glaze,
            show_wet: w.show_wet,
            tilt: (w.tilt_on, w.tilt_ring, w.tilt_spoke),
            gotas: self.gotas,
            grids: (w.grid_ratio, w.flow_ratio),
        }
    }

    /// A chave da cena — a ponte compara-a a cada quadro e repinta o preview quando muda.
    #[must_use]
    pub fn wet_preview_chave(&self) -> ChaveDoPreview {
        let w = &self.paint.wetpaint;
        ChaveDoPreview {
            brush: self.paint.brush,
            knobs: w.knobs,
            versoes: (
                self.paint.shape_image_version,
                self.paint.texture_image_version,
                self.paint.paper_image_version,
            ),
            papel: self.papel,
            substrato: (self.paint.substrate_depth, self.paint.substrate_rough),
            flags: (w.paper_visual, w.km_mixing, w.km_glaze, w.show_wet),
            tilt: (w.tilt_on, w.tilt_ring, w.tilt_spoke),
            gotas: self.gotas,
            grids: (w.grid_ratio, w.flow_ratio),
        }
    }

    /// Põe a cena num tool novo (o do preview). O traço é sempre o do PINCEL (`WetTool::Paint`, o de
    /// fábrica do tool novo): a cena NÃO leva a ferramenta molhada, e por isso o preview mostra o que o
    /// pincel deposita também quando a activa é o Smear (gate `o_preview_pinta_o_pincel_mesmo_com…`).
    fn instalar_cena(&mut self, c: &CenaDoPreview) {
        let mut brush = c.brush;
        brush.radius_px = (brush.radius_px * 2.0).clamp(DIAMETRO_MIN, DIAMETRO_MAX) * 0.5;
        self.paint.brush = brush;
        self.paint.shape_image = c.shape_image.clone();
        self.paint.texture_image = c.texture_image.clone();
        self.paint.paper_image = c.paper_image.clone();
        for v in [
            &mut self.paint.shape_image_version,
            &mut self.paint.texture_image_version,
            &mut self.paint.paper_image_version,
        ] {
            *v = v.wrapping_add(1);
        }
        self.sync_paper_across_slots();
        self.papel = c.papel;
        self.gotas = c.gotas;
        self.paint.substrate_depth = c.substrate_depth;
        self.paint.substrate_rough = c.substrate_rough;
        let w = &mut self.paint.wetpaint;
        w.knobs = c.knobs;
        w.paper_visual = c.paper_visual;
        w.km_mixing = c.km_mixing;
        w.km_glaze = c.km_glaze;
        w.show_wet = c.show_wet;
        (w.tilt_on, w.tilt_ring, w.tilt_spoke) = c.tilt;
        (w.grid_ratio, w.flow_ratio) = c.grids;
        self.paint.eraser = false;
        self.invalidate_composite();
    }
}

fn ponto(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O quadro de agora pela saída do ecrã; `None` quando nada mudou desde o anterior.
fn quadro(t: &mut PainterTool) -> Option<(std::sync::Arc<Vec<u8>>, u32, u32)> {
    t.take_preview_arc()
}

/// **Pinta o preview de `cena`**, chamando `emite(rgba, w, h)` a cada quadro (o último é o traço com a
/// água assente). `deve_parar` é perguntado a cada evento do gesto: `true` ⇒ desiste (a cena mudou) e
/// devolve `false`. Corre em qualquer thread: o tool é criado e largado aqui, no relógio fixo.
pub fn pintar_preview(
    cena: &CenaDoPreview,
    deve_parar: &dyn Fn() -> bool,
    emite: &mut dyn FnMut(&[u8], u32, u32),
) -> bool {
    let lado = LADO_MINIATURA;
    let mut t = PainterTool::default();
    // Com papel do documento a tela é transparente e o papel aparece por baixo (a mesma lei da obra).
    let fundo = if cena.papel.is_some() { 0u8 } else { 255 };
    t.set_source(vec![fundo; (lado * lado * 4) as usize], lado, lado);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.instalar_cena(cena);

    let l = lado as f32;
    let (x0, x1, cy) = (MARGEM, l - MARGEM, l * 0.5);
    let n = ((x1 - x0) / PASSO_PX).ceil() as u32;
    let cada = n.div_ceil(QUADROS_DO_TRACO).max(1);
    let em = |f: f32| {
        [
            x0 + (x1 - x0) * f,
            cy + AMPLITUDE * (f * std::f32::consts::TAU).sin(),
        ]
    };
    let pressao = |f: f32| 0.15 + 0.85 * (f * std::f32::consts::PI).sin();
    let mut emite_se = |t: &mut PainterTool| {
        if let Some((px, w, h)) = quadro(t) {
            emite(&px, w, h);
        }
    };
    t.on_canvas_pointer(ponto(em(0.0), pressao(0.0), PointerPhase::Down));
    for k in 1..=n {
        if deve_parar() {
            return false;
        }
        let f = k as f32 / n as f32;
        t.on_canvas_pointer(ponto(em(f), pressao(f), PointerPhase::Move));
        t.on_tick(TICK_MS);
        if k % cada == 0 {
            emite_se(&mut t);
        }
    }
    t.on_canvas_pointer(ponto(em(1.0), pressao(1.0), PointerPhase::Up));
    for k in 1..=TICKS_DEPOIS {
        if deve_parar() {
            return false;
        }
        t.on_tick(1000.0 / 60.0);
        if k % TICKS_POR_QUADRO == 0 {
            emite_se(&mut t);
        }
    }
    true
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
