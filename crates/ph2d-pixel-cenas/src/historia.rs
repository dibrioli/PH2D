//! A cena 25 (P12, `docs/PixelArt/P12_plano.md` §8): um boneco 32 × 32 a andar em 4 quadros, três
//! camadas (Color, Shadow, Outline), sete passos de desfazer com o nome do caminho real, o `Fill`
//! trocado de `F` para `Shift+F` num ficheiro de teclas DA CENA, grade 8 × 8 e user data na Color.
//! A montagem fica fora do desfazer; os [`PASSOS`] entram pela história, um a um.

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{
    Background, ColorMode, Grid, Label, LayerId, NewSprite, Op, Patch, PropValue, Sprite, UserData,
    apply, layers,
};
use ph2d_pixel_keys::{Chord, Context, Keymap, PixelAction};

pub const SPEC: NewSprite = NewSprite {
    width: 32,
    height: 32,
    mode: ColorMode::Rgba,
    background: Background::Transparent,
};
pub const FRAMES: u32 = 4;
/// As camadas de baixo para cima.
pub const LAYERS: [&str; 3] = ["Color", "Shadow", "Outline"];
/// Os nomes dos passos como os caminhos reais os gravam (`ph2d_pixel_ponte::nomes`, `gesto::rotulo`).
pub const PASSOS: [&str; 7] = [
    "pixel.history.pencil",
    "pixel.history.line",
    "pixel.history.paint_bucket",
    "pixel.keys.layer.new",
    "pixel.history.pencil",
    "pixel.keys.edit.flip_h",
    "pixel.filter_dialog.hue_saturation.title",
];
pub const USER_TEXT: &str = "base colors";
pub const USER_COLOR: [u8; 4] = [64, 160, 255, 255];
pub const GRID_COLOR: [u8; 4] = [255, 64, 160, 255];
/// O `Fill` que «o utilizador» mudou.
pub const FILL_KEY: Chord = Chord::new(winit::keyboard::KeyCode::KeyF).shift();

const ORIGIN: (i32, i32) = (8, 5);
const T: [u8; 4] = [0; 4];
const OUTLINE: [u8; 4] = [34, 24, 36, 255];
const HAIR: [u8; 4] = [120, 70, 40, 255];
const SKIN: [u8; 4] = [246, 200, 160, 255];
const EYE: [u8; 4] = [30, 30, 50, 255];
const SHIRT: [u8; 4] = [40, 170, 160, 255];
const BELT: [u8; 4] = [90, 60, 40, 255];
const PANTS: [u8; 4] = [60, 90, 190, 255];
const BOOT: [u8; 4] = [80, 50, 36, 255];
const BUCKLE: [u8; 4] = [250, 210, 60, 255];
const BAND: [u8; 4] = [220, 40, 50, 255];
const ORANGE: [u8; 4] = [240, 140, 40, 255];
const PURPLE: [u8; 4] = [150, 70, 200, 255];
const SPARK: [u8; 4] = [255, 240, 120, 255];
const GROUND: [u8; 4] = [20, 20, 30, 90];
/// A sombra do lado direito: escura e translúcida, serve a qualquer cor por baixo.
const SHADE: [u8; 4] = [30, 10, 50, 110];

