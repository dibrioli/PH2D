//! As máscaras do pincel no preset: o que o ficheiro leva e o que se instala dele (filho de [`super`], LOC cap).

use super::*;

impl PainterTool {
    /// As máscaras da forma e do grão que o pincel tem, como o ficheiro as leva (`phTx`).
    pub(super) fn texturas_do_pincel(&self) -> BTreeMap<String, Vec<u8>> {
        let mut texturas = BTreeMap::new();
        if self.paint.brush.shape.kind == TextureKind::Image
            && let Some((lum, w, h)) = self.brush_shape_image()
        {
            texturas.insert(TEXTURA_FORMA.to_owned(), escrever_textura(lum, w, h));
        }
        if self.paint.brush.texture.kind == TextureKind::Image
            && let Some((lum, w, h)) = self.brush_texture_image()
        {
            texturas.insert(TEXTURA_GRAO.to_owned(), escrever_textura(lum, w, h));
        }
        if self.le_o_fluxo()
            && let Some((lum, w, h)) = self.brush_flow_image()
        {
            texturas.insert(TEXTURA_FLUXO.to_owned(), escrever_textura(lum, w, h));
        }
        self.texturas_da_mangueira(&mut texturas); // as imagens da ponta e a cor delas (L19)
        self.texturas_do_dual(&mut texturas); // a imagem da 2.ª ponta (L6)
        texturas
    }

    /// As máscaras da forma e do grão: instaladas DIRECTAMENTE (as portas da UI re-ajustam o aspecto e
    /// a rampa do grão, e o preset já traz os valores que essas portas deram). Um slot `Image` sem a
    /// textura no ficheiro volta a `None` — uma imagem que não está lá não pinta — e o id vai na
    /// lista devolvida.
    pub(super) fn instalar_mascaras(
        &mut self,
        texturas: &BTreeMap<String, Vec<u8>>,
    ) -> Vec<String> {
        let mut falhas = Vec::new();
        let mut ler = |id: &str| {
            let t = texturas.get(id).and_then(|b| ler_textura(b).ok());
            if t.is_none() {
                falhas.push(id.to_owned());
            }
            t
        };
        self.paint.shape_layers.clear();
        self.paint.shape_image = None;
        if self.paint.brush.shape.kind == TextureKind::Image {
            match ler(TEXTURA_FORMA) {
                Some((lum, w, h)) => {
                    self.paint.shape_image = Some(BrushTextureImage::new(lum, w, h));
                }
                None => self.paint.brush.shape.kind = TextureKind::None,
            }
        }
        self.paint.shape_image_pending = false;
        self.paint.shape_image_version = self.paint.shape_image_version.wrapping_add(1);

        self.congela_os_graos_da_imagem_que_sai(false);
        self.paint.texture_image = None;
        if self.paint.brush.texture.kind == TextureKind::Image {
            match ler(TEXTURA_GRAO) {
                Some((lum, w, h)) => {
                    self.paint.texture_image = Some(BrushTextureImage::new(lum, w, h));
                }
                None => self.paint.brush.texture.kind = TextureKind::None,
            }
        }
        self.paint.texture_image_pending = false;
        self.paint.texture_image_version = self.paint.texture_image_version.wrapping_add(1);
        // O Flow só se mexe no meio que o lê: noutro, o `edge_flow` do pincel nem veio do ficheiro.
        if self.paint_media() == PaintMedia::Watercolor {
            self.paint.flow_map.imagem = None;
            if self.paint.brush.edge_flow.kind == TextureKind::Image {
                match ler(TEXTURA_FLUXO) {
                    Some((lum, w, h)) => self
                        .paint
                        .flow_map
                        .instala(BrushTextureImage::new(lum, w, h)),
                    None => self.paint.brush.edge_flow.kind = TextureKind::None,
                }
            }
            self.paint.flow_map.versao = self.paint.flow_map.versao.wrapping_add(1);
        }
        self.instala_a_mangueira(texturas, &mut falhas); // as imagens da ponta (L19)
        self.instala_o_dual(texturas, &mut falhas); // a imagem da 2.ª ponta (L6)
        falhas
    }

    /// O pincel tem um Flow de imagem que o meio de agora lê.
    fn le_o_fluxo(&self) -> bool {
        self.paint_media() == PaintMedia::Watercolor
            && self.paint.brush.edge_flow.kind == TextureKind::Image
    }
}
