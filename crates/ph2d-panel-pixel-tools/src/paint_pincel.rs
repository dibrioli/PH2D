//! ⭐ **O pincel na mão, os pincéis guardados e a Dynamics** (P7b): o menu do tipo de pincel e o
//! popup Dynamics do Aseprite (fotos `docs/PixelArt/aseprite/50`, `55`, `66`) no idioma do
//! Inspector — secções no painel em vez de popups, os mesmos gestos.

use super::*;
use ph2d_editor_core::IconId;
use ph2d_editor_core::pixel_panel::{PixelBrushSlotKind, PixelBrushThumb, PixelSensor};
use ph2d_editor_core::widget::{
    ButtonState, IconButtonStyle, IconGlyph, TRACK_MAX_PX, paint_icon_button, paint_range_slider,
};

impl Corpo<'_> {
    /// O padrão na mão: o tamanho, como ele se alinha e virar/girar (dono 02/11, §6.2 do P7a).
    pub(super) fn padrao(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let Some((w, h)) = info.pattern else {
            return y;
        };
        let (w, h) = (w.to_string(), h.to_string());
        let texto = tr_with("panel.pixel.brush.pattern_size", &[("w", &w), ("h", &h)]);
        let mut y = self.linha(&texto, ColorToken::Text2, y);
        let a = info.options.pattern_align;
        let segs: Vec<(&str, bool, NodeId)> = ids::PATTERN_ALIGNS
            .iter()
            .map(|&(p, id, _)| (tr(p.label_key()), p == a, id))
            .collect();
        y = self.escolha(tr("panel.pixel.pattern"), &segs, sec, y);
        let t = ids::TURNS.map(|(t, id)| (id, t.label_key()));
        let y = self.botoes(&t[..2], y);
        self.botoes(&t[2..], y)
    }

    /// A secção Dynamics aberta (o botão mora na fila do *Save Brush Here*).
    fn dinamica(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let mut y = y;
        let d = info.options.dynamics;
        y = self.caixas(
            &[(
                ids::DYN_STABILIZER,
                tr("panel.pixel.dynamics.stabilizer"),
                d.stabilizer > 0,
            )],
            sec,
            y,
        );
        if d.stabilizer > 0 {
            y = self.campo(
                tr("panel.pixel.dynamics.stabilizer_len"),
                ids::DYN_STABILIZER_LEN,
                sec,
                y,
            );
        }
        for row in ids::DynRow::ALL {
            let actual = crate::seam_pincel::canal(&d, row).sensor;
            let segs: Vec<(&str, bool, NodeId)> = ids::SENSORS
                .iter()
                .enumerate()
                .map(|(k, (s, key))| (tr(key), *s == actual, ids::sensor_id(row, k)))
                .collect();
            y = self.escolha(tr(row.label_key()), &segs, sec, y);
            y = match row {
                _ if actual == PixelSensor::Off => y,
                ids::DynRow::Size => self.campo(
                    tr("panel.pixel.dynamics.min_size"),
                    ids::DYN_MIN_SIZE,
                    sec,
                    y,
                ),
                ids::DynRow::Angle => self.campo(
                    tr("panel.pixel.dynamics.min_angle"),
                    ids::DYN_MIN_ANGLE,
                    sec,
                    y,
                ),
                ids::DynRow::Gradient => {
                    let segs: Vec<(&str, bool, NodeId)> = ids::DITHERS
                        .iter()
                        .map(|&(m, id)| (tr(m.label_key()), m == d.dither, id))
                        .collect();
                    self.escolha(tr("panel.pixel.dynamics.dither"), &segs, sec, y)
                }
            };
        }
        let usados = [d.size, d.angle, d.gradient].map(|c| c.sensor);
        if usados.iter().any(|s| *s != PixelSensor::Off) {
            y = self.bloco(tr("panel.pixel.dynamics.max_hint"), ColorToken::Text3, y);
        }
        for (k, (sensor, id, key)) in ids::THRESHOLDS.into_iter().enumerate() {
            if !usados.contains(&sensor) {
                continue;
            }
            y = self.linha(tr(key), ColorToken::Text3, y);
            let t = if sensor == PixelSensor::Pressure {
                d.pressure
            } else {
                d.velocity
            };
            let full = f32::from(ph2d_editor_core::pixel_panel::PIXEL_THRESHOLD_FULL);
            let linha = Rect::new(self.x, y, self.w, ROW_H_PX);
            let th = TRACK_MAX_PX;
            let trilha = Rect::new(self.x, y + (ROW_H_PX - th) * 0.5, self.w, th);
            let pega = self.pincel.pega.filter(|(j, _)| *j == k).map(|(_, p)| p);
            let (lo, hi) = (f32::from(t.lo) / full, f32::from(t.hi) / full);
            paint_range_slider(trilha, lo, hi, pega, self.scene, self.theme);
            self.hit_index.register(id, linha);
            self.pincel.trilhas[k] = Some(trilha);
            y += ROW_H_PX + ph2d_tokens::control_gap_px();
        }
        if usados.iter().any(|s| *s != PixelSensor::Off) {
            y = self.bloco(tr("panel.pixel.dynamics.range_hint"), ColorToken::Text3, y);
        }
        y
    }

    /// **Os pincéis guardados** (Alt+1…): a miniatura, recuperar, o cadeado, apagar; depois UMA
    /// fila com *Save Brush Here* e, no traço livre (`livre`), o botão da Dynamics — e ela aberta.
    /// Compacto de propósito: tudo isto fica ACIMA da paleta (smoke do dono 01/11, a dobra).
    pub(super) fn guardados(
        &mut self,
        info: &PixelPanelInfo,
        livre: bool,
        sec: Seccao,
        y: f32,
    ) -> f32 {
        let mut y = y;
        if !info.brushes.is_empty() {
            y = self.linha(tr("panel.pixel.brush.saved_title"), ColorToken::Text3, y);
        }
        let side = self.lado();
        let gap = Spacing::Xs.px();
        for (n, s) in info.brushes.iter().enumerate() {
            let mini = Rect::new(self.x, y, side, side);
            let rotulo = match &s.kind {
                PixelBrushSlotKind::Pattern(t) => {
                    miniatura(self.scene, mini, t);
                    None
                }
                PixelBrushSlotKind::Shape { shape, size } => {
                    let size = size.to_string();
                    Some(tr_with(
                        "panel.pixel.brush.slot_shape",
                        &[("shape", &tr(shape.label_key())), ("size", &size)],
                    ))
                }
            };
            stroke_rect(
                self.scene,
                mini,
                StrokeToken::Hairline.px(),
                tinta(ColorToken::Border, self.theme),
            );
            let nome = rotulo_do_espaco(n);
            let nome = rotulo.map_or(nome.clone(), |r| format!("{nome} · {r}"));
            let x = self.x + side + gap;
            let icones = 2.0 * (side + gap);
            let r = Rect::new(x, y, (self.w - (x - self.x) - icones).max(1.0), side);
            let id = ids::brush_slot_id(n);
            if !self.botao_no_cartao(r, id, &nome, false) {
                let b = Button::new(id, &nome).visual(self.store.button_visual(id));
                paint_button(&b, r, self.scene, self.text_system, self.theme);
                self.hit_index.register(id, r);
            }
            let cadeado = if s.locked {
                IconId::LockKeyhole
            } else {
                IconId::LockKeyholeOpen
            };
            let icones = [
                (ids::brush_lock_id(n), cadeado),
                (ids::brush_delete_id(n), IconId::Trash),
            ];
            for (i, (id, icone)) in icones.into_iter().enumerate() {
                #[expect(clippy::cast_precision_loss, reason = "dois ícones")]
                let ix = self.x + self.w - (2.0 - i as f32) * side - (1.0 - i as f32) * gap;
                let ir = Rect::new(ix, y, side, side);
                if !self.icone_no_cartao(ir, id, icone, false) {
                    paint_icon_button(
                        ir,
                        IconGlyph::Builtin(icone),
                        IconButtonStyle::Compact,
                        self.store.button_visual(id),
                        self.scene,
                        self.theme,
                    );
                    self.hit_index.register(id, ir);
                }
            }
            y += side + ph2d_tokens::control_gap_px();
        }
        let aberta = livre && self.pincel.dinamica;
        let mut fila = vec![(ids::SAVE_HERE, tr("panel.pixel.brush.save_here"), false)];
        if livre {
            fila.push((ids::DYNAMICS, tr("panel.pixel.dynamics"), aberta));
        }
        // Um botão por fila, na largura toda: «Save Brush Here» não cabe em meia largura da doca.
        for (i, (id, rotulo, premido)) in fila.into_iter().enumerate() {
            if i > 0 {
                y += side + ph2d_tokens::control_gap_px();
            }
            let r = Rect::new(self.x, y, self.w, side);
            let visual = if premido {
                (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
            } else {
                self.store.button_visual(id)
            };
            if !self.botao_no_cartao(r, id, rotulo, premido) {
                let b = Button::new(id, rotulo).visual(visual);
                paint_button(&b, r, self.scene, self.text_system, self.theme);
                self.hit_index.register(id, r);
            }
        }
        y += side + ph2d_tokens::control_gap_px();
        if aberta {
            self.dinamica(info, sec, y)
        } else {
            y
        }
    }
}

