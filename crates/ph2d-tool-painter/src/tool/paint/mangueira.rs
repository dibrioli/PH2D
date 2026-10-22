//! ⭐ **A ponta de N IMAGENS no tool** (L19, `docs/Painter/Pinceis/03_MOTORES.md` §4; a lei da escolha em
//! [`ph2d_painter_brush::mangueira`]). A imagem 0 é a Shape de sempre (`PaintState::shape_image`/`shape_layers`); as
//! outras moram aqui, cada uma com o MESMO par (silhueta + camadas com a cor dela). Cada dab leva o índice na
//! pegada (`DabTilt::imagem`), e [`PainterTool::stamp_dabs_mangueira`] carimba o lote em CORRIDAS da mesma imagem,
//! trocando-a para o lugar da Shape — todas as rotas (Digital, Impasto, Wet, as caches) carimbam a imagem certa
//! sem saberem que há várias. Cada imagem tem versões PRÓPRIAS (`VERSAO_DA_MANGUEIRA`): uma cache nunca confunde
//! duas. A imagem que entra segue os interruptores da Shape (Per-Layer Color, a silhueta pelo alpha).

use std::mem::swap;

use ph2d_painter_brush::{Dab, HOSE_MAX, TextureKind};

use super::PainterTool;

/// Uma imagem da ponta: a silhueta cinzenta `(lum, w, h)` e o RGBA `(rgba, w, h)` quando há cor.
type ImagemDaPonta<'a> = ((&'a [u8], u32, u32), Option<(Vec<u8>, u32, u32)>);
use super::brush_image::BrushTextureImage;
use super::shape_layers::ShapeLayers;

/// O bit das versões das imagens da ponta: nunca coincide com a versão da Shape principal (que conta de 0).
const VERSAO_DA_MANGUEIRA: u64 = 1 << 62;

/// Uma imagem da ponta (as de índice `1..`): a silhueta e as camadas com a cor.
#[derive(Clone)]
pub(super) struct Imagem {
    image: Option<BrushTextureImage>,
    layers: ShapeLayers,
    uid: u64,
}

/// As imagens além da Shape principal e o pedido do seletor de ficheiro.
#[derive(Clone, Default)]
pub(super) struct Mangueira {
    extras: Vec<Imagem>,
    uid: u64,
    pedido: bool,
    /// O lote já está a ser carimbado em corridas (a troca não se aninha).
    dentro: bool,
}

impl Mangueira {
    /// As imagens da ponta, sem o pedido nem o trinco do lote — o que o preview leva.
    pub(super) fn so_as_imagens(&self) -> Self {
        Self {
            extras: self.extras.clone(),
            uid: self.uid,
            ..Self::default()
        }
    }

    /// Muda quando uma imagem entra ou sai (a chave do preview).
    pub(super) fn chave(&self) -> (u64, usize) {
        (self.uid, self.extras.len())
    }
}

/// ⭐ **A lei da importação RGBA** da Shape — a MESMA para a imagem única e para cada imagem da ponta: a silhueta
/// pela LUMINÂNCIA (uma imagem importada pode não ter alpha; o alpha fica guardado para o interruptor), a cor
/// capturada por camada. `false` = a imagem não serve (vazia ou curta).
pub(super) fn importa_rgba(c: &mut ShapeLayers, rgba: &[u8], w: u32, h: u32) -> bool {
    let n = (w as usize) * (h as usize);
    if n == 0 || rgba.len() < n * 4 {
        return false;
    }
    let alpha: Vec<u8> = (0..n).map(|i| rgba[i * 4 + 3]).collect();
    let rgb: Vec<u8> = (0..n)
        .flat_map(|i| [rgba[i * 4], rgba[i * 4 + 1], rgba[i * 4 + 2]])
        .collect();
    c.set_layers(vec![(alpha, w, h)]);
    c.set_layers_meta(vec![rgb], vec![1.0], vec![0], vec![0]);
    c.set_alpha_from_image(false);
    true
}

