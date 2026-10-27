//! **Os menus Edit e Sprite do lado da família** (P6): Flip/Rotate/Shift das células seleccionadas,
//! Delete, a área de transferência (Copy, Cut, Copy Merged, Paste solto), Rotate/Flip Canvas, Crop,
//! Trim, e o OK dos diálogos (Color Range, Modify, Canvas Size, Sprite Size). Cada acção é UM passo
//! do desfazer do sprite (`ph2d_pixel_core::{edit, canvas, selection}`).

use ph2d_editor_core::pixel_panel::{
    PixelDialogChoice, PixelDialogKind, PixelEditAction as E, PixelModify, PixelModifyBrush,
    PixelRequest, PixelSizeMethod, PixelSpriteAction as Sp,
};
use ph2d_pixel_core::canvas::{self, Borders, SizeMethod};
use ph2d_pixel_core::edit::{self, Rotation};
use ph2d_pixel_core::fill::BucketSample;
use ph2d_pixel_core::layers::Clip;
use ph2d_pixel_core::selection::{self, Mask, Modify, ModifyBrush, SelectMode};
use ph2d_pixel_core::transform::{Floating, Image, stamp};
use ph2d_pixel_core::{ColorMode, Op, Sprite};

use crate::PixelApp;

/// **A área de transferência**: a imagem copiada (com modo e paleta, para colar noutro sprite) e
/// quais células são da selecção de onde saiu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prancheta {
    pub(crate) clip: Clip,
    pub(crate) keep: Vec<bool>,
}

impl Prancheta {
    fn de(s: &Sprite, f: Floating) -> Self {
        Self {
            clip: Clip {
                mode: s.mode(),
                palette: s.palette().to_vec(),
                transparent: s.transparent_index(),
                rect: f.rect(),
                data: f.image.data,
            },
            keep: f.keep,
        }
    }

    /// O RGBA da célula `i` da cópia.
    fn rgba(&self, i: usize) -> [u8; 4] {
        let c = &self.clip;
        let bpp = c.mode.bpp();
        let b = &c.data[i * bpp..i * bpp + bpp];
        match c.mode {
            ColorMode::Rgba => [b[0], b[1], b[2], b[3]],
            ColorMode::Gray => [b[0], b[0], b[0], b[1]],
            ColorMode::Indexed if b[0] == c.transparent => [0; 4],
            ColorMode::Indexed => c.palette.get(usize::from(b[0])).copied().unwrap_or([0; 4]),
        }
    }

    /// A cópia no modo do sprite `s` (o mesmo modo e paleta = os mesmos bytes).
    fn solta_em(&self, s: &Sprite) -> Floating {
        let c = &self.clip;
        let same = c.mode == s.mode()
            && (c.mode != ColorMode::Indexed
                || (c.palette == s.palette() && c.transparent == s.transparent_index()));
        let image = if same {
            Image {
                w: c.rect.w,
                h: c.rect.h,
                bpp: c.mode.bpp(),
                data: c.data.clone(),
            }
        } else {
            let values: Vec<u32> = (0..c.rect.area())
                .map(|i| s.value_of_rgba(self.rgba(i)))
                .collect();
            Image::from_values(c.rect.w, c.rect.h, s.mode().bpp(), &values)
        };
        Floating {
            x: c.rect.x,
            y: c.rect.y,
            image,
            keep: self.keep.clone(),
        }
    }
}

/// Limpa as células de `m` no cel `key` (Background: a cor de fundo). UM passo.
fn limpa(s: &Sprite, key: ph2d_pixel_core::CelKey, m: &Mask, bg: u32) -> Option<Op> {
    let nada = Floating {
        x: 0,
        y: 0,
        image: Image::new(0, 0, s.mode().bpp(), s.transparent_value()),
        keep: Vec::new(),
    };
    stamp(s, key, &nada, Some(m), None, bg)
}

