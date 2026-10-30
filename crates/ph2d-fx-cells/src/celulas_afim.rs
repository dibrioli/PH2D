//! A projecção das células: mundo → pixel da camada das sprites, e a câmera do átlas com a mesma escala.
//! Irmão do `celulas.rs`, cortado pelo tecto de LOC.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// **O afim mundo → pixel da camada das SPRITES** (`[a, b, c, d, e, f]`, o do kurbo) — porta única: a
/// projecção da câmera no sub-rect da cena, sem a matriz de modelo (o passe de sprites nunca a põe),
/// mais o canto do sub-rect.
#[must_use]
pub fn afim_das_sprites(
    camera: &Camera2d,
    window: WindowSize,
    scene_viewport: Option<[f32; 4]>,
) -> [f64; 6] {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o split dá pixels inteiros e não negativos"
    )]
    let (canto, alvo) = scene_viewport.map_or(([0.0, 0.0], window), |v| {
        (
            [v[0], v[1]],
            WindowSize {
                width: v[2].max(1.0) as u32,
                height: v[3].max(1.0) as u32,
            },
        )
    });
    let mut cam = *camera;
    cam.model = Camera2d::MODEL_IDENTITY;
    let [a, b, c, d, e, f] = cam.world_to_screen_affine(alvo).as_coeffs();
    [a, b, c, d, f64::from(canto[0]) + e, f64::from(canto[1]) + f]
}

/// A câmera que projecta no ÁTLAS (`alvo`) com a MESMA escala e translação que a camada.
pub(super) fn camera_do_atlas(camera: &Camera2d, afim: [f64; 6], alvo: (u32, u32)) -> Camera2d {
    let k = afim[0];
    let (w, h) = (f64::from(alvo.0), f64::from(alvo.1));
    let mut c = *camera;
    c.model = Camera2d::MODEL_IDENTITY;
    #[expect(clippy::cast_possible_truncation, reason = "a câmera lê f32")]
    {
        c.height_world = (h / k) as f32;
        c.center = [
            ((w * 0.5 - afim[4]) / k) as f32,
            ((afim[5] - h * 0.5) / k) as f32,
        ];
    }
    c
}
