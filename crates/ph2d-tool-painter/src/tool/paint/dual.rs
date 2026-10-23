//! ⭐ **O DUAL BRUSH no tool** (L6, `docs/Painter/Pinceis/03_MOTORES.md` §4; a lei em [`ph2d_painter_brush::dual`]).
//!
//! A 2.ª ponta tem um MOTOR de traço próprio ([`Stroke`] com o pincel [`DualTip::pincel`]) alimentado pelos MESMOS
//! pontos que o da 1.ª (pen-down, cada movimento, o tique, o assentar e o fim) — o mesmo caminho, o passo, o espalhar,
//! a contagem e a rotação dela. As marcas que ele emite esperam em `pendentes` até ao lote seguinte do PLANO do
//! traço (`super::traco_plano`), que as carimba na MÁSCARA do traço (um plano RGBA, a máscara no alfa) por TODAS as
//! rotas de sempre — trocando, só durante esse carimbo, a tela pela máscara, o pincel pelo dela, a Shape pela imagem
//! dela e a cache do carimbo pela dela — e compõe cada texel com a cobertura da 1.ª recortada (`DualMode::recorta`).
//!
//! ⚠️ Arma onde o plano arma (o Digital, um traço livre que pinta: [`PainterTool::plano_recusado`]) e no **Wet Paint**
//! que pinta: lá a máscara é do traço e recorta a SILHUETA de cada dab que pousa pigmento (e o relevo que ele penteia)
//! — [`RecorteNoWet`], na `wetpaint/dab_route`. A aguada e o Impasto não (`DualForaDoDigital`).

use std::mem::swap;
use std::sync::Arc;

use ph2d_painter_brush::{Dab, DualMode, DualTip, Stroke, StrokePoint, TextureKind};

use super::Region;
use super::brush_image::BrushTextureImage;
use super::shape_layers::ShapeLayers;
use crate::tool::PainterTool;

/// O bit das versões da imagem da 2.ª ponta: nunca coincide com as da Shape (que contam de 0) nem com as das imagens
/// da ponta de N imagens (`1 << 62`).
const VERSAO_DO_DUAL: u64 = 1 << 61;
/// O fluxo de sorteio da 2.ª ponta, descorrelacionado do da 1.ª (HR-5: determinístico por traço).
const SEMENTE_DO_DUAL: u64 = 0x6475_616C_5F74_6970;
/// O id da imagem da 2.ª ponta no preset.
pub(super) const TEXTURA_DO_DUAL: &str = "dual_shape";

/// O estado da 2.ª ponta no tool.
#[derive(Clone, Default)]
pub(crate) struct DualEstado {
    /// O motor de traço da 2.ª ponta — do pen-down ao pen-up de um traço que a leva.
    motor: Option<Stroke>,
    /// As marcas emitidas e ainda não carimbadas na máscara.
    pendentes: Vec<Dab>,
    /// A imagem da 2.ª ponta (a forma `Image`).
    imagem: Option<BrushTextureImage>,
    /// As camadas da Shape durante o carimbo dela: vazias (a máscara não tem cor).
    camadas: ShapeLayers,
    /// A cache do carimbo da 2.ª ponta (trocada com a da Shape: as duas não se desalojam a cada lote).
    cache: Option<(ph2d_painter_brush::StampMask, super::stamp_cache::StampKey)>,
    /// O fluxo de sorteio da textura da 2.ª ponta.
    tex_rng: u64,
    versao: u64,
    /// O botão «Dual image» pediu o seletor de ficheiro à shell.
    pedido: bool,
    /// O fim do traço descarrega as marcas pendentes sem um lote da 1.ª.
    no_fim: bool,
    /// A máscara do traço no Wet Paint (no Digital ela é do plano do traço).
    mascara: Option<Arc<Vec<u8>>>,
    /// O último ponto REAL do caminho e o rumo dele (unitário) — a 2.ª ponta vai À FRENTE ([`PainterTool::dual_estende`]).
    ultimo: [f32; 2],
    rumo: [f32; 2],
    /// A curvatura do caminho (rad/px), alisada ao longo de meio raio — a frente da 2.ª ponta segue o ARCO.
    curva: f32,
}

