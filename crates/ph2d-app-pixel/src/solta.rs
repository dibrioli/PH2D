//! **A moldura solta** (P6, Edit ▸ Transform `Ctrl+T`, a ferramenta Move `V` e o Paste): as células
//! levantadas viajam numa matriz afim com pivô (`ph2d_pixel_core::affine`), com 8 alças de escala,
//! as zonas de giro fora dos cantos, a inclinação (Ctrl numa alça de lado) e o pivô arrastável. O
//! documento NÃO muda até ao `Enter` (ou um clique fora): aí o assentar é UM passo
//! (`edit::commit`, ou `edit::paste` para o que veio da área de transferência); o `Esc` larga tudo.
//! A pré-visualização é o MESMO op de assentar, mostrado como um traço de valores prontos.

use ph2d_editor_core::pixel_panel::{PixelResample, PixelTransformField, PixelTransformInfo};
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

/// O alcance de uma alça (e do pivô) sob o ponteiro, em píxeis de ECRÃ.
const GRAB_PX: f64 = 6.0; // LITERAL-PX-OK: alcance do clique sobre uma alça de 6 px
/// A faixa fora da moldura que ainda gira (as zonas de giro do Aseprite), em píxeis de ECRÃ.
const ROTATE_PX: f64 = 24.0; // LITERAL-PX-OK: alcance das zonas de giro fora da moldura

/// As 8 alças de escala, em meios da fonte: cantos ↖ ↗ ↘ ↙ e lados ↑ → ↓ ←.
pub const HANDLES: [(u8, u8); 8] = [
    (0, 0),
    (2, 0),
    (2, 2),
    (0, 2),
    (1, 0),
    (2, 1),
    (1, 2),
    (0, 1),
];

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
        let (w, h) = self.size();
        let f = self.m.fields(w, h);
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

    /// Os cantos da moldura em ECRÃ (`to_screen` = imagem → ecrã).
    #[must_use]
    pub fn quad(&self, to_screen: ph2d_vector::Affine) -> [Point; 4] {
        let (w, h) = self.size();
        self.m
            .quad(f64::from(w), f64::from(h))
            .map(|p| to_screen * Point::new(p.x, p.y))
    }

    /// As 8 alças e o pivô em ECRÃ.
    #[must_use]
    pub fn handles(&self, to_screen: ph2d_vector::Affine) -> ([Point; 8], Point) {
        let (w, h) = self.size();
        let (w, h) = (f64::from(w), f64::from(h));
        let at = |u: f64, v: f64| {
            let p = self.m.apply(u, v);
            to_screen * Point::new(p.x, p.y)
        };
        let hs = HANDLES.map(|(a, b)| at(f64::from(a) * w / 2.0, f64::from(b) * h / 2.0));
        (hs, at(self.pivot.u, self.pivot.v))
    }

    /// A alça sob o ponto de ecrã `p` (`None` = fora: o clique assenta).
    #[must_use]
    pub fn hit(&self, to_screen: ph2d_vector::Affine, p: Point, skew: bool) -> Option<Handle> {
        let (hs, pivot) = self.handles(to_screen);
        let near = |q: Point, r: f64| (q - p).hypot() <= r;
        if near(pivot, GRAB_PX) {
            return Some(Handle::Pivot);
        }
        for (i, q) in hs.iter().enumerate() {
            if near(*q, GRAB_PX) {
                let (a, b) = HANDLES[i];
                return Some(if skew && i >= 4 {
                    Handle::Skew(a, b)
                } else {
                    Handle::Scale(a, b)
                });
            }
        }
        let quad = self.quad(to_screen);
        if inside(&quad, p) {
            return Some(Handle::Move);
        }
        quad.iter()
            .any(|c| near(*c, ROTATE_PX))
            .then_some(Handle::Rotate)
    }
}

/// O ponto dentro do quadrilátero convexo (qualquer orientação).
fn inside(q: &[Point; 4], p: Point) -> bool {
    let mut sinal = 0.0_f64;
    for i in 0..4 {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        let c = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        if c != 0.0 {
            if sinal != 0.0 && c.signum() != sinal {
                return false;
            }
            sinal = c.signum();
        }
    }
    true
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
            return;
        };
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
        let (w, h) = so.size();
        so.m = so.m.with_fields(so.pivot, w, h, p, false);
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
    m: Mods,
) -> Option<Press> {
    let doc = app.docs.get(&key)?;
    let so = app.solta.as_ref().filter(|s| s.key == key)?;
    let screen = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm)?;
    let pt = Point::new(f64::from(p.0), f64::from(p.1));
    let Some(handle) = so.hit(screen, pt, m.ctrl) else {
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

/// A ferramenta Move (ou Ctrl com uma de selecção): levanta e arrasta. Fora da selecção não faz
/// nada (Aseprite).
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

fn imagem(screen: ph2d_vector::Affine, p: Point) -> Ponto {
    let q = screen.inverse() * p;
    Ponto { x: q.x, y: q.y }
}

/// Mover com um arrasto de alça: a matriz nova é DELTA desde a de partida (Shift = proporção e giro
/// de 15°, Alt = a partir do centro). `true` = consumido.
pub(crate) fn moved(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32), m: Mods) -> bool {
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
        let opt = DragOpts {
            keep_ratio: m.shift,
            from_center: m.alt,
            snap: m.shift,
        };
        (so.m, so.pivot) = a.m0.drag(a.p0, size, a.handle, (a.from, to), opt);
    }
    app.toca(key);
    true
}

/// Largar o arrasto (a moldura fica solta). `true` = havia arrasto.
pub(crate) fn release(app: &mut PixelApp) -> bool {
    app.solta.as_mut().and_then(|s| s.arrasto.take()).is_some()
}