fn modify_in(k: PixelModify) -> Modify {
    match k {
        PixelModify::Expand => Modify::Expand,
        PixelModify::Contract => Modify::Contract,
        PixelModify::Border => Modify::Border,
    }
}

fn method_in(m: PixelSizeMethod) -> SizeMethod {
    match m {
        PixelSizeMethod::Nearest => SizeMethod::Nearest,
        PixelSizeMethod::Bilinear => SizeMethod::Bilinear,
        PixelSizeMethod::RotSprite => SizeMethod::RotSprite,
    }
}

/// O pedido larga a moldura solta antes (o Aseprite assenta as células levantadas antes de outro
/// comando): tudo o que muda o documento, salvo a própria moldura.
#[must_use]
pub fn assenta_antes(r: &PixelRequest) -> bool {
    !matches!(
        r,
        PixelRequest::Tool(_)
            | PixelRequest::Group(_)
            | PixelRequest::Options(_)
            | PixelRequest::Fg(_)
            | PixelRequest::Bg(_)
            | PixelRequest::Swap
            | PixelRequest::Swatch { .. }
            | PixelRequest::SortKey(_)
            | PixelRequest::TransformField { .. }
            | PixelRequest::Edit(E::Commit | E::Cancel | E::Transform | E::Copy | E::CopyMerged)
    )
}

impl PixelApp {
    /// Os menus Edit e Sprite e os campos da moldura; `false` = não é deles.
    pub(crate) fn aplica_edicao(&mut self, r: &PixelRequest) -> bool {
        let Some(key) = self.chave else {
            return matches!(
                r,
                PixelRequest::Edit(_)
                    | PixelRequest::Sprite(_)
                    | PixelRequest::TransformField { .. }
            );
        };
        match r {
            PixelRequest::Edit(a) => self.edita(key, *a),
            PixelRequest::Sprite(a) => self.sprite_menu(key, *a),
            PixelRequest::TransformField { field, value } => self.campo_da_moldura(*field, *value),
            _ => return false,
        }
        true
    }

    fn edita(&mut self, key: u32, a: E) {
        match a {
            E::Transform => {
                crate::input::set_tool(self, crate::Tool::Transform);
                if !self.levanta(key) {
                    self.recusa("pixel.select.refuse.nothing");
                }
                return;
            }
            E::Commit => {
                self.assenta();
                return crate::viva::assenta(self);
            }
            E::Cancel => {
                self.larga();
                if self.live_shape().is_some() {
                    crate::input::cancel(self);
                }
                return;
            }
            E::Paste => {
                let src = self
                    .clip
                    .as_ref()
                    .zip(self.docs.get(&key))
                    .map(|(p, d)| p.solta_em(&d.sprite));
                match src {
                    Some(src) => self.cola(key, src),
                    None => self.recusa("pixel.edit.refuse.clipboard"),
                }
                return;
            }
            _ => {}
        }
        let Some(doc) = self.docs.get(&key) else {
            return;
        };
        let s = &doc.sprite;
        let ck = (doc.layer, doc.frame);
        let bg = self.fundo(s);
        let pinta = matches!(
            a,
            E::FlipH | E::FlipV | E::Rotate180 | E::Rotate90Cw | E::Rotate90Ccw | E::Clear | E::Cut
        ) || matches!(a, E::ShiftLeft | E::ShiftRight | E::ShiftUp | E::ShiftDown);
        if pinta && let Some(motivo) = crate::camadas::recusa_pintar(doc) {
            self.recusa(motivo);
            return;
        }
        let op = match a {
            E::FlipH => edit::flip(s, ck, true, bg),
            E::FlipV => edit::flip(s, ck, false, bg),
            E::Rotate180 => edit::rotate(s, ck, Rotation::R180, bg),
            E::Rotate90Cw => edit::rotate(s, ck, Rotation::Cw90, bg),
            E::Rotate90Ccw => edit::rotate(s, ck, Rotation::Ccw90, bg),
            E::ShiftLeft | E::ShiftRight | E::ShiftUp | E::ShiftDown => {
                if s.mask().is_none() {
                    return self.recusa("pixel.select.refuse.none");
                }
                let (dx, dy) = match a {
                    E::ShiftLeft => (-1, 0),
                    E::ShiftRight => (1, 0),
                    E::ShiftUp => (0, -1),
                    _ => (0, 1),
                };
                edit::shift(s, ck, dx, dy, bg)
            }
            E::Clear => {
                let m = s
                    .mask()
                    .cloned()
                    .unwrap_or_else(|| Mask::full(s.width(), s.height()));
                limpa(s, ck, &m, bg)
            }
            E::Copy | E::Cut | E::CopyMerged => {
                let f = if a == E::CopyMerged {
                    edit::copy_merged(s, doc.frame)
                } else {
                    edit::copy(s, ck)
                };
                let Some(f) = f else {
                    return self.recusa("pixel.layer.refuse.selection");
                };
                let p = Prancheta::de(s, f);
                let op = (a == E::Cut)
                    .then(|| s.mask().and_then(|m| limpa(s, ck, m, bg)))
                    .flatten();
                self.clip = Some(p);
                op
            }
            E::Transform | E::Commit | E::Cancel | E::Paste => None,
        };
        self.passo(key, op);
    }