impl DualEstado {
    /// A imagem da 2.ª ponta e a versão dela — o que o preview leva e a chave dele.
    pub(super) fn imagem_e_versao(&self) -> (Option<BrushTextureImage>, u64) {
        (self.imagem.clone(), self.versao)
    }

    /// **O que é de UM traço** (W16 × L6): o motor, as marcas, o sorteio, a máscara e o rumo trocam com o traço
    /// guardado ([`super::tracos`]); a imagem, a cache e a versão são da ponta e ficam.
    pub(super) fn troca_o_traco(&mut self, o: &mut Self) {
        swap(&mut self.motor, &mut o.motor);
        swap(&mut self.pendentes, &mut o.pendentes);
        swap(&mut self.tex_rng, &mut o.tex_rng);
        swap(&mut self.no_fim, &mut o.no_fim);
        swap(&mut self.mascara, &mut o.mascara);
        swap(&mut self.ultimo, &mut o.ultimo);
        swap(&mut self.rumo, &mut o.rumo);
        swap(&mut self.curva, &mut o.curva);
    }

    /// Instala a imagem da 2.ª ponta (a do pincel copiado para o tool do preview).
    pub(super) fn poe_imagem(&mut self, imagem: Option<BrushTextureImage>) {
        self.imagem = imagem;
        self.versao = self.versao.wrapping_add(1);
        self.cache = None;
    }
}

/// ⭐ **O recorte da 2.ª ponta no pigmento do Wet** (L6): a silhueta `s` de um dab que pousa, no píxel `(px, py)`,
/// recortada pela máscara do traço — o que pousa (e o relevo que o dab penteia) é `modo(s, m)`.
pub(super) struct RecorteNoWet {
    modo: DualMode,
    mascara: Arc<Vec<u8>>,
    w: usize,
    h: usize,
}

impl RecorteNoWet {
    /// A máscara no píxel de canvas `(px, py)`.
    #[inline]
    fn m(&self, px: f32, py: f32) -> f32 {
        let x = (px.max(0.0) as usize).min(self.w - 1);
        let y = (py.max(0.0) as usize).min(self.h - 1);
        f32::from(self.mascara[(y * self.w + x) * 4 + 3]) / 255.0
    }

    /// ⭐ **O canal `padrao` do motor com o recorte**: o que pousa numa célula × `modo(s, m) / s` — DEPOIS do portão do
    /// papel, como o padrão do Shape (a lei B do plano 27), sem mexer na silhueta: recortar a silhueta deixava o motor
    /// pousar a carga do dab inteira no que sobrava (manchas — medido na prancha do pincel seco, 20/10).
    pub(super) fn padrao<'a>(
        &'a self,
        padrao: Option<ph2d_wet_paint::brush::CellFn<'a>>,
        sil: ph2d_wet_paint::brush::CellFn<'a>,
        ratio: u8,
    ) -> impl Fn(i32, i32) -> f64 + Sync + 'a {
        let r = f32::from(ratio.max(1));
        move |x, y| {
            let centro = |c: i32| super::wetpaint::grid_map::cell_subsample_px(c, ratio, 0, r);
            let m = self.m(centro(x), centro(y));
            // O Multiply não depende da ponta (`a·m / a = m`): sem reavaliar a silhueta da célula (medido: no Wet
            // a silhueta com o AA é o que pesa).
            let f = if self.modo == DualMode::Multiply {
                f64::from(m)
            } else {
                let s = sil(x, y);
                if s > 0.0 {
                    f64::from(self.modo.recorta(s as f32, m)) / s
                } else {
                    0.0
                }
            };
            padrao.map_or(1.0, |p| p(x, y)) * f
        }
    }
}

/// O ponto a `l` px de `p` ao longo do arco de rumo `rumo` (unitário) e curvatura `k` (rad/px; no máximo meia volta).
fn frente_no_arco(p: [f32; 2], rumo: [f32; 2], k: f32, l: f32) -> [f32; 2] {
    let kl = (k * l).clamp(-std::f32::consts::PI, std::f32::consts::PI);
    if kl.abs() < 1e-3 {
        return [p[0] + rumo[0] * l, p[1] + rumo[1] * l];
    }
    let (t0, k) = (rumo[1].atan2(rumo[0]), kl / l);
    [
        p[0] + ((t0 + kl).sin() - t0.sin()) / k,
        p[1] - ((t0 + kl).cos() - t0.cos()) / k,
    ]
}

