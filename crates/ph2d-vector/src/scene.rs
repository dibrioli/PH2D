//! [`VectorScene`] — wrapper over `vello::Scene`.
//!
//! Owns the encoded scene graph for one frame. Reset between frames
//! via [`VectorScene::reset`]. Inner `vello::Scene` exposed via
//! [`VectorScene::inner`] / [`VectorScene::inner_mut`] for callers
//! that need the full Vello API; the convenience helpers below
//! cover the 80 % case (rect / path fill).

use std::sync::Arc;
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Rect, Stroke};
use vello::peniko::{
    Blob, Brush, Color, Extend, Fill, ImageAlphaType, ImageBrush, ImageData, ImageFormat,
    ImageQuality,
};

pub struct VectorScene {
    inner: Scene,
}

/// Uma imagem RGBA já preparada como recurso **ESTÁVEL** do Vello — construída UMA vez e
/// redesenhada em vários frames.
///
/// ⚠️ **Por que existe:** [`VectorScene::draw_image_rgba`] reconstrói a `Blob` a cada chamada, e o
/// Vello dá a cada `Blob::new` um **id novo** — então ele trata a imagem como INÉDITA todo frame e
/// a RE-ENVIA ao atlas da GPU (o cache de imagem do Vello é por `data.id()`). Para uma imagem
/// grande desenhada todo frame (um FX raster, plano 24), esse re-upload+repack é uma queda de FPS
/// extrema. Guardar este handle e redesenhá-lo com [`VectorScene::draw_stable_image`] mantém o id
/// ESTÁVEL ⇒ o cache do Vello acerta e pula o upload. O produtor constrói UMA vez (no memo) e
/// clona o handle por frame (clone de `Blob` = refcount + MESMO id).
#[derive(Clone)]
pub struct StableImage {
    data: ImageData,
}

impl StableImage {
    /// Constrói a partir de RGBA **reta** (`width*height*4` bytes). `None` se as dimensões não
    /// batem. Consome o `Arc` (o handle passa a ser o dono do id estável).
    #[must_use]
    pub fn from_rgba(rgba: Arc<Vec<u8>>, width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 || rgba.len() != (width as usize) * (height as usize) * 4 {
            return None;
        }
        Some(Self {
            data: ImageData {
                data: Blob::new(rgba),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width,
                height,
            },
        })
    }

    /// Como [`Self::from_rgba`], mas para bytes **JÁ pré-multiplicados** — o par estável de
    /// [`VectorScene::draw_image_rgba_premultiplied_transformed`].
    ///
    /// ⚠️ **Não é um detalhe de conveniência: o tipo de alfa viaja DENTRO do handle.** O
    /// `from_rgba` carimba `ImageAlphaType::Alpha`, e desenhar bytes pré-multiplicados por ele faz
    /// o Vello multiplicar **outra vez** — a borda escurece e some. Quem produz pela fórmula
    /// canónica (`ph2d_render::premultiply_rgba8`) tem de entrar por aqui.
    ///
    /// ⭐ **Porque existe** (2026-08-30): o traçado 3D produzia um `Arc` novo por traçado e
    /// desenhava-o pela porta CRUA a cada quadro — id novo por quadro, no atlas **persistente** da
    /// `vello` 0.10. Medido: uma vista `2560×1440` punha o atlas no tecto de `8192²` (**256 MB** de
    /// VRAM) e enviava **843,8 MB por segundo** de pixels que não tinham mudado.
    #[must_use]
    pub fn from_rgba_premultiplied(rgba: Arc<Vec<u8>>, width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 || rgba.len() != (width as usize) * (height as usize) * 4 {
            return None;
        }
        Some(Self {
            data: ImageData {
                data: Blob::new(rgba),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::AlphaPremultiplied,
                width,
                height,
            },
        })
    }

    /// Envolve uma [`ImageData`] JÁ construída — o caso do FX raster GPU-resident (plano 24), em
    /// que a `ImageData` vem do `Renderer::register_texture` do Vello (respaldada por uma textura
    /// da GPU, id estável) em vez de bytes de CPU. Desenhá-la amostra a textura direto, sem upload.
    #[must_use]
    pub fn from_image_data(data: ImageData) -> Self {
        Self { data }
    }

