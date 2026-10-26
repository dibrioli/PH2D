//! **Os quatro diálogos do Pixel P6** no `WidgetStore` (Select ▸ Color Range, Select ▸ Modify,
//! Sprite ▸ Canvas Size, Sprite ▸ Sprite Size do Aseprite) — irmão do `color_mode_ops`: um só
//! `ContextMenuKind::PixelEditDialog`, o `kind` diz qual. Os campos numéricos vivem nos widgets; a
//! sincronia entre eles (bordas ↔ tamanho, píxeis ↔ percentagem, Lock Ratio) corre a cada
//! `ValueChanged` ([`WidgetStore::pixel_edit_value_changed`]) e no Enter antes do OK.

use super::super::WidgetStore;
use crate::ids;
use crate::interaction::ContextMenuRequest;
use crate::pixel_panel::{
    NEW_SPRITE_MAX_SIDE, PixelDialogChoice, PixelDialogKind, PixelEditDialog, PixelModifyBrush,
    PixelSizeMethod, pixel_anchor_borders,
};
use ph2d_a11y::NodeId;

/// O tecto do `n` do Modify: o campo do Aseprite aceita até 100 (dado do coordenador da linha, não
/// medido aqui contra o Trial).
const MODIFY_MAX: u32 = 100;
/// O lado máximo (o `u16` do formato).
const SIDE_MAX: f64 = NEW_SPRITE_MAX_SIDE as f64;

/// `v` arredondado e preso a `lo..=hi` (um campo inteiro).
fn int(v: f64, lo: i64, hi: i64) -> i64 {
    #[expect(clippy::cast_precision_loss, reason = "limites de |x| ≤ 65535")]
    let (lo_f, hi_f) = (lo as f64, hi as f64);
    let v = if v.is_finite() { v.round() } else { lo_f };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "preso a lo..=hi antes da conversão"
    )]
    let r = v.clamp(lo_f, hi_f) as i64;
    r
}