impl PainterTool {
    /// O motor da 2.ª ponta nasce no pen-down de um traço que a leva (o plano arma), e pousa as primeiras marcas.
    /// `tilt` é a caneta do evento: a 2.ª ponta deita-se com a 1.ª (J3, `Pinceis/exames/grafite.md`).
    pub(super) fn dual_comeca(&mut self, p: StrokePoint, tilt: [f32; 2]) {
        self.paint.dual.motor = None;
        self.paint.dual.pendentes.clear();
        self.paint.dual.mascara = None;
        let Some(d) = self.paint.brush.dual else {
            return;
        };
        if self.dual_recusado().is_some() {
            return;
        }
        let mut dinamica = self.dinamica_do_traco();
        dinamica.size_pressure = false;
        dinamica.strength_pressure = false;
        dinamica.speed_size = 0.0;
        dinamica.speed_opacity = 0.0;
        let mut m = Stroke::new(
            d.pincel(&self.stroke_spec()),
            dinamica,
            self.paint.seed ^ SEMENTE_DO_DUAL,
        );
        self.paint.dual.tex_rng = self.paint.seed ^ SEMENTE_DO_DUAL.rotate_left(17);
        let mut out = Vec::new();
        m.set_tilt(tilt);
        (
            self.paint.dual.ultimo,
            self.paint.dual.rumo,
            self.paint.dual.curva,
        ) = (p.pos, [0.0; 2], 0.0);
        m.begin(p, &mut out);
        self.paint.dual.pendentes.extend(out);
        self.paint.dual.motor = Some(m);
    }

    /// Um passo do motor da 2.ª ponta (o movimento, o tique, o assentar, o fim) — as marcas ficam pendentes.
    pub(super) fn dual_passo(&mut self, f: impl FnOnce(&mut Stroke, &mut Vec<Dab>)) {
        let Some(mut m) = self.paint.dual.motor.take() else {
            return;
        };
        let mut out = Vec::new();
        f(&mut m, &mut out);
        self.paint.dual.pendentes.extend(out);
        self.paint.dual.motor = Some(m);
    }

    /// O movimento `p`: a dobra da arte relida (como a da 1.ª) e o caminho estendido — ⭐ À FRENTE do ponto pelo raio da
    /// 1.ª, no rumo do último passo (J3, `Pinceis/exames/grafite.md` 3.ª ronda): a 1.ª ponta carimba-se logo a seguir e
    /// a frente dela chega a `p + raio`; com a máscara só até `p` cada dab do traço LENTO perdia a metade da frente
    /// (Multiply sobre o vazio) e o rápido (os dabs do segmento todos atrás de `p`) saía 7–12 % mais escuro — o Rebelle
    /// dá os dois iguais. A frente segue o ARCO da curvatura medida (em linha recta saía da curva apertada do miolo da
    /// espiral: a máscara fora do caminho e o miolo BRANCO); o que passa do fim não tem a 1.ª por baixo.
    pub(super) fn dual_estende(&mut self, p: StrokePoint, tilt: [f32; 2]) {
        let (Some(d), Some(autorado)) = (self.paint.brush.dual, self.paint.stroke_authored) else {
            return;
        };
        let primeira = self.compose_canvas_warp(autorado);
        let vivo = d.pincel(&primeira);
        let e = &mut self.paint.dual;
        let frente = primeira.clamped_radius();
        let passo = [p.pos[0] - e.ultimo[0], p.pos[1] - e.ultimo[1]];
        let l = passo[0].hypot(passo[1]);
        if l > 1e-3 {
            let novo = [passo[0] / l, passo[1] / l];
            if e.rumo != [0.0; 2] {
                let giro = (e.rumo[0] * novo[1] - e.rumo[1] * novo[0])
                    .atan2(e.rumo[0] * novo[0] + e.rumo[1] * novo[1]);
                e.curva += (1.0 - (-l / (0.5 * frente)).exp()) * (giro / l - e.curva);
            }
            e.rumo = novo;
        }
        e.ultimo = p.pos;
        let p = StrokePoint {
            pos: frente_no_arco(p.pos, e.rumo, e.curva, frente),
            ..p
        };
        self.dual_passo(|m, out| {
            m.set_canvas_dab(
                vivo.radius_px,
                vivo.dab_flatten,
                vivo.dab_angle_deg,
                vivo.dab_curve,
            );
            m.set_tilt(tilt);
            m.extend(p, out);
        });
    }

