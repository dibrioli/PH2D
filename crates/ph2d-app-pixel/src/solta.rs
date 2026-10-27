//! **A moldura solta** (P6, a ferramenta Transform `V`/`Ctrl+T`, Ctrl+arrasto com uma selecção e o
//! Paste): as células
//! levantadas viajam numa matriz afim com pivô (`ph2d_pixel_core::affine`). O gizmo é o do SPRITE da
//! casa (dono 26/10, ADR-0192 §12): a mesma pintura, as mesmas alças, as zonas de giro fora dos
//! cantos e o pivô ([`ph2d_editor_core::gizmo`]); a conta do arrasto é a do núcleo, que assenta em
//! células inteiras; a inclinação é pelo campo Skew. O
//! documento NÃO muda até ao `Enter` (ou um clique fora): aí o assentar é UM passo
//! (`edit::commit`, ou `edit::paste` para o que veio da área de transferência); o `Esc` larga tudo.
//! A pré-visualização é o MESMO op de assentar, mostrado como um traço de valores prontos.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::gizmo::{GizmoView, ids as g, sprite_gizmo_hit_at};
use ph2d_editor_core::pixel_panel::{PixelResample, PixelTransformField, PixelTransformInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_pixel_core::affine::{
    Affine, DragOpts, FieldPatch, Handle, Pivot, Point as Ponto, Resample,
};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::transform::{Floating, lift};
use ph2d_pixel_core::{FrameId, LayerId, Op, Sprite, Stroke, edit};
use ph2d_vector::Point;

use crate::PixelApp;
use crate::gesture::Mods;
use crate::input::{Canvas, Press, key_of, to_screen};

/// A alça do núcleo de cada id do gizmo do sprite. A caixa do gizmo é a imagem (`0..w × 0..h`, Y de
/// mundo para cima), então o «TL» dele é o canto `(0, h)` da imagem: as alças vão em meios da fonte.
fn alca(id: NodeId) -> Option<Handle> {
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
struct Arrasto {
    handle: Handle,
    from: Ponto,
    m0: Affine,
    p0: Pivot,
}

/// As células levantadas e a moldura delas.
#[derive(Debug, Clone)]
pub struct Solta {
    pub(crate) key: u32,
    pub(crate) layer: LayerId,
    pub(crate) frame: FrameId,
    /// A fonte, no sítio de onde saiu (ou para onde a cópia a guardou).
    pub(crate) src: Floating,
    /// De onde saiu (o assentar limpa-a primeiro); `None` = veio da área de transferência.
    pub(crate) original: Option<Mask>,
    pub(crate) m: Affine,
    pub(crate) pivot: Pivot,
    arrasto: Option<Arrasto>,
    /// O último ponto de ecrã do ponteiro (o pintor do gizmo troca os cantos por círculos no giro).
    pub(crate) cursor: Option<(f32, f32)>,
    /// O op de assentar e a pré-visualização dele para `(matriz, amostragem)`.
    cache: Option<(Affine, Resample, Option<Op>, Option<Floating>)>,
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
    fn new(
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
            arrasto: None,
            cursor: None,
            cache: None,
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.src.image.w, self.src.image.h)
    }

    /// Há um arrasto de alça em curso.
    #[must_use]
    pub fn dragging(&self) -> bool {
        self.arrasto.is_some()
    }

    /// O op de assentar e as células transformadas (`None` = nada muda / a matriz degenerou).
    fn refresh(
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
            let f = edit::transform(&self.src, &self.m.settle(), method, s.transparent_value());
            let key = (self.layer, self.frame);
            let op = f.as_ref().and_then(|f| match &self.original {
                Some(o) => edit::commit(s, key, o, f, bg),
                None => edit::paste(s, key, f, bg),
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
pub(crate) fn info_of(m: Affine, (w, h): (u32, u32)) -> PixelTransformInfo {
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
pub(crate) fn com_campo(
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
    sprite_gizmo_hit_at(view, true, p.0, p.1).and_then(alca)
}

impl PixelApp {
    /// O documento `key` muda de aspecto (a vista recompõe).
    pub(crate) fn toca(&mut self, key: u32) {
        let stamp = self.stamp();
        if let Some(d) = self.docs.get_mut(&key) {
            d.stamp = stamp;
        }
    }

    /// O valor de uma área limpa numa camada Background (a cor de fundo da barra).
    pub(crate) fn fundo(&self, s: &Sprite) -> u32 {
        s.value_of_rgba(self.bg)
    }

    /// **Edit ▸ Transform**: levanta as células seleccionadas do cel activo (sem selecção, o sprite
    /// inteiro). `false` = nada a levantar (ou a camada recusa).
    pub(crate) fn levanta(&mut self, key: u32) -> bool {
        if self.solta.as_ref().is_some_and(|s| s.key == key) {
            return true;
        }
        if let Some(motivo) = self.docs.get(&key).and_then(crate::camadas::recusa_pintar) {
            self.recusa(motivo);
            return false;
        }
        let Some(doc) = self.docs.get(&key) else {
            return false;
        };
        let s = &doc.sprite;
        let m = s
            .mask()
            .cloned()
            .unwrap_or_else(|| Mask::full(s.width(), s.height()));
        let Some(src) = lift(s, (doc.layer, doc.frame), Some(&m)) else {
            return false;
        };
        self.solta = Some(Solta::new(key, doc.layer, doc.frame, src, Some(m)));
        self.toca(key);
        true
    }

    /// **Edit ▸ Paste**: a cópia solta onde foi copiada (assenta com `Enter` ou um clique fora).
    pub(crate) fn cola(&mut self, key: u32, src: Floating) {
        if let Some(motivo) = self.docs.get(&key).and_then(crate::camadas::recusa_pintar) {
            self.recusa(motivo);
            return;
        }
        let Some(doc) = self.docs.get(&key) else {
            return;
        };
        self.solta = Some(Solta::new(key, doc.layer, doc.frame, src, None));
        self.toca(key);
    }

    /// **Assenta** a moldura: UM passo (nada mudou = nenhum).
    pub(crate) fn assenta(&mut self) {
        let Some(mut so) = self.solta.take() else {
            return;
        };
        let method = resample(self.options.resample);
        let op = self.docs.get(&so.key).and_then(|d| {
            let bg = self.fundo(&d.sprite);
            so.refresh(&d.sprite, method, bg).0.cloned()
        });
        match op {
            Some(op) => {
                let _ = self.commit(so.key, op);
            }
            None => self.toca(so.key),
        }
    }

    /// Larga a moldura sem mudar o documento (`Esc`).
    pub(crate) fn larga(&mut self) {
        if let Some(so) = self.solta.take() {
            self.toca(so.key);
        }
    }

    /// Um campo da barra da moldura (ângulo anti-horário).
    pub(crate) fn campo_da_moldura(&mut self, f: PixelTransformField, v: i32) {
        let Some(so) = self.solta.as_mut() else {
            crate::viva::campo(self, f, v);
            return;
        };
        so.m = com_campo(so.m, so.pivot, so.size(), f, v);
        let key = so.key;
        self.toca(key);
    }

    /// O traço de pré-visualização e a máscara da moldura do documento `key`, se há.
    pub(crate) fn previa(&mut self, key: u32) -> Option<(Stroke, Option<Mask>)> {
        let method = resample(self.options.resample);
        let bg = self.docs.get(&key).map(|d| self.fundo(&d.sprite))?;
        let so = self.solta.as_mut().filter(|s| s.key == key)?;
        let doc = self.docs.get(&key)?;
        let s = &doc.sprite;
        let (layer, frame) = (so.layer, so.frame);
        let (op, f) = so.refresh(s, method, bg);
        let mask = f.map(|f| f.mask(s.width(), s.height()));
        let mut values = Vec::new();
        let mut patch = |p: &ph2d_pixel_core::Patch| {
            let bpp = s.mode().bpp();
            for y in 0..p.rect.h {
                for x in 0..p.rect.w {
                    let i = (y as usize * p.rect.w as usize + x as usize) * bpp;
                    let mut b = [0u8; 4];
                    b[..bpp].copy_from_slice(&p.pixels[i..i + bpp]);
                    values.push((
                        (p.rect.x + x as i32, p.rect.y + y as i32),
                        u32::from_le_bytes(b),
                    ));
                }
            }
        };
        match op {
            Some(Op::Patch(p)) => patch(p),
            Some(Op::Batch(ops)) => {
                for o in ops {
                    if let Op::Patch(p) = o {
                        patch(p);
                    }
                }
            }
            _ => {}
        }
        Some((Stroke::with_values(layer, frame, values), mask))
    }
}

/// Premir em Draw com uma moldura solta no documento `key`: uma alça começa o arrasto; fora dela o
/// clique ASSENTA (e é consumido). `None` = não há moldura.
pub(crate) fn press(
    app: &mut PixelApp,
    c: &Canvas<'_>,
    bits: u64,
    key: u32,
    p: (f32, f32),
) -> Option<Press> {
    let doc = app.docs.get(&key)?;
    let so = app.solta.as_ref().filter(|s| s.key == key)?;
    let screen = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm)?;
    let pt = Point::new(f64::from(p.0), f64::from(p.1));
    let Some(handle) = so.hit(&so.gizmo(screen, janela(c), Some(p)), p) else {
        app.assenta();
        return Some(Press::Done);
    };
    let from = imagem(screen, pt);
    let so = app.solta.as_mut()?;
    so.arrasto = Some(Arrasto {
        handle,
        from,
        m0: so.m,
        p0: so.pivot,
    });
    Some(Press::Done)
}

/// Ctrl+arrasto com uma ferramenta de selecção: levanta e arrasta. Fora da selecção não faz nada
/// (Aseprite).
pub(crate) fn press_move(
    app: &mut PixelApp,
    c: &Canvas<'_>,
    bits: u64,
    key: u32,
    p: (f32, f32),
    cell: (i32, i32),
) -> Press {
    let dentro = app
        .docs
        .get(&key)
        .is_some_and(|d| d.sprite.in_selection(cell.0, cell.1));
    if !dentro || !app.levanta(key) {
        return Press::Done;
    }
    let Some(doc) = app.docs.get(&key) else {
        return Press::Done;
    };
    let Some(screen) = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm) else {
        return Press::Done;
    };
    let from = imagem(screen, Point::new(f64::from(p.0), f64::from(p.1)));
    if let Some(so) = app.solta.as_mut() {
        so.arrasto = Some(Arrasto {
            handle: Handle::Move,
            from,
            m0: so.m,
            p0: so.pivot,
        });
    }
    Press::Done
}

/// A janela em píxeis de ecrã (a câmera do gizmo).
#[expect(clippy::cast_precision_loss, reason = "tamanho de janela")]
pub(crate) fn janela(c: &Canvas<'_>) -> (f32, f32) {
    (c.window.width as f32, c.window.height as f32)
}

/// Os modificadores do gizmo do sprite: Shift = proporção e giro em degraus de 15°, Ctrl = a
/// partir do centro.
pub(crate) fn drag_opts(m: Mods) -> DragOpts {
    DragOpts {
        keep_ratio: m.shift,
        from_center: m.ctrl,
        snap: m.shift,
    }
}

pub(crate) fn imagem(screen: ph2d_vector::Affine, p: Point) -> Ponto {
    let q = screen.inverse() * p;
    Ponto { x: q.x, y: q.y }
}

/// Mover com um arrasto de alça: a matriz nova é DELTA desde a de partida, com os modificadores do
/// gizmo do sprite (Shift = proporção e giro em degraus de 15°, Ctrl = a partir do centro). Sem
/// arrasto, guarda o ponteiro (os cantos viram círculos sobre as zonas de giro). `true` = consumido.
pub(crate) fn moved(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32), m: Mods) -> bool {
    if let Some(so) = app.solta.as_mut() {
        so.cursor = Some(p);
    }
    let Some(so) = app.solta.as_ref().filter(|s| s.dragging()) else {
        return false;
    };
    let key = so.key;
    let Some(screen) = c
        .editing
        .filter(|&b| key_of(c.world, b) == Some(key))
        .and_then(|b| to_screen(c.world, b, app.docs.get(&key)?, c.camera, c.window, c.ppm))
    else {
        return true;
    };
    let to = imagem(screen, Point::new(f64::from(p.0), f64::from(p.1)));
    let Some(so) = app.solta.as_mut() else {
        return true;
    };
    let size = so.size();
    if let Some(a) = &so.arrasto {
        (so.m, so.pivot) = a.m0.drag(a.p0, size, a.handle, (a.from, to), drag_opts(m));
    }
    app.toca(key);
    true
}

/// Largar o arrasto (a moldura fica solta). `true` = havia arrasto.
pub(crate) fn release(app: &mut PixelApp) -> bool {
    app.solta.as_mut().and_then(|s| s.arrasto.take()).is_some()
}
