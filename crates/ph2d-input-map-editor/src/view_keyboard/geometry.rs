//! As tabelas do desenho: o teclado de notebook (em unidades de tecla, plano §13.2) e o comando (em fracções da caixa). Fonte das
//! proporções: a maquete `docs/InputMap/maquetes/03_B_teclado_e_comando.png` (B1 e B3), medida em pixels.

use ph2d_input::{Binding, GamepadAxis, GamepadButton, Key};

/// Uma tecla: código, coluna e linha (em unidades de tecla), largura e altura (em unidades e em filas).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KeyCap {
    pub key: Key,
    pub col: f32,
    pub row: f32,
    pub span: f32,
    pub rows: f32,
}

const fn k(code: u32, col: f32, row: f32, span: f32) -> KeyCap {
    KeyCap {
        key: Key(code),
        col,
        row,
        span,
        rows: 1.0,
    }
}

/// A parte do passo que a tecla enche (o resto é o vão).
pub(crate) const KEY_FILL: f32 = 0.88; // LITERAL-PX-OK: fração do passo, maquete B1
/// A altura da tecla e o passo entre linhas, em passos de tecla.
pub(crate) const KEY_H: f32 = 0.68; // LITERAL-PX-OK: fração do passo, maquete B1
pub(crate) const ROW_PITCH: f32 = 0.8; // LITERAL-PX-OK: fração do passo, maquete B1
/// O passo não passa deste múltiplo da altura de um campo.
pub(crate) const PITCH_MAX_ROWS: f32 = 1.6; // LITERAL-PX-OK: múltiplo do token de linha
/// O tecto da B: o teclado enche o cartão como na maquete B1 (a C fica no de cima).
pub(crate) const PITCH_MAX_ROWS_B: f32 = 2.0; // LITERAL-PX-OK: múltiplo do token de linha

const UP: u32 = 0xF700;
const DOWN: u32 = 0xF701;
const LEFT: u32 = 0xF702;
const RIGHT: u32 = 0xF703;

/// A largura do bloco principal (ANSI) e o vão até ao numérico, em unidades de tecla. UM arranjo só (dono 10/11,
/// plano §13.2): o numérico ao lado do principal, como na foto — nunca desce; abaixo do passo mínimo o desenho rola.
pub(crate) const MAIN_W: f32 = 15.0; // LITERAL-PX-OK: unidades de tecla, ANSI
const MAIN_ROWS: f32 = 7.0; // LITERAL-PX-OK: filas do bloco principal (R0..R6)
const PAD_GAP: f32 = 0.25; // LITERAL-PX-OK: unidades de tecla, foto do dono
const PAD_W: f32 = 4.0; // LITERAL-PX-OK: colunas do numérico
/// A coluna da ← (R6).
const ARROWS_COL: f32 = 12.0; // LITERAL-PX-OK: unidades de tecla, ANSI

/// A extensão do desenho (colunas, filas) em unidades de tecla.
pub(crate) const fn extent() -> (f32, f32) {
    (MAIN_W + PAD_GAP + PAD_W, MAIN_ROWS)
}

/// Uma fila de teclas de 1 u a partir de `col`.
fn run(out: &mut Vec<KeyCap>, codes: impl IntoIterator<Item = u32>, col: f32, row: f32) {
    for (i, c) in codes.into_iter().enumerate() {
        out.push(k(c, col + i as f32, row, 1.0));
    }
}

fn ascii(s: &str) -> impl Iterator<Item = u32> + '_ {
    s.chars().map(u32::from)
}