impl PainterTool {
    /// Quantas imagens a Shape leva (`0` sem imagem; `1` = a de sempre).
    #[must_use]
    pub fn brush_shape_hose_count(&self) -> usize {
        if self.paint.brush.shape.kind == TextureKind::Image && self.paint.shape_image.is_some() {
            1 + self.paint.mangueira.extras.len()
        } else {
            0
        }
    }

    /// O pincel sabe quantas imagens há (a escolha do dab é dele).
    fn sincroniza_a_mangueira(&mut self) {
        self.paint.brush.hose.n = self.brush_shape_hose_count().max(1) as u8;
    }

    /// Esvazia as imagens além da principal (a Shape mudou de fonte).
    pub(super) fn esvazia_a_mangueira(&mut self) {
        self.paint.mangueira.extras.clear();
        self.paint.brush.hose.n = 1;
    }

    /// ⭐ **Acrescenta uma imagem RGBA** à ponta (o botão «Add image» da secção Shape, pelo seletor da shell). Sem
    /// imagem na Shape ela É a primeira (a porta de sempre); acima de [`HOSE_MAX`] é recusada.
    pub fn add_brush_shape_hose_image_rgba(
        &mut self,
        rgba: &[ph2d_color::SrgbRgba],
        w: u32,
        h: u32,
    ) {
        let rgba: &[u8] = bytemuck::cast_slice(rgba);
        // A macro guarda a IMAGEM (como a da Shape, `Passo::FormaRgba`): o tocador acrescenta-a sem seletor.
        crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::MangueiraRgba {
                rgba: rgba.to_vec(),
                w,
                h,
            },
            { self.empurra_rgba(rgba, w, h) }
        )
    }

    /// [`Self::add_brush_shape_hose_image_rgba`] sem a porta da macro (o preset instala por aqui).
    pub(super) fn empurra_rgba(&mut self, rgba: &[u8], w: u32, h: u32) {
        if self.brush_shape_hose_count() == 0 {
            self.set_brush_shape_image_rgba_corpo(rgba, w, h, None);
            return self.sincroniza_a_mangueira();
        }
        let mut layers = ShapeLayers::default();
        if self.brush_shape_hose_count() >= HOSE_MAX || !importa_rgba(&mut layers, rgba, w, h) {
            return;
        }
        let image = layers
            .flatten()
            .map(|(lum, w, h)| BrushTextureImage::new(lum, w, h));
        self.empurra(image, layers);
    }

    /// Acrescenta uma SILHUETA (cinzento, `w × h`) à ponta — a forma como o preset a leva sem cor.
    pub fn add_brush_shape_hose_image(&mut self, lum: Vec<u8>, w: u32, h: u32) {
        self.macro_recusa("add_brush_shape_hose_image");
        self.empurra_lum(lum, w, h);
    }

    /// [`Self::add_brush_shape_hose_image`] sem a porta da macro (o preset instala por aqui).
    pub(super) fn empurra_lum(&mut self, lum: Vec<u8>, w: u32, h: u32) {
        if self.brush_shape_hose_count() == 0 {
            self.set_brush_shape_image(lum, w, h);
            return self.sincroniza_a_mangueira();
        }
        if self.brush_shape_hose_count() < HOSE_MAX && lum.len() == (w as usize) * (h as usize) {
            self.empurra(
                Some(BrushTextureImage::new(lum, w, h)),
                ShapeLayers::default(),
            );
        }
    }

    fn empurra(&mut self, image: Option<BrushTextureImage>, layers: ShapeLayers) {
        let m = &mut self.paint.mangueira;
        m.uid += 1;
        let uid = m.uid;
        m.extras.push(Imagem { image, layers, uid });
        self.sincroniza_a_mangueira();
        self.sincroniza_as_imagens();
    }

    /// ⭐ As imagens da ponta seguem os interruptores da Shape (Per-Layer Color, a silhueta pelo alpha) — a
    /// silhueta de cada uma é a que se carimba E a que o preset grava. Chamada por quem os vira.
    pub(super) fn sincroniza_as_imagens(&mut self) {
        let (por_camada, pelo_alpha) = (
            self.paint.shape_layers.per_layer_color(),
            self.paint.shape_layers.alpha_from_image(),
        );
        for e in &mut self.paint.mangueira.extras {
            if e.layers.is_empty() {
                continue;
            }
            if e.layers.per_layer_color() != por_camada {
                e.layers.toggle_per_layer_color();
            }
            if e.layers.alpha_from_image() != pelo_alpha {
                e.layers.set_alpha_from_image(pelo_alpha);
                e.image = e
                    .layers
                    .flatten()
                    .map(|(lum, w, h)| BrushTextureImage::new(lum, w, h));
            }
        }
    }

    /// Tira as imagens além da principal (o botão «Clear images»).
    pub fn clear_brush_shape_hose(&mut self) {
        self.esvazia_a_mangueira();
    }

    /// O botão «Add image» pede o seletor de ficheiro à shell.
    pub(super) fn request_brush_shape_hose_image(&mut self) {
        self.paint.mangueira.pedido = true;
    }

    /// Toma (e limpa) o pedido do seletor: a shell abre-o e chama [`Self::add_brush_shape_hose_image_rgba`].
    pub fn take_brush_shape_hose_request(&mut self) -> bool {
        // Gravado: o tocador consome o pedido (o «Add image» tocado nunca abre o seletor da shell).
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::TiraPedidoMangueira, {
            std::mem::take(&mut self.paint.mangueira.pedido)
        })
    }

    /// A imagem `k` da ponta como a preset a leva: `(silhueta cinzenta, w, h)` e o RGBA quando há cor.
    pub(super) fn imagem_da_mangueira(&self, k: usize) -> Option<ImagemDaPonta<'_>> {
        if k == 0 {
            let img = self.paint.shape_image.as_ref()?;
            return Some((img.parts(), self.paint.shape_layers.rgba_da_imagem()));
        }
        let e = self.paint.mangueira.extras.get(k - 1)?;
        Some((e.image.as_ref()?.parts(), e.layers.rgba_da_imagem()))
    }

    /// A ponta tem mais de uma imagem a carimbar neste lote (e não está já a carimbar uma corrida).
    pub(super) fn mangueira_viva(&self) -> bool {
        !self.paint.mangueira.dentro && self.brush_shape_hose_count() > 1
    }

    /// ⭐ **O lote em CORRIDAS da mesma imagem**: cada corrida é carimbada com a sua imagem no lugar da Shape.
    pub(super) fn stamp_dabs_mangueira(&mut self, dabs: &[Dab]) {
        self.stamp_dabs_mangueira_com(dabs, |t, corrida| t.stamp_dabs_dispatch(corrida));
    }

    /// [`Self::stamp_dabs_mangueira`] com o carimbo `f` de cada corrida — o Wet carimba por aqui também o lote
    /// ADIADO, que pousa no tique, fora da porta do lote (`wetpaint::dab_route`).
    pub(super) fn stamp_dabs_mangueira_com(
        &mut self,
        dabs: &[Dab],
        mut f: impl FnMut(&mut Self, &[Dab]),
    ) {
        let n = self.brush_shape_hose_count();
        let k_de = |d: &Dab| d.tilt.map_or(0, |t| usize::from(t.imagem)).min(n - 1);
        self.paint.mangueira.dentro = true;
        let mut i = 0;
        while i < dabs.len() {
            let k = k_de(&dabs[i]);
            let j = (i..dabs.len())
                .find(|&j| k_de(&dabs[j]) != k)
                .unwrap_or(dabs.len());
            self.com_imagem(k, |t| f(t, &dabs[i..j]));
            i = j;
        }
        self.paint.mangueira.dentro = false;
    }

    /// Corre `f` com a imagem `k` no lugar da Shape (e devolve a principal ao sair).
    fn com_imagem(&mut self, k: usize, f: impl FnOnce(&mut Self)) {
        if k == 0 || k > self.paint.mangueira.extras.len() {
            return f(self);
        }
        let p = &mut self.paint;
        let (versao, versao_camadas) = (p.shape_image_version, p.shape_layers.version());
        let e = &mut p.mangueira.extras[k - 1];
        let uid = e.uid;
        swap(&mut p.shape_image, &mut e.image);
        swap(&mut p.shape_layers, &mut e.layers);
        let marca = VERSAO_DA_MANGUEIRA | (uid << 32);
        p.shape_layers
            .set_version(marca | (versao_camadas & 0xFFFF_FFFF));
        p.shape_image_version = marca | (versao & 0xFFFF_FFFF);
        f(self);
        let p = &mut self.paint;
        let e = &mut p.mangueira.extras[k - 1];
        swap(&mut p.shape_image, &mut e.image);
        swap(&mut p.shape_layers, &mut e.layers);
        p.shape_image_version = versao;
    }
}