    /// ⭐ **A porta do dual**: porque é que a 2.ª ponta NÃO arma agora (`None` = arma). No Wet Paint arma quando o
    /// pincel POUSA pigmento (a ferramenta Paint, sem a borracha, num traço livre); nos outros meios é a do plano.
    pub(crate) fn dual_recusado(&self) -> Option<super::traco_plano::SemPlano> {
        if self.paint_media() == crate::PaintMedia::WetPaint {
            let pousa = matches!(self.paint.paint_mode, super::PaintMode::WetPaint)
                && self.paint.wetpaint.tool == super::WetTool::Paint
                && !self.paint.eraser
                && self.paint.brush.stroke_method.is_incremental();
            return (!pousa).then_some(super::traco_plano::SemPlano::ForaDoTraco);
        }
        self.plano_recusado()
    }

    /// O recorte do lote do Wet que pousa pigmento (`pousa`): as marcas pendentes vão para a máscara do traço (nasce
    /// vazia: com o dual armado e nenhuma marca, em Multiply, nada pousa). `None` sem o dual.
    pub(super) fn recorte_do_dual_no_wet(&mut self, pousa: bool) -> Option<RecorteNoWet> {
        let d = self
            .paint
            .brush
            .dual
            .filter(|_| pousa && self.dual_recusado().is_none())?;
        let mut m = self.paint.dual.mascara.take();
        self.carimba_a_segunda(&mut m);
        let n4 = self.canvas_rgba.len();
        let mascara = m.unwrap_or_else(|| Arc::new(vec![0u8; n4]));
        self.paint.dual.mascara = Some(Arc::clone(&mascara));
        let (w, h) = self.source_size;
        Some(RecorteNoWet {
            modo: d.modo,
            mascara,
            w: (w as usize).max(1),
            h: (h as usize).max(1),
        })
    }

    /// O fim do traço a descarregar as marcas pendentes (`descarrega_o_dual`).
    pub(super) fn dual_no_fim(&self) -> bool {
        self.paint.dual.no_fim && self.dual_pendente()
    }

    /// Corre `f` como o fim do traço (as marcas pendentes compõem-se sem um lote da 1.ª).
    pub(super) fn como_fim_do_dual(&mut self, f: impl FnOnce(&mut Self)) {
        self.paint.dual.no_fim = true;
        f(self);
        self.paint.dual.no_fim = false;
    }

    /// Há marcas da 2.ª ponta à espera de lote.
    pub(super) fn dual_pendente(&self) -> bool {
        !self.paint.dual.pendentes.is_empty()
    }

    /// A janela das marcas pendentes (o alcance é o da ponta DELA).
    pub(super) fn regiao_do_dual(&mut self) -> Option<Region> {
        if self.paint.dual.pendentes.is_empty() {
            return None;
        }
        let d = self.paint.brush.dual?;
        let pendentes = std::mem::take(&mut self.paint.dual.pendentes);
        let r = self.com_a_segunda(d, |t| t.dab_batch_region(&pendentes));
        self.paint.dual.pendentes = pendentes;
        r
    }