/// O bloco principal ANSI de 15 u (plano §13.2, foto do dono).
fn main_block() -> Vec<KeyCap> {
    let mut out = Vec::new();
    // R0: Esc F1–F12 Insert PrtSc Delete — 16 teclas sobre 15 u.
    const F1: u32 = 0xF704;
    let r0: Vec<u32> = std::iter::once(0x1B)
        .chain(F1..F1 + 12)
        .chain([0xF727, 0xF72E, 0xF728])
        .collect();
    let w0 = MAIN_W / r0.len() as f32;
    for (i, c) in r0.into_iter().enumerate() {
        out.push(k(c, i as f32 * w0, 0.0, w0));
    }
    run(&mut out, ascii("`1234567890-="), 0.0, 1.0);
    out.push(k(0x08, 13.0, 1.0, 2.0)); // LITERAL-PX-OK: unidades ANSI
    out.push(k(0x09, 0.0, 2.0, 1.5)); // LITERAL-PX-OK: unidades ANSI
    run(&mut out, ascii("QWERTYUIOP[]"), 1.5, 2.0); // LITERAL-PX-OK: unidades ANSI
    out.push(k(0x5C, 13.5, 2.0, 1.5)); // LITERAL-PX-OK: unidades ANSI
    out.push(k(0xF716, 0.0, 3.0, 1.75)); // LITERAL-PX-OK: unidades ANSI
    run(&mut out, ascii("ASDFGHJKL;'"), 1.75, 3.0); // LITERAL-PX-OK: unidades ANSI
    out.push(k(0x0D, 12.75, 3.0, 2.25)); // LITERAL-PX-OK: unidades ANSI
    out.push(k(0xF710, 0.0, 4.0, 2.25)); // LITERAL-PX-OK: unidades ANSI
    run(&mut out, ascii("ZXCVBNM,./"), 2.25, 4.0); // LITERAL-PX-OK: unidades ANSI
    out.push(k(0xF711, 12.25, 4.0, 2.75)); // LITERAL-PX-OK: unidades ANSI
    // R5: Ctrl Fn Win Alt Space Alt Ctrl, e a ↑ na coluna 13.
    let r5: [(u32, f32); 7] = [
        (0xF712, 1.25), // LITERAL-PX-OK: unidades ANSI
        (0xF71A, 1.0),
        (0xF718, 1.0),
        (0xF714, 1.25), // LITERAL-PX-OK: unidades ANSI
        (0x20, 5.0),    // LITERAL-PX-OK: unidades ANSI
        (0xF715, 1.25), // LITERAL-PX-OK: unidades ANSI
        (0xF713, 1.25), // LITERAL-PX-OK: unidades ANSI
    ];
    let mut x = 0.0;
    for (c, w) in r5 {
        out.push(k(c, x, 5.0, w));
        x += w;
    }
    out.push(k(UP, ARROWS_COL + 1.0, 5.0, 1.0));
    run(&mut out, [LEFT, DOWN, RIGHT], ARROWS_COL, 6.0);
    out
}

/// O bloco do numérico, de 4 × 6 u, com origem em `(ox, oy)`.
fn numpad(ox: f32, oy: f32) -> Vec<KeyCap> {
    const KP: u32 = 0xF740;
    let mut out = Vec::new();
    run(&mut out, [0xF729, 0xF72B, 0xF72C, 0xF72D], ox, oy);
    run(&mut out, [0xF717, 0xF74B, 0xF74C, 0xF74D], ox, oy + 1.0);
    run(&mut out, [KP + 7, KP + 8, KP + 9], ox, oy + 2.0);
    out.push(KeyCap {
        rows: 2.0,
        ..k(0xF74E, ox + 3.0, oy + 2.0, 1.0)
    });
    run(&mut out, [KP + 4, KP + 5, KP + 6], ox, oy + 3.0);
    run(&mut out, [KP + 1, KP + 2, KP + 3], ox, oy + 4.0);
    out.push(KeyCap {
        rows: 2.0,
        ..k(0xF74F, ox + 3.0, oy + 4.0, 1.0)
    });
    out.push(k(KP, ox, oy + 5.0, 2.0));
    out.push(k(0xF74A, ox + 2.0, oy + 5.0, 1.0));
    out
}

/// O teclado de notebook inteiro: o principal e, ao lado dele, o numérico.
pub(crate) fn keyboard() -> Vec<KeyCap> {
    let mut out = main_block();
    out.extend(numpad(MAIN_W + PAD_GAP, 0.0));
    out
}

/// As teclas desenhadas, pela ordem da tabela.
#[must_use]
pub fn drawn_keys() -> Vec<Key> {
    keyboard().into_iter().map(|c| c.key).collect()
}

