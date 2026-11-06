//! **Os medidores da aba:** a pintura; o cálculo é de [`ph2d_sfx_lab::medida`].

pub use ph2d_sfx_lab::medida::*;

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, rect_to_vello, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

/// O topo da barra do pico: passa de 0 dB para o estouro aparecer.
const DB_TOPO: f32 = 6.0; // LITERAL-PX-OK: dB, não px

/// A largura da coluna: «Loudness (LUFS)» cabe em 11 px.
pub const LARGURA: f32 = 112.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const ROTULO_H: f32 = 16.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const VALOR_H: f32 = 22.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const BARRA_H: f32 = 6.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const CLIP_H: f32 = 18.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const VAO: f32 = 4.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const VAO_GRUPO: f32 = 16.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const RAIO: f32 = 3.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const TEXTO_PEQUENO: f32 = 11.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const TEXTO_VALOR: f32 = 15.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const ESPECTRO_H: f32 = 72.0; // LITERAL-PX-OK: geometria da coluna dos medidores
const BARRA_ESPECTRO_VAO: f32 = 1.0; // LITERAL-PX-OK: geometria da coluna dos medidores

fn fracao(db: f32) -> f32 {
    ((db - DB_PISO) / (DB_TOPO - DB_PISO)).clamp(0.0, 1.0)
}

fn numero(v: f32) -> String {
    if v.is_finite() {
        format!("{v:.1}")
    } else {
        tr("sfxlab.meter.silent").to_owned()
    }
}

struct Pintor<'a> {
    scene: &'a mut VectorScene,
    ts: &'a mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
}

impl Pintor<'_> {
    fn texto(&mut self, s: &str, r: Rect, px: f32, t: T) {
        let alto = self.ts.layout("Ag", px, f32::INFINITY).height();
        let ty = r.y + ((r.h - alto) / 2.0).max(0.0);
        let cor = resolve(t, self.theme);
        paint_text_elided(self.ts, self.scene, s, r.x, ty, px, r.w, cor);
    }

    fn rotulo(&mut self, s: &str, y: f32) -> f32 {
        let r = Rect::new(self.x, y, self.w, ROTULO_H);
        self.texto(s, r, TEXTO_PEQUENO, T::Text2);
        y + ROTULO_H
    }

    fn valor(&mut self, s: &str, y: f32) -> f32 {
        let r = Rect::new(self.x, y, self.w, VALOR_H);
        self.texto(s, r, TEXTO_VALOR, T::Text1);
        y + VALOR_H
    }

    fn barra(&mut self, db: f32, y: f32) -> f32 {
        let r = Rect::new(self.x, y, self.w, BARRA_H);
        fill_rounded_rect(self.scene, r, RAIO, resolve(T::Bg0, self.theme));
        let f = if db.is_finite() { fracao(db) } else { 0.0 };
        let cor = if db > 0.0 { T::Danger } else { T::Accent };
        let cheio = Rect::new(self.x, y, self.w * f, BARRA_H);
        fill_rounded_rect(self.scene, cheio, RAIO, resolve(cor, self.theme));
        y + BARRA_H
    }

    fn clip(&mut self, on: bool, y: f32) -> f32 {
        let r = Rect::new(self.x, y, self.w, CLIP_H);
        let (fundo, tinta) = if on {
            (T::Danger, T::AccentFg)
        } else {
            (T::Bg0, T::Text3)
        };
        fill_rounded_rect(self.scene, r, RAIO, resolve(fundo, self.theme));
        let s = tr("sfxlab.meter.clip");
        let w = self.ts.prefix_width(s, TEXTO_PEQUENO);
        let x = r.x + ((r.w - w) / 2.0).max(0.0);
        self.texto(s, Rect::new(x, y, r.w, CLIP_H), TEXTO_PEQUENO, tinta);
        y + CLIP_H
    }

    fn espectro(&mut self, bandas: &[f32; BANDAS], y: f32) -> f32 {
        let r = Rect::new(self.x, y, self.w, ESPECTRO_H);
        fill_rounded_rect(self.scene, r, RAIO, resolve(T::Bg0, self.theme));
        let passo = r.w / BANDAS as f32;
        let cor = resolve(T::Accent, self.theme);
        for (i, db) in bandas.iter().enumerate() {
            let f = ((db - DB_PISO) / -DB_PISO).clamp(0.0, 1.0);
            let h = r.h * f;
            let b = Rect::new(
                r.x + i as f32 * passo,
                r.y + r.h - h,
                (passo - BARRA_ESPECTRO_VAO).max(BARRA_ESPECTRO_VAO),
                h,
            );
            self.scene.fill_rect(rect_to_vello(b), cor);
        }
        y + ESPECTRO_H
    }
}

/// A coluna dos medidores, dentro da aba. O valor em texto e a unidade no rótulo.
pub fn pinta(
    m: Option<&Medida>,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let vazio = Medida::default();
    let m = m.unwrap_or(&vazio);
    let mut p = Pintor {
        scene,
        ts,
        theme,
        x: r.x,
        w: r.w,
    };
    let y = p.rotulo(tr("sfxlab.meter.peak"), r.y);
    let y = p.valor(&numero(m.pico_db), y);
    let y = p.barra(m.pico_db, y) + VAO;
    let y = p.clip(m.clip(), y) + VAO_GRUPO;
    let y = p.rotulo(tr("sfxlab.meter.loudness"), y);
    let y = p.valor(&numero(m.lufs), y);
    let y = p.barra(m.lufs, y) + VAO_GRUPO;
    let y = p.rotulo(tr("sfxlab.meter.spectrum"), y);
    let y = p.espectro(&m.espectro, y + VAO);
    p.rotulo(tr("sfxlab.meter.spectrum_range"), y);
}