const TOP: [&[u8; 16]; 16] = [
    b"......OOOO......",
    b"....OOHHHHOO....",
    b"...OHHHHHHHHO...",
    b"...OHHHHHHHHO...",
    b"...OHSSSSSSHO...",
    b"...OSSESSESSO...",
    b"...OSSSSSSSSO...",
    b"...OSSSSSSSSO...",
    b"....OSSSSSSO....",
    b".....OOOOOO.....",
    b"....OTTTTTTO....",
    b"...OTTTTTTTTO...",
    b"..OTTTTTTTTTTO..",
    b"..OSOTTTTTTOSO..",
    b"..OO.OTTTTO.OO..",
    b".....OBBBBO.....",
];
/// As pernas de cada quadro: abertas, a passar, abertas com a outra à frente, a passar.
const LEGS: [[&[u8; 16]; 6]; 4] = [
    [
        b".....OPPPPO.....",
        b"....OPPOOPPO....",
        b"....OPPO.OPPO...",
        b"...OPPO...OPPO..",
        b"...OKKO...OKKKO.",
        b"...OOOO...OOOOO.",
    ],
    [
        b".....OPPPPO.....",
        b".....OPPPPO.....",
        b"......OPPO......",
        b"......OPPO......",
        b"......OKKKO.....",
        b"......OOOOO.....",
    ],
    [
        b".....OPPPPO.....",
        b"....OPPOOPPO....",
        b"...OPPO..OPPO...",
        b"..OPPO....OPPO..",
        b".OKKKO....OKKO..",
        b".OOOOO....OOOO..",
    ],
    [
        b".....OPPPPO.....",
        b".....OPPPPO.....",
        b"......OPPO......",
        b"......OPPO......",
        b".....OKKKO......",
        b".....OOOOO......",
    ],
];

fn code(k: u32, x: i32, y: i32) -> u8 {
    let (cx, cy) = (x - ORIGIN.0, y - ORIGIN.1);
    if !(0..16).contains(&cx) {
        return b'.';
    }
    let row = match cy {
        0..16 => TOP[cy as usize],
        16..22 => LEGS[k as usize % 4][cy as usize - 16],
        _ => return b'.',
    };
    row[cx as usize]
}

fn base(c: u8) -> [u8; 4] {
    match c {
        b'H' => HAIR,
        b'S' => SKIN,
        b'E' => EYE,
        b'T' => SHIRT,
        b'B' => BELT,
        b'P' => PANTS,
        b'K' => BOOT,
        _ => T,
    }
}

/// A camada Color no quadro `k`, depois de `n` passos da história (só o quadro 0 os recebe).
#[must_use]
pub fn color(k: u32, n: usize, x: i32, y: i32) -> [u8; 4] {
    let (c, cx, cy) = (code(k, x, y), x - ORIGIN.0, y - ORIGIN.1);
    let f0 = k == 0;
    match c {
        b'B' if f0 && n >= 1 && (7..9).contains(&cx) => BUCKLE,
        b'H' if f0 && n >= 2 && cy == 3 => BAND,
        b'T' if f0 && n >= 7 => PURPLE,
        b'T' if f0 && n >= 3 => ORANGE,
        _ => base(c),
    }
}

/// A sombra: o lado direito de cada cor e a sombra no chão.
#[must_use]
pub fn shadow(k: u32, x: i32, y: i32) -> [u8; 4] {
    let c = code(k, x, y);
    if !matches!(c, b'.' | b'O' | b'E') && code(k, x + 1, y) == b'O' {
        return SHADE;
    }
    let (cx, cy) = (x - ORIGIN.0, y - ORIGIN.1);
    if cy == 22 && (2..14).contains(&cx) {
        return GROUND;
    }
    T
}

#[must_use]
pub fn outline(k: u32, x: i32, y: i32) -> [u8; 4] {
    if code(k, x, y) == b'O' { OUTLINE } else { T }
}

/// A camada nova (passo 4): a estrela do Pencil à direita da cabeça; o Flip leva-a para a esquerda.
#[must_use]
pub fn spark(flipped: bool, x: i32, y: i32) -> [u8; 4] {
    let x = if flipped { 31 - x } else { x };
    let (dx, dy) = (x - 27, y - 4);
    if (dx == 0 && dy.abs() <= 2) || (dy == 0 && dx.abs() <= 2) {
        SPARK
    } else {
        T
    }
}