impl WidgetStore {
    /// Abre um diálogo do Pixel P6 sobre um sprite `size`. Os widgets registam-se aqui (o diálogo
    /// só existe enquanto aberto); tolerância, `n`, pincel, método e Lock Ratio são os da última
    /// vez na sessão; o tamanho semeia-se com o do sprite e a âncora volta ao centro.
    pub fn open_pixel_edit_dialog(&mut self, kind: PixelDialogKind, size: (u32, u32)) {
        use crate::interaction::InteractiveState as S;
        let size = (size.0.max(1), size.1.max(1));
        let d = &mut self.pixel.edit;
        d.kind = kind;
        d.size = size;
        d.anchor = (1, 1);
        d.request = None;
        let d = *d;
        let (sw, sh) = (f64::from(size.0), f64::from(size.1));
        let (plo, phi) = pct_range(size);
        let fields: &[(NodeId, f64, f64, f64)] = match kind {
            PixelDialogKind::ColorRange => &[(
                ids::PIXEL_EDIT_TOLERANCE,
                f64::from(d.tolerance),
                0.0,
                255.0, // LITERAL-PX-OK: a tolerância é um u8, domínio do documento
            )],
            PixelDialogKind::Modify(_) => &[(
                ids::PIXEL_EDIT_MODIFY_N,
                f64::from(d.modify_n),
                1.0,
                f64::from(MODIFY_MAX),
            )],
            PixelDialogKind::CanvasSize => &[
                (ids::PIXEL_EDIT_CANVAS_W, sw, 1.0, SIDE_MAX),
                (ids::PIXEL_EDIT_CANVAS_H, sh, 1.0, SIDE_MAX),
                (ids::PIXEL_EDIT_LEFT, 0.0, -SIDE_MAX, SIDE_MAX),
                (ids::PIXEL_EDIT_TOP, 0.0, -SIDE_MAX, SIDE_MAX),
                (ids::PIXEL_EDIT_RIGHT, 0.0, -SIDE_MAX, SIDE_MAX),
                (ids::PIXEL_EDIT_BOTTOM, 0.0, -SIDE_MAX, SIDE_MAX),
            ],
            PixelDialogKind::SpriteSize => &[
                (ids::PIXEL_EDIT_SPRITE_W, sw, 1.0, SIDE_MAX),
                (ids::PIXEL_EDIT_SPRITE_H, sh, 1.0, SIDE_MAX),
                (ids::PIXEL_EDIT_SPRITE_WPCT, 100.0, plo, phi), // LITERAL-PX-OK: 100 % = o tamanho ao abrir
                (ids::PIXEL_EDIT_SPRITE_HPCT, 100.0, plo, phi), // LITERAL-PX-OK: 100 % = o tamanho ao abrir
            ],
        };
        for &(id, v, lo, hi) in fields {
            self.register(
                id,
                S::NumberInput {
                    state: crate::widget::TextInputState::Normal,
                    value: v,
                    buffer: String::new(),
                    caret: 0,
                    last_committed: v,
                    selection_anchor: None,
                },
            );
            self.set_number_range(id, lo, hi, 1.0);
            self.set_number_drag_rate(id, 0.25); // LITERAL-PX-OK: unidades por px de arrasto, domínio do documento
            self.set_number_value(id, v);
        }
        let anchors = ids::PIXEL_EDIT_ANCHORS.iter().map(|(_, id, _)| *id);
        let brushes = ids::PIXEL_EDIT_BRUSHES.iter().map(|(_, id, _)| *id);
        let methods = ids::PIXEL_EDIT_METHODS.iter().map(|(_, id, _)| *id);
        let ends = [
            ids::PIXEL_EDIT_OK,
            ids::PIXEL_EDIT_CANCEL,
            ids::PIXEL_EDIT_LOCK,
        ];
        for id in anchors.chain(brushes).chain(methods).chain(ends) {
            let state = crate::widget::ButtonState::Normal;
            self.register(id, S::Button { state });
        }
        self.context_menu = Some(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::PixelEditDialog,
        });
    }

    /// O estado do diálogo aberto (as escolhas que não são campos numéricos).
    #[must_use]
    pub fn pixel_edit_dialog(&self) -> PixelEditDialog {
        self.pixel.edit
    }

    /// Canvas Size: escolhe a âncora `(coluna, linha)` e refaz as bordas pelo tamanho escrito.
    pub fn set_pixel_edit_anchor(&mut self, anchor: (u8, u8)) {
        self.pixel.edit.anchor = (anchor.0.min(2), anchor.1.min(2));
        self.sync_canvas_borders();
    }

    pub fn set_pixel_edit_brush(&mut self, b: PixelModifyBrush) {
        self.pixel.edit.brush = b;
    }

    pub fn set_pixel_edit_method(&mut self, m: PixelSizeMethod) {
        self.pixel.edit.method = m;
    }

    /// Sprite Size: liga/desliga o Lock Ratio; ao ligar, a altura segue a largura (Aseprite).
    pub fn set_pixel_edit_lock_ratio(&mut self, on: bool) {
        self.pixel.edit.lock_ratio = on;
        if on {
            self.pixel_edit_value_changed(ids::PIXEL_EDIT_SPRITE_W);
        }
    }

    /// Um campo do diálogo mudou (commit, arrasto, stepper): prende-o à faixa dele e refaz os
    /// irmãos. `true` se o `id` é um campo do diálogo aberto.
    pub fn pixel_edit_value_changed(&mut self, id: NodeId) -> bool {
        if !ids::PIXEL_EDIT_FIELDS.contains(&id) || self.number_value(id).is_none() {
            return false;
        }
        let side = i64::from(NEW_SPRITE_MAX_SIDE);
        let (sw, sh) = self.pixel.edit.size;
        let lock = self.pixel.edit.lock_ratio;
        match id {
            _ if id == ids::PIXEL_EDIT_TOLERANCE => {
                let v = self.int_field(id, 0, 255); // LITERAL-PX-OK: a tolerância é um u8
                self.put(id, v);
            }
            _ if id == ids::PIXEL_EDIT_MODIFY_N => {
                let v = self.int_field(id, 1, i64::from(MODIFY_MAX));
                self.put(id, v);
            }
            _ if id == ids::PIXEL_EDIT_CANVAS_W || id == ids::PIXEL_EDIT_CANVAS_H => {
                let v = self.int_field(id, 1, side);
                self.put(id, v);
                self.sync_canvas_borders();
            }
            _ if ids::PIXEL_EDIT_BORDERS.contains(&id) => self.sync_canvas_size(id),
            _ if id == ids::PIXEL_EDIT_SPRITE_W || id == ids::PIXEL_EDIT_SPRITE_H => {
                let horiz = id == ids::PIXEL_EDIT_SPRITE_W;
                let s = if horiz { sw } else { sh };
                let v = self.int_field(id, 1, side);
                self.put(id, v);
                #[expect(clippy::cast_precision_loss, reason = "um lado ≤ 65535")]
                let pct = v as f64 * 100.0 / f64::from(s); // LITERAL-PX-OK: percentagem
                self.sprite_axis(horiz, pct, false);
                if lock {
                    self.sprite_axis(!horiz, pct, true);
                }
            }
            _ => {
                let horiz = id == ids::PIXEL_EDIT_SPRITE_WPCT;
                let (lo, hi) = pct_range((sw, sh));
                let pct = self.number_value(id).unwrap_or(100.0).clamp(lo, hi); // LITERAL-PX-OK: percentagem
                self.sprite_axis(horiz, pct, true);
                if lock {
                    self.sprite_axis(!horiz, pct, true);
                }
            }
        }
        true
    }

    /// O OK: os campos (presos às faixas) e as escolhas viram o pedido; fecha. A tolerância e o
    /// `n` ficam lembrados na sessão.
    pub fn request_pixel_edit(&mut self) {
        let side = i64::from(NEW_SPRITE_MAX_SIDE);
        let d = self.pixel.edit;
        let choice = match d.kind {
            PixelDialogKind::ColorRange => {
                let t = self.int_field(ids::PIXEL_EDIT_TOLERANCE, 0, 255); // LITERAL-PX-OK: a tolerância é um u8
                let tolerance = u8::try_from(t).unwrap_or(u8::MAX);
                self.pixel.edit.tolerance = tolerance;
                PixelDialogChoice::ColorRange { tolerance }
            }
            PixelDialogKind::Modify(kind) => {
                let n = self.int_field(ids::PIXEL_EDIT_MODIFY_N, 1, i64::from(MODIFY_MAX));
                let n = u32::try_from(n).unwrap_or(1);
                self.pixel.edit.modify_n = n;
                PixelDialogChoice::Modify {
                    kind,
                    n,
                    brush: d.brush,
                }
            }
            PixelDialogKind::CanvasSize => {
                let b = ids::PIXEL_EDIT_BORDERS
                    .map(|id| i32::try_from(self.int_field(id, -side, side)).unwrap_or(0));
                PixelDialogChoice::CanvasSize {
                    left: b[0],
                    top: b[1],
                    right: b[2],
                    bottom: b[3],
                }
            }
            PixelDialogKind::SpriteSize => {
                let w = self.int_field(ids::PIXEL_EDIT_SPRITE_W, 1, side);
                let h = self.int_field(ids::PIXEL_EDIT_SPRITE_H, 1, side);
                PixelDialogChoice::SpriteSize {
                    width: u32::try_from(w).unwrap_or(1),
                    height: u32::try_from(h).unwrap_or(1),
                    method: d.method,
                }
            }
        };
        self.pixel.edit.request = Some(choice);
        self.close_context_menu();
    }

    /// Toma (e limpa) o pedido do OK. Fechar por outro caminho não arma nada.
    pub fn take_pixel_edit_request(&mut self) -> Option<PixelDialogChoice> {
        self.pixel.edit.request.take()
    }

    fn int_field(&self, id: NodeId, lo: i64, hi: i64) -> i64 {
        #[expect(clippy::cast_precision_loss, reason = "limites de |x| ≤ 65535")]
        let fallback = lo as f64;
        int(self.number_value(id).unwrap_or(fallback), lo, hi)
    }

    fn put(&mut self, id: NodeId, v: i64) {
        #[expect(clippy::cast_precision_loss, reason = "|v| ≤ 65535")]
        self.show(id, v as f64);
    }

    /// Escreve o valor de um campo do diálogo — também o TEXTO quando ele tem o foco:
    /// `set_number_value` poupa o texto de um campo focado, e o campo que acabou de receber o foco
    /// (o clique que fez o irmão comitar) guardaria o número velho, que o próximo blur comitaria
    /// por cima da sincronia.
    fn show(&mut self, id: NodeId, v: f64) {
        self.set_number_value(id, v);
        if self.focus_id == Some(id)
            && let Some(crate::interaction::InteractiveState::NumberInput { buffer, caret, .. }) =
                self.states.get_mut(&id)
        {
            *buffer = crate::interaction::format_number(v);
            *caret = buffer.len();
        }
    }

    /// Canvas Size: as bordas a partir do tamanho escrito e da âncora.
    fn sync_canvas_borders(&mut self) {
        let side = i64::from(NEW_SPRITE_MAX_SIDE);
        let (w, h) = self.pixel.edit.size;
        let nw = u32::try_from(self.int_field(ids::PIXEL_EDIT_CANVAS_W, 1, side)).unwrap_or(w);
        let nh = u32::try_from(self.int_field(ids::PIXEL_EDIT_CANVAS_H, 1, side)).unwrap_or(h);
        let b = pixel_anchor_borders(w, h, nw, nh, self.pixel.edit.anchor);
        for (id, v) in ids::PIXEL_EDIT_BORDERS.into_iter().zip(b) {
            self.put(id, i64::from(v));
        }
    }

    /// Canvas Size: uma borda editada refaz o tamanho do seu eixo; o lado preso a `1..=65535`
    /// devolve à borda editada o que sobrar.
    fn sync_canvas_size(&mut self, edited: NodeId) {
        let side = i64::from(NEW_SPRITE_MAX_SIDE);
        let (w, h) = self.pixel.edit.size;
        let horiz = edited == ids::PIXEL_EDIT_LEFT || edited == ids::PIXEL_EDIT_RIGHT;
        let (a, b, size_id, s) = if horiz {
            (
                ids::PIXEL_EDIT_LEFT,
                ids::PIXEL_EDIT_RIGHT,
                ids::PIXEL_EDIT_CANVAS_W,
                i64::from(w),
            )
        } else {
            (
                ids::PIXEL_EDIT_TOP,
                ids::PIXEL_EDIT_BOTTOM,
                ids::PIXEL_EDIT_CANVAS_H,
                i64::from(h),
            )
        };
        let other = if edited == a { b } else { a };
        let o = self.int_field(other, -side, side);
        let e = self.int_field(edited, -side, side);
        let n = (s + e + o).clamp(1, side);
        self.put(edited, n - s - o);
        self.put(size_id, n);
    }

    /// Sprite Size: um eixo a `pct` % do tamanho ao abrir — o campo % e (se `write_px`) o de px.
    fn sprite_axis(&mut self, horiz: bool, pct: f64, write_px: bool) {
        let (sw, sh) = self.pixel.edit.size;
        let (px_id, pct_id, s) = if horiz {
            (ids::PIXEL_EDIT_SPRITE_W, ids::PIXEL_EDIT_SPRITE_WPCT, sw)
        } else {
            (ids::PIXEL_EDIT_SPRITE_H, ids::PIXEL_EDIT_SPRITE_HPCT, sh)
        };
        self.show(pct_id, pct);
        if write_px {
            // O lado vem da percentagem (arredondado ao inteiro mais próximo).
            let v = f64::from(s) * pct / 100.0; // LITERAL-PX-OK: percentagem
            let px = int(v, 1, i64::from(NEW_SPRITE_MAX_SIDE));
            self.put(px_id, px);
        }
    }
}

/// A faixa dos campos % do Sprite Size: de 1 px no lado menor ao tecto no lado maior — a mesma
/// para os dois eixos, para o Lock Ratio nunca levar um deles para fora do documento.
fn pct_range(size: (u32, u32)) -> (f64, f64) {
    let (sw, sh) = (f64::from(size.0.max(1)), f64::from(size.1.max(1)));
    (100.0 / sw.min(sh), SIDE_MAX * 100.0 / sw.max(sh)) // LITERAL-PX-OK: percentagem
}
