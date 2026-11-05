//! **Edit in Pixel**, a porta do Pixel Art: recebe e devolve RGBA8 reto. O objecto e a sprite são `ph2d_sprite_edit::no_pixel`.

use crate::PixelApp;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect, apply};

impl PixelApp {
    /// Um documento com `pixels`. Sem passos de desfazer: é o conteúdo do nascimento, e o primeiro Undo não o apaga.
    pub fn abre_imagem(&mut self, w: u32, h: u32, pixels: Vec<u8>) -> Option<u32> {
        let (mode, background) = (ColorMode::Rgba, Background::Transparent);
        let key = self
            .create(NewSprite {
                width: w,
                height: h,
                mode,
                background,
            })
            .ok()?;
        let s = &mut self.docs.get_mut(&key)?.sprite;
        let (layer, frame, rect) = (s.layers()[0].id, s.frames()[0].id, Rect::new(0, 0, w, h));
        let patch = Patch {
            layer,
            frame,
            bbox: Some(rect),
            rect,
            pixels,
        };
        if apply(s, Op::Patch(patch)).is_err() {
            self.docs.remove(&key);
            return None;
        }
        Some(key)
    }

    /// O objecto `bits` entra no Draw assim que nascer.
    pub fn edita_ao_nascer(&mut self, bits: u64) {
        self.wants_edit = Some(bits);
    }

    /// Tira o documento e devolve a imagem dele: `(w, h, RGBA8 reto)`.
    pub fn fecha_imagem(&mut self, key: u32) -> Option<(u32, u32, Vec<u8>)> {
        let s = self.docs.remove(&key)?.sprite;
        Some((
            s.width(),
            s.height(),
            ph2d_pixel_io::frame_rgba(&s, s.frames()[0].id),
        ))
    }
}

#[test]
fn a_imagem_vai_e_volta_igual_quando_nada_se_pinta() {
    let mut app = PixelApp::default();
    let px = vec![255, 0, 0, 255, 0, 0, 0, 0];
    let key = app.abre_imagem(2, 1, px.clone()).expect("abre");
    assert_eq!(app.fecha_imagem(key), Some((2, 1, px)));
    assert!(app.fecha_imagem(key).is_none());
}
