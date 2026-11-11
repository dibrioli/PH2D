//! **O TOQUE PREPARADO** (Bug #72): o motor e o papel da próxima sessão ficam prontos noutro fio, com o Wet armado e
//! sem sessão, e o pen-down só os recebe. A sessão continua a nascer no toque, com a tela de então: aqui só se
//! adianta a conta, que é função da grade e da [`PaperKey`]. Sem nada pronto, o toque faz a conta como sempre.
//! `PH2D_TOQUE_PREPARADO=0` desliga.

use super::*;
use crate::tool::paint::brush_settings::BrushTextureImage;
use std::thread::JoinHandle;

/// O que o preparo adiantou, e a chave de cada metade.
pub(super) struct Pronto {
    grade: Grade,
    engine: Engine,
    papel: Option<PapelPronto>,
}

/// O plano do papel semeado, com a chave que o pediu.
pub(super) type PapelPronto = (PaperKey, Vec<f32>);

/// A grade do motor: o que `Engine::with_flow_ratio` recebe, e a razão que a lei do papel lê.
#[derive(Clone, Copy, PartialEq)]
struct Grade {
    gw: usize,
    gh: usize,
    flow: usize,
    ratio: u8,
}

/// O estado do preparo no tool: um fio de cada vez, e o último resultado.
#[derive(Default)]
pub(in crate::tool::paint) struct PreparoDoToque {
    fio: Option<JoinHandle<Pronto>>,
    pronto: Option<Pronto>,
}

/// Uma cópia do estado do tool não herda a conta adiantada.
impl Clone for PreparoDoToque {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PreparoDoToque {
    /// Recolhe o fio que acabou, sem esperar.
    fn recolhe(&mut self) {
        if self.fio.as_ref().is_some_and(JoinHandle::is_finished)
            && let Some(fio) = self.fio.take()
            && let Ok(pronto) = fio.join()
        {
            self.pronto = Some(pronto);
        }
    }
}

#[cfg(test)]
thread_local! {
    static NO_TESTE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static USADOS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

/// Liga o preparo neste fio de teste (os testes correm sem ele: cada um é uma sessão descartável).
#[cfg(test)]
pub(super) fn liga_no_teste(ligado: bool) {
    NO_TESTE.with(|n| n.set(ligado));
    USADOS.with(|n| n.set((0, 0)));
}

/// `(motores, papéis)` que os toques deste fio receberam prontos.
#[cfg(test)]
pub(super) fn usados_no_teste() -> (usize, usize) {
    USADOS.with(std::cell::Cell::get)
}

fn ligado() -> bool {
    #[cfg(test)]
    {
        NO_TESTE.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        static L: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *L.get_or_init(|| std::env::var("PH2D_TOQUE_PREPARADO").as_deref() != Ok("0"))
    }
}

/// A lei do papel da sessão, em células: UMA, a que o toque e o preparo correm. Com um Kind armado, o do artista; sem
/// Kind, as alturas do papel de fábrica ao acaso ([`PainterTool::reconcilia_o_papel`]).
pub(super) fn lei_do_papel<'a>(
    tex: ph2d_painter_brush::TextureSettings,
    imagem: Option<&'a BrushTextureImage>,
    period: [f32; 2],
    ratio: u8,
) -> Box<dyn Fn(i64, i64) -> f64 + Sync + 'a> {
    if !tex.is_active() {
        let q = super::super::substrate_relief::alturas_do_papel_de_fabrica();
        return Box::new(move |cx, cy| {
            f64::from(q[super::super::substrate_relief::celula_ao_acaso(cx, cy, q.len())])
        });
    }
    let mascara = imagem.map(BrushTextureImage::as_mask);
    let rot = ph2d_painter_brush::texture::angle_basis(tex.angle_deg);
    Box::new(move |cx, cy| {
        // ⚠️ O papel é uma lei de CANVAS: sob razão > 1 amostrá-lo na coordenada de célula encolheria o dente.
        let px = grid_map::cell_center_texel(cx as i32 + 1, ratio);
        let py = grid_map::cell_center_texel(cy as i32 + 1, ratio);
        f64::from(ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
            &tex,
            px,
            py,
            mascara.as_ref(),
            rot,
            period,
        ))
    })
}

#[cfg(test)]
impl PainterTool {
    /// A conta adiantada já está à espera do toque.
    pub(super) fn preparo_pronto(&mut self) -> bool {
        let preparo = &mut self.paint.wetpaint.preparo;
        preparo.recolhe();
        preparo.pronto.is_some()
    }
}