/// «Alt+n» até ao 9.º espaço (os atalhos), «Brush n» depois.
pub(crate) fn rotulo_do_espaco(n: usize) -> String {
    let k = (n + 1).to_string();
    let key = if n < 9 {
        "panel.pixel.brush.slot"
    } else {
        "panel.pixel.brush.slot_far"
    };
    tr_with(key, &[("n", &k)])
}

/// A miniatura de um padrão, centrada e a escala inteira que cabe no quadrado `r`.
fn miniatura(scene: &mut VectorScene, r: Rect, t: &PixelBrushThumb) {
    let lado = t.w.max(t.h).max(1);
    #[expect(clippy::cast_precision_loss, reason = "≤ 16 células por lado")]
    let cel = (r.w.min(r.h) / lado as f32).floor().max(1.0);
    #[expect(clippy::cast_precision_loss, reason = "≤ 16 células por lado")]
    let (ox, oy) = (
        r.x + (r.w - cel * t.w as f32) * 0.5,
        r.y + (r.h - cel * t.h as f32) * 0.5,
    );
    for (i, c) in t.cells.iter().enumerate() {
        let Some(c) = c else { continue };
        let w = t.w.max(1) as usize;
        #[expect(clippy::cast_precision_loss, reason = "≤ 16 células por lado")]
        let (x, y) = ((i % w) as f32, (i / w) as f32);
        let q = Rect::new(ox + x * cel, oy + y * cel, cel, cel);
        ph2d_editor_core::paint_shapes::fill_rounded_rect_srgb8(scene, q, 0.0, *c);
    }
}