/// O id da textura da imagem `k` no preset (`shape` = a primeira, a de sempre; `shape_2`… as outras).
fn id_da_imagem(k: usize) -> String {
    if k == 0 {
        "shape".to_owned()
    } else {
        format!("shape_{}", k + 1)
    }
}

/// O id da COR da imagem `k` (o RGBA; `_alpha` = a silhueta pelo alpha da imagem, o interruptor da Shape).
fn id_da_cor(k: usize, alfa: bool) -> String {
    format!(
        "{}_rgba{}",
        id_da_imagem(k),
        if alfa { "_alpha" } else { "" }
    )
}

impl PainterTool {
    /// ⭐ **As imagens da ponta no preset** (L19): as silhuetas além da primeira (que vai como `shape`, a de sempre)
    /// e, com o modo de cor da Shape ligado, a COR de cada uma (RGBA) — o preset volta ao bit.
    pub(super) fn texturas_da_mangueira(
        &self,
        t: &mut std::collections::BTreeMap<String, Vec<u8>>,
    ) {
        let cor = self.paint.shape_layers.is_color_mode();
        let alfa = self.paint.shape_layers.alpha_from_image();
        for k in 0..self.brush_shape_hose_count() {
            let Some(((lum, w, h), rgba)) = self.imagem_da_mangueira(k) else {
                continue;
            };
            if k > 0 {
                t.insert(
                    id_da_imagem(k),
                    ph2d_brush_presets::escrever_textura(lum, w, h),
                );
            }
            if let (true, Some((rgba, w, h))) = (cor, rgba) {
                t.insert(
                    id_da_cor(k, alfa),
                    ph2d_brush_presets::escrever_textura_rgba(&rgba, w, h),
                );
            }
        }
    }