    /// A largura em px da imagem.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.data.width
    }

    /// A altura em px da imagem.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.data.height
    }

    // ⛔ **Não acrescente aqui um `probe_id` do handle.** Ele existiu por um turno e a mutação que
    // o devia matar SOBREVIVEU: a identidade que interessa não é a que o produtor GUARDA, é a que
    // a **cena EMITE** — [`VectorScene::probe_image_ids`].
}

impl VectorScene {
    /// Desenha um [`StableImage`] no retângulo de tela `dest = (x0,y0,x1,y1)` — o id estável faz o
    /// Vello reusar a textura do atlas em vez de re-enviá-la. É o par de [`Self::draw_image_rgba`]
    /// para quem redesenha a MESMA imagem em muitos frames.
    pub fn draw_stable_image(
        &mut self,
        image: &StableImage,
        dest: (f64, f64, f64, f64),
        quality: ImageQuality,
    ) {
        let (w, h) = (image.data.width, image.data.height);
        if w == 0 || h == 0 {
            return;
        }
        let (x0, y0, x1, y1) = dest;
        let sx = (x1 - x0) / f64::from(w);
        let sy = (y1 - y0) / f64::from(h);
        let transform = Affine::translate((x0, y0)) * Affine::scale_non_uniform(sx, sy);
        // Clone do `ImageData` = clone da `Blob` = refcount + MESMO id (o que dá o cache-hit).
        let brush = ImageBrush::new(image.data.clone()).with_quality(quality);
        self.inner.draw_image(brush.as_ref(), transform);
    }
}

impl VectorScene {
    /// ⚠️ **Só para gates: os ids de TODA imagem que esta cena vai desenhar**, em ordem de emissão.
    ///
    /// ⭐⭐ **Existe por causa de uma mutação que SOBREVIVEU.** O primeiro gate desta lei perguntava
    /// ao **estado** do produtor — *«o handle que guardaste mudou?»* — e ficava verde quando alguém
    /// acrescentava, ao lado, um desenho pela porta crua: o handle guardado continuava o mesmo, e a
    /// cena passava a cunhar uma residente nova por quadro na mesma.
    ///
    /// ⇒ *a pergunta não é o que o produtor GUARDA, é o que a cena EMITE* — e é a mesma lição que a
    /// caça aos controlos mortos deixou escrita no `CLAUDE.md` §5.0: **o terceiro passo**, o que um
    /// `grep` não vê, é se o valor chega a um consumidor.
    ///
    /// Devolve `u64` de propósito: nenhum tipo do Vello atravessa a fronteira desta crate.
    #[must_use]
    #[doc(hidden)]
    pub fn probe_image_ids(&self) -> Vec<u64> {
        self.inner
            .encoding()
            .resources
            .patches
            .iter()
            .filter_map(|p| match p {
                vello_encoding::Patch::Image { image, .. } => Some(image.data.id()),
                _ => None,
            })
            .collect()
    }

    pub fn new() -> Self {
        Self {
            inner: Scene::new(),
        }
    }

