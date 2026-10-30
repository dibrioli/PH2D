//! ⭐ As imagens do PROJECTO na secção «Photos» da gaveta (dono 26/10: a imagem é um asset do
//! projecto, escolhido da lista dele). A lista é a do navegador de assets — o MESMO índice, que a
//! shell publica também com a gaveta aberta ([`crate::wants_project_images`]); escolher uma copia o
//! ficheiro para o conjunto do Kanban, que é o que viaja ao vivo e no projecto.

use super::Drawer;
use crate::metrics as m;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_asset_index::{AssetEntry, Thumb};
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_vector::{Color, ImageQuality, StableImage, VectorScene};

/// Uma imagem do projecto, com a miniatura já feita textura (o id estável que o Vello cacheia).
#[derive(Clone)]
pub(crate) struct ProjectImage {
    pub asset: [u8; 32],
    pub name: String,
    swatch: [u8; 4],
    thumb: Option<(Thumb, StableImage)>,
}

impl std::fmt::Debug for ProjectImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectImage")
            .field("name", &self.name)
            .finish()
    }
}

impl PartialEq for ProjectImage {
    fn eq(&self, o: &Self) -> bool {
        self.asset == o.asset
            && self.name == o.name
            && self.swatch == o.swatch
            && self.thumb.as_ref().map(|t| &t.0) == o.thumb.as_ref().map(|t| &t.0)
    }
}

impl ProjectImage {
    fn of(e: &AssetEntry, old: &[Self]) -> Option<Self> {
        let ph2d_asset_index::AssetRef::Texture { asset } = e.key else {
            return None;
        };
        // A textura velha fica quando a miniatura é a mesma (senão cada quadro reenviava os pixels).
        let kept = old
            .iter()
            .find(|o| o.asset == asset)
            .and_then(|o| o.thumb.clone())
            .filter(|(t, _)| Some(t) == e.thumb.as_ref());
        let thumb = kept.or_else(|| {
            let t = e.thumb.clone()?;
            let img = StableImage::from_rgba(t.rgba.clone(), t.w, t.h)?;
            Some((t, img))
        });
        Some(Self {
            asset,
            name: e.name.clone(),
            swatch: e.swatch,
            thumb,
        })
    }
}

/// Relê a lista publicada para a gaveta aberta.
pub(crate) fn refresh(d: &mut Drawer, entries: &[AssetEntry]) {
    let fresh: Vec<ProjectImage> = entries
        .iter()
        .filter_map(|e| ProjectImage::of(e, &d.project))
        .collect();
    if fresh != d.project {
        d.project = fresh;
    }
}

/// A miniatura em `r`, a encher sem deformar (o `cover` do fundo que ela vai ser).
pub(crate) fn paint_tile(scene: &mut VectorScene, img: &ProjectImage, r: Rect) {
    let [cr, cg, cb, ca] = img.swatch;
    fill_rounded_rect(
        scene,
        r,
        m::BUTTON_RADIUS,
        Color::from_rgba8(cr, cg, cb, ca),
    );
    let Some((t, tex)) = &img.thumb else {
        return;
    };
    let d = super::cover_dest([t.w, t.h], r);
    scene.draw_stable_image(
        tex,
        (
            f64::from(d.x),
            f64::from(d.y),
            f64::from(d.x + d.w),
            f64::from(d.y + d.h),
        ),
        ImageQuality::Medium,
    );
}

/// ⭐ O ficheiro da imagem `asset` do projecto: o original do disco quando o projecto a conhece
/// por caminho (um JPEG fica JPEG), senão os pixels codificados em PNG (uma tela feita no app).
pub(crate) fn file_bytes(db: &AssetDb, asset: [u8; 32]) -> Option<Vec<u8>> {
    let id = AssetId::from_digest(asset);
    let from_disk = db
        .tracked_paths()
        .into_iter()
        .filter(|p| db.id_for_path(p) == Some(id))
        .find_map(|p| std::fs::read(p).ok());
    if from_disk.is_some() {
        return from_disk;
    }
    let a = db.get(&id)?;
    let (w, h, px) = a.image_rgba8()?;
    let mut out = Vec::new();
    image::ImageEncoder::write_image(
        image::codecs::png::PngEncoder::new(&mut out),
        &px,
        w,
        h,
        image::ExtendedColorType::Rgba8,
    )
    .ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_canvas_made_in_the_app_becomes_a_png_the_kanban_reads() {
        let db = AssetDb::new();
        let px = vec![200u8; 3 * 2 * 4];
        let id = db.insert_image_rgba8(3, 2, px);
        let bytes = file_bytes(&db, *id.as_bytes()).expect("a imagem do projecto vira ficheiro");
        assert_eq!(crate::back::attach::image_size(&bytes), Some([3, 2]));
        assert_eq!(
            file_bytes(&db, [7; 32]),
            None,
            "um asset que não existe não dá ficheiro"
        );
    }
}