    /// Instala as imagens da ponta de um preset (depois da `shape` de sempre): a cor da primeira, as outras pela
    /// cor ou pela silhueta; `n` vem do campo `hose` do pincel. As que faltam vão para `falhas`.
    pub(super) fn instala_a_mangueira(
        &mut self,
        t: &std::collections::BTreeMap<String, Vec<u8>>,
        falhas: &mut Vec<String>,
    ) {
        let n = usize::from(self.paint.brush.hose.n).clamp(1, HOSE_MAX);
        self.paint.mangueira.extras.clear();
        if self.brush_shape_hose_count() == 0 {
            return self.sincroniza_a_mangueira();
        }
        let alfa = t.keys().any(|k| k.ends_with("_rgba_alpha"));
        let cor = |k: usize| {
            t.get(&id_da_cor(k, alfa))
                .and_then(|b| ph2d_brush_presets::ler_textura_rgba(b).ok())
        };
        if let Some((rgba, w, h)) = cor(0)
            && importa_rgba(&mut self.paint.shape_layers, &rgba, w, h)
        {
            if !self.paint.shape_layers.per_layer_color() {
                self.paint.shape_layers.toggle_per_layer_color();
            }
            self.paint.shape_layers.set_alpha_from_image(alfa);
            self.reflatten_shape_image();
        }
        for k in 1..n {
            if let Some((rgba, w, h)) = cor(k) {
                self.empurra_rgba(&rgba, w, h);
            } else if let Some((lum, w, h)) = t
                .get(&id_da_imagem(k))
                .and_then(|b| ph2d_brush_presets::ler_textura(b).ok())
            {
                self.empurra_lum(lum, w, h);
            } else {
                falhas.push(id_da_imagem(k));
            }
        }
        self.sincroniza_a_mangueira();
    }
}
