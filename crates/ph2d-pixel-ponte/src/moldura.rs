//! **A moldura solta** (P6: Transform `V`/`Ctrl+T`, Ctrl+arrasto, Paste): as células levantadas numa
//! matriz afim com pivô, sob o gizmo do SPRITE da casa. Saiu de `ph2d-app-pixel` (P12): a lei pura
//! e a geometria do gizmo; o estado e os gestos ficam na família (`ph2d_app_pixel::solta`).

use ph2d_editor_core::NodeId;
use ph2d_editor_core::gizmo::{GizmoView, ids as g, sprite_gizmo_hit_at};
use ph2d_editor_core::pixel_panel::{PixelResample, PixelTransformField, PixelTransformInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_pixel_core::affine::{Affine, FieldPatch, Handle, Pivot, Point as Ponto, Resample};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::transform::Floating;
use ph2d_pixel_core::{FrameId, LayerId, Op, Sprite, edit};

/// A alça do núcleo de cada id do gizmo do sprite. A caixa do gizmo é a imagem (`0..w × 0..h`, Y de
/// mundo para cima), então o «TL» dele é o canto `(0, h)` da imagem: as alças vão em meios da fonte.
pub fn alca(id: NodeId) -> Option<Handle> {
    let escala = [
        (g::GIZMO_HANDLE_TL, (0, 2)),
        (g::GIZMO_HANDLE_TR, (2, 2)),
        (g::GIZMO_HANDLE_BL, (0, 0)),
        (g::GIZMO_HANDLE_BR, (2, 0)),
        (g::GIZMO_HANDLE_T, (1, 2)),
        (g::GIZMO_HANDLE_B, (1, 0)),
        (g::GIZMO_HANDLE_L, (0, 1)),
        (g::GIZMO_HANDLE_R, (2, 1)),
    ];
    if let Some(&(_, (a, b))) = escala.iter().find(|(i, _)| *i == id) {
        return Some(Handle::Scale(a, b));
    }
    let inclina = [
        (g::GIZMO_SKEW_T, (1, 2)),
        (g::GIZMO_SKEW_B, (1, 0)),
        (g::GIZMO_SKEW_L, (0, 1)),
        (g::GIZMO_SKEW_R, (2, 1)),
    ];
    if let Some(&(_, (a, b))) = inclina.iter().find(|(i, _)| *i == id) {
        return Some(Handle::Skew(a, b));
    }
    let giro = [
        g::GIZMO_ROTATE_TL,
        g::GIZMO_ROTATE_TR,
        g::GIZMO_ROTATE_BL,
        g::GIZMO_ROTATE_BR,
    ];
    Some(match id {
        _ if id == g::GIZMO_BBOX_INTERIOR => Handle::Move,
        _ if id == g::GIZMO_PIVOT => Handle::Pivot,
        _ if giro.contains(&id) => Handle::Rotate,
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub struct Arrasto {
    pub handle: Handle,
    pub from: Ponto,
    pub m0: Affine,
    pub p0: Pivot,
}

/// As células levantadas e a moldura delas.
#[derive(Debug, Clone)]
pub struct Solta {
    pub key: u32,
    pub layer: LayerId,
    pub frame: FrameId,
    /// A fonte, no sítio de onde saiu (ou para onde a cópia a guardou).
    pub src: Floating,
    /// De onde saiu (o assentar limpa-a primeiro); `None` = veio da área de transferência.
    pub original: Option<Mask>,
    pub m: Affine,
    pub pivot: Pivot,
    /// O texto do Text (P8b): `src` é a COBERTURA (1 byte) e a cor mistura-se ao assentar, sobre o
    /// que estiver lá por baixo nesse sítio e cortada à selecção de então.
    pub tinta: Option<[u8; 4]>,
    pub arrasto: Option<Arrasto>,
    /// O último ponto de ecrã do ponteiro (o pintor do gizmo troca os cantos por círculos no giro).
    pub cursor: Option<(f32, f32)>,
    /// O op de assentar e a pré-visualização dele para `(matriz, amostragem)`.
    pub cache: Option<(Affine, Resample, Option<Op>, Option<Floating>)>,
}

/// A amostragem do núcleo.
#[must_use]
pub fn resample(r: PixelResample) -> Resample {
    match r {
        PixelResample::Fast => Resample::Fast,
        PixelResample::RotSprite => Resample::RotSprite,
    }
}

impl Solta {
    #[must_use]
    pub fn new(
        key: u32,
        layer: LayerId,
        frame: FrameId,
        src: Floating,
        original: Option<Mask>,
    ) -> Self {
        let m = Affine::at(f64::from(src.x), f64::from(src.y));
        let pivot = Pivot {
            u: f64::from(src.image.w) / 2.0,
            v: f64::from(src.image.h) / 2.0,
        };
        Self {
            key,
            layer,
            frame,
            src,
            original,
            m,
            pivot,
            tinta: None,
            arrasto: None,
            cursor: None,
            cache: None,
        }
    }

    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.src.image.w, self.src.image.h)
    }

    /// Há um arrasto de alça em curso.
    #[must_use]
    pub fn dragging(&self) -> bool {
        self.arrasto.is_some()
    }

    /// O op de assentar e as células transformadas (`None` = nada muda / a matriz degenerou).
    pub fn refresh(
        &mut self,
        s: &Sprite,
        method: Resample,
        bg: u32,
    ) -> (Option<&Op>, Option<&Floating>) {
        let fresh = self
            .cache
            .as_ref()
            .is_some_and(|(m, r, ..)| *m == self.m && *r == method);
        if !fresh {
            let key = (self.layer, self.frame);
            let bg_t = self.tinta.map_or(s.transparent_value(), |_| 0);
            let f = edit::transform_within(&self.src, &self.m.settle(), method, bg_t, s.bounds());
            let f = match self.tinta {
                Some(fg) => f.map(|f| crate::leis::na_cor(s, &f, fg)),
                None => f,
            };
            let op = f.as_ref().and_then(|f| match (&self.original, self.tinta) {
                (_, Some(_)) => ph2d_pixel_core::transform::stamp(s, key, f, None, None, bg),
                (Some(o), None) => edit::commit(s, key, o, f, bg),
                (None, None) => edit::paste(s, key, f, bg),
            });
            self.cache = Some((self.m, method, op, f));
        }
        let (.., op, f) = self.cache.as_ref().expect("acabou de ser posto");
        (op.as_ref(), f.as_ref())
    }

    /// Os números da barra (ângulo anti-horário, Aseprite).
    #[must_use]
    pub fn info(&self) -> PixelTransformInfo {
        info_of(self.m, self.size())
    }

    /// O gizmo do sprite sobre a moldura ([`gizmo_of`]).
    #[must_use]
    pub fn gizmo(
        &self,
        to_screen: ph2d_vector::Affine,
        window: (f32, f32),
        cursor: Option<(f32, f32)>,
    ) -> GizmoView {
        gizmo_of(self.m, self.size(), self.pivot, to_screen, window, cursor)
    }

    /// A alça sob o ponto de ecrã `p` (`None` = fora: o clique assenta) — a do gizmo do sprite.
    #[must_use]
    pub fn hit(&self, view: &GizmoView, p: (f32, f32)) -> Option<Handle> {
        hit_of(view, p)
    }
}

/// Os números da barra de uma moldura `m` de fonte `w × h` (ângulo anti-horário, Aseprite) — a
/// moldura solta e a forma viva.
#[must_use]
pub fn info_of(m: Affine, (w, h): (u32, u32)) -> PixelTransformInfo {
    let f = m.fields(w, h);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "células e graus de um sprite"
    )]
    let r = |v: f64| v.round() as i32;
    PixelTransformInfo {
        x: f.x,
        y: f.y,
        w: r(f.w),
        h: r(f.h),
        angle: r(-f.rot),
        skew: r(f.skew),
    }
}

