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

    /// Tira o documento e devolve a imagem dele: `(w, h, RGBA8 reto)`, com tudo o que estava por assentar (Paste,
    /// Transform, forma viva): fechar pode vir antes do quadro que assenta, e sem documento já não há onde.
    pub fn fecha_imagem(&mut self, key: u32) -> Option<(u32, u32, Vec<u8>)> {
        if self.solta.as_ref().is_some_and(|s| s.key == key) {
            self.assenta();
        }
        if self.gesture.as_ref().is_some_and(|g| g.key == key) {
            crate::viva::assenta(self);
            self.gesture = None;
        }
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

#[cfg(test)]
fn com_um_pixel_a_colar() -> (PixelApp, u32) {
    let mut app = PixelApp::default();
    let key = app.abre_imagem(2, 1, vec![0; 8]).expect("abre");
    let image = ph2d_pixel_core::transform::Image::new(1, 1, 4, u32::from_le_bytes([9, 8, 7, 255]));
    let f = ph2d_pixel_core::transform::Floating {
        x: 0,
        y: 0,
        image,
        keep: vec![true],
    };
    app.cola(key, f);
    (app, key)
}

/// *O Paste por assentar faz parte do trabalho: sair do Draw e gravar fecham o documento sem passar pelo quadro que assenta.*
/// *Mutação: tirar o `assenta` de `fecha_imagem` ⇒ o píxel perde-se.*
#[test]
fn a_moldura_solta_entra_na_imagem_que_volta() {
    let (mut app, key) = com_um_pixel_a_colar();
    let (_, _, px) = app.fecha_imagem(key).expect("fecha");
    assert_eq!(px[..4], [9, 8, 7, 255]);
    assert!(app.solta.is_none());
}

/// *A moldura de OUTRO documento não é desta edição.*
#[test]
fn a_moldura_de_outro_documento_fica() {
    let (mut app, key) = com_um_pixel_a_colar();
    let outro = app.abre_imagem(1, 1, vec![1, 2, 3, 255]).expect("abre");
    let (_, _, px) = app.fecha_imagem(outro).expect("fecha");
    assert_eq!(px, [1, 2, 3, 255]);
    assert!(app.solta.as_ref().is_some_and(|s| s.key == key));
}

/// *A forma viva por assentar também: desenhada, largada, e o Draw acaba antes de ela assentar.*
/// *Mutação: tirar o `viva::assenta` de `fecha_imagem` ⇒ a forma perde-se.*
#[test]
fn a_forma_viva_entra_na_imagem_que_volta() {
    let mut app = PixelApp::default();
    app.options.shape_fill = true;
    crate::input::set_tool(
        &mut app,
        crate::Tool::Shape(ph2d_editor_core::pixel_panel::PixelShape::STAR),
    );
    let key = app.abre_imagem(8, 8, vec![0; 8 * 8 * 4]).expect("abre");
    let doc = app.docs.get(&key).expect("doc");
    let (de, ate) = ((1, 1), (6, 6));
    let stroke = ph2d_pixel_core::Stroke::new(
        doc.layer,
        doc.frame,
        doc.sprite.value_of_rgba([200, 0, 0, 255]),
        crate::gesture::paint(&app.options, doc, None),
    );
    app.gesture = Some(crate::state::Gesture {
        entity: 1,
        key,
        stroke,
        kind: crate::gesture::Kind::Box { from: de, to: ate },
        held: false,
        cursor: ate,
        mods: crate::gesture::Mods::default(),
        viva: Some(crate::viva::Viva::new(
            ph2d_editor_core::pixel_panel::PixelShape::STAR,
            de,
            ate,
        )),
    });
    app.reaviva();
    let (_, _, px) = app.fecha_imagem(key).expect("fecha");
    assert!(px.chunks(4).any(|p| p[3] != 0), "a forma assentou");
    assert!(app.gesture.is_none());
}
