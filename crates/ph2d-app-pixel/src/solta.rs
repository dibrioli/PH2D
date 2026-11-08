//! **A moldura solta** (P6: Transform `V`/`Ctrl+T`, Ctrl+arrasto, Paste): as células levantadas numa
//! matriz afim com pivô (`ph2d_pixel_core::affine`), sob o gizmo do SPRITE da casa
//! ([`ph2d_editor_core::gizmo`], ADR-0196 §12). O documento NÃO muda até ao `Enter`/clique fora (UM
//! passo, `edit::commit`/`edit::paste`); `Esc` larga. A pré-visualização é o MESMO op de assentar.

use ph2d_editor_core::pixel_panel::PixelTransformField;
use ph2d_pixel_core::affine::{DragOpts, Handle, Point as Ponto};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::transform::{Floating, lift};
use ph2d_pixel_core::{Sprite, Stroke};
use ph2d_vector::Point;

use crate::PixelApp;
use crate::gesture::Mods;
use crate::input::{Canvas, Press, key_of, to_screen};

pub(crate) use ph2d_pixel_ponte::moldura::{Arrasto, com_campo, info_of};
pub use ph2d_pixel_ponte::moldura::{Solta, gizmo_of, hit_of, resample};

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

    /// O texto do Text solto: `cobertura` (1 byte por célula) na cor `fg` (assenta com `Enter` ou
    /// um clique fora, misturado ao que estiver por baixo).
    pub(crate) fn solta_texto(&mut self, key: u32, cobertura: Floating, fg: [u8; 4]) {
        let Some(doc) = self.docs.get(&key) else {
            return;
        };
        let mut so = Solta::new(key, doc.layer, doc.frame, cobertura, None);
        so.tinta = Some(fg);
        self.solta = Some(so);
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
        if !self.solta.as_ref().is_some_and(|s| s.key == key) {
            return crate::texto::previa(self, key)
                .or_else(|| crate::filtros::previa(self, key))
                .or_else(|| crate::slices::previa(self, key))
                .map(|t| (t, None));
        }
        let method = resample(self.options.resample);
        let bg = self.docs.get(&key).map(|d| self.fundo(&d.sprite))?;
        let so = self.solta.as_mut().filter(|s| s.key == key)?;
        let doc = self.docs.get(&key)?;
        let s = &doc.sprite;
        let (layer, frame) = (so.layer, so.frame);
        let texto = so.tinta.is_some();
        let (op, f) = so.refresh(s, method, bg);
        let mask = f.map(|f| formigas(f, texto, (s.width(), s.height())));
        let st = op.map_or_else(
            || Stroke::with_values(layer, frame, []),
            |op| Stroke::of_patches(layer, frame, op, s.mode().bpp()),
        );
        Some((st, mask))
    }
}

/// A máscara das formigas de uma moldura solta `f` num sprite `w × h`: a colagem e o Transform
/// contornam a FORMA levantada; o texto (P8c) contorna só o RETÂNGULO da moldura — o contorno de
/// cada letra tapava as próprias letras.
pub(crate) fn formigas(f: &Floating, texto: bool, (w, h): (u32, u32)) -> Mask {
    if texto {
        Mask::from_rect(w, h, f.rect())
    } else {
        f.mask(w, h)
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
        if matches!(a.handle, Handle::Rotate) {
            so.m = so.m.settle_turn(so.pivot);
        }
    }
    app.toca(key);
    true
}

/// Largar o arrasto (a moldura fica solta). `true` = havia arrasto.
pub(crate) fn release(app: &mut PixelApp) -> bool {
    app.solta.as_mut().and_then(|s| s.arrasto.take()).is_some()
}