/// As setas pintam-se com ícone (a fonte não tem os glifos ←↑↓→).
pub(crate) fn arrow_icon(key: Key) -> Option<ph2d_editor_core::icons::IconId> {
    use ph2d_editor_core::icons::IconId;
    match key.0 {
        UP => Some(IconId::ArrowUp),
        DOWN => Some(IconId::ArrowDown),
        LEFT => Some(IconId::ArrowLeft),
        RIGHT => Some(IconId::ArrowRight),
        _ => None,
    }
}

/// A forma de um alvo do comando.
#[derive(Clone, Copy)]
pub(crate) enum PadShape {
    /// Pílula (gatilhos, ombros, Back e Start): a largura cresce até caber o nome.
    Pill,
    /// Botão redondo da face (A B X Y): raio = meia altura.
    Round,
    /// Um braço da cruz, com a seta dentro.
    Arm(ph2d_editor_core::icons::IconId),
    /// Uma metade de manche: a seta pequena à volta do anel `stick` (índice em [`STICKS`]).
    Half(ph2d_editor_core::icons::IconId, usize),
}

/// Um alvo do comando: o que liga, a chave do nome curto, centro (fracções da caixa) e meio-tamanho (fracções
/// da ALTURA da caixa nos dois eixos, para um redondo ser redondo).
pub(crate) struct PadPart {
    pub binding: Binding,
    pub short_key: &'static str,
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
    pub shape: PadShape,
}

const fn part(
    binding: Binding,
    short_key: &'static str,
    (cx, cy): (f32, f32),
    (hw, hh): (f32, f32),
    shape: PadShape,
) -> PadPart {
    PadPart {
        binding,
        short_key,
        cx,
        cy,
        hw,
        hh,
        shape,
    }
}

const fn b(p: GamepadButton) -> Binding {
    Binding::PadButton(p)
}

const fn half(axis: GamepadAxis, positive: bool) -> Binding {
    Binding::PadAxis { axis, positive }
}

/// A caixa do comando: largura sobre altura.
pub(crate) const PAD_ASPECT: f32 = 1.75; // LITERAL-PX-OK: razão adimensional, maquete B3
/// A parte da largura do desenho que o comando enche (maquete B3: o corpo vai de ~1/8 a ~7/8).
pub(crate) const PAD_FILL: f32 = 0.76; // LITERAL-PX-OK: fração da largura, maquete B3
/// O tecto da altura do comando, em alturas de campo.
pub(crate) const PAD_MAX_ROWS: f32 = 10.0; // LITERAL-PX-OK: múltiplo do token de linha
/// O tecto com a B empilhada (1280): o curso e os campos Dead/Press da aberta cabem sem rolar (medido em
/// (297,28,690,604): a 8 alturas o campo acabava 31 px abaixo da vista; a 6,5 sobram 11 px).
pub(crate) const PAD_MAX_ROWS_STACKED: f32 = 10.0; // LITERAL-PX-OK: múltiplo do token de linha

/// Um troço do contorno: recta até, ou cúbica (dois controlos e o fim), em fracções da caixa.
#[derive(Clone, Copy)]
pub(crate) enum BodySeg {
    Line((f32, f32)),
    Cubic((f32, f32), (f32, f32), (f32, f32)),
}

/// O contorno do lado esquerdo, do meio do topo ao meio do fundo (o direito é o espelho em x): ombro curvo,
/// a pega que desce e volta, e o fundo direito entre as pegas.
pub(crate) const BODY_START: (f32, f32) = (0.5, 0.17); // LITERAL-PX-OK: maquete B3
pub(crate) const BODY: [BodySeg; 6] = [
    BodySeg::Line((0.27, 0.17)), // LITERAL-PX-OK: maquete B3
    BodySeg::Cubic((0.17, 0.17), (0.11, 0.21), (0.08, 0.33)), // LITERAL-PX-OK: maquete B3
    BodySeg::Cubic((0.04, 0.5), (0.0, 0.78), (0.03, 0.9)), // LITERAL-PX-OK: maquete B3
    BodySeg::Cubic((0.05, 1.0), (0.15, 1.0), (0.2, 0.94)), // LITERAL-PX-OK: maquete B3
    BodySeg::Cubic((0.24, 0.89), (0.28, 0.83), (0.35, 0.82)), // LITERAL-PX-OK: maquete B3
    BodySeg::Line((0.5, 0.82)),  // LITERAL-PX-OK: maquete B3
];

