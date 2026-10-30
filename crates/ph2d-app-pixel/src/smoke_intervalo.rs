//! **A cena 16 — o intervalo da linha do tempo** (P6h): a mesma entrada que o oráculo do Aseprite
//! Trial correu (`docs/PixelArt/ferramentas/trial_timeline.py`, `tl.ase`): 6 quadros de 100, 200,
//! …, 600 ms, cada um com o quadrado de uma cor na camada «A» (por baixo, NÃO contínua); a camada
//! «B» (por cima, contínua) só com cels nos quadros 1 e 3; a tag «walk» sobre 2..4. Em Draw, no
//! quadro 2, com os quadros 2..3 escolhidos (a moldura à volta, «Frames [2...3]» nas propriedades).

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PixelRange, PixelRangeKind};
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Rect, layers};

use crate::PixelApp;
use crate::smoke_quadros::{camada, passo, patch};

pub const SIZE: (u32, u32) = (16, 16);
/// As durações (ms) e as cores dos 6 quadros (vermelho, verde, amarelo, azul, laranja, roxo).
pub const DURATIONS: [u32; 6] = [100, 200, 300, 400, 500, 600];
pub const COLORS: [[u8; 4]; 6] = [
    [220, 50, 50, 255],
    [40, 170, 70, 255],
    [235, 200, 40, 255],
    [50, 110, 230, 255],
    [240, 140, 40, 255],
    [150, 70, 200, 255],
];
/// O quadrado de cada quadro na camada A e o ponto da camada B.
pub const SQUARE: Rect = Rect::new(2, 2, 12, 12);
pub const DOT: Rect = Rect::new(6, 6, 4, 4);
/// A tag (posições, base 0) e o intervalo de abertura (posições).
pub const TAG: (u32, u32) = (1, 3);
pub const RANGE: [usize; 2] = [1, 2];

pub(crate) fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let a = app.docs.get(&key)?.sprite.layers()[0].id;
    passo(app, key, |s| {
        layers::edit(s, a, |l| "A".clone_into(&mut l.name))
    })?;
    let mut ids = vec![app.docs.get(&key)?.sprite.frames()[0].id];
    for _ in 1..DURATIONS.len() {
        let after = *ids.last()?;
        let (op, id) =
            frames::new_frame(&mut app.docs.get_mut(&key)?.sprite, after, NewFrame::Empty)?;
        app.commit(key, op).ok()?;
        ids.push(id);
    }
    for (i, &f) in ids.iter().enumerate() {
        patch(app, key, a, f, SQUARE, |_, _| COLORS[i])?;
        passo(app, key, |s| frames::set_duration(s, &[f], DURATIONS[i]))?;
    }
    let b = camada(app, key, a, "B")?;
    passo(app, key, |s| layers::edit(s, b, |l| l.continuous = true))?;
    for &i in &[0, 2] {
        patch(app, key, b, ids[i], DOT, |_, _| [250, 250, 250, 255])?;
    }
    passo(app, key, |s| frames::new_tag(s, "walk", TAG.0, TAG.1))?;
    let doc = app.docs.get_mut(&key)?;
    (doc.frame, doc.layer) = (ids[RANGE[0]], a);
    doc.range = PixelRange {
        kind: PixelRangeKind::Frames,
        layers: Vec::new(),
        frames: RANGE.iter().map(|&i| ids[i].0).collect(),
    };
    Some(bits as u32)
}