    fn sprite_menu(&mut self, key: u32, a: Sp) {
        let Some(s) = self.docs.get(&key).map(|d| &d.sprite) else {
            return;
        };
        let op = match a {
            Sp::CanvasSize => {
                self.abre_dialogo = Some(PixelDialogKind::CanvasSize);
                None
            }
            Sp::SpriteSize => {
                self.abre_dialogo = Some(PixelDialogKind::SpriteSize);
                None
            }
            Sp::RotateCanvas180 => canvas::rotate_canvas(s, Rotation::R180),
            Sp::RotateCanvas90Cw => canvas::rotate_canvas(s, Rotation::Cw90),
            Sp::RotateCanvas90Ccw => canvas::rotate_canvas(s, Rotation::Ccw90),
            Sp::FlipCanvasH => canvas::flip_canvas(s, true),
            Sp::FlipCanvasV => canvas::flip_canvas(s, false),
            Sp::Crop => {
                if s.mask().is_none() {
                    return self.recusa("pixel.select.refuse.none");
                }
                canvas::crop_to_selection(s)
            }
            Sp::Trim => {
                let op = canvas::trim(s);
                if op.is_none() {
                    self.recusa("pixel.sprite.refuse.trim");
                }
                op
            }
        };
        self.passo(key, op);
    }

    /// O OK de um diálogo do Pixel sobre o documento em Draw: UM passo.
    pub fn aplica_dialogo(&mut self, c: PixelDialogChoice) {
        let Some(key) = self.chave else {
            return;
        };
        self.assenta();
        let fg = self.fg;
        let Some(doc) = self.docs.get(&key) else {
            return;
        };
        let s = &doc.sprite;
        let bg = self.fundo(s);
        let op = match c {
            PixelDialogChoice::ColorRange { tolerance } => {
                let ck = (doc.layer, doc.frame);
                let m = selection::color_range(s, ck, fg, tolerance, BucketSample::Layer);
                selection::select(s, &m, SelectMode::Replace)
            }
            PixelDialogChoice::Modify { kind, n, brush } => {
                let b = match brush {
                    PixelModifyBrush::Square => ModifyBrush::Square,
                    PixelModifyBrush::Circle => ModifyBrush::Circle,
                };
                selection::modify(s, modify_in(kind), n, b)
            }
            PixelDialogChoice::CanvasSize {
                left,
                top,
                right,
                bottom,
            } => canvas::canvas_size(
                s,
                Borders {
                    left,
                    top,
                    right,
                    bottom,
                },
                bg,
            ),
            PixelDialogChoice::SpriteSize {
                width,
                height,
                method,
            } => canvas::sprite_size(s, width, height, method_in(method)),
        };
        self.passo(key, op);
    }
}