// As medidas do comando, maquete B3. Centros em fracções da caixa; meio-tamanhos em fracções da altura.
const PILL: (f32, f32) = (0.11, 0.045); // LITERAL-PX-OK: maquete B3
const SMALL_PILL: (f32, f32) = (0.07, 0.035); // LITERAL-PX-OK: maquete B3
const BTN: (f32, f32) = (0.06, 0.06); // LITERAL-PX-OK: maquete B3
const ARM: (f32, f32) = (0.045, 0.045); // LITERAL-PX-OK: maquete B3
/// A seta de uma metade de manche tem o tamanho do braço da cruz: o mesmo glifo das outras setas, com a plaquinha e as
/// barras à vista dos dois lados quando ligada (onda 3, D3: a 0,04 a seta lia-se como uma caixinha).
const TIP: (f32, f32) = ARM;
/// Os gatilhos ASSENTAM na aresta do topo do corpo (o fundo da pílula em `BODY_START.1`, maquete B3); os ombros
/// ficam logo abaixo da aresta, já dentro do corpo, entre a manche esquerda (o espelho: o Y) e o Back/Start.
const LT: (f32, f32) = (0.2, BODY_START.1 - PILL.1); // LITERAL-PX-OK: maquete B3
/// (0,345 e não os 0,34 da maquete: a seta ↑ do LS alargou na onda 3 e tocava o LB.)
const LB: (f32, f32) = (0.345, BODY_START.1 + 0.07); // LITERAL-PX-OK: maquete B3
/// (0,445/0,555 e não 0,44/0,56: a seta ↑ do RS alargou na onda 3 e tocava o Start.)
const BACK: (f32, f32) = (0.445, 0.34); // LITERAL-PX-OK: maquete B3
const START: (f32, f32) = (0.555, 0.34); // LITERAL-PX-OK: maquete B3
const DPAD: (f32, f32) = (0.41, 0.63); // LITERAL-PX-OK: maquete B3
const FACE: (f32, f32) = (0.8, 0.4); // LITERAL-PX-OK: maquete B3
/// O passo dos botões da face e dos braços da cruz (fracções da altura).
const FACE_STEP: f32 = 0.13; // LITERAL-PX-OK: maquete B3
const ARM_STEP: f32 = 0.09; // LITERAL-PX-OK: maquete B3
/// O vão entre o anel da manche e as setas das metades (fracção da altura).
const TIP_GAP: f32 = 0.02; // LITERAL-PX-OK: maquete B3

/// As duas manches: centro (fracções da caixa), a chave do nome e o raio do anel (fracção da altura).
pub(crate) const STICKS: [((f32, f32), &str, f32); 2] = [
    ((0.27, 0.43), "input_map_keyboard.pad.ls", 0.1), // LITERAL-PX-OK: maquete B3
    ((0.63, 0.56), "input_map_keyboard.pad.rs", 0.085), // LITERAL-PX-OK: maquete B3
];

/// A distância do centro de uma manche à seta de cada metade (fracção da altura).
pub(crate) fn tip_reach(r: f32) -> f32 {
    r + TIP_GAP + TIP.1
}