/// A moldura `m` com o campo `f` da barra a valer `v` (ângulo anti-horário).
#[must_use]
pub fn com_campo(
    m: Affine,
    pivot: Pivot,
    (w, h): (u32, u32),
    f: PixelTransformField,
    v: i32,
) -> Affine {
    let v = f64::from(v);
    let mut p = FieldPatch::default();
    match f {
        PixelTransformField::X => p.x = Some(v),
        PixelTransformField::Y => p.y = Some(v),
        PixelTransformField::W => p.w = Some(v),
        PixelTransformField::H => p.h = Some(v),
        PixelTransformField::Angle => p.rot = Some(-v),
        PixelTransformField::Skew => p.skew = Some(v),
    }
    m.with_fields(pivot, w, h, p, false)
}

/// ⭐ **O gizmo do sprite sobre uma moldura** `m` de fonte `w × h` (a solta e a forma viva): a caixa
/// é a fonte (`0..w × 0..h`), o `model` leva-a ao ecrã pela matriz (`to_screen` = imagem → ecrã) e a
/// câmera é o próprio ecrã com Y para cima — o pintor da casa projecta `câmera · model` e desfaz a
/// inversão.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "píxeis de ecrã para o pintor em f32"
)]
pub fn gizmo_of(
    m: Affine,
    (w, h): (u32, u32),
    pivot: Pivot,
    to_screen: ph2d_vector::Affine,
    window: (f32, f32),
    cursor: Option<(f32, f32)>,
) -> GizmoView {
    let o = m.apply(0.0, 0.0);
    let (u, v) = (m.apply(1.0, 0.0), m.apply(0.0, 1.0));
    let k = to_screen
        * ph2d_vector::Affine::new([u.x - o.x, u.y - o.y, v.x - o.x, v.y - o.y, o.x, o.y]);
    let [a, b, c, d, e, f] = k.as_coeffs();
    let (ww, wh) = window;
    let model = [
        a as f32,
        -b as f32,
        c as f32,
        -d as f32,
        e as f32,
        wh - f as f32,
    ];
    GizmoView {
        bbox_min_world: [0.0, 0.0],
        bbox_max_world: [w as f32, h as f32],
        rotation: 0.0,
        camera_center: [ww * 0.5, wh * 0.5],
        camera_height_world: wh,
        window_w: ww,
        window_h: wh,
        canvas: Rect::new(0.0, 0.0, ww, wh),
        cursor_screen: cursor,
        pivot_world: [pivot.u as f32, pivot.v as f32],
        pivot_tool_active: false,
        model,
    }
}

/// A alça do núcleo sob o ponto de ecrã `p` no gizmo `view` (`None` = fora).
#[must_use]
pub fn hit_of(view: &GizmoView, p: (f32, f32)) -> Option<Handle> {
    sprite_gizmo_hit_at(view, true, true, p.0, p.1).and_then(alca)
}