    /// Drop every encoded path. Call once at frame start before
    /// re-emitting draw commands; Vello's internal allocations are
    /// reused across frames so this is cheap.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn inner(&self) -> &Scene {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut Scene {
        &mut self.inner
    }

    /// Fill an axis-aligned rect with a solid color. The most common
    /// "draw a panel background" case.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.inner.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &rect,
        );
    }

    /// Fill an arbitrary path with a brush.
    pub fn fill_path(&mut self, path: &BezPath, brush: &Brush, transform: Affine) {
        self.inner.fill(Fill::NonZero, transform, brush, None, path);
    }

    /// Blit a straight-alpha RGBA8 bitmap into `dest` (screen-space
    /// rect, px). The image's native `width × height` pixel space is
    /// scaled to fill `dest`. Used by the Background-Removal tool to
    /// overlay its live preview on top of the sprite on-canvas (in
    /// place of the real image) without mutating the sprite's GPU
    /// texture.
    ///
    /// `rgba` is taken as an `Arc<Vec<u8>>` so the per-frame redraw
    /// shares the buffer (Arc clone, no pixel copy) with the caller's
    /// cache. Length must be `width * height * 4`; mismatched buffers
    /// are dropped (no draw) rather than panicking.
    ///
    /// `dest` is the destination screen rect as `(x0, y0, x1, y1)` in
    /// pixels (top-left, bottom-right). Taking raw coords keeps callers
    /// free of a direct `kurbo`/`vello` dependency.
    ///
    /// `quality` selects the sampling filter and MUST be derived from
    /// the app-wide `ImageFilterMode` (PixelArt → [`ImageQuality::Low`]
    /// / nearest, Smooth → [`ImageQuality::High`] / bicubic). Threading
    /// it through here is what keeps the Background-Removal preview
    /// consistent with the baked sprite: before, this defaulted to
    /// [`ImageQuality::Medium`] (bilinear), so the preview looked smooth
    /// while a PixelArt-sampled sprite was crisp.
    pub fn draw_image_rgba(
        &mut self,
        rgba: &Arc<Vec<u8>>,
        width: u32,
        height: u32,
        dest: (f64, f64, f64, f64),
        quality: ImageQuality,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        let (x0, y0, x1, y1) = dest;
        let sx = (x1 - x0) / width as f64;
        let sy = (y1 - y0) / height as f64;
        let transform = Affine::translate((x0, y0)) * Affine::scale_non_uniform(sx, sy);
        self.draw_image_rgba_transformed(rgba, width, height, transform, quality);
    }

    /// Same as [`Self::draw_image_rgba`] but takes the full image-local
    /// (pixels 0..width, 0..height) → screen-pixel affine transform.
    /// Used when the destination is rotated / scaled / sheared — e.g.
    /// the Background-Removal preview overlay tracking a rotated
    /// sprite (Enio 2026-05-26): the axis-aligned `dest` rect of
    /// `draw_image_rgba` collapses the orientation, so the overlay
    /// would visibly drift off the sprite once it spins.
    pub fn draw_image_rgba_transformed(
        &mut self,
        rgba: &Arc<Vec<u8>>,
        width: u32,
        height: u32,
        transform: Affine,
        quality: ImageQuality,
    ) {
        self.draw_image_rgba_with_alpha(
            rgba,
            width,
            height,
            transform,
            quality,
            ImageAlphaType::Alpha,
        );
    }

    /// Like [`Self::draw_image_rgba_transformed`] but treats the input
    /// RGBA as ALREADY premultiplied — Vello won't multiply again
    /// before sampling. Use when the caller has produced premultiplied
    /// bytes via the SAME formula the sprite renderer uses (the
    /// canonical `ph2d_render::premultiply_rgba8`), so the overlay
    /// composites byte-identically to the Apply path (Enio 2026-05-26:
    /// "linha clara contornando a forma" was Vello's internal premul
    /// rounding diverging from the wgpu shader's at sub-pixel edges).
    pub fn draw_image_rgba_premultiplied_transformed(
        &mut self,
        rgba: &Arc<Vec<u8>>,
        width: u32,
        height: u32,
        transform: Affine,
        quality: ImageQuality,
    ) {
        self.draw_image_rgba_with_alpha(
            rgba,
            width,
            height,
            transform,
            quality,
            ImageAlphaType::AlphaPremultiplied,
        );
    }

    fn draw_image_rgba_with_alpha(
        &mut self,
        rgba: &Arc<Vec<u8>>,
        width: u32,
        height: u32,
        transform: Affine,
        quality: ImageQuality,
        alpha_type: ImageAlphaType,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        if rgba.len() != (width as usize) * (height as usize) * 4 {
            return;
        }
        let image = ImageData {
            data: Blob::new(rgba.clone()),
            format: ImageFormat::Rgba8,
            alpha_type,
            width,
            height,
        };
        let brush = ImageBrush::new(image).with_quality(quality);
        self.inner.draw_image(brush.as_ref(), transform);
    }

    /// **Preenche `path` com uma IMAGEM LADRILHADA** — a porta do *Texture Pattern* (plano 33, W2).
    ///
    /// ⭐⭐ **Uma `fill()` e mais nada.** Sem camada de clip, sem rasterização, sem blit: a
    /// repetição é do amostrador do Vello (`x_extend`/`y_extend` viajam empacotados em
    /// `sample_alpha` e o `fine.wgsl` honra-os, aplicando o extend **antes** de somar o
    /// `atlas_offset` — o repeat dá a volta dentro do próprio ladrilho e não sangra o vizinho no
    /// atlas). ⇒ um preenchimento com padrão custa o que uma cor chapada custa.
    ///
    /// # Os dois argumentos que o [`Self::fill_path`] tem MORTOS
    ///
    /// - **`rule`** — o `fill_path` fixa `Fill::NonZero`, e num *compound path* com `EvenOdd` isso
    ///   pinta o buraco. É a mesma pedra que fez o `fill_multipoint` da `ph2d-vec-render` ter de
    ///   empurrar o clip com a regra do caminho em vez do `push_clip` normal.
    /// - **`brush_transform`** — o `fill_path` passa `None`. O Vello compõe
    ///   `transform * brush_transform`, então é ELE que põe a colocação do padrão no espaço das
    ///   ÂNCORAS e a faz cavalgar a pose da forma. Com `None` o padrão ficaria colado à TELA e
    ///   escorregaria por baixo da forma — o defeito da origem-da-régua do Illustrator.
    ///
    /// # O que o substrato faz — e o que MUDOU ao subir o vello
    ///
    /// ⭐ **`ImageQuality::High` passou a existir de verdade.** Até ao vello 0.8 ele era bilinear
    /// disfarçado, e o `fine.wgsl` dizia-o em texto (*"We don't have an implementation for
    /// `IMAGE_QUALITY_HIGH` yet, just use the same as medium"*). Do 0.9 em diante há um
    /// `bicubic_sample` a sério (Mitchell, B = C = 1/3, 16 amostras).
    /// ⚠️ **Isto muda pixel sem mudar uma linha nossa:** os dois sítios de produto que mapeiam
    /// «Smooth» para `High` (`ph2d-editor-core/src/project.rs` e `ph2d-vec-render/src/gradient.rs`)
    /// passam a ficar mais nítidos, com o ligeiro sobre-disparo que um filtro Mitchell produz numa
    /// borda de contraste alto.
    ///
    /// ⚠️⚠️ **E TODA imagem desenhada pelo vello deslocou-se meio pixel** — não é escolha nossa, é
    /// a correcção de um defeito do upstream (*"Fixed: blurry image rendering due to incorrect
    /// half-pixel offset"*, vello 0.9): o `fine.wgsl` passou a amostrar no **centro** do pixel e
    /// não no canto. Vale para os três modos de qualidade, e o mais visível é o **`Low`**: meio
    /// pixel muda qual texel o vizinho-mais-próximo escolhe, então uma pré-visualização de arte de
    /// pixel pode aparecer deslocada **uma coluna inteira**.
    ///
    /// # ⛔⛔⛔ A COSTURA do `Repeat` — a nota antiga estava certa no mecanismo e ERRADA no sujeito
    ///
    /// Esta linha dizia: *"`Low` é o único modo **sem costura**, porque em `Repeat` o filtro
    /// bilinear grampeia na fronteira do ladrilho em vez de dar a volta, e o artefacto é meio
    /// texel"*. O grampo existe mesmo — e é **deliberado do Vello**: o atlas dele empacota as
    /// imagens **encostadas, sem folga** (`vello_encoding::image_cache`, `atlas.allocate(size2(w,
    /// h))`), então sem o grampo um tap leria a imagem VIZINHA.
    ///
    /// ⭐⭐⭐ **O que a nota não dizia é que o custo do grampo é uma propriedade do LADRILHO, não do
    /// filtro.** Medido em 2026-08-30 na GPU, com um oráculo por periodicidade (a mesma arte assada
    /// com o período e com 4x o período — as duas produzem a MESMA imagem infinita, então o ladrilho
    /// largo dá a resposta certa exactamente onde o estreito tem costura):
    ///
    /// | ladrilho | salto dele na volta | costura medida |
    /// |---|---|---|
    /// | ruído cru | `236` | `100` níveis |
    /// | ruído **espelhado** | `0` | `7` |
    /// | onda quadrada crua | `215` | `107` |
    /// | onda quadrada **espelhada** | `0` | **`0`** |
    ///
    /// ⇒ **um ladrilho que fecha não tem costura em qualidade nenhuma.** Um motivo assado de uma
    /// FORMA tem a caixa justa (cobertura zero nos quatro lados) e mede **`0`**; quem tem o defeito
    /// é a arte de bordo a bordo que não foi feita para repetir — e aí o artista já vê uma **aresta
    /// dura**, muito maior que a banda do filtro. O painel diz-lo agora
    /// ([`ph2d_vec_pattern::wrap_seam`]).
    ///
    /// ⛔ **E baixar para `Medium` foi medido e REFUTADO:** sob meio pixel de deslocamento — que é o
    /// que um `pan` faz o tempo todo — `Medium` e `High` chegam ao **mesmo** pico (`107`); o
    /// `Medium` só estreita a banda de ~3 texels para ~1, e paga fidelidade de ampliação (pior no
    /// interior em `20` contra `29` níveis a 4 texels/período). *O `0` que o `Medium` marca a 1:1
    /// existe só no alinhamento inteiro, que é medida zero na prática.*
    ///
    /// Continua a valer: `Medium` para o caso liso e `Low` para arte de pixel — e o `Low` é o único
    /// **exactamente** sem costura, porque o vizinho-mais-próximo não tem tap para grampear.
    #[allow(clippy::too_many_arguments)]
    pub fn fill_path_image(
        &mut self,
        path: &BezPath,
        rule: Fill,
        transform: Affine,
        image: &StableImage,
        brush_transform: Affine,
        x_extend: Extend,
        y_extend: Extend,
        quality: ImageQuality,
        alpha: f32,
    ) {
        let brush = ImageBrush::new(image.data.clone())
            .with_x_extend(x_extend)
            .with_y_extend(y_extend)
            .with_quality(quality)
            .with_alpha(alpha);
        self.inner
            .fill(rule, transform, &brush, Some(brush_transform), path);
    }

    /// ⭐ **Uma IMAGEM preenche a FAIXA de um traço** (plano 35, wave B) — irmã do
    /// [`Self::fill_path_image`], e existe pela mesma razão: o `brush_transform` do peniko está
    /// **morto** na porta de traço que havia, e é ele que põe o padrão no espaço local do caminho.
    ///
    /// ⚠️ O Vello compõe `transform * brush_transform` — então quem chama tem de saber sob que afim
    /// a GEOMETRIA vai (ver `stroke_uniform`, onde o caminho não-conforme leva a geometria à tela e
    /// passa `IDENTITY`).
    #[allow(clippy::too_many_arguments)] // os mesmos sete factos do irmão de preenchimento
    pub fn stroke_path_image(
        &mut self,
        path: &BezPath,
        style: &Stroke,
        transform: Affine,
        image: &StableImage,
        brush_transform: Affine,
        x_extend: Extend,
        y_extend: Extend,
        quality: ImageQuality,
        alpha: f32,
    ) {
        let brush = ImageBrush::new(image.data.clone())
            .with_x_extend(x_extend)
            .with_y_extend(y_extend)
            .with_quality(quality)
            .with_alpha(alpha);
        self.inner
            .stroke(style, transform, &brush, Some(brush_transform), path);
    }

    /// Push a clip layer that masks subsequent drawing to `path`.
    /// Pair with [`Self::pop_layer`]. Used by scrollable panels to
    /// keep overflowing content inside the panel rect.
    pub fn push_clip(&mut self, path: &impl vello::kurbo::Shape) {
        self.push_clip_with_rule(path, Fill::NonZero);
    }

    /// [`Self::push_clip`] with an explicit fill rule. A clip whose
    /// path has nested contours (a compound path with a hole) needs
    /// `Fill::EvenOdd` — under `NonZero` the hole would still take
    /// paint. Pair with [`Self::pop_layer`].
    pub fn push_clip_with_rule(&mut self, path: &impl vello::kurbo::Shape, rule: Fill) {
        self.inner.push_layer(
            rule,
            vello::peniko::BlendMode::default(),
            1.0,
            Affine::IDENTITY,
            path,
        );
    }

    /// ⭐⭐⭐ **A CAMADA DE UM OBJECTO** — o que faz *opacidade do objecto* e *modo de mistura*
    /// significarem o que significam no Illustrator, no Figma e no SVG (2026-09-05).
    ///
    /// Tudo o que for desenhado até ao [`Self::pop_layer`] compõe-se **uma vez**, como um só
    /// objecto: é isto que distingue meia-opacidade no OBJECTO de meia-opacidade em cada tinta
    /// dele — com as tintas, o traço transparece sobre o próprio preenchimento.
    ///
    /// ⚠️ **`rect` é o LIMITE da camada, e ele tem custo:** o Vello aloca a mistura sobre esta
    /// caixa, então ela tem de ser a da arte (a caixa de ecrã da forma, com o transbordo do traço)
    /// e não a tela. ⛔ Uma caixa PEQUENA DEMAIS recorta a arte em silêncio — quem chama usa a
    /// mesma porta de limites que o produtor de FX usa para dimensionar a textura dele.
    ///
    /// ⚠️ **Quem chama decide se ISTO é preciso**: com `alpha = 1` e mistura normal, uma camada é
    /// trabalho de GPU para nada — o caminho neutro tem de continuar a não a empurrar.
    pub fn push_object_layer(&mut self, rect: &Rect, blend: vello::peniko::BlendMode, alpha: f32) {
        // A regra de preenchimento do RECORTE da camada: um rectângulo é convexo, então `NonZero`
        // e `EvenOdd` coincidem — a escolha é a do irmão `push_clip` e não uma decisão nova.
        self.inner
            .push_layer(Fill::NonZero, blend, alpha, Affine::IDENTITY, rect);
    }

    /// Pop the most recent layer pushed via [`Self::push_clip`].
    pub fn pop_layer(&mut self) {
        self.inner.pop_layer();
    }
}