fn patch(s: &Sprite, layer: LayerId, k: u32, f: impl Fn(i32, i32) -> [u8; 4]) -> Option<Op> {
    let rect = s.bounds();
    let frame = s.frames().get(k as usize)?.id;
    let mut pixels = Vec::with_capacity(rect.area() * 4);
    for y in 0..rect.h as i32 {
        for x in 0..rect.w as i32 {
            pixels.extend_from_slice(&f(x, y));
        }
    }
    Some(Op::Patch(Patch {
        layer,
        frame,
        bbox: Some(rect),
        rect,
        pixels,
    }))
}

fn user_data() -> UserData {
    let mut mapa = ph2d_pixel_core::PropMap::new();
    mapa.insert("hp".to_owned(), PropValue::I32(10));
    UserData {
        color: USER_COLOR,
        text: USER_TEXT.to_owned(),
        props: [(String::new(), mapa)].into(),
    }
}

/// **Monta o boneco** (fora do desfazer): as três camadas, os quatro quadros, a grade 8 × 8 e o user
/// data da Color. Devolve a camada Color.
pub fn monta(s: &mut Sprite) -> Option<LayerId> {
    let cor = s.layers().first()?.id;
    let op = layers::edit(s, cor, |l| {
        LAYERS[0].clone_into(&mut l.name);
        l.user = user_data();
    })?;
    apply(s, op).ok()?;
    let mut ids = vec![cor];
    for nome in &LAYERS[1..] {
        let (op, id) = layers::new_layer(s, *ids.last()?)?;
        apply(s, op).ok()?;
        let op = layers::edit(s, id, |l| (*nome).clone_into(&mut l.name))?;
        apply(s, op).ok()?;
        ids.push(id);
    }
    for _ in 1..FRAMES {
        let ultimo = s.frames().last()?.id;
        let (op, _) = frames::new_frame(s, ultimo, NewFrame::Empty)?;
        apply(s, op).ok()?;
    }
    for k in 0..FRAMES {
        let ops = [
            patch(s, ids[0], k, |x, y| color(k, 0, x, y))?,
            patch(s, ids[1], k, |x, y| shadow(k, x, y))?,
            patch(s, ids[2], k, |x, y| outline(k, x, y))?,
        ];
        for op in ops {
            apply(s, op).ok()?;
        }
    }
    let mut prefs = *s.prefs();
    prefs.grid = Grid {
        w: 8,
        h: 8,
        color: GRID_COLOR,
        opacity: 200,
        visible: true,
        ..Grid::default()
    };
    prefs.onion.tint = true;
    s.set_prefs(prefs);
    Some(cor)
}

/// O passo `i` da história sobre o sprite como está (os ids da camada nova reservam-se nele).
pub fn passo(s: &mut Sprite, i: usize) -> Option<(Op, Label)> {
    let cor = s.layers().first()?.id;
    let op = match i {
        0..3 => patch(s, cor, 0, |x, y| color(0, i + 1, x, y))?,
        3 => {
            let top = s.layers().last()?.id;
            layers::new_layer(s, top)?.0
        }
        4 | 5 => patch(s, s.layers().last()?.id, 0, |x, y| spark(i == 5, x, y))?,
        6 => patch(s, cor, 0, |x, y| color(0, 7, x, y))?,
        _ => return None,
    };
    Some((op, Label::new(*PASSOS.get(i)?)))
}

/// As teclas da cena: a fábrica com o `Fill` em `Shift+F`, gravadas num ficheiro DA CENA (nunca o
/// `~/.ph2d/pixel-keys.txt` do dono). Devolve o mapa e o ficheiro.
pub fn teclas() -> Option<(Keymap, std::path::PathBuf)> {
    let mut k = Keymap::factory();
    k.remove(
        PixelAction::Fill,
        Chord::new(winit::keyboard::KeyCode::KeyF),
    );
    k.assign(PixelAction::Fill, FILL_KEY, Context::Normal)
        .ok()?;
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pasta = format!("ph2d-pixel-cena25-{}-{n}", std::process::id());
    let p = std::env::temp_dir().join(pasta).join("pixel-keys.txt");
    ph2d_pixel_keys::user_file::save_to(&p, &k).ok()?;
    Some((k, p))
}
