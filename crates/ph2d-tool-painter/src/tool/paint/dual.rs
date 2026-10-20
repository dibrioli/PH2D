//! ⭐ **O DUAL BRUSH no tool** (L6, `docs/Painter/Pinceis/03_MOTORES.md` §4; a lei em [`ph2d_painter_brush::dual`]).
//!
//! A 2.ª ponta tem um MOTOR de traço próprio ([`Stroke`] com o pincel [`DualTip::pincel`]) alimentado pelos MESMOS
//! pontos que o da 1.ª (pen-down, cada movimento, o tique, o assentar e o fim) — o mesmo caminho, o passo, o espalhar,
//! a contagem e a rotação dela. As marcas que ele emite esperam em `pendentes` até ao lote seguinte do PLANO do
//! traço (`super::traco_plano`), que as carimba na MÁSCARA do traço (um plano RGBA, a máscara no alfa) por TODAS as
//! rotas de sempre — trocando, só durante esse carimbo, a tela pela máscara, o pincel pelo dela, a Shape pela imagem
//! dela e a cache do carimbo pela dela — e compõe cada texel com a cobertura da 1.ª recortada (`DualMode::recorta`).
//!
//! ⚠️ Só arma onde o plano arma (o Digital, um traço livre que pinta): a mesma porta, [`PainterTool::plano_recusado`].

use std::mem::swap;
use std::sync::Arc;

use ph2d_painter_brush::{Dab, DualTip, Stroke, StrokePoint, TextureKind};

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
}

impl PainterTool {
    /// O motor da 2.ª ponta nasce no pen-down de um traço que a leva (o plano arma), e pousa as primeiras marcas.
    pub(super) fn dual_comeca(&mut self, p: StrokePoint) {
        self.paint.dual.motor = None;
        self.paint.dual.pendentes.clear();
        let Some(d) = self.paint.brush.dual else {
            return;
        };
        if self.plano_recusado().is_some() {
            return;
        }
        let mut dinamica = self.dinamica_do_traco();
        dinamica.size_pressure = false;
        dinamica.strength_pressure = false;
        dinamica.speed_size = 0.0;
        dinamica.speed_opacity = 0.0;
        let mut m = Stroke::new(d.pincel(&self.stroke_spec()), dinamica, self.paint.seed ^ SEMENTE_DO_DUAL);
        self.paint.dual.tex_rng = self.paint.seed ^ SEMENTE_DO_DUAL.rotate_left(17);
        let mut out = Vec::new();
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

    /// O movimento `p`: a dobra da arte relida (como a da 1.ª) e o caminho estendido.
    pub(super) fn dual_estende(&mut self, p: StrokePoint) {
        let (Some(d), Some(autorado)) = (self.paint.brush.dual, self.paint.stroke_authored) else {
            return;
        };
        let vivo = d.pincel(&self.compose_canvas_warp(autorado));
        self.dual_passo(|m, out| {
            m.set_canvas_dab(vivo.radius_px, vivo.dab_flatten, vivo.dab_angle_deg, vivo.dab_curve);
            m.extend(p, out);
        });
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
    pub(super) fn carimba_a_segunda(&mut self, mascara: &mut Option<Arc<Vec<u8>>>) -> Option<Region> {
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
    pub fn set_brush_dual_image_rgba(&mut self, rgba: &[u8], w: u32, h: u32) {
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
        let forma_imagem = self.paint.brush.dual.is_some_and(|d| d.forma.kind == TextureKind::Image);
        if let (true, Some(img)) = (forma_imagem, self.paint.dual.imagem.as_ref()) {
            let (lum, w, h) = img.parts();
            t.insert(TEXTURA_DO_DUAL.to_owned(), ph2d_brush_presets::escrever_textura(lum, w, h));
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
        if !self.paint.brush.dual.is_some_and(|d| d.forma.kind == TextureKind::Image) {
            return;
        }
        match t.get(TEXTURA_DO_DUAL).and_then(|b| ph2d_brush_presets::ler_textura(b).ok()) {
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
