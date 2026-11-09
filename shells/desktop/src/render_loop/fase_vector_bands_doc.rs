//! **O documento de UM quadro, pronto a codificar** — o que a [`super::fase_vector_bands`] entrega ao Vello
//! por faixa de z, inteiro (sem faixas) ou só a receita aberta. As mesmas referências servem o editor e a
//! vista da câmera (A6); só a câmera, as peles e o destino mudam.

use ph2d_render::draw_bands::{Band, Family, FrameOrder};
use ph2d_vec_render::{
    BrushArts, DilatedPaints, FxImages, LiveGeometry, PatternTiles, WidgetSkins,
};
use ph2d_vec_scene::{VecScene, VecViewState, VecXforms};
use ph2d_vector::{Affine, VectorScene};

pub(super) struct DocDasFaixas<'a> {
    pub(super) scene: &'a VecScene,
    pub(super) view: &'a VecViewState,
    pub(super) xf: &'a VecXforms,
    pub(super) live: &'a LiveGeometry,
    pub(super) fx: &'a FxImages,
    pub(super) patterns: &'a PatternTiles,
    pub(super) brushes: &'a BrushArts,
    pub(super) dilated: &'a DilatedPaints,
    pub(super) order: &'a FrameOrder,
    pub(super) world: &'a ph2d_ecs::World,
    pub(super) ppm: f32,
    pub(super) editing: Option<u64>,
    /// No palco do jogo a faixa Pixel fica vazia (GEF 19 E2).
    pub(super) sem_pixel: bool,
}

impl DocDasFaixas<'_> {
    /// Uma cena por faixa: a Pixel leva a arte dos Pixel Sprites dela; as outras, o documento com o resto
    /// ESCONDIDO (o push/pop do recorte vive fora do filtro).
    pub(super) fn faixas(
        &self,
        pixel: &mut ph2d_app_pixel::PixelApp,
        bands: &[Band],
        skins: &WidgetSkins,
        cam: Affine,
        cenas: &mut Vec<VectorScene>,
        (editor, luz): (bool, &mut ph2d_app_light2d::LuzDoQuadro),
    ) {
        let (normais, sem_luz) = luz.normais_do_quadro();
        for band in bands {
            let mut target = VectorScene::new();
            let (w, ppm, ed) = (self.world, self.ppm, self.editing);
            if band.family == Family::Pixel {
                let ids = if self.sem_pixel {
                    Vec::new()
                } else {
                    self.order.pixel_ids_in(*band)
                };
                ph2d_app_pixel::draw::art(pixel, w, cam, ppm, ed, &ids, editor, &mut target);
                let luz = (&mut *normais, sem_luz);
                ph2d_app_pixel::luz::faixa_normal(pixel, w, (cam, ppm), Some(&ids), luz);
            } else {
                ph2d_app_pixel::luz::faixa_normal(
                    pixel,
                    w,
                    (cam, ppm),
                    None,
                    (&mut *normais, None),
                );
                let keep = self.order.vector_ids_in(*band);
                let mut band_view = self.view.clone();
                for path in self.scene.paths() {
                    if !keep.contains(&path.id) {
                        band_view.hidden.push(path.id);
                    }
                }
                self.dispatch(&band_view, skins, cam, &mut target);
            }
            cenas.push(target);
        }
    }

    /// O documento inteiro numa cena (sem faixas).
    pub(super) fn inteiro(&self, skins: &WidgetSkins, cam: Affine, target: &mut VectorScene) {
        self.dispatch(self.view, skins, cam, target);
    }

    /// Só a receita aberta, na cena ACIMA do vidro (faixas e `dispatch` saltam-na).
    pub(super) fn isolada(&self, skins: &WidgetSkins, cam: Affine, target: &mut VectorScene) {
        ph2d_vec_render::dispatch_isolated(
            self.scene,
            self.view,
            self.xf,
            self.live,
            self.fx,
            skins,
            self.patterns,
            self.brushes,
            self.dilated,
            cam,
            target,
        );
    }

    fn dispatch(&self, view: &VecViewState, skins: &WidgetSkins, cam: Affine, t: &mut VectorScene) {
        ph2d_vec_render::dispatch(
            self.scene,
            view,
            self.xf,
            self.live,
            self.fx,
            skins,
            self.patterns,
            self.brushes,
            self.dilated,
            cam,
            t,
        );
    }
}