    /// ⭐ **Carimba as marcas pendentes na MÁSCARA** do traço (criada transparente no 1.º lote); devolve a janela que
    /// mudou.
    pub(super) fn carimba_a_segunda(
        &mut self,
        mascara: &mut Option<Arc<Vec<u8>>>,
    ) -> Option<Region> {
        let d = self.paint.brush.dual?;
        if self.paint.dual.pendentes.is_empty() {
            return None;
        }
        let dabs = std::mem::take(&mut self.paint.dual.pendentes);
        let n4 = self.canvas_rgba.len();
        let m = mascara.get_or_insert_with(|| Arc::new(vec![0u8; n4]));
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, m, &self.undo.write_state);
        let r = self.com_a_segunda(d, |t| {
            let r = t.dab_batch_region(&dabs);
            t.stamp_dabs_routed(&dabs);
            r
        });
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, m, &self.undo.write_state);
        r
    }

    /// Corre `f` com a 2.ª ponta no lugar da 1.ª: o pincel dela, a imagem dela no lugar da Shape, camadas vazias, sem
    /// rampas de cor, a cache e o sorteio de textura dela; tudo volta ao sair.
    fn com_a_segunda<R>(&mut self, d: DualTip, f: impl FnOnce(&mut Self) -> R) -> R {
        let p = &mut self.paint;
        let pincel = p.brush;
        let guardado = (
            p.acumulando_no_plano,
            p.shape_color_ramp_enabled,
            p.texture_ramp_enabled,
            p.shape_image_version,
        );
        // A máscara carimba-se pelas rotas do carimbo da 2.ª ponta: nunca pelo fluido (no Wet o modo é o dele) nem pelo
        // Composite (que o modo Paint acordaria); e o mapa das cópias da costura (`dab_groups`) que o lote da 1.ª
        // ponta lê a seguir volta intacto (as rotas limpam-no).
        let modo = std::mem::replace(&mut p.paint_mode, super::PaintMode::Paint);
        let composto = std::mem::replace(&mut p.composite_enabled, false);
        let grupos = std::mem::take(&mut p.dab_groups);
        p.brush = d.pincel(&pincel);
        p.acumulando_no_plano = true;
        p.shape_color_ramp_enabled = false;
        p.texture_ramp_enabled = false;
        p.shape_image_version = VERSAO_DO_DUAL | p.dual.versao;
        swap(&mut p.shape_image, &mut p.dual.imagem);
        swap(&mut p.shape_layers, &mut p.dual.camadas);
        swap(&mut p.stamp_cache, &mut p.dual.cache);
        swap(&mut p.tex_rng, &mut p.dual.tex_rng);
        let r = f(self);
        let p = &mut self.paint;
        swap(&mut p.shape_image, &mut p.dual.imagem);
        swap(&mut p.shape_layers, &mut p.dual.camadas);
        swap(&mut p.stamp_cache, &mut p.dual.cache);
        swap(&mut p.tex_rng, &mut p.dual.tex_rng);
        p.brush = pincel;
        p.paint_mode = modo;
        p.composite_enabled = composto;
        p.dab_groups = grupos;
        (
            p.acumulando_no_plano,
            p.shape_color_ramp_enabled,
            p.texture_ramp_enabled,
            p.shape_image_version,
        ) = guardado;
        r
    }

    /// A imagem da 2.ª ponta (silhueta cinzenta `w × h`): a forma dela passa a `Image`.
    pub fn set_brush_dual_image(&mut self, lum: Vec<u8>, w: u32, h: u32) {
        self.macro_recusa("set_brush_dual_image");
        self.instala_a_imagem_do_dual(lum, w, h);
    }

    /// A imagem da 2.ª ponta a partir de RGBA (o seletor de ficheiro da shell): a silhueta pela LUMINÂNCIA, a lei
    /// da importação da Shape ([`super::mangueira::importa_rgba`]).
    pub fn set_brush_dual_image_rgba(&mut self, rgba: &[ph2d_color::SrgbRgba], w: u32, h: u32) {
        let rgba: &[u8] = bytemuck::cast_slice(rgba);
        // A macro guarda a IMAGEM (como a da ponta de N imagens, `Passo::MangueiraRgba`).
        crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::DualRgba {
                rgba: rgba.to_vec(),
                w,
                h,
            },
            { self.importa_a_imagem_do_dual(rgba, w, h) }
        )
    }

    fn importa_a_imagem_do_dual(&mut self, rgba: &[u8], w: u32, h: u32) {
        let mut c = ShapeLayers::default();
        if super::mangueira::importa_rgba(&mut c, rgba, w, h)
            && let Some((lum, w, h)) = c.flatten()
        {
            self.instala_a_imagem_do_dual(lum, w, h);
        }
    }

    fn instala_a_imagem_do_dual(&mut self, lum: Vec<u8>, w: u32, h: u32) {
        if lum.len() != (w as usize) * (h as usize) || lum.is_empty() {
            return;
        }
        let e = &mut self.paint.dual;
        e.imagem = Some(BrushTextureImage::new(lum, w, h));
        e.versao = e.versao.wrapping_add(1);
        e.cache = None;
        if let Some(d) = &mut self.paint.brush.dual {
            d.forma.kind = TextureKind::Image;
        }
    }

    /// A 2.ª ponta tem imagem.
    #[must_use]
    pub fn brush_dual_has_image(&self) -> bool {
        self.paint.dual.imagem.is_some()
    }

    /// O botão «Dual image» pede o seletor de ficheiro à shell.
    pub(super) fn request_brush_dual_image(&mut self) {
        self.paint.dual.pedido = true;
    }

    /// Toma (e limpa) o pedido do seletor: a shell abre-o e chama [`Self::set_brush_dual_image_rgba`].
    pub fn take_brush_dual_image_request(&mut self) -> bool {
        // Gravado: o tocador consome o pedido (o «Dual image» tocado nunca abre o seletor da shell).
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::TiraPedidoDual, {
            std::mem::take(&mut self.paint.dual.pedido)
        })
    }

    /// A imagem da 2.ª ponta no preset (só com a forma `Image`).
    pub(super) fn texturas_do_dual(&self, t: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        let forma_imagem = self
            .paint
            .brush
            .dual
            .is_some_and(|d| d.forma.kind == TextureKind::Image);
        if let (true, Some(img)) = (forma_imagem, self.paint.dual.imagem.as_ref()) {
            let (lum, w, h) = img.parts();
            t.insert(
                TEXTURA_DO_DUAL.to_owned(),
                ph2d_brush_presets::escrever_textura(lum, w, h),
            );
        }
    }

    /// Instala a imagem da 2.ª ponta de um preset; uma forma `Image` sem a textura no ficheiro volta ao disco e o id
    /// vai para `falhas`.
    pub(super) fn instala_o_dual(
        &mut self,
        t: &std::collections::BTreeMap<String, Vec<u8>>,
        falhas: &mut Vec<String>,
    ) {
        self.paint.dual.imagem = None;
        self.paint.dual.cache = None;
        self.paint.dual.versao = self.paint.dual.versao.wrapping_add(1);
        if !self
            .paint
            .brush
            .dual
            .is_some_and(|d| d.forma.kind == TextureKind::Image)
        {
            return;
        }
        match t
            .get(TEXTURA_DO_DUAL)
            .and_then(|b| ph2d_brush_presets::ler_textura(b).ok())
        {
            Some((lum, w, h)) => self.instala_a_imagem_do_dual(lum, w, h),
            None => {
                if let Some(d) = &mut self.paint.brush.dual {
                    d.forma.kind = TextureKind::None;
                }
                falhas.push(TEXTURA_DO_DUAL.to_owned());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DualEstado;

    /// Dois traços sobrepostos com o dual não partilham o motor nem as marcas; a imagem da ponta não viaja.
    #[test]
    fn o_traco_leva_o_motor_e_as_marcas_e_a_imagem_fica() {
        let (mut a, mut b) = (DualEstado::default(), DualEstado::default());
        a.tex_rng = 7;
        a.no_fim = true;
        a.rumo = [1.0, 0.0];
        a.versao = 5;
        b.tex_rng = 9;
        b.curva = 0.5;
        b.versao = 11;
        a.troca_o_traco(&mut b);
        assert_eq!(
            (a.tex_rng, a.no_fim, a.rumo, a.curva),
            (9, false, [0.0; 2], 0.5)
        );
        assert_eq!(
            (b.tex_rng, b.no_fim, b.rumo, b.curva),
            (7, true, [1.0, 0.0], 0.0)
        );
        assert_eq!(
            (a.versao, b.versao),
            (5, 11),
            "a versão da imagem é da ponta"
        );
    }
}