impl PainterTool {
    fn grade_da_proxima_sessao(&self) -> Grade {
        let (w, h) = self.source_size;
        let ratio = grid_map::clamp_ratio(self.paint.wetpaint.grid_ratio);
        let (gw, gh) = grid_map::grid_dims(w as usize, h as usize, ratio);
        let flow =
            ph2d_wet_paint::flow::clamp_ratio(usize::from(self.paint.wetpaint.flow_ratio.max(1)));
        Grade {
            gw,
            gh,
            flow,
            ratio,
        }
    }

    /// A chave do papel que o pincel de agora pede ([`PainterTool::reconcilia_o_papel`]).
    pub(super) fn chave_do_papel(&self, brush: &BrushSpec) -> PaperKey {
        let (w, h) = self.source_size;
        PaperKey {
            tex: self.papel_na_escala(brush.paper),
            image_version: self.paint.paper_image_version,
            period: [
                if self.paint.tiling[0] { w as f32 } else { 0.0 },
                if self.paint.tiling[1] { h as f32 } else { 0.0 },
            ],
            tooth: brush
                .paper_depth
                .clamp(0.0, ph2d_painter_brush::PAPER_TOOTH_MAX),
        }
    }

    /// O tique sem sessão: adianta o que o próximo toque vai pedir. Uma comparação de chaves quando já está pronto.
    pub(super) fn prepara_o_toque(&mut self) {
        if !matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            // Fora do Wet o preparo não fica a segurar a grade.
            let preparo = &mut self.paint.wetpaint.preparo;
            preparo.recolhe();
            preparo.pronto = None;
            return;
        }
        let wp = &self.paint.wetpaint;
        if !ligado()
            || self.source_size.0 == 0
            || self.source_size.1 == 0
            || wp.relogio_fixo.is_some()
            || wp.fita.is_some()
            || wp.ritmo.is_some()
        {
            return;
        }
        let grade = self.grade_da_proxima_sessao();
        let chave = self.chave_do_papel(&self.paint.brush);
        let preparo = &mut self.paint.wetpaint.preparo;
        preparo.recolhe();
        if preparo.fio.is_some() {
            return; // um fio de cada vez: o tique seguinte vê o que ficou desactualizado
        }
        let pronto = preparo.pronto.take();
        if let Some(p) = &pronto
            && p.grade == grade
            && p.papel.as_ref().is_some_and(|(c, _)| *c == chave)
        {
            preparo.pronto = pronto;
            return;
        }
        // O motor serve se a grade é a mesma; o papel, se a chave é a mesma.
        let motor = pronto.filter(|p| p.grade == grade).map(|p| p.engine);
        let imagem = self.paint.paper_image.clone();
        let fio = std::thread::Builder::new()
            .name("ph2d-toque-preparado".into())
            .spawn(move || {
                let engine = motor
                    .unwrap_or_else(|| Engine::with_flow_ratio(grade.gw, grade.gh, grade.flow));
                // O dente que o toque daria: sem Kind as alturas entram inteiras.
                let k = if chave.tex.is_active() {
                    chave.tooth.max(0.0)
                } else {
                    1.0
                };
                let lei = lei_do_papel(chave.tex, imagem.as_ref(), chave.period, grade.ratio);
                let plano = engine.papel_semeado(k, &*lei);
                drop(lei);
                Pronto {
                    grade,
                    engine,
                    papel: Some((chave, plano)),
                }
            });
        if let Ok(fio) = fio {
            self.paint.wetpaint.preparo.fio = Some(fio);
        }
    }

    /// O motor pronto do toque (e o papel, que a sessão guarda até o primeiro lote), se a grade é a pedida.
    pub(super) fn toma_o_preparo(
        &mut self,
        gw: usize,
        gh: usize,
        flow: usize,
        ratio: u8,
    ) -> Option<(Engine, Option<PapelPronto>)> {
        let preparo = &mut self.paint.wetpaint.preparo;
        preparo.recolhe();
        // Com a sessão viva o preparo não corre: o que ficasse aqui segurava um motor inteiro até ela acabar. O fio ainda
        // a correr larga-se: acaba sozinho e liberta o motor.
        preparo.fio = None;
        let p = preparo.pronto.take()?;
        let pedida = Grade {
            gw,
            gh,
            flow,
            ratio,
        };
        if p.grade != pedida {
            return None;
        }
        #[cfg(test)]
        USADOS.with(|n| {
            let (a, b) = n.get();
            n.set((a + 1, b));
        });
        Some((p.engine, p.papel))
    }
}

/// Conta um papel recebido pronto.
pub(super) fn conta_o_papel() {
    #[cfg(test)]
    USADOS.with(|n| {
        let (a, b) = n.get();
        n.set((a, b + 1));
    });
}