/// Os alvos do comando. As metades de manche são as quatro setas à volta de cada anel.
pub(crate) fn pad() -> Vec<PadPart> {
    use GamepadAxis as A;
    use GamepadButton as P;
    use PadShape::{Arm, Half, Pill, Round};
    use ph2d_editor_core::icons::IconId as I;
    let wide = |h: f32| h / PAD_ASPECT;
    let (fx, fy) = FACE;
    let (px, py) = DPAD;
    let mirror = |(x, y): (f32, f32)| (1.0 - x, y);
    let pills = [
        (P::LeftTrigger, "input_map_keyboard.pad.lt", LT, PILL),
        (P::LeftBumper, "input_map_keyboard.pad.lb", LB, SMALL_PILL),
        (
            P::RightBumper,
            "input_map_keyboard.pad.rb",
            mirror(LB),
            SMALL_PILL,
        ),
        (
            P::RightTrigger,
            "input_map_keyboard.pad.rt",
            mirror(LT),
            PILL,
        ),
    ];
    let mut out: Vec<PadPart> = pills
        .into_iter()
        .map(|(p, k, at, size)| part(b(p), k, at, size, Pill))
        .collect();
    out.extend([
        part(
            b(P::Select),
            "input_map_keyboard.pad.back",
            BACK,
            SMALL_PILL,
            Pill,
        ),
        part(
            b(P::Start),
            "input_map_keyboard.pad.start",
            START,
            SMALL_PILL,
            Pill,
        ),
        part(
            b(P::North),
            "input_map_keyboard.pad.y",
            (fx, fy - FACE_STEP),
            BTN,
            Round,
        ),
        part(
            b(P::South),
            "input_map_keyboard.pad.a",
            (fx, fy + FACE_STEP),
            BTN,
            Round,
        ),
        part(
            b(P::West),
            "input_map_keyboard.pad.x",
            (fx - wide(FACE_STEP), fy),
            BTN,
            Round,
        ),
        part(
            b(P::East),
            "input_map_keyboard.pad.b",
            (fx + wide(FACE_STEP), fy),
            BTN,
            Round,
        ),
    ]);
    let dk = "input_map_keyboard.pad.dpad";
    out.extend([
        part(b(P::DPadUp), dk, (px, py - ARM_STEP), ARM, Arm(I::ArrowUp)),
        part(
            b(P::DPadDown),
            dk,
            (px, py + ARM_STEP),
            ARM,
            Arm(I::ArrowDown),
        ),
        part(
            b(P::DPadLeft),
            dk,
            (px - wide(ARM_STEP), py),
            ARM,
            Arm(I::ArrowLeft),
        ),
        part(
            b(P::DPadRight),
            dk,
            (px + wide(ARM_STEP), py),
            ARM,
            Arm(I::ArrowRight),
        ),
    ]);
    for (i, ((cx, cy), key, r)) in STICKS.into_iter().enumerate() {
        let (x, y) = if i == 0 {
            (A::LeftStickX, A::LeftStickY)
        } else {
            (A::RightStickX, A::RightStickY)
        };
        let d = tip_reach(r);
        out.extend([
            part(half(y, true), key, (cx, cy - d), TIP, Half(I::ArrowUp, i)),
            part(
                half(y, false),
                key,
                (cx, cy + d),
                TIP,
                Half(I::ArrowDown, i),
            ),
            part(
                half(x, false),
                key,
                (cx - wide(d), cy),
                TIP,
                Half(I::ArrowLeft, i),
            ),
            part(
                half(x, true),
                key,
                (cx + wide(d), cy),
                TIP,
                Half(I::ArrowRight, i),
            ),
        ]);
    }
    out
}

/// O centro da cruz e o passo dos braços (fracções da caixa e da altura), para o desenho da cruz inteira.
pub(crate) const fn dpad() -> ((f32, f32), f32, (f32, f32)) {
    (DPAD, ARM_STEP, ARM)
}

/// O ícone da direcção de uma metade de manche — o ladrilho «LS ←» da B e o chip da A (uma porta só).
#[must_use]
pub fn half_icon(b: Binding) -> Option<ph2d_editor_core::icons::IconId> {
    use ph2d_editor_core::icons::IconId as I;
    let Binding::PadAxis { axis, positive } = b else {
        return None;
    };
    Some(match (axis, positive) {
        (GamepadAxis::LeftStickX | GamepadAxis::RightStickX, false) => I::ArrowLeft,
        (GamepadAxis::LeftStickX | GamepadAxis::RightStickX, true) => I::ArrowRight,
        (GamepadAxis::LeftStickY | GamepadAxis::RightStickY, true) => I::ArrowUp,
        (GamepadAxis::LeftStickY | GamepadAxis::RightStickY, false) => I::ArrowDown,
        _ => return None,
    })
}