impl Default for VectorScene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_is_empty() {
        // Vello doesn't expose a "command count" public API; we
        // settle for "construction succeeds + reset is idempotent
        // on an empty scene".
        let mut s = VectorScene::new();
        s.reset();
        s.reset();
        // Inner accessor returns the same Scene reference twice —
        // proves the wrapper isn't accidentally cloning.
        let p1 = s.inner() as *const Scene;
        let p2 = s.inner() as *const Scene;
        assert!(std::ptr::eq(p1, p2));
    }

    #[test]
    fn fill_rect_does_not_panic() {
        // Smoke test the helper. We can't easily inspect the encoded
        // commands without GPU, but if Vello's encoder panics on a
        // malformed input this would surface here.
        let mut s = VectorScene::new();
        s.fill_rect(Rect::new(0.0, 0.0, 100.0, 50.0), Color::WHITE);
        s.fill_rect(Rect::new(10.0, 10.0, 20.0, 20.0), Color::BLACK);
    }

    #[test]
    fn fill_path_with_brush() {
        let mut s = VectorScene::new();
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((10.0, 0.0));
        path.line_to((10.0, 10.0));
        path.line_to((0.0, 10.0));
        path.close_path();
        s.fill_path(&path, &Brush::Solid(Color::WHITE), Affine::IDENTITY);
    }

    #[test]
    fn reset_after_fills_is_idempotent() {
        let mut s = VectorScene::new();
        s.fill_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::WHITE);
        s.fill_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        s.reset();
        // After reset, drawing again must not panic.
        s.fill_rect(Rect::new(2.0, 2.0, 3.0, 3.0), Color::WHITE);
    }
}
